use codex_protocol::protocol::EpiphanyInvestigationCheckpoint;
use codex_protocol::protocol::EpiphanyInvestigationDisposition;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyReorientAction {
    Resume,
    Regather,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyReorientStateStatus {
    Missing,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyReorientCheckpointStatus {
    Missing,
    ResumeReady,
    RegatherRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyReorientReason {
    MissingState,
    MissingCheckpoint,
    CheckpointReady,
    CheckpointRequestedRegather,
    CheckpointPathsDirty,
    CheckpointPathsChanged,
    FrontierChanged,
    UnanchoredCheckpointWhileStateStale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyReorientPressureLevel {
    Unknown,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyReorientFreshnessStatus {
    Unknown,
    Clean,
    Dirty,
    Stale,
    Changed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyReorientInput<'a> {
    pub checkpoint: Option<&'a EpiphanyInvestigationCheckpoint>,
    pub state_present: bool,
    pub pressure_level: EpiphanyReorientPressureLevel,
    pub retrieval_status: EpiphanyReorientFreshnessStatus,
    pub retrieval_dirty_paths: Vec<PathBuf>,
    pub graph_status: EpiphanyReorientFreshnessStatus,
    pub graph_dirty_paths: Vec<PathBuf>,
    pub watcher_status: EpiphanyReorientFreshnessStatus,
    pub watcher_changed_paths: Vec<PathBuf>,
    pub watcher_graph_node_ids: Vec<String>,
    pub active_frontier_node_ids: Vec<String>,
    pub watched_root: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyReorientDecision {
    pub action: EpiphanyReorientAction,
    pub checkpoint_status: EpiphanyReorientCheckpointStatus,
    pub checkpoint_id: Option<String>,
    pub pressure_level: EpiphanyReorientPressureLevel,
    pub retrieval_status: EpiphanyReorientFreshnessStatus,
    pub graph_status: EpiphanyReorientFreshnessStatus,
    pub watcher_status: EpiphanyReorientFreshnessStatus,
    pub reasons: Vec<EpiphanyReorientReason>,
    pub checkpoint_dirty_paths: Vec<PathBuf>,
    pub checkpoint_changed_paths: Vec<PathBuf>,
    pub active_frontier_node_ids: Vec<String>,
    pub next_action: String,
    pub note: String,
}

pub fn recommend_reorientation(
    input: EpiphanyReorientInput<'_>,
) -> (EpiphanyReorientStateStatus, EpiphanyReorientDecision) {
    let build_decision = |action: EpiphanyReorientAction,
                          checkpoint_status: EpiphanyReorientCheckpointStatus,
                          checkpoint_id: Option<String>,
                          reasons: Vec<EpiphanyReorientReason>,
                          checkpoint_dirty_paths: Vec<PathBuf>,
                          checkpoint_changed_paths: Vec<PathBuf>,
                          active_frontier_node_ids: Vec<String>,
                          next_action: String,
                          note: String| EpiphanyReorientDecision {
        action,
        checkpoint_status,
        checkpoint_id,
        pressure_level: input.pressure_level,
        retrieval_status: input.retrieval_status,
        graph_status: input.graph_status,
        watcher_status: input.watcher_status,
        reasons,
        checkpoint_dirty_paths,
        checkpoint_changed_paths,
        active_frontier_node_ids,
        next_action,
        note,
    };

    let high_pressure = matches!(
        input.pressure_level,
        EpiphanyReorientPressureLevel::High | EpiphanyReorientPressureLevel::Critical
    );

    if !input.state_present {
        let note = if high_pressure {
            "No Epiphany state survived, and the last recorded context pressure was high; re-gather before editing."
                .to_string()
        } else {
            "No Epiphany state survived, so there is no authoritative checkpoint to resume."
                .to_string()
        };
        return (
            EpiphanyReorientStateStatus::Missing,
            build_decision(
                EpiphanyReorientAction::Regather,
                EpiphanyReorientCheckpointStatus::Missing,
                None,
                vec![
                    EpiphanyReorientReason::MissingState,
                    EpiphanyReorientReason::MissingCheckpoint,
                ],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                "Re-gather source context before editing.".to_string(),
                note,
            ),
        );
    }

    let Some(checkpoint) = input.checkpoint else {
        let note = if high_pressure {
            "Epiphany state survived, but no durable investigation checkpoint was banked before high context pressure; re-gather before editing."
                .to_string()
        } else {
            "Epiphany state survived, but there is no durable investigation checkpoint to resume from."
                .to_string()
        };
        return (
            EpiphanyReorientStateStatus::Ready,
            build_decision(
                EpiphanyReorientAction::Regather,
                EpiphanyReorientCheckpointStatus::Missing,
                None,
                vec![EpiphanyReorientReason::MissingCheckpoint],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                "Re-gather source context before editing.".to_string(),
                note,
            ),
        );
    };

    let checkpoint_status = match checkpoint.disposition {
        EpiphanyInvestigationDisposition::ResumeReady => {
            EpiphanyReorientCheckpointStatus::ResumeReady
        }
        EpiphanyInvestigationDisposition::RegatherRequired => {
            EpiphanyReorientCheckpointStatus::RegatherRequired
        }
    };
    let workspace_root = input.watched_root.as_deref();
    let checkpoint_dirty_paths = overlapping_checkpoint_paths(
        checkpoint,
        input
            .retrieval_dirty_paths
            .iter()
            .chain(input.graph_dirty_paths.iter()),
        workspace_root,
    );
    let checkpoint_changed_paths = overlapping_checkpoint_paths(
        checkpoint,
        input.watcher_changed_paths.iter(),
        workspace_root,
    );
    let frontier_changed = !input.active_frontier_node_ids.is_empty();
    let unanchored_checkpoint_while_state_stale = checkpoint.code_refs.is_empty()
        && (!input.retrieval_dirty_paths.is_empty()
            || !input.graph_dirty_paths.is_empty()
            || !input.watcher_graph_node_ids.is_empty()
            || frontier_changed);

    let mut reasons = Vec::new();
    if checkpoint.disposition == EpiphanyInvestigationDisposition::RegatherRequired {
        reasons.push(EpiphanyReorientReason::CheckpointRequestedRegather);
    }
    if !checkpoint_dirty_paths.is_empty() {
        reasons.push(EpiphanyReorientReason::CheckpointPathsDirty);
    }
    if !checkpoint_changed_paths.is_empty() {
        reasons.push(EpiphanyReorientReason::CheckpointPathsChanged);
    }
    if frontier_changed {
        reasons.push(EpiphanyReorientReason::FrontierChanged);
    }
    if unanchored_checkpoint_while_state_stale {
        reasons.push(EpiphanyReorientReason::UnanchoredCheckpointWhileStateStale);
    }

    let should_regather = checkpoint.disposition
        == EpiphanyInvestigationDisposition::RegatherRequired
        || !checkpoint_dirty_paths.is_empty()
        || !checkpoint_changed_paths.is_empty()
        || frontier_changed
        || unanchored_checkpoint_while_state_stale;

    if !should_regather {
        reasons.push(EpiphanyReorientReason::CheckpointReady);
        return (
            EpiphanyReorientStateStatus::Ready,
            build_decision(
                EpiphanyReorientAction::Resume,
                checkpoint_status,
                Some(checkpoint.checkpoint_id.clone()),
                reasons,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                checkpoint.next_action.clone().unwrap_or_else(|| {
                    "Resume from the durable checkpoint focus and verify the seam before broad edits."
                        .to_string()
                }),
                "Resume-ready checkpoint remains aligned with current freshness and watcher signals."
                    .to_string(),
            ),
        );
    }

    let mut note_fragments = Vec::new();
    if checkpoint.disposition == EpiphanyInvestigationDisposition::RegatherRequired {
        note_fragments.push("the checkpoint explicitly requests re-gather".to_string());
    }
    if !checkpoint_dirty_paths.is_empty() {
        note_fragments.push(format!(
            "{} checkpoint path(s) are already marked dirty",
            checkpoint_dirty_paths.len()
        ));
    }
    if !checkpoint_changed_paths.is_empty() {
        note_fragments.push(format!(
            "watcher observed {} changed checkpoint path(s)",
            checkpoint_changed_paths.len()
        ));
    }
    if frontier_changed {
        note_fragments.push(format!(
            "watcher hit {} active frontier node(s)",
            input.active_frontier_node_ids.len()
        ));
    }
    if unanchored_checkpoint_while_state_stale {
        note_fragments.push(
            "the checkpoint has no code refs while freshness signals already show drift"
                .to_string(),
        );
    }

    (
        EpiphanyReorientStateStatus::Ready,
        build_decision(
            EpiphanyReorientAction::Regather,
            checkpoint_status,
            Some(checkpoint.checkpoint_id.clone()),
            reasons,
            checkpoint_dirty_paths,
            checkpoint_changed_paths,
            input.active_frontier_node_ids,
            checkpoint
                .next_action
                .clone()
                .unwrap_or_else(|| "Re-gather source context before editing.".to_string()),
            format!("Re-gather before editing: {}.", note_fragments.join("; ")),
        ),
    )
}

fn overlapping_checkpoint_paths<'a>(
    checkpoint: &EpiphanyInvestigationCheckpoint,
    candidate_paths: impl IntoIterator<Item = &'a PathBuf>,
    workspace_root: Option<&Path>,
) -> Vec<PathBuf> {
    let checkpoint_path_keys: HashSet<String> = checkpoint
        .code_refs
        .iter()
        .map(|code_ref| code_ref_path_key(code_ref.path.as_path(), workspace_root))
        .collect();
    if checkpoint_path_keys.is_empty() {
        return Vec::new();
    }

    let mut overlaps = Vec::new();
    let mut seen = HashSet::new();
    for path in candidate_paths {
        let path_key = code_ref_path_key(path.as_path(), workspace_root);
        if checkpoint_path_keys.contains(&path_key) && seen.insert(path_key) {
            overlaps.push(path.clone());
        }
    }
    overlaps
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
    use codex_protocol::protocol::EpiphanyCodeRef;

    fn checkpoint() -> EpiphanyInvestigationCheckpoint {
        EpiphanyInvestigationCheckpoint {
            checkpoint_id: "ix-1".to_string(),
            kind: "implementation".to_string(),
            disposition: EpiphanyInvestigationDisposition::ResumeReady,
            focus: "runtime seam".to_string(),
            summary: Some("resume here".to_string()),
            next_action: Some("Continue from the seam.".to_string()),
            captured_at_turn_id: Some("turn-1".to_string()),
            open_questions: Vec::new(),
            code_refs: vec![EpiphanyCodeRef {
                path: PathBuf::from("src/lib.rs"),
                start_line: None,
                end_line: None,
                symbol: None,
                note: None,
            }],
            evidence_ids: vec!["ev-1".to_string()],
        }
    }

    fn input(checkpoint: Option<&EpiphanyInvestigationCheckpoint>) -> EpiphanyReorientInput<'_> {
        EpiphanyReorientInput {
            checkpoint,
            state_present: true,
            pressure_level: EpiphanyReorientPressureLevel::Low,
            retrieval_status: EpiphanyReorientFreshnessStatus::Clean,
            retrieval_dirty_paths: Vec::new(),
            graph_status: EpiphanyReorientFreshnessStatus::Clean,
            graph_dirty_paths: Vec::new(),
            watcher_status: EpiphanyReorientFreshnessStatus::Clean,
            watcher_changed_paths: Vec::new(),
            watcher_graph_node_ids: Vec::new(),
            active_frontier_node_ids: Vec::new(),
            watched_root: None,
        }
    }

    #[test]
    fn resumes_when_checkpoint_is_clean() {
        let checkpoint = checkpoint();
        let (state_status, decision) = recommend_reorientation(input(Some(&checkpoint)));

        assert_eq!(state_status, EpiphanyReorientStateStatus::Ready);
        assert_eq!(decision.action, EpiphanyReorientAction::Resume);
        assert_eq!(decision.checkpoint_id.as_deref(), Some("ix-1"));
        assert_eq!(
            decision.reasons,
            vec![EpiphanyReorientReason::CheckpointReady]
        );
    }

    #[test]
    fn regathers_when_checkpoint_path_changes() {
        let checkpoint = checkpoint();
        let mut input = input(Some(&checkpoint));
        input.watcher_status = EpiphanyReorientFreshnessStatus::Changed;
        input.watcher_changed_paths = vec![PathBuf::from("./src/lib.rs")];

        let (_state_status, decision) = recommend_reorientation(input);

        assert_eq!(decision.action, EpiphanyReorientAction::Regather);
        assert_eq!(
            decision.reasons,
            vec![EpiphanyReorientReason::CheckpointPathsChanged]
        );
        assert_eq!(
            decision.checkpoint_changed_paths,
            vec![PathBuf::from("./src/lib.rs")]
        );
    }

    #[test]
    fn missing_state_regathers() {
        let mut input = input(None);
        input.state_present = false;
        input.pressure_level = EpiphanyReorientPressureLevel::High;

        let (state_status, decision) = recommend_reorientation(input);

        assert_eq!(state_status, EpiphanyReorientStateStatus::Missing);
        assert_eq!(decision.action, EpiphanyReorientAction::Regather);
        assert!(
            decision
                .reasons
                .contains(&EpiphanyReorientReason::MissingState)
        );
        assert!(decision.note.contains("high"));
    }
}
