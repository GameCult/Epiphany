use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::SecondsFormat;
use epiphany_core::EpiphanyAgentMemoryEntry;
use epiphany_core::GhostlightMemory;
use epiphany_core::GhostlightPerceivedStateOverlay;
use epiphany_core::GhostlightTraitVector;
use epiphany_core::HeartbeatAgentReaction;
use epiphany_core::HeartbeatAgentThoughtAppraisal;
use epiphany_core::HeartbeatAppraisalReview;
use epiphany_core::HeartbeatCandidateImplications;
use epiphany_core::HeartbeatEmotionalAppraisal;
use epiphany_core::HeartbeatParticipantLocalContext;
use epiphany_core::HeartbeatPersonalityProjection;
use epiphany_core::dossier_profile_for_role;
use epiphany_core::load_agent_memory_entry_for_role;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

const CHARACTER_TURN_SCHEMA_VERSION: &str = "epiphany.character_turn_packet.v0";

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return usage();
    };
    let mut role = "face".to_string();
    let mut agent_store = PathBuf::from("state/agents.msgpack");
    let mut artifact_dir = PathBuf::from(".epiphany-character-loop");
    let mut stimulus: Option<String> = None;
    let mut source = "epiphany/character-loop".to_string();
    let mut mode = "public-surface".to_string();
    let mut status = "ready".to_string();
    let mut mood = "attentive".to_string();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--role" => role = next_value(&mut args, "--role")?,
            "--agent-store" => agent_store = next_path(&mut args, "--agent-store")?,
            "--artifact-dir" => artifact_dir = next_path(&mut args, "--artifact-dir")?,
            "--stimulus" => stimulus = Some(next_value(&mut args, "--stimulus")?),
            "--source" => source = next_value(&mut args, "--source")?,
            "--mode" => mode = next_value(&mut args, "--mode")?,
            "--status" => status = next_value(&mut args, "--status")?,
            "--mood" => mood = next_value(&mut args, "--mood")?,
            _ => return Err(anyhow!("unknown argument {arg:?}")),
        }
    }

    let result = match command.as_str() {
        "turn" => {
            let stimulus = read_required_text(stimulus, "stimulus")?;
            run_turn(
                &role,
                &agent_store,
                &artifact_dir,
                &stimulus,
                &source,
                &mode,
                &status,
                &mood,
            )?
        }
        "smoke" => run_smoke()?,
        _ => return usage(),
    };

    let ok = result["ok"].as_bool().unwrap_or(false);
    println!("{}", serde_json::to_string_pretty(&result)?);
    if !ok {
        std::process::exit(1);
    }
    Ok(())
}

fn run_turn(
    role: &str,
    agent_store: &Path,
    artifact_dir: &Path,
    stimulus: &str,
    source: &str,
    mode: &str,
    status: &str,
    mood: &str,
) -> Result<Value> {
    if role.trim().is_empty() {
        return Err(anyhow!("--role must not be empty"));
    }
    ensure_content(stimulus, "stimulus")?;
    let entry = load_agent_memory_entry_for_role(agent_store, role)?.ok_or_else(|| {
        anyhow!(
            "{role} agent memory is missing from {}",
            agent_store.display()
        )
    })?;
    let packet = character_turn_packet(&entry, stimulus, source, mode, status, mood);
    let path = artifact_dir.join(format!(
        "character-turn-{}-{}-{}.json",
        sanitize_file_stem(role),
        now_stamp(),
        short_id()
    ));
    write_json(&path, &packet)?;
    Ok(serde_json::json!({
        "ok": true,
        "schemaVersion": CHARACTER_TURN_SCHEMA_VERSION,
        "roleId": role,
        "agentId": entry.agent.agent_id,
        "turnPath": path,
        "turn": packet,
    }))
}

