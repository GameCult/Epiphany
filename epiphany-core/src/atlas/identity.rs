use anyhow::{Context, Result, bail};
use cultnet_rs::{
    ServiceIdentityProfile, ServiceIdentitySignature, ServiceIdentitySigner,
    ServiceIdentityTrustAnchor, ServiceSignaturePurpose, derive_service_identity_id,
    verify_service_identity_signature,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::contracts::*;

pub const ATLAS_MAX_PUBLICATION_BYTES: usize = 1024 * 1024;

pub enum AtlasRepositorySigningIdentity {}

impl ServiceIdentityProfile for AtlasRepositorySigningIdentity {
    const PRIVATE_TYPE: &'static str = "gamecult.model.atlas_repository_identity.private.v0";
    const PRIVATE_SCHEMA: &'static str = "gamecult.model.atlas_repository_identity.private.v0";
    const PRIVATE_KEY: &'static str = "atlas-repository-signing-identity";
    const TRUST_ANCHOR_TYPE: &'static str =
        "gamecult.model.atlas_repository_identity.trust_anchor.v0";
    const TRUST_ANCHOR_SCHEMA: &'static str =
        "gamecult.model.atlas_repository_identity.trust_anchor.v0";
    const TRUST_ANCHOR_KEY: &'static str = "atlas-repository-signing-identity-public";
    const ID_DOMAIN: &'static [u8] = b"gamecult.model.atlas-repository-identity.id.v0\0";
    const SIGNATURE_DOMAIN: &'static [u8] = b"gamecult.model.atlas-publication.signature.v0\0";
    const PROTECTOR_CONTEXT: &'static str = "gamecult-model-atlas-repository-identity-v0";
}

pub struct AtlasPublicationPurpose;

impl ServiceSignaturePurpose<AtlasRepositorySigningIdentity> for AtlasPublicationPurpose {
    const PURPOSE: &'static [u8] = b"gamecult.model.atlas-publication.v0";
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasPublisherTrustBinding {
    pub schema_version: String,
    pub publisher: AtlasRepositoryIdentity,
    pub signer_identity_id: String,
    pub trusted_from_unix_ms: u64,
    pub expires_at_unix_ms: Option<u64>,
    pub revoked: bool,
    pub trust_anchor: ServiceIdentityTrustAnchor,
}

impl AtlasPublisherTrustBinding {
    pub fn validate_at(&self, now_unix_ms: u64) -> Result<()> {
        require_schema(&self.schema_version, ATLAS_TRUST_BINDING_SCHEMA)?;
        self.publisher.validate()?;
        validate_identifier(&self.signer_identity_id, "Atlas trusted signer identity id")?;
        if self.trust_anchor.schema_version != AtlasRepositorySigningIdentity::TRUST_ANCHOR_SCHEMA
            || self.trust_anchor.identity_id != self.signer_identity_id
            || derive_service_identity_id::<AtlasRepositorySigningIdentity>(
                &self.trust_anchor.public_key,
            )? != self.signer_identity_id
            || self.trusted_from_unix_ms > now_unix_ms
            || self
                .expires_at_unix_ms
                .is_some_and(|expires| expires <= now_unix_ms)
            || self.revoked
        {
            bail!("Atlas publisher trust binding is invalid, inactive, expired, or revoked")
        }
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
pub fn sign_atlas_publication(
    signer: &ServiceIdentitySigner<AtlasRepositorySigningIdentity>,
    publisher: AtlasRepositoryIdentity,
    publication_sequence: u64,
    runtime_id: String,
    runtime_incarnation_id: String,
    body_basis: crate::RepositoryBodyObservationBasis,
    verse_id: String,
    source: Option<AtlasMindSourceVersion>,
    mind_commit: Option<AtlasMindCommitBinding>,
    published_at_unix_ms: u64,
    payload: AtlasPublicationPayload,
) -> Result<AtlasPublicationEnvelope> {
    let canonical_payload_msgpack_sha256 = sha256(&canonical_atlas_payload_msgpack(&payload)?);
    let mut statement = AtlasPublicationStatement {
        schema_version: ATLAS_PUBLICATION_SCHEMA.into(),
        publication_id: String::new(),
        publisher,
        publisher_identity_id: signer.entry().identity_id.clone(),
        publication_sequence,
        runtime_id,
        runtime_incarnation_id,
        body_basis,
        verse_id,
        visibility: AtlasPublicationVisibility::GamecultLocal,
        source,
        mind_commit,
        canonical_payload_msgpack_sha256,
        published_at_unix_ms,
        payload,
    };
    validate_statement_without_id(&statement)?;
    statement.publication_id = derive_publication_id(&statement)?;
    let signature = signer
        .sign::<AtlasPublicationPurpose>(&rmp_serde::to_vec_named(&statement)?)
        .signature;
    Ok(AtlasPublicationEnvelope {
        statement,
        signature,
    })
}

pub fn verify_atlas_publication(
    trust: &AtlasPublisherTrustBinding,
    publication: &AtlasPublicationEnvelope,
    now_unix_ms: u64,
) -> Result<()> {
    if rmp_serde::to_vec(publication)?.len() > ATLAS_MAX_PUBLICATION_BYTES {
        bail!("Atlas publication exceeds the signed transport size limit")
    }
    trust.validate_at(now_unix_ms)?;
    let statement = &publication.statement;
    validate_atlas_publication_statement(statement)?;
    if trust.publisher != statement.publisher
        || trust.signer_identity_id != statement.publisher_identity_id
        || publication.signature.len() != 64
    {
        bail!("Atlas publication owner, signer, or signature shape is invalid")
    }
    verify_service_identity_signature::<AtlasRepositorySigningIdentity, AtlasPublicationPurpose>(
        &trust.trust_anchor,
        &rmp_serde::to_vec_named(statement)?,
        &ServiceIdentitySignature {
            identity_id: statement.publisher_identity_id.clone(),
            signature: publication.signature.clone(),
        },
    )
    .context("Atlas publication signature is invalid")
}

pub fn validate_atlas_publication_statement(statement: &AtlasPublicationStatement) -> Result<()> {
    validate_sha256(&statement.publication_id, "Atlas publication id")?;
    validate_statement_without_id(statement)?;
    if statement.publication_id != derive_publication_id(statement)? {
        bail!("Atlas publication id does not match its canonical signed statement")
    }
    Ok(())
}

pub fn canonical_atlas_payload_msgpack(payload: &AtlasPublicationPayload) -> Result<Vec<u8>> {
    match payload {
        AtlasPublicationPayload::PublisherStatus(status) => Ok(rmp_serde::to_vec_named(status)?),
        AtlasPublicationPayload::SurfaceOffer(offer) => Ok(rmp_serde::to_vec_named(offer)?),
        AtlasPublicationPayload::DependencyClaim(claim) => Ok(rmp_serde::to_vec_named(claim)?),
        AtlasPublicationPayload::DependencyVerification(verification) => {
            Ok(rmp_serde::to_vec_named(verification)?)
        }
    }
}

/// Exact bytes used by the local CultCache `DatabaseEntry` encoding. This is
/// the source-version payload, distinct from the named canonical publication
/// payload above.
pub fn atlas_source_payload_msgpack(payload: &AtlasPublicationPayload) -> Result<Vec<u8>> {
    match payload {
        AtlasPublicationPayload::PublisherStatus(status) => Ok(rmp_serde::to_vec(status)?),
        AtlasPublicationPayload::SurfaceOffer(offer) => Ok(rmp_serde::to_vec(offer)?),
        AtlasPublicationPayload::DependencyClaim(claim) => Ok(rmp_serde::to_vec(claim)?),
        AtlasPublicationPayload::DependencyVerification(verification) => {
            Ok(rmp_serde::to_vec(verification)?)
        }
    }
}

fn validate_statement_without_id(statement: &AtlasPublicationStatement) -> Result<()> {
    require_schema(&statement.schema_version, ATLAS_PUBLICATION_SCHEMA)?;
    statement.publisher.validate()?;
    statement.payload.validate()?;
    validate_identifier(
        &statement.publisher_identity_id,
        "Atlas publication signer identity id",
    )?;
    validate_identifier(&statement.runtime_id, "Atlas publication runtime id")?;
    validate_identifier(
        &statement.runtime_incarnation_id,
        "Atlas publication runtime incarnation id",
    )?;
    validate_identifier(&statement.verse_id, "Atlas publication Verse id")?;
    if !statement.verse_id.starts_with("cultmesh://gamecult-local/") {
        bail!("Atlas publication is not bound to the GameCult-local Verse")
    }
    if statement.publication_sequence == 0
        || statement.publisher != *statement.payload.owner()
        || statement.body_basis.swarm_id != statement.publisher.swarm_id
        || statement.body_basis.workspace_id != statement.publisher.workspace_id
        || statement.body_basis.runtime_id != statement.runtime_id
        || statement.body_basis.generation == 0
    {
        bail!(
            "Atlas publication owner, sequence, runtime, or repository Body basis is inconsistent"
        )
    }
    validate_identifier(
        &statement.body_basis.schema_version,
        "Atlas Body basis schema",
    )?;
    validate_identifier(
        &statement.body_basis.observation_id,
        "Atlas Body observation id",
    )?;
    validate_repository_body_sha256(
        &statement.body_basis.body_binding_sha256,
        "Atlas Body binding digest",
    )?;
    validate_repository_body_sha256(
        &statement.body_basis.manifest_root_sha256,
        "Atlas Body manifest digest",
    )?;

    let canonical_payload_sha256 = sha256(&canonical_atlas_payload_msgpack(&statement.payload)?);
    let source_payload_sha256 = sha256(&atlas_source_payload_msgpack(&statement.payload)?);
    if canonical_payload_sha256 != statement.canonical_payload_msgpack_sha256 {
        bail!("Atlas canonical payload MessagePack digest is invalid")
    }
    match (
        statement.payload.requires_mind_binding(),
        &statement.source,
        &statement.mind_commit,
    ) {
        (false, None, None) => {}
        (true, Some(source), Some(commit)) => {
            source.validate()?;
            commit.validate()?;
            if source != &commit.source
                || source.document_type != statement.payload.schema()
                || source.document_key != statement.payload.key()
                || source.schema_id.as_deref() != Some(statement.payload.schema())
                || source.payload_sha256 != source_payload_sha256
            {
                bail!(
                    "Atlas publication source, Mind receipt binding, schema, key, or payload digest was substituted"
                )
            }
        }
        _ => {
            bail!(
                "Atlas Mind document publications require exact source/commit binding; publisher status forbids it"
            )
        }
    }
    if let AtlasPublicationPayload::PublisherStatus(status) = &statement.payload {
        if status.publisher != statement.publisher
            || status.runtime_id != statement.runtime_id
            || status.runtime_incarnation_id != statement.runtime_incarnation_id
            || status.heartbeat_at_unix_ms != statement.published_at_unix_ms
        {
            bail!(
                "Atlas publisher status payload is not exact for its publication runtime and time"
            )
        }
    }
    Ok(())
}

fn derive_publication_id(statement: &AtlasPublicationStatement) -> Result<String> {
    let mut basis = statement.clone();
    basis.publication_id.clear();
    let basis_msgpack = rmp_serde::to_vec_named(&basis)?;
    Ok(sha256(
        &[
            b"gamecult.model.atlas-publication.id.v0\0".as_slice(),
            basis_msgpack.as_slice(),
        ]
        .concat(),
    ))
}

pub(crate) fn sha256(bytes: &[u8]) -> String {
    format!("sha256-{:x}", Sha256::digest(bytes))
}
