use codex_protocol::protocol::EpiphanyChurnState;
use codex_protocol::protocol::EpiphanyEvidenceRecord;
use codex_protocol::protocol::EpiphanyGraph;
use codex_protocol::protocol::EpiphanyGraphCheckpoint;
use codex_protocol::protocol::EpiphanyGraphFrontier;
use codex_protocol::protocol::EpiphanyGraphs;
use codex_protocol::protocol::EpiphanyInvariant;
use codex_protocol::protocol::EpiphanyInvestigationCheckpoint;
use codex_protocol::protocol::EpiphanyObservation;
use codex_protocol::protocol::EpiphanySubgoal;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct EpiphanyPromotionInput {
    pub has_state_replacements: bool,
    pub active_subgoal_id: Option<String>,
    pub subgoals: Option<Vec<EpiphanySubgoal>>,
    pub invariants: Option<Vec<EpiphanyInvariant>>,
    pub graphs: Option<EpiphanyGraphs>,
    pub graph_frontier: Option<EpiphanyGraphFrontier>,
    pub graph_checkpoint: Option<EpiphanyGraphCheckpoint>,
    pub investigation_checkpoint: Option<EpiphanyInvestigationCheckpoint>,
    pub churn: Option<EpiphanyChurnState>,
    pub observations: Vec<EpiphanyObservation>,
    pub evidence: Vec<EpiphanyEvidenceRecord>,
    pub verifier_evidence: EpiphanyEvidenceRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyPromotionDecision {
    pub accepted: bool,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct EpiphanyStateReplacementValidationInput<'a> {
    pub active_subgoal_id: Option<&'a str>,
    pub subgoals: Option<&'a [EpiphanySubgoal]>,
    pub invariants: Option<&'a [EpiphanyInvariant]>,
    pub graphs: Option<&'a EpiphanyGraphs>,
    pub graph_frontier: Option<&'a EpiphanyGraphFrontier>,
    pub graph_checkpoint: Option<&'a EpiphanyGraphCheckpoint>,
    pub investigation_checkpoint: Option<&'a EpiphanyInvestigationCheckpoint>,
    pub available_evidence_ids: Option<&'a HashSet<&'a str>>,
    pub churn: Option<&'a EpiphanyChurnState>,
}

pub fn validate_state_replacement_patch(
    input: EpiphanyStateReplacementValidationInput<'_>,
) -> Vec<String> {
    let mut reasons = Vec::new();
    validate_state_replacements(input, &mut reasons);
    reasons
}

pub fn evaluate_promotion(input: EpiphanyPromotionInput) -> EpiphanyPromotionDecision {
    let mut reasons = Vec::new();

    if !input.has_state_replacements && input.observations.is_empty() && input.evidence.is_empty() {
        reasons.push("promotion patch must contain at least one mutation".to_string());
    }
    if input.has_state_replacements {
        if input.observations.is_empty() {
            reasons.push(
                "state replacement promotions must include at least one observation".to_string(),
            );
        }
        if input.evidence.is_empty() {
            reasons.push(
                "state replacement promotions must include at least one patch evidence record"
                    .to_string(),
            );
        }
    }

    validate_evidence_record(&input.verifier_evidence, "verifierEvidence", &mut reasons);
    if !is_accepting_status(&input.verifier_evidence.status) {
        reasons.push(format!(
            "verifierEvidence.status must be accepting, got {:?}",
            input.verifier_evidence.status
        ));
    }

    let mut evidence_ids = HashSet::new();
    evidence_ids.insert(input.verifier_evidence.id.as_str());
    for evidence in &input.evidence {
        validate_evidence_record(evidence, "patch.evidence", &mut reasons);
        if !evidence_ids.insert(evidence.id.as_str()) {
            reasons.push(format!("duplicate evidence id {:?}", evidence.id));
        }
    }

    let mut observation_ids = HashSet::new();
    for observation in &input.observations {
        validate_observation(observation, &evidence_ids, &mut reasons);
        if !observation_ids.insert(observation.id.as_str()) {
            reasons.push(format!("duplicate observation id {:?}", observation.id));
        }
    }
    validate_state_replacements(
        EpiphanyStateReplacementValidationInput {
            active_subgoal_id: input.active_subgoal_id.as_deref(),
            subgoals: input.subgoals.as_deref(),
            invariants: input.invariants.as_deref(),
            graphs: input.graphs.as_ref(),
            graph_frontier: input.graph_frontier.as_ref(),
            graph_checkpoint: input.graph_checkpoint.as_ref(),
            investigation_checkpoint: input.investigation_checkpoint.as_ref(),
            available_evidence_ids: Some(&evidence_ids),
            churn: input.churn.as_ref(),
        },
        &mut reasons,
    );
    validate_delta_promotion_policy(&input, &mut reasons);

    EpiphanyPromotionDecision {
        accepted: reasons.is_empty(),
        reasons,
    }
}

fn validate_evidence_record(
    evidence: &EpiphanyEvidenceRecord,
    label: &str,
    reasons: &mut Vec<String>,
) {
    require_nonempty(&evidence.id, &format!("{label}.id"), reasons);
    require_nonempty(&evidence.kind, &format!("{label}.kind"), reasons);
    require_nonempty(&evidence.status, &format!("{label}.status"), reasons);
    require_nonempty(&evidence.summary, &format!("{label}.summary"), reasons);
}

fn validate_observation(
    observation: &EpiphanyObservation,
    evidence_ids: &HashSet<&str>,
    reasons: &mut Vec<String>,
) {
    require_nonempty(&observation.id, "patch.observations.id", reasons);
    require_nonempty(&observation.summary, "patch.observations.summary", reasons);
    require_nonempty(
        &observation.source_kind,
        "patch.observations.source_kind",
        reasons,
    );
    require_nonempty(&observation.status, "patch.observations.status", reasons);
    if observation.evidence_ids.is_empty() {
        reasons.push(format!(
            "observation {:?} must cite at least one evidence id",
            observation.id
        ));
    }
    for evidence_id in &observation.evidence_ids {
        if !evidence_ids.contains(evidence_id.as_str()) {
            reasons.push(format!(
                "observation {:?} cites missing evidence id {:?}",
                observation.id, evidence_id
            ));
        }
    }
}

fn validate_state_replacements(
    input: EpiphanyStateReplacementValidationInput<'_>,
    reasons: &mut Vec<String>,
) {
    if let Some(subgoals) = input.subgoals {
        validate_subgoals(subgoals, input.active_subgoal_id, reasons);
    }
    if let Some(active_subgoal_id) = input.active_subgoal_id {
        require_nonempty(active_subgoal_id, "patch.active_subgoal_id", reasons);
    }
    if let Some(invariants) = input.invariants {
        validate_invariants(invariants, reasons);
    }
    if let Some(graphs) = input.graphs {
        validate_graphs(graphs, reasons);
        if let Some(frontier) = input.graph_frontier {
            validate_frontier_against_graphs(frontier, graphs, reasons);
        }
        if let Some(checkpoint) = input.graph_checkpoint {
            validate_checkpoint_against_graphs(checkpoint, graphs, reasons);
        }
    } else {
        if let Some(frontier) = input.graph_frontier {
            validate_frontier_shape(frontier, reasons);
        }
        if let Some(checkpoint) = input.graph_checkpoint {
            validate_checkpoint_shape(checkpoint, reasons);
        }
    }
    if let Some(checkpoint) = input.investigation_checkpoint {
        validate_investigation_checkpoint(checkpoint, input.available_evidence_ids, reasons);
    }
    if let Some(churn) = input.churn {
        validate_churn(churn, reasons);
    }
}

fn validate_delta_promotion_policy(input: &EpiphanyPromotionInput, reasons: &mut Vec<String>) {
    let Some(churn) = input.churn.as_ref() else {
        return;
    };
    if !is_risky_delta(churn) {
        return;
    }

    if churn
        .warning
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        reasons.push(
            "medium/high churn promotions must include patch.churn.warning explaining the delta"
                .to_string(),
        );
    }
    if !is_strong_verifier_kind(&input.verifier_evidence.kind) {
        reasons.push(format!(
            "medium/high churn promotions require verifierEvidence.kind to be verification/test/smoke/review, got {:?}",
            input.verifier_evidence.kind
        ));
    }
}

