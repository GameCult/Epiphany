use anyhow::Result;
use anyhow::anyhow;
use codex_protocol::protocol::EpiphanyChurnState;
use codex_protocol::protocol::EpiphanyCodeRef;
use codex_protocol::protocol::EpiphanyEvidenceRecord;
use codex_protocol::protocol::EpiphanyGraphEdge;
use codex_protocol::protocol::EpiphanyGraphFrontier;
use codex_protocol::protocol::EpiphanyGraphNode;
use codex_protocol::protocol::EpiphanyGraphs;
use codex_protocol::protocol::EpiphanyObservation;
use codex_protocol::protocol::EpiphanyThreadState;
use sha1::Digest;
use sha1::Sha1;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

const SUMMARY_LIMIT: usize = 220;

#[derive(Debug, Clone)]
pub struct EpiphanyMapProposalInput {
    pub state: EpiphanyThreadState,
    pub observation_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyMapProposal {
    pub observation: EpiphanyObservation,
    pub evidence: EpiphanyEvidenceRecord,
    pub graphs: EpiphanyGraphs,
    pub graph_frontier: EpiphanyGraphFrontier,
    pub churn: EpiphanyChurnState,
}

pub fn propose_map_update(input: EpiphanyMapProposalInput) -> Result<EpiphanyMapProposal> {
    let observation_ids = normalize_observation_ids(input.observation_ids)?;
    let observations = select_observations(&input.state, &observation_ids)?;
    let code_refs = collect_code_refs(&observations)?;
    let fingerprint = fingerprint(&input.state.revision, &observation_ids, &code_refs);
    let mut graphs = input.state.graphs.clone();
    let mut active_node_ids = Vec::new();
    let mut reused_nodes = 0usize;
    let mut created_nodes = 0usize;

    for path in unique_code_ref_paths(&code_refs) {
        let path_code_refs = code_refs_for_path(&code_refs, &path);
        let candidate_node_id = graph_node_id(&path);
        if let Some(node_index) =
            find_architecture_node_for_path(&graphs, &path_code_refs, &path, &candidate_node_id)
        {
            let node = &mut graphs.architecture.nodes[node_index];
            active_node_ids.push(node.id.clone());
            merge_code_refs(&mut node.code_refs, path_code_refs);
            reused_nodes += 1;
        } else {
            let node_id = candidate_node_id;
            active_node_ids.push(node_id.clone());
            graphs.architecture.nodes.push(EpiphanyGraphNode {
                id: node_id,
                title: title_from_path(&path),
                purpose: truncate_chars(
                    &format!(
                        "Candidate implementation surface from verified observation: {}",
                        observations[0].summary
                    ),
                    SUMMARY_LIMIT,
                ),
                mechanism: Some(format!(
                    "Proposed from observation ids: {}",
                    observation_ids.join(", ")
                )),
                metaphor: None,
                status: Some("candidate".to_string()),
                code_refs: path_code_refs,
            });
            created_nodes += 1;
        }
    }

    let mut frontier = input.state.graph_frontier.clone().unwrap_or_default();
    let frontier_node_ids = linked_frontier_node_ids(&graphs, active_node_ids);
    let frontier_edge_ids = incident_edge_ids(&graphs, &frontier_node_ids);
    merge_unique(&mut frontier.active_node_ids, frontier_node_ids);
    merge_unique(&mut frontier.active_edge_ids, frontier_edge_ids);
    merge_unique(
        &mut frontier.dirty_paths,
        unique_code_ref_paths(&code_refs).into_iter().collect(),
    );

    let evidence_id = format!("ev-map-proposal-{fingerprint}");
    let observation = EpiphanyObservation {
        id: format!("obs-map-proposal-{fingerprint}"),
        summary: truncate_chars(
            &format!(
                "Map/churn proposal from verified observations: {}",
                observation_ids.join(", ")
            ),
            SUMMARY_LIMIT,
        ),
        source_kind: "epiphany-map-proposal".to_string(),
        status: "candidate".to_string(),
        code_refs: code_refs.clone(),
        evidence_ids: vec![evidence_id.clone()],
    };
    let evidence = EpiphanyEvidenceRecord {
        id: evidence_id,
        kind: "proposal".to_string(),
        status: "candidate".to_string(),
        summary: truncate_chars(
            &format!(
                "Proposed graph frontier and churn update from verified observations: {}; reused {reused_nodes} existing node(s), created {created_nodes} new node(s)",
                observation_ids.join(", "),
            ),
            SUMMARY_LIMIT,
        ),
        code_refs,
    };
    let understanding_status = proposal_understanding_status(reused_nodes, created_nodes);
    let graph_freshness = proposal_graph_freshness(reused_nodes, created_nodes);
    let churn = EpiphanyChurnState {
        understanding_status,
        diff_pressure: input
            .state
            .churn
            .as_ref()
            .map(|churn| churn.diff_pressure.trim())
            .filter(|value| !value.is_empty())
            .unwrap_or("low")
            .to_string(),
        graph_freshness: Some(graph_freshness),
        warning: Some(format!(
            "Map/churn proposal derived from verified observations; reused {reused_nodes} existing node(s), created {created_nodes} new node(s); promote only after verifier acceptance."
        )),
        unexplained_writes: input
            .state
            .churn
            .as_ref()
            .and_then(|churn| churn.unexplained_writes),
    };

    Ok(EpiphanyMapProposal {
        observation,
        evidence,
        graphs,
        graph_frontier: frontier,
        churn,
    })
}

fn normalize_observation_ids(observation_ids: Vec<String>) -> Result<Vec<String>> {
    let mut normalized = Vec::new();
    let mut seen = HashSet::new();
    for id in observation_ids {
        let id = id.trim();
        if id.is_empty() {
            continue;
        }
        if seen.insert(id.to_string()) {
            normalized.push(id.to_string());
        }
    }
    if normalized.is_empty() {
        Err(anyhow!("observationIds must include at least one id"))
    } else {
        Ok(normalized)
    }
}

fn select_observations<'a>(
    state: &'a EpiphanyThreadState,
    observation_ids: &[String],
) -> Result<Vec<&'a EpiphanyObservation>> {
    let mut selected = Vec::new();
    for observation_id in observation_ids {
        let observation = state
            .observations
            .iter()
            .find(|observation| observation.id == *observation_id)
            .ok_or_else(|| anyhow!("observation id {observation_id:?} not found in state"))?;
        if !is_verified_status(&observation.status) {
            return Err(anyhow!(
                "observation id {observation_id:?} must have verified/accepted status, got {:?}",
                observation.status
            ));
        }
        selected.push(observation);
    }
    Ok(selected)
}

