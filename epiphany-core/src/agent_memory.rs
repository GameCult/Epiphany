use anyhow::Result;
use anyhow::anyhow;
use chrono::SecondsFormat;
use chrono::Utc;
use cultcache_rs::CultCache;
use cultcache_rs::DatabaseEntry;
use cultcache_rs::SingleFileMessagePackBackingStore;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

pub const AGENT_MEMORY_TYPE: &str = "epiphany.agent_memory";
pub const AGENT_MEMORY_SCHEMA_VERSION: &str = "ghostlight.agent_state.v0";
pub const AGENT_MEMORY_SWARM_IDENTITY_TYPE: &str = "epiphany.agent_memory_swarm_identity";
pub const AGENT_MEMORY_SWARM_IDENTITY_SCHEMA_VERSION: &str =
    "epiphany.agent_memory_swarm_identity.v0";
pub const AGENT_MEMORY_SWARM_IDENTITY_KEY: &str = "swarm-identity";
pub const AGENT_MEMORY_GENERATION_WITNESS_TYPE: &str = "epiphany.agent_memory_generation_witness";
pub const AGENT_MEMORY_GENERATION_WITNESS_SCHEMA_VERSION: &str =
    "epiphany.agent_memory_generation_witness.v0";
pub const AGENT_MEMORY_GENERATION_WITNESS_LATEST_KEY: &str = "mind-generation/latest";
pub const AGENT_MEMORY_MIND_ADMISSION_TYPE: &str = "epiphany.agent_memory_mind_admission";
pub const AGENT_MEMORY_MIND_ADMISSION_SCHEMA_VERSION: &str =
    "epiphany.agent_memory_mind_admission.v0";

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.agent_memory_generation_witness",
    schema = "AgentMemoryGenerationWitness"
)]
pub struct AgentMemoryGenerationWitness {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub witness_id: String,
    #[cultcache(key = 2)]
    pub swarm_id: String,
    #[cultcache(key = 3)]
    pub generation: u64,
    #[cultcache(key = 4)]
    pub previous_generation: u64,
    #[cultcache(key = 5)]
    pub previous_source_hash: String,
    #[cultcache(key = 6)]
    pub source_hash: String,
    #[cultcache(key = 7)]
    pub authority_receipt_id: String,
    #[cultcache(key = 8)]
    pub mutation_kind: String,
    #[cultcache(key = 9)]
    pub changed_role_ids: Vec<String>,
    #[cultcache(key = 10)]
    pub committed_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.agent_memory_mind_admission",
    schema = "AgentMemoryMindAdmissionReceipt"
)]
pub struct AgentMemoryMindAdmissionReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub swarm_id: String,
    #[cultcache(key = 3)]
    pub role_id: String,
    #[cultcache(key = 4)]
    pub mutation_kind: String,
    #[cultcache(key = 5)]
    pub reason: String,
    #[cultcache(key = 6)]
    pub status: String,
    #[cultcache(key = 7)]
    pub resulting_source_hash: String,
}
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

pub fn initialize_fresh_agent_memory_store(
    store_path: impl AsRef<Path>,
    swarm_id: &str,
) -> Result<AgentMemoryGenerationWitness> {
    let store_path = store_path.as_ref();
    let swarm_id = swarm_id.trim();
    if swarm_id.is_empty() {
        return Err(anyhow!("fresh agent memory requires a non-empty swarm id"));
    }
    let identity = AgentMemorySwarmIdentity {
        schema_version: AGENT_MEMORY_SWARM_IDENTITY_SCHEMA_VERSION.to_string(),
        swarm_id: swarm_id.to_string(),
    };
    let entries = ROLE_TARGETS
        .iter()
        .map(|(role_id, agent_id, _)| fresh_agent_memory_entry(role_id, agent_id))
        .collect::<Vec<_>>();
    let mut cache = agent_memory_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let opening = cache.snapshot_envelopes();
    if opening.is_empty() {
        let mut batch = vec![
            cache
                .prepare_entry(AGENT_MEMORY_SWARM_IDENTITY_KEY, &identity)?
                .0,
        ];
        for entry in &entries {
            batch.push(cache.prepare_entry(&entry.role_id, entry)?.0);
        }
        if !SingleFileMessagePackBackingStore::new(store_path).compare_and_swap_batch(&[], batch)? {
            return Err(anyhow!(
                "fresh agent memory initialization lost exact atomic compare-and-swap"
            ));
        }
    } else {
        if cache.get::<AgentMemorySwarmIdentity>(AGENT_MEMORY_SWARM_IDENTITY_KEY)? != Some(identity)
        {
            return Err(anyhow!(
                "fresh agent memory initialization collided with another swarm identity"
            ));
        }
        for entry in &entries {
            if cache.get::<EpiphanyAgentMemoryEntry>(&entry.role_id)? != Some(entry.clone()) {
                return Err(anyhow!(
                    "fresh agent memory initialization collided with role {:?}",
                    entry.role_id
                ));
            }
        }
    }
    admit_initial_agent_memory_generation(
        store_path,
        "fresh-bootstrap-admission",
        "Admit freshly initialized canonical organ memory rows.",
        ROLE_TARGETS
            .iter()
            .map(|(role_id, _, _)| (*role_id).to_string())
            .collect(),
    )
}

