use codex_app_server_protocol::ThreadEpiphanyGraphFreshness;
use codex_app_server_protocol::ThreadEpiphanyGraphFreshnessStatus;
use codex_app_server_protocol::ThreadEpiphanyInvalidationInput;
use codex_app_server_protocol::ThreadEpiphanyInvalidationStatus;
use codex_app_server_protocol::ThreadEpiphanyPressure;
use codex_app_server_protocol::ThreadEpiphanyPressureLevel;
use codex_app_server_protocol::ThreadEpiphanyReorientAction;
use codex_app_server_protocol::ThreadEpiphanyReorientCheckpointStatus;
use codex_app_server_protocol::ThreadEpiphanyReorientDecision;
use codex_app_server_protocol::ThreadEpiphanyReorientReason;
use codex_app_server_protocol::ThreadEpiphanyReorientStateStatus;
use codex_app_server_protocol::ThreadEpiphanyRetrievalFreshness;
use codex_app_server_protocol::ThreadEpiphanyRetrievalFreshnessStatus;
use codex_protocol::protocol::EpiphanyRetrievalState;
use codex_protocol::protocol::EpiphanyThreadState;
use epiphany_core::EpiphanyFreshnessInput;
use epiphany_core::EpiphanyFreshnessView;
use epiphany_core::EpiphanyFreshnessWatcherInput;
use epiphany_core::EpiphanyGraphFreshness as CoreEpiphanyGraphFreshness;
use epiphany_core::EpiphanyGraphFreshnessStatus as CoreEpiphanyGraphFreshnessStatus;
use epiphany_core::EpiphanyInvalidationInput as CoreEpiphanyInvalidationInput;
use epiphany_core::EpiphanyInvalidationStatus as CoreEpiphanyInvalidationStatus;
use epiphany_core::EpiphanyReorientAction as CoreEpiphanyReorientAction;
use epiphany_core::EpiphanyReorientCheckpointStatus as CoreEpiphanyReorientCheckpointStatus;
use epiphany_core::EpiphanyReorientFreshnessStatus as CoreEpiphanyReorientFreshnessStatus;
use epiphany_core::EpiphanyReorientInput;
use epiphany_core::EpiphanyReorientPressureLevel as CoreEpiphanyReorientPressureLevel;
use epiphany_core::EpiphanyReorientReason as CoreEpiphanyReorientReason;
use epiphany_core::EpiphanyReorientStateStatus as CoreEpiphanyReorientStateStatus;
use epiphany_core::EpiphanyRetrievalFreshness as CoreEpiphanyRetrievalFreshness;
use epiphany_core::EpiphanyRetrievalFreshnessStatus as CoreEpiphanyRetrievalFreshnessStatus;
use epiphany_core::derive_freshness;
use epiphany_core::recommend_reorientation;

use crate::epiphany_invalidation::EpiphanyInvalidationSnapshot;

pub(super) fn map_epiphany_freshness(
    state: Option<&EpiphanyThreadState>,
    retrieval_override: Option<&EpiphanyRetrievalState>,
    watcher_snapshot: Option<&EpiphanyInvalidationSnapshot>,
) -> (
    Option<u64>,
    ThreadEpiphanyRetrievalFreshness,
    ThreadEpiphanyGraphFreshness,
    ThreadEpiphanyInvalidationInput,
) {
    let watcher = watcher_snapshot.map(|snapshot| EpiphanyFreshnessWatcherInput {
        available: snapshot.available,
        workspace_root: snapshot.workspace_root.as_deref(),
        observed_at_unix_seconds: snapshot.observed_at_unix_seconds,
        changed_paths: snapshot.changed_paths.as_slice(),
    });
    let freshness = derive_freshness(EpiphanyFreshnessInput {
        state,
        retrieval_override,
        watcher,
    });
    map_core_epiphany_freshness(freshness)
}

fn map_core_epiphany_freshness(
    freshness: EpiphanyFreshnessView,
) -> (
    Option<u64>,
    ThreadEpiphanyRetrievalFreshness,
    ThreadEpiphanyGraphFreshness,
    ThreadEpiphanyInvalidationInput,
) {
    (
        freshness.state_revision,
        map_core_epiphany_retrieval_freshness(freshness.retrieval),
        map_core_epiphany_graph_freshness(freshness.graph),
        map_core_epiphany_invalidation_input(freshness.watcher),
    )
}

