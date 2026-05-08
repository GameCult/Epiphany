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
    pub thought_lanes: Option<Value>,
    #[cultcache(key = 11, default)]
    pub bridge: Option<Value>,
    #[cultcache(key = 12, default)]
    pub candidate_interventions: Option<Value>,
    #[cultcache(key = 13, default)]
    pub appraisals: Option<Value>,
    #[cultcache(key = 14, default)]
    pub reactions: Option<Value>,
    #[cultcache(key = 15, default)]
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
    pub agent_store: Option<std::path::PathBuf>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HeartbeatPumpOptions {
    pub base_heartbeat_rate: f64,
    pub min_heartbeat_rate: f64,
    pub max_heartbeat_rate: f64,
    pub min_concurrency: usize,
    pub max_concurrency: usize,
    pub max_ticks: usize,
    pub external_urgency: f64,
    pub coordinator_action: Option<String>,
    pub target_role: Option<String>,
    pub schedule_id: String,
    pub source_scene_ref: String,
    pub agent_store: Option<std::path::PathBuf>,
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
        thought_lanes: None,
        bridge: None,
        candidate_interventions: None,
        appraisals: None,
        reactions: None,
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
            "availableActions": ["init", "tick", "pump", "complete", "status"],
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
        "thoughtLanes": state.thought_lanes,
        "bridge": state.bridge,
        "candidateInterventions": state.candidate_interventions,
        "appraisals": state.appraisals,
        "reactions": state.reactions,
        "adaptivePacing": state.extra.get("adaptivePacing"),
        "availableActions": ["init", "tick", "pump", "complete", "status", "routine"],
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
    let appraisal_profiles = collect_role_appraisal_profiles(options.agent_store.as_deref())?;
    apply_personality_timing_profiles(&mut state, &appraisal_profiles);
    let resonance = build_memory_resonance(&memory_records);
    let incubation = build_incubation(
        &state.incubation,
        &state.bridge,
        &state.candidate_interventions,
        &resonance,
        &memory_records,
    );
    let thought_lanes = build_thought_lanes(&resonance, &incubation, &memory_records);
    let bridge = build_thought_bridge(&state.bridge, &thought_lanes, &resonance, &incubation);
    let candidate_interventions = build_candidate_interventions(&bridge, &incubation);
    let appraisals =
        build_agent_appraisals(&appraisal_profiles, &thought_lanes, &incubation, &bridge);
    let reactions = build_agent_reactions(&appraisals, &bridge);
    apply_mood_timing_from_appraisals(&mut state, &appraisals);
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
        "thoughtLanes": thought_lanes,
        "bridge": bridge,
        "candidateInterventions": candidate_interventions,
        "appraisals": appraisals,
        "reactions": reactions,
        "reviewNotes": [
            "Void is reference material, not a runtime dependency.",
            "The routine mutates only typed heartbeat physiology fields; project truth and role memory mutation stay on their dedicated reviewed surfaces.",
            "Analytic and associative lanes are cognition context, not hidden authority; the bridge decides draft, speech, silence, or further incubation.",
            "Appraisal projects clustered thoughts through each agent's own personality vectors; reaction is derived from that appraisal and remains separate from state mutation.",
            "Sleep is maintenance: slow rumination, memory compression, and dream residue, not absence."
        ],
    });

    state.sleep_cycle = Some(sleep_cycle);
    state.memory_resonance = Some(resonance);
    state.incubation = Some(incubation);
    state.thought_lanes = Some(thought_lanes);
    state.bridge = Some(bridge);
    state.candidate_interventions = Some(candidate_interventions);
    state.appraisals = Some(appraisals);
    state.reactions = Some(reactions);
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
    if let Some(agent_store) = options.agent_store.as_deref() {
        let appraisal_profiles = collect_role_appraisal_profiles(Some(agent_store))?;
        apply_personality_timing_profiles(&mut state, &appraisal_profiles);
    }
    if let Some(appraisals) = state.appraisals.clone() {
        apply_mood_timing_from_appraisals(&mut state, &appraisals);
    }
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
    if let Some(appraisals) = state.appraisals.clone() {
        apply_mood_timing_from_appraisals(&mut state, &appraisals);
    }

    let pacing = adaptive_swarm_pacing(&state, &options);
    state.target_heartbeat_rate = pacing.effective_heartbeat_rate;
    state.extra.insert(
        "adaptivePacing".to_string(),
        serde_json::json!({
            "schema_version": "epiphany.adaptive_heartbeat_pacing.v0",
            "contract": "Swarm pressure controls both heartbeat tempo and concurrency. Relaxed systems sleep slow; urgent systems fill more lanes without re-waking unfinished turns.",
            "pressure": pacing.pressure,
            "effectiveHeartbeatRate": pacing.effective_heartbeat_rate,
            "targetConcurrency": pacing.target_concurrency,
            "runningTurns": pacing.running_turns,
            "activeParticipants": pacing.active_participants,
            "signals": pacing.signals,
        }),
    );

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
        extra: BTreeMap::new(),
    };
    let mut pending = pending;
    pending.extra.insert(
        "personalityCooldownMultiplier".to_string(),
        serde_json::json!(personality_cooldown_multiplier(&selected)),
    );
    pending.extra.insert(
        "moodCooldownMultiplier".to_string(),
        serde_json::json!(mood_cooldown_multiplier(&selected)),
    );
    pending.extra.insert(
        "effectiveCooldownMultiplier".to_string(),
        serde_json::json!(effective_cooldown_multiplier(&selected)),
    );
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
            "personality_cooldown_multiplier": personality_cooldown_multiplier(&selected_after),
            "mood_cooldown_multiplier": mood_cooldown_multiplier(&selected_after),
            "effective_cooldown_multiplier": effective_cooldown_multiplier(&selected_after),
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
            "{} has no actionable lane work; ruminate on role quality and prepare candidate self-memory pressure.",
            selected.display_name
        ),
            "Heartbeat slots control opportunity, not project authority.".to_string(),
            "When no coordinator work is active, idle rumination is slowed by the sleep multiplier and shaped by personality and mood cooldowns so the swarm dreams instead of thrashing.".to_string(),
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
        "personalityCooldownMultiplier": personality_cooldown_multiplier(participant),
        "moodCooldownMultiplier": mood_cooldown_multiplier(participant),
        "effectiveCooldownMultiplier": effective_cooldown_multiplier(participant),
        "personalityTiming": participant.extra.get("personalityTiming"),
        "moodTiming": participant.extra.get("moodTiming"),
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
        "personalityCooldownMultiplier": turn.extra.get("personalityCooldownMultiplier"),
        "moodCooldownMultiplier": turn.extra.get("moodCooldownMultiplier"),
        "effectiveCooldownMultiplier": turn.extra.get("effectiveCooldownMultiplier"),
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
        participant.extra.insert(
            "personalityCooldownMultiplier".to_string(),
            serde_json::json!(timing.cooldown_multiplier),
        );
        participant.extra.insert(
            "personalityTiming".to_string(),
            serde_json::json!({
                "schema_version": "epiphany.personality_timing.v0",
                "source": "state/agents.msgpack",
                "cooldownMultiplier": timing.cooldown_multiplier,
                "workDrive": timing.work_drive,
                "handsiness": timing.handsiness,
                "caution": timing.caution,
                "ruminationBias": timing.rumination_bias,
                "basis": timing.basis,
                "contract": "Cooldown is personality-shaped. Lower multipliers recover faster; higher multipliers yield the floor to other lanes."
            }),
        );
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