fn fresh_agent_memory_entry(role_id: &str, agent_id: &str) -> EpiphanyAgentMemoryEntry {
    let display_name = match role_id {
        "Persona" => "Persona".to_string(),
        "coordinator" => "Self".to_string(),
        "verification" => "Soul".to_string(),
        "implementation" => "Hands".to_string(),
        "research" => "Eyes".to_string(),
        other => {
            let mut chars = other.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        }
    };
    let mut canonical_state = GhostlightCanonicalState::default();
    if role_id == "Persona" {
        let baseline = BTreeMap::from([(
            "baseline".to_string(),
            GhostlightTraitVector {
                mean: 0.5,
                plasticity: 0.5,
                current_activation: 0.5,
            },
        )]);
        canonical_state.underlying_organization = baseline.clone();
        canonical_state.stable_dispositions = baseline.clone();
        canonical_state.behavioral_dimensions = baseline.clone();
        canonical_state.presentation_strategy = baseline.clone();
        canonical_state.voice_style = baseline.clone();
        canonical_state.situational_state = baseline;
    }
    EpiphanyAgentMemoryEntry {
        schema_version: AGENT_MEMORY_SCHEMA_VERSION.to_string(),
        role_id: role_id.to_string(),
        world: GhostlightWorld {
            world_id: "epiphany-agent-memory".to_string(),
            setting: "Fresh local Epiphany organ memory".to_string(),
            time: GhostlightTime {
                label: "initial generation".to_string(),
            },
            canon_context: vec![
                "Typed organ memory begins empty and grows only through admitted decisions."
                    .to_string(),
            ],
        },
        agent: GhostlightAgent {
            agent_id: agent_id.to_string(),
            identity: GhostlightIdentity {
                name: display_name,
                roles: vec![role_id.to_string()],
                origin: "Fresh local Epiphany organ".to_string(),
                public_description: format!("Epiphany {role_id} organ memory."),
                private_notes: Vec::new(),
            },
            canonical_state,
            goals: Vec::new(),
            memories: GhostlightMemories::default(),
            perceived_state_overlays: Vec::new(),
        },
        relationships: Vec::new(),
        events: Vec::new(),
        scenes: Vec::new(),
    }
}

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

pub const PERSONA_STATE_SCHEMA_VERSION: &str = "gamecult.persona_state.v0";

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

