use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::SecondsFormat;
use chrono::Utc;
use cultcache_rs::CultCache;
use cultcache_rs::CultSoaColumnValues;
use cultcache_rs::DatabaseEntry;
use cultcache_rs::SingleFileMessagePackBackingStore;
use cultcache_rs::SoaDocument;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const AGENT_MEMORY_TYPE: &str = "epiphany.agent_memory";
pub const AGENT_MEMORY_SCHEMA_VERSION: &str = "ghostlight.agent_state.v0";
pub const AGENT_MEMORY_SWARM_IDENTITY_TYPE: &str = "epiphany.agent_memory_swarm_identity";
pub const AGENT_MEMORY_SWARM_IDENTITY_SCHEMA_VERSION: &str =
    "epiphany.agent_memory_swarm_identity.v0";
pub const AGENT_MEMORY_SWARM_IDENTITY_KEY: &str = "swarm-identity";
pub const AGENT_STATE_SOA_TYPE: &str = "epiphany.agent_state_soa";
pub const AGENT_STATE_SOA_SCHEMA_VERSION: &str = "epiphany.agent_state_soa.v0";
pub const AGENT_STATE_SOA_KEY: &str = "swarm";

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.agent_memory_swarm_identity",
    schema = "AgentMemorySwarmIdentity"
)]
pub struct AgentMemorySwarmIdentity {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub swarm_id: String,
}

pub fn load_agent_memory_swarm_identity(
    store_path: impl AsRef<Path>,
) -> Result<Option<AgentMemorySwarmIdentity>> {
    let store_path = store_path.as_ref();
    if !store_path.exists() {
        return Ok(None);
    }
    let mut cache = agent_memory_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get(AGENT_MEMORY_SWARM_IDENTITY_KEY)
}

pub fn ensure_agent_memory_swarm_identity(
    store_path: impl AsRef<Path>,
    swarm_id: &str,
) -> Result<AgentMemorySwarmIdentity> {
    let swarm_id = swarm_id.trim();
    if swarm_id.is_empty() {
        return Err(anyhow!(
            "agent memory swarm identity requires a non-empty swarm_id"
        ));
    }
    let store_path = store_path.as_ref();
    let mut cache = agent_memory_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    if let Some(existing) =
        cache.get::<AgentMemorySwarmIdentity>(AGENT_MEMORY_SWARM_IDENTITY_KEY)?
    {
        if existing.swarm_id == swarm_id
            && existing.schema_version == AGENT_MEMORY_SWARM_IDENTITY_SCHEMA_VERSION
        {
            return Ok(existing);
        }
        return Err(anyhow!(
            "agent memory swarm identity collision: store owns {:?}, refused {:?}",
            existing.swarm_id,
            swarm_id
        ));
    }
    let identity = AgentMemorySwarmIdentity {
        schema_version: AGENT_MEMORY_SWARM_IDENTITY_SCHEMA_VERSION.to_string(),
        swarm_id: swarm_id.to_string(),
    };
    let envelope = cache
        .prepare_entry(AGENT_MEMORY_SWARM_IDENTITY_KEY, &identity)?
        .0;
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    if !backing.compare_and_swap_batch(&[], vec![envelope])? {
        let raced = load_agent_memory_swarm_identity(store_path)?;
        if raced.as_ref() == Some(&identity) {
            return Ok(identity);
        }
        return Err(anyhow!(
            "agent memory swarm identity lost immutable compare-and-swap"
        ));
    }
    Ok(identity)
}

const ROLE_TARGETS: &[(&str, &str, &str)] = &[
    (
        "imagination",
        "epiphany.imagination",
        "imagination.agent-state.json",
    ),
    ("modeling", "epiphany.modeling", "modeling.agent-state.json"),
    ("verification", "epiphany.soul", "soul.agent-state.json"),
    ("implementation", "epiphany.hands", "hands.agent-state.json"),
    ("research", "epiphany.eyes", "eyes.agent-state.json"),
    ("Persona", "epiphany.Persona", "Persona.agent-state.json"),
    ("coordinator", "epiphany.self", "self.agent-state.json"),
];

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(type = "epiphany.agent_memory", schema = "EpiphanyAgentMemoryEntry")]
pub struct EpiphanyAgentMemoryEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub role_id: String,
    #[cultcache(key = 2)]
    pub world: GhostlightWorld,
    #[cultcache(key = 3)]
    pub agent: GhostlightAgent,
    #[cultcache(key = 4, default)]
    pub relationships: Vec<GhostlightRelationship>,
    #[cultcache(key = 5, default)]
    pub events: Vec<GhostlightEvent>,
    #[cultcache(key = 6, default)]
    pub scenes: Vec<GhostlightScene>,
}

