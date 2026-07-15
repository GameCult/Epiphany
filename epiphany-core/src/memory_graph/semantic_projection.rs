use super::{
    EpiphanyMemoryDomain, EpiphanyMemoryFreshnessStatus, EpiphanyMemoryGraphSnapshot,
    EpiphanyMemoryLifecycle, EpiphanyMemoryProfile, MEMORY_GRAPH_KEY, MEMORY_GRAPH_TYPE,
    RepoFrontierStatus, memory_graph_model_hash, validate_memory_graph_snapshot,
};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub const SEMANTIC_PROJECTION_SCHEMA_VERSION: &str = "gamecult.epiphany.semantic_projection.v0";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPartition {
    Mind,
    Modeling,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticDocumentKind {
    Node,
    Edge,
    Summary,
    Frontier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticVisibility {
    PrivateVerse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "owner", content = "value", rename_all = "snake_case")]
pub enum SemanticLifecycle {
    Memory(EpiphanyMemoryLifecycle),
    Frontier(RepoFrontierStatus),
    Derived,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticCanonicalLocator {
    pub locator: String,
    pub canonical_type: String,
    pub canonical_key: String,
    pub canonical_document_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticProjectionDocument {
    pub point_id: String,
    pub swarm_id: String,
    pub partition: SemanticPartition,
    pub kind: SemanticDocumentKind,
    pub canonical: SemanticCanonicalLocator,
    pub graph_id: String,
    pub model_revision: u64,
    pub model_hash: String,
    pub domain_id: String,
    pub profile: EpiphanyMemoryProfile,
    pub lifecycle: SemanticLifecycle,
    pub visibility: SemanticVisibility,
    pub source_refs: Vec<String>,
    pub source_hashes: Vec<String>,
    pub canonical_content_hash: String,
    pub canonical_schema_version: String,
    pub projection_schema_version: String,
    pub projection_text: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SemanticProjectionCandidate {
    pub point_id: String,
    pub canonical: SemanticCanonicalLocator,
    pub partition: SemanticPartition,
    pub score: f32,
    pub indexed_model_revision: u64,
    pub indexed_model_hash: String,
    pub indexed_canonical_content_hash: String,
}

pub trait SemanticEmbedder {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}

pub trait SemanticVectorIndex {
    fn replace_partition(
        &self,
        swarm_id: &str,
        partition: SemanticPartition,
        documents: &[(SemanticProjectionDocument, Vec<f32>)],
    ) -> Result<()>;

    fn search(
        &self,
        swarm_id: &str,
        partition: SemanticPartition,
        vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SemanticProjectionCandidate>>;
}

pub fn semantic_point_id(
    swarm_id: &str,
    partition: SemanticPartition,
    canonical_type: &str,
    canonical_key: &str,
    canonical_document_id: &str,
) -> String {
    let partition = match partition {
        SemanticPartition::Mind => "mind",
        SemanticPartition::Modeling => "modeling",
    };
    Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("{swarm_id}|{partition}|{canonical_type}|{canonical_key}|{canonical_document_id}")
            .as_bytes(),
    )
    .to_string()
}

pub fn derive_semantic_projection(
    swarm_id: &str,
    snapshot: &EpiphanyMemoryGraphSnapshot,
) -> Result<Vec<SemanticProjectionDocument>> {
    if swarm_id.trim().is_empty() {
        return Err(anyhow!("semantic projection requires swarm_id"));
    }
    let validation = validate_memory_graph_snapshot(snapshot);
    if !validation.is_empty() {
        return Err(anyhow!("memory graph is invalid: {validation:?}"));
    }
    let computed_hash = memory_graph_model_hash(snapshot)?;
    if !snapshot.model_hash.is_empty() && snapshot.model_hash != computed_hash {
        return Err(anyhow!(
            "memory graph model_hash does not match canonical content"
        ));
    }
    let mut canonical_snapshot = snapshot.clone();
    canonical_snapshot.model_hash = computed_hash;
    let snapshot = &canonical_snapshot;

    let domains = snapshot
        .domains
        .iter()
        .map(|domain| (domain.id.as_str(), domain))
        .collect::<HashMap<_, _>>();
    let nodes = snapshot
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<HashMap<_, _>>();
    let stale_nodes: HashSet<&str> = snapshot
        .freshness
        .as_ref()
        .map(|freshness| {
            freshness
                .stale_node_ids
                .iter()
                .map(String::as_str)
                .collect()
        })
        .unwrap_or_default();
    let stale_edges: HashSet<&str> = snapshot
        .freshness
        .as_ref()
        .map(|freshness| {
            freshness
                .stale_edge_ids
                .iter()
                .map(String::as_str)
                .collect()
        })
        .unwrap_or_default();
    let stale_summaries: HashSet<&str> = snapshot
        .freshness
        .as_ref()
        .map(|freshness| {
            freshness
                .stale_summary_ids
                .iter()
                .map(String::as_str)
                .collect()
        })
        .unwrap_or_default();
    let mut out = Vec::new();

    for node in &snapshot.nodes {
        if excluded_lifecycle(node.lifecycle) || stale_nodes.contains(node.id.as_str()) {
            continue;
        }
        let domain = required_domain(&domains, &node.domain_id)?;
        out.push(document(
            swarm_id,
            snapshot,
            partition(node.profile),
            SemanticDocumentKind::Node,
            &node.id,
            domain,
            node.profile,
            SemanticLifecycle::Memory(node.lifecycle),
            node.anchors
                .iter()
                .map(|anchor| anchor.target.clone())
                .collect(),
            node.source_hashes.clone(),
            node,
            join_text([
                &node.title,
                &node.claim,
                &node.question,
                &node.tension,
                &node.action_implication,
            ]),
        )?);
    }

    for edge in &snapshot.edges {
        if excluded_lifecycle(edge.lifecycle) || stale_edges.contains(edge.id.as_str()) {
            continue;
        }
        let source = nodes
            .get(edge.source_id.as_str())
            .ok_or_else(|| anyhow!("validated edge lost source"))?;
        let target = nodes
            .get(edge.target_id.as_str())
            .ok_or_else(|| anyhow!("validated edge lost target"))?;
        if excluded_lifecycle(source.lifecycle)
            || excluded_lifecycle(target.lifecycle)
            || stale_nodes.contains(source.id.as_str())
            || stale_nodes.contains(target.id.as_str())
        {
            continue;
        }
        let domain = required_domain(&domains, &source.domain_id)?;
        let source_hashes = edge
            .anchors
            .iter()
            .filter_map(|anchor| anchor.source_hash.clone())
            .collect();
        out.push(document(
            swarm_id,
            snapshot,
            partition(edge.profile),
            SemanticDocumentKind::Edge,
            &edge.id,
            domain,
            edge.profile,
            SemanticLifecycle::Memory(edge.lifecycle),
            edge.anchors
                .iter()
                .map(|anchor| anchor.target.clone())
                .collect(),
            source_hashes,
            edge,
            join_text([&edge.claim, &edge.source_id, &edge.target_id]),
        )?);
    }

    for summary in &snapshot.summaries {
        if summary.freshness != EpiphanyMemoryFreshnessStatus::Ready
            || summary.confidence < 70
            || !summary.known_omissions.is_empty()
            || stale_summaries.contains(summary.id.as_str())
        {
            continue;
        }
        let domain = required_domain(&domains, &summary.domain_id)?;
        out.push(document(
            swarm_id,
            snapshot,
            partition(domain.profile),
            SemanticDocumentKind::Summary,
            &summary.id,
            domain,
            domain.profile,
            SemanticLifecycle::Derived,
            summary
                .covers_node_ids
                .iter()
                .chain(&summary.covers_edge_ids)
                .cloned()
                .collect(),
            summary.source_hashes.clone(),
            summary,
            join_text([
                &summary.target,
                &summary.claim,
                &summary.question,
                &summary.tension,
                &summary.action_implication,
            ]),
        )?);
    }

    for item in &snapshot.frontier {
        if !matches!(
            item.status,
            RepoFrontierStatus::Proposed | RepoFrontierStatus::Active | RepoFrontierStatus::Blocked
        ) {
            continue;
        }
        let domain = item
            .target_claim_ids
            .first()
            .and_then(|id| nodes.get(id.as_str()))
            .and_then(|node| domains.get(node.domain_id.as_str()))
            .copied()
            .ok_or_else(|| anyhow!("validated unresolved frontier lost its target domain"))?;
        out.push(document(
            swarm_id,
            snapshot,
            SemanticPartition::Modeling,
            SemanticDocumentKind::Frontier,
            &item.id,
            domain,
            EpiphanyMemoryProfile::RepoArchitecture,
            SemanticLifecycle::Frontier(item.status),
            item.source_scope
                .iter()
                .chain(&item.evidence_refs)
                .cloned()
                .collect(),
            Vec::new(),
            item,
            join_text([
                &item.migration_body,
                &item.question,
                &item.gap,
                &item.recommended_next_organ,
            ]),
        )?);
    }

    out.sort_by(|left, right| left.point_id.cmp(&right.point_id));
    Ok(out)
}

pub fn resolve_semantic_candidate<'a>(
    expected_partition: SemanticPartition,
    candidate: &SemanticProjectionCandidate,
    current_documents: &'a [SemanticProjectionDocument],
) -> Result<&'a SemanticProjectionDocument> {
    if candidate.partition != expected_partition {
        return Err(anyhow!("semantic candidate partition mismatch"));
    }
    let document = current_documents
        .iter()
        .find(|document| document.point_id == candidate.point_id)
        .ok_or_else(|| anyhow!("semantic candidate is missing or no longer live"))?;
    if document.partition != candidate.partition
        || document.canonical != candidate.canonical
        || document.model_revision != candidate.indexed_model_revision
        || document.model_hash != candidate.indexed_model_hash
        || document.canonical_content_hash != candidate.indexed_canonical_content_hash
    {
        return Err(anyhow!(
            "semantic candidate no longer matches canonical state"
        ));
    }
    Ok(document)
}

fn document<T: Serialize>(
    swarm_id: &str,
    snapshot: &EpiphanyMemoryGraphSnapshot,
    partition: SemanticPartition,
    kind: SemanticDocumentKind,
    document_id: &str,
    domain: &EpiphanyMemoryDomain,
    profile: EpiphanyMemoryProfile,
    lifecycle: SemanticLifecycle,
    mut source_refs: Vec<String>,
    mut source_hashes: Vec<String>,
    canonical_value: &T,
    projection_text: String,
) -> Result<SemanticProjectionDocument> {
    source_refs.sort();
    source_refs.dedup();
    source_hashes.sort();
    source_hashes.dedup();
    let canonical = SemanticCanonicalLocator {
        locator: format!(
            "cultcache://{swarm_id}/{MEMORY_GRAPH_TYPE}/{MEMORY_GRAPH_KEY}#{document_id}"
        ),
        canonical_type: MEMORY_GRAPH_TYPE.to_string(),
        canonical_key: MEMORY_GRAPH_KEY.to_string(),
        canonical_document_id: document_id.to_string(),
    };
    Ok(SemanticProjectionDocument {
        point_id: semantic_point_id(
            swarm_id,
            partition,
            MEMORY_GRAPH_TYPE,
            MEMORY_GRAPH_KEY,
            document_id,
        ),
        swarm_id: swarm_id.to_string(),
        partition,
        kind,
        canonical,
        graph_id: snapshot.graph_id.clone(),
        model_revision: snapshot.model_revision,
        model_hash: snapshot.model_hash.clone(),
        domain_id: domain.id.clone(),
        profile,
        lifecycle,
        visibility: SemanticVisibility::PrivateVerse,
        source_refs,
        source_hashes,
        canonical_content_hash: format!(
            "{:x}",
            Sha256::digest(rmp_serde::to_vec_named(canonical_value)?)
        ),
        canonical_schema_version: snapshot
            .schema_version
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        projection_schema_version: SEMANTIC_PROJECTION_SCHEMA_VERSION.to_string(),
        projection_text,
    })
}

fn partition(profile: EpiphanyMemoryProfile) -> SemanticPartition {
    match profile {
        EpiphanyMemoryProfile::RepoArchitecture | EpiphanyMemoryProfile::RepoDataflow => {
            SemanticPartition::Modeling
        }
        _ => SemanticPartition::Mind,
    }
}

fn excluded_lifecycle(lifecycle: EpiphanyMemoryLifecycle) -> bool {
    matches!(
        lifecycle,
        EpiphanyMemoryLifecycle::Retired | EpiphanyMemoryLifecycle::Stale
    )
}

fn required_domain<'a>(
    domains: &HashMap<&str, &'a EpiphanyMemoryDomain>,
    id: &str,
) -> Result<&'a EpiphanyMemoryDomain> {
    domains
        .get(id)
        .copied()
        .ok_or_else(|| anyhow!("validated document lost domain"))
}

