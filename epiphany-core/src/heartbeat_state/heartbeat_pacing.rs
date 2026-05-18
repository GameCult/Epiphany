use super::EpiphanyHeartbeatStateEntry;
use super::HeartbeatInitiativeMultiplier;
use super::HeartbeatParticipant;
use super::HeartbeatPumpOptions;
use super::round3;
use super::round6;
use serde_json::Value;

#[derive(Clone, Debug)]
pub(super) struct AdaptiveSwarmPacing {
    pub(super) pressure: f64,
    pub(super) effective_heartbeat_rate: f64,
    pub(super) target_concurrency: usize,
    pub(super) running_turns: usize,
    pub(super) active_participants: usize,
    pub(super) signals: Value,
}

pub(super) fn adaptive_swarm_pacing(
    state: &EpiphanyHeartbeatStateEntry,
    options: &HeartbeatPumpOptions,
) -> AdaptiveSwarmPacing {
    let active_participants = state
        .participants
        .iter()
        .filter(|participant| participant.status == "active")
        .count();
    let running_turns = running_turn_count(state);
    let mood_signals = swarm_mood_signals(state);
    let external_urgency = options.external_urgency.clamp(0.0, 1.0);
    let pending_pressure = if active_participants == 0 {
        0.0
    } else {
        (running_turns as f64 / active_participants as f64).clamp(0.0, 1.0)
    };
    let pressure = [
        external_urgency,
        mood_signals.max_anxiety,
        mood_signals.max_urgency,
        mood_signals.max_reaction_intensity,
        mood_signals.max_thought_pressure,
        pending_pressure * 0.65,
    ]
    .into_iter()
    .fold(0.0_f64, f64::max)
    .clamp(0.0, 1.0);

    let base_rate = options
        .base_heartbeat_rate
        .max(state.pacing_policy.minimum_effective_rate)
        .max(0.001);
    let min_rate = options.min_heartbeat_rate.max(0.001).min(base_rate);
    let max_rate = options.max_heartbeat_rate.max(base_rate).max(min_rate);
    let pressure_curve = pressure * pressure;
    let effective_heartbeat_rate = round6(min_rate + (max_rate - min_rate) * pressure_curve);

    let max_concurrency = options
        .max_concurrency
        .max(1)
        .min(active_participants.max(1));
    let relaxed_floor = if pressure < 0.18 {
        0
    } else {
        options.min_concurrency.min(max_concurrency)
    };
    let target_concurrency = if pressure < 0.18 {
        relaxed_floor
    } else {
        let span = max_concurrency.saturating_sub(relaxed_floor);
        (relaxed_floor + (span as f64 * pressure).ceil() as usize).min(max_concurrency)
    };
    AdaptiveSwarmPacing {
        pressure: round3(pressure),
        effective_heartbeat_rate,
        target_concurrency,
        running_turns,
        active_participants,
        signals: serde_json::json!({
            "externalUrgency": round3(external_urgency),
            "maxAnxiety": round3(mood_signals.max_anxiety),
            "averageAnxiety": round3(mood_signals.average_anxiety),
            "maxUrgency": round3(mood_signals.max_urgency),
            "maxArousal": round3(mood_signals.max_arousal),
            "maxThoughtPressure": round3(mood_signals.max_thought_pressure),
            "maxReactionIntensity": round3(mood_signals.max_reaction_intensity),
            "pendingPressure": round3(pending_pressure),
            "contract": "Anxiety-like state raises tempo and concurrency; calm state lets the swarm sleep slow.",
        }),
    }
}

#[derive(Default)]
struct SwarmMoodSignals {
    max_anxiety: f64,
    average_anxiety: f64,
    max_urgency: f64,
    max_arousal: f64,
    max_thought_pressure: f64,
    max_reaction_intensity: f64,
}

fn swarm_mood_signals(state: &EpiphanyHeartbeatStateEntry) -> SwarmMoodSignals {
    let mut signals = SwarmMoodSignals::default();
    let mut anxiety_total = 0.0;
    let mut anxiety_count = 0_usize;
    for participant in &state.participants {
        let Some(mood) = participant.extra.get("moodTiming") else {
            continue;
        };
        let anxiety = number_at(mood, "/anxiety");
        let urgency = number_at(mood, "/urgency");
        let arousal = number_at(mood, "/arousal");
        let thought_pressure = number_at(mood, "/thoughtPressure");
        let reaction_intensity = number_at(mood, "/reactionIntensity");
        signals.max_anxiety = signals.max_anxiety.max(anxiety);
        signals.max_urgency = signals.max_urgency.max(urgency);
        signals.max_arousal = signals.max_arousal.max(arousal);
        signals.max_thought_pressure = signals.max_thought_pressure.max(thought_pressure);
        signals.max_reaction_intensity = signals.max_reaction_intensity.max(reaction_intensity);
        anxiety_total += anxiety;
        anxiety_count += 1;
    }
    if anxiety_count > 0 {
        signals.average_anxiety = anxiety_total / anxiety_count as f64;
    }
    signals
}

pub(super) fn running_turn_count(state: &EpiphanyHeartbeatStateEntry) -> usize {
    state
        .participants
        .iter()
        .filter(|participant| {
            participant
                .pending_turn
                .as_ref()
                .is_some_and(|turn| turn.status == "running")
        })
        .count()
}

fn number_at(value: &Value, pointer: &str) -> f64 {
    value
        .pointer(pointer)
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
}

