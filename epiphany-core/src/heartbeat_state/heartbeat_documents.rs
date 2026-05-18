use cultcache_rs::DatabaseEntry;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

pub const HEARTBEAT_STATE_TYPE: &str = "epiphany.agent_heartbeat";
pub const HEARTBEAT_STATE_KEY: &str = "default";
pub const HEARTBEAT_STATE_SCHEMA_VERSION: &str = "epiphany.agent_heartbeat.v0";
pub const HEARTBEAT_COGNITION_TYPE: &str = "epiphany.agent_heartbeat_cognition";
pub const HEARTBEAT_COGNITION_KEY: &str = "default";
pub const HEARTBEAT_COGNITION_SCHEMA_VERSION: &str = "epiphany.agent_heartbeat_cognition.v0";
pub const HEARTBEAT_STATUS_SCHEMA_VERSION: &str = "epiphany.agent_heartbeat_status.v0";
pub const INITIATIVE_SCHEMA_VERSION: &str = "ghostlight.initiative_schedule.v0";
pub const VOID_ROUTINE_SCHEMA_VERSION: &str = "epiphany.void_routine.v0";

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
    pub adaptive_pacing: Option<HeartbeatAdaptivePacing>,
    #[cultcache(key = 15, default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.agent_heartbeat_cognition",
    schema = "EpiphanyHeartbeatCognitionEntry"
)]
pub struct EpiphanyHeartbeatCognitionEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub updated_at: String,
    #[cultcache(key = 2, default)]
    pub latest_run_id: Option<String>,
    #[cultcache(key = 3, default)]
    pub latest_artifact_ref: Option<String>,
    #[cultcache(key = 4, default)]
    pub source: Option<String>,
    #[cultcache(key = 5, default)]
    pub sleep_cycle: Option<Value>,
    #[cultcache(key = 6, default)]
    pub memory_resonance: Option<Value>,
    #[cultcache(key = 7, default)]
    pub incubation: Option<Value>,
    #[cultcache(key = 8, default)]
    pub thought_lanes: Option<Value>,
    #[cultcache(key = 9, default)]
    pub bridge: Option<Value>,
    #[cultcache(key = 10, default)]
    pub candidate_interventions: Option<Value>,
    #[cultcache(key = 11, default)]
    pub appraisals: Option<Value>,
    #[cultcache(key = 12, default)]
    pub reactions: Option<Value>,
    #[cultcache(key = 13, default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.agent_heartbeat",
    schema = "EpiphanyHeartbeatStateEntry"
)]
pub(super) struct LegacyHeartbeatStateWithCognition {
    #[cultcache(key = 0)]
    pub(super) schema_version: String,
    #[cultcache(key = 1)]
    pub(super) target_heartbeat_rate: f64,
    #[cultcache(key = 2)]
    pub(super) scene_clock: f64,
    #[cultcache(key = 3)]
    pub(super) selection_policy: HeartbeatSelectionPolicy,
    #[cultcache(key = 4)]
    pub(super) pacing_policy: HeartbeatPacingPolicy,
    #[cultcache(key = 5)]
    pub(super) participants: Vec<HeartbeatParticipant>,
    #[cultcache(key = 6)]
    pub(super) history: Vec<HeartbeatHistoryEvent>,
    #[cultcache(key = 7, default)]
    pub(super) sleep_cycle: Option<Value>,
    #[cultcache(key = 8, default)]
    pub(super) memory_resonance: Option<Value>,
    #[cultcache(key = 9, default)]
    pub(super) incubation: Option<Value>,
    #[cultcache(key = 10, default)]
    pub(super) thought_lanes: Option<Value>,
    #[cultcache(key = 11, default)]
    pub(super) bridge: Option<Value>,
    #[cultcache(key = 12, default)]
    pub(super) candidate_interventions: Option<Value>,
    #[cultcache(key = 13, default)]
    pub(super) appraisals: Option<Value>,
    #[cultcache(key = 14, default)]
    pub(super) reactions: Option<Value>,
    #[cultcache(key = 15, default)]
    pub(super) extra: BTreeMap<String, Value>,
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
#[serde(rename_all = "camelCase")]
pub struct HeartbeatAdaptivePacing {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    pub contract: String,
    pub pressure: f64,
    pub effective_heartbeat_rate: f64,
    pub target_concurrency: usize,
    pub running_turns: usize,
    pub active_participants: usize,
    pub signals: HeartbeatAdaptivePacingSignals,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatAdaptivePacingSignals {
    pub external_urgency: f64,
    pub max_anxiety: f64,
    pub average_anxiety: f64,
    pub max_urgency: f64,
    pub max_arousal: f64,
    pub max_thought_pressure: f64,
    pub max_reaction_intensity: f64,
    pub pending_pressure: f64,
    pub contract: String,
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
    #[serde(rename = "sceneId", default)]
    pub scene_id: Option<String>,
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
    #[serde(rename = "personalityTiming", default)]
    pub personality_timing: Option<HeartbeatPersonalityTiming>,
    #[serde(rename = "moodTiming", default)]
    pub mood_timing: Option<HeartbeatMoodTiming>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatPersonalityTiming {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    pub source: String,
    pub cooldown_multiplier: f64,
    pub work_drive: f64,
    pub handsiness: f64,
    pub caution: f64,
    pub rumination_bias: f64,
    pub basis: Vec<String>,
    pub contract: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatMoodTiming {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub cooldown_multiplier: f64,
    pub anxiety: f64,
    pub urgency: f64,
    pub arousal: f64,
    pub thought_pressure: f64,
    pub guardedness: f64,
    pub reaction_intensity: f64,
    pub contract: String,
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
