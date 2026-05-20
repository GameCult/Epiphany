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
    #[cultcache(key = 16, default)]
    pub initiative_heat: HeartbeatInitiativeHeatPolicy,
    #[cultcache(key = 17, default)]
    pub protocol: Option<HeartbeatProtocol>,
    #[cultcache(key = 18, default)]
    pub adaptive_pacing: Option<HeartbeatAdaptivePacing>,
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
    pub sleep_cycle: Option<HeartbeatSleepCycle>,
    #[cultcache(key = 6, default)]
    pub memory_resonance: Option<HeartbeatMemoryResonance>,
    #[cultcache(key = 7, default)]
    pub incubation: Option<HeartbeatIncubation>,
    #[cultcache(key = 8, default)]
    pub thought_lanes: Option<HeartbeatThoughtLanes>,
    #[cultcache(key = 9, default)]
    pub bridge: Option<HeartbeatCognitionBridge>,
    #[cultcache(key = 10, default)]
    pub candidate_interventions: Option<HeartbeatCandidateInterventions>,
    #[cultcache(key = 11, default)]
    pub appraisals: Option<HeartbeatAgentThoughtAppraisals>,
    #[cultcache(key = 12, default)]
    pub reactions: Option<HeartbeatAgentReactions>,
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
pub struct HeartbeatProtocol {
    pub domain: String,
    #[serde(default)]
    pub scene_id: Option<String>,
    pub arena: String,
    pub contract: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatAdaptivePacing {
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
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatMemoryResonance {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    pub updated_at: String,
    pub source: String,
    pub record_count: usize,
    pub pairs: Vec<HeartbeatMemoryResonancePair>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatMemoryResonancePair {
    pub left_role: String,
    pub left_memory_id: String,
    pub left_memory_kind: String,
    pub left_summary: String,
    pub right_role: String,
    pub right_memory_id: String,
    pub right_memory_kind: String,
    pub right_summary: String,
    pub strength: f64,
    pub shared_tokens: Vec<String>,
    pub source_roles: Vec<String>,
    pub source_kinds: Vec<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatSourceCoverage {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    pub updated_at: String,
    pub roles: Vec<HeartbeatSourceCoverageRole>,
    pub memory_kinds: Vec<HeartbeatSourceCoverageKind>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatSourceCoverageRole {
    pub role_id: String,
    pub count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatSourceCoverageKind {
    pub kind: String,
    pub count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatIncubation {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    pub updated_at: String,
    pub source_coverage: HeartbeatSourceCoverage,
    pub last_incubation_summary: String,
    pub themes: Vec<HeartbeatIncubationTheme>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatIncubationTheme {
    pub theme_id: String,
    pub summary: String,
    pub strength: f64,
    pub source: String,
    pub source_roles: Vec<String>,
    pub source_kinds: Vec<String>,
    pub source_memory_ids: Vec<String>,
    pub support_count: usize,
    pub evidence_diversity: f64,
    pub exploration_bonus: f64,
    pub novelty: f64,
    pub novelty_to_self: f64,
    pub novelty_to_room: f64,
    pub maturation: f64,
    pub desire_to_speak: f64,
    pub saturation_score: f64,
    pub recent_match_count: usize,
    pub refractory_penalty: f64,
    pub priority_score: f64,
    pub status: String,
    pub latent_question: String,
    pub why_it_pulls: String,
    pub holding_close_because: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatThoughtLanes {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    pub updated_at: String,
    pub analytic: HeartbeatAnalyticLane,
    pub associative: HeartbeatAssociativeLane,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatAnalyticLane {
    pub description: String,
    pub active_threads: Vec<HeartbeatAnalyticThread>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatAssociativeLane {
    pub description: String,
    pub active_threads: Vec<HeartbeatAssociativeThread>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatAnalyticThread {
    pub thread_id: String,
    pub topic: String,
    pub claim: String,
    pub evidence_refs: Vec<String>,
    pub salience: f64,
    pub confidence: f64,
    pub desire_to_act: f64,
    pub counterweight: String,
    pub last_touched_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatAssociativeThread {
    pub thread_id: String,
    pub topic: String,
    pub claim: String,
    pub source_theme_id: String,
    pub novelty: f64,
    pub room_relevance: f64,
    pub desire_to_speak: f64,
    pub status: String,
    pub counterweight: String,
    pub last_touched_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatCognitionBridge {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    pub updated_at: String,
    pub recent_syntheses: Vec<HeartbeatBridgeSynthesis>,
    pub source_coverage: HeartbeatSourceCoverage,
    pub topic_saturation: Vec<HeartbeatTopicSaturation>,
    pub refractory_topics: Vec<HeartbeatRefractoryTopic>,
    pub unresolved_tensions: Vec<HeartbeatBridgeTension>,
    pub decision: HeartbeatBridgeDecision,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatBridgeSynthesis {
    pub timestamp: String,
    pub summary: String,
    pub dominant_topics: Vec<String>,
    pub lane_balance: String,
    pub speak_decision: String,
    pub theme_status: String,
    pub novelty_to_self: f64,
    pub novelty_to_room: f64,
    pub saturation_score: f64,
    pub saturation_note: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatTopicSaturation {
    pub topic: String,
    pub dominance: f64,
    pub recent_mentions: usize,
    pub cooling_advice: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatRefractoryTopic {
    pub topic: String,
    pub penalty: f64,
    pub cools_until: String,
    pub reason: String,
    pub last_triggered_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatBridgeTension {
    pub topic: String,
    pub summary: String,
    pub opened_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatBridgeDecision {
    pub lane_balance: String,
    pub speak_decision: String,
    pub reason: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatSleepCycle {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    pub enabled: bool,
    pub cycle_hours: i64,
    pub nap_duration_minutes: i64,
    pub phase_offset_minutes_local: i64,
    pub reply_mode: String,
    pub is_napping: bool,
    pub current_nap_started_at: Option<String>,
    pub current_nap_ends_at: Option<String>,
    pub next_nap_starts_at: String,
    pub last_dream_at: Option<String>,
    pub dream_count_in_current_nap: u64,
    pub active_dream_themes: Vec<String>,
    pub last_distillation_summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatAgentThoughtAppraisals {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    pub updated_at: String,
    pub thought_cluster_ref: String,
    pub participant_appraisals: Vec<HeartbeatAgentThoughtAppraisal>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatAgentThoughtAppraisal {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    pub appraisal_id: String,
    pub review_status: String,
    pub participant_agent_id: String,
    pub role_id: String,
    pub current_character_state_ref: String,
    pub thought_cluster_ref: String,
    pub participant_local_context: HeartbeatParticipantLocalContext,
    pub observable_thought_summary: String,
    pub personality_projection: Vec<HeartbeatPersonalityProjection>,
    pub interpretation: String,
    pub emotional_appraisal: HeartbeatEmotionalAppraisal,
    pub interpretation_label: String,
    pub confidence_notes: String,
    pub candidate_implications: HeartbeatCandidateImplications,
    pub review: HeartbeatAppraisalReview,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatParticipantLocalContext {
    pub display_name: String,
    pub values: Vec<String>,
    pub reactivity: f64,
    pub plasticity: f64,
    pub expressiveness: f64,
    pub guardedness: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatPersonalityProjection {
    pub group: String,
    pub name: String,
    pub activation: f64,
    pub plasticity: f64,
    pub token_overlap: f64,
    pub projection: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatEmotionalAppraisal {
    pub valence: f64,
    pub arousal: f64,
    pub urgency: f64,
    pub curiosity: f64,
    pub guardedness: f64,
    pub thought_pressure: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatCandidateImplications {
    pub reaction_mode: String,
    pub reaction_intensity: f64,
    pub should_speak: bool,
    pub should_incubate: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatAppraisalReview {
    pub accepted_for_mutation: bool,
    pub rationale: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatAgentReactions {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    pub updated_at: String,
    pub reactions: Vec<HeartbeatAgentReaction>,
    pub contract: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatAgentReaction {
    pub reaction_id: String,
    pub role_id: String,
    pub participant_agent_id: String,
    pub appraisal_id: String,
    pub mode: String,
    pub mood_label: String,
    pub intensity: f64,
    pub bridge_decision: String,
    pub surface: String,
    pub recommended_use: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatCandidateInterventions {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    pub updated_at: String,
    pub items: Vec<HeartbeatCandidateIntervention>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HeartbeatCandidateIntervention {
    pub intervention_id: String,
    pub summary: String,
    pub draft: String,
    pub decision: String,
    pub requires_face: bool,
    pub requires_review: bool,
    pub novelty_to_room: f64,
    pub saturation_score: f64,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatInitiativeHeatPolicy {
    #[serde(default = "default_heat_schema_version")]
    pub schema_version: String,
    #[serde(default = "default_heat_global_multiplier")]
    pub global_multiplier: f64,
    #[serde(default)]
    pub multipliers: Vec<HeartbeatInitiativeMultiplier>,
}

impl Default for HeartbeatInitiativeHeatPolicy {
    fn default() -> Self {
        Self {
            schema_version: default_heat_schema_version(),
            global_multiplier: default_heat_global_multiplier(),
            multipliers: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatInitiativeMultiplier {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default = "default_heat_scope")]
    pub scope: String,
    #[serde(default)]
    pub selector: String,
    #[serde(default = "default_heat_global_multiplier")]
    pub multiplier: f64,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub expires_at_scene_clock: Option<f64>,
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
    #[serde(default)]
    pub scene_id: Option<String>,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default = "default_heat_global_multiplier")]
    pub personality_cooldown_multiplier: f64,
    #[serde(default = "default_heat_global_multiplier")]
    pub mood_cooldown_multiplier: f64,
    #[serde(default = "default_heat_global_multiplier")]
    pub initiative_heat_multiplier: f64,
    #[serde(default)]
    pub initiative_heat: Option<HeartbeatInitiativeHeatProjection>,
    #[serde(default)]
    pub personality_timing: Option<HeartbeatPersonalityTiming>,
    #[serde(default)]
    pub mood_timing: Option<HeartbeatMoodTiming>,
    #[serde(default)]
    pub birth_personality_seed: Option<HeartbeatBirthPersonalitySeed>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatInitiativeHeatProjection {
    pub schema_version: String,
    pub global_multiplier: f64,
    pub effective_multiplier: f64,
    pub basis: Vec<HeartbeatInitiativeHeatBasis>,
    pub contract: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatInitiativeHeatBasis {
    pub id: String,
    pub scope: String,
    pub selector: String,
    pub multiplier: f64,
    pub reason: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatPersonalityTiming {
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
pub struct HeartbeatMoodTiming {
    pub schema_version: String,
    pub source: Option<String>,
    pub cooldown_multiplier: f64,
    #[serde(default)]
    pub emotional_dimensions: Vec<HeartbeatMoodDimension>,
    pub anxiety: f64,
    pub urgency: f64,
    pub arousal: f64,
    pub thought_pressure: f64,
    pub guardedness: f64,
    pub reaction_intensity: f64,
    pub contract: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatMoodDimension {
    pub name: String,
    pub value: f64,
    pub source_path: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatBirthPersonalitySeed {
    pub schema_version: String,
    pub source: String,
    pub projection_id: String,
    pub repo_id: String,
    pub heartbeat_deltas: BTreeMap<String, f64>,
    pub default_mood_pressure: BTreeMap<String, f64>,
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
    #[serde(
        rename = "personalityCooldownMultiplier",
        default = "default_heat_global_multiplier"
    )]
    pub personality_cooldown_multiplier: f64,
    #[serde(
        rename = "moodCooldownMultiplier",
        default = "default_heat_global_multiplier"
    )]
    pub mood_cooldown_multiplier: f64,
    #[serde(
        rename = "initiativeHeatMultiplier",
        default = "default_heat_global_multiplier"
    )]
    pub initiative_heat_multiplier: f64,
    #[serde(
        rename = "effectiveCooldownMultiplier",
        default = "default_heat_global_multiplier"
    )]
    pub effective_cooldown_multiplier: f64,
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
pub struct HeartbeatHeatUpdateOptions {
    pub scope: String,
    pub selector: String,
    pub multiplier: f64,
    pub id: Option<String>,
    pub label: Option<String>,
    pub reason: Option<String>,
    pub expires_after_scene_clock: Option<f64>,
    pub clear: bool,
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

fn default_heat_schema_version() -> String {
    "epiphany.heartbeat_initiative_heat.v0".to_string()
}

fn default_heat_global_multiplier() -> f64 {
    1.0
}

fn default_heat_scope() -> String {
    "agent".to_string()
}