impl SoaDocument for EpiphanyAgentMemoryEntry {
    fn soa_columns(rows: &[Self]) -> BTreeMap<&'static str, CultSoaColumnValues> {
        let mut columns = BTreeMap::new();
        columns.insert(
            "roleId",
            CultSoaColumnValues::new(
                rows.iter()
                    .map(|row| row.role_id.clone())
                    .collect::<Vec<_>>(),
            ),
        );
        columns.insert(
            "agentId",
            CultSoaColumnValues::new(
                rows.iter()
                    .map(|row| row.agent.agent_id.clone())
                    .collect::<Vec<_>>(),
            ),
        );
        columns.insert(
            "displayName",
            CultSoaColumnValues::new(
                rows.iter()
                    .map(|row| row.agent.identity.name.clone())
                    .collect::<Vec<_>>(),
            ),
        );
        columns.insert(
            "profileKind",
            CultSoaColumnValues::new(
                rows.iter()
                    .map(|row| {
                        format!(
                            "{:?}",
                            organ_state_profile_for_role(&row.role_id).profile_kind
                        )
                    })
                    .collect::<Vec<_>>(),
            ),
        );
        columns.insert(
            "portableContract",
            CultSoaColumnValues::new(
                rows.iter()
                    .map(|row| organ_state_profile_for_role(&row.role_id).portable_contract)
                    .collect::<Vec<_>>(),
            ),
        );
        columns.insert(
            "semanticMemoryCount",
            CultSoaColumnValues::new(
                rows.iter()
                    .map(|row| row.agent.memories.semantic.len() as u32)
                    .collect::<Vec<_>>(),
            ),
        );
        columns.insert(
            "episodicMemoryCount",
            CultSoaColumnValues::new(
                rows.iter()
                    .map(|row| row.agent.memories.episodic.len() as u32)
                    .collect::<Vec<_>>(),
            ),
        );
        columns.insert(
            "relationshipMemoryCount",
            CultSoaColumnValues::new(
                rows.iter()
                    .map(|row| row.agent.memories.relationship_summaries.len() as u32)
                    .collect::<Vec<_>>(),
            ),
        );
        columns.insert(
            "goalCount",
            CultSoaColumnValues::new(
                rows.iter()
                    .map(|row| row.agent.goals.len() as u32)
                    .collect::<Vec<_>>(),
            ),
        );
        columns.insert(
            "valueCount",
            CultSoaColumnValues::new(
                rows.iter()
                    .map(|row| row.agent.canonical_state.values.len() as u32)
                    .collect::<Vec<_>>(),
            ),
        );
        columns
    }
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.agent_state_soa",
    schema = "EpiphanyAgentStateSoaEntry"
)]
pub struct EpiphanyAgentStateSoaEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub generated_at: String,
    #[cultcache(key = 2)]
    pub source_store: String,
    #[cultcache(key = 3)]
    pub role_ids: Vec<String>,
    #[cultcache(key = 4)]
    pub agent_ids: Vec<String>,
    #[cultcache(key = 5)]
    pub display_names: Vec<String>,
    #[cultcache(key = 6)]
    pub profile_kinds: Vec<String>,
    #[cultcache(key = 7)]
    pub portable_contracts: Vec<String>,
    #[cultcache(key = 8)]
    pub semantic_memory_counts: Vec<u32>,
    #[cultcache(key = 9)]
    pub episodic_memory_counts: Vec<u32>,
    #[cultcache(key = 10)]
    pub relationship_memory_counts: Vec<u32>,
    #[cultcache(key = 11)]
    pub goal_counts: Vec<u32>,
    #[cultcache(key = 12)]
    pub value_counts: Vec<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightWorld {
    pub world_id: String,
    pub setting: String,
    pub time: GhostlightTime,
    pub canon_context: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightTime {
    pub label: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightAgent {
    pub agent_id: String,
    pub identity: GhostlightIdentity,
    pub canonical_state: GhostlightCanonicalState,
    pub goals: Vec<GhostlightGoal>,
    pub memories: GhostlightMemories,
    #[serde(default)]
    pub perceived_state_overlays: Vec<GhostlightPerceivedStateOverlay>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightRelationship {
    #[serde(default)]
    pub relationship_id: String,
    #[serde(default)]
    pub participant_ids: Vec<String>,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub stance: String,
    #[serde(default)]
    pub salience: f64,
    #[serde(default)]
    pub confidence: f64,
    #[serde(default)]
    pub linked_memory_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightEvent {
    #[serde(default)]
    pub event_id: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp_label: Option<String>,
    #[serde(default)]
    pub participant_ids: Vec<String>,
    #[serde(default)]
    pub linked_memory_ids: Vec<String>,
    #[serde(default)]
    pub salience: f64,
    #[serde(default)]
    pub confidence: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightScene {
    #[serde(default)]
    pub scene_id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub participant_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
    #[serde(default)]
    pub salience: f64,
    #[serde(default)]
    pub status: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightPerceivedStateOverlay {
    #[serde(default)]
    pub overlay_id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub salience: f64,
    #[serde(default)]
    pub confidence: f64,
    #[serde(default)]
    pub linked_memory_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightIdentity {
    pub name: String,
    pub roles: Vec<String>,
    pub origin: String,
    pub public_description: String,
    #[serde(default)]
    pub private_notes: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightCanonicalState {
    pub underlying_organization: BTreeMap<String, GhostlightTraitVector>,
    pub stable_dispositions: BTreeMap<String, GhostlightTraitVector>,
    pub behavioral_dimensions: BTreeMap<String, GhostlightTraitVector>,
    pub presentation_strategy: BTreeMap<String, GhostlightTraitVector>,
    pub voice_style: BTreeMap<String, GhostlightTraitVector>,
    pub situational_state: BTreeMap<String, GhostlightTraitVector>,
    pub values: Vec<GhostlightValue>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightTraitVector {
    pub mean: f64,
    pub plasticity: f64,
    pub current_activation: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightValue {
    pub value_id: String,
    pub label: String,
    pub priority: f64,
    pub unforgivable_if_betrayed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightGoal {
    pub goal_id: String,
    pub description: String,
    pub scope: String,
    pub priority: f64,
    pub emotional_stake: String,
    #[serde(default)]
    pub blockers: Vec<String>,
    pub status: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightMemories {
    pub episodic: Vec<GhostlightMemory>,
    pub semantic: Vec<GhostlightMemory>,
    pub relationship_summaries: Vec<GhostlightMemory>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightMemory {
    pub memory_id: String,
    pub summary: String,
    pub salience: f64,
    pub confidence: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linked_event_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linked_relationship_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AgentMemoryProjection {
    pub schema_version: String,
    pub world: GhostlightWorld,
    pub agents: Vec<GhostlightAgent>,
    #[serde(default)]
    pub relationships: Vec<GhostlightRelationship>,
    #[serde(default)]
    pub events: Vec<GhostlightEvent>,
    #[serde(default)]
    pub scenes: Vec<GhostlightScene>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSelfPatch {
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub evidence_ids: Option<Vec<String>>,
    #[serde(default)]
    pub semantic_memories: Option<Vec<SelfPatchMemory>>,
    #[serde(default)]
    pub episodic_memories: Option<Vec<SelfPatchMemory>>,
    #[serde(default)]
    pub relationship_memories: Option<Vec<SelfPatchMemory>>,
    #[serde(default)]
    pub goals: Option<Vec<SelfPatchGoal>>,
    #[serde(default)]
    pub values: Option<Vec<SelfPatchValue>>,
    #[serde(default)]
    pub private_notes: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfPatchMemory {
    pub memory_id: String,
    pub summary: String,
    pub salience: f64,
    pub confidence: f64,
    #[serde(default)]
    pub linked_event_ids: Option<Vec<String>>,
    #[serde(default)]
    pub linked_relationship_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentMemoryLifecycleOperation {
    #[serde(default)]
    pub agent_id: Option<String>,
    pub reason: String,
    pub actions: Vec<AgentMemoryLifecycleAction>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "action")]
pub enum AgentMemoryLifecycleAction {
    Revise {
        bundle: AgentMemoryBundle,
        #[serde(alias = "memoryId")]
        memory_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        summary: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        salience: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        confidence: Option<f64>,
        reason: String,
    },
    Retire {
        bundle: AgentMemoryBundle,
        #[serde(alias = "memoryId")]
        memory_id: String,
        reason: String,
    },
    Crystallize {
        #[serde(alias = "fromBundle")]
        from_bundle: AgentMemoryBundle,
        #[serde(alias = "toBundle")]
        to_bundle: AgentMemoryBundle,
        #[serde(alias = "memoryId")]
        memory_id: String,
        #[serde(alias = "newMemoryId")]
        #[serde(default, skip_serializing_if = "Option::is_none")]
        new_memory_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        summary: Option<String>,
        reason: String,
    },
    Prune {
        bundle: AgentMemoryBundle,
        #[serde(alias = "maxRecords")]
        max_records: usize,
        #[serde(alias = "minimumSalience")]
        #[serde(default, skip_serializing_if = "Option::is_none")]
        minimum_salience: Option<f64>,
        reason: String,
    },
    Merge {
        bundle: AgentMemoryBundle,
        #[serde(alias = "targetMemoryId")]
        target_memory_id: String,
        #[serde(alias = "sourceMemoryIds")]
        source_memory_ids: Vec<String>,
        summary: String,
        salience: f64,
        confidence: f64,
        reason: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentMemoryBundle {
    Semantic,
    Episodic,
    RelationshipSummaries,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfPatchGoal {
    pub goal_id: String,
    pub description: String,
    pub scope: String,
    pub priority: f64,
    pub emotional_stake: String,
    #[serde(default)]
    pub blockers: Vec<String>,
    pub status: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfPatchValue {
    pub value_id: String,
    pub label: String,
    pub priority: f64,
    pub unforgivable_if_betrayed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentMemoryReview {
    pub status: String,
    pub target_agent_id: String,
    pub target_role_id: String,
    pub target_store: String,
    pub reasons: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpiphanyOrganStateProfileKind {
    WorkOrgan,
    Persona,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyOrganStateProfile {
    pub profile_kind: EpiphanyOrganStateProfileKind,
    pub state_density: String,
    pub portable_contract: String,
    pub relationship_model: String,
    pub affect_model: String,
    pub perceived_overlay_mode: String,
    pub growth_channels: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCanonicalTraitSeed {
    pub role_id: String,
    pub group_name: String,
    pub trait_name: String,
    pub mean: f64,
    pub plasticity: f64,
    pub current_activation: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

pub const PERSONA_STATE_SCHEMA_VERSION: &str = "gamecult.persona_state.v0";

pub fn migrate_agent_memory_json_dir_to_cultcache(
    agent_dir: impl AsRef<Path>,
    store_path: impl AsRef<Path>,
) -> Result<Value> {
    let agent_dir = agent_dir.as_ref();
    let store_path = store_path.as_ref();
    let mut cache = agent_memory_cache(store_path)?;
    let mut migrated = Vec::new();
    for (role_id, expected_agent_id, filename) in ROLE_TARGETS {
        let path = agent_dir.join(filename);
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let projection: AgentMemoryProjection = serde_json::from_str(&raw)
            .with_context(|| format!("failed to decode {}", path.display()))?;
        let entry = entry_from_projection(role_id, expected_agent_id, projection)
            .with_context(|| format!("invalid role memory {}", path.display()))?;
        cache.put(*role_id, &entry)?;
        migrated.push(serde_json::json!({
            "roleId": role_id,
            "agentId": expected_agent_id,
            "source": path,
        }));
    }
    Ok(serde_json::json!({
        "ok": true,
        "store": store_path,
        "migrated": migrated,
    }))
}

pub fn validate_agent_memory_store(store_path: impl AsRef<Path>) -> Result<Vec<String>> {
    let store_path = store_path.as_ref();
    let mut cache = agent_memory_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut errors = Vec::new();
    for (role_id, expected_agent_id, _) in ROLE_TARGETS {
        let Some(entry) = cache.get::<EpiphanyAgentMemoryEntry>(role_id)? else {
            errors.push(format!("{role_id}: missing CultCache role memory entry"));
            continue;
        };
        errors.extend(validate_agent_entry(&entry, expected_agent_id));
    }
    Ok(errors)
}

pub fn repair_agent_memory_store(store_path: impl AsRef<Path>) -> Result<Value> {
    let store_path = store_path.as_ref();
    let mut cache = agent_memory_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut repaired = Vec::new();

    if let Some(mut modeling) = cache.get::<EpiphanyAgentMemoryEntry>("modeling")?
        && modeling.agent.agent_id == "epiphany.proprioception"
    {
        modeling.agent.agent_id = "epiphany.modeling".to_string();
        if modeling.agent.identity.name == "Proprioception" {
            modeling.agent.identity.name = "Modeling".to_string();
        }
        if modeling
            .agent
            .identity
            .roles
            .iter()
            .any(|role| role == "Proprioception")
        {
            modeling.agent.identity.roles = modeling
                .agent
                .identity
                .roles
                .into_iter()
                .map(|role| {
                    if role == "Proprioception" {
                        "Modeling".to_string()
                    } else {
                        role
                    }
                })
                .collect();
        }
        cache.put("modeling".to_string(), &modeling)?;
        repaired.push(serde_json::json!({
            "roleId": "modeling",
            "repair": "renamed legacy epiphany.proprioception vessel to epiphany.modeling",
        }));
    }

    if let Some(mut modeling) = cache.get::<EpiphanyAgentMemoryEntry>("modeling")? {
        let mut renamed = 0usize;
        renamed += replace_deprecated_faculty_name(&mut modeling.agent.identity.public_description);
        for note in &mut modeling.agent.identity.private_notes {
            renamed += replace_deprecated_faculty_name(note);
        }
        for goal in &mut modeling.agent.goals {
            renamed += replace_deprecated_faculty_name(&mut goal.description);
            renamed += replace_deprecated_faculty_name(&mut goal.emotional_stake);
        }
        for memory in modeling
            .agent
            .memories
            .semantic
            .iter_mut()
            .chain(modeling.agent.memories.episodic.iter_mut())
            .chain(modeling.agent.memories.relationship_summaries.iter_mut())
        {
            renamed += replace_deprecated_faculty_name(&mut memory.summary);
        }
        if renamed > 0 {
            cache.put("modeling".to_string(), &modeling)?;
            repaired.push(serde_json::json!({
                "roleId": "modeling",
                "repair": "replaced deprecated Proprioception prose with canonical Modeling doctrine",
                "replacements": renamed,
            }));
        }
    }

    if cache.get::<EpiphanyAgentMemoryEntry>("Persona")?.is_none()
        && let Some(mut face) = cache.get::<EpiphanyAgentMemoryEntry>("face")?
    {
        face.role_id = "Persona".to_string();
        face.agent.agent_id = "epiphany.Persona".to_string();
        if face.agent.identity.name == "Face" {
            face.agent.identity.name = "Persona".to_string();
        }
        if face.agent.identity.roles.is_empty() {
            face.agent.identity.roles.push("Persona".to_string());
        }
        if !face
            .agent
            .identity
            .roles
            .iter()
            .any(|role| role == "Persona")
        {
            face.agent.identity.roles.push("Persona".to_string());
        }
        face.agent.identity.public_description = if face
            .agent
            .identity
            .public_description
            .trim()
            .is_empty()
        {
            "Epiphany Persona is the public-facing project voice; Imagination projects context before speech and Mind interprets side effects after speech.".to_string()
        } else {
            face.agent.identity.public_description
        };
        cache.put("Persona".to_string(), &face)?;
        cache.delete::<EpiphanyAgentMemoryEntry>("face")?;
        repaired.push(serde_json::json!({
            "roleId": "Persona",
            "repair": "promoted legacy face memory entry to Persona role memory and removed obsolete face row",
        }));
    }

    if cache
        .get::<EpiphanyAgentMemoryEntry>("reorientation")?
        .is_some()
    {
        cache.delete::<EpiphanyAgentMemoryEntry>("reorientation")?;
        repaired.push(serde_json::json!({
            "roleId": "reorientation",
            "repair": "removed obsolete reorientation row; Continuity is protocol machinery, not a standing sub-agent identity",
        }));
    }

    let errors = validate_agent_memory_store(store_path)?;
    let soa = refresh_agent_state_soa(store_path)?;
    Ok(serde_json::json!({
        "ok": errors.is_empty(),
        "store": store_path,
        "repaired": repaired,
        "errors": errors,
        "soa": soa,
    }))
}

fn replace_deprecated_faculty_name(value: &mut String) -> usize {
    let count = value.matches("Proprioception").count();
    if count > 0 {
        *value = value.replace("Proprioception", "Modeling");
    }
    count
}

pub fn load_agent_memory_entry_for_role(
    store_path: impl AsRef<Path>,
    role_id: &str,
) -> Result<Option<EpiphanyAgentMemoryEntry>> {
    let store_path = store_path.as_ref();
    let agent_id = agent_id_for_role(role_id).map_err(|message| anyhow!(message))?;
    let mut cache = agent_memory_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let entry = cache.get::<EpiphanyAgentMemoryEntry>(role_id)?;
    if let Some(entry) = &entry
        && entry.agent.agent_id != agent_id
    {
        return Err(anyhow!(
            "{} agent_id {:?} does not match expected {:?}",
            role_id,
            entry.agent.agent_id,
            agent_id
        ));
    }
    Ok(entry)
}

pub fn write_agent_memory_entry_for_role(
    store_path: impl AsRef<Path>,
    entry: &EpiphanyAgentMemoryEntry,
) -> Result<()> {
    let store_path = store_path.as_ref();
    let expected_agent_id =
        agent_id_for_role(&entry.role_id).map_err(|message| anyhow!(message))?;
    if entry.agent.agent_id != expected_agent_id {
        return Err(anyhow!(
            "{} agent_id {:?} does not match expected {:?}",
            entry.role_id,
            entry.agent.agent_id,
            expected_agent_id
        ));
    }
    let mut cache = agent_memory_cache(store_path)?;
    cache.put(entry.role_id.as_str(), entry)?;
    Ok(())
}

pub fn agent_memory_role_ids() -> Vec<&'static str> {
    ROLE_TARGETS
        .iter()
        .map(|(role_id, _, _)| *role_id)
        .collect()
}

pub fn review_agent_self_patch(
    role_id: &str,
    patch_value: &Value,
    store_path: impl AsRef<Path>,
) -> AgentMemoryReview {
    let store_path = store_path.as_ref();
    let mut reasons = Vec::new();
    let target_agent_id = match agent_id_for_role(role_id) {
        Ok(agent_id) => agent_id.to_string(),
        Err(reason) => {
            return AgentMemoryReview {
                status: "rejected".to_string(),
                target_agent_id: String::new(),
                target_role_id: role_id.to_string(),
                target_store: store_path.display().to_string(),
                reasons: vec![reason],
                applied: None,
            };
        }
    };

    match decode_agent_self_patch(patch_value) {
        Ok(patch) => {
            return review_agent_self_patch_document(role_id, &patch, store_path);
        }
        Err(reason) => reasons.push(reason),
    }

    agent_memory_review(role_id, &target_agent_id, store_path, reasons, None)
}

pub(crate) fn decode_agent_self_patch(
    patch_value: &Value,
) -> std::result::Result<AgentSelfPatch, String> {
    if !patch_value.is_object() {
        return Err("selfPatch must be a JSON object".to_string());
    }
    serde_json::from_value(patch_value.clone())
        .map_err(|err| format!("selfPatch is not a valid AgentSelfPatch document: {err}"))
}

pub fn review_agent_self_patch_document(
    role_id: &str,
    patch: &AgentSelfPatch,
    store_path: impl AsRef<Path>,
) -> AgentMemoryReview {
    let store_path = store_path.as_ref();
    let target_agent_id = match agent_id_for_role(role_id) {
        Ok(agent_id) => agent_id,
        Err(reason) => {
            return AgentMemoryReview {
                status: "rejected".to_string(),
                target_agent_id: String::new(),
                target_role_id: role_id.to_string(),
                target_store: store_path.display().to_string(),
                reasons: vec![reason],
                applied: None,
            };
        }
    };

    let reasons = review_agent_self_patch_contract(target_agent_id, patch);
    agent_memory_review(role_id, target_agent_id, store_path, reasons, None)
}

pub(crate) fn review_agent_self_patch_contract(
    expected_agent_id: &str,
    patch: &AgentSelfPatch,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if patch.agent_id.as_deref() != Some(expected_agent_id) {
        reasons.push(format!(
            "selfPatch agentId {:?} does not match this lane; expected {:?}",
            patch.agent_id, expected_agent_id
        ));
    }
    match patch.reason.as_deref() {
        Some(reason) if reason.trim().len() >= 16 && reason.len() <= 800 => {}
        _ => reasons.push(
            "selfPatch reason must be a bounded explanation of at least 16 characters".to_string(),
        ),
    }

    for key in patch.extra.keys() {
        if forbidden_patch_field(key) {
            reasons.push(format!(
                "selfPatch field {key:?} is project truth or authority; use the proper Epiphany control surface instead"
            ));
        } else if !allowed_patch_field(key) {
            reasons.push(format!(
                "selfPatch field {key:?} is not part of the bounded memory mutation contract"
            ));
        }
    }

    let mut mutation_count = 0;
    mutation_count += review_memory_patch_array(
        "semanticMemories",
        patch.semantic_memories.as_ref(),
        &mut reasons,
    );
    mutation_count += review_memory_patch_array(
        "episodicMemories",
        patch.episodic_memories.as_ref(),
        &mut reasons,
    );
    mutation_count += review_memory_patch_array(
        "relationshipMemories",
        patch.relationship_memories.as_ref(),
        &mut reasons,
    );
    mutation_count += review_goal_patch_array(patch.goals.as_ref(), &mut reasons);
    mutation_count += review_value_patch_array(patch.values.as_ref(), &mut reasons);
    mutation_count += review_private_notes(patch.private_notes.as_ref(), &mut reasons);
    review_string_array(
        "evidenceIds",
        patch.evidence_ids.as_ref(),
        &mut reasons,
        16,
        160,
    );
    if mutation_count == 0 {
        reasons.push(
            "selfPatch must contain at least one semantic memory, episodic memory, relationship memory, goal, value, or private note"
                .to_string(),
        );
    }

    reasons
}

pub fn apply_agent_self_patch(
    role_id: &str,
    patch_value: &Value,
    store_path: impl AsRef<Path>,
) -> Result<AgentMemoryReview> {
    let patch = match decode_agent_self_patch(patch_value) {
        Ok(patch) => patch,
        Err(_) => return Ok(review_agent_self_patch(role_id, patch_value, store_path)),
    };
    apply_agent_self_patch_document(role_id, patch, store_path)
}

pub fn apply_agent_self_patch_document(
    role_id: &str,
    patch: AgentSelfPatch,
    store_path: impl AsRef<Path>,
) -> Result<AgentMemoryReview> {
    let store_path = store_path.as_ref();
    let mut review = review_agent_self_patch_document(role_id, &patch, store_path);
    if review.status != "accepted" {
        return Ok(review);
    }
    let mut cache = agent_memory_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut entry = cache
        .get::<EpiphanyAgentMemoryEntry>(role_id)?
        .ok_or_else(|| anyhow!("CultCache has no role memory entry for {role_id:?}"))?;

    if let Some(incoming) = patch.semantic_memories {
        upsert_memories(&mut entry.agent.memories.semantic, incoming);
    }
    if let Some(incoming) = patch.episodic_memories {
        upsert_memories(&mut entry.agent.memories.episodic, incoming);
    }
    if let Some(incoming) = patch.relationship_memories {
        upsert_memories(&mut entry.agent.memories.relationship_summaries, incoming);
    }
    if let Some(incoming) = patch.goals {
        upsert_goals(&mut entry.agent.goals, incoming);
    }
    if let Some(incoming) = patch.values {
        upsert_values(&mut entry.agent.canonical_state.values, incoming);
    }
    if let Some(mut private_notes) = patch.private_notes {
        entry
            .agent
            .identity
            .private_notes
            .append(&mut private_notes);
        let keep_from = entry.agent.identity.private_notes.len().saturating_sub(32);
        entry.agent.identity.private_notes =
            entry.agent.identity.private_notes[keep_from..].to_vec();
    }
    cache.put(role_id.to_string(), &entry)?;
    review.applied = Some(true);
    Ok(review)
}

pub fn review_agent_memory_lifecycle_operation(
    role_id: &str,
    operation: &AgentMemoryLifecycleOperation,
    store_path: impl AsRef<Path>,
) -> AgentMemoryReview {
    let store_path = store_path.as_ref();
    let target_agent_id = match agent_id_for_role(role_id) {
        Ok(agent_id) => agent_id,
        Err(reason) => {
            return AgentMemoryReview {
                status: "rejected".to_string(),
                target_agent_id: String::new(),
                target_role_id: role_id.to_string(),
                target_store: store_path.display().to_string(),
                reasons: vec![reason],
                applied: None,
            };
        }
    };
    let reasons = review_memory_lifecycle_contract(target_agent_id, operation);
    agent_memory_review(role_id, target_agent_id, store_path, reasons, None)
}

pub fn apply_agent_memory_lifecycle_operation(
    role_id: &str,
    operation: AgentMemoryLifecycleOperation,
    store_path: impl AsRef<Path>,
) -> Result<AgentMemoryReview> {
    let store_path = store_path.as_ref();
    let mut review = review_agent_memory_lifecycle_operation(role_id, &operation, store_path);
    if review.status != "accepted" {
        return Ok(review);
    }
    let mut cache = agent_memory_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut entry = cache
        .get::<EpiphanyAgentMemoryEntry>(role_id)?
        .ok_or_else(|| anyhow!("CultCache has no role memory entry for {role_id:?}"))?;

    for action in operation.actions {
        apply_memory_lifecycle_action(&mut entry, action)?;
    }
    cache.put(role_id.to_string(), &entry)?;
    refresh_agent_state_soa(store_path)?;
    review.applied = Some(true);
    Ok(review)
}

fn review_memory_lifecycle_contract(
    expected_agent_id: &str,
    operation: &AgentMemoryLifecycleOperation,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if operation.agent_id.as_deref() != Some(expected_agent_id) {
        reasons.push(format!(
            "lifecycle agentId {:?} does not match this lane; expected {:?}",
            operation.agent_id, expected_agent_id
        ));
    }
    check_patch_text(&operation.reason, "lifecycle reason", &mut reasons, 800);
    if operation.actions.is_empty() {
        reasons.push("lifecycle operation must contain at least one action".to_string());
    }
    if operation.actions.len() > 8 {
        reasons.push("lifecycle operation may contain at most 8 actions".to_string());
    }
    for key in operation.extra.keys() {
        reasons.push(format!(
            "lifecycle field {key:?} is not part of the bounded memory lifecycle contract"
        ));
    }
    for (index, action) in operation.actions.iter().enumerate() {
        review_memory_lifecycle_action(index, action, &mut reasons);
    }
    reasons
}

fn review_memory_lifecycle_action(
    index: usize,
    action: &AgentMemoryLifecycleAction,
    reasons: &mut Vec<String>,
) {
    match action {
        AgentMemoryLifecycleAction::Revise {
            memory_id,
            summary,
            salience,
            confidence,
            reason,
            ..
        } => {
            review_memory_lifecycle_id(index, "memoryId", memory_id, reasons);
            if summary.is_none() && salience.is_none() && confidence.is_none() {
                reasons.push(format!(
                    "lifecycle actions[{index}] revise must change summary, salience, or confidence"
                ));
            }
            if let Some(summary) = summary {
                check_patch_text(
                    summary,
                    &format!("lifecycle actions[{index}].summary"),
                    reasons,
                    800,
                );
            }
            if let Some(salience) = salience {
                check_patch_unit(
                    *salience,
                    &format!("lifecycle actions[{index}].salience"),
                    reasons,
                );
            }
            if let Some(confidence) = confidence {
                check_patch_unit(
                    *confidence,
                    &format!("lifecycle actions[{index}].confidence"),
                    reasons,
                );
            }
            check_patch_text(
                reason,
                &format!("lifecycle actions[{index}].reason"),
                reasons,
                500,
            );
        }
        AgentMemoryLifecycleAction::Retire {
            memory_id, reason, ..
        } => {
            review_memory_lifecycle_id(index, "memoryId", memory_id, reasons);
            check_patch_text(
                reason,
                &format!("lifecycle actions[{index}].reason"),
                reasons,
                500,
            );
        }
        AgentMemoryLifecycleAction::Crystallize {
            memory_id,
            new_memory_id,
            summary,
            reason,
            ..
        } => {
            review_memory_lifecycle_id(index, "memoryId", memory_id, reasons);
            if let Some(new_memory_id) = new_memory_id {
                review_memory_lifecycle_id(index, "newMemoryId", new_memory_id, reasons);
            }
            if let Some(summary) = summary {
                check_patch_text(
                    summary,
                    &format!("lifecycle actions[{index}].summary"),
                    reasons,
                    800,
                );
            }
            check_patch_text(
                reason,
                &format!("lifecycle actions[{index}].reason"),
                reasons,
                500,
            );
        }
        AgentMemoryLifecycleAction::Prune {
            max_records,
            minimum_salience,
            reason,
            ..
        } => {
            if *max_records == 0 || *max_records > 128 {
                reasons.push(format!(
                    "lifecycle actions[{index}].maxRecords must be between 1 and 128"
                ));
            }
            if let Some(minimum_salience) = minimum_salience {
                check_patch_unit(
                    *minimum_salience,
                    &format!("lifecycle actions[{index}].minimumSalience"),
                    reasons,
                );
            }
            check_patch_text(
                reason,
                &format!("lifecycle actions[{index}].reason"),
                reasons,
                500,
            );
        }
        AgentMemoryLifecycleAction::Merge {
            target_memory_id,
            source_memory_ids,
            summary,
            salience,
            confidence,
            reason,
            ..
        } => {
            review_memory_lifecycle_id(index, "targetMemoryId", target_memory_id, reasons);
            if source_memory_ids.len() < 2 || source_memory_ids.len() > 8 {
                reasons.push(format!(
                    "lifecycle actions[{index}].sourceMemoryIds must contain 2 to 8 ids"
                ));
            }
            for source_id in source_memory_ids {
                review_memory_lifecycle_id(index, "sourceMemoryIds", source_id, reasons);
            }
            check_patch_text(
                summary,
                &format!("lifecycle actions[{index}].summary"),
                reasons,
                800,
            );
            check_patch_unit(
                *salience,
                &format!("lifecycle actions[{index}].salience"),
                reasons,
            );
            check_patch_unit(
                *confidence,
                &format!("lifecycle actions[{index}].confidence"),
                reasons,
            );
            check_patch_text(
                reason,
                &format!("lifecycle actions[{index}].reason"),
                reasons,
                500,
            );
        }
    }
}

fn review_memory_lifecycle_id(index: usize, field: &str, value: &str, reasons: &mut Vec<String>) {
    if !valid_identifier(value, "mem-") {
        reasons.push(format!(
            "lifecycle actions[{index}].{field} must start with 'mem-' and avoid whitespace"
        ));
    }
}

fn apply_memory_lifecycle_action(
    entry: &mut EpiphanyAgentMemoryEntry,
    action: AgentMemoryLifecycleAction,
) -> Result<()> {
    match action {
        AgentMemoryLifecycleAction::Revise {
            bundle,
            memory_id,
            summary,
            salience,
            confidence,
            ..
        } => {
            let memory = memory_bundle_mut(entry, bundle)
                .iter_mut()
                .find(|memory| memory.memory_id == memory_id)
                .ok_or_else(|| anyhow!("memory {memory_id:?} not found for revise"))?;
            if let Some(summary) = summary {
                memory.summary = summary;
            }
            if let Some(salience) = salience {
                memory.salience = salience;
            }
            if let Some(confidence) = confidence {
                memory.confidence = confidence;
            }
        }
        AgentMemoryLifecycleAction::Retire {
            bundle, memory_id, ..
        } => {
            let records = memory_bundle_mut(entry, bundle);
            let before = records.len();
            records.retain(|memory| memory.memory_id != memory_id);
            if records.len() == before {
                return Err(anyhow!("memory {memory_id:?} not found for retire"));
            }
        }
        AgentMemoryLifecycleAction::Crystallize {
            from_bundle,
            to_bundle,
            memory_id,
            new_memory_id,
            summary,
            ..
        } => {
            let source = memory_bundle_mut(entry, from_bundle)
                .iter()
                .find(|memory| memory.memory_id == memory_id)
                .cloned()
                .ok_or_else(|| anyhow!("memory {memory_id:?} not found for crystallize"))?;
            let mut crystallized = source;
            if let Some(new_memory_id) = new_memory_id {
                crystallized.memory_id = new_memory_id;
            }
            if let Some(summary) = summary {
                crystallized.summary = summary;
            }
            upsert_ghostlight_memory(memory_bundle_mut(entry, to_bundle), crystallized);
        }
        AgentMemoryLifecycleAction::Prune {
            bundle,
            max_records,
            minimum_salience,
            ..
        } => {
            let records = memory_bundle_mut(entry, bundle);
            if let Some(minimum_salience) = minimum_salience {
                records.retain(|memory| memory.salience >= minimum_salience);
            }
            records.sort_by(|left, right| {
                right
                    .salience
                    .partial_cmp(&left.salience)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| {
                        right
                            .confidence
                            .partial_cmp(&left.confidence)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .then_with(|| left.memory_id.cmp(&right.memory_id))
            });
            records.truncate(max_records);
        }
        AgentMemoryLifecycleAction::Merge {
            bundle,
            target_memory_id,
            source_memory_ids,
            summary,
            salience,
            confidence,
            ..
        } => {
            let records = memory_bundle_mut(entry, bundle);
            for source_id in &source_memory_ids {
                if !records.iter().any(|memory| memory.memory_id == *source_id) {
                    return Err(anyhow!("memory {source_id:?} not found for merge"));
                }
            }
            records.retain(|memory| {
                !source_memory_ids
                    .iter()
                    .any(|source_id| source_id == &memory.memory_id)
            });
            upsert_ghostlight_memory(
                records,
                GhostlightMemory {
                    memory_id: target_memory_id,
                    summary,
                    salience,
                    confidence,
                    linked_event_ids: None,
                    linked_relationship_id: None,
                },
            );
        }
    }
    Ok(())
}

fn memory_bundle_mut(
    entry: &mut EpiphanyAgentMemoryEntry,
    bundle: AgentMemoryBundle,
) -> &mut Vec<GhostlightMemory> {
    match bundle {
        AgentMemoryBundle::Semantic => &mut entry.agent.memories.semantic,
        AgentMemoryBundle::Episodic => &mut entry.agent.memories.episodic,
        AgentMemoryBundle::RelationshipSummaries => {
            &mut entry.agent.memories.relationship_summaries
        }
    }
}

fn upsert_ghostlight_memory(records: &mut Vec<GhostlightMemory>, incoming: GhostlightMemory) {
    if let Some(existing) = records
        .iter_mut()
        .find(|memory| memory.memory_id == incoming.memory_id)
    {
        *existing = incoming;
    } else {
        records.push(incoming);
    }
}

fn agent_memory_review(
    role_id: &str,
    target_agent_id: &str,
    store_path: &Path,
    reasons: Vec<String>,
    applied: Option<bool>,
) -> AgentMemoryReview {
    AgentMemoryReview {
        status: if reasons.is_empty() {
            "accepted"
        } else {
            "rejected"
        }
        .to_string(),
        target_agent_id: target_agent_id.to_string(),
        target_role_id: role_id.to_string(),
        target_store: store_path.display().to_string(),
        reasons,
        applied,
    }
}

pub fn apply_agent_canonical_trait_seeds(
    seeds: &[AgentCanonicalTraitSeed],
    store_path: impl AsRef<Path>,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    let mut cache = agent_memory_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut applied = Vec::new();
    for seed in seeds {
        if seed.trait_name.trim().is_empty() {
            return Err(anyhow!(
                "trait seed for role {:?} has an empty trait_name",
                seed.role_id
            ));
        }
        let vector = GhostlightTraitVector {
            mean: clamp_unit(seed.mean),
            plasticity: clamp_unit(seed.plasticity),
            current_activation: clamp_unit(seed.current_activation),
        };
        let mut entry = cache
            .get::<EpiphanyAgentMemoryEntry>(&seed.role_id)?
            .ok_or_else(|| anyhow!("CultCache has no role memory entry for {:?}", seed.role_id))?;
        let group = canonical_group_mut(&mut entry.agent.canonical_state, &seed.group_name)?;
        let previous = group.get(&seed.trait_name).cloned();
        if seed.trait_name != "baseline" {
            group.remove("baseline");
        }
        group.insert(seed.trait_name.clone(), vector.clone());
        cache.put(seed.role_id.clone(), &entry)?;
        applied.push(serde_json::json!({
            "roleId": seed.role_id,
            "groupName": seed.group_name,
            "traitName": seed.trait_name,
            "source": seed.source,
            "previous": previous,
            "vector": vector,
            "status": "applied",
        }));
    }
    Ok(serde_json::json!({
        "status": if applied.is_empty() { "no-seeds" } else { "applied" },
        "agentStore": store_path,
        "applied": applied.len(),
        "seeds": applied,
    }))
}

pub fn agent_memory_status(store_path: impl AsRef<Path>) -> Result<Value> {
    let store_path = store_path.as_ref();
    if !store_path.exists() {
        return Ok(serde_json::json!({
            "ok": false,
            "store": store_path,
            "present": false,
            "entryType": AGENT_MEMORY_TYPE,
            "roles": [],
        }));
    }
    let mut cache = agent_memory_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut roles = Vec::new();
    for (role_id, expected_agent_id, _) in ROLE_TARGETS {
        let entry = cache.get::<EpiphanyAgentMemoryEntry>(role_id)?;
        roles.push(match entry {
            Some(entry) => serde_json::json!({
                "roleId": role_id,
                "agentId": entry.agent.agent_id,
                "displayName": entry.agent.identity.name,
                "organStateProfile": organ_state_profile_for_role(role_id),
                "semanticMemories": entry.agent.memories.semantic.len(),
                "episodicMemories": entry.agent.memories.episodic.len(),
                "relationshipMemories": entry.agent.memories.relationship_summaries.len(),
                "goals": entry.agent.goals.len(),
                "values": entry.agent.canonical_state.values.len(),
            }),
            None => serde_json::json!({
                "roleId": role_id,
                "agentId": expected_agent_id,
                "missing": true,
            }),
        });
    }
    let errors = validate_agent_memory_store(store_path)?;
    let swarm_identity = cache
        .get::<AgentMemorySwarmIdentity>(AGENT_MEMORY_SWARM_IDENTITY_KEY)?
        .map(|identity| {
            serde_json::json!({
                "schemaVersion": identity.schema_version,
                "swarmId": identity.swarm_id,
            })
        });
    let agent_state_soa = project_agent_state_soa_from_cache(store_path, &cache)?;
    let persisted_agent_state_soa = cache
        .get::<EpiphanyAgentStateSoaEntry>(AGENT_STATE_SOA_KEY)?
        .map(|entry| {
            serde_json::json!({
                "schemaVersion": entry.schema_version,
                "generatedAt": entry.generated_at,
                "rowCount": entry.role_ids.len(),
                "sourceStore": entry.source_store,
            })
        });
    Ok(serde_json::json!({
        "ok": errors.is_empty(),
        "store": store_path,
        "present": true,
        "entryType": AGENT_MEMORY_TYPE,
        "swarmIdentityEntryType": AGENT_MEMORY_SWARM_IDENTITY_TYPE,
        "swarmIdentity": swarm_identity,
        "agentStateSoaEntryType": AGENT_STATE_SOA_TYPE,
        "errors": errors,
        "agentStateSoa": {
            "schemaVersion": agent_state_soa.schema_version,
            "rowCount": agent_state_soa.role_ids.len(),
            "roleIds": agent_state_soa.role_ids,
            "agentIds": agent_state_soa.agent_ids,
            "displayNames": agent_state_soa.display_names,
            "profileKinds": agent_state_soa.profile_kinds,
            "portableContracts": agent_state_soa.portable_contracts,
            "semanticMemoryCounts": agent_state_soa.semantic_memory_counts,
            "episodicMemoryCounts": agent_state_soa.episodic_memory_counts,
            "relationshipMemoryCounts": agent_state_soa.relationship_memory_counts,
            "goalCounts": agent_state_soa.goal_counts,
            "valueCounts": agent_state_soa.value_counts,
        },
        "persistedAgentStateSoa": persisted_agent_state_soa,
        "roles": roles,
    }))
}

pub fn refresh_agent_state_soa(store_path: impl AsRef<Path>) -> Result<Value> {
    let store_path = store_path.as_ref();
    let mut cache = agent_memory_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let entry = project_agent_state_soa_from_cache(store_path, &cache)?;
    validate_agent_state_soa_entry(&entry)?;
    cache.put(AGENT_STATE_SOA_KEY, &entry)?;
    Ok(serde_json::json!({
        "ok": true,
        "store": store_path,
        "entryType": AGENT_STATE_SOA_TYPE,
        "key": AGENT_STATE_SOA_KEY,
        "rowCount": entry.role_ids.len(),
        "roleIds": entry.role_ids,
        "agentIds": entry.agent_ids,
    }))
}

pub fn load_agent_state_soa_entry(
    store_path: impl AsRef<Path>,
) -> Result<Option<EpiphanyAgentStateSoaEntry>> {
    let store_path = store_path.as_ref();
    if !store_path.exists() {
        return Ok(None);
    }
    let mut cache = agent_memory_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<EpiphanyAgentStateSoaEntry>(AGENT_STATE_SOA_KEY)
}

fn project_agent_state_soa_from_cache(
    store_path: &Path,
    cache: &CultCache,
) -> Result<EpiphanyAgentStateSoaEntry> {
    let table = cache.soa::<EpiphanyAgentMemoryEntry>()?;
    let entry = EpiphanyAgentStateSoaEntry {
        schema_version: AGENT_STATE_SOA_SCHEMA_VERSION.to_string(),
        generated_at: now_rfc3339(),
        source_store: store_path.display().to_string(),
        role_ids: table.column::<String>("roleId")?.values().to_vec(),
        agent_ids: table.column::<String>("agentId")?.values().to_vec(),
        display_names: table.column::<String>("displayName")?.values().to_vec(),
        profile_kinds: table.column::<String>("profileKind")?.values().to_vec(),
        portable_contracts: table
            .column::<String>("portableContract")?
            .values()
            .to_vec(),
        semantic_memory_counts: table
            .column::<u32>("semanticMemoryCount")?
            .values()
            .to_vec(),
        episodic_memory_counts: table
            .column::<u32>("episodicMemoryCount")?
            .values()
            .to_vec(),
        relationship_memory_counts: table
            .column::<u32>("relationshipMemoryCount")?
            .values()
            .to_vec(),
        goal_counts: table.column::<u32>("goalCount")?.values().to_vec(),
        value_counts: table.column::<u32>("valueCount")?.values().to_vec(),
    };
    validate_agent_state_soa_entry(&entry)?;
    Ok(entry)
}

fn validate_agent_state_soa_entry(entry: &EpiphanyAgentStateSoaEntry) -> Result<()> {
    if entry.schema_version != AGENT_STATE_SOA_SCHEMA_VERSION {
        return Err(anyhow!(
            "agent state SoA schema_version must be {:?}",
            AGENT_STATE_SOA_SCHEMA_VERSION
        ));
    }
    let len = entry.role_ids.len();
    for (name, candidate) in [
        ("agentIds", entry.agent_ids.len()),
        ("displayNames", entry.display_names.len()),
        ("profileKinds", entry.profile_kinds.len()),
        ("portableContracts", entry.portable_contracts.len()),
        ("semanticMemoryCounts", entry.semantic_memory_counts.len()),
        ("episodicMemoryCounts", entry.episodic_memory_counts.len()),
        (
            "relationshipMemoryCounts",
            entry.relationship_memory_counts.len(),
        ),
        ("goalCounts", entry.goal_counts.len()),
        ("valueCounts", entry.value_counts.len()),
    ] {
        if candidate != len {
            return Err(anyhow!(
                "agent state SoA column {name} has length {candidate}, expected {len}"
            ));
        }
    }
    Ok(())
}

pub fn project_persona_state_for_role(
    store_path: impl AsRef<Path>,
    role_id: &str,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    let profile = organ_state_profile_for_role(role_id);
    if profile.profile_kind != EpiphanyOrganStateProfileKind::Persona {
        return Err(anyhow!(
            "{role_id:?} is {:?}, not persona; use epiphany.work_organ_state.v0 for work organs",
            profile.profile_kind
        ));
    }
    let entry = load_agent_memory_entry_for_role(store_path, role_id)?
        .ok_or_else(|| anyhow!("CultCache has no role memory entry for {role_id:?}"))?;
    Ok(project_persona_state_from_entry(&entry, store_path))
}

pub fn project_persona_state_from_entry(
    entry: &EpiphanyAgentMemoryEntry,
    store_path: &Path,
) -> Value {
    let exported_at = now_rfc3339();
    let persona_id = entry.agent.agent_id.clone();
    let repo_name = "EpiphanyAgent";
    serde_json::json!({
        "schemaVersion": PERSONA_STATE_SCHEMA_VERSION,
        "provenance": {
            "sourceSystem": "epiphany",
            "sourceDocumentId": format!("{}#{}", store_path.display(), entry.role_id),
            "sourceUpdatedAt": exported_at,
            "exportedAt": exported_at,
            "authority": "projection",
        },
        "personaId": persona_id,
        "publicName": &entry.agent.identity.name,
        "publicDescription": &entry.agent.identity.public_description,
        "presentation": {
            "voiceSummary": &entry.agent.identity.public_description,
            "defaultRenderer": "chat",
            "homeContext": {
                "kind": "repo",
                "id": repo_name,
                "label": repo_name,
            },
            "jurisdiction": entry
                .agent
                .identity
                .roles
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", "),
            "publicHandles": [],
        },
        "privateNotes": &entry.agent.identity.private_notes,
        "values": entry.agent.canonical_state.values.iter().map(persona_value).collect::<Vec<_>>(),
        "activationProfile": {
            "underlyingOrganization": persona_trait_map(&entry.agent.canonical_state.underlying_organization),
            "stableDispositions": persona_trait_map(&entry.agent.canonical_state.stable_dispositions),
            "behavioralDimensions": persona_trait_map(&entry.agent.canonical_state.behavioral_dimensions),
            "presentationStrategy": persona_trait_map(&entry.agent.canonical_state.presentation_strategy),
            "voiceStyle": persona_trait_map(&entry.agent.canonical_state.voice_style),
            "situationalState": persona_trait_map(&entry.agent.canonical_state.situational_state),
        },
        "thoughtMemory": {
            "shortTerm": [],
            "memories": entry
                .agent
                .memories
                .semantic
                .iter()
                .chain(entry.agent.memories.episodic.iter())
                .chain(entry.agent.memories.relationship_summaries.iter())
                .map(|memory| persona_memory_thought(memory, &persona_id, &exported_at))
                .collect::<Vec<_>>(),
            "incubation": [],
        },
        "agencyPressure": {
            "pressures": entry.agent.goals.iter().map(|goal| persona_goal_thought(goal, &persona_id, &exported_at)).collect::<Vec<_>>(),
        },
        "candidateActions": {
            "actions": [],
        },
        "voidbotProjection": {
            "candidateInterventions": [],
        },
        "affect": {
            "needs": [],
            "socialBonds": [],
            "statusReads": entry.agent.perceived_state_overlays.iter().map(|overlay| persona_status_read(overlay, &persona_id, &exported_at)).collect::<Vec<_>>(),
            "moodDimensions": [],
            "socialBiases": [],
            "doctrineStances": entry.agent.canonical_state.values.iter().map(|value| persona_doctrine_stance(value, &persona_id, &exported_at)).collect::<Vec<_>>(),
        },
        "updatedAt": exported_at,
    })
}

fn persona_trait_map(vectors: &BTreeMap<String, GhostlightTraitVector>) -> Value {
    let mut map = serde_json::Map::new();
    for (name, vector) in vectors {
        map.insert(
            name.clone(),
            serde_json::json!({
                "mean": vector.mean,
                "plasticity": vector.plasticity,
                "currentActivation": vector.current_activation,
            }),
        );
    }
    Value::Object(map)
}

fn persona_value(value: &GhostlightValue) -> Value {
    serde_json::json!({
        "id": value.value_id,
        "label": value.label,
        "priority": value.priority,
        "summary": if value.unforgivable_if_betrayed {
            "Protected value; betrayal is marked as unforgivable in local Persona state."
        } else {
            "Value projected from local Persona organ memory."
        },
    })
}

fn persona_target(kind: &str, id: &str, label: &str) -> Value {
    serde_json::json!({
        "kind": kind,
        "id": id,
        "label": label,
    })
}

fn persona_memory_thought(memory: &GhostlightMemory, persona_id: &str, timestamp: &str) -> Value {
    serde_json::json!({
        "id": memory.memory_id,
        "status": "crystallized",
        "target": persona_target("self", persona_id, persona_id),
        "summary": memory.summary,
        "claim": memory.summary,
        "tension": "Projected from local Persona memory; consumers should preserve provenance and avoid treating this as omniscient project truth.",
        "actionImplication": "Use as Persona memory pressure, not as direct action authority.",
        "intensity": memory.salience,
        "valence": 0,
        "createdAt": timestamp,
        "updatedAt": timestamp,
        "tags": ["epiphany", "Persona-memory"],
    })
}

fn persona_goal_thought(goal: &GhostlightGoal, persona_id: &str, timestamp: &str) -> Value {
    serde_json::json!({
        "id": goal.goal_id,
        "status": persona_goal_status(&goal.status),
        "target": persona_target("self", persona_id, persona_id),
        "summary": goal.description,
        "claim": goal.description,
        "tension": goal.emotional_stake,
        "actionImplication": "This goal may create Persona agency pressure, but action still requires the caller's review path.",
        "intensity": goal.priority,
        "createdAt": timestamp,
        "updatedAt": timestamp,
        "tags": ["epiphany", "Persona-goal", &goal.scope],
    })
}

fn persona_status_read(
    overlay: &GhostlightPerceivedStateOverlay,
    persona_id: &str,
    timestamp: &str,
) -> Value {
    serde_json::json!({
        "id": overlay.overlay_id,
        "status": "active",
        "target": persona_target("self", persona_id, &overlay.label),
        "statusKind": "uncertainty",
        "summary": overlay.summary,
        "confidence": overlay.confidence,
        "intensity": overlay.salience,
        "valence": 0,
        "updatedAt": timestamp,
        "extensions": {
            "source": overlay.source,
            "linkedMemoryIds": overlay.linked_memory_ids,
        },
    })
}

fn persona_doctrine_stance(value: &GhostlightValue, persona_id: &str, timestamp: &str) -> Value {
    serde_json::json!({
        "id": format!("stance-{}", value.value_id),
        "status": "active",
        "target": persona_target("self", persona_id, persona_id),
        "stanceKind": "aligned",
        "principle": value.label,
        "summary": value.label,
        "actionImplication": "Let this value bend Persona speech and interpretation without granting automatic action authority.",
        "intensity": value.priority,
        "updatedAt": timestamp,
    })
}

fn persona_goal_status(status: &str) -> &'static str {
    match status {
        "active" => "active",
        "blocked" | "dormant" => "cooling",
        "resolved" => "resolved",
        "abandoned" => "retired",
        _ => "draft",
    }
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn organ_state_profile_for_role(role_id: &str) -> EpiphanyOrganStateProfile {
    match role_id {
        "Persona" => EpiphanyOrganStateProfile {
            profile_kind: EpiphanyOrganStateProfileKind::Persona,
            state_density: "persona_grade".to_string(),
            portable_contract: "gamecult.persona_state.v0".to_string(),
            relationship_model: "relationship_summaries_and_directional_stance_matter".to_string(),
            affect_model: "persona_affect_allowed_and_expected".to_string(),
            perceived_overlay_mode: "observer_local_and_fallible".to_string(),
            growth_channels: vec![
                "heartbeat appraisal and reaction".to_string(),
                "character-loop interpretation".to_string(),
                "persona affect and social-read interpretation".to_string(),
                "episodic and relationship memory accumulation".to_string(),
                "reviewed selfPatch".to_string(),
                "sleep/distillation".to_string(),
            ],
            notes: vec![
                "Epiphany Persona is an organ; Persona is the portable person-state contract shared with Ghostlight and VoidBot-style repo Personas.".to_string(),
                "Dense canonical families, affect, perceived overlays, and relationship pressure are appropriate for Persona state.".to_string(),
            ],
        },
        _ => EpiphanyOrganStateProfile {
            profile_kind: EpiphanyOrganStateProfileKind::WorkOrgan,
            state_density: "lean_work_organ".to_string(),
            portable_contract: "epiphany.work_organ_state.v0".to_string(),
            relationship_model: "role_local_summary_only".to_string(),
            affect_model: "no_affect_or_persona_machinery".to_string(),
            perceived_overlay_mode: "minimal_until_a_real_need_exists".to_string(),
            growth_channels: vec![
                "reviewed selfPatch".to_string(),
                "heartbeat rumination pressure".to_string(),
                "sleep/distillation".to_string(),
                "birth-time repo memory and light operating-pressure seeding".to_string(),
            ],
            notes: vec![
                "Work organs need sharp role identity, durable mission memory, values/goals, and heartbeat activation; they do not need Persona affect or full Persona machinery.".to_string(),
                "Sparse canonical bundles are acceptable here as long as memory, goals, values, and private notes can deepen over time.".to_string(),
            ],
        },
    }
}

pub fn project_agent_memory_to_json_dir(
    store_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    let output_dir = output_dir.as_ref();
    let mut cache = agent_memory_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;
    let mut projected = Vec::new();
    for (role_id, _, filename) in ROLE_TARGETS {
        let entry = cache
            .get::<EpiphanyAgentMemoryEntry>(role_id)?
            .ok_or_else(|| anyhow!("missing role memory entry for {role_id:?}"))?;
        let projection = AgentMemoryProjection {
            schema_version: entry.schema_version,
            world: entry.world,
            agents: vec![entry.agent],
            relationships: entry.relationships,
            events: entry.events,
            scenes: entry.scenes,
        };
        let path = output_dir.join(filename);
        fs::write(
            &path,
            format!("{}\n", serde_json::to_string_pretty(&projection)?),
        )
        .with_context(|| format!("failed to write {}", path.display()))?;
        projected.push(serde_json::json!({"roleId": role_id, "path": path}));
    }
    Ok(serde_json::json!({
        "ok": true,
        "store": store_path,
        "outputDir": output_dir,
        "projected": projected,
    }))
}

fn agent_memory_cache(store_path: &Path) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<AgentMemorySwarmIdentity>()?;
    cache.register_entry_type::<EpiphanyAgentMemoryEntry>()?;
    cache.register_entry_type::<EpiphanyAgentStateSoaEntry>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(store_path));
    Ok(cache)
}

fn canonical_group_mut<'a>(
    state: &'a mut GhostlightCanonicalState,
    group_name: &str,
) -> Result<&'a mut BTreeMap<String, GhostlightTraitVector>> {
    match group_name {
        "underlying_organization" => Ok(&mut state.underlying_organization),
        "stable_dispositions" => Ok(&mut state.stable_dispositions),
        "behavioral_dimensions" => Ok(&mut state.behavioral_dimensions),
        "presentation_strategy" => Ok(&mut state.presentation_strategy),
        "voice_style" => Ok(&mut state.voice_style),
        "situational_state" => Ok(&mut state.situational_state),
        other => Err(anyhow!(
            "unknown canonical_state group {:?}; expected one of the six Ghostlight trait bundles",
            other
        )),
    }
}

fn entry_from_projection(
    role_id: &str,
    expected_agent_id: &str,
    projection: AgentMemoryProjection,
) -> Result<EpiphanyAgentMemoryEntry> {
    if projection.schema_version != AGENT_MEMORY_SCHEMA_VERSION {
        return Err(anyhow!(
            "schema_version must be {:?}",
            AGENT_MEMORY_SCHEMA_VERSION
        ));
    }
    if projection.agents.len() != 1 {
        return Err(anyhow!(
            "role memory projection must contain exactly one agent"
        ));
    }
    let agent = projection.agents.into_iter().next().expect("checked len");
    if agent.agent_id != expected_agent_id {
        return Err(anyhow!(
            "agent_id {:?} does not match expected {:?}",
            agent.agent_id,
            expected_agent_id
        ));
    }
    Ok(EpiphanyAgentMemoryEntry {
        schema_version: projection.schema_version,
        role_id: role_id.to_string(),
        world: projection.world,
        agent,
        relationships: projection.relationships,
        events: projection.events,
        scenes: projection.scenes,
    })
}

fn validate_agent_entry(entry: &EpiphanyAgentMemoryEntry, expected_agent_id: &str) -> Vec<String> {
    let mut errors = Vec::new();
    if entry.schema_version != AGENT_MEMORY_SCHEMA_VERSION {
        errors.push(format!(
            "{}: schema_version must be {:?}",
            entry.role_id, AGENT_MEMORY_SCHEMA_VERSION
        ));
    }
    check_string(&entry.world.world_id, "world.world_id", &mut errors, 120);
    check_string(&entry.world.setting, "world.setting", &mut errors, 800);
    check_string(
        &entry.world.time.label,
        "world.time.label",
        &mut errors,
        200,
    );
    if entry.world.canon_context.is_empty() {
        errors.push("world.canon_context must not be empty".to_string());
    }
    if entry.agent.agent_id != expected_agent_id {
        errors.push(format!(
            "{}: agent_id {:?} does not match expected {:?}",
            entry.role_id, entry.agent.agent_id, expected_agent_id
        ));
    }
    check_string(
        &entry.agent.identity.name,
        "identity.name",
        &mut errors,
        200,
    );
    check_string(
        &entry.agent.identity.origin,
        "identity.origin",
        &mut errors,
        800,
    );
    check_string(
        &entry.agent.identity.public_description,
        "identity.public_description",
        &mut errors,
        800,
    );
    if entry.agent.identity.roles.is_empty() {
        errors.push("identity.roles must not be empty".to_string());
    }
    for (bundle, memories) in [
        ("episodic", &entry.agent.memories.episodic),
        ("semantic", &entry.agent.memories.semantic),
        (
            "relationship_summaries",
            &entry.agent.memories.relationship_summaries,
        ),
    ] {
        for (index, memory) in memories.iter().enumerate() {
            validate_memory(memory, &format!("memories.{bundle}[{index}]"), &mut errors);
        }
    }
    for (index, overlay) in entry.agent.perceived_state_overlays.iter().enumerate() {
        validate_overlay(
            overlay,
            &format!("perceived_state_overlays[{index}]"),
            &mut errors,
        );
    }
    for (index, relationship) in entry.relationships.iter().enumerate() {
        validate_relationship(
            relationship,
            &format!("relationships[{index}]"),
            &mut errors,
        );
    }
    for (index, event) in entry.events.iter().enumerate() {
        validate_event(event, &format!("events[{index}]"), &mut errors);
    }
    for (index, scene) in entry.scenes.iter().enumerate() {
        validate_scene(scene, &format!("scenes[{index}]"), &mut errors);
    }
    for (index, goal) in entry.agent.goals.iter().enumerate() {
        validate_goal(goal, &format!("goals[{index}]"), &mut errors);
    }
    for (index, value) in entry.agent.canonical_state.values.iter().enumerate() {
        validate_value(value, &format!("values[{index}]"), &mut errors);
    }
    let profile = organ_state_profile_for_role(&entry.role_id);
    for (group_name, group) in [
        (
            "underlying_organization",
            &entry.agent.canonical_state.underlying_organization,
        ),
        (
            "stable_dispositions",
            &entry.agent.canonical_state.stable_dispositions,
        ),
        (
            "behavioral_dimensions",
            &entry.agent.canonical_state.behavioral_dimensions,
        ),
        (
            "presentation_strategy",
            &entry.agent.canonical_state.presentation_strategy,
        ),
        ("voice_style", &entry.agent.canonical_state.voice_style),
        (
            "situational_state",
            &entry.agent.canonical_state.situational_state,
        ),
    ] {
        if group.is_empty() && profile.profile_kind == EpiphanyOrganStateProfileKind::Persona {
            errors.push(format!("canonical_state.{group_name} must not be empty"));
        }
        for (name, vector) in group {
            validate_trait_vector(
                vector,
                &format!("canonical_state.{group_name}.{name}"),
                &mut errors,
            );
        }
    }
    errors
}

fn validate_memory(memory: &GhostlightMemory, path: &str, errors: &mut Vec<String>) {
    check_string(&memory.memory_id, &format!("{path}.memory_id"), errors, 120);
    check_string(&memory.summary, &format!("{path}.summary"), errors, 800);
    check_unit(memory.salience, &format!("{path}.salience"), errors);
    check_unit(memory.confidence, &format!("{path}.confidence"), errors);
}

fn validate_overlay(
    overlay: &GhostlightPerceivedStateOverlay,
    path: &str,
    errors: &mut Vec<String>,
) {
    check_optional_identifier(&overlay.overlay_id, &format!("{path}.overlay_id"), errors);
    check_string(&overlay.summary, &format!("{path}.summary"), errors, 800);
    check_optional_text(&overlay.label, &format!("{path}.label"), errors, 240);
    check_optional_text(&overlay.source, &format!("{path}.source"), errors, 240);
    check_unit(overlay.salience, &format!("{path}.salience"), errors);
    check_unit(overlay.confidence, &format!("{path}.confidence"), errors);
}

fn validate_relationship(
    relationship: &GhostlightRelationship,
    path: &str,
    errors: &mut Vec<String>,
) {
    check_optional_identifier(
        &relationship.relationship_id,
        &format!("{path}.relationship_id"),
        errors,
    );
    check_string(
        &relationship.summary,
        &format!("{path}.summary"),
        errors,
        800,
    );
    check_optional_text(&relationship.stance, &format!("{path}.stance"), errors, 240);
    if relationship.participant_ids.is_empty() {
        errors.push(format!("{path}.participant_ids must not be empty"));
    }
    check_unit(relationship.salience, &format!("{path}.salience"), errors);
    check_unit(
        relationship.confidence,
        &format!("{path}.confidence"),
        errors,
    );
}

fn validate_event(event: &GhostlightEvent, path: &str, errors: &mut Vec<String>) {
    check_optional_identifier(&event.event_id, &format!("{path}.event_id"), errors);
    check_string(&event.kind, &format!("{path}.kind"), errors, 120);
    check_string(&event.summary, &format!("{path}.summary"), errors, 800);
    if event.participant_ids.is_empty() {
        errors.push(format!("{path}.participant_ids must not be empty"));
    }
    check_unit(event.salience, &format!("{path}.salience"), errors);
    check_unit(event.confidence, &format!("{path}.confidence"), errors);
}

fn validate_scene(scene: &GhostlightScene, path: &str, errors: &mut Vec<String>) {
    check_optional_identifier(&scene.scene_id, &format!("{path}.scene_id"), errors);
    check_string(&scene.label, &format!("{path}.label"), errors, 240);
    check_string(&scene.summary, &format!("{path}.summary"), errors, 800);
    if scene.participant_ids.is_empty() {
        errors.push(format!("{path}.participant_ids must not be empty"));
    }
    check_unit(scene.salience, &format!("{path}.salience"), errors);
    check_optional_text(&scene.status, &format!("{path}.status"), errors, 120);
}

fn validate_goal(goal: &GhostlightGoal, path: &str, errors: &mut Vec<String>) {
    check_string(&goal.goal_id, &format!("{path}.goal_id"), errors, 120);
    check_string(
        &goal.description,
        &format!("{path}.description"),
        errors,
        800,
    );
    if !matches!(
        goal.scope.as_str(),
        "immediate" | "scene" | "case" | "arc" | "life"
    ) {
        errors.push(format!("{path}.scope is not a Ghostlight scope"));
    }
    check_unit(goal.priority, &format!("{path}.priority"), errors);
    check_string(
        &goal.emotional_stake,
        &format!("{path}.emotional_stake"),
        errors,
        400,
    );
    if !matches!(
        goal.status.as_str(),
        "active" | "blocked" | "dormant" | "resolved" | "abandoned"
    ) {
        errors.push(format!("{path}.status is not a Ghostlight status"));
    }
}

fn validate_value(value: &GhostlightValue, path: &str, errors: &mut Vec<String>) {
    check_string(&value.value_id, &format!("{path}.value_id"), errors, 120);
    check_string(&value.label, &format!("{path}.label"), errors, 240);
    check_unit(value.priority, &format!("{path}.priority"), errors);
}

fn validate_trait_vector(vector: &GhostlightTraitVector, path: &str, errors: &mut Vec<String>) {
    check_unit(vector.mean, &format!("{path}.mean"), errors);
    check_unit(vector.plasticity, &format!("{path}.plasticity"), errors);
    check_unit(
        vector.current_activation,
        &format!("{path}.current_activation"),
        errors,
    );
}

fn check_string(value: &str, path: &str, errors: &mut Vec<String>, max_len: usize) {
    if value.trim().is_empty() || value.len() > max_len {
        errors.push(format!(
            "{path} must be non-empty text under {max_len} characters"
        ));
    }
}

fn check_optional_text(value: &str, path: &str, errors: &mut Vec<String>, max_len: usize) {
    if !value.is_empty() && (value.trim().is_empty() || value.len() > max_len) {
        errors.push(format!(
            "{path} must be empty or text under {max_len} characters"
        ));
    }
}

fn check_optional_identifier(value: &str, path: &str, errors: &mut Vec<String>) {
    if value.is_empty() {
        return;
    }
    if value.len() > 120
        || !value.chars().all(|ch| {
            ch.is_ascii() && (ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
        })
    {
        errors.push(format!(
            "{path} must be empty or an ASCII identifier without whitespace"
        ));
    }
}

fn check_unit(value: f64, path: &str, errors: &mut Vec<String>) {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        errors.push(format!("{path} must be between 0 and 1"));
    }
}

fn clamp_unit(value: f64) -> f64 {
    if !value.is_finite() {
        0.5
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn agent_id_for_role(role_id: &str) -> std::result::Result<&'static str, String> {
    ROLE_TARGETS
        .iter()
        .find_map(|(candidate_role, agent_id, _)| (*candidate_role == role_id).then_some(*agent_id))
        .ok_or_else(|| format!("unknown role id: {role_id}"))
}

fn allowed_patch_field(key: &str) -> bool {
    matches!(
        key,
        "agentId"
            | "reason"
            | "evidenceIds"
            | "semanticMemories"
            | "episodicMemories"
            | "relationshipMemories"
            | "goals"
            | "values"
            | "privateNotes"
    )
}

fn forbidden_patch_field(key: &str) -> bool {
    matches!(
        key,
        "statePatch"
            | "objective"
            | "activeSubgoalId"
            | "subgoals"
            | "invariants"
            | "graphs"
            | "graphFrontier"
            | "graphCheckpoint"
            | "scratch"
            | "investigationCheckpoint"
            | "jobBindings"
            | "planning"
            | "churn"
            | "mode"
            | "codeEdits"
            | "files"
            | "authorityScope"
            | "backendJobId"
            | "rawResult"
    )
}

fn review_memory_patch_array(
    field: &str,
    value: Option<&Vec<SelfPatchMemory>>,
    reasons: &mut Vec<String>,
) -> usize {
    let Some(value) = value else {
        return 0;
    };
    if value.len() > 8 {
        reasons.push(format!("selfPatch {field} may contain at most 8 records"));
    }
    for (index, item) in value.iter().enumerate() {
        if !valid_identifier(&item.memory_id, "mem-") {
            reasons.push(format!(
                "selfPatch {field}[{index}].memoryId must start with 'mem-' and avoid whitespace"
            ));
        }
        check_patch_text(
            &item.summary,
            &format!("selfPatch {field}[{index}].summary"),
            reasons,
            600,
        );
        check_patch_unit(
            item.salience,
            &format!("selfPatch {field}[{index}].salience"),
            reasons,
        );
        check_patch_unit(
            item.confidence,
            &format!("selfPatch {field}[{index}].confidence"),
            reasons,
        );
    }
    value.len()
}

fn review_goal_patch_array(value: Option<&Vec<SelfPatchGoal>>, reasons: &mut Vec<String>) -> usize {
    let Some(value) = value else {
        return 0;
    };
    if value.len() > 6 {
        reasons.push("selfPatch goals may contain at most 6 records".to_string());
    }
    for (index, item) in value.iter().enumerate() {
        if !valid_identifier(&item.goal_id, "goal-") {
            reasons.push(format!(
                "selfPatch goals[{index}].goalId must start with 'goal-' and avoid whitespace"
            ));
        }
        check_patch_text(
            &item.description,
            &format!("selfPatch goals[{index}].description"),
            reasons,
            700,
        );
        if !matches!(
            item.scope.as_str(),
            "immediate" | "scene" | "case" | "arc" | "life"
        ) {
            reasons.push(format!(
                "selfPatch goals[{index}].scope is not a Ghostlight scope"
            ));
        }
        check_patch_unit(
            item.priority,
            &format!("selfPatch goals[{index}].priority"),
            reasons,
        );
        check_patch_text(
            &item.emotional_stake,
            &format!("selfPatch goals[{index}].emotionalStake"),
            reasons,
            400,
        );
        if !matches!(
            item.status.as_str(),
            "active" | "blocked" | "dormant" | "resolved" | "abandoned"
        ) {
            reasons.push(format!(
                "selfPatch goals[{index}].status is not a Ghostlight status"
            ));
        }
    }
    value.len()
}

fn review_value_patch_array(
    value: Option<&Vec<SelfPatchValue>>,
    reasons: &mut Vec<String>,
) -> usize {
    let Some(value) = value else {
        return 0;
    };
    if value.len() > 6 {
        reasons.push("selfPatch values may contain at most 6 records".to_string());
    }
    for (index, item) in value.iter().enumerate() {
        if !valid_identifier(&item.value_id, "value-") {
            reasons.push(format!(
                "selfPatch values[{index}].valueId must start with 'value-' and avoid whitespace"
            ));
        }
        check_patch_text(
            &item.label,
            &format!("selfPatch values[{index}].label"),
            reasons,
            240,
        );
        check_patch_unit(
            item.priority,
            &format!("selfPatch values[{index}].priority"),
            reasons,
        );
    }
    value.len()
}

fn review_private_notes(value: Option<&Vec<String>>, reasons: &mut Vec<String>) -> usize {
    let Some(value) = value else {
        return 0;
    };
    if value.len() > 6 {
        reasons.push("selfPatch privateNotes may contain at most 6 records".to_string());
    }
    for (index, item) in value.iter().enumerate() {
        check_patch_text(
            item,
            &format!("selfPatch privateNotes[{index}]"),
            reasons,
            600,
        );
    }
    value.len()
}

fn review_string_array(
    field: &str,
    value: Option<&Vec<String>>,
    reasons: &mut Vec<String>,
    max_items: usize,
    max_len: usize,
) {
    let Some(value) = value else {
        return;
    };
    if value.len() > max_items {
        reasons.push(format!(
            "selfPatch {field} may contain at most {max_items} records"
        ));
    }
    for (index, item) in value.iter().enumerate() {
        check_patch_text(
            item,
            &format!("selfPatch {field}[{index}]"),
            reasons,
            max_len,
        );
    }
}

fn check_patch_text(value: &str, path: &str, reasons: &mut Vec<String>, max_len: usize) {
    if value.trim().is_empty() || value.len() > max_len {
        reasons.push(format!(
            "{path} must be non-empty text under {max_len} characters"
        ));
    }
}

fn check_patch_unit(value: f64, path: &str, reasons: &mut Vec<String>) {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        reasons.push(format!("{path} must be between 0 and 1"));
    }
}

fn valid_identifier(value: &str, prefix: &str) -> bool {
    value.starts_with(prefix)
        && value.len() <= 120
        && value.chars().all(|ch| {
            ch.is_ascii() && (ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
        })
}

fn upsert_memories(records: &mut Vec<GhostlightMemory>, incoming: Vec<SelfPatchMemory>) {
    let mut index: BTreeMap<String, GhostlightMemory> = records
        .iter()
        .cloned()
        .map(|record| (record.memory_id.clone(), record))
        .collect();
    for item in incoming {
        index.insert(
            item.memory_id.clone(),
            GhostlightMemory {
                memory_id: item.memory_id,
                summary: item.summary,
                salience: item.salience,
                confidence: item.confidence,
                linked_event_ids: item.linked_event_ids,
                linked_relationship_id: item.linked_relationship_id,
            },
        );
    }
    *records = index.into_values().collect();
}

fn upsert_goals(records: &mut Vec<GhostlightGoal>, incoming: Vec<SelfPatchGoal>) {
    let mut index: BTreeMap<String, GhostlightGoal> = records
        .iter()
        .cloned()
        .map(|record| (record.goal_id.clone(), record))
        .collect();
    for item in incoming {
        index.insert(
            item.goal_id.clone(),
            GhostlightGoal {
                goal_id: item.goal_id,
                description: item.description,
                scope: item.scope,
                priority: item.priority,
                emotional_stake: item.emotional_stake,
                blockers: item.blockers,
                status: item.status,
            },
        );
    }
    *records = index.into_values().collect();
}

fn upsert_values(records: &mut Vec<GhostlightValue>, incoming: Vec<SelfPatchValue>) {
    let mut index: BTreeMap<String, GhostlightValue> = records
        .iter()
        .cloned()
        .map(|record| (record.value_id.clone(), record))
        .collect();
    for item in incoming {
        index.insert(
            item.value_id.clone(),
            GhostlightValue {
                value_id: item.value_id,
                label: item.label,
                priority: item.priority,
                unforgivable_if_betrayed: item.unforgivable_if_betrayed,
            },
        );
    }
    *records = index.into_values().collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn role_memory_migrates_reviews_and_applies_native_patch() -> Result<()> {
        let temp = tempdir()?;
        let agent_dir = temp.path().join("agents");
        fs::create_dir_all(&agent_dir)?;
        fs::write(
            agent_dir.join("modeling.agent-state.json"),
            sample_agent_json("epiphany.modeling", "Modeling"),
        )?;
        for (role_id, agent_id, filename) in ROLE_TARGETS {
            if *role_id == "modeling" {
                continue;
            }
            fs::write(
                agent_dir.join(filename),
                sample_agent_json(agent_id, &format!("Agent {role_id}")),
            )?;
        }
        let store = temp.path().join("agents.msgpack");
        migrate_agent_memory_json_dir_to_cultcache(&agent_dir, &store)?;
        assert!(validate_agent_memory_store(&store)?.is_empty());

        let patch: AgentSelfPatch = serde_json::from_value(serde_json::json!({
            "agentId": "epiphany.modeling",
            "reason": "Modeling should remember accepted graph growth must stay source-grounded.",
            "semanticMemories": [{
                "memoryId": "mem-body-native-source-grounding",
                "summary": "Native role memory patches update typed CultCache organ state rather than JSON dossier files.",
                "salience": 0.74,
                "confidence": 0.86
            }]
        }))?;
        let review = review_agent_self_patch_document("modeling", &patch, &store);
        assert_eq!(review.status, "accepted");
        let applied = apply_agent_self_patch_document("modeling", patch, &store)?;
        assert_eq!(applied.status, "accepted");
        assert_eq!(applied.applied, Some(true));

        let mut cache = agent_memory_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let modeling = cache.get_required::<EpiphanyAgentMemoryEntry>("modeling")?;
        assert!(
            modeling
                .agent
                .memories
                .semantic
                .iter()
                .any(|memory| memory.memory_id == "mem-body-native-source-grounding")
        );
        Ok(())
    }

    #[test]
    fn memory_lifecycle_operation_revises_crystallizes_merges_retires_and_prunes() -> Result<()> {
        let temp = tempdir()?;
        let agent_dir = temp.path().join("agents");
        fs::create_dir_all(&agent_dir)?;
        for (role_id, agent_id, filename) in ROLE_TARGETS {
            fs::write(
                agent_dir.join(filename),
                sample_agent_json(agent_id, &format!("Agent {role_id}")),
            )?;
        }
        let store = temp.path().join("agents.msgpack");
        migrate_agent_memory_json_dir_to_cultcache(&agent_dir, &store)?;

        let patch: AgentSelfPatch = serde_json::from_value(serde_json::json!({
            "agentId": "epiphany.modeling",
            "reason": "Seed memories so the lifecycle operation can prove maintenance without identity edits.",
            "semanticMemories": [
                {
                    "memoryId": "mem-lifecycle-alpha",
                    "summary": "Alpha memory starts as short doctrine needing revision.",
                    "salience": 0.42,
                    "confidence": 0.7
                },
                {
                    "memoryId": "mem-lifecycle-beta",
                    "summary": "Beta support should merge with gamma.",
                    "salience": 0.31,
                    "confidence": 0.72
                },
                {
                    "memoryId": "mem-lifecycle-gamma",
                    "summary": "Gamma support should merge with beta.",
                    "salience": 0.33,
                    "confidence": 0.74
                }
            ]
        }))?;
        assert_eq!(
            apply_agent_self_patch_document("modeling", patch, &store)?.status,
            "accepted"
        );

        let operation = AgentMemoryLifecycleOperation {
            agent_id: Some("epiphany.modeling".to_string()),
            reason: "Sleep maintenance is revising, crystallizing, merging, retiring, and pruning bounded role memory.".to_string(),
            actions: vec![
                AgentMemoryLifecycleAction::Revise {
                    bundle: AgentMemoryBundle::Semantic,
                    memory_id: "mem-lifecycle-alpha".to_string(),
                    summary: Some("Alpha memory was revised before crystallization.".to_string()),
                    salience: Some(0.81),
                    confidence: None,
                    reason: "The old wording was too weak for durable modeling doctrine.".to_string(),
                },
                AgentMemoryLifecycleAction::Crystallize {
                    from_bundle: AgentMemoryBundle::Semantic,
                    to_bundle: AgentMemoryBundle::Episodic,
                    memory_id: "mem-lifecycle-alpha".to_string(),
                    new_memory_id: Some("mem-lifecycle-alpha-episode".to_string()),
                    summary: Some("Alpha became an episodic scar from sleep maintenance.".to_string()),
                    reason: "The revised pressure should leave a separate episodic witness.".to_string(),
                },
                AgentMemoryLifecycleAction::Merge {
                    bundle: AgentMemoryBundle::Semantic,
                    target_memory_id: "mem-lifecycle-merged".to_string(),
                    source_memory_ids: vec![
                        "mem-lifecycle-beta".to_string(),
                        "mem-lifecycle-gamma".to_string(),
                    ],
                    summary: "Beta and gamma collapsed into one support memory.".to_string(),
                    salience: 0.77,
                    confidence: 0.82,
                    reason: "Duplicate support belongs in one clearer memory.".to_string(),
                },
                AgentMemoryLifecycleAction::Retire {
                    bundle: AgentMemoryBundle::Semantic,
                    memory_id: "mem-lifecycle-alpha".to_string(),
                    reason: "The revised alpha pressure was crystallized and should leave semantic room.".to_string(),
                },
                AgentMemoryLifecycleAction::Prune {
                    bundle: AgentMemoryBundle::Semantic,
                    max_records: 1,
                    minimum_salience: Some(0.5),
                    reason: "Keep only strong semantic residue after merge.".to_string(),
                },
            ],
            extra: BTreeMap::new(),
        };

        let review = review_agent_memory_lifecycle_operation("modeling", &operation, &store);
        assert_eq!(review.status, "accepted");
        let applied = apply_agent_memory_lifecycle_operation("modeling", operation, &store)?;
        assert_eq!(applied.status, "accepted");
        assert_eq!(applied.applied, Some(true));

        let mut cache = agent_memory_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let modeling = cache.get_required::<EpiphanyAgentMemoryEntry>("modeling")?;
        assert_eq!(modeling.agent.memories.semantic.len(), 1);
        assert_eq!(
            modeling.agent.memories.semantic[0].memory_id,
            "mem-lifecycle-merged"
        );
        assert!(
            modeling
                .agent
                .memories
                .episodic
                .iter()
                .any(|memory| memory.memory_id == "mem-lifecycle-alpha-episode")
        );
        assert!(validate_agent_memory_store(&store)?.is_empty());
        Ok(())
    }

    #[test]
    fn canonical_trait_seeds_replace_baseline_vectors() -> Result<()> {
        let temp = tempdir()?;
        let agent_dir = temp.path().join("agents");
        fs::create_dir_all(&agent_dir)?;
        for (role_id, agent_id, filename) in ROLE_TARGETS {
            fs::write(
                agent_dir.join(filename),
                sample_agent_json(agent_id, &format!("Agent {role_id}")),
            )?;
        }
        let store = temp.path().join("agents.msgpack");
        migrate_agent_memory_json_dir_to_cultcache(&agent_dir, &store)?;

        let applied = apply_agent_canonical_trait_seeds(
            &[AgentCanonicalTraitSeed {
                role_id: "coordinator".to_string(),
                group_name: "underlying_organization".to_string(),
                trait_name: "routing_discipline".to_string(),
                mean: 0.92,
                plasticity: 0.22,
                current_activation: 0.9,
                source: Some("startup personality projection smoke".to_string()),
            }],
            &store,
        )?;
        assert_eq!(applied["applied"], 1);

        let mut cache = agent_memory_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let entry = cache.get_required::<EpiphanyAgentMemoryEntry>("coordinator")?;
        assert!(
            !entry
                .agent
                .canonical_state
                .underlying_organization
                .contains_key("baseline")
        );
        assert_eq!(
            entry
                .agent
                .canonical_state
                .underlying_organization
                .get("routing_discipline")
                .map(|vector| vector.mean),
            Some(0.92)
        );
        Ok(())
    }

    #[test]
    fn organ_state_profiles_distinguish_persona_from_lane_organs() {
        let persona = organ_state_profile_for_role("Persona");
        assert_eq!(persona.profile_kind, EpiphanyOrganStateProfileKind::Persona);
        assert_eq!(persona.state_density, "persona_grade");
        assert_eq!(persona.portable_contract, "gamecult.persona_state.v0");
        assert_eq!(persona.affect_model, "persona_affect_allowed_and_expected");

        let hands = organ_state_profile_for_role("implementation");
        assert_eq!(hands.profile_kind, EpiphanyOrganStateProfileKind::WorkOrgan);
        assert_eq!(hands.state_density, "lean_work_organ");
        assert_eq!(hands.portable_contract, "epiphany.work_organ_state.v0");
        assert_eq!(hands.affect_model, "no_affect_or_persona_machinery");
    }

    #[test]
    fn agent_memory_projects_swarm_state_as_soa_columns() -> Result<()> {
        let temp = tempdir()?;
        let agent_dir = temp.path().join("agents");
        fs::create_dir_all(&agent_dir)?;
        for (role_id, agent_id, filename) in ROLE_TARGETS {
            fs::write(
                agent_dir.join(filename),
                sample_agent_json(agent_id, &format!("Agent {role_id}")),
            )?;
        }
        let store = temp.path().join("agents.msgpack");
        migrate_agent_memory_json_dir_to_cultcache(&agent_dir, &store)?;

        let refreshed = refresh_agent_state_soa(&store)?;
        assert_eq!(refreshed["rowCount"], 7);

        let mut cache = agent_memory_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let entry = cache.get_required::<EpiphanyAgentStateSoaEntry>(AGENT_STATE_SOA_KEY)?;
        assert_eq!(entry.schema_version, AGENT_STATE_SOA_SCHEMA_VERSION);
        assert_eq!(entry.role_ids.len(), 7);
        assert!(entry.role_ids.iter().any(|role| role == "Persona"));
        assert!(
            entry
                .agent_ids
                .iter()
                .any(|agent_id| agent_id == "epiphany.Persona")
        );
        assert_eq!(entry.role_ids.len(), entry.semantic_memory_counts.len());
        assert!(
            entry
                .portable_contracts
                .iter()
                .any(|contract| contract == "gamecult.persona_state.v0")
        );
        Ok(())
    }

    #[test]
    fn repair_agent_memory_store_promotes_legacy_face_and_modeling_vessel() -> Result<()> {
        let temp = tempdir()?;
        let agent_dir = temp.path().join("agents");
        fs::create_dir_all(&agent_dir)?;
        for (role_id, agent_id, filename) in ROLE_TARGETS {
            if *role_id == "Persona" {
                continue;
            }
            let actual_agent_id = if *role_id == "modeling" {
                "epiphany.proprioception"
            } else {
                agent_id
            };
            fs::write(
                agent_dir.join(filename),
                sample_agent_json(actual_agent_id, &format!("Agent {role_id}")),
            )?;
        }
        fs::write(
            agent_dir.join("face.agent-state.json"),
            sample_agent_json("epiphany.face", "Face"),
        )?;

        let store = temp.path().join("agents.msgpack");
        let mut cache = agent_memory_cache(&store)?;
        for (role_id, agent_id, filename) in ROLE_TARGETS {
            if *role_id == "Persona" {
                continue;
            }
            let raw = fs::read_to_string(agent_dir.join(filename))?;
            let projection: AgentMemoryProjection = serde_json::from_str(&raw)?;
            let expected_agent_id = if *role_id == "modeling" {
                "epiphany.proprioception"
            } else {
                agent_id
            };
            let mut entry = entry_from_projection(role_id, expected_agent_id, projection)?;
            if *role_id == "modeling" {
                entry.agent.identity.public_description =
                    "Proprioception models source-grounded anatomy.".to_string();
                entry.agent.goals[0].emotional_stake =
                    "If Proprioception lies, Hands cuts blind.".to_string();
                entry.agent.memories.semantic.push(GhostlightMemory {
                    memory_id: "legacy-proprioception-prose".to_string(),
                    summary: "Proprioception leaves a verified trail.".to_string(),
                    salience: 0.8,
                    confidence: 0.8,
                    linked_event_ids: None,
                    linked_relationship_id: None,
                });
            }
            cache.put((*role_id).to_string(), &entry)?;
        }
        let raw = fs::read_to_string(agent_dir.join("face.agent-state.json"))?;
        let projection: AgentMemoryProjection = serde_json::from_str(&raw)?;
        let face = entry_from_projection("face", "epiphany.face", projection)?;
        cache.put("face".to_string(), &face)?;
        let projection: AgentMemoryProjection =
            serde_json::from_str(&sample_agent_json("epiphany.life", "Life"))?;
        let reorientation = entry_from_projection("reorientation", "epiphany.life", projection)?;
        cache.put("reorientation".to_string(), &reorientation)?;

        let errors = validate_agent_memory_store(&store)?;
        assert!(errors.iter().any(|error| error.contains("proprioception")));
        assert!(
            errors
                .iter()
                .any(|error| error.contains("Persona: missing"))
        );

        let repaired = repair_agent_memory_store(&store)?;
        assert_eq!(repaired["ok"], true);
        assert!(validate_agent_memory_store(&store)?.is_empty());
        let refreshed = refresh_agent_state_soa(&store)?;
        assert_eq!(refreshed["rowCount"], 7);

        let mut cache = agent_memory_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let modeling = cache.get_required::<EpiphanyAgentMemoryEntry>("modeling")?;
        assert_eq!(modeling.agent.agent_id, "epiphany.modeling");
        assert!(
            !modeling
                .agent
                .identity
                .public_description
                .contains("Proprioception")
        );
        assert!(
            modeling
                .agent
                .goals
                .iter()
                .all(|goal| !goal.emotional_stake.contains("Proprioception"))
        );
        assert!(
            modeling
                .agent
                .memories
                .semantic
                .iter()
                .all(|memory| { !memory.summary.contains("Proprioception") })
        );
        let persona = cache.get_required::<EpiphanyAgentMemoryEntry>("Persona")?;
        assert_eq!(persona.role_id, "Persona");
        assert_eq!(persona.agent.agent_id, "epiphany.Persona");
        assert!(
            persona
                .agent
                .identity
                .roles
                .iter()
                .any(|role| role == "Persona")
        );
        assert!(
            cache
                .get::<EpiphanyAgentMemoryEntry>("reorientation")?
                .is_none()
        );
        Ok(())
    }

    fn sample_agent_json(agent_id: &str, name: &str) -> String {
        serde_json::json!({
            "schema_version": AGENT_MEMORY_SCHEMA_VERSION,
            "world": {
                "world_id": "epiphany-agent-memory",
                "setting": "Epiphany local harness role memory",
                "time": {"label": "standing memory"},
                "canon_context": ["Organ-state records preserve lane identity."]
            },
            "agents": [{
                "agent_id": agent_id,
                "identity": {
                    "name": name,
                    "roles": ["test"],
                    "origin": "test lane",
                    "public_description": "Test role memory.",
                    "private_notes": []
                },
                "canonical_state": {
                    "underlying_organization": {"test": {"mean": 0.5, "plasticity": 0.5, "current_activation": 0.5}},
                    "stable_dispositions": {"test": {"mean": 0.5, "plasticity": 0.5, "current_activation": 0.5}},
                    "behavioral_dimensions": {"test": {"mean": 0.5, "plasticity": 0.5, "current_activation": 0.5}},
                    "presentation_strategy": {"test": {"mean": 0.5, "plasticity": 0.5, "current_activation": 0.5}},
                    "voice_style": {"test": {"mean": 0.5, "plasticity": 0.5, "current_activation": 0.5}},
                    "situational_state": {"test": {"mean": 0.5, "plasticity": 0.5, "current_activation": 0.5}},
                    "values": [{
                        "value_id": "value-test",
                        "label": "Test value.",
                        "priority": 0.5,
                        "unforgivable_if_betrayed": false
                    }]
                },
                "goals": [{
                    "goal_id": "goal-test",
                    "description": "Keep the test role valid.",
                    "scope": "life",
                    "priority": 0.5,
                    "emotional_stake": "Broken memory makes broken agents.",
                    "blockers": [],
                    "status": "active"
                }],
                "memories": {
                    "episodic": [],
                    "semantic": [{
                        "memory_id": "mem-test",
                        "summary": "Test memory.",
                        "salience": 0.5,
                        "confidence": 0.5
                    }],
                    "relationship_summaries": []
                },
                "perceived_state_overlays": []
            }],
            "relationships": [],
            "events": [],
            "scenes": []
        })
        .to_string()
    }
}
#[test]
fn immutable_swarm_identity_separates_stores_and_refuses_collision() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let first_store = temp.path().join("first.msgpack");
    let second_store = temp.path().join("second.msgpack");
    let first = ensure_agent_memory_swarm_identity(&first_store, "swarm-alpha")?;
    let second = ensure_agent_memory_swarm_identity(&second_store, "swarm-beta")?;
    assert_eq!(
        load_agent_memory_swarm_identity(&first_store)?,
        Some(first.clone())
    );
    assert_eq!(
        ensure_agent_memory_swarm_identity(&first_store, "swarm-alpha")?,
        first
    );
    let collision = ensure_agent_memory_swarm_identity(&first_store, "swarm-beta")
        .expect_err("immutable store identity must refuse substitution");
    assert!(collision.to_string().contains("collision"));
    assert_ne!(
        crate::semantic_point_id(
            &first.swarm_id,
            crate::SemanticPartition::Mind,
            AGENT_MEMORY_TYPE,
            "Persona",
            "memory-1"
        ),
        crate::semantic_point_id(
            &second.swarm_id,
            crate::SemanticPartition::Mind,
            AGENT_MEMORY_TYPE,
            "Persona",
            "memory-1"
        )
    );
    Ok(())
}
