use cultcache_rs::DatabaseEntry;
use serde::Deserialize;
use serde::Serialize;

pub const HEARTBEAT_STATE_TYPE: &str = "epiphany.agent_heartbeat";
pub const HEARTBEAT_STATE_KEY: &str = "default";
pub const HEARTBEAT_STATE_SCHEMA_VERSION: &str = "epiphany.agent_heartbeat.v1";
pub const HEARTBEAT_STATUS_SCHEMA_VERSION: &str = "epiphany.agent_heartbeat_status.v0";
pub const INITIATIVE_SCHEMA_VERSION: &str = "epiphany.heartbeat_schedule.v0";
pub const HEARTBEAT_STALE_TURN_REPAIR_TYPE: &str = "epiphany.heartbeat.stale_turn_repair";
pub const HEARTBEAT_STALE_TURN_REPAIR_SCHEMA_VERSION: &str =
    "epiphany.heartbeat.stale_turn_repair.v0";
pub const HEARTBEAT_STALE_TURN_REPAIR_LATEST_KEY: &str = "heartbeat/stale-turn-repair/latest";
pub const HEARTBEAT_ARTIFACT_RETENTION_PLAN_TYPE: &str =
    "epiphany.heartbeat.artifact_retention_plan";
pub const HEARTBEAT_ARTIFACT_RETENTION_PLAN_SCHEMA_VERSION: &str =
    "epiphany.heartbeat.artifact_retention_plan.v0";
pub const HEARTBEAT_ARTIFACT_RETENTION_PLAN_LATEST_KEY: &str =
    "heartbeat/artifact-retention-plan/latest";
pub const HEARTBEAT_ARTIFACT_RETENTION_RECEIPT_TYPE: &str =
    "epiphany.heartbeat.artifact_retention_receipt";
pub const HEARTBEAT_ARTIFACT_RETENTION_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.heartbeat.artifact_retention_receipt.v0";
pub const HEARTBEAT_ARTIFACT_RETENTION_RECEIPT_LATEST_KEY: &str =
    "heartbeat/artifact-retention-receipt/latest";

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
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.heartbeat.stale_turn_repair",
    schema = "EpiphanyHeartbeatStaleTurnRepairReceipt"
)]
pub struct EpiphanyHeartbeatStaleTurnRepairReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub repaired_at_utc: String,
    #[cultcache(key = 3)]
    pub role_id: String,
    #[cultcache(key = 4)]
    pub agent_id: String,
    #[cultcache(key = 5)]
    pub action_id: String,
    #[cultcache(key = 6)]
    pub schedule_id: String,
    #[cultcache(key = 7)]
    pub started_at_utc: String,
    #[cultcache(key = 8)]
    pub stale_age_seconds: i64,
    #[cultcache(key = 9)]
    pub reason: String,
    #[cultcache(key = 10)]
    pub resulting_status: String,
    #[cultcache(key = 11)]
    pub next_ready_at: f64,
    #[cultcache(key = 12)]
    pub private_state_exposed: bool,
    #[cultcache(key = 13)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatArtifactRetentionMember {
    pub directory_name: String,
    pub manifest_sha256: String,
    pub file_count: u64,
    pub byte_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.heartbeat.artifact_retention_plan",
    schema = "EpiphanyHeartbeatArtifactRetentionPlan"
)]
pub struct EpiphanyHeartbeatArtifactRetentionPlan {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub plan_id: String,
    #[cultcache(key = 2)]
    pub artifact_root: String,
    #[cultcache(key = 3)]
    pub retain_pulse_count: u64,
    #[cultcache(key = 4)]
    pub batch_size: u64,
    #[cultcache(key = 5)]
    pub members: Vec<HeartbeatArtifactRetentionMember>,
    #[cultcache(key = 6)]
    pub planned_at_utc: String,
    #[cultcache(key = 7)]
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.heartbeat.artifact_retention_receipt",
    schema = "EpiphanyHeartbeatArtifactRetentionReceipt"
)]
pub struct EpiphanyHeartbeatArtifactRetentionReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub plan_id: String,
    #[cultcache(key = 3)]
    pub status: String,
    #[cultcache(key = 4)]
    pub deleted_directories: Vec<String>,
    #[cultcache(key = 5)]
    pub deleted_file_count: u64,
    #[cultcache(key = 6)]
    pub deleted_byte_count: u64,
    #[cultcache(key = 7)]
    pub completed_at_utc: String,
    #[cultcache(key = 8)]
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatSelectionPolicy {
    pub mode: String,
    pub reaction_precedence: bool,
    pub minimum_speed: f64,
    pub tie_breakers: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatPacingPolicy {
    pub cooldown_starts_after_turn_completion: bool,
    pub work_base_recovery: f64,
    pub idle_base_recovery: f64,
    pub sleep_heartbeat_rate_multiplier: f64,
    pub minimum_effective_rate: f64,
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
    #[serde(rename = "initiativeFrozen", default)]
    pub initiative_frozen: bool,
    #[serde(rename = "initiativeFreezeReason", default)]
    pub initiative_freeze_reason: Option<String>,
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
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct HeartbeatTickOptions {
    pub(super) target_heartbeat_rate: f64,
    pub(super) schedule_id: String,
    pub(super) source_scene_ref: String,
    pub(super) resident_self_store: Option<std::path::PathBuf>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HeartbeatStaleTurnRepairOptions {
    pub max_age_seconds: i64,
    pub now_utc: Option<String>,
    pub reason: String,
}