fn canonical_agent_memory_source_hash(
    cache: &mut CultCache,
    identity: &AgentMemorySwarmIdentity,
    replacements: &BTreeMap<String, EpiphanyAgentMemoryEntry>,
) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(AGENT_MEMORY_SWARM_IDENTITY_SCHEMA_VERSION.as_bytes());
    hasher.update([0]);
    hasher.update(identity.swarm_id.as_bytes());
    for (role_id, _, _) in ROLE_TARGETS {
        let entry = replacements
            .get(*role_id)
            .cloned()
            .or_else(|| {
                cache
                    .get::<EpiphanyAgentMemoryEntry>(role_id)
                    .ok()
                    .flatten()
            })
            .ok_or_else(|| anyhow!("canonical Mind generation is missing role {role_id:?}"))?;
        let envelope = cache.prepare_entry(*role_id, &entry)?.0;
        hasher.update((*role_id).as_bytes());
        hasher.update((envelope.payload.len() as u64).to_le_bytes());
        hasher.update(&envelope.payload);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn commit_agent_memory_generation(
    store_path: &Path,
    expected_generation: u64,
    expected_source_hash: &str,
    role_id: &str,
    next_entry: EpiphanyAgentMemoryEntry,
    mutation_kind: &str,
    reason: &str,
) -> Result<AgentMemoryGenerationWitness> {
    let expected_agent_id = agent_id_for_role(role_id).map_err(|message| anyhow!(message))?;
    let validation = validate_agent_entry(&next_entry, expected_agent_id);
    if !validation.is_empty() {
        return Err(anyhow!("Mind commit candidate is invalid: {validation:?}"));
    }
    let mut cache = agent_memory_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let identity = cache
        .get::<AgentMemorySwarmIdentity>(AGENT_MEMORY_SWARM_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("Mind commit requires immutable agent memory swarm identity"))?;
    let current =
        cache.get::<AgentMemoryGenerationWitness>(AGENT_MEMORY_GENERATION_WITNESS_LATEST_KEY)?;
    let current_generation = current.as_ref().map_or(0, |witness| witness.generation);
    let empty = BTreeMap::new();
    let current_source_hash = canonical_agent_memory_source_hash(&mut cache, &identity, &empty)?;
    if let Some(witness) = &current
        && (witness.schema_version != AGENT_MEMORY_GENERATION_WITNESS_SCHEMA_VERSION
            || witness.swarm_id != identity.swarm_id
            || witness.source_hash != current_source_hash
            || witness.generation == 0)
    {
        return Err(anyhow!(
            "stored Mind generation witness does not authenticate current canonical memory"
        ));
    }
    if current_generation != expected_generation || current_source_hash != expected_source_hash {
        return Err(anyhow!(
            "agent memory generation changed before Mind commit"
        ));
    }
    let mut replacements = BTreeMap::new();
    replacements.insert(role_id.to_string(), next_entry);
    let source_hash = canonical_agent_memory_source_hash(&mut cache, &identity, &replacements)?;
    if source_hash == current_source_hash {
        return Err(anyhow!(
            "Mind commit would mint a generation without changing canonical memory"
        ));
    }
    let generation = current_generation + 1;
    let fingerprint = format!(
        "{}|{}|{}|{}|{}",
        identity.swarm_id, generation, mutation_kind, role_id, source_hash
    );
    let witness_id = format!(
        "mind-generation-{}",
        uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, fingerprint.as_bytes())
    );
    let receipt_id = format!(
        "mind-admission-{}",
        uuid::Uuid::new_v5(
            &uuid::Uuid::NAMESPACE_OID,
            format!("{fingerprint}|{reason}").as_bytes()
        )
    );
    let receipt = AgentMemoryMindAdmissionReceipt {
        schema_version: AGENT_MEMORY_MIND_ADMISSION_SCHEMA_VERSION.to_string(),
        receipt_id: receipt_id.clone(),
        swarm_id: identity.swarm_id.clone(),
        role_id: role_id.to_string(),
        mutation_kind: mutation_kind.to_string(),
        reason: reason.to_string(),
        status: "admitted".to_string(),
        resulting_source_hash: source_hash.clone(),
    };
    let committed_at = now_rfc3339();
    let witness = AgentMemoryGenerationWitness {
        schema_version: AGENT_MEMORY_GENERATION_WITNESS_SCHEMA_VERSION.to_string(),
        witness_id: witness_id.clone(),
        swarm_id: identity.swarm_id.clone(),
        generation,
        previous_generation: current_generation,
        previous_source_hash: current_source_hash,
        source_hash: source_hash.clone(),
        authority_receipt_id: receipt_id.clone(),
        mutation_kind: mutation_kind.to_string(),
        changed_role_ids: vec![role_id.to_string()],
        committed_at: committed_at.clone(),
    };
    let opening = cache.snapshot_envelopes();
    let mut expected = Vec::new();
    let mut batch = Vec::new();
    for (canonical_role, _, _) in ROLE_TARGETS {
        if let Some(existing) = opening.iter().find(|envelope| {
            envelope.r#type == AGENT_MEMORY_TYPE && envelope.key == *canonical_role
        }) {
            expected.push(existing.clone());
        }
        let entry = replacements
            .get(*canonical_role)
            .cloned()
            .or_else(|| {
                cache
                    .get::<EpiphanyAgentMemoryEntry>(canonical_role)
                    .ok()
                    .flatten()
            })
            .ok_or_else(|| anyhow!("canonical Mind generation lost role {canonical_role:?}"))?;
        batch.push(cache.prepare_entry(*canonical_role, &entry)?.0);
    }
    if let Some(existing) = opening.iter().find(|envelope| {
        envelope.r#type == AGENT_MEMORY_GENERATION_WITNESS_TYPE
            && envelope.key == AGENT_MEMORY_GENERATION_WITNESS_LATEST_KEY
    }) {
        expected.push(existing.clone());
    }
    batch.push(
        cache
            .prepare_entry(AGENT_MEMORY_GENERATION_WITNESS_LATEST_KEY, &witness)?
            .0,
    );
    batch.push(cache.prepare_entry(&witness_id, &witness)?.0);
    batch.push(cache.prepare_entry(&receipt_id, &receipt)?.0);
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    if !backing.compare_and_swap_batch(&expected, batch)? {
        return Err(anyhow!(
            "agent memory Mind commit lost exact atomic compare-and-swap"
        ));
    }
    Ok(witness)
}

