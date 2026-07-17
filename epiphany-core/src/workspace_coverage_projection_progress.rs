use crate::workspace_coverage_process_documents::authenticate_workspace_coverage_managed_process_launch_with_envelope_digest;
use crate::workspace_coverage_projection_batch_checkpoint::{
    WorkspaceCoverageProjectionBatchCheckpointAdmission, load_authenticated_current_checkpoint,
};
use crate::workspace_coverage_projector::{
    ATTEMPT_TYPE, CHUNKER_ID, CLAIM_TYPE, OBLIGATION_TYPE, PLAN_TYPE, PROJECTION_SCHEMA,
    WorkspaceCoverageProjectionAttempt, WorkspaceCoverageProjectionClaim,
    exact_obligation_body_authority, validate_claim_attempt_link, validate_projection_attempt,
    validate_projection_claim, workspace_coverage_claim_key,
};
use crate::workspace_retrieval_coverage::{
    WorkspaceCoverageObligation, WorkspaceCoverageProjectionPlan,
    validate_workspace_coverage_projection_plan,
};
use crate::{
    BODY_HEAD_KEY, BODY_HEAD_TYPE, HostIncarnationIdentityEntry, RepositoryBodyHead,
    WORKSPACE_COVERAGE_MAXIMUM_FILE_BYTES, WorkspaceCoverageManagedProcessLaunchEntry,
    WorkspaceCoveragePolicy, authenticate_workspace_coverage_managed_process_launch,
};
use anyhow::{Context, Result, anyhow, bail};
use chrono::DateTime;
use cultcache_rs::{
    CacheBackingStore, CultCacheEnvelope, DatabaseEntry, OwnedRedbMessagePackBackingStore,
    SingleFileMessagePackBackingStore,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::Serialize;
use sha2::Digest;
use std::path::Path;

pub const WORKSPACE_COVERAGE_PROJECTION_PROGRESS_TYPE: &str =
    "epiphany.workspace_coverage.projection_progress";
pub const WORKSPACE_COVERAGE_PROJECTION_PROGRESS_SCHEMA_VERSION: &str =
    "epiphany.workspace_coverage.projection_progress.v0";
const PROGRESS_DOMAIN: &[u8] = b"epiphany.workspace-coverage.projection-progress.v0\0";

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.workspace_coverage.projection_progress",
    schema = "WorkspaceCoverageProjectionProgressEntry"
)]
pub struct WorkspaceCoverageProjectionProgressEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub progress_id: String,
    #[cultcache(key = 2)]
    pub runtime_id: String,
    #[cultcache(key = 3)]
    pub launch_id: String,
    #[cultcache(key = 4)]
    pub launch_envelope_digest: String,
    #[cultcache(key = 5)]
    pub provider_incarnation_id: String,
    #[cultcache(key = 6)]
    pub provider_public_key: Vec<u8>,
    #[cultcache(key = 7)]
    pub claim_id: String,
    #[cultcache(key = 8)]
    pub claim_epoch: u64,
    #[cultcache(key = 9)]
    pub attempt_id: String,
    #[cultcache(key = 10)]
    pub plan_id: String,
    #[cultcache(key = 11)]
    pub body_observation_id: String,
    #[cultcache(key = 12)]
    pub body_generation: u64,
    #[cultcache(key = 13)]
    pub embedding_artifact_identity: String,
    #[cultcache(key = 14)]
    pub embedding_dimensions: u32,
    #[cultcache(key = 15)]
    pub phase: String,
    #[cultcache(key = 16)]
    pub status: String,
    #[cultcache(key = 17)]
    pub completed_units: u64,
    #[cultcache(key = 18)]
    pub total_units: u64,
    #[cultcache(key = 19)]
    pub unit_kind: String,
    #[cultcache(key = 20)]
    pub operation_started_at_utc: String,
    #[cultcache(key = 21)]
    /// Provider-observed operation timeout. This is telemetry, not Body policy
    /// or deployment authority; consumers must not use it to authorize work.
    pub operation_timeout_ms: u64,
    #[cultcache(key = 22)]
    pub observed_at_utc: String,
    #[cultcache(key = 23)]
    pub last_advanced_at_utc: String,
    #[cultcache(key = 24)]
    pub sequence: u64,
    /// Reference to a separately authenticated durable batch checkpoint. This
    /// event chain binds the reference; it does not prove checkpoint storage,
    /// contents, durability, or readiness.
    #[cultcache(key = 25, default)]
    pub checkpoint_id: Option<String>,
    #[cultcache(key = 26, default)]
    pub checkpoint_binding_sha256: Option<String>,
    #[cultcache(key = 27)]
    pub provider_signature: Vec<u8>,
    #[cultcache(key = 28)]
    pub signature_algorithm: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceCoverageAdvancingAuthority {
    pub progress_id: String,
    pub checkpoint_id: String,
    pub checkpoint_binding_sha256: String,
    pub claim_id: String,
    pub claim_epoch: u64,
    pub plan_id: String,
    pub completed_units: u64,
    pub total_units: u64,
    pub last_advanced_at_utc: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressStatement<'a> {
    schema_version: &'a str,
    progress_id: &'a str,
    runtime_id: &'a str,
    launch_id: &'a str,
    launch_envelope_digest: &'a str,
    provider_incarnation_id: &'a str,
    provider_public_key: &'a [u8],
    claim_id: &'a str,
    claim_epoch: u64,
    attempt_id: &'a str,
    plan_id: &'a str,
    body_observation_id: &'a str,
    body_generation: u64,
    embedding_artifact_identity: &'a str,
    embedding_dimensions: u32,
    phase: &'a str,
    status: &'a str,
    completed_units: u64,
    total_units: u64,
    unit_kind: &'a str,
    operation_started_at_utc: &'a str,
    operation_timeout_ms: u64,
    observed_at_utc: &'a str,
    last_advanced_at_utc: &'a str,
    sequence: u64,
    checkpoint_id: &'a Option<String>,
    checkpoint_binding_sha256: &'a Option<String>,
    signature_algorithm: &'a str,
}