fn map_core_epiphany_retrieval_freshness(
    retrieval: CoreEpiphanyRetrievalFreshness,
) -> ThreadEpiphanyRetrievalFreshness {
    ThreadEpiphanyRetrievalFreshness {
        status: match retrieval.status {
            CoreEpiphanyRetrievalFreshnessStatus::Missing => {
                ThreadEpiphanyRetrievalFreshnessStatus::Missing
            }
            CoreEpiphanyRetrievalFreshnessStatus::Ready => {
                ThreadEpiphanyRetrievalFreshnessStatus::Ready
            }
            CoreEpiphanyRetrievalFreshnessStatus::Stale => {
                ThreadEpiphanyRetrievalFreshnessStatus::Stale
            }
            CoreEpiphanyRetrievalFreshnessStatus::Indexing => {
                ThreadEpiphanyRetrievalFreshnessStatus::Indexing
            }
            CoreEpiphanyRetrievalFreshnessStatus::Unavailable => {
                ThreadEpiphanyRetrievalFreshnessStatus::Unavailable
            }
        },
        semantic_available: retrieval.semantic_available,
        last_indexed_at_unix_seconds: retrieval.last_indexed_at_unix_seconds,
        indexed_file_count: retrieval.indexed_file_count,
        indexed_chunk_count: retrieval.indexed_chunk_count,
        dirty_paths: retrieval.dirty_paths,
        note: retrieval.note,
    }
}

fn map_core_epiphany_graph_freshness(
    graph: CoreEpiphanyGraphFreshness,
) -> ThreadEpiphanyGraphFreshness {
    ThreadEpiphanyGraphFreshness {
        status: match graph.status {
            CoreEpiphanyGraphFreshnessStatus::Missing => {
                ThreadEpiphanyGraphFreshnessStatus::Missing
            }
            CoreEpiphanyGraphFreshnessStatus::Ready => ThreadEpiphanyGraphFreshnessStatus::Ready,
            CoreEpiphanyGraphFreshnessStatus::Stale => ThreadEpiphanyGraphFreshnessStatus::Stale,
        },
        graph_freshness: graph.graph_freshness,
        checkpoint_id: graph.checkpoint_id,
        dirty_path_count: graph.dirty_path_count,
        dirty_paths: graph.dirty_paths,
        open_question_count: graph.open_question_count,
        open_gap_count: graph.open_gap_count,
        note: graph.note,
    }
}

fn map_core_epiphany_invalidation_input(
    watcher: CoreEpiphanyInvalidationInput,
) -> ThreadEpiphanyInvalidationInput {
    ThreadEpiphanyInvalidationInput {
        status: match watcher.status {
            CoreEpiphanyInvalidationStatus::Unavailable => {
                ThreadEpiphanyInvalidationStatus::Unavailable
            }
            CoreEpiphanyInvalidationStatus::Clean => ThreadEpiphanyInvalidationStatus::Clean,
            CoreEpiphanyInvalidationStatus::Changed => ThreadEpiphanyInvalidationStatus::Changed,
        },
        watched_root: watcher.watched_root,
        observed_at_unix_seconds: watcher.observed_at_unix_seconds,
        changed_path_count: watcher.changed_path_count,
        changed_paths: watcher.changed_paths,
        graph_node_ids: watcher.graph_node_ids,
        active_frontier_node_ids: watcher.active_frontier_node_ids,
        note: watcher.note,
    }
}

pub(super) fn map_epiphany_reorient(
    state: Option<&EpiphanyThreadState>,
    pressure: &ThreadEpiphanyPressure,
    retrieval: &ThreadEpiphanyRetrievalFreshness,
    graph: &ThreadEpiphanyGraphFreshness,
    watcher: &ThreadEpiphanyInvalidationInput,
) -> (
    ThreadEpiphanyReorientStateStatus,
    ThreadEpiphanyReorientDecision,
) {
    let (state_status, decision) = recommend_reorientation(EpiphanyReorientInput {
        checkpoint: state.and_then(|state| state.investigation_checkpoint.as_ref()),
        state_present: state.is_some(),
        pressure_level: map_core_reorient_pressure_level(pressure.level),
        retrieval_status: map_core_reorient_retrieval_status(retrieval.status),
        retrieval_dirty_paths: retrieval.dirty_paths.clone(),
        graph_status: map_core_reorient_graph_status(graph.status),
        graph_dirty_paths: graph.dirty_paths.clone(),
        watcher_status: map_core_reorient_watcher_status(watcher.status),
        watcher_changed_paths: watcher.changed_paths.clone(),
        watcher_graph_node_ids: watcher.graph_node_ids.clone(),
        active_frontier_node_ids: watcher.active_frontier_node_ids.clone(),
        watched_root: watcher.watched_root.clone(),
    });
    (
        map_protocol_reorient_state_status(state_status),
        map_protocol_reorient_decision(decision),
    )
}

