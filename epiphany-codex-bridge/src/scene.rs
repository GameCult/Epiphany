use codex_app_server_protocol::ThreadEpiphanyScene;
use codex_app_server_protocol::ThreadEpiphanySceneAction;
use codex_app_server_protocol::ThreadEpiphanySceneChurn;
use codex_app_server_protocol::ThreadEpiphanySceneGraph;
use codex_app_server_protocol::ThreadEpiphanySceneInvestigationCheckpoint;
use codex_app_server_protocol::ThreadEpiphanySceneRecord;
use codex_app_server_protocol::ThreadEpiphanySceneRecords;
use codex_app_server_protocol::ThreadEpiphanySceneRetrieval;
use codex_app_server_protocol::ThreadEpiphanySceneSource;
use codex_app_server_protocol::ThreadEpiphanySceneStateStatus;
use codex_app_server_protocol::ThreadEpiphanySceneStatusCount;
use codex_app_server_protocol::ThreadEpiphanySceneSubgoal;
use codex_protocol::protocol::EpiphanyThreadState;
use epiphany_core::EpiphanyScene as CoreEpiphanyScene;
use epiphany_core::EpiphanySceneAction as CoreEpiphanySceneAction;
use epiphany_core::EpiphanySceneChurn as CoreEpiphanySceneChurn;
use epiphany_core::EpiphanySceneGraph as CoreEpiphanySceneGraph;
use epiphany_core::EpiphanySceneInput;
use epiphany_core::EpiphanySceneInvestigationCheckpoint as CoreEpiphanySceneInvestigationCheckpoint;
use epiphany_core::EpiphanySceneRecord as CoreEpiphanySceneRecord;
use epiphany_core::EpiphanySceneRecords as CoreEpiphanySceneRecords;
use epiphany_core::EpiphanySceneRetrieval as CoreEpiphanySceneRetrieval;
use epiphany_core::EpiphanySceneSource as CoreEpiphanySceneSource;
use epiphany_core::EpiphanySceneStateStatus as CoreEpiphanySceneStateStatus;
use epiphany_core::EpiphanySceneStatusCount as CoreEpiphanySceneStatusCount;
use epiphany_core::EpiphanySceneSubgoal as CoreEpiphanySceneSubgoal;
use epiphany_core::derive_scene;

pub fn map_epiphany_scene(
    state: Option<&EpiphanyThreadState>,
    loaded: bool,
    reorient_binding_id: &str,
) -> ThreadEpiphanyScene {
    map_core_epiphany_scene(derive_scene(EpiphanySceneInput {
        state,
        loaded,
        reorient_binding_id,
    }))
}

fn map_core_epiphany_scene(scene: CoreEpiphanyScene) -> ThreadEpiphanyScene {
    ThreadEpiphanyScene {
        state_status: match scene.state_status {
            CoreEpiphanySceneStateStatus::Missing => ThreadEpiphanySceneStateStatus::Missing,
            CoreEpiphanySceneStateStatus::Ready => ThreadEpiphanySceneStateStatus::Ready,
        },
        source: match scene.source {
            CoreEpiphanySceneSource::Stored => ThreadEpiphanySceneSource::Stored,
            CoreEpiphanySceneSource::Live => ThreadEpiphanySceneSource::Live,
        },
        revision: scene.revision,
        objective: scene.objective,
        active_subgoal: scene.active_subgoal.map(map_core_epiphany_scene_subgoal),
        subgoals: scene
            .subgoals
            .into_iter()
            .map(map_core_epiphany_scene_subgoal)
            .collect(),
        invariant_status_counts: scene
            .invariant_status_counts
            .into_iter()
            .map(map_core_epiphany_scene_status_count)
            .collect(),
        graph: map_core_epiphany_scene_graph(scene.graph),
        retrieval: scene.retrieval.map(map_core_epiphany_scene_retrieval),
        investigation_checkpoint: scene
            .investigation_checkpoint
            .map(map_core_epiphany_scene_investigation_checkpoint),
        observations: map_core_epiphany_scene_records(scene.observations),
        evidence: map_core_epiphany_scene_records(scene.evidence),
        churn: scene.churn.map(map_core_epiphany_scene_churn),
        available_actions: scene
            .available_actions
            .into_iter()
            .map(map_core_epiphany_scene_action)
            .collect(),
    }
}

