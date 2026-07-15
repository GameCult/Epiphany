use super::*;
use anyhow::Result;
use epiphany_state_model::EpiphanyMemoryEdgeKind;
use epiphany_state_model::EpiphanyMemoryNodeKind;

fn anchor(id: &str, hash: &str) -> EpiphanyMemoryAnchor {
    EpiphanyMemoryAnchor {
        id: id.to_string(),
        kind: "source".to_string(),
        target: format!("src/{id}.rs"),
        source_hash: Some(hash.to_string()),
        ..Default::default()
    }
}

fn fixture_snapshot() -> EpiphanyMemoryGraphSnapshot {
    let domain_id = memory_graph_domain_id(
        EpiphanyMemoryProfile::RepoArchitecture,
        "crate",
        "epiphany-core",
    );
    let node_a = memory_graph_node_id(&domain_id, "module", "src/memory_graph.rs", Some("a"));
    let node_b = memory_graph_node_id(&domain_id, "module", "src/memory_graph.rs", Some("b"));
    let edge = memory_graph_edge_id(&node_a, &node_b, "owns", ["src/memory_graph.rs"]);
    EpiphanyMemoryGraphSnapshot {
        schema_version: Some(MEMORY_GRAPH_SCHEMA_VERSION.to_string()),
        graph_id: "graph".to_string(),
        domains: vec![EpiphanyMemoryDomain {
            id: domain_id.clone(),
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            title: "Core".to_string(),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        }],
        nodes: vec![
            EpiphanyMemoryNode {
                id: node_a.clone(),
                domain_id: domain_id.clone(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                kind: EpiphanyMemoryNodeKind::Module,
                title: "Memory graph".to_string(),
                claim: "Memory graph owns shared graph law.".to_string(),
                question: "How much profile policy belongs here?".to_string(),
                tension: String::new(),
                action_implication: "Route repo and agent memory through this module.".to_string(),
                anchors: vec![anchor("a", "hash-a")],
                source_hashes: vec!["hash-a".to_string()],
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                confidence: 90,
                ..Default::default()
            },
            EpiphanyMemoryNode {
                id: node_b.clone(),
                domain_id: domain_id.clone(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                kind: EpiphanyMemoryNodeKind::TestSeam,
                title: "Validation tests".to_string(),
                claim: "Validation tests pin the shared graph skeleton.".to_string(),
                question: String::new(),
                tension: "Without them profile code can bypass graph law.".to_string(),
                action_implication: "Run memory_graph tests before profile work.".to_string(),
                anchors: vec![anchor("b", "hash-b")],
                source_hashes: vec!["hash-b".to_string()],
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                confidence: 85,
                ..Default::default()
            },
        ],
        edges: vec![EpiphanyMemoryEdge {
            id: edge.clone(),
            source_id: node_a.clone(),
            target_id: node_b.clone(),
            kind: EpiphanyMemoryEdgeKind::Verifies,
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            claim: "Validation tests verify graph law.".to_string(),
            anchors: vec![anchor("edge", "hash-a")],
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            confidence: 80,
        }],
        summaries: vec![EpiphanyMemorySummary {
            id: "summary-1".to_string(),
            domain_id,
            covers_node_ids: vec![node_a, node_b],
            covers_edge_ids: vec![edge],
            target: "memory_graph".to_string(),
            claim: "Memory graph owns shared repo and agent memory graph law.".to_string(),
            question: "Which profile policy is allowed at the substrate?".to_string(),
            tension: String::new(),
            action_implication: "Use shared context packets before profile producers.".to_string(),
            anchor_count: 3,
            source_hashes: vec!["hash-a".to_string(), "hash-b".to_string()],
            freshness: EpiphanyMemoryFreshnessStatus::Ready,
            confidence: 92,
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn role_snapshot() -> EpiphanyMemoryGraphSnapshot {
    let domain_id = memory_graph_domain_id(EpiphanyMemoryProfile::RoleSelf, "role", "body");
    let node_id = memory_graph_node_id(&domain_id, "memory", "keeps boundaries typed", None);
    EpiphanyMemoryGraphSnapshot {
        graph_id: "role-graph".to_string(),
        domains: vec![EpiphanyMemoryDomain {
            id: domain_id.clone(),
            profile: EpiphanyMemoryProfile::RoleSelf,
            title: "Modeling".to_string(),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        }],
        nodes: vec![EpiphanyMemoryNode {
            id: node_id.clone(),
            domain_id: domain_id.clone(),
            profile: EpiphanyMemoryProfile::RoleSelf,
            kind: EpiphanyMemoryNodeKind::RoleMemory,
            title: "Typed boundaries".to_string(),
            claim: "Modeling remembers that typed boundaries keep the machine legible.".to_string(),
            question: "Which boundary is currently lying?".to_string(),
            tension: String::new(),
            action_implication: "Prefer one owned graph store over profile-local stores."
                .to_string(),
            source_hashes: vec!["anchor:missing".to_string()],
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            confidence: 88,
            ..Default::default()
        }],
        summaries: vec![EpiphanyMemorySummary {
            id: "summary-role-body".to_string(),
            domain_id,
            covers_node_ids: vec![node_id],
            target: "body role memory".to_string(),
            claim: "Modeling role memory enters the shared graph as typed state.".to_string(),
            question: "How should role memory shape context cuts?".to_string(),
            tension: String::new(),
            action_implication: "Compose role memory with repo graph truth before retrieval."
                .to_string(),
            anchor_count: 0,
            freshness: EpiphanyMemoryFreshnessStatus::Ready,
            confidence: 80,
            ..Default::default()
        }],
        freshness: Some(EpiphanyMemoryFreshness {
            status: EpiphanyMemoryFreshnessStatus::Ready,
            note: Some("Role memory is ready.".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn frontier_item(id: &str, claim_id: &str) -> RepoFrontierItem {
    RepoFrontierItem {
        id: id.to_string(),
        migration_body: format!("Migrate {id}"),
        question: "Which owner remains split?".to_string(),
        gap: "The durable cut is not yet represented.".to_string(),
        target_claim_ids: vec![claim_id.to_string()],
        source_scope: vec!["epiphany-core/src/memory_graph".to_string()],
        recommended_next_organ: "Eyes".to_string(),
        status: RepoFrontierStatus::Active,
        created_at: Some("2026-07-13T00:00:00Z".to_string()),
        ..Default::default()
    }
}

fn patch_for(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    operations: Vec<RepoModelPatchOperation>,
) -> RepoModelPatch {
    RepoModelPatch {
        purpose: RepoModelPatchPurpose::Evolution,
        patch_id: "patch-1".to_string(),
        base_revision: snapshot.model_revision,
        base_hash: memory_graph_model_hash(snapshot).expect("model hash"),
        applied_at: "2026-07-13T01:00:00Z".to_string(),
        operations,
    }
}

fn make_canonical(snapshot: &mut EpiphanyMemoryGraphSnapshot, revision: u64) {
    snapshot.model_revision = revision;
    snapshot.model_hash = memory_graph_model_hash(snapshot).expect("canonical hash");
}

#[test]
fn repo_model_patch_advances_exact_revision_and_hash() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("memory.cc");
    let snapshot = fixture_snapshot();
    write_memory_graph_snapshot(&path, &snapshot)?;
    let claim_id = snapshot.nodes[0].id.clone();
    let patch = patch_for(
        &snapshot,
        vec![RepoModelPatchOperation::UpsertFrontier {
            item: frontier_item("frontier-1", &claim_id),
        }],
    );

    let applied = apply_repo_model_patch(&path, &patch)?;

    assert_eq!(applied.model_revision, snapshot.model_revision + 1);
    assert_eq!(applied.model_hash, memory_graph_model_hash(&applied)?);
    assert_eq!(load_memory_graph_snapshot(&path)?, Some(applied));
    Ok(())
}

#[test]
fn stale_repo_model_patch_is_rejected_without_mutation() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("memory.cc");
    let snapshot = fixture_snapshot();
    write_memory_graph_snapshot(&path, &snapshot)?;
    let mut stale = patch_for(&snapshot, Vec::new());
    stale.base_hash = "0".repeat(64);

    assert!(apply_repo_model_patch(&path, &stale).is_err());
    assert_eq!(load_memory_graph_snapshot(&path)?, Some(snapshot));
    Ok(())
}

#[test]
fn repo_model_patch_retires_and_supersedes_frontier() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("memory.cc");
    let mut snapshot = fixture_snapshot();
    let claim_id = snapshot.nodes[0].id.clone();
    snapshot.frontier = vec![
        frontier_item("old", &claim_id),
        frontier_item("replacement", &claim_id),
    ];
    make_canonical(&mut snapshot, 1);
    write_memory_graph_snapshot(&path, &snapshot)?;
    let patch = patch_for(
        &snapshot,
        vec![RepoModelPatchOperation::RetireFrontier {
            item_id: "old".to_string(),
            retired_at: None,
            superseded_by: Some("replacement".to_string()),
        }],
    );

    let applied = apply_repo_model_patch(&path, &patch)?;
    let old = applied
        .frontier
        .iter()
        .find(|item| item.id == "old")
        .unwrap();
    assert_eq!(old.status, RepoFrontierStatus::Superseded);
    assert_eq!(old.superseded_by.as_deref(), Some("replacement"));
    assert_eq!(old.retired_at.as_deref(), Some(patch.applied_at.as_str()));
    Ok(())
}

#[test]
fn repo_model_patch_rejects_missing_claims_and_dependencies_without_mutation() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("memory.cc");
    let snapshot = fixture_snapshot();
    write_memory_graph_snapshot(&path, &snapshot)?;
    let mut item = frontier_item("orphan", "missing-claim");
    item.dependency_item_ids
        .push("missing-frontier".to_string());
    let patch = patch_for(
        &snapshot,
        vec![RepoModelPatchOperation::UpsertFrontier { item }],
    );

    let error = apply_repo_model_patch(&path, &patch)
        .unwrap_err()
        .to_string();
    assert!(error.contains("missing claim"));
    assert!(error.contains("missing dependency"));
    assert_eq!(load_memory_graph_snapshot(&path)?, Some(snapshot));
    Ok(())
}

#[test]
fn repo_model_patch_requires_revision_for_existing_ids_and_rejects_dependency_cycles() -> Result<()>
{
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("memory.cc");
    let mut snapshot = fixture_snapshot();
    let claim_id = snapshot.nodes[0].id.clone();
    snapshot.frontier = vec![frontier_item("existing", &claim_id)];
    make_canonical(&mut snapshot, 1);
    write_memory_graph_snapshot(&path, &snapshot)?;

    let duplicate = patch_for(
        &snapshot,
        vec![RepoModelPatchOperation::UpsertFrontier {
            item: frontier_item("existing", &claim_id),
        }],
    );
    assert!(
        apply_repo_model_patch(&path, &duplicate)
            .unwrap_err()
            .to_string()
            .contains("use revise")
    );
    assert_eq!(load_memory_graph_snapshot(&path)?, Some(snapshot.clone()));

    let mut first = frontier_item("first", &claim_id);
    first.dependency_item_ids = vec!["second".to_string()];
    let mut second = frontier_item("second", &claim_id);
    second.dependency_item_ids = vec!["first".to_string()];
    let cycle = patch_for(
        &snapshot,
        vec![
            RepoModelPatchOperation::UpsertFrontier { item: first },
            RepoModelPatchOperation::UpsertFrontier { item: second },
        ],
    );
    assert!(
        apply_repo_model_patch(&path, &cycle)
            .unwrap_err()
            .to_string()
            .contains("acyclic")
    );
    assert_eq!(load_memory_graph_snapshot(&path)?, Some(snapshot));
    Ok(())
}

#[test]
fn memory_graph_ids_are_stable_and_normalized() {
    let a = memory_graph_node_id("domain", "module", ".\\Src\\Lib.rs", Some("Thing"));
    let b = memory_graph_node_id("domain", "module", "src/lib.rs", Some("Thing"));
    assert_eq!(a, b);
    assert!(a.starts_with("memnode-"));
}

#[test]
fn memory_graph_documents_round_trip_through_serde() {
    let snapshot = fixture_snapshot();

    let encoded = serde_json::to_string(&snapshot).expect("serialize memory graph snapshot");
    let decoded: EpiphanyMemoryGraphSnapshot =
        serde_json::from_str(&encoded).expect("deserialize memory graph snapshot");

    assert_eq!(decoded, snapshot);
    assert_eq!(
        decoded.nodes[0].profile,
        EpiphanyMemoryProfile::RepoArchitecture
    );
}

#[test]
fn memory_graph_entry_round_trips_through_cultcache() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let store_path = temp.path().join("memory-graph.msgpack");
    let snapshot = fixture_snapshot();

    let written = write_memory_graph_snapshot(&store_path, &snapshot)?;
    let loaded =
        load_memory_graph_snapshot(&store_path)?.expect("memory graph snapshot should load");

    assert_eq!(written.graph_id, snapshot.graph_id);
    assert_eq!(loaded, snapshot);
    Ok(())
}

#[test]
fn memory_graph_entry_validation_rejects_mismatched_identity() {
    let snapshot = fixture_snapshot();
    let mut entry = EpiphanyMemoryGraphEntry::from_snapshot(&snapshot).expect("snapshot entry");
    entry.graph_id = "other".to_string();

    let error = validate_memory_graph_entry(&entry).expect_err("identity mismatch is invalid");

    assert!(error.to_string().contains("does not match snapshot"));
}

#[test]
fn memory_graph_entry_rejects_embedded_legacy_schema() {
    let mut snapshot = fixture_snapshot();
    snapshot.schema_version = Some("epiphany.memory_graph.v0".to_string());
    let entry = EpiphanyMemoryGraphEntry::from_snapshot(&snapshot).expect("snapshot entry");
    let error = validate_memory_graph_entry(&entry).expect_err("embedded v0 is invalid");
    assert!(error.to_string().contains("snapshot schema_version"));
}

#[test]
fn memory_graph_profile_law_keeps_role_memory_from_repo_lifecycle() {
    assert!(lifecycle_allowed_for_profile(
        EpiphanyMemoryProfile::RepoArchitecture,
        EpiphanyMemoryLifecycle::Observed
    ));
    assert!(!lifecycle_allowed_for_profile(
        EpiphanyMemoryProfile::RoleSelf,
        EpiphanyMemoryLifecycle::Observed
    ));
    assert!(!lifecycle_allowed_for_profile(
        EpiphanyMemoryProfile::RepoDataflow,
        EpiphanyMemoryLifecycle::Spoken
    ));
}

#[test]
fn memory_graph_validation_rejects_missing_edge_node_and_bad_lifecycle() {
    let mut snapshot = fixture_snapshot();
    snapshot.edges[0].target_id = "missing".to_string();
    snapshot.nodes[0].lifecycle = EpiphanyMemoryLifecycle::Spoken;

    let errors = validate_memory_graph_snapshot(&snapshot);

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("missing target node"))
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("lifecycle is not legal"))
    );
}

#[test]
fn memory_graph_validation_requires_conservative_summary_anatomy() {
    let mut snapshot = fixture_snapshot();
    snapshot.summaries[0].action_implication.clear();
    snapshot.summaries[0]
        .covers_node_ids
        .push("missing".to_string());

    let errors = validate_memory_graph_snapshot(&snapshot);

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("action implication"))
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("covers missing node"))
    );
}

