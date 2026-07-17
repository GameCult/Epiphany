use crate::repository_body_observer::RepositoryBodyReadSession;
use crate::semantic_backend::{
    CollectionCompatibility, OllamaEmbedder, QDRANT_POINT_BATCH_MAX, QdrantBackend, SemanticPoint,
    SemanticStoredPoint,
};
use crate::workspace_coverage_projection_batch_checkpoint::{
    ObservedWorkspaceCoverageBatchInput, WorkspaceCoverageProjectionBatchCheckpointAdmission,
    admit_observed_workspace_coverage_batch, load_authenticated_checkpoint_chain,
};
use crate::workspace_coverage_projection_progress::{
    authenticate_workspace_coverage_projection_progress,
    load_latest_workspace_coverage_projection_progress,
    publish_workspace_coverage_progress_for_checkpoint,
    publish_workspace_coverage_progress_genesis, reconcile_workspace_coverage_checkpoint_progress,
};
use crate::{
    BODY_BINDING_KEY, BODY_BINDING_TYPE, BODY_HEAD_KEY, BODY_HEAD_TYPE, BODY_MANIFEST_TYPE,
    BODY_OBSERVATION_TYPE, HostIncarnationIdentityEntry, RepositoryBodyHead,
    RepositoryBodyObservationBasis, WORKSPACE_COVERAGE_HEAD_SCHEMA_VERSION,
    WORKSPACE_COVERAGE_RECEIPT_SCHEMA_VERSION, WorkspaceCoverageChunkDescriptor,
    WorkspaceCoverageHead, WorkspaceCoverageObligation, WorkspaceCoveragePointBinding,
    WorkspaceCoveragePointPayload, WorkspaceCoveragePolicy, WorkspaceCoverageProjectionPlan,
    WorkspaceCoverageReceipt, WorkspaceCoverageVectorBinding,
    derive_workspace_coverage_obligation_from_authenticated_manifest,
    derive_workspace_coverage_projection_plan, load_latest_workspace_coverage_provider_heartbeat,
    observe_runtime_repository_body_basis, refine_workspace_coverage_obligation_utf8,
    runtime_repository_body_store_binding, validate_workspace_coverage_head,
    validate_workspace_coverage_projection_plan, workspace_coverage_execution_collection,
    workspace_coverage_process_documents::{
        authenticate_workspace_coverage_managed_process_launch_with_envelope_digest,
        authenticate_workspace_coverage_provider_heartbeat_with_envelope_digest,
        authenticate_workspace_coverage_termination_with_envelope_digest,
    },
};
use anyhow::{Result, anyhow, bail};
use cultcache_rs::{
    CacheBackingStore, CultCacheEnvelope, DatabaseEntry, SingleFileMessagePackBackingStore,
};
use ed25519_dalek::SigningKey;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub(crate) const CLAIM_TYPE: &str = "gamecult.epiphany.workspace_coverage_projection_claim";
pub(crate) const ATTEMPT_TYPE: &str = "gamecult.epiphany.workspace_coverage_projection_attempt";
pub(crate) const CLAIM_KEY: &str = "workspace-coverage-projector-current";
const CLAIM_SCHEMA: &str = "gamecult.epiphany.workspace_coverage_projection_claim.v1";
const ATTEMPT_SCHEMA: &str = "gamecult.epiphany.workspace_coverage_projection_attempt.v1";
const CLAIM_SCHEMA_V2: &str = "gamecult.epiphany.workspace_coverage_projection_claim.v2";
const ATTEMPT_SCHEMA_V2: &str = "gamecult.epiphany.workspace_coverage_projection_attempt.v2";
const CLAIM_SCHEMA_V3: &str = "gamecult.epiphany.workspace_coverage_projection_claim.v3";
const ATTEMPT_SCHEMA_V3: &str = "gamecult.epiphany.workspace_coverage_projection_attempt.v3";
const RECOVERY_TYPE: &str = "gamecult.epiphany.workspace_coverage_recovery_receipt";
const RECOVERY_SCHEMA: &str = "gamecult.epiphany.workspace_coverage_recovery_receipt.v0";
pub(crate) const PROJECTION_SCHEMA: &str = "gamecult.epiphany.workspace_bytes_projection.v0";
pub(crate) const CHUNKER_ID: &str = "utf8_lines_96_overlap_8_v0";
pub const WORKSPACE_COVERAGE_MAXIMUM_FILE_BYTES: u64 = 4 * 1024 * 1024;
const CHUNK_LINES: usize = 96;
const CHUNK_OVERLAP_LINES: usize = 8;
const RECEIPT_TYPE: &str = "gamecult.epiphany.workspace_coverage_receipt";
const HEAD_TYPE: &str = "gamecult.epiphany.workspace_coverage_head";
const HEAD_KEY: &str = "current";
pub(crate) const OBLIGATION_TYPE: &str = "gamecult.epiphany.workspace_coverage_obligation";
pub(crate) const PLAN_TYPE: &str = "gamecult.epiphany.workspace_coverage_projection_plan";

#[cfg(test)]
#[derive(Clone, Debug)]
struct WorkspaceCoverageProjectionInput {
    pub point_id: String,
    pub text: String,
    pub vector: Vec<f32>,
}

