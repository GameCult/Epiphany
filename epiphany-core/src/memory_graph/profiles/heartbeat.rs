use crate::heartbeat_state::EpiphanyHeartbeatCognitionEntry;
#[cfg(test)]
use crate::heartbeat_state::HeartbeatBridgeDecision;
#[cfg(test)]
use crate::heartbeat_state::HeartbeatBridgeSynthesis;
#[cfg(test)]
use crate::heartbeat_state::HeartbeatBridgeTension;
#[cfg(test)]
use crate::heartbeat_state::HeartbeatCandidateIntervention;
#[cfg(test)]
use crate::heartbeat_state::HeartbeatCandidateInterventions;
#[cfg(test)]
use crate::heartbeat_state::HeartbeatCognitionBridge;
#[cfg(test)]
use crate::heartbeat_state::HeartbeatIncubation;
#[cfg(test)]
use crate::heartbeat_state::HeartbeatIncubationTheme;
#[cfg(test)]
use crate::heartbeat_state::HeartbeatMemoryResonance;
#[cfg(test)]
use crate::heartbeat_state::HeartbeatMemoryResonancePair;
#[cfg(test)]
use crate::heartbeat_state::HeartbeatSourceCoverage;
use crate::memory_graph::EpiphanyMemoryDomain;
use crate::memory_graph::EpiphanyMemoryGraphSnapshot;
use crate::memory_graph::EpiphanyMemoryLifecycle;
use crate::memory_graph::EpiphanyMemoryNode;
use crate::memory_graph::EpiphanyMemoryNodeKind;
use crate::memory_graph::EpiphanyMemoryProfile;
use crate::memory_graph::EpiphanyMemorySummary;
use crate::memory_graph::memory_graph_domain_id;
use crate::memory_graph::memory_graph_node_id;

