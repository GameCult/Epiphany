use crate::memory_graph::EpiphanyMemoryAnchor;
use crate::memory_graph::EpiphanyMemoryDomain;
use crate::memory_graph::EpiphanyMemoryEdge;
use crate::memory_graph::EpiphanyMemoryEdgeKind;
use crate::memory_graph::EpiphanyMemoryFreshness;
use crate::memory_graph::EpiphanyMemoryFreshnessStatus;
use crate::memory_graph::EpiphanyMemoryGraphSnapshot;
use crate::memory_graph::EpiphanyMemoryLifecycle;
use crate::memory_graph::EpiphanyMemoryNode;
use crate::memory_graph::EpiphanyMemoryNodeKind;
use crate::memory_graph::EpiphanyMemoryProfile;
use crate::memory_graph::EpiphanyMemorySummary;
use crate::memory_graph::memory_graph_domain_id;
use crate::memory_graph::memory_graph_edge_id;
use crate::memory_graph::memory_graph_node_id;
use anyhow::{Context, Result};
use epiphany_state_model::EpiphanyCodeRef;
use epiphany_state_model::EpiphanyGraph;
use epiphany_state_model::EpiphanyGraphEdge;
use epiphany_state_model::EpiphanyGraphLink;
use epiphany_state_model::EpiphanyGraphNode;
use epiphany_state_model::EpiphanyGraphs;
use epiphany_state_model::EpiphanyThreadState;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepoMemoryGraphRefresh {
    Reused,
    Refreshed,
}

pub fn refresh_or_validate_repo_memory_graph(
    store_path: &Path,
    source_identity: &str,
    state: &EpiphanyThreadState,
    repo_root: &Path,
    graph_id: impl Into<String>,
) -> Result<(EpiphanyMemoryGraphSnapshot, RepoMemoryGraphRefresh)> {
    let graph_id = graph_id.into();
    for _ in 0..4 {
        let inspected = crate::memory_graph::store::load_memory_graph_envelope(store_path)?;
        if let Some(envelope) = &inspected {
            let entry: crate::memory_graph::EpiphanyMemoryGraphEntry =
                rmp_serde::from_slice(&envelope.payload)?;
            crate::memory_graph::validate_memory_graph_entry(&entry)?;
            let snapshot = entry.snapshot()?;
            if snapshot.model_revision > 0 {
                return Ok((snapshot, RepoMemoryGraphRefresh::Reused));
            }
            let provenance_matches = snapshot.source.as_ref().is_some_and(|source| {
                source.kind == "thread_state_repo_graph"
                    && source.identity == source_identity
                    && source.revision == state.revision
            });
            if provenance_matches && snapshot_anchor_bytes_match(&snapshot, repo_root) {
                return Ok((snapshot, RepoMemoryGraphRefresh::Reused));
            }
        }

        let snapshot = memory_graph_from_epiphany_graphs(
            graph_id.clone(),
            &state.graphs,
            source_identity,
            state.revision,
            repo_root,
        )?;
        pause_after_legacy_read_for_test();
        if crate::memory_graph::store::replace_inspected_legacy_memory_graph(
            store_path,
            inspected.as_ref(),
            &snapshot,
            &chrono::Utc::now().to_rfc3339(),
        )? {
            return Ok((snapshot, RepoMemoryGraphRefresh::Refreshed));
        }
        // Another process won. Reload under the next iteration; canonical state
        // is authoritative and a competing legacy refresh may be retried.
    }
    Err(anyhow::anyhow!(
        "memory graph bootstrap could not settle after concurrent writers"
    ))
}

#[cfg(not(test))]
fn pause_after_legacy_read_for_test() {}

#[cfg(test)]
fn pause_after_legacy_read_for_test() {
    if let Some((ready, release)) = REFRESH_AFTER_LEGACY_READ_HOOK
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .expect("refresh test hook lock")
        .take()
    {
        ready.wait();
        release.wait();
    }
}

