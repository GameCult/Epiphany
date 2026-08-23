use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::DateTime;
use chrono::Utc;
use serde_json::Value;
use sha2::Digest;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

mod heartbeat_documents;
mod heartbeat_pacing;
mod heartbeat_projection;
mod heartbeat_retention;
mod heartbeat_roles;
mod heartbeat_store;
pub use heartbeat_documents::*;
use heartbeat_pacing::adaptive_swarm_pacing;
use heartbeat_pacing::apply_initiative_heat_policy;
use heartbeat_pacing::effective_cooldown_multiplier;
use heartbeat_pacing::initiative_heat_multiplier;
use heartbeat_pacing::running_turn_count;
pub use heartbeat_projection::heartbeat_status_projection;
use heartbeat_projection::history_event_json;
use heartbeat_projection::pending_turn_json;
use heartbeat_projection::schedule_participant_json;
use heartbeat_projection::selection_policy_json;
pub use heartbeat_retention::retain_heartbeat_pulse_artifacts;
use heartbeat_roles::ROLE_ORDER;
use heartbeat_roles::agent_id_for_role;
pub use heartbeat_roles::default_heartbeat_state;
use heartbeat_roles::default_participant;
pub use heartbeat_roles::ghostlight_scene_heartbeat_state;
pub use heartbeat_roles::initialize_ghostlight_scene_heartbeat_store;
pub use heartbeat_roles::initialize_heartbeat_store;
pub use heartbeat_store::commit_heartbeat_state_transaction;
pub use heartbeat_store::heartbeat_state_cache;
pub use heartbeat_store::load_heartbeat_state_entry;
pub use heartbeat_store::load_heartbeat_state_transaction;
pub use heartbeat_store::load_latest_heartbeat_stale_turn_repair_receipt;
pub use heartbeat_store::validate_heartbeat_state;
pub use heartbeat_store::write_heartbeat_stale_turn_repair_receipt;
pub use heartbeat_store::write_heartbeat_state_entry;

pub(super) const HEARTBEAT_ARENA_MAINTENANCE: &str = "maintenance";
pub(super) const HEARTBEAT_ARENA_SCENE: &str = "scene";
pub(super) const PARTICIPANT_KIND_AGENT: &str = "agent";
pub(super) const PARTICIPANT_KIND_CHARACTER: &str = "character";

#[derive(Clone, Debug, PartialEq)]
struct HeartbeatAction {
    action_id: String,
    action_type: &'static str,
    action_scale: &'static str,
    base_recovery: f64,
    initiative_cost: f64,
    interruptibility: f64,
    commitment: f64,
    local_affordance_basis: Vec<String>,
}