fn collect_code_refs(observations: &[&EpiphanyObservation]) -> Result<Vec<EpiphanyCodeRef>> {
    let mut code_refs = Vec::new();
    let mut seen = HashSet::new();
    for observation in observations {
        for code_ref in &observation.code_refs {
            let key = code_ref_key(code_ref);
            if seen.insert(key) {
                code_refs.push(code_ref.clone());
            }
        }
    }
    if code_refs.is_empty() {
        Err(anyhow!(
            "map proposals require at least one code ref on the selected observations"
        ))
    } else {
        Ok(code_refs)
    }
}

fn unique_code_ref_paths(code_refs: &[EpiphanyCodeRef]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    for code_ref in code_refs {
        let key = code_ref.path.to_string_lossy().to_string();
        if seen.insert(key) {
            paths.push(code_ref.path.clone());
        }
    }
    paths
}

fn code_refs_for_path(code_refs: &[EpiphanyCodeRef], path: &Path) -> Vec<EpiphanyCodeRef> {
    code_refs
        .iter()
        .filter(|code_ref| code_ref.path == path)
        .cloned()
        .collect()
}

fn find_architecture_node_for_path(
    graphs: &EpiphanyGraphs,
    path_code_refs: &[EpiphanyCodeRef],
    path: &Path,
    candidate_node_id: &str,
) -> Option<usize> {
    graphs
        .architecture
        .nodes
        .iter()
        .position(|node| has_exact_code_ref_overlap(&node.code_refs, path_code_refs))
        .or_else(|| {
            graphs
                .architecture
                .nodes
                .iter()
                .position(|node| node.code_refs.iter().any(|code_ref| code_ref.path == path))
        })
        .or_else(|| {
            graphs
                .architecture
                .nodes
                .iter()
                .position(|node| node.id == candidate_node_id)
        })
}

fn has_exact_code_ref_overlap(
    existing_code_refs: &[EpiphanyCodeRef],
    proposed_code_refs: &[EpiphanyCodeRef],
) -> bool {
    let existing_keys = existing_code_refs
        .iter()
        .map(code_ref_key)
        .collect::<HashSet<_>>();
    proposed_code_refs
        .iter()
        .map(code_ref_key)
        .any(|key| existing_keys.contains(&key))
}

