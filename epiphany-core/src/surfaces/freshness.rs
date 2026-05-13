use epiphany_state_model::EpiphanyRetrievalState;
use epiphany_state_model::EpiphanyRetrievalStatus;
use epiphany_state_model::EpiphanyThreadState;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyRetrievalFreshnessStatus {
    Missing,
    Ready,
    Stale,
    Indexing,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyRetrievalFreshness {
    pub status: EpiphanyRetrievalFreshnessStatus,
    pub semantic_available: Option<bool>,
    pub last_indexed_at_unix_seconds: Option<i64>,
    pub indexed_file_count: Option<u32>,
    pub indexed_chunk_count: Option<u32>,
    pub dirty_paths: Vec<PathBuf>,
    pub note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyGraphFreshnessStatus {
    Missing,
    Ready,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyGraphFreshness {
    pub status: EpiphanyGraphFreshnessStatus,
    pub graph_freshness: Option<String>,
    pub checkpoint_id: Option<String>,
    pub dirty_path_count: u32,
    pub dirty_paths: Vec<PathBuf>,
    pub open_question_count: u32,
    pub open_gap_count: u32,
    pub note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyInvalidationStatus {
    Unavailable,
    Clean,
    Changed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyInvalidationInput {
    pub status: EpiphanyInvalidationStatus,
    pub watched_root: Option<PathBuf>,
    pub observed_at_unix_seconds: Option<i64>,
    pub changed_path_count: u32,
    pub changed_paths: Vec<PathBuf>,
    pub graph_node_ids: Vec<String>,
    pub active_frontier_node_ids: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Copy)]
pub struct EpiphanyFreshnessWatcherInput<'a> {
    pub available: bool,
    pub workspace_root: Option<&'a Path>,
    pub observed_at_unix_seconds: Option<i64>,
    pub changed_paths: &'a [PathBuf],
}

#[derive(Debug, Clone, Copy)]
pub struct EpiphanyFreshnessInput<'a> {
    pub state: Option<&'a EpiphanyThreadState>,
    pub retrieval_override: Option<&'a EpiphanyRetrievalState>,
    pub watcher: Option<EpiphanyFreshnessWatcherInput<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyFreshnessView {
    pub state_revision: Option<u64>,
    pub retrieval: EpiphanyRetrievalFreshness,
    pub graph: EpiphanyGraphFreshness,
    pub watcher: EpiphanyInvalidationInput,
}

pub fn derive_freshness(input: EpiphanyFreshnessInput<'_>) -> EpiphanyFreshnessView {
    let retrieval = retrieval_state_for_reflection(input.state, input.retrieval_override);
    EpiphanyFreshnessView {
        state_revision: input.state.map(|state| state.revision),
        retrieval: retrieval_freshness(retrieval),
        graph: graph_freshness(input.state),
        watcher: invalidation_input(input.state, input.watcher),
    }
}

fn retrieval_state_for_reflection<'a>(
    state: Option<&'a EpiphanyThreadState>,
    retrieval_override: Option<&'a EpiphanyRetrievalState>,
) -> Option<&'a EpiphanyRetrievalState> {
    retrieval_override.or_else(|| state.and_then(|state| state.retrieval.as_ref()))
}

fn retrieval_freshness(retrieval: Option<&EpiphanyRetrievalState>) -> EpiphanyRetrievalFreshness {
    let Some(retrieval) = retrieval else {
        return EpiphanyRetrievalFreshness {
            status: EpiphanyRetrievalFreshnessStatus::Missing,
            semantic_available: None,
            last_indexed_at_unix_seconds: None,
            indexed_file_count: None,
            indexed_chunk_count: None,
            dirty_paths: Vec::new(),
            note: "No retrieval freshness is available for this thread view.".to_string(),
        };
    };

    let dirty_path_count = retrieval.dirty_paths.len();
    let note = match retrieval.status {
        EpiphanyRetrievalStatus::Ready if dirty_path_count == 0 => {
            "Retrieval catalog is ready.".to_string()
        }
        EpiphanyRetrievalStatus::Ready => {
            format!("Retrieval catalog is ready with {dirty_path_count} dirty path(s) noted.")
        }
        EpiphanyRetrievalStatus::Stale => {
            format!("Retrieval catalog is stale; {dirty_path_count} dirty path(s) need refresh.")
        }
        EpiphanyRetrievalStatus::Indexing => "Retrieval catalog is indexing.".to_string(),
        EpiphanyRetrievalStatus::Unavailable => "Retrieval catalog is unavailable.".to_string(),
    };

    EpiphanyRetrievalFreshness {
        status: match retrieval.status {
            EpiphanyRetrievalStatus::Ready => EpiphanyRetrievalFreshnessStatus::Ready,
            EpiphanyRetrievalStatus::Stale => EpiphanyRetrievalFreshnessStatus::Stale,
            EpiphanyRetrievalStatus::Indexing => EpiphanyRetrievalFreshnessStatus::Indexing,
            EpiphanyRetrievalStatus::Unavailable => EpiphanyRetrievalFreshnessStatus::Unavailable,
        },
        semantic_available: Some(retrieval.semantic_available),
        last_indexed_at_unix_seconds: retrieval.last_indexed_at_unix_seconds,
        indexed_file_count: retrieval.indexed_file_count,
        indexed_chunk_count: retrieval.indexed_chunk_count,
        dirty_paths: retrieval.dirty_paths.clone(),
        note,
    }
}

fn graph_freshness(state: Option<&EpiphanyThreadState>) -> EpiphanyGraphFreshness {
    let Some(state) = state else {
        return EpiphanyGraphFreshness {
            status: EpiphanyGraphFreshnessStatus::Missing,
            graph_freshness: None,
            checkpoint_id: None,
            dirty_path_count: 0,
            dirty_paths: Vec::new(),
            open_question_count: 0,
            open_gap_count: 0,
            note: "Epiphany state is missing, so graph freshness cannot be assessed.".to_string(),
        };
    };

    let frontier = state.graph_frontier.as_ref();
    let dirty_paths = frontier
        .map(|frontier| frontier.dirty_paths.clone())
        .unwrap_or_default();
    let dirty_path_count = dirty_paths.len() as u32;
    let open_question_count = frontier
        .map(|frontier| frontier.open_question_ids.len() as u32)
        .unwrap_or_default();
    let open_gap_count = frontier
        .map(|frontier| frontier.open_gap_ids.len() as u32)
        .unwrap_or_default();
    let graph_freshness = state
        .churn
        .as_ref()
        .and_then(|churn| churn.graph_freshness.clone());
    let freshness_hint_stale = graph_freshness
        .as_deref()
        .is_some_and(|freshness| !matches!(freshness, "fresh" | "ready" | "current" | "ok"));
    let is_stale = dirty_path_count > 0
        || open_question_count > 0
        || open_gap_count > 0
        || freshness_hint_stale;
    let note = if is_stale {
        format!(
            "Graph freshness is stale; frontier has {dirty_path_count} dirty path(s), {open_question_count} open question id(s), and {open_gap_count} open gap id(s)."
        )
    } else {
        "Graph freshness is ready.".to_string()
    };

    EpiphanyGraphFreshness {
        status: if is_stale {
            EpiphanyGraphFreshnessStatus::Stale
        } else {
            EpiphanyGraphFreshnessStatus::Ready
        },
        graph_freshness,
        checkpoint_id: state
            .graph_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_id.clone()),
        dirty_path_count,
        dirty_paths,
        open_question_count,
        open_gap_count,
        note,
    }
}

