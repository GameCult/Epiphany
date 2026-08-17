use crate::EpiphanyMemoryContextPacket;
use crate::EpiphanyMemoryContextQuery;
use crate::EpiphanyMemoryProfile;
use crate::EpiphanyPromptContextInput;
use crate::EpiphanyRepoModelView;
use crate::assemble_repo_model_view;
use crate::load_epiphany_cultmesh_cluster_topology;
use crate::load_epiphany_cultmesh_status;
use crate::plan_memory_graph_context_cut;
use crate::query_epiphany_local_verse_context;
use crate::render_epiphany_prompt_context;
use epiphany_state_model::EpiphanyThreadState;
use sha2::Digest;
use std::path::Path;
use std::path::PathBuf;

pub const EPIPHANY_LOCAL_VERSE_RUNTIME_ID: &str = "epiphany-local";

pub fn local_verse_store_path(runtime_store_path: &Path) -> PathBuf {
    sibling_state_store_path(runtime_store_path, "local-verse.ccmp")
}

pub fn memory_graph_store_path(runtime_store_path: &Path) -> PathBuf {
    sibling_state_store_path(runtime_store_path, "memory-graph.msgpack")
}

fn sibling_state_store_path(runtime_store_path: &Path, filename: &str) -> PathBuf {
    runtime_store_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(filename)
}

pub fn role_launch_context_focus(state: &EpiphanyThreadState, role_label: &str) -> String {
    let objective = state
        .objective
        .as_deref()
        .map(str::trim)
        .filter(|objective| !objective.is_empty())
        .unwrap_or("Epiphany worker launch");
    format!("Launch `{role_label}` worker for: {objective}")
}

pub fn reorient_launch_context_focus(state: &EpiphanyThreadState, next_action: &str) -> String {
    let objective = state
        .objective
        .as_deref()
        .map(str::trim)
        .filter(|objective| !objective.is_empty())
        .unwrap_or("Epiphany reorientation");
    format!("Launch reorientation worker for: {objective}. Next action: {next_action}")
}

pub fn render_launch_dynamic_prompt_context(
    runtime_store_path: &Path,
    local_verse_store: &Path,
    state: &EpiphanyThreadState,
    focus: String,
) -> Result<String, String> {
    render_launch_dynamic_prompt_context_with_snapshot(
        runtime_store_path,
        local_verse_store,
        state,
        focus,
    )
    .map(|(context, _)| context)
}

pub fn render_modeling_launch_dynamic_prompt_context(
    runtime_store_path: &Path,
    local_verse_store: &Path,
    state: &EpiphanyThreadState,
    focus: String,
) -> Result<String, String> {
    let (context, snapshot) = render_launch_dynamic_prompt_context_with_snapshot(
        runtime_store_path,
        local_verse_store,
        state,
        focus,
    )?;
    Ok(append_modeling_repo_model_shape_snapshot(
        context, &snapshot,
    ))
}

fn render_launch_dynamic_prompt_context_with_snapshot(
    runtime_store_path: &Path,
    local_verse_store: &Path,
    state: &EpiphanyThreadState,
    focus: String,
) -> Result<(String, EpiphanyRepoModelView), String> {
    load_epiphany_cultmesh_status(local_verse_store, EPIPHANY_LOCAL_VERSE_RUNTIME_ID)
        .map_err(|error| {
            format!(
                "failed to inspect local Verse context store {}: {error}",
                local_verse_store.display()
            )
        })?
        .ok_or_else(|| {
            format!(
                "local Verse is not bootstrapped at {}; initialize it before building worker launch context",
                local_verse_store.display()
            )
        })?;
    let topology =
        load_epiphany_cultmesh_cluster_topology(local_verse_store, EPIPHANY_LOCAL_VERSE_RUNTIME_ID)
            .map_err(|error| {
                format!(
                    "failed to inspect local Verse topology store {}: {error}",
                    local_verse_store.display()
                )
            })?;
    if topology.is_empty() {
        return Err(format!(
            "local Verse has no persisted cluster topology at {}; initialize it before building worker launch context",
            local_verse_store.display()
        ));
    }
    let local_verse =
        query_epiphany_local_verse_context(local_verse_store, EPIPHANY_LOCAL_VERSE_RUNTIME_ID)
            .map_err(|error| {
                format!(
                    "failed to query local Verse context store {}: {error}",
                    local_verse_store.display()
                )
            })?;
    let (memory_context, snapshot) =
        launch_memory_context(runtime_store_path, state, focus.as_str()).map_err(|error| {
            format!(
                "failed to build launch memory context beside {}: {error}",
                runtime_store_path.display()
            )
        })?;
    Ok((
        render_epiphany_prompt_context(&EpiphanyPromptContextInput {
            focus,
            local_verse,
            memory_context,
        }),
        snapshot,
    ))
}

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
        Some(crate::RepoFrontierPlanDecisionSource::AuthenticatedOperatorReview {
            command_id,
            admission_id,
            packet_sha256,
            source_actor_id,
        }) => [command_id, admission_id, packet_sha256, source_actor_id]
            .into_iter()
            .all(|value| !value.trim().is_empty()),
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
    let expected_source_scope = current_item
        .adopted_plan
        .as_ref()
        .map(|plan| plan.safe_paths.as_slice())
        .unwrap_or(current_item.source_scope.as_slice());
    if route.schema_version != crate::REPO_FRONTIER_ROUTE_SCHEMA_VERSION
        || route.contract != crate::REPO_FRONTIER_ROUTE_CONTRACT
        || route.next_organ != crate::RepoFrontierNextOrgan::Hands
        || route.frontier_item_hash != current_item_hash
        || route.migration_body != current_item.migration_body
        || route.question != current_item.question
        || route.gap != current_item.gap
        || route.target_claim_ids != current_item.target_claim_ids
        || route.source_scope != expected_source_scope
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

