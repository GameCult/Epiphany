use crate::workspace_coverage_process_documents::authenticate_workspace_coverage_managed_process_launch_with_envelope_digest;
use crate::workspace_coverage_projector::{
    exact_obligation_body_authority, payload_for, validate_claim_attempt_link,
    validate_projection_attempt, validate_projection_claim, WorkspaceCoverageProjectionAttempt,
    WorkspaceCoverageProjectionClaim, ATTEMPT_TYPE, CLAIM_KEY, CLAIM_TYPE, OBLIGATION_TYPE,
    PLAN_TYPE,
};
use crate::workspace_retrieval_coverage::{
    workspace_coverage_execution_collection, WorkspaceCoverageObligation,
    WorkspaceCoverageProjectionPlan,
};
use crate::{
    validate_workspace_coverage_projection_plan, HostIncarnationIdentityEntry,
    RepositoryBodyBinding, RepositoryBodyHead, RepositoryBodyManifest, RepositoryBodyObservation,
    WorkspaceCoverageManagedProcessLaunchEntry, WorkspaceCoveragePointBinding,
    WorkspaceCoverageVectorBinding, BODY_BINDING_KEY, BODY_BINDING_TYPE, BODY_HEAD_KEY,
    BODY_HEAD_TYPE, BODY_MANIFEST_TYPE, BODY_OBSERVATION_TYPE,
};
use anyhow::{anyhow, bail, Context, Result};
use chrono::DateTime;
use cultcache_rs::{
    CacheBackingStore, CultCacheEnvelope, DatabaseEntry, SingleFileMessagePackBackingStore,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::Path;

pub const WORKSPACE_COVERAGE_PROJECTION_BATCH_CHECKPOINT_TYPE: &str =
    "gamecult.epiphany.workspace_coverage_projection_batch_checkpoint";
pub const WORKSPACE_COVERAGE_PROJECTION_BATCH_CHECKPOINT_SCHEMA_VERSION: &str =
    "gamecult.epiphany.workspace_coverage_projection_batch_checkpoint.v0";
pub const WORKSPACE_COVERAGE_BATCH_CHECKPOINT_MAX_POINTS: usize = 128;
const HEAD_TYPE: &str = "gamecult.epiphany.workspace_coverage_projection_batch_checkpoint_head";
const HEAD_SCHEMA: &str =
    "gamecult.epiphany.workspace_coverage_projection_batch_checkpoint_head.v0";
const OBSERVATION_METHOD: &str = "provider-attested-qdrant-readback-v0";
const SIGNATURE_DOMAIN: &[u8] = b"epiphany.workspace-coverage.batch-checkpoint.v0\0";

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.workspace_coverage_projection_batch_checkpoint",
    schema = "WorkspaceCoverageProjectionBatchCheckpointEntry"
)]
pub struct WorkspaceCoverageProjectionBatchCheckpointEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub checkpoint_id: String,
    #[cultcache(key = 2)]
    pub sequence: u64,
    #[cultcache(key = 3, default)]
    pub predecessor_checkpoint_id: Option<String>,
    #[cultcache(key = 4, default)]
    pub predecessor_checkpoint_envelope_digest: Option<String>,
    #[cultcache(key = 5)]
    pub runtime_id: String,
    #[cultcache(key = 6)]
    pub workspace_id: String,
    #[cultcache(key = 7)]
    pub body_binding_sha256: String,
    #[cultcache(key = 8)]
    pub body_binding_envelope_digest: String,
    #[cultcache(key = 9)]
    pub body_head_envelope_digest: String,
    #[cultcache(key = 10)]
    pub body_observation_id: String,
    #[cultcache(key = 11)]
    pub body_generation: u64,
    #[cultcache(key = 12)]
    pub body_observation_envelope_digest: String,
    #[cultcache(key = 13)]
    pub manifest_root_sha256: String,
    #[cultcache(key = 14)]
    pub body_manifest_envelope_digest: String,
    #[cultcache(key = 15)]
    pub obligation_id: String,
    #[cultcache(key = 16)]
    pub obligation_envelope_digest: String,
    #[cultcache(key = 17)]
    pub plan_id: String,
    #[cultcache(key = 18)]
    pub plan_envelope_digest: String,
    #[cultcache(key = 19)]
    pub claim_id: String,
    #[cultcache(key = 20)]
    pub claim_epoch: u64,
    #[cultcache(key = 21)]
    pub claim_envelope_digest: String,
    #[cultcache(key = 22)]
    pub attempt_id: String,
    #[cultcache(key = 23)]
    pub attempt_envelope_digest: String,
    #[cultcache(key = 24)]
    pub managed_process_launch_id: String,
    #[cultcache(key = 25)]
    pub managed_process_launch_envelope_digest: String,
    #[cultcache(key = 26)]
    pub provider_incarnation_id: String,
    #[cultcache(key = 27)]
    pub provider_public_key: Vec<u8>,
    #[cultcache(key = 28)]
    pub projection_schema_version: String,
    #[cultcache(key = 29)]
    pub chunker_id: String,
    #[cultcache(key = 30)]
    pub embedding_provider_id: String,
    #[cultcache(key = 31)]
    pub embedding_artifact_identity: String,
    #[cultcache(key = 32)]
    pub vector_dimensions: u32,
    #[cultcache(key = 33)]
    pub collection_name: String,
    #[cultcache(key = 34)]
    pub batch_ordinal: u64,
    #[cultcache(key = 35)]
    pub first_plan_ordinal: u64,
    #[cultcache(key = 36)]
    pub point_count: u64,
    #[cultcache(key = 37)]
    pub cumulative_point_count: u64,
    #[cultcache(key = 38)]
    pub total_point_count: u64,
    #[cultcache(key = 39)]
    pub point_ids: Vec<String>,
    #[cultcache(key = 40)]
    pub point_bindings: Vec<WorkspaceCoveragePointBinding>,
    #[cultcache(key = 41)]
    pub vector_bindings: Vec<WorkspaceCoverageVectorBinding>,
    #[cultcache(key = 42)]
    pub observed_at_utc: String,
    #[cultcache(key = 43)]
    pub observation_method: String,
    #[cultcache(key = 44)]
    pub provider_signature: Vec<u8>,
    #[cultcache(key = 45)]
    pub signature_algorithm: String,
}