pub(crate) struct WorkspaceCoverageProjectionExecutionAuthority<'a> {
    pub local_verse_store: &'a Path,
    pub runtime_id: &'a str,
    pub trusted_host: &'a HostIncarnationIdentityEntry,
    pub provider_signing_key: &'a SigningKey,
    pub operation_timeout_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkspaceCoverageObservedBinding {
    collection_name: String,
    point_count: u64,
    point_set_sha256: String,
    point_binding_set_sha256: String,
    vector_binding_set_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceCoverageRecoveryTarget {
    pub claim_id: String,
    pub claim_epoch: u64,
    pub plan_id: String,
    pub managed_process_launch_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceCoverageRecoveryOutcome {
    pub claim_id: String,
    pub claim_epoch: u64,
    pub managed_process_launch_id: String,
    pub executor_incarnation: String,
    pub recovery_receipt_id: String,
    pub recovery_receipt_digest: String,
}

pub(crate) trait WorkspaceCoverageProjectionPort {
    fn ensure_exact_collection(
        &mut self,
        collection: &str,
        compatibility: &CollectionCompatibility,
    ) -> Result<()>;
    fn upsert_waited_batch(
        &mut self,
        collection: &str,
        points: &[SemanticPoint<WorkspaceCoveragePointPayload>],
    ) -> Result<()>;
    fn observe_exact_ids(
        &mut self,
        collection: &str,
        point_ids: &[String],
    ) -> Result<Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>>;
    fn observe_all(
        &mut self,
        collection: &str,
    ) -> Result<Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>>;
}

/// Read-only Qdrant surface used by Mind's bounded repository-readiness join.
/// It can observe one exact collection, but cannot create, mutate, or retire it.
#[allow(dead_code)] // Sealed read surface for the Mind readiness join.
pub(crate) trait WorkspaceCoverageEvidencePort {
    fn authenticate_exact_collection(
        &mut self,
        collection: &str,
        compatibility: &CollectionCompatibility,
    ) -> Result<()>;

    fn observe_all_for_evidence(
        &mut self,
        collection: &str,
    ) -> Result<Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkspaceCoverageRetirementCandidate {
    pub collection_name: String,
    pub compatibility: CollectionCompatibility,
}

pub(crate) trait WorkspaceCoverageRetirementPort {
    fn retire_exact(
        &mut self,
        collection: &str,
        compatibility: &CollectionCompatibility,
    ) -> Result<()>;
}

impl WorkspaceCoverageRetirementPort for QdrantBackend {
    fn retire_exact(
        &mut self,
        collection: &str,
        compatibility: &CollectionCompatibility,
    ) -> Result<()> {
        self.retire_exact_collection(collection, compatibility)
    }
}

impl WorkspaceCoverageProjectionPort for QdrantBackend {
    fn ensure_exact_collection(
        &mut self,
        name: &str,
        expected: &CollectionCompatibility,
    ) -> Result<()> {
        QdrantBackend::ensure_exact_collection(self, name, expected)
    }
    fn upsert_waited_batch(
        &mut self,
        name: &str,
        points: &[SemanticPoint<WorkspaceCoveragePointPayload>],
    ) -> Result<()> {
        self.upsert_point_batch_waited(name, points).map(|_| ())
    }
    fn observe_exact_ids(
        &mut self,
        name: &str,
        point_ids: &[String],
    ) -> Result<Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>> {
        self.retrieve_points_by_ids(name, point_ids)
    }
    fn observe_all(
        &mut self,
        name: &str,
    ) -> Result<Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>> {
        self.points_for_scope(name, &[])
    }
}

impl WorkspaceCoverageEvidencePort for QdrantBackend {
    fn authenticate_exact_collection(
        &mut self,
        name: &str,
        expected: &CollectionCompatibility,
    ) -> Result<()> {
        if !self.collection_exists(name)? || self.collection_compatibility(name)? != *expected {
            bail!("live workspace coverage collection is absent or incompatible");
        }
        Ok(())
    }

    fn observe_all_for_evidence(
        &mut self,
        name: &str,
    ) -> Result<Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>> {
        self.points_for_scope(name, &[])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)] // Sealed evidence for the Mind readiness join.
pub(crate) struct WorkspaceCoverageReadinessEvidence {
    pub workspace_id: String,
    pub body_binding_sha256: String,
    pub manifest_root_sha256: String,
    pub policy_id: String,
    pub policy_sha256: String,
    pub obligation_id: String,
    pub obligation_envelope_digest: String,
    pub plan_id: String,
    pub plan_envelope_digest: String,
    pub claim_id: String,
    pub claim_epoch: u64,
    pub claim_envelope_digest: String,
    pub attempt_id: String,
    pub attempt_envelope_digest: String,
    pub receipt_id: String,
    pub receipt_envelope_digest: String,
    pub head_envelope_digest: String,
    pub collection_name: String,
    pub observed_point_count: u64,
    pub observed_point_set_sha256: String,
    pub observed_point_binding_set_sha256: String,
    pub observed_vector_binding_set_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.workspace_coverage_projection_claim",
    schema = "WorkspaceCoverageProjectionClaim"
)]
pub(crate) struct WorkspaceCoverageProjectionClaim {
    #[cultcache(key = 0)]
    pub(crate) schema_version: String,
    #[cultcache(key = 1)]
    pub(crate) claim_id: String,
    #[cultcache(key = 2)]
    pub(crate) claim_epoch: u64,
    #[cultcache(key = 3)]
    pub(crate) plan_id: String,
    #[cultcache(key = 4)]
    pub(crate) attempt_id: String,
    #[cultcache(key = 5)]
    pub(crate) obligation_id: String,
    #[cultcache(key = 6)]
    pub(crate) body_observation_id: String,
    #[cultcache(key = 7)]
    pub(crate) body_generation: u64,
    #[cultcache(key = 8)]
    pub(crate) manifest_root_sha256: String,
    #[cultcache(key = 9)]
    pub(crate) status: String,
    #[cultcache(key = 10)]
    pub(crate) executor_id: String,
    #[cultcache(key = 11)]
    pub(crate) executor_incarnation: String,
    #[cultcache(key = 12, default)]
    pub(crate) managed_process_launch_id: String,
    #[cultcache(key = 13, default)]
    termination_evidence_id: Option<String>,
    #[cultcache(key = 14, default)]
    pub(crate) termination_evidence_digest: Option<String>,
    #[cultcache(key = 15, default)]
    pub(crate) recovery_receipt_id: Option<String>,
    #[cultcache(key = 16, default)]
    pub(crate) recovery_receipt_digest: Option<String>,
}

pub(crate) fn validate_projection_claim(claim: &WorkspaceCoverageProjectionClaim) -> Result<()> {
    if !matches!(
        claim.schema_version.as_str(),
        CLAIM_SCHEMA | CLAIM_SCHEMA_V2 | CLAIM_SCHEMA_V3
    ) || claim.claim_id.trim().is_empty()
        || claim.claim_epoch == 0
        || claim.plan_id.trim().is_empty()
        || claim.attempt_id.trim().is_empty()
        || claim.obligation_id.trim().is_empty()
        || claim.body_observation_id.trim().is_empty()
        || claim.manifest_root_sha256.trim().is_empty()
        || claim.executor_id.trim().is_empty()
        || claim.executor_incarnation.trim().is_empty()
        || claim.managed_process_launch_id.trim().is_empty()
        || !matches!(claim.status.as_str(), "running" | "failed" | "succeeded")
    {
        bail!("invalid workspace coverage projection claim");
    }
    match claim.status.as_str() {
        "failed" if claim.schema_version == CLAIM_SCHEMA_V2 => {
            validate_terminal_evidence(
                &claim.termination_evidence_id,
                &claim.termination_evidence_digest,
            )?;
        }
        _ if claim.termination_evidence_id.is_some()
            || claim.termination_evidence_digest.is_some() =>
        {
            bail!("workspace coverage claim has misplaced termination evidence")
        }
        _ => {}
    }
    if claim.schema_version == CLAIM_SCHEMA_V2 && claim.status != "failed" {
        bail!("workspace coverage recovery claim schema is terminal-only");
    }
    match claim.schema_version.as_str() {
        CLAIM_SCHEMA
            if claim.recovery_receipt_id.is_some() || claim.recovery_receipt_digest.is_some() =>
        {
            bail!("ordinary workspace coverage claim cites recovery evidence")
        }
        CLAIM_SCHEMA_V2 | CLAIM_SCHEMA_V3 => {
            validate_terminal_evidence(&claim.recovery_receipt_id, &claim.recovery_receipt_digest)?
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn validate_projection_attempt(
    attempt: &WorkspaceCoverageProjectionAttempt,
) -> Result<()> {
    if !matches!(
        attempt.schema_version.as_str(),
        ATTEMPT_SCHEMA | ATTEMPT_SCHEMA_V2 | ATTEMPT_SCHEMA_V3
    ) || attempt.attempt_id.trim().is_empty()
        || attempt.claim_id.trim().is_empty()
        || attempt.claim_epoch == 0
        || attempt.plan_id.trim().is_empty()
        || attempt.started_at.trim().is_empty()
        || attempt.executor_id.trim().is_empty()
        || attempt.executor_incarnation.trim().is_empty()
        || attempt.managed_process_launch_id.trim().is_empty()
        || !matches!(attempt.status.as_str(), "running" | "failed" | "succeeded")
    {
        bail!("invalid workspace coverage projection attempt");
    }
    chrono::DateTime::parse_from_rfc3339(&attempt.started_at)?;
    match attempt.status.as_str() {
        "running" if attempt.completed_at.is_some() || attempt.error.is_some() => {
            bail!("running workspace coverage attempt is already terminal")
        }
        "failed"
            if attempt.completed_at.is_none()
                || attempt.error.as_deref().is_none_or(str::is_empty) =>
        {
            bail!("failed workspace coverage attempt lacks terminal evidence")
        }
        "succeeded" if attempt.completed_at.is_none() || attempt.error.is_some() => {
            bail!("successful workspace coverage attempt has invalid terminal evidence")
        }
        _ => {}
    }
    if let Some(value) = &attempt.completed_at {
        let completed = chrono::DateTime::parse_from_rfc3339(value)?;
        let started = chrono::DateTime::parse_from_rfc3339(&attempt.started_at)?;
        if completed < started {
            bail!("workspace coverage attempt completes before it starts");
        }
    }
    match attempt.status.as_str() {
        "failed" if attempt.schema_version == ATTEMPT_SCHEMA_V2 => {
            validate_terminal_evidence(
                &attempt.termination_evidence_id,
                &attempt.termination_evidence_digest,
            )?;
        }
        _ if attempt.termination_evidence_id.is_some()
            || attempt.termination_evidence_digest.is_some() =>
        {
            bail!("workspace coverage attempt has misplaced termination evidence")
        }
        _ => {}
    }
    if attempt.schema_version == ATTEMPT_SCHEMA_V2 && attempt.status != "failed" {
        bail!("workspace coverage recovery attempt schema is terminal-only");
    }
    match attempt.schema_version.as_str() {
        ATTEMPT_SCHEMA
            if attempt.recovery_receipt_id.is_some()
                || attempt.recovery_receipt_digest.is_some() =>
        {
            bail!("ordinary workspace coverage attempt cites recovery evidence")
        }
        ATTEMPT_SCHEMA_V2 | ATTEMPT_SCHEMA_V3 => validate_terminal_evidence(
            &attempt.recovery_receipt_id,
            &attempt.recovery_receipt_digest,
        )?,
        _ => {}
    }
    Ok(())
}

fn validate_terminal_evidence(id: &Option<String>, digest: &Option<String>) -> Result<()> {
    let (Some(id), Some(digest)) = (id.as_deref(), digest.as_deref()) else {
        bail!("recovered workspace coverage terminal state lacks exact evidence")
    };
    let Some(hex) = digest.strip_prefix("sha256-") else {
        bail!("recovered workspace coverage terminal evidence is invalid")
    };
    if id.trim().is_empty() || hex.len() != 64 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("recovered workspace coverage terminal evidence is invalid")
    }
    Ok(())
}

pub(crate) fn validate_claim_attempt_link(
    claim: &WorkspaceCoverageProjectionClaim,
    attempt: &WorkspaceCoverageProjectionAttempt,
) -> Result<()> {
    validate_projection_claim(claim)?;
    validate_projection_attempt(attempt)?;
    if claim.claim_id != attempt.claim_id
        || !matches!(
            (
                claim.schema_version.as_str(),
                attempt.schema_version.as_str()
            ),
            (CLAIM_SCHEMA, ATTEMPT_SCHEMA)
                | (CLAIM_SCHEMA_V2, ATTEMPT_SCHEMA_V2)
                | (CLAIM_SCHEMA_V3, ATTEMPT_SCHEMA_V3)
        )
        || claim.claim_epoch != attempt.claim_epoch
        || claim.attempt_id != attempt.attempt_id
        || claim.plan_id != attempt.plan_id
        || claim.status != attempt.status
        || claim.executor_id != attempt.executor_id
        || claim.executor_incarnation != attempt.executor_incarnation
        || claim.managed_process_launch_id != attempt.managed_process_launch_id
        || claim.termination_evidence_id != attempt.termination_evidence_id
        || claim.termination_evidence_digest != attempt.termination_evidence_digest
        || claim.recovery_receipt_id != attempt.recovery_receipt_id
        || claim.recovery_receipt_digest != attempt.recovery_receipt_digest
    {
        bail!("workspace coverage claim/attempt authority is split");
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.workspace_coverage_projection_attempt",
    schema = "WorkspaceCoverageProjectionAttempt"
)]
pub(crate) struct WorkspaceCoverageProjectionAttempt {
    #[cultcache(key = 0)]
    pub(crate) schema_version: String,
    #[cultcache(key = 1)]
    pub(crate) attempt_id: String,
    #[cultcache(key = 2)]
    pub(crate) claim_id: String,
    #[cultcache(key = 3)]
    pub(crate) claim_epoch: u64,
    #[cultcache(key = 4)]
    pub(crate) plan_id: String,
    #[cultcache(key = 5)]
    pub(crate) status: String,
    #[cultcache(key = 6)]
    pub(crate) started_at: String,
    #[cultcache(key = 7)]
    pub(crate) completed_at: Option<String>,
    #[cultcache(key = 8)]
    pub(crate) error: Option<String>,
    #[cultcache(key = 9)]
    pub(crate) executor_id: String,
    #[cultcache(key = 10)]
    pub(crate) executor_incarnation: String,
    #[cultcache(key = 11)]
    pub(crate) managed_process_launch_id: String,
    #[cultcache(key = 12, default)]
    pub(crate) termination_evidence_id: Option<String>,
    #[cultcache(key = 13, default)]
    pub(crate) termination_evidence_digest: Option<String>,
    #[cultcache(key = 14, default)]
    pub(crate) recovery_receipt_id: Option<String>,
    #[cultcache(key = 15, default)]
    pub(crate) recovery_receipt_digest: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.workspace_coverage_recovery_receipt",
    schema = "WorkspaceCoverageRecoveryReceipt"
)]
struct WorkspaceCoverageRecoveryReceipt {
    #[cultcache(key = 0)]
    schema_version: String,
    #[cultcache(key = 1)]
    recovery_id: String,
    #[cultcache(key = 2)]
    old_claim_id: String,
    #[cultcache(key = 3)]
    old_attempt_id: String,
    #[cultcache(key = 4)]
    old_claim_epoch: u64,
    #[cultcache(key = 5)]
    termination_id: String,
    #[cultcache(key = 6)]
    termination_envelope_digest: String,
    #[cultcache(key = 7)]
    replacement_launch_id: String,
    #[cultcache(key = 8)]
    replacement_launch_envelope_digest: String,
    #[cultcache(key = 9)]
    ready_heartbeat_id: String,
    #[cultcache(key = 10)]
    ready_heartbeat_envelope_digest: String,
    #[cultcache(key = 11)]
    new_claim_id: String,
    #[cultcache(key = 12)]
    new_attempt_id: String,
    #[cultcache(key = 13)]
    new_claim_epoch: u64,
    #[cultcache(key = 14)]
    recovered_at_utc: String,
    #[cultcache(key = 15)]
    old_launch_id: String,
}

fn validate_recovery_receipt(receipt: &WorkspaceCoverageRecoveryReceipt) -> Result<()> {
    if receipt.schema_version != RECOVERY_SCHEMA
        || receipt.recovery_id.trim().is_empty()
        || receipt.old_claim_id.trim().is_empty()
        || receipt.old_attempt_id.trim().is_empty()
        || receipt.old_launch_id.trim().is_empty()
        || receipt.old_claim_epoch == 0
        || receipt.replacement_launch_id.trim().is_empty()
        || receipt.ready_heartbeat_id.trim().is_empty()
        || receipt.new_claim_id.trim().is_empty()
        || receipt.new_attempt_id.trim().is_empty()
        || receipt.old_claim_epoch.checked_add(1) != Some(receipt.new_claim_epoch)
    {
        bail!("invalid workspace coverage recovery receipt");
    }
    validate_terminal_evidence(
        &Some(receipt.termination_id.clone()),
        &Some(receipt.termination_envelope_digest.clone()),
    )?;
    validate_terminal_evidence(
        &Some(receipt.replacement_launch_id.clone()),
        &Some(receipt.replacement_launch_envelope_digest.clone()),
    )?;
    validate_terminal_evidence(
        &Some(receipt.ready_heartbeat_id.clone()),
        &Some(receipt.ready_heartbeat_envelope_digest.clone()),
    )?;
    chrono::DateTime::parse_from_rfc3339(&receipt.recovered_at_utc)?;
    Ok(())
}

fn cultcache_envelope_digest(value: &CultCacheEnvelope) -> String {
    let mut digest = Sha256::new();
    digest.update(value.r#type.as_bytes());
    digest.update([0]);
    digest.update(value.key.as_bytes());
    digest.update([0]);
    digest.update(&value.payload);
    format!("sha256-{:x}", digest.finalize())
}

pub fn authenticate_workspace_coverage_recovery_receipt(
    body_store: impl AsRef<Path>,
    cultmesh_store: impl AsRef<Path>,
    runtime_id: &str,
    host: &HostIncarnationIdentityEntry,
    recovery_id: &str,
    expected_digest: &str,
) -> Result<()> {
    let opening = SingleFileMessagePackBackingStore::new(body_store.as_ref()).pull_all()?;
    let receipt_env = find(&opening, RECOVERY_TYPE, recovery_id)
        .ok_or_else(|| anyhow!("workspace coverage recovery receipt is absent"))?;
    if cultcache_envelope_digest(receipt_env) != expected_digest {
        bail!("workspace coverage recovery receipt digest disagrees with authority");
    }
    let receipt: WorkspaceCoverageRecoveryReceipt = decode(receipt_env)?;
    validate_recovery_receipt(&receipt)?;
    if receipt.recovery_id != recovery_id {
        bail!("workspace coverage recovery receipt key disagrees with payload");
    }
    let (_, termination_digest) = authenticate_workspace_coverage_termination_with_envelope_digest(
        cultmesh_store.as_ref(),
        runtime_id,
        &receipt.old_launch_id,
        host,
    )?;
    let (replacement_launch, replacement_launch_digest) =
        authenticate_workspace_coverage_managed_process_launch_with_envelope_digest(
            cultmesh_store.as_ref(),
            runtime_id,
            &receipt.replacement_launch_id,
            host,
        )?;
    let (ready, ready_digest) =
        authenticate_workspace_coverage_provider_heartbeat_with_envelope_digest(
            cultmesh_store.as_ref(),
            runtime_id,
            &receipt.ready_heartbeat_id,
            host,
        )?;
    let latest = load_latest_workspace_coverage_provider_heartbeat(
        cultmesh_store,
        runtime_id,
        &receipt.replacement_launch_id,
    )?
    .ok_or_else(|| anyhow!("workspace coverage recovery receipt has no current heartbeat"))?;
    if termination_digest != receipt.termination_envelope_digest
        || replacement_launch_digest != receipt.replacement_launch_envelope_digest
        || ready_digest != receipt.ready_heartbeat_envelope_digest
        || replacement_launch.launch_id != receipt.replacement_launch_id
        || ready.launch_id != receipt.replacement_launch_id
        || ready.status != "ready"
        || latest.sequence < ready.sequence
        || (latest.sequence == ready.sequence && latest.heartbeat_id != ready.heartbeat_id)
    {
        bail!("workspace coverage recovery receipt external evidence has moved or disagrees");
    }
    let claim_env = find(&opening, CLAIM_TYPE, CLAIM_KEY)
        .ok_or_else(|| anyhow!("workspace coverage recovered claim is absent"))?;
    let claim: WorkspaceCoverageProjectionClaim = decode(claim_env)?;
    let attempt_env = find(&opening, ATTEMPT_TYPE, &claim.attempt_id)
        .ok_or_else(|| anyhow!("workspace coverage recovered attempt is absent"))?;
    let attempt: WorkspaceCoverageProjectionAttempt = decode(attempt_env)?;
    validate_claim_attempt_link(&claim, &attempt)?;
    if claim.claim_id != receipt.new_claim_id
        || attempt.attempt_id != receipt.new_attempt_id
        || claim.claim_epoch != receipt.new_claim_epoch
        || claim.recovery_receipt_id.as_deref() != Some(recovery_id)
        || claim.recovery_receipt_digest.as_deref() != Some(expected_digest)
    {
        bail!("workspace coverage recovery receipt disagrees with current Body authority");
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub(crate) struct WorkspaceCoveragePreparedPoint {
    pub point_id: String,
    pub text: String,
}

#[derive(Clone, Debug)]
pub(crate) struct PreparedWorkspaceCoverageProjection {
    pub body_store: PathBuf,
    pub basis: RepositoryBodyObservationBasis,
    pub obligation: WorkspaceCoverageObligation,
    pub plan: WorkspaceCoverageProjectionPlan,
    pub points: Vec<WorkspaceCoveragePreparedPoint>,
}

#[derive(Clone, Debug)]
pub(crate) struct WorkspaceCoverageAcquisition {
    pub body_store: PathBuf,
    pub obligation: WorkspaceCoverageObligation,
    pub plan: WorkspaceCoverageProjectionPlan,
    pub claim: WorkspaceCoverageProjectionClaim,
    pub attempt: WorkspaceCoverageProjectionAttempt,
    prior_head: Option<CultCacheEnvelope>,
}

#[derive(Clone, Debug)]
pub(crate) enum WorkspaceCoverageAcquireResult {
    Current(WorkspaceCoverageReceipt),
    Contended,
    Acquired(WorkspaceCoverageAcquisition),
}

pub(crate) enum WorkspaceCoverageCurrentState {
    Current(WorkspaceCoverageReceipt),
    NeedsPreparation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceCoverageTerminalAuthority {
    pub body_observation_id: String,
    pub body_generation: u64,
    pub plan_id: String,
    pub claim_id: String,
    pub claim_epoch: u64,
    pub managed_process_launch_id: String,
    pub receipt_id: String,
}

/// Reads the provider-owned terminal authority used by deployment health.
/// This deliberately does not re-read Qdrant: terminal receipt admission
/// already performed the whole-collection readback. Mind readiness owns the
/// stronger live query-evidence join.
pub fn authenticate_current_workspace_coverage_terminal_authority(
    runtime_store: &Path,
) -> Result<Option<WorkspaceCoverageTerminalAuthority>> {
    let basis = crate::load_current_runtime_repository_body_basis(runtime_store)?;
    let route = runtime_repository_body_store_binding(runtime_store)?
        .ok_or_else(|| anyhow!("runtime has no repository Body-store binding"))?;
    let opening = SingleFileMessagePackBackingStore::new(&route.body_store_path).pull_all()?;
    exact_body_authority(&opening, &basis)?;
    let Some(head_env) = find(&opening, HEAD_TYPE, HEAD_KEY) else {
        return Ok(None);
    };
    let head: WorkspaceCoverageHead = decode(head_env)?;
    let obligation: WorkspaceCoverageObligation = decode(
        find(&opening, OBLIGATION_TYPE, &head.obligation_id)
            .ok_or_else(|| anyhow!("current workspace coverage head names a missing obligation"))?,
    )?;
    let plan: WorkspaceCoverageProjectionPlan = decode(
        find(&opening, PLAN_TYPE, &head.plan_id)
            .ok_or_else(|| anyhow!("current workspace coverage head names a missing plan"))?,
    )?;
    let receipt: WorkspaceCoverageReceipt = decode(
        find(&opening, RECEIPT_TYPE, &head.receipt_id)
            .ok_or_else(|| anyhow!("current workspace coverage head names a missing receipt"))?,
    )?;
    let claim: WorkspaceCoverageProjectionClaim = decode(
        find(&opening, CLAIM_TYPE, CLAIM_KEY)
            .ok_or_else(|| anyhow!("current workspace coverage lost its projection claim"))?,
    )?;
    let attempt: WorkspaceCoverageProjectionAttempt = decode(
        find(&opening, ATTEMPT_TYPE, &claim.attempt_id)
            .ok_or_else(|| anyhow!("current workspace coverage lost its projection attempt"))?,
    )?;
    validate_workspace_coverage_head(&obligation, &plan, &receipt, &head)?;
    validate_claim_attempt_link(&claim, &attempt)?;
    let policy =
        WorkspaceCoveragePolicy::bounded_regular_files_v0(WORKSPACE_COVERAGE_MAXIMUM_FILE_BYTES)?;
    let policy_sha256 = format!(
        "{:x}",
        Sha256::digest(rmp_serde::to_vec_named(&policy).map_err(|error| anyhow!(error))?)
    );
    let exact = head.workspace_id == basis.workspace_id
        && head.body_observation_id == basis.observation_id
        && head.body_generation == basis.generation
        && head.manifest_root_sha256 == basis.manifest_root_sha256
        && obligation.runtime_id == basis.runtime_id
        && obligation.workspace_id == basis.workspace_id
        && obligation.swarm_id == basis.swarm_id
        && obligation.body_binding_sha256 == basis.body_binding_sha256
        && obligation.body_observation_id == basis.observation_id
        && obligation.body_generation == basis.generation
        && obligation.manifest_root_sha256 == basis.manifest_root_sha256
        && obligation.policy_id == policy.policy_id
        && obligation.policy_sha256 == policy_sha256
        && plan.projection_schema_version == PROJECTION_SCHEMA
        && plan.chunker_id == CHUNKER_ID
        && claim.status == "succeeded"
        && attempt.status == "succeeded"
        && claim.claim_id == receipt.claim_id
        && claim.claim_epoch == receipt.claim_epoch
        && claim.plan_id == plan.plan_id
        && claim.obligation_id == obligation.obligation_id;
    if !exact {
        return Ok(None);
    }
    exact_obligation_body_authority(&opening, &obligation)?;
    Ok(Some(WorkspaceCoverageTerminalAuthority {
        body_observation_id: basis.observation_id,
        body_generation: basis.generation,
        plan_id: plan.plan_id,
        claim_id: claim.claim_id,
        claim_epoch: claim.claim_epoch,
        managed_process_launch_id: claim.managed_process_launch_id,
        receipt_id: receipt.receipt_id,
    }))
}

fn collection_compatibility(plan: &WorkspaceCoverageProjectionPlan) -> CollectionCompatibility {
    CollectionCompatibility {
        managed_by: "epiphany-workspace-coverage-projector".into(),
        corpus_kind: "repository_body_workspace_coverage".into(),
        schema_version: 0,
        projection_version: plan.plan_id.clone(),
        embedding_provider_id: plan.embedding_provider_id.clone(),
        embedding_model: plan.embedding_model.clone(),
        vector_size: plan.vector_dimensions as usize,
    }
}

/// Derives disposable Qdrant collections exclusively from sealed Body-store
/// history. Every attempt is validated before any candidate is returned: one
/// malformed historical row refuses the whole GC pass.
pub(crate) fn workspace_coverage_retirement_candidates(
    body_store: impl AsRef<Path>,
) -> Result<Vec<WorkspaceCoverageRetirementCandidate>> {
    let opening = SingleFileMessagePackBackingStore::new(body_store.as_ref()).pull_all()?;

    let attempts = opening
        .iter()
        .filter(|entry| entry.r#type == ATTEMPT_TYPE)
        .map(decode::<WorkspaceCoverageProjectionAttempt>)
        .collect::<Result<Vec<_>>>()?;

    let mut protected_collection = None;
    if let Some(head_env) = find(&opening, HEAD_TYPE, HEAD_KEY) {
        let head: WorkspaceCoverageHead = decode(head_env)?;
        let obligation: WorkspaceCoverageObligation = decode(
            find(&opening, OBLIGATION_TYPE, &head.obligation_id)
                .ok_or_else(|| anyhow!("workspace coverage head names missing obligation"))?,
        )?;
        let plan: WorkspaceCoverageProjectionPlan = decode(
            find(&opening, PLAN_TYPE, &head.plan_id)
                .ok_or_else(|| anyhow!("workspace coverage head names missing plan"))?,
        )?;
        let receipt: WorkspaceCoverageReceipt = decode(
            find(&opening, RECEIPT_TYPE, &head.receipt_id)
                .ok_or_else(|| anyhow!("workspace coverage head names missing receipt"))?,
        )?;
        validate_workspace_coverage_head(&obligation, &plan, &receipt, &head)?;
        protected_collection = Some(receipt.collection_name);
    }

    for claim_env in opening.iter().filter(|entry| entry.r#type == CLAIM_TYPE) {
        let claim: WorkspaceCoverageProjectionClaim = decode(claim_env)?;
        let attempt = attempts
            .iter()
            .find(|attempt| attempt.attempt_id == claim.attempt_id)
            .ok_or_else(|| anyhow!("workspace coverage claim names missing attempt"))?;
        validate_claim_attempt_link(&claim, attempt)?;
        let plan: WorkspaceCoverageProjectionPlan = decode(
            find(&opening, PLAN_TYPE, &claim.plan_id)
                .ok_or_else(|| anyhow!("workspace coverage claim names missing plan"))?,
        )?;
        if plan.obligation_id != claim.obligation_id {
            bail!("workspace coverage claim/plan authority is split");
        }
    }

    let mut candidates = std::collections::BTreeMap::new();
    for attempt in attempts {
        validate_projection_attempt(&attempt)?;
        let plan: WorkspaceCoverageProjectionPlan = decode(
            find(&opening, PLAN_TYPE, &attempt.plan_id)
                .ok_or_else(|| anyhow!("workspace coverage attempt names missing plan"))?,
        )?;
        let obligation: WorkspaceCoverageObligation = decode(
            find(&opening, OBLIGATION_TYPE, &plan.obligation_id)
                .ok_or_else(|| anyhow!("workspace coverage plan names missing obligation"))?,
        )?;
        validate_workspace_coverage_projection_plan(&obligation, &plan)?;
        if attempt.status == "running" {
            continue;
        }
        let name = workspace_coverage_execution_collection(
            &plan.plan_id,
            &attempt.claim_id,
            attempt.claim_epoch,
        )?;
        if protected_collection.as_deref() == Some(&name) {
            continue;
        }
        candidates.insert(
            name.clone(),
            WorkspaceCoverageRetirementCandidate {
                collection_name: name,
                compatibility: collection_compatibility(&plan),
            },
        );
    }
    Ok(candidates.into_values().collect())
}

pub(crate) fn retire_workspace_coverage_collections(
    body_store: impl AsRef<Path>,
    port: &mut impl WorkspaceCoverageRetirementPort,
) -> Result<usize> {
    let candidates = workspace_coverage_retirement_candidates(body_store)?;
    for candidate in &candidates {
        port.retire_exact(&candidate.collection_name, &candidate.compatibility)?;
    }
    Ok(candidates.len())
}

/// Classifies the persisted projection using typed authority only. This is the
/// service's idle path: no Repository Body file is opened or materialized.
pub(crate) fn classify_current_workspace_coverage(
    runtime_store: &Path,
    basis: &RepositoryBodyObservationBasis,
    embedding_provider_id: &str,
    embedding_model: &str,
    vector_dimensions: u32,
) -> Result<WorkspaceCoverageCurrentState> {
    if embedding_provider_id.trim().is_empty()
        || embedding_model.trim().is_empty()
        || vector_dimensions == 0
    {
        bail!("workspace coverage classification requires exact embedding identity");
    }
    let route = runtime_repository_body_store_binding(runtime_store)?
        .ok_or_else(|| anyhow!("runtime has no repository Body-store binding"))?;
    let opening = SingleFileMessagePackBackingStore::new(&route.body_store_path).pull_all()?;
    exact_body_authority(&opening, basis)?;
    let Some(head_env) = find(&opening, HEAD_TYPE, HEAD_KEY) else {
        return Ok(WorkspaceCoverageCurrentState::NeedsPreparation);
    };
    let head: WorkspaceCoverageHead = decode(head_env)?;
    let obligation: WorkspaceCoverageObligation = decode(
        find(
            &opening,
            "gamecult.epiphany.workspace_coverage_obligation",
            &head.obligation_id,
        )
        .ok_or_else(|| anyhow!("current workspace coverage head names a missing obligation"))?,
    )?;
    let plan: WorkspaceCoverageProjectionPlan = decode(
        find(
            &opening,
            "gamecult.epiphany.workspace_coverage_projection_plan",
            &head.plan_id,
        )
        .ok_or_else(|| anyhow!("current workspace coverage head names a missing plan"))?,
    )?;
    let receipt: WorkspaceCoverageReceipt = decode(
        find(&opening, RECEIPT_TYPE, &head.receipt_id)
            .ok_or_else(|| anyhow!("current workspace coverage head names a missing receipt"))?,
    )?;
    validate_workspace_coverage_head(&obligation, &plan, &receipt, &head)?;
    let policy =
        WorkspaceCoveragePolicy::bounded_regular_files_v0(WORKSPACE_COVERAGE_MAXIMUM_FILE_BYTES)?;
    let policy_sha256 = format!(
        "{:x}",
        Sha256::digest(rmp_serde::to_vec_named(&policy).map_err(|error| anyhow!(error))?)
    );
    let exact = head.workspace_id == basis.workspace_id
        && head.body_observation_id == basis.observation_id
        && head.body_generation == basis.generation
        && head.manifest_root_sha256 == basis.manifest_root_sha256
        && obligation.workspace_id == basis.workspace_id
        && obligation.swarm_id == basis.swarm_id
        && obligation.runtime_id == basis.runtime_id
        && obligation.body_binding_sha256 == basis.body_binding_sha256
        && obligation.body_observation_id == basis.observation_id
        && obligation.body_generation == basis.generation
        && obligation.manifest_root_sha256 == basis.manifest_root_sha256
        && obligation.policy_id == policy.policy_id
        && obligation.policy_sha256 == policy_sha256
        && plan.projection_schema_version == PROJECTION_SCHEMA
        && plan.chunker_id == CHUNKER_ID
        && plan.embedding_provider_id == embedding_provider_id
        && plan.embedding_model == embedding_model
        && plan.vector_dimensions == vector_dimensions
        && receipt.embedding_provider_id == embedding_provider_id
        && receipt.embedding_model == embedding_model
        && receipt.vector_dimensions == vector_dimensions;
    Ok(if exact {
        WorkspaceCoverageCurrentState::Current(receipt)
    } else {
        WorkspaceCoverageCurrentState::NeedsPreparation
    })
}

/// Authenticates and re-observes the exact current workspace projection.
///
/// A persisted coverage head is historical evidence only. This reader returns
/// evidence only after the current succeeded claim/attempt chain and the live
/// Qdrant payload/vector bindings agree with that head's sealed plan.
#[allow(dead_code)] // Wired by the Mind readiness join, never by the projector.
pub(crate) fn observe_current_workspace_coverage_evidence(
    runtime_store: &Path,
    basis: &RepositoryBodyObservationBasis,
    policy: &WorkspaceCoveragePolicy,
    embedding_provider_id: &str,
    embedding_model: &str,
    vector_dimensions: u32,
    port: &mut impl WorkspaceCoverageEvidencePort,
) -> Result<Option<WorkspaceCoverageReadinessEvidence>> {
    if policy.schema_version != crate::WORKSPACE_COVERAGE_POLICY_SCHEMA_VERSION
        || policy.policy_id.trim().is_empty()
        || policy.maximum_file_bytes == 0
        || embedding_provider_id.trim().is_empty()
        || embedding_model.trim().is_empty()
        || vector_dimensions == 0
    {
        bail!("workspace coverage evidence requires an exact policy and embedding identity");
    }
    let route = runtime_repository_body_store_binding(runtime_store)?
        .ok_or_else(|| anyhow!("runtime has no repository Body-store binding"))?;
    let opening = SingleFileMessagePackBackingStore::new(&route.body_store_path).pull_all()?;
    exact_body_authority(&opening, basis)?;
    let Some(head_env) = find(&opening, HEAD_TYPE, HEAD_KEY) else {
        return Ok(None);
    };
    let head: WorkspaceCoverageHead = decode(head_env)?;
    let obligation_env = find(&opening, OBLIGATION_TYPE, &head.obligation_id)
        .ok_or_else(|| anyhow!("current workspace coverage head names a missing obligation"))?;
    let plan_env = find(&opening, PLAN_TYPE, &head.plan_id)
        .ok_or_else(|| anyhow!("current workspace coverage head names a missing plan"))?;
    let receipt_env = find(&opening, RECEIPT_TYPE, &head.receipt_id)
        .ok_or_else(|| anyhow!("current workspace coverage head names a missing receipt"))?;
    let claim_env = find(&opening, CLAIM_TYPE, CLAIM_KEY)
        .ok_or_else(|| anyhow!("current workspace coverage lost its projection claim"))?;
    let obligation: WorkspaceCoverageObligation = decode(obligation_env)?;
    let plan: WorkspaceCoverageProjectionPlan = decode(plan_env)?;
    let receipt: WorkspaceCoverageReceipt = decode(receipt_env)?;
    let claim: WorkspaceCoverageProjectionClaim = decode(claim_env)?;
    let attempt_env = find(&opening, ATTEMPT_TYPE, &claim.attempt_id)
        .ok_or_else(|| anyhow!("current workspace coverage lost its projection attempt"))?;
    let attempt: WorkspaceCoverageProjectionAttempt = decode(attempt_env)?;
    validate_workspace_coverage_head(&obligation, &plan, &receipt, &head)?;
    validate_claim_attempt_link(&claim, &attempt)?;

    let policy_sha256 = format!(
        "{:x}",
        Sha256::digest(rmp_serde::to_vec_named(policy).map_err(|error| anyhow!(error))?)
    );
    let exact = head.workspace_id == basis.workspace_id
        && head.body_observation_id == basis.observation_id
        && head.body_generation == basis.generation
        && head.manifest_root_sha256 == basis.manifest_root_sha256
        && obligation.workspace_id == basis.workspace_id
        && obligation.swarm_id == basis.swarm_id
        && obligation.runtime_id == basis.runtime_id
        && obligation.body_binding_sha256 == basis.body_binding_sha256
        && obligation.body_observation_id == basis.observation_id
        && obligation.body_generation == basis.generation
        && obligation.manifest_root_sha256 == basis.manifest_root_sha256
        && obligation.policy_id == policy.policy_id
        && obligation.policy_sha256 == policy_sha256
        && plan.projection_schema_version == PROJECTION_SCHEMA
        && plan.chunker_id == CHUNKER_ID
        && plan.embedding_provider_id == embedding_provider_id
        && plan.embedding_model == embedding_model
        && plan.vector_dimensions == vector_dimensions
        && receipt.embedding_provider_id == embedding_provider_id
        && receipt.embedding_model == embedding_model
        && receipt.vector_dimensions == vector_dimensions
        && claim.status == "succeeded"
        && claim.claim_id == receipt.claim_id
        && claim.claim_epoch == receipt.claim_epoch
        && claim.plan_id == plan.plan_id
        && claim.obligation_id == obligation.obligation_id
        && claim.body_observation_id == obligation.body_observation_id
        && claim.body_generation == obligation.body_generation
        && claim.manifest_root_sha256 == obligation.manifest_root_sha256;
    if !exact {
        return Ok(None);
    }

    port.authenticate_exact_collection(&receipt.collection_name, &collection_compatibility(&plan))?;
    let observed = observe_sealed_bindings(
        &obligation,
        &plan,
        &receipt.collection_name,
        port.observe_all_for_evidence(&receipt.collection_name)?,
        None,
    )?;
    if observed.point_count != receipt.observed_point_count
        || observed.point_set_sha256 != receipt.observed_point_set_sha256
        || observed.point_binding_set_sha256 != receipt.observed_point_binding_set_sha256
        || observed.vector_binding_set_sha256 != receipt.observed_vector_binding_set_sha256
    {
        bail!("live workspace coverage observation disagrees with its exact receipt");
    }

    // Qdrant observation is outside the Body-store lock domain. Close the
    // mixed-epoch window by reopening both the runtime route and Body store,
    // then require every authority envelope consumed above to remain exact.
    let closing_route = runtime_repository_body_store_binding(runtime_store)?
        .ok_or_else(|| anyhow!("runtime lost its repository Body-store binding"))?;
    if closing_route != route {
        return Ok(None);
    }
    let closing =
        SingleFileMessagePackBackingStore::new(&closing_route.body_store_path).pull_all()?;
    for expected in [
        head_env,
        obligation_env,
        plan_env,
        claim_env,
        attempt_env,
        receipt_env,
    ] {
        if find(&closing, &expected.r#type, &expected.key) != Some(expected) {
            return Ok(None);
        }
    }
    Ok(Some(WorkspaceCoverageReadinessEvidence {
        workspace_id: obligation.workspace_id,
        body_binding_sha256: obligation.body_binding_sha256,
        manifest_root_sha256: obligation.manifest_root_sha256,
        policy_id: obligation.policy_id,
        policy_sha256: obligation.policy_sha256,
        obligation_id: obligation.obligation_id,
        obligation_envelope_digest: cultcache_envelope_digest(obligation_env),
        plan_id: plan.plan_id,
        plan_envelope_digest: cultcache_envelope_digest(plan_env),
        claim_id: claim.claim_id,
        claim_epoch: claim.claim_epoch,
        claim_envelope_digest: cultcache_envelope_digest(claim_env),
        attempt_id: attempt.attempt_id,
        attempt_envelope_digest: cultcache_envelope_digest(attempt_env),
        receipt_id: receipt.receipt_id,
        receipt_envelope_digest: cultcache_envelope_digest(receipt_env),
        head_envelope_digest: cultcache_envelope_digest(head_env),
        collection_name: observed.collection_name,
        observed_point_count: observed.point_count,
        observed_point_set_sha256: observed.point_set_sha256,
        observed_point_binding_set_sha256: observed.point_binding_set_sha256,
        observed_vector_binding_set_sha256: observed.vector_binding_set_sha256,
    }))
}

pub(crate) fn prepare_workspace_coverage_projection(
    body: &RepositoryBodyReadSession,
    embedding_provider_id: &str,
    embedding_model: &str,
    vector_dimensions: u32,
) -> Result<PreparedWorkspaceCoverageProjection> {
    let basis = body.basis();
    let policy =
        WorkspaceCoveragePolicy::bounded_regular_files_v0(WORKSPACE_COVERAGE_MAXIMUM_FILE_BYTES)?;
    let raw_obligation = derive_workspace_coverage_obligation_from_authenticated_manifest(
        basis,
        body.manifest(),
        &policy,
    )?;
    let mut verified_text = Vec::new();
    let mut non_utf8_paths = Vec::new();
    for entry in &raw_obligation.classifications {
        if !matches!(
            entry.disposition,
            crate::WorkspaceCoverageDisposition::IncludeRegularFile
        ) {
            continue;
        }
        let verified = body.read_regular_file(&entry.path)?;
        if verified.relative_path != entry.path || verified.raw_sha256 != entry.raw_sha256 {
            bail!("verified Body bytes lost their manifest identity");
        }
        if std::str::from_utf8(&verified.bytes).is_ok() {
            verified_text.push(verified);
        } else {
            non_utf8_paths.push(entry.path.clone());
        }
    }
    let obligation = refine_workspace_coverage_obligation_utf8(&raw_obligation, &non_utf8_paths)?;
    let mut descriptors = Vec::new();
    let mut prepared_points = Vec::new();
    for verified in verified_text {
        let file_descriptors = chunk_descriptors(
            &verified.relative_path,
            &verified.raw_sha256,
            &verified.bytes,
        )?;
        for descriptor in &file_descriptors {
            prepared_points.push(WorkspaceCoveragePreparedPoint {
                point_id: String::new(),
                text: std::str::from_utf8(
                    &verified.bytes[descriptor.byte_start as usize..descriptor.byte_end as usize],
                )?
                .to_string(),
            });
        }
        descriptors.extend(file_descriptors);
    }
    let plan = derive_workspace_coverage_projection_plan(
        &obligation,
        PROJECTION_SCHEMA,
        CHUNKER_ID,
        embedding_provider_id,
        embedding_model,
        vector_dimensions,
        descriptors,
    )?;
    let body_store = body.body_store().to_path_buf();
    if prepared_points.len() != plan.planned_points.len() {
        bail!("prepared workspace text lost its sealed plan cardinality");
    }
    for (prepared, planned) in prepared_points.iter_mut().zip(&plan.planned_points) {
        prepared.point_id = planned.point_id.clone();
        if format!("{:x}", Sha256::digest(prepared.text.as_bytes())) != planned.chunk_sha256 {
            bail!("prepared workspace text lost its sealed chunk identity");
        }
    }
    Ok(PreparedWorkspaceCoverageProjection {
        body_store,
        basis: basis.clone(),
        obligation,
        plan,
        points: prepared_points,
    })
}

pub(crate) fn acquire_workspace_coverage_projection(
    prepared: &PreparedWorkspaceCoverageProjection,
    executor_id: &str,
    executor_incarnation: &str,
    managed_process_launch_id: &str,
) -> Result<WorkspaceCoverageAcquireResult> {
    if executor_id.trim().is_empty()
        || executor_incarnation.trim().is_empty()
        || managed_process_launch_id.trim().is_empty()
    {
        bail!("workspace coverage acquisition requires exact executor lifecycle identity");
    }
    let body_store = prepared.body_store.clone();
    let obligation = prepared.obligation.clone();
    let plan = prepared.plan.clone();
    let backing = SingleFileMessagePackBackingStore::new(&body_store);
    let opening = backing.pull_all()?;
    let basis = &prepared.basis;
    let authority = exact_body_authority(&opening, basis)?;
    if let Some(receipt) = validate_current_projection(&opening, prepared)? {
        return Ok(WorkspaceCoverageAcquireResult::Current(receipt));
    }
    let existing_claim = find(&opening, CLAIM_TYPE, CLAIM_KEY);
    let claim_epoch = match existing_claim {
        Some(envelope) => {
            let prior: WorkspaceCoverageProjectionClaim = decode(envelope)?;
            validate_projection_claim(&prior)?;
            let prior_attempt_env = find(&opening, ATTEMPT_TYPE, &prior.attempt_id)
                .ok_or_else(|| anyhow!("existing workspace coverage claim attempt missing"))?;
            let prior_attempt: WorkspaceCoverageProjectionAttempt = decode(prior_attempt_env)?;
            validate_claim_attempt_link(&prior, &prior_attempt)?;
            if prior.status == "running" {
                if prior.schema_version == CLAIM_SCHEMA_V3
                    && prior_attempt.schema_version == ATTEMPT_SCHEMA_V3
                    && prior.executor_id == executor_id
                    && prior.executor_incarnation == executor_incarnation
                    && prior.managed_process_launch_id == managed_process_launch_id
                    && prior.plan_id == plan.plan_id
                    && prior.obligation_id == obligation.obligation_id
                    && prior.body_observation_id == basis.observation_id
                    && prior.body_generation == basis.generation
                    && prior.manifest_root_sha256 == basis.manifest_root_sha256
                {
                    return Ok(WorkspaceCoverageAcquireResult::Acquired(
                        WorkspaceCoverageAcquisition {
                            body_store,
                            obligation,
                            plan,
                            claim: prior,
                            attempt: prior_attempt,
                            prior_head: find(&opening, HEAD_TYPE, HEAD_KEY).cloned(),
                        },
                    ));
                }
                return Ok(WorkspaceCoverageAcquireResult::Contended);
            }
            prior
                .claim_epoch
                .checked_add(1)
                .ok_or_else(|| anyhow!("claim epoch exhausted"))?
        }
        None => 1,
    };
    let claim_id = uuid::Uuid::new_v4().to_string();
    let attempt_id = uuid::Uuid::new_v4().to_string();
    let claim = WorkspaceCoverageProjectionClaim {
        schema_version: CLAIM_SCHEMA.into(),
        claim_id: claim_id.clone(),
        claim_epoch,
        plan_id: plan.plan_id.clone(),
        attempt_id: attempt_id.clone(),
        obligation_id: obligation.obligation_id.clone(),
        body_observation_id: basis.observation_id.clone(),
        body_generation: basis.generation,
        manifest_root_sha256: basis.manifest_root_sha256.clone(),
        status: "running".into(),
        executor_id: executor_id.into(),
        executor_incarnation: executor_incarnation.into(),
        managed_process_launch_id: managed_process_launch_id.into(),
        termination_evidence_id: None,
        termination_evidence_digest: None,
        recovery_receipt_id: None,
        recovery_receipt_digest: None,
    };
    let attempt = WorkspaceCoverageProjectionAttempt {
        schema_version: ATTEMPT_SCHEMA.into(),
        attempt_id: attempt_id.clone(),
        claim_id,
        claim_epoch,
        plan_id: plan.plan_id.clone(),
        status: "running".into(),
        started_at: chrono::Utc::now().to_rfc3339(),
        completed_at: None,
        error: None,
        executor_id: executor_id.into(),
        executor_incarnation: executor_incarnation.into(),
        managed_process_launch_id: managed_process_launch_id.into(),
        termination_evidence_id: None,
        termination_evidence_digest: None,
        recovery_receipt_id: None,
        recovery_receipt_digest: None,
    };
    let mut expected = authority.clone();
    if let Some(existing) = existing_claim {
        expected.push(existing.clone());
    }
    let mut replacements = authority;
    let obligation_env = envelope(
        "gamecult.epiphany.workspace_coverage_obligation",
        &obligation.obligation_id,
        &obligation,
    )?;
    let plan_env = envelope(
        "gamecult.epiphany.workspace_coverage_projection_plan",
        &plan.plan_id,
        &plan,
    )?;
    preserve_or_insert_immutable(&opening, &mut expected, &mut replacements, obligation_env)?;
    preserve_or_insert_immutable(&opening, &mut expected, &mut replacements, plan_env)?;
    replacements.push(envelope(CLAIM_TYPE, CLAIM_KEY, &claim)?);
    replacements.push(envelope(ATTEMPT_TYPE, &attempt_id, &attempt)?);
    if !backing.compare_and_swap_batch(&expected, replacements)? {
        bail!("workspace coverage acquisition lost exact Body/claim CAS");
    }
    Ok(WorkspaceCoverageAcquireResult::Acquired(
        WorkspaceCoverageAcquisition {
            body_store,
            obligation,
            plan,
            claim,
            attempt,
            prior_head: find(&opening, HEAD_TYPE, HEAD_KEY).cloned(),
        },
    ))
}

/// Atomically transfers a projection lease only after CultMesh proves the old
/// process dead and the replacement process ready. This does not launch either
/// process; the supervisor must establish the evidence ordering first.
pub fn current_workspace_coverage_recovery_target(
    runtime_store: impl AsRef<Path>,
) -> Result<Option<WorkspaceCoverageRecoveryTarget>> {
    let runtime_store = runtime_store.as_ref();
    let route = runtime_repository_body_store_binding(runtime_store)?
        .ok_or_else(|| anyhow!("workspace coverage runtime has no Body-store binding"))?;
    let opening = SingleFileMessagePackBackingStore::new(&route.body_store_path).pull_all()?;
    let Some(claim_env) = find(&opening, CLAIM_TYPE, CLAIM_KEY) else {
        return Ok(None);
    };
    let claim: WorkspaceCoverageProjectionClaim = decode(claim_env)?;
    let attempt_env = find(&opening, ATTEMPT_TYPE, &claim.attempt_id)
        .ok_or_else(|| anyhow!("workspace coverage current attempt is absent"))?;
    let attempt: WorkspaceCoverageProjectionAttempt = decode(attempt_env)?;
    validate_claim_attempt_link(&claim, &attempt)?;
    if claim.status != "running" {
        return Ok(None);
    }
    Ok(Some(WorkspaceCoverageRecoveryTarget {
        claim_id: claim.claim_id,
        claim_epoch: claim.claim_epoch,
        plan_id: claim.plan_id,
        managed_process_launch_id: claim.managed_process_launch_id,
    }))
}

pub fn recover_workspace_coverage_projection(
    runtime_store: impl AsRef<Path>,
    cultmesh_store: impl AsRef<Path>,
    runtime_id: &str,
    host: &HostIncarnationIdentityEntry,
    old_launch_id: &str,
    replacement_launch_id: &str,
    replacement_ready_heartbeat_id: &str,
    expected_old_claim_id: &str,
) -> Result<WorkspaceCoverageRecoveryOutcome> {
    let runtime_store = runtime_store.as_ref();
    let body_route = runtime_repository_body_store_binding(runtime_store)?
        .ok_or_else(|| anyhow!("workspace coverage recovery runtime has no Body-store binding"))?;
    if body_route.runtime_id != runtime_id {
        bail!("workspace coverage recovery runtime disagrees with its Body route");
    }
    let cultmesh_store = cultmesh_store.as_ref();
    let (old_launch, _) =
        authenticate_workspace_coverage_managed_process_launch_with_envelope_digest(
            cultmesh_store,
            runtime_id,
            old_launch_id,
            host,
        )?;
    let (termination, termination_digest) =
        authenticate_workspace_coverage_termination_with_envelope_digest(
            cultmesh_store,
            runtime_id,
            old_launch_id,
            host,
        )?;
    let (replacement_launch, replacement_launch_digest) =
        authenticate_workspace_coverage_managed_process_launch_with_envelope_digest(
            cultmesh_store,
            runtime_id,
            replacement_launch_id,
            host,
        )?;
    let (ready, ready_digest) =
        authenticate_workspace_coverage_provider_heartbeat_with_envelope_digest(
            cultmesh_store,
            runtime_id,
            replacement_ready_heartbeat_id,
            host,
        )?;
    let latest = load_latest_workspace_coverage_provider_heartbeat(
        cultmesh_store,
        runtime_id,
        &replacement_launch.launch_id,
    )?
    .ok_or_else(|| anyhow!("workspace coverage recovery replacement has no current heartbeat"))?;
    if old_launch.launch_id == replacement_launch.launch_id
        || replacement_launch.replaces_launch_id.as_deref() != Some(old_launch.launch_id.as_str())
        || replacement_launch.replaces_termination_id.as_deref()
            != Some(termination.termination_id.as_str())
        || replacement_launch
            .replaces_termination_envelope_digest
            .as_deref()
            != Some(termination_digest.as_str())
        || ready.launch_id != replacement_launch.launch_id
        || latest.heartbeat_id != ready.heartbeat_id
        || ready.status != "ready"
        || ready.sequence == 0
    {
        bail!("workspace coverage recovery replacement is not exact and ready");
    }
    let terminated_at = chrono::DateTime::parse_from_rfc3339(&termination.observed_at_utc)?;
    let replacement_launched_at =
        chrono::DateTime::parse_from_rfc3339(&replacement_launch.launched_at_utc)?;
    if terminated_at >= replacement_launched_at {
        bail!("workspace coverage recovery requires termination before replacement launch");
    }

    let body_store = PathBuf::from(&body_route.body_store_path);
    let body_basis = observe_runtime_repository_body_basis(runtime_store)?;
    RepositoryBodyReadSession::open(runtime_store, &body_basis)?;
    let backing = SingleFileMessagePackBackingStore::new(&body_store);
    let opening = backing.pull_all()?;
    let claim_env = find(&opening, CLAIM_TYPE, CLAIM_KEY)
        .ok_or_else(|| anyhow!("workspace coverage recovery running claim is absent"))?;
    let old_claim: WorkspaceCoverageProjectionClaim = decode(claim_env)?;
    let attempt_env = find(&opening, ATTEMPT_TYPE, &old_claim.attempt_id)
        .ok_or_else(|| anyhow!("workspace coverage recovery running attempt is absent"))?;
    let old_attempt: WorkspaceCoverageProjectionAttempt = decode(attempt_env)?;
    validate_claim_attempt_link(&old_claim, &old_attempt)?;
    if old_claim.schema_version != CLAIM_SCHEMA
        || old_attempt.schema_version != ATTEMPT_SCHEMA
        || old_claim.status != "running"
        || old_claim.claim_id != expected_old_claim_id
        || old_claim.managed_process_launch_id != old_launch.launch_id
        || old_claim.executor_id != old_launch.provider_daemon_id
        || old_claim.executor_incarnation != old_launch.provider_incarnation_id
    {
        bail!("workspace coverage recovery refuses moved, terminal, legacy, or misbound authority");
    }

    let body_head_env = find(&opening, BODY_HEAD_TYPE, BODY_HEAD_KEY)
        .ok_or_else(|| anyhow!("current Body head missing"))?;
    let body_head: RepositoryBodyHead = decode(body_head_env)?;
    if body_head.observation_id != old_claim.body_observation_id
        || body_head.generation != old_claim.body_generation
        || body_head.manifest_root_sha256 != old_claim.manifest_root_sha256
    {
        bail!("workspace coverage recovery refuses moved Body authority");
    }
    let mut authority = vec![
        find(&opening, BODY_BINDING_TYPE, BODY_BINDING_KEY)
            .cloned()
            .ok_or_else(|| anyhow!("Body binding missing"))?,
        body_head_env.clone(),
        find(
            &opening,
            BODY_OBSERVATION_TYPE,
            &old_claim.body_observation_id,
        )
        .cloned()
        .ok_or_else(|| anyhow!("Body observation missing"))?,
        find(
            &opening,
            BODY_MANIFEST_TYPE,
            &old_claim.manifest_root_sha256,
        )
        .cloned()
        .ok_or_else(|| anyhow!("Body manifest missing"))?,
    ];
    let obligation_env = find(&opening, OBLIGATION_TYPE, &old_claim.obligation_id)
        .cloned()
        .ok_or_else(|| anyhow!("projection obligation missing"))?;
    let plan_env = find(&opening, PLAN_TYPE, &old_claim.plan_id)
        .cloned()
        .ok_or_else(|| anyhow!("projection plan missing"))?;
    let obligation: WorkspaceCoverageObligation = decode(&obligation_env)?;
    let plan: WorkspaceCoverageProjectionPlan = decode(&plan_env)?;
    validate_workspace_coverage_projection_plan(&obligation, &plan)?;
    if plan.plan_id != old_claim.plan_id
        || plan.obligation_id != old_claim.obligation_id
        || obligation.runtime_id != body_basis.runtime_id
        || obligation.swarm_id != body_basis.swarm_id
        || obligation.workspace_id != body_basis.workspace_id
        || obligation.body_binding_sha256 != body_basis.body_binding_sha256
        || obligation.body_observation_id != body_basis.observation_id
        || obligation.body_generation != body_basis.generation
        || obligation.manifest_root_sha256 != body_basis.manifest_root_sha256
        || old_claim.body_observation_id != body_basis.observation_id
        || old_claim.body_generation != body_basis.generation
        || old_claim.manifest_root_sha256 != body_basis.manifest_root_sha256
    {
        bail!("workspace coverage recovery refuses substituted plan");
    }
    authority.push(obligation_env.clone());
    authority.push(plan_env.clone());
    let prior_head = find(&opening, HEAD_TYPE, HEAD_KEY).cloned();
    if let Some(head) = &prior_head {
        authority.push(head.clone());
    }

    let claim_epoch = old_claim
        .claim_epoch
        .checked_add(1)
        .ok_or_else(|| anyhow!("claim epoch exhausted"))?;
    let claim_id = uuid::Uuid::new_v4().to_string();
    let attempt_id = uuid::Uuid::new_v4().to_string();
    let recovery_id = uuid::Uuid::new_v4().to_string();
    let receipt = WorkspaceCoverageRecoveryReceipt {
        schema_version: RECOVERY_SCHEMA.into(),
        recovery_id: recovery_id.clone(),
        old_claim_id: old_claim.claim_id.clone(),
        old_attempt_id: old_attempt.attempt_id.clone(),
        old_claim_epoch: old_claim.claim_epoch,
        termination_id: termination.termination_id.clone(),
        termination_envelope_digest: termination_digest.clone(),
        replacement_launch_id: replacement_launch.launch_id.clone(),
        replacement_launch_envelope_digest: replacement_launch_digest,
        ready_heartbeat_id: ready.heartbeat_id.clone(),
        ready_heartbeat_envelope_digest: ready_digest,
        new_claim_id: claim_id.clone(),
        new_attempt_id: attempt_id.clone(),
        new_claim_epoch: claim_epoch,
        recovered_at_utc: ready.observed_at_utc.clone(),
        old_launch_id: old_launch.launch_id.clone(),
    };
    validate_recovery_receipt(&receipt)?;
    let receipt_env = envelope(RECOVERY_TYPE, &recovery_id, &receipt)?;
    let receipt_digest = cultcache_envelope_digest(&receipt_env);

    let completed_at = termination.observed_at_utc.clone();
    let mut failed_claim = old_claim.clone();
    failed_claim.schema_version = CLAIM_SCHEMA_V2.into();
    failed_claim.status = "failed".into();
    failed_claim.termination_evidence_id = Some(termination.termination_id.clone());
    failed_claim.termination_evidence_digest = Some(termination_digest.clone());
    failed_claim.recovery_receipt_id = Some(recovery_id.clone());
    failed_claim.recovery_receipt_digest = Some(receipt_digest.clone());
    let mut failed_attempt = old_attempt.clone();
    failed_attempt.schema_version = ATTEMPT_SCHEMA_V2.into();
    failed_attempt.status = "failed".into();
    failed_attempt.completed_at = Some(completed_at.clone());
    failed_attempt.error = Some("executor process terminated; lease recovered".into());
    failed_attempt.termination_evidence_id = Some(termination.termination_id.clone());
    failed_attempt.termination_evidence_digest = Some(termination_digest);
    failed_attempt.recovery_receipt_id = Some(recovery_id.clone());
    failed_attempt.recovery_receipt_digest = Some(receipt_digest.clone());
    validate_claim_attempt_link(&failed_claim, &failed_attempt)?;

    let claim = WorkspaceCoverageProjectionClaim {
        schema_version: CLAIM_SCHEMA_V3.into(),
        claim_id: claim_id.clone(),
        claim_epoch,
        plan_id: plan.plan_id.clone(),
        attempt_id: attempt_id.clone(),
        obligation_id: obligation.obligation_id.clone(),
        body_observation_id: old_claim.body_observation_id.clone(),
        body_generation: old_claim.body_generation,
        manifest_root_sha256: old_claim.manifest_root_sha256.clone(),
        status: "running".into(),
        executor_id: replacement_launch.provider_daemon_id.clone(),
        executor_incarnation: replacement_launch.provider_incarnation_id.clone(),
        managed_process_launch_id: replacement_launch.launch_id.clone(),
        termination_evidence_id: None,
        termination_evidence_digest: None,
        recovery_receipt_id: Some(recovery_id.clone()),
        recovery_receipt_digest: Some(receipt_digest.clone()),
    };
    let attempt = WorkspaceCoverageProjectionAttempt {
        schema_version: ATTEMPT_SCHEMA_V3.into(),
        attempt_id: attempt_id.clone(),
        claim_id,
        claim_epoch,
        plan_id: plan.plan_id.clone(),
        status: "running".into(),
        started_at: ready.observed_at_utc.clone(),
        completed_at: None,
        error: None,
        executor_id: replacement_launch.provider_daemon_id.clone(),
        executor_incarnation: replacement_launch.provider_incarnation_id.clone(),
        managed_process_launch_id: replacement_launch.launch_id.clone(),
        termination_evidence_id: None,
        termination_evidence_digest: None,
        recovery_receipt_id: Some(recovery_id.clone()),
        recovery_receipt_digest: Some(receipt_digest),
    };
    validate_claim_attempt_link(&claim, &attempt)?;
    let mut expected = authority.clone();
    expected.extend([claim_env.clone(), attempt_env.clone()]);
    let mut replacements = authority;
    let history_key = format!("history/{}", old_claim.claim_id);
    if find(&opening, CLAIM_TYPE, &history_key).is_some() {
        bail!("workspace coverage recovery claim history identity already exists");
    }
    if find(&opening, RECOVERY_TYPE, &recovery_id).is_some() {
        bail!("workspace coverage recovery receipt identity already exists");
    }
    replacements.extend([
        receipt_env,
        envelope(CLAIM_TYPE, &history_key, &failed_claim)?,
        envelope(ATTEMPT_TYPE, &failed_attempt.attempt_id, &failed_attempt)?,
        envelope(CLAIM_TYPE, CLAIM_KEY, &claim)?,
        envelope(ATTEMPT_TYPE, &attempt.attempt_id, &attempt)?,
    ]);
    if !backing.compare_and_swap_batch(&expected, replacements)? {
        bail!("workspace coverage recovery lost exact Body/plan/claim/attempt CAS");
    }
    Ok(WorkspaceCoverageRecoveryOutcome {
        claim_id: claim.claim_id,
        claim_epoch: claim.claim_epoch,
        managed_process_launch_id: claim.managed_process_launch_id,
        executor_incarnation: claim.executor_incarnation,
        recovery_receipt_id: recovery_id,
        recovery_receipt_digest: claim
            .recovery_receipt_digest
            .ok_or_else(|| anyhow!("recovered claim lost recovery receipt digest"))?,
    })
}

fn validate_current_projection(
    opening: &[CultCacheEnvelope],
    prepared: &PreparedWorkspaceCoverageProjection,
) -> Result<Option<WorkspaceCoverageReceipt>> {
    let Some(head_env) = find(opening, HEAD_TYPE, HEAD_KEY) else {
        return Ok(None);
    };
    let head: WorkspaceCoverageHead = decode(head_env)?;
    let obligation_env = find(
        opening,
        "gamecult.epiphany.workspace_coverage_obligation",
        &head.obligation_id,
    )
    .ok_or_else(|| anyhow!("current workspace coverage head names a missing obligation"))?;
    let plan_env = find(
        opening,
        "gamecult.epiphany.workspace_coverage_projection_plan",
        &head.plan_id,
    )
    .ok_or_else(|| anyhow!("current workspace coverage head names a missing plan"))?;
    let receipt_env = find(opening, RECEIPT_TYPE, &head.receipt_id)
        .ok_or_else(|| anyhow!("current workspace coverage head names a missing receipt"))?;
    let obligation: WorkspaceCoverageObligation = decode(obligation_env)?;
    let plan: WorkspaceCoverageProjectionPlan = decode(plan_env)?;
    let receipt: WorkspaceCoverageReceipt = decode(receipt_env)?;
    validate_workspace_coverage_head(&obligation, &plan, &receipt, &head)?;
    if obligation == prepared.obligation && plan == prepared.plan {
        Ok(Some(receipt))
    } else {
        Ok(None)
    }
}

pub(crate) fn fail_workspace_coverage_projection(
    acquisition: &WorkspaceCoverageAcquisition,
    error: &str,
) -> Result<()> {
    if error.trim().is_empty() {
        bail!("projection failure requires an error");
    }
    let backing = SingleFileMessagePackBackingStore::new(&acquisition.body_store);
    let opening = backing.pull_all()?;
    let claim_env =
        find(&opening, CLAIM_TYPE, CLAIM_KEY).ok_or_else(|| anyhow!("running claim missing"))?;
    let attempt_env = find(&opening, ATTEMPT_TYPE, &acquisition.attempt.attempt_id)
        .ok_or_else(|| anyhow!("running attempt missing"))?;
    let claim: WorkspaceCoverageProjectionClaim = decode(claim_env)?;
    let attempt: WorkspaceCoverageProjectionAttempt = decode(attempt_env)?;
    let obligation_env = find(
        &opening,
        "gamecult.epiphany.workspace_coverage_obligation",
        &acquisition.obligation.obligation_id,
    )
    .ok_or_else(|| anyhow!("persisted projection obligation missing"))?;
    let plan_env = find(
        &opening,
        "gamecult.epiphany.workspace_coverage_projection_plan",
        &acquisition.plan.plan_id,
    )
    .ok_or_else(|| anyhow!("persisted projection plan missing"))?;
    if decode::<WorkspaceCoverageObligation>(obligation_env)? != acquisition.obligation
        || decode::<WorkspaceCoverageProjectionPlan>(plan_env)? != acquisition.plan
    {
        bail!("projection failure refuses substituted obligation or plan");
    }
    if claim != acquisition.claim || attempt != acquisition.attempt || claim.status != "running" {
        bail!("projection failure refuses substituted claim or attempt");
    }
    let mut failed_claim = claim.clone();
    failed_claim.status = "failed".into();
    let mut failed_attempt = attempt.clone();
    failed_attempt.status = "failed".into();
    failed_attempt.completed_at = Some(chrono::Utc::now().to_rfc3339());
    failed_attempt.error = Some(error.into());
    let expected = vec![
        obligation_env.clone(),
        plan_env.clone(),
        claim_env.clone(),
        attempt_env.clone(),
    ];
    let replacements = vec![
        obligation_env.clone(),
        plan_env.clone(),
        envelope(CLAIM_TYPE, CLAIM_KEY, &failed_claim)?,
        envelope(ATTEMPT_TYPE, &failed_attempt.attempt_id, &failed_attempt)?,
    ];
    if !backing.compare_and_swap_batch(&expected, replacements)? {
        bail!("projection failure lost exact claim/attempt CAS");
    }
    Ok(())
}

pub(crate) fn execute_workspace_coverage_projection(
    acquisition: &WorkspaceCoverageAcquisition,
    prepared: &PreparedWorkspaceCoverageProjection,
    embedder: &OllamaEmbedder,
    port: &mut impl WorkspaceCoverageProjectionPort,
    authority: &WorkspaceCoverageProjectionExecutionAuthority<'_>,
) -> Result<WorkspaceCoverageReceipt> {
    let result = (|| {
        if prepared.body_store != acquisition.body_store
            || prepared.obligation != acquisition.obligation
            || prepared.plan != acquisition.plan
        {
            bail!("execution refuses a prepared projection other than the acquired projection");
        }
        execute_batched_and_commit(acquisition, prepared, embedder, port, authority)
    })();
    terminalize_execution_result(acquisition, result)
}

fn execute_batched_and_commit(
    acquisition: &WorkspaceCoverageAcquisition,
    prepared: &PreparedWorkspaceCoverageProjection,
    embedder: &OllamaEmbedder,
    port: &mut impl WorkspaceCoverageProjectionPort,
    authority: &WorkspaceCoverageProjectionExecutionAuthority<'_>,
) -> Result<WorkspaceCoverageReceipt> {
    if authority.operation_timeout_ms == 0 {
        bail!("workspace coverage execution authority requires a positive operation timeout");
    }
    let planned = &acquisition.plan.planned_points;
    if prepared.points.len() != planned.len() {
        bail!("prepared projection does not equal the sealed plan point count");
    }
    for (ordinal, (prepared_point, planned_point)) in
        prepared.points.iter().zip(planned).enumerate()
    {
        if prepared_point.point_id != planned_point.point_id
            || format!("{:x}", Sha256::digest(prepared_point.text.as_bytes()))
                != planned_point.chunk_sha256
        {
            bail!("prepared projection disagrees with sealed plan ordinal {ordinal}");
        }
    }
    let collection_name = workspace_coverage_execution_collection(
        &acquisition.plan.plan_id,
        &acquisition.claim.claim_id,
        acquisition.claim.claim_epoch,
    )?;
    port.ensure_exact_collection(
        &collection_name,
        &collection_compatibility(&acquisition.plan),
    )?;

    let checkpoint_chain = load_authenticated_checkpoint_chain(
        &acquisition.body_store,
        authority.local_verse_store,
        authority.trusted_host,
        &acquisition.claim.claim_id,
        acquisition.claim.claim_epoch,
    )?;
    let mut next_ordinal = if let Some(current) = checkpoint_chain.last() {
        reconcile_workspace_coverage_checkpoint_progress(
            &acquisition.body_store,
            authority.local_verse_store,
            authority.runtime_id,
            authority.trusted_host,
            authority.provider_signing_key,
            &acquisition.claim.claim_id,
            acquisition.claim.claim_epoch,
        )?;
        for checkpoint in &checkpoint_chain {
            authenticate_checkpoint_readback(&collection_name, &checkpoint.checkpoint, port)?;
        }
        current.checkpoint.cumulative_point_count as usize
    } else {
        if let Some(existing) = load_latest_workspace_coverage_projection_progress(
            authority.local_verse_store,
            authority.runtime_id,
            &acquisition.claim.managed_process_launch_id,
            &acquisition.claim.claim_id,
        )? {
            let existing = authenticate_workspace_coverage_projection_progress(
                authority.local_verse_store,
                authority.runtime_id,
                &existing.progress_id,
                authority.trusted_host,
            )?;
            if existing.sequence != 1
                || existing.completed_units != 0
                || existing.checkpoint_id.is_some()
                || existing.plan_id != acquisition.plan.plan_id
                || existing.attempt_id != acquisition.attempt.attempt_id
            {
                bail!("checkpoint-free restart found progress other than exact genesis");
            }
        } else {
            publish_workspace_coverage_progress_genesis(
                &acquisition.body_store,
                authority.local_verse_store,
                authority.runtime_id,
                authority.trusted_host,
                authority.provider_signing_key,
                authority.operation_timeout_ms,
            )?;
        }
        0
    };

    while next_ordinal < planned.len() {
        let end = (next_ordinal + QDRANT_POINT_BATCH_MAX).min(planned.len());
        let texts = prepared.points[next_ordinal..end]
            .iter()
            .map(|point| point.text.clone())
            .collect::<Vec<_>>();
        let vectors = embedder.embed_documents(&texts)?;
        if vectors.len() != texts.len() {
            bail!("embedding provider omitted sealed batch vectors");
        }
        let mut points = Vec::with_capacity(end - next_ordinal);
        let mut expected_point_bindings = Vec::with_capacity(end - next_ordinal);
        let mut expected_vector_bindings = Vec::with_capacity(end - next_ordinal);
        for ((prepared_point, expected), vector) in prepared.points[next_ordinal..end]
            .iter()
            .zip(&planned[next_ordinal..end])
            .zip(vectors)
        {
            if vector.len() != acquisition.plan.vector_dimensions as usize
                || vector.iter().any(|value| !value.is_finite())
            {
                bail!("embedding provider returned an invalid sealed batch vector");
            }
            let payload = payload_for(&acquisition.obligation, &acquisition.plan, expected);
            expected_point_bindings.push(WorkspaceCoveragePointBinding {
                point_id: expected.point_id.clone(),
                payload_sha256: digest(&payload)?,
            });
            expected_vector_bindings.push(WorkspaceCoverageVectorBinding {
                point_id: expected.point_id.clone(),
                vector_sha256: digest(&vector)?,
            });
            points.push(SemanticPoint {
                id: prepared_point.point_id.clone(),
                vector,
                payload,
            });
        }
        port.upsert_waited_batch(&collection_name, &points)?;
        let ids = points
            .iter()
            .map(|point| point.id.clone())
            .collect::<Vec<_>>();
        let observed = port.observe_exact_ids(&collection_name, &ids)?;
        authenticate_batch_readback(
            &observed,
            &expected_point_bindings,
            &expected_vector_bindings,
            acquisition.plan.vector_dimensions,
        )?;
        let checkpoint = admit_observed_workspace_coverage_batch(
            &acquisition.body_store,
            authority.local_verse_store,
            authority.runtime_id,
            authority.trusted_host,
            authority.provider_signing_key,
            ObservedWorkspaceCoverageBatchInput {
                claim_id: acquisition.claim.claim_id.clone(),
                attempt_id: acquisition.attempt.attempt_id.clone(),
                plan_id: acquisition.plan.plan_id.clone(),
                first_plan_ordinal: next_ordinal as u64,
                point_bindings: expected_point_bindings,
                vector_bindings: expected_vector_bindings,
            },
        )?;
        publish_workspace_coverage_progress_for_checkpoint(
            authority.local_verse_store,
            authority.runtime_id,
            authority.provider_signing_key,
            &checkpoint,
        )?;
        next_ordinal = end;
    }

    let final_chain = load_authenticated_checkpoint_chain(
        &acquisition.body_store,
        authority.local_verse_store,
        authority.trusted_host,
        &acquisition.claim.claim_id,
        acquisition.claim.claim_epoch,
    )?;
    let observed = observe_final_projection_against_authenticated_checkpoint_chain(
        &acquisition.obligation,
        &acquisition.plan,
        &collection_name,
        port.observe_all(&collection_name)?,
        &final_chain,
    )?;
    commit_workspace_coverage_success(acquisition, observed)
}

fn authenticate_checkpoint_readback(
    collection_name: &str,
    checkpoint: &crate::workspace_coverage_projection_batch_checkpoint::WorkspaceCoverageProjectionBatchCheckpointEntry,
    port: &mut impl WorkspaceCoverageProjectionPort,
) -> Result<()> {
    if checkpoint.collection_name != collection_name {
        bail!("checkpoint collection disagrees with the sealed execution collection");
    }
    let observed = port.observe_exact_ids(collection_name, &checkpoint.point_ids)?;
    authenticate_batch_readback(
        &observed,
        &checkpoint.point_bindings,
        &checkpoint.vector_bindings,
        checkpoint.vector_dimensions,
    )
}

fn authenticate_batch_readback(
    observed: &[SemanticStoredPoint<WorkspaceCoveragePointPayload>],
    expected_points: &[WorkspaceCoveragePointBinding],
    expected_vectors: &[WorkspaceCoverageVectorBinding],
    vector_dimensions: u32,
) -> Result<()> {
    if observed.len() != expected_points.len() || observed.len() != expected_vectors.len() {
        bail!("exact batch readback count disagrees with submitted bindings");
    }
    // Qdrant's retrieve response is a set; it does not promise to preserve the
    // request order. Authenticate the exact set by point identity instead of
    // letting transport ordering impersonate a payload/vector substitution.
    let mut observed = observed.iter().collect::<Vec<_>>();
    let mut expected_points = expected_points.iter().collect::<Vec<_>>();
    let mut expected_vectors = expected_vectors.iter().collect::<Vec<_>>();
    observed.sort_by(|left, right| left.id.cmp(&right.id));
    expected_points.sort_by(|left, right| left.point_id.cmp(&right.point_id));
    expected_vectors.sort_by(|left, right| left.point_id.cmp(&right.point_id));
    for ((point, expected_point), expected_vector) in observed
        .into_iter()
        .zip(expected_points)
        .zip(expected_vectors)
    {
        let payload = point
            .payload
            .as_ref()
            .ok_or_else(|| anyhow!("exact batch readback omitted payload"))?;
        let vector = point
            .vector
            .as_ref()
            .ok_or_else(|| anyhow!("exact batch readback omitted vector"))?;
        if point.id != expected_point.point_id
            || point.id != expected_vector.point_id
            || digest(payload)? != expected_point.payload_sha256
            || vector.len() != vector_dimensions as usize
            || vector.iter().any(|value| !value.is_finite())
            || digest(vector)? != expected_vector.vector_sha256
        {
            bail!("exact batch readback disagrees with submitted payload/vector bindings");
        }
    }
    Ok(())
}

#[cfg(test)]
fn sealed_batch_ranges(point_count: usize) -> Vec<std::ops::Range<usize>> {
    (0..point_count)
        .step_by(QDRANT_POINT_BATCH_MAX)
        .map(|start| start..(start + QDRANT_POINT_BATCH_MAX).min(point_count))
        .collect()
}

#[cfg(test)]
fn execute_preembedded_workspace_coverage_projection(
    acquisition: &WorkspaceCoverageAcquisition,
    inputs: Vec<WorkspaceCoverageProjectionInput>,
    port: &mut impl WorkspaceCoverageProjectionPort,
) -> Result<WorkspaceCoverageReceipt> {
    let result = execute_and_commit(acquisition, inputs, port);
    terminalize_execution_result(acquisition, result)
}

fn terminalize_execution_result(
    acquisition: &WorkspaceCoverageAcquisition,
    result: Result<WorkspaceCoverageReceipt>,
) -> Result<WorkspaceCoverageReceipt> {
    match result {
        Ok(receipt) => Ok(receipt),
        Err(error) => {
            let message = format!("{error:#}");
            if let Err(terminal_error) = fail_workspace_coverage_projection(acquisition, &message) {
                bail!(
                    "projection failed ({message}) and failure terminalization failed: {terminal_error:#}"
                );
            }
            Err(error)
        }
    }
}

#[cfg(test)]
fn execute_and_commit(
    acquisition: &WorkspaceCoverageAcquisition,
    mut inputs: Vec<WorkspaceCoverageProjectionInput>,
    port: &mut impl WorkspaceCoverageProjectionPort,
) -> Result<WorkspaceCoverageReceipt> {
    inputs.sort_by(|left, right| left.point_id.cmp(&right.point_id));
    let mut planned = acquisition.plan.planned_points.clone();
    planned.sort_by(|left, right| left.point_id.cmp(&right.point_id));
    if inputs.len() != planned.len() {
        bail!("projection inputs do not equal the sealed plan point count");
    }
    let mut points = Vec::with_capacity(planned.len());
    let mut submitted_vector_bindings = Vec::with_capacity(planned.len());
    for (input, expected) in inputs.into_iter().zip(&planned) {
        if input.point_id != expected.point_id
            || input.vector.len() != acquisition.plan.vector_dimensions as usize
            || input.vector.iter().any(|value| !value.is_finite())
            || format!("{:x}", Sha256::digest(input.text.as_bytes())) != expected.chunk_sha256
        {
            bail!("projection input does not match its sealed point descriptor");
        }
        submitted_vector_bindings.push(WorkspaceCoverageVectorBinding {
            point_id: expected.point_id.clone(),
            vector_sha256: digest(&input.vector)?,
        });
        points.push(SemanticPoint {
            id: expected.point_id.clone(),
            vector: input.vector,
            payload: payload_for(&acquisition.obligation, &acquisition.plan, expected),
        });
    }
    let collection_name = workspace_coverage_execution_collection(
        &acquisition.plan.plan_id,
        &acquisition.claim.claim_id,
        acquisition.claim.claim_epoch,
    )?;
    let compatibility = collection_compatibility(&acquisition.plan);
    port.ensure_exact_collection(&collection_name, &compatibility)?;
    if !points.is_empty() {
        port.upsert_waited_batch(&collection_name, &points)?;
    }
    let observed = observe_exact_bindings(
        acquisition,
        &collection_name,
        port.observe_all(&collection_name)?,
        &submitted_vector_bindings,
    )?;
    commit_workspace_coverage_success(acquisition, observed)
}

#[cfg(test)]
fn observe_exact_bindings(
    acquisition: &WorkspaceCoverageAcquisition,
    collection_name: &str,
    observed: Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>,
    submitted_vector_bindings: &[WorkspaceCoverageVectorBinding],
) -> Result<WorkspaceCoverageObservedBinding> {
    let binding = observe_sealed_bindings(
        &acquisition.obligation,
        &acquisition.plan,
        collection_name,
        observed,
        Some(submitted_vector_bindings),
    )?;
    Ok(binding)
}

pub(crate) fn observe_final_projection_against_authenticated_checkpoint_chain(
    obligation: &WorkspaceCoverageObligation,
    plan: &WorkspaceCoverageProjectionPlan,
    collection_name: &str,
    observed: Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>,
    chain: &[WorkspaceCoverageProjectionBatchCheckpointAdmission],
) -> Result<WorkspaceCoverageObservedBinding> {
    let mut expected_points = Vec::new();
    let mut expected_vectors = Vec::new();
    let mut expected_ordinal = 0_u64;
    for admission in chain {
        let checkpoint = &admission.checkpoint;
        if checkpoint.obligation_id != obligation.obligation_id
            || checkpoint.plan_id != plan.plan_id
            || checkpoint.first_plan_ordinal != expected_ordinal
            || checkpoint.point_bindings.len() != checkpoint.vector_bindings.len()
        {
            bail!("authenticated checkpoint chain disagrees with final sealed projection");
        }
        expected_points.extend(checkpoint.point_bindings.iter().cloned());
        expected_vectors.extend(checkpoint.vector_bindings.iter().cloned());
        expected_ordinal = checkpoint.cumulative_point_count;
    }
    if expected_ordinal != plan.expected_point_count
        || expected_points.len() as u64 != plan.expected_point_count
        || expected_vectors.len() != expected_points.len()
    {
        bail!("authenticated checkpoint chain does not cover the complete sealed plan");
    }
    for ((planned, point), vector) in plan
        .planned_points
        .iter()
        .zip(&expected_points)
        .zip(&expected_vectors)
    {
        if point.point_id != planned.point_id
            || vector.point_id != planned.point_id
            || point.payload_sha256 != digest(&payload_for(obligation, plan, planned))?
        {
            bail!("authenticated checkpoint chain order or payload binding is invalid");
        }
    }
    let ordered = authenticate_final_observation_against_checkpoint_bindings(
        observed,
        &expected_points,
        &expected_vectors,
    )?;
    observe_sealed_bindings(
        obligation,
        plan,
        collection_name,
        ordered,
        Some(&expected_vectors),
    )
}

fn authenticate_final_observation_against_checkpoint_bindings(
    observed: Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>,
    expected_points: &[WorkspaceCoveragePointBinding],
    expected_vectors: &[WorkspaceCoverageVectorBinding],
) -> Result<Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>> {
    let mut observed_by_id = std::collections::HashMap::with_capacity(observed.len());
    for point in observed {
        let point_id = point.id.clone();
        if observed_by_id.insert(point_id.clone(), point).is_some() {
            bail!("final Qdrant observation duplicated checkpointed point {point_id}");
        }
    }
    if observed_by_id.len() != expected_points.len() {
        bail!("final Qdrant observation cardinality disagrees with authenticated checkpoint chain");
    }
    let mut ordered = Vec::with_capacity(expected_points.len());
    for (point, vector) in expected_points.iter().zip(expected_vectors) {
        let observed = observed_by_id
            .remove(&point.point_id)
            .ok_or_else(|| anyhow!("final Qdrant observation omitted a checkpointed point"))?;
        let payload = observed
            .payload
            .as_ref()
            .ok_or_else(|| anyhow!("final Qdrant observation omitted checkpointed payload"))?;
        let observed_vector = observed
            .vector
            .as_ref()
            .ok_or_else(|| anyhow!("final Qdrant observation omitted checkpointed vector"))?;
        if digest(payload)? != point.payload_sha256
            || digest(observed_vector)? != vector.vector_sha256
        {
            bail!("final Qdrant payload/vector disagrees with authenticated checkpoint evidence");
        }
        ordered.push(observed);
    }
    if !observed_by_id.is_empty() {
        bail!("final Qdrant observation contains uncheckpointed points");
    }
    Ok(ordered)
}

fn observe_sealed_bindings(
    obligation: &WorkspaceCoverageObligation,
    plan: &WorkspaceCoverageProjectionPlan,
    collection_name: &str,
    observed: Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>,
    expected_vector_bindings: Option<&[WorkspaceCoverageVectorBinding]>,
) -> Result<WorkspaceCoverageObservedBinding> {
    let mut bindings = Vec::with_capacity(observed.len());
    let mut vector_bindings = Vec::with_capacity(observed.len());
    for point in observed {
        let payload = point
            .payload
            .ok_or_else(|| anyhow!("observed workspace point omitted payload"))?;
        bindings.push(WorkspaceCoveragePointBinding {
            point_id: point.id.clone(),
            payload_sha256: digest(&payload)?,
        });
        let vector = point
            .vector
            .ok_or_else(|| anyhow!("observed workspace point omitted vector"))?;
        if vector.len() != plan.vector_dimensions as usize
            || vector.iter().any(|value| !value.is_finite())
        {
            bail!("observed workspace point has an invalid vector");
        }
        vector_bindings.push(WorkspaceCoverageVectorBinding {
            point_id: point.id,
            vector_sha256: digest(&vector)?,
        });
    }
    bindings.sort_by(|left, right| left.point_id.cmp(&right.point_id));
    if bindings
        .windows(2)
        .any(|pair| pair[0].point_id == pair[1].point_id)
    {
        bail!("observed workspace collection contains duplicate point IDs");
    }
    if bindings != plan.point_bindings {
        bail!("observed workspace point bindings do not equal the sealed plan");
    }
    vector_bindings.sort_by(|left, right| left.point_id.cmp(&right.point_id));
    if expected_vector_bindings.is_some_and(|expected| vector_bindings != expected) {
        bail!("observed workspace vectors do not equal the submitted vectors");
    }
    // Payload hashes bind the exact obligation as well as the sealed plan.
    // Recompute one expected payload per point rather than trusting Qdrant's
    // typed decoding to establish semantic equality.
    for point in &plan.planned_points {
        let expected = payload_for(obligation, plan, point);
        let expected_binding = WorkspaceCoveragePointBinding {
            point_id: point.point_id.clone(),
            payload_sha256: digest(&expected)?,
        };
        if !bindings.contains(&expected_binding) {
            bail!("observed workspace payload does not equal its sealed obligation and plan");
        }
    }
    let ids = bindings
        .iter()
        .map(|binding| binding.point_id.clone())
        .collect::<Vec<_>>();
    Ok(WorkspaceCoverageObservedBinding {
        collection_name: collection_name.into(),
        point_count: bindings.len() as u64,
        point_set_sha256: digest(&ids)?,
        point_binding_set_sha256: digest(&bindings)?,
        vector_binding_set_sha256: digest(&vector_bindings)?,
    })
}

fn commit_workspace_coverage_success(
    acquisition: &WorkspaceCoverageAcquisition,
    observed: WorkspaceCoverageObservedBinding,
) -> Result<WorkspaceCoverageReceipt> {
    if observed.point_count != acquisition.plan.expected_point_count
        || observed.point_set_sha256 != acquisition.plan.expected_point_set_sha256
        || observed.point_binding_set_sha256 != acquisition.plan.point_binding_set_sha256
    {
        bail!("observed binding proof is not the sealed plan");
    }
    let backing = SingleFileMessagePackBackingStore::new(&acquisition.body_store);
    let opening = backing.pull_all()?;
    let authority = exact_obligation_body_authority(&opening, &acquisition.obligation)?;
    let obligation_env = find(
        &opening,
        "gamecult.epiphany.workspace_coverage_obligation",
        &acquisition.obligation.obligation_id,
    )
    .ok_or_else(|| anyhow!("persisted projection obligation missing"))?;
    let plan_env = find(
        &opening,
        "gamecult.epiphany.workspace_coverage_projection_plan",
        &acquisition.plan.plan_id,
    )
    .ok_or_else(|| anyhow!("persisted projection plan missing"))?;
    let claim_env =
        find(&opening, CLAIM_TYPE, CLAIM_KEY).ok_or_else(|| anyhow!("running claim missing"))?;
    let attempt_env = find(&opening, ATTEMPT_TYPE, &acquisition.attempt.attempt_id)
        .ok_or_else(|| anyhow!("running attempt missing"))?;
    if decode::<WorkspaceCoverageObligation>(obligation_env)? != acquisition.obligation
        || decode::<WorkspaceCoverageProjectionPlan>(plan_env)? != acquisition.plan
        || decode::<WorkspaceCoverageProjectionClaim>(claim_env)? != acquisition.claim
        || decode::<WorkspaceCoverageProjectionAttempt>(attempt_env)? != acquisition.attempt
    {
        bail!("success refuses substituted obligation, plan, claim, or attempt");
    }
    if find(&opening, HEAD_TYPE, HEAD_KEY) != acquisition.prior_head.as_ref() {
        bail!("workspace coverage success lost its acquired prior head");
    }
    let observed_at = chrono::Utc::now().to_rfc3339();
    let receipt_id = digest(&(
        &acquisition.obligation.obligation_id,
        &acquisition.plan.plan_id,
        &observed.collection_name,
        &observed.point_binding_set_sha256,
        &observed.vector_binding_set_sha256,
        &acquisition.claim.claim_id,
        acquisition.claim.claim_epoch,
    ))?;
    let receipt = WorkspaceCoverageReceipt {
        schema_version: WORKSPACE_COVERAGE_RECEIPT_SCHEMA_VERSION.into(),
        receipt_id,
        obligation_id: acquisition.obligation.obligation_id.clone(),
        plan_id: acquisition.plan.plan_id.clone(),
        collection_name: observed.collection_name,
        embedding_provider_id: acquisition.plan.embedding_provider_id.clone(),
        embedding_model: acquisition.plan.embedding_model.clone(),
        vector_dimensions: acquisition.plan.vector_dimensions,
        observed_point_count: observed.point_count,
        observed_point_set_sha256: observed.point_set_sha256,
        observed_at,
        observation_method: "qdrant_scroll_exact_point_bindings".into(),
        observed_point_binding_set_sha256: observed.point_binding_set_sha256,
        claim_id: acquisition.claim.claim_id.clone(),
        claim_epoch: acquisition.claim.claim_epoch,
        observed_vector_binding_set_sha256: observed.vector_binding_set_sha256,
    };
    let head = WorkspaceCoverageHead {
        schema_version: WORKSPACE_COVERAGE_HEAD_SCHEMA_VERSION.into(),
        workspace_id: acquisition.obligation.workspace_id.clone(),
        obligation_id: acquisition.obligation.obligation_id.clone(),
        plan_id: acquisition.plan.plan_id.clone(),
        receipt_id: receipt.receipt_id.clone(),
        body_observation_id: acquisition.obligation.body_observation_id.clone(),
        body_generation: acquisition.obligation.body_generation,
        manifest_root_sha256: acquisition.obligation.manifest_root_sha256.clone(),
        claim_id: acquisition.claim.claim_id.clone(),
        claim_epoch: acquisition.claim.claim_epoch,
    };
    validate_workspace_coverage_head(&acquisition.obligation, &acquisition.plan, &receipt, &head)?;
    let mut completed_claim = acquisition.claim.clone();
    completed_claim.status = "succeeded".into();
    let mut completed_attempt = acquisition.attempt.clone();
    completed_attempt.status = "succeeded".into();
    completed_attempt.completed_at = Some(chrono::Utc::now().to_rfc3339());
    let mut expected = authority.clone();
    expected.extend([
        obligation_env.clone(),
        plan_env.clone(),
        claim_env.clone(),
        attempt_env.clone(),
    ]);
    if let Some(prior_head) = &acquisition.prior_head {
        expected.push(prior_head.clone());
    }
    let mut replacements = authority;
    replacements.extend([
        obligation_env.clone(),
        plan_env.clone(),
        envelope(CLAIM_TYPE, CLAIM_KEY, &completed_claim)?,
        envelope(
            ATTEMPT_TYPE,
            &completed_attempt.attempt_id,
            &completed_attempt,
        )?,
        envelope(RECEIPT_TYPE, &receipt.receipt_id, &receipt)?,
        envelope(HEAD_TYPE, HEAD_KEY, &head)?,
    ]);
    if !backing.compare_and_swap_batch(&expected, replacements)? {
        bail!("workspace coverage success lost exact Body/plan/claim CAS");
    }
    Ok(receipt)
}

pub(crate) fn payload_for(
    obligation: &WorkspaceCoverageObligation,
    plan: &WorkspaceCoverageProjectionPlan,
    point: &crate::WorkspaceCoveragePlannedPoint,
) -> WorkspaceCoveragePointPayload {
    WorkspaceCoveragePointPayload {
        obligation_id: obligation.obligation_id.clone(),
        projection_schema_version: plan.projection_schema_version.clone(),
        chunker_id: plan.chunker_id.clone(),
        body_path: point.body_path.clone(),
        source_raw_sha256: point.source_raw_sha256.clone(),
        source_raw_byte_length: point.source_raw_byte_length,
        chunk_index: point.chunk_index,
        byte_start: point.byte_start,
        byte_end: point.byte_end,
        chunk_sha256: point.chunk_sha256.clone(),
    }
}

pub(crate) fn exact_obligation_body_authority(
    opening: &[CultCacheEnvelope],
    obligation: &WorkspaceCoverageObligation,
) -> Result<Vec<CultCacheEnvelope>> {
    let head_env = find(opening, BODY_HEAD_TYPE, BODY_HEAD_KEY)
        .ok_or_else(|| anyhow!("current Body head missing"))?;
    let head: RepositoryBodyHead = decode(head_env)?;
    if head.workspace_id != obligation.workspace_id
        || head.generation != obligation.body_generation
        || head.observation_id != obligation.body_observation_id
        || head.manifest_root_sha256 != obligation.manifest_root_sha256
    {
        bail!("workspace coverage success requires its exact current Body authority");
    }
    [
        (BODY_BINDING_TYPE, BODY_BINDING_KEY),
        (BODY_HEAD_TYPE, BODY_HEAD_KEY),
        (
            BODY_OBSERVATION_TYPE,
            obligation.body_observation_id.as_str(),
        ),
        (BODY_MANIFEST_TYPE, obligation.manifest_root_sha256.as_str()),
    ]
    .into_iter()
    .map(|(ty, key)| {
        find(opening, ty, key)
            .cloned()
            .ok_or_else(|| anyhow!("Body authority envelope {ty}/{key} missing"))
    })
    .collect()
}

fn digest(value: &(impl Serialize + ?Sized)) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(rmp_serde::to_vec_named(value)?)
    ))
}

fn preserve_or_insert_immutable(
    opening: &[CultCacheEnvelope],
    expected: &mut Vec<CultCacheEnvelope>,
    replacements: &mut Vec<CultCacheEnvelope>,
    proposed: CultCacheEnvelope,
) -> Result<()> {
    if let Some(existing) = find(opening, &proposed.r#type, &proposed.key) {
        let mut comparable = proposed;
        comparable.stored_at = existing.stored_at.clone();
        if &comparable != existing {
            bail!("immutable workspace coverage document identity collision");
        }
        expected.push(existing.clone());
        replacements.push(existing.clone());
    } else {
        replacements.push(proposed);
    }
    Ok(())
}

fn chunk_descriptors(
    path: &str,
    raw_sha256: &str,
    bytes: &[u8],
) -> Result<Vec<WorkspaceCoverageChunkDescriptor>> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| anyhow!("named text chunker refuses non-UTF-8 bytes"))?;
    let mut line_starts = vec![0usize];
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' && index + 1 < bytes.len() {
            line_starts.push(index + 1);
        }
    }
    let mut result = Vec::new();
    let mut start_line = 0usize;
    let mut chunk_index = 0u32;
    while start_line < line_starts.len() {
        let end_line = (start_line + CHUNK_LINES).min(line_starts.len());
        let start = line_starts[start_line];
        let end = if end_line == line_starts.len() {
            bytes.len()
        } else {
            line_starts[end_line]
        };
        result.push(WorkspaceCoverageChunkDescriptor {
            body_path: path.into(),
            source_raw_sha256: raw_sha256.into(),
            source_raw_byte_length: bytes.len() as u64,
            chunk_index,
            byte_start: start as u64,
            byte_end: end as u64,
            chunk_sha256: format!("{:x}", Sha256::digest(&bytes[start..end])),
        });
        if end_line == line_starts.len() {
            break;
        }
        start_line = end_line - CHUNK_OVERLAP_LINES;
        chunk_index = chunk_index
            .checked_add(1)
            .ok_or_else(|| anyhow!("chunk index exhausted"))?;
    }
    Ok(result)
}

fn exact_body_authority(
    opening: &[CultCacheEnvelope],
    basis: &RepositoryBodyObservationBasis,
) -> Result<Vec<CultCacheEnvelope>> {
    let head_env = find(opening, BODY_HEAD_TYPE, BODY_HEAD_KEY)
        .ok_or_else(|| anyhow!("current Body head missing"))?;
    let head: RepositoryBodyHead = decode(head_env)?;
    if head.workspace_id != basis.workspace_id
        || head.generation != basis.generation
        || head.observation_id != basis.observation_id
        || head.manifest_root_sha256 != basis.manifest_root_sha256
    {
        bail!("workspace coverage acquisition requires the exact current Body head");
    }
    [
        (BODY_BINDING_TYPE, BODY_BINDING_KEY),
        (BODY_HEAD_TYPE, BODY_HEAD_KEY),
        (BODY_OBSERVATION_TYPE, basis.observation_id.as_str()),
        (BODY_MANIFEST_TYPE, basis.manifest_root_sha256.as_str()),
    ]
    .into_iter()
    .map(|(ty, key)| {
        find(opening, ty, key)
            .cloned()
            .ok_or_else(|| anyhow!("Body authority envelope {ty}/{key} missing"))
    })
    .collect()
}

fn find<'a>(
    entries: &'a [CultCacheEnvelope],
    ty: &str,
    key: &str,
) -> Option<&'a CultCacheEnvelope> {
    entries
        .iter()
        .find(|entry| entry.r#type == ty && entry.key == key)
}
fn decode<T: serde::de::DeserializeOwned>(value: &CultCacheEnvelope) -> Result<T> {
    Ok(rmp_serde::from_slice(&value.payload)?)
}
fn envelope<T: Serialize>(ty: &str, key: &str, value: &T) -> Result<CultCacheEnvelope> {
    Ok(CultCacheEnvelope {
        key: key.into(),
        r#type: ty.into(),
        payload: rmp_serde::to_vec(value)?,
        stored_at: chrono::Utc::now().to_rfc3339(),
        schema_id: Some(ty.into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[derive(Clone, Copy)]
    enum Hostility {
        Honest,
        IncompatibleCollection,
        WrongPayload,
        Extra,
        Missing,
        Duplicate,
        MissingVector,
        WrongVector,
        WrongVectorDimension,
        NonFiniteVector,
    }

    struct FakeProjectionPort {
        hostility: Hostility,
        points: Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>,
        upsert_calls: usize,
        collection_present: bool,
        collection_compatibility: Option<CollectionCompatibility>,
    }

    impl FakeProjectionPort {
        fn new(hostility: Hostility) -> Self {
            Self {
                hostility,
                points: Vec::new(),
                upsert_calls: 0,
                collection_present: true,
                collection_compatibility: None,
            }
        }
    }

    impl WorkspaceCoverageProjectionPort for FakeProjectionPort {
        fn ensure_exact_collection(
            &mut self,
            _: &str,
            compatibility: &CollectionCompatibility,
        ) -> Result<()> {
            if matches!(self.hostility, Hostility::IncompatibleCollection) {
                bail!("incompatible preexisting collection metadata");
            }
            self.collection_compatibility = Some(compatibility.clone());
            Ok(())
        }
        fn upsert_waited_batch(
            &mut self,
            _: &str,
            points: &[SemanticPoint<WorkspaceCoveragePointPayload>],
        ) -> Result<()> {
            self.upsert_calls += 1;
            for point in points {
                let stored = SemanticStoredPoint {
                    id: point.id.clone(),
                    payload: Some(point.payload.clone()),
                    vector: Some(point.vector.clone()),
                };
                if let Some(existing) = self.points.iter_mut().find(|item| item.id == point.id) {
                    *existing = stored;
                } else {
                    self.points.push(stored);
                }
            }
            Ok(())
        }
        fn observe_exact_ids(
            &mut self,
            _: &str,
            point_ids: &[String],
        ) -> Result<Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>> {
            point_ids
                .iter()
                .map(|id| {
                    self.points
                        .iter()
                        .find(|point| &point.id == id)
                        .cloned()
                        .ok_or_else(|| anyhow!("requested exact point is absent"))
                })
                .collect()
        }
        fn observe_all(
            &mut self,
            _: &str,
        ) -> Result<Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>> {
            let mut points = self.points.clone();
            match self.hostility {
                Hostility::Honest | Hostility::IncompatibleCollection => {}
                Hostility::WrongPayload => points[0].payload.as_mut().unwrap().chunk_index += 1,
                Hostility::Extra => points.push(SemanticStoredPoint {
                    id: uuid::Uuid::new_v4().to_string(),
                    payload: points[0].payload.clone(),
                    vector: points[0].vector.clone(),
                }),
                Hostility::Missing => {
                    points.pop();
                }
                Hostility::Duplicate => points.push(points[0].clone()),
                Hostility::MissingVector => points[0].vector = None,
                Hostility::WrongVector => points[0].vector.as_mut().unwrap()[0] += 0.5,
                Hostility::WrongVectorDimension => {
                    points[0].vector.as_mut().unwrap().pop();
                }
                Hostility::NonFiniteVector => {
                    points[0].vector.as_mut().unwrap()[0] = f32::NAN;
                }
            }
            Ok(points)
        }
    }

    impl WorkspaceCoverageEvidencePort for FakeProjectionPort {
        fn authenticate_exact_collection(
            &mut self,
            _: &str,
            expected: &CollectionCompatibility,
        ) -> Result<()> {
            if !self.collection_present || self.collection_compatibility.as_ref() != Some(expected)
            {
                bail!("live collection is absent or incompatible");
            }
            Ok(())
        }

        fn observe_all_for_evidence(
            &mut self,
            collection: &str,
        ) -> Result<Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>> {
            WorkspaceCoverageProjectionPort::observe_all(self, collection)
        }
    }

    #[test]
    fn sealed_batch_ranges_preserve_plan_order_and_never_exceed_qdrant_bound() {
        let ranges = sealed_batch_ranges(QDRANT_POINT_BATCH_MAX * 2 + 3);
        assert_eq!(
            ranges,
            vec![
                0..QDRANT_POINT_BATCH_MAX,
                QDRANT_POINT_BATCH_MAX..QDRANT_POINT_BATCH_MAX * 2,
                QDRANT_POINT_BATCH_MAX * 2..QDRANT_POINT_BATCH_MAX * 2 + 3,
            ]
        );
        assert!(
            ranges
                .iter()
                .all(|range| range.len() <= QDRANT_POINT_BATCH_MAX)
        );
        assert_eq!(
            ranges.into_iter().flatten().collect::<Vec<_>>(),
            (0..QDRANT_POINT_BATCH_MAX * 2 + 3).collect::<Vec<_>>()
        );
    }

    #[test]
    fn exact_batch_readback_rejects_vector_substitution() -> Result<()> {
        let (repo, _state, runtime, basis) = coverage_fixture()?;
        let (_, acquisition) = acquire_test(&runtime, &basis, "provider", "model", 3)?;
        let planned = acquisition
            .plan
            .planned_points
            .first()
            .ok_or_else(|| anyhow!("fixture produced no planned points"))?;
        let payload = payload_for(&acquisition.obligation, &acquisition.plan, planned);
        let expected_vector = vec![0.25_f32; 3];
        let point_bindings = vec![WorkspaceCoveragePointBinding {
            point_id: planned.point_id.clone(),
            payload_sha256: digest(&payload)?,
        }];
        let vector_bindings = vec![WorkspaceCoverageVectorBinding {
            point_id: planned.point_id.clone(),
            vector_sha256: digest(&expected_vector)?,
        }];
        let observed = vec![SemanticStoredPoint {
            id: planned.point_id.clone(),
            payload: Some(payload),
            vector: Some(vec![0.5_f32; 3]),
        }];
        assert!(
            authenticate_batch_readback(&observed, &point_bindings, &vector_bindings, 3).is_err()
        );
        drop(repo);
        Ok(())
    }

    #[test]
    fn exact_batch_readback_authenticates_qdrant_set_independent_of_response_order() -> Result<()> {
        let (repo, _state, runtime, basis) = coverage_fixture()?;
        let (_, acquisition) = acquire_test(&runtime, &basis, "provider", "model", 3)?;
        let planned = acquisition
            .plan
            .planned_points
            .first()
            .ok_or_else(|| anyhow!("fixture produced no planned points"))?;
        let payload_a = payload_for(&acquisition.obligation, &acquisition.plan, planned);
        let mut payload_b = payload_a.clone();
        payload_b.chunk_index += 1;
        let point_a = planned.point_id.clone();
        let point_b = uuid::Uuid::new_v4().to_string();
        let vector_a = vec![0.25_f32; 3];
        let vector_b = vec![0.5_f32; 3];
        let expected_points = vec![
            WorkspaceCoveragePointBinding {
                point_id: point_a.clone(),
                payload_sha256: digest(&payload_a)?,
            },
            WorkspaceCoveragePointBinding {
                point_id: point_b.clone(),
                payload_sha256: digest(&payload_b)?,
            },
        ];
        let expected_vectors = vec![
            WorkspaceCoverageVectorBinding {
                point_id: point_a.clone(),
                vector_sha256: digest(&vector_a)?,
            },
            WorkspaceCoverageVectorBinding {
                point_id: point_b.clone(),
                vector_sha256: digest(&vector_b)?,
            },
        ];
        let reversed = vec![
            SemanticStoredPoint {
                id: point_b,
                payload: Some(payload_b),
                vector: Some(vector_b),
            },
            SemanticStoredPoint {
                id: point_a,
                payload: Some(payload_a),
                vector: Some(vector_a),
            },
        ];

        authenticate_batch_readback(&reversed, &expected_points, &expected_vectors, 3)?;
        drop(repo);
        Ok(())
    }

    #[test]
    fn final_checkpoint_proof_rejects_mutated_or_missing_earlier_point() -> Result<()> {
        let (repo, _state, runtime, basis) = coverage_fixture()?;
        let (_, acquisition) = acquire_test(&runtime, &basis, "provider", "model", 3)?;
        let planned = acquisition
            .plan
            .planned_points
            .first()
            .ok_or_else(|| anyhow!("fixture produced no planned points"))?;
        let payload = payload_for(&acquisition.obligation, &acquisition.plan, planned);
        let vector = vec![0.25_f32; 3];
        let point_bindings = vec![WorkspaceCoveragePointBinding {
            point_id: planned.point_id.clone(),
            payload_sha256: digest(&payload)?,
        }];
        let vector_bindings = vec![WorkspaceCoverageVectorBinding {
            point_id: planned.point_id.clone(),
            vector_sha256: digest(&vector)?,
        }];
        let observed = vec![SemanticStoredPoint {
            id: planned.point_id.clone(),
            payload: Some(payload),
            vector: Some(vector),
        }];
        authenticate_final_observation_against_checkpoint_bindings(
            observed.clone(),
            &point_bindings,
            &vector_bindings,
        )?;
        let mut mutated = observed.clone();
        mutated[0].vector.as_mut().unwrap()[0] += 1.0;
        assert!(
            authenticate_final_observation_against_checkpoint_bindings(
                mutated,
                &point_bindings,
                &vector_bindings,
            )
            .is_err()
        );
        assert!(
            authenticate_final_observation_against_checkpoint_bindings(
                Vec::new(),
                &point_bindings,
                &vector_bindings,
            )
            .is_err()
        );
        let mut duplicated = observed.clone();
        duplicated.push(observed[0].clone());
        assert!(
            authenticate_final_observation_against_checkpoint_bindings(
                duplicated,
                &point_bindings,
                &vector_bindings,
            )
            .is_err()
        );
        let mut substituted_duplicate = observed[0].clone();
        substituted_duplicate.vector.as_mut().unwrap()[0] += 1.0;
        let mut duplicated = observed.clone();
        duplicated.push(substituted_duplicate);
        assert!(
            authenticate_final_observation_against_checkpoint_bindings(
                duplicated,
                &point_bindings,
                &vector_bindings,
            )
            .is_err()
        );
        drop(repo);
        Ok(())
    }

    struct AdvancingEvidencePort {
        inner: FakeProjectionPort,
        body_store: PathBuf,
    }

    impl WorkspaceCoverageEvidencePort for AdvancingEvidencePort {
        fn authenticate_exact_collection(
            &mut self,
            collection: &str,
            expected: &CollectionCompatibility,
        ) -> Result<()> {
            self.inner
                .authenticate_exact_collection(collection, expected)
        }

        fn observe_all_for_evidence(
            &mut self,
            collection: &str,
        ) -> Result<Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>> {
            let points = self.inner.observe_all_for_evidence(collection)?;
            let backing = SingleFileMessagePackBackingStore::new(&self.body_store);
            let opening = backing.pull_all()?;
            let current = find(&opening, CLAIM_TYPE, CLAIM_KEY)
                .ok_or_else(|| anyhow!("test current claim disappeared"))?
                .clone();
            let mut advanced: WorkspaceCoverageProjectionClaim = decode(&current)?;
            advanced.executor_id.push_str("-advanced");
            if !backing.compare_and_swap_batch(
                &[current],
                vec![envelope(CLAIM_TYPE, CLAIM_KEY, &advanced)?],
            )? {
                bail!("test authority advance lost its CAS");
            }
            Ok(points)
        }
    }

    #[derive(Default)]
    struct FakeRetirementPort {
        present: std::collections::BTreeMap<String, CollectionCompatibility>,
        retired: Vec<String>,
    }

    impl WorkspaceCoverageRetirementPort for FakeRetirementPort {
        fn retire_exact(
            &mut self,
            collection: &str,
            compatibility: &CollectionCompatibility,
        ) -> Result<()> {
            let Some(actual) = self.present.get(collection) else {
                return Ok(());
            };
            if actual != compatibility {
                bail!("incompatible same-name collection");
            }
            self.present.remove(collection);
            self.retired.push(collection.into());
            Ok(())
        }
    }

    fn coverage_fixture() -> Result<(
        tempfile::TempDir,
        tempfile::TempDir,
        PathBuf,
        RepositoryBodyObservationBasis,
    )> {
        let repo = tempfile::tempdir()?;
        let state = tempfile::tempdir()?;
        git(repo.path(), &["init"])?;
        std::fs::write(repo.path().join("source.rs"), "fn one() {}\n")?;
        git(repo.path(), &["add", "."])?;
        let runtime = state.path().join("runtime.cc");
        let agents = state.path().join("agents.cc");
        let body = state.path().join("body.cc");
        crate::initialize_runtime_spine(
            &runtime,
            crate::RuntimeSpineInitOptions {
                runtime_id: "retirement-runtime".into(),
                display_name: "retirement".into(),
                created_at: "2026-07-16T00:00:00Z".into(),
            },
        )?;
        crate::ensure_agent_memory_swarm_identity(&agents, "retirement-swarm")?;
        crate::bind_runtime_to_agent_memory_swarm(&runtime, &agents, "2026-07-16T00:00:01Z")?;
        crate::bind_repository_body(repo.path(), &body, &runtime, "retirement-workspace")?;
        let basis = crate::observe_runtime_repository_body_basis(&runtime)?;
        Ok((repo, state, runtime, basis))
    }

    #[test]
    fn retirement_collects_failed_retry_but_preserves_running_attempt() -> Result<()> {
        let (_repo, _state, runtime, basis) = coverage_fixture()?;
        let (_, failed) = acquire_test(&runtime, &basis, "provider", "model", 3)?;
        let failed_name = workspace_coverage_execution_collection(
            &failed.plan.plan_id,
            &failed.claim.claim_id,
            failed.claim.claim_epoch,
        )?;
        fail_workspace_coverage_projection(&failed, "backend refused")?;
        let (_, running) = acquire_test(&runtime, &basis, "provider", "model", 3)?;
        let candidates = workspace_coverage_retirement_candidates(&running.body_store)?;
        assert_eq!(
            candidates
                .iter()
                .map(|c| c.collection_name.as_str())
                .collect::<Vec<_>>(),
            vec![failed_name]
        );
        assert!(!candidates.iter().any(|c| {
            c.collection_name
                == workspace_coverage_execution_collection(
                    &running.plan.plan_id,
                    &running.claim.claim_id,
                    running.claim.claim_epoch,
                )
                .unwrap()
        }));
        Ok(())
    }

    #[test]
    fn prior_success_retires_only_after_new_head_commits() -> Result<()> {
        let (repo, _state, runtime, basis_a) = coverage_fixture()?;
        let (_, acquired_a) = acquire_test(&runtime, &basis_a, "provider", "model", 3)?;
        let receipt_a = execute_preembedded_workspace_coverage_projection(
            &acquired_a,
            projection_inputs(&acquired_a, repo.path())?,
            &mut FakeProjectionPort::new(Hostility::Honest),
        )?;
        assert!(workspace_coverage_retirement_candidates(&acquired_a.body_store)?.is_empty());

        std::fs::write(repo.path().join("source.rs"), "fn two() {}\n")?;
        let basis_b = crate::observe_runtime_repository_body_basis(&runtime)?;
        let (_, acquired_b) = acquire_test(&runtime, &basis_b, "provider", "model", 3)?;
        assert!(workspace_coverage_retirement_candidates(&acquired_b.body_store)?.is_empty());
        execute_preembedded_workspace_coverage_projection(
            &acquired_b,
            projection_inputs(&acquired_b, repo.path())?,
            &mut FakeProjectionPort::new(Hostility::Honest),
        )?;
        let candidates = workspace_coverage_retirement_candidates(&acquired_b.body_store)?;
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].collection_name, receipt_a.collection_name);
        Ok(())
    }

    #[test]
    fn retirement_refuses_incompatible_collection_and_absence_is_idempotent() -> Result<()> {
        let (_repo, _state, runtime, basis) = coverage_fixture()?;
        let (_, acquired) = acquire_test(&runtime, &basis, "provider", "model", 3)?;
        fail_workspace_coverage_projection(&acquired, "failed")?;
        let candidate = workspace_coverage_retirement_candidates(&acquired.body_store)?.remove(0);
        let mut port = FakeRetirementPort::default();
        let mut incompatible = candidate.compatibility.clone();
        incompatible.embedding_model.push_str("-alien");
        port.present
            .insert(candidate.collection_name.clone(), incompatible);
        assert!(retire_workspace_coverage_collections(&acquired.body_store, &mut port).is_err());
        port.present.clear();
        assert_eq!(
            retire_workspace_coverage_collections(&acquired.body_store, &mut port)?,
            1
        );
        assert_eq!(
            retire_workspace_coverage_collections(&acquired.body_store, &mut port)?,
            1
        );
        assert!(port.retired.is_empty());
        Ok(())
    }

    fn projection_inputs(
        acquisition: &WorkspaceCoverageAcquisition,
        repo: &Path,
    ) -> Result<Vec<WorkspaceCoverageProjectionInput>> {
        acquisition
            .plan
            .planned_points
            .iter()
            .map(|point| {
                let bytes = std::fs::read(repo.join(&point.body_path))?;
                let text = std::str::from_utf8(
                    &bytes[point.byte_start as usize..point.byte_end as usize],
                )?
                .to_string();
                Ok(WorkspaceCoverageProjectionInput {
                    point_id: point.point_id.clone(),
                    text,
                    vector: vec![0.25; acquisition.plan.vector_dimensions as usize],
                })
            })
            .collect()
    }

    fn acquire_test(
        runtime: &Path,
        basis: &RepositoryBodyObservationBasis,
        provider: &str,
        model: &str,
        dimensions: u32,
    ) -> Result<(
        PreparedWorkspaceCoverageProjection,
        WorkspaceCoverageAcquisition,
    )> {
        let body = RepositoryBodyReadSession::open(runtime, basis)?;
        let prepared = prepare_workspace_coverage_projection(&body, provider, model, dimensions)?;
        let acquisition = match acquire_workspace_coverage_projection(
            &prepared,
            "test-executor",
            "test-incarnation",
            "test-startup-receipt",
        )? {
            WorkspaceCoverageAcquireResult::Acquired(acquisition) => acquisition,
            WorkspaceCoverageAcquireResult::Current(_) => {
                bail!("test unexpectedly found a current projection")
            }
            WorkspaceCoverageAcquireResult::Contended => {
                bail!("test unexpectedly found a contended projection")
            }
        };
        Ok((prepared, acquisition))
    }

    #[test]
    fn live_coverage_evidence_requires_exact_rescroll_and_authority_chain() -> Result<()> {
        let (repo, _state, runtime, basis) = coverage_fixture()?;
        let (_, acquisition) = acquire_test(&runtime, &basis, "provider", "model", 3)?;
        let inputs = projection_inputs(&acquisition, repo.path())?;
        let expected_claim_id = acquisition.claim.claim_id.clone();
        let mut projection = FakeProjectionPort::new(Hostility::Honest);
        let receipt = execute_preembedded_workspace_coverage_projection(
            &acquisition,
            inputs,
            &mut projection,
        )?;
        let policy = WorkspaceCoveragePolicy::bounded_regular_files_v0(
            WORKSPACE_COVERAGE_MAXIMUM_FILE_BYTES,
        )?;

        let evidence = observe_current_workspace_coverage_evidence(
            &runtime,
            &basis,
            &policy,
            "provider",
            "model",
            3,
            &mut projection,
        )?
        .expect("live exact coverage evidence");
        assert_eq!(evidence.claim_id, expected_claim_id);
        assert_eq!(evidence.receipt_id, receipt.receipt_id);
        assert_eq!(
            evidence.observed_vector_binding_set_sha256,
            receipt.observed_vector_binding_set_sha256
        );
        assert!(evidence.head_envelope_digest.starts_with("sha256-"));

        let wrong_policy = WorkspaceCoveragePolicy::bounded_regular_files_v0(
            WORKSPACE_COVERAGE_MAXIMUM_FILE_BYTES - 1,
        )?;
        assert!(
            observe_current_workspace_coverage_evidence(
                &runtime,
                &basis,
                &wrong_policy,
                "provider",
                "model",
                3,
                &mut projection,
            )?
            .is_none()
        );
        Ok(())
    }

    #[test]
    fn stored_current_coverage_cannot_substitute_for_live_qdrant_evidence() -> Result<()> {
        let (repo, _state, runtime, basis) = coverage_fixture()?;
        let (_, acquisition) = acquire_test(&runtime, &basis, "provider", "model", 3)?;
        let inputs = projection_inputs(&acquisition, repo.path())?;
        let mut projection = FakeProjectionPort::new(Hostility::Honest);
        execute_preembedded_workspace_coverage_projection(&acquisition, inputs, &mut projection)?;
        let terminal = authenticate_current_workspace_coverage_terminal_authority(&runtime)?
            .expect("succeeded current receipt/head must own terminal health");
        assert_eq!(terminal.claim_id, acquisition.claim.claim_id);
        assert_eq!(terminal.plan_id, acquisition.plan.plan_id);
        assert!(matches!(
            classify_current_workspace_coverage(&runtime, &basis, "provider", "model", 3)?,
            WorkspaceCoverageCurrentState::Current(_)
        ));
        let policy = WorkspaceCoveragePolicy::bounded_regular_files_v0(
            WORKSPACE_COVERAGE_MAXIMUM_FILE_BYTES,
        )?;

        let mut missing_live_collection = FakeProjectionPort::new(Hostility::Honest);
        missing_live_collection.collection_present = false;
        assert!(
            observe_current_workspace_coverage_evidence(
                &runtime,
                &basis,
                &policy,
                "provider",
                "model",
                3,
                &mut missing_live_collection,
            )
            .is_err()
        );

        let mut wrong_compatibility = FakeProjectionPort {
            hostility: Hostility::Honest,
            points: projection.points.clone(),
            upsert_calls: 0,
            collection_present: true,
            collection_compatibility: projection.collection_compatibility.clone(),
        };
        wrong_compatibility
            .collection_compatibility
            .as_mut()
            .expect("test collection compatibility")
            .vector_size += 1;
        assert!(
            observe_current_workspace_coverage_evidence(
                &runtime,
                &basis,
                &policy,
                "provider",
                "model",
                3,
                &mut wrong_compatibility,
            )
            .is_err()
        );

        for hostility in [Hostility::WrongPayload, Hostility::WrongVector] {
            let mut hostile = FakeProjectionPort {
                hostility,
                points: projection.points.clone(),
                upsert_calls: 0,
                collection_present: true,
                collection_compatibility: projection.collection_compatibility.clone(),
            };
            assert!(
                observe_current_workspace_coverage_evidence(
                    &runtime,
                    &basis,
                    &policy,
                    "provider",
                    "model",
                    3,
                    &mut hostile,
                )
                .is_err()
            );
        }
        Ok(())
    }

    #[test]
    fn terminal_health_refuses_a_receipt_for_an_older_body() -> Result<()> {
        let (repo, _state, runtime, basis) = coverage_fixture()?;
        let (_, acquisition) = acquire_test(&runtime, &basis, "provider", "model", 3)?;
        let inputs = projection_inputs(&acquisition, repo.path())?;
        execute_preembedded_workspace_coverage_projection(
            &acquisition,
            inputs,
            &mut FakeProjectionPort::new(Hostility::Honest),
        )?;
        assert!(authenticate_current_workspace_coverage_terminal_authority(&runtime)?.is_some());
        std::fs::write(repo.path().join("source.rs"), "pub fn changed() {}\n")?;
        crate::observe_runtime_repository_body_basis(&runtime)?;
        assert!(authenticate_current_workspace_coverage_terminal_authority(&runtime)?.is_none());
        Ok(())
    }

    #[test]
    fn live_evidence_refuses_authority_advanced_during_qdrant_scroll() -> Result<()> {
        let (repo, _state, runtime, basis) = coverage_fixture()?;
        let (_, acquisition) = acquire_test(&runtime, &basis, "provider", "model", 3)?;
        let inputs = projection_inputs(&acquisition, repo.path())?;
        let mut projection = FakeProjectionPort::new(Hostility::Honest);
        execute_preembedded_workspace_coverage_projection(&acquisition, inputs, &mut projection)?;
        let body_store = PathBuf::from(
            runtime_repository_body_store_binding(&runtime)?
                .expect("runtime Body route")
                .body_store_path,
        );
        let mut advancing = AdvancingEvidencePort {
            inner: projection,
            body_store,
        };
        let policy = WorkspaceCoveragePolicy::bounded_regular_files_v0(
            WORKSPACE_COVERAGE_MAXIMUM_FILE_BYTES,
        )?;
        assert!(
            observe_current_workspace_coverage_evidence(
                &runtime,
                &basis,
                &policy,
                "provider",
                "model",
                3,
                &mut advancing,
            )?
            .is_none(),
            "authority advanced during live scroll must request retry, not emit evidence"
        );
        Ok(())
    }

    #[test]
    fn named_chunker_preserves_utf8_boundaries_and_eight_line_overlap() -> Result<()> {
        let text = (0..105)
            .map(|index| format!("line {index} λ\n"))
            .collect::<String>();
        let chunks = chunk_descriptors("a", &"11".repeat(32), text.as_bytes())?;
        assert_eq!(chunks.len(), 2);
        let overlap = &text.as_bytes()[chunks[1].byte_start as usize..chunks[0].byte_end as usize];
        assert_eq!(overlap.iter().filter(|byte| **byte == b'\n').count(), 8);
        assert!(
            std::str::from_utf8(
                &text.as_bytes()[chunks[0].byte_start as usize..chunks[0].byte_end as usize]
            )
            .is_ok()
        );
        assert!(
            std::str::from_utf8(
                &text.as_bytes()[chunks[1].byte_start as usize..chunks[1].byte_end as usize]
            )
            .is_ok()
        );
        assert_eq!(chunks[0].chunk_index, 0);
        assert_eq!(chunks[1].chunk_index, 1);
        assert!(chunk_descriptors("binary", &"22".repeat(32), &[0xff, 0xfe]).is_err());
        Ok(())
    }

    fn git(repo: &Path, args: &[&str]) -> Result<()> {
        let output = Command::new("git")
            .args(["-C", repo.to_str().unwrap()])
            .args(args)
            .output()?;
        if !output.status.success() {
            bail!("git failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    #[test]
    fn preparation_authenticates_body_store_constant_times_and_current_path_reads_no_files()
    -> Result<()> {
        let repo = tempfile::tempdir()?;
        let state = tempfile::tempdir()?;
        git(repo.path(), &["init"])?;
        for index in 0..12 {
            std::fs::write(
                repo.path().join(format!("source-{index}.rs")),
                format!("fn f{index}() {{}}\n"),
            )?;
        }
        git(repo.path(), &["add", "."])?;
        let runtime = state.path().join("runtime.cc");
        let agents = state.path().join("agents.cc");
        let body_store = state.path().join("body.cc");
        crate::initialize_runtime_spine(
            &runtime,
            crate::RuntimeSpineInitOptions {
                runtime_id: "read-session-runtime".into(),
                display_name: "read session".into(),
                created_at: "2026-07-16T00:00:00Z".into(),
            },
        )?;
        crate::ensure_agent_memory_swarm_identity(&agents, "read-session-swarm")?;
        crate::bind_runtime_to_agent_memory_swarm(&runtime, &agents, "2026-07-16T00:00:01Z")?;
        crate::bind_repository_body(repo.path(), &body_store, &runtime, "read-session-workspace")?;
        let basis = crate::observe_runtime_repository_body_basis(&runtime)?;

        crate::repository_body_observer::reset_repository_body_read_counters();
        let body = RepositoryBodyReadSession::open(&runtime, &basis)?;
        let prepared = prepare_workspace_coverage_projection(&body, "provider", "model", 3)?;
        let (store_loads, file_reads) =
            crate::repository_body_observer::repository_body_read_counters();
        assert_eq!(store_loads, 2, "Body store validation count changed");
        assert_eq!(file_reads, 12);

        let acquisition = match acquire_workspace_coverage_projection(
            &prepared,
            "test-executor",
            "test-incarnation",
            "test-startup-receipt",
        )? {
            WorkspaceCoverageAcquireResult::Acquired(value) => value,
            _ => bail!("initial projection was not acquired"),
        };
        let inputs = projection_inputs(&acquisition, repo.path())?;
        let receipt = execute_preembedded_workspace_coverage_projection(
            &acquisition,
            inputs,
            &mut FakeProjectionPort::new(Hostility::Honest),
        )?;
        crate::repository_body_observer::reset_repository_body_read_counters();
        match classify_current_workspace_coverage(&runtime, &basis, "provider", "model", 3)? {
            WorkspaceCoverageCurrentState::Current(current) => assert_eq!(current, receipt),
            WorkspaceCoverageCurrentState::NeedsPreparation => {
                bail!("exact projection was not current")
            }
        }
        assert_eq!(
            crate::repository_body_observer::repository_body_read_counters().1,
            0,
            "current classification rematerialized Repository Body files"
        );
        Ok(())
    }

    #[test]
    fn acquisition_is_exclusive_and_failure_can_terminalize_after_body_advance() -> Result<()> {
        let repo = tempfile::tempdir()?;
        let state = tempfile::tempdir()?;
        git(repo.path(), &["init"])?;
        std::fs::write(repo.path().join("source.rs"), "fn one() {}")?;
        std::fs::write(repo.path().join("binary.dat"), [0xff, 0xfe, 0xfd])?;
        git(repo.path(), &["add", "."])?;
        let runtime = state.path().join("runtime.cc");
        let agents = state.path().join("agents.cc");
        let body = state.path().join("body.cc");
        crate::initialize_runtime_spine(
            &runtime,
            crate::RuntimeSpineInitOptions {
                runtime_id: "projector-runtime".into(),
                display_name: "projector test".into(),
                created_at: "2026-07-15T00:00:00Z".into(),
            },
        )?;
        crate::ensure_agent_memory_swarm_identity(&agents, "projector-swarm")?;
        crate::bind_runtime_to_agent_memory_swarm(&runtime, &agents, "2026-07-15T00:00:01Z")?;
        crate::bind_repository_body(repo.path(), &body, &runtime, "projector-workspace")?;
        let basis = crate::observe_runtime_repository_body_basis(&runtime)?;
        let (_, first) = acquire_test(&runtime, &basis, "provider", "model", 3)?;
        assert!(first.obligation.classifications.iter().any(|entry| {
            entry.path == "binary.dat"
                && entry.disposition
                    == crate::WorkspaceCoverageDisposition::ExcludeNonUtf8RegularFile
        }));
        assert_eq!(first.obligation.included_entry_count, 1);
        assert!(acquire_test(&runtime, &basis, "provider", "model", 3).is_err());
        fail_workspace_coverage_projection(&first, "expected test failure")?;
        let (_, second) = acquire_test(&runtime, &basis, "provider", "model", 3)?;
        assert_eq!(second.claim.claim_epoch, first.claim.claim_epoch + 1);
        std::fs::write(repo.path().join("source.rs"), "fn two() {}")?;
        let advanced = crate::observe_runtime_repository_body_basis(&runtime)?;
        assert!(advanced.generation > basis.generation);
        fail_workspace_coverage_projection(&second, "Body advanced during projection")?;
        let entries = SingleFileMessagePackBackingStore::new(&second.body_store).pull_all()?;
        assert!(
            find(
                &entries,
                "gamecult.epiphany.workspace_coverage_receipt",
                "current"
            )
            .is_none()
        );
        assert!(
            find(
                &entries,
                "gamecult.epiphany.workspace_coverage_head",
                "current"
            )
            .is_none()
        );
        Ok(())
    }

    #[test]
    fn exact_observation_commits_receipt_and_hostile_point_sets_terminalize() -> Result<()> {
        for hostility in [
            Hostility::Honest,
            Hostility::IncompatibleCollection,
            Hostility::WrongPayload,
            Hostility::Extra,
            Hostility::Missing,
            Hostility::Duplicate,
            Hostility::MissingVector,
            Hostility::WrongVector,
            Hostility::WrongVectorDimension,
            Hostility::NonFiniteVector,
        ] {
            let repo = tempfile::tempdir()?;
            let state = tempfile::tempdir()?;
            git(repo.path(), &["init"])?;
            std::fs::write(repo.path().join("source.rs"), "fn one() {}\n")?;
            git(repo.path(), &["add", "."])?;
            let runtime = state.path().join("runtime.cc");
            let agents = state.path().join("agents.cc");
            let body = state.path().join("body.cc");
            crate::initialize_runtime_spine(
                &runtime,
                crate::RuntimeSpineInitOptions {
                    runtime_id: "projection-execution-runtime".into(),
                    display_name: "projection execution".into(),
                    created_at: "2026-07-15T00:00:00Z".into(),
                },
            )?;
            crate::ensure_agent_memory_swarm_identity(&agents, "projection-execution-swarm")?;
            crate::bind_runtime_to_agent_memory_swarm(&runtime, &agents, "2026-07-15T00:00:01Z")?;
            crate::bind_repository_body(
                repo.path(),
                &body,
                &runtime,
                "projection-execution-workspace",
            )?;
            let basis = crate::observe_runtime_repository_body_basis(&runtime)?;
            let (_, acquisition) = acquire_test(&runtime, &basis, "provider", "model", 3)?;
            let inputs = projection_inputs(&acquisition, repo.path())?;
            let result = execute_preembedded_workspace_coverage_projection(
                &acquisition,
                inputs,
                &mut FakeProjectionPort::new(hostility),
            );
            let entries =
                SingleFileMessagePackBackingStore::new(&acquisition.body_store).pull_all()?;
            match hostility {
                Hostility::Honest => {
                    let receipt = result?;
                    assert!(find(&entries, RECEIPT_TYPE, &receipt.receipt_id).is_some());
                    assert!(find(&entries, HEAD_TYPE, HEAD_KEY).is_some());
                    let body = RepositoryBodyReadSession::open(&runtime, &basis)?;
                    let prepared =
                        prepare_workspace_coverage_projection(&body, "provider", "model", 3)?;
                    match acquire_workspace_coverage_projection(
                        &prepared,
                        "replacement-executor",
                        "replacement-incarnation",
                        "replacement-startup-receipt",
                    )? {
                        WorkspaceCoverageAcquireResult::Current(current) => {
                            assert_eq!(current, receipt)
                        }
                        WorkspaceCoverageAcquireResult::Acquired(_) => {
                            bail!("exact current projection was claimed again")
                        }
                        WorkspaceCoverageAcquireResult::Contended => {
                            bail!("exact current projection was contended")
                        }
                    }
                    let backing = SingleFileMessagePackBackingStore::new(&acquisition.body_store);
                    let before_corruption = backing.pull_all()?;
                    let receipt_env = find(&before_corruption, RECEIPT_TYPE, &receipt.receipt_id)
                        .unwrap()
                        .clone();
                    let mut substituted = receipt.clone();
                    substituted.observed_point_count += 1;
                    assert!(backing.compare_and_swap_batch(
                        std::slice::from_ref(&receipt_env),
                        vec![envelope(RECEIPT_TYPE, &receipt.receipt_id, &substituted)?],
                    )?);
                    assert!(
                        acquire_workspace_coverage_projection(
                            &prepared,
                            "replacement-executor",
                            "replacement-incarnation",
                            "replacement-startup-receipt",
                        )
                        .is_err()
                    );
                }
                _ => {
                    assert!(result.is_err());
                    assert!(find(&entries, RECEIPT_TYPE, HEAD_KEY).is_none());
                    let claim: WorkspaceCoverageProjectionClaim =
                        decode(find(&entries, CLAIM_TYPE, CLAIM_KEY).unwrap())?;
                    assert_eq!(claim.status, "failed");
                }
            }
        }
        Ok(())
    }

    #[test]
    fn body_advance_prevents_receipt_and_terminalizes_attempt() -> Result<()> {
        let repo = tempfile::tempdir()?;
        let state = tempfile::tempdir()?;
        git(repo.path(), &["init"])?;
        std::fs::write(repo.path().join("source.rs"), "fn one() {}\n")?;
        git(repo.path(), &["add", "."])?;
        let runtime = state.path().join("runtime.cc");
        let agents = state.path().join("agents.cc");
        let body = state.path().join("body.cc");
        crate::initialize_runtime_spine(
            &runtime,
            crate::RuntimeSpineInitOptions {
                runtime_id: "body-advance-runtime".into(),
                display_name: "body advance".into(),
                created_at: "2026-07-15T00:00:00Z".into(),
            },
        )?;
        crate::ensure_agent_memory_swarm_identity(&agents, "body-advance-swarm")?;
        crate::bind_runtime_to_agent_memory_swarm(&runtime, &agents, "2026-07-15T00:00:01Z")?;
        crate::bind_repository_body(repo.path(), &body, &runtime, "body-advance-workspace")?;
        let basis = crate::observe_runtime_repository_body_basis(&runtime)?;
        let (_, acquisition) = acquire_test(&runtime, &basis, "provider", "model", 3)?;
        let inputs = projection_inputs(&acquisition, repo.path())?;
        std::fs::write(repo.path().join("source.rs"), "fn two() {}\n")?;
        crate::observe_runtime_repository_body_basis(&runtime)?;
        assert!(
            execute_preembedded_workspace_coverage_projection(
                &acquisition,
                inputs,
                &mut FakeProjectionPort::new(Hostility::Honest)
            )
            .is_err()
        );
        let entries = SingleFileMessagePackBackingStore::new(&acquisition.body_store).pull_all()?;
        assert!(find(&entries, HEAD_TYPE, HEAD_KEY).is_none());
        let claim: WorkspaceCoverageProjectionClaim =
            decode(find(&entries, CLAIM_TYPE, CLAIM_KEY).unwrap())?;
        assert_eq!(claim.status, "failed");
        Ok(())
    }

    #[test]
    fn terminal_cas_loser_and_forged_observation_cannot_mint_receipts() -> Result<()> {
        let repo = tempfile::tempdir()?;
        let state = tempfile::tempdir()?;
        git(repo.path(), &["init"])?;
        std::fs::write(repo.path().join("source.rs"), "fn one() {}\n")?;
        git(repo.path(), &["add", "."])?;
        let runtime = state.path().join("runtime.cc");
        let agents = state.path().join("agents.cc");
        let body = state.path().join("body.cc");
        crate::initialize_runtime_spine(
            &runtime,
            crate::RuntimeSpineInitOptions {
                runtime_id: "cas-loser-runtime".into(),
                display_name: "cas loser".into(),
                created_at: "2026-07-15T00:00:00Z".into(),
            },
        )?;
        crate::ensure_agent_memory_swarm_identity(&agents, "cas-loser-swarm")?;
        crate::bind_runtime_to_agent_memory_swarm(&runtime, &agents, "2026-07-15T00:00:01Z")?;
        crate::bind_repository_body(repo.path(), &body, &runtime, "cas-loser-workspace")?;
        let basis = crate::observe_runtime_repository_body_basis(&runtime)?;
        let (_, acquisition) = acquire_test(&runtime, &basis, "provider", "model", 3)?;
        let collection_name = workspace_coverage_execution_collection(
            &acquisition.plan.plan_id,
            &acquisition.claim.claim_id,
            acquisition.claim.claim_epoch,
        )?;
        let forged = WorkspaceCoverageObservedBinding {
            collection_name: collection_name.clone(),
            point_count: acquisition.plan.expected_point_count,
            point_set_sha256: acquisition.plan.expected_point_set_sha256.clone(),
            point_binding_set_sha256: "ff".repeat(32),
            vector_binding_set_sha256: "ee".repeat(32),
        };
        assert!(commit_workspace_coverage_success(&acquisition, forged).is_err());
        fail_workspace_coverage_projection(&acquisition, "another terminal writer won")?;
        let exact_but_late = WorkspaceCoverageObservedBinding {
            collection_name,
            point_count: acquisition.plan.expected_point_count,
            point_set_sha256: acquisition.plan.expected_point_set_sha256.clone(),
            point_binding_set_sha256: acquisition.plan.point_binding_set_sha256.clone(),
            vector_binding_set_sha256: "ee".repeat(32),
        };
        assert!(commit_workspace_coverage_success(&acquisition, exact_but_late).is_err());
        let entries = SingleFileMessagePackBackingStore::new(&acquisition.body_store).pull_all()?;
        assert!(find(&entries, HEAD_TYPE, HEAD_KEY).is_none());
        assert!(!entries.iter().any(|entry| entry.r#type == RECEIPT_TYPE));
        Ok(())
    }

    #[test]
    fn empty_plan_observes_empty_collection_without_empty_upsert() -> Result<()> {
        let repo = tempfile::tempdir()?;
        let state = tempfile::tempdir()?;
        git(repo.path(), &["init"])?;
        std::fs::write(repo.path().join("empty.txt"), "")?;
        git(repo.path(), &["add", "."])?;
        let runtime = state.path().join("runtime.cc");
        let agents = state.path().join("agents.cc");
        let body = state.path().join("body.cc");
        crate::initialize_runtime_spine(
            &runtime,
            crate::RuntimeSpineInitOptions {
                runtime_id: "empty-runtime".into(),
                display_name: "empty".into(),
                created_at: "2026-07-15T00:00:00Z".into(),
            },
        )?;
        crate::ensure_agent_memory_swarm_identity(&agents, "empty-swarm")?;
        crate::bind_runtime_to_agent_memory_swarm(&runtime, &agents, "2026-07-15T00:00:01Z")?;
        crate::bind_repository_body(repo.path(), &body, &runtime, "empty-workspace")?;
        let basis = crate::observe_runtime_repository_body_basis(&runtime)?;
        let (_, acquisition) = acquire_test(&runtime, &basis, "provider", "model", 3)?;
        assert!(acquisition.plan.planned_points.is_empty());
        let mut port = FakeProjectionPort::new(Hostility::Honest);
        let receipt =
            execute_preembedded_workspace_coverage_projection(&acquisition, Vec::new(), &mut port)?;
        assert_eq!(port.upsert_calls, 0);
        assert_eq!(receipt.observed_point_count, 0);
        Ok(())
    }

    #[test]
    fn successive_generations_replace_only_the_acquired_prior_head() -> Result<()> {
        let repo = tempfile::tempdir()?;
        let state = tempfile::tempdir()?;
        git(repo.path(), &["init"])?;
        std::fs::write(repo.path().join("source.rs"), "fn a() {}\n")?;
        git(repo.path(), &["add", "."])?;
        let runtime = state.path().join("runtime.cc");
        let agents = state.path().join("agents.cc");
        let body = state.path().join("body.cc");
        crate::initialize_runtime_spine(
            &runtime,
            crate::RuntimeSpineInitOptions {
                runtime_id: "head-cas-runtime".into(),
                display_name: "head cas".into(),
                created_at: "2026-07-15T00:00:00Z".into(),
            },
        )?;
        crate::ensure_agent_memory_swarm_identity(&agents, "head-cas-swarm")?;
        crate::bind_runtime_to_agent_memory_swarm(&runtime, &agents, "2026-07-15T00:00:01Z")?;
        crate::bind_repository_body(repo.path(), &body, &runtime, "head-cas-workspace")?;

        let basis_a = crate::observe_runtime_repository_body_basis(&runtime)?;
        let (_, acquisition_a) = acquire_test(&runtime, &basis_a, "provider", "model", 3)?;
        let inputs_a = projection_inputs(&acquisition_a, repo.path())?;
        execute_preembedded_workspace_coverage_projection(
            &acquisition_a,
            inputs_a,
            &mut FakeProjectionPort::new(Hostility::Honest),
        )?;
        let backing = SingleFileMessagePackBackingStore::new(&acquisition_a.body_store);
        let after_a = backing.pull_all()?;
        let head_a = find(&after_a, HEAD_TYPE, HEAD_KEY).unwrap().clone();

        std::fs::write(repo.path().join("source.rs"), "fn b() {}\n")?;
        let basis_b = crate::observe_runtime_repository_body_basis(&runtime)?;
        let (_, acquisition_b) = acquire_test(&runtime, &basis_b, "provider", "model", 3)?;
        assert_eq!(acquisition_b.prior_head.as_ref(), Some(&head_a));
        let inputs_b = projection_inputs(&acquisition_b, repo.path())?;
        execute_preembedded_workspace_coverage_projection(
            &acquisition_b,
            inputs_b,
            &mut FakeProjectionPort::new(Hostility::Honest),
        )?;
        let after_b = backing.pull_all()?;
        let head_b = find(&after_b, HEAD_TYPE, HEAD_KEY).unwrap().clone();
        let decoded_b: WorkspaceCoverageHead = decode(&head_b)?;
        assert_eq!(decoded_b.body_generation, basis_b.generation);

        std::fs::write(repo.path().join("source.rs"), "fn c() {}\n")?;
        let basis_c = crate::observe_runtime_repository_body_basis(&runtime)?;
        let (_, acquisition_c) = acquire_test(&runtime, &basis_c, "provider", "model", 3)?;
        assert_eq!(acquisition_c.prior_head.as_ref(), Some(&head_b));
        assert!(backing.compare_and_swap_batch(&[head_b], vec![head_a.clone()])?);
        let inputs_c = projection_inputs(&acquisition_c, repo.path())?;
        assert!(
            execute_preembedded_workspace_coverage_projection(
                &acquisition_c,
                inputs_c,
                &mut FakeProjectionPort::new(Hostility::Honest)
            )
            .is_err()
        );
        let final_entries = backing.pull_all()?;
        assert_eq!(find(&final_entries, HEAD_TYPE, HEAD_KEY), Some(&head_a));
        assert!(
            !final_entries
                .iter()
                .filter(|entry| entry.r#type == RECEIPT_TYPE)
                .any(|entry| decode::<WorkspaceCoverageReceipt>(entry)
                    .is_ok_and(|receipt| receipt.claim_id == acquisition_c.claim.claim_id))
        );
        Ok(())
    }
}
