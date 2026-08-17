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
    pub body_modeling_action: Option<EpiphanyModelingContinuationAction>,
    pub research_continuation_action: Option<RepoFrontierResearchContinuationAction>,
    pub frontier_planning_stage: RepoFrontierPlanningLifecycleStage,
    pub proposal_modeling: Option<EpiphanyProposalModelingWorkProjection>,
    pub hands_frontier_ready: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyModelingContinuationAction {
    Launch,
    Wait,
    Review,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyProposalModelingWorkProjection {
    pub request: RepoFrontierProposalModelingRequest,
    pub action: EpiphanyModelingContinuationAction,
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
                        (action == EpiphanyModelingContinuationAction::Launch).then_some(work),
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
        hands_frontier_ready: crate::runtime_has_actionable_hands_frontier(store_path)?,
    })
}

fn body_modeling_continuation_action(
    cache: &CultCache,
    work_id: &str,
) -> Result<EpiphanyModelingContinuationAction> {
    let mut bindings = cache
        .get_all::<EpiphanyBodyModelingLaunchBinding>()?
        .into_iter()
        .filter(|binding| binding.work_id == work_id)
        .collect::<Vec<_>>();
    bindings.sort_by_key(|binding| binding.attempt_ordinal);
    let Some(binding) = bindings.last() else {
        return Ok(EpiphanyModelingContinuationAction::Launch);
    };
    let job = cache
        .get::<crate::EpiphanyRuntimeJob>(&binding.job_id)?
        .ok_or_else(|| anyhow!("Body Modeling launch binding lost its runtime job"))?;
    Ok(match job.status {
        crate::EpiphanyRuntimeJobStatus::Failed | crate::EpiphanyRuntimeJobStatus::Cancelled => {
            EpiphanyModelingContinuationAction::Launch
        }
        crate::EpiphanyRuntimeJobStatus::Completed => {
            if cache
                .get::<crate::EpiphanyRuntimeRoleWorkerResult>(&binding.job_id)?
                .is_some()
            {
                EpiphanyModelingContinuationAction::Review
            } else {
                EpiphanyModelingContinuationAction::Wait
            }
        }
        _ => EpiphanyModelingContinuationAction::Wait,
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
                action: EpiphanyModelingContinuationAction::Launch,
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
                EpiphanyModelingContinuationAction::Launch
            }
            crate::EpiphanyRuntimeJobStatus::Completed if result.is_some() => {
                crate::runtime_spine::validate_proposal_modeling_worker_fulfillment(
                    cache,
                    result.as_ref().expect("checked terminal result"),
                )?;
                EpiphanyModelingContinuationAction::Review
            }
            _ => EpiphanyModelingContinuationAction::Wait,
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
) -> Result<Option<EpiphanyModelingContinuationAction>> {
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
        != Some(EpiphanyModelingContinuationAction::Review)
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
) -> Result<Option<EpiphanyModelingContinuationAction>> {
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
        .filter(|work| work.action == EpiphanyModelingContinuationAction::Review)
        .and_then(|work| work.job_id))
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
        || result.claim_repair_request_id.is_some()
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
        || result.claim_repair_request_id.is_some()
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
    let current_work = project_current_work(store_path)?;
    let work = current_work
        .body_modeling
        .ok_or_else(|| anyhow!("Mind has no unresolved Body Modeling work"))?;
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mind = crate::assemble_mind_view(store_path)?;
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
            claim_repair_context: None,
            frontier_planning_context: None,
            frontier_research_context: None,
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
            claim_repair_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            repo_frontier_verdict_modeling_authority: None,
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
    let work = project_current_work(store_path)?
        .proposal_modeling
        .filter(|work| work.action == EpiphanyModelingContinuationAction::Launch)
        .ok_or_else(|| anyhow!("Mind has no launchable proposal Modeling work"))?;
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
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
            claim_repair_context: None,
            frontier_planning_context: None,
            frontier_research_context: None,
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
            claim_repair_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            repo_frontier_verdict_modeling_authority: None,
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
    fn keyed_modeling_lifecycles_ignore_thread_state() -> Result<()> {
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
            Some(EpiphanyModelingContinuationAction::Launch)
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
            Some(EpiphanyModelingContinuationAction::Wait)
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
            claim_repair_request_id: None,
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
            Some(EpiphanyModelingContinuationAction::Review)
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
            EpiphanyModelingContinuationAction::Launch
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
            EpiphanyModelingContinuationAction::Wait
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
            claim_repair_request_id: None,
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
            EpiphanyModelingContinuationAction::Review
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
        Ok(())
    }
}
