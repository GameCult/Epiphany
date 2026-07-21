use std::{
    net::UdpSocket,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result, anyhow, bail};
use cultcache_rs::{CultCache, DatabaseEntry, SingleFileMessagePackBackingStore};
use cultnet_rs::{
    CultNetMessage, CultNetRawDocumentRecord, CultNetRawPayloadEncoding,
    CultNetRudpSocketTransportConnection, CultNetRudpSocketTransportOptions, CultNetWireContract,
    GameCultServiceTrustAnchorRecord, ServiceIdentityProfile, ServiceIdentitySignature,
    ServiceIdentitySigner, ServiceSignaturePurpose, decode_cultnet_message_from_slice,
    encode_cultnet_message_to_vec, verify_service_identity_signature_with_public_key,
};
use sha2::{Digest, Sha256};

pub const PERMIT_REQUEST_SCHEMA: &str = "bifrost.persona_discord_delivery_permit_request.v0";
pub const PERMIT_SCHEMA: &str = "epiphany.persona_discord_delivery_permit.v0";
pub const PERMIT_REQUEST_PURPOSE: &str = "bifrost.persona-discord.delivery-permit-request.v0";
pub const PERMIT_PURPOSE: &str = "epiphany.persona-discord.delivery-permit.v0";
pub const PERMIT_RUDP_CONNECTION_ID: u32 = 0xe91f_0002;

pub enum BifrostPermitRequestIdentity {}
pub struct BifrostPermitRequestPurpose;
pub enum EpiphanyPermitIdentity {}
pub struct EpiphanyPermitPurpose;

macro_rules! profile {
    ($profile:ty,$purpose:ty,$prefix:literal,$id:literal,$sig:literal,$purpose_value:ident) => {
        impl ServiceIdentityProfile for $profile {
            const PRIVATE_TYPE: &'static str = concat!($prefix, ".identity.private.v0");
            const PRIVATE_SCHEMA: &'static str = Self::PRIVATE_TYPE;
            const PRIVATE_KEY: &'static str = concat!($prefix, "-identity");
            const TRUST_ANCHOR_TYPE: &'static str = concat!($prefix, ".identity.trust_anchor.v0");
            const TRUST_ANCHOR_SCHEMA: &'static str = Self::TRUST_ANCHOR_TYPE;
            const TRUST_ANCHOR_KEY: &'static str = concat!($prefix, "-identity-public");
            const ID_DOMAIN: &'static [u8] = $id;
            const SIGNATURE_DOMAIN: &'static [u8] = $sig;
            const PROTECTOR_CONTEXT: &'static str = concat!($prefix, "-v0");
        }
        impl ServiceSignaturePurpose<$profile> for $purpose {
            const PURPOSE: &'static [u8] = $purpose_value.as_bytes();
        }
    };
}
profile!(
    BifrostPermitRequestIdentity,
    BifrostPermitRequestPurpose,
    "bifrost.persona_discord_delivery_permit_request",
    b"bifrost.persona-discord-delivery-permit-request.identity.v0\0",
    b"bifrost.persona-discord-delivery-permit-request.signature.v0\0",
    PERMIT_REQUEST_PURPOSE
);
profile!(
    EpiphanyPermitIdentity,
    EpiphanyPermitPurpose,
    "epiphany.persona_discord_delivery_permit",
    b"epiphany.persona-discord-delivery-permit.identity.v0\0",
    b"epiphany.persona-discord-delivery-permit.signature.v0\0",
    PERMIT_PURPOSE
);

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "bifrost.persona_discord_delivery_permit_request",
    schema = "bifrost.persona_discord_delivery_permit_request.v0"
)]
pub struct PersonaDiscordPermitRequest {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub request_payload_sha256: String,
    #[cultcache(key = 3)]
    pub target_runtime_id: String,
    #[cultcache(key = 4)]
    pub nonce: String,
    #[cultcache(key = 5)]
    pub issued_at: String,
    #[cultcache(key = 6)]
    pub expires_at: String,
    #[cultcache(key = 7)]
    pub requester_identity_id: String,
    #[cultcache(key = 8)]
    pub private_state_exposed: bool,
    #[cultcache(key = 9)]
    pub requester_signature: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.persona_discord_delivery_permit",
    schema = "epiphany.persona_discord_delivery_permit.v0"
)]
pub struct PersonaDiscordPermit {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub permit_id: String,
    #[cultcache(key = 2)]
    pub request_id: String,
    #[cultcache(key = 3)]
    pub request_payload_sha256: String,
    #[cultcache(key = 4)]
    pub target_runtime_id: String,
    #[cultcache(key = 5)]
    pub nonce: String,
    #[cultcache(key = 6)]
    pub requester_identity_id: String,
    #[cultcache(key = 7)]
    pub brake_state_document_id: String,
    #[cultcache(key = 8)]
    pub brake_state_document_sha256: String,
    #[cultcache(key = 9)]
    pub brake_observed_at: String,
    #[cultcache(key = 10)]
    pub issued_at: String,
    #[cultcache(key = 11)]
    pub expires_at: String,
    #[cultcache(key = 12)]
    pub provider_identity_id: String,
    #[cultcache(key = 13)]
    pub private_state_exposed: bool,
    #[cultcache(key = 14)]
    pub provider_signature: Vec<u8>,
}

