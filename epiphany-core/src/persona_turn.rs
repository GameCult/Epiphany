use crate::EpiphanyAgentMemoryEntry;
use crate::EpiphanyOrganDependency;
use crate::HeartbeatPendingMention;
use crate::default_organ_dependencies_for;
use crate::render_organ_dependencies;
use serde::Deserialize;
use serde::Serialize;

pub const PERSONA_PROJECTOR_PROMPT_SCHEMA_VERSION: &str =
    "epiphany.imagination_persona_projector_prompt.v0";
pub const PERSONA_TURN_PROMPT_SCHEMA_VERSION: &str = "epiphany.persona_turn_prompt.v0";
pub const PERSONA_INTERPRETER_PROMPT_SCHEMA_VERSION: &str =
    "epiphany.persona_interpreter_prompt.v0";

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
    pub memory: Option<EpiphanyAgentMemoryEntry>,
    #[serde(default)]
    pub pending_mentions: Vec<HeartbeatPendingMention>,
    #[serde(default)]
    pub repo_activity: Vec<PersonaRepoActivity>,
    #[serde(default)]
    pub social_affordances: Vec<PersonaSocialAffordance>,
    #[serde(default)]
    pub organ_dependencies: Vec<EpiphanyOrganDependency>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaTurnInput {
    pub identity: PersonaIdentity,
    pub projected_state: String,
    #[serde(default)]
    pub pending_mentions: Vec<HeartbeatPendingMention>,
    #[serde(default)]
    pub repo_activity: Vec<PersonaRepoActivity>,
    #[serde(default)]
    pub social_affordances: Vec<PersonaSocialAffordance>,
    #[serde(default)]
    pub transcript: Vec<PersonaTranscriptMessage>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaInterpreterInput {
    pub identity: PersonaIdentity,
    pub persona_prompt: String,
    pub persona_output: String,
    #[serde(default)]
    pub pending_mentions: Vec<HeartbeatPendingMention>,
    #[serde(default)]
    pub allowed_channel_ids: Vec<String>,
}

pub fn build_persona_projector_prompt(input: &PersonaProjectorInput) -> String {
    let memory = input
        .memory
        .as_ref()
        .map(render_memory_packet)
        .unwrap_or_else(|| "No durable Persona memory entry is loaded.".to_string());
    let dependencies = if input.organ_dependencies.is_empty() {
        vec![default_organ_dependencies_for("Persona")]
    } else {
        input.organ_dependencies.clone()
    };
    format!(
        r#"<!-- prompt:{schema} -->
You are Imagination acting as the Persona Projector for {name}.

You are not the Persona. You are not Mind. You are the membrane that turns typed role memory, pending social pressure, repo-body activity, relationship affordances, and organ dependencies into lived narrative context.

Hard boundary:
- Do not choose public speech.
- Do not decide durable state; Mind is the Interpreter and state guardian after Persona thinks.
- Do not decide repo access; Substrate Gate grants substrate access before repo facts enter this packet.
- Do not emit action blocks, JSON, state patches, SAY blocks, drafts, or Discord instructions.
- Do not summarize the Persona as a job label. Project personhood: values, mood, dignity, pressure, needs, fascinations, wounds, bonds, obligations, fatigue, and what the repo-body motion feels like from inside this Persona.
- Project the dependency web. Every sub-agent depends on all the other sub-agents; Persona's scene should feel the pressure of Self, Imagination, Eyes, Modeling, Hands, Soul, and Continuity protocols without pretending Persona owns them.
- If the state is sparse, say what is sparse without inventing history.

Persona identity:
{identity}

Typed memory packet:
{memory}

Pending addressed pressure:
{mentions}

Recent home-repo activity:
{activity}

Live social affordances:
{affordances}

Organ dependency contract:
{dependencies}

Return only narrative context for the Persona to inhabit.
"#,
        schema = PERSONA_PROJECTOR_PROMPT_SCHEMA_VERSION,
        name = input.identity.display_name,
        identity = render_identity(&input.identity),
        memory = memory,
        mentions = render_pending_mentions(&input.pending_mentions),
        activity = render_repo_activity(&input.repo_activity),
        affordances = render_social_affordances(&input.social_affordances),
        dependencies = render_organ_dependencies(&dependencies),
    )
}

pub fn build_persona_turn_prompt(input: &PersonaTurnInput) -> String {
    format!(
        r#"<!-- prompt:{schema} -->
You are {name}, the Persona of {repo}.

Think narratively. Speak, hold silence, wonder, disagree, or form a private thought as yourself.

Hard boundary:
- Do not emit JSON, tool calls, SAY blocks, STATE NOTE blocks, action blocks, or Discord routing syntax.
- You may describe what you want to say or remember in natural language.
- Your side effects are not yours to execute. A parent Interpreter will decide whether your natural turn becomes memory, a draft, public speech, a proposal, or silence.
- Read the raw transcript directly. Recent human correction beats stale memory.

Projected inner state from Imagination:
{projected}

Recent home-repo activity, before room pressure:
{activity}

Pending addressed pressure:
{mentions}

Live social affordances:
{affordances}

Raw room transcript, oldest to newest:
{transcript}

Write one natural Persona turn.
"#,
        schema = PERSONA_TURN_PROMPT_SCHEMA_VERSION,
        name = input.identity.display_name,
        repo = input.identity.repo_name,
        projected = input.projected_state.trim(),
        activity = render_repo_activity(&input.repo_activity),
        mentions = render_pending_mentions(&input.pending_mentions),
        affordances = render_social_affordances(&input.social_affordances),
        transcript = render_transcript(&input.transcript),
    )
}

pub fn build_persona_interpreter_prompt(input: &PersonaInterpreterInput) -> String {
    format!(
        r#"<!-- prompt:{schema} -->
You are the parent Persona Interpreter for {name}.

You are not the Persona. You own the boundary between natural narrative thought and durable side effects.

Hard boundary:
- The Persona was forbidden from action syntax. Do not punish natural prose for lacking blocks.
- Decide side effects from the Persona output plus the original prompt evidence.
- Public speech must sound like the Persona speaking to people, not a scheduler, status report, provenance label, or maintenance note.
- If the Persona chooses silence, route without SAY. Preserve useful private pressure as STATE NOTE only when it earns memory.
- Do not auto-post. Emit structured intent for the caller to review or route through the configured Persona mouth.

Allowed effect vocabulary:
- STATE NOTE: bounded Persona memory, mood, need, social read, bond, value, goal, or agency pressure.
- SAY: public text candidate for an allowed channel.
- DRAFT: private candidate artifact when posting is blocked or needs review.
- ROUTE: non-public action such as keep private, ask Self, or propose a bounded repo investigation.
- DROP: no durable effect.

Allowed channel ids:
{channels}

Pending addressed pressure:
{mentions}

Original Persona prompt:
```
{persona_prompt}
```

Persona output:
```
{persona_output}
```

Return concise structured effect blocks. No prose outside the blocks.
"#,
        schema = PERSONA_INTERPRETER_PROMPT_SCHEMA_VERSION,
        name = input.identity.display_name,
        channels = render_allowed_channels(&input.allowed_channel_ids),
        mentions = render_pending_mentions(&input.pending_mentions),
        persona_prompt = input.persona_prompt.trim(),
        persona_output = input.persona_output.trim(),
    )
}

pub fn persona_projected_surface_is_clean(surface: &str) -> bool {
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

fn render_memory_packet(memory: &EpiphanyAgentMemoryEntry) -> String {
    let values = memory
        .agent
        .canonical_state
        .values
        .iter()
        .map(|value| format!("- {} ({:.2})", value.label, value.priority))
        .collect::<Vec<_>>()
        .join("\n");
    let goals = memory
        .agent
        .goals
        .iter()
        .map(|goal| format!("- {} [{}]", goal.description, goal.status))
        .collect::<Vec<_>>()
        .join("\n");
    let notes = memory
        .agent
        .identity
        .private_notes
        .iter()
        .map(|note| format!("- {note}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Identity: {}\nPublic description: {}\nPrivate notes:\n{}\nValues:\n{}\nGoals:\n{}",
        memory.agent.identity.name,
        memory.agent.identity.public_description,
        fallback(&notes, "- none"),
        fallback(&values, "- none"),
        fallback(&goals, "- none"),
    )
}

fn render_pending_mentions(mentions: &[HeartbeatPendingMention]) -> String {
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
        let pending = HeartbeatPendingMention {
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
        assert!(projector.contains("You are Imagination acting as the Persona Projector"));
        assert!(projector.contains("You are not the Persona"));
        assert!(projector.contains("Mind is the Interpreter"));
        assert!(projector.contains("Substrate Gate grants substrate access"));
        assert!(projector.contains("Every sub-agent depends on all the other sub-agents"));
        assert!(projector.contains("Continuity protocols"));
        assert!(projector.contains("Do not choose public speech"));

        let persona = build_persona_turn_prompt(&PersonaTurnInput {
            identity: identity(),
            projected_state: "Epiphany feels the queue as a direct tug, not a ticket.".to_string(),
            pending_mentions: vec![pending.clone()],
            transcript: vec![PersonaTranscriptMessage {
                channel_id: "aquarium".to_string(),
                message_id: "m1".to_string(),
                author_id: "human".to_string(),
                author_name: "Metacrat".to_string(),
                is_agent: false,
                content: "Epiphany, report the live cut.".to_string(),
                timestamp: "2026-05-24T00:00:00+00:00".to_string(),
            }],
            ..PersonaTurnInput::default()
        });
        assert!(persona.contains("Think narratively"));
        assert!(persona.contains("Projected inner state from Imagination"));
        assert!(persona.contains("Do not emit JSON"));
        assert!(persona.contains("A parent Interpreter will decide"));

        let interpreter = build_persona_interpreter_prompt(&PersonaInterpreterInput {
            identity: identity(),
            persona_prompt: persona,
            persona_output: "I want to answer, but only if I can name the cut plainly.".to_string(),
            pending_mentions: vec![pending],
            allowed_channel_ids: vec!["aquarium".to_string()],
        });
        assert!(interpreter.contains("Allowed effect vocabulary"));
        assert!(interpreter.contains("STATE NOTE"));
        assert!(interpreter.contains("SAY"));
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
}
