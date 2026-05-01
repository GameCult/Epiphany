use codex_protocol::protocol::EpiphanyChurnState;
use codex_protocol::protocol::EpiphanyCodeRef;
use codex_protocol::protocol::EpiphanyEvidenceRecord;
use codex_protocol::protocol::EpiphanyGraph;
use codex_protocol::protocol::EpiphanyGraphCheckpoint;
use codex_protocol::protocol::EpiphanyGraphEdge;
use codex_protocol::protocol::EpiphanyGraphFrontier;
use codex_protocol::protocol::EpiphanyGraphLink;
use codex_protocol::protocol::EpiphanyGraphNode;
use codex_protocol::protocol::EpiphanyInvestigationCheckpoint;
use codex_protocol::protocol::EpiphanyInvestigationDisposition;
use codex_protocol::protocol::EpiphanyModeState;
use codex_protocol::protocol::EpiphanyObservation;
use codex_protocol::protocol::EpiphanyScratchPad;
use codex_protocol::protocol::EpiphanyThreadState;
use std::collections::HashSet;
use std::path::Path;

const SUBGOAL_LIMIT: usize = 4;
const INVARIANT_LIMIT: usize = 4;
const NODE_LIMIT: usize = 3;
const EDGE_LIMIT: usize = 2;
const LINK_LIMIT: usize = 3;
const OBSERVATION_LIMIT: usize = 3;
const EVIDENCE_LIMIT: usize = 3;
const CODE_REF_LIMIT: usize = 2;
const DIRTY_PATH_LIMIT: usize = 4;
const EPIPHANY_STATE_INTRO: &str = include_str!("prompts/epiphany_state_intro.md");
const EPIPHANY_DOCTRINE_SECTION: &str = include_str!("prompts/epiphany_doctrine.md");

pub fn render_epiphany_state(state: &EpiphanyThreadState) -> String {
    let mut sections = vec![EPIPHANY_STATE_INTRO.trim_end().to_string()];

    sections.push(render_overview(state));
    sections.push(EPIPHANY_DOCTRINE_SECTION.trim_end().to_string());

    if let Some(subgoals) = render_subgoals(state) {
        sections.push(subgoals);
    }
    if let Some(invariants) = render_invariants(state) {
        sections.push(invariants);
    }
    if let Some(graphs) = render_graphs(state) {
        sections.push(graphs);
    }
    if let Some(checkpoint) =
        render_investigation_checkpoint(state.investigation_checkpoint.as_ref())
    {
        sections.push(checkpoint);
    }
    if let Some(scratch) = render_scratch(state.scratch.as_ref()) {
        sections.push(scratch);
    }
    if let Some(observations) = render_observations(&state.observations) {
        sections.push(observations);
    }
    if let Some(evidence) = render_evidence(&state.recent_evidence) {
        sections.push(evidence);
    }
    if let Some(churn) = render_churn(state.churn.as_ref()) {
        sections.push(churn);
    }
    if let Some(mode) = render_mode(state.mode.as_ref()) {
        sections.push(mode);
    }

    format!("{}\n", sections.join("\n\n"))
}

fn render_overview(state: &EpiphanyThreadState) -> String {
    let mut lines = vec![format!("- Revision: {}", state.revision)];
    if let Some(objective) = state.objective.as_deref() {
        lines.push(format!("- Objective: {}", compact_text(objective)));
    }
    if let Some(active_subgoal_id) = state.active_subgoal_id.as_deref() {
        lines.push(format!("- Active subgoal: `{active_subgoal_id}`"));
    }
    if let Some(last_updated_turn_id) = state.last_updated_turn_id.as_deref() {
        lines.push(format!("- Last updated turn: `{last_updated_turn_id}`"));
    }
    if let Some(frontier) = state.graph_frontier.as_ref() {
        let focus_ids = collect_focus_node_ids(Some(frontier), state.graph_checkpoint.as_ref());
        if !focus_ids.is_empty() {
            lines.push(format!(
                "- Frontier focus: {}",
                render_inline_ids(&focus_ids)
            ));
        }
    }
    format!("## Overview\n{}", lines.join("\n"))
}