fn character_turn_packet(
    entry: &EpiphanyAgentMemoryEntry,
    stimulus: &str,
    source: &str,
    mode: &str,
    status: &str,
    mood: &str,
) -> Value {
    let dossier_profile = dossier_profile_for_role(&entry.role_id);
    let projection_seed =
        deterministic_projection_seed(entry, stimulus, source, mode, status, mood);
    let appraisal_seed = deterministic_appraisal_seed(entry, &projection_seed, stimulus, source);
    let reaction_seed = deterministic_reaction_seed(&entry.role_id, &appraisal_seed);
    serde_json::json!({
        "schema_version": CHARACTER_TURN_SCHEMA_VERSION,
        "protocol": {
            "bundle": "epiphany.character_loop",
            "referenceLineage": "Ghostlight-style character-local cognition, rebuilt as Epiphany-native protocol.",
            "mode": mode,
            "roleId": entry.role_id,
            "agentId": entry.agent.agent_id,
            "dossierProfile": dossier_profile,
            "turnKind": "stimulus_response",
            "responseContract": "Respond from projected local state and visible stimulus; do not leak hidden agent streams or invent omniscient truth.",
        },
        "source": source,
        "createdAt": now_iso(),
        "stimulus": {
            "kind": "operator_or_aquarium_input",
            "content": stimulus,
            "status": status,
            "mood": mood,
        },
        "projectedLocalContext": {
            "identity": {
                "name": entry.agent.identity.name,
                "roles": entry.agent.identity.roles,
                "origin": entry.agent.identity.origin,
                "publicDescription": entry.agent.identity.public_description,
            },
            "world": {
                "worldId": entry.world.world_id,
                "setting": entry.world.setting,
                "time": entry.world.time.label,
                "canonContext": entry.world.canon_context,
            },
            "dossierProfile": dossier_profile_for_role(&entry.role_id),
            "activeGoals": entry.agent.goals.iter().map(|goal| serde_json::json!({
                "goalId": goal.goal_id,
                "description": goal.description,
                "scope": goal.scope,
                "priority": goal.priority,
                "emotionalStake": goal.emotional_stake,
                "status": goal.status,
            })).collect::<Vec<_>>(),
            "values": entry.agent.canonical_state.values.iter().map(|value| serde_json::json!({
                "valueId": value.value_id,
                "label": value.label,
                "priority": value.priority,
                "unforgivableIfBetrayed": value.unforgivable_if_betrayed,
            })).collect::<Vec<_>>(),
            "semanticMemories": entry.agent.memories.semantic.iter().map(memory_projection).collect::<Vec<_>>(),
            "episodicMemories": entry.agent.memories.episodic.iter().map(memory_projection).collect::<Vec<_>>(),
            "relationshipMemories": entry.agent.memories.relationship_summaries.iter().map(memory_projection).collect::<Vec<_>>(),
            "perceivedStateOverlays": entry.agent.perceived_state_overlays.iter().take(6).map(perceived_overlay_projection).collect::<Vec<_>>(),
            "privateNoteCount": entry.agent.identity.private_notes.len(),
        },
        "projectionSeed": projection_seed,
        "appraisalSeed": appraisal_seed,
        "reactionSeed": reaction_seed,
        "allowedOutputs": [
            "bubble",
            "discordAquariumDraft",
            "roleFinding",
            "silence",
            "selfPatch"
        ],
        "cognitionLanes": {
            "schema_version": "epiphany.cognition_lanes.v0",
            "analytic": {
                "description": "Literal, evidence-facing lane: what happened, what constraints apply, what action is justified.",
                "requiredThreadShape": {
                    "topic": "string",
                    "claim": "string",
                    "evidenceRefs": "string[]",
                    "confidence": "0..1",
                    "counterweight": "string"
                }
            },
            "associative": {
                "description": "Pattern-facing lane: what this rhymes with, what older seam or emotional salience may matter, what should incubate.",
                "requiredThreadShape": {
                    "topic": "string",
                    "claim": "string",
                    "sourceThemeIds": "string[]",
                    "novelty": "0..1",
                    "counterweight": "string"
                }
            },
            "bridge": {
                "schema_version": "epiphany.cognition_bridge.v0",
                "contract": "Synthesize or separate the lanes, name saturation and unresolved tension, then choose bubble, draft, finding, silence, or selfPatch. The bridge may steer attention; it may not mutate project truth."
            },
            "appraisal": {
                "schema_version": "epiphany.agent_thought_appraisal.v0",
                "contract": "Project the active thought cluster through this actor's own personality vectors before reacting. Interpretation, emotion, and reaction are participant-local; state mutation still requires reviewed selfPatch or project-state acceptance."
            }
        },
        "guardrails": [
            "Humans talk to Face; other organs expose internals through Aquarium and typed artifacts rather than becoming direct chats.",
            "Use character-local projection: the actor receives its own dossier and visible stimulus, not omniscient hidden state.",
            "Run both cognition lanes before choosing an output: analytic keeps the promise honest, associative keeps the living signal from going flat.",
            "Appraise the thought cluster through this role's personality before reacting; do not use a global mood knob as a substitute for participant-local appraisal.",
            "If speech or action would be noise, return silence plus bounded rumination or selfPatch.",
            "Posting, state mutation, objective adoption, and semantic acceptance remain separate reviewed Epiphany surfaces."
        ],
        "resultShape": {
            "observedStimulus": "string",
            "localInterpretation": "string",
            "candidateBubble": "string|null",
            "candidateDraft": "string|null",
            "analyticThread": "object",
            "associativeThread": "object",
            "bridgeSynthesis": "object",
            "appraisal": "object",
            "reaction": "object",
            "decision": "bubble|draft|finding|silence|blocked",
            "selfPatch": "optional bounded role memory mutation"
        }
    })
}

