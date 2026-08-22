use crate::EpiphanyRepoModelView;
use sha2::Digest;

pub fn validate_frontier_plan_decision_chain(
    decision: &crate::RepoFrontierPlanDecisionReceipt,
    route: &crate::RepoFrontierRoute,
    current_model: &EpiphanyRepoModelView,
) -> Result<(), String> {
    validate_current_frontier_route(route, current_model, None)?;
    let plan = route
        .adopted_plan
        .as_ref()
        .ok_or_else(|| "Soul frontier plan decision has no adopted plan".to_string())?;
    let source_is_complete = match decision.decision_source.as_ref() {
        Some(crate::RepoFrontierPlanDecisionSource::MindWorker { result_id, job_id }) => {
            !result_id.trim().is_empty() && !job_id.trim().is_empty()
        }
        None => false,
    };
    let decision_basis = crate::EpiphanyRepoModelBasis {
        projection_digest: decision.model_projection_digest.clone(),
        source_documents: decision.model_source_documents.clone(),
    };
    decision_basis.validate().map_err(|error| {
        format!("Soul frontier plan decision has an invalid keyed basis: {error}")
    })?;
    if decision.schema_version != crate::REPO_FRONTIER_PLAN_DECISION_RECEIPT_SCHEMA_VERSION
        || decision.contract != crate::REPO_FRONTIER_PLAN_DECISION_CONTRACT
        || decision.decision != crate::RepoFrontierPlanDecision::Adopt
        || decision.legacy_mind_worker_result_id.is_some()
        || decision.legacy_mind_worker_job_id.is_some()
        || decision.planning_request_id != plan.planning_request_id
        || decision.candidate_id != plan.candidate_id
        || decision.candidate_sha256 != plan.candidate_sha256
        || decision_basis != current_model.reasoning_basis()
        || decision.frontier_item_id != route.frontier_item_id
        || decision.frontier_item_hash != route.frontier_item_hash
        || !source_is_complete
    {
        return Err(
            "Soul frontier plan decision does not exactly bind its keyed basis, adopted plan, route, and decision source"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_current_frontier_route<'a>(
    route: &crate::RepoFrontierRoute,
    current_model: &'a EpiphanyRepoModelView,
    verification_request: Option<&crate::RepoFrontierVerificationRequest>,
) -> Result<&'a crate::RepoFrontierItem, String> {
    let route_basis = crate::EpiphanyRepoModelBasis {
        projection_digest: route.model_projection_digest.clone(),
        source_documents: route.model_source_documents.clone(),
    };
    route_basis
        .validate()
        .map_err(|error| format!("frontier route has an invalid keyed basis: {error}"))?;
    if route_basis != current_model.reasoning_basis() {
        return Err("frontier route does not bind the current keyed RepoModel view".to_string());
    }
    let current_item = current_model
        .frontier
        .iter()
        .find(|item| item.id == route.frontier_item_id)
        .ok_or_else(|| {
            "routed frontier item disappeared from the current keyed view".to_string()
        })?;
    let current_item_hash = format!(
        "{:x}",
        sha2::Sha256::digest(
            rmp_serde::to_vec_named(current_item)
                .map_err(|error| format!("failed to hash current frontier item: {error}"))?
        )
    );
    let expected_authorized_paths = current_item
        .adopted_plan
        .as_ref()
        .map(|plan| plan.safe_paths.as_slice())
        .unwrap_or(current_item.repository_scope.as_slice());
    if route.schema_version != crate::REPO_FRONTIER_ROUTE_SCHEMA_VERSION
        || route.contract != crate::REPO_FRONTIER_ROUTE_CONTRACT
        || route.next_organ != crate::RepoFrontierNextOrgan::Hands
        || route.frontier_item_hash != current_item_hash
        || route.migration_body != current_item.migration_body
        || route.question != current_item.question
        || route.gap != current_item.gap
        || route.target_claim_ids != current_item.target_claim_ids
        || route.authorized_paths != expected_authorized_paths
        || route.adopted_plan != current_item.adopted_plan
        || current_item.status != crate::RepoFrontierStatus::Active
        || current_item.recommended_next_organ != "Hands"
    {
        return Err(
            "frontier route does not exactly bind its current keyed frontier item".to_string(),
        );
    }
    if let Some(request) = verification_request {
        if request.schema_version != crate::REPO_FRONTIER_VERIFICATION_REQUEST_SCHEMA_VERSION
            || request.contract != crate::REPO_FRONTIER_VERIFICATION_REQUEST_CONTRACT
            || request.route_id != route.route_id
            || request.model_projection_digest != route.model_projection_digest
            || request.model_source_documents != route.model_source_documents
            || request.frontier_item_id != route.frontier_item_id
            || request.frontier_item_hash != route.frontier_item_hash
        {
            return Err(
                "Soul verification request does not exactly bind the current keyed route"
                    .to_string(),
            );
        }
    }
    Ok(current_item)
}
