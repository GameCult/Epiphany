use std::path::Path;

use anyhow::{Result, anyhow};
use cultcache_rs::{CultCache, CultCacheEnvelope, DatabaseEntry};
use crate::state_model::{
    EpiphanyEvidenceRecord, EpiphanyInvariant, EpiphanyInvestigationCheckpoint,
    EpiphanyModeState, EpiphanyObservation, EpiphanySubgoal,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{EpiphanyMindDocumentVersion, runtime_spine_cache};

pub const MIND_SCHEMA_EPOCH: &str = "epiphany.mind.epoch.v11";
pub const MIND_OBJECTIVE_KEY: &str = "objective";
pub const MIND_FOCUS_KEY: &str = "focus";
pub const MIND_MODE_KEY: &str = "mode";

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.repository_body_observation.v1",
    schema = "EpiphanyMindRepositoryBodyObservationDocument"
)]
pub struct EpiphanyMindRepositoryBodyObservationDocument {
    #[cultcache(key = 0)]
    pub basis: crate::RepositoryBodyObservationBasis,
    #[cultcache(key = 1)]
    pub source_observation: EpiphanyMindDocumentVersion,
}

impl EpiphanyMindRepositoryBodyObservationDocument {
    pub fn validate(&self) -> Result<()> {
        self.source_observation.validate()?;
        if self.source_observation.store_id != "epiphany-repository-body"
            || self.source_observation.document_type != crate::BODY_OBSERVATION_TYPE
            || self.source_observation.document_key != self.basis.observation_id
        {
            return Err(anyhow!(
                "Mind Body observation does not bind its exact external observation"
            ));
        }
        let source: crate::RepositoryBodyObservation =
            rmp_serde::from_slice(&self.source_observation.payload_msgpack)?;
        if self.basis.schema_version != crate::BODY_SCHEMA_VERSION
            || source.schema_version != self.basis.schema_version
            || source.observation_id != self.basis.observation_id
            || source.workspace_id != self.basis.workspace_id
            || source.swarm_id != self.basis.swarm_id
            || source.runtime_id != self.basis.runtime_id
            || source.scope != self.basis.scope
            || source.generation != self.basis.generation
            || source.manifest_root_sha256 != self.basis.manifest_root_sha256
            || source.scan_started_at != self.basis.scan_started_at
            || source.scan_finished_at != self.basis.scan_finished_at
        {
            return Err(anyhow!(
                "Mind Body observation basis diverges from its exact source payload"
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.mind.identity.v1", schema = "EpiphanyMindIdentity")]
pub struct EpiphanyMindIdentity {
    #[cultcache(key = 0)]
    pub schema_epoch: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.objective.v1",
    schema = "EpiphanyMindObjectiveDocument"
)]
pub struct EpiphanyMindObjectiveDocument {
    #[cultcache(key = 0)]
    pub objective: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.mind.focus.v1", schema = "EpiphanyMindFocusDocument")]
pub struct EpiphanyMindFocusDocument {
    #[cultcache(key = 0)]
    pub active_subgoal_id: Option<String>,
    #[cultcache(key = 1)]
    pub investigation_checkpoint_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(type = "epiphany.mind.mode.v1", schema = "EpiphanyMindModeDocument")]
pub struct EpiphanyMindModeDocument {
    #[cultcache(key = 0)]
    pub mode: EpiphanyModeState,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.subgoal.v1",
    schema = "EpiphanyMindSubgoalDocument"
)]
pub struct EpiphanyMindSubgoalDocument {
    #[cultcache(key = 0)]
    pub value: EpiphanySubgoal,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.invariant.v1",
    schema = "EpiphanyMindInvariantDocument"
)]
pub struct EpiphanyMindInvariantDocument {
    #[cultcache(key = 0)]
    pub value: EpiphanyInvariant,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.observation.v1",
    schema = "EpiphanyMindObservationDocument"
)]
pub struct EpiphanyMindObservationDocument {
    #[cultcache(key = 0)]
    pub value: EpiphanyObservation,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.evidence.v1",
    schema = "EpiphanyMindEvidenceDocument"
)]
pub struct EpiphanyMindEvidenceDocument {
    #[cultcache(key = 0)]
    pub value: EpiphanyEvidenceRecord,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.persona_memory.v2",
    schema = "EpiphanyMindPersonaMemoryDocument"
)]
pub struct EpiphanyMindPersonaMemoryDocument {
    #[cultcache(key = 0)]
    pub memory_id: String,
    #[cultcache(key = 1)]
    pub agent_id: String,
    #[cultcache(key = 2)]
    pub memory_kind: String,
    #[cultcache(key = 3)]
    pub summary: String,
    #[cultcache(key = 4)]
    pub salience: f64,
    #[cultcache(key = 5)]
    pub confidence: f64,
    #[cultcache(key = 6)]
    pub effect_document_id: String,
    #[cultcache(key = 7)]
    pub decision_context_id: String,
}

