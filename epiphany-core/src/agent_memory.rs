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

pub const AGENT_MEMORY_TYPE: &str = "epiphany.agent_memory";
pub const AGENT_MEMORY_SCHEMA_VERSION: &str = "ghostlight.agent_state.v0";

const ROLE_TARGETS: &[(&str, &str, &str)] = &[
    (
        "imagination",
        "epiphany.imagination",
        "imagination.agent-state.json",
    ),
    ("modeling", "epiphany.body", "body.agent-state.json"),
    ("verification", "epiphany.soul", "soul.agent-state.json"),
    ("implementation", "epiphany.hands", "hands.agent-state.json"),
    ("reorientation", "epiphany.life", "life.agent-state.json"),
    ("research", "epiphany.eyes", "eyes.agent-state.json"),
    ("face", "epiphany.face", "face.agent-state.json"),
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
    pub relationships: Vec<Value>,
    #[cultcache(key = 5, default)]
    pub events: Vec<Value>,
    #[cultcache(key = 6, default)]
    pub scenes: Vec<Value>,
    #[cultcache(key = 7, default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightWorld {
    pub world_id: String,
    pub setting: String,
    pub time: GhostlightTime,
    pub canon_context: Vec<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightTime {
    pub label: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightAgent {
    pub agent_id: String,
    pub identity: GhostlightIdentity,
    pub canonical_state: GhostlightCanonicalState,
    pub goals: Vec<GhostlightGoal>,
    pub memories: GhostlightMemories,
    pub perceived_state_overlays: Vec<Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightIdentity {
    pub name: String,
    pub roles: Vec<String>,
    pub origin: String,
    pub public_description: String,
    #[serde(default)]
    pub private_notes: Vec<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
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
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightTraitVector {
    pub mean: f64,
    pub plasticity: f64,
    pub current_activation: f64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightValue {
    pub value_id: String,
    pub label: String,
    pub priority: f64,
    pub unforgivable_if_betrayed: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
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
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GhostlightMemories {
    pub episodic: Vec<GhostlightMemory>,
    pub semantic: Vec<GhostlightMemory>,
    pub relationship_summaries: Vec<GhostlightMemory>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
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
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AgentMemoryProjection {
    pub schema_version: String,
    pub world: GhostlightWorld,
    pub agents: Vec<GhostlightAgent>,
    #[serde(default)]
    pub relationships: Vec<Value>,
    #[serde(default)]
    pub events: Vec<Value>,
    #[serde(default)]
    pub scenes: Vec<Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
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
pub enum EpiphanyDossierProfileKind {
    LaneCore,
    EmbodiedActor,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyDossierProfile {
    pub profile_kind: EpiphanyDossierProfileKind,
    pub canonical_density: String,
    pub relationship_model: String,
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
        Ok(patch) => reasons.extend(review_agent_self_patch_contract(&target_agent_id, &patch)),
        Err(reason) => reasons.push(reason),
    }

    AgentMemoryReview {
        status: if reasons.is_empty() {
            "accepted"
        } else {
            "rejected"
        }
        .to_string(),
        target_agent_id,
        target_role_id: role_id.to_string(),
        target_store: store_path.display().to_string(),
        reasons,
        applied: None,
    }
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
    let store_path = store_path.as_ref();
    let mut review = review_agent_self_patch(role_id, patch_value, store_path);
    if review.status != "accepted" {
        return Ok(review);
    }
    let patch: AgentSelfPatch = serde_json::from_value(patch_value.clone())
        .context("failed to decode accepted selfPatch")?;
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
            extra: BTreeMap::new(),
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
                "dossierProfile": dossier_profile_for_role(role_id),
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
    Ok(serde_json::json!({
        "ok": errors.is_empty(),
        "store": store_path,
        "present": true,
        "entryType": AGENT_MEMORY_TYPE,
        "errors": errors,
        "roles": roles,
    }))
}

pub fn dossier_profile_for_role(role_id: &str) -> EpiphanyDossierProfile {
    match role_id {
        "face" => EpiphanyDossierProfile {
            profile_kind: EpiphanyDossierProfileKind::EmbodiedActor,
            canonical_density: "dense_ghostlight_core_preferred".to_string(),
            relationship_model: "relationship_summaries_and_directional_stance_matter".to_string(),
            perceived_overlay_mode: "observer_local_and_fallible".to_string(),
            growth_channels: vec![
                "heartbeat appraisal and reaction".to_string(),
                "character-loop interpretation".to_string(),
                "episodic and relationship memory accumulation".to_string(),
                "reviewed selfPatch".to_string(),
                "sleep/distillation".to_string(),
            ],
            notes: vec![
                "Face should behave like an embodied, responsive, and fallible public creature rather than a thin tool wrapper.".to_string(),
                "Dense Ghostlight-style canonical families, perceived overlays, and relationship pressure are appropriate here.".to_string(),
            ],
        },
        _ => EpiphanyDossierProfile {
            profile_kind: EpiphanyDossierProfileKind::LaneCore,
            canonical_density: "lean_role_lattice".to_string(),
            relationship_model: "role_local_summary_only".to_string(),
            perceived_overlay_mode: "minimal_until_a_real_need_exists".to_string(),
            growth_channels: vec![
                "reviewed selfPatch".to_string(),
                "heartbeat rumination pressure".to_string(),
                "sleep/distillation".to_string(),
                "birth-time repo personality and memory seeding".to_string(),
            ],
            notes: vec![
                "Most standing Epiphany organs need sharp role identity, room to grow, and resistance to personality sludge more than they need full dramatic embodiment.".to_string(),
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
            extra: entry.extra,
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
    cache.register_entry_type::<EpiphanyAgentMemoryEntry>()?;
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
        extra: projection.extra,
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
    for (index, goal) in entry.agent.goals.iter().enumerate() {
        validate_goal(goal, &format!("goals[{index}]"), &mut errors);
    }
    for (index, value) in entry.agent.canonical_state.values.iter().enumerate() {
        validate_value(value, &format!("values[{index}]"), &mut errors);
    }
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
        if group.is_empty() {
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
                extra: BTreeMap::new(),
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
                extra: BTreeMap::new(),
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
                extra: BTreeMap::new(),
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
            agent_dir.join("body.agent-state.json"),
            sample_agent_json("epiphany.body", "Body"),
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

        let patch = serde_json::json!({
            "agentId": "epiphany.body",
            "reason": "The Body should remember accepted graph growth must stay source-grounded.",
            "semanticMemories": [{
                "memoryId": "mem-body-native-source-grounding",
                "summary": "Native role memory patches update typed CultCache state rather than JSON dossier files.",
                "salience": 0.74,
                "confidence": 0.86
            }]
        });
        let review = review_agent_self_patch("modeling", &patch, &store);
        assert_eq!(review.status, "accepted");
        let applied = apply_agent_self_patch("modeling", &patch, &store)?;
        assert_eq!(applied.status, "accepted");
        assert_eq!(applied.applied, Some(true));

        let mut cache = agent_memory_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let body = cache.get_required::<EpiphanyAgentMemoryEntry>("modeling")?;
        assert!(
            body.agent
                .memories
                .semantic
                .iter()
                .any(|memory| memory.memory_id == "mem-body-native-source-grounding")
        );
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
    fn dossier_profiles_distinguish_face_from_lane_organs() {
        let face = dossier_profile_for_role("face");
        assert_eq!(face.profile_kind, EpiphanyDossierProfileKind::EmbodiedActor);
        assert_eq!(face.canonical_density, "dense_ghostlight_core_preferred");

        let hands = dossier_profile_for_role("implementation");
        assert_eq!(hands.profile_kind, EpiphanyDossierProfileKind::LaneCore);
        assert_eq!(hands.canonical_density, "lean_role_lattice");
    }

    fn sample_agent_json(agent_id: &str, name: &str) -> String {
        serde_json::json!({
            "schema_version": AGENT_MEMORY_SCHEMA_VERSION,
            "world": {
                "world_id": "epiphany-agent-memory",
                "setting": "Epiphany local harness role memory",
                "time": {"label": "standing memory"},
                "canon_context": ["Role dossiers preserve lane identity."]
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