#[test]
fn memory_graph_compose_merges_profiles_into_one_validated_snapshot() {
    let mut repo = fixture_snapshot();
    repo.freshness = Some(derive_memory_graph_freshness(
        &repo,
        &["hash-a".to_string()],
    ));

    let composed = compose_memory_graph_snapshots("composed-memory", vec![repo, role_snapshot()])
        .expect("profile snapshots should compose");

    assert_eq!(composed.graph_id, "composed-memory");
    assert!(composed.embedding_manifest.is_none());
    assert!(
        composed
            .domains
            .iter()
            .any(|domain| domain.profile == EpiphanyMemoryProfile::RepoArchitecture)
    );
    assert!(
        composed
            .domains
            .iter()
            .any(|domain| domain.profile == EpiphanyMemoryProfile::RoleSelf)
    );
    assert_eq!(
        composed
            .freshness
            .as_ref()
            .map(|freshness| freshness.status),
        Some(EpiphanyMemoryFreshnessStatus::Stale)
    );
}

#[test]
fn memory_graph_compose_rejects_duplicate_profile_authority() {
    let snapshot = fixture_snapshot();

    let errors =
        compose_memory_graph_snapshots("duplicate-memory", vec![snapshot.clone(), snapshot])
            .expect_err("duplicate ids must not be laundered by compose");

    assert!(errors.iter().any(|error| error.path == "domains"));
    assert!(errors.iter().any(|error| error.path == "nodes"));
    assert!(errors.iter().any(|error| error.path == "edges"));
    assert!(errors.iter().any(|error| error.path == "summaries"));
}