fn map_core_reorient_pressure_level(
    level: ThreadEpiphanyPressureLevel,
) -> CoreEpiphanyReorientPressureLevel {
    match level {
        ThreadEpiphanyPressureLevel::Unknown => CoreEpiphanyReorientPressureLevel::Unknown,
        ThreadEpiphanyPressureLevel::Low => CoreEpiphanyReorientPressureLevel::Low,
        ThreadEpiphanyPressureLevel::Elevated => CoreEpiphanyReorientPressureLevel::Medium,
        ThreadEpiphanyPressureLevel::High => CoreEpiphanyReorientPressureLevel::High,
        ThreadEpiphanyPressureLevel::Critical => CoreEpiphanyReorientPressureLevel::Critical,
    }
}

fn map_core_reorient_retrieval_status(
    status: ThreadEpiphanyRetrievalFreshnessStatus,
) -> CoreEpiphanyReorientFreshnessStatus {
    match status {
        ThreadEpiphanyRetrievalFreshnessStatus::Missing
        | ThreadEpiphanyRetrievalFreshnessStatus::Unavailable => {
            CoreEpiphanyReorientFreshnessStatus::Unknown
        }
        ThreadEpiphanyRetrievalFreshnessStatus::Ready => CoreEpiphanyReorientFreshnessStatus::Clean,
        ThreadEpiphanyRetrievalFreshnessStatus::Stale => CoreEpiphanyReorientFreshnessStatus::Stale,
        ThreadEpiphanyRetrievalFreshnessStatus::Indexing => {
            CoreEpiphanyReorientFreshnessStatus::Dirty
        }
    }
}

fn map_core_reorient_graph_status(
    status: ThreadEpiphanyGraphFreshnessStatus,
) -> CoreEpiphanyReorientFreshnessStatus {
    match status {
        ThreadEpiphanyGraphFreshnessStatus::Missing => CoreEpiphanyReorientFreshnessStatus::Unknown,
        ThreadEpiphanyGraphFreshnessStatus::Ready => CoreEpiphanyReorientFreshnessStatus::Clean,
        ThreadEpiphanyGraphFreshnessStatus::Stale => CoreEpiphanyReorientFreshnessStatus::Stale,
    }
}

fn map_core_reorient_watcher_status(
    status: ThreadEpiphanyInvalidationStatus,
) -> CoreEpiphanyReorientFreshnessStatus {
    match status {
        ThreadEpiphanyInvalidationStatus::Unavailable => {
            CoreEpiphanyReorientFreshnessStatus::Unknown
        }
        ThreadEpiphanyInvalidationStatus::Clean => CoreEpiphanyReorientFreshnessStatus::Clean,
        ThreadEpiphanyInvalidationStatus::Changed => CoreEpiphanyReorientFreshnessStatus::Changed,
    }
}

fn map_protocol_reorient_state_status(
    status: CoreEpiphanyReorientStateStatus,
) -> ThreadEpiphanyReorientStateStatus {
    match status {
        CoreEpiphanyReorientStateStatus::Missing => ThreadEpiphanyReorientStateStatus::Missing,
        CoreEpiphanyReorientStateStatus::Ready => ThreadEpiphanyReorientStateStatus::Ready,
    }
}

fn map_protocol_reorient_decision(
    decision: epiphany_core::EpiphanyReorientDecision,
) -> ThreadEpiphanyReorientDecision {
    ThreadEpiphanyReorientDecision {
        action: map_protocol_reorient_action(decision.action),
        checkpoint_status: map_protocol_reorient_checkpoint_status(decision.checkpoint_status),
        checkpoint_id: decision.checkpoint_id,
        pressure_level: map_protocol_reorient_pressure_level(decision.pressure_level),
        retrieval_status: map_protocol_reorient_retrieval_status(decision.retrieval_status),
        graph_status: map_protocol_reorient_graph_status(decision.graph_status),
        watcher_status: map_protocol_reorient_watcher_status(decision.watcher_status),
        reasons: decision
            .reasons
            .into_iter()
            .map(map_protocol_reorient_reason)
            .collect(),
        checkpoint_dirty_paths: decision.checkpoint_dirty_paths,
        checkpoint_changed_paths: decision.checkpoint_changed_paths,
        active_frontier_node_ids: decision.active_frontier_node_ids,
        next_action: decision.next_action,
        note: decision.note,
    }
}

fn map_protocol_reorient_action(
    action: CoreEpiphanyReorientAction,
) -> ThreadEpiphanyReorientAction {
    match action {
        CoreEpiphanyReorientAction::Resume => ThreadEpiphanyReorientAction::Resume,
        CoreEpiphanyReorientAction::Regather => ThreadEpiphanyReorientAction::Regather,
    }
}

fn map_protocol_reorient_checkpoint_status(
    status: CoreEpiphanyReorientCheckpointStatus,
) -> ThreadEpiphanyReorientCheckpointStatus {
    match status {
        CoreEpiphanyReorientCheckpointStatus::Missing => {
            ThreadEpiphanyReorientCheckpointStatus::Missing
        }
        CoreEpiphanyReorientCheckpointStatus::ResumeReady => {
            ThreadEpiphanyReorientCheckpointStatus::ResumeReady
        }
        CoreEpiphanyReorientCheckpointStatus::RegatherRequired => {
            ThreadEpiphanyReorientCheckpointStatus::RegatherRequired
        }
    }
}

