use anyhow::{Result, bail};
use cultcache_rs::DatabaseEntry;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

use crate::RepositoryBodyObservationBasis;

pub const ATLAS_SURFACE_OFFER_SCHEMA: &str = "gamecult.model.surface_offer.v0";
pub const ATLAS_DEPENDENCY_CLAIM_SCHEMA: &str = "gamecult.model.dependency_claim.v0";
pub const ATLAS_DEPENDENCY_VERIFICATION_SCHEMA: &str = "gamecult.model.dependency_verification.v0";
pub const ATLAS_DEPENDENCY_IMPACT_SCHEMA: &str = "epiphany.model.dependency_impact.v0";
pub const ATLAS_PUBLICATION_SCHEMA: &str = "gamecult.model.atlas_publication.v0";
pub const ATLAS_PROJECTION_SCHEMA: &str = "gamecult.model.entanglement_projection.v0";
pub const ATLAS_TRUST_BINDING_SCHEMA: &str = "gamecult.model.atlas_publisher_trust.v0";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasRepositoryIdentity {
    pub swarm_id: String,
    pub workspace_id: String,
    pub repository_uri: String,
}

impl AtlasRepositoryIdentity {
    pub fn new(swarm_id: impl Into<String>, workspace_id: impl Into<String>) -> Result<Self> {
        let swarm_id = swarm_id.into();
        let workspace_id = workspace_id.into();
        validate_identifier(&swarm_id, "Atlas swarm id")?;
        validate_identifier(&workspace_id, "Atlas workspace id")?;
        Ok(Self {
            repository_uri: format!("gamecult://swarm/{swarm_id}/workspace/{workspace_id}"),
            swarm_id,
            workspace_id,
        })
    }

