use super::ContextualUserFragment;
use codex_protocol::protocol::EPIPHANY_STATE_CLOSE_TAG;
use codex_protocol::protocol::EPIPHANY_STATE_OPEN_TAG;
use codex_protocol::protocol::EpiphanyThreadState;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EpiphanyStateInstructions {
    text: String,
}

impl EpiphanyStateInstructions {
    pub(crate) fn from_state(state: &EpiphanyThreadState) -> Self {
        Self {
            text: epiphany_core::render_epiphany_state(state),
        }
    }
}

impl ContextualUserFragment for EpiphanyStateInstructions {
    const ROLE: &'static str = "developer";
    const START_MARKER: &'static str = EPIPHANY_STATE_OPEN_TAG;
    const END_MARKER: &'static str = EPIPHANY_STATE_CLOSE_TAG;

    fn body(&self) -> String {
        self.text.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::EpiphanyStateInstructions;
    use crate::context::ContextualUserFragment;
    use codex_protocol::config_types::ModeKind;
    use codex_protocol::protocol::EpiphanyChurnState;
    use codex_protocol::protocol::EpiphanyCodeRef;
    use codex_protocol::protocol::EpiphanyGraph;
    use codex_protocol::protocol::EpiphanyGraphCheckpoint;
    use codex_protocol::protocol::EpiphanyGraphEdge;
    use codex_protocol::protocol::EpiphanyGraphFrontier;
    use codex_protocol::protocol::EpiphanyGraphNode;
    use codex_protocol::protocol::EpiphanyGraphs;
    use codex_protocol::protocol::EpiphanyModeState;
    use codex_protocol::protocol::EpiphanyThreadState;
    use std::path::PathBuf;

    #[test]
    fn render_prefers_frontier_focus_and_omits_empty_sections() {
        let state = EpiphanyThreadState {
            revision: 7,
            objective: Some("Map the actual turn loop.".to_string()),
            graphs: EpiphanyGraphs {
                architecture: EpiphanyGraph {
                    nodes: vec![
                        EpiphanyGraphNode {
                            id: "arch-overview".to_string(),
                            title: "Overview".to_string(),
                            purpose: "Loose context nobody is touching.".to_string(),
                            ..Default::default()
                        },
                        EpiphanyGraphNode {
                            id: "arch-session".to_string(),
                            title: "Session".to_string(),
                            purpose: "Owns prompt assembly.".to_string(),
                            mechanism: Some("Builds the initial context bundle.".to_string()),
                            metaphor: Some("The foreman keeping the queue honest.".to_string()),
                            code_refs: vec![EpiphanyCodeRef {
                                path: PathBuf::from("core/src/session/mod.rs"),
                                start_line: Some(2433),
                                end_line: Some(2617),
                                symbol: Some("build_initial_context".to_string()),
                                note: None,
                            }],
                            ..Default::default()
                        },
                        EpiphanyGraphNode {
                            id: "arch-rollout".to_string(),
                            title: "Rollout".to_string(),
                            purpose: "Persists thread state.".to_string(),
                            ..Default::default()
                        },
                        EpiphanyGraphNode {
                            id: "arch-extra".to_string(),
                            title: "Extra".to_string(),
                            purpose: "Should be trimmed.".to_string(),
                            ..Default::default()
                        },
                    ],
                    edges: vec![EpiphanyGraphEdge {
                        source_id: "arch-session".to_string(),
                        target_id: "arch-rollout".to_string(),
                        kind: "writes".to_string(),
                        id: Some("edge-session-rollout".to_string()),
                        label: Some("persists".to_string()),
                        mechanism: Some("Writes baseline state after a turn.".to_string()),
                        ..Default::default()
                    }],
                },
                dataflow: EpiphanyGraph {
                    nodes: vec![EpiphanyGraphNode {
                        id: "flow-build-context".to_string(),
                        title: "Build context".to_string(),
                        purpose: "Turns state into prompt context.".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
            graph_frontier: Some(EpiphanyGraphFrontier {
                active_node_ids: vec!["arch-session".to_string(), "flow-build-context".to_string()],
                active_edge_ids: vec!["edge-session-rollout".to_string()],
                dirty_paths: vec![PathBuf::from("core/src/session/mod.rs")],
                ..Default::default()
            }),
            graph_checkpoint: Some(EpiphanyGraphCheckpoint {
                checkpoint_id: "cp-7".to_string(),
                graph_revision: 7,
                frontier_node_ids: vec!["arch-session".to_string()],
                ..Default::default()
            }),
            churn: Some(EpiphanyChurnState {
                understanding_status: "stable".to_string(),
                diff_pressure: "low".to_string(),
                graph_freshness: Some("fresh".to_string()),
                warning: None,
                unexplained_writes: Some(0),
            }),
            mode: Some(EpiphanyModeState {
                name: "epiphany".to_string(),
                kind: Some(ModeKind::Default),
            }),
            ..Default::default()
        };

        let rendered = EpiphanyStateInstructions::from_state(&state).render();

        assert!(rendered.contains("<epiphany_state>"));
        assert!(rendered.contains("`arch-session`"));
        assert!(rendered.contains("`flow-build-context`"));
        assert!(!rendered.contains("## Scratch"));
        assert!(!rendered.contains("## Invariants"));
        assert!(rendered.contains("`core/src/session/mod.rs:2433-2617`"));
        assert!(rendered.contains("... 1 more graph nodes not shown"));
    }
}