#[cfg(test)]
static REFRESH_AFTER_LEGACY_READ_HOOK: std::sync::OnceLock<
    std::sync::Mutex<
        Option<(
            std::sync::Arc<std::sync::Barrier>,
            std::sync::Arc<std::sync::Barrier>,
        )>,
    >,
> = std::sync::OnceLock::new();

fn snapshot_anchor_bytes_match(snapshot: &EpiphanyMemoryGraphSnapshot, repo_root: &Path) -> bool {
    snapshot
        .nodes
        .iter()
        .flat_map(|node| &node.anchors)
        .chain(snapshot.edges.iter().flat_map(|edge| &edge.anchors))
        .all(|anchor| {
            let (Some(code_ref), Some(expected)) = (&anchor.code_ref, &anchor.source_hash) else {
                return false;
            };
            fs::read(repo_root.join(&code_ref.path))
                .ok()
                .is_some_and(|bytes| format!("sha256:{:x}", Sha256::digest(bytes)) == *expected)
        })
}

pub fn memory_graph_from_epiphany_graphs(
    graph_id: impl Into<String>,
    graphs: &EpiphanyGraphs,
    source_identity: impl Into<String>,
    source_revision: u64,
    repo_root: &Path,
) -> Result<EpiphanyMemoryGraphSnapshot> {
    let architecture_domain = memory_graph_domain_id(
        EpiphanyMemoryProfile::RepoArchitecture,
        "accepted_graph",
        "architecture",
    );
    let dataflow_domain = memory_graph_domain_id(
        EpiphanyMemoryProfile::RepoDataflow,
        "accepted_graph",
        "dataflow",
    );

    let mut domains = Vec::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut summaries = Vec::new();
    let mut node_map = HashMap::new();

    if !graphs.architecture.is_empty() {
        domains.push(repo_domain(
            architecture_domain.clone(),
            EpiphanyMemoryProfile::RepoArchitecture,
            "Accepted architecture graph",
        ));
        import_graph_nodes(
            &graphs.architecture,
            &architecture_domain,
            EpiphanyMemoryProfile::RepoArchitecture,
            &mut node_map,
            &mut nodes,
            repo_root,
        );
        import_graph_edges(
            &graphs.architecture,
            EpiphanyMemoryProfile::RepoArchitecture,
            &node_map,
            &mut edges,
            repo_root,
        );
        summaries.push(repo_summary(
            "summary-accepted-architecture-graph",
            &architecture_domain,
            "accepted architecture graph",
            "Accepted architecture graph nodes are available as memory graph repo-profile claims.",
            &nodes,
            &edges,
            EpiphanyMemoryProfile::RepoArchitecture,
        ));
    }

    if !graphs.dataflow.is_empty() {
        domains.push(repo_domain(
            dataflow_domain.clone(),
            EpiphanyMemoryProfile::RepoDataflow,
            "Accepted dataflow graph",
        ));
        import_graph_nodes(
            &graphs.dataflow,
            &dataflow_domain,
            EpiphanyMemoryProfile::RepoDataflow,
            &mut node_map,
            &mut nodes,
            repo_root,
        );
        import_graph_edges(
            &graphs.dataflow,
            EpiphanyMemoryProfile::RepoDataflow,
            &node_map,
            &mut edges,
            repo_root,
        );
        summaries.push(repo_summary(
            "summary-accepted-dataflow-graph",
            &dataflow_domain,
            "accepted dataflow graph",
            "Accepted dataflow graph nodes are available as memory graph repo-profile claims.",
            &nodes,
            &edges,
            EpiphanyMemoryProfile::RepoDataflow,
        ));
    }

    import_graph_links(graphs.links.as_slice(), &node_map, &mut edges, repo_root);

    let stale_node_ids = nodes
        .iter()
        .filter(|node| node.lifecycle == EpiphanyMemoryLifecycle::Stale)
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    let stale_edge_ids = edges
        .iter()
        .filter(|edge| edge.lifecycle == EpiphanyMemoryLifecycle::Stale)
        .map(|edge| edge.id.clone())
        .collect::<Vec<_>>();
    let stale = !stale_node_ids.is_empty() || !stale_edge_ids.is_empty();
    if stale {
        for summary in &mut summaries {
            summary.freshness = EpiphanyMemoryFreshnessStatus::Stale;
        }
    }
    Ok(EpiphanyMemoryGraphSnapshot {
        schema_version: Some("epiphany.memory_graph.v1".to_string()),
        graph_id: graph_id.into(),
        source: Some(crate::memory_graph::EpiphanyMemoryGraphSource {
            kind: "thread_state_repo_graph".to_string(),
            identity: source_identity.into(),
            revision: source_revision,
        }),
        domains,
        nodes,
        edges,
        summaries,
        freshness: Some(EpiphanyMemoryFreshness {
            status: if stale {
                EpiphanyMemoryFreshnessStatus::Stale
            } else {
                EpiphanyMemoryFreshnessStatus::Ready
            },
            stale_node_ids,
            stale_edge_ids,
            note: Some(
                if stale {
                    "One or more accepted graph anchors could not be verified against source bytes."
                } else {
                    "Imported from accepted Epiphany graph state."
                }
                .to_string(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    })
}

fn repo_domain(id: String, profile: EpiphanyMemoryProfile, title: &str) -> EpiphanyMemoryDomain {
    EpiphanyMemoryDomain {
        id,
        profile,
        title: title.to_string(),
        description: Some("Repo profile imported from accepted Epiphany graph state.".to_string()),
        lifecycle: EpiphanyMemoryLifecycle::Accepted,
    }
}

fn import_graph_nodes(
    graph: &EpiphanyGraph,
    domain_id: &str,
    profile: EpiphanyMemoryProfile,
    node_map: &mut HashMap<String, String>,
    nodes: &mut Vec<EpiphanyMemoryNode>,
    repo_root: &Path,
) {
    for node in &graph.nodes {
        let memory_id = memory_graph_node_id(domain_id, "accepted_graph_node", &node.id, None);
        node_map.insert(node.id.clone(), memory_id.clone());
        nodes.push(
            memory_node_from_graph_node(memory_id.clone(), domain_id, profile, node, repo_root)
                .unwrap_or_else(|_| stale_memory_node(memory_id, domain_id, profile, node)),
        );
    }
}

fn memory_node_from_graph_node(
    id: String,
    domain_id: &str,
    profile: EpiphanyMemoryProfile,
    node: &EpiphanyGraphNode,
    repo_root: &Path,
) -> Result<EpiphanyMemoryNode> {
    let anchors = anchors_from_code_refs(&id, &node.code_refs, repo_root)?;
    let source_hashes = if anchors.is_empty() {
        vec!["anchor:missing".to_string()]
    } else {
        anchors
            .iter()
            .filter_map(|anchor| anchor.source_hash.clone())
            .collect()
    };
    let grounded = !anchors.is_empty();
    Ok(EpiphanyMemoryNode {
        id,
        domain_id: domain_id.to_string(),
        profile,
        kind: memory_node_kind(node),
        title: node.title.clone(),
        claim: graph_node_claim(node),
        question: "What changes if this accepted graph node is stale?".to_string(),
        tension: node.mechanism.clone().unwrap_or_default(),
        action_implication: format!(
            "Use accepted graph node {} when selecting repo context.",
            node.id
        ),
        anchors,
        source_hashes,
        lifecycle: if grounded {
            graph_status_lifecycle(node.status.as_deref())
        } else {
            EpiphanyMemoryLifecycle::Stale
        },
        salience: 70,
        confidence: if grounded { 80 } else { 0 },
        ..Default::default()
    })
}

fn stale_memory_node(
    id: String,
    domain_id: &str,
    profile: EpiphanyMemoryProfile,
    node: &EpiphanyGraphNode,
) -> EpiphanyMemoryNode {
    EpiphanyMemoryNode {
        id,
        domain_id: domain_id.to_string(),
        profile,
        kind: memory_node_kind(node),
        title: node.title.clone(),
        claim: graph_node_claim(node),
        question: "Source anchor is unavailable; refresh after restoring it.".to_string(),
        tension: "Accepted anatomy cannot be verified against source bytes.".to_string(),
        action_implication: "Do not use this claim as fresh compressed anatomy.".to_string(),
        source_hashes: vec!["anchor:missing".to_string()],
        lifecycle: EpiphanyMemoryLifecycle::Stale,
        salience: 70,
        confidence: 0,
        ..Default::default()
    }
}

fn import_graph_edges(
    graph: &EpiphanyGraph,
    profile: EpiphanyMemoryProfile,
    node_map: &HashMap<String, String>,
    edges: &mut Vec<EpiphanyMemoryEdge>,
    repo_root: &Path,
) {
    for edge in &graph.edges {
        let source_id = node_map
            .get(&edge.source_id)
            .cloned()
            .unwrap_or_else(|| edge.source_id.clone());
        let target_id = node_map
            .get(&edge.target_id)
            .cloned()
            .unwrap_or_else(|| edge.target_id.clone());
        edges.push(memory_edge_from_graph_edge(
            source_id, target_id, profile, edge, repo_root,
        ));
    }
}

fn memory_edge_from_graph_edge(
    source_id: String,
    target_id: String,
    profile: EpiphanyMemoryProfile,
    edge: &EpiphanyGraphEdge,
    repo_root: &Path,
) -> EpiphanyMemoryEdge {
    let id = edge.id.clone().unwrap_or_else(|| {
        memory_graph_edge_id(
            &source_id,
            &target_id,
            edge.kind.as_str(),
            edge.code_refs.iter().map(code_ref_key),
        )
    });
    let anchors = anchors_from_code_refs(
        edge.id.as_deref().unwrap_or("edge"),
        &edge.code_refs,
        repo_root,
    );
    let grounded = edge.code_refs.is_empty() || anchors.is_ok();
    EpiphanyMemoryEdge {
        id,
        source_id,
        target_id,
        kind: memory_edge_kind(edge.kind.as_str()),
        profile,
        claim: graph_edge_claim(edge),
        anchors: anchors.unwrap_or_default(),
        lifecycle: if grounded {
            EpiphanyMemoryLifecycle::Accepted
        } else {
            EpiphanyMemoryLifecycle::Stale
        },
        confidence: if grounded { 80 } else { 0 },
    }
}

fn import_graph_links(
    links: &[EpiphanyGraphLink],
    node_map: &HashMap<String, String>,
    edges: &mut Vec<EpiphanyMemoryEdge>,
    repo_root: &Path,
) {
    for link in links {
        let Some(source_id) = node_map.get(&link.dataflow_node_id).cloned() else {
            continue;
        };
        let Some(target_id) = node_map.get(&link.architecture_node_id).cloned() else {
            continue;
        };
        let anchors = anchors_from_code_refs("link", &link.code_refs, repo_root);
        let grounded = link.code_refs.is_empty() || anchors.is_ok();
        edges.push(EpiphanyMemoryEdge {
            id: memory_graph_edge_id(
                &source_id,
                &target_id,
                "grounds",
                link.code_refs.iter().map(code_ref_key),
            ),
            source_id,
            target_id,
            kind: EpiphanyMemoryEdgeKind::Grounds,
            profile: EpiphanyMemoryProfile::RepoDataflow,
            claim: link
                .relationship
                .clone()
                .unwrap_or_else(|| "Dataflow node is linked to architecture node.".to_string()),
            anchors: anchors.unwrap_or_default(),
            lifecycle: if grounded {
                EpiphanyMemoryLifecycle::Accepted
            } else {
                EpiphanyMemoryLifecycle::Stale
            },
            confidence: if grounded { 75 } else { 0 },
        });
    }
}

fn repo_summary(
    id: &str,
    domain_id: &str,
    target: &str,
    claim: &str,
    nodes: &[EpiphanyMemoryNode],
    edges: &[EpiphanyMemoryEdge],
    profile: EpiphanyMemoryProfile,
) -> EpiphanyMemorySummary {
    let covers_node_ids = nodes
        .iter()
        .filter(|node| node.profile == profile)
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    let covers_edge_ids = edges
        .iter()
        .filter(|edge| edge.profile == profile)
        .map(|edge| edge.id.clone())
        .collect::<Vec<_>>();
    let anchor_count = nodes
        .iter()
        .filter(|node| node.profile == profile)
        .map(|node| node.anchors.len() as u32)
        .sum();
    EpiphanyMemorySummary {
        id: id.to_string(),
        domain_id: domain_id.to_string(),
        covers_node_ids,
        covers_edge_ids,
        target: target.to_string(),
        claim: claim.to_string(),
        question: "Which profile producer should refresh this accepted graph next?".to_string(),
        tension: String::new(),
        action_implication: "Use this summary for broad repo context; descend when exact nodes are stale or relevant.".to_string(),
        anchor_count,
        freshness: EpiphanyMemoryFreshnessStatus::Ready,
        confidence: 80,
        ..Default::default()
    }
}

fn anchors_from_code_refs(
    prefix: &str,
    code_refs: &[EpiphanyCodeRef],
    repo_root: &Path,
) -> Result<Vec<EpiphanyMemoryAnchor>> {
    code_refs
        .iter()
        .enumerate()
        .map(|(index, code_ref)| {
            let bytes = fs::read(repo_root.join(&code_ref.path))
                .with_context(|| format!("missing source anchor {}", code_ref.path.display()))?;
            let digest = format!("sha256:{:x}", Sha256::digest(&bytes));
            Ok(EpiphanyMemoryAnchor {
                id: format!("anchor-{prefix}-{index}"),
                kind: "code_ref".to_string(),
                target: code_ref_key(code_ref),
                code_ref: Some(code_ref.clone()),
                source_hash: Some(digest),
                ..Default::default()
            })
        })
        .collect()
}

fn code_ref_key(code_ref: &EpiphanyCodeRef) -> String {
    let mut key = code_ref.path.to_string_lossy().replace('\\', "/");
    if let Some(symbol) = code_ref.symbol.as_deref() {
        key.push('#');
        key.push_str(symbol);
    }
    key
}

fn graph_node_claim(node: &EpiphanyGraphNode) -> String {
    if !node.purpose.trim().is_empty() {
        return node.purpose.clone();
    }
    format!("Accepted graph node {} exists.", node.id)
}

fn graph_edge_claim(edge: &EpiphanyGraphEdge) -> String {
    edge.mechanism
        .clone()
        .or_else(|| edge.label.clone())
        .unwrap_or_else(|| {
            format!(
                "Accepted graph edge {} connects {} to {}.",
                edge.kind, edge.source_id, edge.target_id
            )
        })
}

fn graph_status_lifecycle(status: Option<&str>) -> EpiphanyMemoryLifecycle {
    match status.unwrap_or_default() {
        "observed" => EpiphanyMemoryLifecycle::Observed,
        "proposed" => EpiphanyMemoryLifecycle::Proposed,
        "stale" => EpiphanyMemoryLifecycle::Stale,
        "retired" => EpiphanyMemoryLifecycle::Retired,
        _ => EpiphanyMemoryLifecycle::Accepted,
    }
}

fn memory_node_kind(node: &EpiphanyGraphNode) -> EpiphanyMemoryNodeKind {
    let text = format!("{} {}", node.title, node.purpose).to_lowercase();
    if text.contains("schema") {
        EpiphanyMemoryNodeKind::Schema
    } else if text.contains("runtime") || text.contains("contract") {
        EpiphanyMemoryNodeKind::RuntimeContract
    } else if text.contains("adapter") || text.contains("bridge") {
        EpiphanyMemoryNodeKind::Adapter
    } else if text.contains("test") || text.contains("smoke") {
        EpiphanyMemoryNodeKind::TestSeam
    } else if text.contains("state") || text.contains("store") {
        EpiphanyMemoryNodeKind::StateStore
    } else {
        EpiphanyMemoryNodeKind::Module
    }
}

fn memory_edge_kind(kind: &str) -> EpiphanyMemoryEdgeKind {
    match kind {
        "owns" => EpiphanyMemoryEdgeKind::Owns,
        "reads" => EpiphanyMemoryEdgeKind::Reads,
        "writes" => EpiphanyMemoryEdgeKind::Writes,
        "derives" => EpiphanyMemoryEdgeKind::Derives,
        "adapts" => EpiphanyMemoryEdgeKind::Adapts,
        "persists" => EpiphanyMemoryEdgeKind::Persists,
        "launches" => EpiphanyMemoryEdgeKind::Launches,
        "verifies" | "tests" => EpiphanyMemoryEdgeKind::Verifies,
        "depends_on" | "depends" => EpiphanyMemoryEdgeKind::DependsOn,
        _ => EpiphanyMemoryEdgeKind::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_graph::validate_memory_graph_snapshot;
    use epiphany_state_model::EpiphanyCodeRef;
    use epiphany_state_model::EpiphanyGraph;
    use epiphany_state_model::EpiphanyGraphEdge;
    use epiphany_state_model::EpiphanyGraphNode;
    use epiphany_state_model::EpiphanyGraphs;
    use std::path::PathBuf;

    #[test]
    fn repo_profile_imports_accepted_graphs_into_valid_memory_graph() {
        let graphs = EpiphanyGraphs {
            architecture: EpiphanyGraph {
                nodes: vec![EpiphanyGraphNode {
                    id: "core".to_string(),
                    title: "Core policy".to_string(),
                    purpose: "Owns shared policy.".to_string(),
                    code_refs: vec![EpiphanyCodeRef {
                        path: PathBuf::from("epiphany-core/src/lib.rs"),
                        symbol: Some("policy".to_string()),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                edges: vec![EpiphanyGraphEdge {
                    id: Some("edge-core-self".to_string()),
                    source_id: "core".to_string(),
                    target_id: "core".to_string(),
                    kind: "owns".to_string(),
                    label: Some("owns itself for fixture".to_string()),
                    ..Default::default()
                }],
            },
            ..Default::default()
        };

        let snapshot = memory_graph_from_epiphany_graphs(
            "repo-profile",
            &graphs,
            "test-state",
            1,
            Path::new("."),
        )
        .unwrap();
        let errors = validate_memory_graph_snapshot(&snapshot);

        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(snapshot.domains.len(), 1);
        assert_eq!(snapshot.nodes.len(), 1);
        assert_eq!(snapshot.edges.len(), 1);
        assert_eq!(snapshot.summaries.len(), 1);
        assert_eq!(
            snapshot.nodes[0].profile,
            EpiphanyMemoryProfile::RepoArchitecture
        );
    }

    #[test]
    fn repo_profile_keeps_bad_topology_visible() {
        let graphs = EpiphanyGraphs {
            architecture: EpiphanyGraph {
                edges: vec![EpiphanyGraphEdge {
                    source_id: "missing-source".to_string(),
                    target_id: "missing-target".to_string(),
                    kind: "owns".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            ..Default::default()
        };

        let snapshot = memory_graph_from_epiphany_graphs(
            "repo-profile",
            &graphs,
            "test-state",
            1,
            Path::new("."),
        )
        .unwrap();
        let errors = validate_memory_graph_snapshot(&snapshot);

        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("missing source node"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("missing target node"))
        );
    }

    fn anchored_state(path: &str, revision: u64) -> EpiphanyThreadState {
        EpiphanyThreadState {
            revision,
            graphs: EpiphanyGraphs {
                architecture: EpiphanyGraph {
                    nodes: vec![EpiphanyGraphNode {
                        id: "body".to_string(),
                        title: "Body".to_string(),
                        purpose: "Grounded body anatomy.".to_string(),
                        code_refs: vec![EpiphanyCodeRef {
                            path: PathBuf::from(path),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn repo_cache_reuse_requires_revision_and_anchor_bytes() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        std::fs::write(temp.path().join("body.rs"), "one")?;
        let store = temp.path().join("memory-graph.msgpack");
        let state = anchored_state("body.rs", 1);

        let (first, first_refresh) =
            refresh_or_validate_repo_memory_graph(&store, "thread", &state, temp.path(), "repo")?;
        assert_eq!(first_refresh, RepoMemoryGraphRefresh::Refreshed);
        assert!(first.nodes[0].source_hashes[0].starts_with("sha256:"));
        let (same, same_refresh) =
            refresh_or_validate_repo_memory_graph(&store, "thread", &state, temp.path(), "repo")?;
        assert_eq!(same_refresh, RepoMemoryGraphRefresh::Reused);
        assert_eq!(first.source, same.source);

        std::fs::write(temp.path().join("body.rs"), "two")?;
        let (changed, bytes_refresh) =
            refresh_or_validate_repo_memory_graph(&store, "thread", &state, temp.path(), "repo")?;
        assert_eq!(bytes_refresh, RepoMemoryGraphRefresh::Refreshed);
        assert_ne!(first.nodes[0].source_hashes, changed.nodes[0].source_hashes);

        let next = anchored_state("body.rs", 2);
        let (revised, revision_refresh) =
            refresh_or_validate_repo_memory_graph(&store, "thread", &next, temp.path(), "repo")?;
        assert_eq!(revision_refresh, RepoMemoryGraphRefresh::Refreshed);
        assert_eq!(revised.source.unwrap().revision, 2);
        Ok(())
    }

    #[test]
    fn repo_refresh_cannot_replace_canonical_model() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        std::fs::write(temp.path().join("body.rs"), "one")?;
        let store = temp.path().join("memory-graph.msgpack");
        let state = anchored_state("body.rs", 1);
        let (mut canonical, _) =
            refresh_or_validate_repo_memory_graph(&store, "thread", &state, temp.path(), "repo")?;
        canonical.model_revision = 1;
        canonical.model_hash = crate::memory_graph::memory_graph_model_hash(&canonical)?;
        crate::memory_graph::write_memory_graph_snapshot(&store, &canonical)?;

        std::fs::write(temp.path().join("body.rs"), "changed")?;
        let newer = anchored_state("body.rs", 999);
        let (preserved, refresh) = refresh_or_validate_repo_memory_graph(
            &store,
            "thread",
            &newer,
            temp.path(),
            "counterfeit",
        )?;
        assert_eq!(refresh, RepoMemoryGraphRefresh::Reused);
        assert_eq!(preserved.model_revision, 1);
        assert_eq!(preserved.model_hash, canonical.model_hash);
        assert_eq!(preserved.graph_id, canonical.graph_id);
        Ok(())
    }

    #[test]
    fn canonical_patch_winning_after_refresh_read_survives_refresh_resume() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        std::fs::write(temp.path().join("body.rs"), "one")?;
        let store = temp.path().join("memory-graph.msgpack");
        let state = anchored_state("body.rs", 1);
        let (legacy, _) =
            refresh_or_validate_repo_memory_graph(&store, "thread", &state, temp.path(), "repo")?;
        let claim = legacy.nodes[0].id.clone();
        let ready = std::sync::Arc::new(std::sync::Barrier::new(2));
        let release = std::sync::Arc::new(std::sync::Barrier::new(2));
        *REFRESH_AFTER_LEGACY_READ_HOOK
            .get_or_init(|| std::sync::Mutex::new(None))
            .lock()
            .unwrap() = Some((ready.clone(), release.clone()));

        let refresh_store = store.clone();
        let refresh_root = temp.path().to_path_buf();
        let refresh_state = anchored_state("body.rs", 999);
        let refresh = std::thread::spawn(move || {
            refresh_or_validate_repo_memory_graph(
                &refresh_store,
                "thread",
                &refresh_state,
                &refresh_root,
                "counterfeit-refresh",
            )
        });
        ready.wait();
        let patch = crate::memory_graph::RepoModelPatch {
            purpose: crate::memory_graph::RepoModelPatchPurpose::Evolution,
            patch_id: "canonical-wins-refresh-race".to_string(),
            base_revision: 0,
            base_hash: crate::memory_graph::memory_graph_model_hash(&legacy)?,
            applied_at: "2026-07-13T03:00:00Z".to_string(),
            operations: vec![
                crate::memory_graph::RepoModelPatchOperation::UpsertFrontier {
                    item: crate::memory_graph::RepoFrontierItem {
                        id: "frontier-survives-refresh".to_string(),
                        migration_body: "Preserve canonical Modeling authority.".to_string(),
                        question: "Can a stale refresh overwrite this?".to_string(),
                        gap: "The old refresh replaced the complete file.".to_string(),
                        target_claim_ids: vec![claim],
                        source_scope: vec!["body.rs".to_string()],
                        recommended_next_organ: "Soul".to_string(),
                        status: crate::memory_graph::RepoFrontierStatus::Active,
                        ..Default::default()
                    },
                },
            ],
        };
        let canonical = crate::memory_graph::apply_repo_model_patch(&store, &patch)?;
        release.wait();
        let (observed, outcome) = refresh.join().expect("refresh thread")?;
        assert_eq!(outcome, RepoMemoryGraphRefresh::Reused);
        assert_eq!(observed.model_revision, canonical.model_revision);
        assert_eq!(observed.model_hash, canonical.model_hash);
        assert_eq!(observed.frontier, canonical.frontier);
        assert_eq!(observed.lifecycle_receipts, canonical.lifecycle_receipts);
        Ok(())
    }

    #[test]
    fn missing_anchor_cannot_emit_ready_summary() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let snapshot = memory_graph_from_epiphany_graphs(
            "repo",
            &anchored_state("gone.rs", 1).graphs,
            "thread",
            1,
            temp.path(),
        )?;
        assert_eq!(
            snapshot.freshness.unwrap().status,
            EpiphanyMemoryFreshnessStatus::Stale
        );
        assert!(
            snapshot
                .summaries
                .iter()
                .all(|summary| summary.freshness == EpiphanyMemoryFreshnessStatus::Stale)
        );
        Ok(())
    }

    #[test]
    fn missing_edge_anchor_stales_edge_summary_and_snapshot() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        std::fs::write(temp.path().join("body.rs"), "body")?;
        let mut state = anchored_state("body.rs", 1);
        state.graphs.architecture.edges.push(EpiphanyGraphEdge {
            id: Some("edge-body".to_string()),
            source_id: "body".to_string(),
            target_id: "body".to_string(),
            kind: "owns".to_string(),
            code_refs: vec![EpiphanyCodeRef {
                path: PathBuf::from("missing-edge.rs"),
                ..Default::default()
            }],
            ..Default::default()
        });
        let snapshot =
            memory_graph_from_epiphany_graphs("repo", &state.graphs, "thread", 1, temp.path())?;
        assert_eq!(snapshot.edges[0].lifecycle, EpiphanyMemoryLifecycle::Stale);
        assert_eq!(
            snapshot.summaries[0].freshness,
            EpiphanyMemoryFreshnessStatus::Stale
        );
        let freshness = snapshot.freshness.expect("freshness");
        assert_eq!(freshness.status, EpiphanyMemoryFreshnessStatus::Stale);
        assert_eq!(freshness.stale_edge_ids, vec!["edge-body".to_string()]);
        Ok(())
    }
}
