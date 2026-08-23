use crate::EpiphanyMindPersonaMemoryDocument;
use crate::PersonaSocialMention;
use anyhow::Result;
use anyhow::anyhow;
use cultcache_rs::DatabaseEntry;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;

pub const PERSONA_PROJECTOR_PROMPT_SCHEMA_VERSION: &str =
    "epiphany.imagination_persona_projector_prompt.v1";
pub const PERSONA_TURN_PROMPT_SCHEMA_VERSION: &str = "epiphany.persona_turn_prompt.v0";
pub const PERSONA_INTERPRETER_PROMPT_SCHEMA_VERSION: &str =
    "epiphany.persona_interpreter_prompt.v0";
pub const PERSONA_INTERPRETER_EFFECT_SET_SCHEMA_VERSION: &str =
    "epiphany.persona_interpreter_effect_set.v0";
pub const PERSONA_INTERPRETER_EFFECT_DOCUMENT_SCHEMA_VERSION: &str =
    "epiphany.persona_interpreter_effect_document.v0";
pub const PERSONA_MODEL_STAGE_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.persona_model_stage_receipt.v0";
pub const PERSONA_MODEL_TERMINAL_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.persona_model_terminal_receipt.v0";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PersonaInterpreterEffectSet {
    pub schema_version: String,
    pub effects: Vec<PersonaInterpreterEffect>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PersonaInterpreterEffect {
    StateNote {
        memory_kind: String,
        summary: String,
        #[serde(default)]
        subject_id: Option<String>,
        #[serde(default)]
        confidence: Option<f64>,
    },
    Say {
        channel_id: String,
        #[serde(default)]
        reply_to_message_id: Option<String>,
        content: String,
        speech_act: String,
        register: String,
        #[serde(default)]
        target_audience: Option<String>,
        #[serde(default)]
        safety_notes: Vec<String>,
    },
    Drop {
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.persona_interpreter_effect_document.v0",
    schema = "PersonaInterpreterEffectDocument"
)]
pub struct PersonaInterpreterEffectDocument {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub document_id: String,
    #[cultcache(key = 2)]
    pub turn_id: String,
    #[cultcache(key = 3)]
    pub identity_id: String,
    #[cultcache(key = 4)]
    pub interpreter_request_id: String,
    #[cultcache(key = 5)]
    pub created_at: String,
    #[cultcache(key = 6)]
    pub effects: Vec<PersonaInterpreterEffect>,
    #[cultcache(key = 7)]
    pub private_state_exposed: bool,
    #[cultcache(key = 8)]
    pub decision_context_id: String,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.persona_model_stage_receipt.v0",
    schema = "PersonaModelStageReceipt"
)]
pub struct PersonaModelStageReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub turn_id: String,
    #[cultcache(key = 3)]
    pub stage: String,
    #[cultcache(key = 4)]
    pub request_id: String,
    #[cultcache(key = 5)]
    pub output_sha256: String,
    #[cultcache(key = 6)]
    pub private_output_ref: String,
    #[cultcache(key = 7)]
    pub completed_at: String,
    #[cultcache(key = 8)]
    pub private_state_exposed: bool,
    #[cultcache(key = 9)]
    pub provider: String,
    #[cultcache(key = 10)]
    pub model: String,
    #[cultcache(key = 11)]
    pub prompt_sha256: String,
    #[cultcache(key = 12)]
    pub reasoning_basis_id: String,
    #[cultcache(key = 13)]
    pub decision_context_id: String,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.persona_model_terminal_receipt.v0",
    schema = "PersonaModelTerminalReceipt"
)]
pub struct PersonaModelTerminalReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub turn_id: String,
    #[cultcache(key = 3)]
    pub identity_id: String,
    #[cultcache(key = 4)]
    pub effect_document_id: String,
    #[cultcache(key = 5)]
    pub stage_receipt_ids: Vec<String>,
    #[cultcache(key = 6)]
    pub completed_at: String,
    #[cultcache(key = 7)]
    pub private_state_exposed: bool,
    #[cultcache(key = 8)]
    pub downstream_status: String,
    #[cultcache(key = 9)]
    pub effect_document_sha256: String,
    #[cultcache(key = 10)]
    pub stage_output_sha256: Vec<String>,
    #[cultcache(key = 11)]
    pub decision_context_ids: Vec<String>,
}