pub fn append_modeling_repo_model_shape_context(
    context: String,
    runtime_store_path: &Path,
) -> Result<String, String> {
    let view = assemble_repo_model_view(runtime_store_path)
        .map_err(|error| format!("failed to load canonical RepoModel shape: {error}"))?;
    Ok(append_modeling_repo_model_shape_snapshot(context, &view))
}

fn append_modeling_repo_model_shape_snapshot(
    mut context: String,
    view: &EpiphanyRepoModelView,
) -> String {
    context.push_str("\n\n<canonical_repo_model_shape>\n");
    context.push_str(&format!("projectionDigest: {}\n", view.projection_digest));
    context.push_str("existingDomains:\n");
    for domain in &view.domains {
        let profile = debug_variant_snake_case(domain.profile);
        let lifecycle = debug_variant_snake_case(domain.lifecycle);
        context.push_str(&format!(
            "- id={} profile={} lifecycle={} title={}\n",
            domain.id, profile, lifecycle, domain.title
        ));
    }
    context.push_str("existingClaims:\n");
    for node in &view.nodes {
        context.push_str(&format!(
            "- id={} domain={} title={}\n",
            node.id, node.domain_id, node.title
        ));
    }
    context.push_str("existingFrontier:\n");
    for item in &view.frontier {
        let status = debug_variant_snake_case(item.status);
        context.push_str(&format!(
            "- id={} status={} recommendedNextOrgan={} targetClaims={} sourceScope={}\n",
            item.id,
            status,
            item.recommended_next_organ,
            item.target_claim_ids.join(" | "),
            item.source_scope.join(" | "),
        ));
    }
    context.push_str("existingSurfaceOffers:\n");
    for offer in &view.surface_offers {
        context.push_str(&format!(
            "- surfaceId={} label={} contract={} lifecycle={} sourceRefs={}\n",
            offer.surface_id,
            offer.label,
            render_atlas_contract_descriptor(&offer.contract),
            render_atlas_offer_lifecycle(&offer.lifecycle),
            offer
                .body_evidence
                .iter()
                .map(|source| format!("{}@{}", source.path, source.raw_sha256))
                .collect::<Vec<_>>()
                .join(" | "),
        ));
    }
    context.push_str("existingDependencyClaims:\n");
    for claim in &view.dependency_claims {
        context.push_str(&format!(
            "- claimId={} label={} target={} kind={} failureSemantics={} impactScope={} lifecycle={} sourceRefs={}\n",
            claim.claim_id,
            claim.label,
            render_atlas_dependency_target(&claim.target),
            debug_variant_snake_case(claim.entanglement_kind),
            debug_variant_snake_case(claim.failure_semantics),
            render_atlas_impact_scope(&claim.impact_scope),
            debug_variant_snake_case(claim.lifecycle),
            claim
                .body_evidence
                .iter()
                .map(|source| format!("{}@{}", source.path, source.raw_sha256))
                .collect::<Vec<_>>()
                .join(" | "),
        ));
    }
    context.push_str("currentDependencyVerifications:\n");
    for verification in &view.dependency_verifications {
        context.push_str(&format!(
            "- claimId={} verdict={} claimPublication={} offerPublication={}\n",
            verification.claim_id,
            debug_variant_snake_case(verification.verdict),
            verification.claim_publication_id,
            verification.offer_publication_id,
        ));
    }
    context.push_str("currentDependencyImpacts:\n");
    for impact in &view.dependency_impacts {
        context.push_str(&format!(
            "- impactId={} claimId={} criticality={} projection={}\n",
            impact.impact_id,
            impact.claim_id,
            debug_variant_snake_case(impact.criticality),
            impact.projection_sha256,
        ));
    }
    context.push_str("RepoModel mutations address semantic document identities directly. New nodes must reference one exact existing domain id or a domain created by the same mutation. Every unresolved frontier item must target at least one exact existing claim id or a claim created by the same mutation. Every frontier source_scope must be non-empty and contain safe relative paths in strict lexicographic ascending order without duplicates. Recommended next organ must use its exact canonical spelling. RepoArchitecture and RepoDataflow nodes/edges may use only observed, proposed, accepted, stale, or retired lifecycle. Atlas create operations do not accept caller-authored offer or claim ids: runtime derives opaque UUIDs from the admitted proposal. Lifecycle operations may name only ids listed above. Exact dependency targets must name a provider repository and surface; unresolved targets remain unresolved. Local-surface impact scopes may name only locally owned surface ids listed above. The projection digest is display and audit identity, never a global stale-write fence.\n");
    context.push_str("</canonical_repo_model_shape>");
    context
}