fn validate_subgoals(
    subgoals: &[EpiphanySubgoal],
    active_subgoal_id: Option<&str>,
    reasons: &mut Vec<String>,
) {
    let mut ids = HashSet::new();
    for subgoal in subgoals {
        require_nonempty(&subgoal.id, "patch.subgoals.id", reasons);
        require_nonempty(&subgoal.title, "patch.subgoals.title", reasons);
        require_nonempty(&subgoal.status, "patch.subgoals.status", reasons);
        if !ids.insert(subgoal.id.as_str()) {
            reasons.push(format!("duplicate subgoal id {:?}", subgoal.id));
        }
    }
    if let Some(active_subgoal_id) = active_subgoal_id
        && !active_subgoal_id.trim().is_empty()
        && !ids.contains(active_subgoal_id)
    {
        reasons.push(format!(
            "active subgoal id {:?} is not present in patch.subgoals",
            active_subgoal_id
        ));
    }
}

fn validate_invariants(invariants: &[EpiphanyInvariant], reasons: &mut Vec<String>) {
    let mut ids = HashSet::new();
    for invariant in invariants {
        require_nonempty(&invariant.id, "patch.invariants.id", reasons);
        require_nonempty(
            &invariant.description,
            "patch.invariants.description",
            reasons,
        );
        require_nonempty(&invariant.status, "patch.invariants.status", reasons);
        if !ids.insert(invariant.id.as_str()) {
            reasons.push(format!("duplicate invariant id {:?}", invariant.id));
        }
    }
}