fn render_subgoals(state: &EpiphanyThreadState) -> Option<String> {
    if state.subgoals.is_empty() {
        return None;
    }

    let mut lines = Vec::new();
    for subgoal in state.subgoals.iter().take(SUBGOAL_LIMIT) {
        let active_suffix = state
            .active_subgoal_id
            .as_deref()
            .filter(|active_id| *active_id == subgoal.id)
            .map(|_| "; active")
            .unwrap_or("");
        lines.push(format!(
            "- `{}` [{}{}]: {}",
            subgoal.id,
            subgoal.status,
            active_suffix,
            compact_text(&subgoal.title)
        ));
        if let Some(summary) = subgoal.summary.as_deref() {
            lines.push(format!("  summary: {}", compact_text(summary)));
        }
    }
    push_omitted_count(&mut lines, state.subgoals.len(), SUBGOAL_LIMIT, "subgoals");

    Some(format!("## Subgoals\n{}", lines.join("\n")))
}

fn render_invariants(state: &EpiphanyThreadState) -> Option<String> {
    if state.invariants.is_empty() {
        return None;
    }

    let mut lines = Vec::new();
    for invariant in state.invariants.iter().take(INVARIANT_LIMIT) {
        lines.push(format!(
            "- `{}` [{}]: {}",
            invariant.id,
            invariant.status,
            compact_text(&invariant.description)
        ));
        if let Some(rationale) = invariant.rationale.as_deref() {
            lines.push(format!("  rationale: {}", compact_text(rationale)));
        }
    }
    push_omitted_count(
        &mut lines,
        state.invariants.len(),
        INVARIANT_LIMIT,
        "invariants",
    );

    Some(format!("## Invariants\n{}", lines.join("\n")))
}

fn render_graphs(state: &EpiphanyThreadState) -> Option<String> {
    if state.graphs.is_empty() && state.graph_frontier.is_none() && state.graph_checkpoint.is_none()
    {
        return None;
    }

    let mut lines = Vec::new();
    let focus_node_ids = collect_focus_node_ids(
        state.graph_frontier.as_ref(),
        state.graph_checkpoint.as_ref(),
    );

    if let Some(frontier) = state.graph_frontier.as_ref() {
        append_frontier_lines(&mut lines, frontier);
    }
    if let Some(checkpoint) = state.graph_checkpoint.as_ref() {
        append_checkpoint_lines(&mut lines, checkpoint);
    }

    append_graph_lines(
        &mut lines,
        "Architecture graph",
        &state.graphs.architecture,
        &focus_node_ids,
        state.graph_frontier.as_ref(),
    );
    append_graph_lines(
        &mut lines,
        "Dataflow graph",
        &state.graphs.dataflow,
        &focus_node_ids,
        state.graph_frontier.as_ref(),
    );

    if !state.graphs.links.is_empty() {
        for link in select_links(&state.graphs.links, &focus_node_ids)
            .into_iter()
            .take(LINK_LIMIT)
        {
            let relationship = link
                .relationship
                .as_deref()
                .map(|value| format!(" ({})", compact_text(value)))
                .unwrap_or_default();
            let code_refs = render_code_refs(&link.code_refs);
            lines.push(format!(
                "- Link: `{}` -> `{}`{}{}",
                link.dataflow_node_id,
                link.architecture_node_id,
                relationship,
                render_ref_suffix(&code_refs)
            ));
        }
        push_omitted_count(
            &mut lines,
            state.graphs.links.len(),
            LINK_LIMIT,
            "graph links",
        );
    }

    Some(format!("## Graphs\n{}", lines.join("\n")))
}

fn append_frontier_lines(lines: &mut Vec<String>, frontier: &EpiphanyGraphFrontier) {
    if !frontier.active_node_ids.is_empty() {
        lines.push(format!(
            "- Frontier active nodes: {}",
            render_inline_ids(&frontier.active_node_ids)
        ));
    }
    if !frontier.active_edge_ids.is_empty() {
        lines.push(format!(
            "- Frontier active edges: {}",
            render_inline_ids(&frontier.active_edge_ids)
        ));
    }
    if !frontier.open_question_ids.is_empty() {
        lines.push(format!(
            "- Open questions: {}",
            render_inline_ids(&frontier.open_question_ids)
        ));
    }
    if !frontier.open_gap_ids.is_empty() {
        lines.push(format!(
            "- Open gaps: {}",
            render_inline_ids(&frontier.open_gap_ids)
        ));
    }
    if !frontier.dirty_paths.is_empty() {
        let dirty_paths = frontier
            .dirty_paths
            .iter()
            .take(DIRTY_PATH_LIMIT)
            .map(|path| format!("`{}`", normalize_path(path)))
            .collect::<Vec<_>>()
            .join(", ");
        let omitted_suffix =
            omitted_suffix(frontier.dirty_paths.len(), DIRTY_PATH_LIMIT, "dirty paths");
        lines.push(format!("- Dirty paths: {dirty_paths}{omitted_suffix}"));
    }
}