impl EpiphanyMindPersonaMemoryDocument {
    pub fn validate(&self) -> Result<()> {
        if [
            self.memory_id.as_str(),
            self.agent_id.as_str(),
            self.memory_kind.as_str(),
            self.summary.as_str(),
            self.effect_document_id.as_str(),
            self.decision_context_id.as_str(),
        ]
        .into_iter()
        .any(str::is_empty)
            || !matches!(self.memory_kind.as_str(), "memory" | "social_read" | "bond")
            || !(0.0..=1.0).contains(&self.salience)
            || !(0.0..=1.0).contains(&self.confidence)
        {
            return Err(anyhow!("Mind Persona memory document is invalid"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.persona_pass_input.v1",
    schema = "EpiphanyMindPersonaPassInputDocument"
)]
pub struct EpiphanyMindPersonaPassInputDocument {
    #[cultcache(key = 0)]
    pub turn_id: String,
    #[cultcache(key = 1)]
    pub projector_input: crate::PersonaProjectorInput,
    #[cultcache(key = 2)]
    pub transcript: Vec<crate::PersonaTranscriptMessage>,
    #[cultcache(key = 3)]
    pub allowed_channel_ids: Vec<String>,
    #[cultcache(key = 4)]
    pub observed_sources: Vec<EpiphanyMindDocumentVersion>,
    #[cultcache(key = 5)]
    pub admitted_at: String,
}

impl EpiphanyMindPersonaPassInputDocument {
    pub fn validate(&self) -> Result<()> {
        if self.turn_id.trim().is_empty()
            || self.projector_input.identity.identity_id.trim().is_empty()
            || self.admitted_at.trim().is_empty()
        {
            return Err(anyhow!("Mind Persona pass input is incomplete"));
        }
        chrono::DateTime::parse_from_rfc3339(&self.admitted_at)
            .map_err(|error| anyhow!("Mind Persona pass input time is invalid: {error}"))?;
        for source in &self.observed_sources {
            source.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.verification_audit.v1",
    schema = "EpiphanyMindVerificationAuditDocument"
)]
pub struct EpiphanyMindVerificationAuditDocument {
    #[cultcache(key = 0)]
    pub audit_id: String,
    #[cultcache(key = 1)]
    pub verification_request_id: String,
    #[cultcache(key = 2)]
    pub frontier_route_id: String,
    #[cultcache(key = 3)]
    pub job_id: String,
    #[cultcache(key = 4)]
    pub result_id: String,
    #[cultcache(key = 5)]
    pub decision_context_id: String,
    #[cultcache(key = 6)]
    pub verdict: String,
    #[cultcache(key = 7)]
    pub summary: String,
    #[cultcache(key = 8)]
    pub evidence_ids: Vec<String>,
    #[cultcache(key = 9)]
    pub risks: Vec<String>,
    #[cultcache(key = 10)]
    pub audited_at: String,
}

impl EpiphanyMindVerificationAuditDocument {
    pub fn validate(&self) -> Result<()> {
        if [
            self.audit_id.as_str(),
            self.verification_request_id.as_str(),
            self.frontier_route_id.as_str(),
            self.job_id.as_str(),
            self.result_id.as_str(),
            self.decision_context_id.as_str(),
            self.verdict.as_str(),
            self.summary.as_str(),
        ]
        .into_iter()
        .any(str::is_empty)
            || !matches!(
                self.verdict.trim().to_ascii_lowercase().as_str(),
                "pass" | "needs-review" | "needs-evidence" | "fail"
            )
            || chrono::DateTime::parse_from_rfc3339(&self.audited_at).is_err()
        {
            return Err(anyhow!("Verification audit document is invalid"));
        }
        let mut evidence_ids = self.evidence_ids.clone();
        evidence_ids.sort();
        evidence_ids.dedup();
        if evidence_ids != self.evidence_ids {
            return Err(anyhow!("Verification audit evidence ids are not canonical"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.investigation_checkpoint.v1",
    schema = "EpiphanyMindInvestigationCheckpointDocument"
)]
pub struct EpiphanyMindInvestigationCheckpointDocument {
    #[cultcache(key = 0)]
    pub value: EpiphanyInvestigationCheckpoint,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EpiphanyMindView {
    pub schema_epoch: String,
    pub runtime_id: String,
    pub projection_digest: String,
    pub source_documents: Vec<EpiphanyMindDocumentVersion>,
    pub objective: Option<String>,
    pub active_subgoal_id: Option<String>,
    pub subgoals: Vec<EpiphanySubgoal>,
    pub invariants: Vec<EpiphanyInvariant>,
    pub observations: Vec<EpiphanyObservation>,
    pub evidence: Vec<EpiphanyEvidenceRecord>,
    pub persona_memories: Vec<EpiphanyMindPersonaMemoryDocument>,
    pub verification_audits: Vec<EpiphanyMindVerificationAuditDocument>,
    pub reorientation_decisions: Vec<crate::EpiphanyMindReorientationDecisionDocument>,
    pub reorientation_failures: Vec<crate::EpiphanyMindReorientationPassFailureDocument>,
    pub agent_pass_admission_refusals: Vec<crate::EpiphanyAgentPassAdmissionRefusal>,
    pub investigation_checkpoint: Option<EpiphanyInvestigationCheckpoint>,
    pub mode: Option<EpiphanyModeState>,
    pub repository_body_observation: Option<crate::RepositoryBodyObservationBasis>,
    pub repo_model: Option<crate::EpiphanyRepoModelView>,
}

pub(crate) fn register_mind_document_types(cache: &mut CultCache) -> Result<()> {
    cache.register_entry_type::<EpiphanyMindIdentity>()?;
    cache.register_entry_type::<EpiphanyMindObjectiveDocument>()?;
    cache.register_entry_type::<EpiphanyMindFocusDocument>()?;
    cache.register_entry_type::<EpiphanyMindModeDocument>()?;
    cache.register_entry_type::<EpiphanyMindSubgoalDocument>()?;
    cache.register_entry_type::<EpiphanyMindInvariantDocument>()?;
    cache.register_entry_type::<EpiphanyMindObservationDocument>()?;
    cache.register_entry_type::<EpiphanyMindEvidenceDocument>()?;
    cache.register_entry_type::<EpiphanyMindPersonaMemoryDocument>()?;
    cache.register_entry_type::<EpiphanyMindPersonaPassInputDocument>()?;
    cache.register_entry_type::<EpiphanyMindVerificationAuditDocument>()?;
    cache.register_entry_type::<crate::EpiphanyMindReorientationDecisionDocument>()?;
    cache.register_entry_type::<crate::EpiphanyMindReorientationPassFailureDocument>()?;
    cache.register_entry_type::<EpiphanyMindInvestigationCheckpointDocument>()?;
    cache.register_entry_type::<EpiphanyMindRepositoryBodyObservationDocument>()?;
    crate::repo_model_documents::register_repo_model_document_types(cache)?;
    cache.register_entry_type::<crate::RepoFrontierPlanDecisionReceipt>()?;
    cache.register_entry_type::<crate::AtlasSurfaceOffer>()?;
    cache.register_entry_type::<crate::AtlasDependencyClaim>()?;
    cache.register_entry_type::<crate::AtlasDependencyVerification>()?;
    cache.register_entry_type::<crate::AtlasDependencyImpact>()?;
    crate::register_current_work_types(cache)?;
    Ok(())
}

pub(crate) fn prepare_mind_document<T: DatabaseEntry>(
    cache: &CultCache,
    key: impl Into<String>,
    value: &T,
) -> Result<CultCacheEnvelope> {
    if !T::TYPE.starts_with("epiphany.mind.") {
        return Err(anyhow!(
            "Mind document preparation refuses non-Mind type {:?}",
            T::TYPE
        ));
    }
    Ok(cache.prepare_entry_named(key, value)?.0)
}

pub(crate) fn validate_mind_write_envelope(envelope: &CultCacheEnvelope) -> Result<()> {
    if let Some(expected_key) = crate::repo_model_documents::repo_model_write_key(envelope)? {
        if envelope.key != expected_key {
            return Err(anyhow!(
                "RepoModel document semantic identity does not match its envelope key"
            ));
        }
        return Ok(());
    }
    let expected_key = if envelope.r#type == crate::RepoFrontierPlanDecisionReceipt::TYPE {
        rmp_serde::from_slice::<crate::RepoFrontierPlanDecisionReceipt>(&envelope.payload)?
            .decision_id
    } else if envelope.r#type == crate::AtlasSurfaceOffer::TYPE {
        let value: crate::AtlasSurfaceOffer = rmp_serde::from_slice(&envelope.payload)?;
        value.validate()?;
        value.surface_id.to_string()
    } else if envelope.r#type == crate::AtlasDependencyClaim::TYPE {
        let value: crate::AtlasDependencyClaim = rmp_serde::from_slice(&envelope.payload)?;
        value.validate()?;
        value.claim_id.to_string()
    } else if envelope.r#type == crate::AtlasDependencyVerification::TYPE {
        let value: crate::AtlasDependencyVerification = rmp_serde::from_slice(&envelope.payload)?;
        value.validate()?;
        value.claim_id.to_string()
    } else if envelope.r#type == crate::AtlasDependencyImpact::TYPE {
        let value: crate::AtlasDependencyImpact = rmp_serde::from_slice(&envelope.payload)?;
        value.validate()?;
        value.impact_id.to_string()
    } else if envelope.r#type == crate::EpiphanyBodyModelingDecisionReceipt::TYPE {
        let value: crate::EpiphanyBodyModelingDecisionReceipt =
            rmp_serde::from_slice(&envelope.payload)?;
        let work = crate::EpiphanyBodyModelingWorkProjection::derive(
            value.runtime_id.clone(),
            value.body_basis.clone(),
            crate::EpiphanyRepoModelBasis {
                projection_digest: value.repo_model_projection_digest.clone(),
                source_documents: value.repo_model_source_documents.clone(),
            },
        )?;
        value.validate(&work)?;
        value.work_id
    } else if envelope.r#type == crate::EpiphanyAgentPassAdmissionRefusal::TYPE {
        let value: crate::EpiphanyAgentPassAdmissionRefusal =
            rmp_serde::from_slice(&envelope.payload)?;
        value.validate()?;
        value.refusal_id
    } else if envelope.r#type == EpiphanyMindIdentity::TYPE {
        let value: EpiphanyMindIdentity = rmp_serde::from_slice(&envelope.payload)?;
        if value.schema_epoch != MIND_SCHEMA_EPOCH || value.runtime_id.trim().is_empty() {
            return Err(anyhow!("Mind identity write is invalid"));
        }
        MIND_SCHEMA_EPOCH.to_string()
    } else if envelope.r#type == EpiphanyMindObjectiveDocument::TYPE {
        let value: EpiphanyMindObjectiveDocument = rmp_serde::from_slice(&envelope.payload)?;
        if value.objective.trim().is_empty() {
            return Err(anyhow!("Mind objective cannot be empty"));
        }
        MIND_OBJECTIVE_KEY.to_string()
    } else if envelope.r#type == EpiphanyMindFocusDocument::TYPE {
        let _: EpiphanyMindFocusDocument = rmp_serde::from_slice(&envelope.payload)?;
        MIND_FOCUS_KEY.to_string()
    } else if envelope.r#type == EpiphanyMindModeDocument::TYPE {
        let _: EpiphanyMindModeDocument = rmp_serde::from_slice(&envelope.payload)?;
        MIND_MODE_KEY.to_string()
    } else if envelope.r#type == EpiphanyMindSubgoalDocument::TYPE {
        rmp_serde::from_slice::<EpiphanyMindSubgoalDocument>(&envelope.payload)?
            .value
            .id
    } else if envelope.r#type == EpiphanyMindInvariantDocument::TYPE {
        rmp_serde::from_slice::<EpiphanyMindInvariantDocument>(&envelope.payload)?
            .value
            .id
    } else if envelope.r#type == EpiphanyMindObservationDocument::TYPE {
        rmp_serde::from_slice::<EpiphanyMindObservationDocument>(&envelope.payload)?
            .value
            .id
    } else if envelope.r#type == EpiphanyMindEvidenceDocument::TYPE {
        rmp_serde::from_slice::<EpiphanyMindEvidenceDocument>(&envelope.payload)?
            .value
            .id
    } else if envelope.r#type == EpiphanyMindPersonaMemoryDocument::TYPE {
        let value: EpiphanyMindPersonaMemoryDocument = rmp_serde::from_slice(&envelope.payload)?;
        value.validate()?;
        value.memory_id
    } else if envelope.r#type == EpiphanyMindPersonaPassInputDocument::TYPE {
        let value: EpiphanyMindPersonaPassInputDocument = rmp_serde::from_slice(&envelope.payload)?;
        value.validate()?;
        value.turn_id
    } else if envelope.r#type == EpiphanyMindVerificationAuditDocument::TYPE {
        let value: EpiphanyMindVerificationAuditDocument =
            rmp_serde::from_slice(&envelope.payload)?;
        value.validate()?;
        value.audit_id
    } else if envelope.r#type == crate::EpiphanyMindReorientationDecisionDocument::TYPE {
        let value: crate::EpiphanyMindReorientationDecisionDocument =
            rmp_serde::from_slice(&envelope.payload)?;
        if value.schema_version != crate::MIND_REORIENTATION_DECISION_SCHEMA_VERSION
            || value.decision_id.trim().is_empty()
            || value.request_id.trim().is_empty()
            || value.decision_context_id.trim().is_empty()
        {
            return Err(anyhow!("Mind reorientation decision is invalid"));
        }
        value.decision_id
    } else if envelope.r#type == crate::EpiphanyMindReorientationPassFailureDocument::TYPE {
        let value: crate::EpiphanyMindReorientationPassFailureDocument =
            rmp_serde::from_slice(&envelope.payload)?;
        if value.schema_version != crate::MIND_REORIENTATION_PASS_FAILURE_SCHEMA_VERSION
            || value.failure_id.trim().is_empty()
            || value.request_id.trim().is_empty()
            || value.decision_context_id.trim().is_empty()
        {
            return Err(anyhow!("Mind reorientation failure is invalid"));
        }
        value.failure_id
    } else if envelope.r#type == EpiphanyMindInvestigationCheckpointDocument::TYPE {
        rmp_serde::from_slice::<EpiphanyMindInvestigationCheckpointDocument>(&envelope.payload)?
            .value
            .checkpoint_id
    } else if envelope.r#type == EpiphanyMindRepositoryBodyObservationDocument::TYPE {
        let value: EpiphanyMindRepositoryBodyObservationDocument =
            rmp_serde::from_slice(&envelope.payload)?;
        value.validate()?;
        value.basis.observation_id
    } else {
        return Err(anyhow!(
            "Mind mutation cannot write non-canonical document type {:?}",
            envelope.r#type
        ));
    };
    if expected_key.trim().is_empty() || envelope.key != expected_key {
        return Err(anyhow!(
            "Mind document semantic identity does not match its envelope key"
        ));
    }
    Ok(())
}