fn validate_graphs(graphs: &EpiphanyGraphs, reasons: &mut Vec<String>) {
    let architecture_ids =
        validate_graph("patch.graphs.architecture", &graphs.architecture, reasons);
    let dataflow_ids = validate_graph("patch.graphs.dataflow", &graphs.dataflow, reasons);
    let mut links = HashSet::new();
    for link in &graphs.links {
        require_nonempty(
            &link.dataflow_node_id,
            "patch.graphs.links.dataflow_node_id",
            reasons,
        );
        require_nonempty(
            &link.architecture_node_id,
            "patch.graphs.links.architecture_node_id",
            reasons,
        );
        if !link.dataflow_node_id.trim().is_empty()
            && !dataflow_ids.contains(link.dataflow_node_id.as_str())
        {
            reasons.push(format!(
                "graph link references missing dataflow node {:?}",
                link.dataflow_node_id
            ));
        }
        if !link.architecture_node_id.trim().is_empty()
            && !architecture_ids.contains(link.architecture_node_id.as_str())
        {
            reasons.push(format!(
                "graph link references missing architecture node {:?}",
                link.architecture_node_id
            ));
        }
        let key = (
            link.dataflow_node_id.as_str(),
            link.architecture_node_id.as_str(),
        );
        if !links.insert(key) {
            reasons.push(format!(
                "duplicate graph link {:?} -> {:?}",
                link.dataflow_node_id, link.architecture_node_id
            ));
        }
    }
}

fn validate_graph<'a>(
    label: &str,
    graph: &'a EpiphanyGraph,
    reasons: &mut Vec<String>,
) -> HashSet<&'a str> {
    let mut node_ids = HashSet::new();
    for node in &graph.nodes {
        require_nonempty(&node.id, &format!("{label}.nodes.id"), reasons);
        require_nonempty(&node.title, &format!("{label}.nodes.title"), reasons);
        require_nonempty(&node.purpose, &format!("{label}.nodes.purpose"), reasons);
        if !node_ids.insert(node.id.as_str()) {
            reasons.push(format!("duplicate {label} node id {:?}", node.id));
        }
    }

    let mut edge_ids = HashSet::new();
    for edge in &graph.edges {
        require_nonempty(
            &edge.source_id,
            &format!("{label}.edges.source_id"),
            reasons,
        );
        require_nonempty(
            &edge.target_id,
            &format!("{label}.edges.target_id"),
            reasons,
        );
        require_nonempty(&edge.kind, &format!("{label}.edges.kind"), reasons);
        if !edge.source_id.trim().is_empty() && !node_ids.contains(edge.source_id.as_str()) {
            reasons.push(format!(
                "{label} edge references missing source node {:?}",
                edge.source_id
            ));
        }
        if !edge.target_id.trim().is_empty() && !node_ids.contains(edge.target_id.as_str()) {
            reasons.push(format!(
                "{label} edge references missing target node {:?}",
                edge.target_id
            ));
        }
        if let Some(edge_id) = edge.id.as_deref()
            && !edge_id.trim().is_empty()
            && !edge_ids.insert(edge_id)
        {
            reasons.push(format!("duplicate {label} edge id {:?}", edge_id));
        }
    }

    node_ids
}