fn append_checkpoint_lines(lines: &mut Vec<String>, checkpoint: &EpiphanyGraphCheckpoint) {
    lines.push(format!(
        "- Checkpoint: `{}` (graph revision {})",
        checkpoint.checkpoint_id, checkpoint.graph_revision
    ));
    if let Some(summary) = checkpoint.summary.as_deref() {
        lines.push(format!("- Checkpoint summary: {}", compact_text(summary)));
    }
    if !checkpoint.frontier_node_ids.is_empty() {
        lines.push(format!(
            "- Checkpoint frontier nodes: {}",
            render_inline_ids(&checkpoint.frontier_node_ids)
        ));
    }
    if !checkpoint.open_question_ids.is_empty() {
        lines.push(format!(
            "- Checkpoint open questions: {}",
            render_inline_ids(&checkpoint.open_question_ids)
        ));
    }
    if !checkpoint.open_gap_ids.is_empty() {
        lines.push(format!(
            "- Checkpoint open gaps: {}",
            render_inline_ids(&checkpoint.open_gap_ids)
        ));
    }
}

fn append_graph_lines(
    lines: &mut Vec<String>,
    label: &str,
    graph: &EpiphanyGraph,
    focus_node_ids: &[String],
    frontier: Option<&EpiphanyGraphFrontier>,
) {
    if graph.is_empty() {
        return;
    }

    let selected_nodes = select_nodes(graph, focus_node_ids);
    let selected_node_ids = selected_nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<HashSet<_>>();
    let active_edge_ids = frontier
        .map(|graph_frontier| {
            graph_frontier
                .active_edge_ids
                .iter()
                .map(String::as_str)
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();

    lines.push(format!(
        "- {label}: {} nodes, {} edges",
        graph.nodes.len(),
        graph.edges.len()
    ));

    for node in &selected_nodes {
        let status = node
            .status
            .as_deref()
            .map(|value| format!(" [{}]", compact_text(value)))
            .unwrap_or_default();
        lines.push(format!(
            "  - Focus node `{}`{}: {}",
            node.id,
            status,
            compact_text(&node.title)
        ));
        lines.push(format!("    purpose: {}", compact_text(&node.purpose)));
        if let Some(mechanism) = node.mechanism.as_deref() {
            lines.push(format!("    mechanism: {}", compact_text(mechanism)));
        }
        if let Some(metaphor) = node.metaphor.as_deref() {
            lines.push(format!("    metaphor: {}", compact_text(metaphor)));
        }
        let code_refs = render_code_refs(&node.code_refs);
        if !code_refs.is_empty() {
            lines.push(format!("    code refs: {}", code_refs.join(", ")));
        }
    }
    push_omitted_count(lines, graph.nodes.len(), NODE_LIMIT, "graph nodes");

    let selected_edges = select_edges(graph, &selected_node_ids, &active_edge_ids);
    for edge in selected_edges.iter().take(EDGE_LIMIT) {
        let edge_id = edge
            .id
            .as_deref()
            .map(|value| format!(" `{value}`"))
            .unwrap_or_default();
        let label_suffix = edge
            .label
            .as_deref()
            .map(|value| format!(" [{}]", compact_text(value)))
            .unwrap_or_default();
        let mechanism_suffix = edge
            .mechanism
            .as_deref()
            .map(|value| format!("; {}", compact_text(value)))
            .unwrap_or_default();
        let code_refs = render_code_refs(&edge.code_refs);
        lines.push(format!(
            "  - Edge{edge_id}{label_suffix}: `{}` -> `{}` ({}){}{}",
            edge.source_id,
            edge.target_id,
            edge.kind,
            mechanism_suffix,
            render_ref_suffix(&code_refs)
        ));
    }
    push_omitted_count(
        lines,
        selected_edges.len(),
        EDGE_LIMIT,
        "focused graph edges",
    );
}

fn render_scratch(scratch: Option<&EpiphanyScratchPad>) -> Option<String> {
    let scratch = scratch?;
    let mut lines = Vec::new();
    if let Some(summary) = scratch.summary.as_deref() {
        lines.push(format!("- Summary: {}", compact_text(summary)));
    }
    if let Some(hypothesis) = scratch.hypothesis.as_deref() {
        lines.push(format!("- Hypothesis: {}", compact_text(hypothesis)));
    }
    if let Some(next_probe) = scratch.next_probe.as_deref() {
        lines.push(format!("- Next probe: {}", compact_text(next_probe)));
    }

    if lines.is_empty() {
        None
    } else {
        Some(format!("## Scratch\n{}", lines.join("\n")))
    }
}

fn render_investigation_checkpoint(
    checkpoint: Option<&EpiphanyInvestigationCheckpoint>,
) -> Option<String> {
    let checkpoint = checkpoint?;
    let mut lines = vec![
        format!("- Id: `{}`", checkpoint.checkpoint_id),
        format!("- Kind: {}", compact_text(&checkpoint.kind)),
        format!(
            "- Disposition: {}",
            match checkpoint.disposition {
                EpiphanyInvestigationDisposition::ResumeReady => "resume_ready",
                EpiphanyInvestigationDisposition::RegatherRequired => "regather_required",
            }
        ),
        format!("- Focus: {}", compact_text(&checkpoint.focus)),
    ];
    if let Some(summary) = checkpoint.summary.as_deref() {
        lines.push(format!("- Summary: {}", compact_text(summary)));
    }
    if let Some(next_action) = checkpoint.next_action.as_deref() {
        lines.push(format!("- Next action: {}", compact_text(next_action)));
    }
    if let Some(turn_id) = checkpoint.captured_at_turn_id.as_deref() {
        lines.push(format!("- Captured at turn: `{turn_id}`"));
    }
    if !checkpoint.open_questions.is_empty() {
        lines.push(format!(
            "- Open questions: {}",
            checkpoint
                .open_questions
                .iter()
                .map(|question| compact_text(question))
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    if !checkpoint.evidence_ids.is_empty() {
        lines.push(format!(
            "- Evidence ids: {}",
            render_inline_ids(&checkpoint.evidence_ids)
        ));
    }
    let code_refs = render_code_refs(&checkpoint.code_refs);
    if !code_refs.is_empty() {
        lines.push(format!("- Code refs: {}", code_refs.join(", ")));
    }

    Some(format!("## Investigation Checkpoint\n{}", lines.join("\n")))
}

fn render_observations(observations: &[EpiphanyObservation]) -> Option<String> {
    if observations.is_empty() {
        return None;
    }

    let mut lines = Vec::new();
    for observation in observations.iter().take(OBSERVATION_LIMIT) {
        lines.push(format!(
            "- `{}` [{} via {}]: {}",
            observation.id,
            observation.status,
            observation.source_kind,
            compact_text(&observation.summary)
        ));
        if !observation.evidence_ids.is_empty() {
            lines.push(format!(
                "  evidence: {}",
                render_inline_ids(&observation.evidence_ids)
            ));
        }
        let code_refs = render_code_refs(&observation.code_refs);
        if !code_refs.is_empty() {
            lines.push(format!("  code refs: {}", code_refs.join(", ")));
        }
    }
    push_omitted_count(
        &mut lines,
        observations.len(),
        OBSERVATION_LIMIT,
        "observations",
    );

    Some(format!("## Recent Observations\n{}", lines.join("\n")))
}

fn render_evidence(evidence: &[EpiphanyEvidenceRecord]) -> Option<String> {
    if evidence.is_empty() {
        return None;
    }

    let mut lines = Vec::new();
    for record in evidence.iter().take(EVIDENCE_LIMIT) {
        let code_refs = render_code_refs(&record.code_refs);
        lines.push(format!(
            "- `{}` [{} / {}]: {}{}",
            record.id,
            record.kind,
            record.status,
            compact_text(&record.summary),
            render_ref_suffix(&code_refs)
        ));
    }
    push_omitted_count(
        &mut lines,
        evidence.len(),
        EVIDENCE_LIMIT,
        "evidence records",
    );

    Some(format!("## Recent Evidence\n{}", lines.join("\n")))
}

fn render_churn(churn: Option<&EpiphanyChurnState>) -> Option<String> {
    let churn = churn?;
    let mut lines = vec![
        format!(
            "- Understanding status: {}",
            compact_text(&churn.understanding_status)
        ),
        format!("- Diff pressure: {}", compact_text(&churn.diff_pressure)),
    ];
    if let Some(graph_freshness) = churn.graph_freshness.as_deref() {
        lines.push(format!(
            "- Graph freshness: {}",
            compact_text(graph_freshness)
        ));
    }
    if let Some(warning) = churn.warning.as_deref() {
        lines.push(format!("- Warning: {}", compact_text(warning)));
    }
    if let Some(unexplained_writes) = churn.unexplained_writes {
        lines.push(format!("- Unexplained writes: {unexplained_writes}"));
    }

    Some(format!("## Churn\n{}", lines.join("\n")))
}

fn render_mode(mode: Option<&EpiphanyModeState>) -> Option<String> {
    let mode = mode?;
    let mut line = format!("- Name: {}", compact_text(&mode.name));
    if let Some(kind) = mode.kind {
        line.push_str(&format!(" ({kind:?})"));
    }
    Some(format!("## Mode\n{line}"))
}

fn collect_focus_node_ids(
    frontier: Option<&EpiphanyGraphFrontier>,
    checkpoint: Option<&EpiphanyGraphCheckpoint>,
) -> Vec<String> {
    let mut focus_node_ids = Vec::new();
    let mut seen = HashSet::new();
    for node_id in frontier
        .into_iter()
        .flat_map(|value| value.active_node_ids.iter())
        .chain(
            checkpoint
                .into_iter()
                .flat_map(|value| value.frontier_node_ids.iter()),
        )
    {
        if seen.insert(node_id.as_str()) {
            focus_node_ids.push(node_id.clone());
        }
    }
    focus_node_ids
}

fn select_nodes<'a>(
    graph: &'a EpiphanyGraph,
    focus_node_ids: &[String],
) -> Vec<&'a EpiphanyGraphNode> {
    let mut selected = Vec::new();
    let mut seen = HashSet::new();
    let focus_set = focus_node_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();

    for node in &graph.nodes {
        if focus_set.contains(node.id.as_str()) && seen.insert(node.id.as_str()) {
            selected.push(node);
            if selected.len() == NODE_LIMIT {
                return selected;
            }
        }
    }

    for node in &graph.nodes {
        if seen.insert(node.id.as_str()) {
            selected.push(node);
            if selected.len() == NODE_LIMIT {
                break;
            }
        }
    }

    selected
}

fn select_edges<'a>(
    graph: &'a EpiphanyGraph,
    selected_node_ids: &HashSet<&'a str>,
    active_edge_ids: &HashSet<&str>,
) -> Vec<&'a EpiphanyGraphEdge> {
    graph
        .edges
        .iter()
        .filter(|edge| {
            edge.id
                .as_deref()
                .is_some_and(|edge_id| active_edge_ids.contains(edge_id))
                || selected_node_ids.contains(edge.source_id.as_str())
                || selected_node_ids.contains(edge.target_id.as_str())
        })
        .collect()
}

fn select_links<'a>(
    links: &'a [EpiphanyGraphLink],
    focus_node_ids: &[String],
) -> Vec<&'a EpiphanyGraphLink> {
    let focus_set = focus_node_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut focused = Vec::new();
    let mut fallback = Vec::new();
    for link in links {
        if focus_set.contains(link.dataflow_node_id.as_str())
            || focus_set.contains(link.architecture_node_id.as_str())
        {
            focused.push(link);
        } else {
            fallback.push(link);
        }
    }
    focused.extend(fallback);
    focused
}

