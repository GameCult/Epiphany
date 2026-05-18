use crate::heartbeat_state::EpiphanyHeartbeatCognitionEntry;
use crate::memory_graph::EpiphanyMemoryDomain;
use crate::memory_graph::EpiphanyMemoryGraphSnapshot;
use crate::memory_graph::EpiphanyMemoryLifecycle;
use crate::memory_graph::EpiphanyMemoryNode;
use crate::memory_graph::EpiphanyMemoryNodeKind;
use crate::memory_graph::EpiphanyMemoryProfile;
use crate::memory_graph::EpiphanyMemorySummary;
use crate::memory_graph::memory_graph_domain_id;
use crate::memory_graph::memory_graph_node_id;
use serde_json::Value;

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
    let Some(pairs) = resonance.get("pairs").and_then(Value::as_array) else {
        return;
    };
    let domain_id = push_domain(
        domains,
        EpiphanyMemoryProfile::ShortTerm,
        "heartbeat resonance",
    );
    let before = nodes.len();
    for pair in pairs.iter().take(8) {
        let left = string_at(pair, "leftSummary");
        let right = string_at(pair, "rightSummary");
        let left_role = string_at(pair, "leftRole");
        let right_role = string_at(pair, "rightRole");
        let id_seed = format!(
            "{}:{}:{}",
            left_role,
            right_role,
            string_at(pair, "leftMemoryId")
        );
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
            salience: score_to_u32(number_at(pair, "strength")),
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
    let Some(themes) = incubation.get("themes").and_then(Value::as_array) else {
        return;
    };
    let domain_id = push_domain(
        domains,
        EpiphanyMemoryProfile::Incubation,
        "heartbeat incubation",
    );
    let before = nodes.len();
    for theme in themes.iter().take(12) {
        let theme_id = string_at(theme, "themeId");
        let status = string_at(theme, "status");
        nodes.push(EpiphanyMemoryNode {
            id: memory_graph_node_id(&domain_id, "incubation", &theme_id, None),
            domain_id: domain_id.clone(),
            profile: EpiphanyMemoryProfile::Incubation,
            kind: EpiphanyMemoryNodeKind::IncubationThread,
            title: theme_id.clone(),
            claim: string_at(theme, "summary"),
            question: string_at_any(
                theme,
                &["latentQuestion", "question"],
                "What is this incubation theme trying to become?",
            ),
            tension: string_at_any(
                theme,
                &["holdingCloseBecause", "tension"],
                "Heartbeat marked this theme but did not preserve a tension.",
            ),
            action_implication: string_at_any(
                theme,
                &["whyItPulls", "actionImplication"],
                "Keep in incubation until sleep or review accounts for it.",
            ),
            source_hashes: vec!["anchor:missing".to_string()],
            lifecycle: incubation_lifecycle(&status),
            salience: score_to_u32(number_at(theme, "priorityScore")),
            confidence: score_to_u32(number_at(theme, "maturation")),
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
        nodes.push(EpiphanyMemoryNode {
            id: memory_graph_node_id(&domain_id, "candidate", &item.intervention_id, None),
            domain_id: domain_id.clone(),
            profile: EpiphanyMemoryProfile::CandidateIntervention,
            kind: EpiphanyMemoryNodeKind::CandidateIntervention,
            title: item.summary.clone(),
            claim: item.draft.clone(),
            question: "Should this candidate intervention be spoken, deferred, or retired?"
                .to_string(),
            tension: "Candidate speech is not authority and still requires review.".to_string(),
            action_implication: "Route through Face/coordinator review before public surfacing."
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
    let decision = bridge
        .pointer("/decision/speakDecision")
        .and_then(Value::as_str)
        .unwrap_or("silence");
    if decision == "silence" {
        return;
    }
    let domain_id = push_domain(
        domains,
        EpiphanyMemoryProfile::AgencyPressure,
        "heartbeat bridge pressure",
    );
    let reason = bridge
        .pointer("/decision/reason")
        .and_then(Value::as_str)
        .unwrap_or("Bridge pressure has no explicit reason.");
    let node = EpiphanyMemoryNode {
        id: memory_graph_node_id(&domain_id, "bridge-pressure", decision, None),
        domain_id: domain_id.clone(),
        profile: EpiphanyMemoryProfile::AgencyPressure,
        kind: EpiphanyMemoryNodeKind::AgencyPressure,
        title: format!("bridge decision: {decision}"),
        claim: reason.to_string(),
        question: "Does this bridge pressure require action, cooling, or silence?".to_string(),
        tension: "Bridge pressure can steer attention but cannot mutate durable truth.".to_string(),
        action_implication: "Coordinator or Face may review this pressure; no automatic speech or state write follows.".to_string(),
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

fn string_at(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn string_at_any(value: &Value, keys: &[&str], fallback: &str) -> String {
    keys.iter()
        .find_map(|key| {
            let value = value.get(key).and_then(Value::as_str)?;
            if value.trim().is_empty() {
                None
            } else {
                Some(value.to_string())
            }
        })
        .unwrap_or_else(|| fallback.to_string())
}

fn number_at(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or_default()
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
    use crate::heartbeat_state::HeartbeatCandidateIntervention;
    use crate::heartbeat_state::HeartbeatCandidateInterventions;
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
            memory_resonance: Some(serde_json::json!({
                "pairs": [{
                    "leftRole": "body",
                    "rightRole": "soul",
                    "leftMemoryId": "mem-a",
                    "leftSummary": "Architecture needs anchors.",
                    "rightSummary": "Verification needs receipts.",
                    "strength": 0.8
                }]
            })),
            incubation: Some(serde_json::json!({
                "themes": [{
                    "themeId": "theme-body-soul",
                    "summary": "Body and Soul keep touching evidence boundaries.",
                    "latentQuestion": "Should this become doctrine?",
                    "holdingCloseBecause": "It is live but not settled.",
                    "whyItPulls": "It affects future review gates.",
                    "status": "deepening",
                    "priorityScore": 0.72,
                    "maturation": 0.6
                }, {
                    "themeId": "theme-live-store-shape",
                    "summary": "Live heartbeat stores may omit optional explanatory fields.",
                    "status": "deepening"
                }]
            })),
            thought_lanes: None,
            bridge: Some(serde_json::json!({
                "decision": {
                    "speakDecision": "hold",
                    "reason": "The thought is live but not ready for speech."
                }
            })),
            candidate_interventions: Some(HeartbeatCandidateInterventions {
                schema_version: "epiphany.candidate_interventions.v0".to_string(),
                updated_at: "2026-05-18T00:00:00Z".to_string(),
                items: vec![HeartbeatCandidateIntervention {
                    intervention_id: "candidate-theme-body-soul".to_string(),
                    summary: "Possible note".to_string(),
                    draft: "Evidence boundaries are lighting up.".to_string(),
                    novelty_to_room: 0.7,
                    ..Default::default()
                }],
            }),
            appraisals: None,
            reactions: None,
            extra: Default::default(),
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