fn apply_mood_timing_from_appraisals(state: &mut EpiphanyHeartbeatStateEntry, appraisals: &Value) {
    let Some(items) = appraisals
        .get("participantAppraisals")
        .and_then(Value::as_array)
    else {
        return;
    };
    for participant in &mut state.participants {
        let Some(appraisal) = items
            .iter()
            .find(|item| item.get("roleId").and_then(Value::as_str) == Some(&participant.role_id))
        else {
            continue;
        };
        let emotional = appraisal
            .get("emotionalAppraisal")
            .and_then(Value::as_object);
        let urgency = emotional
            .and_then(|value| value.get("urgency"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let arousal = emotional
            .and_then(|value| value.get("arousal"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let thought_pressure = emotional
            .and_then(|value| value.get("thoughtPressure"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let guardedness = emotional
            .and_then(|value| value.get("guardedness"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let reaction_intensity = appraisal
            .pointer("/candidateImplications/reactionIntensity")
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let anxiety = (urgency * 0.32
            + arousal * 0.22
            + thought_pressure * 0.24
            + guardedness * 0.12
            + reaction_intensity * 0.10)
            .clamp(0.0, 1.0);
        let multiplier =
            (1.10 - urgency * 0.24 - anxiety * 0.38 - reaction_intensity * 0.12).clamp(0.55, 1.25);
        participant.extra.insert(
            "moodCooldownMultiplier".to_string(),
            serde_json::json!(round3(multiplier)),
        );
        participant.extra.insert(
            "moodTiming".to_string(),
            serde_json::json!({
                "schema_version": "epiphany.mood_timing.v0",
                "source": appraisal.get("appraisalId"),
                "cooldownMultiplier": round3(multiplier),
                "anxiety": round3(anxiety),
                "urgency": round3(urgency),
                "arousal": round3(arousal),
                "thoughtPressure": round3(thought_pressure),
                "guardedness": round3(guardedness),
                "reactionIntensity": round3(reaction_intensity),
                "contract": "Mood bends personality timing. Anxiety and urgency lower cooldown so the lane that needs the floor most gets it sooner."
            }),
        );
    }
}

#[derive(Clone, Debug)]
struct AdaptiveSwarmPacing {
    pressure: f64,
    effective_heartbeat_rate: f64,
    target_concurrency: usize,
    running_turns: usize,
    active_participants: usize,
    signals: Value,
}

fn adaptive_swarm_pacing(
    state: &EpiphanyHeartbeatStateEntry,
    options: &HeartbeatPumpOptions,
) -> AdaptiveSwarmPacing {
    let active_participants = state
        .participants
        .iter()
        .filter(|participant| participant.status == "active")
        .count();
    let running_turns = running_turn_count(state);
    let mood_signals = swarm_mood_signals(state);
    let external_urgency = options.external_urgency.clamp(0.0, 1.0);
    let pending_pressure = if active_participants == 0 {
        0.0
    } else {
        (running_turns as f64 / active_participants as f64).clamp(0.0, 1.0)
    };
    let pressure = [
        external_urgency,
        mood_signals.max_anxiety,
        mood_signals.max_urgency,
        mood_signals.max_reaction_intensity,
        mood_signals.max_thought_pressure,
        pending_pressure * 0.65,
    ]
    .into_iter()
    .fold(0.0_f64, f64::max)
    .clamp(0.0, 1.0);

    let base_rate = options
        .base_heartbeat_rate
        .max(state.pacing_policy.minimum_effective_rate)
        .max(0.001);
    let min_rate = options.min_heartbeat_rate.max(0.001).min(base_rate);
    let max_rate = options.max_heartbeat_rate.max(base_rate).max(min_rate);
    let pressure_curve = pressure * pressure;
    let effective_heartbeat_rate = round6(min_rate + (max_rate - min_rate) * pressure_curve);

    let max_concurrency = options
        .max_concurrency
        .max(1)
        .min(active_participants.max(1));
    let relaxed_floor = if pressure < 0.18 {
        0
    } else {
        options.min_concurrency.min(max_concurrency)
    };
    let target_concurrency = if pressure < 0.18 {
        relaxed_floor
    } else {
        let span = max_concurrency.saturating_sub(relaxed_floor);
        (relaxed_floor + (span as f64 * pressure).ceil() as usize).min(max_concurrency)
    };

    AdaptiveSwarmPacing {
        pressure: round3(pressure),
        effective_heartbeat_rate,
        target_concurrency,
        running_turns,
        active_participants,
        signals: serde_json::json!({
            "externalUrgency": round3(external_urgency),
            "maxAnxiety": round3(mood_signals.max_anxiety),
            "averageAnxiety": round3(mood_signals.average_anxiety),
            "maxUrgency": round3(mood_signals.max_urgency),
            "maxArousal": round3(mood_signals.max_arousal),
            "maxThoughtPressure": round3(mood_signals.max_thought_pressure),
            "maxReactionIntensity": round3(mood_signals.max_reaction_intensity),
            "pendingPressure": round3(pending_pressure),
            "contract": "Anxiety-like state raises tempo and concurrency; calm state lets the swarm sleep slow.",
        }),
    }
}

#[derive(Clone, Debug, Default)]
struct SwarmMoodSignals {
    max_anxiety: f64,
    average_anxiety: f64,
    max_urgency: f64,
    max_arousal: f64,
    max_thought_pressure: f64,
    max_reaction_intensity: f64,
}

fn swarm_mood_signals(state: &EpiphanyHeartbeatStateEntry) -> SwarmMoodSignals {
    let mut signals = SwarmMoodSignals::default();
    let mut anxiety_total = 0.0;
    let mut anxiety_count = 0_usize;
    for participant in &state.participants {
        let Some(mood) = participant.extra.get("moodTiming") else {
            continue;
        };
        let anxiety = number_at(mood, "/anxiety");
        let urgency = number_at(mood, "/urgency");
        let arousal = number_at(mood, "/arousal");
        let thought_pressure = number_at(mood, "/thoughtPressure");
        let reaction_intensity = number_at(mood, "/reactionIntensity");
        signals.max_anxiety = signals.max_anxiety.max(anxiety);
        signals.max_urgency = signals.max_urgency.max(urgency);
        signals.max_arousal = signals.max_arousal.max(arousal);
        signals.max_thought_pressure = signals.max_thought_pressure.max(thought_pressure);
        signals.max_reaction_intensity = signals.max_reaction_intensity.max(reaction_intensity);
        anxiety_total += anxiety;
        anxiety_count += 1;
    }
    if anxiety_count > 0 {
        signals.average_anxiety = anxiety_total / anxiety_count as f64;
    }
    signals
}

fn running_turn_count(state: &EpiphanyHeartbeatStateEntry) -> usize {
    state
        .participants
        .iter()
        .filter(|participant| is_turn_pending(participant))
        .count()
}

fn number_at(value: &Value, pointer: &str) -> f64 {
    value
        .pointer(pointer)
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
}

fn personality_cooldown_multiplier(participant: &HeartbeatParticipant) -> f64 {
    participant
        .extra
        .get("personalityCooldownMultiplier")
        .and_then(Value::as_f64)
        .unwrap_or(1.0)
        .clamp(0.25, 3.0)
}

fn mood_cooldown_multiplier(participant: &HeartbeatParticipant) -> f64 {
    participant
        .extra
        .get("moodCooldownMultiplier")
        .and_then(Value::as_f64)
        .unwrap_or(1.0)
        .clamp(0.25, 3.0)
}

fn effective_cooldown_multiplier(participant: &HeartbeatParticipant) -> f64 {
    round3(personality_cooldown_multiplier(participant) * mood_cooldown_multiplier(participant))
        .clamp(0.20, 4.0)
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
                "leftMemoryKind": left.memory_kind,
                "leftSummary": left.summary,
                "rightRole": right.role_id,
                "rightMemoryId": right.memory_id,
                "rightMemoryKind": right.memory_kind,
                "rightSummary": right.summary,
                "strength": strength,
                "sharedTokens": shared_tokens(&left.tokens, &right.tokens),
                "sourceRoles": [left.role_id, right.role_id],
                "sourceKinds": [left.memory_kind, right.memory_kind],
                "evidenceRefs": [left.memory_id, right.memory_id],
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
    bridge: &Option<Value>,
    candidate_interventions: &Option<Value>,
    resonance: &Value,
    records: &[RoleMemoryRecord],
) -> Value {
    let previous_themes = previous
        .as_ref()
        .and_then(|value| value.get("themes"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let source_coverage = build_source_coverage(records);
    let mut themes = Vec::new();
    for pair in resonance
        .get("pairs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(6)
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
        let source_roles = unique_strings_from_value(pair.get("sourceRoles"));
        let source_kinds = unique_strings_from_value(pair.get("sourceKinds"));
        let source_memory_ids = unique_strings_from_value(pair.get("evidenceRefs"));
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
        let prior_maturation = previous_theme
            .and_then(|theme| theme.get("maturation"))
            .and_then(Value::as_f64)
            .unwrap_or(0.32);
        let strength = pair.get("strength").and_then(Value::as_f64).unwrap_or(0.0);
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
        themes.push(serde_json::json!({
            "themeId": previous_theme
                .and_then(|theme| theme.get("themeId"))
                .and_then(Value::as_str)
                .unwrap_or(theme_id.as_str()),
            "summary": summary,
            "strength": round3(strength),
            "source": "memory_resonance",
            "sourceRoles": source_roles,
            "sourceKinds": source_kinds,
            "sourceMemoryIds": source_memory_ids,
            "supportCount": support_count,
            "evidenceDiversity": round3(evidence_diversity),
            "explorationBonus": round3(exploration_bonus),
            "novelty": novelty,
            "noveltyToSelf": round3(novelty_to_self),
            "noveltyToRoom": round3(novelty_to_room),
            "maturation": maturation,
            "desireToSpeak": desire_to_speak,
            "saturationScore": round3(saturation.score),
            "recentMatchCount": saturation.recent_match_count,
            "refractoryPenalty": round3(refractory_penalty),
            "priorityScore": priority_score,
            "status": status,
            "latentQuestion": build_incubation_question(&source_roles, &source_kinds),
            "whyItPulls": build_incubation_attraction(
                &source_roles,
                &source_kinds,
                novelty_to_self,
                evidence_diversity,
            ),
            "holdingCloseBecause": build_incubation_holding_line(
                status,
                saturation.score,
                novelty_to_self,
                exploration_bonus,
            ),
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
            "sourceRoles": [strongest.role_id.clone()],
            "sourceKinds": [strongest.memory_kind.clone()],
            "sourceMemoryIds": [strongest.memory_id.clone()],
            "supportCount": 1,
            "evidenceDiversity": 0.28,
            "explorationBonus": 0.18,
            "novelty": 0.62,
            "noveltyToSelf": 0.62,
            "noveltyToRoom": 0.58,
            "maturation": round3((strongest.salience * strongest.confidence).clamp(0.18, 0.72)),
            "desireToSpeak": round3((strongest.salience * strongest.confidence * 0.6).clamp(0.12, 0.55)),
            "saturationScore": 0.0,
            "recentMatchCount": 0,
            "refractoryPenalty": 0.0,
            "priorityScore": round3((strongest.salience * strongest.confidence * 0.7).clamp(0.14, 0.7)),
            "status": "incubating",
            "latentQuestion": "Does this hot memory deserve real follow-up, or is it just the loudest ember in the tray?",
            "whyItPulls": "One strong memory is enough to seed a thought, but not enough to rule the room by default.",
            "holdingCloseBecause": "This is a seed, not a verdict. Give it one more pass before it starts issuing prophecies.",
            "updatedAt": now_iso(),
        }));
    }
    themes.sort_by(|left, right| {
        right["priorityScore"]
            .as_f64()
            .unwrap_or_default()
            .total_cmp(&left["priorityScore"].as_f64().unwrap_or_default())
            .then_with(|| {
                right["strength"]
                    .as_f64()
                    .unwrap_or_default()
                    .total_cmp(&left["strength"].as_f64().unwrap_or_default())
            })
    });
    themes.truncate(12);
    serde_json::json!({
        "schema_version": "epiphany.incubation.v0",
        "updatedAt": now_iso(),
        "sourceCoverage": source_coverage,
        "lastIncubationSummary": themes.first().map(|theme| {
            format!(
                "Strongest incubating seam: {} ({}, self={:.2}, room={:.2}, speak={:.2}).",
                theme.get("themeId").and_then(Value::as_str).unwrap_or("unnamed-theme"),
                theme.get("status").and_then(Value::as_str).unwrap_or("incubating"),
                theme.get("noveltyToSelf").and_then(Value::as_f64).unwrap_or(0.0),
                theme.get("noveltyToRoom").and_then(Value::as_f64).unwrap_or(0.0),
                theme.get("desireToSpeak").and_then(Value::as_f64).unwrap_or(0.0),
            )
        }).unwrap_or_else(|| "No incubating thought currently has enough connective tissue to justify special treatment.".to_string()),
        "themes": themes,
    })
}

fn build_thought_lanes(
    resonance: &Value,
    incubation: &Value,
    records: &[RoleMemoryRecord],
) -> Value {
    let analytic_threads = resonance
        .get("pairs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(4)
        .enumerate()
        .map(|(index, pair)| {
            let left_role = pair.get("leftRole").and_then(Value::as_str).unwrap_or("left");
            let right_role = pair.get("rightRole").and_then(Value::as_str).unwrap_or("right");
            let strength = pair.get("strength").and_then(Value::as_f64).unwrap_or(0.0);
            serde_json::json!({
                "threadId": format!("analytic-{left_role}-{right_role}-{index}"),
                "topic": format!("{left_role}/{right_role} evidence seam"),
                "claim": format!("{} and {} share a recurring memory edge; inspect whether this changes lane routing or evidence expectations.", display_name_for_role(left_role), display_name_for_role(right_role)),
                "evidenceRefs": [
                    pair.get("leftMemoryId").and_then(Value::as_str).unwrap_or_default(),
                    pair.get("rightMemoryId").and_then(Value::as_str).unwrap_or_default()
                ],
                "salience": round3(strength),
                "confidence": round3((0.55 + strength).min(0.95)),
                "desireToAct": round3((strength * 1.4).min(0.85)),
                "counterweight": "Shared vocabulary is not proof of shared truth; verify against artifacts before changing project state.",
                "lastTouchedAt": now_iso(),
            })
        })
        .collect::<Vec<_>>();

    let associative_threads = incubation
        .get("themes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(4)
        .enumerate()
        .map(|(index, theme)| {
            let topic = theme
                .get("themeId")
                .and_then(Value::as_str)
                .unwrap_or("incubating-theme");
            let strength = theme.get("strength").and_then(Value::as_f64).unwrap_or(0.0);
            serde_json::json!({
                "threadId": format!("associative-{topic}-{index}"),
                "topic": topic,
                "claim": theme.get("summary").and_then(Value::as_str).unwrap_or("An incubating thought wants another pass before it speaks."),
                "sourceThemeId": topic,
                "novelty": theme.get("novelty").cloned().unwrap_or_else(|| serde_json::json!(round3((0.42 + strength).min(0.92)))),
                "roomRelevance": theme.get("noveltyToRoom").cloned().unwrap_or_else(|| serde_json::json!(round3((0.35 + strength).min(0.88)))),
                "desireToSpeak": theme.get("desireToSpeak").cloned().unwrap_or_else(|| serde_json::json!(round3((strength * 1.25).min(0.8)))),
                "status": theme.get("status").cloned().unwrap_or_else(|| serde_json::json!("incubating")),
                "counterweight": "A theme that keeps returning may be signal, obsession, or stale echo; the bridge must decide.",
                "lastTouchedAt": now_iso(),
            })
        })
        .collect::<Vec<_>>();

    let seed_threads = if analytic_threads.is_empty() && associative_threads.is_empty() {
        records
            .first()
            .map(|record| {
                vec![serde_json::json!({
                    "threadId": format!("analytic-{}-seed", record.role_id),
                    "topic": format!("{} strongest memory", record.role_id),
                    "claim": record.summary,
                    "evidenceRefs": [record.memory_id],
                    "salience": round3(record.salience),
                    "confidence": round3(record.confidence),
                    "desireToAct": 0.2,
                    "counterweight": "One hot memory is only a seed; do not let it annex the whole mind.",
                    "lastTouchedAt": now_iso(),
                })]
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    serde_json::json!({
        "schema_version": "epiphany.cognition_lanes.v0",
        "updatedAt": now_iso(),
        "analytic": {
            "description": "Literal, evidence-facing lane: what is happening, what constraints matter, what action is justified.",
            "activeThreads": if analytic_threads.is_empty() { seed_threads } else { analytic_threads },
        },
        "associative": {
            "description": "Pattern-facing lane: what this rhymes with, what seam is ripening, what surprising branch may be worth a later retrieval hop.",
            "activeThreads": associative_threads,
        },
    })
}

fn build_thought_bridge(
    previous: &Option<Value>,
    thought_lanes: &Value,
    resonance: &Value,
    incubation: &Value,
) -> Value {
    let analytic_count = thought_lanes
        .pointer("/analytic/activeThreads")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default();
    let associative_count = thought_lanes
        .pointer("/associative/activeThreads")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default();
    let resonance_count = resonance
        .get("pairs")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default();
    let strongest_theme = incubation
        .get("themes")
        .and_then(Value::as_array)
        .and_then(|themes| themes.first())
        .cloned();
    let strongest_status = strongest_theme
        .as_ref()
        .and_then(|theme| theme.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("incubating");
    let strongest_novelty_to_self = strongest_theme
        .as_ref()
        .and_then(|theme| theme.get("noveltyToSelf"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let strongest_novelty_to_room = strongest_theme
        .as_ref()
        .and_then(|theme| theme.get("noveltyToRoom"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let strongest_saturation = strongest_theme
        .as_ref()
        .and_then(|theme| theme.get("saturationScore"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let strongest_refractory = strongest_theme
        .as_ref()
        .and_then(|theme| theme.get("refractoryPenalty"))
        .and_then(Value::as_f64)
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
        .and_then(|value| value.get("recentSyntheses"))
        .or_else(|| {
            previous
                .as_ref()
                .and_then(|value| value.get("recent_syntheses"))
        })
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    syntheses.push(serde_json::json!({
        "timestamp": now_iso(),
        "summary": if let Some(theme) = &strongest_theme {
            bridge_summary(theme, speak_decision)
        } else {
            "No strong convergence yet; hold the lanes open without forcing speech.".to_string()
        },
        "dominantTopics": strongest_theme
            .as_ref()
            .and_then(|theme| theme.get("themeId"))
            .cloned()
            .map(|topic| vec![topic])
            .unwrap_or_default(),
        "laneBalance": lane_balance,
        "speakDecision": speak_decision,
        "themeStatus": strongest_status,
        "noveltyToSelf": round3(strongest_novelty_to_self),
        "noveltyToRoom": round3(strongest_novelty_to_room),
        "saturationScore": round3(strongest_saturation),
        "saturationNote": synthesis_saturation_note(strongest_status, strongest_saturation, strongest_novelty_to_self),
    }));
    syntheses.reverse();
    syntheses.truncate(8);
    syntheses.reverse();
    let source_coverage = incubation
        .get("sourceCoverage")
        .cloned()
        .unwrap_or_else(empty_source_coverage);
    let topic_saturation = topic_saturation_from_syntheses_and_themes(
        &syntheses,
        incubation
            .get("themes")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[]),
    );
    let refractory_topics = refractory_topics_from_themes(
        previous,
        incubation
            .get("themes")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[]),
    );

    serde_json::json!({
        "schema_version": "epiphany.cognition_bridge.v0",
        "updatedAt": now_iso(),
        "recentSyntheses": syntheses,
        "sourceCoverage": source_coverage,
        "topicSaturation": topic_saturation,
        "refractoryTopics": refractory_topics,
        "unresolvedTensions": [{
            "topic": "thought authority boundary",
            "summary": "Cognition lanes may shape attention and drafts, but only reviewed Epiphany state surfaces change project truth.",
            "openedAt": now_iso(),
        }],
        "decision": {
            "laneBalance": lane_balance,
            "speakDecision": speak_decision,
            "reason": bridge_decision_reason(
                strongest_status,
                strongest_novelty_to_self,
                strongest_novelty_to_room,
                strongest_saturation,
                strongest_refractory,
                speak_decision,
            ),
        },
    })
}

fn build_candidate_interventions(bridge: &Value, incubation: &Value) -> Value {
    let decision = bridge
        .pointer("/decision/speakDecision")
        .and_then(Value::as_str)
        .unwrap_or("silence");
    let strongest_theme = incubation
        .get("themes")
        .and_then(Value::as_array)
        .and_then(|themes| themes.first());
    let strongest_status = strongest_theme
        .as_ref()
        .and_then(|theme| theme.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("incubating");
    let items = if decision == "draft" && strongest_status == "ripe" {
        strongest_theme
            .map(|theme| {
                vec![serde_json::json!({
                    "interventionId": format!("candidate-{}", theme.get("themeId").and_then(Value::as_str).unwrap_or("theme")),
                    "summary": "Possible Aquarium-facing thought-weather note",
                    "draft": format!("I keep seeing {} rhyme across the swarm; this one finally has enough blood to inspect in the open.", theme.get("themeId").and_then(Value::as_str).unwrap_or("an unnamed seam")),
                    "decision": decision,
                    "requiresFace": true,
                    "requiresReview": true,
                    "noveltyToRoom": theme.get("noveltyToRoom").cloned().unwrap_or(Value::Null),
                    "saturationScore": theme.get("saturationScore").cloned().unwrap_or(Value::Null),
                    "createdAt": now_iso(),
                })]
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    serde_json::json!({
        "schema_version": "epiphany.candidate_interventions.v0",
        "updatedAt": now_iso(),
        "items": items,
    })
}

fn build_agent_appraisals(
    profiles: &[RoleAppraisalProfile],
    thought_lanes: &Value,
    incubation: &Value,
    bridge: &Value,
) -> Value {
    let thought_tokens = cognition_tokens(thought_lanes, incubation, bridge);
    let thought_pressure = thought_pressure(thought_lanes, incubation);
    let bridge_decision = bridge
        .pointer("/decision/speakDecision")
        .and_then(Value::as_str)
        .unwrap_or("silence");
    let focus = incubation
        .get("themes")
        .and_then(Value::as_array)
        .and_then(|themes| themes.first())
        .and_then(|theme| theme.get("themeId"))
        .and_then(Value::as_str)
        .unwrap_or("no-active-theme");
    let appraisals = profiles
        .iter()
        .map(|profile| {
            let projection = personality_projection(profile, &thought_tokens);
            let alignment = projection
                .iter()
                .filter_map(|item| item.get("projection").and_then(Value::as_f64))
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
            serde_json::json!({
                "schema_version": "epiphany.agent_thought_appraisal.v0",
                "appraisalId": format!("appraisal-{}-{}", profile.role_id, stable_theme_suffix(focus)),
                "reviewStatus": "generated_unreviewed",
                "participantAgentId": profile.agent_id,
                "roleId": profile.role_id,
                "currentCharacterStateRef": format!("state/agents.msgpack#{}", profile.role_id),
                "thoughtClusterRef": focus,
                "participantLocalContext": {
                    "displayName": profile.display_name,
                    "values": profile.values,
                    "reactivity": profile.reactivity,
                    "plasticity": profile.plasticity,
                    "expressiveness": profile.expressiveness,
                    "guardedness": profile.guardedness,
                },
                "observableThoughtSummary": strongest_thought_summary(thought_lanes, incubation),
                "personalityProjection": projection,
                "interpretation": format!("{} appraises {} through its current personality vector; reaction should follow this appraisal rather than a global mood knob.", profile.display_name, focus),
                "emotionalAppraisal": {
                    "valence": valence,
                    "arousal": arousal,
                    "urgency": urgency,
                    "curiosity": curiosity,
                    "guardedness": guardedness,
                    "thoughtPressure": round3(thought_pressure),
                },
                "interpretationLabel": label,
                "confidenceNotes": "Deterministic first-pass appraisal from typed role personality vectors and clustered thought state; useful as reaction guidance, not reviewed truth.",
                "candidateImplications": {
                    "reactionMode": reaction_mode(&label, bridge_decision),
                    "reactionIntensity": round3((urgency * 0.55 + arousal * 0.3 + curiosity * 0.15).clamp(0.0, 1.0)),
                    "shouldSpeak": bridge_decision == "draft" && profile.role_id == "face" && guardedness < 0.75,
                    "shouldIncubate": bridge_decision != "silence" && guardedness >= 0.55,
                },
                "review": {
                    "acceptedForMutation": false,
                    "rationale": "Appraisal may steer reaction and display; state mutation still requires the explicit selfPatch or project-state review path.",
                },
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "schema_version": "epiphany.agent_thought_appraisals.v0",
        "updatedAt": now_iso(),
        "thoughtClusterRef": focus,
        "participantAppraisals": appraisals,
    })
}

fn build_agent_reactions(appraisals: &Value, bridge: &Value) -> Value {
    let bridge_decision = bridge
        .pointer("/decision/speakDecision")
        .and_then(Value::as_str)
        .unwrap_or("silence");
    let reactions = appraisals
        .get("participantAppraisals")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|appraisal| {
            let role_id = appraisal
                .get("roleId")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let arousal = appraisal
                .pointer("/emotionalAppraisal/arousal")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            let guardedness = appraisal
                .pointer("/emotionalAppraisal/guardedness")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            let curiosity = appraisal
                .pointer("/emotionalAppraisal/curiosity")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            let mode = appraisal
                .pointer("/candidateImplications/reactionMode")
                .and_then(Value::as_str)
                .unwrap_or("hold");
            let intensity = appraisal
                .pointer("/candidateImplications/reactionIntensity")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            serde_json::json!({
                "reactionId": format!("reaction-{}-{}", role_id, now_stamp()),
                "roleId": role_id,
                "participantAgentId": appraisal.get("participantAgentId"),
                "appraisalId": appraisal.get("appraisalId"),
                "mode": mode,
                "moodLabel": mood_label(arousal, guardedness, curiosity),
                "intensity": round3(intensity),
                "bridgeDecision": bridge_decision,
                "surface": if role_id == "face" { "aquarium" } else { "internal" },
                "recommendedUse": reaction_recommended_use(role_id, mode),
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "schema_version": "epiphany.agent_reactions.v0",
        "updatedAt": now_iso(),
        "reactions": reactions,
        "contract": "Reaction is appraisal output. It may pace, color, draft, or display behavior; it does not mutate state without review.",
    })
}

fn cognition_tokens(thought_lanes: &Value, incubation: &Value, bridge: &Value) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    collect_json_tokens(thought_lanes, &mut tokens);
    collect_json_tokens(incubation, &mut tokens);
    collect_json_tokens(bridge, &mut tokens);
    tokens
}

fn collect_json_tokens(value: &Value, tokens: &mut BTreeSet<String>) {
    match value {
        Value::String(text) => tokens.extend(summary_tokens(text)),
        Value::Array(items) => {
            for item in items {
                collect_json_tokens(item, tokens);
            }
        }
        Value::Object(object) => {
            for (key, value) in object {
                tokens.extend(summary_tokens(key));
                collect_json_tokens(value, tokens);
            }
        }
        _ => {}
    }
}

fn thought_pressure(thought_lanes: &Value, incubation: &Value) -> f64 {
    let analytic = max_number_at(thought_lanes, "/analytic/activeThreads", "desireToAct");
    let associative = max_number_at(thought_lanes, "/associative/activeThreads", "desireToSpeak");
    let theme = max_number_at(incubation, "/themes", "strength");
    (analytic * 0.35 + associative * 0.25 + theme * 0.4).clamp(0.0, 1.0)
}

fn max_number_at(value: &Value, pointer: &str, key: &str) -> f64 {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| item.get(key).and_then(Value::as_f64))
        .max_by(f64::total_cmp)
        .unwrap_or(0.0)
}

fn personality_projection(
    profile: &RoleAppraisalProfile,
    thought_tokens: &BTreeSet<String>,
) -> Vec<Value> {
    let mut scored = profile
        .traits
        .iter()
        .map(|item| {
            let trait_tokens = summary_tokens(&format!("{} {}", item.group, item.name));
            let overlap = token_overlap(&trait_tokens, thought_tokens);
            let projection = round3((item.weight * (0.55 + overlap * 1.8)).clamp(0.0, 1.0));
            serde_json::json!({
                "group": item.group,
                "name": item.name,
                "activation": round3(item.activation),
                "plasticity": round3(item.plasticity),
                "tokenOverlap": round3(overlap),
                "projection": projection,
            })
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        right["projection"]
            .as_f64()
            .unwrap_or_default()
            .total_cmp(&left["projection"].as_f64().unwrap_or_default())
    });
    scored.truncate(6);
    scored
}

fn strongest_thought_summary(thought_lanes: &Value, incubation: &Value) -> String {
    incubation
        .get("themes")
        .and_then(Value::as_array)
        .and_then(|themes| themes.first())
        .and_then(|theme| theme.get("summary"))
        .and_then(Value::as_str)
        .or_else(|| {
            thought_lanes
                .pointer("/analytic/activeThreads")
                .and_then(Value::as_array)
                .and_then(|threads| threads.first())
                .and_then(|thread| thread.get("claim"))
                .and_then(Value::as_str)
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
        ("face", "draft") => "Prepare a reviewed Aquarium-facing draft; do not post automatically.",
        (_, "hold_and_verify") => "Bias toward verifier/modeler review before expression.",
        (_, "inspect") => {
            "Bias the next heartbeat toward a bounded retrieval or modeling inspection."
        }
        (_, "sleep_ruminate") => "Let this organ sleep-ruminate unless real work arrives.",
        _ => "Keep the thought incubating and visible in Aquarium.",
    }
}

fn topic_saturation_from_syntheses_and_themes(syntheses: &[Value], themes: &[Value]) -> Vec<Value> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for synthesis in syntheses {
        for topic in synthesis
            .get("dominantTopics")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            *counts.entry(topic.to_string()).or_default() += 1;
        }
    }
    for theme in themes {
        if let Some(topic) = theme.get("themeId").and_then(Value::as_str) {
            let bonus = if matches!(
                theme.get("status").and_then(Value::as_str),
                Some("refractory" | "stalled")
            ) {
                2
            } else {
                1
            };
            *counts.entry(topic.to_string()).or_default() += bonus;
        }
    }
    counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(topic, count)| {
            serde_json::json!({
                "topic": topic,
                "dominance": round3((count as f64 / syntheses.len().max(1) as f64).min(1.0)),
                "recentMentions": count,
                "coolingAdvice": "Require fresh evidence or a new angle before surfacing this topic again.",
            })
        })
        .collect()
}

fn refractory_topics_from_themes(previous_bridge: &Option<Value>, themes: &[Value]) -> Vec<Value> {
    let previous_topics = previous_bridge
        .as_ref()
        .and_then(|value| value.get("refractoryTopics"))
        .or_else(|| {
            previous_bridge
                .as_ref()
                .and_then(|value| value.get("refractory_topics"))
        })
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let now = chrono::Utc::now();
    themes
        .iter()
        .filter_map(|theme| {
            let status = theme.get("status").and_then(Value::as_str).unwrap_or("incubating");
            let saturation = theme
                .get("saturationScore")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            if !matches!(status, "refractory" | "stalled") && saturation < 0.62 {
                return None;
            }
            let topic = theme
                .get("themeId")
                .and_then(Value::as_str)
                .unwrap_or("unnamed-theme");
            let previous_penalty = previous_topics
                .iter()
                .find(|entry| {
                    theme_similarity(
                        topic,
                        "",
                        entry.get("topic").and_then(Value::as_str).unwrap_or(""),
                        "",
                    ) >= 0.48
                })
                .and_then(|entry| entry.get("penalty"))
                .and_then(Value::as_f64)
                .unwrap_or(0.18);
            let penalty = round3(
                theme
                    .get("refractoryPenalty")
                    .and_then(Value::as_f64)
                    .unwrap_or(previous_penalty)
                    .max(previous_penalty),
            );
            let hours = if penalty >= 0.28 {
                4
            } else if penalty >= 0.20 {
                3
            } else {
                2
            };
            Some(serde_json::json!({
                "topic": topic,
                "penalty": penalty,
                "coolsUntil": (now + Duration::hours(hours)).to_rfc3339_opts(chrono::SecondsFormat::Secs, false).replace('Z', "+00:00"),
                "reason": build_refractory_reason(theme),
                "lastTriggeredAt": now_iso(),
            }))
        })
        .take(6)
        .collect()
}

fn build_source_coverage(records: &[RoleMemoryRecord]) -> Value {
    let mut role_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut kind_counts: BTreeMap<String, usize> = BTreeMap::new();
    for record in records {
        *role_counts.entry(record.role_id.clone()).or_default() += 1;
        *kind_counts.entry(record.memory_kind.clone()).or_default() += 1;
    }
    serde_json::json!({
        "schema_version": "epiphany.source_coverage.v0",
        "updatedAt": now_iso(),
        "roles": role_counts.into_iter().map(|(role_id, count)| serde_json::json!({
            "roleId": role_id,
            "count": count,
        })).collect::<Vec<_>>(),
        "memoryKinds": kind_counts.into_iter().map(|(kind, count)| serde_json::json!({
            "kind": kind,
            "count": count,
        })).collect::<Vec<_>>(),
    })
}

fn empty_source_coverage() -> Value {
    serde_json::json!({
        "schema_version": "epiphany.source_coverage.v0",
        "updatedAt": now_iso(),
        "roles": [],
        "memoryKinds": [],
    })
}

fn unique_strings_from_value(value: Option<&Value>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|item| seen.insert((*item).to_string()))
        .map(str::to_string)
        .collect()
}

fn best_matching_theme<'a>(
    previous_themes: &'a [Value],
    theme_id: &str,
    summary: &str,
    source_memory_ids: &[String],
) -> Option<&'a Value> {
    let best =
        previous_themes.iter().max_by(|left, right| {
            theme_match_score(theme_id, summary, source_memory_ids, left).total_cmp(
                &theme_match_score(theme_id, summary, source_memory_ids, right),
            )
        })?;
    (theme_match_score(theme_id, summary, source_memory_ids, best) >= 0.42).then_some(best)
}

fn previous_support_count(previous_theme: Option<&Value>) -> usize {
    previous_theme
        .and_then(|theme| theme.get("supportCount"))
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize
}

fn novelty_to_self(
    theme_id: &str,
    summary: &str,
    source_memory_ids: &[String],
    previous_themes: &[Value],
    bridge: &Option<Value>,
) -> f64 {
    let mut strongest_match = 0.0_f64;
    for theme in previous_themes {
        strongest_match = strongest_match.max(
            theme_similarity(
                theme_id,
                summary,
                theme.get("themeId").and_then(Value::as_str).unwrap_or(""),
                theme.get("summary").and_then(Value::as_str).unwrap_or(""),
            )
            .max(overlap_ratio_strings(
                source_memory_ids,
                &unique_strings_from_value(theme.get("sourceMemoryIds")),
            )),
        );
    }
    for synthesis in bridge
        .as_ref()
        .and_then(|value| value.get("recentSyntheses"))
        .or_else(|| {
            bridge
                .as_ref()
                .and_then(|value| value.get("recent_syntheses"))
        })
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(6)
    {
        strongest_match = strongest_match.max(theme_similarity(
            theme_id,
            summary,
            synthesis
                .get("dominantTopics")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" / ")
                .as_str(),
            synthesis
                .get("summary")
                .and_then(Value::as_str)
                .unwrap_or(""),
        ));
    }
    (1.0 - strongest_match).clamp(0.0, 1.0)
}

fn novelty_to_room(
    theme_id: &str,
    summary: &str,
    previous_candidate_interventions: &Option<Value>,
    bridge: &Option<Value>,
) -> f64 {
    let mut score = 0.64_f64;
    for intervention in previous_candidate_interventions
        .as_ref()
        .and_then(|value| value.get("items"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(8)
    {
        let similarity = theme_similarity(
            theme_id,
            summary,
            intervention
                .get("interventionId")
                .and_then(Value::as_str)
                .unwrap_or(""),
            intervention
                .get("draft")
                .and_then(Value::as_str)
                .unwrap_or(""),
        );
        if similarity >= 0.42 {
            return 0.22;
        }
    }
    for synthesis in bridge
        .as_ref()
        .and_then(|value| value.get("recentSyntheses"))
        .or_else(|| {
            bridge
                .as_ref()
                .and_then(|value| value.get("recent_syntheses"))
        })
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(6)
    {
        let similarity = theme_similarity(
            theme_id,
            summary,
            synthesis
                .get("dominantTopics")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" / ")
                .as_str(),
            synthesis
                .get("summary")
                .and_then(Value::as_str)
                .unwrap_or(""),
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
    previous_themes: &[Value],
    bridge: &Option<Value>,
    support_count: usize,
) -> SaturationMetrics {
    let mut recent_match_count = 0_usize;
    for theme in previous_themes {
        let similarity = theme_similarity(
            theme_id,
            summary,
            theme.get("themeId").and_then(Value::as_str).unwrap_or(""),
            theme.get("summary").and_then(Value::as_str).unwrap_or(""),
        )
        .max(overlap_ratio_strings(
            source_memory_ids,
            &unique_strings_from_value(theme.get("sourceMemoryIds")),
        ));
        if similarity >= 0.42 {
            recent_match_count += 1;
        }
    }
    for synthesis in bridge
        .as_ref()
        .and_then(|value| value.get("recentSyntheses"))
        .or_else(|| {
            bridge
                .as_ref()
                .and_then(|value| value.get("recent_syntheses"))
        })
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(5)
    {
        let similarity = theme_similarity(
            theme_id,
            summary,
            synthesis
                .get("dominantTopics")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" / ")
                .as_str(),
            synthesis
                .get("summary")
                .and_then(Value::as_str)
                .unwrap_or(""),
        );
        if similarity >= 0.42 {
            recent_match_count += 1;
        }
    }
    let existing_topic_saturation = bridge
        .as_ref()
        .and_then(|value| value.get("topicSaturation"))
        .or_else(|| {
            bridge
                .as_ref()
                .and_then(|value| value.get("topic_saturation"))
        })
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let similarity = theme_similarity(
                theme_id,
                summary,
                entry.get("topic").and_then(Value::as_str).unwrap_or(""),
                "",
            );
            (similarity >= 0.42).then(|| {
                entry
                    .get("dominance")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0)
            })
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

fn refractory_penalty(theme_id: &str, summary: &str, bridge: &Option<Value>) -> f64 {
    let mut penalty = 0.0_f64;
    let now = chrono::Utc::now();
    for topic in bridge
        .as_ref()
        .and_then(|value| value.get("refractoryTopics"))
        .or_else(|| {
            bridge
                .as_ref()
                .and_then(|value| value.get("refractory_topics"))
        })
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let cools_until = topic
            .get("coolsUntil")
            .and_then(Value::as_str)
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&chrono::Utc));
        if cools_until.is_some_and(|deadline| deadline < now) {
            continue;
        }
        let similarity = theme_similarity(
            theme_id,
            summary,
            topic.get("topic").and_then(Value::as_str).unwrap_or(""),
            topic.get("reason").and_then(Value::as_str).unwrap_or(""),
        );
        if similarity >= 0.48 {
            penalty = penalty
                .max(topic.get("penalty").and_then(Value::as_f64).unwrap_or(0.18) * similarity);
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
    source_coverage: &Value,
) -> f64 {
    let mut scores = Vec::new();
    for role_id in source_roles {
        scores.push(inverse_coverage_weight(
            source_coverage.get("roles"),
            "roleId",
            role_id,
        ));
    }
    for kind in source_kinds {
        scores.push(inverse_coverage_weight(
            source_coverage.get("memoryKinds"),
            "kind",
            kind,
        ));
    }
    if scores.is_empty() {
        return 0.18;
    }
    average(scores.into_iter()).unwrap_or(0.18).clamp(0.0, 1.0)
}

fn inverse_coverage_weight(entries: Option<&Value>, key: &str, needle: &str) -> f64 {
    let count = entries
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|entry| entry.get(key).and_then(Value::as_str) == Some(needle))
        .and_then(|entry| entry.get("count"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
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

fn build_refractory_reason(theme: &Value) -> String {
    let topic = theme
        .get("themeId")
        .and_then(Value::as_str)
        .unwrap_or("this seam");
    let novelty_to_self = theme
        .get("noveltyToSelf")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let saturation_score = theme
        .get("saturationScore")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
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

fn bridge_summary(theme: &Value, speak_decision: &str) -> String {
    let topic = theme
        .get("themeId")
        .and_then(Value::as_str)
        .unwrap_or("an unnamed theme");
    let status = theme
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("incubating");
    let novelty_to_self = theme
        .get("noveltyToSelf")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let novelty_to_room = theme
        .get("noveltyToRoom")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
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
    theme: &Value,
) -> f64 {
    theme_similarity(
        theme_id,
        summary,
        theme.get("themeId").and_then(Value::as_str).unwrap_or(""),
        theme.get("summary").and_then(Value::as_str).unwrap_or(""),
    )
    .max(overlap_ratio_strings(
        source_memory_ids,
        &unique_strings_from_value(theme.get("sourceMemoryIds")),
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
                agent_store: None,
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
}