fn validate_frontier_shape(frontier: &EpiphanyGraphFrontier, reasons: &mut Vec<String>) {
    require_unique_nonempty(
        "patch.graph_frontier.active_node_ids",
        frontier.active_node_ids.iter().map(String::as_str),
        reasons,
    );
    require_unique_nonempty(
        "patch.graph_frontier.active_edge_ids",
        frontier.active_edge_ids.iter().map(String::as_str),
        reasons,
    );
    require_unique_nonempty(
        "patch.graph_frontier.open_question_ids",
        frontier.open_question_ids.iter().map(String::as_str),
        reasons,
    );
    require_unique_nonempty(
        "patch.graph_frontier.open_gap_ids",
        frontier.open_gap_ids.iter().map(String::as_str),
        reasons,
    );
}

fn validate_frontier_against_graphs(
    frontier: &EpiphanyGraphFrontier,
    graphs: &EpiphanyGraphs,
    reasons: &mut Vec<String>,
) {
    validate_frontier_shape(frontier, reasons);
    let node_ids = graph_node_ids(graphs);
    let edge_ids = graph_edge_ids(graphs);
    for node_id in &frontier.active_node_ids {
        if !node_id.trim().is_empty() && !node_ids.contains(node_id.as_str()) {
            reasons.push(format!(
                "graph frontier references missing node {:?}",
                node_id
            ));
        }
    }
    for edge_id in &frontier.active_edge_ids {
        if !edge_id.trim().is_empty() && !edge_ids.contains(edge_id.as_str()) {
            reasons.push(format!(
                "graph frontier references missing edge {:?}",
                edge_id
            ));
        }
    }
}

fn validate_checkpoint_shape(checkpoint: &EpiphanyGraphCheckpoint, reasons: &mut Vec<String>) {
    require_nonempty(
        &checkpoint.checkpoint_id,
        "patch.graph_checkpoint.checkpoint_id",
        reasons,
    );
    require_unique_nonempty(
        "patch.graph_checkpoint.frontier_node_ids",
        checkpoint.frontier_node_ids.iter().map(String::as_str),
        reasons,
    );
    require_unique_nonempty(
        "patch.graph_checkpoint.open_question_ids",
        checkpoint.open_question_ids.iter().map(String::as_str),
        reasons,
    );
    require_unique_nonempty(
        "patch.graph_checkpoint.open_gap_ids",
        checkpoint.open_gap_ids.iter().map(String::as_str),
        reasons,
    );
}

fn validate_checkpoint_against_graphs(
    checkpoint: &EpiphanyGraphCheckpoint,
    graphs: &EpiphanyGraphs,
    reasons: &mut Vec<String>,
) {
    validate_checkpoint_shape(checkpoint, reasons);
    let node_ids = graph_node_ids(graphs);
    for node_id in &checkpoint.frontier_node_ids {
        if !node_id.trim().is_empty() && !node_ids.contains(node_id.as_str()) {
            reasons.push(format!(
                "graph checkpoint references missing frontier node {:?}",
                node_id
            ));
        }
    }
}

fn validate_churn(churn: &EpiphanyChurnState, reasons: &mut Vec<String>) {
    require_nonempty(
        &churn.understanding_status,
        "patch.churn.understanding_status",
        reasons,
    );
    require_nonempty(&churn.diff_pressure, "patch.churn.diff_pressure", reasons);
}

