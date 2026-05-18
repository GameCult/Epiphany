use super::EpiphanyHeartbeatStateEntry;
use super::GhostlightSceneParticipantSeed;
use super::HEARTBEAT_ARENA_MAINTENANCE;
use super::HEARTBEAT_ARENA_SCENE;
use super::HEARTBEAT_STATE_SCHEMA_VERSION;
use super::HeartbeatPacingPolicy;
use super::HeartbeatParticipant;
use super::HeartbeatSelectionPolicy;
use super::PARTICIPANT_KIND_AGENT;
use super::PARTICIPANT_KIND_CHARACTER;
use super::write_heartbeat_state_entry;
use anyhow::Result;
use anyhow::anyhow;
use std::collections::BTreeMap;
use std::path::Path;

pub(super) const ROLE_ORDER: &[&str] = &[
    "coordinator",
    "face",
    "imagination",
    "research",
    "modeling",
    "implementation",
    "verification",
    "reorientation",
];

pub fn default_heartbeat_state(target_heartbeat_rate: f64) -> EpiphanyHeartbeatStateEntry {
    EpiphanyHeartbeatStateEntry {
        schema_version: HEARTBEAT_STATE_SCHEMA_VERSION.to_string(),
        target_heartbeat_rate,
        scene_clock: 0.0,
        selection_policy: HeartbeatSelectionPolicy {
            mode: "readiness_queue".to_string(),
            reaction_precedence: true,
            minimum_speed: 0.2,
            tie_breakers: vec![
                "reaction_readiness_desc".to_string(),
                "next_ready_at_asc".to_string(),
                "initiative_speed_desc".to_string(),
                "stable_actor_id_asc".to_string(),
            ],
            extra: BTreeMap::new(),
        },
        pacing_policy: HeartbeatPacingPolicy {
            cooldown_starts_after_turn_completion: true,
            work_base_recovery: 6.0,
            idle_base_recovery: 2.0,
            sleep_heartbeat_rate_multiplier: 0.05,
            minimum_effective_rate: 0.001,
            extra: BTreeMap::new(),
        },
        participants: ROLE_ORDER
            .iter()
            .map(|role_id| default_participant(role_id))
            .collect(),
        history: Vec::new(),
        adaptive_pacing: None,
        extra: BTreeMap::new(),
    }
}

pub fn initialize_heartbeat_store(
    store_path: impl AsRef<Path>,
    target_heartbeat_rate: f64,
) -> Result<EpiphanyHeartbeatStateEntry> {
    write_heartbeat_state_entry(store_path, &default_heartbeat_state(target_heartbeat_rate))
}

pub fn ghostlight_scene_heartbeat_state(
    target_heartbeat_rate: f64,
    scene_id: impl Into<String>,
    participants: Vec<GhostlightSceneParticipantSeed>,
) -> Result<EpiphanyHeartbeatStateEntry> {
    if participants.is_empty() {
        return Err(anyhow!(
            "Ghostlight scene heartbeat requires at least one participant"
        ));
    }
    let scene_id = scene_id.into();
    let mut state = default_heartbeat_state(target_heartbeat_rate);
    state.participants = participants
        .into_iter()
        .map(|seed| ghostlight_scene_participant(&scene_id, seed))
        .collect();
    state.extra.insert(
        "protocol".to_string(),
        serde_json::json!({
            "domain": "ghostlight",
            "sceneId": scene_id,
            "arena": HEARTBEAT_ARENA_SCENE,
            "contract": "Characters and maintenance organs use one initiative timing law; scene participants receive only projected local context.",
        }),
    );
    super::validate_heartbeat_state(&state)?;
    Ok(state)
}

pub fn initialize_ghostlight_scene_heartbeat_store(
    store_path: impl AsRef<Path>,
    target_heartbeat_rate: f64,
    scene_id: impl Into<String>,
    participants: Vec<GhostlightSceneParticipantSeed>,
) -> Result<EpiphanyHeartbeatStateEntry> {
    let state = ghostlight_scene_heartbeat_state(target_heartbeat_rate, scene_id, participants)?;
    write_heartbeat_state_entry(store_path, &state)
}

pub(super) fn default_participant(role_id: &str) -> HeartbeatParticipant {
    HeartbeatParticipant {
        agent_id: agent_id_for_role(role_id).to_string(),
        role_id: role_id.to_string(),
        display_name: display_name_for_role(role_id).to_string(),
        arena: HEARTBEAT_ARENA_MAINTENANCE.to_string(),
        participant_kind: PARTICIPANT_KIND_AGENT.to_string(),
        scene_id: None,
        initiative_speed: initiative_speed_for_role(role_id),
        next_ready_at: 0.0,
        reaction_bias: reaction_bias_for_role(role_id),
        interrupt_threshold: interrupt_threshold_for_role(role_id),
        current_load: 0.0,
        status: "active".to_string(),
        constraints: participant_constraints(role_id)
            .into_iter()
            .map(str::to_string)
            .collect(),
        last_action_id: None,
        last_woke_at: None,
        last_finished_at: None,
        pending_turn: None,
        personality_timing: None,
        mood_timing: None,
        extra: BTreeMap::new(),
    }
}