fn map_protocol_reorient_pressure_level(
    level: CoreEpiphanyReorientPressureLevel,
) -> ThreadEpiphanyPressureLevel {
    match level {
        CoreEpiphanyReorientPressureLevel::Unknown => ThreadEpiphanyPressureLevel::Unknown,
        CoreEpiphanyReorientPressureLevel::Low => ThreadEpiphanyPressureLevel::Low,
        CoreEpiphanyReorientPressureLevel::Medium => ThreadEpiphanyPressureLevel::Elevated,
        CoreEpiphanyReorientPressureLevel::High => ThreadEpiphanyPressureLevel::High,
        CoreEpiphanyReorientPressureLevel::Critical => ThreadEpiphanyPressureLevel::Critical,
    }
}

fn map_protocol_reorient_retrieval_status(
    status: CoreEpiphanyReorientFreshnessStatus,
) -> ThreadEpiphanyRetrievalFreshnessStatus {
    match status {
        CoreEpiphanyReorientFreshnessStatus::Unknown => {
            ThreadEpiphanyRetrievalFreshnessStatus::Missing
        }
        CoreEpiphanyReorientFreshnessStatus::Clean => ThreadEpiphanyRetrievalFreshnessStatus::Ready,
        CoreEpiphanyReorientFreshnessStatus::Dirty => {
            ThreadEpiphanyRetrievalFreshnessStatus::Indexing
        }
        CoreEpiphanyReorientFreshnessStatus::Stale
        | CoreEpiphanyReorientFreshnessStatus::Changed => {
            ThreadEpiphanyRetrievalFreshnessStatus::Stale
        }
    }
}

fn map_protocol_reorient_graph_status(
    status: CoreEpiphanyReorientFreshnessStatus,
) -> ThreadEpiphanyGraphFreshnessStatus {
    match status {
        CoreEpiphanyReorientFreshnessStatus::Unknown => ThreadEpiphanyGraphFreshnessStatus::Missing,
        CoreEpiphanyReorientFreshnessStatus::Clean => ThreadEpiphanyGraphFreshnessStatus::Ready,
        CoreEpiphanyReorientFreshnessStatus::Dirty
        | CoreEpiphanyReorientFreshnessStatus::Stale
        | CoreEpiphanyReorientFreshnessStatus::Changed => ThreadEpiphanyGraphFreshnessStatus::Stale,
    }
}

fn map_protocol_reorient_watcher_status(
    status: CoreEpiphanyReorientFreshnessStatus,
) -> ThreadEpiphanyInvalidationStatus {
    match status {
        CoreEpiphanyReorientFreshnessStatus::Unknown => {
            ThreadEpiphanyInvalidationStatus::Unavailable
        }
        CoreEpiphanyReorientFreshnessStatus::Clean => ThreadEpiphanyInvalidationStatus::Clean,
        CoreEpiphanyReorientFreshnessStatus::Dirty
        | CoreEpiphanyReorientFreshnessStatus::Stale
        | CoreEpiphanyReorientFreshnessStatus::Changed => ThreadEpiphanyInvalidationStatus::Changed,
    }
}

fn map_protocol_reorient_reason(
    reason: CoreEpiphanyReorientReason,
) -> ThreadEpiphanyReorientReason {
    match reason {
        CoreEpiphanyReorientReason::MissingState => ThreadEpiphanyReorientReason::MissingState,
        CoreEpiphanyReorientReason::MissingCheckpoint => {
            ThreadEpiphanyReorientReason::MissingCheckpoint
        }
        CoreEpiphanyReorientReason::CheckpointReady => {
            ThreadEpiphanyReorientReason::CheckpointReady
        }
        CoreEpiphanyReorientReason::CheckpointRequestedRegather => {
            ThreadEpiphanyReorientReason::CheckpointRequestedRegather
        }
        CoreEpiphanyReorientReason::CheckpointPathsDirty => {
            ThreadEpiphanyReorientReason::CheckpointPathsDirty
        }
        CoreEpiphanyReorientReason::CheckpointPathsChanged => {
            ThreadEpiphanyReorientReason::CheckpointPathsChanged
        }
        CoreEpiphanyReorientReason::FrontierChanged => {
            ThreadEpiphanyReorientReason::FrontierChanged
        }
        CoreEpiphanyReorientReason::UnanchoredCheckpointWhileStateStale => {
            ThreadEpiphanyReorientReason::UnanchoredCheckpointWhileStateStale
        }
    }
}