pub(super) fn personality_cooldown_multiplier(participant: &HeartbeatParticipant) -> f64 {
    participant
        .extra
        .get("personalityCooldownMultiplier")
        .and_then(Value::as_f64)
        .unwrap_or(1.0)
        .clamp(0.25, 3.0)
}

pub(super) fn mood_cooldown_multiplier(participant: &HeartbeatParticipant) -> f64 {
    participant
        .extra
        .get("moodCooldownMultiplier")
        .and_then(Value::as_f64)
        .unwrap_or(1.0)
        .clamp(0.25, 3.0)
}

pub(super) fn effective_cooldown_multiplier(participant: &HeartbeatParticipant) -> f64 {
    round3(
        personality_cooldown_multiplier(participant) * mood_cooldown_multiplier(participant)
            / initiative_heat_multiplier(participant),
    )
    .clamp(0.05, 8.0)
}

pub(super) fn initiative_heat_multiplier(participant: &HeartbeatParticipant) -> f64 {
    participant
        .extra
        .get("initiativeHeatMultiplier")
        .and_then(Value::as_f64)
        .unwrap_or(1.0)
        .clamp(0.05, 25.0)
}

pub(super) fn apply_initiative_heat_policy(state: &mut EpiphanyHeartbeatStateEntry) {
    let global = state.initiative_heat.global_multiplier.clamp(0.05, 25.0);
    let active_multipliers = state
        .initiative_heat
        .multipliers
        .iter()
        .filter(|multiplier| {
            multiplier
                .expires_at_scene_clock
                .is_none_or(|expires| expires > state.scene_clock)
        })
        .cloned()
        .collect::<Vec<_>>();
    for participant in &mut state.participants {
        let mut heat = global;
        let mut basis = Vec::<Value>::new();
        for multiplier in &active_multipliers {
            if multiplier_matches_participant(multiplier, participant) {
                let value = multiplier.multiplier.clamp(0.05, 25.0);
                heat *= value;
                basis.push(serde_json::json!({
                    "id": multiplier.id,
                    "scope": multiplier.scope,
                    "selector": multiplier.selector,
                    "multiplier": value,
                    "reason": multiplier.reason,
                }));
            }
        }
        let heat = round3(heat.clamp(0.05, 25.0));
        participant.extra.insert(
            "initiativeHeatMultiplier".to_string(),
            serde_json::json!(heat),
        );
        participant.extra.insert(
            "initiativeHeat".to_string(),
            serde_json::json!({
                "schema_version": "epiphany.heartbeat_initiative_heat_projection.v0",
                "globalMultiplier": global,
                "effectiveMultiplier": heat,
                "basis": basis,
                "contract": "Initiative heat accelerates turn recovery. Proximity or agency pressure may raise heat, but the typed heat policy remains the authority."
            }),
        );
    }
}

fn multiplier_matches_participant(
    multiplier: &HeartbeatInitiativeMultiplier,
    participant: &HeartbeatParticipant,
) -> bool {
    let selector = multiplier.selector.trim();
    match multiplier.scope.as_str() {
        "all" => true,
        "agent" | "agent_id" => eq_key(&participant.agent_id, selector),
        "role" | "role_id" => eq_key(&participant.role_id, selector),
        "arena" => eq_key(participant.arena.as_str(), selector),
        "participant_kind" | "kind" => eq_key(participant.participant_kind.as_str(), selector),
        "group" | "constraint" => {
            participant
                .constraints
                .iter()
                .any(|constraint| eq_key(constraint, selector))
                || participant
                    .extra
                    .get("groups")
                    .and_then(Value::as_array)
                    .is_some_and(|groups| {
                        groups
                            .iter()
                            .filter_map(Value::as_str)
                            .any(|group| eq_key(group, selector))
                    })
        }
        _ => false,
    }
}

fn eq_key(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heartbeat_state::default_heartbeat_state;
    use pretty_assertions::assert_eq;

    #[test]
    fn running_turn_count_reads_only_running_pending_turns() {
        let mut state = default_heartbeat_state(1.0);
        state.participants[0].pending_turn = Some(crate::heartbeat_state::HeartbeatPendingTurn {
            status: "running".to_string(),
            ..Default::default()
        });
        state.participants[1].pending_turn = Some(crate::heartbeat_state::HeartbeatPendingTurn {
            status: "completed".to_string(),
            ..Default::default()
        });

        assert_eq!(running_turn_count(&state), 1);
    }

    #[test]
    fn initiative_heat_multiplies_agent_and_group_recovery_speed() {
        let mut state = default_heartbeat_state(1.0);
        state.initiative_heat.global_multiplier = 2.0;
        state.initiative_heat.multipliers = vec![
            HeartbeatInitiativeMultiplier {
                id: "hands".to_string(),
                scope: "role".to_string(),
                selector: "implementation".to_string(),
                multiplier: 3.0,
                ..Default::default()
            },
            HeartbeatInitiativeMultiplier {
                id: "all-maintenance".to_string(),
                scope: "arena".to_string(),
                selector: "maintenance".to_string(),
                multiplier: 1.5,
                ..Default::default()
            },
        ];

        apply_initiative_heat_policy(&mut state);
        let implementation = state
            .participants
            .iter()
            .find(|participant| participant.role_id == "implementation")
            .expect("implementation participant");
        let research = state
            .participants
            .iter()
            .find(|participant| participant.role_id == "research")
            .expect("research participant");

        assert_eq!(initiative_heat_multiplier(implementation), 9.0);
        assert_eq!(initiative_heat_multiplier(research), 3.0);
        assert!(
            effective_cooldown_multiplier(implementation) < effective_cooldown_multiplier(research)
        );
    }
}
