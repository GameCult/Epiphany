use epiphany_state_model::EpiphanyInvestigationDisposition;
use epiphany_state_model::EpiphanyRetrievalStatus;
use epiphany_state_model::{EpiphanyMemoryProfile, RepoFrontierStatus};
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const EPIPHANY_SCENE_RECORD_LIMIT: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanySceneStateStatus {
    Missing,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanySceneSource {
    Stored,
    Live,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanySceneAction {
    Index,
    Retrieve,
    Distill,
    Context,
    Planning,
    GraphQuery,
    Jobs,
    Roles,
    Coordinator,
    Freshness,
    Pressure,
    Reorient,
    Crrc,
    ReorientLaunch,
    ReorientResult,
    ReorientAccept,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanySceneSubgoal {
    pub id: String,
    pub title: String,
    pub status: String,
    pub summary: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanySceneStatusCount {
    pub status: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanySceneGraph {
    pub architecture_node_count: u32,
    pub architecture_edge_count: u32,
    pub dataflow_node_count: u32,
    pub dataflow_edge_count: u32,
    pub link_count: u32,
    pub active_node_ids: Vec<String>,
    pub active_edge_ids: Vec<String>,
    pub open_question_count: u32,
    pub open_gap_count: u32,
    pub dirty_paths: Vec<PathBuf>,
    pub checkpoint_id: Option<String>,
    pub checkpoint_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanySceneRetrieval {
    pub workspace_root: PathBuf,
    pub status: EpiphanyRetrievalStatus,
    pub semantic_available: bool,
    pub index_revision: Option<String>,
    pub indexed_file_count: Option<u32>,
    pub indexed_chunk_count: Option<u32>,
    pub shard_count: u32,
    pub dirty_path_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanySceneInvestigationCheckpoint {
    pub checkpoint_id: String,
    pub kind: String,
    pub disposition: EpiphanyInvestigationDisposition,
    pub focus: String,
    pub summary: Option<String>,
    pub next_action: Option<String>,
    pub captured_at_turn_id: Option<String>,
    pub open_question_count: u32,
    pub code_ref_count: u32,
    pub evidence_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanySceneRecords {
    pub total_count: u32,
    pub latest: Vec<EpiphanySceneRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanySceneRecord {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub summary: String,
    pub code_ref_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanySceneChurn {
    pub understanding_status: String,
    pub diff_pressure: String,
    pub warning: Option<String>,
    pub unexplained_writes: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyScene {
    pub state_status: EpiphanySceneStateStatus,
    pub source: EpiphanySceneSource,
    pub projection_digest: Option<String>,
    pub objective: Option<String>,
    pub active_subgoal: Option<EpiphanySceneSubgoal>,
    pub subgoals: Vec<EpiphanySceneSubgoal>,
    pub invariant_status_counts: Vec<EpiphanySceneStatusCount>,
    pub graph: EpiphanySceneGraph,
    pub retrieval: Option<EpiphanySceneRetrieval>,
    pub investigation_checkpoint: Option<EpiphanySceneInvestigationCheckpoint>,
    pub observations: EpiphanySceneRecords,
    pub evidence: EpiphanySceneRecords,
    pub churn: Option<EpiphanySceneChurn>,
    pub available_actions: Vec<EpiphanySceneAction>,
}

#[derive(Debug, Clone, Copy)]
pub struct EpiphanySceneInput<'a> {
    pub mind: Option<&'a crate::EpiphanyMindView>,
    pub loaded: bool,
    pub reorientation_work_present: bool,
}

pub fn derive_scene(input: EpiphanySceneInput<'_>) -> EpiphanyScene {
    let source = if input.loaded {
        EpiphanySceneSource::Live
    } else {
        EpiphanySceneSource::Stored
    };
    let mind = input.mind;
    let available_actions = scene_available_actions(
        input.loaded,
        mind.is_some(),
        mind.and_then(|mind| mind.investigation_checkpoint.as_ref())
            .is_some(),
        input.reorientation_work_present,
    );
    let Some(mind) = mind else {
        return EpiphanyScene {
            state_status: EpiphanySceneStateStatus::Missing,
            source,
            projection_digest: None,
            objective: None,
            active_subgoal: None,
            subgoals: Vec::new(),
            invariant_status_counts: Vec::new(),
            graph: EpiphanySceneGraph::default(),
            retrieval: None,
            investigation_checkpoint: None,
            observations: EpiphanySceneRecords::default(),
            evidence: EpiphanySceneRecords::default(),
            churn: None,
            available_actions,
        };
    };

    let subgoals = scene_subgoals(mind);
    let active_subgoal = subgoals.iter().find(|subgoal| subgoal.active).cloned();

    EpiphanyScene {
        state_status: EpiphanySceneStateStatus::Ready,
        source,
        projection_digest: Some(mind.projection_digest.clone()),
        objective: mind.objective.clone(),
        active_subgoal,
        subgoals,
        invariant_status_counts: status_counts(
            mind.invariants.iter().map(|item| item.status.as_str()),
        ),
        graph: scene_graph(mind),
        retrieval: None,
        investigation_checkpoint: mind.investigation_checkpoint.as_ref().map(|checkpoint| {
            EpiphanySceneInvestigationCheckpoint {
                checkpoint_id: checkpoint.checkpoint_id.clone(),
                kind: checkpoint.kind.clone(),
                disposition: checkpoint.disposition,
                focus: checkpoint.focus.clone(),
                summary: checkpoint.summary.clone(),
                next_action: checkpoint.next_action.clone(),
                captured_at_turn_id: checkpoint.captured_at_turn_id.clone(),
                open_question_count: checkpoint.open_questions.len() as u32,
                code_ref_count: checkpoint.code_refs.len() as u32,
                evidence_count: checkpoint.evidence_ids.len() as u32,
            }
        }),
        observations: EpiphanySceneRecords {
            total_count: mind.observations.len() as u32,
            latest: mind
                .observations
                .iter()
                .take(EPIPHANY_SCENE_RECORD_LIMIT)
                .map(|observation| EpiphanySceneRecord {
                    id: observation.id.clone(),
                    kind: observation.source_kind.clone(),
                    status: observation.status.clone(),
                    summary: observation.summary.clone(),
                    code_ref_count: observation.code_refs.len() as u32,
                })
                .collect(),
        },
        evidence: EpiphanySceneRecords {
            total_count: mind.evidence.len() as u32,
            latest: mind
                .evidence
                .iter()
                .take(EPIPHANY_SCENE_RECORD_LIMIT)
                .map(|evidence| EpiphanySceneRecord {
                    id: evidence.id.clone(),
                    kind: evidence.kind.clone(),
                    status: evidence.status.clone(),
                    summary: evidence.summary.clone(),
                    code_ref_count: evidence.code_refs.len() as u32,
                })
                .collect(),
        },
        churn: None,
        available_actions,
    }
}

fn scene_available_actions(
    loaded: bool,
    state_present: bool,
    checkpoint_present: bool,
    reorient_binding_present: bool,
) -> Vec<EpiphanySceneAction> {
    if !loaded {
        return Vec::new();
    }

    let mut actions = vec![
        EpiphanySceneAction::Index,
        EpiphanySceneAction::Retrieve,
        EpiphanySceneAction::Distill,
        EpiphanySceneAction::Context,
        EpiphanySceneAction::Planning,
        EpiphanySceneAction::GraphQuery,
        EpiphanySceneAction::Jobs,
        EpiphanySceneAction::Roles,
        EpiphanySceneAction::Coordinator,
        EpiphanySceneAction::Freshness,
        EpiphanySceneAction::Pressure,
        EpiphanySceneAction::Reorient,
        EpiphanySceneAction::Crrc,
    ];
    if checkpoint_present {
        actions.push(EpiphanySceneAction::ReorientLaunch);
    }
    if reorient_binding_present {
        actions.push(EpiphanySceneAction::ReorientResult);
        actions.push(EpiphanySceneAction::ReorientAccept);
    }
    let _ = state_present;
    actions
}

fn scene_subgoals(mind: &crate::EpiphanyMindView) -> Vec<EpiphanySceneSubgoal> {
    let active_id = mind.active_subgoal_id.as_deref();
    mind.subgoals
        .iter()
        .map(|subgoal| EpiphanySceneSubgoal {
            id: subgoal.id.clone(),
            title: subgoal.title.clone(),
            status: subgoal.status.clone(),
            summary: subgoal.summary.clone(),
            active: active_id == Some(subgoal.id.as_str()),
        })
        .collect()
}

fn scene_graph(mind: &crate::EpiphanyMindView) -> EpiphanySceneGraph {
    let Some(model) = mind.repo_model.as_ref() else {
        return EpiphanySceneGraph::default();
    };
    let active = model
        .frontier
        .iter()
        .filter(|item| {
            matches!(
                item.status,
                RepoFrontierStatus::Proposed
                    | RepoFrontierStatus::Active
                    | RepoFrontierStatus::Blocked
            )
        })
        .collect::<Vec<_>>();
    EpiphanySceneGraph {
        architecture_node_count: model
            .nodes
            .iter()
            .filter(|node| node.profile == EpiphanyMemoryProfile::RepoArchitecture)
            .count() as u32,
        architecture_edge_count: model
            .edges
            .iter()
            .filter(|edge| edge.profile == EpiphanyMemoryProfile::RepoArchitecture)
            .count() as u32,
        dataflow_node_count: model
            .nodes
            .iter()
            .filter(|node| node.profile == EpiphanyMemoryProfile::RepoDataflow)
            .count() as u32,
        dataflow_edge_count: model
            .edges
            .iter()
            .filter(|edge| edge.profile == EpiphanyMemoryProfile::RepoDataflow)
            .count() as u32,
        link_count: model.edges.len() as u32,
        active_node_ids: active
            .iter()
            .flat_map(|item| item.target_claim_ids.iter().cloned())
            .collect(),
        active_edge_ids: Vec::new(),
        open_question_count: active.len() as u32,
        open_gap_count: active
            .iter()
            .filter(|item| item.status == RepoFrontierStatus::Blocked)
            .count() as u32,
        dirty_paths: Vec::new(),
        checkpoint_id: mind
            .investigation_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_id.clone()),
        checkpoint_summary: mind
            .investigation_checkpoint
            .as_ref()
            .and_then(|checkpoint| checkpoint.summary.clone()),
    }
}

fn status_counts<'a>(statuses: impl Iterator<Item = &'a str>) -> Vec<EpiphanySceneStatusCount> {
    let mut counts = BTreeMap::<String, u32>::new();
    for status in statuses {
        *counts.entry(status.to_string()).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(status, count)| EpiphanySceneStatusCount { status, count })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_state_model::EpiphanyInvariant;
    use epiphany_state_model::EpiphanyPlanningState;
    use epiphany_state_model::EpiphanySubgoal;

    #[test]
    fn missing_stored_scene_has_no_actions() {
        let scene = derive_scene(EpiphanySceneInput {
            mind: None,
            loaded: false,
            reorientation_work_present: false,
        });

        assert_eq!(scene.state_status, EpiphanySceneStateStatus::Missing);
        assert_eq!(scene.source, EpiphanySceneSource::Stored);
        assert!(scene.available_actions.is_empty());
    }

    #[test]
    fn live_scene_projects_state_and_actions() {
        let mind = crate::EpiphanyMindView {
            schema_epoch: crate::MIND_SCHEMA_EPOCH.into(),
            runtime_id: "runtime-1".into(),
            projection_digest: "sha256:mind-4".into(),
            source_documents: Vec::new(),
            objective: Some("purify".to_string()),
            active_subgoal_id: Some("subgoal-1".to_string()),
            subgoals: vec![EpiphanySubgoal {
                id: "subgoal-1".to_string(),
                title: "cut".to_string(),
                status: "active".to_string(),
                summary: Some("current cut".to_string()),
            }],
            invariants: vec![
                EpiphanyInvariant {
                    id: "inv-1".to_string(),
                    description: "ready".to_string(),
                    status: "ready".to_string(),
                    rationale: None,
                },
                EpiphanyInvariant {
                    id: "inv-2".to_string(),
                    description: "ready".to_string(),
                    status: "ready".to_string(),
                    rationale: None,
                },
            ],
            observations: Vec::new(),
            evidence: Vec::new(),
            verification_audits: Vec::new(),
            reorientation_decisions: Vec::new(),
            reorientation_failures: Vec::new(),
            investigation_checkpoint: None,
            mode: None,
            planning: EpiphanyPlanningState::default(),
            repository_body_observation: None,
            repo_model: None,
        };

        let scene = derive_scene(EpiphanySceneInput {
            mind: Some(&mind),
            loaded: true,
            reorientation_work_present: true,
        });

        assert_eq!(scene.state_status, EpiphanySceneStateStatus::Ready);
        assert_eq!(scene.projection_digest.as_deref(), Some("sha256:mind-4"));
        assert_eq!(
            scene.active_subgoal.as_ref().map(|item| item.active),
            Some(true)
        );
        assert_eq!(scene.invariant_status_counts[0].count, 2);
        assert!(
            scene
                .available_actions
                .contains(&EpiphanySceneAction::ReorientResult)
        );
    }
}
