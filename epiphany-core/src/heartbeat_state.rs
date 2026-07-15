use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use epiphany_state_model::EpiphanyMemoryContextQuery;
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

mod heartbeat_documents;
mod heartbeat_pacing;
mod heartbeat_projection;
mod heartbeat_roles;
mod heartbeat_store;
pub use heartbeat_documents::*;
use heartbeat_pacing::adaptive_swarm_pacing;
use heartbeat_pacing::apply_initiative_heat_policy;
use heartbeat_pacing::effective_cooldown_multiplier;
use heartbeat_pacing::initiative_heat_multiplier;
use heartbeat_pacing::mood_cooldown_multiplier;
use heartbeat_pacing::personality_cooldown_multiplier;
use heartbeat_pacing::running_turn_count;
pub use heartbeat_projection::heartbeat_status_projection;
use heartbeat_projection::history_event_json;
use heartbeat_projection::pending_turn_json;
use heartbeat_projection::schedule_participant_json;
use heartbeat_projection::selection_policy_json;
use heartbeat_roles::ROLE_ORDER;
use heartbeat_roles::agent_id_for_role;
pub use heartbeat_roles::default_heartbeat_state;
use heartbeat_roles::default_participant;
use heartbeat_roles::display_name_for_role;
pub use heartbeat_roles::ghostlight_scene_heartbeat_state;
pub use heartbeat_roles::initialize_ghostlight_scene_heartbeat_store;
pub use heartbeat_roles::initialize_heartbeat_store;
pub use heartbeat_store::heartbeat_state_cache;
pub use heartbeat_store::load_heartbeat_cognition_entry;
pub use heartbeat_store::load_heartbeat_state_entry;
pub use heartbeat_store::load_latest_heartbeat_stale_turn_repair_receipt;
pub use heartbeat_store::validate_heartbeat_cognition;
pub use heartbeat_store::validate_heartbeat_state;
pub use heartbeat_store::write_heartbeat_cognition_entry;
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

pub fn run_void_routine_store(
    store_path: impl AsRef<Path>,
    artifact_dir: impl AsRef<Path>,
    options: VoidRoutineOptions,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    let mut state =
        load_heartbeat_state_entry(store_path)?.unwrap_or_else(|| default_heartbeat_state(1.0));
    patch_missing_participants(&mut state);
    let previous_cognition = load_heartbeat_cognition_entry(store_path)?;

    let memory_records = collect_role_memory_records(options.agent_store.as_deref())?;
    let appraisal_profiles = collect_role_appraisal_profiles(options.agent_store.as_deref())?;
    apply_personality_timing_profiles(&mut state, &appraisal_profiles);
    let resonance = build_memory_resonance(&memory_records);
    let incubation = build_incubation(
        &previous_cognition
            .as_ref()
            .and_then(|entry| entry.incubation.clone()),
        &previous_cognition
            .as_ref()
            .and_then(|entry| entry.bridge.clone()),
        &previous_cognition
            .as_ref()
            .and_then(|entry| entry.candidate_interventions.clone()),
        &resonance,
        &memory_records,
    );
    let thought_lanes = build_thought_lanes(&resonance, &incubation, &memory_records);
    let bridge = build_thought_bridge(
        &previous_cognition
            .as_ref()
            .and_then(|entry| entry.bridge.clone()),
        &thought_lanes,
        &resonance,
        &incubation,
    );
    let candidate_interventions = build_candidate_interventions(&bridge, &incubation);
    let appraisals =
        build_agent_appraisals(&appraisal_profiles, &thought_lanes, &incubation, &bridge);
    let reactions = build_agent_reactions(&appraisals, &bridge);
    apply_mood_timing_from_appraisals(&mut state, &appraisals);
    let sleep_cycle = update_sleep_cycle(
        previous_cognition
            .as_ref()
            .and_then(|entry| entry.sleep_cycle.as_ref()),
        &incubation,
        options.allow_dream,
    );
    let run_id = format!("epiphany-void-routine-{}", now_stamp());
    let artifact_dir = artifact_dir.as_ref();
    fs::create_dir_all(artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    let artifact_path = artifact_dir.join(format!("{run_id}.routine.json"));
    let routine = serde_json::json!({
        "schema_version": VOID_ROUTINE_SCHEMA_VERSION,
        "runId": run_id,
        "source": options.source,
        "referenceLineage": "VoidBot-style room stewardship, sleep, resonance, and dream maintenance rebuilt as Epiphany-native heartbeat physiology.",
        "storeFile": store_path,
        "agentStore": options.agent_store,
        "updatedAt": now_iso(),
        "sleepCycle": &sleep_cycle,
        "memoryResonance": &resonance,
        "incubation": &incubation,
        "thoughtLanes": &thought_lanes,
        "bridge": &bridge,
        "candidateInterventions": &candidate_interventions,
        "appraisals": &appraisals,
        "reactions": &reactions,
        "reviewNotes": [
            "Void is reference material, not a runtime dependency.",
            "The routine mutates only typed heartbeat physiology fields; project truth and role memory mutation stay on their dedicated reviewed surfaces.",
            "Analytic and associative lanes are cognition context, not hidden authority; the bridge decides draft, speech, silence, or further incubation.",
            "Appraisal projects clustered thoughts through each agent's own personality vectors; reaction is derived from that appraisal and remains separate from state mutation.",
            "Sleep is maintenance: slow rumination, memory compression, and dream residue, not absence."
        ],
    });

    write_json_artifact(&artifact_path, &routine)?;
    write_heartbeat_cognition_entry(
        store_path,
        &EpiphanyHeartbeatCognitionEntry {
            schema_version: HEARTBEAT_COGNITION_SCHEMA_VERSION.to_string(),
            updated_at: now_iso(),
            latest_run_id: Some(run_id),
            latest_artifact_ref: Some(artifact_path.display().to_string()),
            source: Some(options.source),
            sleep_cycle: Some(sleep_cycle),
            memory_resonance: Some(resonance),
            incubation: Some(incubation),
            thought_lanes: Some(thought_lanes),
            bridge: Some(bridge),
            candidate_interventions: Some(candidate_interventions),
            appraisals: Some(appraisals),
            reactions: Some(reactions),
        },
    )?;
    write_heartbeat_state_entry(store_path, &state)?;

    Ok(serde_json::json!({
        "ok": true,
        "storeFile": store_path,
        "artifactPath": artifact_path,
        "routine": routine,
    }))
}

pub fn tick_heartbeat_store(
    store_path: impl AsRef<Path>,
    artifact_dir: impl AsRef<Path>,
    options: HeartbeatTickOptions,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    let mut state = load_heartbeat_state_entry(store_path)?
        .unwrap_or_else(|| default_heartbeat_state(options.target_heartbeat_rate));
    if options.target_heartbeat_rate > 0.0 {
        state.target_heartbeat_rate = options.target_heartbeat_rate;
    }
    patch_missing_participants(&mut state);
    if let Some(agent_store) = options.agent_store.as_deref() {
        let appraisal_profiles = collect_role_appraisal_profiles(Some(agent_store))?;
        apply_personality_timing_profiles(&mut state, &appraisal_profiles);
    }
    if let Some(appraisals) =
        load_heartbeat_cognition_entry(store_path)?.and_then(|entry| entry.appraisals)
    {
        apply_mood_timing_from_appraisals(&mut state, &appraisals);
    }
    apply_initiative_heat_policy(&mut state);
    let result = tick_once(&mut state, &options)?;
    write_heartbeat_state_entry(store_path, &state)?;

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
        "rumination": result["rumination"].clone(),
    }))
}

pub fn pump_heartbeat_store(
    store_path: impl AsRef<Path>,
    artifact_dir: impl AsRef<Path>,
    options: HeartbeatPumpOptions,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    let mut state = load_heartbeat_state_entry(store_path)?
        .unwrap_or_else(|| default_heartbeat_state(options.base_heartbeat_rate.max(0.001)));
    if options.base_heartbeat_rate > 0.0 {
        state.target_heartbeat_rate = options.base_heartbeat_rate;
    }
    patch_missing_participants(&mut state);
    if let Some(agent_store) = options.agent_store.as_deref() {
        let appraisal_profiles = collect_role_appraisal_profiles(Some(agent_store))?;
        apply_personality_timing_profiles(&mut state, &appraisal_profiles);
    }
    if let Some(appraisals) =
        load_heartbeat_cognition_entry(store_path)?.and_then(|entry| entry.appraisals)
    {
        apply_mood_timing_from_appraisals(&mut state, &appraisals);
    }
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
            agent_store: options.agent_store.clone(),
        };
        match tick_once(&mut state, &tick_options) {
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
                    "rumination": result["rumination"],
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
    validate_mention_text("content", &options.content, 4, 4000)?;
    validate_mention_text("visible_prompt", &options.visible_prompt, 4, 1200)?;
    for (label, value) in [
        ("source_surface", &options.source_surface),
        ("channel_id", &options.channel_id),
        ("message_id", &options.message_id),
        ("author_id", &options.author_id),
    ] {
        validate_mention_text(label, value, 1, 240)?;
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
) -> Result<Value> {
    let rate = state.target_heartbeat_rate.max(0.001);
    let work_role = work_role_for_action(
        options.coordinator_action.as_deref(),
        options.target_role.as_deref(),
    );
    let (selected_index, selection_kind, selection_reason) =
        select_participant(state, work_role.as_deref(), options.urgency)?;
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
        personality_cooldown_multiplier: personality_cooldown_multiplier(&selected),
        mood_cooldown_multiplier: mood_cooldown_multiplier(&selected),
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
        let selected_role = selected.role_id.as_str();
        state
            .pending_mentions
            .retain(|mention| mention.target_role_id != selected_role);
    }
    if !options.defer_completion {
        complete_pending_turn(state, selected_index)?;
    }

    let selected_after = state.participants[selected_index].clone();
    let persona_memory_recall = persona_memory_recall_for_scheduled_turn(
        options.agent_store.as_deref(),
        &selected_after,
        &action,
        &selected_pending_mentions,
    );
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
            "personality_cooldown_multiplier": personality_cooldown_multiplier(&selected_after),
            "mood_cooldown_multiplier": mood_cooldown_multiplier(&selected_after),
            "initiative_heat_multiplier": initiative_heat_multiplier(&selected_after),
            "effective_cooldown_multiplier": effective_cooldown_multiplier(&selected_after),
            "initiative_cost": action.initiative_cost,
            "interruptibility": action.interruptibility,
            "commitment": action.commitment,
            "local_affordance_basis": action.local_affordance_basis,
            "pending_mentions": selected_pending_mentions,
            "persona_memory_recall": persona_memory_recall,
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
            "A selected idle lane may ruminate and request bounded self-memory mutation; it may not invent project work.",
            "When no coordinator work is active, idle rumination is slowed by the sleep multiplier and shaped by personality and mood cooldowns so the swarm dreams instead of thrashing."
        ],
    });

    let rumination = if event.action_type == "ruminate_memory" {
        serde_json::json!({
            "roleId": selected_after.role_id,
            "selfPatch": rumination_patch(&selected_after.role_id, &event.action_id),
            "result": null,
            "applied": false,
        })
    } else {
        Value::Null
    };
    Ok(serde_json::json!({
        "event": history_event_json(event),
        "schedule": schedule,
        "rumination": if rumination.is_null() { Value::Null } else { rumination },
    }))
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
    let action_id = format!("heartbeat.{}.ruminate", selected.role_id);
    HeartbeatAction {
        action_id,
        action_type: "ruminate_memory",
        action_scale: "short",
        base_recovery: state.pacing_policy.idle_base_recovery / heartbeat_rate,
        initiative_cost: 1.0,
        interruptibility: 0.9,
        commitment: 0.25,
        local_affordance_basis: vec![
            format!(
            "{} has no actionable lane work; ruminate on role quality and prepare candidate self-memory pressure.",
            selected.display_name
        ),
            "Heartbeat slots control opportunity, not project authority.".to_string(),
            "When no coordinator work is active, idle rumination is slowed by the sleep multiplier and shaped by personality and mood cooldowns so the swarm dreams instead of thrashing.".to_string(),
        ],
    }
}