fn merge_code_refs(target: &mut Vec<EpiphanyCodeRef>, additions: Vec<EpiphanyCodeRef>) {
    let mut seen = target.iter().map(code_ref_key).collect::<HashSet<_>>();
    for addition in additions {
        if seen.insert(code_ref_key(&addition)) {
            target.push(addition);
        }
    }
}

fn linked_frontier_node_ids(graphs: &EpiphanyGraphs, direct_node_ids: Vec<String>) -> Vec<String> {
    let mut linked_node_ids = direct_node_ids;
    let mut seen = linked_node_ids.iter().cloned().collect::<HashSet<_>>();
    let mut changed = true;

    while changed {
        changed = false;
        for link in &graphs.links {
            if seen.contains(&link.architecture_node_id)
                && seen.insert(link.dataflow_node_id.clone())
            {
                linked_node_ids.push(link.dataflow_node_id.clone());
                changed = true;
            }
            if seen.contains(&link.dataflow_node_id)
                && seen.insert(link.architecture_node_id.clone())
            {
                linked_node_ids.push(link.architecture_node_id.clone());
                changed = true;
            }
        }
    }

    linked_node_ids
}

fn incident_edge_ids(graphs: &EpiphanyGraphs, node_ids: &[String]) -> Vec<String> {
    let node_ids = node_ids.iter().map(String::as_str).collect::<HashSet<_>>();
    let mut edge_ids = Vec::new();
    let mut seen = HashSet::new();
    for edge in graphs
        .architecture
        .edges
        .iter()
        .chain(graphs.dataflow.edges.iter())
    {
        if is_incident_to_any_node(edge, &node_ids)
            && let Some(edge_id) = edge.id.as_deref().map(str::trim)
            && !edge_id.is_empty()
            && seen.insert(edge_id.to_string())
        {
            edge_ids.push(edge_id.to_string());
        }
    }
    edge_ids
}

fn is_incident_to_any_node(edge: &EpiphanyGraphEdge, node_ids: &HashSet<&str>) -> bool {
    node_ids.contains(edge.source_id.as_str()) || node_ids.contains(edge.target_id.as_str())
}

fn merge_unique<T>(target: &mut Vec<T>, additions: Vec<T>)
where
    T: Clone + Eq + std::hash::Hash,
{
    let mut seen: HashSet<T> = target.iter().cloned().collect();
    for addition in additions {
        if seen.insert(addition.clone()) {
            target.push(addition);
        }
    }
}