fn memory_projection(memory: &GhostlightMemory) -> Value {
    serde_json::json!({
        "memoryId": memory.memory_id,
        "summary": memory.summary,
        "salience": memory.salience,
        "confidence": memory.confidence,
    })
}

#[derive(Clone, Debug)]
struct TraitSignal {
    group: &'static str,
    name: String,
    activation: f64,
    plasticity: f64,
    weight: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CharacterProjectionSeed {
    #[serde(rename = "schema_version")]
    schema_version: String,
    participant_agent_id: String,
    role_id: String,
    thought_cluster_ref: String,
    thought_tokens: Vec<String>,
    dominant_trait_matches: Vec<HeartbeatPersonalityProjection>,
    relationship_pressure: CharacterPressureSeed,
    perceived_overlay_pressure: CharacterPressureSeed,
    contract: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CharacterPressureSeed {
    strength: f64,
    summary: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CharacterReactionSeed {
    #[serde(rename = "schema_version")]
    schema_version: String,
    #[serde(flatten)]
    reaction: HeartbeatAgentReaction,
    contract: String,
}

fn deterministic_projection_seed(
    entry: &EpiphanyAgentMemoryEntry,
    stimulus: &str,
    source: &str,
    mode: &str,
    status: &str,
    mood: &str,
) -> CharacterProjectionSeed {
    let thought_tokens = stimulus_token_set(stimulus, source, mode, status, mood);
    let traits = collect_trait_signals(entry);
    let mut scored = traits
        .iter()
        .map(|item| {
            let trait_tokens = summary_tokens(&format!("{} {}", item.group, item.name));
            let overlap = token_overlap(&trait_tokens, &thought_tokens);
            let projection = round3((item.weight * (0.55 + overlap * 1.8)).clamp(0.0, 1.0));
            HeartbeatPersonalityProjection {
                group: item.group.to_string(),
                name: item.name.clone(),
                activation: round3(item.activation),
                plasticity: round3(item.plasticity),
                token_overlap: round3(overlap),
                projection,
            }
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| right.projection.total_cmp(&left.projection));
    scored.truncate(6);

    let (relationship_pressure, relationship_summary) =
        strongest_relationship_pressure(entry, &thought_tokens);
    let (overlay_pressure, overlay_summary) = strongest_overlay_pressure(entry, &thought_tokens);

    CharacterProjectionSeed {
        schema_version: "epiphany.personality_projection.v0".to_string(),
        participant_agent_id: entry.agent.agent_id.clone(),
        role_id: entry.role_id.clone(),
        thought_cluster_ref: format!("{}#stimulus", source),
        thought_tokens: thought_tokens.into_iter().collect::<Vec<_>>(),
        dominant_trait_matches: scored,
        relationship_pressure: CharacterPressureSeed {
            strength: round3(relationship_pressure),
            summary: relationship_summary,
        },
        perceived_overlay_pressure: CharacterPressureSeed {
            strength: round3(overlay_pressure),
            summary: overlay_summary,
        },
        contract:
            "Projection is participant-local and fallible. It is a seed for embodied response, not reviewed project truth."
                .to_string(),
    }
}

fn deterministic_appraisal_seed(
    entry: &EpiphanyAgentMemoryEntry,
    projection_seed: &CharacterProjectionSeed,
    stimulus: &str,
    source: &str,
) -> HeartbeatAgentThoughtAppraisal {
    let traits = collect_trait_signals(entry);
    let reactivity = average(traits.iter().take(8).map(|item| item.activation)).unwrap_or(0.5);
    let plasticity = average(traits.iter().take(8).map(|item| item.plasticity)).unwrap_or(0.5);
    let expressiveness = average(
        traits
            .iter()
            .filter(|item| item.group == "voice_style" || item.group == "presentation_strategy")
            .map(|item| item.activation),
    )
    .unwrap_or(reactivity);
    let guarded_baseline = average(
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
                    || name.contains("susp")
            })
            .map(|item| item.activation),
    )
    .unwrap_or((1.0 - expressiveness * 0.45).clamp(0.0, 1.0));
    let alignment = projection_seed
        .dominant_trait_matches
        .first()
        .map(|item| item.projection)
        .unwrap_or(reactivity);
    let relationship_pressure = projection_seed.relationship_pressure.strength;
    let overlay_pressure = projection_seed.perceived_overlay_pressure.strength;
    let thought_pressure = (0.18
        + (summary_tokens(stimulus).len() as f64 / 18.0).min(0.42)
        + relationship_pressure * 0.22
        + overlay_pressure * 0.18)
        .clamp(0.0, 1.0);
    let arousal =
        round3((thought_pressure * (0.35 + reactivity * 0.45 + plasticity * 0.2)).clamp(0.0, 1.0));
    let guardedness = round3(
        (guarded_baseline * 0.58
            + relationship_pressure * 0.18
            + overlay_pressure * 0.14
            + (1.0 - alignment) * 0.10)
            .clamp(0.0, 1.0),
    );
    let curiosity =
        round3(((1.0 - guardedness) * 0.22 + alignment * 0.46 + plasticity * 0.32).clamp(0.0, 1.0));
    let urgency =
        round3((arousal * 0.58 + guardedness * 0.22 + thought_pressure * 0.20).clamp(0.0, 1.0));
    let valence = round3((0.52 + curiosity * 0.22 - guardedness * 0.20).clamp(0.0, 1.0));
    let label = interpretation_label("draft", arousal, guardedness, curiosity);
    HeartbeatAgentThoughtAppraisal {
        schema_version: "epiphany.agent_thought_appraisal.v0".to_string(),
        appraisal_id: format!("face-appraisal-{}", short_id()),
        review_status: "generated_unreviewed".to_string(),
        participant_agent_id: entry.agent.agent_id.clone(),
        role_id: entry.role_id.clone(),
        current_character_state_ref: format!("state/agents.msgpack#{}", entry.role_id),
        thought_cluster_ref: format!("{}#stimulus", source),
        participant_local_context: HeartbeatParticipantLocalContext {
            display_name: entry.agent.identity.name.clone(),
            values: entry
                .agent
                .canonical_state
                .values
                .iter()
                .take(5)
                .map(|value| value.label.clone())
                .collect::<Vec<_>>(),
            reactivity: round3(reactivity),
            plasticity: round3(plasticity),
            expressiveness: round3(expressiveness),
            guardedness: round3(guarded_baseline),
        },
        observable_thought_summary: stimulus.to_string(),
        personality_projection: projection_seed.dominant_trait_matches.clone(),
        interpretation: format!(
            "{} appraises the visible stimulus through its local personality, relationship memory, and perceived overlays; this is embodied response guidance, not omniscient truth.",
            entry.agent.identity.name
        ),
        emotional_appraisal: HeartbeatEmotionalAppraisal {
            valence,
            arousal,
            urgency,
            curiosity,
            guardedness,
            thought_pressure: round3(thought_pressure),
        },
        interpretation_label: label.clone(),
        confidence_notes: "Deterministic Face-local appraisal rebuilt from the same Ghostlight-style personality projection logic used by heartbeat. Useful for reaction shape, still reviewable and fallible.".to_string(),
        candidate_implications: HeartbeatCandidateImplications {
            reaction_mode: reaction_mode(&label, "draft").to_string(),
            reaction_intensity: round3(
                (urgency * 0.55 + arousal * 0.3 + curiosity * 0.15).clamp(0.0, 1.0),
            ),
            should_speak: entry.role_id == "face" && guardedness < 0.78,
            should_incubate: guardedness >= 0.55,
        },
        review: HeartbeatAppraisalReview {
            accepted_for_mutation: false,
            rationale:
                "Appraisal may steer expression and reaction; state mutation still requires explicit review."
                    .to_string(),
        },
    }
}

fn deterministic_reaction_seed(
    role_id: &str,
    appraisal_seed: &HeartbeatAgentThoughtAppraisal,
) -> CharacterReactionSeed {
    let arousal = appraisal_seed.emotional_appraisal.arousal;
    let guardedness = appraisal_seed.emotional_appraisal.guardedness;
    let curiosity = appraisal_seed.emotional_appraisal.curiosity;
    let mode = appraisal_seed.candidate_implications.reaction_mode.as_str();
    let intensity = appraisal_seed.candidate_implications.reaction_intensity;
    CharacterReactionSeed {
        schema_version: "epiphany.agent_reaction.v0".to_string(),
        reaction: HeartbeatAgentReaction {
            reaction_id: format!("reaction-{}-{}", role_id, short_id()),
            role_id: role_id.to_string(),
            participant_agent_id: appraisal_seed.participant_agent_id.clone(),
            appraisal_id: appraisal_seed.appraisal_id.clone(),
            mode: mode.to_string(),
            mood_label: mood_label(arousal, guardedness, curiosity).to_string(),
            intensity: round3(intensity),
            bridge_decision: "draft".to_string(),
            surface: if role_id == "face" {
                "aquarium"
            } else {
                "internal"
            }
            .to_string(),
            recommended_use: reaction_recommended_use(role_id, mode).to_string(),
        },
        contract: "Reaction is derived from local appraisal and may color speech or silence; it still does not mutate durable state by itself.".to_string(),
    }
}

fn collect_trait_signals(entry: &EpiphanyAgentMemoryEntry) -> Vec<TraitSignal> {
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
    traits
}

fn collect_trait_group(
    traits: &mut Vec<TraitSignal>,
    group: &'static str,
    source: &BTreeMap<String, GhostlightTraitVector>,
) {
    for (name, vector) in source {
        let activation = vector.current_activation.clamp(0.0, 1.0);
        let plasticity = vector.plasticity.clamp(0.0, 1.0);
        traits.push(TraitSignal {
            group,
            name: name.clone(),
            activation,
            plasticity,
            weight: round3(activation * (0.65 + plasticity * 0.35)),
        });
    }
}

fn strongest_relationship_pressure(
    entry: &EpiphanyAgentMemoryEntry,
    thought_tokens: &BTreeSet<String>,
) -> (f64, String) {
    entry
        .agent
        .memories
        .relationship_summaries
        .iter()
        .map(|memory| {
            let overlap = token_overlap(&summary_tokens(&memory.summary), thought_tokens);
            let strength = overlap * (memory.salience * memory.confidence);
            (strength, memory.summary.clone())
        })
        .max_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(strength, summary)| (round3(strength.clamp(0.0, 1.0)), summary))
        .unwrap_or((
            0.0,
            "No strong relationship pressure is active.".to_string(),
        ))
}

fn strongest_overlay_pressure(
    entry: &EpiphanyAgentMemoryEntry,
    thought_tokens: &BTreeSet<String>,
) -> (f64, String) {
    entry
        .agent
        .perceived_state_overlays
        .iter()
        .map(|overlay| {
            let tokens = overlay_tokens(overlay);
            let overlap = token_overlap(&tokens, thought_tokens);
            let summary = overlay_summary(overlay);
            (overlap, summary)
        })
        .max_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(strength, summary)| (round3(strength.clamp(0.0, 1.0)), summary))
        .unwrap_or((
            0.0,
            "No perceived overlay is tugging on this stimulus.".to_string(),
        ))
}