fn persona_memory_recall_for_scheduled_turn(
    agent_store: Option<&Path>,
    selected: &HeartbeatParticipant,
    action: &HeartbeatAction,
    pending_mentions: &[HeartbeatPendingMention],
) -> Value {
    if action.action_type != "persona_turn" || selected.role_id != "Persona" {
        return Value::Null;
    }
    let Some(agent_store) = agent_store else {
        return serde_json::json!({
            "schemaVersion": crate::SEMANTIC_PROJECTION_SCHEMA_VERSION,
            "status": "unavailable",
            "cacheStatus": "agent-store-missing",
            "renderedRecall": "- semantic Persona memory recall unavailable: no agent store was provided for this heartbeat tick",
            "privateStateExposed": false,
        });
    };

    let swarm_identity = match crate::load_agent_memory_swarm_identity(agent_store) {
        Ok(Some(identity)) => identity,
        Ok(None) => {
            return serde_json::json!({
                "schemaVersion": crate::SEMANTIC_PROJECTION_SCHEMA_VERSION,
                "status": "unavailable",
                "cacheStatus": "swarm-identity-missing",
                "renderedRecall": "- semantic Persona memory recall unavailable: agent store has no immutable swarm identity",
                "privateStateExposed": false,
            });
        }
        Err(error) => {
            return serde_json::json!({
                "schemaVersion": crate::SEMANTIC_PROJECTION_SCHEMA_VERSION,
                "status": "unavailable",
                "cacheStatus": "swarm-identity-load-failed",
                "renderedRecall": format!("- semantic Persona memory recall unavailable: {}", compact_heartbeat_line(&format!("{error:#}"), 320)),
                "privateStateExposed": false,
            });
        }
    };

    let entry = match crate::agent_memory::load_agent_memory_entry_for_role(agent_store, "Persona")
    {
        Ok(Some(entry)) => entry,
        Ok(None) => {
            return serde_json::json!({
                "schemaVersion": crate::SEMANTIC_PROJECTION_SCHEMA_VERSION,
                "status": "unavailable",
                "cacheStatus": "persona-memory-missing",
                "renderedRecall": "- semantic Persona memory recall unavailable: Persona memory entry is missing",
                "privateStateExposed": false,
            });
        }
        Err(error) => {
            return serde_json::json!({
                "schemaVersion": crate::SEMANTIC_PROJECTION_SCHEMA_VERSION,
                "status": "unavailable",
                "cacheStatus": "persona-memory-load-failed",
                "renderedRecall": format!("- semantic Persona memory recall unavailable: {}", compact_heartbeat_line(&format!("{error:#}"), 320)),
                "privateStateExposed": false,
            });
        }
    };

    let query = persona_memory_recall_query(selected, pending_mentions);
    let mut entries = Vec::new();
    for role in crate::agent_memory_role_ids() {
        if let Ok(Some(role_entry)) = crate::load_agent_memory_entry_for_role(agent_store, role) {
            entries.push(role_entry);
        }
    }
    let swarm_id = swarm_identity.swarm_id;
    let graph = crate::memory_graph_from_agent_memories(&format!("{swarm_id}-mind"), &entries);
    let persona_domain_id =
        crate::memory_graph_domain_id(crate::EpiphanyMemoryProfile::RoleSelf, "role", "Persona");
    let query_document = EpiphanyMemoryContextQuery {
        id: "heartbeat-persona-turn".to_string(),
        profile: Some(crate::EpiphanyMemoryProfile::RoleSelf),
        domain_ids: vec![persona_domain_id],
        node_ids: Vec::new(),
        edge_ids: Vec::new(),
        text: Some(query.clone()),
        budget: Some(8),
    };
    let config = memory_semantic_config_for_heartbeat();
    let packet = crate::semantic_memory_context(
        &graph,
        &swarm_id,
        crate::SemanticPartition::Mind,
        &query_document,
        &config,
    );
    let fallback = packet
        .warnings
        .iter()
        .any(|warning| warning.contains("canonical BM25 fallback"));
    let rendered_recall = crate::render_persona_semantic_memory_recall(&packet);

    serde_json::json!({
        "schemaVersion": crate::SEMANTIC_PROJECTION_SCHEMA_VERSION,
        "status": if fallback { "fallback" } else { "ready" },
        "cacheStatus": if fallback { "canonical-bm25" } else { "shared-mind-qdrant" },
        "identityId": entry.agent.agent_id,
        "roleId": entry.role_id,
        "chunkCount": graph.nodes.len() + graph.summaries.len(),
        "hitCount": packet.nodes.len() + packet.summaries.len(),
        "renderedRecall": rendered_recall,
        "warnings": packet.warnings,
        "privateStateExposed": false,
    })
}