pub fn assemble_mind_view(store_path: impl AsRef<Path>) -> Result<EpiphanyMindView> {
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    assemble_mind_view_from_cache(&cache)
}

pub(crate) fn assemble_mind_view_from_cache(cache: &CultCache) -> Result<EpiphanyMindView> {
    let identity = cache
        .get::<EpiphanyMindIdentity>(MIND_SCHEMA_EPOCH)?
        .ok_or_else(|| anyhow!("writable Mind store has no v1 identity"))?;
    if identity.schema_epoch != MIND_SCHEMA_EPOCH {
        return Err(anyhow!(
            "writable Mind store has an unsupported schema epoch"
        ));
    }

    let objective = cache
        .get::<EpiphanyMindObjectiveDocument>(MIND_OBJECTIVE_KEY)?
        .map(|document| document.objective);
    let focus = cache.get::<EpiphanyMindFocusDocument>(MIND_FOCUS_KEY)?;
    let mode = cache
        .get::<EpiphanyMindModeDocument>(MIND_MODE_KEY)?
        .map(|document| document.mode);
    let mut subgoals = values::<EpiphanyMindSubgoalDocument, _>(&cache, |value| value.value)?;
    let mut invariants = values::<EpiphanyMindInvariantDocument, _>(&cache, |value| value.value)?;
    let mut observations =
        values::<EpiphanyMindObservationDocument, _>(&cache, |value| value.value)?;
    let mut evidence = values::<EpiphanyMindEvidenceDocument, _>(&cache, |value| value.value)?;
    let mut persona_memories = cache.get_all::<EpiphanyMindPersonaMemoryDocument>()?;
    for memory in &persona_memories {
        memory.validate()?;
    }
    let mut verification_audits = cache.get_all::<EpiphanyMindVerificationAuditDocument>()?;
    let mut reorientation_decisions =
        cache.get_all::<crate::EpiphanyMindReorientationDecisionDocument>()?;
    let mut reorientation_failures =
        cache.get_all::<crate::EpiphanyMindReorientationPassFailureDocument>()?;
    let mut agent_pass_admission_refusals =
        cache.get_all::<crate::EpiphanyAgentPassAdmissionRefusal>()?;
    for refusal in &agent_pass_admission_refusals {
        refusal.validate()?;
    }
    let checkpoints =
        values::<EpiphanyMindInvestigationCheckpointDocument, _>(&cache, |value| value.value)?;
    let mut body_observations = cache.get_all::<EpiphanyMindRepositoryBodyObservationDocument>()?;
    for observation in &body_observations {
        observation.validate()?;
        if observation.basis.runtime_id != identity.runtime_id {
            return Err(anyhow!("Mind Body observation belongs to another runtime"));
        }
    }
    body_observations.sort_by_key(|value| value.basis.generation);
    if body_observations.windows(2).any(|pair| {
        pair[0].basis.generation == pair[1].basis.generation
            || pair[0].basis.workspace_id != pair[1].basis.workspace_id
            || pair[0].basis.body_binding_sha256 != pair[1].basis.body_binding_sha256
    }) {
        return Err(anyhow!(
            "Mind Body observation chain is not singular and ordered"
        ));
    }
    let repository_body_observation = body_observations.last().map(|value| value.basis.clone());
    let repo_model = if cache
        .get::<crate::EpiphanyRepoModelIdentityDocument>(crate::REPO_MODEL_IDENTITY_KEY)?
        .is_some()
    {
        Some(crate::repo_model_documents::assemble_repo_model_view_from_cache(cache)?)
    } else {
        None
    };
    subgoals.sort_by(|left, right| left.id.cmp(&right.id));
    invariants.sort_by(|left, right| left.id.cmp(&right.id));
    observations.sort_by(|left, right| left.id.cmp(&right.id));
    evidence.sort_by(|left, right| left.id.cmp(&right.id));
    persona_memories.sort_by(|left, right| left.memory_id.cmp(&right.memory_id));
    verification_audits.sort_by(|left, right| left.audit_id.cmp(&right.audit_id));
    reorientation_decisions.sort_by(|left, right| left.decision_id.cmp(&right.decision_id));
    reorientation_failures.sort_by(|left, right| left.failure_id.cmp(&right.failure_id));
    agent_pass_admission_refusals.sort_by(|left, right| left.refusal_id.cmp(&right.refusal_id));
    let investigation_checkpoint = match focus
        .as_ref()
        .and_then(|document| document.investigation_checkpoint_id.as_deref())
    {
        Some(id) => Some(
            checkpoints
                .into_iter()
                .find(|checkpoint| checkpoint.checkpoint_id == id)
                .ok_or_else(|| anyhow!("Mind focus cites a missing investigation checkpoint"))?,
        ),
        None => None,
    };
    let mut source_documents = canonical_mind_versions(&cache.snapshot_envelopes())?;
    if let Some(repo_model) = &repo_model {
        source_documents.extend(repo_model.source_documents.clone());
    }
    source_documents.sort_by(|left, right| {
        (&left.document_type, &left.document_key).cmp(&(&right.document_type, &right.document_key))
    });
    if source_documents
        .windows(2)
        .any(|pair| pair[0].identity() == pair[1].identity())
    {
        return Err(anyhow!("Mind view repeats a canonical source document"));
    }
    let projection_digest = mind_view_digest(&source_documents)?;
    Ok(EpiphanyMindView {
        schema_epoch: identity.schema_epoch,
        runtime_id: identity.runtime_id,
        projection_digest,
        source_documents,
        objective,
        active_subgoal_id: focus.and_then(|document| document.active_subgoal_id),
        subgoals,
        invariants,
        observations,
        evidence,
        persona_memories,
        verification_audits,
        reorientation_decisions,
        reorientation_failures,
        agent_pass_admission_refusals,
        investigation_checkpoint,
        mode,
        repository_body_observation,
        repo_model,
    })
}

