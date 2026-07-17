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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ObservedWorkspaceCoverageBatchInput {
    pub claim_id: String,
    pub attempt_id: String,
    pub plan_id: String,
    pub first_plan_ordinal: u64,
    pub point_bindings: Vec<WorkspaceCoveragePointBinding>,
    pub vector_bindings: Vec<WorkspaceCoverageVectorBinding>,
}

pub(crate) fn admit_observed_workspace_coverage_batch(
    body_store: impl AsRef<Path>,
    local_verse_store: impl AsRef<Path>,
    runtime_id: &str,
    trusted_host: &HostIncarnationIdentityEntry,
    provider_signing_key: &SigningKey,
    observed: ObservedWorkspaceCoverageBatchInput,
) -> Result<WorkspaceCoverageProjectionBatchCheckpointAdmission> {
    validate_observed_batch_input(&observed)?;
    let body_store = body_store.as_ref();
    let opening = SingleFileMessagePackBackingStore::new(body_store).pull_all()?;
    let claim_env = find(&opening, CLAIM_TYPE, CLAIM_KEY)?;
    let claim: WorkspaceCoverageProjectionClaim = decode(claim_env)?;
    let attempt_env = find(&opening, ATTEMPT_TYPE, &claim.attempt_id)?;
    let attempt: WorkspaceCoverageProjectionAttempt = decode(attempt_env)?;
    validate_claim_attempt_link(&claim, &attempt)?;
    if claim.status != "running"
        || claim.claim_id != observed.claim_id
        || attempt.attempt_id != observed.attempt_id
        || claim.plan_id != observed.plan_id
    {
        bail!("observed batch expected identity disagrees with current running authority");
    }
    let obligation_env = find(&opening, OBLIGATION_TYPE, &claim.obligation_id)?;
    let plan_env = find(&opening, PLAN_TYPE, &claim.plan_id)?;
    let obligation: WorkspaceCoverageObligation = decode(obligation_env)?;
    let plan: WorkspaceCoverageProjectionPlan = decode(plan_env)?;
    validate_workspace_coverage_projection_plan(&obligation, &plan)?;
    let authority = exact_obligation_body_authority(&opening, &obligation)?;
    let binding_env = find(&authority, BODY_BINDING_TYPE, BODY_BINDING_KEY)?;
    let body_head_env = find(&authority, BODY_HEAD_TYPE, BODY_HEAD_KEY)?;
    let observation_env = find(
        &authority,
        BODY_OBSERVATION_TYPE,
        &obligation.body_observation_id,
    )?;
    let manifest_env = find(
        &authority,
        BODY_MANIFEST_TYPE,
        &obligation.manifest_root_sha256,
    )?;
    let binding: RepositoryBodyBinding = decode(binding_env)?;
    if binding.runtime_id != runtime_id || obligation.runtime_id != runtime_id {
        bail!("observed batch runtime disagrees with Body authority");
    }
    let (launch, launch_digest) =
        authenticate_workspace_coverage_managed_process_launch_with_envelope_digest(
            local_verse_store.as_ref(),
            runtime_id,
            &claim.managed_process_launch_id,
            trusted_host,
        )?;
    if launch.provider_incarnation_id != claim.executor_incarnation
        || launch.provider_daemon_id != claim.executor_id
        || provider_signing_key.verifying_key().to_bytes().as_slice()
            != launch.provider_public_key.as_slice()
    {
        bail!("observed batch signer/launch disagrees with claim executor");
    }
    if let Some(current) = load_authenticated_current_checkpoint(
        body_store,
        local_verse_store.as_ref(),
        trusted_host,
        &claim.claim_id,
        claim.claim_epoch,
    )? {
        let progress = crate::workspace_coverage_projection_progress::load_latest_workspace_coverage_projection_progress(
            local_verse_store.as_ref(),
            runtime_id,
            &claim.managed_process_launch_id,
            &claim.claim_id,
        )?
        .ok_or_else(|| anyhow!("current checkpoint exists without canonical progress genesis"))?;
        if progress.sequence != current.checkpoint.sequence.checked_add(1).ok_or_else(|| anyhow!("checkpoint sequence exhausted"))?
            || progress.checkpoint_id.as_deref() != Some(current.checkpoint.checkpoint_id.as_str())
            || progress.checkpoint_binding_sha256.as_deref() != Some(current.checkpoint_envelope_digest.as_str())
            || progress.completed_units != current.checkpoint.cumulative_point_count
        {
            bail!("current checkpoint is ahead of progress; reconcile before admitting a new batch");
        }
    }
    let head_key = checkpoint_head_key(&claim.claim_id, claim.claim_epoch);
    let prior: Option<WorkspaceCoverageProjectionBatchCheckpointHeadEntry> = opening
        .iter()
        .find(|env| env.r#type == HEAD_TYPE && env.key == head_key)
        .map(decode)
        .transpose()?;
    let sequence = prior.as_ref().map_or(Ok(1), |head| {
        head.sequence
            .checked_add(1)
            .ok_or_else(|| anyhow!("checkpoint sequence exhausted"))
    })?;
    let batch_ordinal = sequence
        .checked_sub(1)
        .ok_or_else(|| anyhow!("checkpoint ordinal underflow"))?;
    let point_count: u64 = observed.point_bindings.len().try_into()?;
    let cumulative = observed
        .first_plan_ordinal
        .checked_add(point_count)
        .ok_or_else(|| anyhow!("checkpoint cumulative count overflow"))?;
    let point_ids = observed
        .point_bindings
        .iter()
        .map(|binding| binding.point_id.clone())
        .collect();
    let mut entry = WorkspaceCoverageProjectionBatchCheckpointEntry {
        schema_version: WORKSPACE_COVERAGE_PROJECTION_BATCH_CHECKPOINT_SCHEMA_VERSION.into(),
        checkpoint_id: uuid::Uuid::new_v4().to_string(),
        sequence,
        predecessor_checkpoint_id: prior.as_ref().map(|head| head.checkpoint_id.clone()),
        predecessor_checkpoint_envelope_digest: prior
            .as_ref()
            .map(|head| head.checkpoint_envelope_digest.clone()),
        runtime_id: runtime_id.into(),
        workspace_id: obligation.workspace_id.clone(),
        body_binding_sha256: obligation.body_binding_sha256.clone(),
        body_binding_envelope_digest: envelope_digest(binding_env),
        body_head_envelope_digest: envelope_digest(body_head_env),
        body_observation_id: obligation.body_observation_id.clone(),
        body_generation: obligation.body_generation,
        body_observation_envelope_digest: envelope_digest(observation_env),
        manifest_root_sha256: obligation.manifest_root_sha256.clone(),
        body_manifest_envelope_digest: envelope_digest(manifest_env),
        obligation_id: obligation.obligation_id.clone(),
        obligation_envelope_digest: envelope_digest(obligation_env),
        plan_id: plan.plan_id.clone(),
        plan_envelope_digest: envelope_digest(plan_env),
        claim_id: claim.claim_id.clone(),
        claim_epoch: claim.claim_epoch,
        claim_envelope_digest: envelope_digest(claim_env),
        attempt_id: attempt.attempt_id.clone(),
        attempt_envelope_digest: envelope_digest(attempt_env),
        managed_process_launch_id: launch.launch_id.clone(),
        managed_process_launch_envelope_digest: launch_digest,
        provider_incarnation_id: launch.provider_incarnation_id.clone(),
        provider_public_key: launch.provider_public_key.clone(),
        projection_schema_version: plan.projection_schema_version.clone(),
        chunker_id: plan.chunker_id.clone(),
        embedding_provider_id: plan.embedding_provider_id.clone(),
        embedding_artifact_identity: plan.embedding_model.clone(),
        vector_dimensions: plan.vector_dimensions,
        collection_name: workspace_coverage_execution_collection(
            &plan.plan_id,
            &claim.claim_id,
            claim.claim_epoch,
        )?,
        batch_ordinal,
        first_plan_ordinal: observed.first_plan_ordinal,
        point_count,
        cumulative_point_count: cumulative,
        total_point_count: plan.expected_point_count,
        point_ids,
        point_bindings: observed.point_bindings,
        vector_bindings: observed.vector_bindings,
        observed_at_utc: chrono::Utc::now().to_rfc3339(),
        observation_method: OBSERVATION_METHOD.into(),
        provider_signature: Vec::new(),
        signature_algorithm: "ed25519".into(),
    };
    sign_workspace_coverage_projection_batch_checkpoint(&mut entry, provider_signing_key)?;
    admit_workspace_coverage_projection_batch_checkpoint(
        body_store,
        local_verse_store,
        trusted_host,
        entry,
    )
}

pub(crate) fn load_authenticated_current_checkpoint(
    body_store: impl AsRef<Path>,
    local_verse_store: impl AsRef<Path>,
    trusted_host: &HostIncarnationIdentityEntry,
    claim_id: &str,
    claim_epoch: u64,
) -> Result<Option<WorkspaceCoverageProjectionBatchCheckpointAdmission>> {
    let opening = SingleFileMessagePackBackingStore::new(body_store.as_ref()).pull_all()?;
    let head_key = checkpoint_head_key(claim_id, claim_epoch);
    let Some(head_env) = opening
        .iter()
        .find(|env| env.r#type == HEAD_TYPE && env.key == head_key)
    else {
        return Ok(None);
    };
    let head: WorkspaceCoverageProjectionBatchCheckpointHeadEntry = decode(head_env)?;
    if head.schema_version != HEAD_SCHEMA
        || head.claim_id != claim_id
        || head.claim_epoch != claim_epoch
        || head.sequence == 0
    {
        bail!("current checkpoint head identity is invalid");
    }
    let event_env = find(
        &opening,
        WORKSPACE_COVERAGE_PROJECTION_BATCH_CHECKPOINT_TYPE,
        &checkpoint_key(claim_id, &head.checkpoint_id),
    )?;
    if envelope_digest(event_env) != head.checkpoint_envelope_digest {
        bail!("current checkpoint head digest disagrees with event");
    }
    let checkpoint: WorkspaceCoverageProjectionBatchCheckpointEntry = decode(event_env)?;
    validate_shape(&checkpoint, true)?;
    authenticate_signature(&checkpoint)?;
    let (launch, launch_digest) =
        authenticate_workspace_coverage_managed_process_launch_with_envelope_digest(
            local_verse_store,
            checkpoint.runtime_id.clone(),
            &checkpoint.managed_process_launch_id,
            trusted_host,
        )?;
    validate_launch(&checkpoint, &launch, &launch_digest)?;
    validate_current_head_event_identity(&checkpoint, &head)?;
    let binding_env = find(&opening, BODY_BINDING_TYPE, BODY_BINDING_KEY)?;
    let body_head_env = find(&opening, BODY_HEAD_TYPE, BODY_HEAD_KEY)?;
    let observation_env = find(
        &opening,
        BODY_OBSERVATION_TYPE,
        &checkpoint.body_observation_id,
    )?;
    let manifest_env = find(
        &opening,
        BODY_MANIFEST_TYPE,
        &checkpoint.manifest_root_sha256,
    )?;
    let obligation_env = find(&opening, OBLIGATION_TYPE, &checkpoint.obligation_id)?;
    let plan_env = find(&opening, PLAN_TYPE, &checkpoint.plan_id)?;
    let claim_env = find(&opening, CLAIM_TYPE, CLAIM_KEY)?;
    let attempt_env = find(&opening, ATTEMPT_TYPE, &checkpoint.attempt_id)?;
    let binding: RepositoryBodyBinding = decode(binding_env)?;
    let body_head: RepositoryBodyHead = decode(body_head_env)?;
    let observation: RepositoryBodyObservation = decode(observation_env)?;
    let manifest: RepositoryBodyManifest = decode(manifest_env)?;
    let obligation: WorkspaceCoverageObligation = decode(obligation_env)?;
    let plan: WorkspaceCoverageProjectionPlan = decode(plan_env)?;
    let claim: WorkspaceCoverageProjectionClaim = decode(claim_env)?;
    let attempt: WorkspaceCoverageProjectionAttempt = decode(attempt_env)?;
    validate_projection_claim(&claim)?;
    validate_projection_attempt(&attempt)?;
    validate_claim_attempt_link(&claim, &attempt)?;
    validate_current_projection_binding(
        &checkpoint,
        CurrentProjectionBinding {
            claim_status: &claim.status,
            claim_id: &claim.claim_id,
            claim_epoch: claim.claim_epoch,
            attempt_status: &attempt.status,
            attempt_id: &attempt.attempt_id,
            claim_plan_id: &claim.plan_id,
            attempt_plan_id: &attempt.plan_id,
            plan_id: &plan.plan_id,
            claim_obligation_id: &claim.obligation_id,
            obligation_id: &obligation.obligation_id,
            plan_obligation_id: &plan.obligation_id,
            claim_body_observation_id: &claim.body_observation_id,
            obligation_body_observation_id: &obligation.body_observation_id,
            claim_body_generation: claim.body_generation,
            obligation_body_generation: obligation.body_generation,
            claim_manifest_root: &claim.manifest_root_sha256,
            obligation_manifest_root: &obligation.manifest_root_sha256,
        },
    )?;
    validate_workspace_coverage_projection_plan(&obligation, &plan)?;
    exact_obligation_body_authority(&opening, &obligation)?;
    validate_authority(
        &checkpoint,
        &binding,
        binding_env,
        &body_head,
        body_head_env,
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
    validate_batch_against_plan(&checkpoint, &obligation, &plan)?;
    Ok(Some(WorkspaceCoverageProjectionBatchCheckpointAdmission {
        checkpoint,
        checkpoint_envelope_digest: head.checkpoint_envelope_digest,
    }))
}

pub(crate) fn load_authenticated_checkpoint_chain(
    body_store: impl AsRef<Path>,
    local_verse_store: impl AsRef<Path>,
    trusted_host: &HostIncarnationIdentityEntry,
    claim_id: &str,
    claim_epoch: u64,
) -> Result<Vec<WorkspaceCoverageProjectionBatchCheckpointAdmission>> {
    let Some(current) = load_authenticated_current_checkpoint(
        body_store.as_ref(),
        local_verse_store.as_ref(),
        trusted_host,
        claim_id,
        claim_epoch,
    )? else {
        return Ok(Vec::new());
    };
    let opening = SingleFileMessagePackBackingStore::new(body_store.as_ref()).pull_all()?;
    let head_env = find(
        &opening,
        HEAD_TYPE,
        &checkpoint_head_key(claim_id, claim_epoch),
    )?;
    let head: WorkspaceCoverageProjectionBatchCheckpointHeadEntry = decode(head_env)?;
    if head.checkpoint_id != current.checkpoint.checkpoint_id
        || head.checkpoint_envelope_digest != current.checkpoint_envelope_digest
    {
        bail!("checkpoint head changed while reconstructing authenticated chain");
    }
    let binding_env = find(&opening, BODY_BINDING_TYPE, BODY_BINDING_KEY)?;
    let body_head_env = find(&opening, BODY_HEAD_TYPE, BODY_HEAD_KEY)?;
    let observation_env = find(
        &opening,
        BODY_OBSERVATION_TYPE,
        &current.checkpoint.body_observation_id,
    )?;
    let manifest_env = find(
        &opening,
        BODY_MANIFEST_TYPE,
        &current.checkpoint.manifest_root_sha256,
    )?;
    let obligation_env = find(&opening, OBLIGATION_TYPE, &current.checkpoint.obligation_id)?;
    let plan_env = find(&opening, PLAN_TYPE, &current.checkpoint.plan_id)?;
    let claim_env = find(&opening, CLAIM_TYPE, CLAIM_KEY)?;
    let attempt_env = find(&opening, ATTEMPT_TYPE, &current.checkpoint.attempt_id)?;
    let binding: RepositoryBodyBinding = decode(binding_env)?;
    let body_head: RepositoryBodyHead = decode(body_head_env)?;
    let observation: RepositoryBodyObservation = decode(observation_env)?;
    let manifest: RepositoryBodyManifest = decode(manifest_env)?;
    let obligation: WorkspaceCoverageObligation = decode(obligation_env)?;
    let plan: WorkspaceCoverageProjectionPlan = decode(plan_env)?;
    let claim: WorkspaceCoverageProjectionClaim = decode(claim_env)?;
    let attempt: WorkspaceCoverageProjectionAttempt = decode(attempt_env)?;
    validate_projection_claim(&claim)?;
    validate_projection_attempt(&attempt)?;
    validate_claim_attempt_link(&claim, &attempt)?;
    validate_current_projection_binding(&current.checkpoint, CurrentProjectionBinding {
        claim_status: &claim.status, claim_id: &claim.claim_id, claim_epoch: claim.claim_epoch,
        attempt_status: &attempt.status, attempt_id: &attempt.attempt_id,
        claim_plan_id: &claim.plan_id, attempt_plan_id: &attempt.plan_id, plan_id: &plan.plan_id,
        claim_obligation_id: &claim.obligation_id, obligation_id: &obligation.obligation_id,
        plan_obligation_id: &plan.obligation_id,
        claim_body_observation_id: &claim.body_observation_id,
        obligation_body_observation_id: &obligation.body_observation_id,
        claim_body_generation: claim.body_generation,
        obligation_body_generation: obligation.body_generation,
        claim_manifest_root: &claim.manifest_root_sha256,
        obligation_manifest_root: &obligation.manifest_root_sha256,
    })?;
    validate_workspace_coverage_projection_plan(&obligation, &plan)?;
    exact_obligation_body_authority(&opening, &obligation)?;
    let (launch, launch_digest) =
        authenticate_workspace_coverage_managed_process_launch_with_envelope_digest(
            local_verse_store,
            current.checkpoint.runtime_id.clone(),
            &current.checkpoint.managed_process_launch_id,
            trusted_host,
        )?;

    let mut reversed = Vec::new();
    let mut next_id = Some(head.checkpoint_id.clone());
    let mut next_digest = Some(head.checkpoint_envelope_digest.clone());
    let mut seen_checkpoints = std::collections::HashSet::new();
    while let (Some(id), Some(expected_digest)) = (next_id.take(), next_digest.take()) {
        if !seen_checkpoints.insert(id.clone()) {
            bail!("checkpoint predecessor chain contains a cycle or duplicate event");
        }
        let event_env = find(
            &opening,
            WORKSPACE_COVERAGE_PROJECTION_BATCH_CHECKPOINT_TYPE,
            &checkpoint_key(claim_id, &id),
        )?;
        if envelope_digest(event_env) != expected_digest {
            bail!("checkpoint predecessor digest disagrees with exact event envelope");
        }
        let event: WorkspaceCoverageProjectionBatchCheckpointEntry = decode(event_env)?;
        validate_shape(&event, true)?;
        authenticate_signature(&event)?;
        validate_launch(&event, &launch, &launch_digest)?;
        validate_authority(
            &event, &binding, binding_env, &body_head, body_head_env, &observation,
            observation_env, &manifest, manifest_env, &obligation, obligation_env, &plan,
            plan_env, &claim, claim_env, &attempt, attempt_env,
        )?;
        validate_batch_against_plan(&event, &obligation, &plan)?;
        next_id = event.predecessor_checkpoint_id.clone();
        next_digest = event.predecessor_checkpoint_envelope_digest.clone();
        reversed.push(WorkspaceCoverageProjectionBatchCheckpointAdmission {
            checkpoint: event,
            checkpoint_envelope_digest: expected_digest,
        });
    }
    reversed.reverse();
    validate_reconstructed_chain(&reversed, &head)?;
    Ok(reversed)
}

fn validate_reconstructed_chain(
    chain: &[WorkspaceCoverageProjectionBatchCheckpointAdmission],
    head: &WorkspaceCoverageProjectionBatchCheckpointHeadEntry,
) -> Result<()> {
    let mut prior: Option<&WorkspaceCoverageProjectionBatchCheckpointEntry> = None;
    let mut seen_points = std::collections::HashSet::new();
    for admission in chain {
        let event = &admission.checkpoint;
        let expected_sequence = match prior {
            Some(p) => p.sequence.checked_add(1).ok_or_else(|| anyhow!("checkpoint sequence exhausted"))?,
            None => 1,
        };
        if event.sequence != expected_sequence
            || event.first_plan_ordinal != prior.map_or(0, |p| p.cumulative_point_count)
            || event.batch_ordinal != event.sequence - 1
            || event.point_ids.iter().any(|id| !seen_points.insert(id.clone()))
        {
            bail!("checkpoint chain is noncontiguous or overlaps prior sealed points");
        }
        prior = Some(event);
    }
    if prior.map(|p| p.sequence) != Some(head.sequence)
        || prior.map(|p| p.cumulative_point_count) != Some(head.cumulative_point_count)
    {
        bail!("checkpoint chain does not reconstruct the authenticated current head");
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct CurrentProjectionBinding<'a> {
    claim_status: &'a str,
    claim_id: &'a str,
    claim_epoch: u64,
    attempt_status: &'a str,
    attempt_id: &'a str,
    claim_plan_id: &'a str,
    attempt_plan_id: &'a str,
    plan_id: &'a str,
    claim_obligation_id: &'a str,
    obligation_id: &'a str,
    plan_obligation_id: &'a str,
    claim_body_observation_id: &'a str,
    obligation_body_observation_id: &'a str,
    claim_body_generation: u64,
    obligation_body_generation: u64,
    claim_manifest_root: &'a str,
    obligation_manifest_root: &'a str,
}

fn validate_current_projection_binding(
    checkpoint: &WorkspaceCoverageProjectionBatchCheckpointEntry,
    current: CurrentProjectionBinding<'_>,
) -> Result<()> {
    if current.claim_status != "running"
        || current.attempt_status != "running"
        || checkpoint.claim_id != current.claim_id
        || checkpoint.claim_epoch != current.claim_epoch
        || checkpoint.attempt_id != current.attempt_id
        || checkpoint.plan_id != current.claim_plan_id
        || checkpoint.plan_id != current.attempt_plan_id
        || checkpoint.plan_id != current.plan_id
        || checkpoint.obligation_id != current.claim_obligation_id
        || checkpoint.obligation_id != current.obligation_id
        || current.plan_obligation_id != current.obligation_id
        || checkpoint.body_observation_id != current.claim_body_observation_id
        || checkpoint.body_observation_id != current.obligation_body_observation_id
        || checkpoint.body_generation != current.claim_body_generation
        || checkpoint.body_generation != current.obligation_body_generation
        || checkpoint.manifest_root_sha256 != current.claim_manifest_root
        || checkpoint.manifest_root_sha256 != current.obligation_manifest_root
    {
        bail!("checkpoint no longer belongs to current running Body projection authority");
    }
    Ok(())
}

fn validate_current_head_event_identity(
    checkpoint: &WorkspaceCoverageProjectionBatchCheckpointEntry,
    head: &WorkspaceCoverageProjectionBatchCheckpointHeadEntry,
) -> Result<()> {
    if checkpoint.claim_id != head.claim_id
        || checkpoint.claim_epoch != head.claim_epoch
        || checkpoint.checkpoint_id != head.checkpoint_id
        || checkpoint.sequence != head.sequence
        || checkpoint.cumulative_point_count != head.cumulative_point_count
        || checkpoint.total_point_count != head.total_point_count
        || checkpoint.collection_name != head.collection_name
    {
        bail!("current checkpoint head is stale or substituted");
    }
    Ok(())
}

fn validate_observed_batch_input(observed: &ObservedWorkspaceCoverageBatchInput) -> Result<()> {
    if observed.claim_id.trim().is_empty()
        || observed.attempt_id.trim().is_empty()
        || observed.plan_id.trim().is_empty()
        || observed.point_bindings.is_empty()
        || observed.point_bindings.len() > WORKSPACE_COVERAGE_BATCH_CHECKPOINT_MAX_POINTS
        || observed.point_bindings.len() != observed.vector_bindings.len()
    {
        bail!("observed workspace coverage batch has invalid bounded cardinality");
    }
    for (point, vector) in observed.point_bindings.iter().zip(&observed.vector_bindings) {
        if point.point_id.trim().is_empty()
            || point.point_id != vector.point_id
            || !canonical_sha256(&point.payload_sha256)
            || !canonical_sha256(&vector.vector_sha256)
        {
            bail!("observed workspace coverage batch binding shape is invalid");
        }
    }
    Ok(())
}

pub(crate) fn sign_workspace_coverage_projection_batch_checkpoint(
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

pub(crate) fn admit_workspace_coverage_projection_batch_checkpoint(
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
    fn high_level_builder_input_refuses_caller_identity_and_binding_substitution() {
        let valid = ObservedWorkspaceCoverageBatchInput {
            claim_id: "claim".into(),
            attempt_id: "attempt".into(),
            plan_id: "plan".into(),
            first_plan_ordinal: 0,
            point_bindings: vec![WorkspaceCoveragePointBinding {
                point_id: "sealed-point".into(),
                payload_sha256: "a".repeat(64),
            }],
            vector_bindings: vec![WorkspaceCoverageVectorBinding {
                point_id: "sealed-point".into(),
                vector_sha256: "b".repeat(64),
            }],
        };
        validate_observed_batch_input(&valid).unwrap();
        for mutation in ["claim", "point", "payload", "vector", "empty"] {
            let mut alien = valid.clone();
            match mutation {
                "claim" => alien.claim_id.clear(),
                "point" => alien.vector_bindings[0].point_id = "substituted".into(),
                "payload" => alien.point_bindings[0].payload_sha256 = "not-a-digest".into(),
                "vector" => alien.vector_bindings[0].vector_sha256 = "not-a-digest".into(),
                _ => {
                    alien.point_bindings.clear();
                    alien.vector_bindings.clear();
                }
            }
            assert!(validate_observed_batch_input(&alien).is_err(), "{mutation}");
        }
    }
    #[test]
    fn authenticated_head_identity_refuses_stale_or_substituted_event() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let event = sample(&k);
        let head = WorkspaceCoverageProjectionBatchCheckpointHeadEntry {
            schema_version: HEAD_SCHEMA.into(),
            claim_id: event.claim_id.clone(),
            claim_epoch: event.claim_epoch,
            checkpoint_id: event.checkpoint_id.clone(),
            checkpoint_envelope_digest: format!("sha256-{}", "9".repeat(64)),
            sequence: event.sequence,
            cumulative_point_count: event.cumulative_point_count,
            total_point_count: event.total_point_count,
            collection_name: event.collection_name.clone(),
        };
        validate_current_head_event_identity(&event, &head).unwrap();
        for mutation in ["sequence", "count", "claim", "checkpoint", "collection"] {
            let mut stale = head.clone();
            match mutation {
                "sequence" => stale.sequence += 1,
                "count" => stale.cumulative_point_count += 1,
                "claim" => stale.claim_id.push('x'),
                "checkpoint" => stale.checkpoint_id.push('x'),
                _ => stale.collection_name.push('x'),
            }
            assert!(validate_current_head_event_identity(&event, &stale).is_err(), "{mutation}");
        }
    }
    #[test]
    fn reconstructed_chain_rejects_corrupted_earlier_batch_and_overlap() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let first = sample(&k);
        let mut second = first.clone();
        second.checkpoint_id = uuid::Uuid::new_v4().to_string();
        second.sequence = 2;
        second.batch_ordinal = 1;
        second.first_plan_ordinal = 1;
        second.cumulative_point_count = 2;
        second.point_ids = vec!["point-2".into()];
        second.point_bindings[0].point_id = "point-2".into();
        second.vector_bindings[0].point_id = "point-2".into();
        second.predecessor_checkpoint_id = Some(first.checkpoint_id.clone());
        second.predecessor_checkpoint_envelope_digest = Some(format!("sha256-{}", "8".repeat(64)));
        let chain = vec![
            WorkspaceCoverageProjectionBatchCheckpointAdmission {
                checkpoint: first.clone(),
                checkpoint_envelope_digest: format!("sha256-{}", "8".repeat(64)),
            },
            WorkspaceCoverageProjectionBatchCheckpointAdmission {
                checkpoint: second.clone(),
                checkpoint_envelope_digest: format!("sha256-{}", "9".repeat(64)),
            },
        ];
        let head = WorkspaceCoverageProjectionBatchCheckpointHeadEntry {
            schema_version: HEAD_SCHEMA.into(),
            claim_id: second.claim_id.clone(),
            claim_epoch: second.claim_epoch,
            checkpoint_id: second.checkpoint_id.clone(),
            checkpoint_envelope_digest: chain[1].checkpoint_envelope_digest.clone(),
            sequence: 2,
            cumulative_point_count: 2,
            total_point_count: 2,
            collection_name: second.collection_name.clone(),
        };
        validate_reconstructed_chain(&chain, &head).unwrap();
        let mut corrupt = chain.clone();
        corrupt[0].checkpoint.sequence = 2;
        assert!(validate_reconstructed_chain(&corrupt, &head).is_err());
        let mut overlap = chain.clone();
        overlap[1].checkpoint.point_ids[0] = overlap[0].checkpoint.point_ids[0].clone();
        assert!(validate_reconstructed_chain(&overlap, &head).is_err());
    }
    #[test]
    fn current_body_projection_rejects_historical_or_terminal_checkpoint_authority() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let event = sample(&k);
        let current = CurrentProjectionBinding {
            claim_status: "running",
            claim_id: &event.claim_id,
            claim_epoch: event.claim_epoch,
            attempt_status: "running",
            attempt_id: &event.attempt_id,
            claim_plan_id: &event.plan_id,
            attempt_plan_id: &event.plan_id,
            plan_id: &event.plan_id,
            claim_obligation_id: &event.obligation_id,
            obligation_id: &event.obligation_id,
            plan_obligation_id: &event.obligation_id,
            claim_body_observation_id: &event.body_observation_id,
            obligation_body_observation_id: &event.body_observation_id,
            claim_body_generation: event.body_generation,
            obligation_body_generation: event.body_generation,
            claim_manifest_root: &event.manifest_root_sha256,
            obligation_manifest_root: &event.manifest_root_sha256,
        };
        validate_current_projection_binding(&event, current).unwrap();
        for mutation in [
            "prior-generation",
            "terminal-claim",
            "terminal-attempt",
            "replaced-claim",
            "changed-plan",
            "changed-obligation",
            "changed-manifest",
        ] {
            let mut stale = current;
            match mutation {
                "prior-generation" => stale.obligation_body_generation += 1,
                "terminal-claim" => stale.claim_status = "succeeded",
                "terminal-attempt" => stale.attempt_status = "failed",
                "replaced-claim" => stale.claim_id = "replacement-claim",
                "changed-plan" => stale.plan_id = "replacement-plan",
                "changed-obligation" => stale.obligation_id = "replacement-obligation",
                _ => stale.obligation_manifest_root = "replacement-manifest",
            }
            assert!(validate_current_projection_binding(&event, stale).is_err(), "{mutation}");
        }
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