#[test]
fn memory_graph_freshness_propagates_dirty_source_hash_to_summary() {
    let snapshot = fixture_snapshot();

    let freshness = derive_memory_graph_freshness(&snapshot, &["hash-a".to_string()]);

    assert_eq!(freshness.status, EpiphanyMemoryFreshnessStatus::Stale);
    assert_eq!(freshness.stale_node_ids.len(), 1);
    assert_eq!(freshness.stale_edge_ids.len(), 1);
    assert_eq!(freshness.stale_summary_ids, vec!["summary-1".to_string()]);
}

#[test]
fn memory_graph_context_cut_uses_fresh_high_confidence_summary() {
    let snapshot = fixture_snapshot();
    let packet = plan_memory_graph_context_cut(
        &snapshot,
        &EpiphanyMemoryContextQuery {
            id: "query".to_string(),
            profile: Some(EpiphanyMemoryProfile::RepoArchitecture),
            text: Some("shared graph law".to_string()),
            ..Default::default()
        },
    );

    assert_eq!(packet.summaries.len(), 1);
    assert!(packet.nodes.is_empty());
    assert!(packet.warnings.is_empty());
}

#[test]
fn memory_graph_context_cut_descends_when_summary_is_stale() {
    let mut snapshot = fixture_snapshot();
    snapshot.freshness = Some(derive_memory_graph_freshness(
        &snapshot,
        &["hash-a".to_string()],
    ));
    let packet = plan_memory_graph_context_cut(
        &snapshot,
        &EpiphanyMemoryContextQuery {
            id: "query".to_string(),
            profile: Some(EpiphanyMemoryProfile::RepoArchitecture),
            text: Some("shared graph law".to_string()),
            ..Default::default()
        },
    );

    assert!(packet.summaries.is_empty());
    assert_eq!(packet.nodes.len(), 2);
    assert_eq!(packet.edges.len(), 1);
    assert_eq!(packet.anchors.len(), 3);
    assert!(
        packet
            .warnings
            .iter()
            .any(|warning| warning.contains("descended"))
    );
}

