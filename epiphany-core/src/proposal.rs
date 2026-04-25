use anyhow::Result;
use anyhow::anyhow;
use codex_protocol::protocol::EpiphanyChurnState;
use codex_protocol::protocol::EpiphanyCodeRef;
use codex_protocol::protocol::EpiphanyEvidenceRecord;
use codex_protocol::protocol::EpiphanyGraphEdge;
use codex_protocol::protocol::EpiphanyGraphFrontier;
use codex_protocol::protocol::EpiphanyGraphNode;
use codex_protocol::protocol::EpiphanyGraphs;
use codex_protocol::protocol::EpiphanyObservation;
use codex_protocol::protocol::EpiphanyThreadState;
use sha1::Digest;
use sha1::Sha1;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

const SUMMARY_LIMIT: usize = 220;
const SEMANTIC_REUSE_MIN_SCORE: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectionQuality {
    observation_count: usize,
    evidence_count: usize,
    source_kinds: Vec<String>,
    priority_label: String,
    primary_observation_id: String,
    primary_observation_summary: String,
    semantic_terms: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArchitectureMatchKind {
    ExactCodeRef,
    SamePath,
    DeterministicId,
    Semantic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArchitectureMatch {
    index: usize,
    kind: ArchitectureMatchKind,
}

#[derive(Debug, Clone)]
pub struct EpiphanyMapProposalInput {
    pub state: EpiphanyThreadState,
    pub observation_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyMapProposal {
    pub observation: EpiphanyObservation,
    pub evidence: EpiphanyEvidenceRecord,
    pub graphs: EpiphanyGraphs,
    pub graph_frontier: EpiphanyGraphFrontier,
    pub churn: EpiphanyChurnState,
}

pub fn propose_map_update(input: EpiphanyMapProposalInput) -> Result<EpiphanyMapProposal> {
    let observation_ids = normalize_observation_ids(input.observation_ids)?;
    let observations = select_observations(&input.state, &observation_ids)?;
    let selection_quality = evaluate_selection_quality(&input.state, &observations)?;
    let code_refs = collect_code_refs(&observations)?;
    let code_ref_paths = unique_code_ref_paths(&code_refs);
    let fingerprint = fingerprint(&input.state.revision, &observation_ids, &code_refs);
    let mut graphs = input.state.graphs.clone();
    let mut active_node_ids = Vec::new();
    let mut reused_nodes = 0usize;
    let mut created_nodes = 0usize;
    let mut semantic_reused_nodes = 0usize;

    for path in &code_ref_paths {
        let path_code_refs = code_refs_for_path(&code_refs, &path);
        let candidate_node_id = graph_node_id(&path);
        if let Some(node_match) = find_architecture_node_for_path(
            &graphs,
            &path_code_refs,
            path,
            &candidate_node_id,
            &selection_quality.semantic_terms,
        ) {
            let node = &mut graphs.architecture.nodes[node_match.index];
            active_node_ids.push(node.id.clone());
            merge_code_refs(&mut node.code_refs, path_code_refs);
            reused_nodes += 1;
            if node_match.kind == ArchitectureMatchKind::Semantic {
                semantic_reused_nodes += 1;
            }
        } else {
            let node_id = candidate_node_id;
            active_node_ids.push(node_id.clone());
            graphs.architecture.nodes.push(EpiphanyGraphNode {
                id: node_id,
                title: title_from_path(&path),
                purpose: truncate_chars(
                    &format!(
                        "Candidate implementation surface from verified observation: {}",
                        selection_quality.primary_observation_summary
                    ),
                    SUMMARY_LIMIT,
                ),
                mechanism: Some(format!(
                    "Proposed from prioritized observation {:?} and selected observation ids: {}",
                    selection_quality.primary_observation_id,
                    observation_ids.join(", "),
                )),
                metaphor: None,
                status: Some("candidate".to_string()),
                code_refs: path_code_refs,
            });
            created_nodes += 1;
        }
    }

    let mut frontier = input.state.graph_frontier.clone().unwrap_or_default();
    let frontier_node_ids = linked_frontier_node_ids(&graphs, unique_strings(active_node_ids));
    let frontier_edge_ids = incident_edge_ids(&graphs, &frontier_node_ids);
    merge_unique(&mut frontier.active_node_ids, frontier_node_ids);
    merge_unique(&mut frontier.active_edge_ids, frontier_edge_ids);
    merge_unique(&mut frontier.dirty_paths, code_ref_paths.clone());

    let evidence_id = format!("ev-map-proposal-{fingerprint}");
    let selection_summary = selection_quality_summary(&selection_quality, code_ref_paths.len());
    let observation = EpiphanyObservation {
        id: format!("obs-map-proposal-{fingerprint}"),
        summary: truncate_chars(
            &format!(
                "Map/churn proposal from {}: {}",
                selection_summary,
                observation_ids.join(", "),
            ),
            SUMMARY_LIMIT,
        ),
        source_kind: "epiphany-map-proposal".to_string(),
        status: "candidate".to_string(),
        code_refs: code_refs.clone(),
        evidence_ids: vec![evidence_id.clone()],
    };
    let evidence = EpiphanyEvidenceRecord {
        id: evidence_id,
        kind: "proposal".to_string(),
        status: "candidate".to_string(),
        summary: truncate_chars(
            &format!(
                "Proposed graph frontier and churn update from {}: {}; reused {reused_nodes} existing node(s), {semantic_reused_nodes} by semantic graph signal, created {created_nodes} new node(s)",
                selection_summary,
                observation_ids.join(", "),
            ),
            SUMMARY_LIMIT,
        ),
        code_refs,
    };
    let understanding_status = proposal_understanding_status(reused_nodes, created_nodes);
    let graph_freshness = proposal_graph_freshness(reused_nodes, created_nodes);
    let diff_pressure = proposal_diff_pressure(
        reused_nodes,
        created_nodes,
        code_ref_paths.len(),
        selection_quality.observation_count,
        input.state.churn.as_ref(),
    );
    let churn = EpiphanyChurnState {
        understanding_status,
        diff_pressure,
        graph_freshness: Some(graph_freshness),
        warning: Some(format!(
            "Map/churn proposal derived from {selection_summary}; reused {reused_nodes} existing node(s), {semantic_reused_nodes} by semantic graph signal, created {created_nodes} new node(s); promote only after verifier acceptance."
        )),
        unexplained_writes: input
            .state
            .churn
            .as_ref()
            .and_then(|churn| churn.unexplained_writes),
    };

    Ok(EpiphanyMapProposal {
        observation,
        evidence,
        graphs,
        graph_frontier: frontier,
        churn,
    })
}

fn normalize_observation_ids(observation_ids: Vec<String>) -> Result<Vec<String>> {
    let mut normalized = Vec::new();
    let mut seen = HashSet::new();
    for id in observation_ids {
        let id = id.trim();
        if id.is_empty() {
            continue;
        }
        if seen.insert(id.to_string()) {
            normalized.push(id.to_string());
        }
    }
    if normalized.is_empty() {
        Err(anyhow!("observationIds must include at least one id"))
    } else {
        Ok(normalized)
    }
}

fn select_observations<'a>(
    state: &'a EpiphanyThreadState,
    observation_ids: &[String],
) -> Result<Vec<&'a EpiphanyObservation>> {
    let mut selected = Vec::new();
    for observation_id in observation_ids {
        let observation = state
            .observations
            .iter()
            .find(|observation| observation.id == *observation_id)
            .ok_or_else(|| anyhow!("observation id {observation_id:?} not found in state"))?;
        if !is_verified_status(&observation.status) {
            return Err(anyhow!(
                "observation id {observation_id:?} must have verified/accepted status, got {:?}",
                observation.status
            ));
        }
        selected.push(observation);
    }
    Ok(selected)
}

fn evaluate_selection_quality(
    state: &EpiphanyThreadState,
    observations: &[&EpiphanyObservation],
) -> Result<SelectionQuality> {
    let mut evidence_ids = Vec::new();
    let mut seen_evidence_ids = HashSet::new();
    let mut source_kinds = Vec::new();
    let mut seen_source_kinds = HashSet::new();
    let mut semantic_terms = Vec::new();
    let mut seen_semantic_terms = HashSet::new();
    let mut primary_observation_id = String::new();
    let mut primary_observation_summary = String::new();
    let mut primary_score = 0usize;
    let mut strong_signal_count = 0usize;

    for observation in observations {
        if observation.evidence_ids.is_empty() {
            return Err(anyhow!(
                "observation id {:?} must cite at least one evidence record",
                observation.id
            ));
        }

        let mut has_accepting_evidence = false;
        let mut observation_score = observation_priority_score(observation);
        push_semantic_terms(
            &mut semantic_terms,
            &mut seen_semantic_terms,
            &observation.summary,
        );
        push_semantic_terms(
            &mut semantic_terms,
            &mut seen_semantic_terms,
            &observation.source_kind,
        );
        push_code_ref_semantic_terms(
            &mut semantic_terms,
            &mut seen_semantic_terms,
            &observation.code_refs,
        );
        for evidence_id in &observation.evidence_ids {
            let evidence = state
                .recent_evidence
                .iter()
                .find(|evidence| evidence.id == *evidence_id)
                .ok_or_else(|| {
                    anyhow!(
                        "observation id {:?} cites missing evidence id {:?}",
                        observation.id,
                        evidence_id
                    )
                })?;
            if !is_verified_status(&evidence.status) {
                return Err(anyhow!(
                    "observation id {:?} cites non-accepting evidence id {:?} with status {:?}",
                    observation.id,
                    evidence_id,
                    evidence.status
                ));
            }
            has_accepting_evidence = true;
            observation_score += evidence_priority_score(evidence);
            if seen_evidence_ids.insert(evidence.id.clone()) {
                evidence_ids.push(evidence.id.clone());
            }
            push_semantic_terms(
                &mut semantic_terms,
                &mut seen_semantic_terms,
                &evidence.summary,
            );
            push_semantic_terms(
                &mut semantic_terms,
                &mut seen_semantic_terms,
                &evidence.kind,
            );
            push_code_ref_semantic_terms(
                &mut semantic_terms,
                &mut seen_semantic_terms,
                &evidence.code_refs,
            );
        }

        if !has_accepting_evidence {
            return Err(anyhow!(
                "observation id {:?} must cite at least one accepting evidence record",
                observation.id
            ));
        }

        let source_kind = observation.source_kind.trim();
        if !source_kind.is_empty() && seen_source_kinds.insert(source_kind.to_string()) {
            source_kinds.push(source_kind.to_string());
        }
        if observation_score >= 8 {
            strong_signal_count += 1;
        }
        if primary_observation_id.is_empty() || observation_score > primary_score {
            primary_observation_id = observation.id.clone();
            primary_observation_summary = observation.summary.clone();
            primary_score = observation_score;
        }
    }

    Ok(SelectionQuality {
        observation_count: observations.len(),
        evidence_count: evidence_ids.len(),
        source_kinds,
        priority_label: selection_priority_label(strong_signal_count, observations.len()),
        primary_observation_id,
        primary_observation_summary,
        semantic_terms,
    })
}

fn observation_priority_score(observation: &EpiphanyObservation) -> usize {
    signal_kind_weight(&observation.source_kind)
        + observation.code_refs.len().min(3)
        + usize::from(!observation.summary.trim().is_empty())
}

fn evidence_priority_score(evidence: &EpiphanyEvidenceRecord) -> usize {
    signal_kind_weight(&evidence.kind)
        + evidence.code_refs.len().min(2)
        + usize::from(!evidence.summary.trim().is_empty())
}

fn signal_kind_weight(value: &str) -> usize {
    let lower = value.trim().to_ascii_lowercase();
    if lower.contains("verification") || lower.contains("verifier") {
        4
    } else if lower.contains("test") || lower.contains("smoke") {
        3
    } else if lower.contains("tool") || lower.contains("review") {
        2
    } else if lower.contains("observation") {
        1
    } else {
        0
    }
}

fn selection_priority_label(strong_signal_count: usize, observation_count: usize) -> String {
    if strong_signal_count == observation_count {
        "high-priority verified selection".to_string()
    } else if strong_signal_count > 0 {
        "mixed-priority verified selection".to_string()
    } else {
        "low-priority accepted selection".to_string()
    }
}

fn collect_code_refs(observations: &[&EpiphanyObservation]) -> Result<Vec<EpiphanyCodeRef>> {
    let mut code_refs = Vec::new();
    let mut seen = HashSet::new();
    for observation in observations {
        for code_ref in &observation.code_refs {
            let key = code_ref_key(code_ref);
            if seen.insert(key) {
                code_refs.push(code_ref.clone());
            }
        }
    }
    if code_refs.is_empty() {
        Err(anyhow!(
            "map proposals require at least one code ref on the selected observations"
        ))
    } else {
        Ok(code_refs)
    }
}

fn unique_code_ref_paths(code_refs: &[EpiphanyCodeRef]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    for code_ref in code_refs {
        let key = code_ref.path.to_string_lossy().to_string();
        if seen.insert(key) {
            paths.push(code_ref.path.clone());
        }
    }
    paths
}

fn code_refs_for_path(code_refs: &[EpiphanyCodeRef], path: &Path) -> Vec<EpiphanyCodeRef> {
    code_refs
        .iter()
        .filter(|code_ref| code_ref.path == path)
        .cloned()
        .collect()
}

fn find_architecture_node_for_path(
    graphs: &EpiphanyGraphs,
    path_code_refs: &[EpiphanyCodeRef],
    path: &Path,
    candidate_node_id: &str,
    semantic_terms: &[String],
) -> Option<ArchitectureMatch> {
    graphs
        .architecture
        .nodes
        .iter()
        .position(|node| has_exact_code_ref_overlap(&node.code_refs, path_code_refs))
        .map(|index| ArchitectureMatch {
            index,
            kind: ArchitectureMatchKind::ExactCodeRef,
        })
        .or_else(|| {
            graphs
                .architecture
                .nodes
                .iter()
                .position(|node| node.code_refs.iter().any(|code_ref| code_ref.path == path))
                .map(|index| ArchitectureMatch {
                    index,
                    kind: ArchitectureMatchKind::SamePath,
                })
        })
        .or_else(|| {
            graphs
                .architecture
                .nodes
                .iter()
                .position(|node| node.id == candidate_node_id)
                .map(|index| ArchitectureMatch {
                    index,
                    kind: ArchitectureMatchKind::DeterministicId,
                })
        })
        .or_else(|| {
            find_semantic_architecture_node(graphs, semantic_terms).map(|index| ArchitectureMatch {
                index,
                kind: ArchitectureMatchKind::Semantic,
            })
        })
}

fn has_exact_code_ref_overlap(
    existing_code_refs: &[EpiphanyCodeRef],
    proposed_code_refs: &[EpiphanyCodeRef],
) -> bool {
    let existing_keys = existing_code_refs
        .iter()
        .map(code_ref_key)
        .collect::<HashSet<_>>();
    proposed_code_refs
        .iter()
        .map(code_ref_key)
        .any(|key| existing_keys.contains(&key))
}

fn find_semantic_architecture_node(
    graphs: &EpiphanyGraphs,
    semantic_terms: &[String],
) -> Option<usize> {
    if semantic_terms.is_empty() {
        return None;
    }

    let mut best_index = None;
    let mut best_score = 0usize;
    let mut tied = false;
    for (index, node) in graphs.architecture.nodes.iter().enumerate() {
        let score = semantic_node_score(node, semantic_terms);
        if !semantic_score_can_reuse(node, score) {
            continue;
        }
        if score > best_score {
            best_index = Some(index);
            best_score = score;
            tied = false;
        } else if score == best_score {
            tied = true;
        }
    }

    if tied { None } else { best_index }
}

fn semantic_score_can_reuse(node: &EpiphanyGraphNode, score: usize) -> bool {
    if score < SEMANTIC_REUSE_MIN_SCORE {
        return false;
    }
    // Once a graph node is anchored to code, keep reuse on concrete ref/path/id checks.
    // Semantic matching is only a rescue path for useful map prose that has no refs yet.
    node.code_refs.is_empty()
}

fn semantic_node_score(node: &EpiphanyGraphNode, semantic_terms: &[String]) -> usize {
    let mut node_terms = Vec::new();
    let mut seen = HashSet::new();
    push_semantic_terms(&mut node_terms, &mut seen, &node.id);
    push_semantic_terms(&mut node_terms, &mut seen, &node.title);
    push_semantic_terms(&mut node_terms, &mut seen, &node.purpose);
    if let Some(mechanism) = node.mechanism.as_deref() {
        push_semantic_terms(&mut node_terms, &mut seen, mechanism);
    }
    if let Some(metaphor) = node.metaphor.as_deref() {
        push_semantic_terms(&mut node_terms, &mut seen, metaphor);
    }
    if let Some(status) = node.status.as_deref() {
        push_semantic_terms(&mut node_terms, &mut seen, status);
    }
    push_code_ref_semantic_terms(&mut node_terms, &mut seen, &node.code_refs);

    let node_terms = node_terms.into_iter().collect::<HashSet<_>>();
    semantic_terms
        .iter()
        .filter(|term| node_terms.contains(term.as_str()))
        .count()
}

fn merge_code_refs(target: &mut Vec<EpiphanyCodeRef>, additions: Vec<EpiphanyCodeRef>) {
    let mut seen = target.iter().map(code_ref_key).collect::<HashSet<_>>();
    for addition in additions {
        if seen.insert(code_ref_key(&addition)) {
            target.push(addition);
        }
    }
}

fn linked_frontier_node_ids(graphs: &EpiphanyGraphs, direct_node_ids: Vec<String>) -> Vec<String> {
    let mut linked_node_ids = direct_node_ids;
    let mut seen = linked_node_ids.iter().cloned().collect::<HashSet<_>>();
    let mut changed = true;

    while changed {
        changed = false;
        for link in &graphs.links {
            if seen.contains(&link.architecture_node_id)
                && seen.insert(link.dataflow_node_id.clone())
            {
                linked_node_ids.push(link.dataflow_node_id.clone());
                changed = true;
            }
            if seen.contains(&link.dataflow_node_id)
                && seen.insert(link.architecture_node_id.clone())
            {
                linked_node_ids.push(link.architecture_node_id.clone());
                changed = true;
            }
        }
    }

    linked_node_ids
}

fn incident_edge_ids(graphs: &EpiphanyGraphs, node_ids: &[String]) -> Vec<String> {
    let node_ids = node_ids.iter().map(String::as_str).collect::<HashSet<_>>();
    let mut edge_ids = Vec::new();
    let mut seen = HashSet::new();
    for edge in graphs
        .architecture
        .edges
        .iter()
        .chain(graphs.dataflow.edges.iter())
    {
        if is_incident_to_any_node(edge, &node_ids)
            && let Some(edge_id) = edge.id.as_deref().map(str::trim)
            && !edge_id.is_empty()
            && seen.insert(edge_id.to_string())
        {
            edge_ids.push(edge_id.to_string());
        }
    }
    edge_ids
}

fn is_incident_to_any_node(edge: &EpiphanyGraphEdge, node_ids: &HashSet<&str>) -> bool {
    node_ids.contains(edge.source_id.as_str()) || node_ids.contains(edge.target_id.as_str())
}

fn merge_unique<T>(target: &mut Vec<T>, additions: Vec<T>)
where
    T: Clone + Eq + std::hash::Hash,
{
    let mut seen: HashSet<T> = target.iter().cloned().collect();
    for addition in additions {
        if seen.insert(addition.clone()) {
            target.push(addition);
        }
    }
}

fn unique_strings(values: Vec<String>) -> Vec<String> {
    let mut unique = Vec::new();
    let mut seen = HashSet::new();
    for value in values {
        if seen.insert(value.clone()) {
            unique.push(value);
        }
    }
    unique
}

fn graph_node_id(path: &Path) -> String {
    let mut hasher = Sha1::new();
    hasher.update(path.to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    let suffix: String = format!("{digest:x}").chars().take(12).collect();
    format!("arch-path-{suffix}")
}

fn title_from_path(path: &Path) -> String {
    if let Some(file_name) = path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
    {
        file_name.to_string()
    } else {
        path.to_string_lossy().to_string()
    }
}

fn fingerprint(
    revision: &u64,
    observation_ids: &[String],
    code_refs: &[EpiphanyCodeRef],
) -> String {
    let mut hasher = Sha1::new();
    hasher.update(revision.to_string().as_bytes());
    for observation_id in observation_ids {
        hasher.update(b"\0");
        hasher.update(observation_id.as_bytes());
    }
    for code_ref in code_refs {
        hasher.update(b"\0");
        hasher.update(code_ref_key(code_ref).as_bytes());
    }
    let digest = hasher.finalize();
    format!("{digest:x}").chars().take(12).collect()
}

fn code_ref_key(code_ref: &EpiphanyCodeRef) -> String {
    format!(
        "{}:{}:{}:{}",
        code_ref.path.to_string_lossy(),
        code_ref.start_line.unwrap_or_default(),
        code_ref.end_line.unwrap_or_default(),
        code_ref.symbol.as_deref().unwrap_or_default()
    )
}

fn push_code_ref_semantic_terms(
    target: &mut Vec<String>,
    seen: &mut HashSet<String>,
    code_refs: &[EpiphanyCodeRef],
) {
    for code_ref in code_refs {
        push_path_semantic_terms(target, seen, &code_ref.path);
        if let Some(symbol) = code_ref.symbol.as_deref() {
            push_semantic_terms(target, seen, symbol);
        }
        if let Some(note) = code_ref.note.as_deref() {
            push_semantic_terms(target, seen, note);
        }
    }
}

fn push_path_semantic_terms(target: &mut Vec<String>, seen: &mut HashSet<String>, path: &Path) {
    if let Some(file_stem) = path.file_stem().and_then(|value| value.to_str()) {
        push_semantic_terms(target, seen, file_stem);
    }
    if let Some(parent) = path.parent().and_then(|value| value.to_str()) {
        push_semantic_terms(target, seen, parent);
    }
}

fn push_semantic_terms(target: &mut Vec<String>, seen: &mut HashSet<String>, value: &str) {
    for term in semantic_terms(value) {
        if seen.insert(term.clone()) {
            target.push(term);
        }
    }
}

fn semantic_terms(value: &str) -> Vec<String> {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .map(str::trim)
        .filter(|term| term.len() >= 3)
        .map(str::to_ascii_lowercase)
        .filter(|term| !is_semantic_stopword(term))
        .collect()
}

fn is_semantic_stopword(value: &str) -> bool {
    matches!(
        value,
        "and"
            | "are"
            | "but"
            | "can"
            | "for"
            | "from"
            | "has"
            | "into"
            | "not"
            | "now"
            | "one"
            | "out"
            | "the"
            | "this"
            | "that"
            | "through"
            | "with"
    )
}

fn truncate_chars(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let mut truncated = value
        .chars()
        .take(limit.saturating_sub(3))
        .collect::<String>();
    truncated.push_str("...");
    truncated
}

fn proposal_understanding_status(reused_nodes: usize, created_nodes: usize) -> String {
    match (reused_nodes > 0, created_nodes > 0) {
        (true, false) => "proposal_refines_map",
        (false, true) => "proposal_expands_map",
        (true, true) => "proposal_updates_map",
        (false, false) => "proposal_ready",
    }
    .to_string()
}

fn proposal_graph_freshness(reused_nodes: usize, created_nodes: usize) -> String {
    match (reused_nodes > 0, created_nodes > 0) {
        (true, false) => "proposal-refined",
        (false, true) => "proposal-expanded",
        (true, true) => "proposal-updated",
        (false, false) => "proposal",
    }
    .to_string()
}

fn proposal_diff_pressure(
    reused_nodes: usize,
    created_nodes: usize,
    path_count: usize,
    observation_count: usize,
    existing_churn: Option<&EpiphanyChurnState>,
) -> String {
    let proposal_pressure = if created_nodes > 1
        || (reused_nodes > 0 && created_nodes > 0)
        || path_count > 2
        || observation_count > 3
        || existing_churn
            .and_then(|churn| churn.unexplained_writes)
            .is_some_and(|unexplained_writes| unexplained_writes > 0)
    {
        "high"
    } else if created_nodes > 0 || path_count > 1 || observation_count > 1 {
        "medium"
    } else {
        "low"
    };

    let existing_pressure = existing_churn
        .map(|churn| churn.diff_pressure.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or("low");

    max_pressure(existing_pressure, proposal_pressure).to_string()
}

fn max_pressure<'a>(left: &'a str, right: &'a str) -> &'a str {
    if pressure_rank(left) >= pressure_rank(right) {
        left
    } else {
        right
    }
}

fn pressure_rank(value: &str) -> u8 {
    match value.trim().to_ascii_lowercase().as_str() {
        "critical" => 4,
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

fn selection_quality_summary(selection_quality: &SelectionQuality, path_count: usize) -> String {
    let source_kinds = if selection_quality.source_kinds.is_empty() {
        "unknown sources".to_string()
    } else {
        selection_quality.source_kinds.join("/")
    };
    format!(
        "{} evidence-backed observation(s), {} accepting evidence record(s), {} code path(s), {}, primary observation: {}, source kind(s): {}",
        selection_quality.observation_count,
        selection_quality.evidence_count,
        path_count,
        selection_quality.priority_label,
        selection_quality.primary_observation_id,
        source_kinds
    )
}

fn is_verified_status(status: &str) -> bool {
    matches!(
        status.trim().to_ascii_lowercase().as_str(),
        "ok" | "accepted" | "verified" | "pass" | "passed"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::promotion::EpiphanyPromotionInput;
    use crate::promotion::evaluate_promotion;
    use codex_protocol::protocol::EpiphanyGraph;
    use codex_protocol::protocol::EpiphanyGraphEdge;
    use codex_protocol::protocol::EpiphanyGraphLink;

    fn code_ref(path: &str) -> EpiphanyCodeRef {
        EpiphanyCodeRef {
            path: PathBuf::from(path),
            start_line: Some(10),
            end_line: Some(20),
            symbol: Some("demo".to_string()),
            note: None,
        }
    }

    fn state_with_observation(status: &str) -> EpiphanyThreadState {
        EpiphanyThreadState {
            revision: 4,
            observations: vec![EpiphanyObservation {
                id: "obs-verified".to_string(),
                summary: "The prompt renderer injects Epiphany state".to_string(),
                source_kind: "verification".to_string(),
                status: status.to_string(),
                code_refs: vec![code_ref("epiphany-core/src/prompt.rs")],
                evidence_ids: vec!["ev-verified".to_string()],
            }],
            recent_evidence: vec![EpiphanyEvidenceRecord {
                id: "ev-verified".to_string(),
                kind: "verification".to_string(),
                status: "ok".to_string(),
                summary: "Verified prompt renderer behavior".to_string(),
                code_refs: vec![code_ref("epiphany-core/src/prompt.rs")],
            }],
            churn: Some(EpiphanyChurnState {
                understanding_status: "grounded".to_string(),
                diff_pressure: "medium".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn verifier() -> EpiphanyEvidenceRecord {
        EpiphanyEvidenceRecord {
            id: "ev-verifier".to_string(),
            kind: "verification".to_string(),
            status: "ok".to_string(),
            summary: "Verifier accepted proposal".to_string(),
            code_refs: Vec::new(),
        }
    }

    #[test]
    fn propose_map_update_returns_candidate_patch_parts() {
        let proposal = propose_map_update(EpiphanyMapProposalInput {
            state: state_with_observation("verified"),
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect("proposal");

        assert!(proposal.observation.id.starts_with("obs-map-proposal-"));
        assert_eq!(proposal.evidence.status, "candidate");
        assert_eq!(proposal.graphs.architecture.nodes.len(), 1);
        assert_eq!(proposal.graphs.architecture.nodes[0].title, "prompt.rs");
        assert_eq!(proposal.graph_frontier.active_node_ids.len(), 1);
        assert_eq!(proposal.churn.understanding_status, "proposal_expands_map");
        assert_eq!(proposal.churn.diff_pressure, "medium");
    }

    #[test]
    fn propose_map_update_reuses_existing_architecture_node_for_matching_code_ref() {
        let mut state = state_with_observation("verified");
        state.graphs.architecture.nodes.push(EpiphanyGraphNode {
            id: "prompt-renderer".to_string(),
            title: "Prompt renderer".to_string(),
            purpose: "Render Epiphany state into developer context".to_string(),
            status: Some("grounded".to_string()),
            code_refs: vec![code_ref("epiphany-core/src/prompt.rs")],
            ..Default::default()
        });
        state.observations[0].code_refs.push(EpiphanyCodeRef {
            path: PathBuf::from("epiphany-core/src/prompt.rs"),
            start_line: Some(30),
            end_line: Some(40),
            symbol: Some("render_epiphany_state".to_string()),
            note: Some("newly verified line span".to_string()),
        });

        let proposal = propose_map_update(EpiphanyMapProposalInput {
            state,
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect("proposal");

        assert_eq!(proposal.graphs.architecture.nodes.len(), 1);
        let node = &proposal.graphs.architecture.nodes[0];
        assert_eq!(node.id, "prompt-renderer");
        assert_eq!(node.status.as_deref(), Some("grounded"));
        assert_eq!(node.code_refs.len(), 2);
        assert_eq!(
            proposal.graph_frontier.active_node_ids,
            vec!["prompt-renderer".to_string()]
        );
        assert_eq!(proposal.churn.understanding_status, "proposal_refines_map");
        assert_eq!(
            proposal.churn.graph_freshness.as_deref(),
            Some("proposal-refined")
        );
    }

    #[test]
    fn propose_map_update_reuses_existing_deterministic_path_node_without_refs() {
        let mut state = state_with_observation("verified");
        let node_id = graph_node_id(Path::new("epiphany-core/src/prompt.rs"));
        state.graphs.architecture.nodes.push(EpiphanyGraphNode {
            id: node_id.clone(),
            title: "Prompt path".to_string(),
            purpose: "Existing path-derived proposal node".to_string(),
            ..Default::default()
        });

        let proposal = propose_map_update(EpiphanyMapProposalInput {
            state,
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect("proposal");

        assert_eq!(proposal.graphs.architecture.nodes.len(), 1);
        assert_eq!(proposal.graphs.architecture.nodes[0].id, node_id);
        assert_eq!(proposal.graphs.architecture.nodes[0].code_refs.len(), 1);
        assert_eq!(proposal.churn.understanding_status, "proposal_refines_map");
    }

    #[test]
    fn propose_map_update_reuses_strong_semantic_graph_match_without_refs() {
        let mut state = state_with_observation("verified");
        state.graphs.architecture.nodes.push(EpiphanyGraphNode {
            id: "prompt-renderer".to_string(),
            title: "Prompt renderer".to_string(),
            purpose: "Inject Epiphany state into developer context".to_string(),
            mechanism: Some("Renders typed state as a bounded prompt fragment".to_string()),
            ..Default::default()
        });

        let proposal = propose_map_update(EpiphanyMapProposalInput {
            state,
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect("proposal");

        assert_eq!(proposal.graphs.architecture.nodes.len(), 1);
        assert_eq!(proposal.graphs.architecture.nodes[0].id, "prompt-renderer");
        assert_eq!(proposal.graphs.architecture.nodes[0].code_refs.len(), 1);
        assert_eq!(
            proposal.graph_frontier.active_node_ids,
            vec!["prompt-renderer".to_string()]
        );
        assert!(
            proposal
                .churn
                .warning
                .as_deref()
                .unwrap_or_default()
                .contains("1 by semantic graph signal")
        );
    }

    #[test]
    fn propose_map_update_refuses_ambiguous_semantic_graph_match() {
        let mut state = state_with_observation("verified");
        for node_id in ["prompt-renderer-a", "prompt-renderer-b"] {
            state.graphs.architecture.nodes.push(EpiphanyGraphNode {
                id: node_id.to_string(),
                title: "Prompt renderer".to_string(),
                purpose: "Inject Epiphany state into developer context".to_string(),
                mechanism: Some("Renders typed state as a bounded prompt fragment".to_string()),
                ..Default::default()
            });
        }

        let proposal = propose_map_update(EpiphanyMapProposalInput {
            state,
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect("proposal");

        assert_eq!(proposal.graphs.architecture.nodes.len(), 3);
        assert!(
            proposal
                .graph_frontier
                .active_node_ids
                .iter()
                .any(|node_id| node_id.starts_with("arch-path-"))
        );
        assert_eq!(proposal.churn.understanding_status, "proposal_expands_map");
    }

    #[test]
    fn propose_map_update_prioritizes_stronger_selected_observation() {
        let mut state = state_with_observation("ok");
        state.observations[0].id = "obs-weak".to_string();
        state.observations[0].source_kind = "observation".to_string();
        state.observations[0].summary = "Prompt renderer might be involved".to_string();
        state.observations[0].evidence_ids = vec!["ev-weak".to_string()];
        state.recent_evidence[0].id = "ev-weak".to_string();
        state.recent_evidence[0].kind = "observation".to_string();
        state.recent_evidence[0].summary = "Manual note mentioned prompt renderer".to_string();
        state.observations.push(EpiphanyObservation {
            id: "obs-strong".to_string(),
            summary: "Live smoke verified prompt renderer Epiphany state injection".to_string(),
            source_kind: "smoke".to_string(),
            status: "ok".to_string(),
            code_refs: vec![code_ref("epiphany-core/src/prompt.rs")],
            evidence_ids: vec!["ev-strong".to_string()],
        });
        state.recent_evidence.push(EpiphanyEvidenceRecord {
            id: "ev-strong".to_string(),
            kind: "verification".to_string(),
            status: "ok".to_string(),
            summary: "Verifier accepted prompt renderer state injection".to_string(),
            code_refs: vec![code_ref("epiphany-core/src/prompt.rs")],
        });

        let proposal = propose_map_update(EpiphanyMapProposalInput {
            state,
            observation_ids: vec!["obs-weak".to_string(), "obs-strong".to_string()],
        })
        .expect("proposal");

        assert!(
            proposal.graphs.architecture.nodes[0]
                .purpose
                .contains("Live smoke verified")
        );
        assert!(
            proposal
                .observation
                .summary
                .contains("primary observation: obs-strong")
        );
    }

    #[test]
    fn propose_map_update_focuses_linked_nodes_and_incident_edges() {
        let mut state = state_with_observation("verified");
        state.graphs = EpiphanyGraphs {
            architecture: EpiphanyGraph {
                nodes: vec![
                    EpiphanyGraphNode {
                        id: "session".to_string(),
                        title: "Session".to_string(),
                        purpose: "Build turn context".to_string(),
                        ..Default::default()
                    },
                    EpiphanyGraphNode {
                        id: "prompt-renderer".to_string(),
                        title: "Prompt renderer".to_string(),
                        purpose: "Render Epiphany state".to_string(),
                        code_refs: vec![code_ref("epiphany-core/src/prompt.rs")],
                        ..Default::default()
                    },
                ],
                edges: vec![EpiphanyGraphEdge {
                    id: Some("edge-session-prompt".to_string()),
                    source_id: "session".to_string(),
                    target_id: "prompt-renderer".to_string(),
                    kind: "calls".to_string(),
                    ..Default::default()
                }],
            },
            dataflow: EpiphanyGraph {
                nodes: vec![
                    EpiphanyGraphNode {
                        id: "developer-context".to_string(),
                        title: "Developer context".to_string(),
                        purpose: "Carry rendered instructions".to_string(),
                        ..Default::default()
                    },
                    EpiphanyGraphNode {
                        id: "epiphany-state-fragment".to_string(),
                        title: "Epiphany state fragment".to_string(),
                        purpose: "Expose typed state to the model".to_string(),
                        ..Default::default()
                    },
                ],
                edges: vec![EpiphanyGraphEdge {
                    id: Some("edge-fragment-context".to_string()),
                    source_id: "epiphany-state-fragment".to_string(),
                    target_id: "developer-context".to_string(),
                    kind: "feeds".to_string(),
                    ..Default::default()
                }],
            },
            links: vec![EpiphanyGraphLink {
                dataflow_node_id: "epiphany-state-fragment".to_string(),
                architecture_node_id: "prompt-renderer".to_string(),
                relationship: Some("rendered by".to_string()),
                code_refs: vec![code_ref("epiphany-core/src/prompt.rs")],
            }],
        };

        let proposal = propose_map_update(EpiphanyMapProposalInput {
            state,
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect("proposal");

        assert_eq!(
            proposal.graph_frontier.active_node_ids,
            vec![
                "prompt-renderer".to_string(),
                "epiphany-state-fragment".to_string()
            ]
        );
        assert_eq!(
            proposal.graph_frontier.active_edge_ids,
            vec![
                "edge-session-prompt".to_string(),
                "edge-fragment-context".to_string()
            ]
        );
    }

    #[test]
    fn propose_map_update_returns_promotion_acceptable_patch_parts() {
        let proposal = propose_map_update(EpiphanyMapProposalInput {
            state: state_with_observation("verified"),
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect("proposal");

        let decision = evaluate_promotion(EpiphanyPromotionInput {
            has_state_replacements: true,
            active_subgoal_id: None,
            subgoals: None,
            invariants: None,
            graphs: Some(proposal.graphs),
            graph_frontier: Some(proposal.graph_frontier),
            graph_checkpoint: None,
            churn: Some(proposal.churn),
            observations: vec![proposal.observation],
            evidence: vec![proposal.evidence],
            verifier_evidence: verifier(),
        });

        assert!(decision.accepted, "{:?}", decision.reasons);
    }

    #[test]
    fn propose_map_update_rejects_unverified_observation() {
        let err = propose_map_update(EpiphanyMapProposalInput {
            state: state_with_observation("failed"),
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect_err("unverified observation should fail");

        assert!(
            err.to_string()
                .contains("must have verified/accepted status")
        );
    }

    #[test]
    fn propose_map_update_rejects_missing_code_refs() {
        let mut state = state_with_observation("ok");
        state.observations[0].code_refs.clear();

        let err = propose_map_update(EpiphanyMapProposalInput {
            state,
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect_err("code refs are required");

        assert!(err.to_string().contains("at least one code ref"));
    }

    #[test]
    fn propose_map_update_rejects_observation_without_backing_evidence() {
        let mut state = state_with_observation("ok");
        state.recent_evidence.clear();

        let err = propose_map_update(EpiphanyMapProposalInput {
            state,
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect_err("missing backing evidence should fail");

        assert!(err.to_string().contains("cites missing evidence id"));
    }

    #[test]
    fn propose_map_update_rejects_non_accepting_backing_evidence() {
        let mut state = state_with_observation("ok");
        state.recent_evidence[0].status = "failed".to_string();

        let err = propose_map_update(EpiphanyMapProposalInput {
            state,
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect_err("non-accepting backing evidence should fail");

        assert!(err.to_string().contains("non-accepting evidence"));
    }

    #[test]
    fn propose_map_update_marks_mixed_multi_path_changes_as_high_pressure() {
        let mut state = state_with_observation("ok");
        state.churn = Some(EpiphanyChurnState {
            understanding_status: "grounded".to_string(),
            diff_pressure: "low".to_string(),
            ..Default::default()
        });
        state.graphs.architecture.nodes.push(EpiphanyGraphNode {
            id: "prompt-renderer".to_string(),
            title: "Prompt renderer".to_string(),
            purpose: "Render Epiphany state into developer context".to_string(),
            code_refs: vec![code_ref("epiphany-core/src/prompt.rs")],
            ..Default::default()
        });
        state.observations[0]
            .code_refs
            .push(code_ref("epiphany-core/src/proposal.rs"));

        let proposal = propose_map_update(EpiphanyMapProposalInput {
            state,
            observation_ids: vec!["obs-verified".to_string()],
        })
        .expect("proposal");

        assert_eq!(proposal.churn.understanding_status, "proposal_updates_map");
        assert_eq!(proposal.churn.diff_pressure, "high");
    }
}
