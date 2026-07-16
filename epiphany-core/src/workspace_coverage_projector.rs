use crate::repository_body_observer::RepositoryBodyReadSession;
use crate::semantic_backend::{
    CollectionCompatibility, OllamaEmbedder, QdrantBackend, SemanticPoint, SemanticStoredPoint,
};
use crate::{
    BODY_BINDING_KEY, BODY_BINDING_TYPE, BODY_HEAD_KEY, BODY_HEAD_TYPE, BODY_MANIFEST_TYPE,
    BODY_OBSERVATION_TYPE, RepositoryBodyHead, RepositoryBodyObservationBasis,
    WORKSPACE_COVERAGE_HEAD_SCHEMA_VERSION, WORKSPACE_COVERAGE_RECEIPT_SCHEMA_VERSION,
    WorkspaceCoverageChunkDescriptor, WorkspaceCoverageHead, WorkspaceCoverageObligation,
    WorkspaceCoveragePointBinding, WorkspaceCoveragePointPayload, WorkspaceCoveragePolicy,
    WorkspaceCoverageProjectionPlan, WorkspaceCoverageReceipt, WorkspaceCoverageVectorBinding,
    derive_workspace_coverage_obligation_from_authenticated_manifest,
    derive_workspace_coverage_projection_plan, refine_workspace_coverage_obligation_utf8,
    runtime_repository_body_store_binding, validate_workspace_coverage_head,
    validate_workspace_coverage_projection_plan, workspace_coverage_execution_collection,
};
use anyhow::{Result, anyhow, bail};
use cultcache_rs::{
    CacheBackingStore, CultCacheEnvelope, DatabaseEntry, SingleFileMessagePackBackingStore,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const CLAIM_TYPE: &str = "gamecult.epiphany.workspace_coverage_projection_claim";
const ATTEMPT_TYPE: &str = "gamecult.epiphany.workspace_coverage_projection_attempt";
const CLAIM_KEY: &str = "workspace-coverage-projector-current";
const CLAIM_SCHEMA: &str = "gamecult.epiphany.workspace_coverage_projection_claim.v1";
const ATTEMPT_SCHEMA: &str = "gamecult.epiphany.workspace_coverage_projection_attempt.v1";
const PROJECTION_SCHEMA: &str = "gamecult.epiphany.workspace_bytes_projection.v0";
const CHUNKER_ID: &str = "utf8_lines_96_overlap_8_v0";
const MAXIMUM_FILE_BYTES: u64 = 4 * 1024 * 1024;
const CHUNK_LINES: usize = 96;
const CHUNK_OVERLAP_LINES: usize = 8;
const RECEIPT_TYPE: &str = "gamecult.epiphany.workspace_coverage_receipt";
const HEAD_TYPE: &str = "gamecult.epiphany.workspace_coverage_head";
const HEAD_KEY: &str = "current";
const OBLIGATION_TYPE: &str = "gamecult.epiphany.workspace_coverage_obligation";
const PLAN_TYPE: &str = "gamecult.epiphany.workspace_coverage_projection_plan";

#[derive(Clone, Debug)]
struct WorkspaceCoverageProjectionInput {
    pub point_id: String,
    pub text: String,
    pub vector: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkspaceCoverageObservedBinding {
    collection_name: String,
    point_count: u64,
    point_set_sha256: String,
    point_binding_set_sha256: String,
    vector_binding_set_sha256: String,
}

pub(crate) trait WorkspaceCoverageProjectionPort {
    fn ensure_exact_collection(
        &mut self,
        collection: &str,
        compatibility: &CollectionCompatibility,
    ) -> Result<()>;
    fn upsert(
        &mut self,
        collection: &str,
        points: &[SemanticPoint<WorkspaceCoveragePointPayload>],
    ) -> Result<()>;
    fn observe_all(
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
    fn upsert(
        &mut self,
        name: &str,
        points: &[SemanticPoint<WorkspaceCoveragePointPayload>],
    ) -> Result<()> {
        self.upsert_points(name, points)
    }
    fn observe_all(
        &mut self,
        name: &str,
    ) -> Result<Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>> {
        self.points_for_scope(name, &[])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.workspace_coverage_projection_claim",
    schema = "WorkspaceCoverageProjectionClaim"
)]
pub(crate) struct WorkspaceCoverageProjectionClaim {
    #[cultcache(key = 0)]
    schema_version: String,
    #[cultcache(key = 1)]
    claim_id: String,
    #[cultcache(key = 2)]
    claim_epoch: u64,
    #[cultcache(key = 3)]
    plan_id: String,
    #[cultcache(key = 4)]
    attempt_id: String,
    #[cultcache(key = 5)]
    obligation_id: String,
    #[cultcache(key = 6)]
    body_observation_id: String,
    #[cultcache(key = 7)]
    body_generation: u64,
    #[cultcache(key = 8)]
    manifest_root_sha256: String,
    #[cultcache(key = 9)]
    status: String,
    #[cultcache(key = 10)]
    executor_id: String,
    #[cultcache(key = 11)]
    executor_incarnation: String,
    #[cultcache(key = 12)]
    managed_process_launch_id: String,
}

fn validate_projection_claim(claim: &WorkspaceCoverageProjectionClaim) -> Result<()> {
    if claim.schema_version != CLAIM_SCHEMA
        || claim.claim_id.trim().is_empty()
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
    Ok(())
}

fn validate_projection_attempt(attempt: &WorkspaceCoverageProjectionAttempt) -> Result<()> {
    if attempt.schema_version != ATTEMPT_SCHEMA
        || attempt.attempt_id.trim().is_empty()
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
        chrono::DateTime::parse_from_rfc3339(value)?;
    }
    Ok(())
}

fn validate_claim_attempt_link(
    claim: &WorkspaceCoverageProjectionClaim,
    attempt: &WorkspaceCoverageProjectionAttempt,
) -> Result<()> {
    validate_projection_claim(claim)?;
    validate_projection_attempt(attempt)?;
    if claim.claim_id != attempt.claim_id
        || claim.claim_epoch != attempt.claim_epoch
        || claim.attempt_id != attempt.attempt_id
        || claim.plan_id != attempt.plan_id
        || claim.status != attempt.status
        || claim.executor_id != attempt.executor_id
        || claim.executor_incarnation != attempt.executor_incarnation
        || claim.managed_process_launch_id != attempt.managed_process_launch_id
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
    schema_version: String,
    #[cultcache(key = 1)]
    attempt_id: String,
    #[cultcache(key = 2)]
    claim_id: String,
    #[cultcache(key = 3)]
    claim_epoch: u64,
    #[cultcache(key = 4)]
    plan_id: String,
    #[cultcache(key = 5)]
    status: String,
    #[cultcache(key = 6)]
    started_at: String,
    #[cultcache(key = 7)]
    completed_at: Option<String>,
    #[cultcache(key = 8)]
    error: Option<String>,
    #[cultcache(key = 9)]
    executor_id: String,
    #[cultcache(key = 10)]
    executor_incarnation: String,
    #[cultcache(key = 11)]
    managed_process_launch_id: String,
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
    let policy = WorkspaceCoveragePolicy::bounded_regular_files_v0(MAXIMUM_FILE_BYTES)?;
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

pub(crate) fn prepare_workspace_coverage_projection(
    body: &RepositoryBodyReadSession,
    embedding_provider_id: &str,
    embedding_model: &str,
    vector_dimensions: u32,
) -> Result<PreparedWorkspaceCoverageProjection> {
    let basis = body.basis();
    let policy = WorkspaceCoveragePolicy::bounded_regular_files_v0(MAXIMUM_FILE_BYTES)?;
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
) -> Result<WorkspaceCoverageReceipt> {
    let result = (|| {
        if prepared.body_store != acquisition.body_store
            || prepared.obligation != acquisition.obligation
            || prepared.plan != acquisition.plan
        {
            bail!("execution refuses a prepared projection other than the acquired projection");
        }
        let texts = prepared
            .points
            .iter()
            .map(|point| point.text.clone())
            .collect::<Vec<_>>();
        let vectors = if texts.is_empty() {
            Vec::new()
        } else {
            embedder.embed_documents(&texts)?
        };
        let inputs = prepared
            .points
            .iter()
            .cloned()
            .zip(vectors)
            .map(|(point, vector)| WorkspaceCoverageProjectionInput {
                point_id: point.point_id,
                text: point.text,
                vector,
            })
            .collect();
        execute_and_commit(acquisition, inputs, port)
    })();
    terminalize_execution_result(acquisition, result)
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
        port.upsert(&collection_name, &points)?;
    }
    let observed = observe_exact_bindings(
        acquisition,
        &collection_name,
        port.observe_all(&collection_name)?,
        &submitted_vector_bindings,
    )?;
    commit_workspace_coverage_success(acquisition, observed)
}

fn observe_exact_bindings(
    acquisition: &WorkspaceCoverageAcquisition,
    collection_name: &str,
    observed: Vec<SemanticStoredPoint<WorkspaceCoveragePointPayload>>,
    submitted_vector_bindings: &[WorkspaceCoverageVectorBinding],
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
        if vector.len() != acquisition.plan.vector_dimensions as usize
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
    if bindings != acquisition.plan.point_bindings {
        bail!("observed workspace point bindings do not equal the sealed plan");
    }
    vector_bindings.sort_by(|left, right| left.point_id.cmp(&right.point_id));
    if vector_bindings != submitted_vector_bindings {
        bail!("observed workspace vectors do not equal the submitted vectors");
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

fn payload_for(
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

fn exact_obligation_body_authority(
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
    }

    impl FakeProjectionPort {
        fn new(hostility: Hostility) -> Self {
            Self {
                hostility,
                points: Vec::new(),
                upsert_calls: 0,
            }
        }
    }

    impl WorkspaceCoverageProjectionPort for FakeProjectionPort {
        fn ensure_exact_collection(&mut self, _: &str, _: &CollectionCompatibility) -> Result<()> {
            if matches!(self.hostility, Hostility::IncompatibleCollection) {
                bail!("incompatible preexisting collection metadata");
            }
            Ok(())
        }
        fn upsert(
            &mut self,
            _: &str,
            points: &[SemanticPoint<WorkspaceCoveragePointPayload>],
        ) -> Result<()> {
            self.upsert_calls += 1;
            self.points = points
                .iter()
                .map(|point| SemanticStoredPoint {
                    id: point.id.clone(),
                    payload: Some(point.payload.clone()),
                    vector: Some(point.vector.clone()),
                })
                .collect();
            Ok(())
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

    #[test]
    #[ignore = "requires live local Qdrant and Ollama"]
    fn live_qdrant_ollama_projection_proves_vectors_and_currentness() -> Result<()> {
        let (_repo, _state, runtime, basis) = coverage_fixture()?;
        let ollama_url = std::env::var("EPIPHANY_OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:11434".into());
        let qdrant_url =
            std::env::var("EPIPHANY_QDRANT_URL").unwrap_or_else(|_| "http://127.0.0.1:6333".into());
        let model = std::env::var("EPIPHANY_OLLAMA_MODEL")
            .unwrap_or_else(|_| "qwen3-embedding:0.6b".into());
        let embedder = OllamaEmbedder::new(crate::semantic_backend::OllamaConfig {
            base_url: ollama_url,
            model,
            timeout_ms: 30_000,
            query_instruction: String::new(),
        })?;
        let model_identity = embedder.model_artifact()?.canonical_identity();
        let dimensions = embedder.embedding_dimensions()?;
        let body = RepositoryBodyReadSession::open(&runtime, &basis)?;
        let prepared = prepare_workspace_coverage_projection(
            &body,
            "gamecult-ollama-embedding",
            &model_identity,
            dimensions,
        )?;
        let acquisition = match acquire_workspace_coverage_projection(
            &prepared,
            "epiphany-workspace-coverage-projector",
            &uuid::Uuid::new_v4().to_string(),
            &uuid::Uuid::new_v4().to_string(),
        )? {
            WorkspaceCoverageAcquireResult::Acquired(value) => value,
            _ => bail!("isolated live smoke did not acquire its projection"),
        };
        let compatibility = CollectionCompatibility {
            managed_by: "epiphany-workspace-coverage-projector".into(),
            corpus_kind: "repository_body_workspace_coverage".into(),
            schema_version: 0,
            projection_version: acquisition.plan.plan_id.clone(),
            embedding_provider_id: acquisition.plan.embedding_provider_id.clone(),
            embedding_model: acquisition.plan.embedding_model.clone(),
            vector_size: acquisition.plan.vector_dimensions as usize,
        };
        let mut qdrant = QdrantBackend::new(crate::semantic_backend::QdrantConfig {
            url: qdrant_url,
            api_key: None,
            timeout_ms: 30_000,
        })?;
        let result =
            execute_workspace_coverage_projection(&acquisition, &prepared, &embedder, &mut qdrant);
        let collection = workspace_coverage_execution_collection(
            &acquisition.plan.plan_id,
            &acquisition.claim.claim_id,
            acquisition.claim.claim_epoch,
        )?;
        let receipt = match result {
            Ok(receipt) => receipt,
            Err(error) => {
                let _ = qdrant.retire_exact_collection(&collection, &compatibility);
                return Err(error);
            }
        };
        assert_eq!(receipt.collection_name, collection);
        assert!(matches!(
            classify_current_workspace_coverage(
                &runtime,
                &basis,
                "gamecult-ollama-embedding",
                &model_identity,
                dimensions,
            )?,
            WorkspaceCoverageCurrentState::Current(_)
        ));
        qdrant.retire_exact_collection(&collection, &compatibility)?;
        assert!(!qdrant.collection_exists(&collection)?);
        Ok(())
    }
}