fn persona_memory_recall_query(
    selected: &HeartbeatParticipant,
    pending_mentions: &[HeartbeatPendingMention],
) -> String {
    let mention_text = pending_mentions
        .iter()
        .map(|mention| {
            format!(
                "{}: {}",
                mention.author_name.as_deref().unwrap_or(&mention.author_id),
                mention.visible_prompt
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{} current Persona turn\nPending addressed pressure:\n{}",
        selected.display_name,
        if mention_text.trim().is_empty() {
            "(none)"
        } else {
            mention_text.as_str()
        }
    )
}

fn memory_semantic_config_for_heartbeat() -> crate::MemorySemanticIndexConfig {
    let mut config = crate::MemorySemanticIndexConfig::from_env();
    config.qdrant_timeout_ms = config.qdrant_timeout_ms.min(1_000);
    config.ollama_timeout_ms = config.ollama_timeout_ms.min(1_000);
    config
}

fn compact_heartbeat_line(value: &str, max_len: usize) -> String {
    let mut compacted = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if compacted.len() > max_len {
        let keep = max_len.saturating_sub(3);
        compacted.truncate(keep);
        compacted.push_str("...");
    }
    compacted
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
        .cloned()
        .collect()
}

fn validate_mention_text(label: &str, value: &str, min_len: usize, max_len: usize) -> Result<()> {
    let trimmed = value.trim();
    if trimmed.len() < min_len || value.len() > max_len {
        return Err(anyhow!(
            "pending mention {label} must be between {min_len} and {max_len} characters"
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

fn rumination_patch(role_id: &str, action_id: &str) -> Value {
    let display_name = display_name_for_role(role_id);
    serde_json::json!({
        "agentId": agent_id_for_role(role_id),
        "reason": format!("{display_name} won an idle heartbeat slot and should preserve the habit of using idle wakeups to ruminate before sleep/review distills anything durable."),
        "semanticMemories": [{
            "memoryId": format!("mem-{role_id}-heartbeat-rumination"),
            "summary": "When a heartbeat wakes this lane and no coordinator-approved work is available, the correct move is to ruminate on role quality and surface candidate self-memory pressure rather than inventing project authority; durable distillation belongs in sleep or review.",
            "salience": 0.78,
            "confidence": 0.88,
        }],
        "goals": [{
            "goalId": format!("goal-{role_id}-heartbeat-rumination"),
            "description": "Use idle heartbeat slots to ruminate on this lane's own work and prepare bounded memory pressure before touching project state.",
            "scope": "life",
            "priority": 0.82,
            "emotionalStake": "An idle organ that invents work becomes noise in the bloodstream.",
            "status": "active",
        }],
        "privateNotes": [format!("Last idle heartbeat action `{action_id}` chose rumination over fake urgency.")],
    })
}

#[derive(Clone, Debug)]
struct RoleMemoryRecord {
    role_id: String,
    memory_id: String,
    memory_kind: String,
    summary: String,
    salience: f64,
    confidence: f64,
    tokens: BTreeSet<String>,
}

#[derive(Clone, Debug)]
struct RoleAppraisalProfile {
    role_id: String,
    agent_id: String,
    display_name: String,
    traits: Vec<PersonalityTraitProjection>,
    reactivity: f64,
    plasticity: f64,
    expressiveness: f64,
    guardedness: f64,
    values: Vec<String>,
}

#[derive(Clone, Debug)]
struct PersonalityTraitProjection {
    group: String,
    name: String,
    activation: f64,
    plasticity: f64,
    weight: f64,
}

fn collect_role_memory_records(agent_store: Option<&Path>) -> Result<Vec<RoleMemoryRecord>> {
    let Some(agent_store) = agent_store else {
        return Ok(Vec::new());
    };
    let mut records = Vec::new();
    for role_id in ROLE_ORDER {
        let Some(entry) =
            crate::agent_memory::load_agent_memory_entry_for_role(agent_store, role_id)?
        else {
            continue;
        };
        for memory in entry.agent.memories.semantic.iter() {
            let tokens = summary_tokens(&memory.summary);
            if tokens.is_empty() {
                continue;
            }
            records.push(RoleMemoryRecord {
                role_id: (*role_id).to_string(),
                memory_id: memory.memory_id.clone(),
                memory_kind: "semantic".to_string(),
                summary: memory.summary.clone(),
                salience: memory.salience,
                confidence: memory.confidence,
                tokens,
            });
        }
        for memory in &entry.agent.memories.episodic {
            let tokens = summary_tokens(&memory.summary);
            if tokens.is_empty() {
                continue;
            }
            records.push(RoleMemoryRecord {
                role_id: (*role_id).to_string(),
                memory_id: memory.memory_id.clone(),
                memory_kind: "episodic".to_string(),
                summary: memory.summary.clone(),
                salience: memory.salience,
                confidence: memory.confidence,
                tokens,
            });
        }
        for memory in &entry.agent.memories.relationship_summaries {
            let tokens = summary_tokens(&memory.summary);
            if tokens.is_empty() {
                continue;
            }
            records.push(RoleMemoryRecord {
                role_id: (*role_id).to_string(),
                memory_id: memory.memory_id.clone(),
                memory_kind: "relationship_summary".to_string(),
                summary: memory.summary.clone(),
                salience: memory.salience,
                confidence: memory.confidence,
                tokens,
            });
        }
    }
    records.sort_by(|left, right| {
        (right.salience * right.confidence)
            .total_cmp(&(left.salience * left.confidence))
            .then_with(|| left.role_id.cmp(&right.role_id))
            .then_with(|| left.memory_id.cmp(&right.memory_id))
    });
    records.truncate(48);
    Ok(records)
}

fn collect_role_appraisal_profiles(
    agent_store: Option<&Path>,
) -> Result<Vec<RoleAppraisalProfile>> {
    let Some(agent_store) = agent_store else {
        return Ok(Vec::new());
    };
    let mut profiles = Vec::new();
    for role_id in ROLE_ORDER {
        let Some(entry) =
            crate::agent_memory::load_agent_memory_entry_for_role(agent_store, role_id)?
        else {
            continue;
        };
        let mut traits = Vec::new();
        collect_trait_group(
            &mut traits,
            "underlying_organization",
            &entry.agent.canonical_state.underlying_organization,
        );
        collect_trait_group(
            &mut traits,
            "stable_dispositions",
            &entry.agent.canonical_state.stable_dispositions,
        );
        collect_trait_group(
            &mut traits,
            "behavioral_dimensions",
            &entry.agent.canonical_state.behavioral_dimensions,
        );
        collect_trait_group(
            &mut traits,
            "presentation_strategy",
            &entry.agent.canonical_state.presentation_strategy,
        );
        collect_trait_group(
            &mut traits,
            "voice_style",
            &entry.agent.canonical_state.voice_style,
        );
        collect_trait_group(
            &mut traits,
            "situational_state",
            &entry.agent.canonical_state.situational_state,
        );
        traits.sort_by(|left, right| right.weight.total_cmp(&left.weight));
        let top_weighted = traits.iter().take(8).collect::<Vec<_>>();
        let reactivity = average(top_weighted.iter().map(|item| item.activation)).unwrap_or(0.5);
        let plasticity = average(top_weighted.iter().map(|item| item.plasticity)).unwrap_or(0.5);
        let expressiveness = average(
            traits
                .iter()
                .filter(|item| item.group == "voice_style" || item.group == "presentation_strategy")
                .map(|item| item.activation),
        )
        .unwrap_or(reactivity);
        let guardedness = average(
            traits
                .iter()
                .filter(|item| {
                    let name = item.name.as_str();
                    name.contains("guard")
                        || name.contains("shame")
                        || name.contains("risk")
                        || name.contains("caution")
                        || name.contains("contingent")
                        || name.contains("defens")
                })
                .map(|item| item.activation),
        )
        .unwrap_or((1.0 - expressiveness * 0.45).clamp(0.0, 1.0));
        let values = entry
            .agent
            .canonical_state
            .values
            .iter()
            .take(5)
            .map(|value| value.label.clone())
            .collect();
        profiles.push(RoleAppraisalProfile {
            role_id: (*role_id).to_string(),
            agent_id: entry.agent.agent_id,
            display_name: entry.agent.identity.name,
            traits,
            reactivity: round3(reactivity),
            plasticity: round3(plasticity),
            expressiveness: round3(expressiveness),
            guardedness: round3(guardedness),
            values,
        });
    }
    Ok(profiles)
}

fn collect_trait_group(
    traits: &mut Vec<PersonalityTraitProjection>,
    group: &str,
    source: &BTreeMap<String, crate::agent_memory::GhostlightTraitVector>,
) {
    for (name, vector) in source {
        let activation = vector.current_activation.clamp(0.0, 1.0);
        let plasticity = vector.plasticity.clamp(0.0, 1.0);
        traits.push(PersonalityTraitProjection {
            group: group.to_string(),
            name: name.clone(),
            activation,
            plasticity,
            weight: round3(activation * (0.65 + plasticity * 0.35)),
        });
    }
}

fn apply_personality_timing_profiles(
    state: &mut EpiphanyHeartbeatStateEntry,
    profiles: &[RoleAppraisalProfile],
) {
    for participant in &mut state.participants {
        let Some(profile) = profiles
            .iter()
            .find(|profile| profile.role_id == participant.role_id)
        else {
            continue;
        };
        let timing = personality_timing_for_profile(profile);
        participant.personality_cooldown_multiplier = timing.cooldown_multiplier;
        participant.personality_timing = Some(HeartbeatPersonalityTiming {
            schema_version: "epiphany.personality_timing.v0".to_string(),
            source: "state/agents.msgpack".to_string(),
            cooldown_multiplier: timing.cooldown_multiplier,
            work_drive: timing.work_drive,
            handsiness: timing.handsiness,
            caution: timing.caution,
            rumination_bias: timing.rumination_bias,
            basis: timing.basis,
            contract: "Cooldown is personality-shaped. Lower multipliers recover faster; higher multipliers yield the floor to other lanes.".to_string(),
        });
    }
}

#[derive(Clone, Debug)]
struct PersonalityTiming {
    cooldown_multiplier: f64,
    work_drive: f64,
    handsiness: f64,
    caution: f64,
    rumination_bias: f64,
    basis: Vec<String>,
}

fn personality_timing_for_profile(profile: &RoleAppraisalProfile) -> PersonalityTiming {
    let work_drive = trait_match_score(
        &profile.traits,
        &[
            "objective",
            "diff",
            "implementation",
            "hands",
            "bloodhound",
            "craft",
            "act",
            "source_touch",
            "work",
            "task",
        ],
    );
    let handsiness = trait_match_score(
        &profile.traits,
        &[
            "hands",
            "implementation",
            "diff_truth",
            "objective_pursuit",
            "bloodhound_pressure",
            "source_touch_precision",
            "small_reviewable_cut",
            "craft",
        ],
    );
    let caution = trait_match_score(
        &profile.traits,
        &[
            "review",
            "gate",
            "caution",
            "guard",
            "risk",
            "verification",
            "truth",
            "falsification",
            "routing",
            "discipline",
        ],
    );
    let rumination_bias = trait_match_score(
        &profile.traits,
        &[
            "dream",
            "memory",
            "reflection",
            "imagination",
            "future",
            "translation",
            "watch",
            "continuity",
            "self",
        ],
    );
    let cooldown = (1.05 + profile.guardedness * 0.28 + caution * 0.24 + rumination_bias * 0.08
        - profile.reactivity * 0.16
        - profile.plasticity * 0.10
        - work_drive * 0.18
        - handsiness * 0.34)
        .clamp(0.55, 1.55);
    let mut basis = Vec::new();
    for item in profile.traits.iter().take(6) {
        basis.push(format!(
            "{}.{} activation {:.2} plasticity {:.2}",
            item.group, item.name, item.activation, item.plasticity
        ));
    }
    PersonalityTiming {
        cooldown_multiplier: round3(cooldown),
        work_drive: round3(work_drive),
        handsiness: round3(handsiness),
        caution: round3(caution),
        rumination_bias: round3(rumination_bias),
        basis,
    }
}

fn trait_match_score(traits: &[PersonalityTraitProjection], needles: &[&str]) -> f64 {
    let mut score = 0.0_f64;
    let mut weight = 0.0_f64;
    for item in traits {
        let haystack = format!("{} {}", item.group, item.name);
        let matched = needles
            .iter()
            .any(|needle| haystack.contains(&needle.replace('_', " ")));
        let matched = matched || needles.iter().any(|needle| haystack.contains(*needle));
        if matched {
            let item_weight = 0.7 + item.plasticity * 0.3;
            score += item.activation * item_weight;
            weight += item_weight;
        }
    }
    if weight <= f64::EPSILON {
        0.0
    } else {
        (score / weight).clamp(0.0, 1.0)
    }
}

fn apply_mood_timing_from_appraisals(
    state: &mut EpiphanyHeartbeatStateEntry,
    appraisals: &HeartbeatAgentThoughtAppraisals,
) {
    for participant in &mut state.participants {
        let Some(appraisal) = appraisals
            .participant_appraisals
            .iter()
            .find(|item| item.role_id == participant.role_id)
        else {
            continue;
        };
        let urgency = appraisal.emotional_appraisal.urgency.clamp(0.0, 1.0);
        let arousal = appraisal.emotional_appraisal.arousal.clamp(0.0, 1.0);
        let thought_pressure = appraisal
            .emotional_appraisal
            .thought_pressure
            .clamp(0.0, 1.0);
        let guardedness = appraisal.emotional_appraisal.guardedness.clamp(0.0, 1.0);
        let reaction_intensity = appraisal
            .candidate_implications
            .reaction_intensity
            .clamp(0.0, 1.0);
        let anxiety = (urgency * 0.32
            + arousal * 0.22
            + thought_pressure * 0.24
            + guardedness * 0.12
            + reaction_intensity * 0.10)
            .clamp(0.0, 1.0);
        let multiplier =
            (1.10 - urgency * 0.24 - anxiety * 0.38 - reaction_intensity * 0.12).clamp(0.55, 1.25);
        let emotional_dimensions =
            heartbeat_emotional_dimensions(appraisal, anxiety, reaction_intensity);
        participant.mood_cooldown_multiplier = round3(multiplier);
        participant.mood_timing = Some(HeartbeatMoodTiming {
            schema_version: "epiphany.mood_timing.v0".to_string(),
            source: Some(appraisal.appraisal_id.clone()),
            cooldown_multiplier: round3(multiplier),
            emotional_dimensions,
            anxiety: round3(anxiety),
            urgency: round3(urgency),
            arousal: round3(arousal),
            thought_pressure: round3(thought_pressure),
            guardedness: round3(guardedness),
            reaction_intensity: round3(reaction_intensity),
            contract: "Mood bends personality timing. Anxiety and urgency lower cooldown so the lane that needs the floor most gets it sooner.".to_string(),
        });
    }
}

fn heartbeat_emotional_dimensions(
    appraisal: &HeartbeatAgentThoughtAppraisal,
    anxiety: f64,
    reaction_intensity: f64,
) -> Vec<HeartbeatMoodDimension> {
    let emotional = &appraisal.emotional_appraisal;
    let dimensions = [
        ("valence", emotional.valence),
        ("arousal", emotional.arousal),
        (
            "dominance",
            projected_emotion(appraisal, &["dominance", "command_force"]),
        ),
        ("urgency", emotional.urgency),
        (
            "anger",
            projected_emotion(appraisal, &["anger", "rage", "fury"]),
        ),
        (
            "despair",
            projected_emotion(appraisal, &["despair", "hopelessness"]),
        ),
        (
            "sadness",
            projected_emotion(appraisal, &["sadness", "grief", "sorrow"]),
        ),
        (
            "fear",
            projected_emotion(appraisal, &["fear", "terror"]).max(anxiety),
        ),
        ("anxiety", anxiety),
        (
            "disgust",
            projected_emotion(appraisal, &["disgust", "revulsion"]),
        ),
        ("contempt", projected_emotion(appraisal, &["contempt"])),
        (
            "annoyance",
            projected_emotion(appraisal, &["annoyance", "irritation"]),
        ),
        (
            "dismissal",
            projected_emotion(appraisal, &["dismissal", "dismissiveness"]),
        ),
        (
            "flippancy",
            projected_emotion(appraisal, &["flippancy", "playfulness"]),
        ),
        (
            "playfulness",
            projected_emotion(appraisal, &["playfulness", "play"]),
        ),
        ("irony", projected_emotion(appraisal, &["irony", "dryness"])),
        ("tenderness", projected_emotion(appraisal, &["tenderness"])),
        ("warmth", projected_emotion(appraisal, &["warmth"])),
        ("joy", projected_emotion(appraisal, &["joy", "delight"])),
        (
            "excitement",
            projected_emotion(appraisal, &["excitement"])
                .max(emotional.arousal * emotional.curiosity),
        ),
        (
            "fatigue",
            projected_emotion(appraisal, &["fatigue", "exhaustion"]),
        ),
        ("guardedness", emotional.guardedness),
        ("confidence", projected_emotion(appraisal, &["confidence"])),
        ("shame", projected_emotion(appraisal, &["shame"])),
        ("pride", projected_emotion(appraisal, &["pride"])),
        (
            "threat",
            projected_emotion(appraisal, &["threat", "menace"]),
        ),
        (
            "secrecy",
            projected_emotion(appraisal, &["secrecy", "low_projection"]),
        ),
        ("hesitation", projected_emotion(appraisal, &["hesitation"])),
        (
            "emotionalContainment",
            projected_emotion(appraisal, &["emotional_containment", "containment"]),
        ),
        ("thoughtPressure", emotional.thought_pressure),
        ("reactionIntensity", reaction_intensity),
        (
            "commandForce",
            projected_emotion(appraisal, &["command_force"]),
        ),
    ];
    dimensions
        .into_iter()
        .map(|(name, value)| HeartbeatMoodDimension {
            name: name.to_string(),
            value: round3(value.clamp(0.0, 1.0)),
            source_path: format!("heartbeat.appraisal.{}.{}", appraisal.appraisal_id, name),
        })
        .collect()
}

fn projected_emotion(appraisal: &HeartbeatAgentThoughtAppraisal, needles: &[&str]) -> f64 {
    appraisal
        .personality_projection
        .iter()
        .filter(|projection| {
            let name = projection.name.to_ascii_lowercase();
            needles.iter().any(|needle| {
                let spaced = needle.replace('_', " ");
                name.contains(*needle) || name.contains(&spaced)
            })
        })
        .map(|projection| projection.projection)
        .fold(0.0_f64, f64::max)
}

fn build_memory_resonance(records: &[RoleMemoryRecord]) -> HeartbeatMemoryResonance {
    let mut pairs = Vec::new();
    for (left_index, left) in records.iter().enumerate() {
        for right in records.iter().skip(left_index + 1) {
            if left.role_id == right.role_id {
                continue;
            }
            let overlap = token_overlap(&left.tokens, &right.tokens);
            if overlap <= 0.0 {
                continue;
            }
            let strength = round3(
                overlap * ((left.salience * left.confidence) + (right.salience * right.confidence))
                    / 2.0,
            );
            if strength < 0.08 {
                continue;
            }
            pairs.push(HeartbeatMemoryResonancePair {
                left_role: left.role_id.clone(),
                left_memory_id: left.memory_id.clone(),
                left_memory_kind: left.memory_kind.clone(),
                left_summary: left.summary.clone(),
                right_role: right.role_id.clone(),
                right_memory_id: right.memory_id.clone(),
                right_memory_kind: right.memory_kind.clone(),
                right_summary: right.summary.clone(),
                strength,
                shared_tokens: shared_tokens(&left.tokens, &right.tokens),
                source_roles: vec![left.role_id.clone(), right.role_id.clone()],
                source_kinds: vec![left.memory_kind.clone(), right.memory_kind.clone()],
                evidence_refs: vec![left.memory_id.clone(), right.memory_id.clone()],
            });
        }
    }
    pairs.sort_by(|left, right| right.strength.total_cmp(&left.strength));
    pairs.truncate(8);
    HeartbeatMemoryResonance {
        schema_version: "epiphany.memory_resonance.v0".to_string(),
        updated_at: now_iso(),
        source: "epiphany-native-void-routine".to_string(),
        record_count: records.len(),
        pairs,
    }
}

fn build_incubation(
    previous: &Option<HeartbeatIncubation>,
    bridge: &Option<HeartbeatCognitionBridge>,
    candidate_interventions: &Option<HeartbeatCandidateInterventions>,
    resonance: &HeartbeatMemoryResonance,
    records: &[RoleMemoryRecord],
) -> HeartbeatIncubation {
    let previous_themes = previous
        .as_ref()
        .map(|value| value.themes.clone())
        .unwrap_or_default();
    let source_coverage = build_source_coverage(records);
    let mut themes = Vec::new();
    for pair in resonance.pairs.iter().take(6) {
        let left_role = pair.left_role.as_str();
        let right_role = pair.right_role.as_str();
        let tokens = pair
            .shared_tokens
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("/");
        let source_roles = pair.source_roles.clone();
        let source_kinds = pair.source_kinds.clone();
        let source_memory_ids = pair.evidence_refs.clone();
        let summary = format!(
            "{} and {} keep touching {}; let the swarm decide whether that is signal, stale echo, or a branch worth following.",
            display_name_for_role(left_role),
            display_name_for_role(right_role),
            if tokens.is_empty() {
                "an unnamed seam"
            } else {
                &tokens
            }
        );
        let theme_id = format!(
            "theme-{left_role}-{right_role}-{}",
            stable_theme_suffix(&format!(
                "{tokens}-{}-{}",
                source_kinds.join("-"),
                source_memory_ids.join("-")
            ))
        );
        let previous_theme =
            best_matching_theme(&previous_themes, &theme_id, &summary, &source_memory_ids);
        let support_count = previous_support_count(previous_theme) + 1;
        let novelty_to_self = novelty_to_self(
            &theme_id,
            &summary,
            &source_memory_ids,
            &previous_themes,
            bridge,
        );
        let novelty_to_room = novelty_to_room(&theme_id, &summary, candidate_interventions, bridge);
        let saturation = saturation_metrics(
            &theme_id,
            &summary,
            &source_memory_ids,
            &previous_themes,
            bridge,
            support_count,
        );
        let refractory_penalty = refractory_penalty(&theme_id, &summary, bridge);
        let evidence_diversity =
            evidence_diversity(&source_roles, &source_kinds, &source_memory_ids);
        let exploration_bonus = exploration_bonus(&source_roles, &source_kinds, &source_coverage);
        let quiet_signal_ratio = 0.0;
        let prior_maturation = previous_theme.map(|theme| theme.maturation).unwrap_or(0.32);
        let strength = pair.strength;
        let maturation = round3(
            (prior_maturation * 0.44
                + strength * 0.24
                + evidence_diversity * 0.14
                + exploration_bonus * 0.12
                + novelty_to_self * 0.06
                + novelty_to_room * 0.06
                + (support_count as f64 / 10.0).min(1.0) * 0.08
                - saturation.score * 0.18
                - refractory_penalty * 0.14
                - quiet_signal_ratio * 0.12)
                .clamp(0.0, 1.0),
        );
        let novelty = round3(
            (novelty_to_self * 0.55 + novelty_to_room * 0.45 - quiet_signal_ratio * 0.1)
                .clamp(0.0, 1.0),
        );
        let desire_to_speak = round3(
            (strength * 0.18
                + maturation * 0.18
                + novelty_to_room * 0.18
                + novelty_to_self * 0.10
                + evidence_diversity * 0.12
                + exploration_bonus * 0.10
                - saturation.score * 0.24
                - refractory_penalty * 0.16)
                .clamp(0.0, 1.0),
        );
        let status = theme_status(
            novelty_to_self,
            novelty_to_room,
            saturation.score,
            refractory_penalty,
            support_count,
            evidence_diversity,
            desire_to_speak,
            maturation,
        );
        let priority_score = round3(
            (desire_to_speak * 0.34
                + novelty_to_self * 0.20
                + novelty_to_room * 0.18
                + evidence_diversity * 0.12
                + exploration_bonus * 0.16
                - saturation.score * 0.18
                - refractory_penalty * 0.14)
                .clamp(0.0, 1.0),
        );
        themes.push(HeartbeatIncubationTheme {
            theme_id: previous_theme
                .map(|theme| theme.theme_id.clone())
                .unwrap_or(theme_id),
            summary,
            strength: round3(strength),
            source: "memory_resonance".to_string(),
            source_roles: source_roles.clone(),
            source_kinds: source_kinds.clone(),
            source_memory_ids,
            support_count,
            evidence_diversity: round3(evidence_diversity),
            exploration_bonus: round3(exploration_bonus),
            novelty,
            novelty_to_self: round3(novelty_to_self),
            novelty_to_room: round3(novelty_to_room),
            maturation,
            desire_to_speak,
            saturation_score: round3(saturation.score),
            recent_match_count: saturation.recent_match_count,
            refractory_penalty: round3(refractory_penalty),
            priority_score,
            status: status.to_string(),
            latent_question: build_incubation_question(&source_roles, &source_kinds).to_string(),
            why_it_pulls: build_incubation_attraction(
                &source_roles,
                &source_kinds,
                novelty_to_self,
                evidence_diversity,
            )
            .to_string(),
            holding_close_because: build_incubation_holding_line(
                status,
                saturation.score,
                novelty_to_self,
                exploration_bonus,
            )
            .to_string(),
            updated_at: now_iso(),
        });
    }
    if themes.is_empty() && !records.is_empty() {
        let strongest = &records[0];
        themes.push(HeartbeatIncubationTheme {
            theme_id: format!("theme-{}-strongest-memory", strongest.role_id),
            summary: format!(
                "{} carries the hottest current memory: {}",
                display_name_for_role(&strongest.role_id),
                strongest.summary
            ),
            strength: round3(strongest.salience * strongest.confidence),
            source: "strongest_memory".to_string(),
            source_roles: vec![strongest.role_id.clone()],
            source_kinds: vec![strongest.memory_kind.clone()],
            source_memory_ids: vec![strongest.memory_id.clone()],
            support_count: 1,
            evidence_diversity: 0.28,
            exploration_bonus: 0.18,
            novelty: 0.62,
            novelty_to_self: 0.62,
            novelty_to_room: 0.58,
            maturation: round3((strongest.salience * strongest.confidence).clamp(0.18, 0.72)),
            desire_to_speak: round3(
                (strongest.salience * strongest.confidence * 0.6).clamp(0.12, 0.55),
            ),
            saturation_score: 0.0,
            recent_match_count: 0,
            refractory_penalty: 0.0,
            priority_score: round3(
                (strongest.salience * strongest.confidence * 0.7).clamp(0.14, 0.7),
            ),
            status: "incubating".to_string(),
            latent_question:
                "Does this hot memory deserve real follow-up, or is it just the loudest ember in the tray?"
                    .to_string(),
            why_it_pulls:
                "One strong memory is enough to seed a thought, but not enough to rule the room by default."
                    .to_string(),
            holding_close_because:
                "This is a seed, not a verdict. Give it one more pass before it starts issuing prophecies."
                    .to_string(),
            updated_at: now_iso(),
        });
    }
    themes.sort_by(|left, right| {
        right
            .priority_score
            .total_cmp(&left.priority_score)
            .then_with(|| right.strength.total_cmp(&left.strength))
    });
    themes.truncate(12);
    let last_incubation_summary = themes
        .first()
        .map(|theme| {
            format!(
                "Strongest incubating seam: {} ({}, self={:.2}, room={:.2}, speak={:.2}).",
                theme.theme_id,
                theme.status,
                theme.novelty_to_self,
                theme.novelty_to_room,
                theme.desire_to_speak,
            )
        })
        .unwrap_or_else(|| {
            "No incubating thought currently has enough connective tissue to justify special treatment."
                .to_string()
        });
    HeartbeatIncubation {
        schema_version: "epiphany.incubation.v0".to_string(),
        updated_at: now_iso(),
        source_coverage,
        last_incubation_summary,
        themes,
    }
}

fn build_thought_lanes(
    resonance: &HeartbeatMemoryResonance,
    incubation: &HeartbeatIncubation,
    records: &[RoleMemoryRecord],
) -> HeartbeatThoughtLanes {
    let analytic_threads = resonance
        .pairs
        .iter()
        .take(4)
        .enumerate()
        .map(|(index, pair)| {
            let left_role = pair.left_role.as_str();
            let right_role = pair.right_role.as_str();
            let strength = pair.strength;
            HeartbeatAnalyticThread {
                thread_id: format!("analytic-{left_role}-{right_role}-{index}"),
                topic: format!("{left_role}/{right_role} evidence seam"),
                claim: format!(
                    "{} and {} share a recurring memory edge; inspect whether this changes lane routing or evidence expectations.",
                    display_name_for_role(left_role),
                    display_name_for_role(right_role)
                ),
                evidence_refs: vec![pair.left_memory_id.clone(), pair.right_memory_id.clone()],
                salience: round3(strength),
                confidence: round3((0.55 + strength).min(0.95)),
                desire_to_act: round3((strength * 1.4).min(0.85)),
                counterweight:
                    "Shared vocabulary is not proof of shared truth; verify against artifacts before changing project state."
                        .to_string(),
                last_touched_at: now_iso(),
            }
        })
        .collect::<Vec<_>>();

    let associative_threads = incubation
        .themes
        .iter()
        .take(4)
        .enumerate()
        .map(|(index, theme)| {
            let topic = theme.theme_id.as_str();
            HeartbeatAssociativeThread {
                thread_id: format!("associative-{topic}-{index}"),
                topic: topic.to_string(),
                claim: theme.summary.clone(),
                source_theme_id: topic.to_string(),
                novelty: theme.novelty,
                room_relevance: theme.novelty_to_room,
                desire_to_speak: theme.desire_to_speak,
                status: theme.status.clone(),
                counterweight:
                    "A theme that keeps returning may be signal, obsession, or stale echo; the bridge must decide."
                        .to_string(),
                last_touched_at: now_iso(),
            }
        })
        .collect::<Vec<_>>();

    let seed_threads = if analytic_threads.is_empty() && associative_threads.is_empty() {
        records
            .first()
            .map(|record| {
                vec![HeartbeatAnalyticThread {
                    thread_id: format!("analytic-{}-seed", record.role_id),
                    topic: format!("{} strongest memory", record.role_id),
                    claim: record.summary.clone(),
                    evidence_refs: vec![record.memory_id.clone()],
                    salience: round3(record.salience),
                    confidence: round3(record.confidence),
                    desire_to_act: 0.2,
                    counterweight:
                        "One hot memory is only a seed; do not let it annex the whole mind."
                            .to_string(),
                    last_touched_at: now_iso(),
                }]
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    HeartbeatThoughtLanes {
        schema_version: "epiphany.cognition_lanes.v0".to_string(),
        updated_at: now_iso(),
        analytic: HeartbeatAnalyticLane {
            description:
                "Literal, evidence-facing lane: what is happening, what constraints matter, what action is justified."
                    .to_string(),
            active_threads: if analytic_threads.is_empty() {
                seed_threads
            } else {
                analytic_threads
            },
        },
        associative: HeartbeatAssociativeLane {
            description:
                "Pattern-facing lane: what this rhymes with, what seam is ripening, what surprising branch may be worth a later retrieval hop."
                    .to_string(),
            active_threads: associative_threads,
        },
    }
}

fn build_thought_bridge(
    previous: &Option<HeartbeatCognitionBridge>,
    thought_lanes: &HeartbeatThoughtLanes,
    resonance: &HeartbeatMemoryResonance,
    incubation: &HeartbeatIncubation,
) -> HeartbeatCognitionBridge {
    let analytic_count = thought_lanes.analytic.active_threads.len();
    let associative_count = thought_lanes.associative.active_threads.len();
    let resonance_count = resonance.pairs.len();
    let strongest_theme = incubation.themes.first().cloned();
    let strongest_status = strongest_theme
        .as_ref()
        .map(|theme| theme.status.as_str())
        .unwrap_or("incubating");
    let strongest_novelty_to_self = strongest_theme
        .as_ref()
        .map(|theme| theme.novelty_to_self)
        .unwrap_or(0.0);
    let strongest_novelty_to_room = strongest_theme
        .as_ref()
        .map(|theme| theme.novelty_to_room)
        .unwrap_or(0.0);
    let strongest_saturation = strongest_theme
        .as_ref()
        .map(|theme| theme.saturation_score)
        .unwrap_or(0.0);
    let strongest_refractory = strongest_theme
        .as_ref()
        .map(|theme| theme.refractory_penalty)
        .unwrap_or(0.0);
    let lane_balance = match analytic_count.cmp(&associative_count) {
        std::cmp::Ordering::Greater => "analytic-heavy",
        std::cmp::Ordering::Less => "associative-heavy",
        std::cmp::Ordering::Equal => "balanced",
    };
    let speak_decision = if strongest_theme.is_none() && resonance_count == 0 {
        "silence"
    } else if strongest_status == "ripe"
        && strongest_novelty_to_room >= 0.58
        && strongest_saturation < 0.56
        && strongest_refractory < 0.18
    {
        "draft"
    } else if matches!(strongest_status, "stalled" | "refractory") {
        "silence"
    } else if resonance_count > 0 {
        "hold"
    } else {
        "silence"
    };
    let mut syntheses = previous
        .as_ref()
        .map(|value| value.recent_syntheses.clone())
        .unwrap_or_default();
    syntheses.push(HeartbeatBridgeSynthesis {
        timestamp: now_iso(),
        summary: if let Some(theme) = &strongest_theme {
            bridge_summary(theme, speak_decision)
        } else {
            "No strong convergence yet; hold the lanes open without forcing speech.".to_string()
        },
        dominant_topics: strongest_theme
            .as_ref()
            .map(|theme| vec![theme.theme_id.clone()])
            .unwrap_or_default(),
        lane_balance: lane_balance.to_string(),
        speak_decision: speak_decision.to_string(),
        theme_status: strongest_status.to_string(),
        novelty_to_self: round3(strongest_novelty_to_self),
        novelty_to_room: round3(strongest_novelty_to_room),
        saturation_score: round3(strongest_saturation),
        saturation_note: synthesis_saturation_note(
            strongest_status,
            strongest_saturation,
            strongest_novelty_to_self,
        )
        .unwrap_or("No saturation warning.")
        .to_string(),
    });
    syntheses.reverse();
    syntheses.truncate(8);
    syntheses.reverse();
    let source_coverage = incubation.source_coverage.clone();
    let topic_saturation =
        topic_saturation_from_syntheses_and_themes(&syntheses, &incubation.themes);
    let refractory_topics = refractory_topics_from_themes(previous, &incubation.themes);

    HeartbeatCognitionBridge {
        schema_version: "epiphany.cognition_bridge.v0".to_string(),
        updated_at: now_iso(),
        recent_syntheses: syntheses,
        source_coverage,
        topic_saturation,
        refractory_topics,
        unresolved_tensions: vec![HeartbeatBridgeTension {
            topic: "thought authority boundary".to_string(),
            summary:
                "Cognition lanes may shape attention and drafts, but only reviewed Epiphany state surfaces change project truth."
                    .to_string(),
            opened_at: now_iso(),
        }],
        decision: HeartbeatBridgeDecision {
            lane_balance: lane_balance.to_string(),
            speak_decision: speak_decision.to_string(),
            reason: bridge_decision_reason(
                strongest_status,
                strongest_novelty_to_self,
                strongest_novelty_to_room,
                strongest_saturation,
                strongest_refractory,
                speak_decision,
            )
            .to_string(),
        },
    }
}

fn build_candidate_interventions(
    bridge: &HeartbeatCognitionBridge,
    incubation: &HeartbeatIncubation,
) -> HeartbeatCandidateInterventions {
    let decision = bridge.decision.speak_decision.as_str();
    let strongest_theme = incubation.themes.first();
    let strongest_status = strongest_theme
        .map(|theme| theme.status.as_str())
        .unwrap_or("incubating");
    let items = if decision == "draft" && strongest_status == "ripe" {
        strongest_theme
            .map(|theme| {
                let theme_id = theme.theme_id.as_str();
                vec![HeartbeatCandidateIntervention {
                    intervention_id: format!("candidate-{theme_id}"),
                    summary: "Possible Aquarium-facing thought-weather note".to_string(),
                    draft: format!("I keep seeing {} rhyme across the swarm; this one finally has enough blood to inspect in the open.", theme_id),
                    decision: decision.to_string(),
                    requires_persona: true,
                    requires_review: true,
                    novelty_to_room: theme.novelty_to_room,
                    saturation_score: theme.saturation_score,
                    created_at: now_iso(),
                }]
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    HeartbeatCandidateInterventions {
        schema_version: "epiphany.candidate_interventions.v0".to_string(),
        updated_at: now_iso(),
        items,
    }
}

fn build_agent_appraisals(
    profiles: &[RoleAppraisalProfile],
    thought_lanes: &HeartbeatThoughtLanes,
    incubation: &HeartbeatIncubation,
    bridge: &HeartbeatCognitionBridge,
) -> HeartbeatAgentThoughtAppraisals {
    let thought_tokens = cognition_tokens(thought_lanes, incubation, bridge);
    let thought_pressure = thought_pressure(thought_lanes, incubation);
    let bridge_decision = bridge.decision.speak_decision.as_str();
    let focus = incubation
        .themes
        .first()
        .map(|theme| theme.theme_id.as_str())
        .unwrap_or("no-active-theme");
    let appraisals = profiles
        .iter()
        .map(|profile| {
            let projection = personality_projection(profile, &thought_tokens);
            let alignment = projection
                .iter()
                .map(|item| item.projection)
                .next()
                .unwrap_or(profile.reactivity);
            let arousal = round3(
                (thought_pressure * (0.35 + profile.reactivity * 0.45 + profile.plasticity * 0.2))
                    .clamp(0.0, 1.0),
            );
            let guardedness = round3(
                (profile.guardedness * 0.65 + thought_pressure * 0.25 + (1.0 - alignment) * 0.1)
                    .clamp(0.0, 1.0),
            );
            let curiosity = round3(
                ((1.0 - profile.guardedness) * 0.25 + alignment * 0.45 + profile.plasticity * 0.3)
                    .clamp(0.0, 1.0),
            );
            let urgency = round3((arousal * 0.65 + guardedness * 0.25 + thought_pressure * 0.1).clamp(0.0, 1.0));
            let valence = round3((0.55 + curiosity * 0.2 - guardedness * 0.18).clamp(0.0, 1.0));
            let label = interpretation_label(bridge_decision, arousal, guardedness, curiosity);
            HeartbeatAgentThoughtAppraisal {
                schema_version: "epiphany.agent_thought_appraisal.v0".to_string(),
                appraisal_id: format!("appraisal-{}-{}", profile.role_id, stable_theme_suffix(focus)),
                review_status: "generated_unreviewed".to_string(),
                participant_agent_id: profile.agent_id.clone(),
                role_id: profile.role_id.clone(),
                current_character_state_ref: format!("state/agents.msgpack#{}", profile.role_id),
                thought_cluster_ref: focus.to_string(),
                participant_local_context: HeartbeatParticipantLocalContext {
                    display_name: profile.display_name.clone(),
                    values: profile.values.clone(),
                    reactivity: profile.reactivity,
                    plasticity: profile.plasticity,
                    expressiveness: profile.expressiveness,
                    guardedness: profile.guardedness,
                },
                observable_thought_summary: strongest_thought_summary(thought_lanes, incubation),
                personality_projection: projection,
                interpretation: format!("{} appraises {} through its current personality vector; reaction should follow this appraisal rather than a global mood knob.", profile.display_name, focus),
                emotional_appraisal: HeartbeatEmotionalAppraisal {
                    valence,
                    arousal,
                    urgency,
                    curiosity,
                    guardedness,
                    thought_pressure: round3(thought_pressure),
                },
                interpretation_label: label.to_string(),
                confidence_notes: "Deterministic first-pass appraisal from typed role personality vectors and clustered thought state; useful as reaction guidance, not reviewed truth.".to_string(),
                candidate_implications: HeartbeatCandidateImplications {
                    reaction_mode: reaction_mode(&label, bridge_decision).to_string(),
                    reaction_intensity: round3((urgency * 0.55 + arousal * 0.3 + curiosity * 0.15).clamp(0.0, 1.0)),
                    should_speak: bridge_decision == "draft" && profile.role_id == "Persona" && guardedness < 0.75,
                    should_incubate: bridge_decision != "silence" && guardedness >= 0.55,
                },
                review: HeartbeatAppraisalReview {
                    accepted_for_mutation: false,
                    rationale: "Appraisal may steer reaction and display; state mutation still requires the explicit selfPatch or project-state review path.".to_string(),
                },
            }
        })
        .collect::<Vec<_>>();
    HeartbeatAgentThoughtAppraisals {
        schema_version: "epiphany.agent_thought_appraisals.v0".to_string(),
        updated_at: now_iso(),
        thought_cluster_ref: focus.to_string(),
        participant_appraisals: appraisals,
    }
}

fn build_agent_reactions(
    appraisals: &HeartbeatAgentThoughtAppraisals,
    bridge: &HeartbeatCognitionBridge,
) -> HeartbeatAgentReactions {
    let bridge_decision = bridge.decision.speak_decision.as_str();
    let reactions = appraisals
        .participant_appraisals
        .iter()
        .map(|appraisal| {
            let role_id = appraisal.role_id.as_str();
            let arousal = appraisal.emotional_appraisal.arousal;
            let guardedness = appraisal.emotional_appraisal.guardedness;
            let curiosity = appraisal.emotional_appraisal.curiosity;
            let mode = appraisal.candidate_implications.reaction_mode.as_str();
            let intensity = appraisal.candidate_implications.reaction_intensity;
            HeartbeatAgentReaction {
                reaction_id: format!("reaction-{}-{}", role_id, now_stamp()),
                role_id: role_id.to_string(),
                participant_agent_id: appraisal.participant_agent_id.clone(),
                appraisal_id: appraisal.appraisal_id.clone(),
                mode: mode.to_string(),
                mood_label: mood_label(arousal, guardedness, curiosity).to_string(),
                intensity: round3(intensity),
                bridge_decision: bridge_decision.to_string(),
                surface: if role_id == "Persona" {
                    "aquarium"
                } else {
                    "internal"
                }
                .to_string(),
                recommended_use: reaction_recommended_use(role_id, mode).to_string(),
            }
        })
        .collect::<Vec<_>>();
    HeartbeatAgentReactions {
        schema_version: "epiphany.agent_reactions.v0".to_string(),
        updated_at: now_iso(),
        reactions,
        contract: "Reaction is appraisal output. It may pace, color, draft, or display behavior; it does not mutate state without review.".to_string(),
    }
}

fn cognition_tokens(
    thought_lanes: &HeartbeatThoughtLanes,
    incubation: &HeartbeatIncubation,
    bridge: &HeartbeatCognitionBridge,
) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    for thread in &thought_lanes.analytic.active_threads {
        tokens.extend(summary_tokens(&thread.topic));
        tokens.extend(summary_tokens(&thread.claim));
        tokens.extend(summary_tokens(&thread.counterweight));
    }
    for thread in &thought_lanes.associative.active_threads {
        tokens.extend(summary_tokens(&thread.topic));
        tokens.extend(summary_tokens(&thread.claim));
        tokens.extend(summary_tokens(&thread.counterweight));
    }
    for theme in &incubation.themes {
        tokens.extend(summary_tokens(&theme.theme_id));
        tokens.extend(summary_tokens(&theme.summary));
        tokens.extend(summary_tokens(&theme.latent_question));
        tokens.extend(summary_tokens(&theme.why_it_pulls));
        tokens.extend(summary_tokens(&theme.holding_close_because));
    }
    for synthesis in &bridge.recent_syntheses {
        tokens.extend(summary_tokens(&synthesis.summary));
        for topic in &synthesis.dominant_topics {
            tokens.extend(summary_tokens(topic));
        }
    }
    tokens.extend(summary_tokens(&bridge.decision.reason));
    tokens
}

fn thought_pressure(
    thought_lanes: &HeartbeatThoughtLanes,
    incubation: &HeartbeatIncubation,
) -> f64 {
    let analytic = thought_lanes
        .analytic
        .active_threads
        .iter()
        .map(|thread| thread.desire_to_act)
        .max_by(f64::total_cmp)
        .unwrap_or(0.0);
    let associative = thought_lanes
        .associative
        .active_threads
        .iter()
        .map(|thread| thread.desire_to_speak)
        .max_by(f64::total_cmp)
        .unwrap_or(0.0);
    let theme = incubation
        .themes
        .iter()
        .map(|theme| theme.strength)
        .max_by(f64::total_cmp)
        .unwrap_or(0.0);
    (analytic * 0.35 + associative * 0.25 + theme * 0.4).clamp(0.0, 1.0)
}

fn personality_projection(
    profile: &RoleAppraisalProfile,
    thought_tokens: &BTreeSet<String>,
) -> Vec<HeartbeatPersonalityProjection> {
    let mut scored = profile
        .traits
        .iter()
        .map(|item| {
            let trait_tokens = summary_tokens(&format!("{} {}", item.group, item.name));
            let overlap = token_overlap(&trait_tokens, thought_tokens);
            let projection = round3((item.weight * (0.55 + overlap * 1.8)).clamp(0.0, 1.0));
            HeartbeatPersonalityProjection {
                group: item.group.clone(),
                name: item.name.clone(),
                activation: round3(item.activation),
                plasticity: round3(item.plasticity),
                token_overlap: round3(overlap),
                projection,
            }
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| right.projection.total_cmp(&left.projection));
    scored.truncate(6);
    scored
}

fn strongest_thought_summary(
    thought_lanes: &HeartbeatThoughtLanes,
    incubation: &HeartbeatIncubation,
) -> String {
    incubation
        .themes
        .first()
        .map(|theme| theme.summary.as_str())
        .or_else(|| {
            thought_lanes
                .analytic
                .active_threads
                .first()
                .map(|thread| thread.claim.as_str())
        })
        .unwrap_or("No salient thought cluster is active.")
        .to_string()
}

fn interpretation_label(
    bridge_decision: &str,
    arousal: f64,
    guardedness: f64,
    curiosity: f64,
) -> String {
    if guardedness > 0.72 && arousal > 0.35 {
        "protective_appraisal".to_string()
    } else if curiosity > 0.68 {
        "investigative_appraisal".to_string()
    } else if bridge_decision == "draft" {
        "expressive_appraisal".to_string()
    } else if arousal < 0.12 {
        "low_pressure_appraisal".to_string()
    } else {
        "incubating_appraisal".to_string()
    }
}

fn reaction_mode(label: &str, bridge_decision: &str) -> &'static str {
    match (label, bridge_decision) {
        ("protective_appraisal", _) => "hold_and_verify",
        ("investigative_appraisal", _) => "inspect",
        ("expressive_appraisal", "draft") => "draft",
        ("low_pressure_appraisal", _) => "sleep_ruminate",
        _ => "incubate",
    }
}

fn mood_label(arousal: f64, guardedness: f64, curiosity: f64) -> &'static str {
    if guardedness > 0.72 && arousal > 0.35 {
        "wary"
    } else if curiosity > 0.68 && arousal > 0.25 {
        "keen"
    } else if arousal < 0.12 {
        "drowsy"
    } else if guardedness > curiosity {
        "watchful"
    } else {
        "interested"
    }
}

fn reaction_recommended_use(role_id: &str, mode: &str) -> &'static str {
    match (role_id, mode) {
        ("Persona", "draft") => {
            "Prepare a reviewed Aquarium-facing draft; do not post automatically."
        }
        (_, "hold_and_verify") => "Bias toward verifier/modeler review before expression.",
        (_, "inspect") => {
            "Bias the next heartbeat toward a bounded retrieval or modeling inspection."
        }
        (_, "sleep_ruminate") => "Let this organ sleep-ruminate unless real work arrives.",
        _ => "Keep the thought incubating and visible in Aquarium.",
    }
}

fn topic_saturation_from_syntheses_and_themes(
    syntheses: &[HeartbeatBridgeSynthesis],
    themes: &[HeartbeatIncubationTheme],
) -> Vec<HeartbeatTopicSaturation> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for synthesis in syntheses {
        for topic in &synthesis.dominant_topics {
            *counts.entry(topic.to_string()).or_default() += 1;
        }
    }
    for theme in themes {
        let bonus = if matches!(theme.status.as_str(), "refractory" | "stalled") {
            2
        } else {
            1
        };
        *counts.entry(theme.theme_id.clone()).or_default() += bonus;
    }
    counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(topic, count)| HeartbeatTopicSaturation {
            topic,
            dominance: round3((count as f64 / syntheses.len().max(1) as f64).min(1.0)),
            recent_mentions: count,
            cooling_advice:
                "Require fresh evidence or a new angle before surfacing this topic again."
                    .to_string(),
        })
        .collect()
}

fn refractory_topics_from_themes(
    previous_bridge: &Option<HeartbeatCognitionBridge>,
    themes: &[HeartbeatIncubationTheme],
) -> Vec<HeartbeatRefractoryTopic> {
    let previous_topics = previous_bridge
        .as_ref()
        .map(|value| value.refractory_topics.clone())
        .unwrap_or_default();
    let now = chrono::Utc::now();
    themes
        .iter()
        .filter_map(|theme| {
            let status = theme.status.as_str();
            let saturation = theme.saturation_score;
            if !matches!(status, "refractory" | "stalled") && saturation < 0.62 {
                return None;
            }
            let topic = theme.theme_id.as_str();
            let previous_penalty = previous_topics
                .iter()
                .find(|entry| theme_similarity(topic, "", &entry.topic, "") >= 0.48)
                .map(|entry| entry.penalty)
                .unwrap_or(0.18);
            let penalty = round3(theme.refractory_penalty.max(previous_penalty));
            let hours = if penalty >= 0.28 {
                4
            } else if penalty >= 0.20 {
                3
            } else {
                2
            };
            Some(HeartbeatRefractoryTopic {
                topic: topic.to_string(),
                penalty,
                cools_until: (now + Duration::hours(hours))
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
                    .replace('Z', "+00:00"),
                reason: build_refractory_reason(theme),
                last_triggered_at: now_iso(),
            })
        })
        .take(6)
        .collect()
}

fn build_source_coverage(records: &[RoleMemoryRecord]) -> HeartbeatSourceCoverage {
    let mut role_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut kind_counts: BTreeMap<String, usize> = BTreeMap::new();
    for record in records {
        *role_counts.entry(record.role_id.clone()).or_default() += 1;
        *kind_counts.entry(record.memory_kind.clone()).or_default() += 1;
    }
    HeartbeatSourceCoverage {
        schema_version: "epiphany.source_coverage.v0".to_string(),
        updated_at: now_iso(),
        roles: role_counts
            .into_iter()
            .map(|(role_id, count)| HeartbeatSourceCoverageRole { role_id, count })
            .collect(),
        memory_kinds: kind_counts
            .into_iter()
            .map(|(kind, count)| HeartbeatSourceCoverageKind { kind, count })
            .collect(),
    }
}

fn best_matching_theme<'a>(
    previous_themes: &'a [HeartbeatIncubationTheme],
    theme_id: &str,
    summary: &str,
    source_memory_ids: &[String],
) -> Option<&'a HeartbeatIncubationTheme> {
    let best =
        previous_themes.iter().max_by(|left, right| {
            theme_match_score(theme_id, summary, source_memory_ids, left).total_cmp(
                &theme_match_score(theme_id, summary, source_memory_ids, right),
            )
        })?;
    (theme_match_score(theme_id, summary, source_memory_ids, best) >= 0.42).then_some(best)
}

fn previous_support_count(previous_theme: Option<&HeartbeatIncubationTheme>) -> usize {
    previous_theme.map(|theme| theme.support_count).unwrap_or(0)
}

fn novelty_to_self(
    theme_id: &str,
    summary: &str,
    source_memory_ids: &[String],
    previous_themes: &[HeartbeatIncubationTheme],
    bridge: &Option<HeartbeatCognitionBridge>,
) -> f64 {
    let mut strongest_match = 0.0_f64;
    for theme in previous_themes {
        strongest_match = strongest_match.max(
            theme_similarity(theme_id, summary, &theme.theme_id, &theme.summary).max(
                overlap_ratio_strings(source_memory_ids, &theme.source_memory_ids),
            ),
        );
    }
    for synthesis in bridge
        .as_ref()
        .into_iter()
        .flat_map(|value| value.recent_syntheses.iter())
        .into_iter()
        .take(6)
    {
        strongest_match = strongest_match.max(theme_similarity(
            theme_id,
            summary,
            &synthesis.dominant_topics.join(" / "),
            &synthesis.summary,
        ));
    }
    (1.0 - strongest_match).clamp(0.0, 1.0)
}

fn novelty_to_room(
    theme_id: &str,
    summary: &str,
    previous_candidate_interventions: &Option<HeartbeatCandidateInterventions>,
    bridge: &Option<HeartbeatCognitionBridge>,
) -> f64 {
    let mut score = 0.64_f64;
    for intervention in previous_candidate_interventions
        .as_ref()
        .into_iter()
        .flat_map(|value| value.items.iter())
        .take(8)
    {
        let similarity = theme_similarity(
            theme_id,
            summary,
            &intervention.intervention_id,
            &intervention.draft,
        );
        if similarity >= 0.42 {
            return 0.22;
        }
    }
    for synthesis in bridge
        .as_ref()
        .into_iter()
        .flat_map(|value| value.recent_syntheses.iter())
        .into_iter()
        .take(6)
    {
        let similarity = theme_similarity(
            theme_id,
            summary,
            &synthesis.dominant_topics.join(" / "),
            &synthesis.summary,
        );
        if similarity >= 0.42 {
            score = score.min(0.44);
        }
    }
    score
}

#[derive(Clone, Copy, Debug)]
struct SaturationMetrics {
    score: f64,
    recent_match_count: usize,
}

fn saturation_metrics(
    theme_id: &str,
    summary: &str,
    source_memory_ids: &[String],
    previous_themes: &[HeartbeatIncubationTheme],
    bridge: &Option<HeartbeatCognitionBridge>,
    support_count: usize,
) -> SaturationMetrics {
    let mut recent_match_count = 0_usize;
    for theme in previous_themes {
        let similarity = theme_similarity(theme_id, summary, &theme.theme_id, &theme.summary).max(
            overlap_ratio_strings(source_memory_ids, &theme.source_memory_ids),
        );
        if similarity >= 0.42 {
            recent_match_count += 1;
        }
    }
    for synthesis in bridge
        .as_ref()
        .into_iter()
        .flat_map(|value| value.recent_syntheses.iter())
        .into_iter()
        .take(5)
    {
        let similarity = theme_similarity(
            theme_id,
            summary,
            &synthesis.dominant_topics.join(" / "),
            &synthesis.summary,
        );
        if similarity >= 0.42 {
            recent_match_count += 1;
        }
    }
    let existing_topic_saturation = bridge
        .as_ref()
        .into_iter()
        .flat_map(|value| value.topic_saturation.iter())
        .filter_map(|entry| {
            let similarity = theme_similarity(theme_id, summary, &entry.topic, "");
            (similarity >= 0.42).then_some(entry.dominance)
        })
        .fold(0.0_f64, f64::max);
    SaturationMetrics {
        score: (recent_match_count as f64 * 0.16
            + existing_topic_saturation * 0.42
            + (support_count as f64 / 10.0).min(1.0) * 0.16)
            .clamp(0.0, 1.0),
        recent_match_count,
    }
}

fn refractory_penalty(
    theme_id: &str,
    summary: &str,
    bridge: &Option<HeartbeatCognitionBridge>,
) -> f64 {
    let mut penalty = 0.0_f64;
    let now = chrono::Utc::now();
    for topic in bridge
        .as_ref()
        .into_iter()
        .flat_map(|value| value.refractory_topics.iter())
    {
        let cools_until = chrono::DateTime::parse_from_rfc3339(&topic.cools_until)
            .ok()
            .map(|value| value.with_timezone(&chrono::Utc));
        if cools_until.is_some_and(|deadline| deadline < now) {
            continue;
        }
        let similarity = theme_similarity(theme_id, summary, &topic.topic, &topic.reason);
        if similarity >= 0.48 {
            penalty = penalty.max(topic.penalty * similarity);
        }
    }
    penalty.clamp(0.0, 0.45)
}

fn evidence_diversity(
    source_roles: &[String],
    source_kinds: &[String],
    source_memory_ids: &[String],
) -> f64 {
    (source_roles.len() as f64 * 0.18
        + source_kinds.len() as f64 * 0.16
        + (source_memory_ids.len() as f64 / 4.0).min(1.0) * 0.10)
        .clamp(0.0, 1.0)
}

fn exploration_bonus(
    source_roles: &[String],
    source_kinds: &[String],
    source_coverage: &HeartbeatSourceCoverage,
) -> f64 {
    let mut scores = Vec::new();
    for role_id in source_roles {
        scores.push(inverse_role_coverage_weight(source_coverage, role_id));
    }
    for kind in source_kinds {
        scores.push(inverse_kind_coverage_weight(source_coverage, kind));
    }
    if scores.is_empty() {
        return 0.18;
    }
    average(scores.into_iter()).unwrap_or(0.18).clamp(0.0, 1.0)
}

fn inverse_role_coverage_weight(source_coverage: &HeartbeatSourceCoverage, needle: &str) -> f64 {
    inverse_coverage_count(
        source_coverage
            .roles
            .iter()
            .find(|entry| entry.role_id == needle)
            .map(|entry| entry.count)
            .unwrap_or(0),
    )
}

fn inverse_kind_coverage_weight(source_coverage: &HeartbeatSourceCoverage, needle: &str) -> f64 {
    inverse_coverage_count(
        source_coverage
            .memory_kinds
            .iter()
            .find(|entry| entry.kind == needle)
            .map(|entry| entry.count)
            .unwrap_or(0),
    )
}

fn inverse_coverage_count(count: usize) -> f64 {
    match count {
        0 | 1 => 0.90,
        2 => 0.64,
        3 => 0.42,
        _ => 0.20,
    }
}

fn theme_status(
    novelty_to_self: f64,
    novelty_to_room: f64,
    saturation_score: f64,
    refractory_penalty: f64,
    support_count: usize,
    evidence_diversity: f64,
    desire_to_speak: f64,
    maturation: f64,
) -> &'static str {
    if novelty_to_self < 0.28 && saturation_score >= 0.62 {
        "stalled"
    } else if refractory_penalty >= 0.18 && novelty_to_room < 0.72 {
        "refractory"
    } else if support_count >= 6 && evidence_diversity < 0.34 {
        "stalled"
    } else if support_count >= 3 && novelty_to_self < 0.55 {
        "cooling"
    } else if saturation_score >= 0.56 && novelty_to_self < 0.42 {
        "cooling"
    } else if desire_to_speak >= 0.74
        && (novelty_to_self >= 0.55 || novelty_to_room >= 0.82)
        && saturation_score < 0.50
        || maturation >= 0.82
    {
        "ripe"
    } else {
        "incubating"
    }
}

fn build_incubation_question(source_roles: &[String], source_kinds: &[String]) -> &'static str {
    if source_roles.len() > 1 && source_kinds.len() > 1 {
        "Why are several organs and memory kinds rhyming here, and what would falsify the rhyme before it hardens into doctrine?"
    } else if source_roles.len() > 1 {
        "Is this cross-organ rhyme a real pressure seam or just shared vocabulary bouncing around the hull?"
    } else {
        "Does this hot local seam deserve reinforcement, or is it only loud because nothing else moved yet?"
    }
}

fn build_incubation_attraction(
    source_roles: &[String],
    source_kinds: &[String],
    novelty_to_self: f64,
    evidence_diversity: f64,
) -> &'static str {
    if novelty_to_self >= 0.62 && evidence_diversity >= 0.45 {
        "It keeps pulling because it is still finding genuinely different support instead of merely changing hats."
    } else if source_roles.len() >= 2 {
        "It keeps pulling because more than one organ is worrying the same seam at once."
    } else if source_kinds.len() >= 2 {
        "It keeps pulling because the same seam is surfacing across different memory kinds, not just one hot recollection."
    } else {
        "It keeps pulling because one live seam still has some blood in it; that does not make it king."
    }
}

fn build_incubation_holding_line(
    status: &str,
    saturation_score: f64,
    novelty_to_self: f64,
    exploration_bonus: f64,
) -> &'static str {
    match status {
        "stalled" => {
            "This seam is repeating itself without earning new structure. Merge the receipts, cool it off, and go somewhere less domesticated."
        }
        "refractory" => {
            "This seam has been chewing the same meat too recently. Let it cool unless a genuinely different source family forces it back open."
        }
        "ripe" => {
            "This seam has enough connective tissue that silence should be a deliberate choice rather than a reflex."
        }
        _ if saturation_score >= 0.55 && novelty_to_self < 0.45 => {
            "The family resemblance is getting too strong. Follow a stranger branch before this theme speaks again."
        }
        _ if exploration_bonus >= 0.45 => {
            "This one is still drawing energy from underworked terrain, so another pass might actually change it."
        }
        _ => {
            "Give it another pass so the thought can either grow teeth or admit it was only a pleasant loop."
        }
    }
}

fn build_refractory_reason(theme: &HeartbeatIncubationTheme) -> String {
    let topic = theme.theme_id.as_str();
    let novelty_to_self = theme.novelty_to_self;
    let saturation_score = theme.saturation_score;
    if novelty_to_self < 0.3 {
        format!(
            "{topic} is matching Epiphany's own recent thought history too closely to deserve another immediate pass."
        )
    } else if saturation_score >= 0.62 {
        format!(
            "{topic} has dominated too many recent bridge syntheses and needs cooling before it becomes machine religion."
        )
    } else {
        format!("{topic} needs a brief cooling period before it earns attention again.")
    }
}

fn bridge_summary(theme: &HeartbeatIncubationTheme, speak_decision: &str) -> String {
    let topic = theme.theme_id.as_str();
    let status = theme.status.as_str();
    let novelty_to_self = theme.novelty_to_self;
    let novelty_to_room = theme.novelty_to_room;
    match (speak_decision, status) {
        ("draft", _) => format!(
            "Analytic evidence edges and associative incubation currently converge on {topic}; this seam is still fresh enough to surface (self={novelty_to_self:.2}, room={novelty_to_room:.2})."
        ),
        (_, "refractory" | "stalled") => format!(
            "{topic} is still loud, but the bridge is cooling it off because the machine has been worrying the same seam too hard."
        ),
        _ => format!(
            "{topic} is still live, so the bridge is letting it sit and deepen instead of forcing novelty theater."
        ),
    }
}

fn synthesis_saturation_note(
    status: &str,
    saturation_score: f64,
    novelty_to_self: f64,
) -> Option<&'static str> {
    if matches!(status, "refractory" | "stalled") {
        Some(
            "This topic is currently under cooling discipline; demand stranger support before resurfacing it.",
        )
    } else if saturation_score >= 0.55 && novelty_to_self < 0.45 {
        Some(
            "Recent syntheses are getting too familial; rebranch before letting this seam become doctrine.",
        )
    } else if status == "incubating" {
        Some(
            "A live thought is allowed to sit without performing a fresh retrieval errand every pass.",
        )
    } else {
        None
    }
}