fn ghostlight_scene_participant(
    scene_id: &str,
    seed: GhostlightSceneParticipantSeed,
) -> HeartbeatParticipant {
    let role_id = ghostlight_role_id(seed.agent_id.as_str());
    HeartbeatParticipant {
        agent_id: seed.agent_id,
        role_id,
        display_name: seed.display_name,
        arena: HEARTBEAT_ARENA_SCENE.to_string(),
        participant_kind: PARTICIPANT_KIND_CHARACTER.to_string(),
        scene_id: Some(scene_id.to_string()),
        initiative_speed: seed.initiative_speed,
        next_ready_at: 0.0,
        reaction_bias: seed.reaction_bias,
        interrupt_threshold: seed.interrupt_threshold,
        current_load: 0.0,
        status: "active".to_string(),
        constraints: seed.constraints,
        last_action_id: None,
        last_woke_at: None,
        last_finished_at: None,
        pending_turn: None,
        personality_timing: None,
        mood_timing: None,
        extra: BTreeMap::new(),
    }
}

fn ghostlight_role_id(agent_id: &str) -> String {
    let sanitized = agent_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    format!("ghostlight.character.{sanitized}")
}

pub(super) fn agent_id_for_role(role_id: &str) -> &'static str {
    match role_id {
        "coordinator" => "epiphany.self",
        "face" => "epiphany.face",
        "imagination" => "epiphany.imagination",
        "research" => "epiphany.eyes",
        "modeling" => "epiphany.body",
        "implementation" => "epiphany.hands",
        "verification" => "epiphany.soul",
        "reorientation" => "epiphany.life",
        _ => "epiphany.unknown",
    }
}

pub(super) fn display_name_for_role(role_id: &str) -> &'static str {
    match role_id {
        "coordinator" => "Self",
        "face" => "Face",
        "imagination" => "Imagination",
        "research" => "Eyes",
        "modeling" => "Body",
        "implementation" => "Hands",
        "verification" => "Soul",
        "reorientation" => "Life",
        _ => "Unknown",
    }
}

fn initiative_speed_for_role(role_id: &str) -> f64 {
    match role_id {
        "coordinator" => 1.28,
        "face" => 1.12,
        "imagination" => 0.82,
        "research" => 0.78,
        "modeling" => 0.92,
        "implementation" => 0.74,
        "verification" => 0.88,
        "reorientation" => 1.04,
        _ => 1.0,
    }
}

fn reaction_bias_for_role(role_id: &str) -> f64 {
    match role_id {
        "coordinator" => 0.88,
        "face" => 0.84,
        "imagination" => 0.54,
        "research" => 0.62,
        "modeling" => 0.74,
        "implementation" => 0.58,
        "verification" => 0.82,
        "reorientation" => 0.86,
        _ => 0.5,
    }
}

fn interrupt_threshold_for_role(role_id: &str) -> f64 {
    match role_id {
        "coordinator" => 0.42,
        "face" => 0.52,
        "imagination" => 0.64,
        "research" => 0.58,
        "modeling" => 0.5,
        "implementation" => 0.5,
        "verification" => 0.48,
        "reorientation" => 0.44,
        _ => 0.5,
    }
}

fn participant_constraints(role_id: &str) -> Vec<&'static str> {
    let role_specific = match role_id {
        "coordinator" => {
            "Routes and reviews; must not implement, verify, or accept its own comfort."
        }
        "face" => {
            "Publicly translates agent thought into #aquarium only; must not moderate or speak outside the room."
        }
        "imagination" => "Synthesizes futures; must not adopt objectives.",
        "research" => "Scouts known work; must not turn research into procrastination.",
        "modeling" => {
            "Grows source-grounded maps and checkpoints; must not edit implementation code."
        }
        "implementation" => {
            "Touches source only with accepted guidance and verifier-readable evidence."
        }
        "verification" => "Falsifies promises; must not bless theater.",
        "reorientation" => "Protects continuity; must not fake survived context.",
        _ => "Unknown role.",
    };
    vec![
        "Runs Ghostlight-shaped persistent role memory.",
        "May improve lane memory when awake and idle.",
        "Project truth belongs in EpiphanyThreadState, not role memory.",
        role_specific,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn default_state_defines_the_fixed_lane_catalog() {
        let state = default_heartbeat_state(1.5);

        assert_eq!(state.target_heartbeat_rate, 1.5);
        assert_eq!(state.pacing_policy.work_base_recovery, 6.0);
        assert_eq!(state.pacing_policy.idle_base_recovery, 2.0);
        assert_eq!(
            state
                .participants
                .iter()
                .map(|participant| participant.role_id.as_str())
                .collect::<Vec<_>>(),
            ROLE_ORDER
        );
        assert_eq!(
            state
                .participants
                .iter()
                .find(|participant| participant.role_id == "face")
                .map(|participant| participant.agent_id.as_str()),
            Some("epiphany.face")
        );
    }

    #[test]
    fn ghostlight_scene_state_replaces_maintenance_lanes_with_scene_characters() -> Result<()> {
        let state = ghostlight_scene_heartbeat_state(
            0.5,
            "room",
            vec![GhostlightSceneParticipantSeed {
                agent_id: "Ariadne Prime".to_string(),
                display_name: "Ariadne".to_string(),
                initiative_speed: 1.2,
                reaction_bias: 0.8,
                interrupt_threshold: 0.4,
                constraints: vec!["Speak only in the room.".to_string()],
            }],
        )?;

        assert_eq!(state.participants.len(), 1);
        let participant = &state.participants[0];
        assert_eq!(participant.role_id, "ghostlight.character.ariadne-prime");
        assert_eq!(participant.arena, HEARTBEAT_ARENA_SCENE);
        assert_eq!(participant.participant_kind, PARTICIPANT_KIND_CHARACTER);
        assert_eq!(
            state
                .extra
                .get("protocol")
                .and_then(|value| value.get("domain"))
                .and_then(|value| value.as_str()),
            Some("ghostlight")
        );
        Ok(())
    }
}