pub struct PersonaPermitIssuerConfig {
    pub runtime_id: String,
    pub cultmesh_store: PathBuf,
    pub replay_store: PathBuf,
    pub requester_anchor: GameCultServiceTrustAnchorRecord,
    pub provider_anchor: GameCultServiceTrustAnchorRecord,
}

pub fn ensure_epiphany_permit_identity(
    identity_store: &Path,
    anchor_output: &Path,
    runtime_id: &str,
) -> Result<()> {
    let signer = if identity_store.exists() {
        cultnet_rs::open_service_identity_at::<EpiphanyPermitIdentity>(identity_store)?
    } else {
        cultnet_rs::enroll_service_identity_at::<EpiphanyPermitIdentity>(identity_store)?
    };
    if anchor_output.exists() {
        let anchor: GameCultServiceTrustAnchorRecord =
            rmp_serde::from_slice(&std::fs::read(anchor_output)?)?;
        anchor.validate()?;
        if anchor.service_id != "epiphany-persona-discord-permit"
            || anchor.runtime_id != runtime_id
            || anchor.signer_identity_id != signer.entry().identity_id
            || anchor.signer_public_key != signer.entry().public_key
            || anchor.signing_purpose != PERMIT_PURPOSE
            || anchor.signed_schema != PERMIT_SCHEMA
        {
            bail!("existing permit anchor candidate differs from enrolled identity/profile");
        }
        return Ok(());
    }
    let anchor = GameCultServiceTrustAnchorRecord {
        schema_version: cultnet_rs::GAMECULT_SERVICE_TRUST_ANCHOR_SCHEMA.into(),
        trust_anchor_id: format!("epiphany-persona-discord-permit:{runtime_id}:v0"),
        service_id: "epiphany-persona-discord-permit".into(),
        runtime_id: runtime_id.into(),
        signer_identity_id: signer.entry().identity_id.clone(),
        signer_public_key: signer.entry().public_key.clone(),
        signature_algorithm: "ed25519".into(),
        signing_purpose: PERMIT_PURPOSE.into(),
        signed_schema: PERMIT_SCHEMA.into(),
        binding_authority: "root".into(),
        bound_at_unix_millis: chrono::Utc::now().timestamp_millis().max(1) as u64,
        expires_at_unix_millis: None,
        private_state_exposed: false,
    };
    anchor.validate()?;
    if let Some(parent) = anchor_output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(anchor_output, rmp_serde::to_vec(&anchor)?)?;
    Ok(())
}