#[test]
fn memory_graph_context_cut_rederives_forged_ready_freshness() {
    let mut snapshot = fixture_snapshot();
    snapshot.nodes[0].lifecycle = EpiphanyMemoryLifecycle::Stale;
    snapshot.freshness = Some(EpiphanyMemoryFreshness {
        status: EpiphanyMemoryFreshnessStatus::Ready,
        note: Some("forged cache claim".to_string()),
        ..Default::default()
    });
    let packet = plan_memory_graph_context_cut(
        &snapshot,
        &EpiphanyMemoryContextQuery {
            id: "query".to_string(),
            profile: Some(EpiphanyMemoryProfile::RepoArchitecture),
            text: Some("shared graph law".to_string()),
            ..Default::default()
        },
    );
    assert!(packet.summaries.is_empty());
    assert!(
        packet
            .warnings
            .iter()
            .any(|warning| warning.contains("descended"))
    );
}

#[test]
fn memory_graph_context_cut_reports_missing_explicit_ids() {
    let snapshot = fixture_snapshot();
    let packet = plan_memory_graph_context_cut(
        &snapshot,
        &EpiphanyMemoryContextQuery {
            id: "query".to_string(),
            node_ids: vec!["missing-node".to_string()],
            edge_ids: vec!["missing-edge".to_string()],
            ..Default::default()
        },
    );

    assert_eq!(packet.missing_node_ids, vec!["missing-node".to_string()]);
    assert_eq!(packet.missing_edge_ids, vec!["missing-edge".to_string()]);
}

