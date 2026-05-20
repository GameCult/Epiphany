use epiphany_core::EpiphanyFreshnessInput;
use epiphany_core::EpiphanyFreshnessView;
use epiphany_core::EpiphanyFreshnessWatcherInput;
use epiphany_core::EpiphanyGraphFreshness as CoreEpiphanyGraphFreshness;
use epiphany_core::EpiphanyGraphFreshnessStatus as CoreEpiphanyGraphFreshnessStatus;
use epiphany_core::EpiphanyInvalidationInput as CoreEpiphanyInvalidationInput;
use epiphany_core::EpiphanyInvalidationStatus as CoreEpiphanyInvalidationStatus;
use epiphany_core::EpiphanyPressure;
use epiphany_core::EpiphanyPressureLevel;
use epiphany_core::EpiphanyReorientDecision as CoreEpiphanyReorientDecision;
use epiphany_core::EpiphanyReorientFreshnessStatus as CoreEpiphanyReorientFreshnessStatus;
use epiphany_core::EpiphanyReorientInput;
use epiphany_core::EpiphanyReorientPressureLevel as CoreEpiphanyReorientPressureLevel;
use epiphany_core::EpiphanyReorientStateStatus as CoreEpiphanyReorientStateStatus;
use epiphany_core::EpiphanyRetrievalFreshness as CoreEpiphanyRetrievalFreshness;
use epiphany_core::EpiphanyRetrievalFreshnessStatus as CoreEpiphanyRetrievalFreshnessStatus;
use epiphany_core::derive_freshness;
use epiphany_core::recommend_reorientation;
use epiphany_state_model::EpiphanyRetrievalState;
use epiphany_state_model::EpiphanyThreadState;
use std::path::Path;
use std::path::PathBuf;

pub struct EpiphanyFreshnessWatcherSnapshot<'a> {
    pub available: bool,
    pub workspace_root: Option<&'a Path>,
    pub observed_at_unix_seconds: Option<i64>,
    pub changed_paths: &'a [PathBuf],
}

pub fn derive_epiphany_freshness_view(
    state: Option<&EpiphanyThreadState>,
    retrieval_override: Option<&EpiphanyRetrievalState>,
    watcher_snapshot: Option<EpiphanyFreshnessWatcherSnapshot<'_>>,
) -> EpiphanyFreshnessView {
    let watcher = watcher_snapshot.map(|snapshot| EpiphanyFreshnessWatcherInput {
        available: snapshot.available,
        workspace_root: snapshot.workspace_root,
        observed_at_unix_seconds: snapshot.observed_at_unix_seconds,
        changed_paths: snapshot.changed_paths,
    });
    derive_freshness(EpiphanyFreshnessInput {
        state,
        retrieval_override,
        watcher,
    })
}

pub fn derive_epiphany_reorient(
    state: Option<&EpiphanyThreadState>,
    pressure: &EpiphanyPressure,
    retrieval: &CoreEpiphanyRetrievalFreshness,
    graph: &CoreEpiphanyGraphFreshness,
    watcher: &CoreEpiphanyInvalidationInput,
) -> (
    CoreEpiphanyReorientStateStatus,
    CoreEpiphanyReorientDecision,
) {
    recommend_reorientation(EpiphanyReorientInput {
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
    })
}

fn map_core_reorient_pressure_level(
    level: EpiphanyPressureLevel,
) -> CoreEpiphanyReorientPressureLevel {
    match level {
        EpiphanyPressureLevel::Unknown => CoreEpiphanyReorientPressureLevel::Unknown,
        EpiphanyPressureLevel::Low => CoreEpiphanyReorientPressureLevel::Low,
        EpiphanyPressureLevel::Elevated => CoreEpiphanyReorientPressureLevel::Medium,
        EpiphanyPressureLevel::High => CoreEpiphanyReorientPressureLevel::High,
        EpiphanyPressureLevel::Critical => CoreEpiphanyReorientPressureLevel::Critical,
    }
}

fn map_core_reorient_retrieval_status(
    status: CoreEpiphanyRetrievalFreshnessStatus,
) -> CoreEpiphanyReorientFreshnessStatus {
    match status {
        CoreEpiphanyRetrievalFreshnessStatus::Missing
        | CoreEpiphanyRetrievalFreshnessStatus::Unavailable => {
            CoreEpiphanyReorientFreshnessStatus::Unknown
        }
        CoreEpiphanyRetrievalFreshnessStatus::Ready => CoreEpiphanyReorientFreshnessStatus::Clean,
        CoreEpiphanyRetrievalFreshnessStatus::Stale => CoreEpiphanyReorientFreshnessStatus::Stale,
        CoreEpiphanyRetrievalFreshnessStatus::Indexing => {
            CoreEpiphanyReorientFreshnessStatus::Dirty
        }
    }
}

fn map_core_reorient_graph_status(
    status: CoreEpiphanyGraphFreshnessStatus,
) -> CoreEpiphanyReorientFreshnessStatus {
    match status {
        CoreEpiphanyGraphFreshnessStatus::Missing => CoreEpiphanyReorientFreshnessStatus::Unknown,
        CoreEpiphanyGraphFreshnessStatus::Ready => CoreEpiphanyReorientFreshnessStatus::Clean,
        CoreEpiphanyGraphFreshnessStatus::Stale => CoreEpiphanyReorientFreshnessStatus::Stale,
    }
}

fn map_core_reorient_watcher_status(
    status: CoreEpiphanyInvalidationStatus,
) -> CoreEpiphanyReorientFreshnessStatus {
    match status {
        CoreEpiphanyInvalidationStatus::Unavailable => CoreEpiphanyReorientFreshnessStatus::Unknown,
        CoreEpiphanyInvalidationStatus::Clean => CoreEpiphanyReorientFreshnessStatus::Clean,
        CoreEpiphanyInvalidationStatus::Changed => CoreEpiphanyReorientFreshnessStatus::Changed,
    }
}
