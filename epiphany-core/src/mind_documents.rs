use std::path::Path;

use anyhow::{Result, anyhow};
use cultcache_rs::{CultCache, CultCacheEnvelope, DatabaseEntry};
use epiphany_state_model::{
    EpiphanyBacklogItem, EpiphanyEvidenceRecord, EpiphanyInvariant,
    EpiphanyInvestigationCheckpoint, EpiphanyModeState, EpiphanyObjectiveDraft,
    EpiphanyObservation, EpiphanyPlanningCapture, EpiphanyPlanningState, EpiphanyRoadmapStream,
    EpiphanySubgoal,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{EpiphanyMindDocumentVersion, runtime_spine_cache};

pub const MIND_SCHEMA_EPOCH: &str = "epiphany.mind.epoch.v1";
pub const MIND_OBJECTIVE_KEY: &str = "objective";
pub const MIND_FOCUS_KEY: &str = "focus";
pub const MIND_MODE_KEY: &str = "mode";

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
    type = "epiphany.mind.investigation_checkpoint.v1",
    schema = "EpiphanyMindInvestigationCheckpointDocument"
)]
pub struct EpiphanyMindInvestigationCheckpointDocument {
    #[cultcache(key = 0)]
    pub value: EpiphanyInvestigationCheckpoint,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.planning_capture.v1",
    schema = "EpiphanyMindPlanningCaptureDocument"
)]
pub struct EpiphanyMindPlanningCaptureDocument {
    #[cultcache(key = 0)]
    pub value: EpiphanyPlanningCapture,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.backlog_item.v1",
    schema = "EpiphanyMindBacklogItemDocument"
)]
pub struct EpiphanyMindBacklogItemDocument {
    #[cultcache(key = 0)]
    pub value: EpiphanyBacklogItem,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.roadmap_stream.v1",
    schema = "EpiphanyMindRoadmapStreamDocument"
)]
pub struct EpiphanyMindRoadmapStreamDocument {
    #[cultcache(key = 0)]
    pub value: EpiphanyRoadmapStream,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.objective_draft.v1",
    schema = "EpiphanyMindObjectiveDraftDocument"
)]
pub struct EpiphanyMindObjectiveDraftDocument {
    #[cultcache(key = 0)]
    pub value: EpiphanyObjectiveDraft,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EpiphanyMindView {
    pub projection_digest: String,
    pub source_documents: Vec<EpiphanyMindDocumentVersion>,
    pub objective: Option<String>,
    pub active_subgoal_id: Option<String>,
    pub subgoals: Vec<EpiphanySubgoal>,
    pub invariants: Vec<EpiphanyInvariant>,
    pub observations: Vec<EpiphanyObservation>,
    pub evidence: Vec<EpiphanyEvidenceRecord>,
    pub investigation_checkpoint: Option<EpiphanyInvestigationCheckpoint>,
    pub mode: Option<EpiphanyModeState>,
    pub planning: EpiphanyPlanningState,
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
    cache.register_entry_type::<EpiphanyMindInvestigationCheckpointDocument>()?;
    cache.register_entry_type::<EpiphanyMindPlanningCaptureDocument>()?;
    cache.register_entry_type::<EpiphanyMindBacklogItemDocument>()?;
    cache.register_entry_type::<EpiphanyMindRoadmapStreamDocument>()?;
    cache.register_entry_type::<EpiphanyMindObjectiveDraftDocument>()?;
    crate::repo_model_documents::register_repo_model_document_types(cache)?;
    cache.register_entry_type::<crate::RepoFrontierPlanDecisionReceipt>()?;
    cache.register_entry_type::<crate::RepoFrontierRelinquishmentReceipt>()?;
    cache.register_entry_type::<crate::RepoFrontierExecutionAmendmentReceipt>()?;
    cache.register_entry_type::<crate::AtlasSurfaceOffer>()?;
    cache.register_entry_type::<crate::AtlasDependencyClaim>()?;
    cache.register_entry_type::<crate::AtlasDependencyVerification>()?;
    cache.register_entry_type::<crate::AtlasDependencyImpact>()?;
    crate::register_current_work_types(cache)?;
    Ok(())
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
    } else if envelope.r#type == crate::RepoFrontierRelinquishmentReceipt::TYPE {
        rmp_serde::from_slice::<crate::RepoFrontierRelinquishmentReceipt>(&envelope.payload)?
            .receipt_id
    } else if envelope.r#type == crate::RepoFrontierExecutionAmendmentReceipt::TYPE {
        rmp_serde::from_slice::<crate::RepoFrontierExecutionAmendmentReceipt>(&envelope.payload)?
            .receipt_id
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
    } else if envelope.r#type == EpiphanyMindInvestigationCheckpointDocument::TYPE {
        rmp_serde::from_slice::<EpiphanyMindInvestigationCheckpointDocument>(&envelope.payload)?
            .value
            .checkpoint_id
    } else if envelope.r#type == EpiphanyMindPlanningCaptureDocument::TYPE {
        rmp_serde::from_slice::<EpiphanyMindPlanningCaptureDocument>(&envelope.payload)?
            .value
            .id
    } else if envelope.r#type == EpiphanyMindBacklogItemDocument::TYPE {
        rmp_serde::from_slice::<EpiphanyMindBacklogItemDocument>(&envelope.payload)?
            .value
            .id
    } else if envelope.r#type == EpiphanyMindRoadmapStreamDocument::TYPE {
        rmp_serde::from_slice::<EpiphanyMindRoadmapStreamDocument>(&envelope.payload)?
            .value
            .id
    } else if envelope.r#type == EpiphanyMindObjectiveDraftDocument::TYPE {
        rmp_serde::from_slice::<EpiphanyMindObjectiveDraftDocument>(&envelope.payload)?
            .value
            .id
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
    let checkpoints =
        values::<EpiphanyMindInvestigationCheckpointDocument, _>(&cache, |value| value.value)?;
    let mut captures =
        values::<EpiphanyMindPlanningCaptureDocument, _>(&cache, |value| value.value)?;
    let mut backlog_items =
        values::<EpiphanyMindBacklogItemDocument, _>(&cache, |value| value.value)?;
    let mut roadmap_streams =
        values::<EpiphanyMindRoadmapStreamDocument, _>(&cache, |value| value.value)?;
    let mut objective_drafts =
        values::<EpiphanyMindObjectiveDraftDocument, _>(&cache, |value| value.value)?;
    let repo_model = if cache
        .get::<crate::EpiphanyRepoModelIdentityDocument>(crate::REPO_MODEL_IDENTITY_KEY)?
        .is_some()
    {
        Some(crate::assemble_repo_model_view(store_path.as_ref())?)
    } else {
        None
    };
    subgoals.sort_by(|left, right| left.id.cmp(&right.id));
    invariants.sort_by(|left, right| left.id.cmp(&right.id));
    observations.sort_by(|left, right| left.id.cmp(&right.id));
    evidence.sort_by(|left, right| left.id.cmp(&right.id));
    captures.sort_by(|left, right| left.id.cmp(&right.id));
    backlog_items.sort_by(|left, right| left.id.cmp(&right.id));
    roadmap_streams.sort_by(|left, right| left.id.cmp(&right.id));
    objective_drafts.sort_by(|left, right| left.id.cmp(&right.id));
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
    source_documents.sort_by(|left, right| {
        (&left.document_type, &left.document_key).cmp(&(&right.document_type, &right.document_key))
    });
    let projection_digest = digest_versions(&source_documents)?;
    Ok(EpiphanyMindView {
        projection_digest,
        source_documents,
        objective,
        active_subgoal_id: focus.and_then(|document| document.active_subgoal_id),
        subgoals,
        invariants,
        observations,
        evidence,
        investigation_checkpoint,
        mode,
        planning: EpiphanyPlanningState {
            workspace_root: None,
            captures,
            backlog_items,
            roadmap_streams,
            objective_drafts,
        },
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
        .filter(|envelope| envelope.r#type.starts_with("epiphany.mind."))
        .map(|envelope| EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", envelope))
        .collect()
}

fn digest_versions(versions: &[EpiphanyMindDocumentVersion]) -> Result<String> {
    let mut digest = Sha256::new();
    digest.update(b"epiphany.mind_view.v1\0");
    digest.update(rmp_serde::to_vec_named(versions)?);
    Ok(format!("sha256:{:x}", digest.finalize()))
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
