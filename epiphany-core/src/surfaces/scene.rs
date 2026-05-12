use codex_protocol::protocol::EpiphanyInvestigationDisposition;
use codex_protocol::protocol::EpiphanyRetrievalStatus;
use codex_protocol::protocol::EpiphanyThreadState;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const EPIPHANY_SCENE_RECORD_LIMIT: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanySceneStateStatus {
    Missing,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanySceneSource {
    Stored,
    Live,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    RoleLaunch,
    RoleResult,
    RoleAccept,
    JobLaunch,
    JobInterrupt,
    Freshness,
    Pressure,
    Reorient,
    Crrc,
    ReorientLaunch,
    ReorientResult,
    ReorientAccept,
    Propose,
    Promote,
    Update,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanySceneSubgoal {
    pub id: String,
    pub title: String,
    pub status: String,
    pub summary: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanySceneStatusCount {
    pub status: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EpiphanySceneRecords {
    pub total_count: u32,
    pub latest: Vec<EpiphanySceneRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanySceneRecord {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub summary: String,
    pub code_ref_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanySceneChurn {
    pub understanding_status: String,
    pub diff_pressure: String,
    pub graph_freshness: Option<String>,
    pub warning: Option<String>,
    pub unexplained_writes: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyScene {
    pub state_status: EpiphanySceneStateStatus,
    pub source: EpiphanySceneSource,
    pub revision: Option<u64>,
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
    pub state: Option<&'a EpiphanyThreadState>,
    pub loaded: bool,
    pub reorient_binding_id: &'a str,
}

pub fn derive_scene(input: EpiphanySceneInput<'_>) -> EpiphanyScene {
    let source = if input.loaded {
        EpiphanySceneSource::Live
    } else {
        EpiphanySceneSource::Stored
    };
    let state = input.state;
    let available_actions = scene_available_actions(
        input.loaded,
        state.is_some(),
        state
            .and_then(|state| state.investigation_checkpoint.as_ref())
            .is_some(),
        state.is_some_and(|state| {
            state
                .job_bindings
                .iter()
                .any(|binding| binding.id == input.reorient_binding_id)
        }),
    );
    let Some(state) = state else {
        return EpiphanyScene {
            state_status: EpiphanySceneStateStatus::Missing,
            source,
            revision: None,
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

    let subgoals = scene_subgoals(state);
    let active_subgoal = subgoals.iter().find(|subgoal| subgoal.active).cloned();

    EpiphanyScene {
        state_status: EpiphanySceneStateStatus::Ready,
        source,
        revision: Some(state.revision),
        objective: state.objective.clone(),
        active_subgoal,
        subgoals,
        invariant_status_counts: status_counts(
            state.invariants.iter().map(|item| item.status.as_str()),
        ),
        graph: scene_graph(state),
        retrieval: state
            .retrieval
            .as_ref()
            .map(|retrieval| EpiphanySceneRetrieval {
                workspace_root: retrieval.workspace_root.clone(),
                status: retrieval.status,
                semantic_available: retrieval.semantic_available,
                index_revision: retrieval.index_revision.clone(),
                indexed_file_count: retrieval.indexed_file_count,
                indexed_chunk_count: retrieval.indexed_chunk_count,
                shard_count: retrieval.shards.len() as u32,
                dirty_path_count: retrieval.dirty_paths.len() as u32,
            }),
        investigation_checkpoint: state.investigation_checkpoint.as_ref().map(|checkpoint| {
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
            total_count: state.observations.len() as u32,
            latest: state
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
            total_count: state.recent_evidence.len() as u32,
            latest: state
                .recent_evidence
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
        churn: state.churn.as_ref().map(|churn| EpiphanySceneChurn {
            understanding_status: churn.understanding_status.clone(),
            diff_pressure: churn.diff_pressure.clone(),
            graph_freshness: churn.graph_freshness.clone(),
            warning: churn.warning.clone(),
            unexplained_writes: churn.unexplained_writes,
        }),
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
        EpiphanySceneAction::RoleLaunch,
        EpiphanySceneAction::RoleResult,
        EpiphanySceneAction::RoleAccept,
        EpiphanySceneAction::JobLaunch,
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
    actions.push(EpiphanySceneAction::Update);
    if state_present {
        actions.push(EpiphanySceneAction::JobInterrupt);
        actions.push(EpiphanySceneAction::Propose);
        actions.push(EpiphanySceneAction::Promote);
    }
    actions
}

fn scene_subgoals(state: &EpiphanyThreadState) -> Vec<EpiphanySceneSubgoal> {
    let active_id = state.active_subgoal_id.as_deref();
    state
        .subgoals
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

fn scene_graph(state: &EpiphanyThreadState) -> EpiphanySceneGraph {
    let frontier = state.graph_frontier.as_ref();
    let checkpoint = state.graph_checkpoint.as_ref();
    EpiphanySceneGraph {
        architecture_node_count: state.graphs.architecture.nodes.len() as u32,
        architecture_edge_count: state.graphs.architecture.edges.len() as u32,
        dataflow_node_count: state.graphs.dataflow.nodes.len() as u32,
        dataflow_edge_count: state.graphs.dataflow.edges.len() as u32,
        link_count: state.graphs.links.len() as u32,
        active_node_ids: frontier
            .map(|frontier| frontier.active_node_ids.clone())
            .unwrap_or_default(),
        active_edge_ids: frontier
            .map(|frontier| frontier.active_edge_ids.clone())
            .unwrap_or_default(),
        open_question_count: frontier
            .map(|frontier| frontier.open_question_ids.len() as u32)
            .unwrap_or_default(),
        open_gap_count: frontier
            .map(|frontier| frontier.open_gap_ids.len() as u32)
            .unwrap_or_default(),
        dirty_paths: frontier
            .map(|frontier| frontier.dirty_paths.clone())
            .unwrap_or_default(),
        checkpoint_id: checkpoint.map(|checkpoint| checkpoint.checkpoint_id.clone()),
        checkpoint_summary: checkpoint.and_then(|checkpoint| checkpoint.summary.clone()),
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
    use codex_protocol::protocol::EpiphanyInvariant;
    use codex_protocol::protocol::EpiphanyJobBinding;
    use codex_protocol::protocol::EpiphanyJobKind;
    use codex_protocol::protocol::EpiphanySubgoal;

    #[test]
    fn missing_stored_scene_has_no_actions() {
        let scene = derive_scene(EpiphanySceneInput {
            state: None,
            loaded: false,
            reorient_binding_id: "reorient-worker",
        });

        assert_eq!(scene.state_status, EpiphanySceneStateStatus::Missing);
        assert_eq!(scene.source, EpiphanySceneSource::Stored);
        assert!(scene.available_actions.is_empty());
    }

    #[test]
    fn live_scene_projects_state_and_actions() {
        let state = EpiphanyThreadState {
            revision: 4,
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
            job_bindings: vec![EpiphanyJobBinding {
                id: "reorient-worker".to_string(),
                kind: EpiphanyJobKind::Specialist,
                scope: "reorient".to_string(),
                owner_role: "life".to_string(),
                authority_scope: None,
                linked_subgoal_ids: Vec::new(),
                linked_graph_node_ids: Vec::new(),
                blocking_reason: None,
            }],
            ..Default::default()
        };

        let scene = derive_scene(EpiphanySceneInput {
            state: Some(&state),
            loaded: true,
            reorient_binding_id: "reorient-worker",
        });

        assert_eq!(scene.state_status, EpiphanySceneStateStatus::Ready);
        assert_eq!(scene.revision, Some(4));
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
        assert!(
            scene
                .available_actions
                .contains(&EpiphanySceneAction::Promote)
        );
    }
}
