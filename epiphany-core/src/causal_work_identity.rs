use sha2::{Digest, Sha256};

fn digest_id(prefix: &str, causal_parts: &[&str]) -> String {
    let mut digest = Sha256::new();
    for part in causal_parts {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part.as_bytes());
    }
    format!("{prefix}-{:x}", digest.finalize())
}

pub fn proposal_modeling_request_id(
    runtime_id: &str,
    proposal_id: &str,
    payload_sha256: &str,
) -> String {
    digest_id(
        "repo-frontier-proposal-modeling",
        &[runtime_id, proposal_id, payload_sha256],
    )
}

pub fn frontier_research_request_id(
    runtime_id: &str,
    model_hash: &str,
    frontier_item_hash: &str,
) -> String {
    digest_id(
        "repo-frontier-research",
        &[runtime_id, model_hash, frontier_item_hash],
    )
}

pub fn frontier_planning_request_id(
    runtime_id: &str,
    model_hash: &str,
    frontier_item_hash: &str,
) -> String {
    digest_id(
        "repo-frontier-planning",
        &[runtime_id, model_hash, frontier_item_hash],
    )
}

pub fn frontier_plan_mind_request_id(
    runtime_id: &str,
    planning_request_id: &str,
    imagination_result_id: &str,
    candidate_sha256: &str,
) -> String {
    digest_id(
        "repo-frontier-plan-mind",
        &[
            runtime_id,
            planning_request_id,
            imagination_result_id,
            candidate_sha256,
        ],
    )
}

pub fn admitted_model_direction_request_id(
    runtime_id: &str,
    model_hash: &str,
    admission_receipt_id: &str,
    previous_terminal_result_id: Option<&str>,
) -> String {
    digest_id(
        "admitted-model-direction-consideration",
        &[
            runtime_id,
            model_hash,
            admission_receipt_id,
            previous_terminal_result_id.unwrap_or(""),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provenance_does_not_participate_in_causal_identity() {
        let first = frontier_research_request_id("runtime", "model", "frontier");
        let after_thread_transition = frontier_research_request_id("runtime", "model", "frontier");
        assert_eq!(first, after_thread_transition);
        assert_ne!(
            first,
            frontier_research_request_id("runtime", "model", "other")
        );
    }
}
