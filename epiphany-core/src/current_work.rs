use anyhow::{Result, anyhow};
use cultcache_rs::{CultCache, DatabaseEntry};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::{EpiphanyMindDocumentVersion, EpiphanyRepoModelBasis, RepositoryBodyObservationBasis};

pub const BODY_MODELING_DECISION_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.mind.body_modeling_decision.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyBodyModelingWorkProjection {
    pub work_id: String,
    pub runtime_id: String,
    pub body_basis: RepositoryBodyObservationBasis,
    pub repo_model_basis: EpiphanyRepoModelBasis,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.body_modeling_decision.v1",
    schema = "EpiphanyBodyModelingDecisionReceipt"
)]
pub struct EpiphanyBodyModelingDecisionReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub work_id: String,
    #[cultcache(key = 2)]
    pub runtime_id: String,
    #[cultcache(key = 3)]
    pub body_basis: RepositoryBodyObservationBasis,
    #[cultcache(key = 4)]
    pub repo_model_projection_digest: String,
    #[cultcache(key = 5)]
    pub repo_model_source_documents: Vec<EpiphanyMindDocumentVersion>,
    #[cultcache(key = 6)]
    pub decision_context_id: String,
    #[cultcache(key = 7)]
    pub result_id: String,
    #[cultcache(key = 8)]
    pub job_id: String,
    #[cultcache(key = 9)]
    pub disposition: String,
    #[cultcache(key = 10)]
    pub decided_at: String,
}

