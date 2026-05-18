use super::EpiphanyMemoryGraphSnapshot;
use super::EpiphanyMemoryLifecycleReceipt;
use super::EpiphanyMemoryPatchCandidate;
use super::EpiphanyMemoryPatchReview;
use super::EpiphanyMemoryPatchReviewStatus;
use super::validate_memory_graph_snapshot;
use std::collections::HashSet;

pub fn review_memory_patch_candidate(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    candidate: &EpiphanyMemoryPatchCandidate,
) -> EpiphanyMemoryPatchReview {
    let accepted_receipt_id = accepted_receipt_id(candidate);
    if candidate_already_applied(snapshot, &accepted_receipt_id) {
        return EpiphanyMemoryPatchReview {
            candidate_id: candidate.id.clone(),
            status: EpiphanyMemoryPatchReviewStatus::Accepted,
            accepted_domain_ids: candidate
                .proposed_domains
                .iter()
                .map(|domain| domain.id.clone())
                .collect(),
            accepted_node_ids: candidate
                .proposed_nodes
                .iter()
                .map(|node| node.id.clone())
                .collect(),
            accepted_edge_ids: candidate
                .proposed_edges
                .iter()
                .map(|edge| edge.id.clone())
                .collect(),
            errors: Vec::new(),
            warnings: vec!["candidate was already applied".to_string()],
        };
    }

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if candidate.id.trim().is_empty() {
        errors.push("candidate id is required".to_string());
    }
    if candidate.status.trim().is_empty() {
        errors.push("candidate status is required".to_string());
    } else if candidate.status != "proposed" {
        warnings.push(format!(
            "candidate status {:?} is reviewable but not the normal proposed status",
            candidate.status
        ));
    }
    if candidate.proposed_domains.is_empty()
        && candidate.proposed_nodes.is_empty()
        && candidate.proposed_edges.is_empty()
    {
        errors.push("candidate must propose at least one domain, node, or edge".to_string());
    }

    let existing_domain_ids = snapshot
        .domains
        .iter()
        .map(|domain| domain.id.as_str())
        .collect::<HashSet<_>>();
    let existing_node_ids = snapshot
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<HashSet<_>>();
    let existing_edge_ids = snapshot
        .edges
        .iter()
        .map(|edge| edge.id.as_str())
        .collect::<HashSet<_>>();

    let mut proposed_domain_ids = HashSet::new();
    for domain in &candidate.proposed_domains {
        if domain.profile != candidate.profile {
            errors.push(format!(
                "proposed domain {} profile does not match candidate profile",
                domain.id
            ));
        }
        if existing_domain_ids.contains(domain.id.as_str()) {
            errors.push(format!(
                "proposed domain {} already exists; revision candidates need a separate reviewer",
                domain.id
            ));
        }
        if !proposed_domain_ids.insert(domain.id.as_str()) {
            errors.push(format!("candidate repeats proposed domain {}", domain.id));
        }
    }

    let mut proposed_node_ids = HashSet::new();
    for node in &candidate.proposed_nodes {
        if node.profile != candidate.profile {
            errors.push(format!(
                "proposed node {} profile does not match candidate profile",
                node.id
            ));
        }
        if existing_node_ids.contains(node.id.as_str()) {
            errors.push(format!(
                "proposed node {} already exists; revision candidates need a separate reviewer",
                node.id
            ));
        }
        if !proposed_node_ids.insert(node.id.as_str()) {
            errors.push(format!("candidate repeats proposed node {}", node.id));
        }
    }

    let mut proposed_edge_ids = HashSet::new();
    for edge in &candidate.proposed_edges {
        if edge.profile != candidate.profile {
            errors.push(format!(
                "proposed edge {} profile does not match candidate profile",
                edge.id
            ));
        }
        if existing_edge_ids.contains(edge.id.as_str()) {
            errors.push(format!(
                "proposed edge {} already exists; revision candidates need a separate reviewer",
                edge.id
            ));
        }
        if !proposed_edge_ids.insert(edge.id.as_str()) {
            errors.push(format!("candidate repeats proposed edge {}", edge.id));
        }
    }

    let mut projected = snapshot.clone();
    projected
        .domains
        .extend(candidate.proposed_domains.iter().cloned());
    projected
        .nodes
        .extend(candidate.proposed_nodes.iter().cloned());
    projected
        .edges
        .extend(candidate.proposed_edges.iter().cloned());
    errors.extend(
        validate_memory_graph_snapshot(&projected)
            .into_iter()
            .map(|error| format!("{}: {}", error.path, error.message)),
    );

    EpiphanyMemoryPatchReview {
        candidate_id: candidate.id.clone(),
        status: if errors.is_empty() {
            EpiphanyMemoryPatchReviewStatus::Accepted
        } else {
            EpiphanyMemoryPatchReviewStatus::Rejected
        },
        accepted_domain_ids: if errors.is_empty() {
            candidate
                .proposed_domains
                .iter()
                .map(|domain| domain.id.clone())
                .collect()
        } else {
            Vec::new()
        },
        accepted_node_ids: if errors.is_empty() {
            candidate
                .proposed_nodes
                .iter()
                .map(|node| node.id.clone())
                .collect()
        } else {
            Vec::new()
        },
        accepted_edge_ids: if errors.is_empty() {
            candidate
                .proposed_edges
                .iter()
                .map(|edge| edge.id.clone())
                .collect()
        } else {
            Vec::new()
        },
        errors,
        warnings,
    }
}

pub fn apply_memory_patch_candidate(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    candidate: &EpiphanyMemoryPatchCandidate,
    accepted_at: Option<String>,
) -> (EpiphanyMemoryGraphSnapshot, EpiphanyMemoryPatchReview) {
    let review = review_memory_patch_candidate(snapshot, candidate);
    if review.status != EpiphanyMemoryPatchReviewStatus::Accepted {
        return (snapshot.clone(), review);
    }
    if candidate_already_applied(snapshot, &accepted_receipt_id(candidate)) {
        return (snapshot.clone(), review);
    }

    let mut accepted = snapshot.clone();
    accepted
        .domains
        .extend(candidate.proposed_domains.iter().cloned());
    accepted
        .nodes
        .extend(candidate.proposed_nodes.iter().cloned());
    accepted
        .edges
        .extend(candidate.proposed_edges.iter().cloned());
    accepted
        .lifecycle_receipts
        .push(EpiphanyMemoryLifecycleReceipt {
            id: accepted_receipt_id(candidate),
            operation: "memoryPatchCandidate.accept".to_string(),
            status: "accepted".to_string(),
            node_ids: review.accepted_node_ids.clone(),
            edge_ids: review.accepted_edge_ids.clone(),
            reason: (!candidate.reasons.is_empty()).then(|| candidate.reasons.join(" ")),
            created_at: accepted_at,
        });

    (accepted, review)
}

fn accepted_receipt_id(candidate: &EpiphanyMemoryPatchCandidate) -> String {
    format!("memreceipt-{}", candidate.id)
}

fn candidate_already_applied(snapshot: &EpiphanyMemoryGraphSnapshot, receipt_id: &str) -> bool {
    snapshot.lifecycle_receipts.iter().any(|receipt| {
        receipt.id == receipt_id
            && receipt.operation == "memoryPatchCandidate.accept"
            && receipt.status == "accepted"
    })
}
