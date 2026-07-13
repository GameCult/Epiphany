use crate::agent_memory::EpiphanyAgentMemoryEntry;
use crate::agent_memory::GhostlightGoal;
use crate::agent_memory::GhostlightMemory;
use crate::agent_memory::GhostlightValue;
use crate::memory_graph::EpiphanyMemoryDomain;
use crate::memory_graph::EpiphanyMemoryGraphSnapshot;
use crate::memory_graph::EpiphanyMemoryLifecycle;
use crate::memory_graph::EpiphanyMemoryNode;
use crate::memory_graph::EpiphanyMemoryNodeKind;
use crate::memory_graph::EpiphanyMemoryProfile;
use crate::memory_graph::EpiphanyMemorySummary;
use crate::memory_graph::memory_graph_domain_id;
use crate::memory_graph::memory_graph_node_id;

pub fn memory_graph_from_agent_memories(
    graph_id: impl Into<String>,
    entries: &[EpiphanyAgentMemoryEntry],
) -> EpiphanyMemoryGraphSnapshot {
    let mut domains = Vec::new();
    let mut nodes = Vec::new();
    let mut summaries = Vec::new();

    for entry in entries {
        let domain_id = memory_graph_domain_id(
            EpiphanyMemoryProfile::RoleSelf,
            "role",
            entry.role_id.as_str(),
        );
        domains.push(EpiphanyMemoryDomain {
            id: domain_id.clone(),
            profile: EpiphanyMemoryProfile::RoleSelf,
            title: format!("{} role self-memory", entry.role_id),
            description: Some(
                "Reviewed organ-state memory imported into the shared memory graph.".to_string(),
            ),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
        });

        let before = nodes.len();
        import_memories(
            &domain_id,
            &entry.role_id,
            "semantic",
            &entry.agent.memories.semantic,
            &mut nodes,
        );
        import_memories(
            &domain_id,
            &entry.role_id,
            "episodic",
            &entry.agent.memories.episodic,
            &mut nodes,
        );
        import_memories(
            &domain_id,
            &entry.role_id,
            "relationship",
            &entry.agent.memories.relationship_summaries,
            &mut nodes,
        );
        import_goals(&domain_id, &entry.role_id, &entry.agent.goals, &mut nodes);
        import_values(
            &domain_id,
            &entry.role_id,
            &entry.agent.canonical_state.values,
            &mut nodes,
        );

        let role_node_ids = nodes[before..]
            .iter()
            .map(|node| node.id.clone())
            .collect::<Vec<_>>();
        if !role_node_ids.is_empty() {
            summaries.push(EpiphanyMemorySummary {
                id: format!("summary-role-self-{}", entry.role_id),
                domain_id: domain_id.clone(),
                covers_node_ids: role_node_ids,
                target: format!("role:{}", entry.role_id),
                claim: format!(
                    "{} reviewed role self-memory records are available in the shared memory graph.",
                    entry.role_id
                ),
                question: "Which memory should shape this role's next context packet?".to_string(),
                tension: "Role memory is self truth, not project truth.".to_string(),
                action_implication: "Use this summary for role-local context; keep project architecture in repo profiles.".to_string(),
                anchor_count: 0,
                freshness: crate::memory_graph::EpiphanyMemoryFreshnessStatus::Ready,
                confidence: 80,
                ..Default::default()
            });
        }
    }

    EpiphanyMemoryGraphSnapshot {
        schema_version: Some("epiphany.memory_graph.v1".to_string()),
        graph_id: graph_id.into(),
        domains,
        nodes,
        summaries,
        freshness: Some(crate::memory_graph::EpiphanyMemoryFreshness {
            status: crate::memory_graph::EpiphanyMemoryFreshnessStatus::Ready,
            note: Some("Imported from reviewed organ-state memory.".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn import_memories(
    domain_id: &str,
    role_id: &str,
    memory_kind: &str,
    memories: &[GhostlightMemory],
    nodes: &mut Vec<EpiphanyMemoryNode>,
) {
    for memory in memories {
        nodes.push(EpiphanyMemoryNode {
            id: memory_graph_node_id(domain_id, memory_kind, memory.memory_id.as_str(), None),
            domain_id: domain_id.to_string(),
            profile: EpiphanyMemoryProfile::RoleSelf,
            kind: EpiphanyMemoryNodeKind::RoleMemory,
            title: format!("{role_id} {memory_kind} memory"),
            claim: memory.summary.clone(),
            question: "How should this memory shape future role judgment?".to_string(),
            tension: String::new(),
            action_implication: format!("Use as {role_id} role-local memory, not project truth."),
            source_hashes: vec!["anchor:missing".to_string()],
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            salience: score_to_u32(memory.salience),
            confidence: score_to_u32(memory.confidence),
            ..Default::default()
        });
    }
}

fn import_goals(
    domain_id: &str,
    role_id: &str,
    goals: &[GhostlightGoal],
    nodes: &mut Vec<EpiphanyMemoryNode>,
) {
    for goal in goals {
        nodes.push(EpiphanyMemoryNode {
            id: memory_graph_node_id(domain_id, "goal", goal.goal_id.as_str(), None),
            domain_id: domain_id.to_string(),
            profile: EpiphanyMemoryProfile::RoleSelf,
            kind: EpiphanyMemoryNodeKind::RoleMemory,
            title: format!("{role_id} goal"),
            claim: goal.description.clone(),
            question: "Does this goal still serve the current role?".to_string(),
            tension: goal.blockers.join("; "),
            action_implication: format!(
                "Goal status is {} with scope {}.",
                goal.status, goal.scope
            ),
            source_hashes: vec!["anchor:missing".to_string()],
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            salience: score_to_u32(goal.priority),
            confidence: 80,
            ..Default::default()
        });
    }
}

fn import_values(
    domain_id: &str,
    role_id: &str,
    values: &[GhostlightValue],
    nodes: &mut Vec<EpiphanyMemoryNode>,
) {
    for value in values {
        nodes.push(EpiphanyMemoryNode {
            id: memory_graph_node_id(domain_id, "value", value.value_id.as_str(), None),
            domain_id: domain_id.to_string(),
            profile: EpiphanyMemoryProfile::Identity,
            kind: EpiphanyMemoryNodeKind::Identity,
            title: format!("{role_id} value: {}", value.label),
            claim: value.label.clone(),
            question: "How should this value constrain future role behavior?".to_string(),
            tension: if value.unforgivable_if_betrayed {
                "Marked unforgivable if betrayed.".to_string()
            } else {
                String::new()
            },
            action_implication: format!("Respect this value when selecting {role_id} context."),
            source_hashes: vec!["anchor:missing".to_string()],
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            salience: score_to_u32(value.priority),
            confidence: 80,
            ..Default::default()
        });
    }
}

fn score_to_u32(value: f64) -> u32 {
    if value.is_finite() {
        (value.clamp(0.0, 1.0) * 100.0).round() as u32
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_memory::GhostlightAgent;
    use crate::agent_memory::GhostlightCanonicalState;
    use crate::agent_memory::GhostlightMemories;
    use crate::agent_memory::GhostlightWorld;
    use crate::memory_graph::validate_memory_graph_snapshot;

    #[test]
    fn agent_profile_imports_reviewed_role_memory_into_valid_graph() {
        let entry = EpiphanyAgentMemoryEntry {
            schema_version: "ghostlight.agent_state.v0".to_string(),
            role_id: "modeling".to_string(),
            world: GhostlightWorld::default(),
            agent: GhostlightAgent {
                memories: GhostlightMemories {
                    semantic: vec![GhostlightMemory {
                        memory_id: "mem-1".to_string(),
                        summary: "Modeling should reject graph updates without source anchors."
                            .to_string(),
                        salience: 0.8,
                        confidence: 0.9,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                goals: vec![GhostlightGoal {
                    goal_id: "goal-1".to_string(),
                    description: "Keep architecture maps source-grounded.".to_string(),
                    scope: "role".to_string(),
                    priority: 0.7,
                    emotional_stake: "coherence".to_string(),
                    status: "active".to_string(),
                    ..Default::default()
                }],
                canonical_state: GhostlightCanonicalState {
                    values: vec![GhostlightValue {
                        value_id: "value-1".to_string(),
                        label: "Coherence over velocity".to_string(),
                        priority: 1.0,
                        unforgivable_if_betrayed: true,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
            relationships: Vec::new(),
            events: Vec::new(),
            scenes: Vec::new(),
        };

        let snapshot = memory_graph_from_agent_memories("agent-profile", &[entry]);
        let errors = validate_memory_graph_snapshot(&snapshot);

        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(snapshot.domains.len(), 1);
        assert_eq!(snapshot.nodes.len(), 3);
        assert_eq!(snapshot.summaries.len(), 1);
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|node| node.profile == EpiphanyMemoryProfile::Identity)
        );
    }
}
