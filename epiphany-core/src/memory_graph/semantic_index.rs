use super::{
    EpiphanyMemoryContextPacket, EpiphanyMemoryContextQuery, EpiphanyMemoryGraphSnapshot,
    SEMANTIC_PROJECTION_SCHEMA_VERSION, SemanticPartition, SemanticProjectionCandidate,
    SemanticProjectionDocument, derive_semantic_projection,
    plan_memory_graph_context_cut_for_partition, resolve_semantic_candidate,
};
use crate::semantic_backend::{
    CollectionCompatibility, OllamaConfig, OllamaEmbedder, QdrantBackend, QdrantConfig,
    SemanticPoint,
};
use anyhow::{Result, anyhow};
use cultcache_rs::{CacheBackingStore, DatabaseEntry, SingleFileMessagePackBackingStore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::env;
use std::path::Path;
use uuid::Uuid;

pub const MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION: &str =
    "gamecult.epiphany.memory_semantic_index_receipt.v2";
pub const MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION: &str =
    "gamecult.epiphany.memory_semantic_projection_obligation.v0";
pub const MEMORY_SEMANTIC_PROJECTION_ATTEMPT_SCHEMA_VERSION: &str =
    "gamecult.epiphany.memory_semantic_projection_attempt.v1";
const MODELING_COLLECTION_DEFAULT: &str = "epiphany_modeling_v1";
const MIND_COLLECTION_DEFAULT: &str = "epiphany_mind_v1";
const QUERY_LIMIT_MAX: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemorySemanticIndexConfig {
    pub qdrant_url: String,
    pub qdrant_api_key: Option<String>,
    pub qdrant_timeout_ms: u64,
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub ollama_timeout_ms: u64,
    pub embedding_provider_id: String,
    pub modeling_collection: String,
    pub mind_collection: String,
    pub modeling_query_instruction: String,
    pub mind_query_instruction: String,
}

impl MemorySemanticIndexConfig {
    pub fn from_env() -> Self {
        Self {
            qdrant_url: env_value("EPIPHANY_QDRANT_URL", "http://127.0.0.1:6333"),
            qdrant_api_key: env::var("EPIPHANY_QDRANT_API_KEY")
                .ok()
                .filter(|value| !value.trim().is_empty()),
            qdrant_timeout_ms: env_u64("EPIPHANY_QDRANT_TIMEOUT_MS", 30_000),
            ollama_base_url: env_value("EPIPHANY_OLLAMA_BASE_URL", "http://127.0.0.1:11434"),
            ollama_model: env_value("EPIPHANY_OLLAMA_MODEL", "qwen3-embedding:0.6b"),
            ollama_timeout_ms: env_u64("EPIPHANY_OLLAMA_TIMEOUT_MS", 30_000),
            embedding_provider_id: env_value(
                "EPIPHANY_EMBEDDING_PROVIDER_ID",
                "gamecult-ollama-embedding",
            ),
            modeling_collection: env_value(
                "EPIPHANY_MODELING_QDRANT_COLLECTION",
                MODELING_COLLECTION_DEFAULT,
            ),
            mind_collection: env_value(
                "EPIPHANY_MIND_QDRANT_COLLECTION",
                MIND_COLLECTION_DEFAULT,
            ),
            modeling_query_instruction: "Given a repository architecture, dataflow, ownership, invariant, or frontier question, rank the canonical Modeling documents that answer it.".to_string(),
            mind_query_instruction: "Given a swarm memory, doctrine, evidence, decision, relationship, or rehydration question, rank the canonical Mind documents that answer it.".to_string(),
        }
    }

    fn collection(&self, partition: SemanticPartition) -> &str {
        match partition {
            SemanticPartition::Mind => &self.mind_collection,
            SemanticPartition::Modeling => &self.modeling_collection,
        }
    }

    fn query_instruction(&self, partition: SemanticPartition) -> &str {
        match partition {
            SemanticPartition::Mind => &self.mind_query_instruction,
            SemanticPartition::Modeling => &self.modeling_query_instruction,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct MemorySemanticPointPayload {
    point_id: String,
    swarm_id: String,
    partition: SemanticPartition,
    obligation_id: String,
    claim_id: String,
    claim_epoch: String,
    canonical_locator: String,
    canonical_type: String,
    canonical_key: String,
    canonical_document_id: String,
    canonical_schema_version: String,
    graph_id: String,
    indexed_model_revision: u64,
    indexed_model_hash: String,
    indexed_canonical_content_hash: String,
    projection_schema_version: String,
    embedding_provider_id: String,
    embedding_model: String,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.memory_semantic_projection_obligation",
    schema = "MemorySemanticProjectionObligation"
)]
pub struct MemorySemanticProjectionObligation {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub obligation_id: String,
    #[cultcache(key = 2)]
    pub swarm_id: String,
    #[cultcache(key = 3)]
    pub partition: String,
    #[cultcache(key = 4)]
    pub canonical_source_id: String,
    #[cultcache(key = 5)]
    pub source_commit_id: String,
    #[cultcache(key = 6)]
    pub graph_id: String,
    #[cultcache(key = 7)]
    pub source_generation: u64,
    #[cultcache(key = 8)]
    pub source_model_hash: String,
    #[cultcache(key = 9)]
    pub canonical_content_set_hash: String,
    #[cultcache(key = 10)]
    pub projection_schema_version: String,
    #[cultcache(key = 11)]
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.memory_semantic_projection_attempt",
    schema = "MemorySemanticProjectionAttempt"
)]
pub struct MemorySemanticProjectionAttempt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub attempt_id: String,
    #[cultcache(key = 2)]
    pub obligation_id: String,
    #[cultcache(key = 3)]
    pub started_at: String,
    #[cultcache(key = 4)]
    pub completed_at: Option<String>,
    #[cultcache(key = 5)]
    pub status: String,
    #[cultcache(key = 6)]
    pub error: Option<String>,
    #[cultcache(key = 7)]
    pub claim_id: String,
    #[cultcache(key = 8)]
    pub claim_epoch: u64,
    #[cultcache(key = 9)]
    pub executor_id: String,
    #[cultcache(key = 10)]
    pub executor_incarnation: String,
    #[cultcache(key = 11)]
    pub authority_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MemorySemanticProjectionHealthStatus {
    Pending,
    Failed,
    Stale,
    Ready,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemorySemanticProjectionSourceHead {
    pub swarm_id: String,
    pub partition: String,
    pub canonical_source_id: String,
    pub source_commit_id: String,
    pub graph_id: String,
    pub source_generation: u64,
    pub source_model_hash: String,
    pub canonical_content_set_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemorySemanticProjectionHealth {
    pub status: MemorySemanticProjectionHealthStatus,
    pub obligation_id: String,
    pub receipt_id: Option<String>,
    pub latest_attempt_id: Option<String>,
    pub latest_error: Option<String>,
    pub query_eligible: bool,
}

#[derive(Clone, Debug)]
pub struct MemorySemanticProjectionReadiness {
    pub(crate) obligation: MemorySemanticProjectionObligation,
    pub(crate) current: MemorySemanticProjectionSourceHead,
    pub(crate) receipt: MemorySemanticIndexReceipt,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MemorySemanticLiveEvidence {
    pub(crate) collection_name: String,
    pub(crate) observed_point_count: u32,
    pub(crate) observed_vector_binding_root_sha256: String,
}

pub(super) trait MemorySemanticEvidencePort {
    fn collection_exists(&mut self, name: &str) -> Result<bool>;
    fn collection_compatibility(&mut self, name: &str) -> Result<CollectionCompatibility>;
    fn points_for_scope(
        &mut self,
        name: &str,
        scope: &[(&str, &str)],
    ) -> Result<Vec<crate::semantic_backend::SemanticStoredPoint<MemorySemanticPointPayload>>>;
}

impl MemorySemanticEvidencePort for QdrantBackend {
    fn collection_exists(&mut self, name: &str) -> Result<bool> {
        QdrantBackend::collection_exists(self, name)
    }
    fn collection_compatibility(&mut self, name: &str) -> Result<CollectionCompatibility> {
        QdrantBackend::collection_compatibility(self, name)
    }
    fn points_for_scope(
        &mut self,
        name: &str,
        scope: &[(&str, &str)],
    ) -> Result<Vec<crate::semantic_backend::SemanticStoredPoint<MemorySemanticPointPayload>>> {
        QdrantBackend::points_for_scope(self, name, scope)
    }
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.memory_semantic_index_receipt",
    schema = "MemorySemanticIndexReceipt"
)]
pub struct MemorySemanticIndexReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub swarm_id: String,
    #[cultcache(key = 3)]
    pub partition: String,
    #[cultcache(key = 4)]
    pub collection_name: String,
    #[cultcache(key = 5)]
    pub graph_id: String,
    #[cultcache(key = 6)]
    pub model_revision: u64,
    #[cultcache(key = 7)]
    pub model_hash: String,
    #[cultcache(key = 8)]
    pub embedding_provider_id: String,
    #[cultcache(key = 9)]
    pub embedding_model: String,
    #[cultcache(key = 10)]
    pub vector_dimensions: u32,
    #[cultcache(key = 11)]
    pub indexed_document_count: u32,
    #[cultcache(key = 12)]
    pub deleted_document_count: u32,
    #[cultcache(key = 13)]
    pub canonical_content_set_hash: String,
    #[cultcache(key = 14)]
    pub indexed_at: String,
    #[cultcache(key = 15)]
    pub status: String,
    #[cultcache(key = 16)]
    pub obligation_id: String,
    #[cultcache(key = 17)]
    pub canonical_source_id: String,
    #[cultcache(key = 18)]
    pub source_commit_id: String,
    #[cultcache(key = 19)]
    pub source_generation: u64,
    #[cultcache(key = 20)]
    pub projection_schema_version: String,
    #[cultcache(key = 21, default)]
    pub claim_id: String,
    #[cultcache(key = 22, default)]
    pub claim_epoch: u64,
    #[cultcache(key = 23, default)]
    pub observed_vector_binding_root_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MemorySemanticProjectionNamespace {
    pub(super) obligation_id: String,
    pub(super) claim_id: String,
    pub(super) claim_epoch: u64,
}

pub fn validate_memory_semantic_projection_obligation(
    obligation: &MemorySemanticProjectionObligation,
) -> Result<()> {
    if obligation.schema_version != MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION {
        return Err(anyhow!("unsupported semantic projection obligation schema"));
    }
    for (label, value) in [
        ("obligation_id", obligation.obligation_id.as_str()),
        ("swarm_id", obligation.swarm_id.as_str()),
        ("partition", obligation.partition.as_str()),
        (
            "canonical_source_id",
            obligation.canonical_source_id.as_str(),
        ),
        ("source_commit_id", obligation.source_commit_id.as_str()),
        ("graph_id", obligation.graph_id.as_str()),
        ("source_model_hash", obligation.source_model_hash.as_str()),
        (
            "canonical_content_set_hash",
            obligation.canonical_content_set_hash.as_str(),
        ),
        ("created_at", obligation.created_at.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("semantic projection obligation missing {label}"));
        }
    }
    if !matches!(obligation.partition.as_str(), "mind" | "modeling") {
        return Err(anyhow!(
            "semantic projection obligation has invalid partition"
        ));
    }
    if obligation.projection_schema_version != SEMANTIC_PROJECTION_SCHEMA_VERSION {
        return Err(anyhow!("semantic projection obligation schema mismatch"));
    }
    Ok(())
}

pub fn derive_memory_semantic_projection_obligation(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    swarm_id: &str,
    partition: SemanticPartition,
    canonical_source_id: &str,
    source_commit_id: &str,
    created_at: &str,
) -> Result<MemorySemanticProjectionObligation> {
    for (label, value) in [
        ("swarm_id", swarm_id),
        ("canonical_source_id", canonical_source_id),
        ("source_commit_id", source_commit_id),
        ("created_at", created_at),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("semantic projection obligation missing {label}"));
        }
    }
    chrono::DateTime::parse_from_rfc3339(created_at)
        .map_err(|_| anyhow!("semantic projection obligation timestamp must be RFC3339"))?;
    let documents = derive_semantic_projection(swarm_id, snapshot)?
        .into_iter()
        .filter(|document| document.partition == partition)
        .collect::<Vec<_>>();
    let canonical_content_set_hash = canonical_content_set_hash(&documents);
    let partition = partition_name(partition).to_string();
    let identity = format!("{swarm_id}|{partition}|{canonical_source_id}|{source_commit_id}");
    let obligation = MemorySemanticProjectionObligation {
        schema_version: MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION.to_string(),
        obligation_id: format!(
            "memory-semantic-projection-{partition}-{:x}",
            Sha256::digest(identity.as_bytes())
        ),
        swarm_id: swarm_id.to_string(),
        partition,
        canonical_source_id: canonical_source_id.to_string(),
        source_commit_id: source_commit_id.to_string(),
        graph_id: snapshot.graph_id.clone(),
        source_generation: snapshot.model_revision,
        source_model_hash: crate::memory_graph_model_hash(snapshot)?,
        canonical_content_set_hash,
        projection_schema_version: SEMANTIC_PROJECTION_SCHEMA_VERSION.to_string(),
        created_at: created_at.to_string(),
    };
    validate_memory_semantic_projection_obligation(&obligation)?;
    Ok(obligation)
}

pub fn validate_memory_semantic_projection_attempt(
    attempt: &MemorySemanticProjectionAttempt,
) -> Result<()> {
    if attempt.schema_version != MEMORY_SEMANTIC_PROJECTION_ATTEMPT_SCHEMA_VERSION {
        return Err(anyhow!("unsupported semantic projection attempt schema"));
    }
    if attempt.attempt_id.trim().is_empty()
        || attempt.obligation_id.trim().is_empty()
        || attempt.claim_id.trim().is_empty()
        || attempt.claim_epoch == 0
        || attempt.executor_id.trim().is_empty()
        || attempt.executor_incarnation.trim().is_empty()
        || attempt.authority_id.trim().is_empty()
        || attempt.started_at.trim().is_empty()
        || chrono::DateTime::parse_from_rfc3339(&attempt.started_at).is_err()
        || attempt
            .completed_at
            .as_deref()
            .is_some_and(|completed_at| chrono::DateTime::parse_from_rfc3339(completed_at).is_err())
    {
        return Err(anyhow!(
            "semantic projection attempt is missing identity or time"
        ));
    }
    if attempt.completed_at.as_deref().is_some_and(|completed_at| {
        chrono::DateTime::parse_from_rfc3339(completed_at).ok()
            < chrono::DateTime::parse_from_rfc3339(&attempt.started_at).ok()
    }) {
        return Err(anyhow!(
            "semantic projection attempt completes before it starts"
        ));
    }
    match attempt.status.as_str() {
        "running" if attempt.completed_at.is_none() && attempt.error.is_none() => Ok(()),
        "failed" if attempt.completed_at.is_some() && attempt.error.is_some() => Ok(()),
        "succeeded" if attempt.completed_at.is_some() && attempt.error.is_none() => Ok(()),
        _ => Err(anyhow!(
            "semantic projection attempt status does not match its terminal fields"
        )),
    }
}

pub fn derive_memory_semantic_projection_health(
    obligation: &MemorySemanticProjectionObligation,
    current: &MemorySemanticProjectionSourceHead,
    attempts: &[MemorySemanticProjectionAttempt],
    receipts: &[MemorySemanticIndexReceipt],
) -> Result<MemorySemanticProjectionHealth> {
    validate_memory_semantic_projection_obligation(obligation)?;
    for attempt in attempts
        .iter()
        .filter(|attempt| attempt.obligation_id == obligation.obligation_id)
    {
        validate_memory_semantic_projection_attempt(attempt)?;
    }
    let stale = !obligation_matches_source(obligation, current);
    let mut matching_receipts = receipts
        .iter()
        .filter(|receipt| receipt_matches_obligation(receipt, obligation))
        .collect::<Vec<_>>();
    matching_receipts.sort_by_key(|receipt| {
        chrono::DateTime::parse_from_rfc3339(&receipt.indexed_at)
            .expect("matching receipt has valid RFC3339 time")
    });
    let receipt = matching_receipts.last().copied();
    let mut matching_attempts = attempts
        .iter()
        .filter(|attempt| attempt.obligation_id == obligation.obligation_id)
        .collect::<Vec<_>>();
    matching_attempts.sort_by_key(|attempt| {
        chrono::DateTime::parse_from_rfc3339(
            attempt
                .completed_at
                .as_deref()
                .unwrap_or(attempt.started_at.as_str()),
        )
        .expect("validated attempt has RFC3339 ordering time")
    });
    let latest_attempt = matching_attempts.last().copied();
    let repair_after_receipt = latest_attempt
        .zip(receipt)
        .is_some_and(|(attempt, receipt)| {
            chrono::DateTime::parse_from_rfc3339(&attempt.started_at).ok()
                > chrono::DateTime::parse_from_rfc3339(&receipt.indexed_at).ok()
                && attempt.status != "succeeded"
        });
    let status = if stale {
        MemorySemanticProjectionHealthStatus::Stale
    } else if repair_after_receipt
        && latest_attempt.is_some_and(|attempt| attempt.status == "failed")
    {
        MemorySemanticProjectionHealthStatus::Failed
    } else if repair_after_receipt {
        MemorySemanticProjectionHealthStatus::Pending
    } else if receipt.is_some() {
        MemorySemanticProjectionHealthStatus::Ready
    } else if latest_attempt.is_some_and(|attempt| attempt.status == "failed") {
        MemorySemanticProjectionHealthStatus::Failed
    } else {
        MemorySemanticProjectionHealthStatus::Pending
    };
    Ok(MemorySemanticProjectionHealth {
        status,
        obligation_id: obligation.obligation_id.clone(),
        receipt_id: receipt.map(|receipt| receipt.receipt_id.clone()),
        latest_attempt_id: latest_attempt.map(|attempt| attempt.attempt_id.clone()),
        latest_error: latest_attempt.and_then(|attempt| attempt.error.clone()),
        query_eligible: status == MemorySemanticProjectionHealthStatus::Ready
            && receipt.is_some_and(|receipt| {
                memory_semantic_projection_query_eligible(obligation, current, receipt)
            }),
    })
}

pub fn memory_semantic_projection_query_eligible(
    obligation: &MemorySemanticProjectionObligation,
    current: &MemorySemanticProjectionSourceHead,
    receipt: &MemorySemanticIndexReceipt,
) -> bool {
    validate_memory_semantic_projection_obligation(obligation).is_ok()
        && obligation_matches_source(obligation, current)
        && receipt_matches_obligation(receipt, obligation)
        && receipt.vector_dimensions > 0
        && receipt.indexed_document_count > 0
}

pub(super) fn memory_semantic_projection_terminal_success(
    obligation: &MemorySemanticProjectionObligation,
    current: &MemorySemanticProjectionSourceHead,
    receipt: &MemorySemanticIndexReceipt,
) -> bool {
    validate_memory_semantic_projection_obligation(obligation).is_ok()
        && obligation_matches_source(obligation, current)
        && receipt_matches_obligation(receipt, obligation)
        && ((receipt.indexed_document_count > 0 && receipt.vector_dimensions > 0)
            || (receipt.indexed_document_count == 0
                && receipt.vector_dimensions == 0
                && obligation.canonical_content_set_hash == canonical_content_set_hash(&[])
                && receipt.observed_vector_binding_root_sha256
                    == format!("{:x}", Sha256::digest([]))))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub fn bind_memory_semantic_index_receipt(
    mut receipt: MemorySemanticIndexReceipt,
    obligation: &MemorySemanticProjectionObligation,
) -> Result<MemorySemanticIndexReceipt> {
    validate_memory_semantic_projection_obligation(obligation)?;
    if receipt.schema_version != MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION
        || !is_sha256(&receipt.observed_vector_binding_root_sha256)
        || receipt.swarm_id != obligation.swarm_id
        || receipt.partition != obligation.partition
        || receipt.graph_id != obligation.graph_id
        || receipt.model_revision != obligation.source_generation
        || receipt.model_hash != obligation.source_model_hash
        || receipt.canonical_content_set_hash != obligation.canonical_content_set_hash
        || receipt.status != "ready"
    {
        return Err(anyhow!(
            "semantic index result does not match the exact projection obligation"
        ));
    }
    receipt.obligation_id = obligation.obligation_id.clone();
    receipt.canonical_source_id = obligation.canonical_source_id.clone();
    receipt.source_commit_id = obligation.source_commit_id.clone();
    receipt.source_generation = obligation.source_generation;
    receipt.projection_schema_version = obligation.projection_schema_version.clone();
    Ok(receipt)
}

fn obligation_matches_source(
    obligation: &MemorySemanticProjectionObligation,
    current: &MemorySemanticProjectionSourceHead,
) -> bool {
    obligation.swarm_id == current.swarm_id
        && obligation.partition == current.partition
        && obligation.canonical_source_id == current.canonical_source_id
        && obligation.source_commit_id == current.source_commit_id
        && obligation.graph_id == current.graph_id
        && obligation.source_generation == current.source_generation
        && obligation.source_model_hash == current.source_model_hash
        && obligation.canonical_content_set_hash == current.canonical_content_set_hash
}

fn receipt_matches_obligation(
    receipt: &MemorySemanticIndexReceipt,
    obligation: &MemorySemanticProjectionObligation,
) -> bool {
    receipt.schema_version == MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION
        && receipt.status == "ready"
        && receipt.obligation_id == obligation.obligation_id
        && receipt.swarm_id == obligation.swarm_id
        && receipt.partition == obligation.partition
        && receipt.canonical_source_id == obligation.canonical_source_id
        && receipt.source_commit_id == obligation.source_commit_id
        && receipt.graph_id == obligation.graph_id
        && receipt.source_generation == obligation.source_generation
        && receipt.model_revision == obligation.source_generation
        && receipt.model_hash == obligation.source_model_hash
        && receipt.canonical_content_set_hash == obligation.canonical_content_set_hash
        && receipt.projection_schema_version == obligation.projection_schema_version
        && !receipt.claim_id.trim().is_empty()
        && receipt.claim_epoch > 0
        && is_sha256(&receipt.observed_vector_binding_root_sha256)
        && chrono::DateTime::parse_from_rfc3339(&receipt.indexed_at).is_ok()
}

fn authenticate_observed_projection(
    observed: Vec<crate::semantic_backend::SemanticStoredPoint<MemorySemanticPointPayload>>,
    desired: &BTreeMap<String, MemorySemanticPointPayload>,
    vector_dimensions: usize,
) -> Result<String> {
    if observed.len() != desired.len() {
        return Err(anyhow!(
            "semantic projection live point count disagrees with canonical set"
        ));
    }
    let mut seen = HashSet::new();
    let mut vector_bindings = Vec::with_capacity(observed.len());
    for point in observed {
        if !seen.insert(point.id.clone()) {
            return Err(anyhow!(
                "semantic projection live scroll returned a duplicate point id"
            ));
        }
        let expected = desired.get(&point.id).ok_or_else(|| {
            anyhow!("semantic projection live scroll returned a foreign point id")
        })?;
        if point.payload.as_ref() != Some(expected) {
            return Err(anyhow!(
                "semantic projection live payload disagrees with canonical payload"
            ));
        }
        let vector = point
            .vector
            .ok_or_else(|| anyhow!("semantic projection live point omitted its vector"))?;
        if vector_dimensions == 0 || vector.len() != vector_dimensions {
            return Err(anyhow!(
                "semantic projection live vector dimensions disagree"
            ));
        }
        if vector.iter().any(|value| !value.is_finite()) {
            return Err(anyhow!(
                "semantic projection live vector contains a non-finite value"
            ));
        }
        let vector_sha256 = format!(
            "{:x}",
            Sha256::digest(
                vector
                    .iter()
                    .flat_map(|value| value.to_le_bytes())
                    .collect::<Vec<_>>()
            )
        );
        vector_bindings.push(format!("{}|{}", point.id, vector_sha256));
    }
    if seen.len() != desired.len() || desired.keys().any(|id| !seen.contains(id)) {
        return Err(anyhow!(
            "semantic projection live IDs disagree with canonical set"
        ));
    }
    vector_bindings.sort();
    Ok(format!(
        "{:x}",
        Sha256::digest(vector_bindings.join("\n").as_bytes())
    ))
}

pub(super) fn index_memory_semantic_partition(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    swarm_id: &str,
    partition: SemanticPartition,
    namespace: &MemorySemanticProjectionNamespace,
    indexed_at: &str,
    config: &MemorySemanticIndexConfig,
) -> Result<MemorySemanticIndexReceipt> {
    let documents = derive_semantic_projection(swarm_id, snapshot)?
        .into_iter()
        .filter(|document| document.partition == partition)
        .collect::<Vec<_>>();
    let backend = qdrant(config)?;
    let collection = config.collection(partition);
    if namespace.obligation_id.trim().is_empty()
        || namespace.claim_id.trim().is_empty()
        || namespace.claim_epoch == 0
    {
        return Err(anyhow!("semantic projection physical namespace is invalid"));
    }
    let claim_epoch = namespace.claim_epoch.to_string();
    let scope = [
        ("swarmId", swarm_id),
        ("partition", partition_name(partition)),
        ("obligationId", namespace.obligation_id.as_str()),
        ("claimId", namespace.claim_id.as_str()),
        ("claimEpoch", claim_epoch.as_str()),
    ];
    let content_set_hash = canonical_content_set_hash(&documents);
    let model_hash = crate::memory_graph_model_hash(snapshot)?;

    if documents.is_empty() {
        let (vector_size, deleted_document_count) = if backend.collection_exists(collection)? {
            let actual = backend.collection_compatibility(collection)?;
            let expected = compatibility(config, partition, actual.vector_size);
            if actual != expected {
                return Err(anyhow!(
                    "Qdrant collection {collection} is incompatible: actual {actual:?}, expected {expected:?}"
                ));
            }
            let existing_ids = backend.point_ids_for_scope(collection, &scope)?;
            backend.delete_points(collection, &existing_ids)?;
            let observed_ids = backend.point_ids_for_scope(collection, &scope)?;
            if !observed_ids.is_empty() {
                return Err(anyhow!(
                    "Qdrant scope synchronization failed: expected no points, observed {observed_ids:?}"
                ));
            }
            (actual.vector_size as u32, existing_ids.len() as u32)
        } else {
            // An empty projection has nothing to embed and does not justify creating
            // physical projection state. Absence already represents the empty set.
            (0, 0)
        };
        return Ok(memory_semantic_index_receipt(
            snapshot,
            swarm_id,
            partition,
            indexed_at,
            config,
            collection,
            &model_hash,
            &content_set_hash,
            vector_size,
            0,
            deleted_document_count,
            &format!("{:x}", Sha256::digest([])),
            namespace,
        ));
    }

    let embedder = embedder(config, partition)?;
    let texts = documents
        .iter()
        .map(|document| document.projection_text.clone())
        .collect::<Vec<_>>();
    let embeddings = embedder.embed_documents(&texts)?;
    let vector_size = embeddings
        .first()
        .map(Vec::len)
        .ok_or_else(|| anyhow!("semantic projection produced no embeddings"))?;
    let compatibility = compatibility(config, partition, vector_size);
    if backend.collection_exists(collection)? {
        let actual = backend.collection_compatibility(collection)?;
        if actual != compatibility {
            return Err(anyhow!(
                "Qdrant collection {collection} is incompatible: actual {actual:?}, expected {compatibility:?}"
            ));
        }
    } else {
        backend.create_collection(collection, &compatibility)?;
    }
    let existing_ids = backend.point_ids_for_scope(collection, &scope)?;
    let points = documents
        .iter()
        .zip(embeddings)
        .map(|(document, vector)| SemanticPoint {
            id: physical_point_id(namespace, &document.point_id),
            vector,
            payload: point_payload(document, config, namespace),
        })
        .collect::<Vec<_>>();
    backend.upsert_points(collection, &points)?;
    let live_ids = documents
        .iter()
        .map(|document| physical_point_id(namespace, &document.point_id))
        .collect::<HashSet<_>>();
    let deleted_ids = existing_ids
        .into_iter()
        .filter(|id| !live_ids.contains(id.as_str()))
        .collect::<Vec<_>>();
    backend.delete_points(collection, &deleted_ids)?;
    let observed_ids = backend.point_ids_for_scope(collection, &scope)?;
    let desired_ids = documents
        .iter()
        .map(|document| physical_point_id(namespace, &document.point_id))
        .collect::<HashSet<_>>();
    let observed_ids = observed_ids.into_iter().collect::<HashSet<_>>();
    if observed_ids != desired_ids {
        return Err(anyhow!(
            "Qdrant scope synchronization failed: observed IDs do not equal the desired projection"
        ));
    }
    let desired_payloads = documents
        .iter()
        .map(|document| {
            (
                physical_point_id(namespace, &document.point_id),
                point_payload(document, config, namespace),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let observed_vector_binding_root_sha256 = authenticate_observed_projection(
        backend.points_for_scope::<MemorySemanticPointPayload>(collection, &scope)?,
        &desired_payloads,
        vector_size,
    )?;
    Ok(memory_semantic_index_receipt(
        snapshot,
        swarm_id,
        partition,
        indexed_at,
        config,
        collection,
        &model_hash,
        &content_set_hash,
        vector_size as u32,
        documents.len() as u32,
        deleted_ids.len() as u32,
        &observed_vector_binding_root_sha256,
        namespace,
    ))
}

#[allow(clippy::too_many_arguments)]
fn memory_semantic_index_receipt(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    swarm_id: &str,
    partition: SemanticPartition,
    indexed_at: &str,
    config: &MemorySemanticIndexConfig,
    collection: &str,
    model_hash: &str,
    content_set_hash: &str,
    vector_dimensions: u32,
    indexed_document_count: u32,
    deleted_document_count: u32,
    observed_vector_binding_root_sha256: &str,
    namespace: &MemorySemanticProjectionNamespace,
) -> MemorySemanticIndexReceipt {
    let receipt_id = format!(
        "memory-semantic-index-{}-{}-{}-{}",
        partition_name(partition),
        snapshot.model_revision,
        &content_set_hash[..16],
        &format!(
            "{:x}",
            Sha256::digest(
                format!(
                    "{}|{}|{}|{}",
                    indexed_at, namespace.obligation_id, namespace.claim_id, namespace.claim_epoch
                )
                .as_bytes()
            )
        )[..12]
    );
    MemorySemanticIndexReceipt {
        schema_version: MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id,
        swarm_id: swarm_id.to_string(),
        partition: partition_name(partition).to_string(),
        collection_name: collection.to_string(),
        graph_id: snapshot.graph_id.clone(),
        model_revision: snapshot.model_revision,
        model_hash: model_hash.to_string(),
        embedding_provider_id: config.embedding_provider_id.clone(),
        embedding_model: config.ollama_model.clone(),
        vector_dimensions,
        indexed_document_count,
        deleted_document_count,
        canonical_content_set_hash: content_set_hash.to_string(),
        indexed_at: indexed_at.to_string(),
        status: "ready".to_string(),
        obligation_id: namespace.obligation_id.clone(),
        canonical_source_id: snapshot.graph_id.clone(),
        source_commit_id: String::new(),
        source_generation: snapshot.model_revision,
        projection_schema_version: SEMANTIC_PROJECTION_SCHEMA_VERSION.to_string(),
        claim_id: namespace.claim_id.clone(),
        claim_epoch: namespace.claim_epoch,
        observed_vector_binding_root_sha256: observed_vector_binding_root_sha256.to_string(),
    }
}

pub fn semantic_memory_context(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    swarm_id: &str,
    partition: SemanticPartition,
    query: &EpiphanyMemoryContextQuery,
    readiness: Option<&MemorySemanticProjectionReadiness>,
    config: &MemorySemanticIndexConfig,
) -> EpiphanyMemoryContextPacket {
    let eligible = readiness.is_some_and(|readiness| {
        let source_matches_query = (|| -> Result<bool> {
            let documents = derive_semantic_projection(swarm_id, snapshot)?
                .into_iter()
                .filter(|document| document.partition == partition)
                .collect::<Vec<_>>();
            Ok(readiness.obligation.swarm_id == swarm_id
                && readiness.obligation.partition == partition_name(partition)
                && readiness.obligation.graph_id == snapshot.graph_id
                && readiness.obligation.source_generation == snapshot.model_revision
                && readiness.obligation.source_model_hash
                    == crate::memory_graph_model_hash(snapshot)?
                && readiness.obligation.canonical_content_set_hash
                    == canonical_content_set_hash(&documents))
        })()
        .unwrap_or(false);
        source_matches_query
            && memory_semantic_projection_query_eligible(
                &readiness.obligation,
                &readiness.current,
                &readiness.receipt,
            )
    });
    if !eligible {
        let mut packet =
            plan_memory_graph_context_cut_for_partition(snapshot, query, &[], partition);
        packet.warnings.push(
            "semantic projection unavailable; used canonical BM25 fallback: newest canonical obligation has no exact success receipt"
                .to_string(),
        );
        return packet;
    }
    let readiness = readiness.expect("eligible semantic readiness disappeared");
    match try_semantic_memory_context(snapshot, swarm_id, partition, query, readiness, config) {
        Ok(packet) => packet,
        Err(error) => {
            let mut packet =
                plan_memory_graph_context_cut_for_partition(snapshot, query, &[], partition);
            packet.warnings.push(format!(
                "semantic projection unavailable; used canonical BM25 fallback: {error}"
            ));
            packet
        }
    }
}

fn try_semantic_memory_context(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    swarm_id: &str,
    partition: SemanticPartition,
    query: &EpiphanyMemoryContextQuery,
    readiness: &MemorySemanticProjectionReadiness,
    config: &MemorySemanticIndexConfig,
) -> Result<EpiphanyMemoryContextPacket> {
    let text = query
        .text
        .as_deref()
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| anyhow!("semantic context requires query text"))?;
    let documents = derive_semantic_projection(swarm_id, snapshot)?;
    let backend = qdrant(config)?;
    let collection = config.collection(partition);
    if !backend.collection_exists(collection)? {
        return Err(anyhow!("semantic collection {collection} is missing"));
    }
    let embedder = embedder(config, partition)?;
    let vector = embedder.embed_query(text)?;
    let expected = compatibility(config, partition, vector.len());
    let actual = backend.collection_compatibility(collection)?;
    if actual != expected {
        return Err(anyhow!("semantic collection compatibility mismatch"));
    }
    let limit = query.budget.unwrap_or(12).clamp(1, QUERY_LIMIT_MAX as u32) as usize;
    let query_claim_epoch = readiness.receipt.claim_epoch.to_string();
    let ranked = backend.query_points_for_scope::<MemorySemanticPointPayload>(
        collection,
        &vector,
        limit,
        &[
            ("swarmId", swarm_id),
            ("partition", partition_name(partition)),
            ("obligationId", readiness.receipt.obligation_id.as_str()),
            ("claimId", readiness.receipt.claim_id.as_str()),
            ("claimEpoch", query_claim_epoch.as_str()),
        ],
    )?;
    let mut ranked_ids = Vec::new();
    for hit in ranked {
        let payload = hit
            .payload
            .ok_or_else(|| anyhow!("semantic candidate omitted its typed locator payload"))?;
        if payload.swarm_id != swarm_id
            || payload.partition != partition
            || payload.obligation_id != readiness.receipt.obligation_id
            || payload.claim_id != readiness.receipt.claim_id
            || payload.claim_epoch != readiness.receipt.claim_epoch.to_string()
            || payload.embedding_provider_id != config.embedding_provider_id
            || payload.embedding_model != config.ollama_model
            || payload.projection_schema_version != SEMANTIC_PROJECTION_SCHEMA_VERSION
        {
            continue;
        }
        let candidate = SemanticProjectionCandidate {
            point_id: payload.point_id,
            canonical: super::SemanticCanonicalLocator {
                locator: payload.canonical_locator,
                canonical_type: payload.canonical_type,
                canonical_key: payload.canonical_key,
                canonical_document_id: payload.canonical_document_id,
            },
            partition: payload.partition,
            score: hit.score,
            indexed_model_revision: payload.indexed_model_revision,
            indexed_model_hash: payload.indexed_model_hash,
            indexed_canonical_content_hash: payload.indexed_canonical_content_hash,
        };
        if let Ok(document) = resolve_semantic_candidate(partition, &candidate, &documents) {
            ranked_ids.push(document.canonical.canonical_document_id.clone());
        }
    }
    let mut packet =
        plan_memory_graph_context_cut_for_partition(snapshot, query, &ranked_ids, partition);
    packet.warnings.push(format!(
        "semantic projection ranked {} canonical {} candidates; all payload text was ignored",
        ranked_ids.len(),
        partition_name(partition)
    ));
    Ok(packet)
}

pub(super) fn point_payload(
    document: &SemanticProjectionDocument,
    config: &MemorySemanticIndexConfig,
    namespace: &MemorySemanticProjectionNamespace,
) -> MemorySemanticPointPayload {
    MemorySemanticPointPayload {
        point_id: document.point_id.clone(),
        swarm_id: document.swarm_id.clone(),
        partition: document.partition,
        obligation_id: namespace.obligation_id.clone(),
        claim_id: namespace.claim_id.clone(),
        claim_epoch: namespace.claim_epoch.to_string(),
        canonical_locator: document.canonical.locator.clone(),
        canonical_type: document.canonical.canonical_type.clone(),
        canonical_key: document.canonical.canonical_key.clone(),
        canonical_document_id: document.canonical.canonical_document_id.clone(),
        canonical_schema_version: document.canonical_schema_version.clone(),
        graph_id: document.graph_id.clone(),
        indexed_model_revision: document.model_revision,
        indexed_model_hash: document.model_hash.clone(),
        indexed_canonical_content_hash: document.canonical_content_hash.clone(),
        projection_schema_version: document.projection_schema_version.clone(),
        embedding_provider_id: config.embedding_provider_id.clone(),
        embedding_model: config.ollama_model.clone(),
    }
}

pub(super) fn physical_point_id(
    namespace: &MemorySemanticProjectionNamespace,
    canonical_point_id: &str,
) -> String {
    Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!(
            "{}|{}|{}|{}",
            namespace.obligation_id, namespace.claim_id, namespace.claim_epoch, canonical_point_id
        )
        .as_bytes(),
    )
    .to_string()
}

fn qdrant(config: &MemorySemanticIndexConfig) -> Result<QdrantBackend> {
    QdrantBackend::new(QdrantConfig {
        url: config.qdrant_url.clone(),
        api_key: config.qdrant_api_key.clone(),
        timeout_ms: config.qdrant_timeout_ms,
    })
}

#[allow(dead_code)] // Consumed by the repository-readiness join in the next bounded cut.
pub(crate) fn observe_memory_semantic_live_evidence(
    store_path: &Path,
    config: &MemorySemanticIndexConfig,
    input: &super::MemorySemanticProjectionInput,
    readiness: &MemorySemanticProjectionReadiness,
) -> Result<Option<MemorySemanticLiveEvidence>> {
    let mut backend = qdrant(config)?;
    observe_memory_semantic_live_evidence_with_port(
        store_path,
        config,
        input,
        readiness,
        &mut backend,
    )
}

pub(super) fn observe_memory_semantic_live_evidence_with_port(
    store_path: &Path,
    config: &MemorySemanticIndexConfig,
    input: &super::MemorySemanticProjectionInput,
    readiness: &MemorySemanticProjectionReadiness,
    port: &mut impl MemorySemanticEvidencePort,
) -> Result<Option<MemorySemanticLiveEvidence>> {
    let store_path = store_path.to_path_buf();
    observe_memory_semantic_live_evidence_with(config, input, readiness, port, || {
        semantic_authority_still_exact(&store_path, input, readiness)
    })
}

fn observe_memory_semantic_live_evidence_with(
    config: &MemorySemanticIndexConfig,
    input: &super::MemorySemanticProjectionInput,
    readiness: &MemorySemanticProjectionReadiness,
    port: &mut impl MemorySemanticEvidencePort,
    mut authority_still_exact: impl FnMut() -> Result<bool>,
) -> Result<Option<MemorySemanticLiveEvidence>> {
    if readiness.obligation != input.obligation
        || readiness.current != input.authority.head
        || !receipt_matches_obligation(&readiness.receipt, &input.obligation)
    {
        return Ok(None);
    }
    let partition = match input.obligation.partition.as_str() {
        "mind" => SemanticPartition::Mind,
        "modeling" => SemanticPartition::Modeling,
        _ => return Ok(None),
    };
    if readiness.receipt.collection_name != config.collection(partition) {
        return Ok(None);
    }
    let documents = derive_semantic_projection(&input.obligation.swarm_id, &input.snapshot)?
        .into_iter()
        .filter(|document| document.partition == partition)
        .collect::<Vec<_>>();
    if documents.len() as u32 != readiness.receipt.indexed_document_count
        || canonical_content_set_hash(&documents) != readiness.receipt.canonical_content_set_hash
        || readiness.receipt.embedding_provider_id != config.embedding_provider_id
        || readiness.receipt.embedding_model != config.ollama_model
        || readiness.receipt.vector_dimensions == 0
    {
        return Ok(None);
    }
    if !authority_still_exact()? {
        return Ok(None);
    }
    let collection = readiness.receipt.collection_name.as_str();
    if !port.collection_exists(collection)?
        || port.collection_compatibility(collection)?
            != compatibility(
                config,
                partition,
                readiness.receipt.vector_dimensions as usize,
            )
    {
        return Ok(None);
    }
    let namespace = MemorySemanticProjectionNamespace {
        obligation_id: readiness.receipt.obligation_id.clone(),
        claim_id: readiness.receipt.claim_id.clone(),
        claim_epoch: readiness.receipt.claim_epoch,
    };
    let epoch = namespace.claim_epoch.to_string();
    let scope = [
        ("swarmId", input.obligation.swarm_id.as_str()),
        ("partition", input.obligation.partition.as_str()),
        ("obligationId", namespace.obligation_id.as_str()),
        ("claimId", namespace.claim_id.as_str()),
        ("claimEpoch", epoch.as_str()),
    ];
    let desired = documents
        .iter()
        .map(|document| {
            (
                physical_point_id(&namespace, &document.point_id),
                point_payload(document, config, &namespace),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let observed = port.points_for_scope(collection, &scope)?;
    let count = observed.len() as u32;
    let root = authenticate_observed_projection(
        observed,
        &desired,
        readiness.receipt.vector_dimensions as usize,
    )?;
    if root != readiness.receipt.observed_vector_binding_root_sha256 || !authority_still_exact()? {
        return Ok(None);
    }
    Ok(Some(MemorySemanticLiveEvidence {
        collection_name: collection.to_string(),
        observed_point_count: count,
        observed_vector_binding_root_sha256: root,
    }))
}

#[allow(dead_code)] // Production half of the bounded live-evidence reader.
fn semantic_authority_still_exact(
    store_path: &Path,
    input: &super::MemorySemanticProjectionInput,
    expected: &MemorySemanticProjectionReadiness,
) -> Result<bool> {
    let envelopes = SingleFileMessagePackBackingStore::new(store_path).pull_all()?;
    if input.authority.envelopes.is_empty()
        || input.authority.envelopes.iter().any(|expected| {
            envelopes
                .iter()
                .find(|row| row.r#type == expected.r#type && row.key == expected.key)
                != Some(expected)
        })
    {
        return Ok(false);
    }
    let current = super::load_memory_semantic_projection_readiness(store_path, input)?;
    Ok(current.is_some_and(|current| {
        current.obligation == expected.obligation
            && current.current == expected.current
            && current.receipt == expected.receipt
    }))
}

fn embedder(
    config: &MemorySemanticIndexConfig,
    partition: SemanticPartition,
) -> Result<OllamaEmbedder> {
    OllamaEmbedder::new(OllamaConfig {
        base_url: config.ollama_base_url.clone(),
        model: config.ollama_model.clone(),
        timeout_ms: config.ollama_timeout_ms,
        query_instruction: config.query_instruction(partition).to_string(),
    })
}

fn compatibility(
    config: &MemorySemanticIndexConfig,
    partition: SemanticPartition,
    vector_size: usize,
) -> CollectionCompatibility {
    CollectionCompatibility {
        managed_by: "epiphany".to_string(),
        corpus_kind: partition_name(partition).to_string(),
        schema_version: 1,
        projection_version: SEMANTIC_PROJECTION_SCHEMA_VERSION.to_string(),
        embedding_provider_id: config.embedding_provider_id.clone(),
        embedding_model: config.ollama_model.clone(),
        vector_size,
    }
}

fn partition_name(partition: SemanticPartition) -> &'static str {
    match partition {
        SemanticPartition::Mind => "mind",
        SemanticPartition::Modeling => "modeling",
    }
}

fn canonical_content_set_hash(documents: &[SemanticProjectionDocument]) -> String {
    let mut values = documents
        .iter()
        .map(|document| format!("{}:{}", document.point_id, document.canonical_content_hash))
        .collect::<Vec<_>>();
    values.sort();
    format!("{:x}", Sha256::digest(values.join("\n").as_bytes()))
}

fn env_value(name: &str, default: &str) -> String {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn env_u64(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_graph::{
        EpiphanyMemoryDomain, EpiphanyMemoryLifecycle, EpiphanyMemoryNode, EpiphanyMemoryNodeKind,
        EpiphanyMemoryProfile, MEMORY_GRAPH_SCHEMA_VERSION, memory_graph_model_hash,
    };
    use tokio::runtime::Runtime;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn snapshot() -> EpiphanyMemoryGraphSnapshot {
        let mut snapshot = EpiphanyMemoryGraphSnapshot {
            schema_version: Some(MEMORY_GRAPH_SCHEMA_VERSION.to_string()),
            graph_id: "semantic-fallback".to_string(),
            model_revision: 3,
            domains: vec![EpiphanyMemoryDomain {
                id: "repo".to_string(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                title: "Repository".to_string(),
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                ..Default::default()
            }],
            nodes: vec![EpiphanyMemoryNode {
                id: "canonical-node".to_string(),
                domain_id: "repo".to_string(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                kind: EpiphanyMemoryNodeKind::Module,
                title: "Semantic projection".to_string(),
                claim: "Canonical graph remains correct without Qdrant.".to_string(),
                question: "How is semantic recall ranked?".to_string(),
                tension: "The vector service may be absent.".to_string(),
                action_implication: "Fall back to typed BM25 traversal.".to_string(),
                source_hashes: vec!["anchor:missing".to_string()],
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                confidence: 90,
                ..Default::default()
            }],
            ..Default::default()
        };
        snapshot.model_hash = memory_graph_model_hash(&snapshot).unwrap();
        snapshot
    }

    #[test]
    fn empty_partition_bypasses_ollama_and_does_not_create_collection() -> Result<()> {
        let runtime = Runtime::new()?;
        let qdrant = runtime.block_on(MockServer::start());
        runtime.block_on(
            Mock::given(method("GET"))
                .and(path("/collections/empty_modeling/exists"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "result": { "exists": false },
                    "status": "ok",
                    "time": 0.0
                })))
                .expect(1)
                .mount(&qdrant),
        );
        let mut empty = EpiphanyMemoryGraphSnapshot {
            schema_version: Some(MEMORY_GRAPH_SCHEMA_VERSION.to_string()),
            graph_id: "empty-semantic-source".to_string(),
            model_revision: 9,
            ..Default::default()
        };
        empty.model_hash = memory_graph_model_hash(&empty)?;
        let mut config = MemorySemanticIndexConfig::from_env();
        config.qdrant_url = qdrant.uri();
        config.modeling_collection = "empty_modeling".to_string();
        config.ollama_base_url = "http://127.0.0.1:1".to_string();
        config.ollama_timeout_ms = 5;

        let obligation = derive_memory_semantic_projection_obligation(
            &empty,
            "swarm-empty",
            SemanticPartition::Modeling,
            "source-empty",
            "commit-empty",
            "2026-07-15T11:59:00Z",
        )?;

        let receipt = index_memory_semantic_partition(
            &empty,
            "swarm-empty",
            SemanticPartition::Modeling,
            &MemorySemanticProjectionNamespace {
                obligation_id: obligation.obligation_id.clone(),
                claim_id: "claim-empty".to_string(),
                claim_epoch: 1,
            },
            "2026-07-15T12:00:00Z",
            &config,
        )?;

        assert_eq!(receipt.indexed_document_count, 0);
        assert_eq!(receipt.deleted_document_count, 0);
        assert_eq!(receipt.vector_dimensions, 0);
        assert_eq!(receipt.model_hash, empty.model_hash);
        assert_eq!(receipt.status, "ready");
        let bound = bind_memory_semantic_index_receipt(receipt, &obligation)?;
        let current = MemorySemanticProjectionSourceHead {
            swarm_id: obligation.swarm_id.clone(),
            partition: obligation.partition.clone(),
            canonical_source_id: obligation.canonical_source_id.clone(),
            source_commit_id: obligation.source_commit_id.clone(),
            graph_id: obligation.graph_id.clone(),
            source_generation: obligation.source_generation,
            source_model_hash: obligation.source_model_hash.clone(),
            canonical_content_set_hash: obligation.canonical_content_set_hash.clone(),
        };
        assert!(!memory_semantic_projection_query_eligible(
            &obligation,
            &current,
            &bound
        ));
        let health =
            derive_memory_semantic_projection_health(&obligation, &current, &[], &[bound])?;
        assert_eq!(health.status, MemorySemanticProjectionHealthStatus::Ready);
        assert!(!health.query_eligible);
        Ok(())
    }

    #[test]
    fn projection_obligation_identity_is_commit_owned_and_deterministic() -> Result<()> {
        let snapshot = snapshot();
        let first = derive_memory_semantic_projection_obligation(
            &snapshot,
            "swarm-a",
            SemanticPartition::Modeling,
            "epiphany.runtime/runtime-a/repo-model",
            "repo-model-admission-1",
            "2026-07-15T00:00:00Z",
        )?;
        assert_eq!(
            derive_memory_semantic_projection_obligation(
                &snapshot,
                "swarm-a",
                SemanticPartition::Modeling,
                "epiphany.runtime/runtime-a/repo-model",
                "repo-model-admission-1",
                "2026-07-15T00:00:00Z",
            )?,
            first
        );
        let second = derive_memory_semantic_projection_obligation(
            &snapshot,
            "swarm-a",
            SemanticPartition::Modeling,
            "epiphany.runtime/runtime-a/repo-model",
            "repo-model-admission-2",
            "2026-07-15T00:00:01Z",
        )?;
        assert_ne!(first.obligation_id, second.obligation_id);
        Ok(())
    }

    #[test]
    fn unavailable_vector_service_preserves_canonical_bm25_packet() {
        let snapshot = snapshot();
        let query = EpiphanyMemoryContextQuery {
            id: "fallback-query".to_string(),
            profile: Some(EpiphanyMemoryProfile::RepoArchitecture),
            text: Some("canonical qdrant semantic".to_string()),
            ..Default::default()
        };
        let expected = plan_memory_graph_context_cut_for_partition(
            &snapshot,
            &query,
            &[],
            SemanticPartition::Modeling,
        );
        let mut config = MemorySemanticIndexConfig::from_env();
        config.qdrant_url = "http://127.0.0.1:1".to_string();
        config.qdrant_timeout_ms = 25;
        let actual = semantic_memory_context(
            &snapshot,
            "swarm",
            SemanticPartition::Modeling,
            &query,
            None,
            &config,
        );
        assert_eq!(actual.nodes, expected.nodes);
        assert_eq!(actual.edges, expected.edges);
        assert_eq!(actual.summaries, expected.summaries);
        assert!(
            actual
                .warnings
                .iter()
                .any(|warning| warning.contains("canonical BM25 fallback"))
        );
    }

    #[test]
    fn exact_readiness_for_one_source_cannot_open_another_query_scope() -> Result<()> {
        let snapshot = snapshot();
        let obligation = derive_memory_semantic_projection_obligation(
            &snapshot,
            "swarm-a",
            SemanticPartition::Modeling,
            "runtime/repo-model",
            "admission-current",
            "2026-07-15T10:00:00Z",
        )?;
        let current = MemorySemanticProjectionSourceHead {
            swarm_id: obligation.swarm_id.clone(),
            partition: obligation.partition.clone(),
            canonical_source_id: obligation.canonical_source_id.clone(),
            source_commit_id: obligation.source_commit_id.clone(),
            graph_id: obligation.graph_id.clone(),
            source_generation: obligation.source_generation,
            source_model_hash: obligation.source_model_hash.clone(),
            canonical_content_set_hash: obligation.canonical_content_set_hash.clone(),
        };
        let receipt = MemorySemanticIndexReceipt {
            schema_version: MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "ready-current".to_string(),
            swarm_id: obligation.swarm_id.clone(),
            partition: obligation.partition.clone(),
            collection_name: "modeling".to_string(),
            graph_id: obligation.graph_id.clone(),
            model_revision: obligation.source_generation,
            model_hash: obligation.source_model_hash.clone(),
            embedding_provider_id: "provider".to_string(),
            embedding_model: "model".to_string(),
            vector_dimensions: 3,
            indexed_document_count: 3,
            deleted_document_count: 0,
            canonical_content_set_hash: obligation.canonical_content_set_hash.clone(),
            indexed_at: "2026-07-15T10:01:00Z".to_string(),
            status: "ready".to_string(),
            obligation_id: obligation.obligation_id.clone(),
            canonical_source_id: obligation.canonical_source_id.clone(),
            source_commit_id: obligation.source_commit_id.clone(),
            source_generation: obligation.source_generation,
            projection_schema_version: obligation.projection_schema_version.clone(),
            claim_id: "claim-current".to_string(),
            claim_epoch: 1,
            observed_vector_binding_root_sha256: "0".repeat(64),
        };
        let query = EpiphanyMemoryContextQuery {
            id: "swapped-source".to_string(),
            text: Some("authority".to_string()),
            ..Default::default()
        };
        let mut config = MemorySemanticIndexConfig::from_env();
        config.qdrant_url = "http://127.0.0.1:1".to_string();
        config.ollama_base_url = "http://127.0.0.1:1".to_string();
        let readiness = MemorySemanticProjectionReadiness {
            obligation: obligation.clone(),
            current: current.clone(),
            receipt: receipt.clone(),
        };
        let packet = semantic_memory_context(
            &snapshot,
            "swarm-b",
            SemanticPartition::Modeling,
            &query,
            Some(&readiness),
            &config,
        );
        assert!(packet.warnings.iter().any(|warning| {
            warning.contains("newest canonical obligation has no exact success receipt")
        }));
        Ok(())
    }

    #[test]
    fn mind_and_modeling_have_distinct_physical_collections_and_query_profiles() {
        let config = MemorySemanticIndexConfig::from_env();
        assert_ne!(
            config.collection(SemanticPartition::Mind),
            config.collection(SemanticPartition::Modeling)
        );
        assert_ne!(
            config.query_instruction(SemanticPartition::Mind),
            config.query_instruction(SemanticPartition::Modeling)
        );
    }

    #[test]
    fn physical_point_identity_is_claim_epoch_isolated_while_locator_stays_canonical() {
        let document = derive_semantic_projection("swarm-a", &snapshot())
            .unwrap()
            .into_iter()
            .next()
            .expect("semantic document");
        let first = MemorySemanticProjectionNamespace {
            obligation_id: "obligation-a".to_string(),
            claim_id: "claim-a".to_string(),
            claim_epoch: 1,
        };
        let fenced = MemorySemanticProjectionNamespace {
            claim_id: "claim-b".to_string(),
            claim_epoch: 2,
            ..first.clone()
        };
        let first_id = physical_point_id(&first, &document.point_id);
        let fenced_id = physical_point_id(&fenced, &document.point_id);
        assert_ne!(first_id, fenced_id);
        assert!(Uuid::parse_str(&first_id).is_ok());
        assert!(Uuid::parse_str(&fenced_id).is_ok());

        let payload = point_payload(&document, &MemorySemanticIndexConfig::from_env(), &fenced);
        assert_eq!(payload.point_id, document.point_id);
        assert_eq!(payload.obligation_id, fenced.obligation_id);
        assert_eq!(payload.claim_id, fenced.claim_id);
        assert_eq!(payload.claim_epoch, fenced.claim_epoch.to_string());
    }

    #[test]
    fn pre_namespace_receipt_is_never_query_eligible() {
        let obligation = obligation();
        let current = source_head();
        let mut legacy = receipt();
        legacy.schema_version = "gamecult.epiphany.memory_semantic_index_receipt.v0".to_string();
        legacy.claim_id.clear();
        legacy.claim_epoch = 0;
        assert!(!memory_semantic_projection_query_eligible(
            &obligation,
            &current,
            &legacy
        ));
    }

    fn obligation() -> MemorySemanticProjectionObligation {
        MemorySemanticProjectionObligation {
            schema_version: MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION.to_string(),
            obligation_id: "projection:swarm:modeling:commit-7".to_string(),
            swarm_id: "swarm".to_string(),
            partition: "modeling".to_string(),
            canonical_source_id: "runtime-spine".to_string(),
            source_commit_id: "commit-7".to_string(),
            graph_id: "graph".to_string(),
            source_generation: 7,
            source_model_hash: "model-hash".to_string(),
            canonical_content_set_hash: "content-hash".to_string(),
            projection_schema_version: SEMANTIC_PROJECTION_SCHEMA_VERSION.to_string(),
            created_at: "2026-07-15T10:00:00Z".to_string(),
        }
    }

    fn source_head() -> MemorySemanticProjectionSourceHead {
        let obligation = obligation();
        MemorySemanticProjectionSourceHead {
            swarm_id: obligation.swarm_id,
            partition: obligation.partition,
            canonical_source_id: obligation.canonical_source_id,
            source_commit_id: obligation.source_commit_id,
            graph_id: obligation.graph_id,
            source_generation: obligation.source_generation,
            source_model_hash: obligation.source_model_hash,
            canonical_content_set_hash: obligation.canonical_content_set_hash,
        }
    }

    fn receipt() -> MemorySemanticIndexReceipt {
        let obligation = obligation();
        MemorySemanticIndexReceipt {
            schema_version: MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "receipt-7".to_string(),
            swarm_id: obligation.swarm_id,
            partition: obligation.partition,
            collection_name: "epiphany_modeling_v1".to_string(),
            graph_id: obligation.graph_id,
            model_revision: obligation.source_generation,
            model_hash: obligation.source_model_hash,
            embedding_provider_id: "provider".to_string(),
            embedding_model: "model".to_string(),
            vector_dimensions: 1024,
            indexed_document_count: 3,
            deleted_document_count: 0,
            canonical_content_set_hash: obligation.canonical_content_set_hash,
            indexed_at: "2026-07-15T10:01:00Z".to_string(),
            status: "ready".to_string(),
            obligation_id: obligation.obligation_id,
            canonical_source_id: obligation.canonical_source_id,
            source_commit_id: obligation.source_commit_id,
            source_generation: obligation.source_generation,
            projection_schema_version: obligation.projection_schema_version,
            claim_id: "claim-7".to_string(),
            claim_epoch: 1,
            observed_vector_binding_root_sha256: "0".repeat(64),
        }
    }

    fn failed_attempt() -> MemorySemanticProjectionAttempt {
        MemorySemanticProjectionAttempt {
            schema_version: MEMORY_SEMANTIC_PROJECTION_ATTEMPT_SCHEMA_VERSION.to_string(),
            attempt_id: "attempt-1".to_string(),
            obligation_id: obligation().obligation_id,
            started_at: "2026-07-15T10:00:30Z".to_string(),
            completed_at: Some("2026-07-15T10:00:31Z".to_string()),
            status: "failed".to_string(),
            error: Some("qdrant unavailable".to_string()),
            claim_id: "claim-7".to_string(),
            claim_epoch: 1,
            executor_id: "executor-a".to_string(),
            executor_incarnation: "executor-a-incarnation".to_string(),
            authority_id: "authority-a".to_string(),
        }
    }

    #[test]
    fn projection_health_is_derived_from_exact_canonical_causality() {
        let obligation = obligation();
        let current = source_head();
        let pending =
            derive_memory_semantic_projection_health(&obligation, &current, &[], &[]).unwrap();
        assert_eq!(
            pending.status,
            MemorySemanticProjectionHealthStatus::Pending
        );
        assert!(!pending.query_eligible);

        let failed = derive_memory_semantic_projection_health(
            &obligation,
            &current,
            &[failed_attempt()],
            &[],
        )
        .unwrap();
        assert_eq!(failed.status, MemorySemanticProjectionHealthStatus::Failed);
        assert_eq!(failed.latest_error.as_deref(), Some("qdrant unavailable"));

        let ready = derive_memory_semantic_projection_health(
            &obligation,
            &current,
            &[failed_attempt()],
            &[receipt()],
        )
        .unwrap();
        assert_eq!(ready.status, MemorySemanticProjectionHealthStatus::Ready);
        assert!(ready.query_eligible);

        let mut repair = failed_attempt();
        repair.attempt_id = "attempt-repair".to_string();
        repair.started_at = "2026-07-15T10:02:00Z".to_string();
        repair.completed_at = Some("2026-07-15T10:02:01Z".to_string());
        let repairing = derive_memory_semantic_projection_health(
            &obligation,
            &current,
            &[repair],
            &[receipt()],
        )
        .unwrap();
        assert_eq!(
            repairing.status,
            MemorySemanticProjectionHealthStatus::Failed
        );
        assert!(!repairing.query_eligible);

        let mut newer = current.clone();
        newer.source_generation += 1;
        newer.source_commit_id = "commit-8".to_string();
        let stale = derive_memory_semantic_projection_health(
            &obligation,
            &newer,
            &[failed_attempt()],
            &[receipt()],
        )
        .unwrap();
        assert_eq!(stale.status, MemorySemanticProjectionHealthStatus::Stale);
        assert!(!stale.query_eligible);
    }

    #[test]
    fn hostile_receipt_mismatches_never_grant_query_eligibility() {
        let obligation = obligation();
        let current = source_head();
        for field in [
            "obligation",
            "swarm",
            "partition",
            "source",
            "commit",
            "graph",
            "generation",
            "revision",
            "model",
            "content",
            "projection-schema",
            "status",
            "schema",
            "vector-root",
            "uppercase-vector-root",
            "short-vector-root",
        ] {
            let mut hostile = receipt();
            match field {
                "obligation" => hostile.obligation_id = "other".to_string(),
                "swarm" => hostile.swarm_id = "other".to_string(),
                "partition" => hostile.partition = "mind".to_string(),
                "source" => hostile.canonical_source_id = "other".to_string(),
                "commit" => hostile.source_commit_id = "other".to_string(),
                "graph" => hostile.graph_id = "other".to_string(),
                "generation" => hostile.source_generation += 1,
                "revision" => hostile.model_revision += 1,
                "model" => hostile.model_hash = "other".to_string(),
                "content" => hostile.canonical_content_set_hash = "other".to_string(),
                "projection-schema" => hostile.projection_schema_version = "other".to_string(),
                "status" => hostile.status = "failed".to_string(),
                "schema" => {
                    hostile.schema_version =
                        "gamecult.epiphany.memory_semantic_index_receipt.v1".to_string()
                }
                "vector-root" => hostile.observed_vector_binding_root_sha256.clear(),
                "uppercase-vector-root" => {
                    hostile.observed_vector_binding_root_sha256 = "A".repeat(64)
                }
                "short-vector-root" => hostile.observed_vector_binding_root_sha256 = "0".repeat(63),
                _ => unreachable!(),
            }
            assert!(
                !memory_semantic_projection_query_eligible(&obligation, &current, &hostile),
                "hostile {field} substitution was accepted"
            );
        }
    }

    #[test]
    fn unbound_index_result_is_not_ready_until_exact_obligation_binding() {
        let obligation = obligation();
        let current = source_head();
        let mut result = receipt();
        result.obligation_id.clear();
        result.canonical_source_id.clear();
        result.source_commit_id.clear();
        assert!(!memory_semantic_projection_query_eligible(
            &obligation,
            &current,
            &result
        ));
        let bound = bind_memory_semantic_index_receipt(result, &obligation).unwrap();
        assert!(memory_semantic_projection_query_eligible(
            &obligation,
            &current,
            &bound
        ));
    }

    #[test]
    fn v1_and_default_vector_roots_cannot_bind_as_terminal_success() {
        let obligation = obligation();
        let mut v1 = receipt();
        v1.schema_version = "gamecult.epiphany.memory_semantic_index_receipt.v1".to_string();
        assert!(bind_memory_semantic_index_receipt(v1, &obligation).is_err());
        let mut default_root = receipt();
        default_root.observed_vector_binding_root_sha256.clear();
        assert!(bind_memory_semantic_index_receipt(default_root, &obligation).is_err());
    }

    #[test]
    fn zero_count_receipt_cannot_emptywash_a_nonempty_obligation() {
        let obligation = obligation();
        assert_ne!(
            obligation.canonical_content_set_hash,
            canonical_content_set_hash(&[])
        );
        let current = source_head();
        let mut hostile = receipt();
        hostile.indexed_document_count = 0;
        hostile.vector_dimensions = 0;
        hostile.observed_vector_binding_root_sha256 = format!("{:x}", Sha256::digest([]));
        assert!(receipt_matches_obligation(&hostile, &obligation));
        assert!(!memory_semantic_projection_terminal_success(
            &obligation,
            &current,
            &hostile,
        ));
    }

    fn observed_payload(id: &str) -> MemorySemanticPointPayload {
        MemorySemanticPointPayload {
            point_id: id.to_string(),
            swarm_id: "swarm-a".to_string(),
            partition: SemanticPartition::Modeling,
            obligation_id: "obligation-a".to_string(),
            claim_id: "claim-a".to_string(),
            claim_epoch: "1".to_string(),
            canonical_locator: format!("locator:{id}"),
            canonical_type: "node".to_string(),
            canonical_key: id.to_string(),
            canonical_document_id: id.to_string(),
            canonical_schema_version: "v0".to_string(),
            graph_id: "graph-a".to_string(),
            indexed_model_revision: 1,
            indexed_model_hash: "model-a".to_string(),
            indexed_canonical_content_hash: "content-a".to_string(),
            projection_schema_version: SEMANTIC_PROJECTION_SCHEMA_VERSION.to_string(),
            embedding_provider_id: "provider".to_string(),
            embedding_model: "model".to_string(),
        }
    }

    fn stored(
        id: &str,
        payload: MemorySemanticPointPayload,
        vector: Option<Vec<f32>>,
    ) -> crate::semantic_backend::SemanticStoredPoint<MemorySemanticPointPayload> {
        crate::semantic_backend::SemanticStoredPoint {
            id: id.to_string(),
            payload: Some(payload),
            vector,
        }
    }

    #[test]
    fn observed_projection_authentication_rejects_every_live_substitution_shape() {
        let desired = BTreeMap::from([
            ("a".to_string(), observed_payload("a")),
            ("b".to_string(), observed_payload("b")),
        ]);
        let valid = vec![
            stored("a", observed_payload("a"), Some(vec![1.0, 2.0])),
            stored("b", observed_payload("b"), Some(vec![3.0, 4.0])),
        ];
        let root = authenticate_observed_projection(valid.clone(), &desired, 2).unwrap();
        assert!(!root.is_empty());

        let mut cases = Vec::new();
        cases.push(vec![valid[0].clone()]); // missing
        cases.push(vec![
            valid[0].clone(),
            valid[1].clone(),
            stored("x", observed_payload("x"), Some(vec![1.0, 2.0])),
        ]); // extra
        cases.push(vec![valid[0].clone(), valid[0].clone()]); // duplicate plus missing
        let mut wrong_id = valid.clone();
        wrong_id[1].id = "x".to_string();
        cases.push(wrong_id);
        let mut wrong_payload = valid.clone();
        wrong_payload[1].payload = Some(observed_payload("x"));
        cases.push(wrong_payload);
        let mut absent_vector = valid.clone();
        absent_vector[1].vector = None;
        cases.push(absent_vector);
        let mut wrong_dimension = valid.clone();
        wrong_dimension[1].vector = Some(vec![1.0]);
        cases.push(wrong_dimension);
        let mut nonfinite = valid.clone();
        nonfinite[1].vector = Some(vec![f32::NAN, 1.0]);
        cases.push(nonfinite);
        for hostile in cases {
            assert!(authenticate_observed_projection(hostile, &desired, 2).is_err());
        }

        let mut substituted = valid;
        substituted[1].vector = Some(vec![30.0, 40.0]);
        assert_ne!(
            authenticate_observed_projection(substituted, &desired, 2).unwrap(),
            root
        );
    }

    struct FakeEvidencePort {
        exists: bool,
        compatibility: CollectionCompatibility,
        points: Vec<crate::semantic_backend::SemanticStoredPoint<MemorySemanticPointPayload>>,
    }

    struct StoreAdvancingPort {
        store: std::path::PathBuf,
        compatibility: CollectionCompatibility,
        points: Vec<crate::semantic_backend::SemanticStoredPoint<MemorySemanticPointPayload>>,
        replacement: crate::EpiphanyMemoryGraphEntry,
    }

    impl MemorySemanticEvidencePort for StoreAdvancingPort {
        fn collection_exists(&mut self, _name: &str) -> Result<bool> {
            Ok(true)
        }
        fn collection_compatibility(&mut self, _name: &str) -> Result<CollectionCompatibility> {
            Ok(self.compatibility.clone())
        }
        fn points_for_scope(
            &mut self,
            _name: &str,
            _scope: &[(&str, &str)],
        ) -> Result<Vec<crate::semantic_backend::SemanticStoredPoint<MemorySemanticPointPayload>>>
        {
            let mut cache = crate::runtime_spine_cache(&self.store)?;
            cache.put(crate::MEMORY_GRAPH_KEY, &self.replacement)?;
            Ok(self.points.clone())
        }
    }

    impl MemorySemanticEvidencePort for FakeEvidencePort {
        fn collection_exists(&mut self, _name: &str) -> Result<bool> {
            Ok(self.exists)
        }
        fn collection_compatibility(&mut self, _name: &str) -> Result<CollectionCompatibility> {
            Ok(self.compatibility.clone())
        }
        fn points_for_scope(
            &mut self,
            _name: &str,
            _scope: &[(&str, &str)],
        ) -> Result<Vec<crate::semantic_backend::SemanticStoredPoint<MemorySemanticPointPayload>>>
        {
            Ok(self.points.clone())
        }
    }

    fn live_evidence_fixture() -> (
        MemorySemanticIndexConfig,
        super::super::MemorySemanticProjectionInput,
        MemorySemanticProjectionReadiness,
        FakeEvidencePort,
    ) {
        let snapshot = snapshot();
        let mut config = MemorySemanticIndexConfig::from_env();
        config.embedding_provider_id = "provider".to_string();
        config.ollama_model = "model".to_string();
        config.modeling_collection = "modeling-exact".to_string();
        let obligation = derive_memory_semantic_projection_obligation(
            &snapshot,
            "swarm-a",
            SemanticPartition::Modeling,
            "source-a",
            "commit-a",
            "2026-07-16T00:00:00Z",
        )
        .unwrap();
        let namespace = MemorySemanticProjectionNamespace {
            obligation_id: obligation.obligation_id.clone(),
            claim_id: "claim-a".to_string(),
            claim_epoch: 1,
        };
        let documents = derive_semantic_projection("swarm-a", &snapshot)
            .unwrap()
            .into_iter()
            .filter(|row| row.partition == SemanticPartition::Modeling)
            .collect::<Vec<_>>();
        let desired = documents
            .iter()
            .map(|document| {
                (
                    physical_point_id(&namespace, &document.point_id),
                    point_payload(document, &config, &namespace),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let points = desired
            .iter()
            .enumerate()
            .map(|(index, (id, payload))| {
                stored(
                    id,
                    payload.clone(),
                    Some(vec![index as f32 + 1.0, index as f32 + 2.0]),
                )
            })
            .collect::<Vec<_>>();
        let root = authenticate_observed_projection(points.clone(), &desired, 2).unwrap();
        let raw = memory_semantic_index_receipt(
            &snapshot,
            "swarm-a",
            SemanticPartition::Modeling,
            "2026-07-16T00:01:00Z",
            &config,
            &config.modeling_collection,
            &memory_graph_model_hash(&snapshot).unwrap(),
            &canonical_content_set_hash(&documents),
            2,
            documents.len() as u32,
            0,
            &root,
            &namespace,
        );
        let receipt = bind_memory_semantic_index_receipt(raw, &obligation).unwrap();
        let current = MemorySemanticProjectionSourceHead {
            swarm_id: obligation.swarm_id.clone(),
            partition: obligation.partition.clone(),
            canonical_source_id: obligation.canonical_source_id.clone(),
            source_commit_id: obligation.source_commit_id.clone(),
            graph_id: obligation.graph_id.clone(),
            source_generation: obligation.source_generation,
            source_model_hash: obligation.source_model_hash.clone(),
            canonical_content_set_hash: obligation.canonical_content_set_hash.clone(),
        };
        let input = super::super::MemorySemanticProjectionInput {
            snapshot,
            obligation: obligation.clone(),
            authority: super::super::MemorySemanticProjectionAuthoritySnapshot {
                head: current.clone(),
                envelopes: vec![],
            },
        };
        let readiness = MemorySemanticProjectionReadiness {
            obligation,
            current,
            receipt,
        };
        let port = FakeEvidencePort {
            exists: true,
            compatibility: compatibility(&config, SemanticPartition::Modeling, 2),
            points,
        };
        (config, input, readiness, port)
    }

    #[test]
    fn live_evidence_requires_named_collection_compatibility_and_stable_authority() {
        let (config, input, readiness, mut port) = live_evidence_fixture();
        let mut calls = 0;
        let evidence = observe_memory_semantic_live_evidence_with(
            &config,
            &input,
            &readiness,
            &mut port,
            || {
                calls += 1;
                Ok(true)
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(calls, 2);
        assert_eq!(
            evidence.observed_vector_binding_root_sha256,
            readiness.receipt.observed_vector_binding_root_sha256
        );

        let (_, _, _, mut missing) = live_evidence_fixture();
        missing.exists = false;
        assert!(
            observe_memory_semantic_live_evidence_with(
                &config,
                &input,
                &readiness,
                &mut missing,
                || Ok(true)
            )
            .unwrap()
            .is_none()
        );
        let (_, _, _, mut incompatible) = live_evidence_fixture();
        incompatible.compatibility.vector_size = 3;
        assert!(
            observe_memory_semantic_live_evidence_with(
                &config,
                &input,
                &readiness,
                &mut incompatible,
                || Ok(true)
            )
            .unwrap()
            .is_none()
        );
        let mut wrong_collection = readiness.clone();
        wrong_collection.receipt.collection_name = "compatible-but-wrong".to_string();
        let (_, _, _, mut port) = live_evidence_fixture();
        assert!(
            observe_memory_semantic_live_evidence_with(
                &config,
                &input,
                &wrong_collection,
                &mut port,
                || Ok(true)
            )
            .unwrap()
            .is_none()
        );
        let (_, _, _, mut port) = live_evidence_fixture();
        let mut phase = 0;
        assert!(
            observe_memory_semantic_live_evidence_with(
                &config,
                &input,
                &readiness,
                &mut port,
                || {
                    phase += 1;
                    Ok(phase == 1)
                }
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn store_backed_live_reader_refuses_repo_model_advance_during_scroll() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("semantic-authority-race.cc");
        crate::initialize_runtime_spine(
            &store,
            crate::RuntimeSpineInitOptions {
                runtime_id: "semantic-race-runtime".into(),
                display_name: "Semantic race runtime".into(),
                created_at: "2026-07-16T00:00:00Z".into(),
            },
        )?;
        crate::runtime_spine::tests::bind_test_runtime_swarm(&store, "swarm-a")?;
        let model = snapshot();
        let entry = crate::EpiphanyMemoryGraphEntry::from_snapshot(&model)?;
        let migration = crate::RepoModelMigrationReceipt {
            schema_version: crate::REPO_MODEL_MIGRATION_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "repo-model-migration".into(),
            source_store: "semantic-race-test".into(),
            source_graph_id: model.graph_id.clone(),
            imported_revision: model.model_revision,
            imported_hash: memory_graph_model_hash(&model)?,
            imported_at: "2026-07-16T00:00:01Z".into(),
            contract: crate::REPO_MODEL_MIGRATION_CONTRACT.to_string(),
        };
        let mut cache = crate::runtime_spine_cache(&store)?;
        cache.put(crate::MEMORY_GRAPH_KEY, &entry)?;
        cache.put(&migration.receipt_id, &migration)?;
        crate::migrate_legacy_repo_model_projection_obligation(&store)?
            .expect("projection obligation");
        let mut input = crate::runtime_modeling_semantic_projection_input(&store)?;
        assert_eq!(input.authority.envelopes.len(), 2);
        let mut authority_cache = crate::runtime_spine_cache(&store)?;
        authority_cache.pull_all_backing_stores()?;
        let extra = authority_cache
            .snapshot_envelopes()
            .into_iter()
            .find(|row| {
                !input
                    .authority
                    .envelopes
                    .iter()
                    .any(|captured| captured.r#type == row.r#type && captured.key == row.key)
            })
            .expect("runtime provides an additional exact authority envelope");
        input.authority.envelopes.push(extra);
        assert!(input.authority.envelopes.len() > 2);
        let acquisition =
            super::super::semantic_projector::idunn_acquire_memory_semantic_projection(
                &store,
                &input,
                "executor-a",
                "executor-incarnation-a",
                "execute",
                "idunn-a",
                "2026-07-16T00:00:30Z",
            )?;
        let mut config = MemorySemanticIndexConfig::from_env();
        config.embedding_provider_id = "provider".into();
        config.ollama_model = "model".into();
        config.modeling_collection = "modeling-exact".into();
        let namespace = MemorySemanticProjectionNamespace {
            obligation_id: input.obligation.obligation_id.clone(),
            claim_id: acquisition.claim.claim_id.clone(),
            claim_epoch: acquisition.claim.epoch,
        };
        let documents = derive_semantic_projection("swarm-a", &model)?
            .into_iter()
            .filter(|row| row.partition == SemanticPartition::Modeling)
            .collect::<Vec<_>>();
        let desired = documents
            .iter()
            .map(|document| {
                (
                    physical_point_id(&namespace, &document.point_id),
                    point_payload(document, &config, &namespace),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let points = desired
            .iter()
            .enumerate()
            .map(|(index, (id, payload))| {
                stored(
                    id,
                    payload.clone(),
                    Some(vec![index as f32 + 1.0, index as f32 + 2.0]),
                )
            })
            .collect::<Vec<_>>();
        let root = authenticate_observed_projection(points.clone(), &desired, 2)?;
        let raw = memory_semantic_index_receipt(
            &model,
            "swarm-a",
            SemanticPartition::Modeling,
            "2026-07-16T00:01:00Z",
            &config,
            &config.modeling_collection,
            &memory_graph_model_hash(&model)?,
            &canonical_content_set_hash(&documents),
            2,
            documents.len() as u32,
            0,
            &root,
            &namespace,
        );
        let receipt = bind_memory_semantic_index_receipt(raw, &input.obligation)?;
        super::super::semantic_projector::succeed_memory_semantic_projection_claim(
            &store,
            &acquisition.claim.claim_id,
            &input.authority,
            receipt,
            "2026-07-16T00:01:01Z",
        )?;
        let readiness = super::super::load_memory_semantic_projection_readiness(&store, &input)?
            .expect("authenticated readiness");
        let mut advanced = model.clone();
        advanced.model_revision += 1;
        advanced.model_hash = memory_graph_model_hash(&advanced)?;
        let replacement = crate::EpiphanyMemoryGraphEntry::from_snapshot(&advanced)?;
        let mut port = StoreAdvancingPort {
            store: store.clone(),
            compatibility: compatibility(&config, SemanticPartition::Modeling, 2),
            points,
            replacement,
        };
        assert!(
            observe_memory_semantic_live_evidence_with_port(
                &store, &config, &input, &readiness, &mut port,
            )?
            .is_none()
        );
        Ok(())
    }
}