fn invalidation_input(
    state: Option<&EpiphanyThreadState>,
    watcher: Option<EpiphanyFreshnessWatcherInput<'_>>,
) -> EpiphanyInvalidationInput {
    let Some(watcher) = watcher else {
        return EpiphanyInvalidationInput {
            status: EpiphanyInvalidationStatus::Unavailable,
            watched_root: None,
            observed_at_unix_seconds: None,
            changed_path_count: 0,
            changed_paths: Vec::new(),
            graph_node_ids: Vec::new(),
            active_frontier_node_ids: Vec::new(),
            note: "Watcher-backed invalidation inputs are only available for loaded threads."
                .to_string(),
        };
    };

    if !watcher.available {
        return EpiphanyInvalidationInput {
            status: EpiphanyInvalidationStatus::Unavailable,
            watched_root: None,
            observed_at_unix_seconds: None,
            changed_path_count: 0,
            changed_paths: Vec::new(),
            graph_node_ids: Vec::new(),
            active_frontier_node_ids: Vec::new(),
            note: "The workspace watcher is unavailable for this app-server process.".to_string(),
        };
    }

    let changed_paths = watcher.changed_paths.to_vec();
    let changed_path_count = changed_paths.len() as u32;
    let watched_root = watcher.workspace_root.map(Path::to_path_buf);

    if changed_paths.is_empty() {
        return EpiphanyInvalidationInput {
            status: EpiphanyInvalidationStatus::Clean,
            watched_root,
            observed_at_unix_seconds: watcher.observed_at_unix_seconds,
            changed_path_count,
            changed_paths,
            graph_node_ids: Vec::new(),
            active_frontier_node_ids: Vec::new(),
            note: "Watcher has not observed recent filesystem changes under the workspace root."
                .to_string(),
        };
    }

    let graph_node_ids = state
        .map(|state| {
            graph_node_ids_for_changed_paths(state, &changed_paths, watcher.workspace_root)
        })
        .unwrap_or_default();
    let active_frontier_node_ids = state
        .map(|state| {
            active_frontier_node_ids_for_changed_paths(
                state,
                &changed_paths,
                watcher.workspace_root,
            )
        })
        .unwrap_or_default();
    let note = if graph_node_ids.is_empty() {
        format!(
            "Watcher observed {changed_path_count} recent changed path(s), but no mapped graph node code refs matched yet."
        )
    } else if active_frontier_node_ids.is_empty() {
        format!(
            "Watcher observed {changed_path_count} recent changed path(s) touching {} mapped graph node(s).",
            graph_node_ids.len()
        )
    } else {
        format!(
            "Watcher observed {changed_path_count} recent changed path(s) touching {} mapped graph node(s), including {} active frontier node(s).",
            graph_node_ids.len(),
            active_frontier_node_ids.len()
        )
    };

    EpiphanyInvalidationInput {
        status: EpiphanyInvalidationStatus::Changed,
        watched_root,
        observed_at_unix_seconds: watcher.observed_at_unix_seconds,
        changed_path_count,
        changed_paths,
        graph_node_ids,
        active_frontier_node_ids,
        note,
    }
}