pub(crate) fn workspace_coverage_projection_progress_statement(
    entry: &WorkspaceCoverageProjectionProgressEntry,
) -> Result<Vec<u8>> {
    let mut unsigned = entry.clone();
    unsigned.provider_signature.clear();
    validate_shape(&unsigned, false)?;
    Ok(rmp_serde::to_vec_named(&ProgressStatement {
        schema_version: &entry.schema_version,
        progress_id: &entry.progress_id,
        runtime_id: &entry.runtime_id,
        launch_id: &entry.launch_id,
        launch_envelope_digest: &entry.launch_envelope_digest,
        provider_incarnation_id: &entry.provider_incarnation_id,
        provider_public_key: &entry.provider_public_key,
        claim_id: &entry.claim_id,
        claim_epoch: entry.claim_epoch,
        attempt_id: &entry.attempt_id,
        plan_id: &entry.plan_id,
        body_observation_id: &entry.body_observation_id,
        body_generation: entry.body_generation,
        embedding_artifact_identity: &entry.embedding_artifact_identity,
        embedding_dimensions: entry.embedding_dimensions,
        phase: &entry.phase,
        status: &entry.status,
        completed_units: entry.completed_units,
        total_units: entry.total_units,
        unit_kind: &entry.unit_kind,
        operation_started_at_utc: &entry.operation_started_at_utc,
        operation_timeout_ms: entry.operation_timeout_ms,
        observed_at_utc: &entry.observed_at_utc,
        last_advanced_at_utc: &entry.last_advanced_at_utc,
        sequence: entry.sequence,
        checkpoint_id: &entry.checkpoint_id,
        checkpoint_binding_sha256: &entry.checkpoint_binding_sha256,
        signature_algorithm: &entry.signature_algorithm,
    })?)
}

pub(crate) fn sign_workspace_coverage_projection_progress(
    entry: &mut WorkspaceCoverageProjectionProgressEntry,
    key: &SigningKey,
) -> Result<()> {
    if key.verifying_key().to_bytes().as_slice() != entry.provider_public_key.as_slice() {
        bail!("provider signing key disagrees with progress public key");
    }
    entry.provider_signature.clear();
    let statement = workspace_coverage_projection_progress_statement(entry)?;
    entry.provider_signature = key.sign(&provider_message(&statement)).to_bytes().to_vec();
    Ok(())
}