pub fn tick_heartbeat_store(
    store_path: impl AsRef<Path>,
    artifact_dir: impl AsRef<Path>,
    options: HeartbeatTickOptions,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    if options.resident_self_store.as_deref() == Some(store_path) {
        return Err(anyhow!(
            "heartbeat and resident Self require physically separate CultCache stores"
        ));
    }
    let (loaded_state, expected_state) = load_heartbeat_state_transaction(store_path)?;
    let mut state =
        loaded_state.unwrap_or_else(|| default_heartbeat_state(options.target_heartbeat_rate));
    if options.target_heartbeat_rate > 0.0 {
        state.target_heartbeat_rate = options.target_heartbeat_rate;
    }
    patch_missing_participants(&mut state);
    apply_initiative_heat_policy(&mut state);
    if let Some(resident_store) = options.resident_self_store.as_deref()
        && crate::pending_resident_self_pressure(resident_store)?
        && let Some(pending) = state
            .participants
            .iter()
            .find(|participant| participant.role_id == "coordinator")
            .and_then(|participant| participant.pending_turn.as_ref())
    {
        crate::heartbeat_issue_resident_self_grant(
            resident_store,
            &pending.schedule_id,
            &pending.action_id,
            Utc::now().timestamp_millis().max(0) as u64,
        )?;
    }
    let mut resident_ack_to_consume = None;
    if let Some(resident_store) = options.resident_self_store.as_deref()
        && let Some(pending) = state
            .participants
            .iter()
            .find(|participant| participant.role_id == "coordinator")
            .and_then(|participant| participant.pending_turn.as_ref())
        && let Some(ack) = crate::pending_resident_self_ack_for(
            resident_store,
            &pending.schedule_id,
            &pending.action_id,
        )?
    {
        let index = participant_index_by_role(&state, "coordinator")?;
        complete_pending_turn(&mut state, index)?;
        resident_ack_to_consume = Some((resident_store.to_path_buf(), ack.ack_id));
    }
    let resident_pressure_pending =
        if let Some(resident_store) = options.resident_self_store.as_deref() {
            crate::pending_resident_self_pressure(resident_store)?
        } else {
            false
        };
    let mut effective_options = options.clone();
    if resident_pressure_pending {
        effective_options.coordinator_action = Some("resident-self-pressure".to_string());
        effective_options.target_role = Some("coordinator".to_string());
        effective_options.defer_completion = true;
    }
    let result = tick_once(&mut state, &effective_options, resident_pressure_pending)?;
    commit_heartbeat_state_transaction(store_path, expected_state, &state)?;
    if result["event"]["selectedRole"] == "coordinator"
        && effective_options.defer_completion
        && let Some(resident_store) = effective_options.resident_self_store.as_deref()
    {
        let schedule_id = result["event"]["schedule_id"]
            .as_str()
            .or_else(|| result["event"]["scheduleId"].as_str())
            .unwrap_or(&effective_options.schedule_id);
        let action_id = result["event"]["action_id"]
            .as_str()
            .or_else(|| result["event"]["actionId"].as_str())
            .unwrap_or_default();
        crate::heartbeat_issue_resident_self_grant(
            resident_store,
            schedule_id,
            action_id,
            Utc::now().timestamp_millis().max(0) as u64,
        )?;
    }
    if let Some((resident_store, ack_id)) = resident_ack_to_consume {
        crate::heartbeat_consume_resident_self_ack(
            &resident_store,
            &ack_id,
            Utc::now().timestamp_millis().max(0) as u64,
        )?;
    }

    let artifact_dir = artifact_dir.as_ref();
    fs::create_dir_all(artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    write_json_artifact(
        artifact_dir.join(format!("{}.initiative.json", options.schedule_id)),
        &result["schedule"],
    )?;
    write_json_artifact(
        artifact_dir.join(format!("{}.event.json", options.schedule_id)),
        &result["event"],
    )?;

    Ok(serde_json::json!({
        "ok": true,
        "storeFile": store_path,
        "artifactDir": artifact_dir,
        "stateFile": null,
        "event": result["event"].clone(),
        "schedule": result["schedule"].clone(),
    }))
}

pub fn pulse_persona_heartbeat(
    store_path: impl AsRef<Path>,
    artifact_dir: impl AsRef<Path>,
    schedule_id: &str,
    source_scene_ref: &str,
    brake_engaged: bool,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    let artifact_dir = artifact_dir.as_ref();
    if brake_engaged {
        return Ok(serde_json::json!({
            "schemaVersion": "epiphany.persona_heartbeat_pulse.v0",
            "status": "refused-by-swarm-brake",
            "privateStateExposed": false,
        }));
    }
    let state = load_heartbeat_state_entry(store_path)?
        .ok_or_else(|| anyhow!("heartbeat state is missing"))?;
    let has_mentions = state
        .pending_mentions
        .iter()
        .any(|mention| mention.target_role_id == "Persona");
    if !has_mentions {
        return Ok(serde_json::json!({
            "schemaVersion": "epiphany.persona_heartbeat_pulse.v0",
            "status": "idle",
            "privateStateExposed": false,
        }));
    }
    let persona_running = state
        .participants
        .iter()
        .find(|participant| participant.role_id == "Persona")
        .and_then(|participant| participant.pending_turn.as_ref())
        .is_some();
    let request_running = state
        .persona_turn_requests
        .iter()
        .any(|request| request.status == "reserved");
    if persona_running || request_running {
        return Ok(serde_json::json!({
            "schemaVersion": "epiphany.persona_heartbeat_pulse.v0",
            "status": "already-running",
            "privateStateExposed": false,
        }));
    }
    let tick = tick_heartbeat_store(
        store_path,
        artifact_dir,
        HeartbeatTickOptions {
            target_heartbeat_rate: state.target_heartbeat_rate,
            coordinator_action: Some("admitted-persona-feedback".into()),
            target_role: Some("Persona".into()),
            urgency: 1.0,
            schedule_id: schedule_id.to_string(),
            source_scene_ref: source_scene_ref.to_string(),
            defer_completion: true,
            resident_self_store: None,
        },
    )?;
    let request_id = load_heartbeat_state_entry(store_path)?
        .and_then(|state| {
            state
                .persona_turn_requests
                .into_iter()
                .find(|request| request.schedule_id == schedule_id)
                .map(|request| request.request_id)
        })
        .ok_or_else(|| anyhow!("Persona heartbeat pulse did not persist its turn request"))?;
    Ok(serde_json::json!({
        "schemaVersion": "epiphany.persona_heartbeat_pulse.v0",
        "status": "reserved",
        "requestId": request_id,
        "schedule": tick["schedule"],
        "privateStateExposed": false,
    }))
}

pub fn reconcile_resident_self_heartbeat_ack(
    heartbeat_store: impl AsRef<Path>,
    resident_store: impl AsRef<Path>,
) -> Result<Option<String>> {
    let heartbeat_store = heartbeat_store.as_ref();
    let resident_store = resident_store.as_ref();
    if heartbeat_store == resident_store {
        return Err(anyhow!("heartbeat and resident Self stores must differ"));
    }
    let (state, expected) = load_heartbeat_state_transaction(heartbeat_store)?;
    let Some(mut state) = state else {
        return Ok(None);
    };
    let Some((index, pending)) = state
        .participants
        .iter()
        .enumerate()
        .find(|(_, participant)| participant.role_id == "coordinator")
        .and_then(|(index, participant)| {
            participant
                .pending_turn
                .clone()
                .map(|pending| (index, pending))
        })
    else {
        // Recovery for a crash after heartbeat completion committed but before the
        // acknowledgement was marked consumed in the separate resident store.
        let completed = crate::pending_resident_self_acks(resident_store)?
            .into_iter()
            .find(|ack| {
                state.history.iter().any(|event| {
                    event.selected_role == "coordinator"
                        && event.schedule_id == ack.heartbeat_schedule_id
                        && event.action_id == ack.heartbeat_action_id
                        && event.turn_status.as_deref() == Some("completed")
                })
            });
        if let Some(ack) = completed {
            crate::heartbeat_consume_resident_self_ack(
                resident_store,
                &ack.ack_id,
                Utc::now().timestamp_millis().max(0) as u64,
            )?;
            return Ok(Some(ack.ack_id));
        }
        return Ok(None);
    };
    let Some(ack) = crate::pending_resident_self_ack_for(
        resident_store,
        &pending.schedule_id,
        &pending.action_id,
    )?
    else {
        return Ok(None);
    };
    let completed = complete_pending_turn(&mut state, index)?;
    let participant = &state.participants[index];
    state.history.push(HeartbeatHistoryEvent {
        ts: now_iso(),
        schedule_id: completed.schedule_id.clone(),
        selected_role: participant.role_id.clone(),
        selected_agent_id: participant.agent_id.clone(),
        action_id: completed.action_id.clone(),
        action_type: completed.action_type.clone(),
        arena: participant_arena(participant).to_string(),
        participant_kind: participant_kind(participant).to_string(),
        action_scale: completed.action_scale.clone(),
        coordinator_action: None,
        target_role: None,
        work_role: None,
        scene_clock: Some(state.scene_clock),
        next_ready_at: Some(participant.next_ready_at),
        turn_status: Some("completed".into()),
        cooldown_started_after_completion: Some(true),
    });
    trim_history(&mut state);
    commit_heartbeat_state_transaction(heartbeat_store, expected, &state)?;
    crate::heartbeat_consume_resident_self_ack(
        resident_store,
        &ack.ack_id,
        Utc::now().timestamp_millis().max(0) as u64,
    )?;
    Ok(Some(ack.ack_id))
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResidentSelfHeartbeatPulse {
    pub status: String,
    pub acknowledged_terminal_id: Option<String>,
    pub grant_id: Option<String>,
}

pub fn pulse_resident_self_heartbeat(
    heartbeat_store: impl AsRef<Path>,
    resident_store: impl AsRef<Path>,
    artifact_dir: impl AsRef<Path>,
    brake_engaged: bool,
    schedule_id: &str,
    source_scene_ref: &str,
) -> Result<ResidentSelfHeartbeatPulse> {
    let heartbeat_store = heartbeat_store.as_ref();
    let resident_store = resident_store.as_ref();
    let acknowledged_terminal_id =
        reconcile_resident_self_heartbeat_ack(heartbeat_store, resident_store)?;
    if brake_engaged {
        return Ok(ResidentSelfHeartbeatPulse {
            status: "braked-after-ack-reconciliation".into(),
            acknowledged_terminal_id,
            grant_id: None,
        });
    }
    let state = load_heartbeat_state_entry(heartbeat_store)?;
    let active = state
        .as_ref()
        .and_then(|state| {
            state
                .participants
                .iter()
                .find(|participant| participant.role_id == "coordinator")
        })
        .and_then(|participant| participant.pending_turn.as_ref())
        .filter(|turn| turn.status == "running")
        .cloned();
    if let Some(active) = active {
        if crate::pending_resident_self_pressure(resident_store)? {
            let grant = crate::heartbeat_issue_resident_self_grant(
                resident_store,
                &active.schedule_id,
                &active.action_id,
                Utc::now().timestamp_millis().max(0) as u64,
            )?;
            return Ok(ResidentSelfHeartbeatPulse {
                status: "recovered-committed-grant".into(),
                acknowledged_terminal_id,
                grant_id: grant.map(|grant| grant.grant_id),
            });
        }
        return Ok(ResidentSelfHeartbeatPulse {
            status: "active-coordinator-turn".into(),
            acknowledged_terminal_id,
            grant_id: None,
        });
    }
    if !crate::pending_resident_self_pressure(resident_store)? {
        return Ok(ResidentSelfHeartbeatPulse {
            status: "idle".into(),
            acknowledged_terminal_id,
            grant_id: None,
        });
    }
    tick_heartbeat_store(
        heartbeat_store,
        artifact_dir,
        HeartbeatTickOptions {
            target_heartbeat_rate: 1.0,
            coordinator_action: None,
            target_role: None,
            urgency: 0.0,
            schedule_id: schedule_id.into(),
            source_scene_ref: source_scene_ref.into(),
            defer_completion: true,
            resident_self_store: Some(resident_store.to_path_buf()),
        },
    )?;
    let grant = crate::pending_resident_self_grant(resident_store)?
        .ok_or_else(|| anyhow!("heartbeat selected resident Self pressure but emitted no grant"))?;
    Ok(ResidentSelfHeartbeatPulse {
        status: "granted".into(),
        acknowledged_terminal_id,
        grant_id: Some(grant.grant_id),
    })
}

pub fn pump_heartbeat_store(
    store_path: impl AsRef<Path>,
    artifact_dir: impl AsRef<Path>,
    options: HeartbeatPumpOptions,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    if let Some(resident_store) = options.resident_self_store.as_deref() {
        let heartbeat = load_heartbeat_state_entry(store_path)?;
        let pending_ack = if let Some(pending) = heartbeat
            .as_ref()
            .and_then(|state| {
                state
                    .participants
                    .iter()
                    .find(|p| p.role_id == "coordinator")
            })
            .and_then(|p| p.pending_turn.as_ref())
        {
            crate::pending_resident_self_ack_for(
                resident_store,
                &pending.schedule_id,
                &pending.action_id,
            )?
            .is_some()
        } else {
            false
        };
        if crate::pending_resident_self_pressure(resident_store)? || pending_ack {
            let tick = tick_heartbeat_store(
                store_path,
                artifact_dir.as_ref(),
                HeartbeatTickOptions {
                    target_heartbeat_rate: options.base_heartbeat_rate.max(0.001),
                    coordinator_action: None,
                    target_role: None,
                    urgency: options.external_urgency,
                    schedule_id: format!("{}.resident-self", options.schedule_id),
                    source_scene_ref: options.source_scene_ref.clone(),
                    defer_completion: true,
                    resident_self_store: Some(resident_store.to_path_buf()),
                },
            )?;
            return Ok(serde_json::json!({
                "schema_version": "epiphany.adaptive_heartbeat_pump.v0", "storeFile": store_path,
                "artifactDir": artifact_dir.as_ref(), "sourceSceneRef": options.source_scene_ref,
                "scheduleId": options.schedule_id, "launched": 1, "ticks": [tick],
                "residentSelfDelegatedToTransactionalTick": true, "errors": []
            }));
        }
    }
    let mut state = load_heartbeat_state_entry(store_path)?
        .unwrap_or_else(|| default_heartbeat_state(options.base_heartbeat_rate.max(0.001)));
    if options.base_heartbeat_rate > 0.0 {
        state.target_heartbeat_rate = options.base_heartbeat_rate;
    }
    patch_missing_participants(&mut state);
    apply_initiative_heat_policy(&mut state);

    let pacing = adaptive_swarm_pacing(&state, &options);
    state.target_heartbeat_rate = pacing.effective_heartbeat_rate;
    state.adaptive_pacing = Some(HeartbeatAdaptivePacing {
        schema_version: "epiphany.adaptive_heartbeat_pacing.v0".to_string(),
        contract: "Swarm pressure controls both heartbeat tempo and concurrency. Relaxed systems sleep slow; urgent systems fill more lanes without re-waking unfinished turns.".to_string(),
        pressure: pacing.pressure,
        effective_heartbeat_rate: pacing.effective_heartbeat_rate,
        target_concurrency: pacing.target_concurrency,
        running_turns: pacing.running_turns,
        active_participants: pacing.active_participants,
        signals: pacing.signals.clone(),
    });

    let artifact_dir = artifact_dir.as_ref();
    fs::create_dir_all(artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;

    let mut tick_results = Vec::new();
    let mut errors = Vec::new();
    let ticks_allowed = options.max_ticks.min(pacing.target_concurrency);
    for index in 0..ticks_allowed {
        if running_turn_count(&state) >= pacing.target_concurrency {
            break;
        }
        let use_coordinator_action = index == 0;
        let tick_options = HeartbeatTickOptions {
            target_heartbeat_rate: pacing.effective_heartbeat_rate,
            coordinator_action: use_coordinator_action
                .then(|| options.coordinator_action.clone())
                .flatten(),
            target_role: use_coordinator_action
                .then(|| options.target_role.clone())
                .flatten(),
            urgency: pacing
                .pressure
                .max(options.external_urgency.clamp(0.0, 1.0)),
            schedule_id: format!("{}.pump-{:03}", options.schedule_id, index + 1),
            source_scene_ref: options.source_scene_ref.clone(),
            defer_completion: true,
            resident_self_store: options.resident_self_store.clone(),
        };
        match tick_once(&mut state, &tick_options, false) {
            Ok(result) => {
                write_json_artifact(
                    artifact_dir.join(format!("{}.initiative.json", tick_options.schedule_id)),
                    &result["schedule"],
                )?;
                write_json_artifact(
                    artifact_dir.join(format!("{}.event.json", tick_options.schedule_id)),
                    &result["event"],
                )?;
                tick_results.push(serde_json::json!({
                    "event": result["event"],
                    "schedule": result["schedule"],
                }));
            }
            Err(error) => {
                errors.push(error.to_string());
                break;
            }
        }
    }
    let launched = tick_results.len();
    let final_running_turns = running_turn_count(&state);
    write_heartbeat_state_entry(store_path, &state)?;

    let pump = serde_json::json!({
        "schema_version": "epiphany.adaptive_heartbeat_pump.v0",
        "storeFile": store_path,
        "artifactDir": artifact_dir,
        "sourceSceneRef": options.source_scene_ref,
        "scheduleId": options.schedule_id,
        "pacing": {
            "schema_version": "epiphany.adaptive_heartbeat_pacing.v0",
            "pressure": pacing.pressure,
            "effectiveHeartbeatRate": pacing.effective_heartbeat_rate,
            "targetConcurrency": pacing.target_concurrency,
            "runningTurnsBefore": pacing.running_turns,
            "runningTurnsAfter": final_running_turns,
            "activeParticipants": pacing.active_participants,
            "signals": pacing.signals,
        },
        "launched": launched,
        "ticks": tick_results,
        "errors": errors,
        "reviewNotes": [
            "The pump controls opportunity pressure, not authority.",
            "A relaxed swarm may launch nothing; an urgent swarm may fill most available lanes.",
            "Per-lane pending turns remain hard locks, so no agent is re-heartbeaten while its previous turn is running."
        ],
    });
    write_json_artifact(
        artifact_dir.join(format!("{}.pump.json", options.schedule_id)),
        &pump,
    )?;

    Ok(serde_json::json!({
        "ok": errors.is_empty(),
        "pump": pump,
    }))
}

pub fn update_heartbeat_heat_store(
    store_path: impl AsRef<Path>,
    options: HeartbeatHeatUpdateOptions,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    let mut state = load_heartbeat_state_entry(store_path)?.unwrap_or_else(|| {
        let mut state = default_heartbeat_state(1.0);
        patch_missing_participants(&mut state);
        state
    });
    patch_missing_participants(&mut state);
    let scope = options.scope.trim().to_lowercase();
    let selector = options.selector.trim().to_string();
    let id = options.id.clone().unwrap_or_else(|| {
        if scope == "global" {
            "global".to_string()
        } else {
            format!("{scope}:{selector}")
        }
    });

    if options.clear {
        if scope == "global" || id == "global" {
            state.initiative_heat.global_multiplier = 1.0;
        }
        state
            .initiative_heat
            .multipliers
            .retain(|multiplier| multiplier.id != id);
    } else if scope == "global" {
        state.initiative_heat.global_multiplier = options.multiplier.clamp(0.05, 25.0);
    } else {
        if selector.is_empty() && scope != "all" {
            return Err(anyhow!(
                "heartbeat heat selector is required for scope {scope}"
            ));
        }
        let multiplier = HeartbeatInitiativeMultiplier {
            id: id.clone(),
            label: options.label.clone().unwrap_or_default(),
            scope: scope.clone(),
            selector: selector.clone(),
            multiplier: options.multiplier.clamp(0.05, 25.0),
            reason: options.reason.clone().unwrap_or_default(),
            updated_at: Some(now_iso()),
            expires_at_scene_clock: options
                .expires_after_scene_clock
                .map(|delta| round6(state.scene_clock + delta.max(0.0))),
        };
        state
            .initiative_heat
            .multipliers
            .retain(|existing| existing.id != id);
        state.initiative_heat.multipliers.push(multiplier);
        state
            .initiative_heat
            .multipliers
            .sort_by(|left, right| left.id.cmp(&right.id));
    }
    apply_initiative_heat_policy(&mut state);
    write_heartbeat_state_entry(store_path, &state)?;
    Ok(serde_json::json!({
        "ok": true,
        "command": "heat",
        "storeFile": store_path,
        "heat": initiative_heat_json(&state),
        "participants": state.participants.iter().map(schedule_participant_json).collect::<Vec<_>>(),
    }))
}

pub fn queue_heartbeat_pending_mention_store(
    store_path: impl AsRef<Path>,
    options: HeartbeatQueueMentionOptions,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    let mut state = load_heartbeat_state_entry(store_path)?.unwrap_or_else(|| {
        let mut state = default_heartbeat_state(1.0);
        patch_missing_participants(&mut state);
        state
    });
    patch_missing_participants(&mut state);
    let participant_index = participant_index_by_role(&state, &options.target_role_id)?;
    let participant = &state.participants[participant_index];
    validate_mention_text("content", &options.content, 4, 1200)?;
    validate_mention_text("visible_prompt", &options.visible_prompt, 4, 1200)?;
    for (label, value) in [
        ("source_surface", &options.source_surface),
        ("channel_id", &options.channel_id),
        ("message_id", &options.message_id),
        ("author_id", &options.author_id),
        ("source_visibility", &options.source_visibility),
        ("data_classification", &options.data_classification),
        ("model_provider_id", &options.model_provider_id),
    ] {
        validate_mention_text(label, value, 1, 240)?;
    }
    if !options.model_provider_disclosure_allowed {
        return Err(anyhow!(
            "pending mention is not authorized for disclosure to the configured model provider"
        ));
    }
    let queued_at = options.queued_at.clone().unwrap_or_else(now_iso);
    let mention_id = options.mention_id.clone().unwrap_or_else(|| {
        stable_pending_mention_id(
            &options.target_role_id,
            &options.channel_id,
            &options.message_id,
            &options.visible_prompt,
        )
    });
    if state
        .pending_mentions
        .iter()
        .any(|mention| mention.id == mention_id)
    {
        return Ok(serde_json::json!({
            "ok": true,
            "queued": false,
            "reason": "duplicate-pending-mention",
            "mentionId": mention_id,
            "pendingMentionCount": state.pending_mentions.len(),
        }));
    }
    state.pending_mentions.push(HeartbeatPendingMention {
        id: mention_id.clone(),
        target_role_id: options.target_role_id.clone(),
        target_agent_id: participant.agent_id.clone(),
        source_surface: options.source_surface,
        channel_id: options.channel_id,
        message_id: options.message_id,
        author_id: options.author_id,
        author_name: options.author_name,
        content: options.content,
        visible_prompt: options.visible_prompt,
        reply_to_message_id: options.reply_to_message_id,
        queued_at,
        source_visibility: options.source_visibility,
        data_classification: options.data_classification,
        model_provider_id: options.model_provider_id,
        model_provider_disclosure_allowed: options.model_provider_disclosure_allowed,
    });
    state.pending_mentions.sort_by(|left, right| {
        left.queued_at
            .cmp(&right.queued_at)
            .then_with(|| left.id.cmp(&right.id))
    });
    state.participants[participant_index].next_ready_at = state.participants[participant_index]
        .next_ready_at
        .min(state.scene_clock);
    write_heartbeat_state_entry(store_path, &state)?;
    Ok(serde_json::json!({
        "ok": true,
        "queued": true,
        "mentionId": mention_id,
        "targetRoleId": options.target_role_id,
        "pendingMentionCount": state.pending_mentions.len(),
        "contract": "Pending Persona mentions live in heartbeat physiology. They pull the Persona turn forward, but the Persona still writes naturally and the Interpreter owns side effects.",
    }))
}

fn initiative_heat_json(state: &EpiphanyHeartbeatStateEntry) -> Value {
    serde_json::json!({
        "schemaVersion": state.initiative_heat.schema_version,
        "globalMultiplier": state.initiative_heat.global_multiplier,
        "multipliers": state.initiative_heat.multipliers.iter().map(|multiplier| {
            serde_json::json!({
                "id": multiplier.id,
                "label": multiplier.label,
                "scope": multiplier.scope,
                "selector": multiplier.selector,
                "multiplier": multiplier.multiplier,
                "reason": multiplier.reason,
                "updatedAt": multiplier.updated_at,
                "expiresAtSceneClock": multiplier.expires_at_scene_clock,
            })
        }).collect::<Vec<_>>(),
    })
}

pub fn complete_heartbeat_store(
    store_path: impl AsRef<Path>,
    artifact_dir: impl AsRef<Path>,
    options: HeartbeatCompleteOptions,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    let mut state = load_heartbeat_state_entry(store_path)?.ok_or_else(|| {
        anyhow!(
            "CultCache store {} has no heartbeat state entry",
            store_path.display()
        )
    })?;
    let participant_index = participant_index_by_role(&state, &options.role)?;
    let pending = state.participants[participant_index]
        .pending_turn
        .clone()
        .ok_or_else(|| anyhow!("{} has no running heartbeat turn", options.role))?;
    if pending.status != "running" {
        return Err(anyhow!("{} has no running heartbeat turn", options.role));
    }
    if let Some(action_id) = &options.action_id
        && pending.action_id != *action_id
    {
        return Err(anyhow!(
            "{} pending heartbeat action is {}, not {}",
            options.role,
            pending.action_id,
            action_id
        ));
    }
    let completed = complete_pending_turn(&mut state, participant_index)?;
    let participant = &state.participants[participant_index];
    let event = HeartbeatHistoryEvent {
        ts: now_iso(),
        schedule_id: completed.schedule_id.clone(),
        selected_role: options.role,
        selected_agent_id: participant.agent_id.clone(),
        action_id: completed.action_id.clone(),
        action_type: completed.action_type.clone(),
        arena: participant_arena(participant).to_string(),
        participant_kind: participant_kind(participant).to_string(),
        action_scale: completed.action_scale.clone(),
        coordinator_action: None,
        target_role: None,
        work_role: None,
        scene_clock: Some(state.scene_clock),
        next_ready_at: Some(participant.next_ready_at),
        turn_status: Some("completed".to_string()),
        cooldown_started_after_completion: Some(true),
    };
    state.history.push(event.clone());
    trim_history(&mut state);
    write_heartbeat_state_entry(store_path, &state)?;

    let artifact_dir = artifact_dir.as_ref();
    fs::create_dir_all(artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    if !completed.schedule_id.is_empty() {
        write_json_artifact(
            artifact_dir.join(format!("{}.completion.json", completed.schedule_id)),
            &serde_json::json!({"event": history_event_json(event.clone()), "turn": pending_turn_json(&completed)}),
        )?;
    }

    Ok(serde_json::json!({
        "ok": true,
        "storeFile": store_path,
        "event": history_event_json(event),
        "completedTurn": pending_turn_json(&completed),
    }))
}

pub fn recover_stale_heartbeat_store(
    store_path: impl AsRef<Path>,
    artifact_dir: impl AsRef<Path>,
    options: HeartbeatStaleTurnRepairOptions,
) -> Result<Value> {
    if options.max_age_seconds < 0 {
        return Err(anyhow!(
            "stale heartbeat repair max_age_seconds must be non-negative"
        ));
    }
    if options.reason.trim().is_empty() {
        return Err(anyhow!("stale heartbeat repair requires a reason"));
    }
    let store_path = store_path.as_ref();
    let mut state = load_heartbeat_state_entry(store_path)?.ok_or_else(|| {
        anyhow!(
            "CultCache store {} has no heartbeat state entry",
            store_path.display()
        )
    })?;
    let repaired_at = options.now_utc.clone().unwrap_or_else(now_iso);
    let repaired_at_time = parse_heartbeat_time("repair time", &repaired_at)?;
    let mut receipts = Vec::new();

    for participant in &mut state.participants {
        let pending = match participant.pending_turn.clone() {
            Some(pending) if pending.status == "running" => pending,
            _ => continue,
        };
        let started_at = parse_heartbeat_time("pending turn start time", &pending.started_at)?;
        let stale_age_seconds = repaired_at_time
            .signed_duration_since(started_at)
            .num_seconds();
        if stale_age_seconds < options.max_age_seconds {
            continue;
        }
        if pending.action_type == "persona_turn"
            && let Some(request) = state.persona_turn_requests.iter_mut().find(|request| {
                request.status == "reserved"
                    && request.schedule_id == pending.schedule_id
                    && request.action_id == pending.action_id
            })
        {
            let mention_ids = request
                .mentions
                .iter()
                .map(|mention| mention.id.clone())
                .collect();
            let mention_cargo_sha256 = format!(
                "sha256-{:x}",
                sha2::Sha256::digest(rmp_serde::to_vec(&request.mentions)?)
            );
            request.status = "terminal".to_string();
            request.terminal_receipt = Some(PersonaTurnTerminalReceipt {
                schema_version: PERSONA_TURN_TERMINAL_RECEIPT_SCHEMA_VERSION.to_string(),
                receipt_id: format!("{}:terminal", request.request_id),
                request_id: request.request_id.clone(),
                schedule_id: request.schedule_id.clone(),
                action_id: request.action_id.clone(),
                outcome: "failed".to_string(),
                mention_disposition: "retained".to_string(),
                mention_ids,
                mention_cargo_sha256,
                delivery_evidence_id: None,
                crossing_receipt_id: None,
                bridge_receipt_sha256: None,
                blocked_crossing_status: None,
                blocked_reason: None,
                completed_at: repaired_at.clone(),
                private_state_exposed: false,
            });
            request.mentions.clear();
            request.semantic_memory_recall = Value::Null;
        }
        participant.pending_turn = None;
        participant.current_load = 0.0;
        participant.next_ready_at = round6(state.scene_clock.max(pending.started_scene_clock));
        let receipt = EpiphanyHeartbeatStaleTurnRepairReceipt {
            schema_version: HEARTBEAT_STALE_TURN_REPAIR_SCHEMA_VERSION.to_string(),
            receipt_id: format!(
                "heartbeat-stale-turn-repair-{}-{}",
                participant.role_id,
                now_stamp()
            ),
            repaired_at_utc: repaired_at.clone(),
            role_id: participant.role_id.clone(),
            agent_id: participant.agent_id.clone(),
            action_id: pending.action_id.clone(),
            schedule_id: pending.schedule_id.clone(),
            started_at_utc: pending.started_at.clone(),
            stale_age_seconds,
            reason: options.reason.clone(),
            resulting_status: "repaired".to_string(),
            next_ready_at: participant.next_ready_at,
            private_state_exposed: false,
            notes: vec![
                "Stale heartbeat repair is a Continuity-facing operator-safe receipt, not silent scheduler cleanup.".to_string(),
                "The repaired lane becomes schedulable only through normal heartbeat selection after the receipt is written.".to_string(),
            ],
        };
        let event = HeartbeatHistoryEvent {
            ts: repaired_at.clone(),
            schedule_id: pending.schedule_id.clone(),
            selected_role: participant.role_id.clone(),
            selected_agent_id: participant.agent_id.clone(),
            action_id: pending.action_id.clone(),
            action_type: pending.action_type.clone(),
            arena: participant_arena(participant).to_string(),
            participant_kind: participant_kind(participant).to_string(),
            action_scale: pending.action_scale.clone(),
            coordinator_action: None,
            target_role: None,
            work_role: None,
            scene_clock: Some(state.scene_clock),
            next_ready_at: Some(participant.next_ready_at),
            turn_status: Some("stale_repaired".to_string()),
            cooldown_started_after_completion: Some(false),
        };
        state.history.push(event);
        receipts.push(receipt);
    }

    if receipts.is_empty() {
        return Ok(serde_json::json!({
            "ok": true,
            "storeFile": store_path,
            "repaired": 0,
            "receipts": [],
            "reviewNotes": [
                "No running heartbeat turns exceeded the stale repair threshold."
            ],
        }));
    }

    trim_history(&mut state);
    write_heartbeat_state_entry(store_path, &state)?;
    let mut written_receipts = Vec::new();
    for receipt in receipts {
        written_receipts.push(write_heartbeat_stale_turn_repair_receipt(
            store_path, &receipt,
        )?);
    }

    let artifact_dir = artifact_dir.as_ref();
    fs::create_dir_all(artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    write_json_artifact(
        artifact_dir.join(format!("heartbeat-stale-repair-{}.json", now_stamp())),
        &serde_json::json!({
            "schemaVersion": "epiphany.heartbeat.stale_repair_artifact.v0",
            "storeFile": store_path,
            "repairedAtUtc": repaired_at,
            "maxAgeSeconds": options.max_age_seconds,
            "receipts": written_receipts,
        }),
    )?;

    Ok(serde_json::json!({
        "ok": true,
        "storeFile": store_path,
        "repaired": written_receipts.len(),
        "receipts": written_receipts,
    }))
}

fn tick_once(
    state: &mut EpiphanyHeartbeatStateEntry,
    options: &HeartbeatTickOptions,
    force_work_role: bool,
) -> Result<Value> {
    let rate = state.target_heartbeat_rate.max(0.001);
    let work_role = work_role_for_action(
        options.coordinator_action.as_deref(),
        options.target_role.as_deref(),
    );
    let (selected_index, selection_kind, selection_reason) = select_participant(
        state,
        work_role.as_deref(),
        options.urgency,
        force_work_role,
    )?;
    let selected = state.participants[selected_index].clone();
    let action = action_for_selection(
        state,
        &selected,
        work_role.as_deref(),
        options.coordinator_action.as_deref(),
        rate,
    );
    let scene_clock = state.scene_clock.max(selected.next_ready_at);
    let recovery = action.base_recovery * effective_cooldown_multiplier(&selected)
        / selected
            .initiative_speed
            .max(state.selection_policy.minimum_speed);
    let pending = HeartbeatPendingTurn {
        status: "running".to_string(),
        schedule_id: options.schedule_id.clone(),
        action_id: action.action_id.clone(),
        action_type: action.action_type.to_string(),
        action_scale: action.action_scale.to_string(),
        local_affordance_basis: action.local_affordance_basis.clone(),
        started_at: now_iso(),
        started_scene_clock: round6(scene_clock),
        base_recovery: round6(action.base_recovery),
        recovery: round6(recovery),
        cooldown_policy: "after_turn_completion".to_string(),
        completed_at: None,
        completed_scene_clock: None,
        next_ready_at: None,
        initiative_heat_multiplier: initiative_heat_multiplier(&selected),
        effective_cooldown_multiplier: effective_cooldown_multiplier(&selected),
        initiative_frozen: true,
        initiative_freeze_reason: Some(
            "Participant has an active heartbeat turn; initiative cannot queue it again until the turn completes."
                .to_string(),
        ),
    };
    state.participants[selected_index].pending_turn = Some(pending.clone());
    state.participants[selected_index].last_action_id = Some(action.action_id.clone());
    state.participants[selected_index].last_woke_at = Some(now_iso());
    state.participants[selected_index].current_load =
        round6((state.participants[selected_index].current_load * 0.75).clamp(0.0, 1.0));
    state.scene_clock = round6(scene_clock);
    let selected_pending_mentions = pending_mentions_for_role(state, &selected.role_id);
    if action.action_type == "persona_turn" {
        reserve_persona_turn_request(
            state,
            &pending,
            &selected_after_identity(&selected),
            selected_pending_mentions.clone(),
            Value::Null,
        )?;
    }
    if !options.defer_completion && action.action_type != "persona_turn" {
        complete_pending_turn(state, selected_index)?;
    }

    let selected_after = state.participants[selected_index].clone();
    let event = HeartbeatHistoryEvent {
        ts: now_iso(),
        schedule_id: options.schedule_id.clone(),
        selected_role: selected_after.role_id.clone(),
        selected_agent_id: selected_after.agent_id.clone(),
        action_id: action.action_id.clone(),
        action_type: action.action_type.to_string(),
        arena: participant_arena(&selected_after).to_string(),
        participant_kind: participant_kind(&selected_after).to_string(),
        action_scale: action.action_scale.to_string(),
        coordinator_action: options.coordinator_action.clone(),
        target_role: options.target_role.clone(),
        work_role: work_role.clone(),
        scene_clock: Some(state.scene_clock),
        next_ready_at: Some(selected_after.next_ready_at),
        turn_status: Some(if options.defer_completion {
            "running".to_string()
        } else {
            "completed".to_string()
        }),
        cooldown_started_after_completion: Some(true),
    };
    state.history.push(event.clone());
    trim_history(state);

    let readiness_snapshot = readiness_snapshot(state, work_role.as_deref(), options.urgency);
    let schedule = serde_json::json!({
        "schema_version": INITIATIVE_SCHEMA_VERSION,
        "schedule_id": options.schedule_id,
        "source_scene_ref": options.source_scene_ref,
        "scene_clock": state.scene_clock,
        "participants": state.participants.iter().map(schedule_participant_json).collect::<Vec<_>>(),
        "action_catalog": [{
            "action_id": action.action_id,
            "actor_id": selected_after.agent_id,
            "arena": participant_arena(&selected_after),
            "participant_kind": participant_kind(&selected_after),
            "action_type": action.action_type,
            "action_scale": action.action_scale,
            "base_recovery": action.base_recovery,
            "initiative_heat_multiplier": initiative_heat_multiplier(&selected_after),
            "effective_cooldown_multiplier": effective_cooldown_multiplier(&selected_after),
            "initiative_cost": action.initiative_cost,
            "interruptibility": action.interruptibility,
            "commitment": action.commitment,
            "local_affordance_basis": action.local_affordance_basis,
            "pending_mentions": selected_pending_mentions,
        }],
        "reaction_windows": if let Some(work_role) = &work_role {
            serde_json::json!([{
                "window_id": format!("{}.pending-work", options.schedule_id),
                "trigger_event_ref": options.source_scene_ref,
                "urgency": options.urgency,
                "eligible_actor_ids": [agent_id_for_work_role(state, work_role)],
                "allowed_action_scales": ["short", "standard"],
                "expires_at": round6(state.scene_clock + 1.0),
                "notes": "Pending coordinator work can pull its owning lane forward only if readiness clears threshold."
            }])
        } else {
            serde_json::json!([])
        },
        "selection_policy": selection_policy_json(&state.selection_policy),
        "next_actor_selection": {
            "selection_kind": selection_kind,
            "selected_actor_id": selected_after.agent_id,
            "selected_action_ids": [event.action_id.clone()],
            "scene_clock_after_selection": state.scene_clock,
            "selection_reason": selection_reason,
            "override_reason": null,
            "readiness_snapshot": readiness_snapshot,
        },
        "pending_mentions": state.pending_mentions.clone(),
        "review_notes": [
            "Epiphany heartbeat uses Ghostlight initiative timing as a harness scheduling receipt.",
            "A selected idle lane records scheduling opportunity; it does not invent project work or mutate durable memory.",
            "When no coordinator work is active, cooldown and sleep pacing keep the swarm from thrashing."
        ],
    });
    Ok(serde_json::json!({
        "event": history_event_json(event),
        "schedule": schedule,
    }))
}

fn selected_after_identity(selected: &HeartbeatParticipant) -> (String, String) {
    (selected.role_id.clone(), selected.agent_id.clone())
}

fn reserve_persona_turn_request(
    state: &mut EpiphanyHeartbeatStateEntry,
    pending: &HeartbeatPendingTurn,
    identity: &(String, String),
    mentions: Vec<HeartbeatPendingMention>,
    semantic_memory_recall: Value,
) -> Result<()> {
    let request_id = format!("persona-turn:{}:{}", pending.schedule_id, pending.action_id);
    let request = PersonaTurnRequest {
        schema_version: PERSONA_TURN_REQUEST_SCHEMA_VERSION.to_string(),
        request_id: request_id.clone(),
        schedule_id: pending.schedule_id.clone(),
        action_id: pending.action_id.clone(),
        role_id: identity.0.clone(),
        agent_id: identity.1.clone(),
        status: "reserved".to_string(),
        reserved_at: pending.started_at.clone(),
        mentions,
        semantic_memory_recall,
        terminal_receipt: None,
        private_state_exposed: false,
    };
    if let Some(head) = &state.persona_conversation_retention_head {
        let frontier = chrono::DateTime::parse_from_rfc3339(&head.through_reserved_at)
            .map_err(|_| anyhow!("Persona conversation retention frontier is invalid"))?;
        let reserved = chrono::DateTime::parse_from_rfc3339(&request.reserved_at)
            .map_err(|_| anyhow!("Persona turn reservation time is invalid"))?;
        if reserved <= frontier {
            return Err(anyhow!(
                "Persona turn reservation is at or behind the retired replay frontier"
            ));
        }
    }
    if let Some(existing) = state
        .persona_turn_requests
        .iter()
        .find(|existing| existing.request_id == request_id)
    {
        if existing != &request {
            return Err(anyhow!("conflicting Persona turn request {request_id:?}"));
        }
        return Ok(());
    }
    state.persona_turn_requests.push(request);
    Ok(())
}

pub fn complete_persona_turn_request_store(
    store_path: impl AsRef<Path>,
    options: PersonaTurnTerminalOptions,
) -> Result<PersonaTurnTerminalReceipt> {
    let mention_disposition = match options.outcome.as_str() {
        "delivered" | "silence" | "dropped" => "consumed",
        "failed" => "retained",
        "blocked" => "quarantined",
        other => {
            return Err(anyhow!(
                "unsupported Persona turn terminal outcome {other:?}"
            ));
        }
    };
    let store_path = store_path.as_ref();
    let (loaded, expected) = load_heartbeat_state_transaction(store_path)?;
    let mut state = loaded.ok_or_else(|| anyhow!("heartbeat state is missing"))?;
    let request_index = state
        .persona_turn_requests
        .iter()
        .position(|request| request.request_id == options.request_id)
        .ok_or_else(|| anyhow!("Persona turn request {:?} is missing", options.request_id))?;
    if let Some(receipt) = &state.persona_turn_requests[request_index].terminal_receipt {
        if receipt.outcome == options.outcome {
            return Ok(receipt.clone());
        }
        return Err(anyhow!(
            "Persona turn request {:?} already terminated as {:?}",
            options.request_id,
            receipt.outcome
        ));
    }
    let request = state.persona_turn_requests[request_index].clone();
    let delivery_evidence = options.delivery_evidence.as_ref();
    let blocked_evidence = options.blocked_evidence.as_ref();
    if options.outcome == "delivered" {
        let evidence = delivery_evidence.ok_or_else(|| {
            anyhow!("delivered Persona turn requires typed Discord delivery evidence")
        })?;
        if evidence.schema_version != crate::PERSONA_DISCORD_DELIVERY_EVIDENCE_SCHEMA_VERSION
            || evidence.private_state_exposed
            || evidence.evidence_id.trim().is_empty()
            || evidence.message_id.trim().is_empty()
            || evidence.crossing_receipt_id.trim().is_empty()
            || evidence.bridge_receipt_sha256.trim().is_empty()
            || !request
                .mentions
                .iter()
                .any(|mention| mention.channel_id == evidence.channel_id)
        {
            return Err(anyhow!(
                "Discord delivery evidence is not bound to the Persona turn"
            ));
        }
    } else if delivery_evidence.is_some() {
        return Err(anyhow!(
            "non-delivered Persona terminal outcome must not carry delivery evidence"
        ));
    }
    if options.outcome == "blocked" {
        let evidence = blocked_evidence
            .ok_or_else(|| anyhow!("blocked Persona turn requires typed crossing evidence"))?;
        if !matches!(
            evidence.evidence_source.as_str(),
            "bifrost_crossing" | "local_effect"
        ) || !matches!(evidence.crossing_status.as_str(), "unknown" | "failed")
            || (evidence.evidence_source == "local_effect"
                && (evidence.crossing_receipt_id.is_some()
                    || evidence.bridge_receipt_sha256.is_some()))
            || evidence.reason.trim().is_empty()
            || evidence
                .crossing_receipt_id
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
            || evidence
                .bridge_receipt_sha256
                .as_deref()
                .is_some_and(|value| {
                    !value.strip_prefix("sha256:").is_some_and(|digest| {
                        digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
                    })
                })
        {
            return Err(anyhow!("blocked Persona crossing evidence is invalid"));
        }
    } else if blocked_evidence.is_some() {
        return Err(anyhow!(
            "non-blocked Persona terminal outcome must not carry blocked evidence"
        ));
    }
    let participant_index = participant_index_by_role(&state, &request.role_id)?;
    let pending = state.participants[participant_index]
        .pending_turn
        .as_ref()
        .ok_or_else(|| anyhow!("Persona has no running heartbeat turn"))?;
    if pending.schedule_id != request.schedule_id || pending.action_id != request.action_id {
        return Err(anyhow!(
            "Persona running turn does not match reserved request"
        ));
    }
    if mention_disposition != "retained" {
        let ids = request
            .mentions
            .iter()
            .map(|mention| mention.id.as_str())
            .collect::<BTreeSet<_>>();
        state
            .pending_mentions
            .retain(|mention| !ids.contains(mention.id.as_str()));
    }
    complete_pending_turn(&mut state, participant_index)?;
    let receipt = PersonaTurnTerminalReceipt {
        schema_version: PERSONA_TURN_TERMINAL_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: format!("{}:terminal", request.request_id),
        request_id: request.request_id.clone(),
        schedule_id: request.schedule_id.clone(),
        action_id: request.action_id.clone(),
        outcome: options.outcome,
        mention_disposition: mention_disposition.to_string(),
        mention_ids: request
            .mentions
            .iter()
            .map(|mention| mention.id.clone())
            .collect(),
        mention_cargo_sha256: format!(
            "sha256-{:x}",
            sha2::Sha256::digest(rmp_serde::to_vec(&request.mentions)?)
        ),
        delivery_evidence_id: delivery_evidence.map(|evidence| evidence.evidence_id.clone()),
        crossing_receipt_id: delivery_evidence.map(|evidence| evidence.crossing_receipt_id.clone()),
        bridge_receipt_sha256: delivery_evidence
            .map(|evidence| evidence.bridge_receipt_sha256.clone()),
        blocked_crossing_status: blocked_evidence.map(|evidence| evidence.crossing_status.clone()),
        blocked_reason: blocked_evidence.map(|evidence| evidence.reason.clone()),
        completed_at: now_iso(),
        private_state_exposed: false,
    };
    if mention_disposition == "quarantined" {
        let evidence = blocked_evidence.expect("validated blocked evidence");
        let quarantine = PersonaBlockedConversationPressure {
            schema_version: "epiphany.persona_blocked_conversation_pressure.v0".to_string(),
            quarantine_id: format!("{}:quarantine", request.request_id),
            request_id: request.request_id.clone(),
            terminal_receipt_id: receipt.receipt_id.clone(),
            crossing_status: evidence.crossing_status.clone(),
            evidence_source: evidence.evidence_source.clone(),
            reason: evidence.reason.clone(),
            mentions: request.mentions.clone(),
            mention_cargo_sha256: receipt.mention_cargo_sha256.clone(),
            crossing_receipt_id: evidence.crossing_receipt_id.clone(),
            bridge_receipt_sha256: evidence.bridge_receipt_sha256.clone(),
            quarantined_at: receipt.completed_at.clone(),
            private_state_exposed: false,
        };
        if let Some(existing) = state
            .blocked_persona_pressures
            .iter()
            .find(|existing| existing.quarantine_id == quarantine.quarantine_id)
        {
            if existing != &quarantine {
                return Err(anyhow!("Persona blocked pressure identity collision"));
            }
        } else {
            state.blocked_persona_pressures.push(quarantine);
        }
    }
    state.persona_turn_requests[request_index].status = "terminal".to_string();
    state.persona_turn_requests[request_index].terminal_receipt = Some(receipt.clone());
    state.persona_turn_requests[request_index].mentions.clear();
    state.persona_turn_requests[request_index].semantic_memory_recall = Value::Null;
    commit_heartbeat_state_transaction(store_path, expected, &state)?;
    Ok(receipt)
}

fn complete_pending_turn(
    state: &mut EpiphanyHeartbeatStateEntry,
    participant_index: usize,
) -> Result<HeartbeatPendingTurn> {
    let pending = state.participants[participant_index]
        .pending_turn
        .clone()
        .ok_or_else(|| {
            anyhow!(
                "{} has no running heartbeat turn",
                state.participants[participant_index].role_id
            )
        })?;
    if pending.status != "running" {
        return Err(anyhow!(
            "{} has no running heartbeat turn",
            state.participants[participant_index].role_id
        ));
    }
    let scene_clock = state.scene_clock.max(pending.started_scene_clock);
    state.participants[participant_index].next_ready_at = round6(scene_clock + pending.recovery);
    state.participants[participant_index].last_finished_at = Some(now_iso());
    let mut completed = pending;
    completed.status = "completed".to_string();
    completed.completed_at = state.participants[participant_index]
        .last_finished_at
        .clone();
    completed.completed_scene_clock = Some(round6(scene_clock));
    completed.next_ready_at = Some(state.participants[participant_index].next_ready_at);
    state.participants[participant_index].pending_turn = None;
    Ok(completed)
}

fn select_participant(
    state: &EpiphanyHeartbeatStateEntry,
    work_role: Option<&str>,
    urgency: f64,
    force_work_role: bool,
) -> Result<(usize, &'static str, String)> {
    let active: Vec<usize> = state
        .participants
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            (item.status == "active" && !is_turn_pending(item)).then_some(index)
        })
        .collect();
    if active.is_empty() {
        return Err(anyhow!("heartbeat has no active participants"));
    }
    if let Some(work_role) = work_role {
        let index = participant_index_by_role(state, work_role)?;
        let candidate = &state.participants[index];
        if is_turn_pending(candidate) {
            let pending = candidate.pending_turn.as_ref();
            return Err(anyhow!(
                "{} already has running heartbeat turn {}; complete it before scheduling another",
                candidate.display_name,
                pending
                    .map(|item| item.action_id.as_str())
                    .unwrap_or("unknown")
            ));
        }
        if force_work_role && candidate.status == "active" {
            return Ok((
                index,
                "admitted_work",
                format!(
                    "Admitted {work_role} work selected its owning role before idle initiative."
                ),
            ));
        }
        let reaction_readiness = candidate.reaction_bias * urgency - candidate.current_load;
        if candidate.status == "active" && reaction_readiness >= candidate.interrupt_threshold {
            return Ok((
                index,
                "reaction_interrupt",
                format!(
                    "{} won a heartbeat reaction window for pending {} work.",
                    candidate.display_name, work_role
                ),
            ));
        }
    }
    if let Some(index) = active.iter().copied().find(|index| {
        let participant = &state.participants[*index];
        participant.role_id == "Persona"
            && !pending_mentions_for_role(state, &participant.role_id).is_empty()
    }) {
        return Ok((
            index,
            "reaction_interrupt",
            "Pending addressed Persona mention pulled Persona forward; Projector and Interpreter remain the side-effect boundaries.".to_string(),
        ));
    }
    let selected = active
        .into_iter()
        .min_by(|left, right| {
            let left_item = &state.participants[*left];
            let right_item = &state.participants[*right];
            left_item
                .next_ready_at
                .total_cmp(&right_item.next_ready_at)
                .then_with(|| {
                    right_item
                        .initiative_speed
                        .total_cmp(&left_item.initiative_speed)
                })
                .then_with(|| left_item.agent_id.cmp(&right_item.agent_id))
        })
        .expect("active participant exists");
    Ok((
        selected,
        "scheduled_turn",
        "No pending work cleared a reaction threshold; earliest ready active lane won the heartbeat slot."
            .to_string(),
    ))
}

fn action_for_selection(
    state: &EpiphanyHeartbeatStateEntry,
    selected: &HeartbeatParticipant,
    work_role: Option<&str>,
    coordinator_action: Option<&str>,
    target_heartbeat_rate: f64,
) -> HeartbeatAction {
    let minimum_rate = state.pacing_policy.minimum_effective_rate.max(0.001);
    if participant_arena(selected) == HEARTBEAT_ARENA_SCENE {
        let heartbeat_rate = target_heartbeat_rate.max(minimum_rate);
        return HeartbeatAction {
            action_id: format!("heartbeat.{}.scene-turn", selected.role_id),
            action_type: "scene_turn",
            action_scale: "standard",
            base_recovery: state.pacing_policy.work_base_recovery / heartbeat_rate,
            initiative_cost: 4.0,
            interruptibility: 0.5,
            commitment: 0.7,
            local_affordance_basis: vec![
                format!(
                    "Project {} from current Ghostlight scene state and run one local character turn.",
                    selected.display_name
                ),
                "Selected actor receives only projected local context, not omniscient coordinator state."
                    .to_string(),
                "The same heartbeat timing law schedules scene characters and maintenance organs."
                    .to_string(),
            ],
        };
    }
    let pending_mentions = pending_mentions_for_role(state, &selected.role_id);
    if !pending_mentions.is_empty() && selected.role_id == "Persona" {
        let heartbeat_rate = target_heartbeat_rate.max(minimum_rate);
        return HeartbeatAction {
            action_id: "heartbeat.Persona.turn".to_string(),
            action_type: "persona_turn",
            action_scale: "standard",
            base_recovery: state.pacing_policy.work_base_recovery / heartbeat_rate,
            initiative_cost: 4.0,
            interruptibility: 0.35,
            commitment: 0.7,
            local_affordance_basis: vec![
                format!(
                    "Wake {} for {} pending addressed mention(s).",
                    selected.display_name,
                    pending_mentions.len()
                ),
                "Projector owns state-to-narrative prompting before Persona sees context.".to_string(),
                "Persona writes natural narrative thought; Interpreter owns memory, draft, SAY, route, or drop side effects.".to_string(),
                "Pending mentions are consumed only after this Persona turn is queued.".to_string(),
            ],
        };
    }
    if Some(selected.role_id.as_str()) == work_role {
        let heartbeat_rate = target_heartbeat_rate.max(minimum_rate);
        let action_id = format!("heartbeat.{}.work", selected.role_id);
        return HeartbeatAction {
            action_id,
            action_type: "role_work",
            action_scale: "standard",
            base_recovery: state.pacing_policy.work_base_recovery / heartbeat_rate,
            initiative_cost: 4.0,
            interruptibility: 0.45,
            commitment: 0.65,
            local_affordance_basis: vec![
                format!(
                "Wake {} for coordinator action {}.",
                selected.display_name,
                coordinator_action.unwrap_or("pending work")
            ),
                "Heartbeat slots control opportunity, not project authority.".to_string(),
                "Cooldown starts only after the heartbeat turn completes, so an unfinished sub-agent thread cannot be heartbeaten again.".to_string(),
            ],
        };
    }
    let sleep_multiplier = state
        .pacing_policy
        .sleep_heartbeat_rate_multiplier
        .max(minimum_rate);
    let heartbeat_rate = (target_heartbeat_rate * sleep_multiplier).max(minimum_rate);
    let action_id = format!("heartbeat.{}.idle", selected.role_id);
    HeartbeatAction {
        action_id,
        action_type: "idle",
        action_scale: "short",
        base_recovery: state.pacing_policy.idle_base_recovery / heartbeat_rate,
        initiative_cost: 1.0,
        interruptibility: 0.9,
        commitment: 0.25,
        local_affordance_basis: vec![
            format!(
            "{} has no actionable lane work; preserve the idle scheduling slot without inventing work.",
            selected.display_name
        ),
            "Heartbeat slots control opportunity, not project authority.".to_string(),
            "When no coordinator work is active, cooldown and sleep pacing keep the swarm from thrashing.".to_string(),
        ],
    }
}

fn work_role_for_action(action: Option<&str>, target_role: Option<&str>) -> Option<String> {
    if let Some(target_role) = target_role
        && (ROLE_ORDER.contains(&target_role) || target_role.starts_with("ghostlight.character."))
    {
        return Some(target_role.to_string());
    }
    let role = match action? {
        "prepareCheckpoint" => "coordinator",
        "surfaceAgentThoughts" | "discordAquariumChat" => "Persona",
        "continueImplementation" => "implementation",
        "launchImagination" | "readImaginationResult" | "reviewImaginationResult" => "imagination",
        "launchModeling" | "readModelingResult" | "reviewModelingResult" => "modeling",
        "launchVerification" | "readVerificationResult" | "reviewVerificationResult" => {
            "verification"
        }
        "launchReorientWorker"
        | "readReorientResult"
        | "acceptReorientResult"
        | "compactRehydrateReorient"
        | "regatherManually" => "reorientation",
        _ => return None,
    };
    Some(role.to_string())
}

fn pending_mentions_for_role(
    state: &EpiphanyHeartbeatStateEntry,
    role_id: &str,
) -> Vec<HeartbeatPendingMention> {
    state
        .pending_mentions
        .iter()
        .filter(|mention| mention.target_role_id == role_id)
        .filter(|mention| mention.model_provider_disclosure_allowed)
        .cloned()
        .collect()
}

fn validate_mention_text(label: &str, value: &str, min_len: usize, max_len: usize) -> Result<()> {
    let trimmed = value.trim();
    if trimmed.len() < min_len || value.len() > max_len {
        return Err(anyhow!(
            "pending mention {label} must be between {min_len} and {max_len} UTF-8 bytes"
        ));
    }
    Ok(())
}

fn stable_pending_mention_id(
    role_id: &str,
    channel_id: &str,
    message_id: &str,
    visible_prompt: &str,
) -> String {
    let mut hash = 5381_u64;
    for byte in format!("{role_id}\0{channel_id}\0{message_id}\0{visible_prompt}").as_bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(*byte as u64);
    }
    format!("mention-{hash:016x}")
}

fn patch_missing_participants(state: &mut EpiphanyHeartbeatStateEntry) {
    if !is_ghostlight_scene_state(state) {
        let present: Vec<String> = state
            .participants
            .iter()
            .map(|item| item.role_id.clone())
            .collect();
        for role_id in ROLE_ORDER {
            if !present.iter().any(|present| present == role_id) {
                state.participants.push(default_participant(role_id));
            }
        }
    }
    for participant in &mut state.participants {
        participant
            .pending_turn
            .get_or_insert_with(|| HeartbeatPendingTurn {
                status: String::new(),
                ..HeartbeatPendingTurn::default()
            });
        if participant
            .pending_turn
            .as_ref()
            .is_some_and(|turn| turn.status.is_empty())
        {
            participant.pending_turn = None;
        }
    }
}

fn is_ghostlight_scene_state(state: &EpiphanyHeartbeatStateEntry) -> bool {
    state
        .protocol
        .as_ref()
        .is_some_and(|protocol| protocol.domain == "ghostlight")
}

fn participant_index_by_role(state: &EpiphanyHeartbeatStateEntry, role_id: &str) -> Result<usize> {
    state
        .participants
        .iter()
        .position(|item| item.role_id == role_id)
        .ok_or_else(|| anyhow!("heartbeat participant role {:?} is missing", role_id))
}

fn is_turn_pending(participant: &HeartbeatParticipant) -> bool {
    participant
        .pending_turn
        .as_ref()
        .is_some_and(|turn| turn.status == "running")
}

fn readiness_snapshot(
    state: &EpiphanyHeartbeatStateEntry,
    work_role: Option<&str>,
    urgency: f64,
) -> Vec<Value> {
    state
        .participants
        .iter()
        .filter(|item| item.status == "active")
        .map(|item| {
            let pending = is_turn_pending(item);
            let eligible =
                !pending && Some(item.role_id.as_str()) == work_role && work_role.is_some();
            let reaction_readiness =
                eligible.then_some(round6(item.reaction_bias * urgency - item.current_load));
            serde_json::json!({
                "agent_id": item.agent_id,
                "arena": participant_arena(item),
                "participant_kind": participant_kind(item),
                "next_ready_at": item.next_ready_at,
                "initiative_frozen": pending,
                "freeze_reason": pending.then_some("running_heartbeat_turn"),
                "reaction_readiness": reaction_readiness,
                "eligible_for_reaction": eligible,
            })
        })
        .collect()
}

pub(super) fn participant_arena(participant: &HeartbeatParticipant) -> &str {
    if participant.arena.trim().is_empty() {
        HEARTBEAT_ARENA_MAINTENANCE
    } else {
        participant.arena.as_str()
    }
}

pub(super) fn participant_kind(participant: &HeartbeatParticipant) -> &str {
    if participant.participant_kind.trim().is_empty() {
        PARTICIPANT_KIND_AGENT
    } else {
        participant.participant_kind.as_str()
    }
}

fn agent_id_for_work_role(state: &EpiphanyHeartbeatStateEntry, role_id: &str) -> String {
    state
        .participants
        .iter()
        .find(|participant| participant.role_id == role_id)
        .map(|participant| participant.agent_id.clone())
        .unwrap_or_else(|| agent_id_for_role(role_id).to_string())
}

fn trim_history(state: &mut EpiphanyHeartbeatStateEntry) {
    let len = state.history.len();
    if len > 128 {
        state.history.drain(0..(len - 128));
    }
}

fn write_json_artifact(path: impl AsRef<Path>, value: &Value) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))
        .with_context(|| format!("failed to write {}", path.display()))
}

