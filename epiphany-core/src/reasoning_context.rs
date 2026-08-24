use crate::runtime_store_backend::runtime_spine_backing_store;
use crate::{
    EpiphanyRoleWorkerLaunchDocument, EpiphanyRuntimeWorkerLaunchRequest,
    EpiphanyWorkerLaunchDocument, PersonaInterpreterInput, PersonaProjectorInput, PersonaTurnInput,
    runtime_spine_cache,
};
use anyhow::{Result, anyhow};
use cultcache_rs::{CacheBackingStore, CultCache, CultCacheEnvelope, DatabaseEntry};
use epiphany_model_adapter::EpiphanyModelRequest;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
use epiphany_tool_adapter::{
    EpiphanyToolInvocationIntent, EpiphanyToolInvocationReceipt, receipt_output_for_model,
    tool_invocation_intent_key, tool_invocation_receipt_key,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::Path;

pub const REASONING_BASIS_SCHEMA_VERSION: &str = "epiphany.reasoning_basis.v1";
pub const DECISION_CONTEXT_SCHEMA_VERSION: &str = "epiphany.decision_context.v2";
pub const MODEL_PASS_FAILURE_SCHEMA_VERSION: &str = "epiphany.model_pass_failure.v1";
pub const MIND_COMMIT_RECEIPT_SCHEMA_VERSION: &str = "epiphany.mind_commit_receipt.v1";
pub const DECISION_AUDIT_PROJECTION_SCHEMA_VERSION: &str = "epiphany.decision_audit_projection.v1";
pub const DECISION_AUDIT_INDEX_SCHEMA_VERSION: &str = "epiphany.decision_audit_index.v1";
pub const WORKER_REASONING_PROJECTION_POLICY: &str =
    "epiphany.reasoning_projection.worker_launch.v3";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EpiphanyRolePassAuthorityProjection {
    pub creation_thread_id: String,
    pub role_id: String,
    pub dynamic_prompt_context: Option<String>,
    pub repository_body_observation_basis: Option<crate::RepositoryBodyObservationBasis>,
    pub proposal_modeling_context: Option<crate::RepoFrontierProposalModelingContextProjection>,
    pub frontier_verdict_modeling_context:
        Option<crate::RepoFrontierVerdictModelingLaunchAuthority>,
    pub frontier_planning_context: Option<crate::RepoFrontierPlanningContextProjection>,
    pub frontier_research_context: Option<crate::RepoFrontierResearchContextProjection>,
    pub frontier_verification_context: Option<crate::RepoFrontierVerificationContextProjection>,
    pub frontier_plan_mind_context: Option<crate::RepoFrontierPlanMindContextProjection>,
    pub imagination_consideration_context: Option<crate::ImaginationConsiderationContextProjection>,
    pub admitted_model_direction_consideration_context:
        Option<crate::AdmittedModelDirectionConsiderationContextProjection>,
}

impl From<EpiphanyRoleWorkerLaunchDocument> for EpiphanyRolePassAuthorityProjection {
    fn from(value: EpiphanyRoleWorkerLaunchDocument) -> Self {
        Self {
            creation_thread_id: value.thread_id,
            role_id: value.role_id,
            dynamic_prompt_context: value.dynamic_prompt_context,
            repository_body_observation_basis: value.repository_body_observation_basis,
            proposal_modeling_context: value.proposal_modeling_context,
            frontier_verdict_modeling_context: value.frontier_verdict_modeling_context,
            frontier_planning_context: value.frontier_planning_context,
            frontier_research_context: value.frontier_research_context,
            frontier_verification_context: value.frontier_verification_context,
            frontier_plan_mind_context: value.frontier_plan_mind_context,
            imagination_consideration_context: value.imagination_consideration_context,
            admitted_model_direction_consideration_context: value
                .admitted_model_direction_consideration_context,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyRepoModelPromptProjection {
    pub identity: crate::EpiphanyRepoModelIdentityDocument,
    pub projection_digest: String,
    pub domains: Vec<epiphany_state_model::EpiphanyMemoryDomain>,
    pub nodes: Vec<epiphany_state_model::EpiphanyMemoryNode>,
    pub edges: Vec<epiphany_state_model::EpiphanyMemoryEdge>,
    pub frontier: Vec<epiphany_state_model::RepoFrontierItem>,
    pub claim_obligations: Vec<crate::EpiphanyRepoModelClaimObligationsDocument>,
    pub surface_offers: Vec<crate::AtlasSurfaceOffer>,
    pub dependency_claims: Vec<crate::AtlasDependencyClaim>,
    pub dependency_verifications: Vec<crate::AtlasDependencyVerification>,
    pub dependency_impacts: Vec<crate::AtlasDependencyImpact>,
}

impl From<crate::EpiphanyRepoModelView> for EpiphanyRepoModelPromptProjection {
    fn from(value: crate::EpiphanyRepoModelView) -> Self {
        Self {
            identity: value.identity,
            projection_digest: value.projection_digest,
            domains: value.domains,
            nodes: value.nodes,
            edges: value.edges,
            frontier: value.frontier,
            claim_obligations: value.claim_obligations,
            surface_offers: value.surface_offers,
            dependency_claims: value.dependency_claims,
            dependency_verifications: value.dependency_verifications,
            dependency_impacts: value.dependency_impacts,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EpiphanyMindPromptProjection {
    pub schema_epoch: String,
    pub runtime_id: String,
    pub projection_digest: String,
    pub objective: Option<String>,
    pub active_subgoal_id: Option<String>,
    pub subgoals: Vec<epiphany_state_model::EpiphanySubgoal>,
    pub invariants: Vec<epiphany_state_model::EpiphanyInvariant>,
    pub observations: Vec<epiphany_state_model::EpiphanyObservation>,
    pub evidence: Vec<epiphany_state_model::EpiphanyEvidenceRecord>,
    pub verification_audits: Vec<crate::EpiphanyMindVerificationAuditDocument>,
    pub reorientation_decisions: Vec<crate::EpiphanyMindReorientationDecisionDocument>,
    pub reorientation_failures: Vec<crate::EpiphanyMindReorientationPassFailureDocument>,
    pub agent_pass_admission_refusals: Vec<crate::EpiphanyAgentPassAdmissionRefusal>,
    pub investigation_checkpoint: Option<epiphany_state_model::EpiphanyInvestigationCheckpoint>,
    pub mode: Option<epiphany_state_model::EpiphanyModeState>,
    pub repository_body_observation: Option<crate::RepositoryBodyObservationBasis>,
    pub repo_model: Option<EpiphanyRepoModelPromptProjection>,
}

impl From<crate::EpiphanyMindView> for EpiphanyMindPromptProjection {
    fn from(value: crate::EpiphanyMindView) -> Self {
        Self {
            schema_epoch: value.schema_epoch,
            runtime_id: value.runtime_id,
            projection_digest: value.projection_digest,
            objective: value.objective,
            active_subgoal_id: value.active_subgoal_id,
            subgoals: value.subgoals,
            invariants: value.invariants,
            observations: value.observations,
            evidence: value.evidence,
            verification_audits: value.verification_audits,
            reorientation_decisions: value.reorientation_decisions,
            reorientation_failures: value.reorientation_failures,
            agent_pass_admission_refusals: value.agent_pass_admission_refusals,
            investigation_checkpoint: value.investigation_checkpoint,
            mode: value.mode,
            repository_body_observation: value.repository_body_observation,
            repo_model: value.repo_model.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EpiphanyRoleReasoningProjection {
    pub authority: EpiphanyRolePassAuthorityProjection,
    pub mind: EpiphanyMindPromptProjection,
    pub modeling_body: Option<crate::RepositoryBodyReasoningProjection>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EpiphanyReorientationReasoningProjection {
    pub authority: crate::EpiphanyReorientWorkerLaunchDocument,
    pub request: crate::EpiphanyReorientationRequest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyMindDocumentVersion {
    pub store_id: String,
    pub document_type: String,
    pub document_key: String,
    pub schema_id: Option<String>,
    pub payload_msgpack: Vec<u8>,
    pub payload_sha256: String,
}

impl EpiphanyMindDocumentVersion {
    pub fn from_envelope(store_id: &str, envelope: &CultCacheEnvelope) -> Result<Self> {
        require_non_empty(store_id, "Mind document store id")?;
        require_non_empty(&envelope.r#type, "Mind document type")?;
        require_non_empty(&envelope.key, "Mind document key")?;
        let payload_sha256 = sha256(&envelope.payload);
        Ok(Self {
            store_id: store_id.to_string(),
            document_type: envelope.r#type.clone(),
            document_key: envelope.key.clone(),
            schema_id: envelope.schema_id.clone(),
            payload_msgpack: envelope.payload.clone(),
            payload_sha256,
        })
    }

    pub fn validate(&self) -> Result<()> {
        require_non_empty(&self.store_id, "Mind document store id")?;
        require_non_empty(&self.document_type, "Mind document type")?;
        require_non_empty(&self.document_key, "Mind document key")?;
        if self.payload_sha256 != sha256(&self.payload_msgpack) {
            return Err(anyhow!(
                "Mind document {:?}/{:?} payload digest mismatch",
                self.document_type,
                self.document_key
            ));
        }
        Ok(())
    }

    pub fn identity(&self) -> (&str, &str) {
        (&self.document_type, &self.document_key)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EpiphanyReasoningProjection {
    RolePass(EpiphanyRoleReasoningProjection),
    ReorientLaunch(EpiphanyReorientationReasoningProjection),
    PersonaProjector(PersonaProjectorInput),
    PersonaTurn(PersonaTurnInput),
    PersonaInterpreter(PersonaInterpreterInput),
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.reasoning_basis.v1",
    schema = "EpiphanyReasoningBasis"
)]
pub struct EpiphanyReasoningBasis {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub basis_id: String,
    #[cultcache(key = 2)]
    pub pass_id: String,
    #[cultcache(key = 3)]
    pub organ_id: String,
    #[cultcache(key = 4)]
    pub projection_policy_id: String,
    #[cultcache(key = 5)]
    pub source_documents: Vec<EpiphanyMindDocumentVersion>,
    #[cultcache(key = 6)]
    pub projection_msgpack: Vec<u8>,
    #[cultcache(key = 7, default)]
    pub predecessor_decision_context_ids: Vec<String>,
}

impl EpiphanyReasoningBasis {
    pub fn new(
        pass_id: impl Into<String>,
        organ_id: impl Into<String>,
        projection_policy_id: impl Into<String>,
        mut source_documents: Vec<EpiphanyMindDocumentVersion>,
        projection: EpiphanyReasoningProjection,
    ) -> Result<Self> {
        let pass_id = pass_id.into();
        let organ_id = organ_id.into();
        let projection_policy_id = projection_policy_id.into();
        require_non_empty(&pass_id, "reasoning pass id")?;
        require_non_empty(&organ_id, "reasoning organ id")?;
        require_non_empty(&projection_policy_id, "reasoning projection policy id")?;
        canonicalize_source_documents(&mut source_documents)?;
        let projection_msgpack = rmp_serde::to_vec_named(&projection)?;
        let mut basis = Self {
            schema_version: REASONING_BASIS_SCHEMA_VERSION.to_string(),
            basis_id: String::new(),
            pass_id,
            organ_id,
            projection_policy_id,
            source_documents,
            projection_msgpack,
            predecessor_decision_context_ids: Vec::new(),
        };
        basis.basis_id = format!("reasoning-basis-{}", digest_without_basis_id(&basis)?);
        basis.validate()?;
        Ok(basis)
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != REASONING_BASIS_SCHEMA_VERSION {
            return Err(anyhow!("unsupported reasoning basis schema"));
        }
        require_non_empty(&self.basis_id, "reasoning basis id")?;
        require_non_empty(&self.pass_id, "reasoning pass id")?;
        require_non_empty(&self.organ_id, "reasoning organ id")?;
        require_non_empty(&self.projection_policy_id, "reasoning projection policy id")?;
        let mut canonical = self.source_documents.clone();
        canonicalize_source_documents(&mut canonical)?;
        if canonical != self.source_documents {
            return Err(anyhow!(
                "reasoning basis source documents are not canonical"
            ));
        }
        let projection: EpiphanyReasoningProjection =
            rmp_serde::from_slice(&self.projection_msgpack)
                .map_err(|error| anyhow!("reasoning projection is invalid: {error}"))?;
        if let EpiphanyReasoningProjection::RolePass(role) = &projection {
            if role.authority.creation_thread_id.trim().is_empty()
                || role.authority.role_id.trim().is_empty()
                || self.source_documents.is_empty()
                || role.mind.projection_digest
                    != crate::mind_documents::mind_view_digest(&self.source_documents)?
            {
                return Err(anyhow!(
                    "role reasoning projection does not bind its exact keyed Mind sources"
                ));
            }
        }
        if self
            .predecessor_decision_context_ids
            .iter()
            .any(|context_id| context_id.trim().is_empty())
            || self
                .predecessor_decision_context_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(anyhow!(
                "reasoning basis predecessor contexts are not canonical"
            ));
        }
        let expected = format!("reasoning-basis-{}", digest_without_basis_id(self)?);
        if self.basis_id != expected {
            return Err(anyhow!("reasoning basis identity digest mismatch"));
        }
        Ok(())
    }

    pub fn projection(&self) -> Result<EpiphanyReasoningProjection> {
        rmp_serde::from_slice(&self.projection_msgpack)
            .map_err(|error| anyhow!("reasoning projection is invalid: {error}"))
    }

    pub fn with_predecessor_contexts(
        mut self,
        mut predecessor_decision_context_ids: Vec<String>,
    ) -> Result<Self> {
        predecessor_decision_context_ids.sort();
        predecessor_decision_context_ids.dedup();
        self.predecessor_decision_context_ids = predecessor_decision_context_ids;
        self.basis_id.clear();
        self.basis_id = format!("reasoning-basis-{}", digest_without_basis_id(&self)?);
        self.validate()?;
        Ok(self)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EpiphanyDecisionToolObservation {
    pub intent: EpiphanyToolInvocationIntent,
    pub receipt: EpiphanyToolInvocationReceipt,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.decision_context.v2",
    schema = "EpiphanyDecisionContext"
)]
pub struct EpiphanyDecisionContext {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub context_id: String,
    #[cultcache(key = 2)]
    pub basis_id: String,
    #[cultcache(key = 3)]
    pub terminal_request_id: String,
    #[cultcache(key = 4)]
    pub native_request_msgpack: Vec<u8>,
    #[cultcache(key = 5)]
    pub provider_request_msgpack: Vec<u8>,
    #[cultcache(key = 6)]
    pub tool_observations_msgpack: Vec<u8>,
}

impl EpiphanyDecisionContext {
    pub fn new(
        basis: &EpiphanyReasoningBasis,
        native_request: EpiphanyModelRequest,
        tool_observations: Vec<EpiphanyDecisionToolObservation>,
    ) -> Result<Self> {
        basis.validate()?;
        let provider_request = epiphany_openai_adapter::request_from_native(&native_request);
        validate_request_pair(basis, &native_request, &provider_request)?;
        validate_tool_observations(&native_request, &tool_observations)?;
        let mut context = Self {
            schema_version: DECISION_CONTEXT_SCHEMA_VERSION.to_string(),
            context_id: String::new(),
            basis_id: basis.basis_id.clone(),
            terminal_request_id: native_request.request_id.clone(),
            native_request_msgpack: rmp_serde::to_vec_named(&native_request)?,
            provider_request_msgpack: rmp_serde::to_vec_named(&provider_request)?,
            tool_observations_msgpack: rmp_serde::to_vec_named(&tool_observations)?,
        };
        context.context_id = format!("decision-context-{}", digest_without_context_id(&context)?);
        context.validate(basis)?;
        Ok(context)
    }

    pub fn validate(&self, basis: &EpiphanyReasoningBasis) -> Result<()> {
        if self.schema_version != DECISION_CONTEXT_SCHEMA_VERSION {
            return Err(anyhow!("unsupported decision context schema"));
        }
        let native_request = self.native_request()?;
        let provider_request = self.provider_request()?;
        let tool_observations = self.tool_observations()?;
        if self.basis_id != basis.basis_id || self.terminal_request_id != native_request.request_id
        {
            return Err(anyhow!("decision context ownership mismatch"));
        }
        validate_request_pair(basis, &native_request, &provider_request)?;
        validate_tool_observations(&native_request, &tool_observations)?;
        let expected = format!("decision-context-{}", digest_without_context_id(self)?);
        if self.context_id != expected {
            return Err(anyhow!("decision context identity digest mismatch"));
        }
        Ok(())
    }

    pub fn native_request(&self) -> Result<EpiphanyModelRequest> {
        rmp_serde::from_slice(&self.native_request_msgpack)
            .map_err(|error| anyhow!("decision native request is invalid: {error}"))
    }

    pub fn provider_request(&self) -> Result<EpiphanyOpenAiModelRequest> {
        rmp_serde::from_slice(&self.provider_request_msgpack)
            .map_err(|error| anyhow!("decision provider request is invalid: {error}"))
    }

    pub fn tool_observations(&self) -> Result<Vec<EpiphanyDecisionToolObservation>> {
        rmp_serde::from_slice(&self.tool_observations_msgpack)
            .map_err(|error| anyhow!("decision tool observations are invalid: {error}"))
    }
}

/// Durable terminal record for a model-backed pass that could not produce an
/// admissible structured decision. This is not a transcript or transport log:
/// it binds the failure to the same sealed request/context that the model
/// actually reasoned from.
#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.model_pass_failure.v1",
    schema = "EpiphanyModelPassFailure"
)]
pub struct EpiphanyModelPassFailure {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub failure_id: String,
    #[cultcache(key = 2)]
    pub pass_id: String,
    #[cultcache(key = 3)]
    pub organ_id: String,
    #[cultcache(key = 4)]
    pub model_request_id: String,
    #[cultcache(key = 5)]
    pub reasoning_basis_id: String,
    #[cultcache(key = 6)]
    pub decision_context_id: String,
    #[cultcache(key = 7)]
    pub runtime_session_id: String,
    #[cultcache(key = 8)]
    pub runtime_job_id: String,
    #[cultcache(key = 9)]
    pub failure_kind: String,
    #[cultcache(key = 10)]
    pub summary: String,
    #[cultcache(key = 11)]
    pub failed_at: String,
    #[cultcache(key = 12)]
    pub private_state_exposed: bool,
}

impl EpiphanyModelPassFailure {
    pub fn new(
        basis: &EpiphanyReasoningBasis,
        context: &EpiphanyDecisionContext,
        runtime_session_id: impl Into<String>,
        runtime_job_id: impl Into<String>,
        failure_kind: impl Into<String>,
        summary: impl Into<String>,
        failed_at: impl Into<String>,
    ) -> Result<Self> {
        context.validate(basis)?;
        let mut failure = Self {
            schema_version: MODEL_PASS_FAILURE_SCHEMA_VERSION.to_string(),
            failure_id: String::new(),
            pass_id: basis.pass_id.clone(),
            organ_id: basis.organ_id.clone(),
            model_request_id: context.terminal_request_id.clone(),
            reasoning_basis_id: basis.basis_id.clone(),
            decision_context_id: context.context_id.clone(),
            runtime_session_id: runtime_session_id.into(),
            runtime_job_id: runtime_job_id.into(),
            failure_kind: failure_kind.into(),
            summary: summary.into(),
            failed_at: failed_at.into(),
            private_state_exposed: false,
        };
        failure.failure_id = format!(
            "model-pass-failure-{}",
            sha256(failure.decision_context_id.as_bytes()).trim_start_matches("sha256:")
        );
        failure.validate(basis, context)?;
        Ok(failure)
    }

    pub fn validate(
        &self,
        basis: &EpiphanyReasoningBasis,
        context: &EpiphanyDecisionContext,
    ) -> Result<()> {
        if self.schema_version != MODEL_PASS_FAILURE_SCHEMA_VERSION || self.private_state_exposed {
            return Err(anyhow!(
                "model pass failure schema or privacy marker is invalid"
            ));
        }
        context.validate(basis)?;
        for (value, label) in [
            (&self.failure_id, "failure id"),
            (&self.pass_id, "pass id"),
            (&self.organ_id, "organ id"),
            (&self.model_request_id, "model request id"),
            (&self.reasoning_basis_id, "reasoning basis id"),
            (&self.decision_context_id, "decision context id"),
            (&self.runtime_session_id, "runtime session id"),
            (&self.runtime_job_id, "runtime job id"),
            (&self.failure_kind, "failure kind"),
            (&self.summary, "failure summary"),
            (&self.failed_at, "failure time"),
        ] {
            require_non_empty(value, label)?;
        }
        chrono::DateTime::parse_from_rfc3339(&self.failed_at)
            .map_err(|error| anyhow!("model pass failure time is invalid: {error}"))?;
        let expected_id = format!(
            "model-pass-failure-{}",
            sha256(context.context_id.as_bytes()).trim_start_matches("sha256:")
        );
        if self.failure_id != expected_id
            || self.pass_id != basis.pass_id
            || self.organ_id != basis.organ_id
            || self.model_request_id != context.terminal_request_id
            || self.reasoning_basis_id != basis.basis_id
            || self.decision_context_id != context.context_id
        {
            return Err(anyhow!("model pass failure ownership mismatch"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind_commit_receipt.v1",
    schema = "EpiphanyMindCommitReceipt"
)]
pub struct EpiphanyMindCommitReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub authority: EpiphanyMindCommitAuthority,
    #[cultcache(key = 3)]
    pub invariant_owner: String,
    #[cultcache(key = 4)]
    pub strong_reads: Vec<EpiphanyMindDocumentVersion>,
    #[cultcache(key = 5)]
    pub writes: Vec<EpiphanyMindDocumentVersion>,
    #[cultcache(key = 6)]
    pub committed_at: String,
}

impl EpiphanyMindCommitReceipt {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != MIND_COMMIT_RECEIPT_SCHEMA_VERSION {
            return Err(anyhow!("Mind commit receipt schema is unsupported"));
        }
        require_non_empty(&self.invariant_owner, "Mind commit invariant owner")?;
        chrono::DateTime::parse_from_rfc3339(&self.committed_at)
            .map_err(|error| anyhow!("Mind commit time is invalid: {error}"))?;
        if self.writes.is_empty() {
            return Err(anyhow!("Mind commit receipt has no writes"));
        }
        match &self.authority {
            EpiphanyMindCommitAuthority::ModelDecisionContext {
                decision_context_id,
            } => require_non_empty(decision_context_id, "Mind decision context")?,
            EpiphanyMindCommitAuthority::OperatorProvenance { provenance } => {
                provenance.validate()?
            }
            EpiphanyMindCommitAuthority::TypedOrganProvenance { organ, provenance } => {
                require_non_empty(organ, "Mind provenance organ")?;
                provenance.validate()?;
            }
        }
        validate_mind_document_versions(&self.strong_reads, "strong read")?;
        validate_mind_document_versions(&self.writes, "write")?;
        if self.receipt_id
            != mind_commit_receipt_id(
                &self.authority,
                &self.invariant_owner,
                &self.strong_reads,
                &self.writes,
            )?
        {
            return Err(anyhow!("Mind commit receipt identity digest mismatch"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpiphanyMindCommitAuthority {
    ModelDecisionContext {
        decision_context_id: String,
    },
    OperatorProvenance {
        provenance: EpiphanyMindDocumentVersion,
    },
    TypedOrganProvenance {
        organ: String,
        provenance: EpiphanyMindDocumentVersion,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EpiphanyMindCommitOutcome {
    Committed(EpiphanyMindCommitReceipt),
    Conflict {
        document_identities: Vec<(String, String)>,
    },
}

/// Read-only reconstruction of the durable records that make one model
/// decision auditable. This is an operator projection, never a stored owner.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyDecisionAuditProjection {
    pub schema_version: String,
    pub context_id: String,
    pub reasoning_basis: EpiphanyReasoningBasis,
    pub decision_context: EpiphanyDecisionContext,
    pub reasoning_projection: EpiphanyReasoningProjection,
    pub terminal_native_request: EpiphanyModelRequest,
    pub terminal_provider_request: EpiphanyOpenAiModelRequest,
    pub tool_observations: Vec<EpiphanyDecisionToolObservation>,
    pub terminal_records: EpiphanyDecisionTerminalRecords,
    pub mind_commit_receipts: Vec<EpiphanyMindCommitReceipt>,
    pub transcript_required: bool,
}

/// Read-only, deterministic discovery surface for decisions that already have
/// a complete durable audit chain. This is not persisted state and cannot
/// create, complete, or admit a decision.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyDecisionAuditIndexProjection {
    pub schema_version: String,
    pub decisions: Vec<EpiphanyDecisionAuditIndexEntry>,
    pub transcript_required: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyDecisionAuditIndexEntry {
    pub context_id: String,
    pub basis_id: String,
    pub pass_id: String,
    pub organ_id: String,
    pub terminal_request_id: String,
    pub terminal_record_count: usize,
    pub mind_commit_receipt_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyDecisionTerminalRecords {
    pub role_worker_results: Vec<crate::EpiphanyRuntimeRoleWorkerResult>,
    pub reorient_worker_results: Vec<crate::EpiphanyRuntimeReorientWorkerResult>,
    pub runtime_job_results: Vec<crate::EpiphanyRuntimeJobResult>,
    pub model_pass_failures: Vec<EpiphanyModelPassFailure>,
    pub archived_worker_attempts: Vec<crate::EpiphanyArchivedRuntimeWorkerAttempt>,
    pub persona_stage_receipts: Vec<crate::PersonaModelStageReceipt>,
    pub persona_effect_documents: Vec<crate::PersonaInterpreterEffectDocument>,
    pub persona_terminal_receipts: Vec<crate::PersonaModelTerminalReceipt>,
    pub persona_conversation_receipts: Vec<crate::PersonaConversationExecutionReceipt>,
    pub reorientation_decisions: Vec<crate::EpiphanyMindReorientationDecisionDocument>,
    pub reorientation_failures: Vec<crate::EpiphanyMindReorientationPassFailureDocument>,
}

impl EpiphanyDecisionTerminalRecords {
    fn len(&self) -> usize {
        self.role_worker_results.len()
            + self.reorient_worker_results.len()
            + self.runtime_job_results.len()
            + self.model_pass_failures.len()
            + self.archived_worker_attempts.len()
            + self.persona_stage_receipts.len()
            + self.persona_effect_documents.len()
            + self.persona_terminal_receipts.len()
            + self.persona_conversation_receipts.len()
            + self.reorientation_decisions.len()
            + self.reorientation_failures.len()
    }

    fn canonicalize(&mut self) {
        self.role_worker_results
            .sort_by(|left, right| left.result_id.cmp(&right.result_id));
        self.reorient_worker_results
            .sort_by(|left, right| left.result_id.cmp(&right.result_id));
        self.runtime_job_results
            .sort_by(|left, right| left.result_id.cmp(&right.result_id));
        self.model_pass_failures
            .sort_by(|left, right| left.failure_id.cmp(&right.failure_id));
        self.archived_worker_attempts
            .sort_by(|left, right| left.job_id.cmp(&right.job_id));
        self.persona_stage_receipts
            .sort_by(|left, right| left.receipt_id.cmp(&right.receipt_id));
        self.persona_effect_documents
            .sort_by(|left, right| left.document_id.cmp(&right.document_id));
        self.persona_terminal_receipts
            .sort_by(|left, right| left.receipt_id.cmp(&right.receipt_id));
        self.persona_conversation_receipts
            .sort_by(|left, right| left.receipt_id.cmp(&right.receipt_id));
        self.reorientation_decisions
            .sort_by(|left, right| left.decision_id.cmp(&right.decision_id));
        self.reorientation_failures
            .sort_by(|left, right| left.failure_id.cmp(&right.failure_id));
    }
}

/// Reconstruct one decision solely from durable typed state. Current Mind
/// contents, model stream events, assistant deltas, and runtime event order are
/// deliberately outside this query.
pub fn audit_decision_context(
    store_path: impl AsRef<Path>,
    context_id: &str,
) -> Result<EpiphanyDecisionAuditProjection> {
    require_non_empty(context_id, "decision audit context id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let context = cache
        .get::<EpiphanyDecisionContext>(context_id)?
        .ok_or_else(|| anyhow!("decision audit context is absent"))?;
    audit_decision_context_from_cache(&cache, &context)?
        .ok_or_else(|| anyhow!("decision audit context has no durable terminal decision record"))
}

/// Enumerate only decisions whose complete transcript-free audit projection
/// validates against the current durable store. Sealed contexts that have not
/// terminalized yet are pass physiology, not decisions, and are omitted.
pub fn list_auditable_decision_contexts(
    store_path: impl AsRef<Path>,
) -> Result<EpiphanyDecisionAuditIndexProjection> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut contexts = cache.get_all::<EpiphanyDecisionContext>()?;
    contexts.sort_by(|left, right| left.context_id.cmp(&right.context_id));
    let mut decisions = Vec::new();
    for context in contexts {
        let Some(audit) = audit_decision_context_from_cache(&cache, &context)? else {
            continue;
        };
        let terminal_record_count = audit.terminal_records.len();
        let mind_commit_receipt_count = audit.mind_commit_receipts.len();
        decisions.push(EpiphanyDecisionAuditIndexEntry {
            context_id: audit.context_id,
            basis_id: audit.reasoning_basis.basis_id,
            pass_id: audit.reasoning_basis.pass_id,
            organ_id: audit.reasoning_basis.organ_id,
            terminal_request_id: audit.decision_context.terminal_request_id,
            terminal_record_count,
            mind_commit_receipt_count,
        });
    }
    Ok(EpiphanyDecisionAuditIndexProjection {
        schema_version: DECISION_AUDIT_INDEX_SCHEMA_VERSION.to_string(),
        decisions,
        transcript_required: false,
    })
}

fn audit_decision_context_from_cache(
    cache: &CultCache,
    context: &EpiphanyDecisionContext,
) -> Result<Option<EpiphanyDecisionAuditProjection>> {
    let context_id = context.context_id.as_str();
    let basis = cache
        .get::<EpiphanyReasoningBasis>(&context.basis_id)?
        .ok_or_else(|| anyhow!("decision audit context lost its reasoning basis"))?;
    context.validate(&basis)?;

    let mut terminal_records = EpiphanyDecisionTerminalRecords {
        role_worker_results: cache
            .get_all::<crate::EpiphanyRuntimeRoleWorkerResult>()?
            .into_iter()
            .filter(|record| record.decision_context_id == context_id)
            .collect(),
        reorient_worker_results: cache
            .get_all::<crate::EpiphanyRuntimeReorientWorkerResult>()?
            .into_iter()
            .filter(|record| record.decision_context_id == context_id)
            .collect(),
        runtime_job_results: cache
            .get_all::<crate::EpiphanyRuntimeJobResult>()?
            .into_iter()
            .filter(|record| record.decision_context_id.as_deref() == Some(context_id))
            .collect(),
        model_pass_failures: cache
            .get_all::<EpiphanyModelPassFailure>()?
            .into_iter()
            .filter(|record| record.decision_context_id == context_id)
            .collect(),
        archived_worker_attempts: cache
            .get_all::<crate::EpiphanyArchivedRuntimeWorkerAttempt>()?
            .into_iter()
            .filter(|record| record.decision_context_id() == Some(context_id))
            .collect(),
        persona_stage_receipts: cache
            .get_all::<crate::PersonaModelStageReceipt>()?
            .into_iter()
            .filter(|record| record.decision_context_id == context_id)
            .collect(),
        persona_effect_documents: cache
            .get_all::<crate::PersonaInterpreterEffectDocument>()?
            .into_iter()
            .filter(|record| record.decision_context_id == context_id)
            .collect(),
        persona_terminal_receipts: cache
            .get_all::<crate::PersonaModelTerminalReceipt>()?
            .into_iter()
            .filter(|record| {
                record
                    .decision_context_ids
                    .iter()
                    .any(|candidate| candidate == context_id)
            })
            .collect(),
        persona_conversation_receipts: cache
            .get_all::<crate::PersonaConversationExecutionReceipt>()?
            .into_iter()
            .filter(|record| record.interpreter_decision_context_id.as_deref() == Some(context_id))
            .collect(),
        reorientation_decisions: cache
            .get_all::<crate::EpiphanyMindReorientationDecisionDocument>()?
            .into_iter()
            .filter(|record| record.decision_context_id == context_id)
            .collect(),
        reorientation_failures: cache
            .get_all::<crate::EpiphanyMindReorientationPassFailureDocument>()?
            .into_iter()
            .filter(|record| record.decision_context_id == context_id)
            .collect(),
    };
    terminal_records.canonicalize();
    if terminal_records.len() == 0 {
        return Ok(None);
    }
    validate_decision_terminal_records(&cache, &basis, &context, &terminal_records)?;

    let mut mind_commit_receipts = cache
        .get_all::<EpiphanyMindCommitReceipt>()?
        .into_iter()
        .filter(|receipt| {
            matches!(
                &receipt.authority,
                EpiphanyMindCommitAuthority::ModelDecisionContext {
                    decision_context_id
                } if decision_context_id == context_id
            )
        })
        .collect::<Vec<_>>();
    for receipt in &mind_commit_receipts {
        receipt.validate()?;
    }
    mind_commit_receipts.sort_by(|left, right| left.receipt_id.cmp(&right.receipt_id));

    Ok(Some(EpiphanyDecisionAuditProjection {
        schema_version: DECISION_AUDIT_PROJECTION_SCHEMA_VERSION.to_string(),
        context_id: context.context_id.clone(),
        decision_context: context.clone(),
        reasoning_projection: basis.projection()?,
        terminal_native_request: context.native_request()?,
        terminal_provider_request: context.provider_request()?,
        tool_observations: context.tool_observations()?,
        reasoning_basis: basis,
        terminal_records,
        mind_commit_receipts,
        transcript_required: false,
    }))
}

fn validate_decision_terminal_records(
    cache: &CultCache,
    basis: &EpiphanyReasoningBasis,
    context: &EpiphanyDecisionContext,
    records: &EpiphanyDecisionTerminalRecords,
) -> Result<()> {
    let context_id = context.context_id.as_str();
    let terminal_role_id = role_terminal_semantic_role(basis)?;
    if records.role_worker_results.iter().any(|record| {
        record.job_id != basis.pass_id || terminal_role_id.as_deref() != Some(&record.role_id)
    }) || records
        .reorient_worker_results
        .iter()
        .any(|record| record.job_id != basis.pass_id)
        || records
            .runtime_job_results
            .iter()
            .any(|record| record.job_id != basis.pass_id)
        || records
            .archived_worker_attempts
            .iter()
            .any(|record| record.job_id != basis.pass_id)
        || records
            .reorientation_decisions
            .iter()
            .any(|record| record.job_id != basis.pass_id)
        || records
            .reorientation_failures
            .iter()
            .any(|record| record.job_id != basis.pass_id)
    {
        return Err(anyhow!(
            "decision audit terminal worker record does not belong to its reasoning pass"
        ));
    }
    if records.model_pass_failures.len() > 1 {
        return Err(anyhow!(
            "decision audit context has multiple model-pass failure owners"
        ));
    }
    if !records.model_pass_failures.is_empty() {
        validate_model_failure_companions(records)?;
    }
    for failure in &records.model_pass_failures {
        failure.validate(basis, context)?;
        let binding = cache
            .get::<crate::EpiphanyRuntimeModelExecutionBinding>(&failure.model_request_id)?
            .ok_or_else(|| anyhow!("decision audit model failure lost its runtime binding"))?;
        if failure.runtime_session_id != binding.session_id
            || failure.runtime_job_id != binding.job_id
        {
            return Err(anyhow!(
                "decision audit model pass failure disagrees with its runtime binding"
            ));
        }
    }
    for record in &records.archived_worker_attempts {
        let fulfilled = crate::WorkerProcessStatus::parse(&record.terminal_process_status)?
            .is_fulfilled_terminal();
        record.validate_decision_record(fulfilled)?;
    }
    if records.persona_stage_receipts.iter().any(|record| {
        record.request_id != basis.pass_id || record.reasoning_basis_id != basis.basis_id
    }) || records
        .persona_effect_documents
        .iter()
        .any(|record| record.interpreter_request_id != basis.pass_id)
    {
        return Err(anyhow!(
            "decision audit Persona record does not belong to its reasoning pass"
        ));
    }
    for terminal in &records.persona_terminal_receipts {
        let index = terminal
            .decision_context_ids
            .iter()
            .position(|candidate| candidate == context_id)
            .ok_or_else(|| anyhow!("decision audit Persona terminal lost its context"))?;
        let receipt_id = terminal
            .stage_receipt_ids
            .get(index)
            .ok_or_else(|| anyhow!("decision audit Persona terminal lost its stage receipt"))?;
        if !records
            .persona_stage_receipts
            .iter()
            .any(|record| &record.receipt_id == receipt_id)
        {
            return Err(anyhow!(
                "decision audit Persona terminal does not bind the exact stage receipt"
            ));
        }
    }
    for conversation in &records.persona_conversation_receipts {
        if !records
            .persona_effect_documents
            .iter()
            .any(|effect| effect.document_id == conversation.effect_document_id)
        {
            return Err(anyhow!(
                "decision audit Persona conversation lost its exact effect document"
            ));
        }
    }
    Ok(())
}

fn validate_model_failure_companions(records: &EpiphanyDecisionTerminalRecords) -> Result<()> {
    let role_result_is_failure_projection = |record: &crate::EpiphanyRuntimeRoleWorkerResult| {
        matches!(
            record.verdict.as_str(),
            "failed" | "runtime-error" | "error"
        ) && record
            .item_error
            .as_ref()
            .is_some_and(|error| !error.trim().is_empty())
            && record.research_decision_msgpack.is_none()
            && record.repo_model_mutation_proposal_msgpack.is_none()
            && record.frontier_plan_candidate_msgpack.is_none()
            && record.frontier_plan_mind_decision_msgpack.is_none()
            && record.imagination_consideration_candidate_msgpack.is_none()
            && record
                .admitted_model_direction_consideration_result_msgpack
                .is_none()
    };
    let runtime_result_is_failure_projection = |record: &crate::EpiphanyRuntimeJobResult| {
        matches!(
            record.verdict.as_str(),
            "failed" | "fail" | "runtime-error" | "error" | "blocked" | "cancelled" | "canceled"
        )
    };
    let archived_attempt_is_failure = |record: &crate::EpiphanyArchivedRuntimeWorkerAttempt| {
        crate::WorkerProcessStatus::parse(&record.terminal_process_status)
            .map(|status| !status.is_fulfilled_terminal())
            .unwrap_or(false)
    };

    if records
        .role_worker_results
        .iter()
        .any(|record| !role_result_is_failure_projection(record))
        || records
            .runtime_job_results
            .iter()
            .any(|record| !runtime_result_is_failure_projection(record))
        || records
            .archived_worker_attempts
            .iter()
            .any(|record| !archived_attempt_is_failure(record))
        || !records.reorient_worker_results.is_empty()
        || !records.persona_stage_receipts.is_empty()
        || !records.persona_effect_documents.is_empty()
        || !records.persona_terminal_receipts.is_empty()
        || !records.persona_conversation_receipts.is_empty()
        || !records.reorientation_decisions.is_empty()
    {
        return Err(anyhow!(
            "failed decision context contains a successful or decision-bearing terminal companion"
        ));
    }
    Ok(())
}

fn role_terminal_semantic_role(basis: &EpiphanyReasoningBasis) -> Result<Option<String>> {
    Ok(match basis.projection()? {
        EpiphanyReasoningProjection::RolePass(projection) => Some(projection.authority.role_id),
        _ => None,
    })
}

pub fn worker_reasoning_basis(
    store_path: &Path,
    launch: &EpiphanyRuntimeWorkerLaunchRequest,
) -> Result<EpiphanyReasoningBasis> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache
        .get_envelope::<EpiphanyRuntimeWorkerLaunchRequest>(&launch.job_id)?
        .ok_or_else(|| anyhow!("worker reasoning basis lost its launch envelope"))?;
    let launch_document = launch.launch_document()?;
    let (source_documents, projection) = match launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => {
            let mind = crate::assemble_mind_view(store_path)?;
            let source_documents = mind.source_documents.clone();
            let authority: EpiphanyRolePassAuthorityProjection = document.into();
            let modeling_body = if authority.role_id.eq_ignore_ascii_case("modeling") {
                Some(crate::load_repository_body_reasoning_projection(
                    store_path,
                    authority
                        .repository_body_observation_basis
                        .as_ref()
                        .ok_or_else(|| {
                            anyhow!("Modeling reasoning launch has no repository Body basis")
                        })?,
                )?)
            } else {
                None
            };
            (
                source_documents,
                EpiphanyReasoningProjection::RolePass(EpiphanyRoleReasoningProjection {
                    authority,
                    mind: mind.into(),
                    modeling_body,
                }),
            )
        }
        EpiphanyWorkerLaunchDocument::Reorient(document) => {
            let request = cache
                .get::<crate::EpiphanyReorientationRequest>(&document.request_id)?
                .ok_or_else(|| anyhow!("reorientation reasoning basis lost its request"))?;
            crate::reorientation_work::validate_reorientation_request_current(&cache, &request)?;
            (
                request.source_documents.clone(),
                EpiphanyReasoningProjection::ReorientLaunch(
                    EpiphanyReorientationReasoningProjection {
                        authority: document,
                        request,
                    },
                ),
            )
        }
    };
    EpiphanyReasoningBasis::new(
        &launch.job_id,
        &launch.role,
        WORKER_REASONING_PROJECTION_POLICY,
        source_documents,
        projection,
    )
}

pub fn reasoning_repo_model_basis(
    basis: &EpiphanyReasoningBasis,
) -> Result<crate::EpiphanyRepoModelBasis> {
    basis.validate()?;
    let source_documents = basis
        .source_documents
        .iter()
        .filter(|source| {
            source
                .document_type
                .starts_with("epiphany.mind.repo_model.")
                || matches!(
                    source.document_type.as_str(),
                    crate::AtlasSurfaceOffer::TYPE
                        | crate::AtlasDependencyClaim::TYPE
                        | crate::AtlasDependencyVerification::TYPE
                        | crate::AtlasDependencyImpact::TYPE
                )
        })
        .cloned()
        .collect::<Vec<_>>();
    let repo_model_basis = crate::EpiphanyRepoModelBasis {
        projection_digest: sha256(&rmp_serde::to_vec_named(&source_documents)?),
        source_documents,
    };
    repo_model_basis.validate()?;
    let EpiphanyReasoningProjection::RolePass(role) = basis.projection()? else {
        return Err(anyhow!("RepoModel reasoning basis is not a role pass"));
    };
    if role
        .mind
        .repo_model
        .as_ref()
        .map(|model| model.projection_digest.as_str())
        != Some(repo_model_basis.projection_digest.as_str())
    {
        return Err(anyhow!(
            "role reasoning projection RepoModel diverges from its exact sources"
        ));
    }
    Ok(repo_model_basis)
}

pub fn put_reasoning_basis(
    store_path: &Path,
    basis: &EpiphanyReasoningBasis,
) -> Result<EpiphanyReasoningBasis> {
    basis.validate()?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    match basis.projection()? {
        EpiphanyReasoningProjection::RolePass(projection) => {
            let launch = cache
                .get::<EpiphanyRuntimeWorkerLaunchRequest>(&basis.pass_id)?
                .ok_or_else(|| anyhow!("role reasoning basis lost its worker launch"))?;
            let EpiphanyWorkerLaunchDocument::Role(document) = launch.launch_document()? else {
                return Err(anyhow!("role reasoning basis cites a non-role launch"));
            };
            let current_mind = crate::assemble_mind_view(store_path)?;
            let current_modeling_body = if projection
                .authority
                .role_id
                .eq_ignore_ascii_case("modeling")
            {
                Some(crate::load_repository_body_reasoning_projection(
                    store_path,
                    projection
                        .authority
                        .repository_body_observation_basis
                        .as_ref()
                        .ok_or_else(|| anyhow!("Modeling reasoning basis has no Body basis"))?,
                )?)
            } else {
                None
            };
            let modeling_body_mismatch = projection
                .authority
                .role_id
                .eq_ignore_ascii_case("modeling")
                && projection.authority.repository_body_observation_basis
                    != current_mind.repository_body_observation;
            if projection.authority != document.into()
                || projection.mind != current_mind.clone().into()
                || projection.modeling_body != current_modeling_body
                || basis.source_documents != current_mind.source_documents
                || modeling_body_mismatch
            {
                return Err(anyhow!(
                    "role reasoning projection diverges from its exact launch or keyed Mind sources"
                ));
            }
        }
        EpiphanyReasoningProjection::ReorientLaunch(projection) => {
            let launch = cache
                .get::<EpiphanyRuntimeWorkerLaunchRequest>(&basis.pass_id)?
                .ok_or_else(|| anyhow!("reorientation reasoning basis lost its worker launch"))?;
            let EpiphanyWorkerLaunchDocument::Reorient(document) = launch.launch_document()? else {
                return Err(anyhow!("reorientation basis cites a non-reorient launch"));
            };
            if projection.authority != document
                || projection.request.request_id != document.request_id
                || projection.request.source_documents != basis.source_documents
            {
                return Err(anyhow!(
                    "reorientation reasoning projection diverges from its request"
                ));
            }
            crate::reorientation_work::validate_reorientation_request_current(
                &cache,
                &projection.request,
            )?;
        }
        EpiphanyReasoningProjection::PersonaProjector(_)
        | EpiphanyReasoningProjection::PersonaTurn(_)
        | EpiphanyReasoningProjection::PersonaInterpreter(_) => {}
    }
    for source in &basis.source_documents {
        if source.store_id != "epiphany-mind" {
            return Err(anyhow!(
                "reasoning basis source store {:?} is not an admitted Mind store",
                source.store_id
            ));
        }
        let live = cache
            .snapshot_envelopes()
            .into_iter()
            .find(|envelope| {
                envelope.r#type == source.document_type && envelope.key == source.document_key
            })
            .ok_or_else(|| {
                anyhow!(
                    "reasoning basis source {:?}/{:?} is absent",
                    source.document_type,
                    source.document_key
                )
            })?;
        if EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", &live)? != *source {
            return Err(anyhow!(
                "reasoning basis source {:?}/{:?} changed before sealing",
                source.document_type,
                source.document_key
            ));
        }
    }
    if let Some(existing) = cache.get::<EpiphanyReasoningBasis>(&basis.basis_id)? {
        if existing != *basis {
            return Err(anyhow!("reasoning basis identity collision"));
        }
        return Ok(existing);
    }
    let envelope = cache.prepare_entry(&basis.basis_id, basis)?.0;
    if !runtime_spine_backing_store(store_path)?.compare_and_swap_batch(&[], vec![envelope])? {
        return put_reasoning_basis(store_path, basis);
    }
    Ok(basis.clone())
}

pub fn put_decision_context(
    store_path: &Path,
    context: &EpiphanyDecisionContext,
) -> Result<EpiphanyDecisionContext> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let basis = cache
        .get::<EpiphanyReasoningBasis>(&context.basis_id)?
        .ok_or_else(|| anyhow!("decision context lost its reasoning basis"))?;
    context.validate(&basis)?;
    validate_context_store_ownership(&cache, context)?;
    if let Some(existing) = cache.get::<EpiphanyDecisionContext>(&context.context_id)? {
        if existing != *context {
            return Err(anyhow!("decision context identity collision"));
        }
        return Ok(existing);
    }
    let basis_envelope = cache
        .get_envelope::<EpiphanyReasoningBasis>(&basis.basis_id)?
        .ok_or_else(|| anyhow!("decision context lost its basis envelope"))?;
    let context_envelope = cache.prepare_entry(&context.context_id, context)?.0;
    if !runtime_spine_backing_store(store_path)?.compare_and_swap_batch(
        &[basis_envelope.clone()],
        vec![basis_envelope, context_envelope],
    )? {
        return put_decision_context(store_path, context);
    }
    Ok(context.clone())
}

pub fn seal_model_decision_context(
    store_path: &Path,
    terminal_request_id: &str,
) -> Result<EpiphanyDecisionContext> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let native = cache
        .get::<EpiphanyModelRequest>(terminal_request_id)?
        .ok_or_else(|| anyhow!("terminal native model request is absent"))?;
    let basis_id = native
        .reasoning_basis_id
        .as_deref()
        .ok_or_else(|| anyhow!("terminal model request has no reasoning basis"))?;
    let basis = cache
        .get::<EpiphanyReasoningBasis>(basis_id)?
        .ok_or_else(|| anyhow!("terminal model request lost its reasoning basis"))?;
    let intents = cache.get_all::<EpiphanyToolInvocationIntent>()?;
    let mut observations = Vec::new();
    for input in &native.input {
        let epiphany_model_adapter::EpiphanyModelInputItem::ToolCall {
            call_id,
            name,
            arguments,
        } = input
        else {
            continue;
        };
        let mut matches = intents.iter().filter(|intent| {
            intent.call_id.as_deref() == Some(call_id.as_str())
                && format!("mcp__{}__{}", intent.server, intent.tool_name) == *name
                && intent.arguments_json == *arguments
        });
        let intent = matches
            .next()
            .ok_or_else(|| anyhow!("terminal tool call has no exact governed intent"))?;
        if matches.next().is_some() {
            return Err(anyhow!(
                "terminal tool call has ambiguous governed ownership"
            ));
        }
        let receipt = cache
            .get::<EpiphanyToolInvocationReceipt>(&tool_invocation_receipt_key(&intent.intent_id))?
            .ok_or_else(|| anyhow!("terminal tool call has no governed receipt"))?;
        observations.push(EpiphanyDecisionToolObservation {
            intent: intent.clone(),
            receipt,
        });
    }
    let context = EpiphanyDecisionContext::new(&basis, native, observations)?;
    put_decision_context(store_path, &context)
}

fn validate_context_store_ownership(
    cache: &cultcache_rs::CultCache,
    context: &EpiphanyDecisionContext,
) -> Result<()> {
    let native = context.native_request()?;
    let provider = context.provider_request()?;
    let observations = context.tool_observations()?;
    if cache
        .get::<EpiphanyModelRequest>(&native.request_id)?
        .as_ref()
        != Some(&native)
        || cache
            .get::<EpiphanyOpenAiModelRequest>(&provider.request_id)?
            .as_ref()
            != Some(&provider)
    {
        return Err(anyhow!(
            "decision context terminal request family is absent or substituted"
        ));
    }
    let model_binding =
        crate::runtime_spine::validate_runtime_model_execution_binding(cache, &native.request_id)?;
    if model_binding.reasoning_basis_id.as_deref() != Some(context.basis_id.as_str())
        || model_binding.source_worker_job_id != native.source_worker_job_id
    {
        return Err(anyhow!(
            "decision context terminal request has foreign runtime ownership"
        ));
    }
    for observation in observations {
        let intent = cache
            .get::<EpiphanyToolInvocationIntent>(&tool_invocation_intent_key(
                &observation.intent.intent_id,
            ))?
            .ok_or_else(|| anyhow!("decision context tool intent is absent"))?;
        let receipt = cache
            .get::<EpiphanyToolInvocationReceipt>(&tool_invocation_receipt_key(
                &observation.intent.intent_id,
            ))?
            .ok_or_else(|| anyhow!("decision context tool receipt is absent"))?;
        let tool_binding = crate::runtime_spine::validate_runtime_tool_execution_binding(
            cache,
            &observation.intent.intent_id,
        )?;
        if intent != observation.intent
            || receipt != observation.receipt
            || crate::runtime_spine::validate_terminal_tool_execution_family(
                &tool_binding,
                &intent,
                &receipt,
            )
            .is_err()
        {
            return Err(anyhow!(
                "decision context tool observation has foreign stored ownership"
            ));
        }
        if let Some(request_id) = intent.model_request_id.as_deref() {
            let continuation = cache
                .get::<crate::EpiphanyRuntimeModelExecutionBinding>(request_id)?
                .ok_or_else(|| anyhow!("decision tool continuation request is absent"))?;
            if continuation.session_id != model_binding.session_id
                || continuation.source_worker_job_id != model_binding.source_worker_job_id
                || continuation.reasoning_basis_id != model_binding.reasoning_basis_id
                || tool_binding.session_id != continuation.session_id
                || tool_binding.job_id != continuation.job_id
            {
                return Err(anyhow!(
                    "decision tool continuation belongs to another reasoning pass"
                ));
            }
        } else if tool_binding.job_id != native.source_worker_job_id.as_deref().unwrap_or_default()
        {
            return Err(anyhow!(
                "request-owned tool observation belongs to another worker"
            ));
        }
    }
    Ok(())
}

pub(crate) fn commit_mind_mutation(
    store_path: &Path,
    decision_context_id: &str,
    invariant_owner: &str,
    strong_reads: Vec<CultCacheEnvelope>,
    writes: Vec<CultCacheEnvelope>,
    committed_at: &str,
) -> Result<EpiphanyMindCommitOutcome> {
    require_non_empty(decision_context_id, "Mind mutation decision context id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let context = cache
        .get::<EpiphanyDecisionContext>(decision_context_id)?
        .ok_or_else(|| anyhow!("Mind mutation decision context does not exist"))?;
    let basis = cache
        .get::<EpiphanyReasoningBasis>(&context.basis_id)?
        .ok_or_else(|| anyhow!("Mind mutation decision context lost its basis"))?;
    context.validate(&basis)?;
    let authority = EpiphanyMindCommitAuthority::ModelDecisionContext {
        decision_context_id: decision_context_id.to_string(),
    };
    commit_authorized_mind_mutation(
        store_path,
        authority,
        invariant_owner,
        strong_reads,
        writes,
        Vec::new(),
        committed_at,
    )
}

pub(crate) fn commit_mind_mutation_with_derived_companions(
    store_path: &Path,
    decision_context_id: &str,
    invariant_owner: &str,
    strong_reads: Vec<CultCacheEnvelope>,
    writes: Vec<CultCacheEnvelope>,
    derived_companions: Vec<CultCacheEnvelope>,
    committed_at: &str,
) -> Result<EpiphanyMindCommitOutcome> {
    require_non_empty(decision_context_id, "Mind mutation decision context id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let context = cache
        .get::<EpiphanyDecisionContext>(decision_context_id)?
        .ok_or_else(|| anyhow!("Mind mutation decision context does not exist"))?;
    let basis = cache
        .get::<EpiphanyReasoningBasis>(&context.basis_id)?
        .ok_or_else(|| anyhow!("Mind mutation decision context lost its basis"))?;
    context.validate(&basis)?;
    commit_authorized_mind_mutation(
        store_path,
        EpiphanyMindCommitAuthority::ModelDecisionContext {
            decision_context_id: decision_context_id.to_string(),
        },
        invariant_owner,
        strong_reads,
        writes,
        derived_companions,
        committed_at,
    )
}

pub(crate) fn commit_operator_mind_mutation(
    store_path: &Path,
    provenance: CultCacheEnvelope,
    invariant_owner: &str,
    strong_reads: Vec<CultCacheEnvelope>,
    writes: Vec<CultCacheEnvelope>,
    committed_at: &str,
) -> Result<EpiphanyMindCommitOutcome> {
    commit_operator_mind_mutation_with_derived_companions(
        store_path,
        provenance,
        invariant_owner,
        strong_reads,
        writes,
        Vec::new(),
        committed_at,
    )
}

pub(crate) fn commit_operator_mind_mutation_with_derived_companions(
    store_path: &Path,
    provenance: CultCacheEnvelope,
    invariant_owner: &str,
    strong_reads: Vec<CultCacheEnvelope>,
    writes: Vec<CultCacheEnvelope>,
    derived_companions: Vec<CultCacheEnvelope>,
    committed_at: &str,
) -> Result<EpiphanyMindCommitOutcome> {
    let provenance_version =
        EpiphanyMindDocumentVersion::from_envelope("epiphany-operator", &provenance)?;
    let authority = EpiphanyMindCommitAuthority::OperatorProvenance {
        provenance: provenance_version,
    };
    let mut companions = Vec::with_capacity(derived_companions.len() + 1);
    companions.push(provenance);
    companions.extend(derived_companions);
    commit_authorized_mind_mutation(
        store_path,
        authority,
        invariant_owner,
        strong_reads,
        writes,
        companions,
        committed_at,
    )
}

pub(crate) fn commit_typed_organ_mind_mutation(
    store_path: &Path,
    organ: &str,
    provenance: CultCacheEnvelope,
    invariant_owner: &str,
    strong_reads: Vec<CultCacheEnvelope>,
    writes: Vec<CultCacheEnvelope>,
    committed_at: &str,
) -> Result<EpiphanyMindCommitOutcome> {
    require_non_empty(organ, "Mind mutation organ")?;
    let provenance_version =
        EpiphanyMindDocumentVersion::from_envelope("epiphany-organ", &provenance)?;
    commit_authorized_mind_mutation(
        store_path,
        EpiphanyMindCommitAuthority::TypedOrganProvenance {
            organ: organ.to_string(),
            provenance: provenance_version,
        },
        invariant_owner,
        strong_reads,
        writes,
        vec![provenance],
        committed_at,
    )
}

pub(crate) fn commit_external_typed_observation_mind_mutation(
    store_path: &Path,
    organ: &str,
    provenance: EpiphanyMindDocumentVersion,
    invariant_owner: &str,
    strong_reads: Vec<CultCacheEnvelope>,
    writes: Vec<CultCacheEnvelope>,
    committed_at: &str,
) -> Result<EpiphanyMindCommitOutcome> {
    require_non_empty(organ, "Mind mutation organ")?;
    provenance.validate()?;
    commit_authorized_mind_mutation(
        store_path,
        EpiphanyMindCommitAuthority::TypedOrganProvenance {
            organ: organ.to_string(),
            provenance,
        },
        invariant_owner,
        strong_reads,
        writes,
        Vec::new(),
        committed_at,
    )
}

fn commit_authorized_mind_mutation(
    store_path: &Path,
    authority: EpiphanyMindCommitAuthority,
    invariant_owner: &str,
    strong_reads: Vec<CultCacheEnvelope>,
    writes: Vec<CultCacheEnvelope>,
    companions: Vec<CultCacheEnvelope>,
    committed_at: &str,
) -> Result<EpiphanyMindCommitOutcome> {
    require_non_empty(invariant_owner, "Mind mutation invariant owner")?;
    require_non_empty(committed_at, "Mind mutation commit time")?;
    chrono::DateTime::parse_from_rfc3339(committed_at)
        .map_err(|error| anyhow!("Mind mutation commit time is invalid: {error}"))?;
    if writes.is_empty() {
        return Err(anyhow!("Mind mutation requires at least one write"));
    }
    for write in &writes {
        crate::mind_documents::validate_mind_write_envelope(write)?;
    }
    validate_unique_envelope_identities(&strong_reads, "strong read")?;
    validate_unique_envelope_identities(&writes, "write")?;
    validate_unique_envelope_identities(&companions, "companion")?;
    let expected_ids = strong_reads
        .iter()
        .map(|entry| (entry.r#type.as_str(), entry.key.as_str()))
        .collect::<BTreeSet<_>>();
    let write_ids = writes
        .iter()
        .map(|entry| (entry.r#type.clone(), entry.key.clone()))
        .collect::<BTreeSet<_>>();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut companion_expected = Vec::new();
    let mut companion_replacements = Vec::new();
    for companion in companions {
        if let Some(existing) = cache
            .snapshot_envelopes()
            .into_iter()
            .find(|entry| entry.r#type == companion.r#type && entry.key == companion.key)
        {
            if existing != companion {
                return Err(anyhow!("Mind mutation companion identity collision"));
            }
            companion_expected.push(existing.clone());
            companion_replacements.push(existing);
        } else {
            companion_replacements.push(companion);
        }
    }

    let strong_versions = strong_reads
        .iter()
        .map(|entry| EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", entry))
        .collect::<Result<Vec<_>>>()?;
    let write_versions = writes
        .iter()
        .map(|entry| EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", entry))
        .collect::<Result<Vec<_>>>()?;
    let receipt_id = mind_commit_receipt_id(
        &authority,
        invariant_owner,
        &strong_versions,
        &write_versions,
    )?;
    let receipt = EpiphanyMindCommitReceipt {
        schema_version: MIND_COMMIT_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: receipt_id.clone(),
        authority,
        invariant_owner: invariant_owner.to_string(),
        strong_reads: strong_versions,
        writes: write_versions,
        committed_at: committed_at.to_string(),
    };
    receipt.validate()?;
    if let Some(existing) = cache.get::<EpiphanyMindCommitReceipt>(&receipt_id)? {
        existing.validate()?;
        let mut replay = receipt;
        replay.committed_at = existing.committed_at.clone();
        if existing != replay {
            return Err(anyhow!("Mind commit receipt identity collision"));
        }
        return Ok(EpiphanyMindCommitOutcome::Committed(existing));
    }
    let mut replacements = writes;
    replacements.extend(
        strong_reads
            .iter()
            .filter(|entry| !write_ids.contains(&(entry.r#type.clone(), entry.key.clone())))
            .cloned(),
    );
    replacements.extend(companion_replacements);
    replacements.push(cache.prepare_entry(&receipt_id, &receipt)?.0);
    let mut expected = strong_reads.clone();
    expected.extend(companion_expected);
    if runtime_spine_backing_store(store_path)?.compare_and_swap_batch(&expected, replacements)? {
        return Ok(EpiphanyMindCommitOutcome::Committed(receipt));
    }
    let current = runtime_spine_backing_store(store_path)?.pull_all()?;
    let mut conflicts = strong_reads
        .iter()
        .filter(|expected| {
            current
                .iter()
                .find(|entry| entry.r#type == expected.r#type && entry.key == expected.key)
                != Some(*expected)
        })
        .map(|entry| (entry.r#type.clone(), entry.key.clone()))
        .collect::<Vec<_>>();
    if conflicts.is_empty() {
        conflicts = current_write_collisions(&current, &receipt.writes, &expected_ids);
    }
    conflicts.sort();
    conflicts.dedup();
    Ok(EpiphanyMindCommitOutcome::Conflict {
        document_identities: conflicts,
    })
}

fn validate_request_pair(
    basis: &EpiphanyReasoningBasis,
    native: &EpiphanyModelRequest,
    provider: &EpiphanyOpenAiModelRequest,
) -> Result<()> {
    if native.reasoning_basis_id.as_deref() != Some(basis.basis_id.as_str()) {
        return Err(anyhow!("model request does not bind its reasoning basis"));
    }
    if provider != &epiphany_openai_adapter::request_from_native(native) {
        return Err(anyhow!("native and provider terminal requests diverge"));
    }
    Ok(())
}

fn validate_tool_observations(
    request: &EpiphanyModelRequest,
    observations: &[EpiphanyDecisionToolObservation],
) -> Result<()> {
    let mut observed = BTreeSet::new();
    for observation in observations {
        if observation.intent.intent_id != observation.receipt.intent_id
            || observation.intent.adapter != observation.receipt.adapter
            || observation.intent.server != observation.receipt.server
            || observation.intent.tool_name != observation.receipt.tool_name
            || observation
                .intent
                .call_id
                .as_deref()
                .unwrap_or_default()
                .is_empty()
            || observation.intent.schema_id
                != epiphany_tool_adapter::TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID
            || observation.receipt.schema_id
                != epiphany_tool_adapter::TOOL_ADAPTER_INVOCATION_RECEIPT_SCHEMA_ID
            || observation.receipt.receipt_id.trim().is_empty()
            || observation.receipt.status.trim().is_empty()
        {
            return Err(anyhow!("decision tool observation ownership mismatch"));
        }
        if !observed.insert(observation.intent.intent_id.clone()) {
            return Err(anyhow!("decision context repeats a tool observation"));
        }
    }
    let mut request_observations = Vec::new();
    let mut index = 0usize;
    while index < request.input.len() {
        match &request.input[index] {
            epiphany_model_adapter::EpiphanyModelInputItem::ToolCall {
                call_id,
                name,
                arguments,
            } => {
                let Some(epiphany_model_adapter::EpiphanyModelInputItem::ToolResult {
                    call_id: result_call_id,
                    output,
                }) = request.input.get(index + 1)
                else {
                    return Err(anyhow!(
                        "terminal request tool call is not immediately paired with its result"
                    ));
                };
                if result_call_id != call_id {
                    return Err(anyhow!("terminal request tool result call id mismatch"));
                }
                request_observations.push((call_id, name, arguments, output));
                index += 2;
            }
            epiphany_model_adapter::EpiphanyModelInputItem::ToolResult { .. } => {
                return Err(anyhow!("terminal request contains an unpaired tool result"));
            }
            _ => index += 1,
        }
    }
    if request_observations.len() != observations.len() {
        return Err(anyhow!(
            "decision context tool observation count does not match terminal request"
        ));
    }
    for ((call_id, name, arguments, output), observation) in
        request_observations.into_iter().zip(observations)
    {
        let expected_name = format!(
            "mcp__{}__{}",
            observation.intent.server, observation.intent.tool_name
        );
        if observation.intent.call_id.as_deref() != Some(call_id.as_str())
            || name != &expected_name
            || arguments != &observation.intent.arguments_json
            || output != &receipt_output_for_model(&observation.intent, &observation.receipt)
        {
            return Err(anyhow!(
                "decision context tool observation does not match terminal request bytes"
            ));
        }
    }
    Ok(())
}

fn canonicalize_source_documents(documents: &mut Vec<EpiphanyMindDocumentVersion>) -> Result<()> {
    for document in documents.iter() {
        document.validate()?;
    }
    documents.sort_by(|left, right| {
        left.store_id
            .cmp(&right.store_id)
            .then(left.document_type.cmp(&right.document_type))
            .then(left.document_key.cmp(&right.document_key))
    });
    for pair in documents.windows(2) {
        if pair[0].store_id == pair[1].store_id
            && pair[0].document_type == pair[1].document_type
            && pair[0].document_key == pair[1].document_key
        {
            return Err(anyhow!(
                "reasoning basis repeats a source document identity"
            ));
        }
    }
    Ok(())
}

fn digest_without_basis_id(basis: &EpiphanyReasoningBasis) -> Result<String> {
    let mut canonical = basis.clone();
    canonical.basis_id.clear();
    Ok(sha256(&rmp_serde::to_vec_named(&canonical)?))
}

fn digest_without_context_id(context: &EpiphanyDecisionContext) -> Result<String> {
    let mut canonical = context.clone();
    canonical.context_id.clear();
    Ok(sha256(&rmp_serde::to_vec_named(&canonical)?))
}

fn mind_commit_receipt_id(
    authority: &EpiphanyMindCommitAuthority,
    owner: &str,
    strong_reads: &[EpiphanyMindDocumentVersion],
    writes: &[EpiphanyMindDocumentVersion],
) -> Result<String> {
    Ok(format!(
        "mind-commit-{}",
        sha256(&rmp_serde::to_vec_named(&(
            authority,
            owner,
            strong_reads,
            writes
        ))?)
    ))
}

fn validate_unique_envelope_identities(entries: &[CultCacheEnvelope], label: &str) -> Result<()> {
    let mut identities = BTreeSet::new();
    for entry in entries {
        if !identities.insert((entry.r#type.as_str(), entry.key.as_str())) {
            return Err(anyhow!("Mind mutation repeats {label} identity"));
        }
    }
    Ok(())
}

fn validate_mind_document_versions(
    entries: &[EpiphanyMindDocumentVersion],
    label: &str,
) -> Result<()> {
    let mut identities = BTreeSet::new();
    for entry in entries {
        entry.validate()?;
        if !identities.insert(entry.identity()) {
            return Err(anyhow!("Mind commit receipt repeats {label} identity"));
        }
    }
    Ok(())
}

fn current_write_collisions(
    current: &[CultCacheEnvelope],
    writes: &[EpiphanyMindDocumentVersion],
    expected_ids: &BTreeSet<(&str, &str)>,
) -> Vec<(String, String)> {
    writes
        .iter()
        .filter(|write| {
            !expected_ids.contains(&(write.document_type.as_str(), write.document_key.as_str()))
                && current.iter().any(|entry| {
                    entry.r#type == write.document_type && entry.key == write.document_key
                })
        })
        .map(|write| (write.document_type.clone(), write.document_key.clone()))
        .collect()
}

fn require_non_empty(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{label} cannot be empty"));
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EpiphanyRuntimeJobStatus, RuntimeSpineInitOptions, initialize_runtime_spine};
    use cultcache_rs::{CacheBackingStore, SingleFileMessagePackBackingStore};
    use epiphany_model_adapter::EpiphanyModelInputItem;
    use tempfile::tempdir;

    fn role_document() -> EpiphanyRoleWorkerLaunchDocument {
        EpiphanyRoleWorkerLaunchDocument {
            thread_id: "creation-thread".into(),
            role_id: "Imagination".into(),
            objective: Some("Propose one future".into()),
            dynamic_prompt_context: Some("typed projection".into()),
            repository_body_observation_basis: None,
            proposal_modeling_context: None,
            frontier_verdict_modeling_context: None,
            frontier_planning_context: None,
            frontier_research_context: None,
            frontier_verification_context: None,
            frontier_plan_mind_context: None,
            imagination_consideration_context: None,
            admitted_model_direction_consideration_context: None,
        }
    }

    fn role_launch() -> Result<EpiphanyRuntimeWorkerLaunchRequest> {
        let document = EpiphanyWorkerLaunchDocument::Role(role_document());
        Ok(EpiphanyRuntimeWorkerLaunchRequest {
            schema_version: crate::RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION.into(),
            job_id: "pass-1".into(),
            binding_id: crate::EPIPHANY_IMAGINATION_ROLE_BINDING_ID.into(),
            role: "Imagination".into(),
            authority_scope: "epiphany.role.imagination".into(),
            instruction: "Propose one future".into(),
            output_contract_id: crate::ROLE_WORKER_OUTPUT_CONTRACT_ID.into(),
            document_kind: "role".into(),
            launch_document_msgpack: rmp_serde::to_vec_named(&document)?,
            metadata: Default::default(),
            proposal_modeling_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            repo_frontier_verification_request_id: None,
        })
    }

    fn basis() -> Result<EpiphanyReasoningBasis> {
        let mut document = role_document();
        document.role_id = "Modeling".into();
        document.objective = Some("Map the Body".into());
        let authority = EpiphanyRolePassAuthorityProjection::from(document);
        let payload_msgpack = rmp_serde::to_vec_named(&crate::EpiphanyMindIdentity {
            schema_epoch: crate::MIND_SCHEMA_EPOCH.into(),
            runtime_id: "test-runtime".into(),
        })?;
        let source_documents = vec![EpiphanyMindDocumentVersion {
            store_id: "epiphany-mind".into(),
            document_type: "epiphany.mind.identity.v1".into(),
            document_key: crate::MIND_SCHEMA_EPOCH.into(),
            schema_id: None,
            payload_sha256: sha256(&payload_msgpack),
            payload_msgpack,
        }];
        let projection = EpiphanyReasoningProjection::RolePass(EpiphanyRoleReasoningProjection {
            authority,
            mind: EpiphanyMindPromptProjection {
                schema_epoch: crate::MIND_SCHEMA_EPOCH.into(),
                runtime_id: "test-runtime".into(),
                projection_digest: crate::epiphany_mind_projection_digest(&source_documents)?,
                objective: None,
                active_subgoal_id: None,
                subgoals: Vec::new(),
                invariants: Vec::new(),
                observations: Vec::new(),
                evidence: Vec::new(),
                verification_audits: Vec::new(),
                reorientation_decisions: Vec::new(),
                reorientation_failures: Vec::new(),
                agent_pass_admission_refusals: Vec::new(),
                investigation_checkpoint: None,
                mode: None,
                repository_body_observation: None,
                repo_model: None,
            },
            modeling_body: None,
        });
        EpiphanyReasoningBasis::new(
            "pass-1",
            "Modeling",
            WORKER_REASONING_PROJECTION_POLICY,
            source_documents,
            projection,
        )
    }

    fn requests(
        basis: &EpiphanyReasoningBasis,
    ) -> (EpiphanyModelRequest, EpiphanyOpenAiModelRequest) {
        let mut native = EpiphanyModelRequest::new(
            "request-1",
            "conversation-1",
            "openai-codex",
            "gpt-test",
            "inspect",
        );
        native.reasoning_basis_id = Some(basis.basis_id.clone());
        native.input.push(EpiphanyModelInputItem::UserText {
            text: "projection".into(),
        });
        let provider = EpiphanyOpenAiModelRequest {
            schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_REQUEST_SCHEMA_ID.into(),
            request_id: native.request_id.clone(),
            conversation_id: native.conversation_id.clone(),
            model: native.model.clone(),
            instructions: native.instructions.clone(),
            input: vec![epiphany_openai_adapter::EpiphanyOpenAiInputItem::UserText {
                text: "projection".into(),
            }],
            reasoning_effort: None,
            reasoning_summary: None,
            service_tier: None,
            output_contract_id: None,
            previous_response_id: None,
            tools: Vec::new(),
            output_schema_json: None,
            provider_id: native.provider.clone(),
            wire_dialect: epiphany_openai_adapter::EpiphanyOpenAiWireDialect::Responses,
        };
        (native, provider)
    }

    #[test]
    fn basis_and_context_are_content_addressed_and_reject_substitution() -> Result<()> {
        let reasoning_basis = basis()?;
        assert_eq!(reasoning_basis, basis()?);
        let (native, _) = requests(&reasoning_basis);
        let context = EpiphanyDecisionContext::new(&reasoning_basis, native.clone(), Vec::new())?;
        context.validate(&reasoning_basis)?;
        let mut substituted = context.clone();
        substituted.native_request_msgpack.push(0xff);
        assert!(substituted.validate(&reasoning_basis).is_err());
        Ok(())
    }

    #[test]
    fn role_terminal_identity_uses_the_sealed_semantic_role_not_the_runtime_owner_label()
    -> Result<()> {
        let canonical = basis()?;
        let projection = canonical.projection()?;
        let owner_labeled = EpiphanyReasoningBasis::new(
            &canonical.pass_id,
            crate::EPIPHANY_MODELING_OWNER_ROLE,
            &canonical.projection_policy_id,
            canonical.source_documents.clone(),
            projection,
        )?;

        assert_eq!(
            role_terminal_semantic_role(&owner_labeled)?.as_deref(),
            Some("Modeling")
        );
        assert_ne!(
            role_terminal_semantic_role(&owner_labeled)?.as_deref(),
            Some(crate::EPIPHANY_MODELING_OWNER_ROLE)
        );
        Ok(())
    }

    #[test]
    fn decision_context_binds_exact_provider_and_tool_bytes() -> Result<()> {
        let reasoning_basis = basis()?;
        let (native, _) = requests(&reasoning_basis);
        let context = EpiphanyDecisionContext::new(&reasoning_basis, native.clone(), Vec::new())?;
        let mut substituted_provider_input = context.provider_request()?;
        substituted_provider_input.input.clear();
        let mut substituted_provider_context = context;
        substituted_provider_context.provider_request_msgpack =
            rmp_serde::to_vec_named(&substituted_provider_input)?;
        assert!(
            substituted_provider_context
                .validate(&reasoning_basis)
                .is_err()
        );

        let mut native_with_substituted_tools = native.clone();
        native_with_substituted_tools.tools.push(
            epiphany_model_adapter::EpiphanyModelToolDefinition {
                name: "mcp__test__read".into(),
                description: "test".into(),
                parameters_json: r#"{"type":"object"}"#.into(),
            },
        );
        let tool_context = EpiphanyDecisionContext::new(
            &reasoning_basis,
            native_with_substituted_tools.clone(),
            Vec::new(),
        )?;
        assert_eq!(
            tool_context.provider_request()?,
            epiphany_openai_adapter::request_from_native(&native_with_substituted_tools),
            "provider request must be derived from the complete native request"
        );

        let intent = EpiphanyToolInvocationIntent::new(
            "intent-1",
            epiphany_tool_adapter::EPIPHANY_TOOL_RUNTIME_ADAPTER_ID,
            "test",
            "read",
            r#"{"key":"exact"}"#,
            "model",
            "test exact terminal context",
            "2026-08-17T00:00:00Z",
        )
        .with_model_call("call-1", &native.request_id);
        let mut receipt = EpiphanyToolInvocationReceipt::new(
            "receipt-1",
            &intent.intent_id,
            &intent.adapter,
            &intent.server,
            &intent.tool_name,
            "completed",
            "2026-08-17T00:00:01Z",
        );
        receipt.result_json = Some(r#"{"value":"exact"}"#.into());
        let observation = EpiphanyDecisionToolObservation {
            intent: intent.clone(),
            receipt: receipt.clone(),
        };
        let mut terminal_native = native;
        terminal_native.input.extend([
            EpiphanyModelInputItem::ToolCall {
                call_id: "call-1".into(),
                name: "mcp__test__read".into(),
                arguments: r#"{"key":"exact"}"#.into(),
            },
            EpiphanyModelInputItem::ToolResult {
                call_id: "call-1".into(),
                output: r#"{"value":"exact"}"#.into(),
            },
        ]);
        EpiphanyDecisionContext::new(
            &reasoning_basis,
            terminal_native.clone(),
            vec![observation.clone()],
        )?;

        let mut substituted_receipt = observation;
        substituted_receipt.receipt.result_json = Some(r#"{"value":"foreign"}"#.into());
        assert!(
            EpiphanyDecisionContext::new(
                &reasoning_basis,
                terminal_native.clone(),
                vec![substituted_receipt],
            )
            .is_err()
        );

        terminal_native.input.swap(1, 2);
        assert!(
            EpiphanyDecisionContext::new(
                &reasoning_basis,
                terminal_native,
                vec![EpiphanyDecisionToolObservation { intent, receipt }],
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn decision_context_seals_openrouter_identity_and_terminal_tool_dialect() -> Result<()> {
        let reasoning_basis = basis()?;
        let mut native = EpiphanyModelRequest::new(
            "request-openrouter",
            "conversation-openrouter",
            "openrouter",
            "stealth/ox-alpha",
            "decide",
        );
        native.reasoning_basis_id = Some(reasoning_basis.basis_id.clone());
        native.output_contract_id = Some("epiphany.test.output.v0".into());
        native.output_schema_json = Some(
            serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {"status": {"type": "string"}},
                "required": ["status"]
            })
            .to_string(),
        );

        let context = EpiphanyDecisionContext::new(&reasoning_basis, native.clone(), Vec::new())?;
        let provider = context.provider_request()?;
        assert_eq!(provider.provider_id, "openrouter");
        assert_eq!(
            provider.wire_dialect,
            epiphany_openai_adapter::EpiphanyOpenAiWireDialect::ChatCompletionsTerminalTool
        );
        assert_eq!(
            provider,
            epiphany_openai_adapter::request_from_native(&native)
        );
        Ok(())
    }

    #[test]
    fn decision_context_reuses_runtime_binding_owner_and_survives_transcript_deletion() -> Result<()>
    {
        let temp = tempdir()?;
        let store = temp.path().join("mind.cc");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "decision-context-binding-test".into(),
                display_name: "Decision context binding test".into(),
                created_at: "2026-08-17T00:00:00Z".into(),
            },
        )?;
        let launch = role_launch()?;
        let mut cache = runtime_spine_cache(&store)?;
        cache.put(&launch.job_id, &launch)?;
        let reasoning_basis = worker_reasoning_basis(&store, &launch)?;
        let reasoning_basis = put_reasoning_basis(&store, &reasoning_basis)?;
        let (initial_native, _) = requests(&reasoning_basis);
        crate::open_runtime_model_execution(
            &store,
            crate::RuntimeSpineSessionOptions {
                session_id: "session-1".into(),
                objective: "Prove governed decision context".into(),
                created_at: "2026-08-17T00:00:01Z".into(),
                coordinator_note: "decision context binding test".into(),
            },
            crate::RuntimeSpineJobOptions {
                job_id: "model-job-1".into(),
                session_id: "session-1".into(),
                role: "model-adapter".into(),
                created_at: "2026-08-17T00:00:01Z".into(),
            },
            &initial_native,
            "2026-08-17T00:00:01Z",
        )?;
        let intent = EpiphanyToolInvocationIntent::new(
            "intent-1",
            epiphany_tool_adapter::EPIPHANY_TOOL_RUNTIME_ADAPTER_ID,
            "test",
            "read",
            r#"{"key":"exact"}"#,
            "model",
            "test exact runtime ownership",
            "2026-08-17T00:00:02Z",
        )
        .with_model_call("call-1", &initial_native.request_id);
        crate::put_runtime_tool_execution_intent(
            &store,
            "session-1",
            "model-job-1",
            &intent,
            "2026-08-17T00:00:02Z",
        )?;
        let mut receipt = EpiphanyToolInvocationReceipt::new(
            "receipt-1",
            &intent.intent_id,
            &intent.adapter,
            &intent.server,
            &intent.tool_name,
            "completed",
            "2026-08-17T00:00:03Z",
        );
        receipt.result_json = Some(r#"{"value":"exact"}"#.into());
        crate::put_runtime_tool_execution_receipt(&store, &receipt)?;

        let mut terminal_native = initial_native.clone();
        terminal_native.request_id = "request-2".into();
        terminal_native.input.extend([
            EpiphanyModelInputItem::ToolCall {
                call_id: "call-1".into(),
                name: "mcp__test__read".into(),
                arguments: r#"{"key":"exact"}"#.into(),
            },
            EpiphanyModelInputItem::ToolResult {
                call_id: "call-1".into(),
                output: r#"{"value":"exact"}"#.into(),
            },
        ]);
        crate::open_runtime_model_execution(
            &store,
            crate::RuntimeSpineSessionOptions {
                session_id: "session-1".into(),
                objective: "Prove governed decision context".into(),
                created_at: "2026-08-17T00:00:04Z".into(),
                coordinator_note: "decision context binding test".into(),
            },
            crate::RuntimeSpineJobOptions {
                job_id: "model-job-2".into(),
                session_id: "session-1".into(),
                role: "model-adapter".into(),
                created_at: "2026-08-17T00:00:04Z".into(),
            },
            &terminal_native,
            "2026-08-17T00:00:04Z",
        )?;
        let context = seal_model_decision_context(&store, &terminal_native.request_id)?;

        let native_delta = epiphany_model_adapter::EpiphanyModelStreamEvent {
            schema_id: epiphany_model_adapter::MODEL_ADAPTER_EVENT_SCHEMA_ID.into(),
            request_id: terminal_native.request_id.clone(),
            provider: terminal_native.provider.clone(),
            sequence: 0,
            payload: epiphany_model_adapter::EpiphanyModelStreamPayload::TextDelta {
                text: "optional transcript".into(),
            },
        };
        cache = runtime_spine_cache(&store)?;
        cache.put("request-2:00000000", &native_delta)?;
        assert!(
            cache
                .delete::<epiphany_model_adapter::EpiphanyModelStreamEvent>("request-2:00000000")?
        );
        cache.pull_all_backing_stores()?;
        let retained = cache
            .get::<EpiphanyDecisionContext>(&context.context_id)?
            .expect("decision context remains after transcript deletion");
        let retained_basis = cache
            .get::<EpiphanyReasoningBasis>(&reasoning_basis.basis_id)?
            .expect("reasoning basis remains after transcript deletion");
        retained.validate(&retained_basis)?;

        let mut hostile_binding = cache
            .get::<crate::EpiphanyRuntimeToolExecutionBinding>(&intent.intent_id)?
            .expect("tool binding");
        hostile_binding.bound_at = "not-a-time".into();
        cache.put(&intent.intent_id, &hostile_binding)?;
        let before = SingleFileMessagePackBackingStore::new(&store).pull_all()?;
        assert!(seal_model_decision_context(&store, &terminal_native.request_id).is_err());
        assert_eq!(
            SingleFileMessagePackBackingStore::new(&store).pull_all()?,
            before
        );
        Ok(())
    }

    #[test]
    fn model_pass_failure_atomically_closes_session_and_remains_auditable() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("mind.cc");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "model-pass-failure-test".into(),
                display_name: "Model pass failure test".into(),
                created_at: "2026-08-18T00:00:00Z".into(),
            },
        )?;
        let launch = role_launch()?;
        let mut cache = runtime_spine_cache(&store)?;
        cache.put(&launch.job_id, &launch)?;
        let reasoning_basis =
            put_reasoning_basis(&store, &worker_reasoning_basis(&store, &launch)?)?;
        let (native, _provider) = requests(&reasoning_basis);
        crate::open_runtime_model_execution(
            &store,
            crate::RuntimeSpineSessionOptions {
                session_id: "failed-pass-session".into(),
                objective: "Prove terminal failure ownership".into(),
                created_at: "2026-08-18T00:00:01Z".into(),
                coordinator_note: "model pass failure test".into(),
            },
            crate::RuntimeSpineJobOptions {
                job_id: "failed-model-job".into(),
                session_id: "failed-pass-session".into(),
                role: "model-adapter".into(),
                created_at: "2026-08-18T00:00:01Z".into(),
            },
            &native,
            "2026-08-18T00:00:01Z",
        )?;
        let context = seal_model_decision_context(&store, &native.request_id)?;
        let options = crate::ModelPassFailureTerminalOptions {
            decision_context_id: context.context_id.clone(),
            failure_kind: "provider_or_transport_failure".into(),
            summary: "Provider refused before assistant output.".into(),
            failed_at: "2026-08-18T00:00:03Z".into(),
        };
        let failure = crate::terminalize_model_pass_failure_session(&store, options.clone())?;
        let mut cache = runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        assert_eq!(
            cache
                .get::<crate::EpiphanyRuntimeSession>("failed-pass-session")?
                .expect("session")
                .status,
            crate::EpiphanyRuntimeSessionStatus::Completed
        );
        assert_eq!(
            crate::model_pass_failure_for_request(&store, &native.request_id)?,
            Some(failure.clone())
        );
        let model_job =
            crate::runtime_job_snapshot(&store, "failed-model-job")?.expect("model job snapshot");
        assert_eq!(
            model_job.job.status,
            crate::EpiphanyRuntimeJobStatus::Failed
        );
        assert!(
            model_job
                .result
                .expect("model transport failure result")
                .decision_context_id
                .is_none(),
            "generic model transport must not own decision authority"
        );
        let audit = audit_decision_context(&store, &context.context_id)?;
        assert_eq!(
            audit.terminal_records.model_pass_failures,
            vec![failure.clone()]
        );
        assert!(!audit.transcript_required);
        let index = list_auditable_decision_contexts(&store)?;
        assert_eq!(index.schema_version, DECISION_AUDIT_INDEX_SCHEMA_VERSION);
        assert!(!index.transcript_required);
        assert_eq!(index.decisions.len(), 1);
        assert_eq!(index.decisions[0].context_id, context.context_id);
        assert_eq!(index.decisions[0].pass_id, reasoning_basis.pass_id);
        assert_eq!(index.decisions[0].organ_id, reasoning_basis.organ_id);
        assert_eq!(index.decisions[0].terminal_record_count, 1);

        let failure_summary = "Provider refused before assistant output.".to_string();
        let failure_role_result = crate::EpiphanyRuntimeRoleWorkerResult {
            schema_version: crate::RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION.to_string(),
            result_id: format!("result-worker-{}", reasoning_basis.pass_id),
            job_id: reasoning_basis.pass_id.clone(),
            role_id: role_terminal_semantic_role(&reasoning_basis)?.expect("role pass"),
            verdict: "runtime-error".into(),
            summary: failure_summary.clone(),
            next_safe_move: "Create a fresh work obligation and reasoning pass.".into(),
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
            item_error: Some(failure_summary.clone()),
            metadata: std::collections::BTreeMap::new(),
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
            repository_body_observation_basis: None,
            imagination_consideration_request_id: None,
            imagination_consideration_candidate_msgpack: None,
            admitted_model_direction_consideration_request_id: None,
            admitted_model_direction_consideration_result_msgpack: None,
            decision_context_id: context.context_id.clone(),
        };
        let failure_runtime_result = crate::EpiphanyRuntimeJobResult {
            schema_version: crate::RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
            result_id: format!("result-runtime-{}", reasoning_basis.pass_id),
            job_id: reasoning_basis.pass_id.clone(),
            session_id: "failed-pass-session".into(),
            role: reasoning_basis.organ_id.clone(),
            verdict: "failed".into(),
            summary: failure_summary,
            completed_at: "2026-08-18T00:00:03Z".into(),
            next_safe_move: "Create a fresh work obligation and reasoning pass.".into(),
            evidence_refs: Vec::new(),
            artifact_refs: Vec::new(),
            metadata: std::collections::BTreeMap::new(),
            decision_context_id: Some(context.context_id.clone()),
        };
        let mut cache = runtime_spine_cache(&store)?;
        cache.put(&failure_role_result.job_id, &failure_role_result)?;
        cache.put(&failure_runtime_result.result_id, &failure_runtime_result)?;
        let audit = audit_decision_context(&store, &context.context_id)?;
        assert_eq!(audit.terminal_records.role_worker_results.len(), 1);
        assert_eq!(audit.terminal_records.runtime_job_results.len(), 1);
        assert_eq!(audit.terminal_records.model_pass_failures.len(), 1);
        assert_eq!(
            list_auditable_decision_contexts(&store)?.decisions[0].terminal_record_count,
            3
        );
        let mut false_success = failure_role_result.clone();
        false_success.verdict = "completed".into();
        cache.put(&false_success.job_id, &false_success)?;
        assert!(audit_decision_context(&store, &context.context_id).is_err());
        cache.put(&failure_role_result.job_id, &failure_role_result)?;

        let before = SingleFileMessagePackBackingStore::new(&store).pull_all()?;
        let mut replay = options.clone();
        replay.failed_at = "2026-08-18T00:00:04Z".into();
        assert!(crate::terminalize_model_pass_failure_session(&store, replay).is_err());
        assert_eq!(
            SingleFileMessagePackBackingStore::new(&store).pull_all()?,
            before
        );
        let mut conflict = options;
        conflict.summary = "A different failure tried to claim the context.".into();
        assert!(crate::terminalize_model_pass_failure_session(&store, conflict).is_err());
        assert_eq!(
            SingleFileMessagePackBackingStore::new(&store).pull_all()?,
            before
        );
        for hostile_kind in ["session", "job", "time", "privacy"] {
            let mut hostile = failure.clone();
            match hostile_kind {
                "session" => hostile.runtime_session_id = "foreign-session".into(),
                "job" => hostile.runtime_job_id = "foreign-job".into(),
                "time" => hostile.failed_at = "not-rfc3339".into(),
                "privacy" => hostile.private_state_exposed = true,
                _ => unreachable!(),
            }
            let mut hostile_cache = runtime_spine_cache(&store)?;
            hostile_cache.put(&hostile.failure_id, &hostile)?;
            let hostile_snapshot = SingleFileMessagePackBackingStore::new(&store).pull_all()?;
            assert!(
                crate::model_pass_failure_for_request(&store, &native.request_id).is_err(),
                "hostile {hostile_kind} failure must not project through request lookup"
            );
            assert!(
                audit_decision_context(&store, &context.context_id).is_err(),
                "hostile {hostile_kind} failure must not project through decision audit"
            );
            assert_eq!(
                SingleFileMessagePackBackingStore::new(&store).pull_all()?,
                hostile_snapshot,
                "failure inspection must remain read-only"
            );
            runtime_spine_cache(&store)?.put(&failure.failure_id, &failure)?;
        }
        Ok(())
    }

    #[test]
    fn role_basis_refuses_projection_not_derived_from_its_mind_sources() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("mind.cc");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "mind-projection-test".into(),
                display_name: "Mind projection test".into(),
                created_at: "2026-08-17T00:00:00Z".into(),
            },
        )?;
        let launch = role_launch()?;
        let mut cache = runtime_spine_cache(&store)?;
        cache.put(&launch.job_id, &launch)?;
        let canonical = worker_reasoning_basis(&store, &launch)?;
        let EpiphanyReasoningProjection::RolePass(mut forged_projection) =
            canonical.projection()?
        else {
            unreachable!();
        };
        forged_projection.mind.objective = Some("invented aggregate objective".into());
        let forged = EpiphanyReasoningBasis::new(
            &canonical.pass_id,
            &canonical.organ_id,
            &canonical.projection_policy_id,
            canonical.source_documents.clone(),
            EpiphanyReasoningProjection::RolePass(forged_projection),
        )?;
        let before = SingleFileMessagePackBackingStore::new(&store).pull_all()?;
        assert!(put_reasoning_basis(&store, &forged).is_err());
        assert_eq!(
            SingleFileMessagePackBackingStore::new(&store).pull_all()?,
            before
        );
        Ok(())
    }

    #[test]
    fn disjoint_mind_mutations_merge_and_same_identity_conflicts() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("mind.cc");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "mind-test".into(),
                display_name: "Mind test".into(),
                created_at: "2026-08-14T00:00:00Z".into(),
            },
        )?;
        let launch = role_launch()?;
        let mut cache = runtime_spine_cache(&store)?;
        cache.put(&launch.job_id, &launch)?;
        let basis = worker_reasoning_basis(&store, &launch)?;
        let basis = put_reasoning_basis(&store, &basis)?;
        let (native, _) = requests(&basis);
        crate::open_runtime_model_execution(
            &store,
            crate::RuntimeSpineSessionOptions {
                session_id: "session-1".into(),
                objective: "Test sealed decision context".into(),
                created_at: "2026-08-14T00:00:01Z".into(),
                coordinator_note: "reasoning context test".into(),
            },
            crate::RuntimeSpineJobOptions {
                job_id: "model-job-1".into(),
                session_id: "session-1".into(),
                role: "model-adapter".into(),
                created_at: "2026-08-14T00:00:01Z".into(),
            },
            &native,
            "2026-08-14T00:00:01Z",
        )?;
        let context = put_decision_context(
            &store,
            &EpiphanyDecisionContext::new(&basis, native, Vec::new())?,
        )?;
        let backing = SingleFileMessagePackBackingStore::new(&store);
        let make = |key: &str, summary: &str| -> Result<CultCacheEnvelope> {
            let cache = runtime_spine_cache(&store)?;
            let document = crate::EpiphanyMindObservationDocument {
                value: epiphany_state_model::EpiphanyObservation {
                    id: key.into(),
                    summary: summary.into(),
                    source_kind: "test".into(),
                    status: "accepted".into(),
                    code_refs: Vec::new(),
                    evidence_ids: Vec::new(),
                },
            };
            Ok(cache.prepare_entry(key, &document)?.0)
        };
        let persona_write = make("persona", "one")?;
        let first_persona_receipt = match commit_mind_mutation(
            &store,
            &context.context_id,
            "test-owner",
            Vec::new(),
            vec![persona_write.clone()],
            "2026-08-14T00:00:02Z",
        )? {
            EpiphanyMindCommitOutcome::Committed(receipt) => receipt,
            EpiphanyMindCommitOutcome::Conflict { .. } => panic!("first write must commit"),
        };
        assert_eq!(
            commit_mind_mutation(
                &store,
                &context.context_id,
                "test-owner",
                Vec::new(),
                vec![persona_write],
                "2026-08-14T00:00:20Z",
            )?,
            EpiphanyMindCommitOutcome::Committed(first_persona_receipt),
            "exact replay returns the original receipt independent of retry wall time"
        );
        assert!(matches!(
            commit_mind_mutation(
                &store,
                &context.context_id,
                "test-owner",
                Vec::new(),
                vec![make("hands", "two")?],
                "2026-08-14T00:00:03Z"
            )?,
            EpiphanyMindCommitOutcome::Committed(_)
        ));
        let before_unterminal_audit = backing.pull_all()?;
        assert!(audit_decision_context(&store, &context.context_id).is_err());
        assert_eq!(backing.pull_all()?, before_unterminal_audit);
        let terminal_result = crate::EpiphanyRuntimeJobResult {
            schema_version: crate::RUNTIME_SPINE_SCHEMA_VERSION.into(),
            result_id: "worker-result-1".into(),
            job_id: "pass-1".into(),
            session_id: "session-1".into(),
            role: "Imagination".into(),
            verdict: "completed".into(),
            summary: "Structured Imagination decision".into(),
            completed_at: "2026-08-14T00:00:03Z".into(),
            next_safe_move: "Inspect the durable decision context".into(),
            evidence_refs: Vec::new(),
            artifact_refs: Vec::new(),
            metadata: Default::default(),
            decision_context_id: Some(context.context_id.clone()),
        };
        runtime_spine_cache(&store)?.put(&terminal_result.result_id, &terminal_result)?;
        let before_audit = backing.pull_all()?;
        let audit = audit_decision_context(&store, &context.context_id)?;
        assert_eq!(audit.context_id, context.context_id);
        assert_eq!(audit.reasoning_basis, basis);
        assert_eq!(audit.terminal_native_request.request_id, "request-1");
        assert_eq!(audit.terminal_provider_request.request_id, "request-1");
        assert_eq!(
            audit.terminal_records.runtime_job_results,
            vec![terminal_result.clone()]
        );
        assert_eq!(audit.mind_commit_receipts.len(), 2);
        assert!(!audit.transcript_required);
        assert_eq!(backing.pull_all()?, before_audit);
        let mut archive_cache = runtime_spine_cache(&store)?;
        archive_cache.pull_all_backing_stores()?;
        assert!(
            archive_cache.delete::<crate::EpiphanyRuntimeJobResult>(&terminal_result.result_id)?
        );
        let archived = crate::EpiphanyArchivedRuntimeWorkerAttempt {
            schema_version: crate::ARCHIVED_RUNTIME_WORKER_ATTEMPT_SCHEMA_VERSION.into(),
            job_id: "pass-1".into(),
            request_kind: "proposal-modeling".into(),
            request_id: "proposal-request-1".into(),
            terminal_process_status: "terminal-failure".into(),
            retired_chain_digest: "sha256:historical-family".into(),
            decision: Some(crate::EpiphanyArchivedRuntimeWorkerDecision {
                decision_context_id: context.context_id.clone(),
                role_result: None,
                job_results: vec![terminal_result],
            }),
        };
        archive_cache.put(&archived.job_id, &archived)?;
        let before_archived_audit = backing.pull_all()?;
        let archived_audit = audit_decision_context(&store, &context.context_id)?;
        assert!(
            archived_audit
                .terminal_records
                .runtime_job_results
                .is_empty()
        );
        assert_eq!(
            archived_audit.terminal_records.archived_worker_attempts,
            vec![archived]
        );
        assert_eq!(backing.pull_all()?, before_archived_audit);
        let current = backing
            .pull_all()?
            .into_iter()
            .find(|entry| {
                entry.r#type == crate::EpiphanyMindObservationDocument::TYPE
                    && entry.key == "persona"
            })
            .unwrap();
        assert!(matches!(
            commit_mind_mutation(
                &store,
                &context.context_id,
                "test-owner",
                vec![current.clone()],
                vec![make("persona", "winner")?],
                "2026-08-14T00:00:04Z"
            )?,
            EpiphanyMindCommitOutcome::Committed(_)
        ));
        assert!(matches!(
            commit_mind_mutation(
                &store,
                &context.context_id,
                "test-owner",
                vec![current],
                vec![make("persona", "loser")?],
                "2026-08-14T00:00:05Z"
            )?,
            EpiphanyMindCommitOutcome::Conflict { .. }
        ));
        let hands = backing
            .pull_all()?
            .into_iter()
            .find(|entry| {
                entry.r#type == crate::EpiphanyMindObservationDocument::TYPE && entry.key == "hands"
            })
            .unwrap();
        assert!(matches!(
            commit_mind_mutation(
                &store,
                &context.context_id,
                "test-owner",
                vec![hands.clone()],
                vec![make("modeling", "read-only dependency")?],
                "2026-08-14T00:00:06Z"
            )?,
            EpiphanyMindCommitOutcome::Committed(_)
        ));
        assert!(matches!(
            commit_mind_mutation(
                &store,
                &context.context_id,
                "test-owner",
                vec![hands.clone()],
                vec![make("hands", "dependency changed")?],
                "2026-08-14T00:00:07Z"
            )?,
            EpiphanyMindCommitOutcome::Committed(_)
        ));
        assert!(matches!(
            commit_mind_mutation(
                &store,
                &context.context_id,
                "test-owner",
                vec![hands],
                vec![make("verification", "must not partially appear")?],
                "2026-08-14T00:00:08Z"
            )?,
            EpiphanyMindCommitOutcome::Conflict { .. }
        ));
        let mut cache = runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        assert!(
            cache
                .get::<crate::EpiphanyMindObservationDocument>("verification")?
                .is_none()
        );
        let _ = EpiphanyRuntimeJobStatus::Completed;
        Ok(())
    }
}
