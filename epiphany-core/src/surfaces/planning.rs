use codex_protocol::protocol::EpiphanyPlanningState;
use codex_protocol::protocol::EpiphanyThreadState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyPlanningStateStatus {
    Missing,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyPlanningSummary {
    pub capture_count: u32,
    pub pending_capture_count: u32,
    pub github_issue_capture_count: u32,
    pub backlog_item_count: u32,
    pub ready_backlog_item_count: u32,
    pub roadmap_stream_count: u32,
    pub objective_draft_count: u32,
    pub draft_objective_count: u32,
    pub active_objective: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyPlanningView {
    pub state_status: EpiphanyPlanningStateStatus,
    pub state_revision: Option<u64>,
    pub planning: EpiphanyPlanningState,
    pub summary: EpiphanyPlanningSummary,
}

pub fn derive_planning_view(state: Option<&EpiphanyThreadState>) -> EpiphanyPlanningView {
    let Some(state) = state else {
        return EpiphanyPlanningView {
            state_status: EpiphanyPlanningStateStatus::Missing,
            state_revision: None,
            planning: EpiphanyPlanningState::default(),
            summary: EpiphanyPlanningSummary {
                capture_count: 0,
                pending_capture_count: 0,
                github_issue_capture_count: 0,
                backlog_item_count: 0,
                ready_backlog_item_count: 0,
                roadmap_stream_count: 0,
                objective_draft_count: 0,
                draft_objective_count: 0,
                active_objective: None,
                note: "No authoritative Epiphany state exists for this thread.".to_string(),
            },
        };
    };

    let planning = state.planning.clone();
    let pending_capture_count = planning
        .captures
        .iter()
        .filter(|capture| capture.status == "new" || capture.status == "inbox")
        .count() as u32;
    let github_issue_capture_count = planning
        .captures
        .iter()
        .filter(|capture| capture.source.kind == "github_issue")
        .count() as u32;
    let ready_backlog_item_count = planning
        .backlog_items
        .iter()
        .filter(|item| item.status == "ready")
        .count() as u32;
    let draft_objective_count = planning
        .objective_drafts
        .iter()
        .filter(|draft| draft.status == "draft")
        .count() as u32;
    let note = if planning.is_empty() {
        "Planning substrate is present but empty; captures, backlog, roadmap streams, and objective drafts have not been written yet."
    } else {
        "Planning substrate is available. These records are planning state only until a human explicitly adopts an objective."
    }
    .to_string();

    EpiphanyPlanningView {
        state_status: EpiphanyPlanningStateStatus::Ready,
        state_revision: Some(state.revision),
        planning: planning.clone(),
        summary: EpiphanyPlanningSummary {
            capture_count: planning.captures.len() as u32,
            pending_capture_count,
            github_issue_capture_count,
            backlog_item_count: planning.backlog_items.len() as u32,
            ready_backlog_item_count,
            roadmap_stream_count: planning.roadmap_streams.len() as u32,
            objective_draft_count: planning.objective_drafts.len() as u32,
            draft_objective_count,
            active_objective: state.objective.clone(),
            note,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::protocol::EpiphanyPlanningCapture;
    use codex_protocol::protocol::EpiphanyPlanningSourceRef;

    #[test]
    fn missing_state_returns_empty_planning() {
        let view = derive_planning_view(None);

        assert_eq!(view.state_status, EpiphanyPlanningStateStatus::Missing);
        assert_eq!(view.state_revision, None);
        assert_eq!(view.summary.capture_count, 0);
        assert!(view.planning.is_empty());
    }

    #[test]
    fn summarizes_planning_counts_without_adopting_objective() {
        let state = EpiphanyThreadState {
            revision: 3,
            objective: Some("active objective".to_string()),
            planning: EpiphanyPlanningState {
                captures: vec![
                    EpiphanyPlanningCapture {
                        id: "cap-1".to_string(),
                        title: "capture one".to_string(),
                        confidence: "high".to_string(),
                        status: "new".to_string(),
                        source: EpiphanyPlanningSourceRef {
                            kind: "github_issue".to_string(),
                            issue_number: Some(1),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    EpiphanyPlanningCapture {
                        id: "cap-2".to_string(),
                        title: "capture two".to_string(),
                        confidence: "medium".to_string(),
                        status: "accepted".to_string(),
                        source: EpiphanyPlanningSourceRef {
                            kind: "conversation".to_string(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        };

        let view = derive_planning_view(Some(&state));

        assert_eq!(view.state_status, EpiphanyPlanningStateStatus::Ready);
        assert_eq!(view.state_revision, Some(3));
        assert_eq!(view.summary.capture_count, 2);
        assert_eq!(view.summary.pending_capture_count, 1);
        assert_eq!(view.summary.github_issue_capture_count, 1);
        assert_eq!(
            view.summary.active_objective.as_deref(),
            Some("active objective")
        );
    }
}