fn graph_node_id(path: &Path) -> String {
    let mut hasher = Sha1::new();
    hasher.update(path.to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    let suffix: String = format!("{digest:x}").chars().take(12).collect();
    format!("arch-path-{suffix}")
}

fn title_from_path(path: &Path) -> String {
    if let Some(file_name) = path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
    {
        file_name.to_string()
    } else {
        path.to_string_lossy().to_string()
    }
}

fn fingerprint(
    revision: &u64,
    observation_ids: &[String],
    code_refs: &[EpiphanyCodeRef],
) -> String {
    let mut hasher = Sha1::new();
    hasher.update(revision.to_string().as_bytes());
    for observation_id in observation_ids {
        hasher.update(b"\0");
        hasher.update(observation_id.as_bytes());
    }
    for code_ref in code_refs {
        hasher.update(b"\0");
        hasher.update(code_ref_key(code_ref).as_bytes());
    }
    let digest = hasher.finalize();
    format!("{digest:x}").chars().take(12).collect()
}

fn code_ref_key(code_ref: &EpiphanyCodeRef) -> String {
    format!(
        "{}:{}:{}:{}",
        code_ref.path.to_string_lossy(),
        code_ref.start_line.unwrap_or_default(),
        code_ref.end_line.unwrap_or_default(),
        code_ref.symbol.as_deref().unwrap_or_default()
    )
}

fn truncate_chars(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let mut truncated = value
        .chars()
        .take(limit.saturating_sub(3))
        .collect::<String>();
    truncated.push_str("...");
    truncated
}

fn proposal_understanding_status(reused_nodes: usize, created_nodes: usize) -> String {
    match (reused_nodes > 0, created_nodes > 0) {
        (true, false) => "proposal_refines_map",
        (false, true) => "proposal_expands_map",
        (true, true) => "proposal_updates_map",
        (false, false) => "proposal_ready",
    }
    .to_string()
}

fn proposal_graph_freshness(reused_nodes: usize, created_nodes: usize) -> String {
    match (reused_nodes > 0, created_nodes > 0) {
        (true, false) => "proposal-refined",
        (false, true) => "proposal-expanded",
        (true, true) => "proposal-updated",
        (false, false) => "proposal",
    }
    .to_string()
}

fn is_verified_status(status: &str) -> bool {
    matches!(
        status.trim().to_ascii_lowercase().as_str(),
        "ok" | "accepted" | "verified" | "pass" | "passed"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::promotion::EpiphanyPromotionInput;
    use crate::promotion::evaluate_promotion;
    use codex_protocol::protocol::EpiphanyGraph;
    use codex_protocol::protocol::EpiphanyGraphEdge;
    use codex_protocol::protocol::EpiphanyGraphLink;

    fn code_ref(path: &str) -> EpiphanyCodeRef {
        EpiphanyCodeRef {
            path: PathBuf::from(path),
            start_line: Some(10),
            end_line: Some(20),
            symbol: Some("demo".to_string()),
            note: None,
        }
    }

    fn state_with_observation(status: &str) -> EpiphanyThreadState {
        EpiphanyThreadState {
            revision: 4,
            observations: vec![EpiphanyObservation {
                id: "obs-verified".to_string(),
                summary: "The prompt renderer injects Epiphany state".to_string(),
                source_kind: "verification".to_string(),
                status: status.to_string(),
                code_refs: vec![code_ref("epiphany-core/src/prompt.rs")],
                evidence_ids: vec!["ev-verified".to_string()],
            }],
            churn: Some(EpiphanyChurnState {
                understanding_status: "grounded".to_string(),
                diff_pressure: "medium".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn verifier() -> EpiphanyEvidenceRecord {
        EpiphanyEvidenceRecord {
            id: "ev-verifier".to_string(),
            kind: "verification".to_string(),
            status: "ok".to_string(),
            summary: "Verifier accepted proposal".to_string(),
            code_refs: Vec::new(),
        }
    }

    #[test]
    fn propose_map_update_returns_candidate_patch_parts() {
        let proposal = propose_map_update(EpiphanyMapProposalInput {
            state: state_with_observation("verified"),
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect("proposal");

        assert!(proposal.observation.id.starts_with("obs-map-proposal-"));
        assert_eq!(proposal.evidence.status, "candidate");
        assert_eq!(proposal.graphs.architecture.nodes.len(), 1);
        assert_eq!(proposal.graphs.architecture.nodes[0].title, "prompt.rs");
        assert_eq!(proposal.graph_frontier.active_node_ids.len(), 1);
        assert_eq!(proposal.churn.understanding_status, "proposal_expands_map");
        assert_eq!(proposal.churn.diff_pressure, "medium");
    }

    #[test]
    fn propose_map_update_reuses_existing_architecture_node_for_matching_code_ref() {
        let mut state = state_with_observation("verified");
        state.graphs.architecture.nodes.push(EpiphanyGraphNode {
            id: "prompt-renderer".to_string(),
            title: "Prompt renderer".to_string(),
            purpose: "Render Epiphany state into developer context".to_string(),
            status: Some("grounded".to_string()),
            code_refs: vec![code_ref("epiphany-core/src/prompt.rs")],
            ..Default::default()
        });
        state.observations[0].code_refs.push(EpiphanyCodeRef {
            path: PathBuf::from("epiphany-core/src/prompt.rs"),
            start_line: Some(30),
            end_line: Some(40),
            symbol: Some("render_epiphany_state".to_string()),
            note: Some("newly verified line span".to_string()),
        });

        let proposal = propose_map_update(EpiphanyMapProposalInput {
            state,
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect("proposal");

        assert_eq!(proposal.graphs.architecture.nodes.len(), 1);
        let node = &proposal.graphs.architecture.nodes[0];
        assert_eq!(node.id, "prompt-renderer");
        assert_eq!(node.status.as_deref(), Some("grounded"));
        assert_eq!(node.code_refs.len(), 2);
        assert_eq!(
            proposal.graph_frontier.active_node_ids,
            vec!["prompt-renderer".to_string()]
        );
        assert_eq!(proposal.churn.understanding_status, "proposal_refines_map");
        assert_eq!(
            proposal.churn.graph_freshness.as_deref(),
            Some("proposal-refined")
        );
    }

    #[test]
    fn propose_map_update_reuses_existing_deterministic_path_node_without_refs() {
        let mut state = state_with_observation("verified");
        let node_id = graph_node_id(Path::new("epiphany-core/src/prompt.rs"));
        state.graphs.architecture.nodes.push(EpiphanyGraphNode {
            id: node_id.clone(),
            title: "Prompt path".to_string(),
            purpose: "Existing path-derived proposal node".to_string(),
            ..Default::default()
        });

        let proposal = propose_map_update(EpiphanyMapProposalInput {
            state,
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect("proposal");

        assert_eq!(proposal.graphs.architecture.nodes.len(), 1);
        assert_eq!(proposal.graphs.architecture.nodes[0].id, node_id);
        assert_eq!(proposal.graphs.architecture.nodes[0].code_refs.len(), 1);
        assert_eq!(proposal.churn.understanding_status, "proposal_refines_map");
    }

    #[test]
    fn propose_map_update_focuses_linked_nodes_and_incident_edges() {
        let mut state = state_with_observation("verified");
        state.graphs = EpiphanyGraphs {
            architecture: EpiphanyGraph {
                nodes: vec![
                    EpiphanyGraphNode {
                        id: "session".to_string(),
                        title: "Session".to_string(),
                        purpose: "Build turn context".to_string(),
                        ..Default::default()
                    },
                    EpiphanyGraphNode {
                        id: "prompt-renderer".to_string(),
                        title: "Prompt renderer".to_string(),
                        purpose: "Render Epiphany state".to_string(),
                        code_refs: vec![code_ref("epiphany-core/src/prompt.rs")],
                        ..Default::default()
                    },
                ],
                edges: vec![EpiphanyGraphEdge {
                    id: Some("edge-session-prompt".to_string()),
                    source_id: "session".to_string(),
                    target_id: "prompt-renderer".to_string(),
                    kind: "calls".to_string(),
                    ..Default::default()
                }],
            },
            dataflow: EpiphanyGraph {
                nodes: vec![
                    EpiphanyGraphNode {
                        id: "developer-context".to_string(),
                        title: "Developer context".to_string(),
                        purpose: "Carry rendered instructions".to_string(),
                        ..Default::default()
                    },
                    EpiphanyGraphNode {
                        id: "epiphany-state-fragment".to_string(),
                        title: "Epiphany state fragment".to_string(),
                        purpose: "Expose typed state to the model".to_string(),
                        ..Default::default()
                    },
                ],
                edges: vec![EpiphanyGraphEdge {
                    id: Some("edge-fragment-context".to_string()),
                    source_id: "epiphany-state-fragment".to_string(),
                    target_id: "developer-context".to_string(),
                    kind: "feeds".to_string(),
                    ..Default::default()
                }],
            },
            links: vec![EpiphanyGraphLink {
                dataflow_node_id: "epiphany-state-fragment".to_string(),
                architecture_node_id: "prompt-renderer".to_string(),
                relationship: Some("rendered by".to_string()),
                code_refs: vec![code_ref("epiphany-core/src/prompt.rs")],
            }],
        };

        let proposal = propose_map_update(EpiphanyMapProposalInput {
            state,
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect("proposal");

        assert_eq!(
            proposal.graph_frontier.active_node_ids,
            vec![
                "prompt-renderer".to_string(),
                "epiphany-state-fragment".to_string()
            ]
        );
        assert_eq!(
            proposal.graph_frontier.active_edge_ids,
            vec![
                "edge-session-prompt".to_string(),
                "edge-fragment-context".to_string()
            ]
        );
    }

    #[test]
    fn propose_map_update_returns_promotion_acceptable_patch_parts() {
        let proposal = propose_map_update(EpiphanyMapProposalInput {
            state: state_with_observation("verified"),
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect("proposal");

        let decision = evaluate_promotion(EpiphanyPromotionInput {
            has_state_replacements: true,
            active_subgoal_id: None,
            subgoals: None,
            invariants: None,
            graphs: Some(proposal.graphs),
            graph_frontier: Some(proposal.graph_frontier),
            graph_checkpoint: None,
            churn: Some(proposal.churn),
            observations: vec![proposal.observation],
            evidence: vec![proposal.evidence],
            verifier_evidence: verifier(),
        });

        assert!(decision.accepted, "{:?}", decision.reasons);
    }

    #[test]
    fn propose_map_update_rejects_unverified_observation() {
        let err = propose_map_update(EpiphanyMapProposalInput {
            state: state_with_observation("failed"),
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect_err("unverified observation should fail");

        assert!(
            err.to_string()
                .contains("must have verified/accepted status")
        );
    }

    #[test]
    fn propose_map_update_rejects_missing_code_refs() {
        let mut state = state_with_observation("ok");
        state.observations[0].code_refs.clear();

        let err = propose_map_update(EpiphanyMapProposalInput {
            state,
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect_err("code refs are required");

        assert!(err.to_string().contains("at least one code ref"));
    }
}