impl EpiphanyBodyModelingWorkProjection {
    pub fn derive(
        runtime_id: impl Into<String>,
        body_basis: RepositoryBodyObservationBasis,
        repo_model_basis: EpiphanyRepoModelBasis,
    ) -> Result<Self> {
        let runtime_id = runtime_id.into();
        if runtime_id.trim().is_empty()
            || body_basis.runtime_id != runtime_id
            || body_basis.observation_id.trim().is_empty()
        {
            return Err(anyhow!("Body Modeling work has invalid runtime ownership"));
        }
        repo_model_basis.validate()?;
        let identity = rmp_serde::to_vec_named(&(
            BODY_MODELING_DECISION_RECEIPT_SCHEMA_VERSION,
            runtime_id.as_str(),
            body_basis.observation_id.as_str(),
            body_basis.generation,
            body_basis.body_binding_sha256.as_str(),
            body_basis.manifest_root_sha256.as_str(),
        ))?;
        Ok(Self {
            work_id: format!("body-modeling-{:x}", Sha256::digest(identity)),
            runtime_id,
            body_basis,
            repo_model_basis,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if Self::derive(
            self.runtime_id.clone(),
            self.body_basis.clone(),
            self.repo_model_basis.clone(),
        )? != *self
        {
            return Err(anyhow!("Body Modeling work identity is not canonical"));
        }
        Ok(())
    }
}

impl EpiphanyBodyModelingDecisionReceipt {
    pub fn validate(&self, work: &EpiphanyBodyModelingWorkProjection) -> Result<()> {
        work.validate()?;
        if self.schema_version != BODY_MODELING_DECISION_RECEIPT_SCHEMA_VERSION
            || self.work_id != work.work_id
            || self.runtime_id != work.runtime_id
            || self.body_basis != work.body_basis
            || self.repo_model_projection_digest != work.repo_model_basis.projection_digest
            || self.repo_model_source_documents != work.repo_model_basis.source_documents
            || [
                self.decision_context_id.as_str(),
                self.result_id.as_str(),
                self.job_id.as_str(),
            ]
            .into_iter()
            .any(str::is_empty)
            || !matches!(
                self.disposition.as_str(),
                "checkpoint-ready" | "regather-needed"
            )
            || chrono::DateTime::parse_from_rfc3339(&self.decided_at).is_err()
        {
            return Err(anyhow!(
                "Body Modeling decision does not bind its exact work projection"
            ));
        }
        Ok(())
    }
}

pub(crate) fn register_current_work_types(cache: &mut CultCache) -> Result<()> {
    cache.register_entry_type::<EpiphanyBodyModelingDecisionReceipt>()?;
    Ok(())
}

pub fn current_body_modeling_work(
    store_path: impl AsRef<Path>,
) -> Result<EpiphanyBodyModelingWorkProjection> {
    let body_basis = crate::observe_runtime_repository_body_basis(store_path.as_ref())?;
    let repo_model_basis = crate::assemble_repo_model_view(store_path.as_ref())?.reasoning_basis();
    EpiphanyBodyModelingWorkProjection::derive(
        body_basis.runtime_id.clone(),
        body_basis,
        repo_model_basis,
    )
}

pub fn unresolved_body_modeling_work(
    store_path: impl AsRef<Path>,
) -> Result<Option<EpiphanyBodyModelingWorkProjection>> {
    let store_path = store_path.as_ref();
    let work = current_body_modeling_work(store_path)?;
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let decision = cache.get::<EpiphanyBodyModelingDecisionReceipt>(&work.work_id)?;
    resolve_body_modeling_work(work, decision)
}

fn resolve_body_modeling_work(
    work: EpiphanyBodyModelingWorkProjection,
    receipt: Option<EpiphanyBodyModelingDecisionReceipt>,
) -> Result<Option<EpiphanyBodyModelingWorkProjection>> {
    let Some(receipt) = receipt else {
        return Ok(Some(work));
    };
    let admitted_work = EpiphanyBodyModelingWorkProjection::derive(
        receipt.runtime_id.clone(),
        receipt.body_basis.clone(),
        EpiphanyRepoModelBasis {
            projection_digest: receipt.repo_model_projection_digest.clone(),
            source_documents: receipt.repo_model_source_documents.clone(),
        },
    )?;
    receipt.validate(&admitted_work)?;
    if admitted_work.work_id != work.work_id || admitted_work.body_basis != work.body_basis {
        return Err(anyhow!(
            "Body Modeling decision collides with current Body work identity"
        ));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basis(observation_id: &str, generation: u64) -> RepositoryBodyObservationBasis {
        RepositoryBodyObservationBasis {
            schema_version: "epiphany.repository_body.observation_basis.v0".into(),
            workspace_id: "workspace".into(),
            swarm_id: "swarm".into(),
            runtime_id: "runtime".into(),
            scope: "git_worktree".into(),
            body_binding_sha256: "sha256:body".into(),
            observation_id: observation_id.into(),
            generation,
            manifest_root_sha256: "sha256:manifest".into(),
            scan_started_at: "2026-08-16T00:00:00Z".into(),
            scan_finished_at: "2026-08-16T00:00:01Z".into(),
        }
    }

    fn model_basis_with_payload(payload_msgpack: Vec<u8>) -> EpiphanyRepoModelBasis {
        let source_documents = vec![EpiphanyMindDocumentVersion {
            store_id: "epiphany-mind".into(),
            document_type: "epiphany.mind.repo_model.identity.v1".into(),
            document_key: "repo-model".into(),
            schema_id: None,
            payload_sha256: format!("sha256:{:x}", Sha256::digest(&payload_msgpack)),
            payload_msgpack,
        }];
        EpiphanyRepoModelBasis {
            projection_digest: format!(
                "sha256:{:x}",
                Sha256::digest(rmp_serde::to_vec_named(&source_documents).unwrap())
            ),
            source_documents,
        }
    }

    fn model_basis() -> EpiphanyRepoModelBasis {
        model_basis_with_payload(vec![1])
    }

    #[test]
    fn body_generation_owns_work_identity_while_repo_model_is_projection_cargo() -> Result<()> {
        let first = EpiphanyBodyModelingWorkProjection::derive(
            "runtime",
            basis("observation-1", 1),
            model_basis(),
        )?;
        let changed_model = model_basis_with_payload(vec![2]);
        let projected_again = EpiphanyBodyModelingWorkProjection::derive(
            "runtime",
            basis("observation-1", 1),
            changed_model,
        )?;
        let next_body = EpiphanyBodyModelingWorkProjection::derive(
            "runtime",
            basis("observation-2", 2),
            model_basis(),
        )?;
        assert_eq!(first.work_id, projected_again.work_id);
        assert_ne!(first.repo_model_basis, projected_again.repo_model_basis);
        assert_ne!(first.work_id, next_body.work_id);
        Ok(())
    }

    #[test]
    fn exact_body_decision_fulfills_work_after_its_repo_model_changes() -> Result<()> {
        let admitted_work = EpiphanyBodyModelingWorkProjection::derive(
            "runtime",
            basis("observation-1", 1),
            model_basis(),
        )?;
        let receipt = EpiphanyBodyModelingDecisionReceipt {
            schema_version: BODY_MODELING_DECISION_RECEIPT_SCHEMA_VERSION.into(),
            work_id: admitted_work.work_id.clone(),
            runtime_id: admitted_work.runtime_id.clone(),
            body_basis: admitted_work.body_basis.clone(),
            repo_model_projection_digest: admitted_work.repo_model_basis.projection_digest.clone(),
            repo_model_source_documents: admitted_work.repo_model_basis.source_documents.clone(),
            decision_context_id: "decision-1".into(),
            result_id: "result-1".into(),
            job_id: "job-1".into(),
            disposition: "checkpoint-ready".into(),
            decided_at: "2026-08-16T00:01:00Z".into(),
        };
        let changed_model = model_basis_with_payload(vec![2]);
        let current_work = EpiphanyBodyModelingWorkProjection::derive(
            "runtime",
            basis("observation-1", 1),
            changed_model,
        )?;
        assert!(resolve_body_modeling_work(current_work, Some(receipt))?.is_none());
        Ok(())
    }
}
