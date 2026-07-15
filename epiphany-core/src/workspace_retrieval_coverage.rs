use crate::{
    RepositoryBodyManifest, RepositoryBodyManifestEntry, RepositoryBodyObservationBasis,
    authenticated_repository_body_manifest,
};
use anyhow::{Result, anyhow, bail};
use cultcache_rs::DatabaseEntry;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

pub const WORKSPACE_COVERAGE_POLICY_SCHEMA_VERSION: &str =
    "gamecult.epiphany.workspace_coverage_policy.v0";
pub const WORKSPACE_COVERAGE_OBLIGATION_SCHEMA_VERSION: &str =
    "gamecult.epiphany.workspace_coverage_obligation.v0";
pub const WORKSPACE_COVERAGE_RECEIPT_SCHEMA_VERSION: &str =
    "gamecult.epiphany.workspace_coverage_receipt.v0";
pub const WORKSPACE_COVERAGE_PROJECTION_PLAN_SCHEMA_VERSION: &str =
    "gamecult.epiphany.workspace_coverage_projection_plan.v0";
pub const WORKSPACE_COVERAGE_HEAD_SCHEMA_VERSION: &str =
    "gamecult.epiphany.workspace_coverage_head.v0";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceCoverageDisposition {
    IncludeRegularFile,
    ExcludeGitlink,
    ExcludeSymlink,
    ExcludeUnsupportedKind,
    ExcludeEmptyRegularFile,
    ExcludeNonUtf8RegularFile,
    ExcludeOversize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceCoveragePolicy {
    pub schema_version: String,
    pub policy_id: String,
    pub maximum_file_bytes: u64,
}

impl WorkspaceCoveragePolicy {
    pub fn bounded_regular_files_v0(maximum_file_bytes: u64) -> Result<Self> {
        if maximum_file_bytes == 0 {
            bail!("workspace coverage policy maximum_file_bytes must be positive");
        }
        Ok(Self {
            schema_version: WORKSPACE_COVERAGE_POLICY_SCHEMA_VERSION.into(),
            policy_id: "bounded_regular_files_v0".into(),
            maximum_file_bytes,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceCoverageClassification {
    pub path: String,
    pub raw_sha256: String,
    pub raw_byte_length: u64,
    pub disposition: WorkspaceCoverageDisposition,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.workspace_coverage_obligation",
    schema = "WorkspaceCoverageObligation"
)]
pub(crate) struct WorkspaceCoverageObligation {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub obligation_id: String,
    #[cultcache(key = 2)]
    pub workspace_id: String,
    #[cultcache(key = 3)]
    pub swarm_id: String,
    #[cultcache(key = 4)]
    pub runtime_id: String,
    #[cultcache(key = 5)]
    pub body_binding_sha256: String,
    #[cultcache(key = 6)]
    pub body_observation_id: String,
    #[cultcache(key = 7)]
    pub body_generation: u64,
    #[cultcache(key = 8)]
    pub manifest_root_sha256: String,
    #[cultcache(key = 9)]
    pub policy_id: String,
    #[cultcache(key = 10)]
    pub policy_sha256: String,
    #[cultcache(key = 11)]
    pub classification_set_sha256: String,
    #[cultcache(key = 12)]
    pub classifications: Vec<WorkspaceCoverageClassification>,
    #[cultcache(key = 13)]
    pub included_entry_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceCoverageChunkDescriptor {
    pub body_path: String,
    pub source_raw_sha256: String,
    pub source_raw_byte_length: u64,
    pub chunk_index: u32,
    pub byte_start: u64,
    pub byte_end: u64,
    pub chunk_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceCoveragePlannedPoint {
    pub point_id: String,
    pub body_path: String,
    pub source_raw_sha256: String,
    pub source_raw_byte_length: u64,
    pub chunk_index: u32,
    pub byte_start: u64,
    pub byte_end: u64,
    pub chunk_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceCoveragePointPayload {
    pub obligation_id: String,
    pub projection_schema_version: String,
    pub chunker_id: String,
    pub body_path: String,
    pub source_raw_sha256: String,
    pub source_raw_byte_length: u64,
    pub chunk_index: u32,
    pub byte_start: u64,
    pub byte_end: u64,
    pub chunk_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceCoveragePointBinding {
    pub point_id: String,
    pub payload_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceCoverageVectorBinding {
    pub point_id: String,
    pub vector_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.workspace_coverage_projection_plan",
    schema = "WorkspaceCoverageProjectionPlan"
)]
pub(crate) struct WorkspaceCoverageProjectionPlan {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub plan_id: String,
    #[cultcache(key = 2)]
    pub obligation_id: String,
    #[cultcache(key = 3)]
    pub projection_schema_version: String,
    #[cultcache(key = 4)]
    pub chunker_id: String,
    #[cultcache(key = 5)]
    pub embedding_provider_id: String,
    #[cultcache(key = 6)]
    pub embedding_model: String,
    #[cultcache(key = 7)]
    pub vector_dimensions: u32,
    #[cultcache(key = 8)]
    pub planned_points: Vec<WorkspaceCoveragePlannedPoint>,
    #[cultcache(key = 9)]
    pub expected_point_count: u64,
    #[cultcache(key = 10)]
    pub expected_point_set_sha256: String,
    #[cultcache(key = 11)]
    pub point_bindings: Vec<WorkspaceCoveragePointBinding>,
    #[cultcache(key = 12)]
    pub point_binding_set_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.workspace_coverage_receipt",
    schema = "WorkspaceCoverageReceipt"
)]
pub(crate) struct WorkspaceCoverageReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub obligation_id: String,
    #[cultcache(key = 3)]
    pub plan_id: String,
    #[cultcache(key = 4)]
    pub collection_name: String,
    #[cultcache(key = 5)]
    pub embedding_provider_id: String,
    #[cultcache(key = 6)]
    pub embedding_model: String,
    #[cultcache(key = 7)]
    pub vector_dimensions: u32,
    #[cultcache(key = 8)]
    pub observed_point_count: u64,
    #[cultcache(key = 9)]
    pub observed_point_set_sha256: String,
    #[cultcache(key = 10)]
    pub observed_at: String,
    #[cultcache(key = 11)]
    pub observation_method: String,
    #[cultcache(key = 12)]
    pub observed_point_binding_set_sha256: String,
    #[cultcache(key = 13)]
    pub claim_id: String,
    #[cultcache(key = 14)]
    pub claim_epoch: u64,
    #[cultcache(key = 15)]
    pub observed_vector_binding_set_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.workspace_coverage_head",
    schema = "WorkspaceCoverageHead"
)]
pub(crate) struct WorkspaceCoverageHead {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub workspace_id: String,
    #[cultcache(key = 2)]
    pub obligation_id: String,
    #[cultcache(key = 3)]
    pub plan_id: String,
    #[cultcache(key = 4)]
    pub receipt_id: String,
    #[cultcache(key = 5)]
    pub body_observation_id: String,
    #[cultcache(key = 6)]
    pub body_generation: u64,
    #[cultcache(key = 7)]
    pub manifest_root_sha256: String,
    #[cultcache(key = 8)]
    pub claim_id: String,
    #[cultcache(key = 9)]
    pub claim_epoch: u64,
}

pub(crate) fn derive_workspace_coverage_obligation(
    runtime_store: &Path,
    basis: &RepositoryBodyObservationBasis,
    policy: &WorkspaceCoveragePolicy,
) -> Result<WorkspaceCoverageObligation> {
    let manifest = authenticated_repository_body_manifest(runtime_store, basis)?;
    derive_workspace_coverage_obligation_from_authenticated_manifest(basis, &manifest, policy)
}

pub(crate) fn derive_workspace_coverage_obligation_from_authenticated_manifest(
    basis: &RepositoryBodyObservationBasis,
    manifest: &RepositoryBodyManifest,
    policy: &WorkspaceCoveragePolicy,
) -> Result<WorkspaceCoverageObligation> {
    if policy.schema_version != WORKSPACE_COVERAGE_POLICY_SCHEMA_VERSION
        || policy.policy_id.trim().is_empty()
        || policy.maximum_file_bytes == 0
    {
        bail!("unsupported or invalid workspace coverage policy");
    }
    if manifest.workspace_id != basis.workspace_id
        || manifest.scope != basis.scope
        || manifest.manifest_root_sha256 != basis.manifest_root_sha256
        || basis.generation == 0
    {
        bail!("workspace coverage input manifest disagrees with authenticated Body basis");
    }
    let mut prior_path: Option<&str> = None;
    let mut classifications = Vec::with_capacity(manifest.entries.len());
    for entry in &manifest.entries {
        if entry.path.trim().is_empty()
            || (entry.raw_sha256.trim().is_empty() && entry.kind != "gitlink_nonrecursive")
            || prior_path.is_some_and(|prior| prior >= entry.path.as_str())
        {
            bail!("Body manifest entries must have unique, strictly ordered paths and hashes");
        }
        prior_path = Some(&entry.path);
        classifications.push(classify(entry, policy));
    }
    let policy_sha256 = digest(policy)?;
    let classification_set_sha256 = digest(&classifications)?;
    let included_entry_count = classifications
        .iter()
        .filter(|entry| entry.disposition == WorkspaceCoverageDisposition::IncludeRegularFile)
        .count() as u64;
    let obligation_id = format!(
        "workspace-coverage:{}",
        digest(&(
            &basis.body_binding_sha256,
            &basis.observation_id,
            &basis.manifest_root_sha256,
            &policy_sha256,
            &classification_set_sha256,
        ))?
    );
    Ok(WorkspaceCoverageObligation {
        schema_version: WORKSPACE_COVERAGE_OBLIGATION_SCHEMA_VERSION.into(),
        obligation_id,
        workspace_id: basis.workspace_id.clone(),
        swarm_id: basis.swarm_id.clone(),
        runtime_id: basis.runtime_id.clone(),
        body_binding_sha256: basis.body_binding_sha256.clone(),
        body_observation_id: basis.observation_id.clone(),
        body_generation: basis.generation,
        manifest_root_sha256: basis.manifest_root_sha256.clone(),
        policy_id: policy.policy_id.clone(),
        policy_sha256,
        classification_set_sha256,
        classifications,
        included_entry_count,
    })
}

pub(crate) fn refine_workspace_coverage_obligation_utf8(
    obligation: &WorkspaceCoverageObligation,
    non_utf8_paths: &[String],
) -> Result<WorkspaceCoverageObligation> {
    if !non_utf8_paths.windows(2).all(|pair| pair[0] < pair[1]) {
        bail!("non-UTF-8 Body paths must be unique and canonically ordered");
    }
    let mut refined = obligation.clone();
    for path in non_utf8_paths {
        let classification = refined
            .classifications
            .iter_mut()
            .find(|entry| &entry.path == path)
            .ok_or_else(|| anyhow!("non-UTF-8 path is absent from Body obligation"))?;
        if classification.disposition != WorkspaceCoverageDisposition::IncludeRegularFile {
            bail!("non-UTF-8 refinement may only demote an included regular file");
        }
        classification.disposition = WorkspaceCoverageDisposition::ExcludeNonUtf8RegularFile;
    }
    refined.included_entry_count = refined
        .classifications
        .iter()
        .filter(|entry| entry.disposition == WorkspaceCoverageDisposition::IncludeRegularFile)
        .count() as u64;
    refined.classification_set_sha256 = digest(&refined.classifications)?;
    refined.obligation_id = format!(
        "workspace-coverage:{}",
        digest(&(
            &refined.body_binding_sha256,
            &refined.body_observation_id,
            &refined.manifest_root_sha256,
            &refined.policy_sha256,
            &refined.classification_set_sha256,
        ))?
    );
    Ok(refined)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn derive_workspace_coverage_projection_plan(
    obligation: &WorkspaceCoverageObligation,
    projection_schema_version: &str,
    chunker_id: &str,
    embedding_provider_id: &str,
    embedding_model: &str,
    vector_dimensions: u32,
    chunk_descriptors: Vec<WorkspaceCoverageChunkDescriptor>,
) -> Result<WorkspaceCoverageProjectionPlan> {
    if obligation.schema_version != WORKSPACE_COVERAGE_OBLIGATION_SCHEMA_VERSION
        || projection_schema_version.trim().is_empty()
        || chunker_id.trim().is_empty()
        || embedding_provider_id.trim().is_empty()
        || embedding_model.trim().is_empty()
        || vector_dimensions == 0
        || (obligation.included_entry_count == 0) != chunk_descriptors.is_empty()
    {
        bail!("workspace coverage projection plan is incomplete or noncanonical");
    }
    let included = obligation
        .classifications
        .iter()
        .filter(|entry| entry.disposition == WorkspaceCoverageDisposition::IncludeRegularFile)
        .map(|entry| (entry.path.as_str(), entry))
        .collect::<std::collections::HashMap<_, _>>();
    let mut planned_points = Vec::with_capacity(chunk_descriptors.len());
    let mut prior_descriptor: Option<(String, u32)> = None;
    for descriptor in chunk_descriptors {
        let Some(source) = included.get(descriptor.body_path.as_str()) else {
            bail!("workspace coverage chunk does not name an included Body entry");
        };
        let valid_range = descriptor.byte_start < descriptor.byte_end
            && descriptor.byte_end <= descriptor.source_raw_byte_length;
        if descriptor.source_raw_sha256 != source.raw_sha256
            || descriptor.source_raw_byte_length != source.raw_byte_length
            || !sha256_hex(&descriptor.source_raw_sha256)
            || !sha256_hex(&descriptor.chunk_sha256)
            || !valid_range
            || (prior_descriptor.is_none() && descriptor.chunk_index != 0)
            || prior_descriptor.as_ref().is_some_and(|prior| {
                if prior.0 == descriptor.body_path {
                    prior.1.checked_add(1) != Some(descriptor.chunk_index)
                } else {
                    descriptor.body_path <= prior.0 || descriptor.chunk_index != 0
                }
            })
        {
            bail!("workspace coverage chunk descriptor disagrees with Body or canonical order");
        }
        prior_descriptor = Some((descriptor.body_path.clone(), descriptor.chunk_index));
        let point_id = uuid::Uuid::new_v5(
            &uuid::Uuid::NAMESPACE_URL,
            rmp_serde::to_vec_named(&(
                &obligation.obligation_id,
                projection_schema_version,
                chunker_id,
                &descriptor,
            ))?
            .as_slice(),
        )
        .to_string();
        planned_points.push(WorkspaceCoveragePlannedPoint {
            point_id,
            body_path: descriptor.body_path,
            source_raw_sha256: descriptor.source_raw_sha256,
            source_raw_byte_length: descriptor.source_raw_byte_length,
            chunk_index: descriptor.chunk_index,
            byte_start: descriptor.byte_start,
            byte_end: descriptor.byte_end,
            chunk_sha256: descriptor.chunk_sha256,
        });
    }
    if included.values().any(|entry| {
        !planned_points
            .iter()
            .any(|point| point.body_path == entry.path)
    }) {
        bail!("workspace coverage plan omits an included Body entry");
    }
    let mut point_ids = planned_points
        .iter()
        .map(|point| point.point_id.clone())
        .collect::<Vec<_>>();
    point_ids.sort();
    let mut point_bindings = planned_points
        .iter()
        .map(|point| {
            let payload = point_payload(obligation, projection_schema_version, chunker_id, point);
            Ok(WorkspaceCoveragePointBinding {
                point_id: point.point_id.clone(),
                payload_sha256: digest(&payload)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    point_bindings.sort_by(|left, right| left.point_id.cmp(&right.point_id));
    let expected_point_count = point_ids.len() as u64;
    let expected_point_set_sha256 = digest(&point_ids)?;
    let point_binding_set_sha256 = digest(&point_bindings)?;
    let identity = (
        &obligation.obligation_id,
        projection_schema_version,
        chunker_id,
        embedding_provider_id,
        embedding_model,
        vector_dimensions,
        &planned_points,
        &expected_point_set_sha256,
        &point_bindings,
        &point_binding_set_sha256,
    );
    let plan_id = format!("workspace-coverage-plan:{}", digest(&identity)?);
    Ok(WorkspaceCoverageProjectionPlan {
        schema_version: WORKSPACE_COVERAGE_PROJECTION_PLAN_SCHEMA_VERSION.into(),
        plan_id,
        obligation_id: obligation.obligation_id.clone(),
        projection_schema_version: projection_schema_version.into(),
        chunker_id: chunker_id.into(),
        embedding_provider_id: embedding_provider_id.into(),
        embedding_model: embedding_model.into(),
        vector_dimensions,
        planned_points,
        expected_point_count,
        expected_point_set_sha256,
        point_bindings,
        point_binding_set_sha256,
    })
}

pub(crate) fn validate_workspace_coverage_projection_plan(
    obligation: &WorkspaceCoverageObligation,
    plan: &WorkspaceCoverageProjectionPlan,
) -> Result<()> {
    let expected = derive_workspace_coverage_projection_plan(
        obligation,
        &plan.projection_schema_version,
        &plan.chunker_id,
        &plan.embedding_provider_id,
        &plan.embedding_model,
        plan.vector_dimensions,
        plan.planned_points
            .iter()
            .map(|point| WorkspaceCoverageChunkDescriptor {
                body_path: point.body_path.clone(),
                source_raw_sha256: point.source_raw_sha256.clone(),
                source_raw_byte_length: point.source_raw_byte_length,
                chunk_index: point.chunk_index,
                byte_start: point.byte_start,
                byte_end: point.byte_end,
                chunk_sha256: point.chunk_sha256.clone(),
            })
            .collect(),
    )?;
    if *plan != expected {
        bail!("workspace coverage projection plan identity or derived fields were altered");
    }
    Ok(())
}

pub(crate) fn validate_workspace_coverage_receipt(
    obligation: &WorkspaceCoverageObligation,
    plan: &WorkspaceCoverageProjectionPlan,
    receipt: &WorkspaceCoverageReceipt,
) -> Result<()> {
    validate_workspace_coverage_projection_plan(obligation, plan)?;
    let expected_collection = workspace_coverage_execution_collection(
        &plan.plan_id,
        &receipt.claim_id,
        receipt.claim_epoch,
    )?;
    if receipt.schema_version != WORKSPACE_COVERAGE_RECEIPT_SCHEMA_VERSION
        || receipt.obligation_id != obligation.obligation_id
        || receipt.plan_id != plan.plan_id
        || receipt.receipt_id.trim().is_empty()
        || receipt.collection_name != expected_collection
        || receipt.embedding_provider_id != plan.embedding_provider_id
        || receipt.embedding_model != plan.embedding_model
        || receipt.vector_dimensions != plan.vector_dimensions
        || receipt.observed_at.trim().is_empty()
        || receipt.observation_method != "qdrant_scroll_exact_point_bindings"
        || !sha256_hex(&receipt.observed_vector_binding_set_sha256)
    {
        bail!("workspace coverage receipt lacks exact observed-index authority");
    }
    if receipt.observed_point_count != plan.expected_point_count
        || receipt.observed_point_set_sha256 != plan.expected_point_set_sha256
        || receipt.observed_point_binding_set_sha256 != plan.point_binding_set_sha256
    {
        bail!("workspace coverage receipt does not prove the exact expected point set");
    }
    Ok(())
}

pub(crate) fn validate_workspace_coverage_head(
    obligation: &WorkspaceCoverageObligation,
    plan: &WorkspaceCoverageProjectionPlan,
    receipt: &WorkspaceCoverageReceipt,
    head: &WorkspaceCoverageHead,
) -> Result<()> {
    validate_workspace_coverage_receipt(obligation, plan, receipt)?;
    if head.schema_version != WORKSPACE_COVERAGE_HEAD_SCHEMA_VERSION
        || head.workspace_id != obligation.workspace_id
        || head.obligation_id != obligation.obligation_id
        || head.plan_id != plan.plan_id
        || head.receipt_id != receipt.receipt_id
        || receipt.obligation_id != obligation.obligation_id
        || receipt.plan_id != plan.plan_id
        || head.body_observation_id != obligation.body_observation_id
        || head.body_generation != obligation.body_generation
        || head.manifest_root_sha256 != obligation.manifest_root_sha256
        || head.claim_id != receipt.claim_id
        || head.claim_epoch != receipt.claim_epoch
    {
        bail!("workspace coverage head does not join one obligation to its exact receipt");
    }
    Ok(())
}

pub fn workspace_coverage_execution_collection(
    plan_id: &str,
    claim_id: &str,
    claim_epoch: u64,
) -> Result<String> {
    if plan_id.trim().is_empty() || uuid::Uuid::parse_str(claim_id).is_err() || claim_epoch == 0 {
        bail!("workspace coverage execution collection requires plan and fenced UUID claim");
    }
    let namespace = digest(&(plan_id, claim_id, claim_epoch))?;
    Ok(format!("epiphany_workspace_{}", &namespace[..40]))
}

fn point_payload(
    obligation: &WorkspaceCoverageObligation,
    projection_schema_version: &str,
    chunker_id: &str,
    point: &WorkspaceCoveragePlannedPoint,
) -> WorkspaceCoveragePointPayload {
    WorkspaceCoveragePointPayload {
        obligation_id: obligation.obligation_id.clone(),
        projection_schema_version: projection_schema_version.into(),
        chunker_id: chunker_id.into(),
        body_path: point.body_path.clone(),
        source_raw_sha256: point.source_raw_sha256.clone(),
        source_raw_byte_length: point.source_raw_byte_length,
        chunk_index: point.chunk_index,
        byte_start: point.byte_start,
        byte_end: point.byte_end,
        chunk_sha256: point.chunk_sha256.clone(),
    }
}

fn sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn classify(
    entry: &RepositoryBodyManifestEntry,
    policy: &WorkspaceCoveragePolicy,
) -> WorkspaceCoverageClassification {
    let disposition = match entry.kind.as_str() {
        "gitlink_nonrecursive" => WorkspaceCoverageDisposition::ExcludeGitlink,
        "symlink_target" => WorkspaceCoverageDisposition::ExcludeSymlink,
        "regular" if entry.raw_byte_length == 0 => {
            WorkspaceCoverageDisposition::ExcludeEmptyRegularFile
        }
        "regular" if entry.raw_byte_length > policy.maximum_file_bytes => {
            WorkspaceCoverageDisposition::ExcludeOversize
        }
        "regular" => WorkspaceCoverageDisposition::IncludeRegularFile,
        _ => WorkspaceCoverageDisposition::ExcludeUnsupportedKind,
    };
    WorkspaceCoverageClassification {
        path: entry.path.clone(),
        raw_sha256: entry.raw_sha256.clone(),
        raw_byte_length: entry.raw_byte_length,
        disposition,
    }
}

fn digest(value: &(impl Serialize + ?Sized)) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(rmp_serde::to_vec_named(value).map_err(|error| anyhow!(error))?)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basis() -> RepositoryBodyObservationBasis {
        RepositoryBodyObservationBasis {
            schema_version: "epiphany.repository_body.v2".into(),
            workspace_id: "workspace-1".into(),
            swarm_id: "swarm-1".into(),
            runtime_id: "runtime-1".into(),
            scope: "git_worktree".into(),
            body_binding_sha256: "binding-hash".into(),
            observation_id: "workspace-1:7".into(),
            generation: 7,
            manifest_root_sha256: "manifest-hash".into(),
            scan_started_at: "start".into(),
            scan_finished_at: "finish".into(),
        }
    }

    fn entry(path: &str, kind: &str, size: u64) -> RepositoryBodyManifestEntry {
        RepositoryBodyManifestEntry {
            path: path.into(),
            git_mode: "100644".into(),
            kind: kind.into(),
            raw_byte_length: size,
            raw_sha256: format!("hash-{path}"),
            gitlink_oid: None,
        }
    }

    #[test]
    fn every_manifest_entry_receives_one_policy_disposition() -> Result<()> {
        let manifest = RepositoryBodyManifest {
            schema_version: "epiphany.repository_body.v2".into(),
            manifest_root_sha256: "manifest-hash".into(),
            workspace_id: "workspace-1".into(),
            scope: "git_worktree".into(),
            entries: vec![
                entry("a-empty", "regular", 0),
                entry("a.rs", "regular", 12),
                entry("b.bin", "regular", 101),
                entry("c-link", "symlink_target", 3),
                entry("d-submodule", "gitlink_nonrecursive", 0),
            ],
        };
        let obligation = derive_workspace_coverage_obligation_from_authenticated_manifest(
            &basis(),
            &manifest,
            &WorkspaceCoveragePolicy::bounded_regular_files_v0(100)?,
        )?;
        assert_eq!(obligation.classifications.len(), manifest.entries.len());
        assert_eq!(obligation.included_entry_count, 1);
        assert_eq!(
            obligation.classifications[0].disposition,
            WorkspaceCoverageDisposition::ExcludeEmptyRegularFile
        );
        assert_eq!(
            obligation.classifications[2].disposition,
            WorkspaceCoverageDisposition::ExcludeOversize
        );
        Ok(())
    }

    #[test]
    fn manifest_substitution_and_duplicate_paths_are_refused() -> Result<()> {
        let mut manifest = RepositoryBodyManifest {
            schema_version: "epiphany.repository_body.v2".into(),
            manifest_root_sha256: "wrong".into(),
            workspace_id: "workspace-1".into(),
            scope: "git_worktree".into(),
            entries: vec![entry("a", "regular", 1)],
        };
        let policy = WorkspaceCoveragePolicy::bounded_regular_files_v0(100)?;
        assert!(
            derive_workspace_coverage_obligation_from_authenticated_manifest(
                &basis(),
                &manifest,
                &policy
            )
            .is_err()
        );
        manifest.manifest_root_sha256 = "manifest-hash".into();
        manifest.entries.push(entry("a", "regular", 1));
        assert!(
            derive_workspace_coverage_obligation_from_authenticated_manifest(
                &basis(),
                &manifest,
                &policy
            )
            .is_err()
        );
        Ok(())
    }

    fn coverage_obligation(id: &str) -> WorkspaceCoverageObligation {
        WorkspaceCoverageObligation {
            schema_version: WORKSPACE_COVERAGE_OBLIGATION_SCHEMA_VERSION.into(),
            obligation_id: id.into(),
            workspace_id: "workspace-1".into(),
            swarm_id: "swarm-1".into(),
            runtime_id: "runtime-1".into(),
            body_binding_sha256: "binding".into(),
            body_observation_id: "workspace-1:1".into(),
            body_generation: 1,
            manifest_root_sha256: "manifest".into(),
            policy_id: "bounded_regular_files_v0".into(),
            policy_sha256: "policy".into(),
            classification_set_sha256: "classes".into(),
            classifications: vec![WorkspaceCoverageClassification {
                path: "src/lib.rs".into(),
                raw_sha256: "11".repeat(32),
                raw_byte_length: 10,
                disposition: WorkspaceCoverageDisposition::IncludeRegularFile,
            }],
            included_entry_count: 1,
        }
    }

    fn chunks() -> Vec<WorkspaceCoverageChunkDescriptor> {
        vec![
            WorkspaceCoverageChunkDescriptor {
                body_path: "src/lib.rs".into(),
                source_raw_sha256: "11".repeat(32),
                source_raw_byte_length: 10,
                chunk_index: 0,
                byte_start: 0,
                byte_end: 5,
                chunk_sha256: "22".repeat(32),
            },
            WorkspaceCoverageChunkDescriptor {
                body_path: "src/lib.rs".into(),
                source_raw_sha256: "11".repeat(32),
                source_raw_byte_length: 10,
                chunk_index: 1,
                byte_start: 3,
                byte_end: 10,
                chunk_sha256: "33".repeat(32),
            },
        ]
    }

    fn plan(obligation: &WorkspaceCoverageObligation) -> Result<WorkspaceCoverageProjectionPlan> {
        derive_workspace_coverage_projection_plan(
            obligation,
            "projection-v0",
            "chunker-v0",
            "provider",
            "model",
            3,
            chunks(),
        )
    }

    #[test]
    fn immutable_model_artifact_identity_changes_projection_plan() -> Result<()> {
        let obligation = coverage_obligation("obligation-model-artifact");
        let first = derive_workspace_coverage_projection_plan(
            &obligation,
            "projection-v0",
            "chunker-v0",
            "provider",
            &format!("model:latest@sha256:{}", "aa".repeat(32)),
            3,
            chunks(),
        )?;
        let second = derive_workspace_coverage_projection_plan(
            &obligation,
            "projection-v0",
            "chunker-v0",
            "provider",
            &format!("model:latest@sha256:{}", "bb".repeat(32)),
            3,
            chunks(),
        )?;
        assert_ne!(first.plan_id, second.plan_id);
        Ok(())
    }

    #[test]
    fn receipt_and_head_cannot_substitute_payload_or_claim_authority() -> Result<()> {
        let obligation = coverage_obligation("obligation-1");
        let plan = derive_workspace_coverage_projection_plan(
            &obligation,
            "projection-v0",
            "chunker-v0",
            "provider",
            "model",
            3,
            chunks(),
        )?;
        let claim_id = uuid::Uuid::new_v4().to_string();
        let claim_epoch = 7;
        let mut receipt = WorkspaceCoverageReceipt {
            schema_version: WORKSPACE_COVERAGE_RECEIPT_SCHEMA_VERSION.into(),
            receipt_id: "receipt-1".into(),
            obligation_id: obligation.obligation_id.clone(),
            plan_id: plan.plan_id.clone(),
            collection_name: workspace_coverage_execution_collection(
                &plan.plan_id,
                &claim_id,
                claim_epoch,
            )?,
            embedding_provider_id: "provider".into(),
            embedding_model: "model".into(),
            vector_dimensions: 3,
            observed_point_count: plan.expected_point_count,
            observed_point_set_sha256: plan.expected_point_set_sha256.clone(),
            observed_at: "now".into(),
            observation_method: "qdrant_scroll_exact_point_bindings".into(),
            observed_point_binding_set_sha256: plan.point_binding_set_sha256.clone(),
            claim_id: claim_id.clone(),
            claim_epoch,
            observed_vector_binding_set_sha256: "55".repeat(32),
        };
        assert!(validate_workspace_coverage_receipt(&obligation, &plan, &receipt).is_ok());
        let mut head = WorkspaceCoverageHead {
            schema_version: WORKSPACE_COVERAGE_HEAD_SCHEMA_VERSION.into(),
            workspace_id: obligation.workspace_id.clone(),
            obligation_id: obligation.obligation_id.clone(),
            plan_id: plan.plan_id.clone(),
            receipt_id: receipt.receipt_id.clone(),
            body_observation_id: obligation.body_observation_id.clone(),
            body_generation: obligation.body_generation,
            manifest_root_sha256: obligation.manifest_root_sha256.clone(),
            claim_id,
            claim_epoch,
        };
        assert!(validate_workspace_coverage_head(&obligation, &plan, &receipt, &head).is_ok());
        receipt.observed_point_binding_set_sha256 = "44".repeat(32);
        assert!(validate_workspace_coverage_receipt(&obligation, &plan, &receipt).is_err());
        receipt.observed_point_binding_set_sha256 = plan.point_binding_set_sha256.clone();
        receipt.claim_epoch += 1;
        assert!(validate_workspace_coverage_receipt(&obligation, &plan, &receipt).is_err());
        receipt.claim_epoch = claim_epoch;
        receipt.claim_id = "not-a-uuid".into();
        assert!(validate_workspace_coverage_receipt(&obligation, &plan, &receipt).is_err());
        receipt.claim_id = head.claim_id.clone();
        head.claim_id = uuid::Uuid::new_v4().to_string();
        assert!(validate_workspace_coverage_head(&obligation, &plan, &receipt, &head).is_err());
        receipt.observation_method = "claimed_upsert_count".into();
        assert!(validate_workspace_coverage_head(&obligation, &plan, &receipt, &head).is_err());
        Ok(())
    }

    #[test]
    fn projection_plan_seals_point_set_and_rejects_cross_obligation_use() -> Result<()> {
        let mut obligation = coverage_obligation("obligation-a");
        let mut duplicate = chunks();
        duplicate[1].chunk_index = 0;
        assert!(
            derive_workspace_coverage_projection_plan(
                &obligation,
                "projection",
                "chunker",
                "provider",
                "model",
                3,
                duplicate
            )
            .is_err()
        );
        let mut invalid = chunks();
        invalid[0].byte_end = 11;
        assert!(
            derive_workspace_coverage_projection_plan(
                &obligation,
                "projection",
                "chunker",
                "provider",
                "model",
                3,
                invalid
            )
            .is_err()
        );
        let mut plan = plan(&obligation)?;
        assert!(uuid::Uuid::parse_str(&plan.planned_points[0].point_id).is_ok());
        plan.planned_points[0].chunk_sha256 = "44".repeat(32);
        assert!(validate_workspace_coverage_projection_plan(&obligation, &plan).is_err());
        obligation.obligation_id = "obligation-b".into();
        assert!(validate_workspace_coverage_projection_plan(&obligation, &plan).is_err());
        Ok(())
    }

    #[test]
    fn execution_claim_fences_collection_without_changing_plan() -> Result<()> {
        let obligation = coverage_obligation("body-policy-epoch-a");
        let first = plan(&obligation)?;
        let claim_a = uuid::Uuid::new_v4().to_string();
        let claim_b = uuid::Uuid::new_v4().to_string();
        let collection_a = workspace_coverage_execution_collection(&first.plan_id, &claim_a, 1)?;
        let collection_b = workspace_coverage_execution_collection(&first.plan_id, &claim_b, 2)?;
        assert_ne!(collection_a, collection_b);
        assert_eq!(first, plan(&obligation)?);
        assert!(collection_a.starts_with("epiphany_workspace_"));
        assert_eq!(collection_a.len(), "epiphany_workspace_".len() + 40);
        Ok(())
    }
}