    pub fn validate(&self) -> Result<()> {
        validate_identifier(&self.swarm_id, "Atlas swarm id")?;
        validate_identifier(&self.workspace_id, "Atlas workspace id")?;
        if self.repository_uri
            != format!(
                "gamecult://swarm/{}/workspace/{}",
                self.swarm_id, self.workspace_id
            )
        {
            bail!("Atlas repository URI is not the canonical swarm/workspace identity")
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtlasEntanglementKind {
    Build,
    Runtime,
    Deployment,
    SchemaProtocol,
    DataState,
    InfrastructureControl,
    Governance,
    LorePersona,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtlasCriticality {
    Blocking,
    Degrading,
    Informational,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "version_scheme")]
pub enum AtlasContractDescriptor {
    Semver {
        contract_id: String,
        version: Version,
    },
    ExactSchema {
        contract_id: String,
        schema_id: String,
    },
    ExactDigest {
        contract_id: String,
        sha256: String,
    },
}

impl AtlasContractDescriptor {
    pub fn contract_id(&self) -> &str {
        match self {
            Self::Semver { contract_id, .. }
            | Self::ExactSchema { contract_id, .. }
            | Self::ExactDigest { contract_id, .. } => contract_id,
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_identifier(self.contract_id(), "Atlas contract id")?;
        match self {
            Self::Semver { .. } => Ok(()),
            Self::ExactSchema { schema_id, .. } => {
                validate_identifier(schema_id, "Atlas exact schema id")
            }
            Self::ExactDigest { sha256, .. } => {
                validate_sha256(sha256, "Atlas exact contract digest")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "version_scheme")]
pub enum AtlasContractRequirement {
    Semver {
        contract_id: String,
        requirement: VersionReq,
    },
    ExactSchema {
        contract_id: String,
        schema_id: String,
    },
    ExactDigest {
        contract_id: String,
        sha256: String,
    },
}

impl AtlasContractRequirement {
    pub fn contract_id(&self) -> &str {
        match self {
            Self::Semver { contract_id, .. }
            | Self::ExactSchema { contract_id, .. }
            | Self::ExactDigest { contract_id, .. } => contract_id,
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_identifier(self.contract_id(), "Atlas required contract id")?;
        match self {
            Self::Semver { .. } => Ok(()),
            Self::ExactSchema { schema_id, .. } => {
                validate_identifier(schema_id, "Atlas required exact schema id")
            }
            Self::ExactDigest { sha256, .. } => {
                validate_sha256(sha256, "Atlas required exact contract digest")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum AtlasOfferLifecycle {
    Active,
    Deprecated {
        replacement_surface_id: Option<Uuid>,
    },
    Withdrawn,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasBodyEvidenceRef {
    pub path: String,
    pub raw_sha256: String,
}

impl AtlasBodyEvidenceRef {
    pub fn validate(&self) -> Result<()> {
        if self.path.is_empty()
            || self.path.len() > 1024
            || self.path.starts_with('/')
            || self.path.contains('\\')
            || self
                .path
                .split('/')
                .any(|segment| segment.is_empty() || segment == "." || segment == "..")
        {
            bail!("Atlas Body evidence path is not a canonical portable relative path")
        }
        validate_repository_body_sha256(&self.raw_sha256, "Atlas Body evidence raw content digest")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.model.surface_offer.v0",
    schema = "gamecult.model.surface_offer.v0"
)]
pub struct AtlasSurfaceOffer {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub provider: AtlasRepositoryIdentity,
    #[cultcache(key = 2)]
    pub surface_id: Uuid,
    #[cultcache(key = 3)]
    pub contract: AtlasContractDescriptor,
    #[cultcache(key = 4)]
    pub lifecycle: AtlasOfferLifecycle,
    #[cultcache(key = 5)]
    pub label: String,
    #[cultcache(key = 6)]
    pub body_evidence: Vec<AtlasBodyEvidenceRef>,
}

impl AtlasSurfaceOffer {
    pub fn validate(&self) -> Result<()> {
        require_schema(&self.schema_version, ATLAS_SURFACE_OFFER_SCHEMA)?;
        self.provider.validate()?;
        if self.surface_id.is_nil() {
            bail!("Atlas surface id must be an opaque non-nil UUID")
        }
        self.contract.validate()?;
        validate_label_and_body_evidence(&self.label, &self.body_evidence, "Atlas surface offer")?;
        if matches!(
            self.lifecycle,
            AtlasOfferLifecycle::Deprecated {
                replacement_surface_id: Some(replacement)
            } if replacement == self.surface_id
        ) {
            bail!("Atlas deprecated offer cannot replace itself")
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "resolution")]
pub enum AtlasDependencyTarget {
    Exact {
        provider: AtlasRepositoryIdentity,
        surface_id: Uuid,
        requirement: AtlasContractRequirement,
    },
    Unresolved {
        requirement: AtlasContractRequirement,
    },
}

impl AtlasDependencyTarget {
    pub fn requirement(&self) -> &AtlasContractRequirement {
        match self {
            Self::Exact { requirement, .. } | Self::Unresolved { requirement } => requirement,
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Exact {
                provider,
                surface_id,
                requirement,
            } => {
                provider.validate()?;
                if surface_id.is_nil() {
                    bail!("Atlas exact dependency target surface must be a non-nil UUID")
                }
                requirement.validate()
            }
            Self::Unresolved { requirement } => requirement.validate(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtlasFailureSemantics {
    FailClosed,
    Degrade,
    LastKnownSafe,
    HumanDecision,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtlasClaimLifecycle {
    Active,
    Retired,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "scope")]
pub enum AtlasImpactScope {
    WholeRepository,
    LocalSurfaces { surface_ids: Vec<Uuid> },
}

impl AtlasImpactScope {
    pub fn validate(&self) -> Result<()> {
        if let Self::LocalSurfaces { surface_ids } = self {
            if surface_ids.is_empty()
                || surface_ids.iter().any(Uuid::is_nil)
                || !strictly_sorted_unique(surface_ids)
            {
                bail!("Atlas local impact scope must contain sorted unique non-nil surface UUIDs")
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.model.dependency_claim.v0",
    schema = "gamecult.model.dependency_claim.v0"
)]
pub struct AtlasDependencyClaim {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub consumer: AtlasRepositoryIdentity,
    #[cultcache(key = 2)]
    pub claim_id: Uuid,
    #[cultcache(key = 3)]
    pub target: AtlasDependencyTarget,
    #[cultcache(key = 4)]
    pub entanglement_kind: AtlasEntanglementKind,
    #[cultcache(key = 5)]
    pub failure_semantics: AtlasFailureSemantics,
    #[cultcache(key = 6)]
    pub impact_scope: AtlasImpactScope,
    #[cultcache(key = 7)]
    pub lifecycle: AtlasClaimLifecycle,
    #[cultcache(key = 8)]
    pub label: String,
    #[cultcache(key = 9)]
    pub body_evidence: Vec<AtlasBodyEvidenceRef>,
}

impl AtlasDependencyClaim {
    pub fn validate(&self) -> Result<()> {
        require_schema(&self.schema_version, ATLAS_DEPENDENCY_CLAIM_SCHEMA)?;
        self.consumer.validate()?;
        if self.claim_id.is_nil() {
            bail!("Atlas dependency claim id must be an opaque non-nil UUID")
        }
        self.target.validate()?;
        self.impact_scope.validate()?;
        validate_label_and_body_evidence(
            &self.label,
            &self.body_evidence,
            "Atlas dependency claim",
        )?;
        if let AtlasDependencyTarget::Exact { provider, .. } = &self.target {
            if provider == &self.consumer {
                bail!("Atlas dependency claim cannot target its owning repository")
            }
        }
        if matches!(
            self.entanglement_kind,
            AtlasEntanglementKind::Build
                | AtlasEntanglementKind::Deployment
                | AtlasEntanglementKind::InfrastructureControl
        ) && self.failure_semantics != AtlasFailureSemantics::FailClosed
        {
            bail!("build, deployment, and infrastructure/control dependencies must fail closed")
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtlasVerificationVerdict {
    Passed,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.model.dependency_verification.v0",
    schema = "gamecult.model.dependency_verification.v0"
)]
pub struct AtlasDependencyVerification {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub consumer: AtlasRepositoryIdentity,
    #[cultcache(key = 2)]
    pub claim_id: Uuid,
    #[cultcache(key = 3)]
    pub claim_publication_id: String,
    #[cultcache(key = 4)]
    pub offer_publication_id: String,
    #[cultcache(key = 5)]
    pub exact_contract: AtlasContractDescriptor,
    #[cultcache(key = 6)]
    pub verdict: AtlasVerificationVerdict,
    #[cultcache(key = 7)]
    pub evidence_sha256: String,
}

impl AtlasDependencyVerification {
    pub fn validate(&self) -> Result<()> {
        require_schema(&self.schema_version, ATLAS_DEPENDENCY_VERIFICATION_SCHEMA)?;
        self.consumer.validate()?;
        if self.claim_id.is_nil() {
            bail!("Atlas verification claim id must be non-nil")
        }
        validate_sha256(
            &self.claim_publication_id,
            "Atlas verification claim publication id",
        )?;
        validate_sha256(
            &self.offer_publication_id,
            "Atlas verification offer publication id",
        )?;
        self.exact_contract.validate()?;
        validate_sha256(&self.evidence_sha256, "Atlas verification evidence digest")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.model.dependency_impact.v0",
    schema = "epiphany.model.dependency_impact.v0"
)]
pub struct AtlasDependencyImpact {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub impact_id: Uuid,
    #[cultcache(key = 2)]
    pub consumer: AtlasRepositoryIdentity,
    #[cultcache(key = 3)]
    pub claim_id: Uuid,
    #[cultcache(key = 4)]
    pub claim_source_payload_sha256: String,
    #[cultcache(key = 5)]
    pub projection_sha256: String,
    #[cultcache(key = 6)]
    pub source_publication_ids: Vec<String>,
    #[cultcache(key = 7)]
    pub criticality: AtlasCriticality,
}

impl AtlasDependencyImpact {
    pub fn validate(&self) -> Result<()> {
        require_schema(&self.schema_version, ATLAS_DEPENDENCY_IMPACT_SCHEMA)?;
        self.consumer.validate()?;
        if self.impact_id.is_nil() || self.claim_id.is_nil() {
            bail!("Atlas dependency impact and claim ids must be non-nil")
        }
        validate_sha256(
            &self.claim_source_payload_sha256,
            "Atlas impact local claim source digest",
        )?;
        validate_sha256(
            &self.projection_sha256,
            "Atlas impact source projection digest",
        )?;
        validate_sorted_unique_sha256(
            &self.source_publication_ids,
            "Atlas impact source publication ids",
            true,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasMindSourceVersion {
    pub store_id: String,
    pub document_type: String,
    pub document_key: String,
    pub schema_id: Option<String>,
    pub payload_sha256: String,
}

impl AtlasMindSourceVersion {
    pub fn validate(&self) -> Result<()> {
        validate_identifier(&self.store_id, "Atlas source store id")?;
        validate_identifier(&self.document_type, "Atlas source document type")?;
        validate_identifier(&self.document_key, "Atlas source document key")?;
        if let Some(schema_id) = &self.schema_id {
            validate_identifier(schema_id, "Atlas source schema id")?;
        }
        validate_sha256(&self.payload_sha256, "Atlas source payload digest")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasMindCommitBinding {
    pub receipt_id: String,
    pub receipt_sha256: String,
    pub invariant_owner: String,
    pub source: AtlasMindSourceVersion,
}

impl AtlasMindCommitBinding {
    pub fn validate(&self) -> Result<()> {
        validate_identifier(&self.receipt_id, "Atlas Mind receipt id")?;
        validate_sha256(&self.receipt_sha256, "Atlas Mind receipt digest")?;
        validate_identifier(&self.invariant_owner, "Atlas Mind invariant owner")?;
        self.source.validate()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtlasPublisherState {
    Serving,
    Draining,
    Retired,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasPublicationWatermark {
    pub source_schema: String,
    pub source_key: String,
    pub source_payload_sha256: String,
    pub publication_sequence: u64,
}

impl AtlasPublicationWatermark {
    pub fn validate(&self) -> Result<()> {
        validate_identifier(&self.source_schema, "Atlas watermark source schema")?;
        validate_identifier(&self.source_key, "Atlas watermark source key")?;
        validate_sha256(
            &self.source_payload_sha256,
            "Atlas watermark source payload digest",
        )?;
        if self.publication_sequence == 0 {
            bail!("Atlas watermark publication sequence must be positive")
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasPublisherStatus {
    pub publisher: AtlasRepositoryIdentity,
    pub runtime_id: String,
    pub runtime_incarnation_id: String,
    pub heartbeat_sequence: u64,
    pub heartbeat_at_unix_ms: u64,
    pub state: AtlasPublisherState,
    pub watermarks: Vec<AtlasPublicationWatermark>,
}

impl AtlasPublisherStatus {
    pub fn validate(&self) -> Result<()> {
        self.publisher.validate()?;
        validate_identifier(&self.runtime_id, "Atlas publisher runtime id")?;
        validate_identifier(
            &self.runtime_incarnation_id,
            "Atlas publisher runtime incarnation id",
        )?;
        if self.heartbeat_sequence == 0
            || !self.watermarks.windows(2).all(|pair| {
                (&pair[0].source_schema, &pair[0].source_key)
                    < (&pair[1].source_schema, &pair[1].source_key)
            })
        {
            bail!("Atlas publisher heartbeat sequence or watermark ordering is invalid")
        }
        for watermark in &self.watermarks {
            watermark.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "payload", content = "value")]
pub enum AtlasPublicationPayload {
    PublisherStatus(AtlasPublisherStatus),
    SurfaceOffer(AtlasSurfaceOffer),
    DependencyClaim(AtlasDependencyClaim),
    DependencyVerification(AtlasDependencyVerification),
}

impl AtlasPublicationPayload {
    pub fn schema(&self) -> &'static str {
        match self {
            Self::PublisherStatus(_) => ATLAS_PUBLICATION_SCHEMA,
            Self::SurfaceOffer(_) => ATLAS_SURFACE_OFFER_SCHEMA,
            Self::DependencyClaim(_) => ATLAS_DEPENDENCY_CLAIM_SCHEMA,
            Self::DependencyVerification(_) => ATLAS_DEPENDENCY_VERIFICATION_SCHEMA,
        }
    }

    pub fn key(&self) -> String {
        match self {
            Self::PublisherStatus(status) => {
                format!("publisher-status:{}", status.publisher.repository_uri)
            }
            Self::SurfaceOffer(offer) => offer.surface_id.to_string(),
            Self::DependencyClaim(claim) => claim.claim_id.to_string(),
            Self::DependencyVerification(verification) => verification.claim_id.to_string(),
        }
    }

    pub fn owner(&self) -> &AtlasRepositoryIdentity {
        match self {
            Self::PublisherStatus(status) => &status.publisher,
            Self::SurfaceOffer(offer) => &offer.provider,
            Self::DependencyClaim(claim) => &claim.consumer,
            Self::DependencyVerification(verification) => &verification.consumer,
        }
    }

    pub fn requires_mind_binding(&self) -> bool {
        !matches!(self, Self::PublisherStatus(_))
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::PublisherStatus(status) => status.validate(),
            Self::SurfaceOffer(offer) => offer.validate(),
            Self::DependencyClaim(claim) => claim.validate(),
            Self::DependencyVerification(verification) => verification.validate(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtlasPublicationVisibility {
    GamecultLocal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasPublicationStatement {
    pub schema_version: String,
    pub publication_id: String,
    pub publisher: AtlasRepositoryIdentity,
    pub publisher_identity_id: String,
    pub publication_sequence: u64,
    pub runtime_id: String,
    pub runtime_incarnation_id: String,
    pub body_basis: RepositoryBodyObservationBasis,
    pub verse_id: String,
    pub visibility: AtlasPublicationVisibility,
    pub source: Option<AtlasMindSourceVersion>,
    pub mind_commit: Option<AtlasMindCommitBinding>,
    pub canonical_payload_msgpack_sha256: String,
    pub published_at_unix_ms: u64,
    pub payload: AtlasPublicationPayload,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.model.atlas_publication.v0",
    schema = "gamecult.model.atlas_publication.v0"
)]
pub struct AtlasPublicationEnvelope {
    #[cultcache(key = 0)]
    pub statement: AtlasPublicationStatement,
    #[cultcache(key = 1)]
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasProjectionAudience {
    pub audience_id: String,
    pub gamecult_local: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtlasPublicationFreshness {
    Current,
    LastKnownStale,
    Retired,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasFreshnessPolicy {
    pub publisher_status_maximum_age_ms: u64,
    pub maximum_future_skew_ms: u64,
}

impl AtlasFreshnessPolicy {
    pub fn validate(&self) -> Result<()> {
        if self.publisher_status_maximum_age_ms == 0 {
            bail!("Atlas publisher status maximum age must be positive")
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtlasCompatibility {
    Exact,
    Compatible,
    Unresolved,
    OfferMissing,
    OfferWithdrawn,
    ContractIdMismatch,
    VersionSchemeMismatch,
    VersionMismatch,
    ClaimRetired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtlasVerificationState {
    Passed,
    Failed,
    Missing,
    LastKnownStale,
    ExactEdgeMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasProjectedEntanglement {
    pub claim_id: Uuid,
    pub claim_label: String,
    pub claim_requirement: AtlasContractRequirement,
    pub claim_body_evidence: Vec<AtlasBodyEvidenceRef>,
    pub consumer: AtlasRepositoryIdentity,
    pub provider: Option<AtlasRepositoryIdentity>,
    pub surface_id: Option<Uuid>,
    pub offer_label: Option<String>,
    pub offer_contract: Option<AtlasContractDescriptor>,
    pub offer_lifecycle: Option<AtlasOfferLifecycle>,
    pub offer_body_evidence: Vec<AtlasBodyEvidenceRef>,
    pub entanglement_kind: AtlasEntanglementKind,
    pub failure_semantics: AtlasFailureSemantics,
    pub impact_scope: AtlasImpactScope,
    pub claim_freshness: AtlasPublicationFreshness,
    pub offer_freshness: Option<AtlasPublicationFreshness>,
    pub compatibility: AtlasCompatibility,
    pub verification: AtlasVerificationState,
    pub claim_publication_id: String,
    pub offer_publication_id: Option<String>,
    pub verification_publication_id: Option<String>,
    pub verification_evidence_sha256: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtlasCycleClass {
    ForbiddenBuild,
    ForbiddenDeployment,
    ForbiddenInfrastructureControl,
    ReviewRequired,
    Informational,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasProjectedCycle {
    pub repositories: Vec<AtlasRepositoryIdentity>,
    pub entanglement_kinds: Vec<AtlasEntanglementKind>,
    pub classification: AtlasCycleClass,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasAffectedRepository {
    pub repository: AtlasRepositoryIdentity,
    pub minimum_hops: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasBlastRadius {
    pub source: AtlasRepositoryIdentity,
    pub source_surface_id: Uuid,
    pub affected: Vec<AtlasAffectedRepository>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasPublisherProjectionStatus {
    pub publisher: AtlasRepositoryIdentity,
    pub runtime_id: String,
    pub runtime_incarnation_id: String,
    pub heartbeat_sequence: u64,
    pub heartbeat_at_unix_ms: u64,
    pub freshness: AtlasPublicationFreshness,
    pub watermarks: Vec<AtlasPublicationWatermark>,
    pub status_publication_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.model.entanglement_projection.v0",
    schema = "gamecult.model.entanglement_projection.v0"
)]
pub struct AtlasEntanglementProjection {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub audience_id: String,
    #[cultcache(key = 2)]
    pub evaluated_at_unix_ms: u64,
    #[cultcache(key = 3)]
    pub source_publication_ids: Vec<String>,
    #[cultcache(key = 4)]
    pub publisher_status: Vec<AtlasPublisherProjectionStatus>,
    #[cultcache(key = 5)]
    pub entanglements: Vec<AtlasProjectedEntanglement>,
    #[cultcache(key = 6)]
    pub cycles: Vec<AtlasProjectedCycle>,
    #[cultcache(key = 7)]
    pub blast_radii: Vec<AtlasBlastRadius>,
    #[cultcache(key = 8)]
    pub projection_sha256: String,
}

pub(crate) fn require_schema(actual: &str, expected: &str) -> Result<()> {
    if actual != expected {
        bail!("Atlas document schema does not match its typed contract")
    }
    Ok(())
}

pub(crate) fn validate_identifier(value: &str, field: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 320
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/' | b'@')
        })
    {
        bail!("{field} is not a bounded canonical identifier")
    }
    Ok(())
}

pub(crate) fn validate_sha256(value: &str, field: &str) -> Result<()> {
    if value.len() != 71
        || !value.starts_with("sha256-")
        || !value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        bail!("{field} is not a canonical sha256 digest")
    }
    Ok(())
}

pub(crate) fn validate_repository_body_sha256(value: &str, field: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        bail!("{field} is not a canonical repository Body SHA-256 digest")
    }
    Ok(())
}

pub(crate) fn validate_sorted_unique_sha256(
    values: &[String],
    field: &str,
    require_non_empty: bool,
) -> Result<()> {
    if (require_non_empty && values.is_empty()) || !strictly_sorted_unique(values) {
        bail!("{field} must be a non-empty strictly sorted set")
    }
    for value in values {
        validate_sha256(value, field)?;
    }
    Ok(())
}

fn strictly_sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn validate_label_and_body_evidence(
    label: &str,
    evidence: &[AtlasBodyEvidenceRef],
    field: &str,
) -> Result<()> {
    if label.trim().is_empty() || label.len() > 320 {
        bail!("{field} label must be non-empty and bounded")
    }
    if evidence.is_empty() || !strictly_sorted_unique(evidence) {
        bail!("{field} Body evidence must be a non-empty strictly sorted set")
    }
    for source in evidence {
        source.validate()?;
    }
    Ok(())
}

pub(crate) fn canonical_entanglement_kinds(
    kinds: impl IntoIterator<Item = AtlasEntanglementKind>,
) -> Vec<AtlasEntanglementKind> {
    kinds
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
