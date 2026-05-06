use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::Duration;
use cultcache_rs::CultCache;
use cultcache_rs::DatabaseEntry;
use cultcache_rs::SingleFileMessagePackBackingStore;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub const HEARTBEAT_STATE_TYPE: &str = "epiphany.agent_heartbeat";
pub const HEARTBEAT_STATE_KEY: &str = "default";
pub const HEARTBEAT_STATE_SCHEMA_VERSION: &str = "epiphany.agent_heartbeat.v0";
pub const HEARTBEAT_STATUS_SCHEMA_VERSION: &str = "epiphany.agent_heartbeat_status.v0";
pub const INITIATIVE_SCHEMA_VERSION: &str = "ghostlight.initiative_schedule.v0";
pub const VOID_ROUTINE_SCHEMA_VERSION: &str = "epiphany.void_routine.v0";

const HEARTBEAT_ARENA_MAINTENANCE: &str = "maintenance";
const HEARTBEAT_ARENA_SCENE: &str = "scene";
const PARTICIPANT_KIND_AGENT: &str = "agent";
const PARTICIPANT_KIND_CHARACTER: &str = "character";

const ROLE_ORDER: &[&str] = &[
    "coordinator",
    "face",
    "imagination",
    "research",
    "modeling",
    "implementation",
    "verification",
    "reorientation",
];

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.agent_heartbeat",
    schema = "EpiphanyHeartbeatStateEntry"
)]
pub struct EpiphanyHeartbeatStateEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub target_heartbeat_rate: f64,
    #[cultcache(key = 2)]
    pub scene_clock: f64,
    #[cultcache(key = 3)]
    pub selection_policy: HeartbeatSelectionPolicy,
    #[cultcache(key = 4)]
    pub pacing_policy: HeartbeatPacingPolicy,
    #[cultcache(key = 5)]
    pub participants: Vec<HeartbeatParticipant>,
    #[cultcache(key = 6)]
    pub history: Vec<HeartbeatHistoryEvent>,
    #[cultcache(key = 7, default)]
    pub sleep_cycle: Option<Value>,
    #[cultcache(key = 8, default)]
    pub memory_resonance: Option<Value>,
    #[cultcache(key = 9, default)]
    pub incubation: Option<Value>,
    #[cultcache(key = 10, default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatSelectionPolicy {
    pub mode: String,
    pub reaction_precedence: bool,
    pub minimum_speed: f64,
    pub tie_breakers: Vec<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatPacingPolicy {
    pub cooldown_starts_after_turn_completion: bool,
    pub work_base_recovery: f64,
    pub idle_base_recovery: f64,
    pub sleep_heartbeat_rate_multiplier: f64,
    pub minimum_effective_rate: f64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatParticipant {
    pub agent_id: String,
    pub role_id: String,
    pub display_name: String,
    #[serde(default)]
    pub arena: String,
    #[serde(default)]
    pub participant_kind: String,
    pub initiative_speed: f64,
    pub next_ready_at: f64,
    pub reaction_bias: f64,
    pub interrupt_threshold: f64,
    pub current_load: f64,
    pub status: String,
    pub constraints: Vec<String>,
    pub last_action_id: Option<String>,
    pub last_woke_at: Option<String>,
    pub last_finished_at: Option<String>,
    pub pending_turn: Option<HeartbeatPendingTurn>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatPendingTurn {
    pub status: String,
    #[serde(rename = "scheduleId")]
    pub schedule_id: String,
    #[serde(rename = "actionId")]
    pub action_id: String,
    #[serde(rename = "actionType")]
    pub action_type: String,
    #[serde(rename = "actionScale", default)]
    pub action_scale: String,
    #[serde(rename = "localAffordanceBasis", default)]
    pub local_affordance_basis: Vec<String>,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    #[serde(rename = "startedSceneClock")]
    pub started_scene_clock: f64,
    #[serde(rename = "baseRecovery")]
    pub base_recovery: f64,
    pub recovery: f64,
    #[serde(rename = "cooldownPolicy")]
    pub cooldown_policy: String,
    #[serde(rename = "completedAt", default)]
    pub completed_at: Option<String>,
    #[serde(rename = "completedSceneClock", default)]
    pub completed_scene_clock: Option<f64>,
    #[serde(rename = "nextReadyAt", default)]
    pub next_ready_at: Option<f64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatHistoryEvent {
    pub ts: String,
    #[serde(rename = "scheduleId")]
    pub schedule_id: String,
    #[serde(rename = "selectedRole")]
    pub selected_role: String,
    #[serde(rename = "selectedAgentId")]
    pub selected_agent_id: String,
    #[serde(rename = "actionId")]
    pub action_id: String,
    #[serde(rename = "actionType")]
    pub action_type: String,
    #[serde(default)]
    pub arena: String,
    #[serde(rename = "participantKind", default)]
    pub participant_kind: String,
    #[serde(rename = "actionScale", default)]
    pub action_scale: String,
    #[serde(rename = "coordinatorAction", default)]
    pub coordinator_action: Option<String>,
    #[serde(rename = "targetRole", default)]
    pub target_role: Option<String>,
    #[serde(rename = "workRole", default)]
    pub work_role: Option<String>,
    #[serde(rename = "sceneClock", default)]
    pub scene_clock: Option<f64>,
    #[serde(rename = "nextReadyAt", default)]
    pub next_ready_at: Option<f64>,
    #[serde(rename = "turnStatus", default)]
    pub turn_status: Option<String>,
    #[serde(rename = "cooldownStartedAfterCompletion", default)]
    pub cooldown_started_after_completion: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HeartbeatTickOptions {
    pub target_heartbeat_rate: f64,
    pub coordinator_action: Option<String>,
    pub target_role: Option<String>,
    pub urgency: f64,
    pub schedule_id: String,
    pub source_scene_ref: String,
    pub defer_completion: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HeartbeatCompleteOptions {
    pub role: String,
    pub action_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VoidRoutineOptions {
    pub agent_store: Option<std::path::PathBuf>,
    pub source: String,
    pub allow_dream: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GhostlightSceneParticipantSeed {
    pub agent_id: String,
    pub display_name: String,
    pub initiative_speed: f64,
    pub reaction_bias: f64,
    pub interrupt_threshold: f64,
    pub constraints: Vec<String>,
}

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

pub fn heartbeat_state_cache(store_path: impl AsRef<Path>) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<EpiphanyHeartbeatStateEntry>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(store_path.as_ref()));
    cache.pull_all_backing_stores()?;
    Ok(cache)
}

pub fn load_heartbeat_state_entry(
    store_path: impl AsRef<Path>,
) -> Result<Option<EpiphanyHeartbeatStateEntry>> {
    let cache = heartbeat_state_cache(store_path)?;
    cache.get::<EpiphanyHeartbeatStateEntry>(HEARTBEAT_STATE_KEY)
}

pub fn write_heartbeat_state_entry(
    store_path: impl AsRef<Path>,
    state: &EpiphanyHeartbeatStateEntry,
) -> Result<EpiphanyHeartbeatStateEntry> {
    validate_heartbeat_state(state)?;
    let mut cache = heartbeat_state_cache(store_path)?;
    cache.put(HEARTBEAT_STATE_KEY, state)
}

pub fn validate_heartbeat_state(state: &EpiphanyHeartbeatStateEntry) -> Result<()> {
    if state.schema_version != HEARTBEAT_STATE_SCHEMA_VERSION {
        return Err(anyhow!(
            "heartbeat state schema_version is {:?}, expected {:?}",
            state.schema_version,
            HEARTBEAT_STATE_SCHEMA_VERSION
        ));
    }
    if state.participants.is_empty() {
        return Err(anyhow!("heartbeat state has no participants"));
    }
    if state.target_heartbeat_rate < 0.0 {
        return Err(anyhow!(
            "heartbeat target_heartbeat_rate must be non-negative"
        ));
    }
    for participant in &state.participants {
        if participant.agent_id.trim().is_empty() {
            return Err(anyhow!("heartbeat participant has empty agent_id"));
        }
        if participant.role_id.trim().is_empty() {
            return Err(anyhow!(
                "heartbeat participant {} has empty role_id",
                participant.agent_id
            ));
        }
        if participant.initiative_speed <= 0.0 {
            return Err(anyhow!(
                "heartbeat participant {} initiative_speed must be positive",
                participant.agent_id
            ));
        }
        let arena = participant_arena(participant);
        if !matches!(arena, HEARTBEAT_ARENA_MAINTENANCE | HEARTBEAT_ARENA_SCENE) {
            return Err(anyhow!(
                "heartbeat participant {} arena {:?} is unsupported",
                participant.agent_id,
                arena
            ));
        }
        let participant_kind = participant_kind(participant);
        if !matches!(
            participant_kind,
            PARTICIPANT_KIND_AGENT | PARTICIPANT_KIND_CHARACTER
        ) {
            return Err(anyhow!(
                "heartbeat participant {} participant_kind {:?} is unsupported",
                participant.agent_id,
                participant_kind
            ));
        }
    }
    Ok(())
}

pub fn default_heartbeat_state(target_heartbeat_rate: f64) -> EpiphanyHeartbeatStateEntry {
    EpiphanyHeartbeatStateEntry {
        schema_version: HEARTBEAT_STATE_SCHEMA_VERSION.to_string(),
        target_heartbeat_rate,
        scene_clock: 0.0,
        selection_policy: HeartbeatSelectionPolicy {
            mode: "readiness_queue".to_string(),
            reaction_precedence: true,
            minimum_speed: 0.2,
            tie_breakers: vec![
                "reaction_readiness_desc".to_string(),
                "next_ready_at_asc".to_string(),
                "initiative_speed_desc".to_string(),
                "stable_actor_id_asc".to_string(),
            ],
            extra: BTreeMap::new(),
        },
        pacing_policy: HeartbeatPacingPolicy {
            cooldown_starts_after_turn_completion: true,
            work_base_recovery: 6.0,
            idle_base_recovery: 2.0,
            sleep_heartbeat_rate_multiplier: 0.05,
            minimum_effective_rate: 0.001,
            extra: BTreeMap::new(),
        },
        participants: ROLE_ORDER
            .iter()
            .map(|role_id| default_participant(role_id))
            .collect(),
        history: Vec::new(),
        sleep_cycle: None,
        memory_resonance: None,
        incubation: None,
        extra: BTreeMap::new(),
    }
}

pub fn initialize_heartbeat_store(
    store_path: impl AsRef<Path>,
    target_heartbeat_rate: f64,
) -> Result<EpiphanyHeartbeatStateEntry> {
    write_heartbeat_state_entry(store_path, &default_heartbeat_state(target_heartbeat_rate))
}

pub fn ghostlight_scene_heartbeat_state(
    target_heartbeat_rate: f64,
    scene_id: impl Into<String>,
    participants: Vec<GhostlightSceneParticipantSeed>,
) -> Result<EpiphanyHeartbeatStateEntry> {
    if participants.is_empty() {
        return Err(anyhow!(
            "Ghostlight scene heartbeat requires at least one participant"
        ));
    }
    let scene_id = scene_id.into();
    let mut state = default_heartbeat_state(target_heartbeat_rate);
    state.participants = participants
        .into_iter()
        .map(|seed| ghostlight_scene_participant(&scene_id, seed))
        .collect();
    state.extra.insert(
        "protocol".to_string(),
        serde_json::json!({
            "domain": "ghostlight",
            "sceneId": scene_id,
            "arena": HEARTBEAT_ARENA_SCENE,
            "contract": "Characters and maintenance organs use one initiative timing law; scene participants receive only projected local context.",
        }),
    );
    validate_heartbeat_state(&state)?;
    Ok(state)
}

pub fn initialize_ghostlight_scene_heartbeat_store(
    store_path: impl AsRef<Path>,
    target_heartbeat_rate: f64,
    scene_id: impl Into<String>,
    participants: Vec<GhostlightSceneParticipantSeed>,
) -> Result<EpiphanyHeartbeatStateEntry> {
    let state = ghostlight_scene_heartbeat_state(target_heartbeat_rate, scene_id, participants)?;
    write_heartbeat_state_entry(store_path, &state)
}

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
            "availableActions": ["init", "tick", "complete", "status"],
        }));
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
        "sleepCycle": state.sleep_cycle,
        "memoryResonance": state.memory_resonance,
        "incubation": state.incubation,
        "availableActions": ["init", "tick", "complete", "status", "routine"],
    }))
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

    let memory_records = collect_role_memory_records(options.agent_store.as_deref())?;
    let resonance = build_memory_resonance(&memory_records);
    let incubation = build_incubation(&state.incubation, &resonance, &memory_records);
    let sleep_cycle =
        update_sleep_cycle(state.sleep_cycle.as_ref(), &incubation, options.allow_dream);
    let run_id = format!("epiphany-void-routine-{}", now_stamp());
    let routine = serde_json::json!({
        "schema_version": VOID_ROUTINE_SCHEMA_VERSION,
        "runId": run_id,
        "source": options.source,
        "referenceLineage": "VoidBot-style room stewardship, sleep, resonance, and dream maintenance rebuilt as Epiphany-native heartbeat physiology.",
        "storeFile": store_path,
        "agentStore": options.agent_store,
        "updatedAt": now_iso(),
        "sleepCycle": sleep_cycle,
        "memoryResonance": resonance,
        "incubation": incubation,
        "reviewNotes": [
            "Void is reference material, not a runtime dependency.",
            "The routine mutates only typed heartbeat physiology fields; project truth and role memory mutation stay on their dedicated reviewed surfaces.",
            "Sleep is maintenance: slow rumination, memory compression, and dream residue, not absence."
        ],
    });

    state.sleep_cycle = Some(sleep_cycle);
    state.memory_resonance = Some(resonance);
    state.incubation = Some(incubation);
    state.extra.insert(
        "voidRoutine".to_string(),
        serde_json::json!({
            "lastRunId": run_id,
            "lastRunAt": now_iso(),
            "source": options.source,
            "referenceLineage": "VoidBot reference; Epiphany-native implementation.",
        }),
    );
    write_heartbeat_state_entry(store_path, &state)?;

    let artifact_dir = artifact_dir.as_ref();
    fs::create_dir_all(artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    let artifact_path = artifact_dir.join(format!("{run_id}.routine.json"));
    write_json_artifact(&artifact_path, &routine)?;

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
        cooldown_started_after_completion: None,
        extra: BTreeMap::new(),
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
    let recovery = action.base_recovery
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
        extra: BTreeMap::new(),
    };
    state.participants[selected_index].pending_turn = Some(pending.clone());
    state.participants[selected_index].last_action_id = Some(action.action_id.clone());
    state.participants[selected_index].last_woke_at = Some(now_iso());
    state.participants[selected_index].current_load =
        round6((state.participants[selected_index].current_load * 0.75).clamp(0.0, 1.0));
    state.scene_clock = round6(scene_clock);
    if !options.defer_completion {
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
        extra: BTreeMap::new(),
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
            "initiative_cost": action.initiative_cost,
            "interruptibility": action.interruptibility,
            "commitment": action.commitment,
            "local_affordance_basis": action.local_affordance_basis,
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
        "review_notes": [
            "Epiphany heartbeat uses Ghostlight initiative timing as a harness scheduling receipt.",
            "A selected idle lane may ruminate and request bounded self-memory mutation; it may not invent project work.",
            "When no coordinator work is active, idle rumination uses the sleep heartbeat multiplier so the swarm dreams slowly instead of thrashing."
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
            "{} has no actionable lane work; ruminate and distill role memory.",
            selected.display_name
        ),
            "Heartbeat slots control opportunity, not project authority.".to_string(),
            "When no coordinator work is active, idle rumination uses the sleep heartbeat multiplier so the swarm dreams slowly instead of thrashing.".to_string(),
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
        "surfaceAgentThoughts" | "discordAquariumChat" => "face",
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
        .extra
        .get("protocol")
        .and_then(Value::as_object)
        .and_then(|protocol| protocol.get("domain"))
        .and_then(Value::as_str)
        == Some("ghostlight")
}