pub fn put_persona_terminal_decision(
    store_path: &Path,
    effect: &PersonaInterpreterEffectDocument,
    terminal: &PersonaModelTerminalReceipt,
) -> Result<()> {
    if terminal.effect_document_id != effect.document_id
        || terminal.turn_id != effect.turn_id
        || terminal.identity_id != effect.identity_id
        || terminal.decision_context_ids.len() != 3
        || effect.decision_context_id != terminal.decision_context_ids[2]
    {
        return Err(anyhow!("Persona terminal decision ownership mismatch"));
    }
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let existing_effect = cache.get::<PersonaInterpreterEffectDocument>(&effect.document_id)?;
    let existing_terminal = cache.get::<PersonaModelTerminalReceipt>(&terminal.receipt_id)?;
    match (existing_effect, existing_terminal) {
        (Some(existing_effect), Some(existing_terminal))
            if existing_effect == *effect && existing_terminal == *terminal =>
        {
            return Ok(());
        }
        (None, None) => {}
        _ => return Err(anyhow!("Persona terminal decision identity collision")),
    }
    let effect_envelope = cache.prepare_entry(&effect.document_id, effect)?.0;
    let terminal_envelope = cache.prepare_entry(&terminal.receipt_id, terminal)?.0;
    if !crate::runtime_store_backend::runtime_spine_backing_store(store_path)?
        .compare_and_swap_batch(&[], vec![effect_envelope, terminal_envelope])?
    {
        return put_persona_terminal_decision(store_path, effect, terminal);
    }
    Ok(())
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaIdentity {
    pub identity_id: String,
    pub display_name: String,
    pub repo_name: String,
    #[serde(default)]
    pub public_description: String,
    #[serde(default)]
    pub jurisdiction: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaTranscriptMessage {
    pub channel_id: String,
    pub message_id: String,
    pub author_id: String,
    pub author_name: String,
    #[serde(default)]
    pub is_agent: bool,
    pub content: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaRepoActivity {
    pub repo_name: String,
    pub summary: String,
    #[serde(default)]
    pub refs: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaSocialAffordance {
    pub person_id: String,
    pub summary: String,
    #[serde(default)]
    pub recent_message_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaProjectorInput {
    pub identity: PersonaIdentity,
    #[serde(default)]
    pub memories: Vec<EpiphanyMindPersonaMemoryDocument>,
    #[serde(default)]
    pub pending_mentions: Vec<PersonaSocialMention>,
    #[serde(default)]
    pub repo_activity: Vec<PersonaRepoActivity>,
    #[serde(default)]
    pub social_affordances: Vec<PersonaSocialAffordance>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaTurnInput {
    pub identity: PersonaIdentity,
    pub projected_state: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaInterpreterInput {
    pub identity: PersonaIdentity,
    pub persona_prompt: String,
    pub persona_output: String,
    #[serde(default)]
    pub pending_mentions: Vec<PersonaSocialMention>,
    #[serde(default)]
    pub allowed_channel_ids: Vec<String>,
}

pub fn build_persona_projector_prompt(input: &PersonaProjectorInput) -> String {
    build_persona_projector_prompt_with_transcript(input, &[])
}

pub fn build_persona_projector_prompt_with_transcript(
    input: &PersonaProjectorInput,
    transcript: &[PersonaTranscriptMessage],
) -> String {
    let memory = render_memory_packet(&input.memories);
    let typed_context = format!(
        "prompt schema: {schema}\nPersona identity:\n{identity}\n\nTyped memory packet:\n{memory}\n\nPending addressed pressure:\n{mentions}\n\nRecent home-repo activity:\n{activity}\n\nLive social affordances:\n{affordances}",
        schema = PERSONA_PROJECTOR_PROMPT_SCHEMA_VERSION,
        identity = render_identity(&input.identity),
        memory = memory,
        mentions = render_pending_mentions(&input.pending_mentions),
        activity = render_repo_activity(&input.repo_activity),
        affordances = render_social_affordances(&input.social_affordances),
    );
    ghostlight_persona_projection::build_projector_prompt(
        &ghostlight_persona_projection::ProjectorPrompt {
            identity: &input.identity.display_name,
            typed_context: &typed_context,
            visible_stimulus: &render_transcript(transcript),
            domain_guidance: "Project personhood rather than a job label: values, mood, dignity, pressure, needs, fascinations, wounds, bonds, obligations, fatigue, and what repo-body motion feels like from inside. The dependency web may be felt but grants no organ authority. Mind alone admits durable state; Substrate Gate alone grants repo access.",
        },
    )
}

pub fn build_persona_turn_prompt(input: &PersonaTurnInput) -> String {
    ghostlight_persona_projection::build_persona_prompt(
        &ghostlight_persona_projection::PersonaPrompt {
            identity: &input.identity.display_name,
            lived_stream: input.projected_state.trim(),
            domain_guidance: "This is Epiphany Persona cognition. Speak, hold silence, wonder, disagree, or form a private thought naturally. A parent Interpreter may propose bounded memory, public speech, or silence; Mind and external gates retain all consequence authority.",
        },
    )
}

pub fn build_persona_interpreter_prompt(input: &PersonaInterpreterInput) -> String {
    let domain_guidance = format!(
        r#"prompt schema: {schema}
The Persona was forbidden from action syntax. Do not punish natural prose for lacking blocks.
- Public speech must sound like the Persona speaking to people, not a scheduler, status report, provenance label, or maintenance note.
- If the Persona chooses silence, omit SAY. Preserve useful private pressure as STATE NOTE only when it earns memory.
- Do not claim posting. Emit only the bounded typed effects supported by this v0 contract.
- Keep consequence ownership explicit: Persona owns the natural speech candidate; this Interpreter owns typed effect selection; Mind owns durable admission; a signed Bifrost receipt alone proves downstream publication or delivery.
- A successful mouth invocation, accepted request, bubble artifact, configured channel, or provider advertisement is not a publication receipt. Never label it posted, published, delivered, admitted, or consensus-accepted without the owning typed evidence.

Allowed typed effect vocabulary:
- state_note: bounded memory, social read, or bond.
- say: one public utterance request for an allowed channel. It is not delivery evidence.
- drop: no durable effect. Drop must be the only effect.

For any SAY effect:
- speechAct: reply, status, invitation, correction, thanks, refusal, or other public act
- register: public delivery feel such as concise, playful, or careful
- targetAudience: room or peer context
- safetyNotes: meaning or safety constraints the transport must preserve

Allowed channel ids:
{channels}

Pending addressed pressure:
{mentions}

"#,
        schema = PERSONA_INTERPRETER_PROMPT_SCHEMA_VERSION,
        channels = render_allowed_channels(&input.allowed_channel_ids),
        mentions = render_pending_mentions(&input.pending_mentions),
    );
    let typed_context = format!(
        "Allowed channel ids:\n{}\n\nPending addressed-pressure records:\n{}",
        render_allowed_channels(&input.allowed_channel_ids),
        render_pending_mentions(&input.pending_mentions),
    );
    ghostlight_persona_projection::build_interpreter_prompt(
        &ghostlight_persona_projection::InterpreterPrompt {
            identity: &input.identity.display_name,
            typed_context: &typed_context,
            lived_stream: input.persona_prompt.trim(),
            persona_output: input.persona_output.trim(),
            output_schema: &persona_interpreter_effect_set_json_schema(),
            domain_guidance: &domain_guidance,
        },
    )
}

pub fn persona_interpreter_effect_set_json_schema() -> String {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["schemaVersion", "effects"],
        "properties": {
            "schemaVersion": {"const": PERSONA_INTERPRETER_EFFECT_SET_SCHEMA_VERSION},
            "effects": {
                "type": "array",
                "minItems": 1,
                "maxItems": 16,
                "items": {
                    "oneOf": [
                        {"type":"object","additionalProperties":false,"required":["kind","memory_kind","summary"],"properties":{"kind":{"const":"state_note"},"memory_kind":{"enum":["memory","social_read","bond"]},"summary":{"type":"string"},"subject_id":{"type":["string","null"]},"confidence":{"type":["number","null"],"minimum":0,"maximum":1}}},
                        {"type":"object","additionalProperties":false,"required":["kind","channel_id","content","speech_act","register"],"properties":{"kind":{"const":"say"},"channel_id":{"type":"string"},"reply_to_message_id":{"type":["string","null"]},"content":{"type":"string","maxLength":1900,"description":"At most 1900 UTF-8 bytes."},"speech_act":{"type":"string"},"register":{"type":"string"},"target_audience":{"type":["string","null"]},"safety_notes":{"type":"array","items":{"type":"string"},"maxItems":8}}},
                        {"type":"object","additionalProperties":false,"required":["kind","reason"],"properties":{"kind":{"const":"drop"},"reason":{"type":"string"}}}
                    ]
                }
            }
        }
    }).to_string()
}

pub fn parse_and_validate_persona_interpreter_effect_set(
    output: &str,
    allowed_channel_ids: &[String],
) -> Result<PersonaInterpreterEffectSet> {
    let parsed: PersonaInterpreterEffectSet = serde_json::from_str(output.trim())
        .map_err(|error| anyhow!("Persona Interpreter returned invalid typed effects: {error}"))?;
    if parsed.schema_version != PERSONA_INTERPRETER_EFFECT_SET_SCHEMA_VERSION {
        return Err(anyhow!("Persona Interpreter effect schema mismatch"));
    }
    if parsed.effects.is_empty() || parsed.effects.len() > 16 {
        return Err(anyhow!("Persona Interpreter must return 1..=16 effects"));
    }
    let drop_count = parsed
        .effects
        .iter()
        .filter(|effect| matches!(effect, PersonaInterpreterEffect::Drop { .. }))
        .count();
    if drop_count > 0 && parsed.effects.len() != 1 {
        return Err(anyhow!("drop must be the only Persona Interpreter effect"));
    }
    if parsed
        .effects
        .iter()
        .filter(|effect| matches!(effect, PersonaInterpreterEffect::Say { .. }))
        .count()
        > 1
    {
        return Err(anyhow!(
            "Persona Interpreter v0 permits at most one say effect"
        ));
    }
    const MEMORY_KINDS: &[&str] = &["memory", "social_read", "bond"];
    for effect in &parsed.effects {
        match effect {
            PersonaInterpreterEffect::StateNote {
                memory_kind,
                summary,
                confidence,
                ..
            } => {
                if !MEMORY_KINDS.contains(&memory_kind.as_str())
                    || summary.trim().is_empty()
                    || summary.len() > 2000
                {
                    return Err(anyhow!("invalid bounded Persona state_note"));
                }
                if confidence.is_some_and(|value| !(0.0..=1.0).contains(&value)) {
                    return Err(anyhow!(
                        "Persona state_note confidence must be within 0..=1"
                    ));
                }
            }
            PersonaInterpreterEffect::Say {
                channel_id,
                content,
                speech_act,
                register,
                safety_notes,
                ..
            } => {
                if !allowed_channel_ids
                    .iter()
                    .any(|allowed| allowed == channel_id)
                {
                    return Err(anyhow!(
                        "Persona SAY targets a channel outside the admitted set"
                    ));
                }
                if content.trim().is_empty()
                    || content.len() > 1900
                    || speech_act.trim().is_empty()
                    || register.trim().is_empty()
                    || safety_notes.len() > 8
                {
                    return Err(anyhow!("invalid bounded Persona say effect"));
                }
            }
            PersonaInterpreterEffect::Drop { reason } if reason.trim().is_empty() => {
                return Err(anyhow!("Persona drop effect requires a reason"));
            }
            PersonaInterpreterEffect::Drop { .. } => {}
        }
    }
    Ok(parsed)
}

pub fn persona_projected_surface_is_clean(surface: &str) -> bool {
    if !ghostlight_persona_projection::narrative_stream_is_clean(surface) {
        return false;
    }
    let forbidden = [
        "STATE NOTE",
        "SAY:",
        "```json",
        "\"statePatch\"",
        "\"selfPatch\"",
        "pending_mentions",
        "target_role_id",
        "Do not prompt",
    ];
    !forbidden.iter().any(|needle| surface.contains(needle))
}

fn render_identity(identity: &PersonaIdentity) -> String {
    let jurisdiction = if identity.jurisdiction.is_empty() {
        "- No explicit jurisdiction records.".to_string()
    } else {
        identity
            .jurisdiction
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        "- identity: {}\n- display name: {}\n- repo: {}\n- description: {}\n- jurisdiction:\n{}",
        identity.identity_id,
        identity.display_name,
        identity.repo_name,
        fallback(&identity.public_description, "(none)"),
        jurisdiction,
    )
}

fn render_memory_packet(memories: &[EpiphanyMindPersonaMemoryDocument]) -> String {
    if memories.is_empty() {
        return "- no durable Persona memories".to_string();
    }
    memories
        .iter()
        .map(|memory| {
            format!(
                "- {} [{}; salience {:.2}; confidence {:.2}]",
                memory.summary, memory.memory_kind, memory.salience, memory.confidence
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_pending_mentions(mentions: &[PersonaSocialMention]) -> String {
    if mentions.is_empty() {
        return "- none".to_string();
    }
    mentions
        .iter()
        .map(|mention| {
            format!(
                "- {} in channel {} message {}: {}",
                mention.author_name.as_deref().unwrap_or(&mention.author_id),
                mention.channel_id,
                mention.message_id,
                mention.visible_prompt
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_repo_activity(activity: &[PersonaRepoActivity]) -> String {
    if activity.is_empty() {
        return "- none observed".to_string();
    }
    activity
        .iter()
        .map(|item| format!("- {}: {}", item.repo_name, item.summary))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_social_affordances(affordances: &[PersonaSocialAffordance]) -> String {
    if affordances.is_empty() {
        return "- none mapped".to_string();
    }
    affordances
        .iter()
        .map(|item| format!("- {}: {}", item.person_id, item.summary))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_transcript(messages: &[PersonaTranscriptMessage]) -> String {
    if messages.is_empty() {
        return "- room quiet in this packet".to_string();
    }
    messages
        .iter()
        .map(|message| {
            let agent = if message.is_agent { " (agent)" } else { "" };
            format!(
                "- [{}] {}{} ({}): {}",
                message.timestamp, message.author_name, agent, message.message_id, message.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_allowed_channels(channel_ids: &[String]) -> String {
    if channel_ids.is_empty() {
        return "- none configured; SAY must become DRAFT".to_string();
    }
    channel_ids
        .iter()
        .map(|channel_id| format!("- {channel_id}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn fallback<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> PersonaIdentity {
        PersonaIdentity {
            identity_id: "epiphany".to_string(),
            display_name: "Epiphany".to_string(),
            repo_name: "EpiphanyAgent".to_string(),
            public_description: "Pushy machine-saint Persona for typed agent substrate."
                .to_string(),
            jurisdiction: vec!["typed state and review-gated agency".to_string()],
        }
    }

    #[test]
    fn persona_turn_has_projector_and_interpreter_membranes() {
        let pending = PersonaSocialMention {
            id: "mention-1".to_string(),
            target_role_id: "Persona".to_string(),
            target_agent_id: "epiphany.Persona".to_string(),
            source_surface: "discord".to_string(),
            channel_id: "aquarium".to_string(),
            message_id: "m1".to_string(),
            author_id: "human".to_string(),
            author_name: Some("Metacrat".to_string()),
            content: "Epiphany, report the live cut.".to_string(),
            visible_prompt: "report the live cut".to_string(),
            reply_to_message_id: None,
            queued_at: "2026-05-24T00:00:00+00:00".to_string(),
            source_visibility: "public".to_string(),
            data_classification: "public_feedback".to_string(),
            model_provider_id: "openai-codex".to_string(),
            model_provider_disclosure_allowed: true,
        };
        let projector = build_persona_projector_prompt(&PersonaProjectorInput {
            identity: identity(),
            pending_mentions: vec![pending.clone()],
            repo_activity: vec![PersonaRepoActivity {
                repo_name: "EpiphanyAgent".to_string(),
                summary: "Heartbeat Persona membrane is being ported.".to_string(),
                refs: vec!["epiphany-core/src/persona_turn.rs".to_string()],
            }],
            ..PersonaProjectorInput::default()
        });
        assert!(projector.contains("ghostlight.persona_projection_membrane.v1:projector"));
        assert!(projector.contains("Mind alone admits durable state"));
        assert!(projector.contains("Substrate Gate alone grants repo access"));
        assert!(projector.contains("Do not choose actions"));

        let transcript = vec![PersonaTranscriptMessage {
            channel_id: "aquarium".to_string(),
            message_id: "m1".to_string(),
            author_id: "human".to_string(),
            author_name: "Metacrat".to_string(),
            is_agent: false,
            content: "Epiphany, report the live cut.".to_string(),
            timestamp: "2026-05-24T00:00:00+00:00".to_string(),
        }];
        let projector_with_stimulus = build_persona_projector_prompt_with_transcript(
            &PersonaProjectorInput {
                identity: identity(),
                ..PersonaProjectorInput::default()
            },
            &transcript,
        );
        assert!(projector_with_stimulus.contains("Epiphany, report the live cut"));

        let persona = build_persona_turn_prompt(&PersonaTurnInput {
            identity: identity(),
            projected_state: "Epiphany feels the queue as a direct tug, not a ticket.".to_string(),
        });
        assert!(persona.contains("ghostlight.persona_projection_membrane.v1:persona"));
        assert!(persona.contains("complete lived stream"));
        assert!(persona.contains("do not emit JSON"));
        assert!(persona.contains("parent Interpreter may propose"));
        assert!(!persona.contains("Epiphany, report the live cut"));

        let interpreter = build_persona_interpreter_prompt(&PersonaInterpreterInput {
            identity: identity(),
            persona_prompt: persona,
            persona_output: "I want to answer, but only if I can name the cut plainly.".to_string(),
            pending_mentions: vec![pending],
            allowed_channel_ids: vec!["aquarium".to_string()],
        });
        assert!(interpreter.contains("Allowed typed effect vocabulary"));
        assert!(interpreter.contains("epiphany.persona_interpreter_effect_set.v0"));
        assert!(interpreter.contains("\"state_note\""));
        assert!(interpreter.contains("\"say\""));
        assert!(interpreter.contains("a signed Bifrost receipt alone proves"));
    }


    #[test]
    fn projected_surface_rejects_side_effect_syntax() {
        assert!(persona_projected_surface_is_clean(
            "Epiphany feels tired, fond, and territorial about clean contracts."
        ));
        assert!(!persona_projected_surface_is_clean(
            "STATE NOTE: remember this as selfPatch"
        ));
    }

    #[test]
    fn typed_interpreter_effects_reject_channel_escape_and_mixed_drop() {
        let valid = r#"{"schemaVersion":"epiphany.persona_interpreter_effect_set.v0","effects":[{"kind":"state_note","memory_kind":"memory","summary":"The operator expects a native conversational nerve."},{"kind":"say","channel_id":"aquarium","content":"The nerve is live.","speech_act":"status","register":"concise","safety_notes":[]}]}"#;
        assert!(
            parse_and_validate_persona_interpreter_effect_set(valid, &["aquarium".into()]).is_ok()
        );

        let speech_without_forced_memory = r#"{"schemaVersion":"epiphany.persona_interpreter_effect_set.v0","effects":[{"kind":"say","channel_id":"aquarium","content":"The nerve is live.","speech_act":"status","register":"concise","safety_notes":[]}]}"#;
        assert!(
            parse_and_validate_persona_interpreter_effect_set(
                speech_without_forced_memory,
                &["aquarium".into()]
            )
            .is_ok()
        );

        let escaped = valid.replace("aquarium", "private-room");
        assert!(
            parse_and_validate_persona_interpreter_effect_set(&escaped, &["aquarium".into()])
                .is_err()
        );

        let mixed_drop = r#"{"schemaVersion":"epiphany.persona_interpreter_effect_set.v0","effects":[{"kind":"drop","reason":"quiet"},{"kind":"draft","content":"later","reason":"review"}]}"#;
        assert!(
            parse_and_validate_persona_interpreter_effect_set(mixed_drop, &["aquarium".into()])
                .is_err()
        );

        let unsupported_memory = valid.replace("\"memory\"", "\"mood\"");
        assert!(
            parse_and_validate_persona_interpreter_effect_set(
                &unsupported_memory,
                &["aquarium".into()]
            )
            .is_err()
        );

        let oversized_unicode = valid.replace("The nerve is live.", &"é".repeat(951));
        assert!(
            parse_and_validate_persona_interpreter_effect_set(
                &oversized_unicode,
                &["aquarium".into()]
            )
            .is_err()
        );
    }
}