#[test]
fn memory_graph_context_cut_bm25_ranks_typed_claims_before_snapshot_order() {
    let mut snapshot = fixture_snapshot();
    snapshot.summaries.clear();
    snapshot.nodes.reverse();
    let expected = snapshot.nodes[1].id.clone();
    let packet = plan_memory_graph_context_cut(
        &snapshot,
        &EpiphanyMemoryContextQuery {
            id: "semantic-query".to_string(),
            profile: Some(EpiphanyMemoryProfile::RepoArchitecture),
            text: Some("shared graph law profile policy memory module".to_string()),
            budget: Some(1),
            ..Default::default()
        },
    );

    assert_eq!(packet.nodes.len(), 1);
    assert_eq!(packet.nodes[0].id, expected);
}
#[test]
fn memory_graph_context_cut_seeds_frontier_claim_before_irrelevant_semantic_focus() {
    let mut snapshot = fixture_snapshot();
    let target = snapshot.nodes[1].id.clone();
    snapshot.frontier = vec![frontier_item("laser-guidance", &target)];
    let packet = plan_memory_graph_context_cut(
        &snapshot,
        &EpiphanyMemoryContextQuery {
            id: "irrelevant-focus".to_string(),
            text: Some("bananas meteorology unrelated vocabulary".to_string()),
            budget: Some(1),
            ..Default::default()
        },
    );

    assert_eq!(packet.frontier.len(), 1);
    assert_eq!(packet.frontier[0].id, "laser-guidance");
    assert_eq!(packet.nodes.len(), 1);
    assert_eq!(packet.nodes[0].id, target);
}

