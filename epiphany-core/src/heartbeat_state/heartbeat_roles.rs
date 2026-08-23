use super::EpiphanyHeartbeatStateEntry;
use super::HEARTBEAT_ARENA_MAINTENANCE;
use super::HEARTBEAT_STATE_SCHEMA_VERSION;
use super::HeartbeatPacingPolicy;
use super::HeartbeatParticipant;
use super::HeartbeatSelectionPolicy;
use super::PARTICIPANT_KIND_AGENT;
use super::write_heartbeat_state_entry;
use anyhow::Result;
use std::path::Path;

pub(super) const ROLE_ORDER: &[&str] = &["coordinator"];

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
        },
        pacing_policy: HeartbeatPacingPolicy {
            cooldown_starts_after_turn_completion: true,
            work_base_recovery: 6.0,
            idle_base_recovery: 2.0,
            sleep_heartbeat_rate_multiplier: 0.05,
            minimum_effective_rate: 0.001,
        },
        participants: ROLE_ORDER
            .iter()
            .map(|role_id| default_participant(role_id))
            .collect(),
        history: Vec::new(),
    }
}

pub fn initialize_heartbeat_store(
    store_path: impl AsRef<Path>,
    target_heartbeat_rate: f64,
) -> Result<EpiphanyHeartbeatStateEntry> {
    write_heartbeat_state_entry(store_path, &default_heartbeat_state(target_heartbeat_rate))
}

pub(super) fn default_participant(role_id: &str) -> HeartbeatParticipant {
    HeartbeatParticipant {
        agent_id: agent_id_for_role(role_id).to_string(),
        role_id: role_id.to_string(),
        display_name: display_name_for_role(role_id).to_string(),
        arena: HEARTBEAT_ARENA_MAINTENANCE.to_string(),
        participant_kind: PARTICIPANT_KIND_AGENT.to_string(),
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
    }
}

pub(super) fn agent_id_for_role(role_id: &str) -> &'static str {
    match role_id {
        "coordinator" => "epiphany.self",
        _ => "epiphany.unknown",
    }
}

pub(super) fn display_name_for_role(role_id: &str) -> &'static str {
    match role_id {
        "coordinator" => "Self",
        _ => "Unknown",
    }
}

fn initiative_speed_for_role(role_id: &str) -> f64 {
    match role_id {
        "coordinator" => 1.28,
        _ => 1.0,
    }
}

fn reaction_bias_for_role(role_id: &str) -> f64 {
    match role_id {
        "coordinator" => 0.88,
        _ => 0.5,
    }
}

fn interrupt_threshold_for_role(role_id: &str) -> f64 {
    match role_id {
        "coordinator" => 0.42,
        _ => 0.5,
    }
}

fn participant_constraints(role_id: &str) -> Vec<&'static str> {
    let role_specific = match role_id {
        "coordinator" => {
            "Routes and reviews; must not implement, verify, or accept its own comfort."
        }
        _ => "Unknown role.",
    };
    vec![role_specific]
}