fn bridge_decision_reason(
    strongest_status: &str,
    novelty_to_self: f64,
    novelty_to_room: f64,
    saturation_score: f64,
    refractory_penalty: f64,
    speak_decision: &str,
) -> &'static str {
    match speak_decision {
        "draft" => {
            "Bridge surfaces this seam because it is fresh enough to the swarm-facing surface and not yet overworked."
        }
        "hold" if strongest_status == "incubating" => {
            "Bridge keeps this seam in incubation because it is still live and grounded; retrieval is support, not throat-clearing ritual."
        }
        "hold" => {
            "Bridge holds the seam open without surfacing it yet; there is enough blood for another pass, but not enough freshness for speech."
        }
        "silence" if strongest_status == "refractory" || refractory_penalty >= 0.18 => {
            "Bridge silences this seam temporarily because the machine has been circling it too hard."
        }
        "silence"
            if strongest_status == "stalled"
                || saturation_score >= 0.62
                || novelty_to_self < 0.28 =>
        {
            "Bridge silences this seam because it is mostly self-echo at the moment."
        }
        _ if novelty_to_room < 0.40 => {
            "Bridge withholds surface speech because the thought is not novel enough to the swarm-facing room."
        }
        _ => {
            "Bridge converts analytic and associative lanes into draft/hold/silence guidance without granting authority."
        }
    }
}