pub fn issue_persona_discord_permit(
    config: &PersonaPermitIssuerConfig,
    signer: &ServiceIdentitySigner<EpiphanyPermitIdentity>,
    request: &PersonaDiscordPermitRequest,
) -> Result<PersonaDiscordPermit> {
    validate_provider_anchor(config, signer)?;
    validate_request(config, request)?;
    let key = format!("permit:{}", request.request_payload_sha256);
    if let Some(existing) = load_permit(&config.replay_store, &key)? {
        validate_permit_request_binding(&existing, request)?;
        return Ok(existing);
    }
    let brake =
        crate::load_epiphany_cultmesh_swarm_brake(&config.cultmesh_store, &config.runtime_id)?
            .ok_or_else(|| anyhow!("permit issuer requires canonical brake state"))?;
    if brake.status != "released" {
        bail!("permit issuer is braked");
    }
    let brake_bytes = rmp_serde::to_vec(&brake)?;
    let issued = chrono::Utc::now();
    let mut permit = PersonaDiscordPermit {
        schema_version: PERMIT_SCHEMA.into(),
        permit_id: key.clone(),
        request_id: request.request_id.clone(),
        request_payload_sha256: request.request_payload_sha256.clone(),
        target_runtime_id: request.target_runtime_id.clone(),
        nonce: request.nonce.clone(),
        requester_identity_id: request.requester_identity_id.clone(),
        brake_state_document_id: brake.brake_id.clone(),
        brake_state_document_sha256: format!("sha256-{:x}", Sha256::digest(brake_bytes)),
        brake_observed_at: issued.to_rfc3339(),
        issued_at: issued.to_rfc3339(),
        expires_at: (issued + chrono::Duration::seconds(5)).to_rfc3339(),
        provider_identity_id: signer.entry().identity_id.clone(),
        private_state_exposed: false,
        provider_signature: vec![],
    };
    let proof = signer.sign::<EpiphanyPermitPurpose>(&permit_signing_payload(&permit)?);
    permit.provider_signature = proof.signature;
    let mut cache = CultCache::new();
    cache.register_entry_type::<PersonaDiscordPermit>()?;
    let (entry, _) = cache.prepare_entry(&key, &permit)?;
    if !SingleFileMessagePackBackingStore::new(&config.replay_store)
        .insert_entry_if_absent(entry)?
    {
        let existing = load_permit(&config.replay_store, &key)?
            .ok_or_else(|| anyhow!("permit replay race lost without record"))?;
        if existing != permit {
            bail!("permit replay key collision");
        }
        return Ok(existing);
    }
    Ok(permit)
}

fn validate_provider_anchor(
    config: &PersonaPermitIssuerConfig,
    signer: &ServiceIdentitySigner<EpiphanyPermitIdentity>,
) -> Result<()> {
    let anchor = &config.provider_anchor;
    anchor.validate()?;
    let now = chrono::Utc::now().timestamp_millis().max(0) as u64;
    if anchor.service_id != "epiphany-persona-discord-permit"
        || anchor.runtime_id != config.runtime_id
        || anchor.signer_identity_id != signer.entry().identity_id
        || anchor.signer_public_key != signer.entry().public_key
        || anchor.signing_purpose != PERMIT_PURPOSE
        || anchor.signed_schema != PERMIT_SCHEMA
        || anchor.bound_at_unix_millis > now
        || anchor
            .expires_at_unix_millis
            .is_some_and(|expiry| expiry <= now)
    {
        bail!("permit provider anchor is not the exact current root binding")
    }
    Ok(())
}

fn validate_request(
    config: &PersonaPermitIssuerConfig,
    value: &PersonaDiscordPermitRequest,
) -> Result<()> {
    let anchor = &config.requester_anchor;
    anchor.validate()?;
    let now_millis = chrono::Utc::now().timestamp_millis().max(0) as u64;
    if anchor.service_id != "bifrost-persona-discord-delivery"
        || anchor.runtime_id != "bifrost-discord-yggdrasil"
        || anchor.signing_purpose != PERMIT_REQUEST_PURPOSE
        || anchor.signed_schema != PERMIT_REQUEST_SCHEMA
        || anchor.bound_at_unix_millis > now_millis
        || anchor
            .expires_at_unix_millis
            .is_some_and(|expiry| expiry <= now_millis)
        || value.schema_version != PERMIT_REQUEST_SCHEMA
        || value.target_runtime_id != config.runtime_id
        || value.private_state_exposed
        || value.requester_identity_id != anchor.signer_identity_id
        || !sha(&value.request_payload_sha256)
        || value.requester_signature.len() != 64
    {
        bail!("permit request contract or anchor binding is invalid");
    }
    let issued = chrono::DateTime::parse_from_rfc3339(&value.issued_at)?;
    let expires = chrono::DateTime::parse_from_rfc3339(&value.expires_at)?;
    let now = chrono::Utc::now();
    if expires <= issued
        || expires - issued > chrono::Duration::seconds(5)
        || issued > now + chrono::Duration::seconds(1)
        || expires < now
    {
        bail!("permit request is stale or has an invalid lifetime");
    }
    verify_service_identity_signature_with_public_key::<
        BifrostPermitRequestIdentity,
        BifrostPermitRequestPurpose,
    >(
        &anchor.signer_public_key,
        &permit_request_signing_payload(value)?,
        &ServiceIdentitySignature {
            identity_id: value.requester_identity_id.clone(),
            signature: value.requester_signature.clone(),
        },
    )
}