/// Derived claim-local lookup cache. It grants no checkpoint or Qdrant
/// authority; consumers authenticate the immutable event named here.
#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.workspace_coverage_projection_batch_checkpoint_head",
    schema = "WorkspaceCoverageProjectionBatchCheckpointHeadEntry"
)]
pub struct WorkspaceCoverageProjectionBatchCheckpointHeadEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub claim_id: String,
    #[cultcache(key = 2)]
    pub claim_epoch: u64,
    #[cultcache(key = 3)]
    pub checkpoint_id: String,
    #[cultcache(key = 4)]
    pub checkpoint_envelope_digest: String,
    #[cultcache(key = 5)]
    pub sequence: u64,
    #[cultcache(key = 6)]
    pub cumulative_point_count: u64,
    #[cultcache(key = 7)]
    pub total_point_count: u64,
    #[cultcache(key = 8)]
    pub collection_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceCoverageProjectionBatchCheckpointAdmission {
    /// Provider-attested Body admission. Live Qdrant readback remains a later
    /// port integration and is not proven by this result.
    pub checkpoint: WorkspaceCoverageProjectionBatchCheckpointEntry,
    pub checkpoint_envelope_digest: String,
}

pub fn sign_workspace_coverage_projection_batch_checkpoint(
    entry: &mut WorkspaceCoverageProjectionBatchCheckpointEntry,
    key: &SigningKey,
) -> Result<()> {
    if key.verifying_key().to_bytes().as_slice() != entry.provider_public_key.as_slice() {
        bail!("checkpoint signing key disagrees with provider public key");
    }
    entry.provider_signature.clear();
    validate_shape(entry, false)?;
    entry.provider_signature = key.sign(&signature_message(entry)?).to_bytes().to_vec();
    Ok(())
}