fn join_text<const N: usize>(parts: [&str; N]) -> String {
    parts
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_graph::{
        EpiphanyMemoryEdge, EpiphanyMemoryNode, EpiphanyMemorySummary, RepoFrontierItem,
    };
    use epiphany_state_model::{EpiphanyMemoryEdgeKind, EpiphanyMemoryNodeKind};

    fn snapshot() -> EpiphanyMemoryGraphSnapshot {
        let mut snapshot = EpiphanyMemoryGraphSnapshot {
            schema_version: Some("v0".into()),
            graph_id: "graph".into(),
            model_revision: 7,
            domains: vec![
                EpiphanyMemoryDomain {
                    id: "repo".into(),
                    profile: EpiphanyMemoryProfile::RepoArchitecture,
                    title: "Repo".into(),
                    lifecycle: EpiphanyMemoryLifecycle::Accepted,
                    ..Default::default()
                },
                EpiphanyMemoryDomain {
                    id: "mind".into(),
                    profile: EpiphanyMemoryProfile::Evidence,
                    title: "Mind".into(),
                    lifecycle: EpiphanyMemoryLifecycle::Accepted,
                    ..Default::default()
                },
            ],
            nodes: vec![
                EpiphanyMemoryNode {
                    id: "repo-node".into(),
                    domain_id: "repo".into(),
                    profile: EpiphanyMemoryProfile::RepoArchitecture,
                    kind: EpiphanyMemoryNodeKind::Module,
                    title: "Repo node".into(),
                    claim: "owns map".into(),
                    tension: "map can drift".into(),
                    action_implication: "keep typed".into(),
                    source_hashes: vec!["anchor:missing".into()],
                    lifecycle: EpiphanyMemoryLifecycle::Accepted,
                    confidence: 90,
                    ..Default::default()
                },
                EpiphanyMemoryNode {
                    id: "mind-node".into(),
                    domain_id: "mind".into(),
                    profile: EpiphanyMemoryProfile::Evidence,
                    kind: EpiphanyMemoryNodeKind::Evidence,
                    title: "Lesson".into(),
                    claim: "remember it".into(),
                    tension: "memory can drift".into(),
                    action_implication: "rehydrate".into(),
                    source_hashes: vec!["anchor:missing".into()],
                    lifecycle: EpiphanyMemoryLifecycle::Accepted,
                    confidence: 90,
                    ..Default::default()
                },
                EpiphanyMemoryNode {
                    id: "retired".into(),
                    domain_id: "repo".into(),
                    profile: EpiphanyMemoryProfile::RepoArchitecture,
                    kind: EpiphanyMemoryNodeKind::Module,
                    title: "Old".into(),
                    claim: "old".into(),
                    tension: "obsolete".into(),
                    action_implication: "none".into(),
                    source_hashes: vec!["anchor:missing".into()],
                    lifecycle: EpiphanyMemoryLifecycle::Retired,
                    confidence: 90,
                    ..Default::default()
                },
            ],
            edges: vec![EpiphanyMemoryEdge {
                id: "edge".into(),
                source_id: "repo-node".into(),
                target_id: "mind-node".into(),
                kind: EpiphanyMemoryEdgeKind::Grounds,
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                claim: "grounds".into(),
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                confidence: 80,
                ..Default::default()
            }],
            summaries: vec![EpiphanyMemorySummary {
                id: "summary".into(),
                domain_id: "mind".into(),
                covers_node_ids: vec!["mind-node".into()],
                target: "lesson".into(),
                claim: "durable".into(),
                tension: "retrieval can drift".into(),
                action_implication: "retrieve".into(),
                freshness: EpiphanyMemoryFreshnessStatus::Ready,
                confidence: 80,
                ..Default::default()
            }],
            frontier: vec![RepoFrontierItem {
                id: "frontier".into(),
                migration_body: "finish projection".into(),
                gap: "not wired".into(),
                target_claim_ids: vec!["repo-node".into()],
                recommended_next_organ: "Hands".into(),
                status: RepoFrontierStatus::Active,
                ..Default::default()
            }],
            ..Default::default()
        };
        snapshot.model_hash = memory_graph_model_hash(&snapshot).unwrap();
        snapshot
    }

    #[test]
    fn derives_typed_partitions_and_filters_non_live_documents() {
        let documents = derive_semantic_projection("swarm-a", &snapshot()).unwrap();
        assert_eq!(documents.len(), 5);
        assert!(
            documents
                .iter()
                .any(
                    |document| document.canonical.canonical_document_id == "mind-node"
                        && document.partition == SemanticPartition::Mind
                )
        );
        assert!(
            documents
                .iter()
                .any(|document| document.kind == SemanticDocumentKind::Frontier
                    && document.partition == SemanticPartition::Modeling)
        );
        assert!(
            !documents
                .iter()
                .any(|document| document.canonical.canonical_document_id == "retired")
        );
        let first = derive_semantic_projection("swarm-a", &snapshot()).unwrap();
        assert_eq!(documents, first);
        let other_swarm = derive_semantic_projection("swarm-b", &snapshot()).unwrap();
        assert!(documents.iter().zip(&other_swarm).all(|(left, right)| {
            left.swarm_id != right.swarm_id && left.point_id != right.point_id
        }));
    }

    #[test]
    fn candidate_resolution_revalidates_canonical_identity_and_hashes() {
        let documents = derive_semantic_projection("swarm-a", &snapshot()).unwrap();
        let document = documents
            .iter()
            .find(|document| document.canonical.canonical_document_id == "repo-node")
            .unwrap();
        let candidate = SemanticProjectionCandidate {
            point_id: document.point_id.clone(),
            canonical: document.canonical.clone(),
            partition: document.partition,
            score: 0.9,
            indexed_model_revision: document.model_revision,
            indexed_model_hash: document.model_hash.clone(),
            indexed_canonical_content_hash: document.canonical_content_hash.clone(),
        };
        assert_eq!(
            resolve_semantic_candidate(SemanticPartition::Modeling, &candidate, &documents)
                .unwrap()
                .point_id,
            document.point_id
        );
        let mut stale = candidate.clone();
        stale.indexed_canonical_content_hash = "stale".into();
        assert!(
            resolve_semantic_candidate(SemanticPartition::Modeling, &stale, &documents).is_err()
        );
        assert!(
            resolve_semantic_candidate(SemanticPartition::Mind, &candidate, &documents).is_err()
        );
        let mut stale_revision = candidate.clone();
        stale_revision.indexed_model_revision -= 1;
        assert!(
            resolve_semantic_candidate(SemanticPartition::Modeling, &stale_revision, &documents)
                .is_err()
        );
        let mut stale_model = candidate.clone();
        stale_model.indexed_model_hash = "old-model".into();
        assert!(
            resolve_semantic_candidate(SemanticPartition::Modeling, &stale_model, &documents)
                .is_err()
        );
        let mut wrong_locator = candidate.clone();
        wrong_locator.canonical.canonical_document_id = "other".into();
        assert!(
            resolve_semantic_candidate(SemanticPartition::Modeling, &wrong_locator, &documents)
                .is_err()
        );
        assert!(resolve_semantic_candidate(SemanticPartition::Modeling, &candidate, &[]).is_err());
    }

    #[test]
    fn ranked_edge_cannot_reintroduce_stale_endpoint_and_summary_is_not_duplicated() {
        let mut snapshot = snapshot();
        snapshot.nodes[0].lifecycle = EpiphanyMemoryLifecycle::Stale;
        snapshot.summaries[0].freshness = EpiphanyMemoryFreshnessStatus::Ready;
        snapshot.freshness = Some(crate::memory_graph::derive_memory_graph_freshness(
            &snapshot,
            &[],
        ));
        snapshot.model_hash.clear();
        snapshot.model_hash = memory_graph_model_hash(&snapshot).unwrap();
        let packet = crate::memory_graph::plan_memory_graph_context_cut_with_ranked_ids(
            &snapshot,
            &crate::memory_graph::EpiphanyMemoryContextQuery {
                id: "ranked-hostile".to_string(),
                text: Some("durable lesson".to_string()),
                ..Default::default()
            },
            &["edge".to_string(), "summary".to_string()],
        );
        assert!(!packet.nodes.iter().any(|node| node.id == "repo-node"));
        assert_eq!(
            packet
                .summaries
                .iter()
                .filter(|summary| summary.id == "summary")
                .count(),
            1
        );
    }

    #[test]
    fn stale_summary_descent_never_emits_stale_children() {
        let mut snapshot = snapshot();
        snapshot.nodes[0].lifecycle = EpiphanyMemoryLifecycle::Stale;
        snapshot.summaries[0].covers_node_ids = vec!["repo-node".to_string()];
        snapshot.summaries[0].freshness = EpiphanyMemoryFreshnessStatus::Stale;
        snapshot.freshness = Some(crate::memory_graph::derive_memory_graph_freshness(
            &snapshot,
            &[],
        ));
        let packet = crate::memory_graph::plan_memory_graph_context_cut(
            &snapshot,
            &crate::memory_graph::EpiphanyMemoryContextQuery {
                id: "stale-summary".to_string(),
                text: Some("durable".to_string()),
                ..Default::default()
            },
        );
        assert!(!packet.nodes.iter().any(|node| node.id == "repo-node"));
        assert!(packet.summaries.is_empty());
    }

    #[test]
    fn hostile_composed_graph_cannot_cross_partition_or_domain_through_any_cut_path() {
        let mut snapshot = snapshot();
        snapshot.frontier.push(RepoFrontierItem {
            id: "mind-frontier".into(),
            migration_body: "tempt cross-partition routing".into(),
            gap: "must remain private to Mind".into(),
            target_claim_ids: vec!["mind-node".into()],
            recommended_next_organ: "Self".into(),
            status: RepoFrontierStatus::Active,
            ..Default::default()
        });
        let modeling = crate::memory_graph::plan_memory_graph_context_cut_for_partition(
            &snapshot,
            &crate::memory_graph::EpiphanyMemoryContextQuery {
                id: "hostile-modeling".into(),
                domain_ids: vec!["repo".into()],
                node_ids: vec!["mind-node".into()],
                edge_ids: vec!["edge".into()],
                text: Some("remember durable lesson".into()),
                budget: Some(64),
                ..Default::default()
            },
            &["mind-node".into(), "edge".into(), "summary".into()],
            SemanticPartition::Modeling,
        );
        assert!(modeling.nodes.iter().all(|node| {
            node.domain_id == "repo"
                && matches!(
                    node.profile,
                    EpiphanyMemoryProfile::RepoArchitecture | EpiphanyMemoryProfile::RepoDataflow
                )
        }));
        assert!(modeling.edges.is_empty());
        assert!(modeling.summaries.is_empty());
        assert_eq!(
            modeling
                .frontier
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["frontier"]
        );

        let mind = crate::memory_graph::plan_memory_graph_context_cut_for_partition(
            &snapshot,
            &crate::memory_graph::EpiphanyMemoryContextQuery {
                id: "hostile-mind".into(),
                domain_ids: vec!["mind".into()],
                node_ids: vec!["repo-node".into()],
                edge_ids: vec!["edge".into()],
                text: Some("owns map".into()),
                budget: Some(64),
                ..Default::default()
            },
            &["repo-node".into(), "edge".into(), "frontier".into()],
            SemanticPartition::Mind,
        );
        assert!(mind.nodes.iter().all(|node| {
            node.domain_id == "mind"
                && !matches!(
                    node.profile,
                    EpiphanyMemoryProfile::RepoArchitecture | EpiphanyMemoryProfile::RepoDataflow
                )
        }));
        assert!(mind.edges.is_empty());
        assert!(mind.frontier.is_empty());
    }
}