fn render_code_refs(code_refs: &[EpiphanyCodeRef]) -> Vec<String> {
    let mut rendered = code_refs
        .iter()
        .take(CODE_REF_LIMIT)
        .map(render_code_ref)
        .collect::<Vec<_>>();
    if code_refs.len() > CODE_REF_LIMIT {
        rendered.push(format!(
            "... {} more refs",
            code_refs.len() - CODE_REF_LIMIT
        ));
    }
    rendered
}

fn render_code_ref(code_ref: &EpiphanyCodeRef) -> String {
    let mut location = normalize_path(&code_ref.path);
    match (code_ref.start_line, code_ref.end_line) {
        (Some(start_line), Some(end_line)) if end_line != start_line => {
            location.push_str(&format!(":{start_line}-{end_line}"));
        }
        (Some(start_line), _) => {
            location.push_str(&format!(":{start_line}"));
        }
        _ => {}
    }

    let mut metadata = Vec::new();
    if let Some(symbol) = code_ref.symbol.as_deref() {
        metadata.push(format!("symbol: {}", compact_text(symbol)));
    }
    if let Some(note) = code_ref.note.as_deref() {
        metadata.push(compact_text(note));
    }

    if metadata.is_empty() {
        format!("`{location}`")
    } else {
        format!("`{location}` ({})", metadata.join("; "))
    }
}

