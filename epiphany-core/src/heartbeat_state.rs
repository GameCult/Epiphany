use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use cultcache_rs::CultCache;
use cultcache_rs::DatabaseEntry;
use cultcache_rs::SingleFileMessagePackBackingStore;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const HEARTBEAT_STATE_TYPE: &str = "epiphany.agent_heartbeat";
pub const HEARTBEAT_STATE_KEY: &str = "default";
pub const HEARTBEAT_STATE_SCHEMA_VERSION: &str = "epiphany.agent_heartbeat.v0";

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct EpiphanyHeartbeatStateProjection {
    pub schema_version: String,
    pub target_heartbeat_rate: f64,
    pub scene_clock: f64,
    pub selection_policy: HeartbeatSelectionPolicy,
    pub pacing_policy: HeartbeatPacingPolicy,
    pub participants: Vec<HeartbeatParticipant>,
    pub history: Vec<HeartbeatHistoryEvent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sleep_cycle: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_resonance: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incubation: Option<Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl From<EpiphanyHeartbeatStateProjection> for EpiphanyHeartbeatStateEntry {
    fn from(value: EpiphanyHeartbeatStateProjection) -> Self {
        Self {
            schema_version: value.schema_version,
            target_heartbeat_rate: value.target_heartbeat_rate,
            scene_clock: value.scene_clock,
            selection_policy: value.selection_policy,
            pacing_policy: value.pacing_policy,
            participants: value.participants,
            history: value.history,
            sleep_cycle: value.sleep_cycle,
            memory_resonance: value.memory_resonance,
            incubation: value.incubation,
            extra: value.extra,
        }
    }
}

impl From<EpiphanyHeartbeatStateEntry> for EpiphanyHeartbeatStateProjection {
    fn from(value: EpiphanyHeartbeatStateEntry) -> Self {
        Self {
            schema_version: value.schema_version,
            target_heartbeat_rate: value.target_heartbeat_rate,
            scene_clock: value.scene_clock,
            selection_policy: value.selection_policy,
            pacing_policy: value.pacing_policy,
            participants: value.participants,
            history: value.history,
            sleep_cycle: value.sleep_cycle,
            memory_resonance: value.memory_resonance,
            incubation: value.incubation,
            extra: value.extra,
        }
    }
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

pub fn migrate_heartbeat_json_to_cultcache(
    json_path: impl AsRef<Path>,
    store_path: impl AsRef<Path>,
) -> Result<EpiphanyHeartbeatStateEntry> {
    let state = read_heartbeat_json_projection(json_path)?;
    write_heartbeat_state_entry(store_path, &state)
}

pub fn write_heartbeat_json_projection(
    store_path: impl AsRef<Path>,
    json_path: impl AsRef<Path>,
) -> Result<EpiphanyHeartbeatStateEntry> {
    let state = load_heartbeat_state_entry(store_path.as_ref())?.ok_or_else(|| {
        anyhow!(
            "CultCache store {} has no heartbeat state entry",
            store_path.as_ref().display()
        )
    })?;
    write_projection_json(json_path, &state)?;
    Ok(state)
}

pub fn read_heartbeat_json_projection(
    json_path: impl AsRef<Path>,
) -> Result<EpiphanyHeartbeatStateEntry> {
    let path = json_path.as_ref();
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read heartbeat JSON {}", path.display()))?;
    let projection: EpiphanyHeartbeatStateProjection = serde_json::from_str(&raw)
        .with_context(|| format!("failed to decode heartbeat JSON {}", path.display()))?;
    let state = EpiphanyHeartbeatStateEntry::from(projection);
    validate_heartbeat_state(&state)?;
    Ok(state)
}

pub fn write_projection_json(
    json_path: impl AsRef<Path>,
    state: &EpiphanyHeartbeatStateEntry,
) -> Result<()> {
    validate_heartbeat_state(state)?;
    let path = json_path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let projection = EpiphanyHeartbeatStateProjection::from(state.clone());
    let raw = serde_json::to_string_pretty(&projection)
        .context("failed to encode heartbeat JSON projection")?;
    fs::write(path, format!("{raw}\n"))
        .with_context(|| format!("failed to write heartbeat JSON {}", path.display()))?;
    Ok(())
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
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn heartbeat_json_migrates_to_typed_cultcache_and_projects_back() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let json_path = temp.path().join("heartbeats.json");
        let store_path = temp.path().join("heartbeats.msgpack");
        let projected_path = temp.path().join("heartbeats.projected.json");
        fs::write(&json_path, sample_json())?;

        let migrated = migrate_heartbeat_json_to_cultcache(&json_path, &store_path)?;
        assert_eq!(migrated.schema_version, HEARTBEAT_STATE_SCHEMA_VERSION);
        assert_eq!(migrated.participants[0].role_id, "coordinator");

        let loaded = load_heartbeat_state_entry(&store_path)?.expect("persisted heartbeat state");
        assert_eq!(loaded, migrated);

        write_heartbeat_json_projection(&store_path, &projected_path)?;
        let projected = read_heartbeat_json_projection(&projected_path)?;
        assert_eq!(projected, migrated);
        Ok(())
    }

    #[test]
    fn live_heartbeat_json_shape_is_compatible_when_present() -> Result<()> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../state/agent-heartbeats.json");
        if !path.exists() {
            return Ok(());
        }
        let state = read_heartbeat_json_projection(&path)?;
        validate_heartbeat_state(&state)?;
        assert!(state.participants.iter().any(|item| item.role_id == "face"));
        assert!(
            state
                .participants
                .iter()
                .any(|item| item.role_id == "coordinator")
        );
        Ok(())
    }

    fn sample_json() -> &'static str {
        r#"{
  "schema_version": "epiphany.agent_heartbeat.v0",
  "target_heartbeat_rate": 1.0,
  "scene_clock": 0.0,
  "selection_policy": {
    "mode": "readiness_queue",
    "reaction_precedence": true,
    "minimum_speed": 0.2,
    "tie_breakers": ["next_ready_at_asc"]
  },
  "pacing_policy": {
    "cooldown_starts_after_turn_completion": true,
    "work_base_recovery": 6.0,
    "idle_base_recovery": 2.0,
    "sleep_heartbeat_rate_multiplier": 0.05,
    "minimum_effective_rate": 0.001
  },
  "participants": [
    {
      "agent_id": "epiphany.self",
      "role_id": "coordinator",
      "display_name": "Self",
      "initiative_speed": 1.28,
      "next_ready_at": 0.0,
      "reaction_bias": 0.88,
      "interrupt_threshold": 0.42,
      "current_load": 0.0,
      "status": "active",
      "constraints": ["Routes and reviews."],
      "last_action_id": null,
      "last_woke_at": null,
      "last_finished_at": null,
      "pending_turn": null
    }
  ],
  "history": []
}"#
    }
}
