use anyhow::Result;
use anyhow::anyhow;
use codex_protocol::protocol::EpiphanyChurnState;
use codex_protocol::protocol::EpiphanyCodeRef;
use codex_protocol::protocol::EpiphanyEvidenceRecord;
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

    for path in unique_code_ref_paths(&code_refs) {
        let node_id = graph_node_id(&path);
        active_node_ids.push(node_id.clone());
        if !graphs
            .architecture
            .nodes
            .iter()
            .any(|node| node.id == node_id)
        {
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
                code_refs: code_refs_for_path(&code_refs, &path),
            });
        }
    }

    let mut frontier = input.state.graph_frontier.clone().unwrap_or_default();
    merge_unique(&mut frontier.active_node_ids, active_node_ids);
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
                "Proposed graph frontier and churn update from verified observations: {}",
                observation_ids.join(", ")
            ),
            SUMMARY_LIMIT,
        ),
        code_refs,
    };
    let churn = EpiphanyChurnState {
        understanding_status: "proposal_ready".to_string(),
        diff_pressure: input
            .state
            .churn
            .as_ref()
            .map(|churn| churn.diff_pressure.trim())
            .filter(|value| !value.is_empty())
            .unwrap_or("low")
            .to_string(),
        graph_freshness: Some("proposal".to_string()),
        warning: Some(
            "Map/churn proposal derived from verified observations; promote only after verifier acceptance."
                .to_string(),
        ),
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
        assert_eq!(proposal.churn.understanding_status, "proposal_ready");
        assert_eq!(proposal.churn.diff_pressure, "medium");
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