fn render_ref_suffix(code_refs: &[String]) -> String {
    if code_refs.is_empty() {
        String::new()
    } else {
        format!("; refs: {}", code_refs.join(", "))
    }
}

fn render_inline_ids(ids: &[String]) -> String {
    ids.iter()
        .map(|id| format!("`{id}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn compact_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn omitted_suffix(total: usize, shown: usize, label: &str) -> String {
    if total > shown {
        format!("; ... {} more {label} not shown", total - shown)
    } else {
        String::new()
    }
}

fn push_omitted_count(lines: &mut Vec<String>, total: usize, shown: usize, label: &str) {
    if total > shown {
        lines.push(format!("- ... {} more {label} not shown", total - shown));
    }
}

#[cfg(test)]
mod tests {
    use super::render_epiphany_state;
    use codex_protocol::config_types::ModeKind;
    use codex_protocol::protocol::EpiphanyChurnState;
    use codex_protocol::protocol::EpiphanyCodeRef;
    use codex_protocol::protocol::EpiphanyGraph;
    use codex_protocol::protocol::EpiphanyGraphCheckpoint;
    use codex_protocol::protocol::EpiphanyGraphEdge;
    use codex_protocol::protocol::EpiphanyGraphFrontier;
    use codex_protocol::protocol::EpiphanyGraphNode;
    use codex_protocol::protocol::EpiphanyGraphs;
    use codex_protocol::protocol::EpiphanyInvestigationCheckpoint;
    use codex_protocol::protocol::EpiphanyInvestigationDisposition;
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
            investigation_checkpoint: Some(EpiphanyInvestigationCheckpoint {
                checkpoint_id: "ix-1".to_string(),
                kind: "slice_planning".to_string(),
                disposition: EpiphanyInvestigationDisposition::ResumeReady,
                focus: "Map the state update seam before broad edits.".to_string(),
                next_action: Some(
                    "Re-read the write path, then patch one bounded field.".to_string(),
                ),
                open_questions: vec![
                    "Do scene and context both need the resume packet?".to_string(),
                ],
                code_refs: vec![EpiphanyCodeRef {
                    path: PathBuf::from("core/src/codex_thread.rs"),
                    start_line: Some(376),
                    end_line: Some(424),
                    symbol: Some("epiphany_update_state".to_string()),
                    note: Some("Checkpoint writes must use the same red pen.".to_string()),
                }],
                evidence_ids: vec!["ev-1".to_string()],
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

        let rendered = render_epiphany_state(&state);

        assert!(rendered.contains("`arch-session`"));
        assert!(rendered.contains("`flow-build-context`"));
        assert!(rendered.contains("## Epiphany Doctrine"));
        assert!(rendered.contains("evidence is a distilled durable ledger"));
        assert!(rendered.contains("modeling is the Body"));
        assert!(rendered.contains("implementation is the Hands"));
        assert!(rendered.contains("research is the Eyes"));
        assert!(rendered.contains("Implement user-specified algorithms first"));
        assert!(rendered.contains("bank scratch/map/evidence before the dark"));
        assert!(rendered.contains("## Investigation Checkpoint"));
        assert!(rendered.contains("resume_ready"));
        assert!(rendered.contains("Map the state update seam before broad edits."));
        assert!(!rendered.contains("## Scratch"));
        assert!(!rendered.contains("## Invariants"));
        assert!(rendered.contains("`core/src/session/mod.rs:2433-2617`"));
        assert!(rendered.contains("... 1 more graph nodes not shown"));
    }
}