fn admit_initial_agent_memory_generation(
    store_path: &Path,
    mutation_kind: &str,
    reason: &str,
    changed_role_ids: Vec<String>,
) -> Result<AgentMemoryGenerationWitness> {
    let mut cache = agent_memory_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let identity = cache
        .get::<AgentMemorySwarmIdentity>(AGENT_MEMORY_SWARM_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("initial Mind admission requires immutable swarm identity"))?;
    let source_hash = canonical_agent_memory_source_hash(&mut cache, &identity, &BTreeMap::new())?;
    if let Some(existing) =
        cache.get::<AgentMemoryGenerationWitness>(AGENT_MEMORY_GENERATION_WITNESS_LATEST_KEY)?
    {
        if existing.schema_version != AGENT_MEMORY_GENERATION_WITNESS_SCHEMA_VERSION
            || existing.swarm_id != identity.swarm_id
            || existing.generation != 1
            || existing.previous_generation != 0
            || existing.source_hash != source_hash
            || existing.mutation_kind != mutation_kind
        {
            return Err(anyhow!(
                "initial Mind admission collides with live generation"
            ));
        }
        return Ok(existing);
    }

    for (role_id, expected_agent_id, _) in ROLE_TARGETS {
        let entry = cache
            .get::<EpiphanyAgentMemoryEntry>(role_id)?
            .ok_or_else(|| anyhow!("initial Mind admission requires role {role_id:?}"))?;
        let errors = validate_agent_entry(&entry, expected_agent_id);
        if !errors.is_empty() {
            return Err(anyhow!(
                "initial Mind role {role_id:?} is invalid: {}",
                errors.join("; ")
            ));
        }
    }
    let committed_at = now_rfc3339();
    let fingerprint = format!("{}|{}|{}", identity.swarm_id, mutation_kind, source_hash);
    let witness_id = format!(
        "mind-generation-{}",
        uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, fingerprint.as_bytes())
    );
    let receipt_id = format!(
        "mind-admission-{}",
        uuid::Uuid::new_v5(
            &uuid::Uuid::NAMESPACE_OID,
            format!("{fingerprint}|{reason}").as_bytes()
        )
    );
    let receipt = AgentMemoryMindAdmissionReceipt {
        schema_version: AGENT_MEMORY_MIND_ADMISSION_SCHEMA_VERSION.to_string(),
        receipt_id: receipt_id.clone(),
        swarm_id: identity.swarm_id.clone(),
        role_id: "mind".to_string(),
        mutation_kind: mutation_kind.to_string(),
        reason: reason.to_string(),
        status: "admitted".to_string(),
        resulting_source_hash: source_hash.clone(),
    };
    let witness = AgentMemoryGenerationWitness {
        schema_version: AGENT_MEMORY_GENERATION_WITNESS_SCHEMA_VERSION.to_string(),
        witness_id: witness_id.clone(),
        swarm_id: identity.swarm_id.clone(),
        generation: 1,
        previous_generation: 0,
        previous_source_hash: source_hash.clone(),
        source_hash: source_hash.clone(),
        authority_receipt_id: receipt_id.clone(),
        mutation_kind: mutation_kind.to_string(),
        changed_role_ids,
        committed_at: committed_at.clone(),
    };
    let opening = cache.snapshot_envelopes();
    let mut expected = Vec::new();
    let mut replacements = Vec::new();
    for envelope in opening.iter().filter(|envelope| {
        (envelope.r#type == AGENT_MEMORY_SWARM_IDENTITY_TYPE
            && envelope.key == AGENT_MEMORY_SWARM_IDENTITY_KEY)
            || (envelope.r#type == AGENT_MEMORY_TYPE
                && ROLE_TARGETS
                    .iter()
                    .any(|(role_id, _, _)| envelope.key == *role_id))
    }) {
        expected.push(envelope.clone());
        replacements.push(envelope.clone());
    }
    if expected.len() != ROLE_TARGETS.len() + 1 {
        return Err(anyhow!(
            "initial Mind admission could not authenticate every canonical source envelope"
        ));
    }
    replacements.push(
        cache
            .prepare_entry(AGENT_MEMORY_GENERATION_WITNESS_LATEST_KEY, &witness)?
            .0,
    );
    replacements.push(cache.prepare_entry(&witness_id, &witness)?.0);
    replacements.push(cache.prepare_entry(&receipt_id, &receipt)?.0);
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    if !backing.compare_and_swap_batch(&expected, replacements)? {
        return Err(anyhow!(
            "initial Mind admission lost exact atomic compare-and-swap"
        ));
    }
    Ok(witness)
}