pub fn memory_graph_from_heartbeat_cognition(
    graph_id: impl Into<String>,
    cognition: &EpiphanyHeartbeatCognitionEntry,
) -> EpiphanyMemoryGraphSnapshot {
    let mut domains = Vec::new();
    let mut nodes = Vec::new();
    let mut summaries = Vec::new();

    import_resonance(cognition, &mut domains, &mut nodes, &mut summaries);
    import_incubation(cognition, &mut domains, &mut nodes, &mut summaries);
    import_candidate_interventions(cognition, &mut domains, &mut nodes, &mut summaries);
    import_bridge_pressure(cognition, &mut domains, &mut nodes, &mut summaries);

    EpiphanyMemoryGraphSnapshot {
        schema_version: Some("epiphany.memory_graph.v0".to_string()),
        graph_id: graph_id.into(),
        domains,
        nodes,
        summaries,
        freshness: Some(crate::memory_graph::EpiphanyMemoryFreshness {
            status: crate::memory_graph::EpiphanyMemoryFreshnessStatus::Ready,
            note: Some(
                "Imported from heartbeat cognition as provisional memory pressure.".to_string(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn import_resonance(
    cognition: &EpiphanyHeartbeatCognitionEntry,
    domains: &mut Vec<EpiphanyMemoryDomain>,
    nodes: &mut Vec<EpiphanyMemoryNode>,
    summaries: &mut Vec<EpiphanyMemorySummary>,
) {
    let Some(resonance) = cognition.memory_resonance.as_ref() else {
        return;
    };
    let domain_id = push_domain(
        domains,
        EpiphanyMemoryProfile::ShortTerm,
        "heartbeat resonance",
    );
    let before = nodes.len();
    for pair in resonance.pairs.iter().take(8) {
        let left = pair.left_summary.clone();
        let right = pair.right_summary.clone();
        let left_role = pair.left_role.clone();
        let right_role = pair.right_role.clone();
        let id_seed = format!("{}:{}:{}", left_role, right_role, pair.left_memory_id);
        nodes.push(EpiphanyMemoryNode {
            id: memory_graph_node_id(&domain_id, "resonance", id_seed, None),
            domain_id: domain_id.clone(),
            profile: EpiphanyMemoryProfile::ShortTerm,
            kind: EpiphanyMemoryNodeKind::ShortTermThought,
            title: format!("{left_role}/{right_role} resonance"),
            claim: format!("{left} / {right}"),
            question: "Is this resonance signal, stale echo, or a branch worth following?"
                .to_string(),
            tension: "Shared vocabulary is not proof of shared truth.".to_string(),
            action_implication:
                "Treat as provisional attention pressure until sleep or review accounts for it."
                    .to_string(),
            source_hashes: vec!["anchor:missing".to_string()],
            lifecycle: EpiphanyMemoryLifecycle::Active,
            salience: score_to_u32(pair.strength),
            confidence: 55,
            ..Default::default()
        });
    }
    push_summary(
        summaries,
        &domain_id,
        "summary-heartbeat-resonance",
        "heartbeat resonance",
        "Heartbeat resonance produced provisional short-term thought nodes.",
        &nodes[before..],
        EpiphanyMemoryProfile::ShortTerm,
    );
}

fn import_incubation(
    cognition: &EpiphanyHeartbeatCognitionEntry,
    domains: &mut Vec<EpiphanyMemoryDomain>,
    nodes: &mut Vec<EpiphanyMemoryNode>,
    summaries: &mut Vec<EpiphanyMemorySummary>,
) {
    let Some(incubation) = cognition.incubation.as_ref() else {
        return;
    };
    let domain_id = push_domain(
        domains,
        EpiphanyMemoryProfile::Incubation,
        "heartbeat incubation",
    );
    let before = nodes.len();
    for theme in incubation.themes.iter().take(12) {
        let theme_id = theme.theme_id.clone();
        let status = theme.status.clone();
        nodes.push(EpiphanyMemoryNode {
            id: memory_graph_node_id(&domain_id, "incubation", &theme_id, None),
            domain_id: domain_id.clone(),
            profile: EpiphanyMemoryProfile::Incubation,
            kind: EpiphanyMemoryNodeKind::IncubationThread,
            title: theme_id.clone(),
            claim: theme.summary.clone(),
            question: if theme.latent_question.is_empty() {
                "What is this incubation theme trying to become?".to_string()
            } else {
                theme.latent_question.clone()
            },
            tension: if theme.holding_close_because.is_empty() {
                "Heartbeat marked this theme but did not preserve a tension.".to_string()
            } else {
                theme.holding_close_because.clone()
            },
            action_implication: if theme.why_it_pulls.is_empty() {
                "Keep in incubation until sleep or review accounts for it.".to_string()
            } else {
                theme.why_it_pulls.clone()
            },
            source_hashes: vec!["anchor:missing".to_string()],
            lifecycle: incubation_lifecycle(&status),
            salience: score_to_u32(theme.priority_score),
            confidence: score_to_u32(theme.maturation),
            ..Default::default()
        });
    }
    push_summary(
        summaries,
        &domain_id,
        "summary-heartbeat-incubation",
        "heartbeat incubation",
        "Heartbeat incubation produced provisional incubation threads.",
        &nodes[before..],
        EpiphanyMemoryProfile::Incubation,
    );
}

fn import_candidate_interventions(
    cognition: &EpiphanyHeartbeatCognitionEntry,
    domains: &mut Vec<EpiphanyMemoryDomain>,
    nodes: &mut Vec<EpiphanyMemoryNode>,
    summaries: &mut Vec<EpiphanyMemorySummary>,
) {
    let Some(candidates) = cognition.candidate_interventions.as_ref() else {
        return;
    };
    let domain_id = push_domain(
        domains,
        EpiphanyMemoryProfile::CandidateIntervention,
        "heartbeat candidate interventions",
    );
    let before = nodes.len();
    for item in candidates.items.iter().take(8) {
        let intervention_id = item.intervention_id.clone();
        nodes.push(EpiphanyMemoryNode {
            id: memory_graph_node_id(&domain_id, "candidate", &intervention_id, None),
            domain_id: domain_id.clone(),
            profile: EpiphanyMemoryProfile::CandidateIntervention,
            kind: EpiphanyMemoryNodeKind::CandidateIntervention,
            title: item.summary.clone(),
            claim: item.draft.clone(),
            question: "Should this candidate intervention be spoken, deferred, or retired?"
                .to_string(),
            tension: "Candidate speech is not authority and still requires review.".to_string(),
            action_implication: "Route through Persona/coordinator review before public surfacing."
                .to_string(),
            source_hashes: vec!["anchor:missing".to_string()],
            lifecycle: EpiphanyMemoryLifecycle::Queued,
            salience: score_to_u32(item.novelty_to_room),
            confidence: 55,
            ..Default::default()
        });
    }
    push_summary(
        summaries,
        &domain_id,
        "summary-heartbeat-candidates",
        "heartbeat candidate interventions",
        "Heartbeat produced review-gated candidate interventions.",
        &nodes[before..],
        EpiphanyMemoryProfile::CandidateIntervention,
    );
}

fn import_bridge_pressure(
    cognition: &EpiphanyHeartbeatCognitionEntry,
    domains: &mut Vec<EpiphanyMemoryDomain>,
    nodes: &mut Vec<EpiphanyMemoryNode>,
    summaries: &mut Vec<EpiphanyMemorySummary>,
) {
    let Some(bridge) = cognition.bridge.as_ref() else {
        return;
    };
    let decision = bridge.decision.speak_decision.as_str();
    if decision == "silence" {
        return;
    }
    let domain_id = push_domain(
        domains,
        EpiphanyMemoryProfile::AgencyPressure,
        "heartbeat bridge pressure",
    );
    let reason = if bridge.decision.reason.is_empty() {
        "Bridge pressure has no explicit reason."
    } else {
        bridge.decision.reason.as_str()
    };
    let node = EpiphanyMemoryNode {
        id: memory_graph_node_id(&domain_id, "bridge-pressure", decision, None),
        domain_id: domain_id.clone(),
        profile: EpiphanyMemoryProfile::AgencyPressure,
        kind: EpiphanyMemoryNodeKind::AgencyPressure,
        title: format!("bridge decision: {decision}"),
        claim: reason.to_string(),
        question: "Does this bridge pressure require action, cooling, or silence?".to_string(),
        tension: "Bridge pressure can steer attention but cannot mutate durable truth.".to_string(),
        action_implication: "Coordinator or Persona may review this pressure; no automatic speech or state write follows.".to_string(),
        source_hashes: vec!["anchor:missing".to_string()],
        lifecycle: if decision == "draft" {
            EpiphanyMemoryLifecycle::Obligated
        } else {
            EpiphanyMemoryLifecycle::Active
        },
        salience: 65,
        confidence: 55,
        ..Default::default()
    };
    nodes.push(node);
    push_summary(
        summaries,
        &domain_id,
        "summary-heartbeat-bridge-pressure",
        "heartbeat bridge pressure",
        "Heartbeat bridge produced provisional agency pressure.",
        nodes.last().map(std::slice::from_ref).unwrap_or(&[]),
        EpiphanyMemoryProfile::AgencyPressure,
    );
}

fn push_domain(
    domains: &mut Vec<EpiphanyMemoryDomain>,
    profile: EpiphanyMemoryProfile,
    name: &str,
) -> String {
    let domain_id = memory_graph_domain_id(profile, "heartbeat", name);
    if !domains.iter().any(|domain| domain.id == domain_id) {
        domains.push(EpiphanyMemoryDomain {
            id: domain_id.clone(),
            profile,
            title: name.to_string(),
            description: Some(
                "Heartbeat cognition imported as provisional memory graph pressure.".to_string(),
            ),
            lifecycle: domain_lifecycle(profile),
        });
    }
    domain_id
}

fn domain_lifecycle(profile: EpiphanyMemoryProfile) -> EpiphanyMemoryLifecycle {
    match profile {
        EpiphanyMemoryProfile::CandidateIntervention => EpiphanyMemoryLifecycle::Queued,
        _ => EpiphanyMemoryLifecycle::Active,
    }
}

fn push_summary(
    summaries: &mut Vec<EpiphanyMemorySummary>,
    domain_id: &str,
    id: &str,
    target: &str,
    claim: &str,
    nodes: &[EpiphanyMemoryNode],
    profile: EpiphanyMemoryProfile,
) {
    if nodes.is_empty() {
        return;
    }
    summaries.push(EpiphanyMemorySummary {
        id: id.to_string(),
        domain_id: domain_id.to_string(),
        covers_node_ids: nodes.iter().map(|node| node.id.clone()).collect(),
        target: target.to_string(),
        claim: claim.to_string(),
        question: "What should sleep or review do with this provisional pressure?".to_string(),
        tension: "Heartbeat cognition is provisional physiology, not durable doctrine.".to_string(),
        action_implication: "Do not promote without a later sleep/review operation.".to_string(),
        anchor_count: 0,
        freshness: crate::memory_graph::EpiphanyMemoryFreshnessStatus::Ready,
        confidence: if profile == EpiphanyMemoryProfile::CandidateIntervention {
            60
        } else {
            55
        },
        ..Default::default()
    });
}

fn score_to_u32(value: f64) -> u32 {
    if value.is_finite() {
        (value.clamp(0.0, 1.0) * 100.0).round() as u32
    } else {
        0
    }
}

fn incubation_lifecycle(status: &str) -> EpiphanyMemoryLifecycle {
    match status {
        "ripe" | "deepening" => EpiphanyMemoryLifecycle::Deepening,
        "refractory" | "stalled" => EpiphanyMemoryLifecycle::Cooling,
        _ => EpiphanyMemoryLifecycle::Active,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heartbeat_state::HEARTBEAT_COGNITION_SCHEMA_VERSION;
    use crate::memory_graph::validate_memory_graph_snapshot;

    #[test]
    fn heartbeat_profile_imports_provisional_pressure_without_promoting_it() {
        let cognition = EpiphanyHeartbeatCognitionEntry {
            schema_version: HEARTBEAT_COGNITION_SCHEMA_VERSION.to_string(),
            updated_at: "2026-05-18T00:00:00Z".to_string(),
            latest_run_id: Some("run-1".to_string()),
            latest_artifact_ref: None,
            source: Some("unit-test".to_string()),
            sleep_cycle: None,
            memory_resonance: Some(HeartbeatMemoryResonance {
                schema_version: "epiphany.memory_resonance.v0".to_string(),
                updated_at: "2026-05-18T00:00:00Z".to_string(),
                source: "unit-test".to_string(),
                record_count: 2,
                pairs: vec![HeartbeatMemoryResonancePair {
                    left_role: "body".to_string(),
                    left_memory_id: "mem-a".to_string(),
                    left_memory_kind: "note".to_string(),
                    left_summary: "Architecture needs anchors.".to_string(),
                    right_role: "soul".to_string(),
                    right_memory_id: "mem-b".to_string(),
                    right_memory_kind: "note".to_string(),
                    right_summary: "Verification needs receipts.".to_string(),
                    strength: 0.8,
                    shared_tokens: vec!["evidence".to_string()],
                    source_roles: vec!["body".to_string(), "soul".to_string()],
                    source_kinds: vec!["note".to_string()],
                    evidence_refs: vec!["mem-a".to_string(), "mem-b".to_string()],
                }],
            }),
            incubation: Some(HeartbeatIncubation {
                schema_version: "epiphany.incubation.v0".to_string(),
                updated_at: "2026-05-18T00:00:00Z".to_string(),
                source_coverage: HeartbeatSourceCoverage::default(),
                last_incubation_summary: "Proprioception/Soul evidence seam is live.".to_string(),
                themes: vec![
                    HeartbeatIncubationTheme {
                        theme_id: "theme-body-soul".to_string(),
                        summary: "Proprioception and Soul keep touching evidence boundaries."
                            .to_string(),
                        latent_question: "Should this become doctrine?".to_string(),
                        holding_close_because: "It is live but not settled.".to_string(),
                        why_it_pulls: "It affects future review gates.".to_string(),
                        status: "deepening".to_string(),
                        priority_score: 0.72,
                        maturation: 0.6,
                        ..Default::default()
                    },
                    HeartbeatIncubationTheme {
                        theme_id: "theme-live-store-shape".to_string(),
                        summary: "Live heartbeat stores may omit optional explanatory fields."
                            .to_string(),
                        status: "deepening".to_string(),
                        ..Default::default()
                    },
                ],
            }),
            thought_lanes: None,
            bridge: Some(HeartbeatCognitionBridge {
                schema_version: "epiphany.cognition_bridge.v0".to_string(),
                updated_at: "2026-05-18T00:00:00Z".to_string(),
                decision: HeartbeatBridgeDecision {
                    speak_decision: "hold".to_string(),
                    reason: "The thought is live but not ready for speech.".to_string(),
                    ..Default::default()
                },
                recent_syntheses: vec![HeartbeatBridgeSynthesis {
                    summary: "The thought is live but not ready for speech.".to_string(),
                    ..Default::default()
                }],
                unresolved_tensions: vec![HeartbeatBridgeTension {
                    topic: "thought authority boundary".to_string(),
                    summary: "Bridge pressure is not authority.".to_string(),
                    opened_at: "2026-05-18T00:00:00Z".to_string(),
                }],
                ..Default::default()
            }),
            candidate_interventions: Some(HeartbeatCandidateInterventions {
                schema_version: "epiphany.candidate_interventions.v0".to_string(),
                updated_at: "2026-05-18T00:00:00Z".to_string(),
                items: vec![HeartbeatCandidateIntervention {
                    intervention_id: "candidate-theme-body-soul".to_string(),
                    summary: "Possible note".to_string(),
                    draft: "Evidence boundaries are lighting up.".to_string(),
                    decision: "draft".to_string(),
                    requires_persona: true,
                    requires_review: true,
                    novelty_to_room: 0.7,
                    saturation_score: 0.1,
                    created_at: "2026-05-18T00:00:00Z".to_string(),
                }],
            }),
            appraisals: None,
            reactions: None,
        };

        let snapshot = memory_graph_from_heartbeat_cognition("heartbeat-profile", &cognition);
        let errors = validate_memory_graph_snapshot(&snapshot);

        assert!(errors.is_empty(), "{errors:?}");
        assert!(snapshot.nodes.iter().any(|node| {
            node.profile == EpiphanyMemoryProfile::ShortTerm
                && node.lifecycle == EpiphanyMemoryLifecycle::Active
        }));
        assert!(snapshot.nodes.iter().any(|node| {
            node.profile == EpiphanyMemoryProfile::Incubation
                && node.lifecycle == EpiphanyMemoryLifecycle::Deepening
        }));
        assert!(snapshot.nodes.iter().any(|node| {
            node.profile == EpiphanyMemoryProfile::CandidateIntervention
                && node.lifecycle == EpiphanyMemoryLifecycle::Queued
        }));
        assert!(snapshot.nodes.iter().any(|node| {
            node.profile == EpiphanyMemoryProfile::AgencyPressure
                && node.lifecycle == EpiphanyMemoryLifecycle::Active
        }));
        assert!(
            snapshot
                .nodes
                .iter()
                .all(|node| node.lifecycle != EpiphanyMemoryLifecycle::Promoted)
        );
    }
}