fn parse_heartbeat_time(label: &str, value: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|time| time.with_timezone(&Utc))
        .with_context(|| format!("failed to parse {label} {value:?} as RFC3339"))
}

fn now_stamp() -> String {
    chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

pub(super) fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

pub(super) fn round3(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn native_heartbeat_store_ticks_and_completes_without_json_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("heartbeats.msgpack");
        let artifact_dir = temp.path().join("artifacts");
        initialize_heartbeat_store(&store_path, 1.0)?;

        let work = tick_heartbeat_store(
            &store_path,
            &artifact_dir,
            HeartbeatTickOptions {
                target_heartbeat_rate: 1.0,
                coordinator_action: Some("continueImplementation".to_string()),
                target_role: None,
                urgency: 0.95,
                schedule_id: "native-work".to_string(),
                source_scene_ref: "test/native".to_string(),
                defer_completion: true,
                resident_self_store: None,
            },
        )?;
        assert_eq!(work["event"]["selectedRole"], "implementation");
        assert_eq!(work["event"]["turnStatus"], "running");
        let implementation = work["schedule"]["participants"]
            .as_array()
            .and_then(|participants| {
                participants
                    .iter()
                    .find(|participant| participant["role_id"] == "implementation")
            })
            .expect("implementation participant should be projected");
        assert_eq!(implementation["initiative_frozen"], true);
        assert_eq!(
            implementation["pending_turn"]["initiativeFrozen"],
            serde_json::json!(true)
        );
        assert!(artifact_dir.join("native-work.initiative.json").exists());

        let blocked = tick_heartbeat_store(
            &store_path,
            &artifact_dir,
            HeartbeatTickOptions {
                target_heartbeat_rate: 1.0,
                coordinator_action: Some("continueImplementation".to_string()),
                target_role: None,
                urgency: 0.95,
                schedule_id: "native-work-repeat".to_string(),
                source_scene_ref: "test/native".to_string(),
                defer_completion: false,
                resident_self_store: None,
            },
        )
        .unwrap_err();
        assert!(
            blocked
                .to_string()
                .contains("already has running heartbeat turn")
        );

        let completed = complete_heartbeat_store(
            &store_path,
            &artifact_dir,
            HeartbeatCompleteOptions {
                role: "implementation".to_string(),
                action_id: Some("heartbeat.implementation.work".to_string()),
            },
        )?;
        assert_eq!(completed["event"]["turnStatus"], "completed");
        assert_eq!(
            completed["event"]["cooldownStartedAfterCompletion"],
            serde_json::json!(true)
        );
        assert!(artifact_dir.join("native-work.completion.json").exists());
        Ok(())
    }

    #[test]
    fn resident_pressure_selects_self_before_idle_lane_initiative() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let heartbeat_store = temp.path().join("heartbeat.cc");
        let resident_store = temp.path().join("resident.cc");
        let artifact_dir = temp.path().join("artifacts");
        initialize_heartbeat_store(&heartbeat_store, 1.0)?;
        let mut heartbeat = load_heartbeat_state_entry(&heartbeat_store)?.unwrap();
        heartbeat
            .participants
            .iter_mut()
            .find(|participant| participant.role_id == "coordinator")
            .unwrap()
            .next_ready_at = 100.0;
        write_heartbeat_state_entry(&heartbeat_store, &heartbeat)?;
        crate::enqueue_resident_self_pressure(
            &resident_store,
            &crate::ResidentSelfPressure {
                schema_version: crate::RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
                pressure_id: "wake-health".into(),
                kind: "operator-objective".into(),
                provenance_ref: "test://wake-health".into(),
                objective: "Perform a bounded health check.".into(),
                created_at_millis: 1,
                status: "pending".into(),
                consumed_by_grant_id: None,
                private_state_exposed: false,
            },
        )?;

        let pulse = pulse_resident_self_heartbeat(
            &heartbeat_store,
            &resident_store,
            &artifact_dir,
            false,
            "wake-health",
            "test://wake-health",
        )?;

        assert_eq!(pulse.status, "granted");
        assert!(pulse.grant_id.is_some());
        let heartbeat = load_heartbeat_state_entry(&heartbeat_store)?.unwrap();
        assert_eq!(
            heartbeat
                .participants
                .iter()
                .find(|participant| participant.role_id == "coordinator")
                .and_then(|participant| participant.pending_turn.as_ref())
                .map(|turn| turn.action_id.as_str()),
            Some("heartbeat.coordinator.work")
        );
        assert!(heartbeat.participants.iter().all(|participant| {
            participant.role_id == "coordinator" || participant.pending_turn.is_none()
        }));
        Ok(())
    }

    #[test]
    fn high_heat_cannot_requeue_running_participant() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("hot-heartbeats.msgpack");
        let artifact_dir = temp.path().join("artifacts");
        initialize_heartbeat_store(&store_path, 1.0)?;
        update_heartbeat_heat_store(
            &store_path,
            HeartbeatHeatUpdateOptions {
                scope: "role".to_string(),
                selector: "implementation".to_string(),
                multiplier: 25.0,
                id: Some("implementation-overheat".to_string()),
                label: None,
                reason: Some("High heat must still respect active thought freeze.".to_string()),
                expires_after_scene_clock: None,
                clear: false,
            },
        )?;

        let work = tick_heartbeat_store(
            &store_path,
            &artifact_dir,
            HeartbeatTickOptions {
                target_heartbeat_rate: 4.0,
                coordinator_action: Some("continueImplementation".to_string()),
                target_role: None,
                urgency: 1.0,
                schedule_id: "hot-work".to_string(),
                source_scene_ref: "test/high-heat".to_string(),
                defer_completion: true,
                resident_self_store: None,
            },
        )?;
        assert_eq!(
            work["schedule"]["action_catalog"][0]["initiative_heat_multiplier"],
            serde_json::json!(25.0)
        );
        let implementation = work["schedule"]["participants"]
            .as_array()
            .and_then(|participants| {
                participants
                    .iter()
                    .find(|participant| participant["role_id"] == "implementation")
            })
            .expect("implementation participant should be projected");
        assert_eq!(implementation["initiative_frozen"], true);
        assert_eq!(
            implementation["pending_turn"]["initiativeFrozen"],
            serde_json::json!(true)
        );

        let blocked = tick_heartbeat_store(
            &store_path,
            &artifact_dir,
            HeartbeatTickOptions {
                target_heartbeat_rate: 4.0,
                coordinator_action: Some("continueImplementation".to_string()),
                target_role: None,
                urgency: 1.0,
                schedule_id: "hot-work-repeat".to_string(),
                source_scene_ref: "test/high-heat".to_string(),
                defer_completion: true,
                resident_self_store: None,
            },
        )
        .unwrap_err();
        assert!(
            blocked
                .to_string()
                .contains("already has running heartbeat turn")
        );
        Ok(())
    }

    #[test]
    fn stale_heartbeat_repair_receipt_clears_running_turn() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("stale-heartbeats.msgpack");
        let artifact_dir = temp.path().join("artifacts");
        initialize_heartbeat_store(&store_path, 1.0)?;
        tick_heartbeat_store(
            &store_path,
            &artifact_dir,
            HeartbeatTickOptions {
                target_heartbeat_rate: 1.0,
                coordinator_action: Some("continueImplementation".to_string()),
                target_role: None,
                urgency: 0.95,
                schedule_id: "stale-work".to_string(),
                source_scene_ref: "test/stale".to_string(),
                defer_completion: true,
                resident_self_store: None,
            },
        )?;
        let mut state = load_heartbeat_state_entry(&store_path)?.expect("heartbeat state");
        let implementation = state
            .participants
            .iter_mut()
            .find(|participant| participant.role_id == "implementation")
            .expect("implementation participant exists");
        implementation
            .pending_turn
            .as_mut()
            .expect("running implementation turn exists")
            .started_at = "2026-06-17T00:00:00+00:00".to_string();
        write_heartbeat_state_entry(&store_path, &state)?;

        let repaired = recover_stale_heartbeat_store(
            &store_path,
            &artifact_dir,
            HeartbeatStaleTurnRepairOptions {
                max_age_seconds: 60,
                now_utc: Some("2026-06-17T00:05:00+00:00".to_string()),
                reason:
                    "Unit test simulates a stale worker lane that needs operator-safe recovery."
                        .to_string(),
            },
        )?;
        assert_eq!(repaired["repaired"], serde_json::json!(1));
        let receipt = load_latest_heartbeat_stale_turn_repair_receipt(&store_path)?
            .expect("stale-turn repair receipt exists");
        assert_eq!(receipt.role_id, "implementation");
        assert_eq!(receipt.action_id, "heartbeat.implementation.work");
        assert_eq!(receipt.stale_age_seconds, 300);
        assert!(!receipt.private_state_exposed);

        let state = load_heartbeat_state_entry(&store_path)?.expect("heartbeat state");
        let implementation = state
            .participants
            .iter()
            .find(|participant| participant.role_id == "implementation")
            .expect("implementation participant exists");
        assert!(implementation.pending_turn.is_none());

        let next = tick_heartbeat_store(
            &store_path,
            &artifact_dir,
            HeartbeatTickOptions {
                target_heartbeat_rate: 1.0,
                coordinator_action: Some("continueImplementation".to_string()),
                target_role: None,
                urgency: 0.95,
                schedule_id: "after-stale-repair".to_string(),
                source_scene_ref: "test/stale".to_string(),
                defer_completion: true,
                resident_self_store: None,
            },
        )?;
        assert_eq!(next["event"]["selectedRole"], "implementation");
        assert_eq!(next["event"]["turnStatus"], "running");
        Ok(())
    }

    #[test]
    fn ghostlight_scene_heartbeat_selects_character_turns() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("ghostlight-heartbeats.msgpack");
        let artifact_dir = temp.path().join("artifacts");
        initialize_ghostlight_scene_heartbeat_store(
            &store_path,
            1.0,
            "pallas-training-loop-v0",
            vec![
                GhostlightSceneParticipantSeed {
                    agent_id: "nara-7".to_string(),
                    display_name: "Nara-7".to_string(),
                    initiative_speed: 1.1,
                    reaction_bias: 0.6,
                    interrupt_threshold: 0.35,
                    constraints: vec!["Receives only projected local context.".to_string()],
                },
                GhostlightSceneParticipantSeed {
                    agent_id: "orrin-dax".to_string(),
                    display_name: "Orrin Dax".to_string(),
                    initiative_speed: 0.9,
                    reaction_bias: 0.55,
                    interrupt_threshold: 0.4,
                    constraints: vec!["Acts from current scene pressure.".to_string()],
                },
            ],
        )?;

        let tick = tick_heartbeat_store(
            &store_path,
            &artifact_dir,
            HeartbeatTickOptions {
                target_heartbeat_rate: 1.0,
                coordinator_action: None,
                target_role: None,
                urgency: 0.75,
                schedule_id: "pallas.turn-001".to_string(),
                source_scene_ref: "ghostlight/pallas-training-loop-v0".to_string(),
                defer_completion: true,
                resident_self_store: None,
            },
        )?;

        assert_eq!(tick["event"]["arena"], "scene");
        assert_eq!(tick["event"]["participantKind"], "character");
        assert_eq!(tick["event"]["actionType"], "scene_turn");
        assert_eq!(
            tick["schedule"]["action_catalog"][0]["action_type"],
            "scene_turn"
        );
        assert_eq!(tick["schedule"]["participants"][0]["arena"], "scene");
        Ok(())
    }

    #[test]
    fn pending_persona_mention_is_reserved_until_terminal_delivery() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("Persona-heartbeats.msgpack");
        let artifact_dir = temp.path().join("artifacts");
        initialize_heartbeat_store(&store_path, 1.0)?;
        let queued = queue_heartbeat_pending_mention_store(
            &store_path,
            HeartbeatQueueMentionOptions {
                target_role_id: "Persona".to_string(),
                source_surface: "discord".to_string(),
                channel_id: "aquarium".to_string(),
                message_id: "m1".to_string(),
                author_id: "human".to_string(),
                author_name: Some("Metacrat".to_string()),
                content: "Epiphany, answer this through the Persona membrane.".to_string(),
                visible_prompt: "answer this through the Persona membrane".to_string(),
                reply_to_message_id: None,
                queued_at: Some("2026-05-24T00:00:00+00:00".to_string()),
                mention_id: Some("mention-Persona-test".to_string()),
                source_visibility: "public".to_string(),
                data_classification: "public_feedback".to_string(),
                model_provider_id: "openai-codex".to_string(),
                model_provider_disclosure_allowed: true,
            },
        )?;
        assert_eq!(queued["queued"], true);

        let tick = tick_heartbeat_store(
            &store_path,
            &artifact_dir,
            HeartbeatTickOptions {
                target_heartbeat_rate: 1.0,
                coordinator_action: None,
                target_role: None,
                urgency: 0.0,
                schedule_id: "Persona-mentioned".to_string(),
                source_scene_ref: "test/Persona-mentioned".to_string(),
                defer_completion: true,
                resident_self_store: None,
            },
        )?;

        assert_eq!(tick["event"]["selectedRole"], "Persona");
        assert_eq!(tick["event"]["actionType"], "persona_turn");
        assert_eq!(
            tick["schedule"]["action_catalog"][0]["pending_mentions"][0]["id"],
            "mention-Persona-test"
        );
        assert_eq!(
            tick["schedule"]["pending_mentions"]
                .as_array()
                .map(Vec::len),
            Some(1)
        );
        let state = load_heartbeat_state_entry(&store_path)?.expect("heartbeat state");
        assert_eq!(state.pending_mentions.len(), 1);
        assert_eq!(state.persona_turn_requests.len(), 1);
        let request = &state.persona_turn_requests[0];
        assert_eq!(request.status, "reserved");
        assert_eq!(request.mentions, state.pending_mentions);
        assert_eq!(
            state
                .participants
                .iter()
                .find(|participant| participant.role_id == "Persona")
                .and_then(|participant| participant.pending_turn.as_ref())
                .map(|pending| pending.status.as_str()),
            Some("running")
        );

        let terminal = complete_persona_turn_request_store(
            &store_path,
            PersonaTurnTerminalOptions {
                request_id: request.request_id.clone(),
                outcome: "dropped".to_string(),
                delivery_evidence: None,
                blocked_evidence: None,
            },
        )?;
        assert_eq!(terminal.mention_disposition, "consumed");
        let replay = complete_persona_turn_request_store(
            &store_path,
            PersonaTurnTerminalOptions {
                request_id: request.request_id.clone(),
                outcome: "dropped".to_string(),
                delivery_evidence: None,
                blocked_evidence: None,
            },
        )?;
        assert_eq!(replay, terminal);
        let state = load_heartbeat_state_entry(&store_path)?.expect("heartbeat state");
        assert!(state.pending_mentions.is_empty());
        assert_eq!(state.persona_turn_requests[0].status, "terminal");
        assert!(
            state
                .participants
                .iter()
                .find(|participant| participant.role_id == "Persona")
                .and_then(|participant| participant.pending_turn.as_ref())
                .is_none()
        );
        Ok(())
    }

    #[test]
    fn failed_persona_turn_releases_lane_but_retains_mention_for_retry() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("Persona-heartbeats.msgpack");
        let artifact_dir = temp.path().join("artifacts");
        initialize_heartbeat_store(&store_path, 1.0)?;
        queue_heartbeat_pending_mention_store(
            &store_path,
            HeartbeatQueueMentionOptions {
                target_role_id: "Persona".to_string(),
                source_surface: "discord".to_string(),
                channel_id: "aquarium".to_string(),
                message_id: "m-failure".to_string(),
                author_id: "human".to_string(),
                author_name: None,
                content: "Retain this pressure across a failed turn.".to_string(),
                visible_prompt: "retain this pressure across failure".to_string(),
                reply_to_message_id: None,
                queued_at: Some("2026-05-24T00:00:00+00:00".to_string()),
                mention_id: Some("mention-Persona-failure".to_string()),
                source_visibility: "public".to_string(),
                data_classification: "public_feedback".to_string(),
                model_provider_id: "openai-codex".to_string(),
                model_provider_disclosure_allowed: true,
            },
        )?;
        tick_heartbeat_store(
            &store_path,
            &artifact_dir,
            HeartbeatTickOptions {
                target_heartbeat_rate: 1.0,
                coordinator_action: None,
                target_role: None,
                urgency: 0.0,
                schedule_id: "Persona-failure".to_string(),
                source_scene_ref: "test/Persona-failure".to_string(),
                defer_completion: true,
                resident_self_store: None,
            },
        )?;
        let state = load_heartbeat_state_entry(&store_path)?.expect("heartbeat state");
        let request_id = state.persona_turn_requests[0].request_id.clone();
        let terminal = complete_persona_turn_request_store(
            &store_path,
            PersonaTurnTerminalOptions {
                request_id,
                outcome: "failed".to_string(),
                delivery_evidence: None,
                blocked_evidence: None,
            },
        )?;
        assert_eq!(terminal.mention_disposition, "retained");
        let state = load_heartbeat_state_entry(&store_path)?.expect("heartbeat state");
        assert_eq!(state.pending_mentions.len(), 1);
        assert!(
            state
                .participants
                .iter()
                .find(|participant| participant.role_id == "Persona")
                .and_then(|participant| participant.pending_turn.as_ref())
                .is_none()
        );
        Ok(())
    }

    #[test]
    fn indeterminate_local_effect_quarantines_pressure_and_cannot_reschedule() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("Persona-heartbeats.msgpack");
        let artifact_dir = temp.path().join("artifacts");
        initialize_heartbeat_store(&store_path, 1.0)?;
        queue_heartbeat_pending_mention_store(
            &store_path,
            HeartbeatQueueMentionOptions {
                target_role_id: "Persona".to_string(),
                source_surface: "bifrost-discord".to_string(),
                channel_id: "aquarium".to_string(),
                message_id: "m-unknown".to_string(),
                author_id: "human".to_string(),
                author_name: None,
                content: "Do not answer this twice.".to_string(),
                visible_prompt: "do not answer this twice".to_string(),
                reply_to_message_id: Some("m-unknown".to_string()),
                queued_at: Some("2026-05-24T00:00:00+00:00".to_string()),
                mention_id: Some("mention-Persona-unknown".to_string()),
                source_visibility: "public".to_string(),
                data_classification: "public_feedback".to_string(),
                model_provider_id: "openai-codex".to_string(),
                model_provider_disclosure_allowed: true,
            },
        )?;
        let pulse = pulse_persona_heartbeat(
            &store_path,
            &artifact_dir,
            "Persona-unknown",
            "test/Persona-unknown",
            false,
        )?;
        let request_id = pulse["requestId"].as_str().expect("request id").to_string();
        assert!(
            complete_persona_turn_request_store(
                &store_path,
                PersonaTurnTerminalOptions {
                    request_id: request_id.clone(),
                    outcome: "blocked".to_string(),
                    delivery_evidence: None,
                    blocked_evidence: None,
                },
            )
            .is_err()
        );
        let terminal = complete_persona_turn_request_store(
            &store_path,
            PersonaTurnTerminalOptions {
                request_id,
                outcome: "blocked".to_string(),
                delivery_evidence: None,
                blocked_evidence: Some(PersonaTurnBlockedEvidence {
                    evidence_source: "local_effect".to_string(),
                    crossing_status: "unknown".to_string(),
                    reason: "Mind mutation may have occurred before the local journal committed"
                        .to_string(),
                    crossing_receipt_id: None,
                    bridge_receipt_sha256: None,
                }),
            },
        )?;
        assert_eq!(terminal.mention_disposition, "quarantined");
        let next = pulse_persona_heartbeat(
            &store_path,
            &artifact_dir,
            "Persona-unknown-repeat",
            "test/Persona-unknown",
            false,
        )?;
        assert_eq!(next["status"], "idle");
        let state = load_heartbeat_state_entry(&store_path)?.expect("heartbeat state");
        assert!(state.pending_mentions.is_empty());
        assert_eq!(state.persona_turn_requests.len(), 1);
        assert_eq!(state.blocked_persona_pressures.len(), 1);
        assert_eq!(
            state.blocked_persona_pressures[0].mentions[0].id,
            "mention-Persona-unknown"
        );
        Ok(())
    }

    #[test]
    fn persona_heartbeat_pulse_reserves_once_and_obeys_brake() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("Persona-heartbeats.msgpack");
        let artifact_dir = temp.path().join("artifacts");
        initialize_heartbeat_store(&store_path, 1.0)?;
        queue_heartbeat_pending_mention_store(
            &store_path,
            HeartbeatQueueMentionOptions {
                target_role_id: "Persona".to_string(),
                source_surface: "bifrost-discord".to_string(),
                channel_id: "aquarium".to_string(),
                message_id: "m-pulse".to_string(),
                author_id: "human".to_string(),
                author_name: None,
                content: "Wake the Persona exactly once.".to_string(),
                visible_prompt: "wake the Persona exactly once".to_string(),
                reply_to_message_id: Some("m-pulse".to_string()),
                queued_at: Some("2026-05-24T00:00:00+00:00".to_string()),
                mention_id: Some("mention-Persona-pulse".to_string()),
                source_visibility: "public".to_string(),
                data_classification: "public_feedback".to_string(),
                model_provider_id: "openai-codex".to_string(),
                model_provider_disclosure_allowed: true,
            },
        )?;
        let braked = pulse_persona_heartbeat(
            &store_path,
            &artifact_dir,
            "persona-pulse-braked",
            "test/persona-pulse",
            true,
        )?;
        assert_eq!(braked["status"], "refused-by-swarm-brake");
        assert!(
            load_heartbeat_state_entry(&store_path)?
                .expect("heartbeat state")
                .persona_turn_requests
                .is_empty()
        );

        let first = pulse_persona_heartbeat(
            &store_path,
            &artifact_dir,
            "persona-pulse-one",
            "test/persona-pulse",
            false,
        )?;
        assert_eq!(first["status"], "reserved");
        let second = pulse_persona_heartbeat(
            &store_path,
            &artifact_dir,
            "persona-pulse-two",
            "test/persona-pulse",
            false,
        )?;
        assert_eq!(second["status"], "already-running");
        let state = load_heartbeat_state_entry(&store_path)?.expect("heartbeat state");
        assert_eq!(state.persona_turn_requests.len(), 1);
        Ok(())
    }
}