#[test]
fn memory_graph_identity_rejects_every_mixed_legacy_canonical_shape() {
    let mut revision_without_hash = fixture_snapshot();
    revision_without_hash.model_revision = 1;
    assert!(
        EpiphanyMemoryGraphEntry::from_snapshot(&revision_without_hash)
            .and_then(|entry| validate_memory_graph_entry(&entry))
            .is_err()
    );

    let mut hash_without_revision = fixture_snapshot();
    hash_without_revision.model_hash = memory_graph_model_hash(&hash_without_revision).unwrap();
    assert!(
        EpiphanyMemoryGraphEntry::from_snapshot(&hash_without_revision)
            .and_then(|entry| validate_memory_graph_entry(&entry))
            .is_err()
    );

    let mut frontier_in_legacy = fixture_snapshot();
    let claim = frontier_in_legacy.nodes[0].id.clone();
    frontier_in_legacy.frontier = vec![frontier_item("mixed", &claim)];
    assert!(
        EpiphanyMemoryGraphEntry::from_snapshot(&frontier_in_legacy)
            .and_then(|entry| validate_memory_graph_entry(&entry))
            .is_err()
    );
}

#[test]
fn empty_repo_model_patch_is_rejected_without_revision_churn() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("memory.cc");
    let snapshot = fixture_snapshot();
    write_memory_graph_snapshot(&path, &snapshot)?;
    let patch = patch_for(&snapshot, Vec::new());
    assert!(
        apply_repo_model_patch(&path, &patch)
            .unwrap_err()
            .to_string()
            .contains("at least one operation")
    );
    assert_eq!(load_memory_graph_snapshot(&path)?, Some(snapshot));
    Ok(())
}

#[test]
fn retiring_frontier_target_requires_resolving_frontier_in_same_patch() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("memory.cc");
    let mut snapshot = fixture_snapshot();
    let claim = snapshot.nodes[0].id.clone();
    snapshot.frontier = vec![frontier_item("live-work", &claim)];
    make_canonical(&mut snapshot, 1);
    write_memory_graph_snapshot(&path, &snapshot)?;

    let illegal = patch_for(
        &snapshot,
        vec![RepoModelPatchOperation::RetireNode {
            node_id: claim.clone(),
        }],
    );
    assert!(
        apply_repo_model_patch(&path, &illegal)
            .unwrap_err()
            .to_string()
            .contains("non-live claim")
    );
    assert_eq!(load_memory_graph_snapshot(&path)?, Some(snapshot.clone()));

    let mut resolved = snapshot.frontier[0].clone();
    resolved.status = RepoFrontierStatus::Resolved;
    resolved.updated_at = Some("2026-07-13T01:00:00Z".to_string());
    let legal = patch_for(
        &snapshot,
        vec![
            RepoModelPatchOperation::RetireNode { node_id: claim },
            RepoModelPatchOperation::ReviseFrontier { item: resolved },
        ],
    );
    let applied = apply_repo_model_patch(&path, &legal)?;
    assert_eq!(applied.frontier[0].status, RepoFrontierStatus::Resolved);
    assert_eq!(applied.nodes[0].lifecycle, EpiphanyMemoryLifecycle::Retired);
    Ok(())
}