fn stimulus_token_set(
    stimulus: &str,
    source: &str,
    mode: &str,
    status: &str,
    mood: &str,
) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    tokens.extend(summary_tokens(stimulus));
    tokens.extend(summary_tokens(source));
    tokens.extend(summary_tokens(mode));
    tokens.extend(summary_tokens(status));
    tokens.extend(summary_tokens(mood));
    tokens
}

fn perceived_overlay_projection(overlay: &GhostlightPerceivedStateOverlay) -> Value {
    serde_json::json!({
        "overlayId": overlay.overlay_id,
        "label": overlay.label,
        "summary": overlay.summary,
        "source": overlay.source,
        "salience": overlay.salience,
        "confidence": overlay.confidence,
        "linkedMemoryIds": overlay.linked_memory_ids,
    })
}

fn overlay_tokens(overlay: &GhostlightPerceivedStateOverlay) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    tokens.extend(summary_tokens(&overlay.overlay_id));
    tokens.extend(summary_tokens(&overlay.label));
    tokens.extend(summary_tokens(&overlay.summary));
    tokens.extend(summary_tokens(&overlay.source));
    for memory_id in &overlay.linked_memory_ids {
        tokens.extend(summary_tokens(memory_id));
    }
    tokens
}

fn overlay_summary(overlay: &GhostlightPerceivedStateOverlay) -> String {
    if !overlay.summary.trim().is_empty() {
        overlay.summary.clone()
    } else if !overlay.label.trim().is_empty() {
        overlay.label.clone()
    } else {
        "Perceived overlay relevance".to_string()
    }
}