fn graph_node_ids_for_changed_paths(
    state: &EpiphanyThreadState,
    changed_paths: &[PathBuf],
    workspace_root: Option<&Path>,
) -> Vec<String> {
    let changed_path_keys: HashSet<String> = changed_paths
        .iter()
        .map(|path| epiphany_path_key(path.as_path()))
        .collect();
    let mut node_ids = Vec::new();
    let mut seen = HashSet::new();

    for node in state
        .graphs
        .architecture
        .nodes
        .iter()
        .chain(state.graphs.dataflow.nodes.iter())
    {
        let matches_changed_path = node.code_refs.iter().any(|code_ref| {
            changed_path_keys.contains(&code_ref_path_key(
                Path::new(&code_ref.path),
                workspace_root,
            ))
        });
        if matches_changed_path && seen.insert(node.id.as_str()) {
            node_ids.push(node.id.clone());
        }
    }

    node_ids
}

fn active_frontier_node_ids_for_changed_paths(
    state: &EpiphanyThreadState,
    changed_paths: &[PathBuf],
    workspace_root: Option<&Path>,
) -> Vec<String> {
    let graph_node_ids = graph_node_ids_for_changed_paths(state, changed_paths, workspace_root);
    let frontier_node_ids = state
        .graph_frontier
        .as_ref()
        .map(|frontier| frontier.active_node_ids.as_slice())
        .unwrap_or_default();
    let graph_node_ids: HashSet<&str> = graph_node_ids.iter().map(String::as_str).collect();

    frontier_node_ids
        .iter()
        .filter(|node_id| graph_node_ids.contains(node_id.as_str()))
        .cloned()
        .collect()
}

