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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyCurrentWorkProjection {
    pub mind_projection_digest: String,
    pub body_modeling: Option<EpiphanyBodyModelingWorkProjection>,
    pub research_continuation_action: Option<RepoFrontierResearchContinuationAction>,
    pub frontier_planning_stage: RepoFrontierPlanningLifecycleStage,
    pub proposal_modeling_request: Option<RepoFrontierProposalModelingRequest>,
    pub hands_frontier_ready: bool,
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

pub fn project_current_work(
    store_path: impl AsRef<Path>,
) -> Result<EpiphanyCurrentWorkProjection> {
    let store_path = store_path.as_ref();
    let mind = crate::assemble_mind_view(store_path)?;
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
            let mut cache = crate::runtime_spine_cache(store_path)?;
            cache.pull_all_backing_stores()?;
            resolve_body_modeling_work(
                work.clone(),
                cache.get::<EpiphanyBodyModelingDecisionReceipt>(&work.work_id)?,
            )?
        }
        (None, None) | (Some(_), None) => None,
        (None, Some(_)) => {
            return Err(anyhow!(
                "current work has a RepoModel but no admitted repository Body observation"
            ));
        }
    };
    Ok(EpiphanyCurrentWorkProjection {
        mind_projection_digest: mind.projection_digest,
        body_modeling,
        research_continuation_action: crate::runtime_repo_frontier_research_lifecycle(store_path)?
            .continuation_action(),
        frontier_planning_stage: crate::runtime_repo_frontier_planning_lifecycle(store_path)?.stage,
        proposal_modeling_request:
            crate::runtime_pending_repo_frontier_proposal_modeling_request(store_path)?,
        hands_frontier_ready: crate::runtime_has_actionable_hands_frontier(store_path)?,
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
        EpiphanyMemoryProfile,
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
    fn body_decision_is_derived_from_the_sealed_role_projection() -> Result<()> {
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
            current_work.mind_projection_digest,
            crate::assemble_mind_view(&store)?.projection_digest
        );
        assert_eq!(crate::repository_body_read_counters(), (0, 0));
        let document =
            crate::EpiphanyWorkerLaunchDocument::Role(crate::EpiphanyRoleWorkerLaunchDocument {
                thread_id: "creation-thread".into(),
                role_id: "modeling".into(),
                state_revision: 99,
                objective: Some("obsolete aggregate objective".into()),
                dynamic_prompt_context: None,
                repository_body_observation_basis: Some(body.clone()),
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
        let launch = crate::EpiphanyRuntimeWorkerLaunchRequest {
            schema_version: crate::RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION.into(),
            job_id: "body-job".into(),
            binding_id: crate::EPIPHANY_MODELING_ROLE_BINDING_ID.into(),
            role: crate::EPIPHANY_MODELING_OWNER_ROLE.into(),
            authority_scope: "epiphany.role.modeling".into(),
            instruction: "Model the Body".into(),
            output_contract_id: crate::ROLE_WORKER_OUTPUT_CONTRACT_ID.into(),
            document_kind: "role".into(),
            launch_document_msgpack: rmp_serde::to_vec_named(&document)?,
            metadata: Default::default(),
            organ_launch_contract: crate::default_launch_organ_contract(
                "epiphany.role.modeling",
                "role",
                crate::ROLE_WORKER_OUTPUT_CONTRACT_ID,
            ),
            proposal_modeling_request_id: None,
            claim_repair_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            repo_frontier_verdict_modeling_authority_msgpack: None,
        };
        let mut cache = crate::runtime_spine_cache(&store)?;
        cache.put(&launch.job_id, &launch)?;
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
        let envelope = body_modeling_decision_envelope(
            &store,
            &result,
            "checkpoint-ready",
            "2026-08-17T00:00:02Z",
        )?;
        crate::mind_documents::validate_mind_write_envelope(&envelope)?;
        let receipt: EpiphanyBodyModelingDecisionReceipt =
            rmp_serde::from_slice(&envelope.payload)?;
        assert_eq!(receipt.body_basis, body);
        assert_eq!(
            receipt.repo_model_projection_digest,
            crate::assemble_repo_model_view(&store)?.projection_digest
        );
        let mut cache = crate::runtime_spine_cache(&store)?;
        cache.put(&receipt.work_id, &receipt)?;
        assert!(project_current_work(&store)?.body_modeling.is_none());
        assert_eq!(crate::repository_body_read_counters(), (0, 0));
        Ok(())
    }
}