fn summary_tokens(text: &str) -> BTreeSet<String> {
    text.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-')
        .filter_map(|item| {
            let token = item.trim().to_ascii_lowercase().replace('_', "-");
            (token.len() >= 3).then_some(token)
        })
        .collect()
}

fn token_overlap(left: &BTreeSet<String>, right: &BTreeSet<String>) -> f64 {
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let shared = left.intersection(right).count() as f64;
    let denom = left.len().min(right.len()) as f64;
    if denom <= f64::EPSILON {
        0.0
    } else {
        (shared / denom).clamp(0.0, 1.0)
    }
}

fn average<I>(values: I) -> Option<f64>
where
    I: Iterator<Item = f64>,
{
    let mut total = 0.0_f64;
    let mut count = 0_usize;
    for value in values {
        total += value;
        count += 1;
    }
    (count > 0).then_some(total / count as f64)
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

fn round3(value: f64) -> f64 {
    (value.clamp(0.0, 1.0) * 1000.0).round() / 1000.0
}

fn run_smoke() -> Result<Value> {
    let temp_dir = scoped_temp_dir("epiphany-character-loop-smoke")?;
    let artifact_dir = temp_dir.join("artifacts");
    let result = run_turn(
        "face",
        Path::new("state/agents.msgpack"),
        &artifact_dir,
        "Aquarium hover selected Face and asked what the swarm is feeling.",
        "smoke/character-loop",
        "public-surface",
        "ready",
        "attentive",
    )?;
    let path = PathBuf::from(
        result["turnPath"]
            .as_str()
            .ok_or_else(|| anyhow!("smoke result missing turnPath"))?,
    );
    let packet: Value = serde_json::from_str(
        &fs::read_to_string(&path)
            .with_context(|| format!("failed to read smoke artifact {}", path.display()))?,
    )?;
    let ok = packet["schema_version"] == CHARACTER_TURN_SCHEMA_VERSION
        && packet["protocol"]["bundle"] == "epiphany.character_loop"
        && packet["protocol"]["roleId"] == "face"
        && packet["protocol"]["dossierProfile"]["profileKind"] == "embodied_actor"
        && packet["projectedLocalContext"]["dossierProfile"]["profileKind"] == "embodied_actor"
        && packet["projectionSeed"]["schema_version"] == "epiphany.personality_projection.v0"
        && packet["appraisalSeed"]["schema_version"] == "epiphany.agent_thought_appraisal.v0"
        && packet["reactionSeed"]["schema_version"] == "epiphany.agent_reaction.v0"
        && packet["projectedLocalContext"]["identity"]["name"]
            .as_str()
            .is_some_and(|name| !name.trim().is_empty())
        && packet["stimulus"]["content"]
            .as_str()
            .is_some_and(|text| text.contains("Aquarium"))
        && packet["cognitionLanes"]["schema_version"] == "epiphany.cognition_lanes.v0"
        && packet["cognitionLanes"]["bridge"]["schema_version"] == "epiphany.cognition_bridge.v0"
        && packet["cognitionLanes"]["appraisal"]["schema_version"]
            == "epiphany.agent_thought_appraisal.v0";
    Ok(serde_json::json!({
        "ok": ok,
        "turnPath": path,
        "packet": {
            "schemaVersion": packet["schema_version"],
            "bundle": packet["protocol"]["bundle"],
            "roleId": packet["protocol"]["roleId"],
            "agentId": packet["protocol"]["agentId"],
            "dossierProfile": packet["protocol"]["dossierProfile"]["profileKind"],
            "projectionSchema": packet["projectionSeed"]["schema_version"],
            "appraisalSchema": packet["appraisalSeed"]["schema_version"],
            "reactionSchema": packet["reactionSeed"]["schema_version"],
            "identityName": packet["projectedLocalContext"]["identity"]["name"],
            "allowedOutputs": packet["allowedOutputs"],
            "cognitionLanes": packet["cognitionLanes"]["schema_version"],
        }
    }))
}

fn read_required_text(content: Option<String>, label: &str) -> Result<String> {
    let Some(content) = content else {
        let mut stdin = String::new();
        std::io::stdin()
            .read_to_string(&mut stdin)
            .context("failed to read stdin")?;
        ensure_content(&stdin, label)?;
        return Ok(stdin);
    };
    let path = PathBuf::from(&content);
    if path.exists() {
        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read content file {}", path.display()))?;
        ensure_content(&text, label)?;
        return Ok(text);
    }
    ensure_content(&content, label)?;
    Ok(content)
}

fn ensure_content(content: &str, label: &str) -> Result<()> {
    if content.trim().is_empty() {
        return Err(anyhow!("{label} is empty"));
    }
    Ok(())
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("missing value after {flag}"))
}

fn next_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(next_value(args, flag)?))
}

fn write_json(path: &Path, payload: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(payload)?),
    )
    .with_context(|| format!("failed to write {}", path.display()))
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn now_stamp() -> String {
    chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

fn short_id() -> String {
    Uuid::new_v4()
        .to_string()
        .split('-')
        .next()
        .unwrap_or("id")
        .to_string()
}

fn sanitize_file_stem(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn scoped_temp_dir(prefix: &str) -> Result<PathBuf> {
    let dir = env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    Ok(dir)
}

fn usage() -> Result<()> {
    Err(anyhow!(
        "usage: epiphany-character-loop <turn|smoke> [--role <role>] [--agent-store <path>] [--artifact-dir <path>] [--stimulus <text-or-path>] [--source <source>] [--mode <mode>]"
    ))
}