fn default_participant(role_id: &str) -> HeartbeatParticipant {
    HeartbeatParticipant {
        agent_id: agent_id_for_role(role_id).to_string(),
        role_id: role_id.to_string(),
        display_name: display_name_for_role(role_id).to_string(),
        arena: HEARTBEAT_ARENA_MAINTENANCE.to_string(),
        participant_kind: PARTICIPANT_KIND_AGENT.to_string(),
        initiative_speed: initiative_speed_for_role(role_id),
        next_ready_at: 0.0,
        reaction_bias: reaction_bias_for_role(role_id),
        interrupt_threshold: interrupt_threshold_for_role(role_id),
        current_load: 0.0,
        status: "active".to_string(),
        constraints: participant_constraints(role_id)
            .into_iter()
            .map(str::to_string)
            .collect(),
        last_action_id: None,
        last_woke_at: None,
        last_finished_at: None,
        pending_turn: None,
        extra: BTreeMap::new(),
    }
}

fn ghostlight_scene_participant(
    scene_id: &str,
    seed: GhostlightSceneParticipantSeed,
) -> HeartbeatParticipant {
    let role_id = ghostlight_role_id(seed.agent_id.as_str());
    let mut extra = BTreeMap::new();
    extra.insert("sceneId".to_string(), Value::String(scene_id.to_string()));
    HeartbeatParticipant {
        agent_id: seed.agent_id,
        role_id,
        display_name: seed.display_name,
        arena: HEARTBEAT_ARENA_SCENE.to_string(),
        participant_kind: PARTICIPANT_KIND_CHARACTER.to_string(),
        initiative_speed: seed.initiative_speed,
        next_ready_at: 0.0,
        reaction_bias: seed.reaction_bias,
        interrupt_threshold: seed.interrupt_threshold,
        current_load: 0.0,
        status: "active".to_string(),
        constraints: seed.constraints,
        last_action_id: None,
        last_woke_at: None,
        last_finished_at: None,
        pending_turn: None,
        extra,
    }
}