#[test]
fn verdict_incorporation_closes_adopted_frontier_without_rewriting_execution_plan() -> Result<()> {
    let mut snapshot = fixture_snapshot();
    let claim = snapshot.nodes[0].id.clone();
    let mut item = frontier_item("adopted-work", &claim);
    item.recommended_next_organ = "Hands".into();
    item.adopted_plan = Some(epiphany_state_model::RepoFrontierAdoptedPlan {
        planning_request_id: "planning-1".into(),
        result_id: "imagination-result-1".into(),
        job_id: "imagination-job-1".into(),
        candidate_id: "candidate-1".into(),
        candidate_sha256: "candidate-hash-1".into(),
        safe_paths: item.source_scope.clone(),
        action: "Implement exact bounded change".into(),
        command: "cargo test --lib".into(),
        checks: vec!["focused tests pass".into()],
        stop_conditions: vec!["scope changes".into()],
        rollback_steps: vec!["revert commit".into()],
        commit_message: "Implement exact bounded change".into(),
    });
    snapshot.frontier = vec![item.clone()];
    make_canonical(&mut snapshot, 1);
    let mut resolved = item.clone();
    resolved.status = RepoFrontierStatus::Resolved;
    resolved.evidence_refs = vec!["verification-request-1".into(), "soul-verdict-1".into()];
    resolved.updated_at = Some("2026-07-15T12:00:00Z".into());
    let patch = RepoModelPatch {
        patch_id: "incorporate-adopted-verdict".into(),
        base_revision: snapshot.model_revision,
        base_hash: memory_graph_model_hash(&snapshot)?,
        applied_at: "2026-07-15T12:00:00Z".into(),
        purpose: RepoModelPatchPurpose::IncorporateFrontierVerdict {
            route_id: "route-1".into(),
            soul_verdict_receipt_id: "soul-verdict-1".into(),
        },
        operations: vec![RepoModelPatchOperation::ReviseFrontier {
            item: resolved.clone(),
        }],
    };
    let next = derive_repo_model_patch(&snapshot, &patch)?;
    assert_eq!(next.frontier[0].status, RepoFrontierStatus::Resolved);
    assert_eq!(next.frontier[0].adopted_plan, item.adopted_plan);

    let mut substituted = resolved;
    substituted.adopted_plan.as_mut().unwrap().command = "cargo test unrelated".into();
    let hostile = RepoModelPatch {
        patch_id: "rewrite-adopted-plan".into(),
        operations: vec![RepoModelPatchOperation::ReviseFrontier { item: substituted }],
        ..patch
    };
    assert!(derive_repo_model_patch(&snapshot, &hostile).is_err());
    Ok(())
}

#[test]
fn memory_context_projects_frontier_prerequisites_first_under_budget() {
    let mut snapshot = fixture_snapshot();
    let claim = snapshot.nodes[0].id.clone();
    let mut dependent = frontier_item("dependent", &claim);
    dependent.dependency_item_ids = vec!["prerequisite".to_string()];
    snapshot.frontier = vec![dependent, frontier_item("prerequisite", &claim)];
    let packet = plan_memory_graph_context_cut(
        &snapshot,
        &EpiphanyMemoryContextQuery {
            id: "topological-budget".to_string(),
            budget: Some(1),
            ..Default::default()
        },
    );
    assert_eq!(
        packet
            .frontier
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        vec!["prerequisite"]
    );
}

#[test]
fn raw_snapshot_writer_cannot_replace_canonical_model() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("memory.cc");
    let mut canonical = fixture_snapshot();
    make_canonical(&mut canonical, 1);
    write_memory_graph_snapshot(&path, &canonical)?;
    let mut counterfeit = fixture_snapshot();
    counterfeit.graph_id = "counterfeit".to_string();
    assert!(write_memory_graph_snapshot(&path, &counterfeit).is_err());
    assert_eq!(load_memory_graph_snapshot(&path)?, Some(canonical));
    Ok(())
}

#[test]
fn unresolved_frontier_rejects_source_stale_target_claim() {
    let mut snapshot = fixture_snapshot();
    let claim = snapshot.nodes[0].id.clone();
    snapshot.frontier = vec![frontier_item("stale-target", &claim)];
    snapshot.freshness = Some(epiphany_state_model::EpiphanyMemoryFreshness {
        dirty_source_hashes: vec![snapshot.nodes[0].source_hashes[0].clone()],
        ..Default::default()
    });
    make_canonical(&mut snapshot, 1);
    let entry = EpiphanyMemoryGraphEntry::from_snapshot(&snapshot).unwrap();
    assert!(
        validate_memory_graph_entry(&entry)
            .unwrap_err()
            .to_string()
            .contains("stale source-derived claim")
    );
}