fn validate_permit_request_binding(
    value: &PersonaDiscordPermit,
    request: &PersonaDiscordPermitRequest,
) -> Result<()> {
    if value.request_id != request.request_id
        || value.request_payload_sha256 != request.request_payload_sha256
        || value.target_runtime_id != request.target_runtime_id
        || value.nonce != request.nonce
        || value.requester_identity_id != request.requester_identity_id
    {
        bail!("permit replay substituted request authority");
    }
    Ok(())
}
pub fn permit_request_signing_payload(v: &PersonaDiscordPermitRequest) -> Result<Vec<u8>> {
    Ok(rmp_serde::to_vec(&(
        v.schema_version.as_str(),
        v.request_id.as_str(),
        v.request_payload_sha256.as_str(),
        v.target_runtime_id.as_str(),
        v.nonce.as_str(),
        v.issued_at.as_str(),
        v.expires_at.as_str(),
        v.requester_identity_id.as_str(),
        v.private_state_exposed,
    ))?)
}
pub fn permit_signing_payload(v: &PersonaDiscordPermit) -> Result<Vec<u8>> {
    Ok(rmp_serde::to_vec(&(
        v.schema_version.as_str(),
        v.permit_id.as_str(),
        v.request_id.as_str(),
        v.request_payload_sha256.as_str(),
        v.target_runtime_id.as_str(),
        v.nonce.as_str(),
        v.requester_identity_id.as_str(),
        v.brake_state_document_id.as_str(),
        v.brake_state_document_sha256.as_str(),
        v.brake_observed_at.as_str(),
        v.issued_at.as_str(),
        v.expires_at.as_str(),
        v.provider_identity_id.as_str(),
        v.private_state_exposed,
    ))?)
}
fn sha(v: &str) -> bool {
    v.len() == 71
        && v.starts_with("sha256-")
        && v[7..]
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn load_permit(path: &Path, key: &str) -> Result<Option<PersonaDiscordPermit>> {
    if !path.exists() {
        return Ok(None);
    };
    for row in SingleFileMessagePackBackingStore::new(path).pull_all_read_only_snapshot()? {
        if row.key == key {
            if row.r#type != PersonaDiscordPermit::TYPE {
                bail!("permit replay key occupied by alien type")
            };
            return Ok(Some(rmp_serde::from_slice(&row.payload)?));
        }
    }
    Ok(None)
}

