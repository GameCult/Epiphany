use crate::memory_graph::semantic_index::{
    MemorySemanticIndexConfig, MemorySemanticLiveEvidence, observe_memory_semantic_live_evidence,
};
use crate::semantic_backend::{QdrantBackend, QdrantConfig};
use crate::workspace_coverage_projector::{
    WorkspaceCoverageReadinessEvidence, observe_current_workspace_coverage_evidence,
};
use crate::{
    EpiphanyMemoryGraphEntry, EpiphanyMemoryGraphSnapshot,
    MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION,
    MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION, MemorySemanticIndexReceipt,
    MemorySemanticProjectionAttempt, MemorySemanticProjectionClaim, MemorySemanticProjectionInput,
    MemorySemanticProjectionObligation, MemorySemanticProjectionReadiness,
    MemorySemanticProjectionSourceHead, REPO_MODEL_ADMISSION_CONTRACT,
    REPO_MODEL_ADMISSION_RECEIPT_SCHEMA_VERSION, RepoModelAdmissionReceipt,
    RepositoryBodyObservationBasis, SEMANTIC_PROJECTION_SCHEMA_VERSION, WorkspaceCoveragePolicy,
    load_memory_semantic_projection_readiness, memory_graph_model_hash,
    observe_runtime_repository_body_basis, runtime_modeling_semantic_projection_input,
    runtime_spine_cache, validate_memory_semantic_projection_obligation,
    validate_repository_body_observation_basis,
};
use anyhow::{Result, anyhow, bail};
use cultcache_rs::{
    CacheBackingStore, CultCacheEnvelope, DatabaseEntry, SingleFileMessagePackBackingStore,
};
use sha2::{Digest, Sha256};
use std::path::Path;

pub const REPOSITORY_READINESS_PROJECTION_SCHEMA_VERSION: &str =
    "gamecult.epiphany.repository_readiness_projection.v0";
pub const REPOSITORY_READINESS_PROJECTION_TYPE: &str =
    "gamecult.epiphany.repository_readiness_projection";