#[test]
fn public_context_cut_omits_unvalidated_frontier_with_retired_target() {
    let mut snapshot = fixture_snapshot();
    let claim = snapshot.nodes[0].id.clone();
    snapshot.nodes[0].lifecycle = EpiphanyMemoryLifecycle::Retired;
    snapshot.frontier = vec![frontier_item("unsafe-frontier", &claim)];
    let packet = plan_memory_graph_context_cut(
        &snapshot,
        &EpiphanyMemoryContextQuery {
            id: "defensive-cut".to_string(),
            ..Default::default()
        },
    );
    assert!(packet.frontier.is_empty());
    assert!(
        packet
            .warnings
            .iter()
            .any(|warning| warning.contains("unsafe-frontier") && warning.contains("retired"))
    );
}

#[test]
fn repo_model_patch_compare_and_swap_allows_exactly_one_cross_process_writer() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("memory.cc");
    let snapshot = fixture_snapshot();
    let claim = snapshot.nodes[0].id.clone();
    write_memory_graph_snapshot(&path, &snapshot)?;
    let base_hash = memory_graph_model_hash(&snapshot)?;
    let go = directory.path().join("go");
    let executable = std::env::current_exe()?;
    let mut children = Vec::new();
    for index in 0..6 {
        let ready = directory.path().join(format!("ready-{index}"));
        let child = std::process::Command::new(&executable)
            .arg("--exact")
            .arg("memory_graph::tests::repo_model_patch_cross_process_worker")
            .arg("--ignored")
            .arg("--nocapture")
            .env("EPIPHANY_CAS_WORKER", "1")
            .env("EPIPHANY_CAS_STORE", &path)
            .env("EPIPHANY_CAS_BASE_HASH", &base_hash)
            .env("EPIPHANY_CAS_CLAIM", &claim)
            .env("EPIPHANY_CAS_ID", index.to_string())
            .env("EPIPHANY_CAS_READY", &ready)
            .env("EPIPHANY_CAS_GO", &go)
            .stdout(std::process::Stdio::piped())
            .spawn()?;
        children.push((child, ready));
    }
    for _ in 0..500 {
        if children.iter().all(|(_, ready)| ready.exists()) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(children.iter().all(|(_, ready)| ready.exists()));
    std::fs::write(&go, b"go")?;
    let mut applied = 0;
    let mut stale = 0;
    for (child, _) in children {
        let output = child.wait_with_output()?;
        assert!(
            output.status.success(),
            "worker failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        applied += usize::from(stdout.contains("CAS_APPLIED"));
        stale += usize::from(stdout.contains("CAS_STALE"));
    }
    assert_eq!(applied, 1);
    assert_eq!(stale, 5);
    assert_eq!(
        load_memory_graph_snapshot(&path)?.unwrap().model_revision,
        1
    );
    Ok(())
}

#[test]
#[ignore = "cross-process helper invoked by CAS parent test"]
fn repo_model_patch_cross_process_worker() -> Result<()> {
    if std::env::var("EPIPHANY_CAS_WORKER").is_err() {
        return Ok(());
    }
    let path = std::path::PathBuf::from(std::env::var("EPIPHANY_CAS_STORE")?);
    let base_hash = std::env::var("EPIPHANY_CAS_BASE_HASH")?;
    let claim = std::env::var("EPIPHANY_CAS_CLAIM")?;
    let id = std::env::var("EPIPHANY_CAS_ID")?;
    let ready = std::path::PathBuf::from(std::env::var("EPIPHANY_CAS_READY")?);
    let go = std::path::PathBuf::from(std::env::var("EPIPHANY_CAS_GO")?);
    std::fs::write(ready, b"ready")?;
    for _ in 0..500 {
        if go.exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let patch = RepoModelPatch {
        purpose: RepoModelPatchPurpose::Evolution,
        patch_id: format!("cross-process-{id}"),
        base_revision: 0,
        base_hash,
        applied_at: "2026-07-13T02:00:00Z".to_string(),
        operations: vec![RepoModelPatchOperation::UpsertFrontier {
            item: frontier_item(&format!("cross-process-{id}"), &claim),
        }],
    };
    match apply_repo_model_patch(&path, &patch) {
        Ok(_) => println!("CAS_APPLIED"),
        Err(error) if error.to_string().contains("stale repo model patch") => println!("CAS_STALE"),
        Err(error) => return Err(error),
    }
    Ok(())
}