fn ghostlight_role_id(agent_id: &str) -> String {
    format!(
        "ghostlight.character.{}",
        agent_id
            .trim()
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() {
                    ch.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
    )
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
                "reaction_readiness": reaction_readiness,
                "eligible_for_reaction": eligible,
            })
        })
        .collect()
}

fn participant_arena(participant: &HeartbeatParticipant) -> &str {
    if participant.arena.trim().is_empty() {
        HEARTBEAT_ARENA_MAINTENANCE
    } else {
        participant.arena.as_str()
    }
}

fn participant_kind(participant: &HeartbeatParticipant) -> &str {
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

fn schedule_participant_json(participant: &HeartbeatParticipant) -> Value {
    serde_json::json!({
        "agent_id": participant.agent_id,
        "role_id": participant.role_id,
        "display_name": participant.display_name,
        "arena": participant_arena(participant),
        "participant_kind": participant_kind(participant),
        "initiative_speed": participant.initiative_speed,
        "next_ready_at": participant.next_ready_at,
        "reaction_bias": participant.reaction_bias,
        "interrupt_threshold": participant.interrupt_threshold,
        "current_load": participant.current_load,
        "status": participant.status,
        "pending_turn": participant.pending_turn.as_ref().map(pending_turn_json),
        "constraints": participant.constraints,
    })
}

fn selection_policy_json(policy: &HeartbeatSelectionPolicy) -> Value {
    serde_json::json!({
        "mode": policy.mode,
        "reaction_precedence": policy.reaction_precedence,
        "minimum_speed": policy.minimum_speed,
        "tie_breakers": policy.tie_breakers,
    })
}

fn pending_turn_json(turn: &HeartbeatPendingTurn) -> Value {
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
        "recovery": turn.recovery,
        "cooldownPolicy": turn.cooldown_policy,
        "completedAt": turn.completed_at,
        "completedSceneClock": turn.completed_scene_clock,
        "nextReadyAt": turn.next_ready_at,
    })
}