fn render_atlas_contract_descriptor(contract: &crate::AtlasContractDescriptor) -> String {
    match contract {
        crate::AtlasContractDescriptor::Semver {
            contract_id,
            version,
        } => format!("semver:{contract_id}@{version}"),
        crate::AtlasContractDescriptor::ExactSchema {
            contract_id,
            schema_id,
        } => format!("exact_schema:{contract_id}@{schema_id}"),
        crate::AtlasContractDescriptor::ExactDigest {
            contract_id,
            sha256,
        } => format!("exact_digest:{contract_id}@{sha256}"),
    }
}

fn render_atlas_contract_requirement(requirement: &crate::AtlasContractRequirement) -> String {
    match requirement {
        crate::AtlasContractRequirement::Semver {
            contract_id,
            requirement,
        } => format!("semver:{contract_id}@{requirement}"),
        crate::AtlasContractRequirement::ExactSchema {
            contract_id,
            schema_id,
        } => format!("exact_schema:{contract_id}@{schema_id}"),
        crate::AtlasContractRequirement::ExactDigest {
            contract_id,
            sha256,
        } => format!("exact_digest:{contract_id}@{sha256}"),
    }
}

fn render_atlas_offer_lifecycle(lifecycle: &crate::AtlasOfferLifecycle) -> String {
    match lifecycle {
        crate::AtlasOfferLifecycle::Active => "active".to_string(),
        crate::AtlasOfferLifecycle::Deprecated {
            replacement_surface_id: Some(replacement_surface_id),
        } => format!("deprecated:replacement={replacement_surface_id}"),
        crate::AtlasOfferLifecycle::Deprecated {
            replacement_surface_id: None,
        } => "deprecated".to_string(),
        crate::AtlasOfferLifecycle::Withdrawn => "withdrawn".to_string(),
    }
}

fn render_atlas_dependency_target(target: &crate::AtlasDependencyTarget) -> String {
    match target {
        crate::AtlasDependencyTarget::Exact {
            provider,
            surface_id,
            requirement,
        } => format!(
            "exact:{}#{} requires {}",
            provider.repository_uri,
            surface_id,
            render_atlas_contract_requirement(requirement)
        ),
        crate::AtlasDependencyTarget::Unresolved { requirement } => format!(
            "unresolved:{}",
            render_atlas_contract_requirement(requirement)
        ),
    }
}

fn render_atlas_impact_scope(scope: &crate::AtlasImpactScope) -> String {
    match scope {
        crate::AtlasImpactScope::WholeRepository => "whole_repository".to_string(),
        crate::AtlasImpactScope::LocalSurfaces { surface_ids } => format!(
            "local_surfaces:{}",
            surface_ids
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("|")
        ),
    }
}