fn validate_investigation_checkpoint(
    checkpoint: &EpiphanyInvestigationCheckpoint,
    available_evidence_ids: Option<&HashSet<&str>>,
    reasons: &mut Vec<String>,
) {
    require_nonempty(
        &checkpoint.checkpoint_id,
        "patch.investigation_checkpoint.checkpoint_id",
        reasons,
    );
    require_nonempty(
        &checkpoint.kind,
        "patch.investigation_checkpoint.kind",
        reasons,
    );
    require_nonempty(
        &checkpoint.focus,
        "patch.investigation_checkpoint.focus",
        reasons,
    );
    if let Some(summary) = checkpoint.summary.as_deref() {
        require_nonempty(summary, "patch.investigation_checkpoint.summary", reasons);
    }
    if let Some(next_action) = checkpoint.next_action.as_deref() {
        require_nonempty(
            next_action,
            "patch.investigation_checkpoint.next_action",
            reasons,
        );
    } else {
        reasons.push("patch.investigation_checkpoint.next_action must not be empty".to_string());
    }
    if let Some(turn_id) = checkpoint.captured_at_turn_id.as_deref() {
        require_nonempty(
            turn_id,
            "patch.investigation_checkpoint.captured_at_turn_id",
            reasons,
        );
    }
    require_unique_nonempty(
        "patch.investigation_checkpoint.open_questions",
        checkpoint.open_questions.iter().map(String::as_str),
        reasons,
    );
    require_unique_nonempty(
        "patch.investigation_checkpoint.evidence_ids",
        checkpoint.evidence_ids.iter().map(String::as_str),
        reasons,
    );
    if let Some(available_evidence_ids) = available_evidence_ids {
        for evidence_id in &checkpoint.evidence_ids {
            if !available_evidence_ids.contains(evidence_id.as_str()) {
                reasons.push(format!(
                    "investigation checkpoint cites missing evidence id {:?}",
                    evidence_id
                ));
            }
        }
    }
}

fn graph_node_ids(graphs: &EpiphanyGraphs) -> HashSet<&str> {
    graphs
        .architecture
        .nodes
        .iter()
        .chain(graphs.dataflow.nodes.iter())
        .map(|node| node.id.as_str())
        .collect()
}

fn graph_edge_ids(graphs: &EpiphanyGraphs) -> HashSet<&str> {
    graphs
        .architecture
        .edges
        .iter()
        .chain(graphs.dataflow.edges.iter())
        .filter_map(|edge| edge.id.as_deref())
        .collect()
}

fn require_unique_nonempty<'a>(
    label: &str,
    values: impl Iterator<Item = &'a str>,
    reasons: &mut Vec<String>,
) {
    let mut seen = HashSet::new();
    for value in values {
        require_nonempty(value, label, reasons);
        if !value.trim().is_empty() && !seen.insert(value) {
            reasons.push(format!("duplicate {label} {:?}", value));
        }
    }
}

fn require_nonempty(value: &str, label: &str, reasons: &mut Vec<String>) {
    if value.trim().is_empty() {
        reasons.push(format!("{label} must not be empty"));
    }
}

fn is_accepting_status(status: &str) -> bool {
    matches!(
        status.trim().to_ascii_lowercase().as_str(),
        "ok" | "accepted" | "verified" | "pass" | "passed"
    )
}

fn is_risky_delta(churn: &EpiphanyChurnState) -> bool {
    pressure_rank(&churn.diff_pressure) >= 2
        || churn
            .graph_freshness
            .as_deref()
            .map(|freshness| {
                let freshness = freshness.trim().to_ascii_lowercase();
                freshness.contains("broadened")
                    || freshness.contains("expanded")
                    || freshness.contains("semantic")
                    || freshness.contains("updated")
            })
            .unwrap_or(false)
}

