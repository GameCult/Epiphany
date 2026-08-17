use anyhow::{Result, anyhow};
use cultcache_rs::{CultCache, CultCacheEnvelope, DatabaseEntry};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::{
    EpiphanyMindDocumentVersion, EpiphanyRepoModelBasis, RepoFrontierPlanningLifecycleStage,
    RepoFrontierProposalModelingRequest, RepoFrontierResearchContinuationAction,
    RepositoryBodyObservationBasis,
};

pub const BODY_MODELING_DECISION_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.mind.body_modeling_decision.v1";
pub const BODY_MODELING_LAUNCH_BINDING_SCHEMA_VERSION: &str =
    "epiphany.runtime.body_modeling_launch.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyCurrentWorkProjection {
    pub mind_projection_digest: String,
    pub body_modeling: Option<EpiphanyBodyModelingWorkProjection>,
    pub body_modeling_action: Option<EpiphanyAgentPassContinuationAction>,
    pub research_continuation_action: Option<RepoFrontierResearchContinuationAction>,
    pub frontier_planning_stage: RepoFrontierPlanningLifecycleStage,
    pub proposal_modeling: Option<EpiphanyProposalModelingWorkProjection>,
    pub frontier_verdict_modeling: Option<EpiphanyFrontierVerdictModelingWorkProjection>,
    pub verification: Option<EpiphanyVerificationWorkProjection>,
    pub hands_frontier_ready: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyAgentPassContinuationAction {
    Launch,
    Wait,
    Review,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyProposalModelingWorkProjection {
    pub request: RepoFrontierProposalModelingRequest,
    pub action: EpiphanyAgentPassContinuationAction,
    pub job_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyFrontierVerdictModelingWorkProjection {
    pub request: crate::RepoFrontierModelingRequest,
    pub action: EpiphanyAgentPassContinuationAction,
    pub job_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyVerificationWorkProjection {
    pub request: crate::RepoFrontierVerificationRequest,
    pub action: EpiphanyAgentPassContinuationAction,
    pub job_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyBodyModelingWorkProjection {
    pub work_id: String,
    pub runtime_id: String,
    pub body_basis: RepositoryBodyObservationBasis,
    pub repo_model_basis: EpiphanyRepoModelBasis,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.body_modeling_launch.v1",
    schema = "EpiphanyBodyModelingLaunchBinding"
)]
pub struct EpiphanyBodyModelingLaunchBinding {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub binding_record_id: String,
    #[cultcache(key = 2)]
    pub work_id: String,
    #[cultcache(key = 3)]
    pub job_id: String,
    #[cultcache(key = 4)]
    pub attempt_ordinal: u64,
    #[cultcache(key = 5)]
    pub body_document_sha256: String,
    #[cultcache(key = 6)]
    pub observed_repo_model_projection_digest: String,
    #[cultcache(key = 7)]
    pub launched_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpiphanyBodyModelingLaunchOptions {
    pub job_id: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpiphanyProposalModelingLaunchOptions {
    pub created_at: String,
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
                "modeled" | "checkpoint-ready" | "regather-needed"
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
    cache.register_entry_type::<EpiphanyBodyModelingLaunchBinding>()?;
    Ok(())
}

pub(crate) fn body_modeling_decision_envelope(
    store_path: &Path,
    result: &crate::EpiphanyRuntimeRoleWorkerResult,
    disposition: &str,
    decided_at: &str,
) -> Result<CultCacheEnvelope> {
    let body_basis = result
        .repository_body_observation_basis
        .clone()
        .ok_or_else(|| anyhow!("Body Modeling result has no Body observation basis"))?;
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let context = cache
        .get::<crate::EpiphanyDecisionContext>(&result.decision_context_id)?
        .ok_or_else(|| anyhow!("Body Modeling result has no decision context"))?;
    let reasoning_basis = cache
        .get::<crate::EpiphanyReasoningBasis>(&context.basis_id)?
        .ok_or_else(|| anyhow!("Body Modeling decision context lost its reasoning basis"))?;
    context.validate(&reasoning_basis)?;
    let crate::EpiphanyReasoningProjection::RolePass(role_projection) =
        reasoning_basis.projection()?
    else {
        return Err(anyhow!(
            "Body Modeling decision has no sealed role projection"
        ));
    };
    if role_projection.authority.role_id.to_ascii_lowercase() != "modeling"
        || role_projection
            .authority
            .repository_body_observation_basis
            .as_ref()
            != Some(&body_basis)
    {
        return Err(anyhow!(
            "Body Modeling decision does not bind its projected organ and Body basis"
        ));
    }
    let work = EpiphanyBodyModelingWorkProjection::derive(
        body_basis.runtime_id.clone(),
        body_basis.clone(),
        crate::reasoning_repo_model_basis(&reasoning_basis)?,
    )?;
    let receipt = EpiphanyBodyModelingDecisionReceipt {
        schema_version: BODY_MODELING_DECISION_RECEIPT_SCHEMA_VERSION.to_string(),
        work_id: work.work_id.clone(),
        runtime_id: work.runtime_id.clone(),
        body_basis,
        repo_model_projection_digest: work.repo_model_basis.projection_digest.clone(),
        repo_model_source_documents: work.repo_model_basis.source_documents.clone(),
        decision_context_id: result.decision_context_id.clone(),
        result_id: result.result_id.clone(),
        job_id: result.job_id.clone(),
        disposition: disposition.to_string(),
        decided_at: decided_at.to_string(),
    };
    receipt.validate(&work)?;
    Ok(cache.prepare_entry(&receipt.work_id, &receipt)?.0)
}

pub fn current_body_modeling_work(
    store_path: impl AsRef<Path>,
) -> Result<EpiphanyBodyModelingWorkProjection> {
    let body_basis = crate::current_mind_repository_body_observation(store_path.as_ref())?
        .ok_or_else(|| anyhow!("Mind has no admitted repository Body observation"))?;
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

pub fn project_current_work(store_path: impl AsRef<Path>) -> Result<EpiphanyCurrentWorkProjection> {
    let store_path = store_path.as_ref();
    let mind = crate::assemble_mind_view(store_path)?;
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let (body_modeling, body_modeling_action) = match (
        mind.repository_body_observation.clone(),
        mind.repo_model.as_ref(),
    ) {
        (Some(body), Some(repo_model)) => {
            let work = EpiphanyBodyModelingWorkProjection::derive(
                mind.runtime_id.clone(),
                body,
                repo_model.reasoning_basis(),
            )?;
            let unresolved = resolve_body_modeling_work(
                work.clone(),
                cache.get::<EpiphanyBodyModelingDecisionReceipt>(&work.work_id)?,
            )?;
            match unresolved {
                Some(work) => {
                    let action = body_modeling_continuation_action(&cache, &work.work_id)?;
                    (
                        (action == EpiphanyAgentPassContinuationAction::Launch).then_some(work),
                        Some(action),
                    )
                }
                None => (None, None),
            }
        }
        (None, None) | (Some(_), None) => (None, None),
        (None, Some(_)) => {
            return Err(anyhow!(
                "current work has a RepoModel but no admitted repository Body observation"
            ));
        }
    };
    Ok(EpiphanyCurrentWorkProjection {
        mind_projection_digest: mind.projection_digest,
        body_modeling,
        body_modeling_action,
        research_continuation_action: crate::runtime_repo_frontier_research_lifecycle(store_path)?
            .continuation_action(),
        frontier_planning_stage: crate::runtime_repo_frontier_planning_lifecycle(store_path)?.stage,
        proposal_modeling: current_proposal_modeling_work(&cache)?,
        frontier_verdict_modeling: current_frontier_verdict_modeling_work(&cache)?,
        verification: current_verification_work(&cache)?,
        hands_frontier_ready: crate::runtime_has_actionable_hands_frontier(store_path)?,
    })
}

fn verification_attempt_ordinal(request_id: &str, job_id: &str) -> Result<usize> {
    let prefix = format!("frontier-verification-{request_id}-attempt-");
    job_id
        .strip_prefix(&prefix)
        .ok_or_else(|| anyhow!("frontier Verification job identity is not canonical"))?
        .parse::<usize>()
        .map_err(|_| anyhow!("frontier Verification attempt ordinal is invalid"))
}

fn current_verification_work(
    cache: &CultCache,
) -> Result<Option<EpiphanyVerificationWorkProjection>> {
    let mut requests = cache.get_all::<crate::RepoFrontierVerificationRequest>()?;
    requests.sort_by(|left, right| left.request_id.cmp(&right.request_id));
    let launches = cache.get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?;
    let receipts = cache.get_all::<crate::EpiphanyMindCommitReceipt>()?;
    for request in requests {
        crate::runtime_spine::validate_repo_frontier_verification_request_intrinsic(&request)?;
        let mut request_launches = launches
            .iter()
            .filter(|launch| {
                launch.repo_frontier_verification_request_id.as_deref()
                    == Some(request.request_id.as_str())
            })
            .map(|launch| {
                Ok((
                    verification_attempt_ordinal(&request.request_id, &launch.job_id)?,
                    launch,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        request_launches.sort_by_key(|(ordinal, _)| *ordinal);
        if let Some((_, launch)) = request_launches.last()
            && let Some(result) =
                cache.get::<crate::EpiphanyRuntimeRoleWorkerResult>(&launch.job_id)?
            && receipts.iter().any(|receipt| {
                receipt.invariant_owner == "Soul.verification"
                    && matches!(
                        &receipt.authority,
                        crate::EpiphanyMindCommitAuthority::ModelDecisionContext {
                            decision_context_id
                        } if decision_context_id == &result.decision_context_id
                    )
            })
        {
            continue;
        }
        let Some((_, launch)) = request_launches.last() else {
            if crate::runtime_spine::verification_frontier_is_current(cache, &request)? {
                return Ok(Some(EpiphanyVerificationWorkProjection {
                    request,
                    action: EpiphanyAgentPassContinuationAction::Launch,
                    job_id: None,
                }));
            }
            continue;
        };
        let result = cache.get::<crate::EpiphanyRuntimeRoleWorkerResult>(&launch.job_id)?;
        if let Some(result) = result.as_ref()
            && (result.verification_request_id.as_deref() != Some(request.request_id.as_str())
                || result.frontier_route_id.as_deref() != Some(request.route_id.as_str()))
        {
            return Err(anyhow!(
                "frontier Verification result crossed request authority"
            ));
        }
        let job = cache
            .get::<crate::EpiphanyRuntimeJob>(&launch.job_id)?
            .ok_or_else(|| anyhow!("frontier Verification launch lost its runtime job"))?;
        let action = match job.status {
            crate::EpiphanyRuntimeJobStatus::Failed
            | crate::EpiphanyRuntimeJobStatus::Cancelled
                if crate::runtime_spine::verification_frontier_is_current(cache, &request)? =>
            {
                EpiphanyAgentPassContinuationAction::Launch
            }
            crate::EpiphanyRuntimeJobStatus::Completed if result.is_some() => {
                EpiphanyAgentPassContinuationAction::Review
            }
            _ => EpiphanyAgentPassContinuationAction::Wait,
        };
        return Ok(Some(EpiphanyVerificationWorkProjection {
            request,
            action,
            job_id: Some(launch.job_id.clone()),
        }));
    }
    Ok(None)
}

fn frontier_verdict_attempt_ordinal(request_id: &str, job_id: &str) -> Result<usize> {
    let prefix = format!("frontier-verdict-modeling-{request_id}-attempt-");
    job_id
        .strip_prefix(&prefix)
        .ok_or_else(|| anyhow!("frontier verdict Modeling job identity is not canonical"))?
        .parse::<usize>()
        .map_err(|_| anyhow!("frontier verdict Modeling attempt ordinal is invalid"))
}

fn current_frontier_verdict_modeling_work(
    cache: &CultCache,
) -> Result<Option<EpiphanyFrontierVerdictModelingWorkProjection>> {
    let mut requests = cache.get_all::<crate::RepoFrontierModelingRequest>()?;
    requests.sort_by(|left, right| left.request_id.cmp(&right.request_id));
    let launches = cache.get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?;
    let receipts = cache.get_all::<crate::EpiphanyMindCommitReceipt>()?;
    for request in requests {
        let mut request_launches = launches
            .iter()
            .filter(|launch| {
                launch.repo_frontier_modeling_request_id.as_deref()
                    == Some(request.request_id.as_str())
            })
            .map(|launch| {
                Ok((
                    frontier_verdict_attempt_ordinal(&request.request_id, &launch.job_id)?,
                    launch,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        request_launches.sort_by_key(|(ordinal, _)| *ordinal);
        if let Some((_, launch)) = request_launches.last()
            && let Some(result) =
                cache.get::<crate::EpiphanyRuntimeRoleWorkerResult>(&launch.job_id)?
            && receipts.iter().any(|receipt| {
                receipt.invariant_owner == "Modeling.frontier_verdict"
                    && matches!(
                        &receipt.authority,
                        crate::EpiphanyMindCommitAuthority::ModelDecisionContext {
                            decision_context_id
                        } if decision_context_id == &result.decision_context_id
                    )
            })
        {
            continue;
        }
        crate::runtime_spine::validate_repo_frontier_modeling_request(cache, &request)?;
        let Some((_, launch)) = request_launches.last() else {
            return Ok(Some(EpiphanyFrontierVerdictModelingWorkProjection {
                request,
                action: EpiphanyAgentPassContinuationAction::Launch,
                job_id: None,
            }));
        };
        let result = cache.get::<crate::EpiphanyRuntimeRoleWorkerResult>(&launch.job_id)?;
        if let Some(result) = result.as_ref()
            && result.repo_frontier_modeling_request_id.as_deref()
                != Some(request.request_id.as_str())
        {
            return Err(anyhow!(
                "frontier verdict Modeling result crossed request authority"
            ));
        }
        let job = cache
            .get::<crate::EpiphanyRuntimeJob>(&launch.job_id)?
            .ok_or_else(|| anyhow!("frontier verdict Modeling launch lost its runtime job"))?;
        let action = match job.status {
            crate::EpiphanyRuntimeJobStatus::Failed
            | crate::EpiphanyRuntimeJobStatus::Cancelled => {
                EpiphanyAgentPassContinuationAction::Launch
            }
            crate::EpiphanyRuntimeJobStatus::Completed if result.is_some() => {
                EpiphanyAgentPassContinuationAction::Review
            }
            _ => EpiphanyAgentPassContinuationAction::Wait,
        };
        return Ok(Some(EpiphanyFrontierVerdictModelingWorkProjection {
            request,
            action,
            job_id: Some(launch.job_id.clone()),
        }));
    }
    Ok(None)
}

fn body_modeling_continuation_action(
    cache: &CultCache,
    work_id: &str,
) -> Result<EpiphanyAgentPassContinuationAction> {
    let mut bindings = cache
        .get_all::<EpiphanyBodyModelingLaunchBinding>()?
        .into_iter()
        .filter(|binding| binding.work_id == work_id)
        .collect::<Vec<_>>();
    bindings.sort_by_key(|binding| binding.attempt_ordinal);
    let Some(binding) = bindings.last() else {
        return Ok(EpiphanyAgentPassContinuationAction::Launch);
    };
    let job = cache
        .get::<crate::EpiphanyRuntimeJob>(&binding.job_id)?
        .ok_or_else(|| anyhow!("Body Modeling launch binding lost its runtime job"))?;
    Ok(match job.status {
        crate::EpiphanyRuntimeJobStatus::Failed | crate::EpiphanyRuntimeJobStatus::Cancelled => {
            EpiphanyAgentPassContinuationAction::Launch
        }
        crate::EpiphanyRuntimeJobStatus::Completed => {
            if cache
                .get::<crate::EpiphanyRuntimeRoleWorkerResult>(&binding.job_id)?
                .is_some()
            {
                EpiphanyAgentPassContinuationAction::Review
            } else {
                EpiphanyAgentPassContinuationAction::Wait
            }
        }
        _ => EpiphanyAgentPassContinuationAction::Wait,
    })
}

fn current_proposal_modeling_work(
    cache: &CultCache,
) -> Result<Option<EpiphanyProposalModelingWorkProjection>> {
    let mut requests = cache.get_all::<RepoFrontierProposalModelingRequest>()?;
    requests.sort_by(|left, right| {
        left.selected_at
            .cmp(&right.selected_at)
            .then_with(|| left.request_id.cmp(&right.request_id))
    });
    let bindings = cache.get_all::<crate::RepoFrontierProposalModelingLaunchBinding>()?;
    let receipts = cache.get_all::<crate::EpiphanyMindCommitReceipt>()?;
    for request in requests {
        crate::runtime_spine::validate_repo_frontier_proposal_modeling_request(&request)?;
        let mut request_bindings = bindings
            .iter()
            .filter(|binding| binding.proposal_modeling_request_id == request.request_id)
            .collect::<Vec<_>>();
        request_bindings.sort_by(|left, right| {
            left.launched_at
                .cmp(&right.launched_at)
                .then_with(|| left.job_id.cmp(&right.job_id))
        });
        let Some(binding) = request_bindings.last() else {
            return Ok(Some(EpiphanyProposalModelingWorkProjection {
                request,
                action: EpiphanyAgentPassContinuationAction::Launch,
                job_id: None,
            }));
        };
        let result = cache.get::<crate::EpiphanyRuntimeRoleWorkerResult>(&binding.job_id)?;
        if let Some(result) = result.as_ref() {
            if result.proposal_modeling_request_id.as_deref() != Some(request.request_id.as_str()) {
                return Err(anyhow!(
                    "proposal Modeling launch result crossed request authority"
                ));
            }
            if receipts.iter().any(|receipt| {
                receipt.invariant_owner == "Modeling.proposal_frontier"
                    && matches!(
                        &receipt.authority,
                        crate::EpiphanyMindCommitAuthority::ModelDecisionContext {
                            decision_context_id
                        } if decision_context_id == &result.decision_context_id
                    )
            }) {
                continue;
            }
        }
        let job = cache
            .get::<crate::EpiphanyRuntimeJob>(&binding.job_id)?
            .ok_or_else(|| anyhow!("proposal Modeling launch binding lost its runtime job"))?;
        let action = match job.status {
            crate::EpiphanyRuntimeJobStatus::Failed
            | crate::EpiphanyRuntimeJobStatus::Cancelled => {
                EpiphanyAgentPassContinuationAction::Launch
            }
            crate::EpiphanyRuntimeJobStatus::Completed if result.is_some() => {
                crate::runtime_spine::validate_proposal_modeling_worker_fulfillment(
                    cache,
                    result.as_ref().expect("checked terminal result"),
                )?;
                EpiphanyAgentPassContinuationAction::Review
            }
            _ => EpiphanyAgentPassContinuationAction::Wait,
        };
        return Ok(Some(EpiphanyProposalModelingWorkProjection {
            request,
            action,
            job_id: Some(binding.job_id.clone()),
        }));
    }
    Ok(None)
}

pub fn body_modeling_continuation_action_for_job(
    store_path: impl AsRef<Path>,
    job_id: &str,
) -> Result<Option<EpiphanyAgentPassContinuationAction>> {
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let bindings = cache
        .get_all::<EpiphanyBodyModelingLaunchBinding>()?
        .into_iter()
        .filter(|binding| binding.job_id == job_id)
        .collect::<Vec<_>>();
    let [binding] = bindings.as_slice() else {
        if bindings.is_empty() {
            return Ok(None);
        }
        return Err(anyhow!(
            "runtime job has multiple Body Modeling launch bindings"
        ));
    };
    if cache
        .get::<EpiphanyBodyModelingDecisionReceipt>(&binding.work_id)?
        .is_some()
    {
        return Ok(None);
    }
    Ok(Some(body_modeling_continuation_action(
        &cache,
        &binding.work_id,
    )?))
}

pub fn current_body_modeling_review_job_id(store_path: impl AsRef<Path>) -> Result<Option<String>> {
    let store_path = store_path.as_ref();
    if crate::project_current_work(store_path)?.body_modeling_action
        != Some(EpiphanyAgentPassContinuationAction::Review)
    {
        return Ok(None);
    }
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let work = current_body_modeling_work(store_path)?;
    let mut bindings = cache
        .get_all::<EpiphanyBodyModelingLaunchBinding>()?
        .into_iter()
        .filter(|binding| binding.work_id == work.work_id)
        .collect::<Vec<_>>();
    bindings.sort_by_key(|binding| binding.attempt_ordinal);
    Ok(bindings.last().map(|binding| binding.job_id.clone()))
}

pub fn proposal_modeling_continuation_action_for_job(
    store_path: impl AsRef<Path>,
    job_id: &str,
) -> Result<Option<EpiphanyAgentPassContinuationAction>> {
    Ok(project_current_work(store_path)?
        .proposal_modeling
        .filter(|work| work.job_id.as_deref() == Some(job_id))
        .map(|work| work.action))
}

pub fn current_proposal_modeling_review_job_id(
    store_path: impl AsRef<Path>,
) -> Result<Option<String>> {
    Ok(project_current_work(store_path)?
        .proposal_modeling
        .filter(|work| work.action == EpiphanyAgentPassContinuationAction::Review)
        .and_then(|work| work.job_id))
}

pub fn frontier_verdict_modeling_continuation_action_for_job(
    store_path: impl AsRef<Path>,
    job_id: &str,
) -> Result<Option<EpiphanyAgentPassContinuationAction>> {
    Ok(project_current_work(store_path)?
        .frontier_verdict_modeling
        .filter(|work| work.job_id.as_deref() == Some(job_id))
        .map(|work| work.action))
}

pub fn current_frontier_verdict_modeling_review_job_id(
    store_path: impl AsRef<Path>,
) -> Result<Option<String>> {
    Ok(project_current_work(store_path)?
        .frontier_verdict_modeling
        .filter(|work| work.action == EpiphanyAgentPassContinuationAction::Review)
        .and_then(|work| work.job_id))
}

pub fn current_frontier_research_review_job_id(
    store_path: impl AsRef<Path>,
) -> Result<Option<String>> {
    let lifecycle = crate::runtime_repo_frontier_research_lifecycle(store_path)?;
    Ok(
        (lifecycle.stage == crate::RepoFrontierResearchLifecycleStage::ResultReady)
            .then_some(lifecycle.worker_job_id)
            .flatten(),
    )
}

pub fn current_frontier_verification_review_job_id(
    store_path: impl AsRef<Path>,
) -> Result<Option<String>> {
    Ok(project_current_work(store_path)?
        .verification
        .filter(|work| work.action == EpiphanyAgentPassContinuationAction::Review)
        .and_then(|work| work.job_id))
}

pub fn launch_current_frontier_verification_work(
    store_path: impl AsRef<Path>,
    created_at: &str,
) -> Result<String> {
    let store_path = store_path.as_ref();
    chrono::DateTime::parse_from_rfc3339(created_at)
        .map_err(|_| anyhow!("frontier Verification launch time is invalid"))?;
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let work = current_verification_work(&cache)?
        .filter(|work| work.action == EpiphanyAgentPassContinuationAction::Launch)
        .ok_or_else(|| anyhow!("Mind has no launchable frontier Verification work"))?;
    let request = work.request;
    let context = crate::runtime_spine::repo_frontier_verification_context(&cache, &request)?;
    let identity = cache
        .get::<crate::EpiphanyRuntimeIdentity>(crate::RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("frontier Verification launch requires runtime identity"))?;
    let attempt_ordinal = cache
        .get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?
        .into_iter()
        .filter(|launch| {
            launch.repo_frontier_verification_request_id.as_deref()
                == Some(request.request_id.as_str())
        })
        .count()
        + cache
            .get_all::<crate::EpiphanyArchivedRuntimeWorkerAttempt>()?
            .into_iter()
            .filter(|attempt| {
                attempt.request_kind == "frontier-verification"
                    && attempt.request_id == request.request_id
            })
            .count();
    let job_id = format!(
        "frontier-verification-{}-attempt-{attempt_ordinal}",
        request.request_id
    );
    let launch_document =
        crate::EpiphanyWorkerLaunchDocument::Role(crate::EpiphanyRoleWorkerLaunchDocument {
            thread_id: job_id.clone(),
            role_id: "verification".into(),
            state_revision: 0,
            objective: None,
            dynamic_prompt_context: None,
            repository_body_observation_basis: None,
            proposal_modeling_context: None,
            frontier_verdict_modeling_context: None,
            frontier_planning_context: None,
            frontier_research_context: None,
            frontier_verification_context: Some(context.clone()),
            frontier_plan_mind_context: None,
            imagination_consideration_context: None,
            admitted_model_direction_consideration_context: None,
            active_subgoal_id: None,
            active_subgoals: Vec::new(),
            active_graph_node_ids: Vec::new(),
            investigation_checkpoint: None,
            scratch: None,
            invariants: Vec::new(),
            graphs: None,
            recent_evidence: Vec::new(),
            recent_observations: Vec::new(),
            graph_frontier: None,
            graph_checkpoint: None,
            planning: None,
            churn: None,
        });
    let authority_scope = "epiphany.role.verification".to_string();
    let output_contract_id = launch_document.output_contract_id().to_string();
    let prepared = crate::prepare_runtime_spine_heartbeat_job(
        &cache,
        crate::RuntimeSpineHeartbeatJobOptions {
            runtime_id: identity.runtime_id,
            display_name: "Epiphany Local".into(),
            session_id: crate::EPIPHANY_RUNTIME_ROOT_SESSION_ID.into(),
            objective: "Audit one exact Hands consequence against its routed invariant".into(),
            coordinator_note:
                "Verification current-work launch transaction opened this session.".into(),
            job_id: job_id.clone(),
            role: crate::EPIPHANY_VERIFICATION_OWNER_ROLE.into(),
            binding_id: crate::EPIPHANY_VERIFICATION_ROLE_BINDING_ID.into(),
            authority_scope: authority_scope.clone(),
            instruction: "Act as Epiphany Verification. Audit only the exact typed Hands consequence and route carried by this request; return a structured verdict, evidence ids, and risks.".into(),
            launch_document,
            output_contract_id: output_contract_id.clone(),
            organ_launch_contract: crate::default_launch_organ_contract(
                &authority_scope,
                "role",
                &output_contract_id,
            ),
            proposal_modeling_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            repo_frontier_verification_request_id: Some(request.request_id.clone()),
            created_at: created_at.to_string(),
        },
    )?;
    let grant = crate::substrate_gate::substrate_gate_repo_access_grant_for_worker(
        format!("substrate-grant-{job_id}"),
        job_id.clone(),
        crate::EPIPHANY_VERIFICATION_ROLE_BINDING_ID.into(),
        crate::EPIPHANY_VERIFICATION_OWNER_ROLE.into(),
        authority_scope,
        false,
        created_at.to_string(),
    );
    let snapshot = cache.snapshot_envelopes();
    let mut expected = Vec::new();
    for (document_type, document_key) in [
        (
            crate::RepoFrontierVerificationRequest::TYPE,
            request.request_id.as_str(),
        ),
        (crate::RepoFrontierRoute::TYPE, request.route_id.as_str()),
        (
            crate::RepoFrontierHandsAuthority::TYPE,
            context.hands_authority.authority_id.as_str(),
        ),
        (
            crate::HandsActionIntent::TYPE,
            request.hands_intent_id.as_str(),
        ),
        (
            crate::HandsActionReview::TYPE,
            request.hands_review_id.as_str(),
        ),
        (
            crate::HandsPatchReceipt::TYPE,
            request.hands_patch_receipt_id.as_str(),
        ),
        (
            crate::HandsCommandReceipt::TYPE,
            request.hands_command_receipt_id.as_str(),
        ),
        (
            crate::HandsCommitReceipt::TYPE,
            request.hands_commit_receipt_id.as_str(),
        ),
    ] {
        expected.push(
            snapshot
                .iter()
                .find(|value| value.r#type == document_type && value.key == document_key)
                .cloned()
                .ok_or_else(|| anyhow!("frontier Verification launch lost a strong source"))?,
        );
    }
    for source in &request.frontier_authority_documents {
        let envelope = snapshot
            .iter()
            .find(|value| value.r#type == source.document_type && value.key == source.document_key)
            .ok_or_else(|| anyhow!("frontier Verification launch lost frontier authority"))?;
        if EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", envelope)? != *source {
            return Err(anyhow!(
                "frontier Verification launch strong source changed"
            ));
        }
        if !expected.contains(envelope) {
            expected.push(envelope.clone());
        }
    }
    commit_current_work_launch(
        store_path,
        &cache,
        expected,
        prepared.envelopes,
        vec![cache.prepare_entry(&grant.receipt_id, &grant)?.0],
        "frontier Verification",
    )?;
    Ok(job_id)
}

pub fn launch_current_frontier_research_work(
    store_path: impl AsRef<Path>,
    created_at: &str,
) -> Result<String> {
    let store_path = store_path.as_ref();
    chrono::DateTime::parse_from_rfc3339(created_at)
        .map_err(|_| anyhow!("frontier Research launch time is invalid"))?;
    if crate::runtime_repo_frontier_research_lifecycle(store_path)?.stage
        != crate::RepoFrontierResearchLifecycleStage::LaunchReady
    {
        return Err(anyhow!("Mind has no launchable frontier Research work"));
    }
    let request = crate::select_and_commit_repo_frontier_research_request(store_path, created_at)?;
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let request_envelope = cache
        .get_envelope::<crate::RepoFrontierResearchRequest>(&request.request_id)?
        .ok_or_else(|| anyhow!("frontier Research launch lost its request"))?;
    let identity = cache
        .get::<crate::EpiphanyRuntimeIdentity>(crate::RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("frontier Research launch requires runtime identity"))?;
    let attempt_ordinal = cache
        .get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?
        .into_iter()
        .filter(|launch| {
            launch.repo_frontier_research_request_id.as_deref() == Some(request.request_id.as_str())
        })
        .count()
        + cache
            .get_all::<crate::EpiphanyArchivedRuntimeWorkerAttempt>()?
            .into_iter()
            .filter(|attempt| {
                attempt.request_kind == "frontier-research"
                    && attempt.request_id == request.request_id
            })
            .count();
    let job_id = format!(
        "frontier-research-{}-attempt-{attempt_ordinal}",
        request.request_id
    );
    let launch_document =
        crate::EpiphanyWorkerLaunchDocument::Role(crate::EpiphanyRoleWorkerLaunchDocument {
            thread_id: job_id.clone(),
            role_id: "research".into(),
            state_revision: 0,
            objective: None,
            dynamic_prompt_context: None,
            repository_body_observation_basis: None,
            proposal_modeling_context: None,
            frontier_verdict_modeling_context: None,
            frontier_planning_context: None,
            frontier_research_context: Some((&request).into()),
            frontier_verification_context: None,
            frontier_plan_mind_context: None,
            imagination_consideration_context: None,
            admitted_model_direction_consideration_context: None,
            active_subgoal_id: None,
            active_subgoals: Vec::new(),
            active_graph_node_ids: Vec::new(),
            investigation_checkpoint: None,
            scratch: None,
            invariants: Vec::new(),
            graphs: None,
            recent_evidence: Vec::new(),
            recent_observations: Vec::new(),
            graph_frontier: None,
            graph_checkpoint: None,
            planning: None,
            churn: None,
        });
    let authority_scope = "epiphany.role.research".to_string();
    let output_contract_id = launch_document.output_contract_id().to_string();
    let prepared = crate::prepare_runtime_spine_heartbeat_job(
        &cache,
        crate::RuntimeSpineHeartbeatJobOptions {
            runtime_id: identity.runtime_id,
            display_name: "Epiphany Local".into(),
            session_id: crate::EPIPHANY_RUNTIME_ROOT_SESSION_ID.into(),
            objective: "Gather evidence for one exact external-evidence obligation".into(),
            coordinator_note: "Research current-work launch transaction opened this session."
                .into(),
            job_id: job_id.clone(),
            role: crate::EPIPHANY_RESEARCH_OWNER_ROLE.into(),
            binding_id: crate::EPIPHANY_RESEARCH_ROLE_BINDING_ID.into(),
            authority_scope: authority_scope.clone(),
            instruction: "Act as Epiphany Research. Gather only the evidence required by the exact frontier request and return keyed evidence and observations for Mind review.".into(),
            launch_document,
            output_contract_id: output_contract_id.clone(),
            organ_launch_contract: crate::default_launch_organ_contract(
                &authority_scope,
                "role",
                &output_contract_id,
            ),
            proposal_modeling_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: Some(request.request_id.clone()),
            repo_frontier_verification_request_id: None,
            created_at: created_at.to_string(),
        },
    )?;
    let grant = crate::substrate_gate::substrate_gate_repo_access_grant_for_worker(
        format!("substrate-grant-{job_id}"),
        job_id.clone(),
        crate::EPIPHANY_RESEARCH_ROLE_BINDING_ID.into(),
        crate::EPIPHANY_RESEARCH_OWNER_ROLE.into(),
        authority_scope,
        true,
        created_at.to_string(),
    );
    let snapshot = cache.snapshot_envelopes();
    let mut expected = vec![request_envelope];
    for source in &request.frontier_authority_documents {
        let envelope = snapshot
            .iter()
            .find(|value| value.r#type == source.document_type && value.key == source.document_key)
            .ok_or_else(|| anyhow!("frontier Research launch lost a strong source"))?;
        if EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", envelope)? != *source {
            return Err(anyhow!("frontier Research launch strong source changed"));
        }
        expected.push(envelope.clone());
    }
    commit_current_work_launch(
        store_path,
        &cache,
        expected,
        prepared.envelopes,
        vec![cache.prepare_entry(&grant.receipt_id, &grant)?.0],
        "frontier Research",
    )?;
    Ok(job_id)
}

pub fn accept_frontier_research_result(
    store_path: impl AsRef<Path>,
    job_id: &str,
    accepted_at: &str,
) -> Result<crate::EpiphanyMindCommitReceipt> {
    let store_path = store_path.as_ref();
    chrono::DateTime::parse_from_rfc3339(accepted_at)
        .map_err(|_| anyhow!("frontier Research acceptance time is invalid"))?;
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    if let Some(result) = cache.get::<crate::EpiphanyRuntimeRoleWorkerResult>(job_id)? {
        let mut receipts = cache
            .get_all::<crate::EpiphanyMindCommitReceipt>()?
            .into_iter()
            .filter(|receipt| {
                receipt.invariant_owner == "Eyes.frontier_research"
                    && matches!(
                        &receipt.authority,
                        crate::EpiphanyMindCommitAuthority::ModelDecisionContext {
                            decision_context_id
                        } if decision_context_id == &result.decision_context_id
                    )
            })
            .collect::<Vec<_>>();
        if receipts.len() > 1 {
            return Err(anyhow!(
                "frontier Research result has multiple Mind commit authorities"
            ));
        }
        if let Some(receipt) = receipts.pop() {
            let request_id = result
                .repo_frontier_research_request_id
                .as_deref()
                .ok_or_else(|| anyhow!("frontier Research replay lost its request identity"))?;
            let packet_id = format!("eyes-packet-{request_id}");
            let packet = cache
                .get::<crate::EyesEvidencePacket>(&packet_id)?
                .ok_or_else(|| anyhow!("frontier Research replay lost its evidence packet"))?;
            if packet.research_request_id != request_id
                || packet.decision_context_id != result.decision_context_id
                || !receipt
                    .writes
                    .iter()
                    .any(|write| write.document_type == crate::EpiphanyMindEvidenceDocument::TYPE)
            {
                return Err(anyhow!(
                    "frontier Research replay does not preserve its exact accepted decision"
                ));
            }
            return Ok(receipt);
        }
    }
    if current_frontier_research_review_job_id(store_path)?.as_deref() != Some(job_id) {
        return Err(anyhow!(
            "frontier Research result is not current review work"
        ));
    }
    let job = cache
        .get::<crate::EpiphanyRuntimeJob>(job_id)?
        .ok_or_else(|| anyhow!("frontier Research acceptance lost its runtime job"))?;
    if job.status != crate::EpiphanyRuntimeJobStatus::Completed {
        return Err(anyhow!("frontier Research runtime job is not completed"));
    }
    let launch = cache
        .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(job_id)?
        .ok_or_else(|| anyhow!("frontier Research acceptance lost its launch"))?;
    let request = crate::runtime_spine::frontier_research_request_for_launch(&cache, &launch)?
        .ok_or_else(|| anyhow!("frontier Research acceptance lost its exact request"))?;
    let fulfillment = crate::runtime_typed_request_fulfillment(
        store_path,
        crate::RuntimeTypedRequestRef::FrontierResearch(&request.request_id),
    )?
    .ok_or_else(|| anyhow!("frontier Research result is not exact typed fulfillment"))?;
    if fulfillment.job_id != job_id {
        return Err(anyhow!(
            "frontier Research fulfillment belongs to another attempt"
        ));
    }
    let result = cache
        .get::<crate::EpiphanyRuntimeRoleWorkerResult>(job_id)?
        .ok_or_else(|| anyhow!("frontier Research acceptance lost its typed result"))?;
    if !result.role_id.eq_ignore_ascii_case("research")
        || result.repo_frontier_research_request_id.as_deref() != Some(request.request_id.as_str())
        || result.item_error.is_some()
    {
        return Err(anyhow!("frontier Research result crossed family authority"));
    }
    let patch = result
        .state_patch()?
        .ok_or_else(|| anyhow!("frontier Research result has no keyed state proposal"))?;
    let policy_errors = crate::research_role_state_patch_policy_errors(&patch);
    if !policy_errors.is_empty() {
        return Err(anyhow!(
            "frontier Research state proposal is invalid: {}",
            policy_errors.join("; ")
        ));
    }
    let evidence_ids = patch
        .evidence
        .iter()
        .map(|evidence| evidence.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if evidence_ids.len() != patch.evidence.len()
        || patch.observations.iter().any(|observation| {
            observation.evidence_ids.is_empty()
                || observation
                    .evidence_ids
                    .iter()
                    .any(|id| !evidence_ids.contains(id.as_str()))
        })
    {
        return Err(anyhow!(
            "frontier Research observations do not cite the exact proposed evidence set"
        ));
    }
    let lookups =
        crate::runtime_authenticated_public_source_lookups_for_worker(store_path, job_id)?;
    for lookup in &lookups {
        if !result.evidence_ids.contains(&lookup.receipt_id)
            || !result.files_inspected.contains(&lookup.source_ref)
        {
            return Err(anyhow!(
                "frontier Research result omitted authenticated public-source provenance"
            ));
        }
    }
    if patch
        .evidence
        .iter()
        .any(|evidence| !result.evidence_ids.contains(&evidence.id))
    {
        return Err(anyhow!(
            "frontier Research result did not expose every proposed evidence identity"
        ));
    }
    let finding = crate::interpret_runtime_role_worker_result(
        crate::EpiphanyRoleResultRoleId::Research,
        &result,
    );
    let packet_id = format!("eyes-packet-{}", request.request_id);
    let packet = crate::eyes_evidence_packet_from_research_finding(
        packet_id.clone(),
        request.request_id.clone(),
        result.decision_context_id.clone(),
        &finding,
        &patch,
        &lookups,
        accepted_at.to_string(),
    );
    let mut writes = Vec::new();
    for evidence in &patch.evidence {
        writes.push(crate::mind_documents::prepare_mind_document(
            &cache,
            &evidence.id,
            &crate::EpiphanyMindEvidenceDocument {
                value: evidence.clone(),
            },
        )?);
    }
    for observation in &patch.observations {
        writes.push(crate::mind_documents::prepare_mind_document(
            &cache,
            &observation.id,
            &crate::EpiphanyMindObservationDocument {
                value: observation.clone(),
            },
        )?);
    }
    if let Some(checkpoint) = patch.investigation_checkpoint.as_ref() {
        writes.push(crate::mind_documents::prepare_mind_document(
            &cache,
            &checkpoint.checkpoint_id,
            &crate::EpiphanyMindInvestigationCheckpointDocument {
                value: checkpoint.clone(),
            },
        )?);
    }
    let snapshot = cache.snapshot_envelopes();
    let mut strong_reads = Vec::new();
    for (document_type, document_key) in [
        (
            crate::RepoFrontierResearchRequest::TYPE,
            request.request_id.as_str(),
        ),
        (crate::EpiphanyRuntimeWorkerLaunchRequest::TYPE, job_id),
        (crate::EpiphanyRuntimeJob::TYPE, job_id),
        (crate::EpiphanyRuntimeRoleWorkerResult::TYPE, job_id),
        (
            crate::EpiphanyDecisionContext::TYPE,
            result.decision_context_id.as_str(),
        ),
    ] {
        strong_reads.push(
            snapshot
                .iter()
                .find(|value| value.r#type == document_type && value.key == document_key)
                .cloned()
                .ok_or_else(|| anyhow!("frontier Research admission lost a strong source"))?,
        );
    }
    for source in &request.frontier_authority_documents {
        let envelope = snapshot
            .iter()
            .find(|value| value.r#type == source.document_type && value.key == source.document_key)
            .ok_or_else(|| anyhow!("frontier Research admission lost frontier authority"))?;
        if EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", envelope)? != *source {
            return Err(anyhow!(
                "frontier Research output cannot be rebased onto changed frontier authority"
            ));
        }
        strong_reads.push(envelope.clone());
    }
    let packet_envelope = cache.prepare_entry(&packet_id, &packet)?.0;
    match crate::reasoning_context::commit_mind_mutation_with_derived_companions(
        store_path,
        &result.decision_context_id,
        "Eyes.frontier_research",
        strong_reads,
        writes,
        vec![packet_envelope],
        accepted_at,
    )? {
        crate::EpiphanyMindCommitOutcome::Committed(receipt) => Ok(receipt),
        crate::EpiphanyMindCommitOutcome::Conflict {
            document_identities,
        } => Err(anyhow!(
            "frontier Research admission lost exact keyed reads: {document_identities:?}"
        )),
    }
}

pub fn accept_frontier_verification_result(
    store_path: impl AsRef<Path>,
    job_id: &str,
    accepted_at: &str,
) -> Result<crate::EpiphanyMindCommitReceipt> {
    let store_path = store_path.as_ref();
    chrono::DateTime::parse_from_rfc3339(accepted_at)
        .map_err(|_| anyhow!("frontier Verification acceptance time is invalid"))?;
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    if let Some(result) = cache.get::<crate::EpiphanyRuntimeRoleWorkerResult>(job_id)? {
        let mut receipts = cache
            .get_all::<crate::EpiphanyMindCommitReceipt>()?
            .into_iter()
            .filter(|receipt| {
                receipt.invariant_owner == "Soul.verification"
                    && matches!(
                        &receipt.authority,
                        crate::EpiphanyMindCommitAuthority::ModelDecisionContext {
                            decision_context_id
                        } if decision_context_id == &result.decision_context_id
                    )
            })
            .collect::<Vec<_>>();
        if receipts.len() > 1 {
            return Err(anyhow!(
                "frontier Verification result has multiple Mind commit authorities"
            ));
        }
        if let Some(receipt) = receipts.pop() {
            let audit_id = format!("verification-audit-{}", result.result_id);
            let verdict_id = format!("soul-verdict-{}", result.result_id);
            let audit = cache
                .get::<crate::EpiphanyMindVerificationAuditDocument>(&audit_id)?
                .ok_or_else(|| anyhow!("frontier Verification replay lost its audit"))?;
            let verdict = cache
                .get::<crate::SoulVerdictReceipt>(&verdict_id)?
                .ok_or_else(|| anyhow!("frontier Verification replay lost its Soul verdict"))?;
            let modeling_requests = cache
                .get_all::<crate::RepoFrontierModelingRequest>()?
                .into_iter()
                .filter(|request| request.soul_verdict_receipt_id == verdict_id)
                .collect::<Vec<_>>();
            if audit.decision_context_id != result.decision_context_id
                || audit.result_id != result.result_id
                || verdict.source_result_id != result.result_id
                || modeling_requests.len() != 1
                || !receipt.writes.iter().any(|write| {
                    write.document_type == crate::EpiphanyMindVerificationAuditDocument::TYPE
                        && write.document_key == audit_id
                })
            {
                return Err(anyhow!(
                    "frontier Verification replay does not preserve its exact accepted decision"
                ));
            }
            return Ok(receipt);
        }
    }
    if current_frontier_verification_review_job_id(store_path)?.as_deref() != Some(job_id) {
        return Err(anyhow!(
            "frontier Verification result is not current review work"
        ));
    }
    let job = cache
        .get::<crate::EpiphanyRuntimeJob>(job_id)?
        .ok_or_else(|| anyhow!("frontier Verification acceptance lost its runtime job"))?;
    if job.status != crate::EpiphanyRuntimeJobStatus::Completed {
        return Err(anyhow!(
            "frontier Verification runtime job is not completed"
        ));
    }
    let launch = cache
        .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(job_id)?
        .ok_or_else(|| anyhow!("frontier Verification acceptance lost its launch"))?;
    let request = crate::runtime_spine::frontier_verification_request_for_launch(&cache, &launch)?
        .ok_or_else(|| anyhow!("frontier Verification acceptance lost its exact request"))?;
    let fulfillment = crate::runtime_typed_request_fulfillment(
        store_path,
        crate::RuntimeTypedRequestRef::FrontierVerification(&request.request_id),
    )?
    .ok_or_else(|| anyhow!("frontier Verification result is not exact typed fulfillment"))?;
    if fulfillment.job_id != job_id {
        return Err(anyhow!(
            "frontier Verification fulfillment belongs to another attempt"
        ));
    }
    let result = cache
        .get::<crate::EpiphanyRuntimeRoleWorkerResult>(job_id)?
        .ok_or_else(|| anyhow!("frontier Verification acceptance lost its typed result"))?;
    if !result.role_id.eq_ignore_ascii_case("verification")
        || result.verification_request_id.as_deref() != Some(request.request_id.as_str())
        || result.frontier_route_id.as_deref() != Some(request.route_id.as_str())
        || result.item_error.is_some()
    {
        return Err(anyhow!(
            "frontier Verification result crossed request authority"
        ));
    }
    if !crate::runtime_spine::verification_frontier_is_current(&cache, &request)? {
        return Err(anyhow!(
            "frontier Verification decision remains recorded but its strong frontier changed; a fresh pass is required"
        ));
    }
    let mut evidence_ids = result.evidence_ids.clone();
    evidence_ids.sort();
    evidence_ids.dedup();
    let audit_id = format!("verification-audit-{}", result.result_id);
    let audit = crate::EpiphanyMindVerificationAuditDocument {
        audit_id: audit_id.clone(),
        verification_request_id: request.request_id.clone(),
        frontier_route_id: request.route_id.clone(),
        job_id: job_id.to_string(),
        result_id: result.result_id.clone(),
        decision_context_id: result.decision_context_id.clone(),
        verdict: result.verdict.clone(),
        summary: result.summary.clone(),
        evidence_ids: evidence_ids.clone(),
        risks: result.risks.clone(),
        audited_at: accepted_at.to_string(),
    };
    audit.validate()?;
    let verdict = crate::SoulVerdictReceipt {
        schema_version: crate::SOUL_VERDICT_RECEIPT_SCHEMA_VERSION.into(),
        receipt_id: format!("soul-verdict-{}", result.result_id),
        source_result_id: result.result_id.clone(),
        source_job_id: result.job_id.clone(),
        verdict: result.verdict.clone(),
        summary: result.summary.clone(),
        evidence_ids,
        risks: result.risks.clone(),
        emitted_at: accepted_at.to_string(),
        contract: "Soul verdict emitted by the exact Verification Mind admission owner.".into(),
        verification_request_id: request.request_id.clone(),
        frontier_route_id: request.route_id.clone(),
    };
    let modeling_request =
        crate::runtime_spine::derive_repo_frontier_modeling_request(&cache, &verdict)?;
    let snapshot = cache.snapshot_envelopes();
    let mut strong_reads = Vec::new();
    for (document_type, document_key) in [
        (
            crate::RepoFrontierVerificationRequest::TYPE,
            request.request_id.as_str(),
        ),
        (crate::EpiphanyRuntimeWorkerLaunchRequest::TYPE, job_id),
        (crate::EpiphanyRuntimeJob::TYPE, job_id),
        (crate::EpiphanyRuntimeRoleWorkerResult::TYPE, job_id),
        (
            crate::EpiphanyDecisionContext::TYPE,
            result.decision_context_id.as_str(),
        ),
        (crate::RepoFrontierRoute::TYPE, request.route_id.as_str()),
        (
            crate::HandsActionIntent::TYPE,
            request.hands_intent_id.as_str(),
        ),
        (
            crate::HandsActionReview::TYPE,
            request.hands_review_id.as_str(),
        ),
        (
            crate::HandsPatchReceipt::TYPE,
            request.hands_patch_receipt_id.as_str(),
        ),
        (
            crate::HandsCommandReceipt::TYPE,
            request.hands_command_receipt_id.as_str(),
        ),
        (
            crate::HandsCommitReceipt::TYPE,
            request.hands_commit_receipt_id.as_str(),
        ),
    ] {
        strong_reads.push(
            snapshot
                .iter()
                .find(|value| value.r#type == document_type && value.key == document_key)
                .cloned()
                .ok_or_else(|| anyhow!("frontier Verification admission lost a strong source"))?,
        );
    }
    for source in &request.frontier_authority_documents {
        let envelope = snapshot
            .iter()
            .find(|value| value.r#type == source.document_type && value.key == source.document_key)
            .ok_or_else(|| anyhow!("frontier Verification admission lost frontier authority"))?;
        if EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", envelope)? != *source {
            return Err(anyhow!(
                "frontier Verification output cannot be rebased onto changed frontier authority"
            ));
        }
        if !strong_reads.contains(envelope) {
            strong_reads.push(envelope.clone());
        }
    }
    let hands_authority = cache
        .get_all::<crate::RepoFrontierHandsAuthority>()?
        .into_iter()
        .find(|authority| authority.route_id == request.route_id)
        .ok_or_else(|| anyhow!("frontier Verification admission lost Hands authority"))?;
    strong_reads.push(
        cache
            .get_envelope::<crate::RepoFrontierHandsAuthority>(&hands_authority.authority_id)?
            .ok_or_else(|| anyhow!("frontier Verification admission lost Hands envelope"))?,
    );
    let writes = vec![crate::mind_documents::prepare_mind_document(
        &cache, &audit_id, &audit,
    )?];
    let companions = vec![
        cache.prepare_entry(&verdict.receipt_id, &verdict)?.0,
        cache
            .prepare_entry(&modeling_request.request_id, &modeling_request)?
            .0,
    ];
    match crate::reasoning_context::commit_mind_mutation_with_derived_companions(
        store_path,
        &result.decision_context_id,
        "Soul.verification",
        strong_reads,
        writes,
        companions,
        accepted_at,
    )? {
        crate::EpiphanyMindCommitOutcome::Committed(receipt) => Ok(receipt),
        crate::EpiphanyMindCommitOutcome::Conflict {
            document_identities,
        } => Err(anyhow!(
            "frontier Verification admission lost exact keyed reads: {document_identities:?}"
        )),
    }
}

pub fn accept_body_modeling_result(
    store_path: impl AsRef<Path>,
    job_id: &str,
    accepted_at: &str,
) -> Result<crate::EpiphanyMindCommitReceipt> {
    let store_path = store_path.as_ref();
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let job = cache
        .get::<crate::EpiphanyRuntimeJob>(job_id)?
        .ok_or_else(|| anyhow!("Body Modeling acceptance lost its runtime job"))?;
    if job.status != crate::EpiphanyRuntimeJobStatus::Completed {
        return Err(anyhow!("Body Modeling runtime job is not completed"));
    }
    let result = cache
        .get::<crate::EpiphanyRuntimeRoleWorkerResult>(job_id)?
        .ok_or_else(|| anyhow!("Body Modeling acceptance lost its typed result"))?;
    if result.role_id != "modeling"
        || result.proposal_modeling_request_id.is_some()
        || result.repo_frontier_modeling_request_id.is_some()
    {
        return Err(anyhow!("result is not baseline Body Modeling authority"));
    }
    let proposal = result.repo_model_mutation_proposal()?;
    if proposal.is_none()
        && !matches!(
            result.verdict.as_str(),
            "checkpoint-ready" | "regather-needed"
        )
    {
        return Err(anyhow!(
            "Body Modeling result requiring RepoModel change has no mutation proposal"
        ));
    }
    let mut strong_reads = Vec::new();
    let mut writes = Vec::new();
    if let Some(proposal) = proposal.as_ref() {
        if proposal.proposal_id != format!("repo-model-mutation-proposal-{job_id}") {
            return Err(anyhow!(
                "Body Modeling mutation proposal identity is not runtime-owned"
            ));
        }
        let plan = crate::plan_repo_model_mutation(store_path, proposal)?;
        strong_reads = plan.strong_reads;
        writes = plan.writes;
    }
    let disposition = if proposal.is_some() {
        "modeled"
    } else {
        result.verdict.as_str()
    };
    writes.push(body_modeling_decision_envelope(
        store_path,
        &result,
        disposition,
        accepted_at,
    )?);
    match crate::commit_mind_mutation(
        store_path,
        &result.decision_context_id,
        "Modeling.body_projection",
        strong_reads,
        writes,
        accepted_at,
    )? {
        crate::EpiphanyMindCommitOutcome::Committed(receipt) => Ok(receipt),
        crate::EpiphanyMindCommitOutcome::Conflict {
            document_identities,
        } => Err(anyhow!(
            "Body Modeling admission lost exact keyed reads: {document_identities:?}"
        )),
    }
}

pub fn accept_proposal_modeling_result(
    store_path: impl AsRef<Path>,
    job_id: &str,
    accepted_at: &str,
) -> Result<crate::EpiphanyMindCommitReceipt> {
    let store_path = store_path.as_ref();
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let job = cache
        .get::<crate::EpiphanyRuntimeJob>(job_id)?
        .ok_or_else(|| anyhow!("proposal Modeling acceptance lost its runtime job"))?;
    if job.status != crate::EpiphanyRuntimeJobStatus::Completed {
        return Err(anyhow!("proposal Modeling runtime job is not completed"));
    }
    let result = cache
        .get::<crate::EpiphanyRuntimeRoleWorkerResult>(job_id)?
        .ok_or_else(|| anyhow!("proposal Modeling acceptance lost its typed result"))?;
    let request_id = result
        .proposal_modeling_request_id
        .as_deref()
        .ok_or_else(|| anyhow!("result is not proposal Modeling authority"))?;
    if !result.role_id.eq_ignore_ascii_case("modeling")
        || result.repo_frontier_modeling_request_id.is_some()
    {
        return Err(anyhow!("proposal Modeling result crossed family authority"));
    }
    let evidence = crate::runtime_typed_request_fulfillment(
        store_path,
        crate::RuntimeTypedRequestRef::ProposalModeling(request_id),
    )?
    .ok_or_else(|| anyhow!("proposal Modeling result is not exact typed fulfillment"))?;
    if evidence.job_id != job_id || evidence.result_id != result.result_id {
        return Err(anyhow!("proposal Modeling fulfillment identity mismatch"));
    }
    let proposal = result
        .repo_model_mutation_proposal()?
        .ok_or_else(|| anyhow!("proposal Modeling result has no semantic mutation proposal"))?;
    if proposal.proposal_id != format!("repo-model-mutation-proposal-{job_id}") {
        return Err(anyhow!(
            "proposal Modeling mutation proposal identity is not runtime-owned"
        ));
    }
    let plan = crate::plan_repo_model_mutation(store_path, &proposal)?;
    match crate::commit_mind_mutation(
        store_path,
        &result.decision_context_id,
        "Modeling.proposal_frontier",
        plan.strong_reads,
        plan.writes,
        accepted_at,
    )? {
        crate::EpiphanyMindCommitOutcome::Committed(receipt) => Ok(receipt),
        crate::EpiphanyMindCommitOutcome::Conflict {
            document_identities,
        } => Err(anyhow!(
            "proposal Modeling admission lost exact keyed reads: {document_identities:?}"
        )),
    }
}

pub fn accept_frontier_verdict_modeling_result(
    store_path: impl AsRef<Path>,
    job_id: &str,
    accepted_at: &str,
) -> Result<crate::EpiphanyMindCommitReceipt> {
    let store_path = store_path.as_ref();
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let current = current_frontier_verdict_modeling_work(&cache)?
        .filter(|work| {
            work.action == EpiphanyAgentPassContinuationAction::Review
                && work.job_id.as_deref() == Some(job_id)
        })
        .ok_or_else(|| anyhow!("frontier verdict Modeling result is not current review work"))?;
    let result = cache
        .get::<crate::EpiphanyRuntimeRoleWorkerResult>(job_id)?
        .ok_or_else(|| anyhow!("frontier verdict Modeling acceptance lost its typed result"))?;
    if !result.role_id.eq_ignore_ascii_case("modeling")
        || result.repo_frontier_modeling_request_id.as_deref()
            != Some(current.request.request_id.as_str())
        || result.proposal_modeling_request_id.is_some()
    {
        return Err(anyhow!(
            "frontier verdict Modeling result crossed family authority"
        ));
    }
    let proposal = result
        .repo_model_mutation_proposal()?
        .ok_or_else(|| anyhow!("frontier verdict Modeling result has no mutation proposal"))?;
    if proposal.proposal_id != format!("repo-model-mutation-proposal-{job_id}") {
        return Err(anyhow!(
            "frontier verdict Modeling mutation proposal identity is not runtime-owned"
        ));
    }
    let plan = crate::plan_repo_model_mutation(store_path, &proposal)?;
    let expected_frontier_version = current
        .request
        .model_source_documents
        .iter()
        .find(|source| {
            source.document_type == crate::EpiphanyRepoModelFrontierDocument::TYPE
                && source.document_key == current.request.frontier_item_id
        })
        .ok_or_else(|| {
            anyhow!("frontier verdict Modeling request lost its exact frontier version")
        })?;
    let planned_frontier = plan
        .strong_reads
        .iter()
        .find(|envelope| {
            envelope.r#type == crate::EpiphanyRepoModelFrontierDocument::TYPE
                && envelope.key == current.request.frontier_item_id
        })
        .ok_or_else(|| {
            anyhow!("frontier verdict Modeling mutation omitted its strong frontier read")
        })?;
    if crate::EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", planned_frontier)?
        != *expected_frontier_version
    {
        return Err(anyhow!(
            "frontier verdict Modeling output cannot be rebased onto a changed frontier"
        ));
    }
    match crate::commit_mind_mutation(
        store_path,
        &result.decision_context_id,
        "Modeling.frontier_verdict",
        plan.strong_reads,
        plan.writes,
        accepted_at,
    )? {
        crate::EpiphanyMindCommitOutcome::Committed(receipt) => Ok(receipt),
        crate::EpiphanyMindCommitOutcome::Conflict {
            document_identities,
        } => Err(anyhow!(
            "frontier verdict Modeling admission lost exact keyed reads: {document_identities:?}"
        )),
    }
}

fn commit_current_work_launch(
    store_path: &Path,
    cache: &CultCache,
    mut expected: Vec<CultCacheEnvelope>,
    mut replacements: Vec<CultCacheEnvelope>,
    companions: Vec<CultCacheEnvelope>,
    family: &str,
) -> Result<()> {
    replacements.extend(companions);
    let snapshot = cache.snapshot_envelopes();
    for replacement in &replacements {
        if let Some(existing) = snapshot
            .iter()
            .find(|value| value.r#type == replacement.r#type && value.key == replacement.key)
            && !expected.contains(existing)
        {
            expected.push(existing.clone());
        }
    }
    for envelope in &expected {
        if !replacements.iter().any(|replacement| {
            replacement.r#type == envelope.r#type && replacement.key == envelope.key
        }) {
            replacements.push(envelope.clone());
        }
    }
    if !crate::runtime_store_backend::runtime_spine_backing_store(store_path)?
        .compare_and_swap_batch(&expected, replacements)?
    {
        return Err(anyhow!("{family} launch lost its exact Mind/runtime CAS"));
    }
    Ok(())
}

pub fn launch_current_body_modeling_work(
    store_path: impl AsRef<Path>,
    options: EpiphanyBodyModelingLaunchOptions,
) -> Result<EpiphanyBodyModelingLaunchBinding> {
    let store_path = store_path.as_ref();
    if options.job_id.trim().is_empty()
        || chrono::DateTime::parse_from_rfc3339(&options.created_at).is_err()
    {
        return Err(anyhow!("Body Modeling launch options are invalid"));
    }
    let mind = crate::assemble_mind_view(store_path)?;
    let body_basis = mind
        .repository_body_observation
        .clone()
        .ok_or_else(|| anyhow!("Mind has no admitted repository Body observation"))?;
    let repo_model = mind
        .repo_model
        .as_ref()
        .ok_or_else(|| anyhow!("Mind has no keyed RepoModel state"))?;
    let work = EpiphanyBodyModelingWorkProjection::derive(
        mind.runtime_id.clone(),
        body_basis,
        repo_model.reasoning_basis(),
    )?;
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    if resolve_body_modeling_work(
        work.clone(),
        cache.get::<EpiphanyBodyModelingDecisionReceipt>(&work.work_id)?,
    )?
    .is_none()
        || body_modeling_continuation_action(&cache, &work.work_id)?
            != EpiphanyAgentPassContinuationAction::Launch
    {
        return Err(anyhow!("Mind has no launchable Body Modeling work"));
    }
    let body_source = mind
        .source_documents
        .iter()
        .find(|source| {
            source.document_type == crate::EpiphanyMindRepositoryBodyObservationDocument::TYPE
                && source.document_key == work.body_basis.observation_id
        })
        .cloned()
        .ok_or_else(|| anyhow!("Body Modeling launch lost its admitted Body source"))?;
    let attempt_ordinal = cache
        .get_all::<EpiphanyBodyModelingLaunchBinding>()?
        .into_iter()
        .filter(|binding| binding.work_id == work.work_id)
        .map(|binding| binding.attempt_ordinal)
        .max()
        .map_or(0, |ordinal| ordinal + 1);
    let binding_record_id = format!(
        "body-modeling-launch-{}-attempt-{attempt_ordinal}",
        work.work_id
    );
    let launch_document =
        crate::EpiphanyWorkerLaunchDocument::Role(crate::EpiphanyRoleWorkerLaunchDocument {
            thread_id: binding_record_id.clone(),
            role_id: "modeling".into(),
            state_revision: 0,
            objective: None,
            dynamic_prompt_context: None,
            repository_body_observation_basis: Some(work.body_basis.clone()),
            proposal_modeling_context: None,
            frontier_verdict_modeling_context: None,
            frontier_planning_context: None,
            frontier_research_context: None,
            frontier_verification_context: None,
            frontier_plan_mind_context: None,
            imagination_consideration_context: None,
            admitted_model_direction_consideration_context: None,
            active_subgoal_id: None,
            active_subgoals: Vec::new(),
            active_graph_node_ids: Vec::new(),
            investigation_checkpoint: None,
            scratch: None,
            invariants: Vec::new(),
            graphs: None,
            recent_evidence: Vec::new(),
            recent_observations: Vec::new(),
            graph_frontier: None,
            graph_checkpoint: None,
            planning: None,
            churn: None,
        });
    let authority_scope = "epiphany.role.modeling".to_string();
    let output_contract_id = launch_document.output_contract_id().to_string();
    let instruction = "Act as Epiphany Modeling. Model the admitted repository Body against the exact keyed Mind projection and return the typed role result contract.".to_string();
    let prepared = crate::prepare_runtime_spine_heartbeat_job(
        &cache,
        crate::RuntimeSpineHeartbeatJobOptions {
            runtime_id: work.runtime_id.clone(),
            display_name: "Epiphany Local".into(),
            session_id: crate::EPIPHANY_RUNTIME_ROOT_SESSION_ID.into(),
            objective: "Model the admitted repository Body".into(),
            coordinator_note: "Keyed Mind current-work launch transaction opened this session."
                .into(),
            job_id: options.job_id.clone(),
            role: crate::EPIPHANY_MODELING_OWNER_ROLE.into(),
            binding_id: crate::EPIPHANY_MODELING_ROLE_BINDING_ID.into(),
            authority_scope: authority_scope.clone(),
            instruction: instruction.clone(),
            launch_document: launch_document.clone(),
            output_contract_id: output_contract_id.clone(),
            organ_launch_contract: crate::default_launch_organ_contract(
                &authority_scope,
                launch_document.document_kind(),
                &output_contract_id,
            ),
            proposal_modeling_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            repo_frontier_verification_request_id: None,
            created_at: options.created_at.clone(),
        },
    )?;
    let binding = EpiphanyBodyModelingLaunchBinding {
        schema_version: BODY_MODELING_LAUNCH_BINDING_SCHEMA_VERSION.into(),
        binding_record_id,
        work_id: work.work_id,
        job_id: options.job_id,
        attempt_ordinal,
        body_document_sha256: body_source.payload_sha256.clone(),
        observed_repo_model_projection_digest: work.repo_model_basis.projection_digest.clone(),
        launched_at: options.created_at.clone(),
    };
    let grant = crate::substrate_gate::substrate_gate_repo_access_grant_for_worker(
        format!("substrate-grant-{}", binding.job_id),
        binding.job_id.clone(),
        crate::EPIPHANY_MODELING_ROLE_BINDING_ID.into(),
        crate::EPIPHANY_MODELING_OWNER_ROLE.into(),
        authority_scope,
        false,
        options.created_at,
    );
    let snapshot = cache.snapshot_envelopes();
    let mut expected = Vec::new();
    for source in [&body_source] {
        let envelope = snapshot
            .iter()
            .find(|value| value.r#type == source.document_type && value.key == source.document_key)
            .ok_or_else(|| anyhow!("Body Modeling launch lost a strong Mind source"))?;
        if EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", envelope)? != *source {
            return Err(anyhow!("Body Modeling launch strong Mind source changed"));
        }
        if !expected.contains(envelope) {
            expected.push(envelope.clone());
        }
    }
    commit_current_work_launch(
        store_path,
        &cache,
        expected,
        prepared.envelopes,
        vec![
            cache.prepare_entry(&binding.binding_record_id, &binding)?.0,
            cache.prepare_entry(&grant.receipt_id, &grant)?.0,
        ],
        "Body Modeling",
    )?;
    Ok(binding)
}

pub fn launch_current_proposal_modeling_work(
    store_path: impl AsRef<Path>,
    options: EpiphanyProposalModelingLaunchOptions,
) -> Result<crate::RepoFrontierProposalModelingLaunchBinding> {
    let store_path = store_path.as_ref();
    if chrono::DateTime::parse_from_rfc3339(&options.created_at).is_err() {
        return Err(anyhow!("proposal Modeling launch options are invalid"));
    }
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let work = current_proposal_modeling_work(&cache)?
        .filter(|work| work.action == EpiphanyAgentPassContinuationAction::Launch)
        .ok_or_else(|| anyhow!("Mind has no launchable proposal Modeling work"))?;
    let request = work.request;
    let proposal = cache
        .get::<crate::RepoFrontierWorkProposal>(&request.proposal_id)?
        .ok_or_else(|| anyhow!("proposal Modeling launch lost its proposal"))?;
    crate::runtime_spine::validate_repo_frontier_work_proposal(&proposal)?;
    if proposal.source_kind == crate::RepoFrontierProposalSourceKind::Imagination {
        crate::runtime_spine::validate_autonomous_proposal_binding(&cache, &proposal)?;
    }
    let identity = cache
        .get::<crate::EpiphanyRuntimeIdentity>(crate::RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("proposal Modeling launch requires runtime identity"))?;
    if request.runtime_id != identity.runtime_id
        || request.runtime_id != proposal.runtime_id
        || request.proposal_id != proposal.proposal_id
        || request.proposal_payload_sha256 != proposal.payload_sha256
    {
        return Err(anyhow!("proposal Modeling launch provenance mismatch"));
    }
    let mind = crate::assemble_mind_view(store_path)?;
    let body_basis = mind
        .repository_body_observation
        .clone()
        .ok_or_else(|| anyhow!("proposal Modeling launch requires admitted Body state"))?;
    let model = mind
        .repo_model
        .as_ref()
        .ok_or_else(|| anyhow!("proposal Modeling launch requires keyed RepoModel state"))?;
    let proposal_context = build_proposal_modeling_context_projection(&request, &proposal, model)?;
    let attempt_ordinal = cache
        .get_all::<crate::RepoFrontierProposalModelingLaunchBinding>()?
        .into_iter()
        .filter(|binding| binding.proposal_modeling_request_id == request.request_id)
        .count()
        + cache
            .get_all::<crate::EpiphanyArchivedRuntimeWorkerAttempt>()?
            .into_iter()
            .filter(|attempt| {
                attempt.request_kind == "proposal-modeling"
                    && attempt.request_id == request.request_id
            })
            .count();
    let job_id = format!(
        "proposal-modeling-{}-attempt-{attempt_ordinal}",
        request.request_id
    );
    let binding_record_id = format!("repo-frontier-proposal-modeling-launch-{job_id}");
    let launch_document =
        crate::EpiphanyWorkerLaunchDocument::Role(crate::EpiphanyRoleWorkerLaunchDocument {
            thread_id: request.thread_id.clone(),
            role_id: "modeling".into(),
            state_revision: 0,
            objective: None,
            dynamic_prompt_context: None,
            repository_body_observation_basis: Some(body_basis),
            proposal_modeling_context: Some(proposal_context),
            frontier_verdict_modeling_context: None,
            frontier_planning_context: None,
            frontier_research_context: None,
            frontier_verification_context: None,
            frontier_plan_mind_context: None,
            imagination_consideration_context: None,
            admitted_model_direction_consideration_context: None,
            active_subgoal_id: None,
            active_subgoals: Vec::new(),
            active_graph_node_ids: Vec::new(),
            investigation_checkpoint: None,
            scratch: None,
            invariants: Vec::new(),
            graphs: None,
            recent_evidence: Vec::new(),
            recent_observations: Vec::new(),
            graph_frontier: None,
            graph_checkpoint: None,
            planning: None,
            churn: None,
        });
    let worker_launch_document_sha256 = format!(
        "{:x}",
        Sha256::digest(rmp_serde::to_vec_named(&launch_document)?)
    );
    let authority_scope = "epiphany.role.modeling".to_string();
    let output_contract_id = launch_document.output_contract_id().to_string();
    let prepared = crate::prepare_runtime_spine_heartbeat_job(
        &cache,
        crate::RuntimeSpineHeartbeatJobOptions {
            runtime_id: identity.runtime_id.clone(),
            display_name: "Epiphany Local".into(),
            session_id: crate::EPIPHANY_RUNTIME_ROOT_SESSION_ID.into(),
            objective: "Model one admitted proposal against keyed Mind".into(),
            coordinator_note: "Proposal current-work launch transaction opened this session."
                .into(),
            job_id: job_id.clone(),
            role: crate::EPIPHANY_MODELING_OWNER_ROLE.into(),
            binding_id: crate::EPIPHANY_MODELING_ROLE_BINDING_ID.into(),
            authority_scope: authority_scope.clone(),
            instruction: "Act as Epiphany Modeling. Evaluate the exact admitted proposal against the sealed keyed Mind projection and return only the proposal Modeling result contract.".into(),
            launch_document,
            output_contract_id: output_contract_id.clone(),
            organ_launch_contract: crate::default_launch_organ_contract(
                &authority_scope,
                "role",
                &output_contract_id,
            ),
            proposal_modeling_request_id: Some(request.request_id.clone()),
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            repo_frontier_verification_request_id: None,
            created_at: options.created_at.clone(),
        },
    )?;
    let binding = crate::RepoFrontierProposalModelingLaunchBinding {
        schema_version: crate::REPO_FRONTIER_PROPOSAL_MODELING_LAUNCH_BINDING_SCHEMA_VERSION.into(),
        binding_record_id,
        proposal_modeling_request_id: request.request_id.clone(),
        proposal_id: proposal.proposal_id.clone(),
        proposal_payload_sha256: proposal.payload_sha256.clone(),
        job_id: job_id.clone(),
        binding_id: crate::EPIPHANY_MODELING_ROLE_BINDING_ID.into(),
        runtime_id: identity.runtime_id,
        thread_id: request.thread_id,
        launched_at: options.created_at.clone(),
        worker_launch_document_sha256,
        contract: crate::REPO_FRONTIER_PROPOSAL_MODELING_LAUNCH_BINDING_CONTRACT.into(),
    };
    let grant = crate::substrate_gate::substrate_gate_repo_access_grant_for_worker(
        format!("substrate-grant-{}", binding.job_id),
        binding.job_id.clone(),
        crate::EPIPHANY_MODELING_ROLE_BINDING_ID.into(),
        crate::EPIPHANY_MODELING_OWNER_ROLE.into(),
        authority_scope,
        false,
        options.created_at,
    );
    let snapshot = cache.snapshot_envelopes();
    let mut expected = Vec::new();
    let strong_versions = [
        (
            crate::RepoFrontierProposalModelingRequest::TYPE,
            request.request_id.as_str(),
        ),
        (
            crate::RepoFrontierWorkProposal::TYPE,
            proposal.proposal_id.as_str(),
        ),
    ];
    for (document_type, document_key) in strong_versions {
        let envelope = snapshot
            .iter()
            .find(|value| value.r#type == document_type && value.key == document_key)
            .ok_or_else(|| anyhow!("proposal Modeling launch lost a strong source"))?;
        expected.push(envelope.clone());
    }
    commit_current_work_launch(
        store_path,
        &cache,
        expected,
        prepared.envelopes,
        vec![
            cache.prepare_entry(&binding.binding_record_id, &binding)?.0,
            cache.prepare_entry(&grant.receipt_id, &grant)?.0,
        ],
        "proposal Modeling",
    )?;
    Ok(binding)
}

pub fn launch_current_frontier_verdict_modeling_work(
    store_path: impl AsRef<Path>,
    created_at: &str,
) -> Result<String> {
    let store_path = store_path.as_ref();
    if chrono::DateTime::parse_from_rfc3339(created_at).is_err() {
        return Err(anyhow!("frontier verdict Modeling launch time is invalid"));
    }
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let work = current_frontier_verdict_modeling_work(&cache)?
        .filter(|work| work.action == EpiphanyAgentPassContinuationAction::Launch)
        .ok_or_else(|| anyhow!("Mind has no launchable frontier verdict Modeling work"))?;
    let request = work.request;
    let verdict = cache
        .get::<crate::SoulVerdictReceipt>(&request.soul_verdict_receipt_id)?
        .ok_or_else(|| anyhow!("frontier verdict Modeling launch lost its Soul verdict"))?;
    let frontier_document = cache
        .get::<crate::EpiphanyRepoModelFrontierDocument>(&request.frontier_item_id)?
        .ok_or_else(|| anyhow!("frontier verdict Modeling launch lost its frontier document"))?;
    let frontier_item = frontier_document.value()?;
    let mind = crate::assemble_mind_view(store_path)?;
    let body_basis = mind
        .repository_body_observation
        .clone()
        .ok_or_else(|| anyhow!("frontier verdict Modeling launch requires admitted Body state"))?;
    let identity = cache
        .get::<crate::EpiphanyRuntimeIdentity>(crate::RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("frontier verdict Modeling launch requires runtime identity"))?;
    let attempt_ordinal = cache
        .get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?
        .into_iter()
        .filter(|launch| {
            launch.repo_frontier_modeling_request_id.as_deref() == Some(request.request_id.as_str())
        })
        .map(|launch| frontier_verdict_attempt_ordinal(&request.request_id, &launch.job_id))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max()
        .map_or(0, |ordinal| ordinal + 1);
    let job_id = format!(
        "frontier-verdict-modeling-{}-attempt-{attempt_ordinal}",
        request.request_id
    );
    let authority = crate::RepoFrontierVerdictModelingLaunchAuthority {
        request: request.clone(),
        frontier_item,
        soul_verdict: verdict.clone(),
    };
    let launch_document =
        crate::EpiphanyWorkerLaunchDocument::Role(crate::EpiphanyRoleWorkerLaunchDocument {
            thread_id: job_id.clone(),
            role_id: "modeling".into(),
            state_revision: 0,
            objective: None,
            dynamic_prompt_context: None,
            repository_body_observation_basis: Some(body_basis),
            proposal_modeling_context: None,
            frontier_verdict_modeling_context: Some(authority),
            frontier_planning_context: None,
            frontier_research_context: None,
            frontier_verification_context: None,
            frontier_plan_mind_context: None,
            imagination_consideration_context: None,
            admitted_model_direction_consideration_context: None,
            active_subgoal_id: None,
            active_subgoals: Vec::new(),
            active_graph_node_ids: Vec::new(),
            investigation_checkpoint: None,
            scratch: None,
            invariants: Vec::new(),
            graphs: None,
            recent_evidence: Vec::new(),
            recent_observations: Vec::new(),
            graph_frontier: None,
            graph_checkpoint: None,
            planning: None,
            churn: None,
        });
    let authority_scope = "epiphany.role.modeling".to_string();
    let output_contract_id = launch_document.output_contract_id().to_string();
    let prepared = crate::prepare_runtime_spine_heartbeat_job(
        &cache,
        crate::RuntimeSpineHeartbeatJobOptions {
            runtime_id: identity.runtime_id,
            display_name: "Epiphany Local".into(),
            session_id: crate::EPIPHANY_RUNTIME_ROOT_SESSION_ID.into(),
            objective: "Incorporate one exact Soul verdict into its routed frontier".into(),
            coordinator_note:
                "Frontier verdict current-work launch transaction opened this session.".into(),
            job_id: job_id.clone(),
            role: crate::EPIPHANY_MODELING_OWNER_ROLE.into(),
            binding_id: crate::EPIPHANY_MODELING_ROLE_BINDING_ID.into(),
            authority_scope: authority_scope.clone(),
            instruction: "Act as Epiphany Modeling. Incorporate the exact typed Soul verdict into only its routed frontier item and return the frontier-verdict result contract.".into(),
            launch_document,
            output_contract_id: output_contract_id.clone(),
            organ_launch_contract: crate::default_launch_organ_contract(
                &authority_scope,
                "role",
                &output_contract_id,
            ),
            proposal_modeling_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: Some(request.request_id.clone()),
            repo_frontier_research_request_id: None,
            repo_frontier_verification_request_id: None,
            created_at: created_at.to_string(),
        },
    )?;
    let grant = crate::substrate_gate::substrate_gate_repo_access_grant_for_worker(
        format!("substrate-grant-{job_id}"),
        job_id.clone(),
        crate::EPIPHANY_MODELING_ROLE_BINDING_ID.into(),
        crate::EPIPHANY_MODELING_OWNER_ROLE.into(),
        authority_scope,
        false,
        created_at.to_string(),
    );
    let snapshot = cache.snapshot_envelopes();
    let mut expected = Vec::new();
    for (document_type, document_key) in [
        (
            crate::RepoFrontierModelingRequest::TYPE,
            request.request_id.as_str(),
        ),
        (crate::SoulVerdictReceipt::TYPE, verdict.receipt_id.as_str()),
        (
            crate::EpiphanyRepoModelFrontierDocument::TYPE,
            request.frontier_item_id.as_str(),
        ),
    ] {
        let envelope = snapshot
            .iter()
            .find(|value| value.r#type == document_type && value.key == document_key)
            .ok_or_else(|| anyhow!("frontier verdict Modeling launch lost a strong source"))?;
        expected.push(envelope.clone());
    }
    commit_current_work_launch(
        store_path,
        &cache,
        expected,
        prepared.envelopes,
        vec![cache.prepare_entry(&grant.receipt_id, &grant)?.0],
        "frontier verdict Modeling",
    )?;
    Ok(job_id)
}

fn build_proposal_modeling_context_projection(
    request: &crate::RepoFrontierProposalModelingRequest,
    proposal: &crate::RepoFrontierWorkProposal,
    model: &crate::EpiphanyRepoModelView,
) -> Result<crate::RepoFrontierProposalModelingContextProjection> {
    Ok(crate::RepoFrontierProposalModelingContextProjection {
        schema_version: crate::REPO_FRONTIER_PROPOSAL_MODELING_CONTEXT_SCHEMA_VERSION.into(),
        contract: crate::REPO_FRONTIER_PROPOSAL_MODELING_CONTEXT_CONTRACT.into(),
        request_id: request.request_id.clone(),
        proposal_id: proposal.proposal_id.clone(),
        proposal_payload_sha256: proposal.payload_sha256.clone(),
        runtime_id: request.runtime_id.clone(),
        thread_id: request.thread_id.clone(),
        repository: request.repository.clone(),
        workspace: request.workspace.clone(),
        source_kind: proposal.source_kind,
        source_actor: proposal.source_actor.clone(),
        source_ref: proposal.source_ref.clone(),
        title: proposal.title.clone(),
        body: proposal.body.clone(),
        desired_outcome: proposal.desired_outcome.clone(),
        constraints: proposal.constraints.clone(),
        scope_hints: proposal.scope_hints.clone(),
        evidence_refs: proposal.evidence_refs.clone(),
        public_source_refs: proposal.public_source_refs.clone(),
        private_state_included: proposal.private_state_included,
        model_projection_digest: model.projection_digest.clone(),
        model_source_documents: model.source_documents.clone(),
    })
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
    use epiphany_state_model::{
        EpiphanyMemoryDomain, EpiphanyMemoryLifecycle, EpiphanyMemoryNode, EpiphanyMemoryNodeKind,
        EpiphanyMemoryProfile, RepoFrontierItem, RepoFrontierStatus,
    };

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

    #[test]
    fn keyed_modeling_and_research_lifecycles_ignore_thread_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("body-decision.cc");
        crate::initialize_runtime_spine(
            &store,
            crate::RuntimeSpineInitOptions {
                runtime_id: "runtime".into(),
                display_name: "Body decision".into(),
                created_at: "2026-08-17T00:00:00Z".into(),
            },
        )?;
        crate::runtime_spine::tests::bind_test_runtime_swarm(&store, "swarm")?;
        crate::runtime_spine::tests::bind_test_repository_body(&store, "workspace")?;
        let body = crate::observe_runtime_repository_body_basis(&store)?;
        let domain = EpiphanyMemoryDomain {
            id: "body-domain".into(),
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            title: "Body".into(),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        };
        let node = EpiphanyMemoryNode {
            id: "body-node".into(),
            domain_id: domain.id.clone(),
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            kind: EpiphanyMemoryNodeKind::Module,
            title: "Body node".into(),
            claim: "The Body is typed".into(),
            question: "What does the Body contain?".into(),
            action_implication: "Model it".into(),
            source_hashes: vec!["anchor:missing".into()],
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        };
        crate::initialize_keyed_repo_model(
            &store,
            &crate::EpiphanyRepoModelSeed::new(
                "body-seed",
                "body-graph",
                "swarm",
                "workspace",
                body.body_binding_sha256.clone(),
                crate::EpiphanyRepoModelSeedDocuments {
                    domains: vec![domain],
                    nodes: vec![node],
                    edges: Vec::new(),
                    summaries: Vec::new(),
                    frontier: Vec::new(),
                    lifecycle_receipts: Vec::new(),
                },
            )?,
            "2026-08-17T00:00:01Z",
        )?;
        crate::reset_repository_body_read_counters();
        let projected_work = current_body_modeling_work(&store)?;
        assert_eq!(projected_work.body_basis, body);
        assert_eq!(crate::repository_body_read_counters(), (0, 0));
        let current_work = project_current_work(&store)?;
        assert_eq!(current_work.body_modeling, Some(projected_work.clone()));
        assert_eq!(
            current_work.body_modeling_action,
            Some(EpiphanyAgentPassContinuationAction::Launch)
        );
        assert_eq!(
            current_work.mind_projection_digest,
            crate::assemble_mind_view(&store)?.projection_digest
        );
        assert_eq!(crate::repository_body_read_counters(), (0, 0));
        let resident_pressure = crate::resident_self_body_modeling_pressure(&store, 1)?
            .expect("unresolved Body work must create Resident Self pressure");
        assert_eq!(
            resident_pressure.provenance_ref,
            format!(
                "{}{}",
                crate::RESIDENT_SELF_BODY_MODELING_PROVENANCE_PREFIX,
                projected_work.work_id
            )
        );
        let scheduled = launch_current_body_modeling_work(
            &store,
            EpiphanyBodyModelingLaunchOptions {
                job_id: "body-scheduled-job".into(),
                created_at: "2026-08-17T00:00:02Z".into(),
            },
        )?;
        assert_eq!(scheduled.work_id, projected_work.work_id);
        assert_eq!(scheduled.attempt_ordinal, 0);
        let scheduled_work = project_current_work(&store)?;
        assert!(scheduled_work.body_modeling.is_none());
        assert_eq!(
            scheduled_work.body_modeling_action,
            Some(EpiphanyAgentPassContinuationAction::Wait)
        );
        assert!(crate::resident_self_body_modeling_pressure(&store, 2)?.is_none());
        let mut scheduled_cache = crate::runtime_spine_cache(&store)?;
        scheduled_cache.pull_all_backing_stores()?;
        assert!(
            scheduled_cache
                .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
                .is_none()
        );
        assert!(
            scheduled_cache
                .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>("body-scheduled-job")?
                .is_some()
        );
        assert!(
            scheduled_cache
                .get::<crate::SubstrateGateRepoAccessGrantReceipt>(
                    "substrate-grant-body-scheduled-job"
                )?
                .is_some()
        );
        assert!(
            launch_current_body_modeling_work(
                &store,
                EpiphanyBodyModelingLaunchOptions {
                    job_id: "body-scheduled-job-duplicate".into(),
                    created_at: "2026-08-17T00:00:03Z".into(),
                },
            )
            .is_err()
        );
        assert_eq!(crate::repository_body_read_counters(), (0, 0));
        let mut cache = crate::runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let launch = cache
            .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>("body-scheduled-job")?
            .unwrap();
        let reasoning_basis = crate::worker_reasoning_basis(&store, &launch)?;
        crate::put_reasoning_basis(&store, &reasoning_basis)?;
        let mut native = epiphany_model_adapter::EpiphanyModelRequest::new(
            "body-request",
            "body-conversation",
            "openai-codex",
            "gpt-test",
            "model",
        );
        native.reasoning_basis_id = Some(reasoning_basis.basis_id.clone());
        native.source_worker_job_id = Some(launch.job_id.clone());
        let provider = epiphany_openai_adapter::request_from_native(&native);
        let context =
            crate::EpiphanyDecisionContext::new(&reasoning_basis, native, provider, Vec::new())?;
        cache.put(&context.context_id, &context)?;
        let result = crate::EpiphanyRuntimeRoleWorkerResult {
            schema_version: crate::RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION.into(),
            result_id: "body-result".into(),
            job_id: launch.job_id.clone(),
            role_id: "modeling".into(),
            verdict: "checkpoint-ready".into(),
            summary: "modeled".into(),
            next_safe_move: "admit".into(),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: Vec::new(),
            frontier_node_ids: Vec::new(),
            evidence_ids: Vec::new(),
            artifact_refs: Vec::new(),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            state_patch_msgpack: None,
            self_patch_msgpack: None,
            item_error: None,
            metadata: Default::default(),
            repo_model_mutation_proposal_msgpack: None,
            verification_request_id: None,
            frontier_route_id: None,
            repo_frontier_modeling_request_id: None,
            proposal_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_candidate_msgpack: None,
            frontier_plan_mind_request_id: None,
            frontier_plan_mind_decision_msgpack: None,
            repository_body_observation_basis: Some(body.clone()),
            imagination_consideration_request_id: None,
            imagination_consideration_candidate_msgpack: None,
            admitted_model_direction_consideration_request_id: None,
            admitted_model_direction_consideration_result_msgpack: None,
            decision_context_id: context.context_id,
        };
        cache.put(&result.job_id, &result)?;
        let mut job = cache
            .get::<crate::EpiphanyRuntimeJob>(&result.job_id)?
            .unwrap();
        job.status = crate::EpiphanyRuntimeJobStatus::Completed;
        job.updated_at = "2026-08-17T00:00:04Z".into();
        cache.put(&job.job_id, &job)?;
        assert_eq!(
            project_current_work(&store)?.body_modeling_action,
            Some(EpiphanyAgentPassContinuationAction::Review)
        );
        let continuation = crate::EpiphanyCoordinatorRunReceipt {
            schema_version: crate::COORDINATOR_RUN_RECEIPT_SCHEMA_VERSION.into(),
            receipt_id: "body-continuation".into(),
            session_id: crate::EPIPHANY_RUNTIME_ROOT_SESSION_ID.into(),
            thread_id: "transport-thread".into(),
            mode: "execute".into(),
            status: "planned".into(),
            final_action: "waitForModelingResult".into(),
            final_reason: None,
            step_count: 1,
            created_at: "2026-08-17T00:00:04Z".into(),
            model_provider: None,
            runtime_store: store.display().to_string(),
            artifact_refs: Vec::new(),
            sealed_artifact_refs: Vec::new(),
            metadata: Default::default(),
            resident_grant_id: None,
            resident_launch_digest: None,
            resident_policy_digest: None,
            resident_argv_digest: None,
            resident_objective_digest: None,
            resident_release_commit: None,
            resident_release_manifest_digest: None,
            resident_executable_digest: None,
            final_runtime_job_id: Some(result.job_id.clone()),
        };
        assert_eq!(
            crate::resident_self::resident_self_safe_continuation_action(&store, &continuation)?,
            Some("reviewModelingResult".into())
        );
        let receipt = accept_body_modeling_result(&store, &result.job_id, "2026-08-17T00:00:05Z")?;
        assert_eq!(
            receipt.authority,
            crate::EpiphanyMindCommitAuthority::ModelDecisionContext {
                decision_context_id: result.decision_context_id.clone(),
            }
        );
        let completed_work = project_current_work(&store)?;
        assert!(completed_work.body_modeling.is_none());
        assert!(completed_work.body_modeling_action.is_none());
        assert!(
            crate::resident_self::resident_self_safe_continuation_action(&store, &continuation)?
                .is_none()
        );
        assert_eq!(crate::repository_body_read_counters(), (0, 0));

        let proposal = crate::intake_user_repo_frontier_proposal(
            &store,
            crate::RepoFrontierUserProposalInput {
                proposal_id: "proposal-1".into(),
                source_actor: "operator".into(),
                source_ref: "operator://proposal-1".into(),
                repository: "epiphany".into(),
                workspace: "workspace".into(),
                thread_id: "creation-provenance-only".into(),
                runtime_id: "runtime".into(),
                title: "Model one proposal".into(),
                body: "Add one typed frontier without touching unrelated Mind state.".into(),
                desired_outcome: "One inspectable frontier".into(),
                constraints: Vec::new(),
                scope_hints: vec!["epiphany-core/src".into()],
                evidence_refs: Vec::new(),
                public_source_refs: Vec::new(),
                proposed_at: "2026-08-17T00:00:06Z".into(),
                private_state_included: false,
            },
        )?;
        let request = crate::select_repo_frontier_work_proposal_for_modeling(
            &store,
            &proposal.proposal_id,
            "2026-08-17T00:00:07Z",
        )?;
        let proposal_work = project_current_work(&store)?
            .proposal_modeling
            .expect("selected proposal must become current work");
        assert_eq!(proposal_work.request, request);
        assert_eq!(
            proposal_work.action,
            EpiphanyAgentPassContinuationAction::Launch
        );
        let racers = (0..2)
            .map(|_| {
                let store = store.clone();
                std::thread::spawn(move || {
                    launch_current_proposal_modeling_work(
                        &store,
                        EpiphanyProposalModelingLaunchOptions {
                            created_at: "2026-08-17T00:00:08Z".into(),
                        },
                    )
                })
            })
            .collect::<Vec<_>>();
        let race_results = racers
            .into_iter()
            .map(|racer| racer.join().expect("proposal launch racer panicked"))
            .collect::<Vec<_>>();
        let proposal_launch = race_results
            .iter()
            .find_map(|result| result.as_ref().ok())
            .cloned()
            .expect("one proposal launch racer must commit");
        assert!(race_results.iter().all(|result| {
            result
                .as_ref()
                .map(|binding| binding == &proposal_launch)
                .unwrap_or(true)
        }));
        assert_eq!(
            proposal_launch.proposal_modeling_request_id,
            request.request_id
        );
        assert_eq!(
            proposal_launch.job_id,
            format!("proposal-modeling-{}-attempt-0", request.request_id)
        );
        assert_eq!(
            project_current_work(&store)?
                .proposal_modeling
                .expect("launched proposal remains current")
                .action,
            EpiphanyAgentPassContinuationAction::Wait
        );
        assert!(
            launch_current_proposal_modeling_work(
                &store,
                EpiphanyProposalModelingLaunchOptions {
                    created_at: "2026-08-17T00:00:09Z".into(),
                },
            )
            .is_err()
        );
        let mut final_cache = crate::runtime_spine_cache(&store)?;
        final_cache.pull_all_backing_stores()?;
        assert_eq!(
            final_cache
                .get_all::<crate::RepoFrontierProposalModelingLaunchBinding>()?
                .into_iter()
                .filter(|binding| binding.proposal_modeling_request_id == request.request_id)
                .count(),
            1
        );
        assert!(
            final_cache
                .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
                .is_none()
        );
        let proposal_runtime_launch = final_cache
            .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&proposal_launch.job_id)?
            .expect("proposal runtime launch");
        let proposal_document = proposal_runtime_launch.launch_document()?;
        let crate::EpiphanyWorkerLaunchDocument::Role(proposal_document) = proposal_document else {
            panic!("proposal Modeling must be a role pass")
        };
        assert_eq!(proposal_document.state_revision, 0);
        assert!(proposal_document.proposal_modeling_context.is_some());
        assert!(
            proposal_document
                .repository_body_observation_basis
                .is_some()
        );

        let proposal_reasoning_basis =
            crate::worker_reasoning_basis(&store, &proposal_runtime_launch)?;
        crate::put_reasoning_basis(&store, &proposal_reasoning_basis)?;
        let mut proposal_native = epiphany_model_adapter::EpiphanyModelRequest::new(
            "proposal-request",
            "proposal-conversation",
            "openai-codex",
            "gpt-test",
            "model",
        );
        proposal_native.reasoning_basis_id = Some(proposal_reasoning_basis.basis_id.clone());
        proposal_native.source_worker_job_id = Some(proposal_launch.job_id.clone());
        let proposal_provider = epiphany_openai_adapter::request_from_native(&proposal_native);
        let proposal_context = crate::EpiphanyDecisionContext::new(
            &proposal_reasoning_basis,
            proposal_native,
            proposal_provider,
            Vec::new(),
        )?;
        final_cache.put(&proposal_context.context_id, &proposal_context)?;
        let result_id = "proposal-result";
        let mutation = crate::EpiphanyRepoModelMutationProposal::new(
            format!("repo-model-mutation-proposal-{}", proposal_launch.job_id),
            request.request_id.clone(),
            result_id,
            vec![proposal.proposal_id.clone()],
            body.clone(),
            vec![crate::EpiphanyRepoModelMutationOperation::PutFrontier {
                item: RepoFrontierItem {
                    id: "proposal-frontier".into(),
                    migration_body: "epiphany".into(),
                    question: "Should this proposal advance?".into(),
                    gap: "The proposal needs an adopted plan.".into(),
                    target_claim_ids: vec!["body-node".into()],
                    source_scope: vec!["epiphany-core/src".into()],
                    recommended_next_organ: "Imagination".into(),
                    status: RepoFrontierStatus::Active,
                    evidence_refs: vec![proposal.proposal_id.clone()],
                    ..Default::default()
                },
            }],
        )?;
        let proposal_result = crate::EpiphanyRuntimeRoleWorkerResult {
            schema_version: crate::RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION.into(),
            result_id: result_id.into(),
            job_id: proposal_launch.job_id.clone(),
            role_id: "modeling".into(),
            verdict: "checkpoint-update-needed".into(),
            summary: "proposal modeled".into(),
            next_safe_move: "admit".into(),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: Vec::new(),
            frontier_node_ids: vec!["body-node".into()],
            evidence_ids: vec![proposal.proposal_id.clone()],
            artifact_refs: Vec::new(),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            state_patch_msgpack: None,
            self_patch_msgpack: None,
            item_error: None,
            metadata: Default::default(),
            repo_model_mutation_proposal_msgpack: Some(rmp_serde::to_vec_named(&mutation)?),
            verification_request_id: None,
            frontier_route_id: None,
            repo_frontier_modeling_request_id: None,
            proposal_modeling_request_id: Some(request.request_id.clone()),
            repo_frontier_research_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_candidate_msgpack: None,
            frontier_plan_mind_request_id: None,
            frontier_plan_mind_decision_msgpack: None,
            repository_body_observation_basis: Some(body.clone()),
            imagination_consideration_request_id: None,
            imagination_consideration_candidate_msgpack: None,
            admitted_model_direction_consideration_request_id: None,
            admitted_model_direction_consideration_result_msgpack: None,
            decision_context_id: proposal_context.context_id.clone(),
        };
        let process = crate::ProcessInstanceIdentity {
            process_id: 42,
            creation_token: 7,
            created_at_rfc3339: Some("2026-08-17T00:00:10Z".into()),
            executable_path: "proposal-worker".into(),
        };
        let activation_token = "proposal-activation";
        crate::claim_runtime_worker_process(
            &store,
            &proposal_launch.job_id,
            &process,
            &format!("{:x}", Sha256::digest(activation_token.as_bytes())),
            "2026-08-17T00:00:10Z",
        )?;
        crate::activate_runtime_worker_process(
            &store,
            &proposal_launch.job_id,
            &process,
            activation_token,
            "2026-08-17T00:00:11Z",
        )?;
        crate::put_runtime_role_worker_result(&store, &proposal_result)?;
        crate::complete_runtime_job(
            &store,
            crate::RuntimeSpineJobResultOptions {
                result_id: format!("runtime-result-{}", proposal_launch.job_id),
                job_id: proposal_launch.job_id.clone(),
                completed_at: "2026-08-17T00:00:12Z".into(),
                verdict: proposal_result.verdict.clone(),
                summary: proposal_result.summary.clone(),
                next_safe_move: proposal_result.next_safe_move.clone(),
                evidence_refs: proposal_result.evidence_ids.clone(),
                artifact_refs: Vec::new(),
                decision_context_id: Some(proposal_context.context_id.clone()),
            },
        )?;
        assert_eq!(
            project_current_work(&store)?
                .proposal_modeling
                .expect("terminal proposal remains current until admission")
                .action,
            EpiphanyAgentPassContinuationAction::Review
        );
        let proposal_commit = accept_proposal_modeling_result(
            &store,
            &proposal_launch.job_id,
            "2026-08-17T00:00:13Z",
        )?;
        assert_eq!(
            proposal_commit.invariant_owner,
            "Modeling.proposal_frontier"
        );
        assert!(project_current_work(&store)?.proposal_modeling.is_none());
        let mut accepted_cache = crate::runtime_spine_cache(&store)?;
        accepted_cache.pull_all_backing_stores()?;
        assert!(
            accepted_cache
                .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
                .is_none()
        );

        let model = crate::assemble_repo_model_view(&store)?;
        let frontier_item = model
            .frontier
            .iter()
            .find(|item| item.id == "proposal-frontier")
            .cloned()
            .expect("proposal admission created its frontier");
        let frontier_hash = format!(
            "{:x}",
            Sha256::digest(rmp_serde::to_vec_named(&frontier_item)?)
        );
        let route = crate::RepoFrontierRoute {
            schema_version: crate::REPO_FRONTIER_ROUTE_SCHEMA_VERSION.into(),
            route_id: "route-proposal-frontier".into(),
            next_organ: crate::RepoFrontierNextOrgan::Hands,
            model_projection_digest: model.projection_digest.clone(),
            model_source_documents: model.source_documents.clone(),
            frontier_item_id: frontier_item.id.clone(),
            frontier_item_hash: frontier_hash.clone(),
            migration_body: frontier_item.migration_body.clone(),
            question: frontier_item.question.clone(),
            gap: frontier_item.gap.clone(),
            target_claim_ids: frontier_item.target_claim_ids.clone(),
            source_scope: frontier_item.source_scope.clone(),
            adopted_plan: frontier_item.adopted_plan.clone(),
            selected_at: "2026-08-17T00:00:14Z".into(),
            contract: crate::REPO_FRONTIER_ROUTE_CONTRACT.into(),
        };
        accepted_cache.put(&route.route_id, &route)?;

        let hands_job_id = "hands-verification-fixture";
        let grant = crate::substrate_gate_coordinator_implementation_grant(
            "substrate-grant-hands-verification-fixture".into(),
            hands_job_id.into(),
            route.source_scope.clone(),
            "2026-08-17T00:00:14.100Z".into(),
        );
        crate::put_substrate_gate_repo_access_grant_receipt(&store, &grant)?;
        let intent = crate::HandsActionIntent {
            schema_version: crate::HANDS_ACTION_INTENT_SCHEMA_VERSION.into(),
            intent_id: "hands-intent-verification-fixture".into(),
            runtime_job_id: hands_job_id.into(),
            binding_id: "implementation-worker".into(),
            role: "epiphany-hands".into(),
            authority_scope: "epiphany.role.implementation".into(),
            requested_action: "continueImplementation".into(),
            requested_paths: route.source_scope.clone(),
            substrate_gate_grant_receipt_id: grant.receipt_id.clone(),
            requested_at: "2026-08-17T00:00:14.200Z".into(),
            contract: "The exact Hands consequence under Verification.".into(),
            frontier_route_id: String::new(),
            plan_candidate_sha256: String::new(),
            plan_action: String::new(),
        };
        crate::put_hands_action_intent(&store, &intent)?;
        let mut hands_review = crate::hands_action_review_for_intent(
            "hands-review-verification-fixture".into(),
            &intent,
            "approved".into(),
            vec!["patch".into(), "command".into(), "commit".into()],
            vec!["Bounded fixture consequence is authorized.".into()],
            "2026-08-17T00:00:14.300Z".into(),
        );
        hands_review.required_receipts = vec![
            crate::HANDS_PATCH_RECEIPT_TYPE.into(),
            crate::HANDS_COMMAND_RECEIPT_TYPE.into(),
            crate::HANDS_COMMIT_RECEIPT_TYPE.into(),
        ];
        crate::put_hands_action_review(&store, &hands_review)?;
        let hands_authority = crate::RepoFrontierHandsAuthority {
            schema_version: crate::REPO_FRONTIER_HANDS_AUTHORITY_SCHEMA_VERSION.into(),
            authority_id: "repo-frontier-hands-authority-verification-fixture".into(),
            route_id: route.route_id.clone(),
            model_projection_digest: route.model_projection_digest.clone(),
            model_source_documents: route.model_source_documents.clone(),
            frontier_item_id: route.frontier_item_id.clone(),
            frontier_item_hash: route.frontier_item_hash.clone(),
            hands_intent_id: intent.intent_id.clone(),
            hands_review_id: hands_review.review_id.clone(),
            substrate_grant_receipt_id: grant.receipt_id.clone(),
            requested_paths: route.source_scope.clone(),
            granted_at: hands_review.reviewed_at.clone(),
            contract: crate::REPO_FRONTIER_HANDS_AUTHORITY_CONTRACT.into(),
        };
        crate::put_repo_frontier_hands_authority(&store, &hands_authority)?;
        let hands_patch = crate::hands_patch_receipt_for_review(
            "hands-patch-verification-fixture".into(),
            &intent,
            &hands_review,
            route.source_scope.clone(),
            "Applied the exact bounded change.".into(),
            "2026-08-17T00:00:14.400Z".into(),
        );
        crate::put_hands_patch_receipt(&store, &hands_patch)?;
        let hands_command = crate::hands_command_receipt_for_review(
            "hands-command-verification-fixture".into(),
            &intent,
            &hands_review,
            "cargo test exact-verification-fixture".into(),
            "0".into(),
            "verification.stdout".into(),
            "verification.stderr".into(),
            "The bounded command passed.".into(),
            "2026-08-17T00:00:14.500Z".into(),
        );
        crate::put_hands_command_receipt(&store, &hands_command)?;
        let hands_commit = crate::hands_commit_receipt_for_review(
            "hands-commit-verification-fixture".into(),
            &intent,
            &hands_review,
            "0123456789abcdef0123456789abcdef01234567".into(),
            "codex/verification-fixture".into(),
            route.source_scope.clone(),
            "Committed the exact bounded consequence.".into(),
            "2026-08-17T00:00:14.600Z".into(),
        );
        crate::put_hands_commit_receipt(&store, &hands_commit)?;

        accepted_cache.pull_all_backing_stores()?;
        let mut verification_requests = accepted_cache
            .get_all::<crate::RepoFrontierVerificationRequest>()?
            .into_iter()
            .filter(|request| request.route_id == route.route_id)
            .collect::<Vec<_>>();
        assert_eq!(verification_requests.len(), 1);
        let verification_request = verification_requests.pop().unwrap();
        assert_eq!(
            project_current_work(&store)?
                .verification
                .expect("Hands commit creates exact Verification work")
                .action,
            EpiphanyAgentPassContinuationAction::Launch
        );
        let verification_job_id =
            launch_current_frontier_verification_work(&store, "2026-08-17T00:00:14.700Z")?;
        accepted_cache.pull_all_backing_stores()?;
        let verification_launch = accepted_cache
            .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&verification_job_id)?
            .expect("exact Verification launch");
        assert_eq!(
            verification_launch
                .repo_frontier_verification_request_id
                .as_deref(),
            Some(verification_request.request_id.as_str())
        );
        let crate::EpiphanyWorkerLaunchDocument::Role(verification_document) =
            verification_launch.launch_document()?
        else {
            panic!("Verification must launch as a role pass")
        };
        assert!(verification_document.dynamic_prompt_context.is_none());
        assert_eq!(
            verification_document
                .frontier_verification_context
                .as_ref()
                .expect("sealed typed Verification projection")
                .request,
            verification_request
        );
        let verification_basis = crate::worker_reasoning_basis(&store, &verification_launch)?;
        crate::put_reasoning_basis(&store, &verification_basis)?;
        let mut verification_native = epiphany_model_adapter::EpiphanyModelRequest::new(
            "verification-model-request",
            "verification-conversation",
            "openai-codex",
            "gpt-test",
            "verify",
        );
        verification_native.reasoning_basis_id = Some(verification_basis.basis_id.clone());
        verification_native.source_worker_job_id = Some(verification_job_id.clone());
        let verification_provider =
            epiphany_openai_adapter::request_from_native(&verification_native);
        crate::open_runtime_model_execution(
            &store,
            crate::RuntimeSpineSessionOptions {
                session_id: "verification-model-session".into(),
                objective: "Verify the exact Hands consequence.".into(),
                created_at: "2026-08-17T00:00:14.750Z".into(),
                coordinator_note: "Bound Verification model pass.".into(),
            },
            crate::RuntimeSpineJobOptions {
                job_id: "verification-model-job".into(),
                session_id: "verification-model-session".into(),
                role: "openai-model".into(),
                created_at: "2026-08-17T00:00:14.750Z".into(),
                summary: "Bound Verification inference.".into(),
                artifact_refs: Vec::new(),
            },
            &verification_native,
            &verification_provider,
            "2026-08-17T00:00:14.750Z",
        )?;
        let verification_context = crate::EpiphanyDecisionContext::new(
            &verification_basis,
            verification_native,
            verification_provider,
            Vec::new(),
        )?;
        crate::put_decision_context(&store, &verification_context)?;
        let verification_process = crate::ProcessInstanceIdentity {
            process_id: 42,
            creation_token: 7,
            created_at_rfc3339: Some("2026-08-17T00:00:14.800Z".into()),
            executable_path: "verification-worker".into(),
        };
        let verification_activation = "verification-activation";
        crate::claim_runtime_worker_process(
            &store,
            &verification_job_id,
            &verification_process,
            &format!("{:x}", Sha256::digest(verification_activation.as_bytes())),
            "2026-08-17T00:00:14.800Z",
        )?;
        crate::activate_runtime_worker_process(
            &store,
            &verification_job_id,
            &verification_process,
            verification_activation,
            "2026-08-17T00:00:14.900Z",
        )?;
        let verification_result = crate::EpiphanyRuntimeRoleWorkerResult {
            schema_version: crate::RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION.into(),
            result_id: "verification-result".into(),
            job_id: verification_job_id.clone(),
            role_id: "verification".into(),
            verdict: "pass".into(),
            summary: "verified exact consequence".into(),
            next_safe_move: "incorporate verdict".into(),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: Vec::new(),
            frontier_node_ids: vec!["body-node".into()],
            evidence_ids: vec![hands_commit.receipt_id.clone()],
            artifact_refs: Vec::new(),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            state_patch_msgpack: None,
            self_patch_msgpack: None,
            item_error: None,
            metadata: Default::default(),
            repo_model_mutation_proposal_msgpack: None,
            verification_request_id: Some(verification_request.request_id.clone()),
            frontier_route_id: Some(route.route_id.clone()),
            repo_frontier_modeling_request_id: None,
            proposal_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_candidate_msgpack: None,
            frontier_plan_mind_request_id: None,
            frontier_plan_mind_decision_msgpack: None,
            repository_body_observation_basis: None,
            imagination_consideration_request_id: None,
            imagination_consideration_candidate_msgpack: None,
            admitted_model_direction_consideration_request_id: None,
            admitted_model_direction_consideration_result_msgpack: None,
            decision_context_id: verification_context.context_id.clone(),
        };
        crate::put_runtime_role_worker_result(&store, &verification_result)?;
        crate::complete_runtime_job(
            &store,
            crate::RuntimeSpineJobResultOptions {
                result_id: format!("runtime-result-{verification_job_id}"),
                job_id: verification_job_id.clone(),
                completed_at: "2026-08-17T00:00:15Z".into(),
                verdict: verification_result.verdict.clone(),
                summary: verification_result.summary.clone(),
                next_safe_move: verification_result.next_safe_move.clone(),
                evidence_refs: verification_result.evidence_ids.clone(),
                artifact_refs: Vec::new(),
                decision_context_id: Some(verification_context.context_id.clone()),
            },
        )?;
        assert_eq!(
            project_current_work(&store)?
                .verification
                .expect("terminal Verification work awaits admission")
                .action,
            EpiphanyAgentPassContinuationAction::Review
        );

        // The same terminal decision must refuse without mutation if its exact
        // frontier changes. There is no silent rebase onto newer strong state.
        let verification_conflict_store = temp.path().join("verification-conflict.cc");
        std::fs::copy(&store, &verification_conflict_store)?;
        let mut competing_verification_item = frontier_item.clone();
        competing_verification_item.gap =
            "A competing decision changed the exact verified frontier.".into();
        competing_verification_item.updated_at = Some("2026-08-17T00:00:15.050Z".into());
        let competing_verification_proposal = crate::EpiphanyRepoModelMutationProposal::new(
            "repo-model-mutation-proposal-verification-conflict",
            "verification-conflict-request",
            "verification-conflict-result",
            vec![hands_commit.receipt_id.clone()],
            body.clone(),
            vec![crate::EpiphanyRepoModelMutationOperation::PutFrontier {
                item: competing_verification_item,
            }],
        )?;
        let competing_verification_plan = crate::plan_repo_model_mutation(
            &verification_conflict_store,
            &competing_verification_proposal,
        )?;
        let mut verification_conflict_cache =
            crate::runtime_spine_cache(&verification_conflict_store)?;
        verification_conflict_cache.pull_all_backing_stores()?;
        let conflict_provenance = verification_conflict_cache
            .get_envelope::<crate::HandsCommitReceipt>(&hands_commit.receipt_id)?
            .expect("typed Hands conflict provenance");
        assert!(matches!(
            crate::commit_typed_organ_mind_mutation(
                &verification_conflict_store,
                "Modeling",
                conflict_provenance,
                "Modeling.verification_conflict_fixture",
                competing_verification_plan.strong_reads,
                competing_verification_plan.writes,
                "2026-08-17T00:00:15.050Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));
        verification_conflict_cache.pull_all_backing_stores()?;
        let conflict_before = verification_conflict_cache.snapshot_envelopes();
        let conflict_error = accept_frontier_verification_result(
            &verification_conflict_store,
            &verification_job_id,
            "2026-08-17T00:00:16Z",
        )
        .expect_err("stale exact frontier must block Verification admission");
        assert!(conflict_error.to_string().contains("frontier"));
        verification_conflict_cache.pull_all_backing_stores()?;
        assert_eq!(
            verification_conflict_cache.snapshot_envelopes(),
            conflict_before
        );

        // An unrelated keyed graph write after inference must not stale this
        // decision. Verification owns the exact frontier and Hands chain, not
        // a global RepoModel revision.
        let mut concurrent_node = model
            .nodes
            .iter()
            .find(|node| node.id == "body-node")
            .cloned()
            .expect("seeded Body node");
        concurrent_node.id = "verification-concurrent-node".into();
        concurrent_node.title = "Concurrent Verification neighbor".into();
        concurrent_node.claim = "Disjoint keyed Mind writes merge.".into();
        concurrent_node.updated_at = Some("2026-08-17T00:00:15.100Z".into());
        let concurrent_proposal = crate::EpiphanyRepoModelMutationProposal::new(
            "repo-model-mutation-proposal-verification-concurrent",
            verification_request.request_id.clone(),
            verification_result.result_id.clone(),
            vec![hands_commit.receipt_id.clone()],
            body.clone(),
            vec![crate::EpiphanyRepoModelMutationOperation::PutNode {
                node: concurrent_node,
            }],
        )?;
        let concurrent_plan = crate::plan_repo_model_mutation(&store, &concurrent_proposal)?;
        accepted_cache.pull_all_backing_stores()?;
        let hands_provenance = accepted_cache
            .get_envelope::<crate::HandsCommitReceipt>(&hands_commit.receipt_id)?
            .expect("typed Hands provenance");
        assert!(matches!(
            crate::commit_typed_organ_mind_mutation(
                &store,
                "Modeling",
                hands_provenance,
                "Modeling.verification_concurrent_fixture",
                concurrent_plan.strong_reads,
                concurrent_plan.writes,
                "2026-08-17T00:00:15.100Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));

        let verification_commit = accept_frontier_verification_result(
            &store,
            &verification_job_id,
            "2026-08-17T00:00:16Z",
        )?;
        assert_eq!(verification_commit.invariant_owner, "Soul.verification");
        assert!(project_current_work(&store)?.verification.is_none());
        let replayed_verification_commit = accept_frontier_verification_result(
            &store,
            &verification_job_id,
            "2026-08-17T00:00:16Z",
        )?;
        assert_eq!(replayed_verification_commit, verification_commit);
        accepted_cache.pull_all_backing_stores()?;
        let verdict = accepted_cache
            .get::<crate::SoulVerdictReceipt>(&format!(
                "soul-verdict-{}",
                verification_result.result_id
            ))?
            .expect("Verification admission writes its Soul verdict");
        let verdict_request =
            crate::runtime_spine::derive_repo_frontier_modeling_request(&accepted_cache, &verdict)?;
        assert!(
            accepted_cache
                .get::<crate::RepoFrontierModelingRequest>(&verdict_request.request_id)?
                .is_some()
        );
        assert_eq!(
            project_current_work(&store)?
                .frontier_verdict_modeling
                .expect("Soul verdict creates frontier Modeling work")
                .action,
            EpiphanyAgentPassContinuationAction::Launch
        );
        let verdict_racers = (0..2)
            .map(|_| {
                let store = store.clone();
                std::thread::spawn(move || {
                    launch_current_frontier_verdict_modeling_work(&store, "2026-08-17T00:00:17Z")
                })
            })
            .collect::<Vec<_>>();
        let verdict_launches = verdict_racers
            .into_iter()
            .map(|racer| racer.join().expect("verdict launch racer panicked"))
            .collect::<Vec<_>>();
        let verdict_job_id = verdict_launches
            .iter()
            .find_map(|result| result.as_ref().ok())
            .cloned()
            .expect("one frontier verdict launch racer must commit");
        assert!(
            verdict_launches
                .iter()
                .filter(|result| result.is_ok())
                .count()
                == 1
        );
        assert_eq!(
            verdict_job_id,
            format!(
                "frontier-verdict-modeling-{}-attempt-0",
                verdict_request.request_id
            )
        );
        let mut verdict_cache = crate::runtime_spine_cache(&store)?;
        verdict_cache.pull_all_backing_stores()?;
        let verdict_launch = verdict_cache
            .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&verdict_job_id)?
            .expect("frontier verdict runtime launch");
        let crate::EpiphanyWorkerLaunchDocument::Role(verdict_document) =
            verdict_launch.launch_document()?
        else {
            panic!("frontier verdict Modeling must be a role pass")
        };
        assert!(verdict_document.dynamic_prompt_context.is_none());
        assert_eq!(
            verdict_document
                .frontier_verdict_modeling_context
                .as_ref()
                .expect("sealed typed verdict context")
                .soul_verdict,
            verdict
        );
        let verdict_basis = crate::worker_reasoning_basis(&store, &verdict_launch)?;
        crate::put_reasoning_basis(&store, &verdict_basis)?;
        let mut verdict_native = epiphany_model_adapter::EpiphanyModelRequest::new(
            "frontier-verdict-request",
            "frontier-verdict-conversation",
            "openai-codex",
            "gpt-test",
            "model",
        );
        verdict_native.reasoning_basis_id = Some(verdict_basis.basis_id.clone());
        verdict_native.source_worker_job_id = Some(verdict_job_id.clone());
        let verdict_provider = epiphany_openai_adapter::request_from_native(&verdict_native);
        let verdict_context = crate::EpiphanyDecisionContext::new(
            &verdict_basis,
            verdict_native,
            verdict_provider,
            Vec::new(),
        )?;
        verdict_cache.put(&verdict_context.context_id, &verdict_context)?;

        // A separate typed mutation may extend another semantic identity after
        // this pass seals its basis. That is concurrency, not staleness: the
        // verdict result owns only the routed frontier and its exact
        // dependencies.
        let mut unrelated_node = model
            .nodes
            .iter()
            .find(|node| node.id == "body-node")
            .cloned()
            .expect("seeded Body node");
        unrelated_node.id = "concurrent-unrelated-node".into();
        unrelated_node.title = "Concurrent unrelated node".into();
        unrelated_node.claim = "Disjoint keyed Mind writes merge".into();
        unrelated_node.updated_at = Some("2026-08-17T00:00:17.500Z".into());
        let unrelated_proposal = crate::EpiphanyRepoModelMutationProposal::new(
            "repo-model-mutation-proposal-concurrent-unrelated",
            "concurrent-unrelated-request",
            "concurrent-unrelated-result",
            vec![verdict.receipt_id.clone()],
            body.clone(),
            vec![crate::EpiphanyRepoModelMutationOperation::PutNode {
                node: unrelated_node,
            }],
        )?;
        let unrelated_plan = crate::plan_repo_model_mutation(&store, &unrelated_proposal)?;
        let verdict_provenance = verdict_cache
            .get_envelope::<crate::SoulVerdictReceipt>(&verdict.receipt_id)?
            .expect("typed Soul provenance");
        assert!(matches!(
            crate::commit_typed_organ_mind_mutation(
                &store,
                "Modeling",
                verdict_provenance,
                "Modeling.concurrent_unrelated_fixture",
                unrelated_plan.strong_reads,
                unrelated_plan.writes,
                "2026-08-17T00:00:17.500Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));

        let mut resolved_item = frontier_item.clone();
        resolved_item.status = RepoFrontierStatus::Resolved;
        resolved_item.gap = "The exact consequence passed Soul verification.".into();
        resolved_item.updated_at = Some("2026-08-17T00:00:18Z".into());
        resolved_item
            .evidence_refs
            .push(verification_request.request_id.clone());
        resolved_item.evidence_refs.push(verdict.receipt_id.clone());
        resolved_item.evidence_refs.sort();
        resolved_item.evidence_refs.dedup();
        let verdict_result_id = "frontier-verdict-result";
        let verdict_mutation = crate::EpiphanyRepoModelMutationProposal::new(
            format!("repo-model-mutation-proposal-{verdict_job_id}"),
            verdict_request.request_id.clone(),
            verdict_result_id,
            vec![verdict.receipt_id.clone()],
            body.clone(),
            vec![crate::EpiphanyRepoModelMutationOperation::PutFrontier {
                item: resolved_item,
            }],
        )?;
        let verdict_result = crate::EpiphanyRuntimeRoleWorkerResult {
            schema_version: crate::RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION.into(),
            result_id: verdict_result_id.into(),
            job_id: verdict_job_id.clone(),
            role_id: "modeling".into(),
            verdict: "checkpoint-update-needed".into(),
            summary: "Soul verdict incorporated".into(),
            next_safe_move: "admit".into(),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: Vec::new(),
            frontier_node_ids: vec!["body-node".into()],
            evidence_ids: vec![verdict.receipt_id.clone()],
            artifact_refs: Vec::new(),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            state_patch_msgpack: None,
            self_patch_msgpack: None,
            item_error: None,
            metadata: Default::default(),
            repo_model_mutation_proposal_msgpack: Some(rmp_serde::to_vec_named(&verdict_mutation)?),
            verification_request_id: None,
            frontier_route_id: None,
            repo_frontier_modeling_request_id: Some(verdict_request.request_id.clone()),
            proposal_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_candidate_msgpack: None,
            frontier_plan_mind_request_id: None,
            frontier_plan_mind_decision_msgpack: None,
            repository_body_observation_basis: Some(body.clone()),
            imagination_consideration_request_id: None,
            imagination_consideration_candidate_msgpack: None,
            admitted_model_direction_consideration_request_id: None,
            admitted_model_direction_consideration_result_msgpack: None,
            decision_context_id: verdict_context.context_id.clone(),
        };
        let verdict_process = crate::ProcessInstanceIdentity {
            process_id: 43,
            creation_token: 8,
            created_at_rfc3339: Some("2026-08-17T00:00:18Z".into()),
            executable_path: "verdict-worker".into(),
        };
        let verdict_activation = "verdict-activation";
        crate::claim_runtime_worker_process(
            &store,
            &verdict_job_id,
            &verdict_process,
            &format!("{:x}", Sha256::digest(verdict_activation.as_bytes())),
            "2026-08-17T00:00:18Z",
        )?;
        crate::activate_runtime_worker_process(
            &store,
            &verdict_job_id,
            &verdict_process,
            verdict_activation,
            "2026-08-17T00:00:19Z",
        )?;
        crate::put_runtime_role_worker_result(&store, &verdict_result)?;
        crate::complete_runtime_job(
            &store,
            crate::RuntimeSpineJobResultOptions {
                result_id: format!("runtime-result-{verdict_job_id}"),
                job_id: verdict_job_id.clone(),
                completed_at: "2026-08-17T00:00:20Z".into(),
                verdict: verdict_result.verdict.clone(),
                summary: verdict_result.summary.clone(),
                next_safe_move: verdict_result.next_safe_move.clone(),
                evidence_refs: verdict_result.evidence_ids.clone(),
                artifact_refs: Vec::new(),
                decision_context_id: Some(verdict_context.context_id.clone()),
            },
        )?;
        assert_eq!(
            project_current_work(&store)?
                .frontier_verdict_modeling
                .expect("terminal verdict work awaits admission")
                .action,
            EpiphanyAgentPassContinuationAction::Review
        );

        // The same fixture, copied before admission, proves the other half of
        // the merge law. If another valid mutation changes this exact frontier,
        // the sealed verdict output is stale and must not be silently rebased.
        let conflict_store = temp.path().join("frontier-conflict.cc");
        std::fs::copy(&store, &conflict_store)?;
        let mut competing_item = frontier_item.clone();
        competing_item.gap = "A competing decision changed this exact frontier.".into();
        competing_item.updated_at = Some("2026-08-17T00:00:20.500Z".into());
        let competing_proposal = crate::EpiphanyRepoModelMutationProposal::new(
            "repo-model-mutation-proposal-competing-frontier",
            "competing-frontier-request",
            "competing-frontier-result",
            vec![verdict.receipt_id.clone()],
            body.clone(),
            vec![crate::EpiphanyRepoModelMutationOperation::PutFrontier {
                item: competing_item,
            }],
        )?;
        let competing_plan = crate::plan_repo_model_mutation(&conflict_store, &competing_proposal)?;
        let mut conflict_cache = crate::runtime_spine_cache(&conflict_store)?;
        conflict_cache.pull_all_backing_stores()?;
        let conflict_provenance = conflict_cache
            .get_envelope::<crate::SoulVerdictReceipt>(&verdict.receipt_id)?
            .expect("copied Soul provenance");
        assert!(matches!(
            crate::commit_typed_organ_mind_mutation(
                &conflict_store,
                "Soul",
                conflict_provenance,
                "Soul.competing_frontier_fixture",
                competing_plan.strong_reads,
                competing_plan.writes,
                "2026-08-17T00:00:20.500Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));
        let mut conflict_before = crate::runtime_spine_cache(&conflict_store)?;
        conflict_before.pull_all_backing_stores()?;
        let conflict_before = conflict_before.snapshot_envelopes();
        assert!(
            accept_frontier_verdict_modeling_result(
                &conflict_store,
                &verdict_job_id,
                "2026-08-17T00:00:21Z",
            )
            .is_err()
        );
        let mut conflict_after = crate::runtime_spine_cache(&conflict_store)?;
        conflict_after.pull_all_backing_stores()?;
        assert_eq!(conflict_after.snapshot_envelopes(), conflict_before);

        let verdict_commit = accept_frontier_verdict_modeling_result(
            &store,
            &verdict_job_id,
            "2026-08-17T00:00:21Z",
        )?;
        assert_eq!(verdict_commit.invariant_owner, "Modeling.frontier_verdict");
        assert!(
            project_current_work(&store)?
                .frontier_verdict_modeling
                .is_none()
        );
        let final_model = crate::assemble_repo_model_view(&store)?;
        assert_eq!(
            final_model
                .frontier
                .iter()
                .find(|item| item.id == frontier_item.id)
                .expect("frontier persists")
                .status,
            RepoFrontierStatus::Resolved
        );
        assert!(
            final_model
                .nodes
                .iter()
                .any(|node| node.id == "concurrent-unrelated-node")
        );

        // Eyes is its own exact current-work family. It launches from an
        // explicit external-evidence obligation, survives unrelated keyed
        // Mind commits, and conflicts if that exact frontier changes.
        let eyes_frontier = RepoFrontierItem {
            id: "eyes-frontier".into(),
            migration_body: "epiphany".into(),
            question: "What external evidence resolves this claim?".into(),
            gap: "The current Body cannot answer this external question.".into(),
            target_claim_ids: vec!["body-node".into()],
            source_scope: vec!["epiphany-core/src".into()],
            recommended_next_organ: "Eyes".into(),
            status: RepoFrontierStatus::Active,
            evidence_refs: vec![verdict.receipt_id.clone()],
            ..Default::default()
        };
        let eyes_proposal = crate::EpiphanyRepoModelMutationProposal::new(
            "repo-model-mutation-proposal-eyes-frontier",
            "eyes-frontier-fixture-request",
            "eyes-frontier-fixture-result",
            vec![verdict.receipt_id.clone()],
            body.clone(),
            vec![crate::EpiphanyRepoModelMutationOperation::PutFrontier {
                item: eyes_frontier.clone(),
            }],
        )?;
        let eyes_plan = crate::plan_repo_model_mutation(&store, &eyes_proposal)?;
        let mut eyes_cache = crate::runtime_spine_cache(&store)?;
        eyes_cache.pull_all_backing_stores()?;
        let eyes_provenance = eyes_cache
            .get_envelope::<crate::SoulVerdictReceipt>(&verdict.receipt_id)?
            .expect("typed provenance for Eyes obligation");
        assert!(matches!(
            crate::commit_typed_organ_mind_mutation(
                &store,
                "Modeling",
                eyes_provenance,
                "Modeling.external_evidence_obligation_fixture",
                eyes_plan.strong_reads,
                eyes_plan.writes,
                "2026-08-17T00:00:22Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));
        assert_eq!(
            project_current_work(&store)?.research_continuation_action,
            Some(RepoFrontierResearchContinuationAction::LaunchResearch)
        );
        let research_job = launch_current_frontier_research_work(&store, "2026-08-17T00:00:23Z")?;
        let mut research_cache = crate::runtime_spine_cache(&store)?;
        research_cache.pull_all_backing_stores()?;
        assert!(
            research_cache
                .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
                .is_none()
        );
        let research_launch = research_cache
            .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&research_job)?
            .expect("exact Research launch");
        let research_request_id = research_launch
            .repo_frontier_research_request_id
            .clone()
            .expect("exact Research request identity");
        let research_request = research_cache
            .get::<crate::RepoFrontierResearchRequest>(&research_request_id)?
            .expect("exact Research request");
        assert_eq!(research_request.frontier_item_id, eyes_frontier.id);
        assert!(
            research_request.frontier_authority_documents.len()
                < research_request.model_source_documents.len()
        );
        assert_eq!(
            project_current_work(&store)?.research_continuation_action,
            None
        );
        let research_basis = crate::worker_reasoning_basis(&store, &research_launch)?;
        crate::put_reasoning_basis(&store, &research_basis)?;
        let mut research_native = epiphany_model_adapter::EpiphanyModelRequest::new(
            "research-request",
            "research-conversation",
            "openai-codex",
            "gpt-test",
            "model",
        );
        research_native.reasoning_basis_id = Some(research_basis.basis_id.clone());
        research_native.source_worker_job_id = Some(research_job.clone());
        let research_provider = epiphany_openai_adapter::request_from_native(&research_native);
        let research_context = crate::EpiphanyDecisionContext::new(
            &research_basis,
            research_native,
            research_provider,
            Vec::new(),
        )?;
        research_cache.put(&research_context.context_id, &research_context)?;
        let stale_research_store = temp.path().join("stale-research-output.cc");
        std::fs::copy(&store, &stale_research_store)?;
        let evidence = epiphany_state_model::EpiphanyEvidenceRecord {
            id: "research-evidence-1".into(),
            kind: "external-source".into(),
            status: "ok".into(),
            summary: "The bounded external claim is supported.".into(),
            code_refs: vec![epiphany_state_model::EpiphanyCodeRef {
                path: "epiphany-core/src/current_work.rs".into(),
                start_line: Some(1),
                end_line: None,
                symbol: Some("accept_frontier_research_result".into()),
                note: None,
            }],
        };
        let observation = epiphany_state_model::EpiphanyObservation {
            id: "research-observation-1".into(),
            summary: "External evidence answers the exact frontier question.".into(),
            source_kind: "research".into(),
            status: "ok".into(),
            code_refs: Vec::new(),
            evidence_ids: vec![evidence.id.clone()],
        };
        let research_patch = crate::EpiphanyRoleStatePatchDocument {
            evidence: vec![evidence.clone()],
            observations: vec![observation.clone()],
            ..Default::default()
        };
        let research_result = crate::EpiphanyRuntimeRoleWorkerResult {
            schema_version: crate::RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION.into(),
            result_id: "research-result".into(),
            job_id: research_job.clone(),
            role_id: "research".into(),
            verdict: "evidence-ready".into(),
            summary: "gathered exact external evidence".into(),
            next_safe_move: "admit evidence".into(),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: vec!["epiphany-core/src/current_work.rs".into()],
            frontier_node_ids: vec!["body-node".into()],
            evidence_ids: vec![evidence.id.clone()],
            artifact_refs: Vec::new(),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            state_patch_msgpack: Some(rmp_serde::to_vec_named(&research_patch)?),
            self_patch_msgpack: None,
            item_error: None,
            metadata: Default::default(),
            repo_model_mutation_proposal_msgpack: None,
            verification_request_id: None,
            frontier_route_id: None,
            repo_frontier_modeling_request_id: None,
            proposal_modeling_request_id: None,
            repo_frontier_research_request_id: Some(research_request_id.clone()),
            frontier_planning_request_id: None,
            frontier_plan_candidate_msgpack: None,
            frontier_plan_mind_request_id: None,
            frontier_plan_mind_decision_msgpack: None,
            repository_body_observation_basis: None,
            imagination_consideration_request_id: None,
            imagination_consideration_candidate_msgpack: None,
            admitted_model_direction_consideration_request_id: None,
            admitted_model_direction_consideration_result_msgpack: None,
            decision_context_id: research_context.context_id.clone(),
        };
        let research_process = crate::ProcessInstanceIdentity {
            process_id: 44,
            creation_token: 9,
            created_at_rfc3339: Some("2026-08-17T00:00:24Z".into()),
            executable_path: "research-worker".into(),
        };
        let research_activation = "research-activation";
        crate::claim_runtime_worker_process(
            &store,
            &research_job,
            &research_process,
            &format!("{:x}", Sha256::digest(research_activation.as_bytes())),
            "2026-08-17T00:00:24Z",
        )?;
        crate::activate_runtime_worker_process(
            &store,
            &research_job,
            &research_process,
            research_activation,
            "2026-08-17T00:00:25Z",
        )?;
        crate::put_runtime_role_worker_result(&store, &research_result)?;
        crate::complete_runtime_job(
            &store,
            crate::RuntimeSpineJobResultOptions {
                result_id: format!("runtime-result-{research_job}"),
                job_id: research_job.clone(),
                completed_at: "2026-08-17T00:00:26Z".into(),
                verdict: research_result.verdict.clone(),
                summary: research_result.summary.clone(),
                next_safe_move: research_result.next_safe_move.clone(),
                evidence_refs: research_result.evidence_ids.clone(),
                artifact_refs: Vec::new(),
                decision_context_id: Some(research_context.context_id.clone()),
            },
        )?;
        assert_eq!(
            project_current_work(&store)?.research_continuation_action,
            Some(RepoFrontierResearchContinuationAction::ReviewResearchResult)
        );

        let mut unrelated_research_node = final_model
            .nodes
            .iter()
            .find(|node| node.id == "body-node")
            .cloned()
            .expect("seeded Body node");
        unrelated_research_node.id = "research-concurrent-node".into();
        unrelated_research_node.title = "Research concurrent node".into();
        unrelated_research_node.claim = "Unrelated Mind work does not stale Eyes".into();
        let disjoint_research_proposal = crate::EpiphanyRepoModelMutationProposal::new(
            "repo-model-mutation-proposal-research-concurrent",
            "research-concurrent-request",
            "research-concurrent-result",
            vec![verdict.receipt_id.clone()],
            body.clone(),
            vec![crate::EpiphanyRepoModelMutationOperation::PutNode {
                node: unrelated_research_node,
            }],
        )?;
        let disjoint_research_plan =
            crate::plan_repo_model_mutation(&store, &disjoint_research_proposal)?;
        let mut disjoint_cache = crate::runtime_spine_cache(&store)?;
        disjoint_cache.pull_all_backing_stores()?;
        let disjoint_provenance = disjoint_cache
            .get_envelope::<crate::SoulVerdictReceipt>(&verdict.receipt_id)?
            .expect("typed provenance for disjoint Research mutation");
        assert!(matches!(
            crate::commit_typed_organ_mind_mutation(
                &store,
                "Modeling",
                disjoint_provenance,
                "Modeling.concurrent_with_research_fixture",
                disjoint_research_plan.strong_reads,
                disjoint_research_plan.writes,
                "2026-08-17T00:00:26.500Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));

        let mut competing_eyes_frontier = eyes_frontier.clone();
        competing_eyes_frontier.gap = "The exact external question changed.".into();
        competing_eyes_frontier.updated_at = Some("2026-08-17T00:00:27Z".into());
        let competing_eyes_proposal = crate::EpiphanyRepoModelMutationProposal::new(
            "repo-model-mutation-proposal-competing-eyes-frontier",
            "competing-eyes-frontier-request",
            "competing-eyes-frontier-result",
            vec![verdict.receipt_id.clone()],
            body.clone(),
            vec![crate::EpiphanyRepoModelMutationOperation::PutFrontier {
                item: competing_eyes_frontier,
            }],
        )?;
        let competing_eyes_plan =
            crate::plan_repo_model_mutation(&stale_research_store, &competing_eyes_proposal)?;
        let mut stale_research_cache = crate::runtime_spine_cache(&stale_research_store)?;
        stale_research_cache.pull_all_backing_stores()?;
        let competing_eyes_provenance = stale_research_cache
            .get_envelope::<crate::SoulVerdictReceipt>(&verdict.receipt_id)?
            .expect("copied typed provenance");
        assert!(matches!(
            crate::commit_typed_organ_mind_mutation(
                &stale_research_store,
                "Modeling",
                competing_eyes_provenance,
                "Modeling.competing_eyes_frontier_fixture",
                competing_eyes_plan.strong_reads,
                competing_eyes_plan.writes,
                "2026-08-17T00:00:27Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));
        let stale_process = crate::ProcessInstanceIdentity {
            process_id: 45,
            creation_token: 10,
            created_at_rfc3339: Some("2026-08-17T00:00:27Z".into()),
            executable_path: "stale-research-worker".into(),
        };
        let stale_activation = "stale-research-activation";
        crate::claim_runtime_worker_process(
            &stale_research_store,
            &research_job,
            &stale_process,
            &format!("{:x}", Sha256::digest(stale_activation.as_bytes())),
            "2026-08-17T00:00:27Z",
        )?;
        crate::activate_runtime_worker_process(
            &stale_research_store,
            &research_job,
            &stale_process,
            stale_activation,
            "2026-08-17T00:00:27.100Z",
        )?;
        crate::put_runtime_role_worker_result(&stale_research_store, &research_result)?;
        crate::complete_runtime_job(
            &stale_research_store,
            crate::RuntimeSpineJobResultOptions {
                result_id: format!("runtime-result-{research_job}"),
                job_id: research_job.clone(),
                completed_at: "2026-08-17T00:00:27.200Z".into(),
                verdict: research_result.verdict.clone(),
                summary: research_result.summary.clone(),
                next_safe_move: research_result.next_safe_move.clone(),
                evidence_refs: research_result.evidence_ids.clone(),
                artifact_refs: Vec::new(),
                decision_context_id: Some(research_context.context_id.clone()),
            },
        )?;
        assert_eq!(
            crate::runtime_role_worker_result(&stale_research_store, &research_job)?
                .expect("stale output remains a durable decision")
                .decision_context_id,
            research_context.context_id
        );
        let mut research_conflict_before = crate::runtime_spine_cache(&stale_research_store)?;
        research_conflict_before.pull_all_backing_stores()?;
        let research_conflict_before = research_conflict_before.snapshot_envelopes();
        assert!(
            accept_frontier_research_result(
                &stale_research_store,
                &research_job,
                "2026-08-17T00:00:28Z",
            )
            .is_err()
        );
        let mut research_conflict_after = crate::runtime_spine_cache(&stale_research_store)?;
        research_conflict_after.pull_all_backing_stores()?;
        assert_eq!(
            research_conflict_after.snapshot_envelopes(),
            research_conflict_before
        );

        let research_commit =
            accept_frontier_research_result(&store, &research_job, "2026-08-17T00:00:28Z")?;
        assert_eq!(research_commit.invariant_owner, "Eyes.frontier_research");
        assert_eq!(
            accept_frontier_research_result(&store, &research_job, "2026-08-17T00:00:29Z")?,
            research_commit
        );
        let mut admitted_research_cache = crate::runtime_spine_cache(&store)?;
        admitted_research_cache.pull_all_backing_stores()?;
        assert!(
            admitted_research_cache
                .get::<crate::EpiphanyMindEvidenceDocument>(&evidence.id)?
                .is_some()
        );
        assert!(
            admitted_research_cache
                .get::<crate::EpiphanyMindObservationDocument>(&observation.id)?
                .is_some()
        );
        let packet = admitted_research_cache
            .get::<crate::EyesEvidencePacket>(&format!("eyes-packet-{research_request_id}"))?
            .expect("request-bound Eyes packet");
        assert_eq!(packet.research_request_id, research_request_id);
        assert_eq!(packet.decision_context_id, research_context.context_id);
        let after_research = project_current_work(&store)?;
        assert!(after_research.research_continuation_action.is_none());
        assert!(after_research.body_modeling_action.is_none());
        assert!(after_research.proposal_modeling.is_none());
        assert!(after_research.frontier_verdict_modeling.is_none());
        Ok(())
    }
}
