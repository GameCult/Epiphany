use super::EpiphanyMemoryDomain;
use super::EpiphanyMemoryEdge;
use super::EpiphanyMemoryGraphSnapshot;
use super::EpiphanyMemoryLifecycle;
use super::EpiphanyMemoryNode;
use super::EpiphanyMemoryProfile;
use super::EpiphanyMemorySummary;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyMemoryGraphValidationError {
    pub path: String,
    pub message: String,
}

impl EpiphanyMemoryGraphValidationError {
    pub(crate) fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

pub fn validate_memory_graph_snapshot(
    snapshot: &EpiphanyMemoryGraphSnapshot,
) -> Vec<EpiphanyMemoryGraphValidationError> {
    let mut errors = Vec::new();
    if snapshot.graph_id.trim().is_empty() {
        errors.push(EpiphanyMemoryGraphValidationError::new(
            "graph_id",
            "graph id is required",
        ));
    }

    let domain_ids = collect_unique(
        snapshot.domains.iter().map(|domain| domain.id.as_str()),
        "domains",
        &mut errors,
    );
    for (index, domain) in snapshot.domains.iter().enumerate() {
        validate_domain(domain, index, &mut errors);
    }

    let node_ids = collect_unique(
        snapshot.nodes.iter().map(|node| node.id.as_str()),
        "nodes",
        &mut errors,
    );
    for (index, node) in snapshot.nodes.iter().enumerate() {
        validate_node(node, index, &domain_ids, &mut errors);
    }

    let edge_ids = collect_unique(
        snapshot.edges.iter().map(|edge| edge.id.as_str()),
        "edges",
        &mut errors,
    );
    for (index, edge) in snapshot.edges.iter().enumerate() {
        validate_edge(edge, index, &node_ids, &mut errors);
    }

    for (index, summary) in snapshot.summaries.iter().enumerate() {
        validate_summary(
            summary,
            index,
            &domain_ids,
            &node_ids,
            &edge_ids,
            &mut errors,
        );
    }

    errors
}

fn validate_domain(
    domain: &EpiphanyMemoryDomain,
    index: usize,
    errors: &mut Vec<EpiphanyMemoryGraphValidationError>,
) {
    let path = format!("domains[{index}]");
    required(
        &domain.id,
        format!("{path}.id"),
        "domain id is required",
        errors,
    );
    required(
        &domain.title,
        format!("{path}.title"),
        "domain title is required",
        errors,
    );
    if !lifecycle_allowed_for_profile(domain.profile, domain.lifecycle) {
        errors.push(EpiphanyMemoryGraphValidationError::new(
            format!("{path}.lifecycle"),
            "domain lifecycle is not legal for profile",
        ));
    }
}

fn validate_node(
    node: &EpiphanyMemoryNode,
    index: usize,
    domain_ids: &HashSet<String>,
    errors: &mut Vec<EpiphanyMemoryGraphValidationError>,
) {
    let path = format!("nodes[{index}]");
    required(
        &node.id,
        format!("{path}.id"),
        "node id is required",
        errors,
    );
    required(
        &node.domain_id,
        format!("{path}.domain_id"),
        "node domain id is required",
        errors,
    );
    if !domain_ids.contains(&node.domain_id) {
        errors.push(EpiphanyMemoryGraphValidationError::new(
            format!("{path}.domain_id"),
            "node references missing domain",
        ));
    }
    required(
        &node.title,
        format!("{path}.title"),
        "node title is required",
        errors,
    );
    required(
        &node.claim,
        format!("{path}.claim"),
        "node claim is required",
        errors,
    );
    required(
        &node.action_implication,
        format!("{path}.action_implication"),
        "node action implication is required",
        errors,
    );
    if node.question.trim().is_empty() && node.tension.trim().is_empty() {
        errors.push(EpiphanyMemoryGraphValidationError::new(
            format!("{path}.question"),
            "node must preserve a question or tension",
        ));
    }
    if node.anchors.is_empty()
        && !node
            .source_hashes
            .iter()
            .any(|hash| hash == "anchor:missing")
    {
        errors.push(EpiphanyMemoryGraphValidationError::new(
            format!("{path}.anchors"),
            "node requires anchors or source hash anchor:missing",
        ));
    }
    if !lifecycle_allowed_for_profile(node.profile, node.lifecycle) {
        errors.push(EpiphanyMemoryGraphValidationError::new(
            format!("{path}.lifecycle"),
            "node lifecycle is not legal for profile",
        ));
    }
}

fn validate_edge(
    edge: &EpiphanyMemoryEdge,
    index: usize,
    node_ids: &HashSet<String>,
    errors: &mut Vec<EpiphanyMemoryGraphValidationError>,
) {
    let path = format!("edges[{index}]");
    required(
        &edge.id,
        format!("{path}.id"),
        "edge id is required",
        errors,
    );
    if !node_ids.contains(&edge.source_id) {
        errors.push(EpiphanyMemoryGraphValidationError::new(
            format!("{path}.source_id"),
            "edge references missing source node",
        ));
    }
    if !node_ids.contains(&edge.target_id) {
        errors.push(EpiphanyMemoryGraphValidationError::new(
            format!("{path}.target_id"),
            "edge references missing target node",
        ));
    }
    required(
        &edge.claim,
        format!("{path}.claim"),
        "edge claim is required",
        errors,
    );
    if !lifecycle_allowed_for_profile(edge.profile, edge.lifecycle) {
        errors.push(EpiphanyMemoryGraphValidationError::new(
            format!("{path}.lifecycle"),
            "edge lifecycle is not legal for profile",
        ));
    }
}

fn validate_summary(
    summary: &EpiphanyMemorySummary,
    index: usize,
    domain_ids: &HashSet<String>,
    node_ids: &HashSet<String>,
    edge_ids: &HashSet<String>,
    errors: &mut Vec<EpiphanyMemoryGraphValidationError>,
) {
    let path = format!("summaries[{index}]");
    required(
        &summary.id,
        format!("{path}.id"),
        "summary id is required",
        errors,
    );
    if !domain_ids.contains(&summary.domain_id) {
        errors.push(EpiphanyMemoryGraphValidationError::new(
            format!("{path}.domain_id"),
            "summary references missing domain",
        ));
    }
    if summary.covers_node_ids.is_empty() && summary.covers_edge_ids.is_empty() {
        errors.push(EpiphanyMemoryGraphValidationError::new(
            format!("{path}.covers_node_ids"),
            "summary must cover at least one node or edge",
        ));
    }
    for node_id in &summary.covers_node_ids {
        if !node_ids.contains(node_id) {
            errors.push(EpiphanyMemoryGraphValidationError::new(
                format!("{path}.covers_node_ids"),
                format!("summary covers missing node {node_id}"),
            ));
        }
    }
    for edge_id in &summary.covers_edge_ids {
        if !edge_ids.contains(edge_id) {
            errors.push(EpiphanyMemoryGraphValidationError::new(
                format!("{path}.covers_edge_ids"),
                format!("summary covers missing edge {edge_id}"),
            ));
        }
    }
    required(
        &summary.target,
        format!("{path}.target"),
        "summary target is required",
        errors,
    );
    required(
        &summary.claim,
        format!("{path}.claim"),
        "summary claim is required",
        errors,
    );
    required(
        &summary.action_implication,
        format!("{path}.action_implication"),
        "summary action implication is required",
        errors,
    );
    if summary.question.trim().is_empty() && summary.tension.trim().is_empty() {
        errors.push(EpiphanyMemoryGraphValidationError::new(
            format!("{path}.question"),
            "summary must preserve a question or tension",
        ));
    }
}

pub fn lifecycle_allowed_for_profile(
    profile: EpiphanyMemoryProfile,
    lifecycle: EpiphanyMemoryLifecycle,
) -> bool {
    use EpiphanyMemoryLifecycle::*;
    use EpiphanyMemoryProfile::*;
    match profile {
        RepoArchitecture | RepoDataflow => {
            matches!(lifecycle, Observed | Proposed | Accepted | Stale | Retired)
        }
        RoleSelf | Identity => matches!(
            lifecycle,
            Proposed | Accepted | Revised | Retired | Crystallized
        ),
        ShortTerm => matches!(
            lifecycle,
            Active | Clustered | Distilled | Incubated | Pruned
        ),
        Incubation => matches!(lifecycle, Active | Deepening | Cooling | Promoted | Retired),
        AgencyPressure => matches!(lifecycle, Active | Obligated | Cooling | Answered | Retired),
        CandidateIntervention => {
            matches!(lifecycle, Queued | Deferred | Spoken | Applied | Retired)
        }
        Evidence => matches!(
            lifecycle,
            Observed | Reviewed | Accepted | Contradicted | Superseded | Retired
        ),
    }
}

fn collect_unique<'a>(
    ids: impl IntoIterator<Item = &'a str>,
    path: &str,
    errors: &mut Vec<EpiphanyMemoryGraphValidationError>,
) -> HashSet<String> {
    let mut seen = HashSet::new();
    for id in ids {
        if id.trim().is_empty() {
            continue;
        }
        if !seen.insert(id.to_string()) {
            errors.push(EpiphanyMemoryGraphValidationError::new(
                path,
                format!("duplicate id {id}"),
            ));
        }
    }
    seen
}

fn required(
    value: &str,
    path: String,
    message: &str,
    errors: &mut Vec<EpiphanyMemoryGraphValidationError>,
) {
    if value.trim().is_empty() {
        errors.push(EpiphanyMemoryGraphValidationError::new(path, message));
    }
}
