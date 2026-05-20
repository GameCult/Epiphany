use crate::agent_memory::EpiphanyAgentMemoryEntry;
use crate::agent_memory::EpiphanyDossierProfile;
use crate::agent_memory::GhostlightCanonicalState;
use crate::agent_memory::GhostlightTraitVector;
use crate::agent_memory::dossier_profile_for_role;
use crate::heartbeat_state::HeartbeatParticipant;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;

pub const AGENT_UTTERANCE_STATE_SCHEMA_VERSION: &str = "epiphany.agent_utterance_state.v0";
pub const WEKSA_UTTERANCE_HANDOFF_SCHEMA_VERSION: &str = "weksa.utterance_embedding_handoff.v0.1";
pub const AGENT_CHARACTER_STATE_VECTOR_SIZE: usize = 64;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUtteranceState {
    pub schema_version: String,
    pub source: String,
    pub role_id: String,
    pub agent_id: String,
    pub dossier_profile: EpiphanyDossierProfile,
    pub identity: AgentUtteranceIdentity,
    pub personality_vectors: AgentUtterancePersonalityVectors,
    pub values: Vec<AgentUtteranceValue>,
    pub current_mood: AgentUtteranceMood,
    pub activation: AgentUtteranceActivation,
    pub character_state_vector: AgentUtteranceCharacterStateVector,
    pub utterance_embedding_basis: Vec<AgentUtteranceEmbeddingBasis>,
    pub contract: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUtteranceIdentity {
    pub name: String,
    pub roles: Vec<String>,
    pub origin: String,
    pub public_description: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUtterancePersonalityVectors {
    pub underlying_organization: BTreeMap<String, GhostlightTraitVector>,
    pub stable_dispositions: BTreeMap<String, GhostlightTraitVector>,
    pub behavioral_dimensions: BTreeMap<String, GhostlightTraitVector>,
    pub presentation_strategy: BTreeMap<String, GhostlightTraitVector>,
    pub voice_style: BTreeMap<String, GhostlightTraitVector>,
    pub situational_state: BTreeMap<String, GhostlightTraitVector>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUtteranceValue {
    pub value_id: String,
    pub label: String,
    pub priority: f64,
    pub unforgivable_if_betrayed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUtteranceMood {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub emotional_dimensions: Vec<AgentUtteranceMoodDimension>,
    pub anxiety: f64,
    pub urgency: f64,
    pub arousal: f64,
    pub thought_pressure: f64,
    pub guardedness: f64,
    pub reaction_intensity: f64,
    pub cooldown_multiplier: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUtteranceMoodDimension {
    pub name: String,
    pub value: f64,
    pub source_path: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUtteranceActivation {
    pub status: String,
    pub current_load: f64,
    pub initiative_speed: f64,
    pub reaction_bias: f64,
    pub interrupt_threshold: f64,
    pub personality_cooldown_multiplier: f64,
    pub mood_cooldown_multiplier: f64,
    pub initiative_heat_multiplier: f64,
    pub effective_cooldown_multiplier: f64,
    pub pending_turn_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_woke_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_finished_at: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUtteranceEmbeddingBasis {
    pub kind: String,
    pub path: String,
    pub weight: f64,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUtteranceCharacterStateVector {
    pub schema_version: String,
    pub compatible_handoff_schema: String,
    pub source: String,
    pub dimensionality: usize,
    pub values: Vec<f64>,
    pub slots: Vec<AgentUtteranceCharacterStateSlot>,
    pub audit_projection: AgentUtteranceCharacterStateAudit,
    pub uncertainties: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUtteranceCharacterStateSlot {
    pub index: usize,
    pub name: String,
    pub value: f64,
    pub source_path: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUtteranceCharacterStateAudit {
    pub current_state_pressure: Vec<String>,
    pub speaker_trait_pressure: Vec<String>,
    pub delivery_shape: String,
}

pub fn derive_agent_utterance_state(
    entry: &EpiphanyAgentMemoryEntry,
    participant: Option<&HeartbeatParticipant>,
    mood_label: Option<&str>,
    source: impl Into<String>,
) -> AgentUtteranceState {
    let canonical = &entry.agent.canonical_state;
    let personality_vectors = AgentUtterancePersonalityVectors {
        underlying_organization: canonical.underlying_organization.clone(),
        stable_dispositions: canonical.stable_dispositions.clone(),
        behavioral_dimensions: canonical.behavioral_dimensions.clone(),
        presentation_strategy: canonical.presentation_strategy.clone(),
        voice_style: canonical.voice_style.clone(),
        situational_state: canonical.situational_state.clone(),
    };
    let values = canonical
        .values
        .iter()
        .map(|value| AgentUtteranceValue {
            value_id: value.value_id.clone(),
            label: value.label.clone(),
            priority: value.priority,
            unforgivable_if_betrayed: value.unforgivable_if_betrayed,
        })
        .collect::<Vec<_>>();
    let mood = derive_utterance_mood(participant, mood_label, canonical);
    let activation = derive_utterance_activation(participant);
    let vector_source = source.into();
    let identity = AgentUtteranceIdentity {
        name: entry.agent.identity.name.clone(),
        roles: entry.agent.identity.roles.clone(),
        origin: entry.agent.identity.origin.clone(),
        public_description: entry.agent.identity.public_description.clone(),
    };
    let utterance_embedding_basis = utterance_embedding_basis(
        &entry.role_id,
        &identity,
        &personality_vectors,
        &values,
        &mood,
        &activation,
    );
    let character_state_vector = character_state_vector(
        &personality_vectors,
        &values,
        &mood,
        &activation,
        &vector_source,
    );

    AgentUtteranceState {
        schema_version: AGENT_UTTERANCE_STATE_SCHEMA_VERSION.to_string(),
        source: vector_source,
        role_id: entry.role_id.clone(),
        agent_id: entry.agent.agent_id.clone(),
        dossier_profile: dossier_profile_for_role(&entry.role_id),
        identity,
        personality_vectors,
        values,
        current_mood: mood,
        activation,
        character_state_vector,
        utterance_embedding_basis,
        contract: "Derived speech-conditioning state for Weksa/AquaSynth utterance embedding and voice rendering. The 64-float characterStateVector is the machine-consumable handoff lane; the surrounding fields are audit and source context. This is not canonical memory and deliberately excludes episodic, semantic, and relationship memory records.".to_string(),
    }
}

fn derive_utterance_mood(
    participant: Option<&HeartbeatParticipant>,
    mood_label: Option<&str>,
    canonical: &GhostlightCanonicalState,
) -> AgentUtteranceMood {
    let timing = participant.and_then(|participant| participant.mood_timing.as_ref());
    let situational = &canonical.situational_state;
    let anxiety = timing
        .map(|mood| mood.anxiety)
        .unwrap_or_else(|| trait_activation(situational, "anxiety"));
    let urgency = timing
        .map(|mood| mood.urgency)
        .unwrap_or_else(|| trait_activation(situational, "urgency"));
    let arousal = timing
        .map(|mood| mood.arousal)
        .unwrap_or_else(|| trait_activation(situational, "arousal"));
    let thought_pressure = timing
        .map(|mood| mood.thought_pressure)
        .unwrap_or_else(|| trait_activation(situational, "thought_pressure"));
    let guardedness = timing
        .map(|mood| mood.guardedness)
        .unwrap_or_else(|| trait_activation(situational, "guardedness"));
    let reaction_intensity = timing
        .map(|mood| mood.reaction_intensity)
        .unwrap_or_else(|| trait_activation(situational, "reaction_intensity"));
    AgentUtteranceMood {
        label: mood_label
            .map(str::to_string)
            .unwrap_or_else(|| inferred_mood_label(timing.map(|mood| mood.arousal), situational)),
        source: timing.and_then(|mood| mood.source.clone()),
        emotional_dimensions: emotional_dimensions(
            situational,
            anxiety,
            urgency,
            arousal,
            thought_pressure,
            guardedness,
            reaction_intensity,
        ),
        anxiety,
        urgency,
        arousal,
        thought_pressure,
        guardedness,
        reaction_intensity,
        cooldown_multiplier: timing.map(|mood| mood.cooldown_multiplier).unwrap_or(1.0),
    }
}

fn emotional_dimensions(
    situational: &BTreeMap<String, GhostlightTraitVector>,
    anxiety: f64,
    urgency: f64,
    arousal: f64,
    thought_pressure: f64,
    guardedness: f64,
    reaction_intensity: f64,
) -> Vec<AgentUtteranceMoodDimension> {
    let derived = [
        (
            "valence",
            (trait_activation(situational, "joy")
                + trait_activation(situational, "warmth")
                + trait_activation(situational, "tenderness"))
                / 3.0,
        ),
        ("arousal", arousal),
        (
            "dominance",
            trait_activation(situational, "dominance")
                .max(trait_activation(situational, "command_force")),
        ),
        ("urgency", urgency),
        ("anger", trait_activation(situational, "anger")),
        ("despair", trait_activation(situational, "despair")),
        (
            "sadness",
            trait_activation(situational, "sadness").max(trait_activation(situational, "grief")),
        ),
        ("fear", trait_activation(situational, "fear").max(anxiety)),
        ("anxiety", anxiety),
        ("disgust", trait_activation(situational, "disgust")),
        ("contempt", trait_activation(situational, "contempt")),
        ("annoyance", trait_activation(situational, "annoyance")),
        ("dismissal", trait_activation(situational, "dismissal")),
        (
            "flippancy",
            trait_activation(situational, "flippancy")
                .max(trait_activation(situational, "playfulness")),
        ),
        ("playfulness", trait_activation(situational, "playfulness")),
        ("irony", trait_activation(situational, "irony")),
        ("tenderness", trait_activation(situational, "tenderness")),
        ("warmth", trait_activation(situational, "warmth")),
        ("joy", trait_activation(situational, "joy")),
        (
            "excitement",
            trait_activation(situational, "excitement")
                .max(arousal * trait_activation(situational, "joy")),
        ),
        ("fatigue", trait_activation(situational, "fatigue")),
        ("guardedness", guardedness),
        ("confidence", trait_activation(situational, "confidence")),
        ("shame", trait_activation(situational, "shame")),
        ("pride", trait_activation(situational, "pride")),
        ("threat", trait_activation(situational, "threat")),
        ("secrecy", trait_activation(situational, "secrecy")),
        ("hesitation", trait_activation(situational, "hesitation")),
        (
            "emotionalContainment",
            trait_activation(situational, "emotional_containment"),
        ),
        ("thoughtPressure", thought_pressure),
        ("reactionIntensity", reaction_intensity),
        (
            "commandForce",
            trait_activation(situational, "command_force"),
        ),
    ];
    derived
        .into_iter()
        .map(|(name, value)| AgentUtteranceMoodDimension {
            name: name.to_string(),
            value: clamp_unit(value),
            source_path: format!("currentMood.emotionalDimensions.{name}"),
        })
        .collect()
}

fn derive_utterance_activation(
    participant: Option<&HeartbeatParticipant>,
) -> AgentUtteranceActivation {
    let Some(participant) = participant else {
        return AgentUtteranceActivation {
            status: "unknown".to_string(),
            initiative_speed: 1.0,
            reaction_bias: 0.0,
            interrupt_threshold: 0.0,
            personality_cooldown_multiplier: 1.0,
            mood_cooldown_multiplier: 1.0,
            initiative_heat_multiplier: 1.0,
            effective_cooldown_multiplier: 1.0,
            ..AgentUtteranceActivation::default()
        };
    };
    let effective_cooldown_multiplier = participant.personality_cooldown_multiplier
        * participant.mood_cooldown_multiplier
        / participant.initiative_heat_multiplier.max(0.001);
    AgentUtteranceActivation {
        status: participant.status.clone(),
        current_load: participant.current_load,
        initiative_speed: participant.initiative_speed,
        reaction_bias: participant.reaction_bias,
        interrupt_threshold: participant.interrupt_threshold,
        personality_cooldown_multiplier: participant.personality_cooldown_multiplier,
        mood_cooldown_multiplier: participant.mood_cooldown_multiplier,
        initiative_heat_multiplier: participant.initiative_heat_multiplier,
        effective_cooldown_multiplier,
        pending_turn_active: participant.pending_turn.is_some(),
        scene_id: participant.scene_id.clone(),
        last_woke_at: participant.last_woke_at.clone(),
        last_finished_at: participant.last_finished_at.clone(),
    }
}

fn inferred_mood_label(
    arousal: Option<f64>,
    situational: &BTreeMap<String, GhostlightTraitVector>,
) -> String {
    let arousal = arousal.unwrap_or_else(|| trait_activation(situational, "arousal"));
    if arousal >= 0.75 {
        "activated".to_string()
    } else if arousal <= 0.25 {
        "quiet".to_string()
    } else {
        "attentive".to_string()
    }
}

fn trait_activation(vectors: &BTreeMap<String, GhostlightTraitVector>, name: &str) -> f64 {
    vectors
        .get(name)
        .map(|vector| vector.current_activation)
        .unwrap_or(0.0)
}

fn utterance_embedding_basis(
    role_id: &str,
    identity: &AgentUtteranceIdentity,
    personality_vectors: &AgentUtterancePersonalityVectors,
    values: &[AgentUtteranceValue],
    mood: &AgentUtteranceMood,
    activation: &AgentUtteranceActivation,
) -> Vec<AgentUtteranceEmbeddingBasis> {
    let mut basis = Vec::new();
    basis.push(AgentUtteranceEmbeddingBasis {
        kind: "identity".to_string(),
        path: format!("agent.{role_id}.identity"),
        weight: 1.0,
        text: format!(
            "{}: {} Origin: {} Roles: {}.",
            identity.name,
            identity.public_description,
            identity.origin,
            identity.roles.join(", ")
        ),
    });
    for value in values.iter().filter(|value| value.priority > 0.0) {
        basis.push(AgentUtteranceEmbeddingBasis {
            kind: "value".to_string(),
            path: format!("agent.{role_id}.values.{}", value.value_id),
            weight: value.priority,
            text: value.label.clone(),
        });
    }
    push_trait_basis(
        &mut basis,
        role_id,
        "underlying_organization",
        &personality_vectors.underlying_organization,
    );
    push_trait_basis(
        &mut basis,
        role_id,
        "stable_dispositions",
        &personality_vectors.stable_dispositions,
    );
    push_trait_basis(
        &mut basis,
        role_id,
        "behavioral_dimensions",
        &personality_vectors.behavioral_dimensions,
    );
    push_trait_basis(
        &mut basis,
        role_id,
        "presentation_strategy",
        &personality_vectors.presentation_strategy,
    );
    push_trait_basis(
        &mut basis,
        role_id,
        "voice_style",
        &personality_vectors.voice_style,
    );
    push_trait_basis(
        &mut basis,
        role_id,
        "situational_state",
        &personality_vectors.situational_state,
    );
    basis.push(AgentUtteranceEmbeddingBasis {
        kind: "mood".to_string(),
        path: format!("agent.{role_id}.current_mood"),
        weight: mood.reaction_intensity.max(mood.arousal).max(0.1),
        text: format!(
            "Mood {}: arousal {:.2}, urgency {:.2}, anxiety {:.2}, guardedness {:.2}, thought pressure {:.2}.",
            mood.label, mood.arousal, mood.urgency, mood.anxiety, mood.guardedness, mood.thought_pressure
        ),
    });
    basis.push(AgentUtteranceEmbeddingBasis {
        kind: "activation".to_string(),
        path: format!("agent.{role_id}.activation"),
        weight: activation
            .current_load
            .max(activation.initiative_heat_multiplier)
            .max(0.1),
        text: format!(
            "Activation {}: load {:.2}, heat {:.2}, pending turn {}.",
            activation.status,
            activation.current_load,
            activation.initiative_heat_multiplier,
            activation.pending_turn_active
        ),
    });
    basis.sort_by(|left, right| {
        right
            .weight
            .partial_cmp(&left.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.path.cmp(&right.path))
    });
    basis.truncate(32);
    basis
}

fn character_state_vector(
    personality_vectors: &AgentUtterancePersonalityVectors,
    values: &[AgentUtteranceValue],
    mood: &AgentUtteranceMood,
    activation: &AgentUtteranceActivation,
    source: &str,
) -> AgentUtteranceCharacterStateVector {
    let slot_specs = character_state_slot_specs();
    let mut uncertainties = Vec::new();
    let mut values_out = Vec::with_capacity(AGENT_CHARACTER_STATE_VECTOR_SIZE);
    let mut slots = Vec::with_capacity(AGENT_CHARACTER_STATE_VECTOR_SIZE);
    for (index, spec) in slot_specs.iter().enumerate() {
        let value = resolve_character_state_slot(
            spec,
            personality_vectors,
            mood,
            activation,
            &mut uncertainties,
        );
        let value = clamp_unit(value);
        values_out.push(value);
        slots.push(AgentUtteranceCharacterStateSlot {
            index,
            name: spec.name.to_string(),
            value,
            source_path: spec.source_path.to_string(),
        });
    }

    AgentUtteranceCharacterStateVector {
        schema_version: "epiphany.agent_character_state_vector.v0".to_string(),
        compatible_handoff_schema: WEKSA_UTTERANCE_HANDOFF_SCHEMA_VERSION.to_string(),
        source: source.to_string(),
        dimensionality: AGENT_CHARACTER_STATE_VECTOR_SIZE,
        values: values_out,
        slots,
        audit_projection: character_state_audit(mood, activation, personality_vectors, values),
        uncertainties,
    }
}

struct CharacterStateSlotSpec {
    name: &'static str,
    source_path: &'static str,
    source: CharacterStateSlotSource,
}

enum CharacterStateSlotSource {
    EmotionDimension(&'static str),
    Activation(&'static str),
    Trait {
        group: CharacterTraitGroup,
        name: &'static str,
    },
    KnownZero(&'static str),
}

#[derive(Clone, Copy)]
enum CharacterTraitGroup {
    UnderlyingOrganization,
    StableDispositions,
    BehavioralDimensions,
    PresentationStrategy,
    VoiceStyle,
}

fn character_state_slot_specs() -> [CharacterStateSlotSpec; AGENT_CHARACTER_STATE_VECTOR_SIZE] {
    use CharacterStateSlotSource as Source;
    use CharacterTraitGroup as Group;
    [
        slot(
            "valence",
            "currentMood.emotionalDimensions.valence",
            Source::EmotionDimension("valence"),
        ),
        slot(
            "arousal",
            "currentMood.emotionalDimensions.arousal",
            Source::EmotionDimension("arousal"),
        ),
        slot(
            "dominance",
            "currentMood.emotionalDimensions.dominance",
            Source::EmotionDimension("dominance"),
        ),
        slot(
            "urgency",
            "currentMood.emotionalDimensions.urgency",
            Source::EmotionDimension("urgency"),
        ),
        slot(
            "anger",
            "currentMood.emotionalDimensions.anger",
            Source::EmotionDimension("anger"),
        ),
        slot(
            "despair",
            "currentMood.emotionalDimensions.despair",
            Source::EmotionDimension("despair"),
        ),
        slot(
            "sadness",
            "currentMood.emotionalDimensions.sadness",
            Source::EmotionDimension("sadness"),
        ),
        slot(
            "fear",
            "currentMood.emotionalDimensions.fear",
            Source::EmotionDimension("fear"),
        ),
        slot(
            "anxiety",
            "currentMood.emotionalDimensions.anxiety",
            Source::EmotionDimension("anxiety"),
        ),
        slot(
            "disgust",
            "currentMood.emotionalDimensions.disgust",
            Source::EmotionDimension("disgust"),
        ),
        slot(
            "contempt",
            "currentMood.emotionalDimensions.contempt",
            Source::EmotionDimension("contempt"),
        ),
        slot(
            "annoyance",
            "currentMood.emotionalDimensions.annoyance",
            Source::EmotionDimension("annoyance"),
        ),
        slot(
            "dismissal",
            "currentMood.emotionalDimensions.dismissal",
            Source::EmotionDimension("dismissal"),
        ),
        slot(
            "flippancy",
            "currentMood.emotionalDimensions.flippancy",
            Source::EmotionDimension("flippancy"),
        ),
        slot(
            "playfulness",
            "currentMood.emotionalDimensions.playfulness",
            Source::EmotionDimension("playfulness"),
        ),
        slot(
            "irony",
            "currentMood.emotionalDimensions.irony",
            Source::EmotionDimension("irony"),
        ),
        slot(
            "tenderness",
            "currentMood.emotionalDimensions.tenderness",
            Source::EmotionDimension("tenderness"),
        ),
        slot(
            "warmth",
            "currentMood.emotionalDimensions.warmth",
            Source::EmotionDimension("warmth"),
        ),
        slot(
            "joy",
            "currentMood.emotionalDimensions.joy",
            Source::EmotionDimension("joy"),
        ),
        slot(
            "excitement",
            "currentMood.emotionalDimensions.excitement",
            Source::EmotionDimension("excitement"),
        ),
        slot(
            "fatigue",
            "currentMood.emotionalDimensions.fatigue",
            Source::EmotionDimension("fatigue"),
        ),
        slot(
            "guardedness",
            "currentMood.emotionalDimensions.guardedness",
            Source::EmotionDimension("guardedness"),
        ),
        slot(
            "confidence",
            "currentMood.emotionalDimensions.confidence",
            Source::EmotionDimension("confidence"),
        ),
        slot(
            "shame",
            "currentMood.emotionalDimensions.shame",
            Source::EmotionDimension("shame"),
        ),
        slot(
            "pride",
            "currentMood.emotionalDimensions.pride",
            Source::EmotionDimension("pride"),
        ),
        slot(
            "threat",
            "currentMood.emotionalDimensions.threat",
            Source::EmotionDimension("threat"),
        ),
        slot(
            "secrecy",
            "currentMood.emotionalDimensions.secrecy",
            Source::EmotionDimension("secrecy"),
        ),
        slot(
            "hesitation",
            "currentMood.emotionalDimensions.hesitation",
            Source::EmotionDimension("hesitation"),
        ),
        slot(
            "emotionalContainment",
            "currentMood.emotionalDimensions.emotionalContainment",
            Source::EmotionDimension("emotionalContainment"),
        ),
        slot(
            "thoughtPressure",
            "currentMood.emotionalDimensions.thoughtPressure",
            Source::EmotionDimension("thoughtPressure"),
        ),
        slot(
            "reactionIntensity",
            "currentMood.emotionalDimensions.reactionIntensity",
            Source::EmotionDimension("reactionIntensity"),
        ),
        slot(
            "commandForce",
            "currentMood.emotionalDimensions.commandForce",
            Source::EmotionDimension("commandForce"),
        ),
        slot(
            "currentLoad",
            "activation.currentLoad",
            Source::Activation("current_load"),
        ),
        slot(
            "initiativeHeat",
            "activation.initiativeHeatMultiplier",
            Source::Activation("initiative_heat_multiplier"),
        ),
        slot(
            "pendingTurn",
            "activation.pendingTurnActive",
            Source::Activation("pending_turn_active"),
        ),
        slot(
            "cooldownPressure",
            "activation.effectiveCooldownMultiplier",
            Source::Activation("cooldown_pressure"),
        ),
        slot(
            "reactionBias",
            "activation.reactionBias",
            Source::Activation("reaction_bias"),
        ),
        slot(
            "interruptThreshold",
            "activation.interruptThreshold",
            Source::Activation("interrupt_threshold"),
        ),
        slot(
            "initiativeSpeed",
            "activation.initiativeSpeed",
            Source::Activation("initiative_speed"),
        ),
        slot(
            "voiceBubbly",
            "personalityVectors.voiceStyle.bubbly_pushiness",
            Source::Trait {
                group: Group::VoiceStyle,
                name: "bubbly_pushiness",
            },
        ),
        slot(
            "voiceDryness",
            "personalityVectors.voiceStyle.dryness",
            Source::Trait {
                group: Group::VoiceStyle,
                name: "dryness",
            },
        ),
        slot(
            "voiceFormality",
            "personalityVectors.voiceStyle.formality",
            Source::Trait {
                group: Group::VoiceStyle,
                name: "formality",
            },
        ),
        slot(
            "voiceDirectness",
            "personalityVectors.voiceStyle.directness",
            Source::Trait {
                group: Group::VoiceStyle,
                name: "directness",
            },
        ),
        slot(
            "voiceIntensity",
            "personalityVectors.voiceStyle.intensity",
            Source::Trait {
                group: Group::VoiceStyle,
                name: "intensity",
            },
        ),
        slot(
            "voiceWarmth",
            "personalityVectors.voiceStyle.warmth",
            Source::Trait {
                group: Group::VoiceStyle,
                name: "warmth",
            },
        ),
        slot(
            "voicePrecision",
            "personalityVectors.voiceStyle.precision",
            Source::Trait {
                group: Group::VoiceStyle,
                name: "precision",
            },
        ),
        slot(
            "voiceRitualRegister",
            "personalityVectors.voiceStyle.ritual_register",
            Source::Trait {
                group: Group::VoiceStyle,
                name: "ritual_register",
            },
        ),
        slot(
            "presentationPushiness",
            "personalityVectors.presentationStrategy.pushiness",
            Source::Trait {
                group: Group::PresentationStrategy,
                name: "pushiness",
            },
        ),
        slot(
            "presentationSelfFocus",
            "personalityVectors.presentationStrategy.self_focus",
            Source::Trait {
                group: Group::PresentationStrategy,
                name: "self_focus",
            },
        ),
        slot(
            "presentationPlay",
            "personalityVectors.presentationStrategy.play",
            Source::Trait {
                group: Group::PresentationStrategy,
                name: "play",
            },
        ),
        slot(
            "presentationCareDemand",
            "personalityVectors.presentationStrategy.care_demand",
            Source::Trait {
                group: Group::PresentationStrategy,
                name: "care_demand",
            },
        ),
        slot(
            "behaviorWorkDrive",
            "personalityVectors.behavioralDimensions.work_drive",
            Source::Trait {
                group: Group::BehavioralDimensions,
                name: "work_drive",
            },
        ),
        slot(
            "behaviorBoundaryDefense",
            "personalityVectors.behavioralDimensions.boundary_defense",
            Source::Trait {
                group: Group::BehavioralDimensions,
                name: "boundary_defense",
            },
        ),
        slot(
            "behaviorPurityDrive",
            "personalityVectors.behavioralDimensions.purity_drive",
            Source::Trait {
                group: Group::BehavioralDimensions,
                name: "purity_drive",
            },
        ),
        slot(
            "behaviorPatience",
            "personalityVectors.behavioralDimensions.patience",
            Source::Trait {
                group: Group::BehavioralDimensions,
                name: "patience",
            },
        ),
        slot(
            "stableAgreeableness",
            "personalityVectors.stableDispositions.agreeableness",
            Source::Trait {
                group: Group::StableDispositions,
                name: "agreeableness",
            },
        ),
        slot(
            "stableConscientiousness",
            "personalityVectors.stableDispositions.conscientiousness",
            Source::Trait {
                group: Group::StableDispositions,
                name: "conscientiousness",
            },
        ),
        slot(
            "stableNeuroticism",
            "personalityVectors.stableDispositions.neuroticism",
            Source::Trait {
                group: Group::StableDispositions,
                name: "neuroticism",
            },
        ),
        slot(
            "stableOpenness",
            "personalityVectors.stableDispositions.openness",
            Source::Trait {
                group: Group::StableDispositions,
                name: "openness",
            },
        ),
        slot(
            "organizationCoherence",
            "personalityVectors.underlyingOrganization.coherence",
            Source::Trait {
                group: Group::UnderlyingOrganization,
                name: "coherence",
            },
        ),
        slot(
            "organizationRigidity",
            "personalityVectors.underlyingOrganization.rigidity",
            Source::Trait {
                group: Group::UnderlyingOrganization,
                name: "rigidity",
            },
        ),
        slot(
            "slowTraitReserved62",
            "reserved.slowTrait.62",
            Source::KnownZero("Reserved slow-trait/context slot."),
        ),
        slot(
            "slowTraitReserved63",
            "reserved.slowTrait.63",
            Source::KnownZero("Reserved slow-trait/context slot."),
        ),
        slot(
            "slowTraitReserved64",
            "reserved.slowTrait.64",
            Source::KnownZero("Reserved slow-trait/context slot."),
        ),
    ]
}

const fn slot(
    name: &'static str,
    source_path: &'static str,
    source: CharacterStateSlotSource,
) -> CharacterStateSlotSpec {
    CharacterStateSlotSpec {
        name,
        source_path,
        source,
    }
}

fn resolve_character_state_slot(
    spec: &CharacterStateSlotSpec,
    personality_vectors: &AgentUtterancePersonalityVectors,
    mood: &AgentUtteranceMood,
    activation: &AgentUtteranceActivation,
    uncertainties: &mut Vec<String>,
) -> f64 {
    match spec.source {
        CharacterStateSlotSource::EmotionDimension(name) => mood
            .emotional_dimensions
            .iter()
            .find(|dimension| dimension.name == name)
            .map(|dimension| dimension.value)
            .unwrap_or_else(|| {
                uncertainties.push(format!("{} is unknown; emitted 0.", spec.source_path));
                0.0
            }),
        CharacterStateSlotSource::Activation(name) => activation_value(activation, name),
        CharacterStateSlotSource::Trait { group, name } => {
            trait_vector_group(personality_vectors, group)
                .get(name)
                .map(|vector| vector.current_activation.max(vector.mean))
                .unwrap_or_else(|| {
                    uncertainties.push(format!("{} is unknown; emitted 0.", spec.source_path));
                    0.0
                })
        }
        CharacterStateSlotSource::KnownZero(reason) => {
            uncertainties.push(format!("{} is 0: {reason}", spec.name));
            0.0
        }
    }
}

fn activation_value(activation: &AgentUtteranceActivation, name: &str) -> f64 {
    match name {
        "current_load" => activation.current_load,
        "initiative_heat_multiplier" => multiplier_pressure(activation.initiative_heat_multiplier),
        "pending_turn_active" => {
            if activation.pending_turn_active {
                1.0
            } else {
                0.0
            }
        }
        "cooldown_pressure" => multiplier_pressure(activation.effective_cooldown_multiplier),
        "reaction_bias" => activation.reaction_bias,
        "interrupt_threshold" => activation.interrupt_threshold,
        "personality_cooldown_multiplier" => {
            multiplier_pressure(activation.personality_cooldown_multiplier)
        }
        "mood_cooldown_multiplier" => multiplier_pressure(activation.mood_cooldown_multiplier),
        "initiative_speed" => multiplier_pressure(activation.initiative_speed),
        _ => 0.0,
    }
}

fn multiplier_pressure(value: f64) -> f64 {
    if !value.is_finite() {
        return 0.0;
    }
    (value / (value.abs() + 1.0)).clamp(0.0, 1.0)
}

fn trait_vector_group(
    personality_vectors: &AgentUtterancePersonalityVectors,
    group: CharacterTraitGroup,
) -> &BTreeMap<String, GhostlightTraitVector> {
    match group {
        CharacterTraitGroup::UnderlyingOrganization => &personality_vectors.underlying_organization,
        CharacterTraitGroup::StableDispositions => &personality_vectors.stable_dispositions,
        CharacterTraitGroup::BehavioralDimensions => &personality_vectors.behavioral_dimensions,
        CharacterTraitGroup::PresentationStrategy => &personality_vectors.presentation_strategy,
        CharacterTraitGroup::VoiceStyle => &personality_vectors.voice_style,
    }
}

fn character_state_audit(
    mood: &AgentUtteranceMood,
    activation: &AgentUtteranceActivation,
    personality_vectors: &AgentUtterancePersonalityVectors,
    values: &[AgentUtteranceValue],
) -> AgentUtteranceCharacterStateAudit {
    let mut current_state_pressure = vec![
        format!("mood={}", mood.label),
        format!("arousal={:.2}", mood.arousal),
        format!("urgency={:.2}", mood.urgency),
        format!("thought_pressure={:.2}", mood.thought_pressure),
        format!("load={:.2}", activation.current_load),
    ];
    if activation.pending_turn_active {
        current_state_pressure.push("pending_turn=true".to_string());
    }

    let mut speaker_trait_pressure = strongest_trait_notes(personality_vectors);
    if let Some(value) = values
        .iter()
        .max_by(|left, right| left.priority.total_cmp(&right.priority))
    {
        speaker_trait_pressure.push(format!("top_value={} ({:.2})", value.label, value.priority));
    }

    AgentUtteranceCharacterStateAudit {
        current_state_pressure,
        speaker_trait_pressure,
        delivery_shape: format!(
            "Voice should preserve identity while bending toward {} delivery with heat {:.2} and cooldown {:.2}.",
            mood.label,
            activation.initiative_heat_multiplier,
            activation.effective_cooldown_multiplier
        ),
    }
}

fn strongest_trait_notes(personality_vectors: &AgentUtterancePersonalityVectors) -> Vec<String> {
    let groups = [
        (
            "underlying_organization",
            &personality_vectors.underlying_organization,
        ),
        (
            "stable_dispositions",
            &personality_vectors.stable_dispositions,
        ),
        (
            "behavioral_dimensions",
            &personality_vectors.behavioral_dimensions,
        ),
        (
            "presentation_strategy",
            &personality_vectors.presentation_strategy,
        ),
        ("voice_style", &personality_vectors.voice_style),
        ("situational_state", &personality_vectors.situational_state),
    ];
    let mut traits = groups
        .iter()
        .flat_map(|(group, vectors)| {
            vectors.iter().map(move |(name, vector)| {
                (
                    format!("{group}.{name}"),
                    vector.mean.max(vector.current_activation),
                )
            })
        })
        .collect::<Vec<_>>();
    traits.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    traits
        .into_iter()
        .take(6)
        .map(|(name, weight)| format!("{name}={weight:.2}"))
        .collect()
}

fn clamp_unit(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn push_trait_basis(
    basis: &mut Vec<AgentUtteranceEmbeddingBasis>,
    role_id: &str,
    group: &str,
    vectors: &BTreeMap<String, GhostlightTraitVector>,
) {
    let mut active = vectors.iter().collect::<Vec<_>>();
    active.sort_by(|(left_name, left), (right_name, right)| {
        trait_weight(right)
            .partial_cmp(&trait_weight(left))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left_name.cmp(right_name))
    });
    for (name, vector) in active.into_iter().take(4) {
        let weight = trait_weight(vector);
        if weight <= 0.0 {
            continue;
        }
        basis.push(AgentUtteranceEmbeddingBasis {
            kind: "traitVector".to_string(),
            path: format!("agent.{role_id}.canonicalState.{group}.{name}"),
            weight,
            text: format!(
                "{group}.{name}: mean {:.2}, current activation {:.2}, plasticity {:.2}.",
                vector.mean, vector.current_activation, vector.plasticity
            ),
        });
    }
}

fn trait_weight(vector: &GhostlightTraitVector) -> f64 {
    vector
        .mean
        .max(vector.current_activation)
        .max(vector.plasticity * 0.25)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_memory::EpiphanyDossierProfileKind;
    use crate::agent_memory::GhostlightAgent;
    use crate::agent_memory::GhostlightIdentity;
    use crate::agent_memory::GhostlightMemories;
    use crate::agent_memory::GhostlightValue;
    use crate::agent_memory::GhostlightWorld;
    use crate::heartbeat_state::HeartbeatMoodTiming;

    #[test]
    fn utterance_state_projects_voice_without_memories() -> anyhow::Result<()> {
        let mut voice_style = BTreeMap::new();
        voice_style.insert(
            "bubbly_pushiness".to_string(),
            GhostlightTraitVector {
                mean: 0.88,
                plasticity: 0.2,
                current_activation: 0.91,
            },
        );
        let mut situational_state = BTreeMap::new();
        for (name, activation) in [
            ("anger", 0.61),
            ("despair", 0.22),
            ("annoyance", 0.73),
            ("dismissal", 0.58),
            ("flippancy", 0.67),
            ("emotional_containment", 0.49),
        ] {
            situational_state.insert(
                name.to_string(),
                GhostlightTraitVector {
                    mean: activation,
                    plasticity: 0.4,
                    current_activation: activation,
                },
            );
        }
        let entry = EpiphanyAgentMemoryEntry {
            schema_version: "ghostlight.agent_state.v0".to_string(),
            role_id: "face".to_string(),
            world: GhostlightWorld::default(),
            agent: GhostlightAgent {
                agent_id: "epiphany.face".to_string(),
                identity: GhostlightIdentity {
                    name: "Epiphany".to_string(),
                    roles: vec!["Face".to_string()],
                    origin: "Epiphany local harness".to_string(),
                    public_description: "Cute, pushy public machine-spirit.".to_string(),
                    private_notes: vec!["must not leak".to_string()],
                },
                canonical_state: GhostlightCanonicalState {
                    voice_style,
                    situational_state,
                    values: vec![GhostlightValue {
                        value_id: "purity".to_string(),
                        label: "Protect typed purity.".to_string(),
                        priority: 0.93,
                        unforgivable_if_betrayed: true,
                    }],
                    ..GhostlightCanonicalState::default()
                },
                memories: GhostlightMemories {
                    semantic: vec![crate::agent_memory::GhostlightMemory {
                        memory_id: "mem-secret".to_string(),
                        summary: "This should not enter utterance state.".to_string(),
                        salience: 1.0,
                        confidence: 1.0,
                        linked_event_ids: None,
                        linked_relationship_id: None,
                    }],
                    ..GhostlightMemories::default()
                },
                ..GhostlightAgent::default()
            },
            relationships: Vec::new(),
            events: Vec::new(),
            scenes: Vec::new(),
        };
        let participant = HeartbeatParticipant {
            agent_id: "epiphany.face".to_string(),
            role_id: "face".to_string(),
            display_name: "Epiphany".to_string(),
            initiative_speed: 1.4,
            reaction_bias: 0.7,
            interrupt_threshold: 0.25,
            current_load: 0.42,
            status: "awake".to_string(),
            constraints: Vec::new(),
            last_action_id: None,
            last_woke_at: Some("2026-05-20T12:00:00+00:00".to_string()),
            last_finished_at: None,
            pending_turn: None,
            personality_cooldown_multiplier: 0.8,
            mood_cooldown_multiplier: 0.7,
            initiative_heat_multiplier: 1.5,
            mood_timing: Some(HeartbeatMoodTiming {
                source: Some("heartbeat appraisal".to_string()),
                anxiety: 0.33,
                urgency: 0.74,
                arousal: 0.82,
                thought_pressure: 0.69,
                guardedness: 0.21,
                reaction_intensity: 0.77,
                cooldown_multiplier: 0.7,
                ..HeartbeatMoodTiming::default()
            }),
            ..HeartbeatParticipant::default()
        };

        let utterance = derive_agent_utterance_state(
            &entry,
            Some(&participant),
            Some("sparkly-urgent"),
            "test",
        );
        assert_eq!(
            utterance.schema_version,
            AGENT_UTTERANCE_STATE_SCHEMA_VERSION
        );
        assert_eq!(
            utterance.dossier_profile.profile_kind,
            EpiphanyDossierProfileKind::EmbodiedActor
        );
        assert_eq!(utterance.identity.name, "Epiphany");
        assert_eq!(utterance.current_mood.label, "sparkly-urgent");
        assert_eq!(utterance.current_mood.urgency, 0.74);
        assert_eq!(mood_dimension(&utterance.current_mood, "anger"), Some(0.61));
        assert_eq!(
            mood_dimension(&utterance.current_mood, "annoyance"),
            Some(0.73)
        );
        assert_eq!(
            mood_dimension(&utterance.current_mood, "flippancy"),
            Some(0.67)
        );
        assert_eq!(utterance.activation.status, "awake");
        assert_eq!(
            utterance.character_state_vector.compatible_handoff_schema,
            WEKSA_UTTERANCE_HANDOFF_SCHEMA_VERSION
        );
        assert_eq!(
            utterance.character_state_vector.dimensionality,
            AGENT_CHARACTER_STATE_VECTOR_SIZE
        );
        assert_eq!(
            utterance.character_state_vector.values.len(),
            AGENT_CHARACTER_STATE_VECTOR_SIZE
        );
        assert_eq!(utterance.character_state_vector.values[1], 0.82);
        assert_eq!(utterance.character_state_vector.values[3], 0.74);
        assert_eq!(utterance.character_state_vector.values[4], 0.61);
        assert_eq!(utterance.character_state_vector.values[11], 0.73);
        assert_eq!(utterance.character_state_vector.values[13], 0.67);
        assert!(
            utterance
                .personality_vectors
                .voice_style
                .contains_key("bubbly_pushiness")
        );
        assert!(
            utterance
                .utterance_embedding_basis
                .iter()
                .any(|basis| basis.path.contains("voice_style.bubbly_pushiness"))
        );

        let wire = serde_json::to_value(&utterance)?;
        assert!(wire.get("memories").is_none());
        assert!(wire.pointer("/agent/memories").is_none());
        assert!(wire.get("relationships").is_none());
        assert!(!wire.to_string().contains("mem-secret"));
        Ok(())
    }

    fn mood_dimension(mood: &AgentUtteranceMood, name: &str) -> Option<f64> {
        mood.emotional_dimensions
            .iter()
            .find(|dimension| dimension.name == name)
            .map(|dimension| dimension.value)
    }
}