/// A historical proof that Mind joined one exact Repository Body observation,
/// canonical RepoModel admission, and both live semantic projections. It is not
/// a current-readiness flag: callers must derive a new projection to claim now.
#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.repository_readiness_projection",
    schema = "RepositoryReadinessProjection"
)]
pub struct RepositoryReadinessProjection {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub projection_id: String,
    #[cultcache(key = 2)]
    pub status: String,
    #[cultcache(key = 3)]
    pub observed_ready_at: String,
    #[cultcache(key = 4)]
    pub workspace_id: String,
    #[cultcache(key = 5)]
    pub body_binding_sha256: String,
    #[cultcache(key = 6)]
    pub body_manifest_root_sha256: String,
    #[cultcache(key = 7)]
    pub body_r1_observation_id: String,
    #[cultcache(key = 8)]
    pub body_r1_generation: u64,
    #[cultcache(key = 9)]
    pub body_r2_observation_id: String,
    #[cultcache(key = 10)]
    pub body_r2_generation: u64,
    #[cultcache(key = 11)]
    pub repo_model_graph_id: String,
    #[cultcache(key = 12)]
    pub repo_model_revision: u64,
    #[cultcache(key = 13)]
    pub repo_model_hash: String,
    #[cultcache(key = 14)]
    pub admission_receipt_id: String,
    #[cultcache(key = 15)]
    pub modeling_obligation_id: String,
    #[cultcache(key = 16)]
    pub modeling_receipt_id: String,
    #[cultcache(key = 17)]
    pub modeling_collection_name: String,
    #[cultcache(key = 18)]
    pub modeling_observed_point_count: u32,
    #[cultcache(key = 19)]
    pub modeling_observed_vector_binding_root_sha256: String,
    #[cultcache(key = 20)]
    pub workspace_policy_id: String,
    #[cultcache(key = 21)]
    pub workspace_policy_sha256: String,
    #[cultcache(key = 22)]
    pub workspace_coverage_receipt_id: String,
    #[cultcache(key = 23)]
    pub workspace_coverage_collection_name: String,
    #[cultcache(key = 24)]
    pub workspace_coverage_observed_point_count: u64,
    #[cultcache(key = 25)]
    pub workspace_coverage_point_binding_set_sha256: String,
    #[cultcache(key = 26)]
    pub workspace_coverage_vector_binding_set_sha256: String,
    #[cultcache(key = 27)]
    pub opening_non_body_authority_sha256: String,
    #[cultcache(key = 28)]
    pub closing_non_body_authority_sha256: String,
    #[cultcache(key = 29)]
    pub contract: String,
    #[cultcache(key = 30)]
    pub swarm_id: String,
    #[cultcache(key = 31)]
    pub runtime_id: String,
    #[cultcache(key = 32)]
    pub body_scope: String,
    #[cultcache(key = 33)]
    pub body_r1: RepositoryBodyObservationBasis,
    #[cultcache(key = 34)]
    pub body_r2: RepositoryBodyObservationBasis,
    #[cultcache(key = 35)]
    pub truth_interval_started_at: String,
    #[cultcache(key = 36)]
    pub truth_interval_closed_at: String,
    #[cultcache(key = 37)]
    pub admission_envelope_sha256: String,
    #[cultcache(key = 38)]
    pub modeling_source_generation: u64,
    #[cultcache(key = 39)]
    pub modeling_canonical_content_set_hash: String,
    #[cultcache(key = 40)]
    pub modeling_claim_id: String,
    #[cultcache(key = 41)]
    pub modeling_claim_epoch: u64,
    #[cultcache(key = 42)]
    pub modeling_embedding_provider_id: String,
    #[cultcache(key = 43)]
    pub modeling_embedding_model: String,
    #[cultcache(key = 44)]
    pub modeling_vector_dimensions: u32,
    #[cultcache(key = 45)]
    pub modeling_receipt_envelope_sha256: String,
    #[cultcache(key = 46)]
    pub modeling_live_evidence_sha256: String,
    #[cultcache(key = 47)]
    pub workspace_obligation_envelope_sha256: String,
    #[cultcache(key = 48)]
    pub workspace_plan_envelope_sha256: String,
    #[cultcache(key = 49)]
    pub workspace_claim_id: String,
    #[cultcache(key = 50)]
    pub workspace_claim_epoch: u64,
    #[cultcache(key = 51)]
    pub workspace_claim_envelope_sha256: String,
    #[cultcache(key = 52)]
    pub workspace_attempt_id: String,
    #[cultcache(key = 53)]
    pub workspace_attempt_envelope_sha256: String,
    #[cultcache(key = 54)]
    pub workspace_receipt_envelope_sha256: String,
    #[cultcache(key = 55)]
    pub workspace_head_envelope_sha256: String,
    #[cultcache(key = 56)]
    pub workspace_observed_point_set_sha256: String,
    #[cultcache(key = 57)]
    pub admission_body_basis: RepositoryBodyObservationBasis,
    #[cultcache(key = 58)]
    pub modeling_obligation_envelope_sha256: String,
    #[cultcache(key = 59)]
    pub modeling_canonical_source_id: String,
    #[cultcache(key = 60)]
    pub modeling_source_commit_id: String,
    #[cultcache(key = 61)]
    pub modeling_source_model_hash: String,
    #[cultcache(key = 62)]
    pub modeling_current_content_set_hash: String,
    #[cultcache(key = 63)]
    pub modeling_claim_envelope_sha256: String,
    #[cultcache(key = 64)]
    pub modeling_attempt_id: String,
    #[cultcache(key = 65)]
    pub modeling_attempt_envelope_sha256: String,
    #[cultcache(key = 66)]
    pub workspace_embedding_provider_id: String,
    #[cultcache(key = 67)]
    pub workspace_embedding_model: String,
    #[cultcache(key = 68)]
    pub workspace_vector_dimensions: u32,
    #[cultcache(key = 69)]
    pub workspace_obligation_id: String,
    #[cultcache(key = 70)]
    pub workspace_plan_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceProjectionIdentity {
    pub policy: WorkspaceCoveragePolicy,
    pub qdrant_url: String,
    pub qdrant_api_key: Option<String>,
    pub qdrant_timeout_ms: u64,
    pub embedding_provider_id: String,
    pub embedding_model: String,
    pub vector_dimensions: u32,
}

#[derive(Clone)]
struct JoinedReadinessEvidence {
    r1: RepositoryBodyObservationBasis,
    r2: RepositoryBodyObservationBasis,
    graph_id: String,
    model_revision: u64,
    model_hash: String,
    admission: RepoModelAdmissionReceipt,
    admission_envelope_sha256: String,
    modeling_obligation_id: String,
    modeling_receipt_id: String,
    modeling_source_generation: u64,
    modeling_canonical_content_set_hash: String,
    modeling_claim_id: String,
    modeling_claim_epoch: u64,
    modeling_embedding_provider_id: String,
    modeling_embedding_model: String,
    modeling_vector_dimensions: u32,
    modeling_receipt_envelope_sha256: String,
    modeling_obligation_envelope_sha256: String,
    modeling_canonical_source_id: String,
    modeling_source_commit_id: String,
    modeling_source_model_hash: String,
    modeling_current_content_set_hash: String,
    modeling_claim_envelope_sha256: String,
    modeling_attempt_id: String,
    modeling_attempt_envelope_sha256: String,
    workspace_identity: WorkspaceProjectionIdentity,
    modeling_live: Option<MemorySemanticLiveEvidence>,
    coverage_live: Option<WorkspaceCoverageReadinessEvidence>,
    opening_authority: Vec<CultCacheEnvelope>,
    closing_authority: Vec<CultCacheEnvelope>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReadinessAppendResult {
    Appended,
    ConcurrentIdentical,
}

/// The I/O boundary around Mind's readiness join.  Implementations may observe
/// and authenticate evidence, but they do not decide whether the evidence is a
/// coherent readiness interval; that decision remains in `observe_with_ports`.
trait ReadinessObservationPorts {
    fn observe_body(&mut self) -> Result<RepositoryBodyObservationBasis>;
    fn observe_store_snapshot(&mut self) -> Result<Vec<CultCacheEnvelope>>;
    fn authenticate_historical_body_basis(
        &mut self,
        basis: &RepositoryBodyObservationBasis,
    ) -> Result<()>;
    fn observe_semantic_state(&mut self) -> Result<RawSemanticObservation>;
    fn observe_semantic_live(&mut self) -> Result<Option<MemorySemanticLiveEvidence>>;
    fn observe_workspace_live(
        &mut self,
        r1: &RepositoryBodyObservationBasis,
        _opening: &JoinedReadinessEvidence,
    ) -> Result<Option<WorkspaceCoverageReadinessEvidence>>;
    fn close_time(&mut self) -> String;
    fn append_projection(
        &mut self,
        snapshot: &[CultCacheEnvelope],
        projection: &RepositoryReadinessProjection,
    ) -> Result<ReadinessAppendResult>;
}

#[derive(Clone, Debug, PartialEq)]
struct RawSemanticObservation {
    snapshot: EpiphanyMemoryGraphSnapshot,
    authority_envelopes: Vec<CultCacheEnvelope>,
    obligation: MemorySemanticProjectionObligation,
    current: MemorySemanticProjectionSourceHead,
    receipt: MemorySemanticIndexReceipt,
}

fn validate_exact_raw_semantic_chain(
    r1: &RepositoryBodyObservationBasis,
    model: &EpiphanyMemoryGraphSnapshot,
    model_hash: &str,
    raw: &RawSemanticObservation,
) -> Result<()> {
    let obligation = &raw.obligation;
    let current = &raw.current;
    let receipt = &raw.receipt;
    validate_memory_semantic_projection_obligation(obligation)?;
    if obligation.schema_version != MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION
        || obligation.projection_schema_version != SEMANTIC_PROJECTION_SCHEMA_VERSION
        || obligation.partition != "modeling"
        || obligation.swarm_id != r1.swarm_id
        || obligation.graph_id != model.graph_id
        || obligation.source_generation != model.model_revision
        || obligation.source_model_hash != model_hash
        || obligation.canonical_source_id.trim().is_empty()
        || obligation.source_commit_id.trim().is_empty()
        || obligation.canonical_content_set_hash.trim().is_empty()
    {
        bail!("raw Modeling obligation does not match the canonical RepoModel");
    }
    let expected_current = MemorySemanticProjectionSourceHead {
        swarm_id: obligation.swarm_id.clone(),
        partition: obligation.partition.clone(),
        canonical_source_id: obligation.canonical_source_id.clone(),
        source_commit_id: obligation.source_commit_id.clone(),
        graph_id: obligation.graph_id.clone(),
        source_generation: obligation.source_generation,
        source_model_hash: obligation.source_model_hash.clone(),
        canonical_content_set_hash: obligation.canonical_content_set_hash.clone(),
    };
    if *current != expected_current {
        bail!("raw Modeling source head does not equal its exact obligation");
    }
    if receipt.schema_version != MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION
        || receipt.status != "ready"
        || receipt.obligation_id != obligation.obligation_id
        || receipt.swarm_id != obligation.swarm_id
        || receipt.partition != obligation.partition
        || receipt.canonical_source_id != obligation.canonical_source_id
        || receipt.source_commit_id != obligation.source_commit_id
        || receipt.graph_id != obligation.graph_id
        || receipt.source_generation != obligation.source_generation
        || receipt.model_revision != model.model_revision
        || receipt.model_hash != model_hash
        || receipt.canonical_content_set_hash != obligation.canonical_content_set_hash
        || receipt.projection_schema_version != obligation.projection_schema_version
        || receipt.collection_name.trim().is_empty()
        || receipt.embedding_provider_id.trim().is_empty()
        || receipt.embedding_model.trim().is_empty()
        || receipt.vector_dimensions == 0
        || receipt.claim_id.trim().is_empty()
        || receipt.claim_epoch == 0
        || receipt.observed_vector_binding_root_sha256.len() != 64
        || !receipt
            .observed_vector_binding_root_sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || chrono::DateTime::parse_from_rfc3339(&receipt.indexed_at).is_err()
    {
        bail!("raw Modeling receipt does not match its exact obligation and RepoModel");
    }
    Ok(())
}

fn observe_with_ports<P: ReadinessObservationPorts>(
    ports: &mut P,
    workspace_identity: &WorkspaceProjectionIdentity,
) -> Result<RepositoryReadinessProjection> {
    let r1 = ports.observe_body()?;
    let opening_snapshot = ports.observe_store_snapshot()?;
    let opening_semantic = ports.observe_semantic_state()?;
    let mut evidence = derive_joined_opening(
        &opening_snapshot,
        &r1,
        &opening_semantic,
        workspace_identity,
    )?;
    ports.authenticate_historical_body_basis(
        evidence
            .admission
            .repository_body_observation_basis
            .as_ref()
            .expect("join selected only Body-grounded admission"),
    )?;
    let modeling_live = ports.observe_semantic_live()?;
    let coverage_live = ports.observe_workspace_live(&r1, &evidence)?;
    let r2 = ports.observe_body()?;
    if !same_body_content(&r1, &r2) {
        bail!("Repository Body changed during readiness observation");
    }
    let closing_semantic = ports.observe_semantic_state()?;
    let closing_modeling_live = ports.observe_semantic_live()?;
    if closing_modeling_live != modeling_live {
        bail!("live Modeling semantic evidence changed during readiness observation");
    }
    // Coverage is deliberately rebound to R1 on both reads. R2 closes the Body
    // interval; it must not silently change the projection being authenticated.
    let closing_coverage_live = ports.observe_workspace_live(&r1, &evidence)?;
    if closing_coverage_live != coverage_live {
        bail!("live workspace coverage evidence changed during readiness observation");
    }
    let closing_snapshot = ports.observe_store_snapshot()?;
    let closing_evidence = derive_joined_opening(
        &closing_snapshot,
        &r1,
        &closing_semantic,
        workspace_identity,
    )?;
    if opening_semantic != closing_semantic || evidence.admission != closing_evidence.admission {
        bail!("non-Body readiness authority advanced during observation");
    }
    evidence.r2 = r2;
    evidence.modeling_live = modeling_live;
    evidence.coverage_live = coverage_live;
    evidence.closing_authority = readiness_authority_from_snapshot(&closing_snapshot);
    let observed_ready_at = ports.close_time();
    let projection = projection_from_join(evidence, &observed_ready_at)?;
    ports.append_projection(&closing_snapshot, &projection)?;
    Ok(projection)
}

fn same_body_content(
    a: &RepositoryBodyObservationBasis,
    b: &RepositoryBodyObservationBasis,
) -> bool {
    a.workspace_id == b.workspace_id
        && a.swarm_id == b.swarm_id
        && a.runtime_id == b.runtime_id
        && a.scope == b.scope
        && a.body_binding_sha256 == b.body_binding_sha256
        && a.manifest_root_sha256 == b.manifest_root_sha256
}

fn select_current_admission(
    receipts: Vec<(String, RepoModelAdmissionReceipt)>,
    model_revision: u64,
    model_hash: &str,
) -> Result<RepoModelAdmissionReceipt> {
    let mut matching = receipts.into_iter().filter(|(_, receipt)| {
        receipt.schema_version == REPO_MODEL_ADMISSION_RECEIPT_SCHEMA_VERSION
            && receipt.contract == REPO_MODEL_ADMISSION_CONTRACT
            && receipt.admitted_revision == model_revision
            && receipt.admitted_hash == model_hash
    });
    let selected = matching.next().ok_or_else(|| {
        anyhow!("repository readiness requires exactly one current RepoModel admission receipt")
    })?;
    if matching.next().is_some() {
        bail!("repository readiness requires exactly one current RepoModel admission receipt");
    }
    if selected.0 != selected.1.receipt_id {
        bail!("current RepoModel admission is stored under an alien key");
    }
    if selected.1.repository_body_observation_basis.is_none() {
        bail!("current RepoModel admission receipt has no authenticated Body basis");
    }
    Ok(selected.1)
}

fn authority_bytes(rows: &[CultCacheEnvelope]) -> Result<Vec<u8>> {
    let mut rows = rows.to_vec();
    rows.sort_by(|a, b| (&a.r#type, &a.key).cmp(&(&b.r#type, &b.key)));
    rmp_serde::to_vec_named(&rows).map_err(Into::into)
}

fn envelope_sha256(envelope: &CultCacheEnvelope) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(rmp_serde::to_vec_named(envelope)?)
    ))
}

fn projection_from_join(
    evidence: JoinedReadinessEvidence,
    observed_ready_at: &str,
) -> Result<RepositoryReadinessProjection> {
    let observed_ready = chrono::DateTime::parse_from_rfc3339(observed_ready_at)
        .map_err(|_| anyhow!("repository readiness observation timestamp must be RFC3339"))?;
    let body_closed = chrono::DateTime::parse_from_rfc3339(&evidence.r2.scan_finished_at)
        .map_err(|_| anyhow!("Repository Body R2 close timestamp must be RFC3339"))?;
    if observed_ready < body_closed {
        bail!("repository readiness close time predates Body R2 observation");
    }
    if !same_body_content(&evidence.r1, &evidence.r2) {
        bail!("Repository Body changed during readiness observation");
    }
    if evidence.admission.admitted_revision != evidence.model_revision
        || evidence.admission.admitted_hash != evidence.model_hash
        || !evidence
            .admission
            .repository_body_observation_basis
            .as_ref()
            .is_some_and(|basis| same_body_content(basis, &evidence.r1))
    {
        bail!("current RepoModel admission is not authenticated by Body R1");
    }
    let modeling = evidence
        .modeling_live
        .ok_or_else(|| anyhow!("live Modeling semantic evidence is absent"))?;
    let coverage = evidence
        .coverage_live
        .ok_or_else(|| anyhow!("live workspace coverage evidence is absent"))?;
    let opening = authority_bytes(&evidence.opening_authority)?;
    let closing = authority_bytes(&evidence.closing_authority)?;
    if opening != closing {
        bail!("non-Body readiness authority advanced during observation");
    }
    let authority_sha256 = format!("{:x}", Sha256::digest(&opening));
    let modeling_live_sha256 = format!(
        "{:x}",
        Sha256::digest(rmp_serde::to_vec_named(&(
            &modeling.collection_name,
            modeling.observed_point_count,
            &modeling.observed_vector_binding_root_sha256,
        ))?)
    );
    let r1 = evidence.r1.clone();
    let r2 = evidence.r2.clone();
    let identity = rmp_serde::to_vec_named(&(
        &evidence.r1.workspace_id,
        &evidence.r1.body_binding_sha256,
        &evidence.r1.manifest_root_sha256,
        &evidence.graph_id,
        evidence.model_revision,
        &evidence.model_hash,
        &evidence.admission.receipt_id,
        &evidence.modeling_obligation_id,
        &evidence.modeling_receipt_id,
        &modeling.observed_vector_binding_root_sha256,
        &coverage.receipt_id,
        &coverage.observed_point_binding_set_sha256,
        &coverage.observed_vector_binding_set_sha256,
        observed_ready_at,
    ))?;
    Ok(RepositoryReadinessProjection {
        schema_version: REPOSITORY_READINESS_PROJECTION_SCHEMA_VERSION.into(),
        projection_id: format!("repository-readiness-{:x}", Sha256::digest(identity)),
        status: "observed_ready_at".into(),
        observed_ready_at: observed_ready_at.into(),
        workspace_id: evidence.r1.workspace_id,
        body_binding_sha256: evidence.r1.body_binding_sha256,
        body_manifest_root_sha256: evidence.r1.manifest_root_sha256,
        body_r1_observation_id: evidence.r1.observation_id,
        body_r1_generation: evidence.r1.generation,
        body_r2_observation_id: evidence.r2.observation_id,
        body_r2_generation: evidence.r2.generation,
        repo_model_graph_id: evidence.graph_id,
        repo_model_revision: evidence.model_revision,
        repo_model_hash: evidence.model_hash,
        admission_receipt_id: evidence.admission.receipt_id,
        modeling_obligation_id: evidence.modeling_obligation_id,
        modeling_receipt_id: evidence.modeling_receipt_id,
        modeling_collection_name: modeling.collection_name,
        modeling_observed_point_count: modeling.observed_point_count,
        modeling_observed_vector_binding_root_sha256: modeling.observed_vector_binding_root_sha256,
        workspace_policy_id: coverage.policy_id,
        workspace_policy_sha256: coverage.policy_sha256,
        workspace_coverage_receipt_id: coverage.receipt_id,
        workspace_coverage_collection_name: coverage.collection_name,
        workspace_coverage_observed_point_count: coverage.observed_point_count,
        workspace_coverage_point_binding_set_sha256: coverage.observed_point_binding_set_sha256,
        workspace_coverage_vector_binding_set_sha256: coverage.observed_vector_binding_set_sha256,
        opening_non_body_authority_sha256: authority_sha256.clone(),
        closing_non_body_authority_sha256: authority_sha256,
        contract: "Historical Mind observation only. Current readiness must be rederived from fresh Body and live projection evidence.".into(),
        swarm_id: r1.swarm_id.clone(),
        runtime_id: r1.runtime_id.clone(),
        body_scope: r1.scope.clone(),
        truth_interval_started_at: r1.scan_finished_at.clone(),
        truth_interval_closed_at: observed_ready_at.into(),
        body_r1: r1,
        body_r2: r2,
        admission_envelope_sha256: evidence.admission_envelope_sha256,
        modeling_source_generation: evidence.modeling_source_generation,
        modeling_canonical_content_set_hash: evidence.modeling_canonical_content_set_hash,
        modeling_claim_id: evidence.modeling_claim_id,
        modeling_claim_epoch: evidence.modeling_claim_epoch,
        modeling_embedding_provider_id: evidence.modeling_embedding_provider_id,
        modeling_embedding_model: evidence.modeling_embedding_model,
        modeling_vector_dimensions: evidence.modeling_vector_dimensions,
        modeling_receipt_envelope_sha256: evidence.modeling_receipt_envelope_sha256,
        modeling_live_evidence_sha256: modeling_live_sha256,
        workspace_obligation_envelope_sha256: coverage.obligation_envelope_digest,
        workspace_plan_envelope_sha256: coverage.plan_envelope_digest,
        workspace_claim_id: coverage.claim_id,
        workspace_claim_epoch: coverage.claim_epoch,
        workspace_claim_envelope_sha256: coverage.claim_envelope_digest,
        workspace_attempt_id: coverage.attempt_id,
        workspace_attempt_envelope_sha256: coverage.attempt_envelope_digest,
        workspace_receipt_envelope_sha256: coverage.receipt_envelope_digest,
        workspace_head_envelope_sha256: coverage.head_envelope_digest,
        workspace_observed_point_set_sha256: coverage.observed_point_set_sha256,
        admission_body_basis: evidence
            .admission
            .repository_body_observation_basis
            .clone()
            .expect("join authenticated admission Body basis"),
        modeling_obligation_envelope_sha256: evidence.modeling_obligation_envelope_sha256,
        modeling_canonical_source_id: evidence.modeling_canonical_source_id,
        modeling_source_commit_id: evidence.modeling_source_commit_id,
        modeling_source_model_hash: evidence.modeling_source_model_hash,
        modeling_current_content_set_hash: evidence.modeling_current_content_set_hash,
        modeling_claim_envelope_sha256: evidence.modeling_claim_envelope_sha256,
        modeling_attempt_id: evidence.modeling_attempt_id,
        modeling_attempt_envelope_sha256: evidence.modeling_attempt_envelope_sha256,
        workspace_embedding_provider_id: evidence.workspace_identity.embedding_provider_id,
        workspace_embedding_model: evidence.workspace_identity.embedding_model,
        workspace_vector_dimensions: evidence.workspace_identity.vector_dimensions,
        workspace_obligation_id: coverage.obligation_id,
        workspace_plan_id: coverage.plan_id,
    })
}

fn readiness_authority_from_snapshot(rows: &[CultCacheEnvelope]) -> Vec<CultCacheEnvelope> {
    rows.iter()
        .cloned()
        .filter(|row| {
            row.r#type == crate::EpiphanyRuntimeSwarmBinding::TYPE
                || row.r#type == EpiphanyMemoryGraphEntry::TYPE
                || row.r#type == RepoModelAdmissionReceipt::TYPE
                || row.r#type == MemorySemanticProjectionObligation::TYPE
                || row.r#type == MemorySemanticProjectionClaim::TYPE
                || row.r#type == MemorySemanticProjectionAttempt::TYPE
                || row.r#type == MemorySemanticIndexReceipt::TYPE
        })
        .collect()
}

fn decode_keyed_rows<T: DatabaseEntry>(rows: &[CultCacheEnvelope]) -> Result<Vec<(String, T)>> {
    rows.iter()
        .filter(|row| row.r#type == T::TYPE)
        .map(|row| Ok((row.key.clone(), rmp_serde::from_slice(&row.payload)?)))
        .collect()
}

fn exact_envelope<'a, T: DatabaseEntry>(
    rows: &'a [CultCacheEnvelope],
    key: &str,
) -> Result<&'a CultCacheEnvelope> {
    rows.iter()
        .find(|row| row.r#type == T::TYPE && row.key == key)
        .ok_or_else(|| anyhow!("authority is missing {} envelope {key}", T::TYPE))
}

fn derive_joined_opening(
    snapshot: &[CultCacheEnvelope],
    r1: &RepositoryBodyObservationBasis,
    semantic: &RawSemanticObservation,
    workspace_identity: &WorkspaceProjectionIdentity,
) -> Result<JoinedReadinessEvidence> {
    let model_entry: EpiphanyMemoryGraphEntry = rmp_serde::from_slice(
        &exact_envelope::<EpiphanyMemoryGraphEntry>(snapshot, crate::MEMORY_GRAPH_KEY)?.payload,
    )?;
    let model = model_entry.snapshot()?;
    let model_hash = memory_graph_model_hash(&model)?;
    validate_exact_raw_semantic_chain(r1, &model, &model_hash, semantic)?;
    let admission = select_current_admission(
        decode_keyed_rows::<RepoModelAdmissionReceipt>(snapshot)?,
        model.model_revision,
        &model_hash,
    )?;
    if !admission
        .repository_body_observation_basis
        .as_ref()
        .is_some_and(|basis| same_body_content(basis, r1))
    {
        bail!("current RepoModel admission does not authenticate Body R1");
    }
    if semantic.snapshot != model {
        bail!("Modeling semantic input is not the canonical RepoModel");
    }
    for consumed in &semantic.authority_envelopes {
        if !snapshot.contains(consumed) {
            bail!("Modeling semantic authority is outside the exact store snapshot");
        }
    }
    let receipt_envelope =
        exact_envelope::<MemorySemanticIndexReceipt>(snapshot, &semantic.receipt.receipt_id)?;
    let receipt: MemorySemanticIndexReceipt = rmp_serde::from_slice(&receipt_envelope.payload)?;
    let obligation_envelope = exact_envelope::<MemorySemanticProjectionObligation>(
        snapshot,
        &semantic.obligation.obligation_id,
    )?;
    let obligation: MemorySemanticProjectionObligation =
        rmp_serde::from_slice(&obligation_envelope.payload)?;
    if receipt != semantic.receipt || obligation != semantic.obligation {
        bail!("Modeling semantic readiness is outside the exact store snapshot");
    }
    let claims = decode_keyed_rows::<MemorySemanticProjectionClaim>(snapshot)?
        .into_iter()
        .filter(|(_, claim)| claim.claim_id == receipt.claim_id)
        .collect::<Vec<_>>();
    if claims.len() != 1 {
        bail!("authority requires exactly one current semantic claim");
    }
    let (claim_key, claim) = claims.into_iter().next().expect("one claim");
    if claim_key != claim.scope_id {
        bail!("semantic claim is stored under an alien key");
    }
    let claim_envelope =
        exact_envelope::<MemorySemanticProjectionClaim>(snapshot, &claim.scope_id)?;
    let attempt_envelope =
        exact_envelope::<MemorySemanticProjectionAttempt>(snapshot, &claim.attempt_id)?;
    let attempt: MemorySemanticProjectionAttempt =
        rmp_serde::from_slice(&attempt_envelope.payload)?;
    if attempt.attempt_id != claim.attempt_id
        || claim.status != "succeeded"
        || claim.obligation_id != obligation.obligation_id
        || claim.epoch != receipt.claim_epoch
        || attempt.status != "succeeded"
        || attempt.claim_id != claim.claim_id
        || attempt.claim_epoch != claim.epoch
        || attempt.obligation_id != claim.obligation_id
    {
        bail!("semantic claim/attempt chain is not the authenticated success chain");
    }
    Ok(JoinedReadinessEvidence {
        r1: r1.clone(),
        r2: r1.clone(),
        graph_id: model.graph_id,
        model_revision: model.model_revision,
        model_hash,
        admission_envelope_sha256: envelope_sha256(exact_envelope::<RepoModelAdmissionReceipt>(
            snapshot,
            &admission.receipt_id,
        )?)?,
        admission,
        modeling_obligation_id: obligation.obligation_id.clone(),
        modeling_receipt_id: receipt.receipt_id.clone(),
        modeling_source_generation: receipt.source_generation,
        modeling_canonical_content_set_hash: receipt.canonical_content_set_hash.clone(),
        modeling_claim_id: receipt.claim_id.clone(),
        modeling_claim_epoch: receipt.claim_epoch,
        modeling_embedding_provider_id: receipt.embedding_provider_id.clone(),
        modeling_embedding_model: receipt.embedding_model.clone(),
        modeling_vector_dimensions: receipt.vector_dimensions,
        modeling_receipt_envelope_sha256: envelope_sha256(receipt_envelope)?,
        modeling_obligation_envelope_sha256: envelope_sha256(obligation_envelope)?,
        modeling_canonical_source_id: obligation.canonical_source_id,
        modeling_source_commit_id: obligation.source_commit_id,
        modeling_source_model_hash: obligation.source_model_hash,
        modeling_current_content_set_hash: semantic.current.canonical_content_set_hash.clone(),
        modeling_claim_envelope_sha256: envelope_sha256(claim_envelope)?,
        modeling_attempt_id: claim.attempt_id,
        modeling_attempt_envelope_sha256: envelope_sha256(attempt_envelope)?,
        workspace_identity: workspace_identity.clone(),
        modeling_live: None,
        coverage_live: None,
        opening_authority: readiness_authority_from_snapshot(snapshot),
        closing_authority: Vec::new(),
    })
}

fn append_readiness_envelope_if_snapshot_unchanged(
    backing: &SingleFileMessagePackBackingStore,
    snapshot: &[CultCacheEnvelope],
    envelope: CultCacheEnvelope,
) -> Result<ReadinessAppendResult> {
    if backing.append_if_snapshot_unchanged(snapshot, vec![envelope.clone()])? {
        return Ok(ReadinessAppendResult::Appended);
    }
    let mut actual = backing.pull_all()?;
    let candidates = actual
        .iter()
        .enumerate()
        .filter(|(_, row)| row.r#type == envelope.r#type && row.key == envelope.key)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if candidates.len() == 1 {
        let winner = actual.remove(candidates[0]);
        actual.sort_by(|a, b| (&a.r#type, &a.key).cmp(&(&b.r#type, &b.key)));
        let mut expected_baseline = snapshot.to_vec();
        expected_baseline.sort_by(|a, b| (&a.r#type, &a.key).cmp(&(&b.r#type, &b.key)));
        if actual == expected_baseline && winner.payload == envelope.payload {
            return Ok(ReadinessAppendResult::ConcurrentIdentical);
        }
    }
    bail!("repository readiness authority changed before atomic projection append")
}

struct ProductionReadinessPorts<'a> {
    runtime_store: &'a Path,
    semantic_config: &'a MemorySemanticIndexConfig,
    workspace_identity: &'a WorkspaceProjectionIdentity,
    close_clock: Option<Box<dyn FnOnce() -> String + 'a>>,
    pending_semantic: Option<(
        MemorySemanticProjectionInput,
        MemorySemanticProjectionReadiness,
    )>,
}

impl ReadinessObservationPorts for ProductionReadinessPorts<'_> {
    fn observe_body(&mut self) -> Result<RepositoryBodyObservationBasis> {
        observe_runtime_repository_body_basis(self.runtime_store)
    }

    fn observe_store_snapshot(&mut self) -> Result<Vec<CultCacheEnvelope>> {
        SingleFileMessagePackBackingStore::new(self.runtime_store)
            .pull_all()
            .map_err(Into::into)
    }

    fn authenticate_historical_body_basis(
        &mut self,
        basis: &RepositoryBodyObservationBasis,
    ) -> Result<()> {
        validate_repository_body_observation_basis(self.runtime_store, basis)
    }

    fn observe_semantic_state(&mut self) -> Result<RawSemanticObservation> {
        let input = runtime_modeling_semantic_projection_input(self.runtime_store)?;
        let readiness = load_memory_semantic_projection_readiness(self.runtime_store, &input)?
            .ok_or_else(|| anyhow!("current Modeling semantic projection is not ready"))?;
        let raw = RawSemanticObservation {
            snapshot: input.snapshot().clone(),
            authority_envelopes: input.authority.envelopes.clone(),
            obligation: readiness.obligation.clone(),
            current: readiness.current.clone(),
            receipt: readiness.receipt.clone(),
        };
        self.pending_semantic = Some((input, readiness));
        Ok(raw)
    }

    fn observe_semantic_live(&mut self) -> Result<Option<MemorySemanticLiveEvidence>> {
        let (input, readiness) = self.pending_semantic.take().ok_or_else(|| {
            anyhow!("semantic live observation requires raw semantic authority first")
        })?;
        observe_memory_semantic_live_evidence(
            self.runtime_store,
            self.semantic_config,
            &input,
            &readiness,
        )
    }

    fn observe_workspace_live(
        &mut self,
        r1: &RepositoryBodyObservationBasis,
        _opening: &JoinedReadinessEvidence,
    ) -> Result<Option<WorkspaceCoverageReadinessEvidence>> {
        let mut backend = QdrantBackend::new(QdrantConfig {
            url: self.workspace_identity.qdrant_url.clone(),
            api_key: self.workspace_identity.qdrant_api_key.clone(),
            timeout_ms: self.workspace_identity.qdrant_timeout_ms,
        })?;
        observe_current_workspace_coverage_evidence(
            self.runtime_store,
            r1,
            &self.workspace_identity.policy,
            &self.workspace_identity.embedding_provider_id,
            &self.workspace_identity.embedding_model,
            self.workspace_identity.vector_dimensions,
            &mut backend,
        )
    }

    fn close_time(&mut self) -> String {
        self.close_clock.take().expect("close clock is called once")()
    }

    fn append_projection(
        &mut self,
        snapshot: &[CultCacheEnvelope],
        projection: &RepositoryReadinessProjection,
    ) -> Result<ReadinessAppendResult> {
        let mut cache = runtime_spine_cache(self.runtime_store)?;
        cache.register_entry_type::<RepositoryReadinessProjection>()?;
        cache.pull_all_backing_stores()?;
        let (envelope, _) = cache.prepare_entry(&projection.projection_id, projection)?;
        let backing = SingleFileMessagePackBackingStore::new(self.runtime_store);
        append_readiness_envelope_if_snapshot_unchanged(&backing, snapshot, envelope)
    }
}

/// Derives and immutably records one Mind-owned whole-repository readiness
/// observation. The returned document is a receipt, never reusable admission.
pub fn observe_repository_readiness(
    runtime_store: impl AsRef<Path>,
    semantic_config: &MemorySemanticIndexConfig,
    workspace_identity: &WorkspaceProjectionIdentity,
) -> Result<RepositoryReadinessProjection> {
    observe_repository_readiness_with_clock(
        runtime_store.as_ref(),
        semantic_config,
        workspace_identity,
        || chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    )
}

fn observe_repository_readiness_with_clock(
    runtime_store: &Path,
    semantic_config: &MemorySemanticIndexConfig,
    workspace_identity: &WorkspaceProjectionIdentity,
    close_clock: impl FnOnce() -> String,
) -> Result<RepositoryReadinessProjection> {
    let mut ports = ProductionReadinessPorts {
        runtime_store,
        semantic_config,
        workspace_identity,
        close_clock: Some(Box::new(close_clock)),
        pending_semantic: None,
    };
    observe_with_ports(&mut ports, workspace_identity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_state_model::RepoModelPatchPurpose;

    #[derive(Clone)]
    struct FakePorts {
        calls: Vec<&'static str>,
        bodies: Vec<RepositoryBodyObservationBasis>,
        snapshots: Vec<Vec<CultCacheEnvelope>>,
        semantics: Vec<RawSemanticObservation>,
        semantic_live: Vec<Option<MemorySemanticLiveEvidence>>,
        coverage: Option<WorkspaceCoverageReadinessEvidence>,
        closing_coverage: Option<WorkspaceCoverageReadinessEvidence>,
        semantic_reads: usize,
        coverage_reads: usize,
        append: Result<ReadinessAppendResult, &'static str>,
        published: usize,
        coverage_body_ids: Vec<String>,
        body_basis_auth: Result<(), &'static str>,
    }

    impl ReadinessObservationPorts for FakePorts {
        fn observe_body(&mut self) -> Result<RepositoryBodyObservationBasis> {
            self.calls
                .push(if self.bodies.len() == 2 { "r1" } else { "r2" });
            Ok(self.bodies.remove(0))
        }

        fn observe_store_snapshot(&mut self) -> Result<Vec<CultCacheEnvelope>> {
            self.calls.push(if self.snapshots.len() == 2 {
                "opening"
            } else {
                "closing-snapshot"
            });
            Ok(self.snapshots.remove(0))
        }

        fn authenticate_historical_body_basis(
            &mut self,
            _basis: &RepositoryBodyObservationBasis,
        ) -> Result<()> {
            self.calls.push("authenticate-admission-body");
            self.body_basis_auth.map_err(|message| anyhow!(message))
        }

        fn observe_semantic_state(&mut self) -> Result<RawSemanticObservation> {
            self.calls.push(if self.semantic_reads == 0 {
                "semantic-state"
            } else {
                "closing-semantic-state"
            });
            Ok(self.semantics.remove(0))
        }

        fn observe_semantic_live(&mut self) -> Result<Option<MemorySemanticLiveEvidence>> {
            self.calls.push(if self.semantic_reads == 0 {
                "semantic"
            } else {
                "closing-semantic"
            });
            self.semantic_reads += 1;
            Ok(self.semantic_live.remove(0))
        }

        fn observe_workspace_live(
            &mut self,
            r1: &RepositoryBodyObservationBasis,
            _opening: &JoinedReadinessEvidence,
        ) -> Result<Option<WorkspaceCoverageReadinessEvidence>> {
            self.calls.push(if self.coverage_reads == 0 {
                "coverage"
            } else {
                "closing-coverage"
            });
            self.coverage_body_ids.push(r1.observation_id.clone());
            let value = if self.coverage_reads == 0 {
                self.coverage.clone()
            } else {
                self.closing_coverage.clone()
            };
            self.coverage_reads += 1;
            Ok(value)
        }

        fn close_time(&mut self) -> String {
            self.calls.push("clock");
            "2026-07-16T01:00:00Z".into()
        }

        fn append_projection(
            &mut self,
            _snapshot: &[CultCacheEnvelope],
            _projection: &RepositoryReadinessProjection,
        ) -> Result<ReadinessAppendResult> {
            self.calls.push("append");
            match self.append {
                Ok(ref result) => {
                    self.published += 1;
                    Ok(result.clone())
                }
                Err(message) => bail!(message),
            }
        }
    }

    fn basis(observation: &str, generation: u64, root: &str) -> RepositoryBodyObservationBasis {
        RepositoryBodyObservationBasis {
            schema_version: "gamecult.epiphany.repository_body_observation_basis.v0".into(),
            workspace_id: "workspace".into(),
            swarm_id: "swarm".into(),
            runtime_id: "runtime".into(),
            scope: "whole_repository".into(),
            body_binding_sha256: "binding".into(),
            observation_id: observation.into(),
            generation,
            manifest_root_sha256: root.into(),
            scan_started_at: "2026-07-16T00:00:00Z".into(),
            scan_finished_at: "2026-07-16T00:00:01Z".into(),
        }
    }

    fn admission(body: RepositoryBodyObservationBasis) -> RepoModelAdmissionReceipt {
        RepoModelAdmissionReceipt {
            schema_version: REPO_MODEL_ADMISSION_RECEIPT_SCHEMA_VERSION.into(),
            receipt_id: "admission".into(),
            review_id: "review".into(),
            result_id: "result".into(),
            patch_id: "patch".into(),
            patch_sha256: "patch-hash".into(),
            previous_revision: 6,
            previous_hash: "previous".into(),
            admitted_revision: 7,
            admitted_hash: "model-hash".into(),
            admitted_at: "2026-07-16T00:00:00Z".into(),
            contract: REPO_MODEL_ADMISSION_CONTRACT.into(),
            purpose: RepoModelPatchPurpose::Evolution,
            frontier_route_id: String::new(),
            verification_request_id: String::new(),
            soul_verdict_receipt_id: String::new(),
            frontier_modeling_request_id: String::new(),
            proposal_modeling_request_id: String::new(),
            claim_repair_request_id: String::new(),
            frontier_plan_decision_id: String::new(),
            repository_body_observation_basis: Some(body),
        }
    }

    fn envelope<T: DatabaseEntry>(key: &str, value: &T) -> CultCacheEnvelope {
        CultCacheEnvelope {
            key: key.into(),
            r#type: T::TYPE.into(),
            payload: rmp_serde::to_vec_named(value).unwrap(),
            stored_at: "2026-07-16T00:00:00Z".into(),
            schema_id: None,
        }
    }

    fn raw_fixture(
        body: RepositoryBodyObservationBasis,
    ) -> Result<(Vec<CultCacheEnvelope>, RawSemanticObservation)> {
        let mut model = EpiphanyMemoryGraphSnapshot {
            graph_id: "graph".into(),
            model_revision: 7,
            ..Default::default()
        };
        model.model_hash = memory_graph_model_hash(&model)?;
        let model_entry = EpiphanyMemoryGraphEntry::from_snapshot(&model)?;
        let mut admitted = admission(body);
        admitted.admitted_hash = model.model_hash.clone();
        let obligation = MemorySemanticProjectionObligation {
            schema_version: MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION.into(),
            obligation_id: "semantic-obligation".into(),
            swarm_id: "swarm".into(),
            partition: "modeling".into(),
            canonical_source_id: "source".into(),
            source_commit_id: "commit".into(),
            graph_id: model.graph_id.clone(),
            source_generation: model.model_revision,
            source_model_hash: model.model_hash.clone(),
            canonical_content_set_hash: "semantic-content".into(),
            projection_schema_version: SEMANTIC_PROJECTION_SCHEMA_VERSION.into(),
            created_at: "2026-07-16T00:00:00Z".into(),
        };
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
            schema_version: MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION.into(),
            receipt_id: "semantic-receipt".into(),
            swarm_id: "swarm".into(),
            partition: "modeling".into(),
            collection_name: "modeling".into(),
            graph_id: model.graph_id.clone(),
            model_revision: 7,
            model_hash: model.model_hash.clone(),
            embedding_provider_id: "semantic-provider".into(),
            embedding_model: "semantic-model".into(),
            vector_dimensions: 1024,
            indexed_document_count: 3,
            deleted_document_count: 0,
            canonical_content_set_hash: obligation.canonical_content_set_hash.clone(),
            indexed_at: "2026-07-16T00:00:00Z".into(),
            status: "ready".into(),
            obligation_id: obligation.obligation_id.clone(),
            canonical_source_id: obligation.canonical_source_id.clone(),
            source_commit_id: obligation.source_commit_id.clone(),
            source_generation: 7,
            projection_schema_version: obligation.projection_schema_version.clone(),
            claim_id: "semantic-claim".into(),
            claim_epoch: 1,
            observed_vector_binding_root_sha256: "0".repeat(64),
        };
        let claim = MemorySemanticProjectionClaim {
            schema_version: "claim-v0".into(),
            scope_id: "modeling".into(),
            claim_id: receipt.claim_id.clone(),
            obligation_id: obligation.obligation_id.clone(),
            attempt_id: "semantic-attempt".into(),
            executor_id: "executor".into(),
            epoch: 1,
            status: "succeeded".into(),
            claimed_at: "2026-07-16T00:00:00Z".into(),
            completed_at: Some("2026-07-16T00:00:01Z".into()),
            executor_incarnation: "incarnation".into(),
            authority_kind: "test".into(),
            authority_id: "authority".into(),
        };
        let attempt = MemorySemanticProjectionAttempt {
            schema_version: "attempt-v0".into(),
            attempt_id: claim.attempt_id.clone(),
            obligation_id: obligation.obligation_id.clone(),
            started_at: "2026-07-16T00:00:00Z".into(),
            completed_at: Some("2026-07-16T00:00:01Z".into()),
            status: "succeeded".into(),
            error: None,
            claim_id: claim.claim_id.clone(),
            claim_epoch: claim.epoch,
            executor_id: claim.executor_id.clone(),
            executor_incarnation: claim.executor_incarnation.clone(),
            authority_id: claim.authority_id.clone(),
        };
        let rows = vec![
            envelope(crate::MEMORY_GRAPH_KEY, &model_entry),
            envelope(&admitted.receipt_id, &admitted),
            envelope(&obligation.obligation_id, &obligation),
            envelope(&receipt.receipt_id, &receipt),
            envelope(&claim.scope_id, &claim),
            envelope(&attempt.attempt_id, &attempt),
        ];
        let raw = RawSemanticObservation {
            snapshot: model,
            authority_envelopes: rows.clone(),
            obligation,
            current,
            receipt,
        };
        Ok((rows, raw))
    }

    fn keyed(receipts: Vec<RepoModelAdmissionReceipt>) -> Vec<(String, RepoModelAdmissionReceipt)> {
        receipts
            .into_iter()
            .map(|receipt| (receipt.receipt_id.clone(), receipt))
            .collect()
    }

    fn joined(
        r1: RepositoryBodyObservationBasis,
        r2: RepositoryBodyObservationBasis,
    ) -> JoinedReadinessEvidence {
        JoinedReadinessEvidence {
            admission: admission(r1.clone()),
            admission_envelope_sha256: "admission-envelope".into(),
            r1,
            r2,
            graph_id: "graph".into(),
            model_revision: 7,
            model_hash: "model-hash".into(),
            modeling_obligation_id: "semantic-obligation".into(),
            modeling_receipt_id: "semantic-receipt".into(),
            modeling_source_generation: 7,
            modeling_canonical_content_set_hash: "semantic-content".into(),
            modeling_claim_id: "semantic-claim".into(),
            modeling_claim_epoch: 1,
            modeling_embedding_provider_id: "semantic-provider".into(),
            modeling_embedding_model: "semantic-model".into(),
            modeling_vector_dimensions: 1024,
            modeling_receipt_envelope_sha256: "semantic-receipt-envelope".into(),
            modeling_obligation_envelope_sha256: "semantic-obligation-envelope".into(),
            modeling_canonical_source_id: "source".into(),
            modeling_source_commit_id: "commit".into(),
            modeling_source_model_hash: "model-hash".into(),
            modeling_current_content_set_hash: "semantic-content".into(),
            modeling_claim_envelope_sha256: "semantic-claim-envelope".into(),
            modeling_attempt_id: "semantic-attempt".into(),
            modeling_attempt_envelope_sha256: "semantic-attempt-envelope".into(),
            workspace_identity: WorkspaceProjectionIdentity {
                policy: WorkspaceCoveragePolicy::bounded_regular_files_v0(1024).unwrap(),
                qdrant_url: "http://workspace-qdrant".into(),
                qdrant_api_key: None,
                qdrant_timeout_ms: 1000,
                embedding_provider_id: "workspace-provider".into(),
                embedding_model: "workspace-model".into(),
                vector_dimensions: 1024,
            },
            modeling_live: Some(MemorySemanticLiveEvidence {
                collection_name: "modeling".into(),
                observed_point_count: 3,
                observed_vector_binding_root_sha256: "semantic-root".into(),
            }),
            coverage_live: Some(WorkspaceCoverageReadinessEvidence {
                workspace_id: "workspace".into(),
                body_binding_sha256: "binding".into(),
                manifest_root_sha256: "root-a".into(),
                policy_id: "policy".into(),
                policy_sha256: "policy-hash".into(),
                obligation_id: "coverage-obligation".into(),
                obligation_envelope_digest: "a".into(),
                plan_id: "plan".into(),
                plan_envelope_digest: "b".into(),
                claim_id: "claim".into(),
                claim_epoch: 1,
                claim_envelope_digest: "c".into(),
                attempt_id: "attempt".into(),
                attempt_envelope_digest: "d".into(),
                receipt_id: "coverage-receipt".into(),
                receipt_envelope_digest: "e".into(),
                head_envelope_digest: "f".into(),
                collection_name: "coverage".into(),
                observed_point_count: 9,
                observed_point_set_sha256: "points".into(),
                observed_point_binding_set_sha256: "bindings".into(),
                observed_vector_binding_set_sha256: "vectors".into(),
            }),
            opening_authority: Vec::new(),
            closing_authority: Vec::new(),
        }
    }

    fn fake_ports(
        r1: RepositoryBodyObservationBasis,
        r2: RepositoryBodyObservationBasis,
    ) -> Result<FakePorts> {
        let opening = joined(r1.clone(), r1.clone());
        let (rows, raw) = raw_fixture(r1.clone())?;
        Ok(FakePorts {
            calls: Vec::new(),
            bodies: vec![r1, r2],
            snapshots: vec![rows.clone(), rows],
            semantics: vec![raw.clone(), raw],
            semantic_live: vec![opening.modeling_live.clone(), opening.modeling_live.clone()],
            coverage: opening.coverage_live.clone(),
            closing_coverage: opening.coverage_live.clone(),
            semantic_reads: 0,
            coverage_reads: 0,
            append: Ok(ReadinessAppendResult::Appended),
            published: 0,
            coverage_body_ids: Vec::new(),
            body_basis_auth: Ok(()),
        })
    }

    #[test]
    fn orchestration_uses_exact_two_read_interval_and_r1_bound_coverage() -> Result<()> {
        let r1 = basis("observation-a1", 1, "root-a");
        let r2 = basis("observation-a2", 3, "root-a");
        let mut ports = fake_ports(r1.clone(), r2)?;
        let identity = joined(r1.clone(), r1.clone()).workspace_identity;
        let projection = observe_with_ports(&mut ports, &identity)?;
        assert_eq!(projection.body_r1_observation_id, "observation-a1");
        assert_eq!(projection.body_r2_observation_id, "observation-a2");
        assert_eq!(
            ports.coverage_body_ids,
            vec![r1.observation_id.clone(), r1.observation_id]
        );
        assert_eq!(
            ports.calls,
            vec![
                "r1",
                "opening",
                "semantic-state",
                "authenticate-admission-body",
                "semantic",
                "coverage",
                "r2",
                "closing-semantic-state",
                "closing-semantic",
                "closing-coverage",
                "closing-snapshot",
                "clock",
                "append"
            ]
        );
        Ok(())
    }

    #[test]
    fn body_or_closing_authority_drift_never_reaches_append() {
        let r1 = basis("a1", 1, "root-a");
        let mut body_drift = fake_ports(r1.clone(), basis("b", 2, "root-b")).unwrap();
        let identity = joined(r1.clone(), r1.clone()).workspace_identity;
        assert!(observe_with_ports(&mut body_drift, &identity).is_err());
        assert!(!body_drift.calls.contains(&"append"));

        let mut authority_drift = fake_ports(r1.clone(), basis("a2", 2, "root-a")).unwrap();
        authority_drift.snapshots[1].push(CultCacheEnvelope {
            key: "alien".into(),
            r#type: RepoModelAdmissionReceipt::TYPE.into(),
            payload: vec![1],
            stored_at: "2026-07-16T00:00:02Z".into(),
            schema_id: None,
        });
        assert!(observe_with_ports(&mut authority_drift, &identity).is_err());
        assert!(!authority_drift.calls.contains(&"append"));
    }

    #[test]
    fn raw_opening_stale_or_alien_admission_is_rejected_before_live_queries() {
        let r1 = basis("a1", 1, "root-a");
        let identity = joined(r1.clone(), r1.clone()).workspace_identity;
        let mut alien = fake_ports(r1.clone(), basis("a2", 2, "root-a")).unwrap();
        alien.snapshots[0]
            .iter_mut()
            .find(|row| row.r#type == RepoModelAdmissionReceipt::TYPE)
            .unwrap()
            .key = "alien".into();
        assert!(observe_with_ports(&mut alien, &identity).is_err());
        assert!(!alien.calls.contains(&"semantic"));

        let mut stale = fake_ports(r1.clone(), basis("a2", 2, "root-a")).unwrap();
        let row = stale.snapshots[0]
            .iter_mut()
            .find(|row| row.r#type == RepoModelAdmissionReceipt::TYPE)
            .unwrap();
        let mut receipt: RepoModelAdmissionReceipt = rmp_serde::from_slice(&row.payload).unwrap();
        receipt.admitted_revision -= 1;
        row.payload = rmp_serde::to_vec_named(&receipt).unwrap();
        assert!(observe_with_ports(&mut stale, &identity).is_err());
        assert!(!stale.calls.contains(&"semantic"));
    }

    #[test]
    fn raw_semantic_cross_field_forgery_is_rejected_before_live_query() {
        let r1 = basis("a1", 1, "root-a");
        let identity = joined(r1.clone(), r1.clone()).workspace_identity;
        let mut head_forgery = fake_ports(r1.clone(), basis("a2", 2, "root-a")).unwrap();
        head_forgery.semantics[0].current.canonical_content_set_hash = "alien-content".into();
        assert!(observe_with_ports(&mut head_forgery, &identity).is_err());
        assert!(!head_forgery.calls.contains(&"semantic"));

        let mut receipt_forgery = fake_ports(r1.clone(), basis("a2", 2, "root-a")).unwrap();
        receipt_forgery.semantics[0].receipt.source_commit_id = "alien-commit".into();
        assert!(observe_with_ports(&mut receipt_forgery, &identity).is_err());
        assert!(!receipt_forgery.calls.contains(&"semantic"));
    }

    #[test]
    fn unauthenticated_historical_admission_basis_is_rejected_before_live_query() {
        let r1 = basis("a1", 1, "root-a");
        let identity = joined(r1.clone(), r1.clone()).workspace_identity;
        let mut ports = fake_ports(r1.clone(), basis("a2", 2, "root-a")).unwrap();
        ports.body_basis_auth = Err("historical basis is not authentic");
        assert!(observe_with_ports(&mut ports, &identity).is_err());
        assert!(ports.calls.contains(&"authenticate-admission-body"));
        assert!(!ports.calls.contains(&"semantic"));
    }

    #[test]
    fn cas_refusal_returns_no_success_and_identical_concurrent_append_is_accepted() -> Result<()> {
        let r1 = basis("a1", 1, "root-a");
        let r2 = basis("a2", 2, "root-a");
        let mut refused = fake_ports(r1.clone(), r2.clone())?;
        let identity = joined(r1.clone(), r1.clone()).workspace_identity;
        refused.append = Err("CAS refused");
        assert!(observe_with_ports(&mut refused, &identity).is_err());
        assert_eq!(refused.published, 0);

        let mut identical = fake_ports(r1, r2)?;
        identical.append = Ok(ReadinessAppendResult::ConcurrentIdentical);
        observe_with_ports(&mut identical, &identity)?;
        assert_eq!(identical.published, 1);
        Ok(())
    }

    fn raw_envelope(kind: &str, key: &str, payload: u8) -> CultCacheEnvelope {
        CultCacheEnvelope {
            key: key.into(),
            r#type: kind.into(),
            payload: vec![payload],
            stored_at: "2026-07-16T00:00:00Z".into(),
            schema_id: None,
        }
    }

    #[test]
    fn real_append_primitive_distinguishes_exact_and_hostile_concurrency() -> Result<()> {
        let run = |concurrent: Option<CultCacheEnvelope>| -> Result<ReadinessAppendResult> {
            let temp = tempfile::tempdir()?;
            let path = temp.path().join("state.msgpack");
            let mut backing = SingleFileMessagePackBackingStore::new(&path);
            let baseline = raw_envelope("baseline", "one", 1);
            backing.push(&baseline)?;
            let snapshot = backing.pull_all()?;
            if let Some(row) = concurrent {
                backing.push(&row)?;
            }
            append_readiness_envelope_if_snapshot_unchanged(
                &backing,
                &snapshot,
                raw_envelope(REPOSITORY_READINESS_PROJECTION_TYPE, "projection", 7),
            )
        };

        assert_eq!(
            run(None).map_err(|e| anyhow!("exact append: {e}"))?,
            ReadinessAppendResult::Appended
        );
        assert_eq!(
            run(Some(raw_envelope(
                REPOSITORY_READINESS_PROJECTION_TYPE,
                "projection",
                7
            )))
            .map_err(|e| anyhow!("identical concurrent: {e}"))?,
            ReadinessAppendResult::ConcurrentIdentical
        );
        assert!(run(Some(raw_envelope("unrelated", "row", 9))).is_err());
        assert!(
            run(Some(raw_envelope(
                REPOSITORY_READINESS_PROJECTION_TYPE,
                "projection",
                8
            )))
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn stale_or_multiple_current_admission_is_refused() {
        let body = basis("a", 1, "root-a");
        let mut stale = admission(body.clone());
        stale.admitted_revision = 6;
        assert!(select_current_admission(keyed(vec![stale]), 7, "model-hash").is_err());
        assert!(
            select_current_admission(
                keyed(vec![admission(body.clone()), admission(body)]),
                7,
                "model-hash"
            )
            .is_err()
        );
        let grounded = admission(basis("a", 1, "root-a"));
        let mut ungrounded = grounded.clone();
        ungrounded.receipt_id = "ungrounded-duplicate".into();
        ungrounded.repository_body_observation_basis = None;
        assert!(
            select_current_admission(keyed(vec![grounded, ungrounded]), 7, "model-hash").is_err()
        );
        let alien = admission(basis("a", 1, "root-a"));
        assert!(
            select_current_admission(vec![("alien-key".into(), alien)], 7, "model-hash").is_err()
        );
    }

    #[test]
    fn missing_semantic_or_stored_only_coverage_cannot_substitute_for_live_evidence() {
        let body = basis("a", 1, "root-a");
        let mut missing_semantic = joined(body.clone(), body.clone());
        missing_semantic.modeling_live = None;
        assert!(projection_from_join(missing_semantic, "2026-07-16T01:00:00Z").is_err());
        let mut stored_only_coverage = joined(body.clone(), body);
        stored_only_coverage.coverage_live = None;
        assert!(projection_from_join(stored_only_coverage, "2026-07-16T01:00:00Z").is_err());
    }

    #[test]
    fn body_change_and_non_body_authority_advance_are_refused() {
        let a = basis("a", 1, "root-a");
        let b = basis("b", 2, "root-b");
        assert!(projection_from_join(joined(a.clone(), b), "2026-07-16T01:00:00Z").is_err());
        let mut advanced = joined(a.clone(), a);
        advanced.closing_authority.push(CultCacheEnvelope {
            key: "duplicate".into(),
            r#type: "epiphany.mind.repo_model_admission_receipt".into(),
            payload: vec![1],
            stored_at: "2026-07-16T00:00:02Z".into(),
            schema_id: None,
        });
        assert!(projection_from_join(advanced, "2026-07-16T01:00:00Z").is_err());
    }

    #[test]
    fn a_to_b_to_a_is_accepted_when_final_content_identity_is_equal() -> Result<()> {
        let r1 = basis("observation-a1", 1, "root-a");
        let r2 = basis("observation-a2", 3, "root-a");
        let projection = projection_from_join(joined(r1, r2), "2026-07-16T01:00:00Z")?;
        assert_eq!(projection.status, "observed_ready_at");
        assert_eq!(projection.body_r2_generation, 3);
        Ok(())
    }

    #[test]
    fn proof_close_time_cannot_predate_r2() {
        let body = basis("a", 1, "root-a");
        assert!(projection_from_join(joined(body.clone(), body), "2026-07-15T23:59:59Z").is_err());
    }
}