pub fn admit_workspace_coverage_projection_batch_checkpoint(
    body_store: impl AsRef<Path>,
    local_verse_store: impl AsRef<Path>,
    trusted_host: &HostIncarnationIdentityEntry,
    entry: WorkspaceCoverageProjectionBatchCheckpointEntry,
) -> Result<WorkspaceCoverageProjectionBatchCheckpointAdmission> {
    validate_shape(&entry, true)?;
    authenticate_signature(&entry)?;
    let (launch, launch_envelope_digest) =
        authenticate_workspace_coverage_managed_process_launch_with_envelope_digest(
            local_verse_store,
            entry.runtime_id.clone(),
            &entry.managed_process_launch_id,
            trusted_host,
        )?;
    validate_launch(&entry, &launch, &launch_envelope_digest)?;
    let backing = SingleFileMessagePackBackingStore::new(body_store.as_ref());
    let opening = backing.pull_all()?;
    let binding_env = find(&opening, BODY_BINDING_TYPE, BODY_BINDING_KEY)?;
    let head_env = find(&opening, BODY_HEAD_TYPE, BODY_HEAD_KEY)?;
    let observation_env = find(&opening, BODY_OBSERVATION_TYPE, &entry.body_observation_id)?;
    let manifest_env = find(&opening, BODY_MANIFEST_TYPE, &entry.manifest_root_sha256)?;
    let obligation_env = find(&opening, OBLIGATION_TYPE, &entry.obligation_id)?;
    let plan_env = find(&opening, PLAN_TYPE, &entry.plan_id)?;
    let claim_env = find(&opening, CLAIM_TYPE, CLAIM_KEY)?;
    let attempt_env = find(&opening, ATTEMPT_TYPE, &entry.attempt_id)?;
    let binding: RepositoryBodyBinding = decode(binding_env)?;
    let body_head: RepositoryBodyHead = decode(head_env)?;
    let observation: RepositoryBodyObservation = decode(observation_env)?;
    let manifest: RepositoryBodyManifest = decode(manifest_env)?;
    let obligation: WorkspaceCoverageObligation = decode(obligation_env)?;
    let plan: WorkspaceCoverageProjectionPlan = decode(plan_env)?;
    let claim: WorkspaceCoverageProjectionClaim = decode(claim_env)?;
    let attempt: WorkspaceCoverageProjectionAttempt = decode(attempt_env)?;
    validate_projection_claim(&claim)?;
    validate_projection_attempt(&attempt)?;
    validate_claim_attempt_link(&claim, &attempt)?;
    validate_workspace_coverage_projection_plan(&obligation, &plan)?;
    exact_obligation_body_authority(&opening, &obligation)?;
    validate_authority(
        &entry,
        &binding,
        binding_env,
        &body_head,
        head_env,
        &observation,
        observation_env,
        &manifest,
        manifest_env,
        &obligation,
        obligation_env,
        &plan,
        plan_env,
        &claim,
        claim_env,
        &attempt,
        attempt_env,
    )?;
    validate_batch_against_plan(&entry, &obligation, &plan)?;

    let event_key = checkpoint_key(&entry.claim_id, &entry.checkpoint_id);
    if opening.iter().any(|env| {
        env.r#type == WORKSPACE_COVERAGE_PROJECTION_BATCH_CHECKPOINT_TYPE && env.key == event_key
    }) {
        bail!("workspace coverage checkpoint identity collision");
    }
    let event_env = envelope(
        WORKSPACE_COVERAGE_PROJECTION_BATCH_CHECKPOINT_TYPE,
        &event_key,
        &entry,
    )?;
    let event_digest = envelope_digest(&event_env);
    let head_key = checkpoint_head_key(&entry.claim_id, entry.claim_epoch);
    let prior_head_env = opening
        .iter()
        .find(|env| env.r#type == HEAD_TYPE && env.key == head_key)
        .cloned();
    let prior_head: Option<WorkspaceCoverageProjectionBatchCheckpointHeadEntry> =
        prior_head_env.as_ref().map(decode).transpose()?;
    let prior_event_env = match prior_head.as_ref() {
        Some(head) => {
            let predecessor_key = checkpoint_key(&head.claim_id, &head.checkpoint_id);
            let predecessor_env = find(
                &opening,
                WORKSPACE_COVERAGE_PROJECTION_BATCH_CHECKPOINT_TYPE,
                &predecessor_key,
            )?
            .clone();
            if envelope_digest(&predecessor_env) != head.checkpoint_envelope_digest {
                bail!("checkpoint head digest disagrees with predecessor event");
            }
            let predecessor: WorkspaceCoverageProjectionBatchCheckpointEntry =
                decode(&predecessor_env)?;
            validate_shape(&predecessor, true)?;
            authenticate_signature(&predecessor)?;
            validate_launch(&predecessor, &launch, &launch_envelope_digest)?;
            validate_predecessor_identity(&predecessor, head, &entry)?;
            Some(predecessor_env)
        }
        None => None,
    };
    validate_chain(&entry, prior_head.as_ref())?;
    let next_head = WorkspaceCoverageProjectionBatchCheckpointHeadEntry {
        schema_version: HEAD_SCHEMA.into(),
        claim_id: entry.claim_id.clone(),
        claim_epoch: entry.claim_epoch,
        checkpoint_id: entry.checkpoint_id.clone(),
        checkpoint_envelope_digest: event_digest,
        sequence: entry.sequence,
        cumulative_point_count: entry.cumulative_point_count,
        total_point_count: entry.total_point_count,
        collection_name: entry.collection_name.clone(),
    };
    let authority = vec![
        binding_env.clone(),
        head_env.clone(),
        observation_env.clone(),
        manifest_env.clone(),
        obligation_env.clone(),
        plan_env.clone(),
        claim_env.clone(),
        attempt_env.clone(),
    ];
    let mut expected = authority.clone();
    if let Some(prior) = prior_head_env {
        expected.push(prior);
    }
    let mut replacements = authority;
    if let Some(predecessor) = prior_event_env {
        expected.push(predecessor.clone());
        replacements.push(predecessor);
    }
    replacements.push(event_env);
    replacements.push(envelope(HEAD_TYPE, &head_key, &next_head)?);
    if !backing.compare_and_swap_batch(&expected, replacements)? {
        bail!("workspace coverage checkpoint lost exact Body/plan/claim/head compare-and-swap");
    }
    Ok(WorkspaceCoverageProjectionBatchCheckpointAdmission {
        checkpoint: entry,
        checkpoint_envelope_digest: next_head.checkpoint_envelope_digest,
    })
}

#[allow(clippy::too_many_arguments)]
fn validate_authority(
    e: &WorkspaceCoverageProjectionBatchCheckpointEntry,
    binding: &RepositoryBodyBinding,
    binding_env: &CultCacheEnvelope,
    head: &RepositoryBodyHead,
    head_env: &CultCacheEnvelope,
    observation: &RepositoryBodyObservation,
    observation_env: &CultCacheEnvelope,
    manifest: &RepositoryBodyManifest,
    manifest_env: &CultCacheEnvelope,
    obligation: &WorkspaceCoverageObligation,
    obligation_env: &CultCacheEnvelope,
    plan: &WorkspaceCoverageProjectionPlan,
    plan_env: &CultCacheEnvelope,
    claim: &WorkspaceCoverageProjectionClaim,
    claim_env: &CultCacheEnvelope,
    attempt: &WorkspaceCoverageProjectionAttempt,
    attempt_env: &CultCacheEnvelope,
) -> Result<()> {
    if e.runtime_id != binding.runtime_id
        || e.workspace_id != binding.workspace_id
        || e.body_binding_sha256 != obligation.body_binding_sha256
        || e.body_binding_envelope_digest != envelope_digest(binding_env)
        || e.body_head_envelope_digest != envelope_digest(head_env)
        || e.body_observation_id != head.observation_id
        || e.body_generation != head.generation
        || e.manifest_root_sha256 != head.manifest_root_sha256
        || observation.observation_id != e.body_observation_id
        || observation.generation != e.body_generation
        || observation.manifest_root_sha256 != e.manifest_root_sha256
        || manifest.manifest_root_sha256 != e.manifest_root_sha256
        || e.body_observation_envelope_digest != envelope_digest(observation_env)
        || e.body_manifest_envelope_digest != envelope_digest(manifest_env)
        || obligation.obligation_id != e.obligation_id
        || obligation.plan_identity(e).is_err()
        || e.obligation_envelope_digest != envelope_digest(obligation_env)
        || plan.plan_id != e.plan_id
        || e.plan_envelope_digest != envelope_digest(plan_env)
        || claim.claim_id != e.claim_id
        || claim.claim_epoch != e.claim_epoch
        || claim.status != "running"
        || claim.plan_id != e.plan_id
        || claim.attempt_id != e.attempt_id
        || claim.obligation_id != e.obligation_id
        || claim.body_observation_id != e.body_observation_id
        || claim.body_generation != e.body_generation
        || claim.manifest_root_sha256 != e.manifest_root_sha256
        || claim.executor_incarnation != e.provider_incarnation_id
        || claim.managed_process_launch_id != e.managed_process_launch_id
        || e.claim_envelope_digest != envelope_digest(claim_env)
        || attempt.attempt_id != e.attempt_id
        || attempt.claim_id != e.claim_id
        || attempt.claim_epoch != e.claim_epoch
        || attempt.plan_id != e.plan_id
        || attempt.status != "running"
        || attempt.executor_incarnation != e.provider_incarnation_id
        || attempt.managed_process_launch_id != e.managed_process_launch_id
        || e.attempt_envelope_digest != envelope_digest(attempt_env)
    {
        bail!("workspace coverage checkpoint authority is substituted or terminal");
    }
    Ok(())
}

trait ObligationIdentity {
    fn plan_identity(&self, e: &WorkspaceCoverageProjectionBatchCheckpointEntry) -> Result<()>;
}
impl ObligationIdentity for WorkspaceCoverageObligation {
    fn plan_identity(&self, e: &WorkspaceCoverageProjectionBatchCheckpointEntry) -> Result<()> {
        if self.runtime_id != e.runtime_id
            || self.workspace_id != e.workspace_id
            || self.body_observation_id != e.body_observation_id
            || self.body_generation != e.body_generation
            || self.manifest_root_sha256 != e.manifest_root_sha256
        {
            bail!("obligation identity mismatch")
        }
        Ok(())
    }
}

fn validate_batch_against_plan(
    e: &WorkspaceCoverageProjectionBatchCheckpointEntry,
    obligation: &WorkspaceCoverageObligation,
    plan: &WorkspaceCoverageProjectionPlan,
) -> Result<()> {
    if plan.obligation_id != e.obligation_id
        || plan.projection_schema_version != e.projection_schema_version
        || plan.chunker_id != e.chunker_id
        || plan.embedding_provider_id != e.embedding_provider_id
        || plan.embedding_model != e.embedding_artifact_identity
        || plan.vector_dimensions != e.vector_dimensions
        || plan.expected_point_count != e.total_point_count
        || workspace_coverage_execution_collection(&plan.plan_id, &e.claim_id, e.claim_epoch)?
            != e.collection_name
    {
        bail!("checkpoint plan/model identity mismatch");
    }
    let start: usize = e.first_plan_ordinal.try_into()?;
    let end = start
        .checked_add(e.point_count.try_into()?)
        .ok_or_else(|| anyhow!("batch range overflow"))?;
    let planned = plan
        .planned_points
        .get(start..end)
        .ok_or_else(|| anyhow!("checkpoint batch exceeds sealed plan"))?;
    if planned
        .iter()
        .map(|p| p.point_id.clone())
        .collect::<Vec<_>>()
        != e.point_ids
    {
        bail!("checkpoint point order/range disagrees with sealed plan");
    }
    for ((planned, point), vector) in planned
        .iter()
        .zip(&e.point_bindings)
        .zip(&e.vector_bindings)
    {
        let expected_payload = payload_for(obligation, plan, planned);
        if point.point_id != planned.point_id
            || vector.point_id != planned.point_id
            || point.payload_sha256 != digest_named(&expected_payload)?
            || !canonical_sha256(&vector.vector_sha256)
        {
            bail!("checkpoint observed bindings disagree with sealed point payload/vector shape");
        }
    }
    Ok(())
}

fn validate_shape(e: &WorkspaceCoverageProjectionBatchCheckpointEntry, signed: bool) -> Result<()> {
    if e.schema_version != WORKSPACE_COVERAGE_PROJECTION_BATCH_CHECKPOINT_SCHEMA_VERSION
        || e.sequence == 0
        || e.claim_epoch == 0
        || e.body_generation == 0
        || e.vector_dimensions == 0
        || e.total_point_count == 0
        || e.point_count == 0
        || e.point_count as usize > WORKSPACE_COVERAGE_BATCH_CHECKPOINT_MAX_POINTS
        || e.point_ids.len() != e.point_count as usize
        || e.point_bindings.len() != e.point_ids.len()
        || e.vector_bindings.len() != e.point_ids.len()
        || e.cumulative_point_count > e.total_point_count
        || e.first_plan_ordinal.checked_add(e.point_count) != Some(e.cumulative_point_count)
        || e.observation_method != OBSERVATION_METHOD
        || e.signature_algorithm != "ed25519"
        || e.provider_public_key.len() != 32
        || (signed && e.provider_signature.len() != 64)
        || (!signed && !e.provider_signature.is_empty())
    {
        bail!("workspace coverage checkpoint shape is invalid");
    }
    if e.point_ids
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len()
        != e.point_ids.len()
    {
        bail!("workspace coverage checkpoint point ids are not unique");
    }
    for value in [
        &e.checkpoint_id,
        &e.runtime_id,
        &e.workspace_id,
        &e.body_binding_sha256,
        &e.body_observation_id,
        &e.manifest_root_sha256,
        &e.obligation_id,
        &e.plan_id,
        &e.claim_id,
        &e.attempt_id,
        &e.managed_process_launch_id,
        &e.provider_incarnation_id,
        &e.embedding_artifact_identity,
        &e.collection_name,
    ] {
        if value.trim().is_empty() {
            bail!("workspace coverage checkpoint identity is empty");
        }
    }
    uuid::Uuid::parse_str(&e.checkpoint_id)?;
    uuid::Uuid::parse_str(&e.managed_process_launch_id)?;
    uuid::Uuid::parse_str(&e.provider_incarnation_id)?;
    DateTime::parse_from_rfc3339(&e.observed_at_utc)?;
    for d in [
        &e.body_binding_envelope_digest,
        &e.body_head_envelope_digest,
        &e.body_observation_envelope_digest,
        &e.body_manifest_envelope_digest,
        &e.obligation_envelope_digest,
        &e.plan_envelope_digest,
        &e.claim_envelope_digest,
        &e.attempt_envelope_digest,
        &e.managed_process_launch_envelope_digest,
    ] {
        if !canonical_envelope_digest(d) {
            bail!("checkpoint authority digest is invalid");
        }
    }
    match (
        &e.predecessor_checkpoint_id,
        &e.predecessor_checkpoint_envelope_digest,
    ) {
        (None, None) if e.sequence == 1 => {}
        (Some(id), Some(d)) if e.sequence > 1 && !id.is_empty() && canonical_envelope_digest(d) => {
        }
        _ => bail!("checkpoint predecessor evidence is partial or misplaced"),
    }
    Ok(())
}

fn validate_chain(
    e: &WorkspaceCoverageProjectionBatchCheckpointEntry,
    prior: Option<&WorkspaceCoverageProjectionBatchCheckpointHeadEntry>,
) -> Result<()> {
    match prior {
        None if e.sequence == 1
            && e.batch_ordinal == 0
            && e.first_plan_ordinal == 0
            && e.predecessor_checkpoint_id.is_none() =>
        {
            Ok(())
        }
        Some(p)
            if p.schema_version == HEAD_SCHEMA
                && p.claim_id == e.claim_id
                && p.claim_epoch == e.claim_epoch
                && p.sequence.checked_add(1) == Some(e.sequence)
                && e.batch_ordinal == p.sequence
                && e.first_plan_ordinal == p.cumulative_point_count
                && e.predecessor_checkpoint_id.as_deref() == Some(&p.checkpoint_id)
                && e.predecessor_checkpoint_envelope_digest.as_deref()
                    == Some(&p.checkpoint_envelope_digest)
                && e.total_point_count == p.total_point_count
                && e.collection_name == p.collection_name =>
        {
            Ok(())
        }
        _ => bail!("workspace coverage checkpoint chain is noncontiguous or mutable"),
    }
}

fn validate_predecessor_identity(
    predecessor: &WorkspaceCoverageProjectionBatchCheckpointEntry,
    head: &WorkspaceCoverageProjectionBatchCheckpointHeadEntry,
    next: &WorkspaceCoverageProjectionBatchCheckpointEntry,
) -> Result<()> {
    if predecessor.checkpoint_id != head.checkpoint_id
        || predecessor.sequence != head.sequence
        || predecessor.claim_id != head.claim_id
        || predecessor.claim_epoch != head.claim_epoch
        || predecessor.cumulative_point_count != head.cumulative_point_count
        || predecessor.total_point_count != head.total_point_count
        || predecessor.collection_name != head.collection_name
        || predecessor.runtime_id != next.runtime_id
        || predecessor.workspace_id != next.workspace_id
        || predecessor.body_binding_sha256 != next.body_binding_sha256
        || predecessor.body_binding_envelope_digest != next.body_binding_envelope_digest
        || predecessor.body_head_envelope_digest != next.body_head_envelope_digest
        || predecessor.body_observation_id != next.body_observation_id
        || predecessor.body_generation != next.body_generation
        || predecessor.body_observation_envelope_digest != next.body_observation_envelope_digest
        || predecessor.manifest_root_sha256 != next.manifest_root_sha256
        || predecessor.body_manifest_envelope_digest != next.body_manifest_envelope_digest
        || predecessor.obligation_id != next.obligation_id
        || predecessor.obligation_envelope_digest != next.obligation_envelope_digest
        || predecessor.plan_id != next.plan_id
        || predecessor.plan_envelope_digest != next.plan_envelope_digest
        || predecessor.claim_id != next.claim_id
        || predecessor.claim_epoch != next.claim_epoch
        || predecessor.claim_envelope_digest != next.claim_envelope_digest
        || predecessor.attempt_id != next.attempt_id
        || predecessor.attempt_envelope_digest != next.attempt_envelope_digest
        || predecessor.managed_process_launch_id != next.managed_process_launch_id
        || predecessor.managed_process_launch_envelope_digest
            != next.managed_process_launch_envelope_digest
        || predecessor.provider_incarnation_id != next.provider_incarnation_id
        || predecessor.provider_public_key != next.provider_public_key
        || predecessor.projection_schema_version != next.projection_schema_version
        || predecessor.chunker_id != next.chunker_id
        || predecessor.embedding_provider_id != next.embedding_provider_id
        || predecessor.embedding_artifact_identity != next.embedding_artifact_identity
        || predecessor.vector_dimensions != next.vector_dimensions
    {
        bail!("checkpoint predecessor/head/claim immutable identity disagrees");
    }
    Ok(())
}

fn validate_launch(
    e: &WorkspaceCoverageProjectionBatchCheckpointEntry,
    l: &WorkspaceCoverageManagedProcessLaunchEntry,
    d: &str,
) -> Result<()> {
    if e.managed_process_launch_id != l.launch_id
        || e.managed_process_launch_envelope_digest != d
        || e.runtime_id != l.runtime_id
        || e.provider_incarnation_id != l.provider_incarnation_id
        || e.provider_public_key != l.provider_public_key
    {
        bail!("checkpoint disagrees with exact managed launch")
    }
    Ok(())
}
fn authenticate_signature(e: &WorkspaceCoverageProjectionBatchCheckpointEntry) -> Result<()> {
    let k: [u8; 32] = e.provider_public_key.as_slice().try_into()?;
    let s: [u8; 64] = e.provider_signature.as_slice().try_into()?;
    VerifyingKey::from_bytes(&k)?
        .verify(&signature_message(e)?, &Signature::from_bytes(&s))
        .context("checkpoint signature invalid")
}
fn signature_message(e: &WorkspaceCoverageProjectionBatchCheckpointEntry) -> Result<Vec<u8>> {
    let mut u = e.clone();
    u.provider_signature.clear();
    validate_shape(&u, false)?;
    let b = rmp_serde::to_vec_named(&u)?;
    let mut out = SIGNATURE_DOMAIN.to_vec();
    out.extend_from_slice(&(b.len() as u64).to_be_bytes());
    out.extend(b);
    Ok(out)
}
fn checkpoint_key(c: &str, id: &str) -> String {
    format!("workspace-coverage/checkpoint/{c}/{id}")
}
fn checkpoint_head_key(c: &str, e: u64) -> String {
    format!("workspace-coverage/checkpoint-head/{c}/{e}")
}
fn find<'a>(v: &'a [CultCacheEnvelope], t: &str, k: &str) -> Result<&'a CultCacheEnvelope> {
    v.iter()
        .find(|e| e.r#type == t && e.key == k)
        .ok_or_else(|| anyhow!("missing authority envelope {t}/{k}"))
}
fn decode<T: serde::de::DeserializeOwned>(e: &CultCacheEnvelope) -> Result<T> {
    Ok(rmp_serde::from_slice(&e.payload)?)
}
fn envelope<T: Serialize>(t: &str, k: &str, v: &T) -> Result<CultCacheEnvelope> {
    Ok(CultCacheEnvelope {
        r#type: t.into(),
        key: k.into(),
        payload: rmp_serde::to_vec(v)?,
        stored_at: chrono::Utc::now().to_rfc3339(),
        schema_id: Some(t.into()),
    })
}
fn envelope_digest(e: &CultCacheEnvelope) -> String {
    let mut d = Sha256::new();
    d.update(e.r#type.as_bytes());
    d.update([0]);
    d.update(e.key.as_bytes());
    d.update([0]);
    d.update(&e.payload);
    format!("sha256-{:x}", d.finalize())
}
fn digest_named<T: Serialize>(v: &T) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(v)?)))
}
fn canonical_sha256(v: &str) -> bool {
    v.len() == 64
        && v.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn canonical_envelope_digest(v: &str) -> bool {
    v.strip_prefix("sha256-").is_some_and(canonical_sha256)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn sample(k: &SigningKey) -> WorkspaceCoverageProjectionBatchCheckpointEntry {
        let mut e = WorkspaceCoverageProjectionBatchCheckpointEntry {
            schema_version: WORKSPACE_COVERAGE_PROJECTION_BATCH_CHECKPOINT_SCHEMA_VERSION.into(),
            checkpoint_id: uuid::Uuid::new_v4().to_string(),
            sequence: 1,
            predecessor_checkpoint_id: None,
            predecessor_checkpoint_envelope_digest: None,
            runtime_id: "runtime".into(),
            workspace_id: "workspace".into(),
            body_binding_sha256: "body".into(),
            body_binding_envelope_digest: format!("sha256-{}", "a".repeat(64)),
            body_head_envelope_digest: format!("sha256-{}", "b".repeat(64)),
            body_observation_id: "observation".into(),
            body_generation: 1,
            body_observation_envelope_digest: format!("sha256-{}", "c".repeat(64)),
            manifest_root_sha256: "manifest".into(),
            body_manifest_envelope_digest: format!("sha256-{}", "d".repeat(64)),
            obligation_id: "obligation".into(),
            obligation_envelope_digest: format!("sha256-{}", "e".repeat(64)),
            plan_id: "plan".into(),
            plan_envelope_digest: format!("sha256-{}", "f".repeat(64)),
            claim_id: "claim".into(),
            claim_epoch: 1,
            claim_envelope_digest: format!("sha256-{}", "1".repeat(64)),
            attempt_id: "attempt".into(),
            attempt_envelope_digest: format!("sha256-{}", "2".repeat(64)),
            managed_process_launch_id: uuid::Uuid::new_v4().to_string(),
            managed_process_launch_envelope_digest: format!("sha256-{}", "3".repeat(64)),
            provider_incarnation_id: uuid::Uuid::new_v4().to_string(),
            provider_public_key: k.verifying_key().to_bytes().to_vec(),
            projection_schema_version: "projection".into(),
            chunker_id: "chunker".into(),
            embedding_provider_id: "provider".into(),
            embedding_artifact_identity: "model".into(),
            vector_dimensions: 3,
            collection_name: "collection".into(),
            batch_ordinal: 0,
            first_plan_ordinal: 0,
            point_count: 1,
            cumulative_point_count: 1,
            total_point_count: 2,
            point_ids: vec!["point".into()],
            point_bindings: vec![WorkspaceCoveragePointBinding {
                point_id: "point".into(),
                payload_sha256: "a".repeat(64),
            }],
            vector_bindings: vec![WorkspaceCoverageVectorBinding {
                point_id: "point".into(),
                vector_sha256: "b".repeat(64),
            }],
            observed_at_utc: "2026-07-17T00:00:00Z".into(),
            observation_method: OBSERVATION_METHOD.into(),
            provider_signature: vec![],
            signature_algorithm: "ed25519".into(),
        };
        sign_workspace_coverage_projection_batch_checkpoint(&mut e, k).unwrap();
        e
    }
    #[test]
    fn shape_signature_and_genesis_are_strict() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let e = sample(&k);
        authenticate_signature(&e).unwrap();
        validate_chain(&e, None).unwrap();
        let mut bad = e.clone();
        bad.sequence = 2;
        assert!(validate_chain(&bad, None).is_err());
        bad = e.clone();
        bad.provider_signature[0] ^= 1;
        assert!(authenticate_signature(&bad).is_err());
    }
    #[test]
    fn gaps_overlap_reorder_and_mutable_head_are_refused() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let mut e = sample(&k);
        e.sequence = 2;
        e.batch_ordinal = 1;
        e.first_plan_ordinal = 1;
        e.cumulative_point_count = 2;
        e.predecessor_checkpoint_id = Some("prior".into());
        e.predecessor_checkpoint_envelope_digest = Some(format!("sha256-{}", "9".repeat(64)));
        let h = WorkspaceCoverageProjectionBatchCheckpointHeadEntry {
            schema_version: HEAD_SCHEMA.into(),
            claim_id: e.claim_id.clone(),
            claim_epoch: 1,
            checkpoint_id: "prior".into(),
            checkpoint_envelope_digest: e.predecessor_checkpoint_envelope_digest.clone().unwrap(),
            sequence: 1,
            cumulative_point_count: 1,
            total_point_count: 2,
            collection_name: "collection".into(),
        };
        assert!(validate_chain(&e, Some(&h)).is_ok());
        for m in ["gap", "overlap", "total", "collection"] {
            let mut x = e.clone();
            match m {
                "gap" => x.first_plan_ordinal = 2,
                "overlap" => x.batch_ordinal = 0,
                "total" => x.total_point_count = 3,
                _ => x.collection_name.push('x'),
            };
            assert!(validate_chain(&x, Some(&h)).is_err(), "{m}");
        }
    }
    #[test]
    fn bounded_batch_and_identity_collision_are_refused() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let mut e = sample(&k);
        e.point_count = 129;
        assert!(validate_shape(&e, true).is_err());
        let existing = e.clone();
        let mut collision = e;
        collision.collection_name.push('x');
        assert_ne!(existing, collision);
        assert_eq!(
            checkpoint_key(&existing.claim_id, &existing.checkpoint_id),
            checkpoint_key(&collision.claim_id, &collision.checkpoint_id)
        );
    }
    #[test]
    fn nonlexical_plan_order_and_arbitrary_canonical_vector_hash_are_shape_valid() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let mut e = sample(&k);
        e.point_count = 2;
        e.cumulative_point_count = 2;
        e.point_ids = vec!["z-plan-first".into(), "a-plan-second".into()];
        e.point_bindings = e
            .point_ids
            .iter()
            .map(|id| WorkspaceCoveragePointBinding {
                point_id: id.clone(),
                payload_sha256: "a".repeat(64),
            })
            .collect();
        e.vector_bindings = vec![
            WorkspaceCoverageVectorBinding {
                point_id: e.point_ids[0].clone(),
                vector_sha256: "0123456789abcdef".repeat(4),
            },
            WorkspaceCoverageVectorBinding {
                point_id: e.point_ids[1].clone(),
                vector_sha256: "fedcba9876543210".repeat(4),
            },
        ];
        e.provider_signature.clear();
        sign_workspace_coverage_projection_batch_checkpoint(&mut e, &k).unwrap();
        assert!(validate_shape(&e, true).is_ok());
    }
    #[test]
    fn checked_arithmetic_refuses_maximum_overflow() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let mut e = sample(&k);
        e.first_plan_ordinal = u64::MAX;
        e.cumulative_point_count = u64::MAX;
        assert!(validate_shape(&e, true).is_err());
        let head = WorkspaceCoverageProjectionBatchCheckpointHeadEntry {
            schema_version: HEAD_SCHEMA.into(),
            claim_id: e.claim_id.clone(),
            claim_epoch: e.claim_epoch,
            checkpoint_id: "prior".into(),
            checkpoint_envelope_digest: format!("sha256-{}", "9".repeat(64)),
            sequence: u64::MAX,
            cumulative_point_count: 1,
            total_point_count: e.total_point_count,
            collection_name: e.collection_name.clone(),
        };
        e.sequence = 1;
        assert!(validate_chain(&e, Some(&head)).is_err());
    }
    #[test]
    fn authority_model_collection_and_total_are_signed() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let e = sample(&k);
        for m in ["body", "plan", "model", "collection", "total", "launch"] {
            let mut x = e.clone();
            match m {
                "body" => x.body_generation += 1,
                "plan" => x.plan_id.push('x'),
                "model" => x.embedding_artifact_identity.push('x'),
                "collection" => x.collection_name.push('x'),
                "total" => x.total_point_count += 1,
                _ => {
                    x.managed_process_launch_envelope_digest = format!("sha256-{}", "4".repeat(64))
                }
            };
            assert!(authenticate_signature(&x).is_err(), "{m}");
        }
    }
}