pub fn serve_persona_discord_permit_rudp(
    socket: UdpSocket,
    config: &PersonaPermitIssuerConfig,
    signer: &ServiceIdentitySigner<EpiphanyPermitIdentity>,
) -> Result<()> {
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    let mut transport =
        CultNetRudpSocketTransportConnection::new(CultNetRudpSocketTransportOptions::server(
            &config.runtime_id,
            socket,
            PERMIT_RUDP_CONNECTION_ID,
        ))?;
    loop {
        match transport.receive_once() {
            Ok(Some(frame)) => {
                if let Ok(reply) = process_frame(config, signer, &frame.payload) {
                    let payload = encode_cultnet_message_to_vec(
                        &reply,
                        CultNetWireContract::CultNetSchemaV0,
                    )?;
                    transport.send("schema", payload)?;
                }
            }
            Ok(None) => {}
            Err(error) => eprintln!("permit issuer discarded hostile frame: {error:#}"),
        }
        transport.poll_resends()?;
    }
}
fn process_frame(
    config: &PersonaPermitIssuerConfig,
    signer: &ServiceIdentitySigner<EpiphanyPermitIdentity>,
    payload: &[u8],
) -> Result<CultNetMessage> {
    let message = decode_cultnet_message_from_slice(payload, CultNetWireContract::CultNetSchemaV0)
        .context("permit issuer rejected non-CultNet payload")?;
    let CultNetMessage::DocumentPutRaw { document, .. } = message else {
        bail!("permit issuer accepts one raw typed request")
    };
    if document.schema_id != PERMIT_REQUEST_SCHEMA
        || document.payload_encoding != CultNetRawPayloadEncoding::Messagepack
        || document.source_role.as_deref() != Some("bifrost-persona-discord-delivery")
    {
        bail!("foreign permit request envelope")
    }
    let request: PersonaDiscordPermitRequest = rmp_serde::from_slice(&document.payload)?;
    if document.record_key != request.request_id
        || document.source_runtime_id.as_deref() != Some("bifrost-discord-yggdrasil")
        || document.source_agent_id.as_deref() != Some(request.requester_identity_id.as_str())
        || document.tags.as_deref() != Some(&["cultnet.transport.rudp.v0".to_string()])
    {
        bail!("permit envelope identity substitution")
    }
    let permit = issue_persona_discord_permit(config, signer, &request)?;
    Ok(CultNetMessage::DocumentPutRaw {
        message_id: format!("permit-{}", permit.permit_id),
        document: CultNetRawDocumentRecord {
            schema_id: PERMIT_SCHEMA.into(),
            record_key: permit.permit_id.clone(),
            stored_at: permit.issued_at.clone(),
            payload_encoding: CultNetRawPayloadEncoding::Messagepack,
            payload: rmp_serde::to_vec(&permit)?,
            source_runtime_id: Some(config.runtime_id.clone()),
            source_agent_id: Some(permit.provider_identity_id.clone()),
            source_role: Some("epiphany-persona-discord-permit".into()),
            tags: Some(vec!["cultnet.transport.rudp.v0".into()]),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anchor(
        service: &str,
        runtime: &str,
        identity_id: String,
        public_key: Vec<u8>,
        purpose: &str,
        schema: &str,
    ) -> GameCultServiceTrustAnchorRecord {
        GameCultServiceTrustAnchorRecord {
            schema_version: cultnet_rs::GAMECULT_SERVICE_TRUST_ANCHOR_SCHEMA.into(),
            trust_anchor_id: format!("{service}:{runtime}:{schema}"),
            service_id: service.into(),
            runtime_id: runtime.into(),
            signer_identity_id: identity_id,
            signer_public_key: public_key,
            signature_algorithm: "ed25519".into(),
            signing_purpose: purpose.into(),
            signed_schema: schema.into(),
            binding_authority: "root".into(),
            bound_at_unix_millis: chrono::Utc::now().timestamp_millis().max(1) as u64,
            expires_at_unix_millis: None,
            private_state_exposed: false,
        }
    }

    #[test]
    fn changed_nonce_cannot_reauthorize_one_delivery_request() {
        let permit = PersonaDiscordPermit {
            schema_version: PERMIT_SCHEMA.into(),
            permit_id: "permit:sha256-x".into(),
            request_id: "delivery-1".into(),
            request_payload_sha256: format!("sha256-{}", "a".repeat(64)),
            target_runtime_id: "epiphany-yggdrasil".into(),
            nonce: "nonce-1".into(),
            requester_identity_id: "bifrost".into(),
            brake_state_document_id: "epiphany/swarm-brake".into(),
            brake_state_document_sha256: format!("sha256-{}", "b".repeat(64)),
            brake_observed_at: "2026-07-21T00:00:00Z".into(),
            issued_at: "2026-07-21T00:00:00Z".into(),
            expires_at: "2026-07-21T00:00:05Z".into(),
            provider_identity_id: "epiphany".into(),
            private_state_exposed: false,
            provider_signature: vec![0; 64],
        };
        let request = PersonaDiscordPermitRequest {
            schema_version: PERMIT_REQUEST_SCHEMA.into(),
            request_id: "delivery-1".into(),
            request_payload_sha256: permit.request_payload_sha256.clone(),
            target_runtime_id: permit.target_runtime_id.clone(),
            nonce: "nonce-2".into(),
            issued_at: "2026-07-21T00:00:00Z".into(),
            expires_at: "2026-07-21T00:00:05Z".into(),
            requester_identity_id: permit.requester_identity_id.clone(),
            private_state_exposed: false,
            requester_signature: vec![0; 64],
        };
        assert!(validate_permit_request_binding(&permit, &request).is_err());
    }

    #[test]
    fn cultnet_wire_request_issues_one_exact_replayable_permit() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let requester = cultnet_rs::enroll_service_identity_at::<BifrostPermitRequestIdentity>(
            &temp.path().join("requester.cc"),
        )?;
        let provider = cultnet_rs::enroll_service_identity_at::<EpiphanyPermitIdentity>(
            &temp.path().join("provider.cc"),
        )?;
        let request_anchor = anchor(
            "bifrost-persona-discord-delivery",
            "bifrost-discord-yggdrasil",
            requester.entry().identity_id.clone(),
            requester.entry().public_key.clone(),
            PERMIT_REQUEST_PURPOSE,
            PERMIT_REQUEST_SCHEMA,
        );
        let provider_anchor = anchor(
            "epiphany-persona-discord-permit",
            "epiphany-yggdrasil",
            provider.entry().identity_id.clone(),
            provider.entry().public_key.clone(),
            PERMIT_PURPOSE,
            PERMIT_SCHEMA,
        );
        let cultmesh = temp.path().join("cultmesh.cc");
        crate::write_epiphany_cultmesh_swarm_brake(
            &cultmesh,
            "epiphany-yggdrasil",
            crate::default_epiphany_cultmesh_swarm_brake(&chrono::Utc::now().to_rfc3339()),
        )?;
        let config = PersonaPermitIssuerConfig {
            runtime_id: "epiphany-yggdrasil".into(),
            cultmesh_store: cultmesh,
            replay_store: temp.path().join("replay.cc"),
            requester_anchor: request_anchor,
            provider_anchor,
        };
        let now = chrono::Utc::now();
        let mut request = PersonaDiscordPermitRequest {
            schema_version: PERMIT_REQUEST_SCHEMA.into(),
            request_id: "delivery-1".into(),
            request_payload_sha256: format!("sha256-{}", "a".repeat(64)),
            target_runtime_id: "epiphany-yggdrasil".into(),
            nonce: "nonce-1".into(),
            issued_at: now.to_rfc3339(),
            expires_at: (now + chrono::Duration::seconds(5)).to_rfc3339(),
            requester_identity_id: requester.entry().identity_id.clone(),
            private_state_exposed: false,
            requester_signature: vec![],
        };
        request.requester_signature = requester
            .sign::<BifrostPermitRequestPurpose>(&permit_request_signing_payload(&request)?)
            .signature;
        let message = CultNetMessage::DocumentPutRaw {
            message_id: "permit-request-1".into(),
            document: CultNetRawDocumentRecord {
                schema_id: PERMIT_REQUEST_SCHEMA.into(),
                record_key: request.request_id.clone(),
                stored_at: request.issued_at.clone(),
                payload_encoding: CultNetRawPayloadEncoding::Messagepack,
                payload: rmp_serde::to_vec(&request)?,
                source_runtime_id: Some("bifrost-discord-yggdrasil".into()),
                source_agent_id: Some(request.requester_identity_id.clone()),
                source_role: Some("bifrost-persona-discord-delivery".into()),
                tags: Some(vec!["cultnet.transport.rudp.v0".into()]),
            },
        };
        let wire = encode_cultnet_message_to_vec(&message, CultNetWireContract::CultNetSchemaV0)?;
        let first = process_frame(&config, &provider, &wire)?;
        let second = process_frame(&config, &provider, &wire)?;
        assert_eq!(first, second);
        Ok(())
    }
}