fn history_event_json(event: HeartbeatHistoryEvent) -> Value {
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

fn rumination_patch(role_id: &str, action_id: &str) -> Value {
    let display_name = display_name_for_role(role_id);
    serde_json::json!({
        "agentId": agent_id_for_role(role_id),
        "reason": format!("{display_name} won an idle heartbeat slot and should preserve the habit of using idle wakeups to distill role memory instead of manufacturing project work."),
        "semanticMemories": [{
            "memoryId": format!("mem-{role_id}-heartbeat-rumination"),
            "summary": "When a heartbeat wakes this lane and no coordinator-approved work is available, the correct move is to ruminate on role quality, cut stale memory, and return a bounded self-memory improvement rather than inventing project authority.",
            "salience": 0.78,
            "confidence": 0.88,
        }],
        "goals": [{
            "goalId": format!("goal-{role_id}-heartbeat-self-distill"),
            "description": "Use idle heartbeat slots to become sharper at this lane's own work before touching project state.",
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
    summary: String,
    salience: f64,
    confidence: f64,
    tokens: BTreeSet<String>,
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
        for memory in entry
            .agent
            .memories
            .semantic
            .iter()
            .chain(entry.agent.memories.episodic.iter())
            .chain(entry.agent.memories.relationship_summaries.iter())
        {
            let tokens = summary_tokens(&memory.summary);
            if tokens.is_empty() {
                continue;
            }
            records.push(RoleMemoryRecord {
                role_id: (*role_id).to_string(),
                memory_id: memory.memory_id.clone(),
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

fn build_memory_resonance(records: &[RoleMemoryRecord]) -> Value {
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
            pairs.push(serde_json::json!({
                "leftRole": left.role_id,
                "leftMemoryId": left.memory_id,
                "leftSummary": left.summary,
                "rightRole": right.role_id,
                "rightMemoryId": right.memory_id,
                "rightSummary": right.summary,
                "strength": strength,
                "sharedTokens": shared_tokens(&left.tokens, &right.tokens),
            }));
        }
    }
    pairs.sort_by(|left, right| {
        right["strength"]
            .as_f64()
            .unwrap_or_default()
            .total_cmp(&left["strength"].as_f64().unwrap_or_default())
    });
    pairs.truncate(8);
    serde_json::json!({
        "schema_version": "epiphany.memory_resonance.v0",
        "updatedAt": now_iso(),
        "source": "epiphany-native-void-routine",
        "recordCount": records.len(),
        "pairs": pairs,
    })
}

fn build_incubation(
    previous: &Option<Value>,
    resonance: &Value,
    records: &[RoleMemoryRecord],
) -> Value {
    let mut themes = previous
        .as_ref()
        .and_then(|value| value.get("themes"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for pair in resonance
        .get("pairs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(4)
    {
        let left_role = pair
            .get("leftRole")
            .and_then(Value::as_str)
            .unwrap_or("left");
        let right_role = pair
            .get("rightRole")
            .and_then(Value::as_str)
            .unwrap_or("right");
        let tokens = pair
            .get("sharedTokens")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .take(3)
                    .collect::<Vec<_>>()
                    .join("/")
            })
            .unwrap_or_default();
        themes.push(serde_json::json!({
            "themeId": format!("theme-{left_role}-{right_role}-{}", stable_theme_suffix(&tokens)),
            "summary": format!("{left_role} and {right_role} keep touching {tokens}; let the swarm dream on whether that is signal or stale echo."),
            "strength": pair.get("strength").cloned().unwrap_or(Value::Null),
            "source": "memory_resonance",
            "updatedAt": now_iso(),
        }));
    }
    if themes.is_empty() && !records.is_empty() {
        let strongest = &records[0];
        themes.push(serde_json::json!({
            "themeId": format!("theme-{}-strongest-memory", strongest.role_id),
            "summary": format!("{} carries the hottest current memory: {}", display_name_for_role(&strongest.role_id), strongest.summary),
            "strength": round3(strongest.salience * strongest.confidence),
            "source": "strongest_memory",
            "updatedAt": now_iso(),
        }));
    }
    themes.sort_by(|left, right| {
        right["strength"]
            .as_f64()
            .unwrap_or_default()
            .total_cmp(&left["strength"].as_f64().unwrap_or_default())
    });
    themes.dedup_by(|left, right| left.get("themeId") == right.get("themeId"));
    themes.truncate(12);
    serde_json::json!({
        "schema_version": "epiphany.incubation.v0",
        "updatedAt": now_iso(),
        "themes": themes,
    })
}

fn update_sleep_cycle(previous: Option<&Value>, incubation: &Value, allow_dream: bool) -> Value {
    let cycle_hours = previous
        .and_then(|value| value.get("cycleHours"))
        .and_then(Value::as_i64)
        .unwrap_or(4)
        .clamp(2, 12);
    let nap_duration_minutes = previous
        .and_then(|value| value.get("napDurationMinutes"))
        .and_then(Value::as_i64)
        .unwrap_or(60)
        .clamp(15, cycle_hours * 60 - 5);
    let phase_offset_minutes = previous
        .and_then(|value| value.get("phaseOffsetMinutesLocal"))
        .and_then(Value::as_i64)
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
        .get("themes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|theme| theme.get("themeId").and_then(Value::as_str))
        .take(4)
        .map(str::to_string)
        .collect::<Vec<_>>();
    let previous_dream_count = previous
        .and_then(|value| value.get("dreamCountInCurrentNap"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let dream_count = if is_napping && allow_dream {
        previous_dream_count.saturating_add(1)
    } else if is_napping {
        previous_dream_count
    } else {
        0
    };
    serde_json::json!({
        "schema_version": "epiphany.sleep_cycle.v0",
        "enabled": true,
        "cycleHours": cycle_hours,
        "napDurationMinutes": nap_duration_minutes,
        "phaseOffsetMinutesLocal": phase_offset_minutes,
        "replyMode": "sleep_rumination",
        "isNapping": is_napping,
        "currentNapStartedAt": if is_napping { Some(nap_start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)) } else { None },
        "currentNapEndsAt": if is_napping { Some(nap_end.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)) } else { None },
        "nextNapStartsAt": next_nap_start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "lastDreamAt": if is_napping && allow_dream { Some(now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)) } else { previous.and_then(|value| value.get("lastDreamAt")).cloned().and_then(|value| value.as_str().map(str::to_string)) },
        "dreamCountInCurrentNap": dream_count,
        "activeDreamThemes": active_themes,
        "lastDistillationSummary": if is_napping {
            "Sleep pass prefers memory compression, resonance cooling, and dream residue over active work."
        } else {
            "Awake pass keeps resonance/incubation fresh without speaking unless Face has a real surface reason."
        },
    })
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

fn now_iso() -> String {
    chrono::Utc::now()
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
        .replace('Z', "+00:00")
}

fn now_stamp() -> String {
    chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn round3(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}

const STOP_WORDS: &[&str] = &[
    "about", "after", "agent", "before", "being", "between", "could", "from", "have", "into",
    "lane", "memory", "more", "must", "should", "state", "than", "that", "their", "there", "this",
    "through", "when", "with", "work",
];

fn agent_id_for_role(role_id: &str) -> &'static str {
    match role_id {
        "coordinator" => "epiphany.self",
        "face" => "epiphany.face",
        "imagination" => "epiphany.imagination",
        "research" => "epiphany.eyes",
        "modeling" => "epiphany.body",
        "implementation" => "epiphany.hands",
        "verification" => "epiphany.soul",
        "reorientation" => "epiphany.life",
        _ => "epiphany.unknown",
    }
}

fn display_name_for_role(role_id: &str) -> &'static str {
    match role_id {
        "coordinator" => "Self",
        "face" => "Face",
        "imagination" => "Imagination",
        "research" => "Eyes",
        "modeling" => "Body",
        "implementation" => "Hands",
        "verification" => "Soul",
        "reorientation" => "Life",
        _ => "Unknown",
    }
}

fn initiative_speed_for_role(role_id: &str) -> f64 {
    match role_id {
        "coordinator" => 1.28,
        "face" => 1.12,
        "imagination" => 0.82,
        "research" => 0.78,
        "modeling" => 0.92,
        "implementation" => 0.74,
        "verification" => 0.88,
        "reorientation" => 1.04,
        _ => 1.0,
    }
}

fn reaction_bias_for_role(role_id: &str) -> f64 {
    match role_id {
        "coordinator" => 0.88,
        "face" => 0.84,
        "imagination" => 0.54,
        "research" => 0.62,
        "modeling" => 0.74,
        "implementation" => 0.58,
        "verification" => 0.82,
        "reorientation" => 0.86,
        _ => 0.5,
    }
}

fn interrupt_threshold_for_role(role_id: &str) -> f64 {
    match role_id {
        "coordinator" => 0.42,
        "face" => 0.52,
        "imagination" => 0.64,
        "research" => 0.58,
        "modeling" => 0.5,
        "implementation" => 0.5,
        "verification" => 0.48,
        "reorientation" => 0.44,
        _ => 0.5,
    }
}

fn participant_constraints(role_id: &str) -> Vec<&'static str> {
    let role_specific = match role_id {
        "coordinator" => {
            "Routes and reviews; must not implement, verify, or accept its own comfort."
        }
        "face" => {
            "Publicly translates agent thought into #aquarium only; must not moderate or speak outside the room."
        }
        "imagination" => "Synthesizes futures; must not adopt objectives.",
        "research" => "Scouts known work; must not turn research into procrastination.",
        "modeling" => {
            "Grows source-grounded maps and checkpoints; must not edit implementation code."
        }
        "implementation" => {
            "Touches source only with accepted guidance and verifier-readable evidence."
        }
        "verification" => "Falsifies promises; must not bless theater.",
        "reorientation" => "Protects continuity; must not fake survived context.",
        _ => "Unknown role.",
    };
    vec![
        "Runs Ghostlight-shaped persistent role memory.",
        "May improve lane memory when awake and idle.",
        "Project truth belongs in EpiphanyThreadState, not role memory.",
        role_specific,
    ]
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
            },
        )?;
        assert_eq!(work["event"]["selectedRole"], "implementation");
        assert_eq!(work["event"]["turnStatus"], "running");
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
        assert!(artifact_dir.join("native-work.completion.json").exists());
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
}