pub(crate) fn publish_workspace_coverage_progress_genesis(
    body_store: &Path,
    coverage_store: &OwnedRedbMessagePackBackingStore,
    local_verse_store: impl AsRef<Path>,
    runtime_id: &str,
    trusted_host: &HostIncarnationIdentityEntry,
    provider_signing_key: &SigningKey,
    operation_timeout_ms: u64,
) -> Result<WorkspaceCoverageProjectionProgressEntry> {
    if operation_timeout_ms == 0 {
        bail!("workspace coverage progress operation timeout must be positive");
    }
    let opening = coverage_store.pull_all()?;
    let body_opening = SingleFileMessagePackBackingStore::new(body_store).pull_all()?;
    let claim_env = find_current_claim(&opening, &body_opening)?;
    let claim: WorkspaceCoverageProjectionClaim = rmp_serde::from_slice(&claim_env.payload)?;
    let attempt_env = find_authority(&opening, ATTEMPT_TYPE, &claim.attempt_id)?;
    let attempt: WorkspaceCoverageProjectionAttempt = rmp_serde::from_slice(&attempt_env.payload)?;
    validate_projection_claim(&claim)?;
    validate_projection_attempt(&attempt)?;
    validate_claim_attempt_link(&claim, &attempt)?;
    if claim.status != "running" {
        bail!("workspace coverage progress genesis requires running claim authority");
    }
    let obligation_env = find_authority(&opening, OBLIGATION_TYPE, &claim.obligation_id)?;
    let plan_env = find_authority(&opening, PLAN_TYPE, &claim.plan_id)?;
    let obligation: WorkspaceCoverageObligation = rmp_serde::from_slice(&obligation_env.payload)?;
    let plan: WorkspaceCoverageProjectionPlan = rmp_serde::from_slice(&plan_env.payload)?;
    validate_workspace_coverage_projection_plan(&obligation, &plan)?;
    let body_authority = exact_obligation_body_authority(&body_opening, &obligation)?;
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
        || obligation.runtime_id != runtime_id
    {
        bail!("workspace coverage progress genesis signer/launch/Body authority disagrees");
    }
    if load_latest_workspace_coverage_projection_progress(
        coverage_store,
        &launch.launch_id,
        &claim.claim_id,
    )?
    .is_some()
    {
        bail!("workspace coverage progress genesis already exists");
    }
    let now = chrono::Utc::now().to_rfc3339();
    let mut entry = WorkspaceCoverageProjectionProgressEntry {
        schema_version: WORKSPACE_COVERAGE_PROJECTION_PROGRESS_SCHEMA_VERSION.into(),
        progress_id: uuid::Uuid::new_v4().to_string(),
        runtime_id: runtime_id.into(),
        launch_id: launch.launch_id.clone(),
        launch_envelope_digest: launch_digest,
        provider_incarnation_id: launch.provider_incarnation_id.clone(),
        provider_public_key: launch.provider_public_key.clone(),
        claim_id: claim.claim_id,
        claim_epoch: claim.claim_epoch,
        attempt_id: attempt.attempt_id,
        plan_id: plan.plan_id,
        body_observation_id: obligation.body_observation_id.clone(),
        body_generation: obligation.body_generation,
        embedding_artifact_identity: plan.embedding_model,
        embedding_dimensions: plan.vector_dimensions,
        phase: "preparing".into(),
        status: "warming".into(),
        completed_units: 0,
        total_units: plan.expected_point_count,
        unit_kind: "points".into(),
        operation_started_at_utc: now.clone(),
        operation_timeout_ms,
        observed_at_utc: now.clone(),
        last_advanced_at_utc: now,
        sequence: 1,
        checkpoint_id: None,
        checkpoint_binding_sha256: None,
        provider_signature: Vec::new(),
        signature_algorithm: "ed25519".into(),
    };
    sign_workspace_coverage_projection_progress(&mut entry, provider_signing_key)?;
    let published = write_workspace_coverage_projection_progress(coverage_store, entry)?;
    let closing_body = SingleFileMessagePackBackingStore::new(body_store).pull_all()?;
    for expected in body_authority {
        if closing_body
            .iter()
            .find(|entry| entry.r#type == expected.r#type && entry.key == expected.key)
            != Some(&expected)
        {
            bail!("Repository Body changed during workspace coverage progress genesis");
        }
    }
    Ok(published)
}

pub(crate) fn prepare_workspace_coverage_progress_for_checkpoint(
    coverage_store: &OwnedRedbMessagePackBackingStore,
    provider_signing_key: &SigningKey,
    admission: &WorkspaceCoverageProjectionBatchCheckpointAdmission,
) -> Result<(
    Vec<CultCacheEnvelope>,
    Vec<CultCacheEnvelope>,
    WorkspaceCoverageProjectionProgressEntry,
)> {
    let checkpoint = &admission.checkpoint;
    let prior_key = latest_key(&checkpoint.managed_process_launch_id, &checkpoint.claim_id);
    let opening = coverage_store.pull_all()?;
    let prior_env = opening
        .iter()
        .find(|env| {
            env.r#type == WORKSPACE_COVERAGE_PROJECTION_PROGRESS_TYPE && env.key == prior_key
        })
        .cloned()
        .ok_or_else(|| anyhow!("checkpoint progress requires canonical genesis"))?;
    let prior: WorkspaceCoverageProjectionProgressEntry = decode(&prior_env)?;
    let expected_sequence = checkpoint
        .sequence
        .checked_add(1)
        .ok_or_else(|| anyhow!("checkpoint sequence exhausted"))?;
    if prior.sequence != checkpoint.sequence
        || checkpoint.cumulative_point_count <= prior.completed_units
    {
        bail!("checkpoint progress does not durably advance completed units");
    }
    let now = chrono::Utc::now().to_rfc3339();
    let mut entry = WorkspaceCoverageProjectionProgressEntry {
        schema_version: WORKSPACE_COVERAGE_PROJECTION_PROGRESS_SCHEMA_VERSION.into(),
        progress_id: uuid::Uuid::new_v4().to_string(),
        runtime_id: checkpoint.runtime_id.clone(),
        launch_id: checkpoint.managed_process_launch_id.clone(),
        launch_envelope_digest: checkpoint.managed_process_launch_envelope_digest.clone(),
        provider_incarnation_id: checkpoint.provider_incarnation_id.clone(),
        provider_public_key: checkpoint.provider_public_key.clone(),
        claim_id: checkpoint.claim_id.clone(),
        claim_epoch: checkpoint.claim_epoch,
        attempt_id: checkpoint.attempt_id.clone(),
        plan_id: checkpoint.plan_id.clone(),
        body_observation_id: checkpoint.body_observation_id.clone(),
        body_generation: checkpoint.body_generation,
        embedding_artifact_identity: checkpoint.embedding_artifact_identity.clone(),
        embedding_dimensions: checkpoint.vector_dimensions,
        phase: "projecting".into(),
        status: "warming".into(),
        completed_units: checkpoint.cumulative_point_count,
        total_units: checkpoint.total_point_count,
        unit_kind: "points".into(),
        operation_started_at_utc: prior.operation_started_at_utc.clone(),
        operation_timeout_ms: prior.operation_timeout_ms,
        observed_at_utc: now.clone(),
        last_advanced_at_utc: now,
        sequence: expected_sequence,
        checkpoint_id: Some(checkpoint.checkpoint_id.clone()),
        checkpoint_binding_sha256: Some(admission.checkpoint_envelope_digest.clone()),
        provider_signature: Vec::new(),
        signature_algorithm: "ed25519".into(),
    };
    sign_workspace_coverage_projection_progress(&mut entry, provider_signing_key)?;
    validate_shape(&entry, true)?;
    validate_transition(&prior, &entry)?;
    let event_key = progress_key(&entry.progress_id);
    let latest_key = latest_key(&entry.launch_id, &entry.claim_id);
    let expected = vec![prior_env];
    let replacements = vec![
        progress_envelope(&event_key, &entry)?,
        progress_envelope(&latest_key, &entry)?,
    ];
    Ok((expected, replacements, entry))
}

fn checkpoint_progress_alignment(
    progress: &WorkspaceCoverageProjectionProgressEntry,
    admission: &WorkspaceCoverageProjectionBatchCheckpointAdmission,
) -> Result<()> {
    let checkpoint = &admission.checkpoint;
    checkpoint_progress_binding_alignment(
        progress,
        checkpoint.sequence,
        &checkpoint.checkpoint_id,
        &admission.checkpoint_envelope_digest,
        checkpoint.cumulative_point_count,
    )
}

fn checkpoint_progress_binding_alignment(
    progress: &WorkspaceCoverageProjectionProgressEntry,
    checkpoint_sequence: u64,
    checkpoint_id: &str,
    checkpoint_digest: &str,
    cumulative_point_count: u64,
) -> Result<()> {
    let expected = checkpoint_sequence
        .checked_add(1)
        .ok_or_else(|| anyhow!("checkpoint sequence exhausted"))?;
    if progress.sequence == expected
        && progress.checkpoint_id.as_deref() == Some(checkpoint_id)
        && progress.checkpoint_binding_sha256.as_deref() == Some(checkpoint_digest)
        && progress.completed_units == cumulative_point_count
    {
        return Ok(());
    }
    bail!("progress does not map exactly to the authenticated current checkpoint")
}

pub(crate) fn write_workspace_coverage_projection_progress(
    store: &OwnedRedbMessagePackBackingStore,
    entry: WorkspaceCoverageProjectionProgressEntry,
) -> Result<WorkspaceCoverageProjectionProgressEntry> {
    validate_shape(&entry, true)?;
    verify_signature(&entry)?;
    let opening = store.pull_all()?;
    let event_key = progress_key(&entry.progress_id);
    if let Some(existing_env) = opening.iter().find(|env| {
        env.r#type == WORKSPACE_COVERAGE_PROJECTION_PROGRESS_TYPE && env.key == event_key
    }) {
        let existing: WorkspaceCoverageProjectionProgressEntry = decode(existing_env)?;
        if existing_event_is_idempotent(&existing, &entry)? {
            return Ok(existing);
        }
    }
    let latest_key = latest_key(&entry.launch_id, &entry.claim_id);
    let latest = opening
        .iter()
        .find(|env| {
            env.r#type == WORKSPACE_COVERAGE_PROJECTION_PROGRESS_TYPE && env.key == latest_key
        })
        .cloned();
    let mut expected = Vec::new();
    if let Some(envelope) = latest.as_ref() {
        let prior: WorkspaceCoverageProjectionProgressEntry =
            rmp_serde::from_slice(&envelope.payload)?;
        validate_transition(&prior, &entry)?;
        expected.push(envelope.clone());
    } else if entry.sequence != 1 {
        bail!("first workspace coverage progress sequence must be one");
    }
    let replacements = vec![
        progress_envelope(&event_key, &entry)?,
        progress_envelope(&latest_key, &entry)?,
    ];
    if !store.compare_and_swap_batch(&expected, replacements)? {
        bail!("workspace coverage progress lost exact launch/claim latest compare-and-swap");
    }
    Ok(entry)
}

pub fn load_workspace_coverage_projection_progress(
    store: &OwnedRedbMessagePackBackingStore,
    progress_id: &str,
) -> Result<Option<WorkspaceCoverageProjectionProgressEntry>> {
    require("progress id", progress_id)?;
    load_progress(store, &progress_key(progress_id))
}

pub fn load_latest_workspace_coverage_projection_progress(
    store: &OwnedRedbMessagePackBackingStore,
    launch_id: &str,
    claim_id: &str,
) -> Result<Option<WorkspaceCoverageProjectionProgressEntry>> {
    require("launch id", launch_id)?;
    require("claim id", claim_id)?;
    load_progress(store, &latest_key(launch_id, claim_id))
}

pub fn authenticate_workspace_coverage_projection_progress(
    store: &OwnedRedbMessagePackBackingStore,
    local_verse_store: impl AsRef<Path>,
    runtime_id: &str,
    progress_id: &str,
    host: &HostIncarnationIdentityEntry,
) -> Result<WorkspaceCoverageProjectionProgressEntry> {
    let entry = load_workspace_coverage_projection_progress(store, progress_id)?
        .ok_or_else(|| anyhow!("workspace coverage progress is absent"))?;
    let launch = authenticate_workspace_coverage_managed_process_launch(
        local_verse_store.as_ref(),
        runtime_id,
        &entry.launch_id,
        host,
    )?;
    let (_, digest) = authenticate_workspace_coverage_managed_process_launch_with_envelope_digest(
        local_verse_store.as_ref(),
        runtime_id,
        &entry.launch_id,
        host,
    )?;
    authenticate_against_launch(&entry, &launch, &digest)?;
    Ok(entry)
}

/// Authenticates durable, current, nonterminal workspace advancement.
/// Provider telemetry timeouts and heartbeat timestamps are intentionally not
/// consulted here. The supervisor separately joins this evidence to the exact
/// live managed process and applies its own no-advance lease.
pub fn authenticate_current_workspace_coverage_advancement(
    body_store: &Path,
    coverage_store: &OwnedRedbMessagePackBackingStore,
    local_verse_store: impl AsRef<Path>,
    runtime_id: &str,
    launch_id: &str,
    host: &HostIncarnationIdentityEntry,
    observed_at: chrono::DateTime<chrono::Utc>,
    no_advance_lease: chrono::Duration,
) -> Result<Option<WorkspaceCoverageAdvancingAuthority>> {
    if no_advance_lease <= chrono::Duration::zero() {
        bail!("workspace coverage no-advance lease must be positive");
    }
    let opening = coverage_store.pull_all()?;
    let body_opening = SingleFileMessagePackBackingStore::new(body_store).pull_all()?;
    let claim: WorkspaceCoverageProjectionClaim =
        decode(find_current_claim(&opening, &body_opening)?)?;
    validate_projection_claim(&claim)?;
    let attempt: WorkspaceCoverageProjectionAttempt =
        decode(find_authority(&opening, ATTEMPT_TYPE, &claim.attempt_id)?)?;
    validate_projection_attempt(&attempt)?;
    validate_claim_attempt_link(&claim, &attempt)?;
    if claim.status != "running" || attempt.status != "running" {
        return Ok(None);
    }
    let obligation: WorkspaceCoverageObligation = decode(find_authority(
        &opening,
        OBLIGATION_TYPE,
        &claim.obligation_id,
    )?)?;
    let plan: WorkspaceCoverageProjectionPlan =
        decode(find_authority(&opening, PLAN_TYPE, &claim.plan_id)?)?;
    validate_workspace_coverage_projection_plan(&obligation, &plan)?;
    let body_authority = exact_obligation_body_authority(&body_opening, &obligation)?;
    let policy =
        WorkspaceCoveragePolicy::bounded_regular_files_v0(WORKSPACE_COVERAGE_MAXIMUM_FILE_BYTES)?;
    let policy_sha256 = format!(
        "{:x}",
        sha2::Sha256::digest(rmp_serde::to_vec_named(&policy).map_err(|error| anyhow!(error))?)
    );
    if obligation.policy_id != policy.policy_id
        || obligation.policy_sha256 != policy_sha256
        || plan.projection_schema_version != PROJECTION_SCHEMA
        || plan.chunker_id != CHUNKER_ID
    {
        return Ok(None);
    }

    let Some(latest) = load_latest_workspace_coverage_projection_progress(
        coverage_store,
        launch_id,
        &claim.claim_id,
    )?
    else {
        return Ok(None);
    };
    let progress = authenticate_workspace_coverage_projection_progress(
        coverage_store,
        local_verse_store.as_ref(),
        runtime_id,
        &latest.progress_id,
        host,
    )?;
    if progress.status != "warming"
        || progress.completed_units == 0
        || progress.launch_id != launch_id
        || progress.claim_id != claim.claim_id
        || progress.claim_epoch != claim.claim_epoch
        || progress.attempt_id != attempt.attempt_id
        || progress.plan_id != plan.plan_id
        || progress.body_observation_id != obligation.body_observation_id
        || progress.body_generation != obligation.body_generation
    {
        return Ok(None);
    }
    let Some(checkpoint) = load_authenticated_current_checkpoint(
        body_store,
        coverage_store,
        local_verse_store.as_ref(),
        host,
        &claim.claim_id,
        claim.claim_epoch,
    )?
    else {
        return Ok(None);
    };
    checkpoint_progress_alignment(&progress, &checkpoint)?;
    if !progress_is_within_no_advance_lease(
        &progress.last_advanced_at_utc,
        observed_at,
        no_advance_lease,
    )? {
        return Ok(None);
    }
    let authority = WorkspaceCoverageAdvancingAuthority {
        progress_id: progress.progress_id,
        checkpoint_id: checkpoint.checkpoint.checkpoint_id,
        checkpoint_binding_sha256: checkpoint.checkpoint_envelope_digest,
        claim_id: claim.claim_id,
        claim_epoch: claim.claim_epoch,
        plan_id: plan.plan_id,
        completed_units: progress.completed_units,
        total_units: progress.total_units,
        last_advanced_at_utc: progress.last_advanced_at_utc,
    };
    let closing_body = SingleFileMessagePackBackingStore::new(body_store).pull_all()?;
    for expected in body_authority {
        if closing_body
            .iter()
            .find(|row| row.r#type == expected.r#type && row.key == expected.key)
            != Some(&expected)
        {
            bail!("Repository Body changed while authenticating workspace advancement");
        }
    }
    Ok(Some(authority))
}

fn progress_is_within_no_advance_lease(
    last_advanced_at_utc: &str,
    observed_at: chrono::DateTime<chrono::Utc>,
    no_advance_lease: chrono::Duration,
) -> Result<bool> {
    let advanced = DateTime::parse_from_rfc3339(last_advanced_at_utc)?.with_timezone(&chrono::Utc);
    let age = observed_at.signed_duration_since(advanced);
    Ok(age >= chrono::Duration::zero() && age <= no_advance_lease)
}

fn authenticate_against_launch(
    entry: &WorkspaceCoverageProjectionProgressEntry,
    launch: &WorkspaceCoverageManagedProcessLaunchEntry,
    launch_digest: &str,
) -> Result<()> {
    validate_shape(entry, true)?;
    if entry.runtime_id != launch.runtime_id
        || entry.launch_id != launch.launch_id
        || entry.launch_envelope_digest != launch_digest
        || entry.provider_incarnation_id != launch.provider_incarnation_id
        || entry.provider_public_key != launch.provider_public_key
    {
        bail!("workspace coverage progress disagrees with exact managed launch");
    }
    verify_signature(entry)
}

fn validate_shape(entry: &WorkspaceCoverageProjectionProgressEntry, signed: bool) -> Result<()> {
    if entry.schema_version != WORKSPACE_COVERAGE_PROJECTION_PROGRESS_SCHEMA_VERSION {
        bail!("workspace coverage progress schema is invalid");
    }
    for (name, value) in [
        ("progress id", entry.progress_id.as_str()),
        ("runtime id", &entry.runtime_id),
        ("launch id", &entry.launch_id),
        ("launch digest", &entry.launch_envelope_digest),
        ("provider incarnation", &entry.provider_incarnation_id),
        ("claim id", &entry.claim_id),
        ("attempt id", &entry.attempt_id),
        ("plan id", &entry.plan_id),
        ("Body observation id", &entry.body_observation_id),
        (
            "embedding artifact identity",
            &entry.embedding_artifact_identity,
        ),
        ("unit kind", &entry.unit_kind),
    ] {
        require(name, value)?;
    }
    uuid::Uuid::parse_str(&entry.progress_id).context("progress id must be UUID")?;
    uuid::Uuid::parse_str(&entry.launch_id).context("launch id must be UUID")?;
    uuid::Uuid::parse_str(&entry.provider_incarnation_id)
        .context("provider incarnation must be UUID")?;
    if entry.claim_epoch == 0
        || entry.body_generation == 0
        || entry.embedding_dimensions == 0
        || entry.operation_timeout_ms == 0
        || entry.sequence == 0
        || entry.total_units == 0
        || entry.completed_units > entry.total_units
    {
        bail!("workspace coverage progress numeric bounds are invalid");
    }
    if !matches!(
        entry.phase.as_str(),
        "preparing" | "projecting" | "verifying" | "committing"
    ) || !matches!(entry.status.as_str(), "warming" | "failed" | "complete")
    {
        bail!("workspace coverage progress phase or status is invalid");
    }
    if entry.status == "complete"
        && (entry.phase != "committing" || entry.completed_units != entry.total_units)
    {
        bail!("completed workspace coverage progress lacks complete committed work");
    }
    match (&entry.checkpoint_id, &entry.checkpoint_binding_sha256) {
        (Some(id), Some(binding)) => {
            require("progress checkpoint id", id)?;
            if !binding.starts_with("sha256-")
                || binding.len() != 71
                || !binding[7..]
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            {
                bail!("workspace coverage progress checkpoint binding is not canonical SHA-256");
            }
        }
        (None, None)
            if entry.sequence == 1
                && entry.phase == "preparing"
                && entry.status == "warming"
                && entry.completed_units == 0 => {}
        _ => bail!("workspace coverage progress lacks exact durable checkpoint evidence"),
    }
    let operation = DateTime::parse_from_rfc3339(&entry.operation_started_at_utc)?;
    let observed = DateTime::parse_from_rfc3339(&entry.observed_at_utc)?;
    let advanced = DateTime::parse_from_rfc3339(&entry.last_advanced_at_utc)?;
    if operation > observed || advanced > observed {
        bail!("workspace coverage progress times are causally invalid");
    }
    if entry.sequence == 1
        && (entry.phase != "preparing"
            || entry.status != "warming"
            || entry.completed_units != 0
            || entry.checkpoint_id.is_some()
            || entry.checkpoint_binding_sha256.is_some()
            || operation != observed
            || advanced != observed)
    {
        bail!("workspace coverage progress genesis is not canonical");
    }
    if entry.provider_public_key.len() != 32
        || entry.signature_algorithm != "ed25519"
        || (signed && entry.provider_signature.len() != 64)
        || (!signed && !entry.provider_signature.is_empty())
    {
        bail!("workspace coverage progress signature material is invalid");
    }
    Ok(())
}

fn validate_transition(
    prior: &WorkspaceCoverageProjectionProgressEntry,
    next: &WorkspaceCoverageProjectionProgressEntry,
) -> Result<()> {
    validate_shape(prior, true)?;
    validate_shape(next, true)?;
    verify_signature(prior)?;
    verify_signature(next)?;
    authenticate_same_authority(prior, next)?;
    if next.sequence
        != prior
            .sequence
            .checked_add(1)
            .ok_or_else(|| anyhow!("progress sequence exhausted"))?
    {
        bail!("workspace coverage progress sequence must advance exactly once");
    }
    let prior_observed = DateTime::parse_from_rfc3339(&prior.observed_at_utc)?;
    let next_observed = DateTime::parse_from_rfc3339(&next.observed_at_utc)?;
    let prior_advanced = DateTime::parse_from_rfc3339(&prior.last_advanced_at_utc)?;
    let next_advanced = DateTime::parse_from_rfc3339(&next.last_advanced_at_utc)?;
    if next_observed <= prior_observed
        || next.completed_units < prior.completed_units
        || next_advanced < prior_advanced
        || (next.completed_units > prior.completed_units) != (next_advanced > prior_advanced)
    {
        bail!("workspace coverage progress time or completion regressed");
    }
    if next.completed_units > prior.completed_units
        && (next.checkpoint_id == prior.checkpoint_id
            || next.checkpoint_binding_sha256 == prior.checkpoint_binding_sha256)
    {
        bail!("workspace coverage advancement must name a new durable checkpoint");
    }
    if next.completed_units == prior.completed_units {
        if next.checkpoint_id != prior.checkpoint_id
            || next.checkpoint_binding_sha256 != prior.checkpoint_binding_sha256
        {
            bail!("workspace coverage checkpoint changed without durable advancement");
        }
        if next.status == "warming" {
            bail!("nonterminal workspace coverage progress must advance durable units");
        }
    }
    let prior_phase = phase_rank(&prior.phase)?;
    let next_phase = phase_rank(&next.phase)?;
    if next_phase < prior_phase || next_phase > prior_phase + 1 {
        bail!("workspace coverage progress phase transition is invalid");
    }
    match (prior.status.as_str(), next.status.as_str()) {
        ("warming", "warming" | "failed" | "complete") => {}
        ("failed" | "complete", _) => {
            bail!("terminal workspace coverage progress cannot advance")
        }
        _ => bail!("workspace coverage progress status transition is invalid"),
    }
    Ok(())
}

fn existing_event_is_idempotent(
    existing: &WorkspaceCoverageProjectionProgressEntry,
    candidate: &WorkspaceCoverageProjectionProgressEntry,
) -> Result<bool> {
    if existing == candidate {
        return Ok(true);
    }
    bail!("workspace coverage progress identity collision")
}

fn authenticate_same_authority(
    a: &WorkspaceCoverageProjectionProgressEntry,
    b: &WorkspaceCoverageProjectionProgressEntry,
) -> Result<()> {
    if a.runtime_id != b.runtime_id
        || a.launch_id != b.launch_id
        || a.launch_envelope_digest != b.launch_envelope_digest
        || a.provider_incarnation_id != b.provider_incarnation_id
        || a.provider_public_key != b.provider_public_key
        || a.claim_id != b.claim_id
        || a.claim_epoch != b.claim_epoch
        || a.attempt_id != b.attempt_id
        || a.plan_id != b.plan_id
        || a.body_observation_id != b.body_observation_id
        || a.body_generation != b.body_generation
        || a.embedding_artifact_identity != b.embedding_artifact_identity
        || a.embedding_dimensions != b.embedding_dimensions
        || a.operation_started_at_utc != b.operation_started_at_utc
        || a.total_units != b.total_units
        || a.unit_kind != b.unit_kind
    {
        bail!("workspace coverage progress authority or immutable plan identity changed");
    }
    Ok(())
}

fn phase_rank(value: &str) -> Result<u8> {
    Ok(match value {
        "preparing" => 0,
        "projecting" => 1,
        "verifying" => 2,
        "committing" => 3,
        _ => bail!("invalid progress phase"),
    })
}
fn verify_signature(entry: &WorkspaceCoverageProjectionProgressEntry) -> Result<()> {
    let key_bytes: [u8; 32] = entry
        .provider_public_key
        .as_slice()
        .try_into()
        .context("workspace coverage progress provider key length")?;
    let sig_bytes: [u8; 64] = entry
        .provider_signature
        .as_slice()
        .try_into()
        .context("workspace coverage progress signature length")?;
    VerifyingKey::from_bytes(&key_bytes)?
        .verify(
            &provider_message(&workspace_coverage_projection_progress_statement(entry)?),
            &Signature::from_bytes(&sig_bytes),
        )
        .context("workspace coverage progress signature is invalid")
}
fn require(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("workspace coverage {name} is empty")
    }
    Ok(())
}
fn find_authority<'a>(
    entries: &'a [CultCacheEnvelope],
    ty: &str,
    key: &str,
) -> Result<&'a CultCacheEnvelope> {
    entries
        .iter()
        .find(|entry| entry.r#type == ty && entry.key == key)
        .ok_or_else(|| anyhow!("workspace coverage progress authority {ty}/{key} is absent"))
}
fn find_current_claim<'a>(
    coverage_opening: &'a [CultCacheEnvelope],
    body_opening: &[CultCacheEnvelope],
) -> Result<&'a CultCacheEnvelope> {
    let head: RepositoryBodyHead =
        decode(find_authority(body_opening, BODY_HEAD_TYPE, BODY_HEAD_KEY)?)?;
    find_authority(
        coverage_opening,
        CLAIM_TYPE,
        &workspace_coverage_claim_key(&head.observation_id, head.generation),
    )
}
fn decode<T: serde::de::DeserializeOwned>(entry: &CultCacheEnvelope) -> Result<T> {
    rmp_serde::from_slice(&entry.payload).context("decoding workspace coverage progress authority")
}
fn progress_key(id: &str) -> String {
    format!("epiphany-local/workspace-coverage/projection-progress/event/{id}")
}
fn progress_envelope(
    key: &str,
    entry: &WorkspaceCoverageProjectionProgressEntry,
) -> Result<CultCacheEnvelope> {
    Ok(CultCacheEnvelope {
        r#type: WORKSPACE_COVERAGE_PROJECTION_PROGRESS_TYPE.into(),
        key: key.into(),
        payload: rmp_serde::to_vec(entry)?,
        stored_at: chrono::Utc::now().to_rfc3339(),
        schema_id: Some(WORKSPACE_COVERAGE_PROJECTION_PROGRESS_TYPE.into()),
    })
}
fn load_progress(
    store: &OwnedRedbMessagePackBackingStore,
    key: &str,
) -> Result<Option<WorkspaceCoverageProjectionProgressEntry>> {
    store
        .pull_all()?
        .iter()
        .find(|env| env.r#type == WORKSPACE_COVERAGE_PROJECTION_PROGRESS_TYPE && env.key == key)
        .map(decode)
        .transpose()
}
fn latest_key(launch: &str, claim: &str) -> String {
    format!("epiphany-local/workspace-coverage/projection-progress/latest/{launch}/{claim}")
}
fn provider_message(statement: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(PROGRESS_DOMAIN.len() + 8 + statement.len());
    out.extend_from_slice(PROGRESS_DOMAIN);
    out.extend_from_slice(&(statement.len() as u64).to_be_bytes());
    out.extend_from_slice(statement);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(key: &SigningKey) -> WorkspaceCoverageProjectionProgressEntry {
        let mut value = WorkspaceCoverageProjectionProgressEntry {
            schema_version: WORKSPACE_COVERAGE_PROJECTION_PROGRESS_SCHEMA_VERSION.into(),
            progress_id: uuid::Uuid::new_v4().to_string(),
            runtime_id: "runtime".into(),
            launch_id: uuid::Uuid::new_v4().to_string(),
            launch_envelope_digest: "sha256-launch".into(),
            provider_incarnation_id: uuid::Uuid::new_v4().to_string(),
            provider_public_key: key.verifying_key().to_bytes().to_vec(),
            claim_id: "claim".into(),
            claim_epoch: 1,
            attempt_id: "attempt".into(),
            plan_id: "plan".into(),
            body_observation_id: "body".into(),
            body_generation: 1,
            embedding_artifact_identity: "ollama:sha256-x".into(),
            embedding_dimensions: 1024,
            phase: "preparing".into(),
            status: "warming".into(),
            completed_units: 0,
            total_units: 100,
            unit_kind: "chunks".into(),
            operation_started_at_utc: "2026-07-16T10:00:00Z".into(),
            operation_timeout_ms: 120_000,
            observed_at_utc: "2026-07-16T10:00:00Z".into(),
            last_advanced_at_utc: "2026-07-16T10:00:00Z".into(),
            sequence: 1,
            checkpoint_id: None,
            checkpoint_binding_sha256: None,
            provider_signature: vec![],
            signature_algorithm: "ed25519".into(),
        };
        sign_workspace_coverage_projection_progress(&mut value, key).unwrap();
        value
    }
    fn advance(
        prior: &WorkspaceCoverageProjectionProgressEntry,
        key: &SigningKey,
    ) -> WorkspaceCoverageProjectionProgressEntry {
        let mut next = prior.clone();
        next.progress_id = uuid::Uuid::new_v4().to_string();
        next.sequence += 1;
        next.completed_units += 1;
        next.phase = "projecting".into();
        next.observed_at_utc = "2026-07-16T10:00:02Z".into();
        next.last_advanced_at_utc = next.observed_at_utc.clone();
        next.checkpoint_id = Some(format!("checkpoint-{}", next.sequence));
        next.checkpoint_binding_sha256 = Some(format!("sha256-{}", "b".repeat(64)));
        sign_workspace_coverage_projection_progress(&mut next, key).unwrap();
        next
    }
    #[test]
    fn valid_signed_transition_is_admitted() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let a = entry(&k);
        let b = advance(&a, &k);
        validate_transition(&a, &b).unwrap();
    }
    #[test]
    fn replay_and_gap_are_refused() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let a = entry(&k);
        let mut b = advance(&a, &k);
        b.sequence = 1;
        assert!(validate_transition(&a, &b).is_err());
        b.sequence = 3;
        assert!(validate_transition(&a, &b).is_err());
    }
    #[test]
    fn substitution_and_mutable_total_are_refused() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let a = entry(&k);
        for mutate in ["plan", "body", "model", "total", "operation"] {
            let mut b = advance(&a, &k);
            match mutate {
                "plan" => b.plan_id.push('x'),
                "body" => b.body_generation += 1,
                "model" => b.embedding_artifact_identity.push('x'),
                "total" => b.total_units += 1,
                _ => b.operation_started_at_utc = "2026-07-16T10:00:01Z".into(),
            };
            assert!(validate_transition(&a, &b).is_err(), "{mutate}");
        }
    }
    #[test]
    fn checkpoint_progress_mapping_rejects_every_split_state() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let genesis = entry(&k);
        assert!(
            checkpoint_progress_binding_alignment(
                &genesis,
                1,
                "checkpoint",
                &format!("sha256-{}", "b".repeat(64)),
                1
            )
            .is_err()
        );
        let current = advance(&genesis, &k);
        let digest = current.checkpoint_binding_sha256.clone().unwrap();
        let id = current.checkpoint_id.clone().unwrap();
        checkpoint_progress_binding_alignment(&current, 1, &id, &digest, 1).unwrap();
        assert!(
            checkpoint_progress_binding_alignment(&current, 2, "next-checkpoint", &digest, 2)
                .is_err()
        );
        let mut skipped = current.clone();
        skipped.sequence = 3;
        assert!(checkpoint_progress_binding_alignment(&skipped, 1, &id, &digest, 1).is_err());
        assert!(
            checkpoint_progress_binding_alignment(&current, 1, "stale-head", &digest, 1).is_err()
        );
        assert!(
            checkpoint_progress_binding_alignment(
                &current,
                1,
                &id,
                &format!("sha256-{}", "c".repeat(64)),
                1
            )
            .is_err()
        );
    }
    #[test]
    fn provider_timeout_observation_is_not_progress_authority() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let genesis = entry(&k);
        let mut next = advance(&genesis, &k);
        next.operation_timeout_ms += 1;
        next.provider_signature.clear();
        sign_workspace_coverage_projection_progress(&mut next, &k).unwrap();
        validate_transition(&genesis, &next).unwrap();
    }
    #[test]
    fn completion_regression_and_phase_jump_are_refused() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let a = entry(&k);
        let mut b = advance(&a, &k);
        b.completed_units = 0;
        assert!(validate_transition(&a, &b).is_err());
        let mut b = advance(&a, &k);
        b.phase = "verifying".into();
        assert!(validate_transition(&a, &b).is_err());
    }
    #[test]
    fn wrong_key_and_signature_substitution_are_refused() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let wrong = SigningKey::from_bytes(&[8; 32]);
        let mut a = entry(&k);
        assert!(sign_workspace_coverage_projection_progress(&mut a, &wrong).is_err());
        a.plan_id.push('x');
        assert!(verify_signature(&a).is_err());
    }
    #[test]
    fn identity_collision_is_refused_but_exact_replay_is_idempotent() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let a = entry(&k);
        assert!(existing_event_is_idempotent(&a, &a).unwrap());
        let mut collision = a.clone();
        collision.completed_units += 1;
        assert!(existing_event_is_idempotent(&a, &collision).is_err());
    }
    #[test]
    fn advancement_requires_new_durable_checkpoint() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let a = entry(&k);
        let mut b = advance(&a, &k);
        b.checkpoint_id = a.checkpoint_id.clone();
        assert!(validate_transition(&a, &b).is_err());
        let mut initial = entry(&k);
        assert!(validate_shape(&initial, true).is_ok());
        initial.completed_units = 1;
        assert!(validate_shape(&initial, true).is_err());
    }
    #[test]
    fn genesis_is_exact_and_nonterminal_events_must_advance() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let genesis = entry(&k);
        for mutate in [
            "phase",
            "status",
            "count",
            "checkpoint",
            "observed",
            "advanced",
        ] {
            let mut alien = genesis.clone();
            match mutate {
                "phase" => alien.phase = "projecting".into(),
                "status" => alien.status = "failed".into(),
                "count" => alien.completed_units = 1,
                "checkpoint" => {
                    alien.checkpoint_id = Some("checkpoint".into());
                    alien.checkpoint_binding_sha256 = Some(format!("sha256-{}", "a".repeat(64)));
                }
                "observed" => alien.observed_at_utc = "2026-07-16T10:00:01Z".into(),
                _ => alien.last_advanced_at_utc = "2026-07-16T10:00:01Z".into(),
            }
            assert!(validate_shape(&alien, true).is_err(), "{mutate}");
        }
        let first = advance(&genesis, &k);
        let mut no_advance = first.clone();
        no_advance.progress_id = uuid::Uuid::new_v4().to_string();
        no_advance.sequence += 1;
        no_advance.observed_at_utc = "2026-07-16T10:00:03Z".into();
        no_advance.provider_signature.clear();
        sign_workspace_coverage_projection_progress(&mut no_advance, &k).unwrap();
        assert!(validate_transition(&first, &no_advance).is_err());
    }
    #[test]
    fn provider_cannot_author_stalled_and_terminal_statuses_do_not_advance() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let genesis = entry(&k);
        let mut stalled = genesis.clone();
        stalled.status = "stalled".into();
        assert!(validate_shape(&stalled, true).is_err());

        let first = advance(&genesis, &k);
        let mut failed = first.clone();
        failed.progress_id = uuid::Uuid::new_v4().to_string();
        failed.sequence += 1;
        failed.status = "failed".into();
        failed.observed_at_utc = "2026-07-16T10:00:03Z".into();
        failed.provider_signature.clear();
        sign_workspace_coverage_projection_progress(&mut failed, &k).unwrap();
        validate_transition(&first, &failed).unwrap();
        let mut replay = failed.clone();
        replay.progress_id = uuid::Uuid::new_v4().to_string();
        replay.sequence += 1;
        replay.observed_at_utc = "2026-07-16T10:00:04Z".into();
        replay.provider_signature.clear();
        sign_workspace_coverage_projection_progress(&mut replay, &k).unwrap();
        assert!(validate_transition(&failed, &replay).is_err());
    }
    #[test]
    fn transition_reauthenticates_both_signed_events() {
        let k = SigningKey::from_bytes(&[7; 32]);
        let mut prior = entry(&k);
        let next = advance(&prior, &k);
        prior.plan_id.push('x');
        assert!(validate_transition(&prior, &next).is_err());
        let prior = entry(&k);
        let mut next = advance(&prior, &k);
        next.provider_signature[0] ^= 1;
        assert!(validate_transition(&prior, &next).is_err());
    }

    #[test]
    fn no_advance_lease_uses_durable_advancement_time_only() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-07-16T10:05:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let lease = chrono::Duration::seconds(300);
        assert!(progress_is_within_no_advance_lease("2026-07-16T10:00:00Z", now, lease).unwrap());
        assert!(!progress_is_within_no_advance_lease("2026-07-16T09:59:59Z", now, lease).unwrap());
        assert!(!progress_is_within_no_advance_lease("2026-07-16T10:05:01Z", now, lease).unwrap());
    }
}