fn debug_variant_snake_case(value: impl std::fmt::Debug) -> String {
    let name = format!("{value:?}");
    let mut output = String::with_capacity(name.len() + 4);
    for (index, character) in name.chars().enumerate() {
        if character.is_ascii_uppercase() && index > 0 {
            output.push('_');
        }
        output.push(character.to_ascii_lowercase());
    }
    output
}

fn launch_memory_context(
    runtime_store_path: &Path,
    _state: &EpiphanyThreadState,
    focus: &str,
) -> Result<(EpiphanyMemoryContextPacket, EpiphanyRepoModelView), String> {
    let view = assemble_repo_model_view(runtime_store_path)
        .map_err(|error| format!("failed to load keyed RepoModel view: {error}"))?;
    let projection = view.memory_context_projection();

    let mut packet = plan_memory_graph_context_cut(
        &projection,
        &EpiphanyMemoryContextQuery {
            id: format!("launch-context-query-{}", view.projection_digest),
            profile: Some(EpiphanyMemoryProfile::RepoArchitecture),
            text: Some(focus.to_string()),
            budget: Some(5),
            ..Default::default()
        },
    );
    packet.warnings.push(format!(
        "RepoModel context is assembled from exact keyed Mind documents at projection digest {}.",
        view.projection_digest
    ));
    if packet.nodes.is_empty() && packet.summaries.is_empty() {
        packet.warnings.push(
            "Memory graph context is empty for this launch focus; the accepted repo graph may be thin or stale.".to_string(),
        );
    }
    Ok((packet, view))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EpiphanyMemoryDomain;
    use crate::EpiphanyMemoryLifecycle;
    use crate::EpiphanyMemoryNode;
    use crate::EpiphanyMemoryNodeKind;
    use crate::EpiphanyRoleResultRoleId;
    use crate::RepoFrontierItem;
    use crate::RepoFrontierStatus;
    use crate::RuntimeSpineHeartbeatJobOptions;
    use crate::RuntimeSpineInitOptions;
    use crate::build_epiphany_role_launch_request_with_dynamic_context;
    use crate::initialize_runtime_spine;
    use crate::open_runtime_spine_heartbeat_job;
    use crate::runtime_worker_launch_request;
    use crate::seed_epiphany_local_verse_context;
    use std::fs;
    use uuid::Uuid;

    #[test]
    fn native_launch_context_has_no_bridge_or_host_dependency() {
        let source = include_str!("coordinator_launch_context.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        for forbidden in [
            "epiphany_codex_bridge",
            "epiphany_core::",
            "AppServerClient",
            "serde_json",
        ] {
            assert!(
                !production.contains(forbidden),
                "native launch context contains host marker {forbidden:?}"
            );
        }
    }

    #[test]
    fn canonical_model_frontier_survives_newer_thread_state_and_guides_launch() -> anyhow::Result<()>
    {
        let temp = std::env::temp_dir().join(format!(
            "epiphany-canonical-model-launch-{}",
            Uuid::new_v4()
        ));
        fs::create_dir(&temp)?;
        let runtime_store = temp.join("runtime-spine.msgpack");
        initialize_runtime_spine(
            &runtime_store,
            RuntimeSpineInitOptions {
                runtime_id: "canonical-model-launch-test".to_string(),
                display_name: "Canonical Model Launch Test".to_string(),
                created_at: "2026-06-12T00:00:00Z".to_string(),
            },
        )?;
        crate::runtime_spine::tests::bind_test_runtime_swarm(
            &runtime_store,
            "canonical-model-launch-swarm",
        )?;
        let documents = crate::EpiphanyRepoModelSeedDocuments {
            domains: vec![EpiphanyMemoryDomain {
                id: "repo".to_string(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                title: "Canonical repository model".to_string(),
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                ..Default::default()
            }],
            nodes: vec![EpiphanyMemoryNode {
                id: "claim-modeling-authority".to_string(),
                domain_id: "repo".to_string(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                kind: EpiphanyMemoryNodeKind::RuntimeContract,
                title: "Modeling authority".to_string(),
                claim: "Canonical Modeling state survives transcript revision churn.".to_string(),
                question: "Which downstream organ consumes it?".to_string(),
                action_implication: "Route the exact claim into launch context.".to_string(),
                source_hashes: vec!["anchor:missing".to_string()],
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                ..Default::default()
            }],
            frontier: vec![RepoFrontierItem {
                id: "frontier-modeling-handoff".to_string(),
                migration_body: "Carry the canonical repository frontier into organ prompts."
                    .to_string(),
                question: "Can Hands see the exact target claim?".to_string(),
                gap: "Launch assembly previously saw only semantically similar prose.".to_string(),
                target_claim_ids: vec!["claim-modeling-authority".to_string()],
                source_scope: vec!["epiphany-core/src".to_string()],
                recommended_next_organ: "Hands".to_string(),
                status: RepoFrontierStatus::Active,
                ..Default::default()
            }],
            edges: Vec::new(),
            summaries: Vec::new(),
            lifecycle_receipts: Vec::new(),
        };
        let seed = crate::EpiphanyRepoModelSeed::new(
            "canonical-model-seed",
            "canonical-model",
            "canonical-model-launch-swarm",
            "canonical-model-workspace",
            "sha256:canonical-model-body",
            documents.clone(),
        )?;
        crate::initialize_keyed_repo_model(&runtime_store, &seed, "2026-06-12T00:00:01Z")?;
        let newer_thread_state = EpiphanyThreadState {
            revision: 999,
            objective: Some("Discuss irrelevant weather bananas.".to_string()),
            ..Default::default()
        };

        let (packet, _) = launch_memory_context(
            &runtime_store,
            &newer_thread_state,
            "irrelevant weather bananas",
        )
        .map_err(anyhow::Error::msg)?;

        assert_eq!(packet.frontier[0].id, "frontier-modeling-handoff");
        assert!(
            packet
                .nodes
                .iter()
                .any(|node| node.id == "claim-modeling-authority")
        );
        let preserved = crate::assemble_repo_model_view(&runtime_store)?;
        assert_eq!(preserved.frontier, documents.frontier);
        let shape = append_modeling_repo_model_shape_context("base".to_string(), &runtime_store)
            .map_err(anyhow::Error::msg)?;
        assert_eq!(
            shape,
            append_modeling_repo_model_shape_snapshot("base".to_string(), &preserved)
        );
        assert!(shape.contains("<canonical_repo_model_shape>"));
        assert!(shape.contains("id=repo"));
        assert!(shape.contains("existingClaims:"));
        assert!(shape.contains("id=claim-modeling-authority domain=repo"));
        assert!(shape.contains("existingFrontier:"));
        assert!(shape.contains("id=frontier-modeling-handoff status=active"));
        assert!(shape.contains("projectionDigest: sha256:"));
        assert!(shape.contains("semantic document identities directly"));
        assert!(shape.contains("strict lexicographic ascending order without duplicates"));
        assert!(shape.contains("projection digest is display and audit identity"));
        fs::remove_dir_all(&temp)?;
        Ok(())
    }

    #[test]
    fn launch_context_persists_on_runtime_worker_request() -> anyhow::Result<()> {
        let temp =
            std::env::temp_dir().join(format!("epiphany-bridge-launch-context-{}", Uuid::new_v4()));
        fs::create_dir(&temp)?;
        let runtime_store = temp.join("runtime-spine.msgpack");
        crate::initialize_runtime_spine(
            &runtime_store,
            crate::RuntimeSpineInitOptions {
                runtime_id: "launch-context-runtime".to_string(),
                display_name: "Launch context test".to_string(),
                created_at: "2026-07-12T00:00:00Z".to_string(),
            },
        )?;
        let agent_store = temp.join("agents.msgpack");
        crate::ensure_agent_memory_swarm_identity(&agent_store, "launch-context-swarm")?;
        crate::bind_runtime_to_agent_memory_swarm(
            &runtime_store,
            &agent_store,
            "2026-07-12T00:00:01Z",
        )?;
        crate::runtime_spine::tests::bind_test_repository_body(
            &runtime_store,
            "launch-context-workspace",
        )?;
        let body_basis = crate::observe_runtime_repository_body_basis(&runtime_store)?;
        let seed = crate::EpiphanyRepoModelSeed::new(
            "launch-context-seed",
            "launch-context-graph",
            body_basis.swarm_id.clone(),
            body_basis.workspace_id.clone(),
            body_basis.body_binding_sha256.clone(),
            crate::EpiphanyRepoModelSeedDocuments {
                domains: Vec::new(),
                nodes: Vec::new(),
                edges: Vec::new(),
                summaries: Vec::new(),
                frontier: Vec::new(),
                lifecycle_receipts: Vec::new(),
            },
        )?;
        crate::initialize_keyed_repo_model(&runtime_store, &seed, "2026-07-12T00:00:02Z")?;
        let state = EpiphanyThreadState {
            revision: 7,
            objective: Some("Test launch context.".to_string()),
            ..Default::default()
        };
        seed_epiphany_local_verse_context(
            local_verse_store_path(&runtime_store),
            EPIPHANY_LOCAL_VERSE_RUNTIME_ID,
            "2026-07-12T00:00:00Z",
            "repo:C:/fixture/Epiphany",
        )?;

        let rendered = render_launch_dynamic_prompt_context(
            &runtime_store,
            &local_verse_store_path(&runtime_store),
            &state,
            role_launch_context_focus(&state, "modeling"),
        )
        .map_err(anyhow::Error::msg)?;

        assert!(rendered.contains("<epiphany_dynamic_context>"));
        assert!(rendered.contains("Test launch context."));
        assert!(rendered.contains("Odin"));
        assert!(rendered.contains("Yggdrasil"));
        assert!(rendered.contains("Memory graph"));
        assert!(local_verse_store_path(&runtime_store).exists());
        assert!(runtime_store.exists());

        let mut launch_request = build_epiphany_role_launch_request_with_dynamic_context(
            "thread-1",
            EpiphanyRoleResultRoleId::Modeling,
            Some(state.revision),
            Some(60),
            &state,
            Some(rendered.clone()),
        )
        .map_err(anyhow::Error::msg)?;
        let crate::EpiphanyWorkerLaunchDocument::Role(role_document) =
            &mut launch_request.launch_document
        else {
            unreachable!("role launch builder returned reorient document")
        };
        role_document.repository_body_observation_basis = Some(body_basis);
        open_runtime_spine_heartbeat_job(
            &runtime_store,
            RuntimeSpineHeartbeatJobOptions {
                runtime_id: "launch-context-runtime".to_string(),
                display_name: "Launch context test".to_string(),
                session_id: "epiphany-main".to_string(),
                objective: "Test persisted launch context.".to_string(),
                coordinator_note: "Bridge launch-context smoke.".to_string(),
                job_id: "job-launch-context".to_string(),
                role: launch_request.owner_role.clone(),
                binding_id: launch_request.binding_id.clone(),
                authority_scope: launch_request.authority_scope.clone(),
                instruction: launch_request.instruction.clone(),
                launch_document: launch_request.launch_document.clone(),
                output_contract_id: launch_request.output_contract_id.clone(),
                organ_launch_contract: launch_request.organ_launch_contract.clone(),
                proposal_modeling_request_id: None,
                frontier_planning_request_id: None,
                frontier_plan_mind_request_id: None,
                imagination_consideration_request_id: None,
                admitted_model_direction_consideration_request_id: None,
                repo_frontier_modeling_request_id: None,
                repo_frontier_research_request_id: None,
                repo_frontier_verification_request_id: None,
                created_at: "2026-06-02T00:00:00Z".to_string(),
            },
        )?;
        let stored = runtime_worker_launch_request(&runtime_store, "job-launch-context")?
            .expect("runtime worker launch request should be persisted");
        let stored_document = stored.launch_document()?;
        let stored_context = stored_document
            .dynamic_prompt_context()
            .expect("stored launch document should carry dynamic context");
        assert!(stored_context.contains("Odin"));
        assert!(stored_context.contains("Memory graph"));
        assert!(stored_context.contains("Test launch context."));

        fs::remove_dir_all(&temp)?;
        Ok(())
    }

    #[test]
    fn launch_context_refuses_to_bootstrap_shared_state() -> anyhow::Result<()> {
        let temp = std::env::temp_dir().join(format!(
            "epiphany-launch-context-unbootstrapped-{}",
            Uuid::new_v4()
        ));
        fs::create_dir(&temp)?;
        let runtime_store = temp.join("runtime-spine.msgpack");
        let local_verse_store = local_verse_store_path(&runtime_store);
        let state = EpiphanyThreadState {
            revision: 1,
            objective: Some("Prove launch assembly cannot initialize shared state.".to_string()),
            ..Default::default()
        };

        let error = render_launch_dynamic_prompt_context(
            &runtime_store,
            &local_verse_store,
            &state,
            role_launch_context_focus(&state, "modeling"),
        )
        .expect_err("unbootstrapped launch context must fail closed");

        assert!(error.contains("local Verse is not bootstrapped"));
        assert!(!local_verse_store.exists());
        Ok(())
    }
}