pub fn map_core_epiphany_scene_action(
    action: CoreEpiphanySceneAction,
) -> ThreadEpiphanySceneAction {
    match action {
        CoreEpiphanySceneAction::Index => ThreadEpiphanySceneAction::Index,
        CoreEpiphanySceneAction::Retrieve => ThreadEpiphanySceneAction::Retrieve,
        CoreEpiphanySceneAction::Distill => ThreadEpiphanySceneAction::Distill,
        CoreEpiphanySceneAction::Context => ThreadEpiphanySceneAction::Context,
        CoreEpiphanySceneAction::Planning => ThreadEpiphanySceneAction::Planning,
        CoreEpiphanySceneAction::GraphQuery => ThreadEpiphanySceneAction::GraphQuery,
        CoreEpiphanySceneAction::Jobs => ThreadEpiphanySceneAction::Jobs,
        CoreEpiphanySceneAction::Roles => ThreadEpiphanySceneAction::Roles,
        CoreEpiphanySceneAction::Coordinator => ThreadEpiphanySceneAction::Coordinator,
        CoreEpiphanySceneAction::RoleLaunch => ThreadEpiphanySceneAction::RoleLaunch,
        CoreEpiphanySceneAction::RoleResult => ThreadEpiphanySceneAction::RoleResult,
        CoreEpiphanySceneAction::RoleAccept => ThreadEpiphanySceneAction::RoleAccept,
        CoreEpiphanySceneAction::JobLaunch => ThreadEpiphanySceneAction::JobLaunch,
        CoreEpiphanySceneAction::JobInterrupt => ThreadEpiphanySceneAction::JobInterrupt,
        CoreEpiphanySceneAction::Freshness => ThreadEpiphanySceneAction::Freshness,
        CoreEpiphanySceneAction::Pressure => ThreadEpiphanySceneAction::Pressure,
        CoreEpiphanySceneAction::Reorient => ThreadEpiphanySceneAction::Reorient,
        CoreEpiphanySceneAction::Crrc => ThreadEpiphanySceneAction::Crrc,
        CoreEpiphanySceneAction::ReorientLaunch => ThreadEpiphanySceneAction::ReorientLaunch,
        CoreEpiphanySceneAction::ReorientResult => ThreadEpiphanySceneAction::ReorientResult,
        CoreEpiphanySceneAction::ReorientAccept => ThreadEpiphanySceneAction::ReorientAccept,
        CoreEpiphanySceneAction::Propose => ThreadEpiphanySceneAction::Propose,
        CoreEpiphanySceneAction::Promote => ThreadEpiphanySceneAction::Promote,
        CoreEpiphanySceneAction::Update => ThreadEpiphanySceneAction::Update,
    }
}

fn map_core_epiphany_scene_subgoal(
    subgoal: CoreEpiphanySceneSubgoal,
) -> ThreadEpiphanySceneSubgoal {
    ThreadEpiphanySceneSubgoal {
        id: subgoal.id,
        title: subgoal.title,
        status: subgoal.status,
        summary: subgoal.summary,
        active: subgoal.active,
    }
}

fn map_core_epiphany_scene_status_count(
    count: CoreEpiphanySceneStatusCount,
) -> ThreadEpiphanySceneStatusCount {
    ThreadEpiphanySceneStatusCount {
        status: count.status,
        count: count.count,
    }
}

fn map_core_epiphany_scene_graph(graph: CoreEpiphanySceneGraph) -> ThreadEpiphanySceneGraph {
    ThreadEpiphanySceneGraph {
        architecture_node_count: graph.architecture_node_count,
        architecture_edge_count: graph.architecture_edge_count,
        dataflow_node_count: graph.dataflow_node_count,
        dataflow_edge_count: graph.dataflow_edge_count,
        link_count: graph.link_count,
        active_node_ids: graph.active_node_ids,
        active_edge_ids: graph.active_edge_ids,
        open_question_count: graph.open_question_count,
        open_gap_count: graph.open_gap_count,
        dirty_paths: graph.dirty_paths,
        checkpoint_id: graph.checkpoint_id,
        checkpoint_summary: graph.checkpoint_summary,
    }
}

fn map_core_epiphany_scene_retrieval(
    retrieval: CoreEpiphanySceneRetrieval,
) -> ThreadEpiphanySceneRetrieval {
    ThreadEpiphanySceneRetrieval {
        workspace_root: retrieval.workspace_root,
        status: retrieval.status,
        semantic_available: retrieval.semantic_available,
        index_revision: retrieval.index_revision,
        indexed_file_count: retrieval.indexed_file_count,
        indexed_chunk_count: retrieval.indexed_chunk_count,
        shard_count: retrieval.shard_count,
        dirty_path_count: retrieval.dirty_path_count,
    }
}

fn map_core_epiphany_scene_investigation_checkpoint(
    checkpoint: CoreEpiphanySceneInvestigationCheckpoint,
) -> ThreadEpiphanySceneInvestigationCheckpoint {
    ThreadEpiphanySceneInvestigationCheckpoint {
        checkpoint_id: checkpoint.checkpoint_id,
        kind: checkpoint.kind,
        disposition: checkpoint.disposition,
        focus: checkpoint.focus,
        summary: checkpoint.summary,
        next_action: checkpoint.next_action,
        captured_at_turn_id: checkpoint.captured_at_turn_id,
        open_question_count: checkpoint.open_question_count,
        code_ref_count: checkpoint.code_ref_count,
        evidence_count: checkpoint.evidence_count,
    }
}

fn map_core_epiphany_scene_records(
    records: CoreEpiphanySceneRecords,
) -> ThreadEpiphanySceneRecords {
    ThreadEpiphanySceneRecords {
        total_count: records.total_count,
        latest: records
            .latest
            .into_iter()
            .map(map_core_epiphany_scene_record)
            .collect(),
    }
}

fn map_core_epiphany_scene_record(record: CoreEpiphanySceneRecord) -> ThreadEpiphanySceneRecord {
    ThreadEpiphanySceneRecord {
        id: record.id,
        kind: record.kind,
        status: record.status,
        summary: record.summary,
        code_ref_count: record.code_ref_count,
    }
}

fn map_core_epiphany_scene_churn(churn: CoreEpiphanySceneChurn) -> ThreadEpiphanySceneChurn {
    ThreadEpiphanySceneChurn {
        understanding_status: churn.understanding_status,
        diff_pressure: churn.diff_pressure,
        graph_freshness: churn.graph_freshness,
        warning: churn.warning,
        unexplained_writes: churn.unexplained_writes,
    }
}
