use anyhow::{Result, bail};
use cultcache_rs::DatabaseEntry;
use serde::{Deserialize, Serialize};

use super::contracts::*;
use super::identity::{AtlasPublisherTrustBinding, sha256, verify_atlas_publication};
use super::impact_ingress::{
    AtlasImpactBrakeState, AtlasImpactIngressPolicy, AtlasImpactLane, AtlasImpactLaneState,
    AtlasImpactProposalAuthority, AtlasLocalClaimInput, AtlasLocalImpactProposal,
    evaluate_local_atlas_impacts,
};
use super::projector::{atlas_projection_digest, evaluate_atlas_compatibility};

pub const ATLAS_SURFACE_OFFER_WRITE_INTENT_SCHEMA: &str =
    "epiphany.model.atlas.surface_offer_write_intent.v0";
pub const ATLAS_DEPENDENCY_CLAIM_WRITE_INTENT_SCHEMA: &str =
    "epiphany.model.atlas.dependency_claim_write_intent.v0";
pub const ATLAS_DEPENDENCY_VERIFICATION_WRITE_INTENT_SCHEMA: &str =
    "epiphany.soul.atlas.dependency_verification_write_intent.v0";
pub const ATLAS_DEPENDENCY_IMPACT_WRITE_INTENT_SCHEMA: &str =
    "epiphany.self.atlas.dependency_impact_write_intent.v0";

const LOCAL_MIND_STORE_ID: &str = "epiphany-mind";
const BODY_SCHEMA_VERSION: &str = "epiphany.repository_body.v2";

/// Current local Body authority passed to a pure planner. The eventual writer
/// must revalidate this exact basis before committing the returned intent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasPlannerContext {
    pub local_repository: AtlasRepositoryIdentity,
    pub current_body_basis: crate::RepositoryBodyObservationBasis,
}

/// One exact typed document version returned by a strong read. The source
/// digest is checked against the canonical typed document before it can steer
/// a plan.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasStrongRead<T> {
    pub source: AtlasMindSourceVersion,
    pub document: T,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "transition")]
