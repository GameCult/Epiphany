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
pub struct WorkspaceCoverageObligation {
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

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.workspace_coverage_projection_plan",
    schema = "WorkspaceCoverageProjectionPlan"
)]
pub struct WorkspaceCoverageProjectionPlan {
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
    pub collection_name: String,
    #[cultcache(key = 6)]
    pub embedding_provider_id: String,
    #[cultcache(key = 7)]
    pub embedding_model: String,
    #[cultcache(key = 8)]
    pub vector_dimensions: u32,
    #[cultcache(key = 9)]
    pub expected_point_ids: Vec<String>,
    #[cultcache(key = 10)]
    pub expected_point_count: u64,
    #[cultcache(key = 11)]
    pub expected_point_set_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.workspace_coverage_receipt",
    schema = "WorkspaceCoverageReceipt"
)]
pub struct WorkspaceCoverageReceipt {
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
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.workspace_coverage_head",
    schema = "WorkspaceCoverageHead"
)]
pub struct WorkspaceCoverageHead {
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
}

pub fn derive_workspace_coverage_obligation(
    runtime_store: &Path,
    basis: &RepositoryBodyObservationBasis,
    policy: &WorkspaceCoveragePolicy,
) -> Result<WorkspaceCoverageObligation> {
    let manifest = authenticated_repository_body_manifest(runtime_store, basis)?;
    derive_workspace_coverage_obligation_from_authenticated_manifest(basis, &manifest, policy)
}

fn derive_workspace_coverage_obligation_from_authenticated_manifest(
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

#[allow(clippy::too_many_arguments)]
pub fn derive_workspace_coverage_projection_plan(
    obligation: &WorkspaceCoverageObligation,
    projection_schema_version: &str,
    chunker_id: &str,
    embedding_provider_id: &str,
    embedding_model: &str,
    vector_dimensions: u32,
    expected_point_ids: Vec<String>,
) -> Result<WorkspaceCoverageProjectionPlan> {
    if obligation.schema_version != WORKSPACE_COVERAGE_OBLIGATION_SCHEMA_VERSION
        || projection_schema_version.trim().is_empty()
        || chunker_id.trim().is_empty()
        || embedding_provider_id.trim().is_empty()
        || embedding_model.trim().is_empty()
        || vector_dimensions == 0
        || !strictly_ordered_nonempty(&expected_point_ids)
        || (obligation.included_entry_count == 0) != expected_point_ids.is_empty()
    {
        bail!("workspace coverage projection plan is incomplete or noncanonical");
    }
    let expected_point_count = expected_point_ids.len() as u64;
    let expected_point_set_sha256 = digest(&expected_point_ids)?;
    let collection_namespace_hash = digest(&(
        &obligation.obligation_id,
        projection_schema_version,
        chunker_id,
        embedding_provider_id,
        embedding_model,
        vector_dimensions,
    ))?;
    let collection_name = format!("epiphany_workspace_{}", &collection_namespace_hash[..40]);
    let identity = (
        &obligation.obligation_id,
        projection_schema_version,
        chunker_id,
        &collection_name,
        embedding_provider_id,
        embedding_model,
        vector_dimensions,
        &expected_point_ids,
        &expected_point_set_sha256,
    );
    Ok(WorkspaceCoverageProjectionPlan {
        schema_version: WORKSPACE_COVERAGE_PROJECTION_PLAN_SCHEMA_VERSION.into(),
        plan_id: format!("workspace-coverage-plan:{}", digest(&identity)?),
        obligation_id: obligation.obligation_id.clone(),
        projection_schema_version: projection_schema_version.into(),
        chunker_id: chunker_id.into(),
        collection_name,
        embedding_provider_id: embedding_provider_id.into(),
        embedding_model: embedding_model.into(),
        vector_dimensions,
        expected_point_ids,
        expected_point_count,
        expected_point_set_sha256,
    })
}

pub fn validate_workspace_coverage_projection_plan(
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
        plan.expected_point_ids.clone(),
    )?;
    if *plan != expected {
        bail!("workspace coverage projection plan identity or derived fields were altered");
    }
    Ok(())
}