fn agent_memory_source_head(cache: &mut CultCache) -> Result<(u64, String)> {
    let identity = cache
        .get::<AgentMemorySwarmIdentity>(AGENT_MEMORY_SWARM_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("Mind commit requires immutable agent memory swarm identity"))?;
    let canonical_hash = canonical_agent_memory_source_hash(cache, &identity, &BTreeMap::new())?;
    if let Some(witness) =
        cache.get::<AgentMemoryGenerationWitness>(AGENT_MEMORY_GENERATION_WITNESS_LATEST_KEY)?
    {
        if witness.schema_version != AGENT_MEMORY_GENERATION_WITNESS_SCHEMA_VERSION
            || witness.swarm_id != identity.swarm_id
            || witness.source_hash != canonical_hash
            || witness.generation == 0
        {
            return Err(anyhow!(
                "stored Mind generation witness does not authenticate current canonical memory"
            ));
        }
        return Ok((witness.generation, canonical_hash));
    }
    Ok((0, canonical_hash))
}

fn require_agent_memory_migration_open(cache: &mut CultCache, operation: &str) -> Result<()> {
    cache.pull_all_backing_stores()?;
    if cache
        .get::<AgentMemoryGenerationWitness>(AGENT_MEMORY_GENERATION_WITNESS_LATEST_KEY)?
        .is_some()
    {
        return Err(anyhow!(
            "{operation} is bootstrap/migration-only and cannot mutate canonical Mind after generation admission"
        ));
    }
    Ok(())
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

pub fn write_agent_memory_entry_for_role_migration(
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
    require_agent_memory_migration_open(&mut cache, "raw agent-memory replacement")?;
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

fn agent_memory_review(
    role_id: &str,
    target_agent_id: &str,
    store_path: &Path,
    reasons: Vec<String>,
    applied: Option<bool>,
) -> AgentMemoryReview {
    AgentMemoryReview {
        status: if reasons.is_empty() {
            "accepted".to_string()
        } else {
            "rejected".to_string()
        },
        target_agent_id: target_agent_id.to_string(),
        target_role_id: role_id.to_string(),
        target_store: store_path.display().to_string(),
        reasons,
        applied,
    }
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
    let (expected_generation, expected_source_hash) = agent_memory_source_head(&mut cache)?;
    let reason = patch
        .reason
        .clone()
        .unwrap_or_else(|| "admitted bounded self patch".to_string());

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
    commit_agent_memory_generation(
        store_path,
        expected_generation,
        &expected_source_hash,
        role_id,
        entry,
        "self_patch",
        &reason,
    )?;
    review.applied = Some(true);
    Ok(review)
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

fn agent_memory_cache(store_path: &Path) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<AgentMemorySwarmIdentity>()?;
    cache.register_entry_type::<AgentMemoryGenerationWitness>()?;
    cache.register_entry_type::<AgentMemoryMindAdmissionReceipt>()?;
    cache.register_entry_type::<EpiphanyAgentMemoryEntry>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(store_path));
    Ok(cache)
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
    fn fresh_store_initialization_is_exact_and_idempotent() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("fresh-agent-memory.cc");
        let witness = initialize_fresh_agent_memory_store(&store, "fresh-swarm")?;

        assert_eq!(witness.generation, 1);
        assert_eq!(witness.changed_role_ids.len(), ROLE_TARGETS.len());
        assert_eq!(
            initialize_fresh_agent_memory_store(&store, "fresh-swarm")?,
            witness
        );
        Ok(())
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
    assert_ne!(first.swarm_id, second.swarm_id);
    Ok(())
}