fn values<T, V>(cache: &CultCache, take: impl Fn(T) -> V) -> Result<Vec<V>>
where
    T: DatabaseEntry,
{
    Ok(cache.get_all::<T>()?.into_iter().map(take).collect())
}

fn canonical_mind_versions(
    envelopes: &[CultCacheEnvelope],
) -> Result<Vec<EpiphanyMindDocumentVersion>> {
    envelopes
        .iter()
        .filter(|envelope| {
            matches!(
                envelope.r#type.as_str(),
                EpiphanyMindIdentity::TYPE
                    | EpiphanyMindObjectiveDocument::TYPE
                    | EpiphanyMindFocusDocument::TYPE
                    | EpiphanyMindModeDocument::TYPE
                    | EpiphanyMindSubgoalDocument::TYPE
                    | EpiphanyMindInvariantDocument::TYPE
                    | EpiphanyMindObservationDocument::TYPE
                    | EpiphanyMindEvidenceDocument::TYPE
                    | EpiphanyMindPersonaMemoryDocument::TYPE
                    | EpiphanyMindPersonaPassInputDocument::TYPE
                    | EpiphanyMindVerificationAuditDocument::TYPE
                    | crate::EpiphanyMindReorientationDecisionDocument::TYPE
                    | crate::EpiphanyMindReorientationPassFailureDocument::TYPE
                    | crate::EpiphanyAgentPassAdmissionRefusal::TYPE
                    | EpiphanyMindInvestigationCheckpointDocument::TYPE
                    | EpiphanyMindRepositoryBodyObservationDocument::TYPE
            )
        })
        .map(|envelope| EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", envelope))
        .collect()
}

pub(crate) fn mind_view_digest(versions: &[EpiphanyMindDocumentVersion]) -> Result<String> {
    let mut digest = Sha256::new();
    digest.update(b"epiphany.mind_view.v1\0");
    digest.update(rmp_serde::to_vec_named(versions)?);
    Ok(format!("sha256:{:x}", digest.finalize()))
}

pub fn epiphany_mind_projection_digest(versions: &[EpiphanyMindDocumentVersion]) -> Result<String> {
    mind_view_digest(versions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RuntimeSpineInitOptions, initialize_runtime_spine};

    #[test]
    fn keyed_mind_view_is_deterministic_and_has_no_global_revision() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("mind.cc");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "mind-view".into(),
                display_name: "Mind view".into(),
                created_at: "2026-08-14T00:00:00Z".into(),
            },
        )?;
        let mut cache = runtime_spine_cache(&store)?;
        cache.put(
            MIND_OBJECTIVE_KEY,
            &EpiphanyMindObjectiveDocument {
                objective: "Build a concurrent Mind".into(),
            },
        )?;
        for id in ["subgoal-b", "subgoal-a"] {
            cache.put(
                id,
                &EpiphanyMindSubgoalDocument {
                    value: EpiphanySubgoal {
                        id: id.into(),
                        title: id.into(),
                        status: "active".into(),
                        summary: None,
                    },
                },
            )?;
        }
        let first = assemble_mind_view(&store)?;
        let second = assemble_mind_view(&store)?;
        assert_eq!(first, second);
        assert_eq!(first.objective.as_deref(), Some("Build a concurrent Mind"));
        assert_eq!(
            first
                .subgoals
                .iter()
                .map(|subgoal| subgoal.id.as_str())
                .collect::<Vec<_>>(),
            ["subgoal-a", "subgoal-b"]
        );
        assert!(first.projection_digest.starts_with("sha256:"));
        Ok(())
    }
}