pub fn validate_workspace_coverage_receipt(
    obligation: &WorkspaceCoverageObligation,
    plan: &WorkspaceCoverageProjectionPlan,
    receipt: &WorkspaceCoverageReceipt,
) -> Result<()> {
    validate_workspace_coverage_projection_plan(obligation, plan)?;
    if receipt.schema_version != WORKSPACE_COVERAGE_RECEIPT_SCHEMA_VERSION
        || receipt.obligation_id != obligation.obligation_id
        || receipt.plan_id != plan.plan_id
        || receipt.receipt_id.trim().is_empty()
        || receipt.collection_name != plan.collection_name
        || receipt.embedding_provider_id != plan.embedding_provider_id
        || receipt.embedding_model != plan.embedding_model
        || receipt.vector_dimensions != plan.vector_dimensions
        || receipt.observed_at.trim().is_empty()
        || receipt.observation_method != "qdrant_scroll_exact_point_ids"
    {
        bail!("workspace coverage receipt lacks exact observed-index authority");
    }
    if receipt.observed_point_count != plan.expected_point_count
        || receipt.observed_point_set_sha256 != plan.expected_point_set_sha256
    {
        bail!("workspace coverage receipt does not prove the exact expected point set");
    }
    Ok(())
}

pub fn validate_workspace_coverage_head(
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
    {
        bail!("workspace coverage head does not join one obligation to its exact receipt");
    }
    Ok(())
}

fn strictly_ordered_nonempty(values: &[String]) -> bool {
    values.iter().all(|value| !value.trim().is_empty())
        && values.windows(2).all(|pair| pair[0] < pair[1])
}