fn code_ref_path_key(path: &Path, workspace_root: Option<&Path>) -> String {
    if let Some(workspace_root) = workspace_root
        && let Ok(relative_path) = path.strip_prefix(workspace_root)
    {
        return epiphany_path_key(relative_path);
    }
    epiphany_path_key(path)
}

fn epiphany_path_key(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_state_model::EpiphanyCodeRef;
    use epiphany_state_model::EpiphanyGraphFrontier;
    use epiphany_state_model::EpiphanyGraphNode;
    use epiphany_state_model::EpiphanyGraphs;

    #[test]
    fn missing_state_reports_missing_freshness() {
        let view = derive_freshness(EpiphanyFreshnessInput {
            state: None,
            retrieval_override: None,
            watcher: None,
        });

        assert_eq!(view.state_revision, None);
        assert_eq!(
            view.retrieval.status,
            EpiphanyRetrievalFreshnessStatus::Missing
        );
        assert_eq!(view.graph.status, EpiphanyGraphFreshnessStatus::Missing);
        assert_eq!(view.watcher.status, EpiphanyInvalidationStatus::Unavailable);
    }

    #[test]
    fn stale_retrieval_and_frontier_make_graph_stale() {
        let state = EpiphanyThreadState {
            revision: 7,
            retrieval: Some(EpiphanyRetrievalState {
                workspace_root: PathBuf::from("E:/repo"),
                status: EpiphanyRetrievalStatus::Stale,
                semantic_available: true,
                indexed_file_count: Some(12),
                dirty_paths: vec![PathBuf::from("src/lib.rs")],
                ..Default::default()
            }),
            graph_frontier: Some(EpiphanyGraphFrontier {
                dirty_paths: vec![PathBuf::from("src/lib.rs")],
                open_gap_ids: vec!["gap-1".to_string()],
                ..Default::default()
            }),
            ..Default::default()
        };

        let view = derive_freshness(EpiphanyFreshnessInput {
            state: Some(&state),
            retrieval_override: None,
            watcher: None,
        });

        assert_eq!(view.state_revision, Some(7));
        assert_eq!(
            view.retrieval.status,
            EpiphanyRetrievalFreshnessStatus::Stale
        );
        assert_eq!(view.retrieval.indexed_file_count, Some(12));
        assert_eq!(view.graph.status, EpiphanyGraphFreshnessStatus::Stale);
        assert_eq!(view.graph.open_gap_count, 1);
    }

    #[test]
    fn watcher_maps_changed_paths_to_graph_and_frontier() {
        let state = EpiphanyThreadState {
            graph_frontier: Some(EpiphanyGraphFrontier {
                active_node_ids: vec!["node-1".to_string()],
                ..Default::default()
            }),
            graphs: EpiphanyGraphs {
                architecture: epiphany_state_model::EpiphanyGraph {
                    nodes: vec![EpiphanyGraphNode {
                        id: "node-1".to_string(),
                        title: "node".to_string(),
                        purpose: "module".to_string(),
                        mechanism: None,
                        metaphor: None,
                        status: None,
                        code_refs: vec![EpiphanyCodeRef {
                            path: PathBuf::from("src/lib.rs"),
                            start_line: None,
                            end_line: None,
                            symbol: None,
                            note: None,
                        }],
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let changed_paths = vec![PathBuf::from("./src/lib.rs")];

        let view = derive_freshness(EpiphanyFreshnessInput {
            state: Some(&state),
            retrieval_override: None,
            watcher: Some(EpiphanyFreshnessWatcherInput {
                available: true,
                workspace_root: None,
                observed_at_unix_seconds: Some(42),
                changed_paths: &changed_paths,
            }),
        });

        assert_eq!(view.watcher.status, EpiphanyInvalidationStatus::Changed);
        assert_eq!(view.watcher.graph_node_ids, vec!["node-1".to_string()]);
        assert_eq!(
            view.watcher.active_frontier_node_ids,
            vec!["node-1".to_string()]
        );
    }
}
