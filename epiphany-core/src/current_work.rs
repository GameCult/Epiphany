use anyhow::{Result, anyhow};
use cultcache_rs::{CultCache, CultCacheEnvelope, DatabaseEntry};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::{
    EpiphanyMindDocumentVersion, EpiphanyRepoModelBasis, RepoFrontierPlanningLifecycleStage,
    RepoFrontierProposalModelingRequest, RepositoryBodyObservationBasis,
};

#[cfg(test)]
use crate::RepoFrontierResearchContinuationAction;

pub const BODY_MODELING_DECISION_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.mind.body_modeling_decision.v1";
pub const AGENT_PASS_ADMISSION_REFUSAL_SCHEMA_VERSION: &str =
    "epiphany.mind.agent_pass_admission_refusal.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EpiphanyAgentPassFamily {
    BodyModeling,
    ProposalModeling,
    FrontierVerdictModeling,
    FrontierVerification,
}

impl EpiphanyAgentPassFamily {
    pub fn request_kind(self) -> &'static str {
        match self {
            Self::BodyModeling => "body-modeling",
            Self::ProposalModeling => "proposal-modeling",
            Self::FrontierVerdictModeling => "frontier-verdict-modeling",
            Self::FrontierVerification => "frontier-verification",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EpiphanyAgentPassAdmissionRefusalKind {
    RepoModelMutationRefused,
    StrongReadConflict,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyCurrentWorkProjection {
    pub mind_projection_digest: String,
    pub body_modeling: Option<EpiphanyBodyModelingCurrentWorkProjection>,
    pub research: crate::RepoFrontierResearchLifecycle,
    pub frontier_planning: crate::RepoFrontierPlanningLifecycle,
    pub proposal_modeling: Option<EpiphanyProposalModelingWorkProjection>,
    pub frontier_verdict_modeling: Option<EpiphanyFrontierVerdictModelingWorkProjection>,
    pub verification: Option<EpiphanyVerificationWorkProjection>,
    pub reorientation: Option<crate::EpiphanyReorientationWorkProjection>,
    pub imagination_considerations: Vec<EpiphanyImaginationConsiderationWorkProjection>,
    pub admitted_model_direction_consideration:
        Option<EpiphanyAdmittedModelDirectionConsiderationWorkProjection>,
    pub hands_frontier_ready: bool,
}

impl EpiphanyCurrentWorkProjection {
    pub fn projection_digest(&self) -> Result<String> {
        Ok(format!(
            "sha256:{:x}",
            Sha256::digest(rmp_serde::to_vec_named(self)?)
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyAgentPassContinuationAction {
    Launch,
    Wait,
    Review,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyAgentPassAttemptProjection {
    pub action: EpiphanyAgentPassContinuationAction,
    pub job_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.agent_pass_admission_refusal.v1",
    schema = "EpiphanyAgentPassAdmissionRefusal"
)]
pub struct EpiphanyAgentPassAdmissionRefusal {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub refusal_id: String,
    #[cultcache(key = 2)]
    pub pass_family: EpiphanyAgentPassFamily,
    #[cultcache(key = 3)]
    pub request_id: String,
    #[cultcache(key = 4)]
    pub job_id: String,
    #[cultcache(key = 5)]
    pub result_id: String,
    #[cultcache(key = 6)]
    pub decision_context_id: String,
    #[cultcache(key = 7)]
    pub invariant_owner: String,
    #[cultcache(key = 8)]
    pub refusal_kind: EpiphanyAgentPassAdmissionRefusalKind,
    #[cultcache(key = 9)]
    pub reason: String,
    #[cultcache(key = 10)]
    pub refused_at: String,
}

impl EpiphanyAgentPassAdmissionRefusal {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != AGENT_PASS_ADMISSION_REFUSAL_SCHEMA_VERSION
            || self.refusal_id != format!("agent-pass-admission-refusal-{}", self.job_id)
            || [
                self.request_id.as_str(),
                self.job_id.as_str(),
                self.result_id.as_str(),
                self.decision_context_id.as_str(),
                self.invariant_owner.as_str(),
                self.reason.as_str(),
            ]
            .into_iter()
            .any(str::is_empty)
            || chrono::DateTime::parse_from_rfc3339(&self.refused_at).is_err()
        {
            return Err(anyhow!("agent-pass admission refusal is invalid"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyAgentPassAdmissionOutcome {
    Committed(crate::EpiphanyMindCommitReceipt),
    Refused {
        refusal: EpiphanyAgentPassAdmissionRefusal,
        commit: crate::EpiphanyMindCommitReceipt,
    },
}

impl EpiphanyAgentPassAttemptProjection {
    pub(crate) fn unattempted() -> Self {
        Self::with(EpiphanyAgentPassContinuationAction::Launch, None)
    }

    pub(crate) fn with(
        action: EpiphanyAgentPassContinuationAction,
        job_id: Option<String>,
    ) -> Self {
        Self { action, job_id }
    }

    fn from_job(job: &crate::EpiphanyRuntimeJob, completed_is_reviewable: bool) -> Self {
        let action = match job.status {
            crate::EpiphanyRuntimeJobStatus::Failed => EpiphanyAgentPassContinuationAction::Launch,
            crate::EpiphanyRuntimeJobStatus::Completed if completed_is_reviewable => {
                EpiphanyAgentPassContinuationAction::Review
            }
            _ => EpiphanyAgentPassContinuationAction::Wait,
        };
        Self::with(action, Some(job.job_id.clone()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyProposalModelingWorkProjection {
    pub request: RepoFrontierProposalModelingRequest,
    #[serde(flatten)]
    pub attempt: EpiphanyAgentPassAttemptProjection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyFrontierVerdictModelingWorkProjection {
    pub request: crate::RepoFrontierModelingRequest,
    #[serde(flatten)]
    pub attempt: EpiphanyAgentPassAttemptProjection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyVerificationWorkProjection {
    pub request: crate::RepoFrontierVerificationRequest,
    #[serde(flatten)]
    pub attempt: EpiphanyAgentPassAttemptProjection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyImaginationConsiderationWorkProjection {
    pub request: crate::ImaginationConsiderationRequest,
    #[serde(flatten)]
    pub attempt: EpiphanyAgentPassAttemptProjection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyAdmittedModelDirectionConsiderationWorkProjection {
    pub request: crate::AdmittedModelDirectionConsiderationRequest,
    #[serde(flatten)]
    pub attempt: EpiphanyAgentPassAttemptProjection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyBodyModelingWorkProjection {
    pub work_id: String,
    pub runtime_id: String,
    pub body_basis: RepositoryBodyObservationBasis,
    pub repo_model_basis: EpiphanyRepoModelBasis,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyBodyModelingCurrentWorkProjection {
    pub work: EpiphanyBodyModelingWorkProjection,
    #[serde(flatten)]
    pub attempt: EpiphanyAgentPassAttemptProjection,
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
    cache.register_entry_type::<EpiphanyAgentPassAdmissionRefusal>()?;
    Ok(())
}

fn role_launch_document(
    launch: &crate::EpiphanyRuntimeWorkerLaunchRequest,
) -> Result<crate::EpiphanyRoleWorkerLaunchDocument> {
    match launch.launch_document()? {
        crate::EpiphanyWorkerLaunchDocument::Role(document) => Ok(document),
        crate::EpiphanyWorkerLaunchDocument::Reorient(_) => {
            Err(anyhow!("typed role work carried reorientation authority"))
        }
    }
}

fn launch_matches_typed_request(
    launch: &crate::EpiphanyRuntimeWorkerLaunchRequest,
    request: crate::RuntimeTypedRequestRef<'_>,
) -> Result<bool> {
    request.matches_launch(launch)
}

fn typed_request_launches<'a>(
    launches: &'a [crate::EpiphanyRuntimeWorkerLaunchRequest],
    request: crate::RuntimeTypedRequestRef<'_>,
) -> Result<Vec<&'a crate::EpiphanyRuntimeWorkerLaunchRequest>> {
    launches
        .iter()
        .map(|launch| Ok(launch_matches_typed_request(launch, request)?.then_some(launch)))
        .filter_map(Result::transpose)
        .collect()
}

fn launch_matches_frontier_planning_request(
    launch: &crate::EpiphanyRuntimeWorkerLaunchRequest,
    request_id: &str,
) -> Result<bool> {
    Ok(role_launch_document(launch)?
        .frontier_planning_context
        .is_some_and(|context| context.request_id == request_id))
}

fn launch_matches_frontier_plan_mind_request(
    launch: &crate::EpiphanyRuntimeWorkerLaunchRequest,
    request_id: &str,
) -> Result<bool> {
    Ok(role_launch_document(launch)?
        .frontier_plan_mind_context
        .is_some_and(|context| context.request.request_id == request_id))
}

fn frontier_planning_launches<'a>(
    launches: &'a [crate::EpiphanyRuntimeWorkerLaunchRequest],
    request_id: &str,
) -> Result<Vec<&'a crate::EpiphanyRuntimeWorkerLaunchRequest>> {
    launches
        .iter()
        .map(|launch| {
            Ok(launch_matches_frontier_planning_request(launch, request_id)?.then_some(launch))
        })
        .filter_map(Result::transpose)
        .collect()
}

fn frontier_plan_mind_launches<'a>(
    launches: &'a [crate::EpiphanyRuntimeWorkerLaunchRequest],
    request_id: &str,
) -> Result<Vec<&'a crate::EpiphanyRuntimeWorkerLaunchRequest>> {
    launches
        .iter()
        .map(|launch| {
            Ok(launch_matches_frontier_plan_mind_request(launch, request_id)?.then_some(launch))
        })
        .filter_map(Result::transpose)
        .collect()
}

pub(crate) fn body_modeling_decision_envelope(
    store_path: &Path,
    result: &crate::EpiphanyRuntimeRoleWorkerResult,
    disposition: &str,
    decided_at: &str,
) -> Result<CultCacheEnvelope> {
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let launch = cache
        .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&result.job_id)?
        .ok_or_else(|| anyhow!("Body Modeling result lost its immutable launch"))?;
    let body_basis = launch
        .repository_body_observation_basis()?
        .ok_or_else(|| anyhow!("Body Modeling result has no Body observation basis"))?;
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

pub fn project_current_work(store_path: impl AsRef<Path>) -> Result<EpiphanyCurrentWorkProjection> {
    let store_path = store_path.as_ref();
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mind = crate::mind_documents::assemble_mind_view_from_cache(&cache)?;
    let body_modeling = match (
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
                Some(work) => Some(EpiphanyBodyModelingCurrentWorkProjection {
                    attempt: body_modeling_attempt(&cache, &work)?,
                    work,
                }),
                None => None,
            }
        }
        (None, None) | (Some(_), None) => None,
        (None, Some(_)) => {
            return Err(anyhow!(
                "current work has a RepoModel but no admitted repository Body observation"
            ));
        }
    };
    let (imagination_considerations, admitted_model_direction_consideration) =
        if mind.repo_model.is_some() {
            (
                current_imagination_consideration_work(&cache)?,
                current_admitted_model_direction_consideration_work(&cache)?,
            )
        } else {
            (Vec::new(), None)
        };
    Ok(EpiphanyCurrentWorkProjection {
        mind_projection_digest: mind.projection_digest,
        body_modeling,
        research: crate::runtime_spine::repo_frontier_research_lifecycle(&cache)?,
        frontier_planning: crate::runtime_spine::repo_frontier_planning_lifecycle(&cache)?,
        proposal_modeling: current_proposal_modeling_work(&cache)?,
        frontier_verdict_modeling: current_frontier_verdict_modeling_work(&cache)?,
        verification: current_verification_work(&cache)?,
        reorientation: crate::reorientation_work::current_reorientation_work(&cache)?,
        imagination_considerations,
        admitted_model_direction_consideration,
        hands_frontier_ready: crate::runtime_spine::has_actionable_hands_frontier(&cache)?,
    })
}

fn consideration_attempt(
    cache: &CultCache,
    job_ids: &[String],
) -> Result<EpiphanyAgentPassAttemptProjection> {
    let Some(job_id) = job_ids.last() else {
        return Ok(EpiphanyAgentPassAttemptProjection::unattempted());
    };
    let job = cache
        .get::<crate::EpiphanyRuntimeJob>(job_id)?
        .ok_or_else(|| anyhow!("consideration launch lost its runtime job"))?;
    Ok(EpiphanyAgentPassAttemptProjection::from_job(&job, false))
}

pub(crate) fn consideration_attempt_ordinal(request_id: &str, job_id: &str) -> Result<usize> {
    let prefix = format!("consideration-{request_id}-attempt-");
    job_id
        .strip_prefix(&prefix)
        .ok_or_else(|| anyhow!("consideration job identity is not canonical"))?
        .parse::<usize>()
        .map_err(|_| anyhow!("consideration attempt ordinal is invalid"))
}

fn current_imagination_consideration_work(
    cache: &CultCache,
) -> Result<Vec<EpiphanyImaginationConsiderationWorkProjection>> {
    let model_basis =
        crate::repo_model_documents::assemble_repo_model_view_from_cache(cache)?.reasoning_basis();
    let candidates = cache.get_all::<crate::ImaginationConsiderationCandidate>()?;
    let launches = cache.get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?;
    let mut requests = cache.get_all::<crate::ImaginationConsiderationRequest>()?;
    requests.sort_by(|left, right| left.request_id.cmp(&right.request_id));
    let mut work = Vec::new();
    for request in requests {
        if request.model_projection_digest != model_basis.projection_digest
            || request.model_source_documents != model_basis.source_documents
            || candidates
                .iter()
                .any(|candidate| candidate.request_id == request.request_id)
        {
            continue;
        }
        crate::validate_current_imagination_consideration_request(cache, &request)?;
        let mut job_ids = typed_request_launches(
            &launches,
            crate::RuntimeTypedRequestRef::ImaginationConsideration(&request.request_id),
        )?
        .into_iter()
        .map(|launch| {
            Ok((
                consideration_attempt_ordinal(&request.request_id, &launch.job_id)?,
                launch.job_id.clone(),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
        job_ids.sort_by_key(|(ordinal, _)| *ordinal);
        for (expected, (ordinal, _)) in job_ids.iter().enumerate() {
            if *ordinal != expected {
                return Err(anyhow!(
                    "Persona feedback consideration has noncontiguous attempt identity"
                ));
            }
        }
        let job_ids = job_ids
            .into_iter()
            .map(|(_, job_id)| job_id)
            .collect::<Vec<_>>();
        let attempt = consideration_attempt(cache, &job_ids)?;
        work.push(EpiphanyImaginationConsiderationWorkProjection { request, attempt });
    }
    Ok(work)
}

fn current_admitted_model_direction_consideration_work(
    cache: &CultCache,
) -> Result<Option<EpiphanyAdmittedModelDirectionConsiderationWorkProjection>> {
    let model_basis =
        crate::repo_model_documents::assemble_repo_model_view_from_cache(cache)?.reasoning_basis();
    let results = cache.get_all::<crate::AdmittedModelDirectionConsiderationResult>()?;
    let launches = cache.get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?;
    let mut requests = cache.get_all::<crate::AdmittedModelDirectionConsiderationRequest>()?;
    requests.sort_by(|left, right| left.request_id.cmp(&right.request_id));
    for request in requests.into_iter().rev() {
        if request.model_projection_digest != model_basis.projection_digest
            || request.model_source_documents != model_basis.source_documents
            || results
                .iter()
                .any(|result| result.request_id == request.request_id && result.terminal)
        {
            continue;
        }
        crate::validate_current_admitted_model_direction_consideration_request(cache, &request)?;
        let mut job_ids = typed_request_launches(
            &launches,
            crate::RuntimeTypedRequestRef::AdmittedModelDirection(&request.request_id),
        )?
        .into_iter()
        .map(|launch| {
            Ok((
                consideration_attempt_ordinal(&request.request_id, &launch.job_id)?,
                launch.job_id.clone(),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
        job_ids.sort_by_key(|(ordinal, _)| *ordinal);
        let job_ids = job_ids
            .into_iter()
            .map(|(_, job_id)| job_id)
            .collect::<Vec<_>>();
        let attempt = consideration_attempt(cache, &job_ids)?;
        return Ok(Some(
            EpiphanyAdmittedModelDirectionConsiderationWorkProjection { request, attempt },
        ));
    }
    Ok(None)
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
    let admission_refusals = cache.get_all::<EpiphanyAgentPassAdmissionRefusal>()?;
    for request in requests {
        crate::runtime_spine::validate_repo_frontier_verification_request_intrinsic(&request)?;
        let mut request_launches = typed_request_launches(
            &launches,
            crate::RuntimeTypedRequestRef::FrontierVerification(&request.request_id),
        )?
        .into_iter()
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
                    attempt: EpiphanyAgentPassAttemptProjection::unattempted(),
                }));
            }
            continue;
        };
        let result = cache.get::<crate::EpiphanyRuntimeRoleWorkerResult>(&launch.job_id)?;
        let job = cache
            .get::<crate::EpiphanyRuntimeJob>(&launch.job_id)?
            .ok_or_else(|| anyhow!("frontier Verification launch lost its runtime job"))?;
        let retry_allowed =
            crate::runtime_spine::verification_frontier_is_current(cache, &request)?;
        let admission_refused = result.as_ref().is_some_and(|result| {
            admission_refusals.iter().any(|refusal| {
                refusal.pass_family == EpiphanyAgentPassFamily::FrontierVerification
                    && refusal.request_id == request.request_id
                    && refusal.job_id == launch.job_id
                    && refusal.result_id == result.result_id
                    && refusal.decision_context_id == result.decision_context_id
            })
        });
        let mut attempt = if admission_refused {
            EpiphanyAgentPassAttemptProjection::with(
                EpiphanyAgentPassContinuationAction::Launch,
                Some(job.job_id.clone()),
            )
        } else {
            EpiphanyAgentPassAttemptProjection::from_job(&job, result.is_some())
        };
        if attempt.action == EpiphanyAgentPassContinuationAction::Launch && !retry_allowed {
            attempt.action = EpiphanyAgentPassContinuationAction::Wait;
        }
        return Ok(Some(EpiphanyVerificationWorkProjection {
            request,
            attempt,
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
    let admission_refusals = cache.get_all::<EpiphanyAgentPassAdmissionRefusal>()?;
    for request in requests {
        let mut request_launches = typed_request_launches(
            &launches,
            crate::RuntimeTypedRequestRef::FrontierVerdictModeling(&request.request_id),
        )?
        .into_iter()
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
                attempt: EpiphanyAgentPassAttemptProjection::unattempted(),
            }));
        };
        let result = cache.get::<crate::EpiphanyRuntimeRoleWorkerResult>(&launch.job_id)?;
        let job = cache
            .get::<crate::EpiphanyRuntimeJob>(&launch.job_id)?
            .ok_or_else(|| anyhow!("frontier verdict Modeling launch lost its runtime job"))?;
        let admission_refused = result.as_ref().is_some_and(|result| {
            admission_refusals.iter().any(|refusal| {
                refusal.pass_family == EpiphanyAgentPassFamily::FrontierVerdictModeling
                    && refusal.request_id == request.request_id
                    && refusal.job_id == launch.job_id
                    && refusal.result_id == result.result_id
                    && refusal.decision_context_id == result.decision_context_id
            })
        });
        let attempt = if admission_refused {
            EpiphanyAgentPassAttemptProjection::with(
                EpiphanyAgentPassContinuationAction::Launch,
                Some(job.job_id.clone()),
            )
        } else {
            EpiphanyAgentPassAttemptProjection::from_job(&job, result.is_some())
        };
        return Ok(Some(EpiphanyFrontierVerdictModelingWorkProjection {
            request,
            attempt,
        }));
    }
    Ok(None)
}

fn body_modeling_attempt_ordinal(work_id: &str, job_id: &str) -> Result<usize> {
    let prefix = format!("{work_id}-attempt-");
    job_id
        .strip_prefix(&prefix)
        .ok_or_else(|| anyhow!("Body Modeling job identity is not canonical"))?
        .parse::<usize>()
        .map_err(|_| anyhow!("Body Modeling attempt ordinal is invalid"))
}

fn body_modeling_launches(
    cache: &CultCache,
    work: &EpiphanyBodyModelingWorkProjection,
) -> Result<Vec<(usize, crate::EpiphanyRuntimeWorkerLaunchRequest)>> {
    let prefix = format!("{}-attempt-", work.work_id);
    let mut launches = cache
        .get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?
        .into_iter()
        .filter(|launch| launch.job_id.starts_with(&prefix))
        .map(|launch| {
            let ordinal = body_modeling_attempt_ordinal(&work.work_id, &launch.job_id)?;
            let document = launch.launch_document()?;
            let role = match &document {
                crate::EpiphanyWorkerLaunchDocument::Role(role) => role,
                crate::EpiphanyWorkerLaunchDocument::Reorient(_) => {
                    return Err(anyhow!("Body Modeling launch used reorientation authority"));
                }
            };
            if launch.role != crate::EPIPHANY_MODELING_OWNER_ROLE
                || launch.binding_id != crate::EPIPHANY_MODELING_ROLE_BINDING_ID
                || document.thread_id() != launch.job_id
                || role.role_id != "modeling"
                || role.repository_body_observation_basis.as_ref() != Some(&work.body_basis)
                || role.family_context_count() != 0
            {
                return Err(anyhow!(
                    "Body Modeling immutable launch crossed family authority"
                ));
            }
            Ok((ordinal, launch))
        })
        .collect::<Result<Vec<_>>>()?;
    launches.sort_by_key(|(ordinal, _)| *ordinal);
    for (expected, (ordinal, _)) in launches.iter().enumerate() {
        if *ordinal != expected {
            return Err(anyhow!(
                "Body Modeling request has noncontiguous attempt identity"
            ));
        }
    }
    Ok(launches)
}

fn body_modeling_attempt(
    cache: &CultCache,
    work: &EpiphanyBodyModelingWorkProjection,
) -> Result<EpiphanyAgentPassAttemptProjection> {
    let launches = body_modeling_launches(cache, work)?;
    let Some((_, launch)) = launches.last() else {
        return Ok(EpiphanyAgentPassAttemptProjection::unattempted());
    };
    let job = cache
        .get::<crate::EpiphanyRuntimeJob>(&launch.job_id)?
        .ok_or_else(|| anyhow!("Body Modeling launch lost its runtime job"))?;
    let result = cache.get::<crate::EpiphanyRuntimeRoleWorkerResult>(&launch.job_id)?;
    let reviewable = result.is_some();
    let admission_refusals = cache.get_all::<EpiphanyAgentPassAdmissionRefusal>()?;
    let admission_refused = result.as_ref().is_some_and(|result| {
        admission_refusals.iter().any(|refusal| {
            refusal.pass_family == EpiphanyAgentPassFamily::BodyModeling
                && refusal.request_id == work.work_id
                && refusal.job_id == launch.job_id
                && refusal.result_id == result.result_id
                && refusal.decision_context_id == result.decision_context_id
        })
    });
    if admission_refused {
        return Ok(EpiphanyAgentPassAttemptProjection::with(
            EpiphanyAgentPassContinuationAction::Launch,
            Some(launch.job_id.clone()),
        ));
    }
    Ok(EpiphanyAgentPassAttemptProjection::from_job(
        &job, reviewable,
    ))
}

pub(crate) fn proposal_modeling_attempt_ordinal(request_id: &str, job_id: &str) -> Result<usize> {
    let prefix = format!("proposal-modeling-{request_id}-attempt-");
    job_id
        .strip_prefix(&prefix)
        .ok_or_else(|| anyhow!("proposal Modeling job identity is not canonical"))?
        .parse::<usize>()
        .map_err(|_| anyhow!("proposal Modeling attempt ordinal is invalid"))
}

fn current_proposal_modeling_work(
    cache: &CultCache,
) -> Result<Option<EpiphanyProposalModelingWorkProjection>> {
    let mut requests = cache.get_all::<RepoFrontierProposalModelingRequest>()?;
    requests.sort_by(|left, right| left.request_id.cmp(&right.request_id));
    let launches = cache.get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?;
    let receipts = cache.get_all::<crate::EpiphanyMindCommitReceipt>()?;
    let admission_refusals = cache.get_all::<EpiphanyAgentPassAdmissionRefusal>()?;
    'requests: for request in requests {
        crate::runtime_spine::validate_repo_frontier_proposal_modeling_request(&request)?;
        let mut request_launches = typed_request_launches(
            &launches,
            crate::RuntimeTypedRequestRef::ProposalModeling(&request.request_id),
        )?
        .into_iter()
        .map(|launch| {
            Ok((
                proposal_modeling_attempt_ordinal(&request.request_id, &launch.job_id)?,
                launch,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
        request_launches.sort_by_key(|(ordinal, _)| *ordinal);
        for (expected, (ordinal, _)) in request_launches.iter().enumerate() {
            if *ordinal != expected {
                return Err(anyhow!(
                    "proposal Modeling request has noncontiguous attempt identity"
                ));
            }
        }
        if request_launches.is_empty() {
            return Ok(Some(EpiphanyProposalModelingWorkProjection {
                request,
                attempt: EpiphanyAgentPassAttemptProjection::unattempted(),
            }));
        }
        let latest_ordinal = request_launches
            .last()
            .expect("checked nonempty launches")
            .0;
        let mut admitted_ordinal = None;
        for (ordinal, launch) in &request_launches {
            let result = cache.get::<crate::EpiphanyRuntimeRoleWorkerResult>(&launch.job_id)?;
            if let Some(result) = result.as_ref() {
                if receipts.iter().any(|receipt| {
                    receipt.invariant_owner == "Modeling.proposal_frontier"
                        && matches!(
                            &receipt.authority,
                            crate::EpiphanyMindCommitAuthority::ModelDecisionContext {
                                decision_context_id
                            } if decision_context_id == &result.decision_context_id
                        )
                }) {
                    if admitted_ordinal.replace(*ordinal).is_some() {
                        return Err(anyhow!(
                            "proposal Modeling request has multiple admitted attempts"
                        ));
                    }
                    continue;
                }
            }
            let job = cache
                .get::<crate::EpiphanyRuntimeJob>(&launch.job_id)?
                .ok_or_else(|| anyhow!("proposal Modeling launch lost its runtime job"))?;
            let admission_refused = admission_refusals.iter().any(|refusal| {
                refusal.pass_family == EpiphanyAgentPassFamily::ProposalModeling
                    && refusal.request_id == request.request_id
                    && refusal.job_id == launch.job_id
                    && result.as_ref().is_some_and(|result| {
                        refusal.result_id == result.result_id
                            && refusal.decision_context_id == result.decision_context_id
                    })
            });
            if result.is_some() {
                crate::runtime_spine::validate_proposal_modeling_worker_fulfillment(
                    cache,
                    result.as_ref().expect("checked terminal result"),
                )?;
            }
            if *ordinal != latest_ordinal
                && !matches!(job.status, crate::EpiphanyRuntimeJobStatus::Failed)
                && !admission_refused
            {
                return Err(anyhow!(
                    "proposal Modeling request has split current attempt authority"
                ));
            }
        }
        if let Some(ordinal) = admitted_ordinal {
            if ordinal != latest_ordinal {
                return Err(anyhow!(
                    "admitted proposal Modeling request retains later attempt authority"
                ));
            }
            continue 'requests;
        }
        let (_, latest) = request_launches.last().expect("checked nonempty launches");
        let result = cache.get::<crate::EpiphanyRuntimeRoleWorkerResult>(&latest.job_id)?;
        let job = cache
            .get::<crate::EpiphanyRuntimeJob>(&latest.job_id)?
            .ok_or_else(|| anyhow!("proposal Modeling launch lost its runtime job"))?;
        let admission_refused = result.as_ref().is_some_and(|result| {
            admission_refusals.iter().any(|refusal| {
                refusal.pass_family == EpiphanyAgentPassFamily::ProposalModeling
                    && refusal.request_id == request.request_id
                    && refusal.job_id == latest.job_id
                    && refusal.result_id == result.result_id
                    && refusal.decision_context_id == result.decision_context_id
            })
        });
        let attempt = if admission_refused {
            EpiphanyAgentPassAttemptProjection::with(
                EpiphanyAgentPassContinuationAction::Launch,
                Some(job.job_id.clone()),
            )
        } else {
            EpiphanyAgentPassAttemptProjection::from_job(&job, result.is_some())
        };
        return Ok(Some(EpiphanyProposalModelingWorkProjection {
            request,
            attempt,
        }));
    }
    Ok(None)
}

pub fn current_body_modeling_review_job_id(store_path: impl AsRef<Path>) -> Result<Option<String>> {
    Ok(crate::project_current_work(store_path)?
        .body_modeling
        .filter(|work| work.attempt.action == EpiphanyAgentPassContinuationAction::Review)
        .and_then(|work| work.attempt.job_id))
}

pub fn current_proposal_modeling_review_job_id(
    store_path: impl AsRef<Path>,
) -> Result<Option<String>> {
    Ok(project_current_work(store_path)?
        .proposal_modeling
        .filter(|work| work.attempt.action == EpiphanyAgentPassContinuationAction::Review)
        .and_then(|work| work.attempt.job_id))
}

pub fn current_frontier_verdict_modeling_review_job_id(
    store_path: impl AsRef<Path>,
) -> Result<Option<String>> {
    Ok(project_current_work(store_path)?
        .frontier_verdict_modeling
        .filter(|work| work.attempt.action == EpiphanyAgentPassContinuationAction::Review)
        .and_then(|work| work.attempt.job_id))
}

pub fn current_frontier_research_review_job_id(
    store_path: impl AsRef<Path>,
) -> Result<Option<String>> {
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let lifecycle = crate::runtime_spine::repo_frontier_research_lifecycle(&cache)?;
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
        .filter(|work| work.attempt.action == EpiphanyAgentPassContinuationAction::Review)
        .and_then(|work| work.attempt.job_id))
}

pub(crate) fn frontier_planning_attempt_ordinal(request_id: &str, job_id: &str) -> Result<usize> {
    let prefix = format!("frontier-planning-{request_id}-attempt-");
    job_id
        .strip_prefix(&prefix)
        .ok_or_else(|| anyhow!("frontier Planning job identity is not canonical"))?
        .parse::<usize>()
        .map_err(|_| anyhow!("frontier Planning attempt ordinal is invalid"))
}

pub(crate) fn frontier_plan_mind_attempt_ordinal(request_id: &str, job_id: &str) -> Result<usize> {
    let prefix = format!("frontier-plan-mind-{request_id}-attempt-");
    job_id
        .strip_prefix(&prefix)
        .ok_or_else(|| anyhow!("frontier plan Mind job identity is not canonical"))?
        .parse::<usize>()
        .map_err(|_| anyhow!("frontier plan Mind attempt ordinal is invalid"))
}

pub fn launch_current_frontier_planning_work(
    store_path: impl AsRef<Path>,
    created_at: &str,
) -> Result<String> {
    let store_path = store_path.as_ref();
    chrono::DateTime::parse_from_rfc3339(created_at)
        .map_err(|_| anyhow!("frontier Planning launch time is invalid"))?;
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let lifecycle = crate::runtime_spine::repo_frontier_planning_lifecycle(&cache)?;
    if lifecycle.stage != RepoFrontierPlanningLifecycleStage::ImaginationLaunchReady {
        return Err(anyhow!("Mind has no launchable frontier Planning work"));
    }
    let request_id = lifecycle
        .planning_request_id
        .ok_or_else(|| anyhow!("frontier Planning work lost its request"))?;
    let request = cache
        .get::<crate::RepoFrontierPlanningRequest>(&request_id)?
        .ok_or_else(|| anyhow!("frontier Planning request disappeared"))?;
    crate::runtime_spine::validate_actionable_repo_frontier_planning_request(&cache, &request)?;
    let identity = cache
        .get::<crate::EpiphanyRuntimeIdentity>(crate::RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("frontier Planning launch requires runtime identity"))?;
    let launches = cache.get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?;
    let mut prior = frontier_planning_launches(&launches, &request.request_id)?
        .into_iter()
        .cloned()
        .map(|launch| {
            Ok((
                frontier_planning_attempt_ordinal(&request.request_id, &launch.job_id)?,
                launch,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    prior.sort_by_key(|(ordinal, _)| *ordinal);
    for (expected, (ordinal, _)) in prior.iter().enumerate() {
        if *ordinal != expected {
            return Err(anyhow!(
                "frontier Planning request has noncontiguous attempt identity"
            ));
        }
    }
    let attempt_ordinal = prior.len();
    let retry_authority = if let Some((_, latest)) = prior.last() {
        let result = cache
            .get::<crate::EpiphanyRuntimeRoleWorkerResult>(&latest.job_id)?
            .ok_or_else(|| anyhow!("frontier Planning retry lost its failure result"))?;
        let review = crate::runtime_spine::repo_frontier_planning_failure_review(
            &cache,
            &request.request_id,
            "imagination",
            &result,
        )?
        .ok_or_else(|| anyhow!("frontier Planning retry is not reviewed"))?;
        Some((result, review))
    } else {
        None
    };
    let job_id = format!(
        "frontier-planning-{}-attempt-{attempt_ordinal}",
        request.request_id
    );
    let launch_document =
        crate::EpiphanyWorkerLaunchDocument::Role(crate::EpiphanyRoleWorkerLaunchDocument {
            thread_id: job_id.clone(),
            role_id: "imagination".into(),
            objective: None,
            dynamic_prompt_context: None,
            repository_body_observation_basis: None,
            proposal_modeling_context: None,
            frontier_verdict_modeling_context: None,
            frontier_planning_context: Some(
                crate::RepoFrontierPlanningContextProjection::from_request(&request),
            ),
            frontier_research_context: None,
            frontier_verification_context: None,
            frontier_plan_mind_context: None,
            imagination_consideration_context: None,
            admitted_model_direction_consideration_context: None,
        });
    let authority_scope = "epiphany.role.imagination".to_string();
    let output_contract_id = launch_document.output_contract_id().to_string();
    let prepared = crate::prepare_runtime_spine_heartbeat_job(
        &cache,
        crate::RuntimeSpineHeartbeatJobOptions {
            runtime_id: identity.runtime_id.clone(),
            job_id: job_id.clone(),
            role: crate::EPIPHANY_IMAGINATION_OWNER_ROLE.into(),
            binding_id: crate::EPIPHANY_IMAGINATION_ROLE_BINDING_ID.into(),
            authority_scope: authority_scope.clone(),
            instruction: "Act as Epiphany Imagination. Propose one bounded candidate for only the exact typed frontier and return the dedicated planning candidate contract.".into(),
            launch_document,
            output_contract_id: output_contract_id.clone(),
            created_at: created_at.into(),
        },
    )?;
    let snapshot = cache.snapshot_envelopes();
    let mut expected = vec![
        cache
            .get_envelope::<crate::RepoFrontierPlanningRequest>(&request.request_id)?
            .ok_or_else(|| anyhow!("frontier Planning launch lost its request envelope"))?,
    ];
    for source in request
        .frontier_authority_documents
        .iter()
        .chain(request.claim_obligation_documents.iter())
    {
        let envelope = snapshot
            .iter()
            .find(|envelope| {
                envelope.r#type == source.document_type && envelope.key == source.document_key
            })
            .ok_or_else(|| anyhow!("frontier Planning launch lost exact authority"))?;
        if EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", envelope)? != *source {
            return Err(anyhow!("frontier Planning launch authority changed"));
        }
        expected.push(envelope.clone());
    }
    for (_, launch) in &prior {
        expected.push(
            cache
                .get_envelope::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&launch.job_id)?
                .ok_or_else(|| anyhow!("frontier Planning retry lost its prior launch envelope"))?,
        );
    }
    if let Some((result, review)) = retry_authority {
        expected.push(
            cache
                .get_envelope::<crate::EpiphanyRuntimeRoleWorkerResult>(&result.job_id)?
                .ok_or_else(|| anyhow!("frontier Planning retry lost its result envelope"))?,
        );
        expected.push(
            cache
                .get_envelope::<crate::RepoFrontierPlanningFailureReview>(&review.review_id)?
                .ok_or_else(|| anyhow!("frontier Planning retry lost its review envelope"))?,
        );
    }
    commit_current_work_launch(
        store_path,
        &cache,
        expected,
        prepared,
        Vec::new(),
        "frontier Planning",
    )?;
    Ok(job_id)
}

pub fn launch_current_frontier_plan_mind_work(
    store_path: impl AsRef<Path>,
    created_at: &str,
) -> Result<String> {
    let store_path = store_path.as_ref();
    chrono::DateTime::parse_from_rfc3339(created_at)
        .map_err(|_| anyhow!("frontier plan Mind launch time is invalid"))?;
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let lifecycle = crate::runtime_spine::repo_frontier_planning_lifecycle(&cache)?;
    if lifecycle.stage != RepoFrontierPlanningLifecycleStage::MindLaunchReady {
        return Err(anyhow!("Mind has no launchable frontier plan review"));
    }
    let planning_request_id = lifecycle
        .planning_request_id
        .ok_or_else(|| anyhow!("frontier plan Mind launch lost its Planning request"))?;
    let request_id = lifecycle
        .mind_request_id
        .ok_or_else(|| anyhow!("frontier plan Mind launch lost its request"))?;
    let request = cache
        .get::<crate::RepoFrontierPlanMindRequest>(&request_id)?
        .ok_or_else(|| anyhow!("frontier plan Mind request disappeared"))?;
    let (planning, candidate) =
        crate::runtime_spine::validate_repo_frontier_plan_mind_request(&cache, &request)?;
    if planning.request_id != planning_request_id {
        return Err(anyhow!(
            "frontier plan Mind request crossed Planning authority"
        ));
    }
    let identity = cache
        .get::<crate::EpiphanyRuntimeIdentity>(crate::RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("frontier plan Mind launch requires runtime identity"))?;
    let launches = cache.get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?;
    let mut prior = frontier_plan_mind_launches(&launches, &request.request_id)?
        .into_iter()
        .cloned()
        .map(|launch| {
            Ok((
                frontier_plan_mind_attempt_ordinal(&request.request_id, &launch.job_id)?,
                launch,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    prior.sort_by_key(|(ordinal, _)| *ordinal);
    for (expected, (ordinal, _)) in prior.iter().enumerate() {
        if *ordinal != expected {
            return Err(anyhow!(
                "frontier plan Mind request has noncontiguous attempt identity"
            ));
        }
    }
    let attempt_ordinal = prior.len();
    let retry_authority = if let Some((_, latest)) = prior.last() {
        let result = cache
            .get::<crate::EpiphanyRuntimeRoleWorkerResult>(&latest.job_id)?
            .ok_or_else(|| anyhow!("frontier plan Mind retry lost its failure result"))?;
        let review = crate::runtime_spine::repo_frontier_planning_failure_review(
            &cache,
            &planning.request_id,
            "mind",
            &result,
        )?
        .ok_or_else(|| anyhow!("frontier plan Mind retry is not reviewed"))?;
        Some((result, review))
    } else {
        None
    };
    let job_id = format!(
        "frontier-plan-mind-{}-attempt-{attempt_ordinal}",
        request.request_id
    );
    let launch_document =
        crate::EpiphanyWorkerLaunchDocument::Role(crate::EpiphanyRoleWorkerLaunchDocument {
            thread_id: job_id.clone(),
            role_id: "mindAdmissionReview".into(),
            objective: None,
            dynamic_prompt_context: None,
            repository_body_observation_basis: None,
            proposal_modeling_context: None,
            frontier_verdict_modeling_context: None,
            frontier_planning_context: None,
            frontier_research_context: None,
            frontier_verification_context: None,
            frontier_plan_mind_context: Some(crate::RepoFrontierPlanMindContextProjection::new(
                &request, &planning, &candidate,
            )),
            imagination_consideration_context: None,
            admitted_model_direction_consideration_context: None,
        });
    let authority_scope = "epiphany.role.mind".to_string();
    let output_contract_id = launch_document.output_contract_id().to_string();
    let prepared = crate::prepare_runtime_spine_heartbeat_job(
        &cache,
        crate::RuntimeSpineHeartbeatJobOptions {
            runtime_id: identity.runtime_id.clone(),
            job_id: job_id.clone(),
            role: crate::EPIPHANY_MIND_OWNER_ROLE.into(),
            binding_id: crate::EPIPHANY_MIND_ROLE_BINDING_ID.into(),
            authority_scope: authority_scope.clone(),
            instruction: "Act as Epiphany Mind. Adopt, refuse, or hold only the exact typed candidate and return the dedicated plan decision contract.".into(),
            launch_document,
            output_contract_id: output_contract_id.clone(),
            created_at: created_at.into(),
        },
    )?;
    let snapshot = cache.snapshot_envelopes();
    let mut expected = Vec::new();
    for (document_type, document_key) in [
        (
            crate::RepoFrontierPlanMindRequest::TYPE,
            request.request_id.as_str(),
        ),
        (
            crate::RepoFrontierPlanningRequest::TYPE,
            planning.request_id.as_str(),
        ),
        (
            crate::EpiphanyRuntimeRoleWorkerResult::TYPE,
            request.imagination_job_id.as_str(),
        ),
    ] {
        expected.push(
            snapshot
                .iter()
                .find(|envelope| envelope.r#type == document_type && envelope.key == document_key)
                .cloned()
                .ok_or_else(|| anyhow!("frontier plan Mind launch lost a strong source"))?,
        );
    }
    for source in planning
        .frontier_authority_documents
        .iter()
        .chain(planning.claim_obligation_documents.iter())
    {
        let envelope = snapshot
            .iter()
            .find(|envelope| {
                envelope.r#type == source.document_type && envelope.key == source.document_key
            })
            .ok_or_else(|| anyhow!("frontier plan Mind launch lost exact authority"))?;
        if EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", envelope)? != *source {
            return Err(anyhow!("frontier plan Mind authority changed"));
        }
        expected.push(envelope.clone());
    }
    for (_, launch) in &prior {
        expected.push(
            cache
                .get_envelope::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&launch.job_id)?
                .ok_or_else(|| {
                    anyhow!("frontier plan Mind retry lost its prior launch envelope")
                })?,
        );
    }
    if let Some((result, review)) = retry_authority {
        expected.push(
            cache
                .get_envelope::<crate::EpiphanyRuntimeRoleWorkerResult>(&result.job_id)?
                .ok_or_else(|| anyhow!("frontier plan Mind retry lost its result envelope"))?,
        );
        expected.push(
            cache
                .get_envelope::<crate::RepoFrontierPlanningFailureReview>(&review.review_id)?
                .ok_or_else(|| anyhow!("frontier plan Mind retry lost its review envelope"))?,
        );
    }
    commit_current_work_launch(
        store_path,
        &cache,
        expected,
        prepared,
        Vec::new(),
        "frontier plan Mind",
    )?;
    Ok(job_id)
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
        .filter(|work| work.attempt.action == EpiphanyAgentPassContinuationAction::Launch)
        .ok_or_else(|| anyhow!("Mind has no launchable frontier Verification work"))?;
    let request = work.request;
    let context = crate::runtime_spine::repo_frontier_verification_context(&cache, &request)?;
    let identity = cache
        .get::<crate::EpiphanyRuntimeIdentity>(crate::RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("frontier Verification launch requires runtime identity"))?;
    let launches = cache.get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?;
    let attempt_ordinal = typed_request_launches(
        &launches,
        crate::RuntimeTypedRequestRef::FrontierVerification(&request.request_id),
    )?
    .len()
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
        });
    let authority_scope = "epiphany.role.verification".to_string();
    let output_contract_id = launch_document.output_contract_id().to_string();
    let prepared = crate::prepare_runtime_spine_heartbeat_job(
        &cache,
        crate::RuntimeSpineHeartbeatJobOptions {
            runtime_id: identity.runtime_id,
            job_id: job_id.clone(),
            role: crate::EPIPHANY_VERIFICATION_OWNER_ROLE.into(),
            binding_id: crate::EPIPHANY_VERIFICATION_ROLE_BINDING_ID.into(),
            authority_scope: authority_scope.clone(),
            instruction: "Act as Epiphany Verification. Audit only the exact typed Hands consequence and route carried by this request; return a structured verdict, evidence ids, and risks.".into(),
            launch_document,
            output_contract_id: output_contract_id.clone(),
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
        prepared,
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
    let mut current = crate::runtime_spine_cache(store_path)?;
    current.pull_all_backing_stores()?;
    if crate::runtime_spine::repo_frontier_research_lifecycle(&current)?.stage
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
    let launches = cache.get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?;
    let attempt_ordinal = typed_request_launches(
        &launches,
        crate::RuntimeTypedRequestRef::FrontierResearch(&request.request_id),
    )?
    .len()
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
        });
    let authority_scope = "epiphany.role.research".to_string();
    let output_contract_id = launch_document.output_contract_id().to_string();
    let prepared = crate::prepare_runtime_spine_heartbeat_job(
        &cache,
        crate::RuntimeSpineHeartbeatJobOptions {
            runtime_id: identity.runtime_id,
            job_id: job_id.clone(),
            role: crate::EPIPHANY_RESEARCH_OWNER_ROLE.into(),
            binding_id: crate::EPIPHANY_RESEARCH_ROLE_BINDING_ID.into(),
            authority_scope: authority_scope.clone(),
            instruction: "Act as Epiphany Research. Gather only the evidence required by the exact frontier request and return keyed evidence and observations for Mind review.".into(),
            launch_document,
            output_contract_id: output_contract_id.clone(),
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
        prepared,
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
            receipt.validate()?;
            if receipt
                .strong_reads
                .iter()
                .filter(|read| read.document_type == crate::RepoFrontierResearchRequest::TYPE)
                .count()
                != 1
                || !receipt.strong_reads.iter().any(|read| {
                    read.document_type == crate::EpiphanyRuntimeRoleWorkerResult::TYPE
                        && read.document_key == job_id
                })
                || !receipt.writes.iter().any(|write| {
                    matches!(
                        write.document_type.as_str(),
                        crate::EpiphanyMindEvidenceDocument::TYPE
                            | crate::EpiphanyMindObservationDocument::TYPE
                    )
                })
            {
                return Err(anyhow!(
                    "frontier Research replay does not preserve its exact accepted decision"
                ));
            }
            return Ok(receipt);
        }
    }
    let research_lifecycle = crate::runtime_spine::repo_frontier_research_lifecycle(&cache)?;
    if research_lifecycle.stage != crate::RepoFrontierResearchLifecycleStage::ResultReady
        || research_lifecycle.worker_job_id.as_deref() != Some(job_id)
    {
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
    let fulfillment = crate::runtime_spine::runtime_typed_request_fulfillment_from_cache(
        store_path,
        &cache,
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
    if !result.role_id.eq_ignore_ascii_case("research") || result.item_error.is_some() {
        return Err(anyhow!("frontier Research result crossed family authority"));
    }
    let decision = result
        .research_decision()?
        .ok_or_else(|| anyhow!("frontier Research result has no typed decision"))?;
    decision.validate()?;
    let lookups = crate::runtime_spine::authenticated_requested_public_source_lookups_for_worker(
        &cache, job_id,
    )?;
    for lookup in &lookups {
        if !result.evidence_ids.contains(&lookup.receipt_id)
            || !result.files_inspected.contains(&lookup.source_ref)
        {
            return Err(anyhow!(
                "frontier Research result omitted authenticated public-source provenance"
            ));
        }
    }
    if decision
        .evidence
        .iter()
        .any(|evidence| !result.evidence_ids.contains(&evidence.id))
    {
        return Err(anyhow!(
            "frontier Research result did not expose every proposed evidence identity"
        ));
    }
    let mut writes = Vec::new();
    for evidence in &decision.evidence {
        writes.push(crate::mind_documents::prepare_mind_document(
            &cache,
            &evidence.id,
            &crate::EpiphanyMindEvidenceDocument {
                value: evidence.clone(),
            },
        )?);
    }
    for observation in &decision.observations {
        writes.push(crate::mind_documents::prepare_mind_document(
            &cache,
            &observation.id,
            &crate::EpiphanyMindObservationDocument {
                value: observation.clone(),
            },
        )?);
    }
    if let Some(checkpoint) = decision.investigation_checkpoint.as_ref() {
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
    match crate::reasoning_context::commit_mind_mutation(
        store_path,
        &result.decision_context_id,
        "Eyes.frontier_research",
        strong_reads,
        writes,
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
) -> Result<EpiphanyAgentPassAdmissionOutcome> {
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
            return Ok(EpiphanyAgentPassAdmissionOutcome::Committed(receipt));
        }
    }
    let verification = current_verification_work(&cache)?;
    if verification
        .filter(|work| work.attempt.action == EpiphanyAgentPassContinuationAction::Review)
        .and_then(|work| work.attempt.job_id)
        .as_deref()
        != Some(job_id)
    {
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
    let fulfillment = crate::runtime_spine::runtime_typed_request_fulfillment_from_cache(
        store_path,
        &cache,
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
    if !result.role_id.eq_ignore_ascii_case("verification") || result.item_error.is_some() {
        return Err(anyhow!(
            "frontier Verification result crossed request authority"
        ));
    }
    if !crate::runtime_spine::verification_frontier_is_current(&cache, &request)? {
        return record_agent_pass_admission_refusal(
            store_path,
            &result,
            EpiphanyAgentPassFamily::FrontierVerification,
            &request.request_id,
            "Soul.verification",
            EpiphanyAgentPassAdmissionRefusalKind::StrongReadConflict,
            "frontier Verification decision remains recorded but its strong frontier changed; a fresh pass is required",
            accepted_at,
        );
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
        receipt_id: format!("soul-verdict-{}", result.result_id),
        source_result_id: result.result_id.clone(),
        source_job_id: result.job_id.clone(),
        verdict: result.verdict.clone(),
        summary: result.summary.clone(),
        evidence_ids,
        risks: result.risks.clone(),
        emitted_at: accepted_at.to_string(),
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
        crate::EpiphanyMindCommitOutcome::Committed(receipt) => {
            Ok(EpiphanyAgentPassAdmissionOutcome::Committed(receipt))
        }
        crate::EpiphanyMindCommitOutcome::Conflict {
            document_identities,
        } => record_agent_pass_admission_refusal(
            store_path,
            &result,
            EpiphanyAgentPassFamily::FrontierVerification,
            &request.request_id,
            "Soul.verification",
            EpiphanyAgentPassAdmissionRefusalKind::StrongReadConflict,
            &format!(
                "frontier Verification admission lost exact keyed reads: {document_identities:?}"
            ),
            accepted_at,
        ),
    }
}

pub fn accept_body_modeling_result(
    store_path: impl AsRef<Path>,
    job_id: &str,
    accepted_at: &str,
) -> Result<EpiphanyAgentPassAdmissionOutcome> {
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
    let launch = cache
        .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(job_id)?
        .ok_or_else(|| anyhow!("Body Modeling acceptance lost its immutable launch"))?;
    let role_launch = role_launch_document(&launch)?;
    if result.role_id != "modeling" || role_launch.family_context_count() != 0 {
        return Err(anyhow!("result is not baseline Body Modeling authority"));
    }
    let body_basis = role_launch
        .repository_body_observation_basis
        .clone()
        .ok_or_else(|| anyhow!("Body Modeling result has no Body observation basis"))?;
    let context = cache
        .get::<crate::EpiphanyDecisionContext>(&result.decision_context_id)?
        .ok_or_else(|| anyhow!("Body Modeling result has no decision context"))?;
    let reasoning_basis = cache
        .get::<crate::EpiphanyReasoningBasis>(&context.basis_id)?
        .ok_or_else(|| anyhow!("Body Modeling result lost its reasoning basis"))?;
    let work = EpiphanyBodyModelingWorkProjection::derive(
        body_basis.runtime_id.clone(),
        body_basis,
        crate::reasoning_repo_model_basis(&reasoning_basis)?,
    )?;
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
        let plan = match crate::plan_repo_model_mutation(store_path, proposal) {
            Ok(plan) => plan,
            Err(error) => {
                return record_agent_pass_admission_refusal(
                    store_path,
                    &result,
                    EpiphanyAgentPassFamily::BodyModeling,
                    &work.work_id,
                    "Modeling.body_projection",
                    EpiphanyAgentPassAdmissionRefusalKind::RepoModelMutationRefused,
                    &error.to_string(),
                    accepted_at,
                );
            }
        };
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
        crate::EpiphanyMindCommitOutcome::Committed(receipt) => {
            Ok(EpiphanyAgentPassAdmissionOutcome::Committed(receipt))
        }
        crate::EpiphanyMindCommitOutcome::Conflict {
            document_identities,
        } => record_agent_pass_admission_refusal(
            store_path,
            &result,
            EpiphanyAgentPassFamily::BodyModeling,
            &work.work_id,
            "Modeling.body_projection",
            EpiphanyAgentPassAdmissionRefusalKind::StrongReadConflict,
            &format!("Body Modeling admission lost exact keyed reads: {document_identities:?}"),
            accepted_at,
        ),
    }
}

pub fn accept_proposal_modeling_result(
    store_path: impl AsRef<Path>,
    job_id: &str,
    accepted_at: &str,
) -> Result<EpiphanyAgentPassAdmissionOutcome> {
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
    let launch = cache
        .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(job_id)?
        .ok_or_else(|| anyhow!("proposal Modeling acceptance lost its immutable launch"))?;
    let role_launch = role_launch_document(&launch)?;
    let request_id = role_launch
        .proposal_modeling_context
        .as_ref()
        .map(|context| context.request_id.as_str())
        .ok_or_else(|| anyhow!("result is not proposal Modeling authority"))?;
    if !result.role_id.eq_ignore_ascii_case("modeling") || role_launch.family_context_count() != 1 {
        return Err(anyhow!("proposal Modeling result crossed family authority"));
    }
    let evidence = crate::runtime_spine::runtime_typed_request_fulfillment_from_cache(
        store_path,
        &cache,
        crate::RuntimeTypedRequestRef::ProposalModeling(request_id),
    )?
    .ok_or_else(|| anyhow!("proposal Modeling result is not exact typed fulfillment"))?;
    if evidence.job_id != job_id || evidence.result_id != result.result_id {
        return Err(anyhow!("proposal Modeling fulfillment identity mismatch"));
    }
    if let Err(error) =
        crate::runtime_spine::validate_proposal_modeling_worker_admission(&cache, &result)
    {
        return record_agent_pass_admission_refusal(
            store_path,
            &result,
            EpiphanyAgentPassFamily::ProposalModeling,
            request_id,
            "Modeling.proposal_frontier",
            EpiphanyAgentPassAdmissionRefusalKind::RepoModelMutationRefused,
            &error.to_string(),
            accepted_at,
        );
    }
    let proposal = result
        .repo_model_mutation_proposal()?
        .ok_or_else(|| anyhow!("proposal Modeling result has no semantic mutation proposal"))?;
    if proposal.proposal_id != format!("repo-model-mutation-proposal-{job_id}") {
        return Err(anyhow!(
            "proposal Modeling mutation proposal identity is not runtime-owned"
        ));
    }
    let plan = match crate::plan_repo_model_mutation(store_path, &proposal) {
        Ok(plan) => plan,
        Err(error) => {
            return record_agent_pass_admission_refusal(
                store_path,
                &result,
                EpiphanyAgentPassFamily::ProposalModeling,
                request_id,
                "Modeling.proposal_frontier",
                EpiphanyAgentPassAdmissionRefusalKind::RepoModelMutationRefused,
                &error.to_string(),
                accepted_at,
            );
        }
    };
    match crate::commit_mind_mutation(
        store_path,
        &result.decision_context_id,
        "Modeling.proposal_frontier",
        plan.strong_reads,
        plan.writes,
        accepted_at,
    )? {
        crate::EpiphanyMindCommitOutcome::Committed(receipt) => {
            Ok(EpiphanyAgentPassAdmissionOutcome::Committed(receipt))
        }
        crate::EpiphanyMindCommitOutcome::Conflict {
            document_identities,
        } => record_agent_pass_admission_refusal(
            store_path,
            &result,
            EpiphanyAgentPassFamily::ProposalModeling,
            request_id,
            "Modeling.proposal_frontier",
            EpiphanyAgentPassAdmissionRefusalKind::StrongReadConflict,
            &format!("proposal Modeling admission lost exact keyed reads: {document_identities:?}"),
            accepted_at,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn record_agent_pass_admission_refusal(
    store_path: &Path,
    result: &crate::EpiphanyRuntimeRoleWorkerResult,
    pass_family: EpiphanyAgentPassFamily,
    request_id: &str,
    invariant_owner: &str,
    refusal_kind: EpiphanyAgentPassAdmissionRefusalKind,
    reason: &str,
    refused_at: &str,
) -> Result<EpiphanyAgentPassAdmissionOutcome> {
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let refusal_id = format!("agent-pass-admission-refusal-{}", result.job_id);
    if let Some(existing) = cache.get::<EpiphanyAgentPassAdmissionRefusal>(&refusal_id)? {
        existing.validate()?;
        if existing.pass_family != pass_family
            || existing.request_id != request_id
            || existing.result_id != result.result_id
            || existing.decision_context_id != result.decision_context_id
            || existing.invariant_owner != invariant_owner
            || existing.refusal_kind != refusal_kind
            || existing.reason != reason
        {
            return Err(anyhow!("agent-pass admission refusal identity collision"));
        }
        let envelope = cache
            .get_envelope::<EpiphanyAgentPassAdmissionRefusal>(&refusal_id)?
            .ok_or_else(|| anyhow!("agent-pass admission refusal lost its envelope"))?;
        let version = EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", &envelope)?;
        let commit = cache
            .get_all::<crate::EpiphanyMindCommitReceipt>()?
            .into_iter()
            .find(|receipt| {
                receipt.invariant_owner == format!("{invariant_owner}.refusal")
                    && receipt.writes.contains(&version)
                    && matches!(
                        &receipt.authority,
                        crate::EpiphanyMindCommitAuthority::ModelDecisionContext {
                            decision_context_id
                        } if decision_context_id == &result.decision_context_id
                    )
            })
            .ok_or_else(|| anyhow!("agent-pass admission refusal lost its commit receipt"))?;
        return Ok(EpiphanyAgentPassAdmissionOutcome::Refused {
            refusal: existing,
            commit,
        });
    }
    if chrono::DateTime::parse_from_rfc3339(refused_at).is_err() {
        return Err(anyhow!("agent-pass admission refusal time is invalid"));
    }
    let job_envelope = cache
        .get_envelope::<crate::EpiphanyRuntimeJob>(&result.job_id)?
        .ok_or_else(|| anyhow!("agent-pass admission refusal lost its runtime job"))?;
    let job = cache
        .get::<crate::EpiphanyRuntimeJob>(&result.job_id)?
        .ok_or_else(|| anyhow!("agent-pass admission refusal lost its runtime job"))?;
    if job.status != crate::EpiphanyRuntimeJobStatus::Completed {
        return Err(anyhow!(
            "agent-pass admission refusal requires a completed structured result"
        ));
    }
    let result_envelope = cache
        .get_envelope::<crate::EpiphanyRuntimeRoleWorkerResult>(&result.job_id)?
        .ok_or_else(|| anyhow!("agent-pass admission refusal lost its typed result"))?;
    if cache
        .get::<crate::EpiphanyRuntimeRoleWorkerResult>(&result.job_id)?
        .as_ref()
        != Some(result)
    {
        return Err(anyhow!(
            "agent-pass admission refusal result was substituted"
        ));
    }
    let refusal = EpiphanyAgentPassAdmissionRefusal {
        schema_version: AGENT_PASS_ADMISSION_REFUSAL_SCHEMA_VERSION.into(),
        refusal_id,
        pass_family,
        request_id: request_id.into(),
        job_id: result.job_id.clone(),
        result_id: result.result_id.clone(),
        decision_context_id: result.decision_context_id.clone(),
        invariant_owner: invariant_owner.into(),
        refusal_kind,
        reason: reason.into(),
        refused_at: refused_at.into(),
    };
    refusal.validate()?;
    let refusal_envelope =
        crate::mind_documents::prepare_mind_document(&cache, &refusal.refusal_id, &refusal)?;
    match crate::commit_mind_mutation(
        store_path,
        &result.decision_context_id,
        &format!("{invariant_owner}.refusal"),
        vec![job_envelope, result_envelope],
        vec![refusal_envelope],
        refused_at,
    )? {
        crate::EpiphanyMindCommitOutcome::Committed(commit) => {
            Ok(EpiphanyAgentPassAdmissionOutcome::Refused { refusal, commit })
        }
        crate::EpiphanyMindCommitOutcome::Conflict {
            document_identities,
        } => Err(anyhow!(
            "agent-pass admission refusal lost exact keyed reads: {document_identities:?}"
        )),
    }
}

pub fn accept_frontier_verdict_modeling_result(
    store_path: impl AsRef<Path>,
    job_id: &str,
    accepted_at: &str,
) -> Result<EpiphanyAgentPassAdmissionOutcome> {
    let store_path = store_path.as_ref();
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let result = cache
        .get::<crate::EpiphanyRuntimeRoleWorkerResult>(job_id)?
        .ok_or_else(|| anyhow!("frontier verdict Modeling acceptance lost its typed result"))?;
    let launch = cache
        .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(job_id)?
        .ok_or_else(|| anyhow!("frontier verdict Modeling acceptance lost its immutable launch"))?;
    let role_launch = role_launch_document(&launch)?;
    let request_id = role_launch
        .frontier_verdict_modeling_context
        .as_ref()
        .map(|context| context.request.request_id.as_str())
        .ok_or_else(|| anyhow!("result is not frontier verdict Modeling authority"))?;
    let current = match current_frontier_verdict_modeling_work(&cache) {
        Ok(Some(work))
            if work.attempt.action == EpiphanyAgentPassContinuationAction::Review
                && work.attempt.job_id.as_deref() == Some(job_id) =>
        {
            work
        }
        Ok(_) => {
            return Err(anyhow!(
                "frontier verdict Modeling result is not current review work"
            ));
        }
        Err(error) => {
            return record_agent_pass_admission_refusal(
                store_path,
                &result,
                EpiphanyAgentPassFamily::FrontierVerdictModeling,
                request_id,
                "Modeling.frontier_verdict",
                EpiphanyAgentPassAdmissionRefusalKind::StrongReadConflict,
                &error.to_string(),
                accepted_at,
            );
        }
    };
    if !result.role_id.eq_ignore_ascii_case("modeling")
        || request_id != current.request.request_id
        || role_launch.family_context_count() != 1
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
    let plan = match crate::plan_repo_model_mutation(store_path, &proposal) {
        Ok(plan) => plan,
        Err(error) => {
            return record_agent_pass_admission_refusal(
                store_path,
                &result,
                EpiphanyAgentPassFamily::FrontierVerdictModeling,
                &current.request.request_id,
                "Modeling.frontier_verdict",
                EpiphanyAgentPassAdmissionRefusalKind::RepoModelMutationRefused,
                &error.to_string(),
                accepted_at,
            );
        }
    };
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
        crate::EpiphanyMindCommitOutcome::Committed(receipt) => {
            Ok(EpiphanyAgentPassAdmissionOutcome::Committed(receipt))
        }
        crate::EpiphanyMindCommitOutcome::Conflict {
            document_identities,
        } => record_agent_pass_admission_refusal(
            store_path,
            &result,
            EpiphanyAgentPassFamily::FrontierVerdictModeling,
            &current.request.request_id,
            "Modeling.frontier_verdict",
            EpiphanyAgentPassAdmissionRefusalKind::StrongReadConflict,
            &format!(
                "frontier verdict Modeling admission lost exact keyed reads: {document_identities:?}"
            ),
            accepted_at,
        ),
    }
}

pub(crate) fn commit_current_work_launch(
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

#[derive(Clone, Copy)]
enum ConsiderationLaunch<'a> {
    PersonaFeedback(&'a crate::ImaginationConsiderationRequest),
    AdmittedModelDirection(&'a crate::AdmittedModelDirectionConsiderationRequest),
}

impl ConsiderationLaunch<'_> {
    fn request_id(&self) -> &str {
        match self {
            Self::PersonaFeedback(request) => &request.request_id,
            Self::AdmittedModelDirection(request) => &request.request_id,
        }
    }

    fn model_source_documents(&self) -> &[EpiphanyMindDocumentVersion] {
        match self {
            Self::PersonaFeedback(request) => &request.model_source_documents,
            Self::AdmittedModelDirection(request) => &request.model_source_documents,
        }
    }
}

fn exact_mind_source_envelopes(
    cache: &CultCache,
    sources: &[EpiphanyMindDocumentVersion],
) -> Result<Vec<CultCacheEnvelope>> {
    let snapshot = cache.snapshot_envelopes();
    sources
        .iter()
        .map(|source| {
            let envelope = snapshot
                .iter()
                .find(|envelope| {
                    envelope.r#type == source.document_type && envelope.key == source.document_key
                })
                .ok_or_else(|| anyhow!("consideration launch lost exact Mind source"))?;
            if EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", envelope)? != *source {
                return Err(anyhow!("consideration launch Mind source changed"));
            }
            Ok(envelope.clone())
        })
        .collect()
}

fn launch_consideration_work(
    store_path: &Path,
    launch: ConsiderationLaunch<'_>,
    created_at: &str,
) -> Result<String> {
    chrono::DateTime::parse_from_rfc3339(created_at)
        .map_err(|_| anyhow!("consideration launch time is invalid"))?;
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let identity = cache
        .get::<crate::EpiphanyRuntimeIdentity>(crate::RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("consideration launch requires runtime identity"))?;
    let model = crate::repo_model_documents::assemble_repo_model_view_from_cache(&cache)?;
    let request_id = launch.request_id().to_string();
    let is_feedback = matches!(launch, ConsiderationLaunch::PersonaFeedback(_));
    let launches = cache.get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?;
    let request_ref = if is_feedback {
        crate::RuntimeTypedRequestRef::ImaginationConsideration(&request_id)
    } else {
        crate::RuntimeTypedRequestRef::AdmittedModelDirection(&request_id)
    };
    let mut prior_launches = typed_request_launches(&launches, request_ref)?
        .into_iter()
        .cloned()
        .map(|worker| {
            Ok((
                consideration_attempt_ordinal(&request_id, &worker.job_id)?,
                worker,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    prior_launches.sort_by_key(|(ordinal, _)| *ordinal);
    for (expected, (ordinal, _)) in prior_launches.iter().enumerate() {
        if *ordinal != expected {
            return Err(anyhow!("consideration has noncontiguous attempt identity"));
        }
    }
    let attempt_ordinal = prior_launches.len();
    let (dynamic_prompt_context, imagination_context, direction_context) = match launch {
        ConsiderationLaunch::PersonaFeedback(request) => {
            crate::validate_current_imagination_consideration_request(&cache, request)?;
            let current = current_imagination_consideration_work(&cache)?
                .into_iter()
                .find(|work| work.request.request_id == request.request_id)
                .ok_or_else(|| anyhow!("Persona feedback consideration is not current"))?;
            if current.attempt.action != EpiphanyAgentPassContinuationAction::Launch {
                return Err(anyhow!("Persona feedback consideration is not launchable"));
            }
            (
                crate::render_imagination_consideration_prompt(request)?,
                Some(crate::ImaginationConsiderationContextProjection::new(
                    request, &model,
                )),
                None,
            )
        }
        ConsiderationLaunch::AdmittedModelDirection(request) => {
            crate::validate_current_admitted_model_direction_consideration_request(
                &cache, request,
            )?;
            let current = current_admitted_model_direction_consideration_work(&cache)?
                .filter(|work| work.request.request_id == request.request_id)
                .ok_or_else(|| anyhow!("admitted model direction consideration is not current"))?;
            if current.attempt.action != EpiphanyAgentPassContinuationAction::Launch {
                return Err(anyhow!(
                    "admitted model direction consideration is not launchable"
                ));
            }
            (
                crate::render_admitted_model_direction_consideration_prompt(request),
                None,
                Some(
                    crate::AdmittedModelDirectionConsiderationContextProjection::new(
                        request, &model,
                    ),
                ),
            )
        }
    };
    let job_id = format!("consideration-{request_id}-attempt-{attempt_ordinal}");
    let launch_document =
        crate::EpiphanyWorkerLaunchDocument::Role(crate::EpiphanyRoleWorkerLaunchDocument {
            thread_id: job_id.clone(),
            role_id: "imagination".into(),
            objective: None,
            dynamic_prompt_context: Some(dynamic_prompt_context),
            repository_body_observation_basis: None,
            proposal_modeling_context: None,
            frontier_verdict_modeling_context: None,
            frontier_planning_context: None,
            frontier_research_context: None,
            frontier_verification_context: None,
            frontier_plan_mind_context: None,
            imagination_consideration_context: imagination_context,
            admitted_model_direction_consideration_context: direction_context,
        });
    let authority_scope = if matches!(launch, ConsiderationLaunch::PersonaFeedback(_)) {
        "epiphany.imagination.persona_feedback_consideration.proposal_only"
    } else {
        "epiphany.imagination.admitted_model_direction_consideration.proposal_only"
    }
    .to_string();
    let output_contract_id = launch_document.output_contract_id().to_string();
    let prepared = crate::prepare_runtime_spine_heartbeat_job(
        &cache,
        crate::RuntimeSpineHeartbeatJobOptions {
            runtime_id: identity.runtime_id,
            job_id: job_id.clone(),
            role: crate::EPIPHANY_IMAGINATION_OWNER_ROLE.into(),
            binding_id: crate::EPIPHANY_IMAGINATION_ROLE_BINDING_ID.into(),
            authority_scope: authority_scope.clone(),
            instruction: "Act as Epiphany Imagination for one exact proposal-only typed consideration pass and return only its dedicated terminal contract.".into(),
            launch_document,
            output_contract_id: output_contract_id.clone(),
            created_at: created_at.into(),
        },
    )?;
    let mut expected = vec![
        cache
            .snapshot_envelopes()
            .into_iter()
            .find(|envelope| {
                envelope.key == request_id && {
                    matches!(
                        launch,
                        ConsiderationLaunch::PersonaFeedback(_)
                            if envelope.r#type == crate::ImaginationConsiderationRequest::TYPE
                    ) || matches!(
                        launch,
                        ConsiderationLaunch::AdmittedModelDirection(_)
                            if envelope.r#type
                                == crate::AdmittedModelDirectionConsiderationRequest::TYPE
                    )
                }
            })
            .ok_or_else(|| anyhow!("consideration launch lost its request envelope"))?,
    ];
    expected.extend(exact_mind_source_envelopes(
        &cache,
        launch.model_source_documents(),
    )?);
    for (_, worker) in &prior_launches {
        expected.push(
            cache
                .get_envelope::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&worker.job_id)?
                .ok_or_else(|| anyhow!("consideration retry lost its prior launch envelope"))?,
        );
    }
    commit_current_work_launch(
        store_path,
        &cache,
        expected,
        prepared,
        Vec::new(),
        "consideration",
    )?;
    Ok(job_id)
}

pub fn launch_current_imagination_consideration_work(
    store_path: impl AsRef<Path>,
    request_id: &str,
    created_at: &str,
) -> Result<String> {
    let store_path = store_path.as_ref();
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let request = cache
        .get::<crate::ImaginationConsiderationRequest>(request_id)?
        .ok_or_else(|| anyhow!("Persona feedback consideration request does not exist"))?;
    launch_consideration_work(
        store_path,
        ConsiderationLaunch::PersonaFeedback(&request),
        created_at,
    )
}

pub fn launch_current_admitted_model_direction_consideration_work(
    store_path: impl AsRef<Path>,
    request_id: &str,
    created_at: &str,
) -> Result<String> {
    let store_path = store_path.as_ref();
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let request = cache
        .get::<crate::AdmittedModelDirectionConsiderationRequest>(request_id)?
        .ok_or_else(|| anyhow!("admitted model direction request does not exist"))?;
    launch_consideration_work(
        store_path,
        ConsiderationLaunch::AdmittedModelDirection(&request),
        created_at,
    )
}

pub fn launch_current_body_modeling_work(
    store_path: impl AsRef<Path>,
    created_at: &str,
) -> Result<String> {
    let store_path = store_path.as_ref();
    chrono::DateTime::parse_from_rfc3339(created_at)
        .map_err(|_| anyhow!("Body Modeling launch time is invalid"))?;
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mind = crate::mind_documents::assemble_mind_view_from_cache(&cache)?;
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
    if resolve_body_modeling_work(
        work.clone(),
        cache.get::<EpiphanyBodyModelingDecisionReceipt>(&work.work_id)?,
    )?
    .is_none()
        || body_modeling_attempt(&cache, &work)?.action
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
    let prior_launches = body_modeling_launches(&cache, &work)?;
    let attempt_ordinal = prior_launches.len();
    let job_id = format!("{}-attempt-{attempt_ordinal}", work.work_id);
    let launch_document =
        crate::EpiphanyWorkerLaunchDocument::Role(crate::EpiphanyRoleWorkerLaunchDocument {
            thread_id: job_id.clone(),
            role_id: "modeling".into(),
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
        });
    let authority_scope = "epiphany.role.modeling".to_string();
    let output_contract_id = launch_document.output_contract_id().to_string();
    let instruction = "Act as Epiphany Modeling. Model the admitted repository Body against the exact keyed Mind projection and return the typed role result contract.".to_string();
    let prepared = crate::prepare_runtime_spine_heartbeat_job(
        &cache,
        crate::RuntimeSpineHeartbeatJobOptions {
            runtime_id: work.runtime_id.clone(),
            job_id: job_id.clone(),
            role: crate::EPIPHANY_MODELING_OWNER_ROLE.into(),
            binding_id: crate::EPIPHANY_MODELING_ROLE_BINDING_ID.into(),
            authority_scope: authority_scope.clone(),
            instruction: instruction.clone(),
            launch_document: launch_document.clone(),
            output_contract_id: output_contract_id.clone(),
            created_at: created_at.into(),
        },
    )?;
    let grant = crate::substrate_gate::substrate_gate_repo_access_grant_for_worker(
        format!("substrate-grant-{job_id}"),
        job_id.clone(),
        crate::EPIPHANY_MODELING_ROLE_BINDING_ID.into(),
        crate::EPIPHANY_MODELING_OWNER_ROLE.into(),
        authority_scope,
        false,
        created_at.into(),
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
    for (_, launch) in &prior_launches {
        expected.push(
            cache
                .get_envelope::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&launch.job_id)?
                .ok_or_else(|| anyhow!("Body Modeling retry lost its prior launch envelope"))?,
        );
    }
    if let Some((_, latest)) = prior_launches.last() {
        expected.push(
            cache
                .get_envelope::<crate::EpiphanyRuntimeJob>(&latest.job_id)?
                .ok_or_else(|| anyhow!("Body Modeling retry lost its prior job envelope"))?,
        );
        if let Some(result) = cache.get::<crate::EpiphanyRuntimeRoleWorkerResult>(&latest.job_id)? {
            expected.push(
                cache
                    .get_envelope::<crate::EpiphanyRuntimeRoleWorkerResult>(&latest.job_id)?
                    .ok_or_else(|| anyhow!("Body Modeling retry lost its result envelope"))?,
            );
            if let Some(refusal) = cache
                .get_all::<EpiphanyAgentPassAdmissionRefusal>()?
                .into_iter()
                .find(|refusal| {
                    refusal.pass_family == EpiphanyAgentPassFamily::BodyModeling
                        && refusal.request_id == work.work_id
                        && refusal.job_id == latest.job_id
                        && refusal.result_id == result.result_id
                        && refusal.decision_context_id == result.decision_context_id
                })
            {
                expected.push(
                    cache
                        .get_envelope::<EpiphanyAgentPassAdmissionRefusal>(&refusal.refusal_id)?
                        .ok_or_else(|| {
                            anyhow!("Body Modeling retry lost its admission refusal envelope")
                        })?,
                );
            }
        }
    }
    commit_current_work_launch(
        store_path,
        &cache,
        expected,
        prepared,
        vec![cache.prepare_entry(&grant.receipt_id, &grant)?.0],
        "Body Modeling",
    )?;
    Ok(job_id)
}

pub fn launch_current_proposal_modeling_work(
    store_path: impl AsRef<Path>,
    options: EpiphanyProposalModelingLaunchOptions,
) -> Result<String> {
    let store_path = store_path.as_ref();
    if chrono::DateTime::parse_from_rfc3339(&options.created_at).is_err() {
        return Err(anyhow!("proposal Modeling launch options are invalid"));
    }
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let work = current_proposal_modeling_work(&cache)?
        .filter(|work| work.attempt.action == EpiphanyAgentPassContinuationAction::Launch)
        .ok_or_else(|| anyhow!("Mind has no launchable proposal Modeling work"))?;
    let request = work.request;
    let proposal = cache
        .get::<crate::RepoFrontierWorkProposal>(&request.proposal_id)?
        .ok_or_else(|| anyhow!("proposal Modeling launch lost its proposal"))?;
    crate::runtime_spine::validate_repo_frontier_work_proposal(&proposal)?;
    crate::runtime_spine::validate_autonomous_proposal_origin(&cache, &proposal)?;
    let identity = cache
        .get::<crate::EpiphanyRuntimeIdentity>(crate::RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("proposal Modeling launch requires runtime identity"))?;
    if request.runtime_id != identity.runtime_id
        || request.proposal_id != proposal.proposal_id
        || request.proposal_payload_sha256 != proposal.payload_sha256
    {
        return Err(anyhow!("proposal Modeling launch provenance mismatch"));
    }
    let mind = crate::mind_documents::assemble_mind_view_from_cache(&cache)?;
    let body_basis = mind
        .repository_body_observation
        .clone()
        .ok_or_else(|| anyhow!("proposal Modeling launch requires admitted Body state"))?;
    let model = mind
        .repo_model
        .as_ref()
        .ok_or_else(|| anyhow!("proposal Modeling launch requires keyed RepoModel state"))?;
    let launches = cache.get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?;
    let attempt_ordinal = typed_request_launches(
        &launches,
        crate::RuntimeTypedRequestRef::ProposalModeling(&request.request_id),
    )?
    .len()
        + cache
            .get_all::<crate::EpiphanyArchivedRuntimeWorkerAttempt>()?
            .into_iter()
            .filter(|attempt| {
                attempt.request_kind == "proposal-modeling"
                    && attempt.request_id == request.request_id
            })
            .count();
    let prior_admission_refusals =
        proposal_modeling_prior_admission_refusals(&cache, &request.request_id, attempt_ordinal)?;
    let proposal_context = build_proposal_modeling_context_projection(
        &request,
        &proposal,
        model,
        prior_admission_refusals.clone(),
    )?;
    let job_id = format!(
        "proposal-modeling-{}-attempt-{attempt_ordinal}",
        request.request_id
    );
    let launch_document =
        crate::EpiphanyWorkerLaunchDocument::Role(crate::EpiphanyRoleWorkerLaunchDocument {
            thread_id: request.thread_id.clone(),
            role_id: "modeling".into(),
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
        });
    let authority_scope = "epiphany.role.modeling".to_string();
    let output_contract_id = launch_document.output_contract_id().to_string();
    let prepared = crate::prepare_runtime_spine_heartbeat_job(
        &cache,
        crate::RuntimeSpineHeartbeatJobOptions {
            runtime_id: identity.runtime_id.clone(),
            job_id: job_id.clone(),
            role: crate::EPIPHANY_MODELING_OWNER_ROLE.into(),
            binding_id: crate::EPIPHANY_MODELING_ROLE_BINDING_ID.into(),
            authority_scope: authority_scope.clone(),
            instruction: "Act as Epiphany Modeling. Evaluate the exact admitted proposal against the sealed keyed Mind projection and return only the proposal Modeling result contract.".into(),
            launch_document,
            output_contract_id: output_contract_id.clone(),
            created_at: options.created_at.clone(),
        },
    )?;
    let grant = crate::substrate_gate::substrate_gate_repo_access_grant_for_worker(
        format!("substrate-grant-{job_id}"),
        job_id.clone(),
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
    for refusal in &prior_admission_refusals {
        let envelope = cache
            .get_envelope::<EpiphanyAgentPassAdmissionRefusal>(&refusal.refusal_id)?
            .ok_or_else(|| anyhow!("proposal Modeling launch lost a refusal source"))?;
        expected.push(envelope);
    }
    commit_current_work_launch(
        store_path,
        &cache,
        expected,
        prepared,
        vec![cache.prepare_entry(&grant.receipt_id, &grant)?.0],
        "proposal Modeling",
    )?;
    Ok(job_id)
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
        .filter(|work| work.attempt.action == EpiphanyAgentPassContinuationAction::Launch)
        .ok_or_else(|| anyhow!("Mind has no launchable frontier verdict Modeling work"))?;
    let request = work.request;
    let verdict = cache
        .get::<crate::SoulVerdictReceipt>(&request.soul_verdict_receipt_id)?
        .ok_or_else(|| anyhow!("frontier verdict Modeling launch lost its Soul verdict"))?;
    let frontier_document = cache
        .get::<crate::EpiphanyRepoModelFrontierDocument>(&request.frontier_item_id)?
        .ok_or_else(|| anyhow!("frontier verdict Modeling launch lost its frontier document"))?;
    let frontier_item = frontier_document.value()?;
    let mind = crate::mind_documents::assemble_mind_view_from_cache(&cache)?;
    let body_basis = mind
        .repository_body_observation
        .clone()
        .ok_or_else(|| anyhow!("frontier verdict Modeling launch requires admitted Body state"))?;
    let identity = cache
        .get::<crate::EpiphanyRuntimeIdentity>(crate::RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("frontier verdict Modeling launch requires runtime identity"))?;
    let launches = cache.get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?;
    let attempt_ordinal = typed_request_launches(
        &launches,
        crate::RuntimeTypedRequestRef::FrontierVerdictModeling(&request.request_id),
    )?
    .into_iter()
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
        });
    let authority_scope = "epiphany.role.modeling".to_string();
    let output_contract_id = launch_document.output_contract_id().to_string();
    let prepared = crate::prepare_runtime_spine_heartbeat_job(
        &cache,
        crate::RuntimeSpineHeartbeatJobOptions {
            runtime_id: identity.runtime_id,
            job_id: job_id.clone(),
            role: crate::EPIPHANY_MODELING_OWNER_ROLE.into(),
            binding_id: crate::EPIPHANY_MODELING_ROLE_BINDING_ID.into(),
            authority_scope: authority_scope.clone(),
            instruction: "Act as Epiphany Modeling. Incorporate the exact typed Soul verdict into only its routed frontier item and return the frontier-verdict result contract.".into(),
            launch_document,
            output_contract_id: output_contract_id.clone(),
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
        prepared,
        vec![cache.prepare_entry(&grant.receipt_id, &grant)?.0],
        "frontier verdict Modeling",
    )?;
    Ok(job_id)
}

fn build_proposal_modeling_context_projection(
    request: &crate::RepoFrontierProposalModelingRequest,
    proposal: &crate::RepoFrontierWorkProposal,
    model: &crate::EpiphanyRepoModelView,
    prior_admission_refusals: Vec<EpiphanyAgentPassAdmissionRefusal>,
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
        title: proposal.title.clone(),
        body: proposal.body.clone(),
        constraints: proposal.constraints.clone(),
        evidence_refs: proposal.evidence_refs.clone(),
        model_projection_digest: model.projection_digest.clone(),
        model_source_documents: model.source_documents.clone(),
        prior_admission_refusals,
    })
}

pub(crate) fn proposal_modeling_prior_admission_refusals(
    cache: &CultCache,
    request_id: &str,
    before_attempt_ordinal: usize,
) -> Result<Vec<EpiphanyAgentPassAdmissionRefusal>> {
    let mut refusals = cache
        .get_all::<EpiphanyAgentPassAdmissionRefusal>()?
        .into_iter()
        .filter(|refusal| {
            refusal.pass_family == EpiphanyAgentPassFamily::ProposalModeling
                && refusal.request_id == request_id
        })
        .map(|refusal| {
            refusal.validate()?;
            Ok((
                proposal_modeling_attempt_ordinal(request_id, &refusal.job_id)?,
                refusal,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    refusals.retain(|(ordinal, _)| *ordinal < before_attempt_ordinal);
    refusals.sort_by_key(|(ordinal, _)| *ordinal);
    Ok(refusals.into_iter().map(|(_, refusal)| refusal).collect())
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
    use crate::state_model::{
        EpiphanyMemoryDomain, EpiphanyMemoryLifecycle, EpiphanyMemoryNode, EpiphanyMemoryNodeKind,
        RepoFrontierItem, RepoFrontierStatus,
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

    fn assert_current_work_reentry_is_read_only(
        store: &Path,
        expected: &EpiphanyCurrentWorkProjection,
    ) -> Result<()> {
        let mut before = crate::runtime_spine_cache(store)?;
        before.pull_all_backing_stores()?;
        let before = before.snapshot_envelopes();
        assert_eq!(&project_current_work(store)?, expected);
        let mut after = crate::runtime_spine_cache(store)?;
        after.pull_all_backing_stores()?;
        let after = after.snapshot_envelopes();
        assert_eq!(after, before);
        assert!(after.iter().all(|envelope| {
            envelope.r#type != "epiphany.thread_state" && envelope.r#type != "epiphany.memory_graph"
        }));
        Ok(())
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
        let mut initial_runtime = crate::runtime_spine_cache(&store)?;
        initial_runtime.pull_all_backing_stores()?;
        let initial_identity = initial_runtime
            .get_envelope::<crate::EpiphanyRuntimeIdentity>(crate::RUNTIME_IDENTITY_KEY)?
            .expect("initialized runtime identity");
        crate::runtime_spine::tests::bind_test_runtime_swarm(&store, "swarm")?;
        crate::runtime_spine::tests::bind_test_repository_body(&store, "workspace")?;
        let body = crate::observe_runtime_repository_body_basis(&store)?;
        let domain = EpiphanyMemoryDomain {
            id: "body-domain".into(),
            title: "Body".into(),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        };
        let node = EpiphanyMemoryNode {
            id: "body-node".into(),
            domain_id: domain.id.clone(),
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
                    frontier: Vec::new(),
                },
            )?,
            "2026-08-17T00:00:01Z",
        )?;
        crate::reset_repository_body_read_counters();
        let current_work = project_current_work(&store)?;
        let projected_work = current_work
            .body_modeling
            .as_ref()
            .expect("current Mind projects Body Modeling work")
            .work
            .clone();
        assert_eq!(projected_work.body_basis, body);
        assert_eq!(crate::repository_body_read_counters(), (0, 0));
        assert_eq!(
            current_work.body_modeling,
            Some(EpiphanyBodyModelingCurrentWorkProjection {
                work: projected_work.clone(),
                attempt: EpiphanyAgentPassAttemptProjection::unattempted(),
            })
        );
        assert_eq!(
            current_work.mind_projection_digest,
            crate::assemble_mind_view(&store)?.projection_digest
        );
        let mut distinct_launch = current_work.clone();
        distinct_launch
            .body_modeling
            .as_mut()
            .expect("fixture has Body Modeling work")
            .work
            .work_id
            .push_str("-distinct-request");
        assert_eq!(
            distinct_launch
                .body_modeling
                .as_ref()
                .map(|work| work.attempt.action),
            current_work
                .body_modeling
                .as_ref()
                .map(|work| work.attempt.action)
        );
        assert_ne!(
            distinct_launch.projection_digest()?,
            current_work.projection_digest()?,
            "distinct exact work cannot alias merely because its route action matches"
        );
        let resident_store = temp.path().join("resident-self.cc");
        assert!(crate::ingest_resident_self_current_work_pressure(
            &resident_store,
            &store,
            1,
        )?);
        let current_work_grant = crate::issue_resident_self_grant(&resident_store, 2)?
            .expect("the exact current-work projection should receive one grant");
        assert_eq!(
            current_work_grant.pressure_kind,
            crate::RESIDENT_SELF_CURRENT_WORK_PRESSURE_KIND
        );
        assert_eq!(
            current_work_grant.provenance_ref,
            format!(
                "{}{}{}",
                crate::RESIDENT_SELF_CURRENT_WORK_PROVENANCE_PREFIX,
                current_work.projection_digest()?,
                "/launchModeling"
            )
        );
        assert_eq!(crate::repository_body_read_counters(), (0, 0));
        assert_current_work_reentry_is_read_only(&store, &current_work)?;
        let scheduled = launch_current_body_modeling_work(&store, "2026-08-17T00:00:02Z")?;
        assert_eq!(scheduled, format!("{}-attempt-0", projected_work.work_id));
        let scheduled_work = project_current_work(&store)?;
        assert_eq!(
            scheduled_work
                .body_modeling
                .as_ref()
                .map(|work| (work.attempt.action, work.attempt.job_id.as_deref())),
            Some((
                EpiphanyAgentPassContinuationAction::Wait,
                Some(scheduled.as_str())
            ))
        );
        let resident_policy = crate::ResidentSelfPolicy {
            workspace: temp.path().join("workspace"),
            coordinator_bin: temp.path().join("epiphany-mvp-coordinator"),
            model_runtime_bin: temp.path().join("epiphany-model-runtime"),
            tool_adapter_bin: temp.path().join("epiphany-tool-mcp-runtime"),
            runtime_store: store.clone(),
            local_verse_store: temp.path().join("local-verse.cc"),
            artifact_root: temp.path().join("artifacts"),
            model_connector_endpoint: "127.0.0.1:17891".parse().unwrap(),
            mcp_config: temp.path().join("mcp.toml"),
            model_provider: "openrouter".into(),
            model: "stealth/ox-alpha".into(),
            provider_credential_path: None,
            max_steps: 4,
            turn_timeout_seconds: 600,
            cooldown_seconds: 10,
            idle_sleep_seconds: 2,
            failure_backoff_seconds: 30,
            release_commit: "release-commit".into(),
            release_manifest_digest: "sha256:release-manifest".into(),
            release_store: temp.path().join("release.cc"),
            release_runtime_id: "runtime".into(),
            release_id: "release-id".into(),
            release_witness_sha256: "sha256:release-witness".into(),
        };
        assert!(
            crate::prepare_resident_self_launch(&resident_store, &resident_policy, 3)?.is_none(),
            "a grant derived from the prior Mind projection must not launch"
        );
        assert!(crate::pending_resident_self_grant(&resident_store)?.is_none());
        let terminal_receipts = crate::resident_self_terminal_receipts(&resident_store)?;
        assert_eq!(terminal_receipts.len(), 1);
        assert_eq!(terminal_receipts[0].terminal_status, "superseded");
        assert_current_work_reentry_is_read_only(&store, &scheduled_work)?;
        let mut scheduled_cache = crate::runtime_spine_cache(&store)?;
        scheduled_cache.pull_all_backing_stores()?;
        assert!(
            scheduled_cache
                .snapshot_envelopes()
                .iter()
                .all(|envelope| envelope.r#type != "epiphany.thread_state")
        );
        assert!(
            scheduled_cache
                .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&scheduled)?
                .is_some()
        );
        assert!(
            scheduled_cache
                .get::<crate::SubstrateGateRepoAccessGrantReceipt>(&format!(
                    "substrate-grant-{scheduled}"
                ))?
                .is_some()
        );
        assert!(launch_current_body_modeling_work(&store, "2026-08-17T00:00:03Z").is_err());
        assert_eq!(crate::repository_body_read_counters(), (0, 0));
        let mut cache = crate::runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let failed_launch = cache
            .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&scheduled)?
            .expect("scheduled Body launch exists");
        drop(cache);
        let failed_basis = crate::worker_reasoning_basis(&store, &failed_launch)?;
        crate::put_reasoning_basis(&store, &failed_basis)?;
        let mut failed_native = epiphany_model_adapter::EpiphanyModelRequest::new(
            "body-failed-request",
            "body-failed-conversation",
            "openai-codex",
            "gpt-test",
            "model",
        );
        failed_native.reasoning_basis_id = Some(failed_basis.basis_id.clone());
        failed_native.source_worker_job_id = Some(failed_launch.job_id.clone());
        crate::open_runtime_model_execution(
            &store,
            crate::RuntimeSpineJobOptions {
                job_id: "body-failed-model-job".into(),
                session_id: "body-failed-model-session".into(),
            },
            &failed_native,
        )?;
        let failed_process = crate::ProcessInstanceIdentity {
            process_id: u32::MAX,
            creation_token: 7,
            created_at_rfc3339: Some("2026-08-17T00:00:02.200Z".into()),
            executable_path: "body-failed-worker".into(),
        };
        let failed_activation = "body-failed-activation";
        crate::claim_runtime_worker_process(
            &store,
            &failed_launch.job_id,
            &failed_process,
            &format!("{:x}", Sha256::digest(failed_activation.as_bytes())),
            "2026-08-17T00:00:02.200Z",
        )?;
        crate::activate_runtime_worker_process(
            &store,
            &failed_launch.job_id,
            &failed_process,
            failed_activation,
            "2026-08-17T00:00:02.300Z",
        )?;
        assert_eq!(
            crate::recover_dead_runtime_worker_attempts(&store, 1_871_270_403_000)?,
            1
        );
        let mut death_cache = crate::runtime_spine_cache(&store)?;
        death_cache.pull_all_backing_stores()?;
        let death_failures = death_cache
            .get_all::<crate::EpiphanyModelPassFailure>()?
            .into_iter()
            .filter(|failure| failure.pass_id == scheduled)
            .collect::<Vec<_>>();
        let [death_failure] = death_failures.as_slice() else {
            panic!("worker death emits one typed model-pass failure")
        };
        let death_context = death_failure.decision_context_id.as_str();
        assert!(crate::audit_decision_context(&store, death_context).is_ok());
        assert_eq!(
            crate::runtime_worker_process_claim(&store, &failed_launch.job_id)?
                .expect("dead worker claim remains auditable")
                .status,
            crate::WorkerProcessStatus::TerminalDeath.as_str()
        );
        let retry_work = project_current_work(&store)?;
        assert_eq!(
            retry_work
                .body_modeling
                .as_ref()
                .map(|work| (work.attempt.action, work.attempt.job_id.as_deref())),
            Some((
                EpiphanyAgentPassContinuationAction::Launch,
                Some(scheduled.as_str())
            ))
        );
        assert_ne!(
            retry_work.projection_digest()?,
            current_work.projection_digest()?,
            "a failed Body attempt must change exact current-work identity before retry"
        );
        assert!(crate::ingest_resident_self_current_work_pressure(
            &resident_store,
            &store,
            4,
        )?);
        assert!(
            !crate::ingest_resident_self_current_work_pressure(&resident_store, &store, 5)?,
            "unchanged failed-attempt state must remain idempotent"
        );
        let retry_grant = crate::issue_resident_self_grant(&resident_store, 6)?
            .expect("failed Body attempt must receive a fresh exact-state grant");
        assert_eq!(
            retry_grant.provenance_ref,
            format!(
                "{}{}{}",
                crate::RESIDENT_SELF_CURRENT_WORK_PROVENANCE_PREFIX,
                retry_work.projection_digest()?,
                "/launchModeling"
            )
        );
        let retried = launch_current_body_modeling_work(&store, "2026-08-17T00:00:04Z")?;
        assert_eq!(retried, format!("{}-attempt-1", projected_work.work_id));
        let mut cache = crate::runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let launch = cache
            .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&retried)?
            .unwrap();
        crate::reset_repository_body_read_counters();
        let reasoning_basis = crate::worker_reasoning_basis(&store, &launch)?;
        crate::put_reasoning_basis(&store, &reasoning_basis)?;
        let (basis_body_store_loads, basis_body_file_reads) =
            crate::repository_body_read_counters();
        assert!(
            basis_body_store_loads > 0,
            "Modeling basis sealing must authenticate its persisted Body projection"
        );
        assert_eq!(
            basis_body_file_reads, 0,
            "basis sealing consumes the typed manifest, not live worktree bytes"
        );
        crate::reset_repository_body_read_counters();
        let mut native = epiphany_model_adapter::EpiphanyModelRequest::new(
            "body-request",
            "body-conversation",
            "openai-codex",
            "gpt-test",
            "model",
        );
        native.reasoning_basis_id = Some(reasoning_basis.basis_id.clone());
        native.source_worker_job_id = Some(launch.job_id.clone());
        let context = crate::EpiphanyDecisionContext::new(&reasoning_basis, native, Vec::new())?;
        cache.put(&context.context_id, &context)?;
        let result = crate::EpiphanyRuntimeRoleWorkerResult {
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
            research_decision_msgpack: None,
            item_error: None,
            metadata: Default::default(),
            repo_model_mutation_proposal_msgpack: None,
            frontier_plan_candidate_msgpack: None,
            frontier_plan_mind_decision_msgpack: None,
            imagination_consideration_candidate_msgpack: None,
            admitted_model_direction_consideration_result_msgpack: None,
            decision_context_id: context.context_id,
        };
        cache.put(&result.job_id, &result)?;
        let mut job = cache
            .get::<crate::EpiphanyRuntimeJob>(&result.job_id)?
            .unwrap();
        job.status = crate::EpiphanyRuntimeJobStatus::Completed;
        cache.put(&job.job_id, &job)?;
        assert_eq!(
            project_current_work(&store)?
                .body_modeling
                .as_ref()
                .map(|work| work.attempt.action),
            Some(EpiphanyAgentPassContinuationAction::Review)
        );
        assert_eq!(
            crate::resident_self::resident_self_current_work_action(&store)?
                .map(|(_, action)| action),
            Some("reviewModelingResult".into()),
        );
        let EpiphanyAgentPassAdmissionOutcome::Committed(receipt) =
            accept_body_modeling_result(&store, &result.job_id, "2026-08-17T00:00:05Z")?
        else {
            panic!("valid Body Modeling result must commit")
        };
        assert_eq!(
            receipt.authority,
            crate::EpiphanyMindCommitAuthority::ModelDecisionContext {
                decision_context_id: result.decision_context_id.clone(),
            }
        );
        let completed_work = project_current_work(&store)?;
        assert!(completed_work.body_modeling.is_none());
        assert!(crate::resident_self::resident_self_current_work_action(&store)?.is_none());
        assert_eq!(crate::repository_body_read_counters(), (0, 0));
        assert_current_work_reentry_is_read_only(&store, &completed_work)?;

        let route = crate::runtime_repository_body_store_binding(&store)?
            .expect("runtime Body route remains bound");
        let (body_binding, _) =
            crate::load_repository_body_status(std::path::Path::new(&route.body_store_path))?
                .expect("runtime Body store remains authenticated");
        crate::bind_runtime_repository_domain(&store, "GameCult/Epiphany")?;
        let direction_request = crate::commit_admitted_model_direction_consideration_request(
            &store,
            "2026-08-17T00:00:05.200Z",
        )?
        .expect("current keyed model produces one direction obligation");
        let direction_job_id = launch_current_admitted_model_direction_consideration_work(
            &store,
            &direction_request.request_id,
            "2026-08-17T00:00:05.300Z",
        )?;
        let mut direction_cache = crate::runtime_spine_cache(&store)?;
        direction_cache.pull_all_backing_stores()?;
        let direction_launch = direction_cache
            .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&direction_job_id)?
            .expect("direction launch remains exact");
        drop(direction_cache);
        let direction_basis = crate::worker_reasoning_basis(&store, &direction_launch)?;
        crate::put_reasoning_basis(&store, &direction_basis)?;
        let mut direction_native = epiphany_model_adapter::EpiphanyModelRequest::new(
            "direction-request",
            "direction-conversation",
            "openai-codex",
            "gpt-test",
            "model",
        );
        direction_native.reasoning_basis_id = Some(direction_basis.basis_id.clone());
        direction_native.source_worker_job_id = Some(direction_job_id.clone());
        let direction_context =
            crate::EpiphanyDecisionContext::new(&direction_basis, direction_native, Vec::new())?;
        let mut direction_cache = crate::runtime_spine_cache(&store)?;
        direction_cache.pull_all_backing_stores()?;
        direction_cache.put(&direction_context.context_id, &direction_context)?;
        let direction_result = crate::AdmittedModelDirectionConsiderationResult {
            schema_version: crate::admitted_model_direction_consideration::RESULT_SCHEMA.into(),
            result_id: crate::admitted_model_direction_consideration_result_id_for_launch(
                &direction_request.request_id,
                &direction_job_id,
            ),
            request_id: direction_request.request_id.clone(),
            runtime_id: direction_request.runtime_id.clone(),
            thread_id: direction_request.thread_id.clone(),
            model_projection_digest: direction_request.model_projection_digest.clone(),
            model_source_documents: direction_request.model_source_documents.clone(),
            disposition: crate::AdmittedModelDirectionDisposition::Suggest,
            summary: "Model one proposal".into(),
            option_drafts: vec![crate::ImaginationOptionDraft {
                title: "Model one proposal".into(),
                summary: "Add one typed frontier without touching unrelated Mind state.".into(),
            }],
            uncertainties: Vec::new(),
            evidence_refs: Vec::new(),
            proposed_at: "2026-08-17T00:00:05.400Z".into(),
            contract: crate::admitted_model_direction_consideration::RESULT_CONTRACT.into(),
            proposal_only: true,
            terminal: true,
        };
        let direction_worker_result = crate::EpiphanyRuntimeRoleWorkerResult {
            result_id: "direction-worker-result".into(),
            job_id: direction_job_id.clone(),
            role_id: "imagination".into(),
            verdict: "suggest".into(),
            summary: direction_result.summary.clone(),
            next_safe_move: "Promote through the exact proposal owner.".into(),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: Vec::new(),
            frontier_node_ids: Vec::new(),
            evidence_ids: Vec::new(),
            artifact_refs: Vec::new(),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            research_decision_msgpack: None,
            item_error: None,
            metadata: Default::default(),
            repo_model_mutation_proposal_msgpack: None,
            frontier_plan_candidate_msgpack: None,
            frontier_plan_mind_decision_msgpack: None,
            imagination_consideration_candidate_msgpack: None,
            admitted_model_direction_consideration_result_msgpack: Some(rmp_serde::to_vec_named(
                &direction_result,
            )?),
            decision_context_id: direction_context.context_id.clone(),
        };
        crate::put_runtime_role_worker_result(&store, &direction_worker_result)?;
        let mut promoted = crate::promote_autonomous_direction_options_for_modeling(
            &store,
            "GameCult/Epiphany",
            &body_binding.git_top_level,
            "2026-08-17T00:00:06Z",
        )?;
        assert_eq!(promoted.len(), 1);
        let request = promoted.remove(0);
        let mut proposal_cache = crate::runtime_spine_cache(&store)?;
        proposal_cache.pull_all_backing_stores()?;
        let proposal = proposal_cache
            .get::<crate::RepoFrontierWorkProposal>(&request.proposal_id)?
            .expect("promotion atomically persists its proposal");
        let proposal_initial_projection = project_current_work(&store)?;
        let proposal_work = proposal_initial_projection
            .proposal_modeling
            .clone()
            .expect("selected proposal must become current work");
        assert_eq!(proposal_work.request, request);
        assert_eq!(
            proposal_work.attempt.action,
            EpiphanyAgentPassContinuationAction::Launch
        );
        let proposal_resident_store = temp.path().join("proposal-resident-self.cc");
        assert!(crate::ingest_resident_self_current_work_pressure(
            &proposal_resident_store,
            &store,
            7,
        )?);
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
        let proposal_job_id = race_results
            .iter()
            .find_map(|result| result.as_ref().ok())
            .cloned()
            .expect("one proposal launch racer must commit");
        assert!(race_results.iter().all(|result| {
            result
                .as_ref()
                .map(|job_id| job_id == &proposal_job_id)
                .unwrap_or(true)
        }));
        assert_eq!(
            proposal_job_id,
            format!("proposal-modeling-{}-attempt-0", request.request_id)
        );
        assert_eq!(
            project_current_work(&store)?
                .proposal_modeling
                .expect("launched proposal remains current")
                .attempt
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
        crate::terminalize_runtime_job(
            &store,
            &proposal_job_id,
            crate::EpiphanyRuntimeJobStatus::Failed,
            "2026-08-17T00:00:09.100Z",
            &format!("worker-runtime-failure-{proposal_job_id}"),
        )?;
        let proposal_retry_projection = project_current_work(&store)?;
        assert_eq!(
            proposal_retry_projection
                .proposal_modeling
                .as_ref()
                .map(|work| (work.attempt.action, work.attempt.job_id.as_deref())),
            Some((
                EpiphanyAgentPassContinuationAction::Launch,
                Some(proposal_job_id.as_str()),
            ))
        );
        assert_ne!(
            proposal_retry_projection.projection_digest()?,
            proposal_initial_projection.projection_digest()?,
            "a failed proposal attempt must change exact current-work identity before retry"
        );
        assert!(crate::ingest_resident_self_current_work_pressure(
            &proposal_resident_store,
            &store,
            8,
        )?);
        assert!(
            !crate::ingest_resident_self_current_work_pressure(
                &proposal_resident_store,
                &store,
                9,
            )?,
            "unchanged proposal failure must remain idempotent"
        );
        let proposal_job_id = launch_current_proposal_modeling_work(
            &store,
            EpiphanyProposalModelingLaunchOptions {
                created_at: "2026-08-17T00:00:09.200Z".into(),
            },
        )?;
        assert_eq!(
            proposal_job_id,
            format!("proposal-modeling-{}-attempt-1", request.request_id)
        );
        let mut final_cache = crate::runtime_spine_cache(&store)?;
        final_cache.pull_all_backing_stores()?;
        assert_eq!(
            final_cache
                .get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?
                .into_iter()
                .map(
                    |launch| crate::RuntimeTypedRequestRef::ProposalModeling(&request.request_id)
                        .matches_launch(&launch)
                )
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .filter(|matches| *matches)
                .count(),
            2
        );
        assert!(
            final_cache
                .snapshot_envelopes()
                .iter()
                .all(|envelope| envelope.r#type != "epiphany.thread_state")
        );
        let proposal_runtime_launch = final_cache
            .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&proposal_job_id)?
            .expect("proposal runtime launch");
        let proposal_document = proposal_runtime_launch.launch_document()?;
        let crate::EpiphanyWorkerLaunchDocument::Role(proposal_document) = proposal_document else {
            panic!("proposal Modeling must be a role pass")
        };
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
        proposal_native.source_worker_job_id = Some(proposal_job_id.clone());
        let proposal_context = crate::EpiphanyDecisionContext::new(
            &proposal_reasoning_basis,
            proposal_native,
            Vec::new(),
        )?;
        final_cache.put(&proposal_context.context_id, &proposal_context)?;
        let result_id = "proposal-result";
        let mutation = crate::EpiphanyRepoModelMutationProposal::new(
            format!("repo-model-mutation-proposal-{proposal_job_id}"),
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
                    repository_scope: vec!["epiphany-core/src".into()],
                    recommended_next_organ: "Imagination".into(),
                    dependency_item_ids: vec!["body-node".into()],
                    status: RepoFrontierStatus::Active,
                    evidence_refs: vec!["body-node".into()],
                    ..Default::default()
                },
            }],
        )?;
        let proposal_result = crate::EpiphanyRuntimeRoleWorkerResult {
            result_id: result_id.into(),
            job_id: proposal_job_id.clone(),
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
            research_decision_msgpack: None,
            item_error: None,
            metadata: Default::default(),
            repo_model_mutation_proposal_msgpack: Some(rmp_serde::to_vec_named(&mutation)?),
            frontier_plan_candidate_msgpack: None,
            frontier_plan_mind_decision_msgpack: None,
            imagination_consideration_candidate_msgpack: None,
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
            &proposal_job_id,
            &process,
            &format!("{:x}", Sha256::digest(activation_token.as_bytes())),
            "2026-08-17T00:00:10Z",
        )?;
        crate::activate_runtime_worker_process(
            &store,
            &proposal_job_id,
            &process,
            activation_token,
            "2026-08-17T00:00:11Z",
        )?;
        crate::put_runtime_role_worker_result(&store, &proposal_result)?;
        assert_eq!(
            crate::runtime_worker_process_claim(&store, &proposal_job_id)?
                .expect("structured Modeling result terminalizes its process claim")
                .status,
            crate::WorkerProcessStatus::TerminalResult.as_str()
        );
        assert_eq!(
            project_current_work(&store)?
                .proposal_modeling
                .expect("terminal proposal remains current until admission")
                .attempt
                .action,
            EpiphanyAgentPassContinuationAction::Review
        );
        let proposal_outcome =
            accept_proposal_modeling_result(&store, &proposal_job_id, "2026-08-17T00:00:13Z")?;
        let EpiphanyAgentPassAdmissionOutcome::Refused {
            refusal,
            commit: refusal_commit,
        } = proposal_outcome
        else {
            panic!("invalid proposal Modeling result must be refused")
        };
        assert_eq!(
            refusal.pass_family,
            EpiphanyAgentPassFamily::ProposalModeling
        );
        assert_eq!(
            refusal.refusal_kind,
            EpiphanyAgentPassAdmissionRefusalKind::RepoModelMutationRefused
        );
        assert!(
            refusal
                .reason
                .contains("not one safe proposal-citing routeable frontier")
        );
        assert_eq!(
            refusal_commit.invariant_owner,
            "Modeling.proposal_frontier.refusal"
        );
        let refused_work = project_current_work(&store)?
            .proposal_modeling
            .expect("refused proposal remains a fresh current obligation");
        assert_eq!(
            refused_work.attempt.action,
            EpiphanyAgentPassContinuationAction::Launch
        );
        assert_eq!(
            refused_work.attempt.job_id.as_deref(),
            Some(proposal_job_id.as_str())
        );

        let proposal_job_id = launch_current_proposal_modeling_work(
            &store,
            EpiphanyProposalModelingLaunchOptions {
                created_at: "2026-08-17T00:00:13.100Z".into(),
            },
        )?;
        assert_eq!(
            proposal_job_id,
            format!("proposal-modeling-{}-attempt-2", request.request_id)
        );
        let mut retry_cache = crate::runtime_spine_cache(&store)?;
        retry_cache.pull_all_backing_stores()?;
        let retry_launch = retry_cache
            .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&proposal_job_id)?
            .expect("refused proposal retry launch");
        let crate::EpiphanyWorkerLaunchDocument::Role(retry_document) =
            retry_launch.launch_document()?
        else {
            panic!("proposal Modeling retry must be a role pass")
        };
        let retry_projection = retry_document
            .proposal_modeling_context
            .expect("proposal Modeling retry has its typed context");
        assert_eq!(retry_projection.prior_admission_refusals, vec![refusal]);
        let retry_basis = crate::worker_reasoning_basis(&store, &retry_launch)?;
        crate::put_reasoning_basis(&store, &retry_basis)?;
        let mut retry_native = epiphany_model_adapter::EpiphanyModelRequest::new(
            "proposal-retry-request",
            "proposal-retry-conversation",
            "openai-codex",
            "gpt-test",
            "model",
        );
        retry_native.reasoning_basis_id = Some(retry_basis.basis_id.clone());
        retry_native.source_worker_job_id = Some(proposal_job_id.clone());
        let retry_context =
            crate::EpiphanyDecisionContext::new(&retry_basis, retry_native, Vec::new())?;
        retry_cache.put(&retry_context.context_id, &retry_context)?;
        let valid_mutation = crate::EpiphanyRepoModelMutationProposal::new(
            format!("repo-model-mutation-proposal-{proposal_job_id}"),
            request.request_id.clone(),
            "proposal-result-retry",
            vec![proposal.proposal_id.clone()],
            body.clone(),
            vec![crate::EpiphanyRepoModelMutationOperation::PutFrontier {
                item: RepoFrontierItem {
                    id: "proposal-frontier".into(),
                    migration_body: "epiphany".into(),
                    question: "Should this proposal advance?".into(),
                    gap: "The proposal needs an adopted plan.".into(),
                    target_claim_ids: vec!["body-node".into()],
                    repository_scope: vec!["epiphany-core/src".into()],
                    recommended_next_organ: "Imagination".into(),
                    status: RepoFrontierStatus::Active,
                    evidence_refs: vec![proposal.proposal_id.clone()],
                    ..Default::default()
                },
            }],
        )?;
        let mut proposal_result = proposal_result;
        proposal_result.result_id = "proposal-result-retry".into();
        proposal_result.job_id = proposal_job_id.clone();
        proposal_result.repo_model_mutation_proposal_msgpack =
            Some(rmp_serde::to_vec_named(&valid_mutation)?);
        proposal_result.decision_context_id = retry_context.context_id.clone();
        let retry_process = crate::ProcessInstanceIdentity {
            process_id: 43,
            creation_token: 8,
            created_at_rfc3339: Some("2026-08-17T00:00:13.200Z".into()),
            executable_path: "proposal-retry-worker".into(),
        };
        let retry_activation = "proposal-retry-activation";
        crate::claim_runtime_worker_process(
            &store,
            &proposal_job_id,
            &retry_process,
            &format!("{:x}", Sha256::digest(retry_activation.as_bytes())),
            "2026-08-17T00:00:13.200Z",
        )?;
        crate::activate_runtime_worker_process(
            &store,
            &proposal_job_id,
            &retry_process,
            retry_activation,
            "2026-08-17T00:00:13.300Z",
        )?;
        crate::put_runtime_role_worker_result(&store, &proposal_result)?;
        assert_eq!(
            project_current_work(&store)?
                .proposal_modeling
                .expect("corrected proposal awaits admission")
                .attempt
                .action,
            EpiphanyAgentPassContinuationAction::Review
        );
        let proposal_outcome =
            accept_proposal_modeling_result(&store, &proposal_job_id, "2026-08-17T00:00:14Z")?;
        let EpiphanyAgentPassAdmissionOutcome::Committed(proposal_commit) = proposal_outcome else {
            panic!("corrected proposal Modeling result must commit")
        };
        assert_eq!(
            proposal_commit.invariant_owner,
            "Modeling.proposal_frontier"
        );
        assert!(project_current_work(&store)?.proposal_modeling.is_none());
        let mut accepted_cache = crate::runtime_spine_cache(&store)?;
        accepted_cache.pull_all_backing_stores()?;
        assert!(
            accepted_cache
                .snapshot_envelopes()
                .iter()
                .all(|envelope| envelope.r#type != "epiphany.thread_state")
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
            authorized_paths: frontier_item.repository_scope.clone(),
            adopted_plan: frontier_item.adopted_plan.clone(),
            selected_at: "2026-08-17T00:00:14Z".into(),
        };
        accepted_cache.put(&route.route_id, &route)?;

        let hands_job_id = "hands-verification-fixture";
        let grant = crate::substrate_gate_coordinator_implementation_grant(
            "substrate-grant-hands-verification-fixture".into(),
            hands_job_id.into(),
            route.authorized_paths.clone(),
            "2026-08-17T00:00:14.100Z".into(),
        );
        crate::put_substrate_gate_repo_access_grant_receipt(&store, &grant)?;
        let intent = crate::HandsActionIntent {
            intent_id: "hands-intent-verification-fixture".into(),
            runtime_job_id: hands_job_id.into(),
            requested_paths: route.authorized_paths.clone(),
            substrate_gate_grant_receipt_id: grant.receipt_id.clone(),
        };
        crate::put_hands_action_intent(&store, &intent)?;
        let hands_authority = crate::RepoFrontierHandsAuthority {
            authority_id: "repo-frontier-hands-authority-verification-fixture".into(),
            route_id: route.route_id.clone(),
            hands_intent_id: intent.intent_id.clone(),
            substrate_grant_receipt_id: grant.receipt_id.clone(),
        };
        crate::put_repo_frontier_hands_authority(&store, &hands_authority)?;
        let hands_patch = crate::hands_patch_receipt_for_intent(
            "hands-patch-verification-fixture".into(),
            &intent,
            route.authorized_paths.clone(),
            "Applied the exact bounded change.".into(),
            "2026-08-17T00:00:14.400Z".into(),
        );
        crate::put_hands_patch_receipt(&store, &hands_patch)?;
        let hands_command = crate::hands_command_receipt_for_intent(
            "hands-command-verification-fixture".into(),
            &intent,
            "cargo test exact-verification-fixture".into(),
            "0".into(),
            "verification.stdout".into(),
            "verification.stderr".into(),
            "The bounded command passed.".into(),
            "2026-08-17T00:00:14.500Z".into(),
        );
        crate::put_hands_command_receipt(&store, &hands_command)?;
        let hands_commit = crate::hands_commit_receipt_for_intent(
            "hands-commit-verification-fixture".into(),
            &intent,
            "0123456789abcdef0123456789abcdef01234567".into(),
            "codex/verification-fixture".into(),
            route.authorized_paths.clone(),
            "Committed the exact bounded consequence.".into(),
            "2026-08-17T00:00:14.600Z".into(),
        );
        let ambiguous_hands_store = temp.path().join("ambiguous-hands.cc");
        std::fs::copy(&store, &ambiguous_hands_store)?;
        let mut duplicate_patch = hands_patch.clone();
        duplicate_patch.receipt_id = "hands-patch-verification-fixture-duplicate".into();
        crate::put_hands_patch_receipt(&ambiguous_hands_store, &duplicate_patch)?;
        assert!(
            crate::put_hands_commit_receipt(&ambiguous_hands_store, &hands_commit)
                .unwrap_err()
                .to_string()
                .contains("one exact patch receipt")
        );
        let persona_cultmesh = temp.path().join("persona-concurrency-cultmesh.cc");
        crate::write_epiphany_cultmesh_swarm_brake(
            &persona_cultmesh,
            "runtime",
            crate::default_epiphany_cultmesh_swarm_brake("2026-08-17T00:00:14.550Z"),
        )?;
        let persona_request = crate::PersonaTurnRequest {
            request_id: "persona-concurrent-with-hands".into(),
            role_id: "Persona".into(),
            agent_id: "epiphany.Persona".into(),
            ..Default::default()
        };
        let persona_basis = crate::EpiphanyReasoningBasis::new(
            "persona:concurrent-with-hands:interpreter",
            "Persona.interpreter",
            "epiphany.reasoning_projection.persona.interpreter.v1",
            Vec::new(),
            crate::EpiphanyReasoningProjection::PersonaInterpreter(
                crate::PersonaInterpreterInput::default(),
            ),
        )?;
        accepted_cache.put(&persona_basis.basis_id, &persona_basis)?;
        let mut persona_native = epiphany_model_adapter::EpiphanyModelRequest::new(
            "persona:concurrent-with-hands:interpreter",
            "persona-turn-concurrent-with-hands",
            "openai-codex",
            "gpt-test",
            "interpret",
        );
        persona_native.reasoning_basis_id = Some(persona_basis.basis_id.clone());
        let persona_context =
            crate::EpiphanyDecisionContext::new(&persona_basis, persona_native, Vec::new())?;
        accepted_cache.put(&persona_context.context_id, &persona_context)?;
        let persona_effect = crate::PersonaInterpreterEffectDocument {
            schema_version: crate::PERSONA_INTERPRETER_EFFECT_DOCUMENT_SCHEMA_VERSION.into(),
            document_id: "persona-effects:concurrent-with-hands".into(),
            turn_id: persona_request.request_id.clone(),
            identity_id: persona_request.agent_id.clone(),
            interpreter_request_id: "persona:concurrent-with-hands:interpreter".into(),
            created_at: "2026-08-17T00:00:14.600Z".into(),
            effects: vec![crate::PersonaInterpreterEffect::StateNote {
                memory_kind: "social_read".into(),
                subject_id: Some("operator".into()),
                summary: "Persona and Hands own disjoint consequences.".into(),
                confidence: Some(0.9),
            }],
            private_state_exposed: false,
            decision_context_id: persona_context.context_id,
        };
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let (hands_outcome, persona_outcome) = std::thread::scope(|scope| {
            let hands_barrier = barrier.clone();
            let hands_store = store.clone();
            let hands_commit = hands_commit.clone();
            let hands = scope.spawn(move || {
                hands_barrier.wait();
                crate::put_hands_commit_receipt(&hands_store, &hands_commit)
            });
            let persona_barrier = barrier.clone();
            let persona_store = store.clone();
            let persona_cultmesh = persona_cultmesh.clone();
            let persona = scope.spawn(move || {
                persona_barrier.wait();
                crate::persona_conversation::admit_persona_state_notes(
                    &persona_store,
                    &persona_cultmesh,
                    "runtime",
                    &persona_request,
                    &persona_effect,
                )
            });
            (
                hands.join().expect("concurrent Hands consequence"),
                persona.join().expect("concurrent Persona Mind admission"),
            )
        });
        hands_outcome?;
        assert_eq!(persona_outcome?, ("admitted".into(), Vec::new()));

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
                .attempt
                .action,
            EpiphanyAgentPassContinuationAction::Launch
        );
        let verification_job_id =
            launch_current_frontier_verification_work(&store, "2026-08-17T00:00:14.700Z")?;
        accepted_cache.pull_all_backing_stores()?;
        let verification_launch = accepted_cache
            .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&verification_job_id)?
            .expect("exact Verification launch");
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
        crate::open_runtime_model_execution(
            &store,
            crate::RuntimeSpineJobOptions {
                job_id: "verification-model-job".into(),
                session_id: "verification-model-session".into(),
            },
            &verification_native,
        )?;
        let verification_context = crate::EpiphanyDecisionContext::new(
            &verification_basis,
            verification_native,
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
            research_decision_msgpack: None,
            item_error: None,
            metadata: Default::default(),
            repo_model_mutation_proposal_msgpack: None,
            frontier_plan_candidate_msgpack: None,
            frontier_plan_mind_decision_msgpack: None,
            imagination_consideration_candidate_msgpack: None,
            admitted_model_direction_consideration_result_msgpack: None,
            decision_context_id: verification_context.context_id.clone(),
        };
        crate::put_runtime_role_worker_result(&store, &verification_result)?;
        assert_eq!(
            project_current_work(&store)?
                .verification
                .expect("terminal Verification work awaits admission")
                .attempt
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
        let conflict_model_before = crate::assemble_repo_model_view(&verification_conflict_store)?;
        let conflict_outcome = accept_frontier_verification_result(
            &verification_conflict_store,
            &verification_job_id,
            "2026-08-17T00:00:16Z",
        )?;
        assert!(matches!(
            conflict_outcome,
            EpiphanyAgentPassAdmissionOutcome::Refused { .. }
        ));
        assert_eq!(
            crate::assemble_repo_model_view(&verification_conflict_store)?,
            conflict_model_before
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
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let (modeling_outcome, verification_commit) = std::thread::scope(|scope| {
            let modeling_barrier = barrier.clone();
            let modeling_store = store.clone();
            let modeling = scope.spawn(move || {
                modeling_barrier.wait();
                crate::commit_typed_organ_mind_mutation(
                    &modeling_store,
                    "Modeling",
                    hands_provenance,
                    "Modeling.verification_concurrent_fixture",
                    concurrent_plan.strong_reads,
                    concurrent_plan.writes,
                    "2026-08-17T00:00:15.100Z",
                )
            });
            let verification_barrier = barrier.clone();
            let verification_store = store.clone();
            let verification_job_id = verification_job_id.clone();
            let verification = scope.spawn(move || {
                verification_barrier.wait();
                accept_frontier_verification_result(
                    &verification_store,
                    &verification_job_id,
                    "2026-08-17T00:00:16Z",
                )
            });
            (
                modeling.join().expect("concurrent Modeling commit"),
                verification.join().expect("concurrent Verification commit"),
            )
        });
        assert!(matches!(
            modeling_outcome?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));
        let EpiphanyAgentPassAdmissionOutcome::Committed(verification_commit) =
            verification_commit?
        else {
            panic!("disjoint Verification admission must commit")
        };
        assert_eq!(verification_commit.invariant_owner, "Soul.verification");
        assert!(project_current_work(&store)?.verification.is_none());
        let EpiphanyAgentPassAdmissionOutcome::Committed(replayed_verification_commit) =
            accept_frontier_verification_result(
                &store,
                &verification_job_id,
                "2026-08-17T00:00:16Z",
            )?
        else {
            panic!("accepted Verification replay must remain committed")
        };
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
                .attempt
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
        let verdict_context =
            crate::EpiphanyDecisionContext::new(&verdict_basis, verdict_native, Vec::new())?;
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
            research_decision_msgpack: None,
            item_error: None,
            metadata: Default::default(),
            repo_model_mutation_proposal_msgpack: Some(rmp_serde::to_vec_named(&verdict_mutation)?),
            frontier_plan_candidate_msgpack: None,
            frontier_plan_mind_decision_msgpack: None,
            imagination_consideration_candidate_msgpack: None,
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
        assert_eq!(
            project_current_work(&store)?
                .frontier_verdict_modeling
                .expect("terminal verdict work awaits admission")
                .attempt
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
        let conflict_model_before = crate::assemble_repo_model_view(&conflict_store)?;
        assert!(matches!(
            accept_frontier_verdict_modeling_result(
                &conflict_store,
                &verdict_job_id,
                "2026-08-17T00:00:21Z",
            )?,
            EpiphanyAgentPassAdmissionOutcome::Refused { .. }
        ));
        assert_eq!(
            crate::assemble_repo_model_view(&conflict_store)?,
            conflict_model_before,
            "refusal must not partially apply a stale RepoModel mutation"
        );

        let EpiphanyAgentPassAdmissionOutcome::Committed(verdict_commit) =
            accept_frontier_verdict_modeling_result(
                &store,
                &verdict_job_id,
                "2026-08-17T00:00:21Z",
            )?
        else {
            panic!("current frontier verdict must commit")
        };
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

        // Planning is one frontier-owned two-pass current-work family. The
        // complete RepoModel is audit cargo; exact frontier/dependency and
        // per-claim obligation documents own admission conflicts.
        let planning_frontier = RepoFrontierItem {
            id: "planning-frontier".into(),
            migration_body: "epiphany".into(),
            question: "How should the keyed Mind cut continue?".into(),
            gap: "The exact frontier needs one bounded plan.".into(),
            target_claim_ids: vec!["body-node".into()],
            repository_scope: vec!["epiphany-core/src".into()],
            recommended_next_organ: "Imagination".into(),
            status: RepoFrontierStatus::Active,
            evidence_refs: vec![verdict.receipt_id.clone()],
            ..Default::default()
        };
        let planning_proposal = crate::EpiphanyRepoModelMutationProposal::new(
            "repo-model-mutation-proposal-planning-frontier",
            "planning-frontier-fixture-request",
            "planning-frontier-fixture-result",
            vec![verdict.receipt_id.clone()],
            body.clone(),
            vec![crate::EpiphanyRepoModelMutationOperation::PutFrontier {
                item: planning_frontier.clone(),
            }],
        )?;
        let planning_plan = crate::plan_repo_model_mutation(&store, &planning_proposal)?;
        let mut planning_cache = crate::runtime_spine_cache(&store)?;
        planning_cache.pull_all_backing_stores()?;
        let planning_provenance = planning_cache
            .get_envelope::<crate::SoulVerdictReceipt>(&verdict.receipt_id)?
            .expect("typed planning provenance");
        assert!(matches!(
            crate::commit_typed_organ_mind_mutation(
                &store,
                "Modeling",
                planning_provenance,
                "Modeling.planning_obligation_fixture",
                planning_plan.strong_reads,
                planning_plan.writes,
                "2026-08-17T00:00:21.100Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));
        let planning_request = crate::select_and_commit_repo_frontier_planning_request(
            &store,
            "2026-08-17T00:00:21.200Z",
        )?;
        assert_eq!(planning_request.frontier_item_id, planning_frontier.id);
        assert_eq!(planning_request.claim_obligation_documents.len(), 1);
        assert!(
            planning_request.frontier_authority_documents.len()
                < planning_request.model_source_documents.len()
        );
        assert!(
            planning_cache
                .snapshot_envelopes()
                .iter()
                .all(|envelope| envelope.r#type != "epiphany.thread_state")
        );

        let mut disjoint_planning_node = final_model
            .nodes
            .iter()
            .find(|node| node.id == "body-node")
            .cloned()
            .expect("seeded Body node");
        disjoint_planning_node.id = "planning-concurrent-node".into();
        disjoint_planning_node.title = "Planning concurrent node".into();
        disjoint_planning_node.claim = "Unrelated keyed work merges with planning".into();
        let disjoint_planning_proposal = crate::EpiphanyRepoModelMutationProposal::new(
            "repo-model-mutation-proposal-planning-concurrent",
            "planning-concurrent-request",
            "planning-concurrent-result",
            vec![verdict.receipt_id.clone()],
            body.clone(),
            vec![crate::EpiphanyRepoModelMutationOperation::PutNode {
                node: disjoint_planning_node,
            }],
        )?;
        let disjoint_planning_plan =
            crate::plan_repo_model_mutation(&store, &disjoint_planning_proposal)?;
        planning_cache.pull_all_backing_stores()?;
        let disjoint_planning_provenance = planning_cache
            .get_envelope::<crate::SoulVerdictReceipt>(&verdict.receipt_id)?
            .expect("typed disjoint planning provenance");
        assert!(matches!(
            crate::commit_typed_organ_mind_mutation(
                &store,
                "Modeling",
                disjoint_planning_provenance,
                "Modeling.concurrent_with_planning_fixture",
                disjoint_planning_plan.strong_reads,
                disjoint_planning_plan.writes,
                "2026-08-17T00:00:21.600Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));
        let mut current_planning_cache = crate::runtime_spine_cache(&store)?;
        current_planning_cache.pull_all_backing_stores()?;
        crate::runtime_spine::validate_actionable_repo_frontier_planning_request(
            &current_planning_cache,
            &planning_request,
        )?;
        let planning_job =
            launch_current_frontier_planning_work(&store, "2026-08-17T00:00:21.700Z")?;
        let mut planning_cache = crate::runtime_spine_cache(&store)?;
        planning_cache.pull_all_backing_stores()?;
        let planning_launch = planning_cache
            .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&planning_job)?
            .expect("exact Planning launch");
        let crate::EpiphanyWorkerLaunchDocument::Role(planning_document) =
            planning_launch.launch_document()?
        else {
            panic!("Planning must launch as a role pass")
        };
        assert_eq!(
            planning_document
                .frontier_planning_context
                .as_ref()
                .map(|context| context.request_id.as_str()),
            Some(planning_request.request_id.as_str())
        );
        let planning_basis = crate::worker_reasoning_basis(&store, &planning_launch)?;
        crate::put_reasoning_basis(&store, &planning_basis)?;
        let mut planning_native = epiphany_model_adapter::EpiphanyModelRequest::new(
            "planning-model-request",
            "planning-conversation",
            "openai-codex",
            "gpt-test",
            "imagine",
        );
        planning_native.reasoning_basis_id = Some(planning_basis.basis_id.clone());
        planning_native.source_worker_job_id = Some(planning_job.clone());
        planning_native.output_contract_id = Some(planning_launch.output_contract_id.clone());
        planning_native.output_schema_json = Some(serde_json::to_string(
            &crate::epiphany_frontier_planning_output_schema(),
        )?);
        crate::open_runtime_model_execution(
            &store,
            crate::RuntimeSpineJobOptions {
                job_id: "planning-model-job".into(),
                session_id: "planning-model-session".into(),
            },
            &planning_native,
        )?;
        let planning_context =
            crate::EpiphanyDecisionContext::new(&planning_basis, planning_native, Vec::new())?;
        crate::put_decision_context(&store, &planning_context)?;
        let mut candidate = crate::RepoFrontierPlanCandidate {
            candidate_id: "pending".into(),
            planning_request_id: planning_request.request_id.clone(),
            model_projection_digest: planning_request.model_projection_digest.clone(),
            model_source_documents: planning_request.model_source_documents.clone(),
            frontier_item_id: planning_request.frontier_item_id.clone(),
            frontier_item_hash: planning_request.frontier_item_hash.clone(),
            safe_paths: planning_request.repository_scope.clone(),
            action: "Replace aggregate planning authority with keyed current work.".into(),
            command: "cargo test --manifest-path epiphany-core/Cargo.toml --lib".into(),
            checks: vec!["planning lifecycle is thread-free".into()],
            stop_conditions: vec!["exact frontier authority changes".into()],
            rollback_steps: vec!["revert the keyed Planning commit".into()],
            commit_message: "Migrate Planning to keyed Mind authority".into(),
            proposed_at: "2026-08-17T00:00:21.800Z".into(),
        };
        candidate.candidate_id = crate::canonical_repo_frontier_plan_candidate_id(&candidate)?;
        let planning_result = crate::EpiphanyRuntimeRoleWorkerResult {
            result_id: "planning-result".into(),
            job_id: planning_job.clone(),
            role_id: "imagination".into(),
            verdict: "candidate-ready".into(),
            summary: "proposed one exact bounded plan".into(),
            next_safe_move: "request Mind review".into(),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: Vec::new(),
            frontier_node_ids: vec!["body-node".into()],
            evidence_ids: vec![verdict.receipt_id.clone()],
            artifact_refs: Vec::new(),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            research_decision_msgpack: None,
            item_error: None,
            metadata: Default::default(),
            repo_model_mutation_proposal_msgpack: None,
            frontier_plan_candidate_msgpack: Some(rmp_serde::to_vec_named(&candidate)?),
            frontier_plan_mind_decision_msgpack: None,
            imagination_consideration_candidate_msgpack: None,
            admitted_model_direction_consideration_result_msgpack: None,
            decision_context_id: planning_context.context_id.clone(),
        };
        let planning_process = crate::ProcessInstanceIdentity {
            process_id: 46,
            creation_token: 11,
            created_at_rfc3339: Some("2026-08-17T00:00:21.900Z".into()),
            executable_path: "planning-worker".into(),
        };
        let planning_activation = "planning-activation";
        crate::claim_runtime_worker_process(
            &store,
            &planning_job,
            &planning_process,
            &format!("{:x}", Sha256::digest(planning_activation.as_bytes())),
            "2026-08-17T00:00:21.900Z",
        )?;
        crate::activate_runtime_worker_process(
            &store,
            &planning_job,
            &planning_process,
            planning_activation,
            "2026-08-17T00:00:21.950Z",
        )?;
        crate::put_runtime_role_worker_result(&store, &planning_result)?;
        let mind_request = crate::commit_repo_frontier_plan_mind_request(
            &store,
            &planning_result.result_id,
            "2026-08-17T00:00:22.100Z",
        )?;
        let mind_job = launch_current_frontier_plan_mind_work(&store, "2026-08-17T00:00:22.200Z")?;
        planning_cache.pull_all_backing_stores()?;
        let mind_launch = planning_cache
            .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&mind_job)?
            .expect("exact plan Mind launch");
        let crate::EpiphanyWorkerLaunchDocument::Role(mind_document) =
            mind_launch.launch_document()?
        else {
            panic!("plan Mind must launch as a role pass")
        };
        assert_eq!(
            mind_document
                .frontier_plan_mind_context
                .as_ref()
                .map(|context| context.request.request_id.as_str()),
            Some(mind_request.request_id.as_str())
        );
        let mind_basis = crate::worker_reasoning_basis(&store, &mind_launch)?;
        crate::put_reasoning_basis(&store, &mind_basis)?;
        let mut mind_native = epiphany_model_adapter::EpiphanyModelRequest::new(
            "plan-mind-model-request",
            "plan-mind-conversation",
            "openai-codex",
            "gpt-test",
            "judge",
        );
        mind_native.reasoning_basis_id = Some(mind_basis.basis_id.clone());
        mind_native.source_worker_job_id = Some(mind_job.clone());
        mind_native.output_contract_id = Some(mind_launch.output_contract_id.clone());
        mind_native.output_schema_json = Some(serde_json::to_string(
            &crate::epiphany_frontier_plan_mind_output_schema(),
        )?);
        crate::open_runtime_model_execution(
            &store,
            crate::RuntimeSpineJobOptions {
                job_id: "plan-mind-model-job".into(),
                session_id: "plan-mind-model-session".into(),
            },
            &mind_native,
        )?;
        let mind_context =
            crate::EpiphanyDecisionContext::new(&mind_basis, mind_native, Vec::new())?;
        crate::put_decision_context(&store, &mind_context)?;
        let mind_decision = crate::RepoFrontierPlanMindDecision {
            mind_request_id: mind_request.request_id.clone(),
            planning_request_id: planning_request.request_id.clone(),
            imagination_result_id: planning_result.result_id.clone(),
            candidate_id: candidate.candidate_id.clone(),
            candidate_sha256: mind_request.candidate_sha256.clone(),
            decision: crate::RepoFrontierPlanDecision::Adopt,
            rationale: "The candidate is bounded by the exact frontier and claim guard.".into(),
            decided_at: "2026-08-17T00:00:22.300Z".into(),
        };
        let mind_result = crate::EpiphanyRuntimeRoleWorkerResult {
            result_id: "plan-mind-result".into(),
            job_id: mind_job.clone(),
            role_id: "mindAdmissionReview".into(),
            verdict: "adopt".into(),
            summary: "adopted the exact bounded plan".into(),
            next_safe_move: "commit plan decision".into(),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: Vec::new(),
            frontier_node_ids: vec!["body-node".into()],
            evidence_ids: vec![candidate.candidate_id.clone()],
            artifact_refs: Vec::new(),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            research_decision_msgpack: None,
            item_error: None,
            metadata: Default::default(),
            repo_model_mutation_proposal_msgpack: None,
            frontier_plan_candidate_msgpack: None,
            frontier_plan_mind_decision_msgpack: Some(rmp_serde::to_vec_named(&mind_decision)?),
            imagination_consideration_candidate_msgpack: None,
            admitted_model_direction_consideration_result_msgpack: None,
            decision_context_id: mind_context.context_id.clone(),
        };
        let mind_process = crate::ProcessInstanceIdentity {
            process_id: 47,
            creation_token: 12,
            created_at_rfc3339: Some("2026-08-17T00:00:22.400Z".into()),
            executable_path: "plan-mind-worker".into(),
        };
        let mind_activation = "plan-mind-activation";
        crate::claim_runtime_worker_process(
            &store,
            &mind_job,
            &mind_process,
            &format!("{:x}", Sha256::digest(mind_activation.as_bytes())),
            "2026-08-17T00:00:22.400Z",
        )?;
        crate::activate_runtime_worker_process(
            &store,
            &mind_job,
            &mind_process,
            mind_activation,
            "2026-08-17T00:00:22.450Z",
        )?;
        crate::put_runtime_role_worker_result(&store, &mind_result)?;
        let plan_decision =
            crate::commit_repo_frontier_plan_decision(&store, &mind_result.result_id)?;
        assert_eq!(
            plan_decision.decision,
            crate::RepoFrontierPlanDecision::Adopt
        );
        assert_eq!(
            crate::commit_repo_frontier_plan_decision(&store, &mind_result.result_id)?,
            plan_decision
        );
        let adopted_model = crate::assemble_repo_model_view(&store)?;
        let adopted_frontier = adopted_model
            .frontier
            .iter()
            .find(|item| item.id == planning_frontier.id)
            .expect("adopted Planning frontier");
        assert_eq!(
            adopted_frontier
                .adopted_plan
                .as_ref()
                .expect("Mind adoption")
                .candidate_id,
            candidate.candidate_id
        );
        assert!(
            adopted_model
                .nodes
                .iter()
                .any(|node| node.id == "planning-concurrent-node")
        );
        assert!(
            planning_cache
                .snapshot_envelopes()
                .iter()
                .all(|envelope| envelope.r#type != "epiphany.thread_state")
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
            repository_scope: vec!["epiphany-core/src".into()],
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
            project_current_work(&store)?.research.continuation_action(),
            Some(RepoFrontierResearchContinuationAction::LaunchResearch)
        );
        let research_job = launch_current_frontier_research_work(&store, "2026-08-17T00:00:23Z")?;
        let mut research_cache = crate::runtime_spine_cache(&store)?;
        research_cache.pull_all_backing_stores()?;
        assert!(
            research_cache
                .snapshot_envelopes()
                .iter()
                .all(|envelope| envelope.r#type != "epiphany.thread_state")
        );
        let research_launch = research_cache
            .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&research_job)?
            .expect("exact Research launch");
        let research_request = crate::runtime_spine::frontier_research_request_for_launch(
            &research_cache,
            &research_launch,
        )?
        .expect("exact Research request");
        assert_eq!(research_request.frontier_item_id, eyes_frontier.id);
        assert!(
            research_request.frontier_authority_documents.len()
                < research_request.model_source_documents.len()
        );
        assert_eq!(
            project_current_work(&store)?.research.continuation_action(),
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
        let research_context =
            crate::EpiphanyDecisionContext::new(&research_basis, research_native, Vec::new())?;
        research_cache.put(&research_context.context_id, &research_context)?;
        let stale_research_store = temp.path().join("stale-research-output.cc");
        std::fs::copy(&store, &stale_research_store)?;
        let evidence = crate::EpiphanyEvidenceRecord {
            id: "research-evidence-1".into(),
            kind: "external-source".into(),
            status: "ok".into(),
            summary: "The bounded external claim is supported.".into(),
            code_refs: vec![crate::EpiphanyCodeRef {
                path: "epiphany-core/src/current_work.rs".into(),
                start_line: Some(1),
                end_line: None,
                symbol: Some("accept_frontier_research_result".into()),
                note: None,
            }],
        };
        let observation = crate::EpiphanyObservation {
            id: "research-observation-1".into(),
            summary: "External evidence answers the exact frontier question.".into(),
            source_kind: "research".into(),
            status: "ok".into(),
            code_refs: Vec::new(),
            evidence_ids: vec![evidence.id.clone()],
        };
        let research_decision = crate::EpiphanyResearchDecision {
            evidence: vec![evidence.clone()],
            observations: vec![observation.clone()],
            investigation_checkpoint: None,
        };
        let research_result = crate::EpiphanyRuntimeRoleWorkerResult {
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
            research_decision_msgpack: Some(rmp_serde::to_vec_named(&research_decision)?),
            item_error: None,
            metadata: Default::default(),
            repo_model_mutation_proposal_msgpack: None,
            frontier_plan_candidate_msgpack: None,
            frontier_plan_mind_decision_msgpack: None,
            imagination_consideration_candidate_msgpack: None,
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
        assert_eq!(
            project_current_work(&store)?.research.continuation_action(),
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
        let after_research = project_current_work(&store)?;
        assert!(after_research.research.continuation_action().is_none());
        assert!(after_research.body_modeling.is_none());
        assert!(after_research.proposal_modeling.is_none());
        assert!(after_research.frontier_verdict_modeling.is_none());
        assert_current_work_reentry_is_read_only(&store, &after_research)?;
        let mut final_runtime = crate::runtime_spine_cache(&store)?;
        final_runtime.pull_all_backing_stores()?;
        assert_eq!(
            final_runtime
                .get_envelope::<crate::EpiphanyRuntimeIdentity>(crate::RUNTIME_IDENTITY_KEY)?
                .expect("runtime identity survives current-work launches"),
            initial_identity,
            "current-work launches must not mutate their shared runtime identity",
        );
        Ok(())
    }
}