fn pressure_rank(value: &str) -> u8 {
    match value.trim().to_ascii_lowercase().as_str() {
        "critical" => 4,
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

fn is_strong_verifier_kind(kind: &str) -> bool {
    kind.split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .any(|token| {
            matches!(
                token.to_ascii_lowercase().as_str(),
                "verification" | "verifier" | "test" | "tests" | "smoke" | "review"
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::protocol::EpiphanyGraphEdge;
    use codex_protocol::protocol::EpiphanyGraphNode;

    fn verifier(status: &str) -> EpiphanyEvidenceRecord {
        EpiphanyEvidenceRecord {
            id: "ev-verifier".to_string(),
            kind: "verification".to_string(),
            status: status.to_string(),
            summary: "Verifier accepted the promotion".to_string(),
            code_refs: Vec::new(),
        }
    }

    fn evidence() -> EpiphanyEvidenceRecord {
        EpiphanyEvidenceRecord {
            id: "ev-1".to_string(),
            kind: "verification".to_string(),
            status: "ok".to_string(),
            summary: "Smoke returned revision 1".to_string(),
            code_refs: Vec::new(),
        }
    }

    fn observation() -> EpiphanyObservation {
        EpiphanyObservation {
            id: "obs-1".to_string(),
            summary: "Smoke passed".to_string(),
            source_kind: "smoke".to_string(),
            status: "ok".to_string(),
            code_refs: Vec::new(),
            evidence_ids: vec!["ev-1".to_string()],
        }
    }

    fn promotion_input(
        has_state_replacements: bool,
        observations: Vec<EpiphanyObservation>,
        evidence: Vec<EpiphanyEvidenceRecord>,
        verifier_status: &str,
    ) -> EpiphanyPromotionInput {
        EpiphanyPromotionInput {
            has_state_replacements,
            active_subgoal_id: None,
            subgoals: None,
            invariants: None,
            graphs: None,
            graph_frontier: None,
            graph_checkpoint: None,
            investigation_checkpoint: None,
            churn: None,
            observations,
            evidence,
            verifier_evidence: verifier(verifier_status),
        }
    }

    #[test]
    fn evaluate_promotion_accepts_verified_evidence_linked_observation() {
        let decision = evaluate_promotion(EpiphanyPromotionInput {
            observations: vec![observation()],
            evidence: vec![evidence()],
            verifier_evidence: verifier("ok"),
            ..promotion_input(false, Vec::new(), Vec::new(), "ok")
        });

        assert!(decision.accepted);
        assert!(decision.reasons.is_empty());
    }

    #[test]
    fn evaluate_promotion_rejects_non_accepting_verifier_status() {
        let decision = evaluate_promotion(promotion_input(true, Vec::new(), Vec::new(), "failed"));

        assert!(!decision.accepted);
        assert!(
            decision
                .reasons
                .iter()
                .any(|reason| reason.contains("verifierEvidence.status"))
        );
    }

    #[test]
    fn evaluate_promotion_rejects_observation_without_existing_evidence() {
        let decision = evaluate_promotion(EpiphanyPromotionInput {
            has_state_replacements: false,
            active_subgoal_id: None,
            subgoals: None,
            invariants: None,
            graphs: None,
            graph_frontier: None,
            graph_checkpoint: None,
            investigation_checkpoint: None,
            churn: None,
            observations: vec![EpiphanyObservation {
                id: "obs-1".to_string(),
                summary: "Smoke passed".to_string(),
                source_kind: "smoke".to_string(),
                status: "ok".to_string(),
                code_refs: Vec::new(),
                evidence_ids: vec!["ev-missing".to_string()],
            }],
            evidence: Vec::new(),
            verifier_evidence: verifier("ok"),
        });

        assert!(!decision.accepted);
        assert!(
            decision
                .reasons
                .iter()
                .any(|reason| reason.contains("missing evidence id"))
        );
    }

    #[test]
    fn evaluate_promotion_rejects_state_replacement_without_observation_evidence() {
        let decision = evaluate_promotion(EpiphanyPromotionInput {
            churn: Some(EpiphanyChurnState {
                understanding_status: "grounded".to_string(),
                diff_pressure: "low".to_string(),
                ..Default::default()
            }),
            ..promotion_input(true, Vec::new(), Vec::new(), "ok")
        });

        assert!(!decision.accepted);
        assert!(
            decision
                .reasons
                .iter()
                .any(|reason| reason.contains("state replacement promotions"))
        );
    }

    #[test]
    fn evaluate_promotion_validates_graph_references_for_map_edits() {
        let decision = evaluate_promotion(EpiphanyPromotionInput {
            graphs: Some(EpiphanyGraphs {
                architecture: EpiphanyGraph {
                    nodes: vec![EpiphanyGraphNode {
                        id: "state".to_string(),
                        title: "State".to_string(),
                        purpose: "Carry the explicit map".to_string(),
                        ..Default::default()
                    }],
                    edges: vec![EpiphanyGraphEdge {
                        source_id: "state".to_string(),
                        target_id: "missing".to_string(),
                        kind: "feeds".to_string(),
                        id: Some("edge-1".to_string()),
                        ..Default::default()
                    }],
                },
                ..Default::default()
            }),
            ..promotion_input(true, vec![observation()], vec![evidence()], "ok")
        });

        assert!(!decision.accepted);
        assert!(
            decision
                .reasons
                .iter()
                .any(|reason| reason.contains("missing target node"))
        );
    }

    #[test]
    fn validate_state_replacement_patch_rejects_structural_map_errors() {
        let graphs = EpiphanyGraphs {
            architecture: EpiphanyGraph {
                nodes: vec![EpiphanyGraphNode {
                    id: "state".to_string(),
                    title: "State".to_string(),
                    purpose: "Carry the explicit map".to_string(),
                    ..Default::default()
                }],
                edges: Vec::new(),
            },
            ..Default::default()
        };
        let frontier = EpiphanyGraphFrontier {
            active_node_ids: vec!["missing".to_string()],
            ..Default::default()
        };

        let reasons = validate_state_replacement_patch(EpiphanyStateReplacementValidationInput {
            active_subgoal_id: None,
            subgoals: None,
            invariants: None,
            graphs: Some(&graphs),
            graph_frontier: Some(&frontier),
            graph_checkpoint: None,
            investigation_checkpoint: None,
            available_evidence_ids: None,
            churn: None,
        });

        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("graph frontier references missing node"))
        );
    }

    #[test]
    fn validate_state_replacement_patch_rejects_investigation_checkpoint_with_missing_evidence() {
        let checkpoint = EpiphanyInvestigationCheckpoint {
            checkpoint_id: "ix-1".to_string(),
            kind: "source_gathering".to_string(),
            focus: "Trace the compaction seam.".to_string(),
            next_action: Some("Re-gather source before editing.".to_string()),
            evidence_ids: vec!["ev-missing".to_string()],
            ..Default::default()
        };

        let available_evidence_ids = HashSet::from(["ev-present"]);
        let reasons = validate_state_replacement_patch(EpiphanyStateReplacementValidationInput {
            active_subgoal_id: None,
            subgoals: None,
            invariants: None,
            graphs: None,
            graph_frontier: None,
            graph_checkpoint: None,
            investigation_checkpoint: Some(&checkpoint),
            available_evidence_ids: Some(&available_evidence_ids),
            churn: None,
        });

        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("missing evidence id"))
        );
    }

    #[test]
    fn evaluate_promotion_accepts_verified_map_and_churn_replacement() {
        let decision = evaluate_promotion(EpiphanyPromotionInput {
            graphs: Some(EpiphanyGraphs {
                architecture: EpiphanyGraph {
                    nodes: vec![EpiphanyGraphNode {
                        id: "state".to_string(),
                        title: "State".to_string(),
                        purpose: "Carry the explicit map".to_string(),
                        ..Default::default()
                    }],
                    edges: Vec::new(),
                },
                ..Default::default()
            }),
            graph_frontier: Some(EpiphanyGraphFrontier {
                active_node_ids: vec!["state".to_string()],
                ..Default::default()
            }),
            churn: Some(EpiphanyChurnState {
                understanding_status: "grounded".to_string(),
                diff_pressure: "low".to_string(),
                graph_freshness: Some("fresh".to_string()),
                ..Default::default()
            }),
            ..promotion_input(true, vec![observation()], vec![evidence()], "ok")
        });

        assert!(decision.accepted, "{:?}", decision.reasons);
    }

    #[test]
    fn evaluate_promotion_accepts_investigation_checkpoint_when_backed_by_evidence() {
        let decision = evaluate_promotion(EpiphanyPromotionInput {
            investigation_checkpoint: Some(EpiphanyInvestigationCheckpoint {
                checkpoint_id: "ix-2".to_string(),
                kind: "slice_planning".to_string(),
                focus: "Map the next bounded edit before compaction.".to_string(),
                next_action: Some("Resume from this packet if pressure stays stable.".to_string()),
                evidence_ids: vec!["ev-1".to_string()],
                ..Default::default()
            }),
            ..promotion_input(true, vec![observation()], vec![evidence()], "ok")
        });

        assert!(decision.accepted, "{:?}", decision.reasons);
    }

    #[test]
    fn evaluate_promotion_rejects_risky_churn_without_warning() {
        let decision = evaluate_promotion(EpiphanyPromotionInput {
            churn: Some(EpiphanyChurnState {
                understanding_status: "proposal_updates_map".to_string(),
                diff_pressure: "high".to_string(),
                graph_freshness: Some("proposal-updated".to_string()),
                ..Default::default()
            }),
            ..promotion_input(true, vec![observation()], vec![evidence()], "ok")
        });

        assert!(!decision.accepted);
        assert!(
            decision
                .reasons
                .iter()
                .any(|reason| reason.contains("patch.churn.warning"))
        );
    }

    #[test]
    fn evaluate_promotion_rejects_expanded_churn_even_with_low_pressure() {
        let decision = evaluate_promotion(EpiphanyPromotionInput {
            churn: Some(EpiphanyChurnState {
                understanding_status: "proposal_expands_map".to_string(),
                diff_pressure: "low".to_string(),
                graph_freshness: Some("proposal-expanded".to_string()),
                ..Default::default()
            }),
            ..promotion_input(true, vec![observation()], vec![evidence()], "ok")
        });

        assert!(!decision.accepted);
        assert!(
            decision
                .reasons
                .iter()
                .any(|reason| reason.contains("patch.churn.warning"))
        );
    }

    #[test]
    fn evaluate_promotion_rejects_risky_churn_with_weak_verifier_kind() {
        let mut input = promotion_input(true, vec![observation()], vec![evidence()], "ok");
        input.churn = Some(EpiphanyChurnState {
            understanding_status: "proposal_refines_map".to_string(),
            diff_pressure: "medium".to_string(),
            graph_freshness: Some("proposal-broadened".to_string()),
            warning: Some("Same-path broadening needs explicit verifier review.".to_string()),
            ..Default::default()
        });
        input.verifier_evidence.kind = "observation".to_string();

        let decision = evaluate_promotion(input);

        assert!(!decision.accepted);
        assert!(
            decision
                .reasons
                .iter()
                .any(|reason| reason.contains("verifierEvidence.kind"))
        );
    }

    #[test]
    fn evaluate_promotion_rejects_substring_verifier_kind_match() {
        let mut input = promotion_input(true, vec![observation()], vec![evidence()], "ok");
        input.churn = Some(EpiphanyChurnState {
            understanding_status: "proposal_refines_map".to_string(),
            diff_pressure: "medium".to_string(),
            graph_freshness: Some("proposal-expanded".to_string()),
            warning: Some("Expansion requires a real verifier kind.".to_string()),
            ..Default::default()
        });
        input.verifier_evidence.kind = "contest".to_string();

        let decision = evaluate_promotion(input);

        assert!(!decision.accepted);
        assert!(
            decision
                .reasons
                .iter()
                .any(|reason| reason.contains("verifierEvidence.kind"))
        );
    }

    #[test]
    fn evaluate_promotion_accepts_risky_churn_with_strong_verifier_and_warning() {
        let decision = evaluate_promotion(EpiphanyPromotionInput {
            churn: Some(EpiphanyChurnState {
                understanding_status: "proposal_refines_map".to_string(),
                diff_pressure: "medium".to_string(),
                graph_freshness: Some("proposal-semantically-anchored".to_string()),
                warning: Some(
                    "Semantic anchoring was verified against source context.".to_string(),
                ),
                ..Default::default()
            }),
            ..promotion_input(true, vec![observation()], vec![evidence()], "ok")
        });

        assert!(decision.accepted, "{:?}", decision.reasons);
    }
}
