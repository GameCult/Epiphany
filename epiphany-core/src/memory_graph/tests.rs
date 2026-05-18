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
            title: "Body".to_string(),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        }],
        nodes: vec![EpiphanyMemoryNode {
            id: node_id.clone(),
            domain_id: domain_id.clone(),
            profile: EpiphanyMemoryProfile::RoleSelf,
            kind: EpiphanyMemoryNodeKind::RoleMemory,
            title: "Typed boundaries".to_string(),
            claim: "Body remembers that typed boundaries keep the machine legible.".to_string(),
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
            claim: "Body role memory enters the shared graph as typed state.".to_string(),
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