fn classify(
    entry: &RepositoryBodyManifestEntry,
    policy: &WorkspaceCoveragePolicy,
) -> WorkspaceCoverageClassification {
    let disposition = match entry.kind.as_str() {
        "gitlink_nonrecursive" => WorkspaceCoverageDisposition::ExcludeGitlink,
        "symlink_target" => WorkspaceCoverageDisposition::ExcludeSymlink,
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
            obligation.classifications[1].disposition,
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

    #[test]
    fn receipt_cannot_convert_counts_or_unobserved_points_into_coverage() -> Result<()> {
        let obligation = WorkspaceCoverageObligation {
            schema_version: WORKSPACE_COVERAGE_OBLIGATION_SCHEMA_VERSION.into(),
            obligation_id: "obligation-1".into(),
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
            classifications: vec![],
            included_entry_count: 2,
        };
        let points = vec!["point-a".to_string(), "point-b".to_string()];
        let plan = derive_workspace_coverage_projection_plan(
            &obligation,
            "projection-v0",
            "chunker-v0",
            "provider",
            "model",
            3,
            points.clone(),
        )?;
        let mut receipt = WorkspaceCoverageReceipt {
            schema_version: WORKSPACE_COVERAGE_RECEIPT_SCHEMA_VERSION.into(),
            receipt_id: "receipt-1".into(),
            obligation_id: obligation.obligation_id.clone(),
            plan_id: plan.plan_id.clone(),
            collection_name: plan.collection_name.clone(),
            embedding_provider_id: "provider".into(),
            embedding_model: "model".into(),
            vector_dimensions: 3,
            observed_point_count: 2,
            observed_point_set_sha256: digest(&points)?,
            observed_at: "now".into(),
            observation_method: "claimed_upsert_count".into(),
        };
        assert!(validate_workspace_coverage_receipt(&obligation, &plan, &receipt).is_err());
        receipt.observation_method = "qdrant_scroll_exact_point_ids".into();
        assert!(validate_workspace_coverage_receipt(&obligation, &plan, &receipt).is_ok());
        let mut head = WorkspaceCoverageHead {
            schema_version: WORKSPACE_COVERAGE_HEAD_SCHEMA_VERSION.into(),
            workspace_id: obligation.workspace_id.clone(),
            obligation_id: obligation.obligation_id.clone(),
            plan_id: plan.plan_id.clone(),
            receipt_id: receipt.receipt_id.clone(),
        };
        assert!(validate_workspace_coverage_head(&obligation, &plan, &receipt, &head).is_ok());
        head.receipt_id = "some-other-receipt".into();
        assert!(validate_workspace_coverage_head(&obligation, &plan, &receipt, &head).is_err());
        head.receipt_id = receipt.receipt_id.clone();
        receipt.observation_method = "claimed_upsert_count".into();
        assert!(validate_workspace_coverage_head(&obligation, &plan, &receipt, &head).is_err());
        receipt.observed_point_count = 1;
        assert!(validate_workspace_coverage_receipt(&obligation, &plan, &receipt).is_err());
        Ok(())
    }

    #[test]
    fn projection_plan_seals_point_set_and_rejects_cross_obligation_use() -> Result<()> {
        let mut obligation = WorkspaceCoverageObligation {
            schema_version: WORKSPACE_COVERAGE_OBLIGATION_SCHEMA_VERSION.into(),
            obligation_id: "obligation-a".into(),
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
            classifications: vec![],
            included_entry_count: 1,
        };
        assert!(
            derive_workspace_coverage_projection_plan(
                &obligation,
                "projection",
                "chunker",
                "provider",
                "model",
                3,
                vec!["duplicate".into(), "duplicate".into()]
            )
            .is_err()
        );
        assert!(
            derive_workspace_coverage_projection_plan(
                &obligation,
                "projection",
                "chunker",
                "provider",
                "model",
                3,
                vec!["z".into(), "a".into()]
            )
            .is_err()
        );
        let mut plan = derive_workspace_coverage_projection_plan(
            &obligation,
            "projection",
            "chunker",
            "provider",
            "model",
            3,
            vec!["a".into(), "z".into()],
        )?;
        plan.expected_point_ids[1] = "y".into();
        assert!(validate_workspace_coverage_projection_plan(&obligation, &plan).is_err());
        obligation.obligation_id = "obligation-b".into();
        assert!(validate_workspace_coverage_projection_plan(&obligation, &plan).is_err());
        Ok(())
    }

    #[test]
    fn collection_namespace_is_derived_from_body_and_embedding_authority() -> Result<()> {
        let obligation = WorkspaceCoverageObligation {
            schema_version: WORKSPACE_COVERAGE_OBLIGATION_SCHEMA_VERSION.into(),
            obligation_id: "body-policy-epoch-a".into(),
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
            classifications: vec![],
            included_entry_count: 1,
        };
        let point_ids = vec!["point".into()];
        let first = derive_workspace_coverage_projection_plan(
            &obligation,
            "projection",
            "chunker",
            "provider",
            "model",
            3,
            point_ids.clone(),
        )?;
        let mut other_obligation = obligation.clone();
        other_obligation.obligation_id = "body-policy-epoch-b".into();
        let other_body = derive_workspace_coverage_projection_plan(
            &other_obligation,
            "projection",
            "chunker",
            "provider",
            "model",
            3,
            point_ids.clone(),
        )?;
        let other_embedding = derive_workspace_coverage_projection_plan(
            &obligation,
            "projection",
            "chunker",
            "provider",
            "model-v2",
            4,
            point_ids,
        )?;
        assert_ne!(first.collection_name, other_body.collection_name);
        assert_ne!(first.plan_id, other_body.plan_id);
        assert_ne!(first.collection_name, other_embedding.collection_name);
        assert_ne!(first.plan_id, other_embedding.plan_id);
        assert!(first.collection_name.starts_with("epiphany_workspace_"));
        assert_eq!(
            first.collection_name.len(),
            "epiphany_workspace_".len() + 40
        );
        Ok(())
    }
}
