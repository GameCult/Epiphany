use super::EpiphanyMemoryEdge;
use super::EpiphanyMemoryEmbeddingManifest;
use super::EpiphanyMemoryGraphSnapshot;
use super::EpiphanyMemoryLifecycle;
use super::EpiphanyMemoryNode;
use super::EpiphanyMemoryProfile;
use super::EpiphanyMemorySummary;
use super::unique_strings;
use serde::Deserialize;
use serde::Serialize;
use sha1::Digest;
use sha1::Sha1;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyMemoryEmbeddingDocument {
    pub id: String,
    pub source_id: String,
    pub document_kind: EpiphanyMemoryEmbeddingDocumentKind,
    pub profile: EpiphanyMemoryProfile,
    pub lifecycle: EpiphanyMemoryLifecycle,
    pub text: String,
    pub source_hashes: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpiphanyMemoryEmbeddingDocumentKind {
    Node,
    Edge,
    Summary,
}

pub fn memory_graph_embedding_documents(
    snapshot: &EpiphanyMemoryGraphSnapshot,
) -> Vec<EpiphanyMemoryEmbeddingDocument> {
    let mut documents = Vec::new();
    let domain_profiles = snapshot
        .domains
        .iter()
        .map(|domain| (domain.id.as_str(), domain.profile))
        .collect::<HashMap<_, _>>();
    documents.extend(snapshot.nodes.iter().map(node_embedding_document));
    documents.extend(snapshot.edges.iter().map(edge_embedding_document));
    documents.extend(
        snapshot
            .summaries
            .iter()
            .map(|summary| summary_embedding_document(summary, &domain_profiles)),
    );
    documents
}

pub fn memory_graph_embedding_manifest(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    collection_name: impl Into<String>,
    embedding_model: impl Into<String>,
    vector_dimensions: Option<u32>,
) -> EpiphanyMemoryEmbeddingManifest {
    let documents = memory_graph_embedding_documents(snapshot);
    EpiphanyMemoryEmbeddingManifest {
        id: memory_graph_embedding_manifest_id(&snapshot.graph_id),
        collection_name: Some(collection_name.into()),
        embedding_model: embedding_model.into(),
        vector_dimensions,
        indexed_document_ids: documents
            .iter()
            .map(|document| document.id.clone())
            .collect(),
        stale_document_ids: Vec::new(),
        source_hashes: unique_strings(
            documents
                .iter()
                .flat_map(|document| document.source_hashes.iter().cloned()),
        ),
    }
}

fn node_embedding_document(node: &EpiphanyMemoryNode) -> EpiphanyMemoryEmbeddingDocument {
    EpiphanyMemoryEmbeddingDocument {
        id: embedding_document_id("node", &node.id),
        source_id: node.id.clone(),
        document_kind: EpiphanyMemoryEmbeddingDocumentKind::Node,
        profile: node.profile,
        lifecycle: node.lifecycle,
        text: join_text_parts([
            node.title.as_str(),
            node.claim.as_str(),
            node.question.as_str(),
            node.tension.as_str(),
            node.action_implication.as_str(),
        ]),
        source_hashes: node.source_hashes.clone(),
    }
}

fn edge_embedding_document(edge: &EpiphanyMemoryEdge) -> EpiphanyMemoryEmbeddingDocument {
    EpiphanyMemoryEmbeddingDocument {
        id: embedding_document_id("edge", &edge.id),
        source_id: edge.id.clone(),
        document_kind: EpiphanyMemoryEmbeddingDocumentKind::Edge,
        profile: edge.profile,
        lifecycle: edge.lifecycle,
        text: join_text_parts(vec![
            format!("{:?}", edge.kind),
            edge.claim.clone(),
            edge.source_id.clone(),
            edge.target_id.clone(),
        ]),
        source_hashes: unique_strings(
            edge.anchors
                .iter()
                .filter_map(|anchor| anchor.source_hash.clone()),
        ),
    }
}

fn summary_embedding_document(
    summary: &EpiphanyMemorySummary,
    domain_profiles: &HashMap<&str, EpiphanyMemoryProfile>,
) -> EpiphanyMemoryEmbeddingDocument {
    EpiphanyMemoryEmbeddingDocument {
        id: embedding_document_id("summary", &summary.id),
        source_id: summary.id.clone(),
        document_kind: EpiphanyMemoryEmbeddingDocumentKind::Summary,
        profile: *domain_profiles
            .get(summary.domain_id.as_str())
            .unwrap_or(&EpiphanyMemoryProfile::Evidence),
        lifecycle: EpiphanyMemoryLifecycle::Observed,
        text: join_text_parts(vec![
            summary.target.clone(),
            summary.claim.clone(),
            summary.question.clone(),
            summary.tension.clone(),
            summary.action_implication.clone(),
            summary.known_omissions.join(" "),
        ]),
        source_hashes: summary.source_hashes.clone(),
    }
}

fn memory_graph_embedding_manifest_id(graph_id: &str) -> String {
    format!("memembed-manifest-{}", short_hash(graph_id))
}

fn embedding_document_id(kind: &str, source_id: &str) -> String {
    format!("memembed-{kind}-{}", short_hash(source_id))
}

fn short_hash(value: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    hex_lower(&digest[..10])
}

fn join_text_parts(parts: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    parts
        .into_iter()
        .map(|part| part.as_ref().trim().to_string())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