pub enum AtlasSurfaceOfferTransition {
    Deprecate {
        replacement: Option<AtlasStrongRead<AtlasSurfaceOffer>>,
    },
    Withdraw,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasSurfaceOfferTransitionInput {
    pub context: AtlasPlannerContext,
    pub expected_current_source: AtlasMindSourceVersion,
    pub current: AtlasStrongRead<AtlasSurfaceOffer>,
    pub transition: AtlasSurfaceOfferTransition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasSurfaceOfferAdmissionInput {
    pub context: AtlasPlannerContext,
    pub offer: AtlasSurfaceOffer,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.model.atlas.surface_offer_write_intent.v0",
    schema = "epiphany.model.atlas.surface_offer_write_intent.v0"
)]
pub struct AtlasSurfaceOfferWriteIntent {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub intent_id: String,
    #[cultcache(key = 2)]
    pub local_repository: AtlasRepositoryIdentity,
    #[cultcache(key = 3)]
    pub body_basis: crate::RepositoryBodyObservationBasis,
    #[cultcache(key = 4)]
    pub expected_current_source: Option<AtlasMindSourceVersion>,
    #[cultcache(key = 5)]
    pub replacement_source: Option<AtlasMindSourceVersion>,
    #[cultcache(key = 6)]
    pub next_source: AtlasMindSourceVersion,
    #[cultcache(key = 7)]
    pub next: AtlasSurfaceOffer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtlasDependencyClaimTransition {
    Retire,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasDependencyClaimTransitionInput {
    pub context: AtlasPlannerContext,
    pub expected_current_source: AtlasMindSourceVersion,
    pub current: AtlasStrongRead<AtlasDependencyClaim>,
    pub transition: AtlasDependencyClaimTransition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasDependencyClaimAdmissionInput {
    pub context: AtlasPlannerContext,
    pub claim: AtlasDependencyClaim,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.model.atlas.dependency_claim_write_intent.v0",
    schema = "epiphany.model.atlas.dependency_claim_write_intent.v0"
)]
pub struct AtlasDependencyClaimWriteIntent {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub intent_id: String,
    #[cultcache(key = 2)]
    pub local_repository: AtlasRepositoryIdentity,
    #[cultcache(key = 3)]
    pub body_basis: crate::RepositoryBodyObservationBasis,
    #[cultcache(key = 4)]
    pub expected_current_source: Option<AtlasMindSourceVersion>,
    #[cultcache(key = 5)]
    pub next_source: AtlasMindSourceVersion,
    #[cultcache(key = 6)]
    pub next: AtlasDependencyClaim,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasEvidenceArtifactVersion {
    pub artifact_id: String,
    pub payload_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasEvidenceArtifactRead {
    pub version: AtlasEvidenceArtifactVersion,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasDependencyVerificationAdmissionInput {
    pub context: AtlasPlannerContext,
    pub now_unix_ms: u64,
    pub expected_claim_source: AtlasMindSourceVersion,
    pub claim: AtlasStrongRead<AtlasDependencyClaim>,
    pub claim_publication: AtlasStrongRead<AtlasPublicationEnvelope>,
    pub claim_publication_trust: AtlasPublisherTrustBinding,
    pub offer_publication: AtlasStrongRead<AtlasPublicationEnvelope>,
    pub offer_publication_trust: AtlasPublisherTrustBinding,
    pub expected_current_verification_source: Option<AtlasMindSourceVersion>,
    pub current_verification: Option<AtlasStrongRead<AtlasDependencyVerification>>,
    pub evidence: AtlasEvidenceArtifactRead,
    pub verdict: AtlasVerificationVerdict,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.soul.atlas.dependency_verification_write_intent.v0",
    schema = "epiphany.soul.atlas.dependency_verification_write_intent.v0"
)]
pub struct AtlasDependencyVerificationWriteIntent {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub intent_id: String,
    #[cultcache(key = 2)]
    pub local_repository: AtlasRepositoryIdentity,
    #[cultcache(key = 3)]
    pub body_basis: crate::RepositoryBodyObservationBasis,
    #[cultcache(key = 4)]
    pub claim_source: AtlasMindSourceVersion,
    #[cultcache(key = 5)]
    pub claim_publication_source: AtlasMindSourceVersion,
    #[cultcache(key = 6)]
    pub offer_publication_source: AtlasMindSourceVersion,
    #[cultcache(key = 7)]
    pub evidence: AtlasEvidenceArtifactVersion,
    #[cultcache(key = 8)]
    pub expected_current_source: Option<AtlasMindSourceVersion>,
    #[cultcache(key = 9)]
    pub next_source: AtlasMindSourceVersion,
    #[cultcache(key = 10)]
    pub next: AtlasDependencyVerification,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasDependencyImpactAdmissionInput {
    pub context: AtlasPlannerContext,
    pub expected_claim_source: AtlasMindSourceVersion,
    pub claim: AtlasStrongRead<AtlasDependencyClaim>,
    pub expected_projection_source: AtlasMindSourceVersion,
    pub projection: AtlasStrongRead<AtlasEntanglementProjection>,
    pub expected_current_impact_source: Option<AtlasMindSourceVersion>,
    pub current_impact: Option<AtlasStrongRead<AtlasDependencyImpact>>,
    pub proposal: AtlasLocalImpactProposal,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.self.atlas.dependency_impact_write_intent.v0",
    schema = "epiphany.self.atlas.dependency_impact_write_intent.v0"
)]
pub struct AtlasDependencyImpactWriteIntent {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub intent_id: String,
    #[cultcache(key = 2)]
    pub local_repository: AtlasRepositoryIdentity,
    #[cultcache(key = 3)]
    pub body_basis: crate::RepositoryBodyObservationBasis,
    #[cultcache(key = 4)]
    pub claim_source: AtlasMindSourceVersion,
    #[cultcache(key = 5)]
    pub projection_source: AtlasMindSourceVersion,
    #[cultcache(key = 6)]
    pub expected_current_source: Option<AtlasMindSourceVersion>,
    #[cultcache(key = 7)]
    pub next_source: AtlasMindSourceVersion,
    #[cultcache(key = 8)]
    pub proposal: AtlasLocalImpactProposal,
}

pub fn plan_surface_offer_transition(
    input: AtlasSurfaceOfferTransitionInput,
) -> Result<AtlasSurfaceOfferWriteIntent> {
    validate_context(&input.context)?;
    validate_exact_local_read(
        &input.current,
        ATLAS_SURFACE_OFFER_SCHEMA,
        &input.current.document.surface_id.to_string(),
    )?;
    require_expected_source(
        &input.expected_current_source,
        &input.current.source,
        "surface offer",
    )?;
    let current = &input.current.document;
    if current.provider != input.context.local_repository {
        bail!("Atlas surface lifecycle planner cannot write a foreign repository offer")
    }

    let (next_lifecycle, replacement_source) = match (&current.lifecycle, input.transition) {
        (AtlasOfferLifecycle::Active, AtlasSurfaceOfferTransition::Deprecate { replacement }) => {
            let replacement_source = replacement
                .as_ref()
                .map(|replacement| validate_replacement_offer(&input.context, current, replacement))
                .transpose()?;
            let replacement_surface_id = replacement
                .as_ref()
                .map(|replacement| replacement.document.surface_id);
            (
                AtlasOfferLifecycle::Deprecated {
                    replacement_surface_id,
                },
                replacement_source,
            )
        }
        (
            AtlasOfferLifecycle::Active | AtlasOfferLifecycle::Deprecated { .. },
            AtlasSurfaceOfferTransition::Withdraw,
        ) => (AtlasOfferLifecycle::Withdrawn, None),
        _ => bail!("Atlas surface offer lifecycle transition is illegal or already complete"),
    };
    let mut next = current.clone();
    next.lifecycle = next_lifecycle;
    next.validate()?;
    let next_source = local_next_source(
        &input.current.source,
        ATLAS_SURFACE_OFFER_SCHEMA,
        next.surface_id.to_string(),
        &next,
    )?;
    let mut intent = AtlasSurfaceOfferWriteIntent {
        schema_version: ATLAS_SURFACE_OFFER_WRITE_INTENT_SCHEMA.into(),
        intent_id: String::new(),
        local_repository: input.context.local_repository,
        body_basis: input.context.current_body_basis,
        expected_current_source: Some(input.current.source),
        replacement_source,
        next_source,
        next,
    };
    intent.intent_id = surface_offer_intent_id(&intent)?;
    intent.validate()?;
    Ok(intent)
}

pub fn admit_surface_offer(
    input: AtlasSurfaceOfferAdmissionInput,
) -> Result<AtlasSurfaceOfferWriteIntent> {
    validate_context(&input.context)?;
    input.offer.validate()?;
    if input.offer.provider != input.context.local_repository
        || input.offer.lifecycle != AtlasOfferLifecycle::Active
    {
        bail!("Atlas offer admission requires an active locally owned offer")
    }
    let basis_source = AtlasMindSourceVersion {
        store_id: LOCAL_MIND_STORE_ID.into(),
        document_type: ATLAS_SURFACE_OFFER_SCHEMA.into(),
        document_key: input.offer.surface_id.to_string(),
        schema_id: Some(ATLAS_SURFACE_OFFER_SCHEMA.into()),
        payload_sha256: sha256(&rmp_serde::to_vec(&input.offer)?),
    };
    let next_source = local_next_source(
        &basis_source,
        ATLAS_SURFACE_OFFER_SCHEMA,
        input.offer.surface_id.to_string(),
        &input.offer,
    )?;
    let mut intent = AtlasSurfaceOfferWriteIntent {
        schema_version: ATLAS_SURFACE_OFFER_WRITE_INTENT_SCHEMA.into(),
        intent_id: String::new(),
        local_repository: input.context.local_repository,
        body_basis: input.context.current_body_basis,
        expected_current_source: None,
        replacement_source: None,
        next_source,
        next: input.offer,
    };
    intent.intent_id = surface_offer_intent_id(&intent)?;
    intent.validate()?;
    Ok(intent)
}

pub fn plan_dependency_claim_transition(
    input: AtlasDependencyClaimTransitionInput,
) -> Result<AtlasDependencyClaimWriteIntent> {
    validate_context(&input.context)?;
    validate_exact_local_read(
        &input.current,
        ATLAS_DEPENDENCY_CLAIM_SCHEMA,
        &input.current.document.claim_id.to_string(),
    )?;
    require_expected_source(
        &input.expected_current_source,
        &input.current.source,
        "dependency claim",
    )?;
    let current = &input.current.document;
    if current.consumer != input.context.local_repository {
        bail!("Atlas claim lifecycle planner cannot write a foreign repository claim")
    }
    let next_lifecycle = match (current.lifecycle, input.transition) {
        (AtlasClaimLifecycle::Active, AtlasDependencyClaimTransition::Retire) => {
            AtlasClaimLifecycle::Retired
        }
        _ => bail!("Atlas dependency claim lifecycle transition is illegal or already complete"),
    };
    let mut next = current.clone();
    next.lifecycle = next_lifecycle;
    next.validate()?;
    let next_source = local_next_source(
        &input.current.source,
        ATLAS_DEPENDENCY_CLAIM_SCHEMA,
        next.claim_id.to_string(),
        &next,
    )?;
    let mut intent = AtlasDependencyClaimWriteIntent {
        schema_version: ATLAS_DEPENDENCY_CLAIM_WRITE_INTENT_SCHEMA.into(),
        intent_id: String::new(),
        local_repository: input.context.local_repository,
        body_basis: input.context.current_body_basis,
        expected_current_source: Some(input.current.source),
        next_source,
        next,
    };
    intent.intent_id = dependency_claim_intent_id(&intent)?;
    intent.validate()?;
    Ok(intent)
}

pub fn admit_dependency_claim(
    input: AtlasDependencyClaimAdmissionInput,
) -> Result<AtlasDependencyClaimWriteIntent> {
    validate_context(&input.context)?;
    input.claim.validate()?;
    if input.claim.consumer != input.context.local_repository
        || input.claim.lifecycle != AtlasClaimLifecycle::Active
    {
        bail!("Atlas claim admission requires an active locally owned claim")
    }
    let basis_source = AtlasMindSourceVersion {
        store_id: LOCAL_MIND_STORE_ID.into(),
        document_type: ATLAS_DEPENDENCY_CLAIM_SCHEMA.into(),
        document_key: input.claim.claim_id.to_string(),
        schema_id: Some(ATLAS_DEPENDENCY_CLAIM_SCHEMA.into()),
        payload_sha256: sha256(&rmp_serde::to_vec(&input.claim)?),
    };
    let next_source = local_next_source(
        &basis_source,
        ATLAS_DEPENDENCY_CLAIM_SCHEMA,
        input.claim.claim_id.to_string(),
        &input.claim,
    )?;
    let mut intent = AtlasDependencyClaimWriteIntent {
        schema_version: ATLAS_DEPENDENCY_CLAIM_WRITE_INTENT_SCHEMA.into(),
        intent_id: String::new(),
        local_repository: input.context.local_repository,
        body_basis: input.context.current_body_basis,
        expected_current_source: None,
        next_source,
        next: input.claim,
    };
    intent.intent_id = dependency_claim_intent_id(&intent)?;
    intent.validate()?;
    Ok(intent)
}

pub fn admit_dependency_verification(
    input: AtlasDependencyVerificationAdmissionInput,
) -> Result<AtlasDependencyVerificationWriteIntent> {
    validate_context(&input.context)?;
    validate_exact_local_read(
        &input.claim,
        ATLAS_DEPENDENCY_CLAIM_SCHEMA,
        &input.claim.document.claim_id.to_string(),
    )?;
    require_expected_source(
        &input.expected_claim_source,
        &input.claim.source,
        "verification claim",
    )?;
    if input.claim.document.consumer != input.context.local_repository
        || input.claim.document.lifecycle != AtlasClaimLifecycle::Active
    {
        bail!("Atlas Soul verification requires an active exact local claim")
    }

    validate_publication_read(&input.claim_publication)?;
    validate_publication_read(&input.offer_publication)?;
    verify_atlas_publication(
        &input.claim_publication_trust,
        &input.claim_publication.document,
        input.now_unix_ms,
    )?;
    verify_atlas_publication(
        &input.offer_publication_trust,
        &input.offer_publication.document,
        input.now_unix_ms,
    )?;

    let claim_publication = &input.claim_publication.document.statement;
    let published_claim = match &claim_publication.payload {
        AtlasPublicationPayload::DependencyClaim(claim) => claim,
        _ => bail!("Atlas verification claim publication has the wrong typed payload"),
    };
    if published_claim != &input.claim.document
        || claim_publication.publisher != input.context.local_repository
        || claim_publication.source.as_ref() != Some(&input.claim.source)
    {
        bail!("Atlas verification claim publication is not the exact current local claim version")
    }

    let (target_provider, target_surface, requirement) = match &input.claim.document.target {
        AtlasDependencyTarget::Exact {
            provider,
            surface_id,
            requirement,
        } => (provider, surface_id, requirement),
        AtlasDependencyTarget::Unresolved { .. } => {
            bail!("Atlas Soul cannot verify an unresolved or ambiguous dependency target")
        }
    };
    let offer_publication = &input.offer_publication.document.statement;
    let published_offer = match &offer_publication.payload {
        AtlasPublicationPayload::SurfaceOffer(offer) => offer,
        _ => bail!("Atlas verification offer publication has the wrong typed payload"),
    };
    if &published_offer.provider != target_provider
        || published_offer.surface_id != *target_surface
        || offer_publication.publisher != *target_provider
        || published_offer.contract.contract_id() != requirement.contract_id()
        || !matches!(
            evaluate_atlas_compatibility(&input.claim.document, Some(published_offer)),
            AtlasCompatibility::Exact | AtlasCompatibility::Compatible
        )
    {
        bail!("Atlas verification offer does not satisfy the claim's exact target and contract")
    }
    input.evidence.validate()?;
    validate_optional_current_verification(
        &input.context,
        input.claim.document.claim_id,
        input.expected_current_verification_source.as_ref(),
        input.current_verification.as_ref(),
    )?;

    let next = AtlasDependencyVerification {
        schema_version: ATLAS_DEPENDENCY_VERIFICATION_SCHEMA.into(),
        consumer: input.context.local_repository.clone(),
        claim_id: input.claim.document.claim_id,
        claim_publication_id: claim_publication.publication_id.clone(),
        offer_publication_id: offer_publication.publication_id.clone(),
        exact_contract: published_offer.contract.clone(),
        verdict: input.verdict,
        evidence_sha256: input.evidence.version.payload_sha256.clone(),
    };
    next.validate()?;
    if input
        .current_verification
        .as_ref()
        .is_some_and(|current| current.document == next)
    {
        bail!("Atlas verification admission would write an unchanged document")
    }
    let next_source = local_next_source(
        input
            .current_verification
            .as_ref()
            .map(|current| &current.source)
            .unwrap_or(&input.claim.source),
        ATLAS_DEPENDENCY_VERIFICATION_SCHEMA,
        next.claim_id.to_string(),
        &next,
    )?;
    let mut intent = AtlasDependencyVerificationWriteIntent {
        schema_version: ATLAS_DEPENDENCY_VERIFICATION_WRITE_INTENT_SCHEMA.into(),
        intent_id: String::new(),
        local_repository: input.context.local_repository,
        body_basis: input.context.current_body_basis,
        claim_source: input.claim.source,
        claim_publication_source: input.claim_publication.source,
        offer_publication_source: input.offer_publication.source,
        evidence: input.evidence.version,
        expected_current_source: input.expected_current_verification_source,
        next_source,
        next,
    };
    intent.intent_id = dependency_verification_intent_id(&intent)?;
    intent.validate()?;
    Ok(intent)
}

pub fn admit_dependency_impact(
    input: AtlasDependencyImpactAdmissionInput,
) -> Result<AtlasDependencyImpactWriteIntent> {
    validate_context(&input.context)?;
    validate_exact_local_read(
        &input.claim,
        ATLAS_DEPENDENCY_CLAIM_SCHEMA,
        &input.claim.document.claim_id.to_string(),
    )?;
    require_expected_source(
        &input.expected_claim_source,
        &input.claim.source,
        "impact claim",
    )?;
    if input.claim.document.consumer != input.context.local_repository
        || input.claim.document.lifecycle != AtlasClaimLifecycle::Active
    {
        bail!("Atlas Self impact admission requires an active exact local claim")
    }
    validate_projection_read(&input.projection)?;
    require_expected_source(
        &input.expected_projection_source,
        &input.projection.source,
        "impact projection",
    )?;
    validate_optional_current_impact(
        &input.context,
        input.proposal.impact.impact_id,
        input.expected_current_impact_source.as_ref(),
        input.current_impact.as_ref(),
    )?;

    let expected = evaluate_local_atlas_impacts(
        &input.context.local_repository,
        &input.projection.document,
        &[AtlasLocalClaimInput {
            claim: input.claim.document.clone(),
            source_payload_sha256: input.claim.source.payload_sha256.clone(),
        }],
        &Default::default(),
        &[
            AtlasImpactLaneState {
                lane: AtlasImpactLane::Modeling,
                pending_proposal_id: None,
                last_completed_at_unix_ms: None,
            },
            AtlasImpactLaneState {
                lane: AtlasImpactLane::Soul,
                pending_proposal_id: None,
                last_completed_at_unix_ms: None,
            },
        ],
        &AtlasImpactBrakeState {
            engaged: false,
            brake_id: None,
        },
        &AtlasImpactIngressPolicy {
            cooldown_after_completion_ms: 0,
        },
        input.projection.document.evaluated_at_unix_ms,
    )?
    .proposals
    .into_iter()
    .find(|proposal| proposal.proposal_id == input.proposal.proposal_id)
    .ok_or_else(|| {
        anyhow::anyhow!("Atlas impact proposal is not implied by the exact claim/projection basis")
    })?;
    if expected != input.proposal {
        bail!("Atlas impact proposal was substituted after Modeling/Soul ingress")
    }
    if input
        .current_impact
        .as_ref()
        .is_some_and(|current| current.document == input.proposal.impact)
    {
        bail!("Atlas impact admission would write an unchanged document")
    }
    let next_source = local_next_source(
        input
            .current_impact
            .as_ref()
            .map(|current| &current.source)
            .unwrap_or(&input.claim.source),
        ATLAS_DEPENDENCY_IMPACT_SCHEMA,
        input.proposal.impact.impact_id.to_string(),
        &input.proposal.impact,
    )?;
    let mut intent = AtlasDependencyImpactWriteIntent {
        schema_version: ATLAS_DEPENDENCY_IMPACT_WRITE_INTENT_SCHEMA.into(),
        intent_id: String::new(),
        local_repository: input.context.local_repository,
        body_basis: input.context.current_body_basis,
        claim_source: input.claim.source,
        projection_source: input.projection.source,
        expected_current_source: input.expected_current_impact_source,
        next_source,
        proposal: input.proposal,
    };
    intent.intent_id = dependency_impact_intent_id(&intent)?;
    intent.validate()?;
    Ok(intent)
}

impl AtlasEvidenceArtifactRead {
    fn validate(&self) -> Result<()> {
        validate_identifier(&self.version.artifact_id, "Atlas verification artifact id")?;
        validate_sha256(
            &self.version.payload_sha256,
            "Atlas verification artifact digest",
        )?;
        if self.payload.is_empty() || sha256(&self.payload) != self.version.payload_sha256 {
            bail!("Atlas verification artifact bytes do not match the admitted exact version")
        }
        Ok(())
    }
}

impl AtlasSurfaceOfferWriteIntent {
    pub fn validate(&self) -> Result<()> {
        require_schema(
            &self.schema_version,
            ATLAS_SURFACE_OFFER_WRITE_INTENT_SCHEMA,
        )?;
        validate_intent_context(&self.local_repository, &self.body_basis, &self.intent_id)?;
        self.next.validate()?;
        if self.next.provider != self.local_repository {
            bail!("Atlas surface offer intent names a foreign write owner")
        }
        match (&self.expected_current_source, &self.next.lifecycle) {
            (None, AtlasOfferLifecycle::Active) => {}
            (
                Some(expected),
                AtlasOfferLifecycle::Deprecated { .. } | AtlasOfferLifecycle::Withdrawn,
            ) => {
                validate_source_identity(
                    expected,
                    LOCAL_MIND_STORE_ID,
                    ATLAS_SURFACE_OFFER_SCHEMA,
                    &self.next.surface_id.to_string(),
                )?;
            }
            _ => bail!("Atlas surface offer admission/transition ownership is ambiguous"),
        }
        validate_exact_source(
            &self.next_source,
            LOCAL_MIND_STORE_ID,
            ATLAS_SURFACE_OFFER_SCHEMA,
            &self.next.surface_id.to_string(),
            &self.next,
        )?;
        match (&self.next.lifecycle, &self.replacement_source) {
            (AtlasOfferLifecycle::Active, None) => {}
            (
                AtlasOfferLifecycle::Deprecated {
                    replacement_surface_id: Some(replacement_surface_id),
                },
                Some(replacement),
            ) => validate_source_identity(
                replacement,
                LOCAL_MIND_STORE_ID,
                ATLAS_SURFACE_OFFER_SCHEMA,
                &replacement_surface_id.to_string(),
            )?,
            (
                AtlasOfferLifecycle::Deprecated {
                    replacement_surface_id: None,
                }
                | AtlasOfferLifecycle::Withdrawn,
                None,
            ) => {}
            _ => bail!("Atlas surface offer intent has an ambiguous replacement target"),
        }
        if self.intent_id != surface_offer_intent_id(self)? {
            bail!("Atlas surface offer write intent id is not canonical")
        }
        Ok(())
    }
}

impl AtlasDependencyClaimWriteIntent {
    pub fn validate(&self) -> Result<()> {
        require_schema(
            &self.schema_version,
            ATLAS_DEPENDENCY_CLAIM_WRITE_INTENT_SCHEMA,
        )?;
        validate_intent_context(&self.local_repository, &self.body_basis, &self.intent_id)?;
        self.next.validate()?;
        if self.next.consumer != self.local_repository {
            bail!("Atlas dependency claim intent names a foreign write owner")
        }
        let key = self.next.claim_id.to_string();
        match (&self.expected_current_source, self.next.lifecycle) {
            (None, AtlasClaimLifecycle::Active) => {}
            (Some(expected), AtlasClaimLifecycle::Retired) => validate_source_identity(
                expected,
                LOCAL_MIND_STORE_ID,
                ATLAS_DEPENDENCY_CLAIM_SCHEMA,
                &key,
            )?,
            _ => bail!("Atlas dependency claim admission/transition ownership is ambiguous"),
        }
        validate_exact_source(
            &self.next_source,
            LOCAL_MIND_STORE_ID,
            ATLAS_DEPENDENCY_CLAIM_SCHEMA,
            &key,
            &self.next,
        )?;
        if self.intent_id != dependency_claim_intent_id(self)? {
            bail!("Atlas dependency claim write intent id is not canonical")
        }
        Ok(())
    }
}

impl AtlasDependencyVerificationWriteIntent {
    pub fn validate(&self) -> Result<()> {
        require_schema(
            &self.schema_version,
            ATLAS_DEPENDENCY_VERIFICATION_WRITE_INTENT_SCHEMA,
        )?;
        validate_intent_context(&self.local_repository, &self.body_basis, &self.intent_id)?;
        self.next.validate()?;
        if self.next.consumer != self.local_repository {
            bail!("Atlas dependency verification intent names a foreign write owner")
        }
        validate_source_identity(
            &self.claim_source,
            LOCAL_MIND_STORE_ID,
            ATLAS_DEPENDENCY_CLAIM_SCHEMA,
            &self.next.claim_id.to_string(),
        )?;
        validate_source_identity(
            &self.claim_publication_source,
            self.claim_publication_source.store_id.as_str(),
            ATLAS_PUBLICATION_SCHEMA,
            &self.next.claim_publication_id,
        )?;
        validate_source_identity(
            &self.offer_publication_source,
            self.offer_publication_source.store_id.as_str(),
            ATLAS_PUBLICATION_SCHEMA,
            &self.next.offer_publication_id,
        )?;
        validate_identifier(&self.evidence.artifact_id, "Atlas verification artifact id")?;
        validate_sha256(
            &self.evidence.payload_sha256,
            "Atlas verification artifact digest",
        )?;
        let key = self.next.claim_id.to_string();
        if let Some(expected) = &self.expected_current_source {
            validate_source_identity(
                expected,
                LOCAL_MIND_STORE_ID,
                ATLAS_DEPENDENCY_VERIFICATION_SCHEMA,
                &key,
            )?;
        }
        validate_exact_source(
            &self.next_source,
            LOCAL_MIND_STORE_ID,
            ATLAS_DEPENDENCY_VERIFICATION_SCHEMA,
            &key,
            &self.next,
        )?;
        if self.intent_id != dependency_verification_intent_id(self)? {
            bail!("Atlas dependency verification write intent id is not canonical")
        }
        Ok(())
    }
}

impl AtlasDependencyImpactWriteIntent {
    pub fn validate(&self) -> Result<()> {
        require_schema(
            &self.schema_version,
            ATLAS_DEPENDENCY_IMPACT_WRITE_INTENT_SCHEMA,
        )?;
        validate_intent_context(&self.local_repository, &self.body_basis, &self.intent_id)?;
        self.proposal.impact.validate()?;
        if self.proposal.impact.consumer != self.local_repository
            || self.proposal.proposal_id != self.proposal.impact.impact_id
            || self.proposal.authority != AtlasImpactProposalAuthority::LocalReviewOnly
        {
            bail!("Atlas dependency impact intent names a foreign or substituted impact")
        }
        let expected_dedupe_key = sha256(&rmp_serde::to_vec_named(&(
            "epiphany.model.dependency-impact-dedupe.v0",
            self.proposal.impact.claim_id,
            &self.claim_source.payload_sha256,
            &self.proposal.impact.source_publication_ids,
            &self.proposal.reason,
            self.proposal.impact.criticality,
        ))?);
        if self.proposal.dedupe_key != expected_dedupe_key
            || self.proposal.proposal_id
                != uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, expected_dedupe_key.as_bytes())
        {
            bail!("Atlas dependency impact intent has a substituted dedupe identity")
        }
        validate_source_identity(
            &self.claim_source,
            LOCAL_MIND_STORE_ID,
            ATLAS_DEPENDENCY_CLAIM_SCHEMA,
            &self.proposal.impact.claim_id.to_string(),
        )?;
        validate_source_identity(
            &self.projection_source,
            self.projection_source.store_id.as_str(),
            ATLAS_PROJECTION_SCHEMA,
            self.projection_source.document_key.as_str(),
        )?;
        let key = self.proposal.impact.impact_id.to_string();
        if let Some(expected) = &self.expected_current_source {
            validate_source_identity(
                expected,
                LOCAL_MIND_STORE_ID,
                ATLAS_DEPENDENCY_IMPACT_SCHEMA,
                &key,
            )?;
        }
        validate_exact_source(
            &self.next_source,
            LOCAL_MIND_STORE_ID,
            ATLAS_DEPENDENCY_IMPACT_SCHEMA,
            &key,
            &self.proposal.impact,
        )?;
        if self.intent_id != dependency_impact_intent_id(self)? {
            bail!("Atlas dependency impact write intent id is not canonical")
        }
        Ok(())
    }
}

fn validate_context(context: &AtlasPlannerContext) -> Result<()> {
    context.local_repository.validate()?;
    validate_body_basis(&context.local_repository, &context.current_body_basis)
}

fn validate_body_basis(
    repository: &AtlasRepositoryIdentity,
    basis: &crate::RepositoryBodyObservationBasis,
) -> Result<()> {
    if basis.schema_version != BODY_SCHEMA_VERSION
        || basis.swarm_id != repository.swarm_id
        || basis.workspace_id != repository.workspace_id
        || basis.generation == 0
        || basis.observation_id != format!("{}:{}", basis.workspace_id, basis.generation)
    {
        bail!("Atlas planner Body basis is not the exact current local repository basis")
    }
    validate_identifier(&basis.runtime_id, "Atlas planner Body runtime id")?;
    validate_identifier(&basis.scope, "Atlas planner Body scope")?;
    validate_identifier(&basis.observation_id, "Atlas planner Body observation id")?;
    validate_repository_body_sha256(
        &basis.body_binding_sha256,
        "Atlas planner Body binding digest",
    )?;
    validate_repository_body_sha256(
        &basis.manifest_root_sha256,
        "Atlas planner Body manifest digest",
    )?;
    let started = chrono::DateTime::parse_from_rfc3339(&basis.scan_started_at)?;
    let finished = chrono::DateTime::parse_from_rfc3339(&basis.scan_finished_at)?;
    if finished < started {
        bail!("Atlas planner Body scan interval is inverted")
    }
    Ok(())
}

fn validate_intent_context(
    repository: &AtlasRepositoryIdentity,
    body_basis: &crate::RepositoryBodyObservationBasis,
    intent_id: &str,
) -> Result<()> {
    validate_sha256(intent_id, "Atlas Mind write intent id")?;
    validate_body_basis(repository, body_basis)
}

fn validate_exact_local_read<T: Serialize>(
    read: &AtlasStrongRead<T>,
    schema: &str,
    key: &str,
) -> Result<()> {
    validate_exact_source(
        &read.source,
        LOCAL_MIND_STORE_ID,
        schema,
        key,
        &read.document,
    )
}

fn validate_exact_source<T: Serialize>(
    source: &AtlasMindSourceVersion,
    store_id: &str,
    schema: &str,
    key: &str,
    document: &T,
) -> Result<()> {
    validate_source_identity(source, store_id, schema, key)?;
    if source.payload_sha256 != sha256(&rmp_serde::to_vec(document)?) {
        bail!("Atlas strong-read source is stale or its typed payload was substituted")
    }
    Ok(())
}

fn validate_source_identity(
    source: &AtlasMindSourceVersion,
    store_id: &str,
    schema: &str,
    key: &str,
) -> Result<()> {
    source.validate()?;
    if source.store_id != store_id
        || source.document_type != schema
        || source.schema_id.as_deref() != Some(schema)
        || source.document_key != key
    {
        bail!("Atlas document version does not identify the exact typed strong-read slot")
    }
    Ok(())
}

fn require_expected_source(
    expected: &AtlasMindSourceVersion,
    current: &AtlasMindSourceVersion,
    target: &str,
) -> Result<()> {
    if expected != current {
        bail!("Atlas {target} request was planned against a stale source version")
    }
    Ok(())
}

fn local_next_source<T: Serialize>(
    store_basis: &AtlasMindSourceVersion,
    schema: &str,
    key: String,
    document: &T,
) -> Result<AtlasMindSourceVersion> {
    if store_basis.store_id != LOCAL_MIND_STORE_ID {
        bail!("Atlas planner cannot emit a write intent for a foreign store")
    }
    Ok(AtlasMindSourceVersion {
        store_id: LOCAL_MIND_STORE_ID.into(),
        document_type: schema.into(),
        document_key: key,
        schema_id: Some(schema.into()),
        payload_sha256: sha256(&rmp_serde::to_vec(document)?),
    })
}

fn validate_replacement_offer(
    context: &AtlasPlannerContext,
    current: &AtlasSurfaceOffer,
    replacement: &AtlasStrongRead<AtlasSurfaceOffer>,
) -> Result<AtlasMindSourceVersion> {
    validate_exact_local_read(
        replacement,
        ATLAS_SURFACE_OFFER_SCHEMA,
        &replacement.document.surface_id.to_string(),
    )?;
    if replacement.document.provider != context.local_repository
        || replacement.document.surface_id == current.surface_id
        || replacement.document.lifecycle != AtlasOfferLifecycle::Active
        || replacement.document.contract.contract_id() != current.contract.contract_id()
    {
        bail!("Atlas replacement surface is foreign, recursive, inactive, or contract-ambiguous")
    }
    Ok(replacement.source.clone())
}

fn validate_publication_read(read: &AtlasStrongRead<AtlasPublicationEnvelope>) -> Result<()> {
    validate_exact_source(
        &read.source,
        read.source.store_id.as_str(),
        ATLAS_PUBLICATION_SCHEMA,
        &read.document.statement.publication_id,
        &read.document,
    )
}

fn validate_projection_read(read: &AtlasStrongRead<AtlasEntanglementProjection>) -> Result<()> {
    require_schema(&read.document.schema_version, ATLAS_PROJECTION_SCHEMA)?;
    validate_exact_source(
        &read.source,
        read.source.store_id.as_str(),
        ATLAS_PROJECTION_SCHEMA,
        &read.document.audience_id,
        &read.document,
    )?;
    if atlas_projection_digest(&read.document)? != read.document.projection_sha256 {
        bail!("Atlas impact projection digest does not seal the exact projection")
    }
    Ok(())
}

fn validate_optional_current_verification(
    context: &AtlasPlannerContext,
    claim_id: uuid::Uuid,
    expected: Option<&AtlasMindSourceVersion>,
    current: Option<&AtlasStrongRead<AtlasDependencyVerification>>,
) -> Result<()> {
    if expected != current.map(|current| &current.source) {
        bail!("Atlas verification target is absent, ambiguous, or stale")
    }
    if let Some(current) = current {
        validate_exact_local_read(
            current,
            ATLAS_DEPENDENCY_VERIFICATION_SCHEMA,
            &claim_id.to_string(),
        )?;
        if current.document.consumer != context.local_repository
            || current.document.claim_id != claim_id
        {
            bail!("Atlas verification target is owned by another repository or claim")
        }
    }
    Ok(())
}

fn validate_optional_current_impact(
    context: &AtlasPlannerContext,
    impact_id: uuid::Uuid,
    expected: Option<&AtlasMindSourceVersion>,
    current: Option<&AtlasStrongRead<AtlasDependencyImpact>>,
) -> Result<()> {
    if expected != current.map(|current| &current.source) {
        bail!("Atlas impact target is absent, ambiguous, or stale")
    }
    if let Some(current) = current {
        validate_exact_local_read(
            current,
            ATLAS_DEPENDENCY_IMPACT_SCHEMA,
            &impact_id.to_string(),
        )?;
        if current.document.consumer != context.local_repository
            || current.document.impact_id != impact_id
        {
            bail!("Atlas impact target is owned by another repository or impact id")
        }
    }
    Ok(())
}

fn surface_offer_intent_id(intent: &AtlasSurfaceOfferWriteIntent) -> Result<String> {
    Ok(sha256(&rmp_serde::to_vec_named(&(
        ATLAS_SURFACE_OFFER_WRITE_INTENT_SCHEMA,
        &intent.local_repository,
        &intent.body_basis,
        &intent.expected_current_source,
        &intent.replacement_source,
        &intent.next_source,
        &intent.next,
    ))?))
}

fn dependency_claim_intent_id(intent: &AtlasDependencyClaimWriteIntent) -> Result<String> {
    Ok(sha256(&rmp_serde::to_vec_named(&(
        ATLAS_DEPENDENCY_CLAIM_WRITE_INTENT_SCHEMA,
        &intent.local_repository,
        &intent.body_basis,
        &intent.expected_current_source,
        &intent.next_source,
        &intent.next,
    ))?))
}

fn dependency_verification_intent_id(
    intent: &AtlasDependencyVerificationWriteIntent,
) -> Result<String> {
    Ok(sha256(&rmp_serde::to_vec_named(&(
        ATLAS_DEPENDENCY_VERIFICATION_WRITE_INTENT_SCHEMA,
        &intent.local_repository,
        &intent.body_basis,
        &intent.claim_source,
        &intent.claim_publication_source,
        &intent.offer_publication_source,
        &intent.evidence,
        &intent.expected_current_source,
        &intent.next_source,
        &intent.next,
    ))?))
}

fn dependency_impact_intent_id(intent: &AtlasDependencyImpactWriteIntent) -> Result<String> {
    Ok(sha256(&rmp_serde::to_vec_named(&(
        ATLAS_DEPENDENCY_IMPACT_WRITE_INTENT_SCHEMA,
        &intent.local_repository,
        &intent.body_basis,
        &intent.claim_source,
        &intent.projection_source,
        &intent.expected_current_source,
        &intent.next_source,
        &intent.proposal,
    ))?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cultnet_rs::{ServiceIdentitySigner, enroll_service_identity_at};
    use semver::{Version, VersionReq};
    use tempfile::TempDir;
    use uuid::Uuid;

    const NOW: u64 = 1_800_000_000_000;

    struct TestRepository {
        identity: AtlasRepositoryIdentity,
        signer: ServiceIdentitySigner<super::super::identity::AtlasRepositorySigningIdentity>,
        trust: AtlasPublisherTrustBinding,
        body: crate::RepositoryBodyObservationBasis,
    }

    fn digest(seed: &str) -> String {
        sha256(seed.as_bytes())
    }

    fn body_digest(seed: &str) -> String {
        digest(seed).trim_start_matches("sha256-").to_owned()
    }

    fn repository(temp: &TempDir, name: &str) -> Result<TestRepository> {
        let identity = AtlasRepositoryIdentity::new("gamecult", format!("workspace-{name}"))?;
        let signer = enroll_service_identity_at::<
            super::super::identity::AtlasRepositorySigningIdentity,
        >(&temp.path().join(format!("{name}.cc")))?;
        let trust = AtlasPublisherTrustBinding {
            schema_version: ATLAS_TRUST_BINDING_SCHEMA.into(),
            publisher: identity.clone(),
            signer_identity_id: signer.entry().identity_id.clone(),
            trusted_from_unix_ms: NOW - 10_000,
            expires_at_unix_ms: Some(NOW + 10_000),
            revoked: false,
            trust_anchor: signer.trust_anchor()?,
        };
        let body = crate::RepositoryBodyObservationBasis {
            schema_version: BODY_SCHEMA_VERSION.into(),
            workspace_id: identity.workspace_id.clone(),
            swarm_id: identity.swarm_id.clone(),
            runtime_id: format!("runtime-{name}"),
            scope: ".".into(),
            body_binding_sha256: body_digest(&format!("binding-{name}")),
            observation_id: format!("{}:1", identity.workspace_id),
            generation: 1,
            manifest_root_sha256: body_digest(&format!("manifest-{name}")),
            scan_started_at: "2027-01-15T00:00:00Z".into(),
            scan_finished_at: "2027-01-15T00:00:01Z".into(),
        };
        Ok(TestRepository {
            identity,
            signer,
            trust,
            body,
        })
    }

    fn context(repository: &TestRepository) -> AtlasPlannerContext {
        AtlasPlannerContext {
            local_repository: repository.identity.clone(),
            current_body_basis: repository.body.clone(),
        }
    }

    fn strong_read<T: Serialize + Clone>(
        store_id: &str,
        schema: &str,
        key: String,
        document: T,
    ) -> Result<AtlasStrongRead<T>> {
        Ok(AtlasStrongRead {
            source: AtlasMindSourceVersion {
                store_id: store_id.into(),
                document_type: schema.into(),
                document_key: key,
                schema_id: Some(schema.into()),
                payload_sha256: sha256(&rmp_serde::to_vec(&document)?),
            },
            document,
        })
    }

    fn descriptor(version: &str) -> AtlasContractDescriptor {
        AtlasContractDescriptor::Semver {
            contract_id: "contract:atlas-test".into(),
            version: Version::parse(version).unwrap(),
        }
    }

    fn requirement(requirement: &str) -> AtlasContractRequirement {
        AtlasContractRequirement::Semver {
            contract_id: "contract:atlas-test".into(),
            requirement: VersionReq::parse(requirement).unwrap(),
        }
    }

    fn offer(repository: &TestRepository, surface_id: Uuid) -> AtlasSurfaceOffer {
        AtlasSurfaceOffer {
            schema_version: ATLAS_SURFACE_OFFER_SCHEMA.into(),
            provider: repository.identity.clone(),
            surface_id,
            contract: descriptor("2.1.0"),
            lifecycle: AtlasOfferLifecycle::Active,
            label: "Test surface".into(),
            body_evidence: vec![AtlasBodyEvidenceRef {
                path: "Cargo.toml".into(),
                raw_sha256: "0".repeat(64),
            }],
        }
    }

    fn claim(
        consumer: &TestRepository,
        provider: &TestRepository,
        claim_id: Uuid,
        surface_id: Uuid,
    ) -> AtlasDependencyClaim {
        AtlasDependencyClaim {
            schema_version: ATLAS_DEPENDENCY_CLAIM_SCHEMA.into(),
            consumer: consumer.identity.clone(),
            claim_id,
            target: AtlasDependencyTarget::Exact {
                provider: provider.identity.clone(),
                surface_id,
                requirement: requirement("^2.0"),
            },
            entanglement_kind: AtlasEntanglementKind::Runtime,
            failure_semantics: AtlasFailureSemantics::HumanDecision,
            impact_scope: AtlasImpactScope::WholeRepository,
            lifecycle: AtlasClaimLifecycle::Active,
            label: "Test dependency".into(),
            body_evidence: vec![AtlasBodyEvidenceRef {
                path: "Cargo.toml".into(),
                raw_sha256: "0".repeat(64),
            }],
        }
    }

    fn mind_publication(
        repository: &TestRepository,
        sequence: u64,
        payload: AtlasPublicationPayload,
    ) -> Result<AtlasPublicationEnvelope> {
        let source = AtlasMindSourceVersion {
            store_id: LOCAL_MIND_STORE_ID.into(),
            document_type: payload.schema().into(),
            document_key: payload.key(),
            schema_id: Some(payload.schema().into()),
            payload_sha256: sha256(&super::super::identity::atlas_source_payload_msgpack(
                &payload,
            )?),
        };
        super::super::identity::sign_atlas_publication(
            &repository.signer,
            repository.identity.clone(),
            sequence,
            repository.body.runtime_id.clone(),
            format!("incarnation-{sequence}"),
            repository.body.clone(),
            format!(
                "cultmesh://gamecult-local/atlas/{}",
                repository.identity.workspace_id
            ),
            Some(source.clone()),
            Some(AtlasMindCommitBinding {
                receipt_id: format!("mind-receipt-{sequence}"),
                receipt_sha256: digest(&format!("receipt-{sequence}")),
                invariant_owner: "Mind".into(),
                source,
            }),
            NOW - 100,
            payload,
        )
    }

    #[test]
    fn offer_transition_is_keyed_to_current_and_replacement_versions() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let local = repository(&temp, "local")?;
        let current = strong_read(
            LOCAL_MIND_STORE_ID,
            ATLAS_SURFACE_OFFER_SCHEMA,
            Uuid::from_u128(1).to_string(),
            offer(&local, Uuid::from_u128(1)),
        )?;
        let replacement = strong_read(
            LOCAL_MIND_STORE_ID,
            ATLAS_SURFACE_OFFER_SCHEMA,
            Uuid::from_u128(2).to_string(),
            offer(&local, Uuid::from_u128(2)),
        )?;
        let intent = plan_surface_offer_transition(AtlasSurfaceOfferTransitionInput {
            context: context(&local),
            expected_current_source: current.source.clone(),
            current: current.clone(),
            transition: AtlasSurfaceOfferTransition::Deprecate {
                replacement: Some(replacement.clone()),
            },
        })?;
        assert_eq!(intent.replacement_source, Some(replacement.source));
        assert!(matches!(
            intent.next.lifecycle,
            AtlasOfferLifecycle::Deprecated {
                replacement_surface_id: Some(id)
            } if id == Uuid::from_u128(2)
        ));
        assert_eq!(intent.expected_current_source, Some(current.source));
        intent.validate()?;
        Ok(())
    }

    #[test]
    fn lifecycle_planners_refuse_stale_foreign_and_terminal_sources() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let local = repository(&temp, "local")?;
        let foreign = repository(&temp, "foreign")?;
        let current_offer = strong_read(
            LOCAL_MIND_STORE_ID,
            ATLAS_SURFACE_OFFER_SCHEMA,
            Uuid::from_u128(3).to_string(),
            offer(&local, Uuid::from_u128(3)),
        )?;
        let mut stale = current_offer.source.clone();
        stale.payload_sha256 = digest("stale");
        assert!(
            plan_surface_offer_transition(AtlasSurfaceOfferTransitionInput {
                context: context(&local),
                expected_current_source: stale,
                current: current_offer,
                transition: AtlasSurfaceOfferTransition::Withdraw,
            })
            .is_err()
        );

        let foreign_offer = strong_read(
            LOCAL_MIND_STORE_ID,
            ATLAS_SURFACE_OFFER_SCHEMA,
            Uuid::from_u128(6).to_string(),
            offer(&foreign, Uuid::from_u128(6)),
        )?;
        assert!(
            plan_surface_offer_transition(AtlasSurfaceOfferTransitionInput {
                context: context(&local),
                expected_current_source: foreign_offer.source.clone(),
                current: foreign_offer,
                transition: AtlasSurfaceOfferTransition::Withdraw,
            })
            .is_err()
        );

        let claim_id = Uuid::from_u128(4);
        let surface_id = Uuid::from_u128(5);
        let mut retired = claim(&local, &foreign, claim_id, surface_id);
        retired.lifecycle = AtlasClaimLifecycle::Retired;
        let retired = strong_read(
            LOCAL_MIND_STORE_ID,
            ATLAS_DEPENDENCY_CLAIM_SCHEMA,
            claim_id.to_string(),
            retired,
        )?;
        assert!(
            plan_dependency_claim_transition(AtlasDependencyClaimTransitionInput {
                context: context(&local),
                expected_current_source: retired.source.clone(),
                current: retired,
                transition: AtlasDependencyClaimTransition::Retire,
            })
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn soul_verification_binds_exact_signed_publications_and_artifact() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let consumer = repository(&temp, "consumer")?;
        let provider = repository(&temp, "provider")?;
        let claim_id = Uuid::from_u128(10);
        let surface_id = Uuid::from_u128(11);
        let claim_document = claim(&consumer, &provider, claim_id, surface_id);
        let offer_document = offer(&provider, surface_id);
        let claim_publication = mind_publication(
            &consumer,
            1,
            AtlasPublicationPayload::DependencyClaim(claim_document.clone()),
        )?;
        let offer_publication = mind_publication(
            &provider,
            1,
            AtlasPublicationPayload::SurfaceOffer(offer_document.clone()),
        )?;
        let claim_read = strong_read(
            LOCAL_MIND_STORE_ID,
            ATLAS_DEPENDENCY_CLAIM_SCHEMA,
            claim_id.to_string(),
            claim_document,
        )?;
        assert_eq!(
            claim_publication.statement.source.as_ref(),
            Some(&claim_read.source)
        );
        let claim_publication_read = strong_read(
            "cultmesh-gamecult-local",
            ATLAS_PUBLICATION_SCHEMA,
            claim_publication.statement.publication_id.clone(),
            claim_publication,
        )?;
        let offer_publication_read = strong_read(
            "cultmesh-gamecult-local",
            ATLAS_PUBLICATION_SCHEMA,
            offer_publication.statement.publication_id.clone(),
            offer_publication,
        )?;
        let evidence_payload = b"verified contract smoke artifact".to_vec();
        let intent = admit_dependency_verification(AtlasDependencyVerificationAdmissionInput {
            context: context(&consumer),
            now_unix_ms: NOW,
            expected_claim_source: claim_read.source.clone(),
            claim: claim_read,
            claim_publication: claim_publication_read,
            claim_publication_trust: consumer.trust,
            offer_publication: offer_publication_read,
            offer_publication_trust: provider.trust,
            expected_current_verification_source: None,
            current_verification: None,
            evidence: AtlasEvidenceArtifactRead {
                version: AtlasEvidenceArtifactVersion {
                    artifact_id: "artifact:verification-smoke".into(),
                    payload_sha256: sha256(&evidence_payload),
                },
                payload: evidence_payload,
            },
            verdict: AtlasVerificationVerdict::Passed,
        })?;
        assert_eq!(intent.next.exact_contract, offer_document.contract);
        assert_eq!(intent.next.verdict, AtlasVerificationVerdict::Passed);
        intent.validate()?;
        Ok(())
    }

    #[test]
    fn self_impact_admission_rederives_exact_projection_and_claim_proposal() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let consumer = repository(&temp, "consumer")?;
        let provider = repository(&temp, "provider")?;
        let claim_id = Uuid::from_u128(20);
        let surface_id = Uuid::from_u128(21);
        let claim_read = strong_read(
            LOCAL_MIND_STORE_ID,
            ATLAS_DEPENDENCY_CLAIM_SCHEMA,
            claim_id.to_string(),
            claim(&consumer, &provider, claim_id, surface_id),
        )?;
        let claim_publication_id = digest("claim-publication");
        let offer_publication_id = digest("offer-publication");
        let mut source_publication_ids =
            vec![claim_publication_id.clone(), offer_publication_id.clone()];
        source_publication_ids.sort();
        let mut projection = AtlasEntanglementProjection {
            schema_version: ATLAS_PROJECTION_SCHEMA.into(),
            audience_id: "gamecult-operator".into(),
            evaluated_at_unix_ms: NOW,
            source_publication_ids,
            publisher_status: Vec::new(),
            entanglements: vec![AtlasProjectedEntanglement {
                claim_id,
                claim_label: claim_read.document.label.clone(),
                claim_requirement: claim_read.document.target.requirement().clone(),
                claim_body_evidence: claim_read.document.body_evidence.clone(),
                consumer: consumer.identity.clone(),
                provider: Some(provider.identity.clone()),
                surface_id: Some(surface_id),
                offer_label: Some("Provider surface".into()),
                offer_contract: Some(descriptor("2.1.0")),
                offer_lifecycle: Some(AtlasOfferLifecycle::Active),
                offer_body_evidence: vec![AtlasBodyEvidenceRef {
                    path: "Cargo.toml".into(),
                    raw_sha256: "0".repeat(64),
                }],
                entanglement_kind: AtlasEntanglementKind::Runtime,
                failure_semantics: AtlasFailureSemantics::HumanDecision,
                impact_scope: AtlasImpactScope::WholeRepository,
                claim_freshness: AtlasPublicationFreshness::Current,
                offer_freshness: Some(AtlasPublicationFreshness::Current),
                compatibility: AtlasCompatibility::VersionMismatch,
                verification: AtlasVerificationState::Missing,
                claim_publication_id,
                offer_publication_id: Some(offer_publication_id),
                verification_publication_id: None,
                verification_evidence_sha256: None,
            }],
            cycles: Vec::new(),
            blast_radii: Vec::new(),
            projection_sha256: String::new(),
        };
        projection.projection_sha256 = atlas_projection_digest(&projection)?;
        let projection_read = strong_read(
            "cultmesh-gamecult-local",
            ATLAS_PROJECTION_SCHEMA,
            projection.audience_id.clone(),
            projection,
        )?;
        let proposal = evaluate_local_atlas_impacts(
            &consumer.identity,
            &projection_read.document,
            &[AtlasLocalClaimInput {
                claim: claim_read.document.clone(),
                source_payload_sha256: claim_read.source.payload_sha256.clone(),
            }],
            &Default::default(),
            &[
                AtlasImpactLaneState {
                    lane: AtlasImpactLane::Modeling,
                    pending_proposal_id: None,
                    last_completed_at_unix_ms: None,
                },
                AtlasImpactLaneState {
                    lane: AtlasImpactLane::Soul,
                    pending_proposal_id: None,
                    last_completed_at_unix_ms: None,
                },
            ],
            &AtlasImpactBrakeState {
                engaged: false,
                brake_id: None,
            },
            &AtlasImpactIngressPolicy {
                cooldown_after_completion_ms: 0,
            },
            NOW,
        )?
        .proposals
        .into_iter()
        .next()
        .expect("mismatched contract produces an impact");
        let intent = admit_dependency_impact(AtlasDependencyImpactAdmissionInput {
            context: context(&consumer),
            expected_claim_source: claim_read.source.clone(),
            claim: claim_read,
            expected_projection_source: projection_read.source.clone(),
            projection: projection_read,
            expected_current_impact_source: None,
            current_impact: None,
            proposal,
        })?;
        assert_eq!(intent.proposal.impact.claim_id, claim_id);
        intent.validate()?;
        Ok(())
    }
}