fn theme_similarity(
    left_topic: &str,
    left_summary: &str,
    right_topic: &str,
    right_summary: &str,
) -> f64 {
    let left_tokens = summary_tokens(&format!("{left_topic} {left_summary}"));
    let right_tokens = summary_tokens(&format!("{right_topic} {right_summary}"));
    token_overlap(&left_tokens, &right_tokens)
}

fn theme_match_score(
    theme_id: &str,
    summary: &str,
    source_memory_ids: &[String],
    theme: &HeartbeatIncubationTheme,
) -> f64 {
    theme_similarity(theme_id, summary, &theme.theme_id, &theme.summary).max(overlap_ratio_strings(
        source_memory_ids,
        &theme.source_memory_ids,
    ))
}

fn overlap_ratio_strings(left: &[String], right: &[String]) -> f64 {
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let left_set = left.iter().cloned().collect::<BTreeSet<_>>();
    let right_set = right.iter().cloned().collect::<BTreeSet<_>>();
    let shared = left_set.intersection(&right_set).count() as f64;
    if shared <= 0.0 {
        return 0.0;
    }
    let union = left_set.union(&right_set).count() as f64;
    if union <= 0.0 { 0.0 } else { shared / union }
}

fn update_sleep_cycle(
    previous: Option<&HeartbeatSleepCycle>,
    incubation: &HeartbeatIncubation,
    allow_dream: bool,
) -> HeartbeatSleepCycle {
    let cycle_hours = previous
        .map(|cycle| cycle.cycle_hours)
        .unwrap_or(4)
        .clamp(2, 12);
    let nap_duration_minutes = previous
        .map(|cycle| cycle.nap_duration_minutes)
        .unwrap_or(60)
        .clamp(15, cycle_hours * 60 - 5);
    let phase_offset_minutes = previous
        .map(|cycle| cycle.phase_offset_minutes_local)
        .unwrap_or(120)
        .clamp(0, cycle_hours * 60 - 1);
    let now = chrono::Utc::now();
    let cycle_minutes = cycle_hours * 60;
    let shifted = now.timestamp().div_euclid(60) - phase_offset_minutes;
    let minute_in_cycle = shifted.rem_euclid(cycle_minutes);
    let is_napping = minute_in_cycle < nap_duration_minutes;
    let cycle_start_minute = shifted - minute_in_cycle + phase_offset_minutes;
    let nap_start =
        chrono::DateTime::<chrono::Utc>::from_timestamp(cycle_start_minute * 60, 0).unwrap_or(now);
    let nap_end = nap_start + Duration::minutes(nap_duration_minutes);
    let next_nap_start = if is_napping {
        nap_start + Duration::minutes(cycle_minutes)
    } else if minute_in_cycle >= nap_duration_minutes {
        nap_start + Duration::minutes(cycle_minutes)
    } else {
        nap_start
    };
    let active_themes = incubation
        .themes
        .iter()
        .map(|theme| theme.theme_id.as_str())
        .take(4)
        .map(str::to_string)
        .collect::<Vec<_>>();
    let previous_dream_count = previous
        .map(|cycle| cycle.dream_count_in_current_nap)
        .unwrap_or(0);
    let dream_count = if is_napping && allow_dream {
        previous_dream_count.saturating_add(1)
    } else if is_napping {
        previous_dream_count
    } else {
        0
    };
    HeartbeatSleepCycle {
        schema_version: "epiphany.sleep_cycle.v0".to_string(),
        enabled: true,
        cycle_hours,
        nap_duration_minutes,
        phase_offset_minutes_local: phase_offset_minutes,
        reply_mode: "sleep_rumination".to_string(),
        is_napping,
        current_nap_started_at: if is_napping {
            Some(nap_start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        } else {
            None
        },
        current_nap_ends_at: if is_napping {
            Some(nap_end.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        } else {
            None
        },
        next_nap_starts_at: next_nap_start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        last_dream_at: if is_napping && allow_dream {
            Some(now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        } else {
            previous.and_then(|cycle| cycle.last_dream_at.clone())
        },
        dream_count_in_current_nap: dream_count,
        active_dream_themes: active_themes,
        last_distillation_summary: if is_napping {
            "Sleep pass prefers memory compression, resonance cooling, and dream residue over active work."
        } else {
            "Awake pass keeps resonance/incubation fresh without speaking unless Persona has a real surface reason."
        }
        .to_string(),
    }
}

fn summary_tokens(summary: &str) -> BTreeSet<String> {
    summary
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(|part| part.trim().to_ascii_lowercase())
        .filter(|part| part.len() >= 4 && !STOP_WORDS.contains(&part.as_str()))
        .collect()
}

fn token_overlap(left: &BTreeSet<String>, right: &BTreeSet<String>) -> f64 {
    let shared = left.intersection(right).count() as f64;
    if shared == 0.0 {
        return 0.0;
    }
    let union = left.union(right).count() as f64;
    if union <= 0.0 { 0.0 } else { shared / union }
}

fn shared_tokens(left: &BTreeSet<String>, right: &BTreeSet<String>) -> Vec<String> {
    left.intersection(right).take(8).cloned().collect()
}

fn stable_theme_suffix(value: &str) -> String {
    let mut hash = 5381_u64;
    for byte in value.as_bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(*byte as u64);
    }
    format!("{:x}", hash & 0xffff)
}

pub(super) fn now_iso() -> String {
    chrono::Utc::now()
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
        .replace('Z', "+00:00")
}

fn parse_heartbeat_time(label: &str, value: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|time| time.with_timezone(&Utc))
        .with_context(|| format!("failed to parse {label} {value:?} as RFC3339"))
}

fn now_stamp() -> String {
    chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

pub(super) fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

pub(super) fn round3(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}

fn average(values: impl Iterator<Item = f64>) -> Option<f64> {
    let mut total = 0.0;
    let mut count = 0_usize;
    for value in values {
        if value.is_finite() {
            total += value;
            count += 1;
        }
    }
    (count > 0).then_some(total / count as f64)
}

const STOP_WORDS: &[&str] = &[
    "about", "after", "agent", "before", "being", "between", "could", "from", "have", "into",
    "lane", "memory", "more", "must", "should", "state", "than", "that", "their", "there", "this",
    "through", "when", "with", "work",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EpiphanyAgentMemoryEntry;
    use crate::GhostlightAgent;
    use crate::GhostlightCanonicalState;
    use crate::GhostlightIdentity;
    use crate::GhostlightMemories;
    use crate::GhostlightMemory;
    use crate::GhostlightValue;
    use crate::GhostlightWorld;
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
                agent_store: None,
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
                agent_store: None,
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
                agent_store: None,
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
                agent_store: None,
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
                agent_store: None,
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
                agent_store: None,
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
                agent_store: None,
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
    fn pending_persona_mention_selects_persona_turn_and_consumes_queue() -> Result<()> {
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
                agent_store: None,
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
            Some(0)
        );
        let state = load_heartbeat_state_entry(&store_path)?.expect("heartbeat state");
        assert!(state.pending_mentions.is_empty());
        Ok(())
    }

    #[test]
    fn persona_turn_action_catalog_carries_memory_recall_from_agent_store() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("Persona-heartbeats.msgpack");
        let agent_store = temp.path().join("agents.msgpack");
        let artifact_dir = temp.path().join("artifacts");
        initialize_heartbeat_store(&store_path, 1.0)?;
        crate::ensure_agent_memory_swarm_identity(&agent_store, "heartbeat-test-swarm")?;
        crate::write_agent_memory_entry_for_role_migration(&agent_store, &persona_memory_entry())?;
        queue_heartbeat_pending_mention_store(
            &store_path,
            HeartbeatQueueMentionOptions {
                target_role_id: "Persona".to_string(),
                source_surface: "discord".to_string(),
                channel_id: "aquarium".to_string(),
                message_id: "m1".to_string(),
                author_id: "human".to_string(),
                author_name: Some("Metacrat".to_string()),
                content: "Epiphany, remember the typed contracts before speaking.".to_string(),
                visible_prompt: "remember the typed contracts before speaking".to_string(),
                reply_to_message_id: None,
                queued_at: Some("2026-05-24T00:00:00+00:00".to_string()),
                mention_id: Some("mention-Persona-memory-test".to_string()),
            },
        )?;

        let tick = tick_heartbeat_store(
            &store_path,
            &artifact_dir,
            HeartbeatTickOptions {
                target_heartbeat_rate: 1.0,
                coordinator_action: None,
                target_role: None,
                urgency: 0.0,
                schedule_id: "Persona-mentioned-memory".to_string(),
                source_scene_ref: "test/Persona-mentioned-memory".to_string(),
                defer_completion: true,
                agent_store: Some(agent_store),
            },
        )?;

        let recall = &tick["schedule"]["action_catalog"][0]["persona_memory_recall"];
        assert_eq!(
            recall["schemaVersion"],
            crate::SEMANTIC_PROJECTION_SCHEMA_VERSION
        );
        assert_eq!(recall["roleId"], "Persona");
        assert_eq!(recall["privateStateExposed"], false);
        assert!(
            !recall["status"].as_str().unwrap_or_default().is_empty(),
            "Persona recall should report whether it came from Qdrant or fallback"
        );
        assert!(
            !recall["cacheStatus"]
                .as_str()
                .unwrap_or_default()
                .is_empty(),
            "Persona recall should report cache status"
        );
        assert!(
            !recall["renderedRecall"]
                .as_str()
                .unwrap_or_default()
                .trim()
                .is_empty(),
            "Persona recall should carry a rendered prompt surface"
        );
        assert!(recall["chunkCount"].as_u64().unwrap_or_default() > 0);
        assert!(
            !recall["renderedRecall"]
                .as_str()
                .unwrap_or_default()
                .contains("sealed private note")
        );
        Ok(())
    }

    fn persona_memory_entry() -> EpiphanyAgentMemoryEntry {
        EpiphanyAgentMemoryEntry {
            schema_version: "ghostlight.agent_state.v0".to_string(),
            role_id: "Persona".to_string(),
            world: GhostlightWorld::default(),
            agent: GhostlightAgent {
                agent_id: "epiphany.Persona".to_string(),
                identity: GhostlightIdentity {
                    name: "Epiphany".to_string(),
                    roles: vec!["Persona".to_string()],
                    origin: "EpiphanyAgent".to_string(),
                    public_description: "Public typed-contract voice.".to_string(),
                    private_notes: vec!["sealed private note".to_string()],
                },
                memories: GhostlightMemories {
                    semantic: vec![GhostlightMemory {
                        memory_id: "semantic-1".to_string(),
                        summary: "Clean typed contracts must shape Persona speech.".to_string(),
                        salience: 0.9,
                        confidence: 0.9,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                canonical_state: GhostlightCanonicalState {
                    values: vec![GhostlightValue {
                        value_id: "value-1".to_string(),
                        label: "Clean typed contracts".to_string(),
                        priority: 0.9,
                        unforgivable_if_betrayed: true,
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
            relationships: Vec::new(),
            events: Vec::new(),
            scenes: Vec::new(),
        }
    }

    #[test]
    fn appraisal_mood_timing_carries_affect_vector() {
        let mut state = default_heartbeat_state(1.0);
        patch_missing_participants(&mut state);
        let appraisals = HeartbeatAgentThoughtAppraisals {
            schema_version: "epiphany.agent_thought_appraisals.v0".to_string(),
            updated_at: "2026-05-20T00:00:00+00:00".to_string(),
            thought_cluster_ref: "test-affect".to_string(),
            participant_appraisals: vec![HeartbeatAgentThoughtAppraisal {
                appraisal_id: "appraisal-Persona-affect".to_string(),
                participant_agent_id: "epiphany.Persona".to_string(),
                role_id: "Persona".to_string(),
                emotional_appraisal: HeartbeatEmotionalAppraisal {
                    valence: 0.2,
                    arousal: 0.81,
                    urgency: 0.66,
                    curiosity: 0.14,
                    guardedness: 0.72,
                    thought_pressure: 0.77,
                },
                personality_projection: vec![
                    HeartbeatPersonalityProjection {
                        name: "anger".to_string(),
                        projection: 0.88,
                        ..HeartbeatPersonalityProjection::default()
                    },
                    HeartbeatPersonalityProjection {
                        name: "dismissal".to_string(),
                        projection: 0.63,
                        ..HeartbeatPersonalityProjection::default()
                    },
                    HeartbeatPersonalityProjection {
                        name: "flippancy".to_string(),
                        projection: 0.41,
                        ..HeartbeatPersonalityProjection::default()
                    },
                ],
                candidate_implications: HeartbeatCandidateImplications {
                    reaction_intensity: 0.79,
                    ..HeartbeatCandidateImplications::default()
                },
                ..HeartbeatAgentThoughtAppraisal::default()
            }],
        };

        apply_mood_timing_from_appraisals(&mut state, &appraisals);
        let persona = state
            .participants
            .iter()
            .find(|participant| participant.role_id == "Persona")
            .expect("Persona participant");
        let mood = persona.mood_timing.as_ref().expect("Persona mood timing");
        assert_eq!(mood.emotional_dimensions.len(), 32);
        assert_eq!(mood_dimension(mood, "anger"), Some(0.88));
        assert_eq!(mood_dimension(mood, "dismissal"), Some(0.63));
        assert_eq!(mood_dimension(mood, "flippancy"), Some(0.41));
        assert_eq!(mood_dimension(mood, "valence"), Some(0.2));
        assert_eq!(mood_dimension(mood, "arousal"), Some(0.81));
    }

    fn mood_dimension(mood: &HeartbeatMoodTiming, name: &str) -> Option<f64> {
        mood.emotional_dimensions
            .iter()
            .find(|dimension| dimension.name == name)
            .map(|dimension| dimension.value)
    }
}
