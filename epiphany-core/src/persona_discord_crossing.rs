use std::path::Path;

use anyhow::{Result, anyhow, bail};
use cultcache_rs::{CultCache, DatabaseEntry, SingleFileMessagePackBackingStore};
use serde::Serialize;
use sha2::{Digest, Sha256};

use cultnet_rs::{
    GAMECULT_SERVICE_TRUST_ANCHOR_SCHEMA, GameCultServiceTrustAnchorRecord, ServiceIdentityProfile,
    ServiceIdentitySignature, ServiceIdentitySigner, ServiceSignaturePurpose,
    verify_service_identity_signature_with_public_key,
};

pub const PERSONA_DISCORD_DELIVERY_REQUEST_SCHEMA_VERSION: &str =
    "epiphany.persona_discord_delivery_request.v0";
pub const PERSONA_DISCORD_DELIVERY_RECEIPT_SCHEMA_VERSION: &str =
    "bifrost.persona_discord_delivery_receipt.v0";
pub const PERSONA_DISCORD_DELIVERY_REQUEST_SIGNING_PURPOSE: &str =
    "epiphany.persona-discord.delivery-request.v0";
pub const PERSONA_DISCORD_DELIVERY_RECEIPT_SIGNING_PURPOSE: &str =
    "bifrost.persona-discord.delivery-receipt.v0";
pub const EPIPHANY_PERSONA_MOUTH_SERVICE_ID: &str = "epiphany-persona-mouth";
pub const EPIPHANY_PERSONA_MOUTH_RUNTIME_ID: &str = "epiphany-yggdrasil";
pub const BIFROST_PERSONA_DELIVERY_SERVICE_ID: &str = "bifrost-persona-discord-delivery";
pub const BIFROST_PERSONA_DELIVERY_RUNTIME_ID: &str = "bifrost-discord-yggdrasil";

pub enum EpiphanyPersonaDeliveryRequestIdentity {}
pub struct EpiphanyPersonaDeliveryRequestPurpose;
pub enum BifrostPersonaDeliveryReceiptIdentity {}
pub struct BifrostPersonaDeliveryReceiptPurpose;

impl ServiceIdentityProfile for EpiphanyPersonaDeliveryRequestIdentity {
    const PRIVATE_TYPE: &'static str =
        "epiphany.persona_discord_delivery_request_identity.private.v0";
    const PRIVATE_SCHEMA: &'static str = Self::PRIVATE_TYPE;
    const PRIVATE_KEY: &'static str = "epiphany-persona-discord-delivery-request-identity";
    const TRUST_ANCHOR_TYPE: &'static str =
        "epiphany.persona_discord_delivery_request_identity.trust_anchor.v0";
    const TRUST_ANCHOR_SCHEMA: &'static str = Self::TRUST_ANCHOR_TYPE;
    const TRUST_ANCHOR_KEY: &'static str =
        "epiphany-persona-discord-delivery-request-identity-public";
    const ID_DOMAIN: &'static [u8] = b"epiphany.persona-discord-delivery-request.identity.v0\0";
    const SIGNATURE_DOMAIN: &'static [u8] =
        b"epiphany.persona-discord-delivery-request.signature.v0\0";
    const PROTECTOR_CONTEXT: &'static str = "epiphany-persona-discord-delivery-request-v0";
}
impl ServiceSignaturePurpose<EpiphanyPersonaDeliveryRequestIdentity>
    for EpiphanyPersonaDeliveryRequestPurpose
{
    const PURPOSE: &'static [u8] = PERSONA_DISCORD_DELIVERY_REQUEST_SIGNING_PURPOSE.as_bytes();
}
impl ServiceIdentityProfile for BifrostPersonaDeliveryReceiptIdentity {
    const PRIVATE_TYPE: &'static str =
        "bifrost.persona_discord_delivery_receipt_identity.private.v0";
    const PRIVATE_SCHEMA: &'static str = Self::PRIVATE_TYPE;
    const PRIVATE_KEY: &'static str = "bifrost-persona-discord-delivery-receipt-identity";
    const TRUST_ANCHOR_TYPE: &'static str =
        "bifrost.persona_discord_delivery_receipt_identity.trust_anchor.v0";
    const TRUST_ANCHOR_SCHEMA: &'static str = Self::TRUST_ANCHOR_TYPE;
    const TRUST_ANCHOR_KEY: &'static str =
        "bifrost-persona-discord-delivery-receipt-identity-public";
    const ID_DOMAIN: &'static [u8] = b"bifrost.persona-discord-delivery-receipt.identity.v0\0";
    const SIGNATURE_DOMAIN: &'static [u8] =
        b"bifrost.persona-discord-delivery-receipt.signature.v0\0";
    const PROTECTOR_CONTEXT: &'static str = "bifrost-persona-discord-delivery-receipt-v0";
}
impl ServiceSignaturePurpose<BifrostPersonaDeliveryReceiptIdentity>
    for BifrostPersonaDeliveryReceiptPurpose
{
    const PURPOSE: &'static [u8] = PERSONA_DISCORD_DELIVERY_RECEIPT_SIGNING_PURPOSE.as_bytes();
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.persona_discord_delivery_request",
    schema = "PersonaDiscordDeliveryRequest"
)]
pub struct PersonaDiscordDeliveryRequest {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub effect_document_id: String,
    #[cultcache(key = 3)]
    pub target_runtime_id: String,
    #[cultcache(key = 4)]
    pub persona_agent_id: String,
    #[cultcache(key = 5)]
    pub channel_id: String,
    #[cultcache(key = 6)]
    pub reply_to_message_id: String,
    #[cultcache(key = 7)]
    pub content: String,
    #[cultcache(key = 8)]
    pub content_sha256: String,
    #[cultcache(key = 9)]
    pub issued_at: String,
    #[cultcache(key = 10)]
    pub expires_at: String,
    #[cultcache(key = 11)]
    pub private_state_exposed: bool,
    #[cultcache(key = 12)]
    pub signer_identity_id: String,
    #[cultcache(key = 13)]
    pub signer_signature: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "bifrost.persona_discord_delivery_receipt",
    schema = "PersonaDiscordDeliveryReceipt"
)]
pub struct PersonaDiscordDeliveryReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub request_id: String,
    #[cultcache(key = 3)]
    pub request_payload_sha256: String,
    #[cultcache(key = 4)]
    pub status: String,
    #[cultcache(key = 5)]
    pub channel_id: String,
    #[cultcache(key = 6)]
    pub reply_to_message_id: String,
    #[cultcache(key = 7)]
    pub message_id: String,
    #[cultcache(key = 8)]
    pub transport: String,
    #[cultcache(key = 9)]
    pub crossing_receipt_id: String,
    #[cultcache(key = 10)]
    pub receipt_url: String,
    #[cultcache(key = 11)]
    pub completed_at: String,
    #[cultcache(key = 12)]
    pub provider_identity_id: String,
    #[cultcache(key = 13)]
    pub private_state_exposed: bool,
    #[cultcache(key = 14)]
    pub provider_signature: Vec<u8>,
}

#[allow(clippy::too_many_arguments)]
pub fn sign_persona_discord_delivery_request(
    signer: &ServiceIdentitySigner<EpiphanyPersonaDeliveryRequestIdentity>,
    request_id: String,
    effect_document_id: String,
    target_runtime_id: String,
    persona_agent_id: String,
    channel_id: String,
    reply_to_message_id: String,
    content: String,
    issued_at: String,
    expires_at: String,
) -> Result<PersonaDiscordDeliveryRequest> {
    let mut request = PersonaDiscordDeliveryRequest {
        schema_version: PERSONA_DISCORD_DELIVERY_REQUEST_SCHEMA_VERSION.into(),
        request_id,
        effect_document_id,
        target_runtime_id,
        persona_agent_id,
        channel_id,
        reply_to_message_id,
        content_sha256: sha256_hyphen(content.as_bytes()),
        content,
        issued_at,
        expires_at,
        private_state_exposed: false,
        signer_identity_id: signer.entry().identity_id.clone(),
        signer_signature: vec![],
    };
    validate_request_shape(&request)?;
    let proof =
        signer.sign::<EpiphanyPersonaDeliveryRequestPurpose>(&request_signing_payload(&request)?);
    request.signer_signature = proof.signature;
    Ok(request)
}

pub fn open_persona_discord_request_identity(
    path: &Path,
) -> Result<ServiceIdentitySigner<EpiphanyPersonaDeliveryRequestIdentity>> {
    cultnet_rs::open_service_identity_at::<EpiphanyPersonaDeliveryRequestIdentity>(path)
}

pub fn enroll_persona_discord_request_identity(path: &Path) -> Result<()> {
    if path.exists() {
        open_persona_discord_request_identity(path)?;
        return Ok(());
    }
    cultnet_rs::enroll_service_identity_at::<EpiphanyPersonaDeliveryRequestIdentity>(path)?;
    Ok(())
}

pub fn export_persona_discord_request_anchor_candidate(
    identity_store: &Path,
    output: &Path,
    runtime_id: &str,
) -> Result<()> {
    let signer = open_persona_discord_request_identity(identity_store)?;
    if output.exists() {
        let existing = load_persona_discord_service_anchor(output)?;
        validate_persona_discord_request_anchor(&existing, runtime_id)?;
        if existing.signer_identity_id != signer.entry().identity_id
            || existing.signer_public_key != signer.entry().public_key
        {
            bail!("existing request anchor candidate belongs to a different identity");
        }
        return Ok(());
    }
    let record = GameCultServiceTrustAnchorRecord {
        schema_version: GAMECULT_SERVICE_TRUST_ANCHOR_SCHEMA.into(),
        trust_anchor_id: format!("epiphany-persona-mouth:{runtime_id}:delivery-request:v0"),
        service_id: EPIPHANY_PERSONA_MOUTH_SERVICE_ID.into(),
        runtime_id: runtime_id.into(),
        signer_identity_id: signer.entry().identity_id.clone(),
        signer_public_key: signer.entry().public_key.clone(),
        signature_algorithm: "ed25519".into(),
        signing_purpose: PERSONA_DISCORD_DELIVERY_REQUEST_SIGNING_PURPOSE.into(),
        signed_schema: PERSONA_DISCORD_DELIVERY_REQUEST_SCHEMA_VERSION.into(),
        binding_authority: "root".into(),
        bound_at_unix_millis: chrono::Utc::now().timestamp_millis().max(1) as u64,
        expires_at_unix_millis: None,
        private_state_exposed: false,
    };
    record.validate()?;
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output, rmp_serde::to_vec(&record)?)?;
    Ok(())
}

pub fn request_signing_payload(request: &PersonaDiscordDeliveryRequest) -> Result<Vec<u8>> {
    Ok(rmp_serde::to_vec(&request_signing_tuple(request))?)
}

pub fn request_payload_sha256(request: &PersonaDiscordDeliveryRequest) -> Result<String> {
    Ok(sha256_hyphen(&request_signing_payload(request)?))
}

pub fn verify_persona_discord_delivery_receipt(
    receipt: &PersonaDiscordDeliveryReceipt,
    request: &PersonaDiscordDeliveryRequest,
    anchor: &GameCultServiceTrustAnchorRecord,
) -> Result<()> {
    validate_service_anchor(
        anchor,
        BIFROST_PERSONA_DELIVERY_SERVICE_ID,
        BIFROST_PERSONA_DELIVERY_RUNTIME_ID,
        PERSONA_DISCORD_DELIVERY_RECEIPT_SIGNING_PURPOSE,
        PERSONA_DISCORD_DELIVERY_RECEIPT_SCHEMA_VERSION,
    )?;
    validate_receipt_shape(receipt)?;
    if receipt.request_id != request.request_id
        || receipt.receipt_id != request.request_id
        || receipt.channel_id != request.channel_id
        || receipt.reply_to_message_id != request.reply_to_message_id
        || receipt.request_payload_sha256 != request_payload_sha256(request)?
        || receipt.provider_identity_id != anchor.signer_identity_id
    {
        bail!("Bifrost Persona delivery receipt is not bound to the exact request");
    }
    verify_service_identity_signature_with_public_key::<
        BifrostPersonaDeliveryReceiptIdentity,
        BifrostPersonaDeliveryReceiptPurpose,
    >(
        &anchor.signer_public_key,
        &receipt_signing_payload(receipt)?,
        &ServiceIdentitySignature {
            identity_id: receipt.provider_identity_id.clone(),
            signature: receipt.provider_signature.clone(),
        },
    )
}

pub fn receipt_signing_payload(receipt: &PersonaDiscordDeliveryReceipt) -> Result<Vec<u8>> {
    Ok(rmp_serde::to_vec(&receipt_signing_tuple(receipt))?)
}

pub fn insert_persona_discord_delivery_request(
    path: &Path,
    request: &PersonaDiscordDeliveryRequest,
) -> Result<()> {
    validate_request_shape(request)?;
    if request.signer_signature.len() != 64 {
        bail!("Persona delivery request signature must be 64 bytes");
    }
    let mut cache = CultCache::new();
    cache.register_entry_type::<PersonaDiscordDeliveryRequest>()?;
    let (entry, _) = cache.prepare_entry(&request.request_id, request)?;
    if !SingleFileMessagePackBackingStore::new(path).insert_entry_if_absent(entry)? {
        let existing = load_persona_discord_delivery_request(path, &request.request_id)?;
        if existing.as_ref() != Some(request) {
            bail!("Persona delivery request identity collision");
        }
    }
    Ok(())
}

pub fn load_persona_discord_delivery_request(
    path: &Path,
    id: &str,
) -> Result<Option<PersonaDiscordDeliveryRequest>> {
    load_exact(path, id)
}

pub fn load_persona_discord_delivery_receipt(
    path: &Path,
    id: &str,
) -> Result<Option<PersonaDiscordDeliveryReceipt>> {
    load_exact(path, id)
}

pub fn load_persona_discord_receipt_anchor(
    path: &Path,
) -> Result<GameCultServiceTrustAnchorRecord> {
    load_persona_discord_service_anchor(path)
}

pub fn load_persona_discord_service_anchor(
    path: &Path,
) -> Result<GameCultServiceTrustAnchorRecord> {
    let bytes = std::fs::read(path)?;
    rmp_serde::from_slice(&bytes)
        .map_err(|error| anyhow!("Bifrost trust anchor is malformed: {error}"))
}

pub fn validate_persona_discord_request_anchor(
    anchor: &GameCultServiceTrustAnchorRecord,
    expected_runtime_id: &str,
) -> Result<()> {
    validate_service_anchor(
        anchor,
        EPIPHANY_PERSONA_MOUTH_SERVICE_ID,
        expected_runtime_id,
        PERSONA_DISCORD_DELIVERY_REQUEST_SIGNING_PURPOSE,
        PERSONA_DISCORD_DELIVERY_REQUEST_SCHEMA_VERSION,
    )
}

fn validate_service_anchor(
    anchor: &GameCultServiceTrustAnchorRecord,
    service_id: &str,
    runtime_id: &str,
    purpose: &str,
    schema: &str,
) -> Result<()> {
    anchor.validate()?;
    let now = chrono::Utc::now().timestamp_millis().max(0) as u64;
    if anchor.schema_version != GAMECULT_SERVICE_TRUST_ANCHOR_SCHEMA
        || anchor.service_id != service_id
        || anchor.runtime_id != runtime_id
        || anchor.signing_purpose != purpose
        || anchor.signed_schema != schema
        || anchor.binding_authority != "root"
        || anchor.signature_algorithm != "ed25519"
        || anchor.bound_at_unix_millis > now
        || anchor
            .expires_at_unix_millis
            .is_some_and(|expiry| expiry <= now)
        || anchor.private_state_exposed
    {
        bail!("service trust anchor does not bind the exact Persona Discord crossing profile");
    }
    Ok(())
}

fn load_exact<T: DatabaseEntry>(path: &Path, id: &str) -> Result<Option<T>> {
    if !path.exists() {
        return Ok(None);
    }
    let entries = SingleFileMessagePackBackingStore::new(path).pull_all_read_only_snapshot()?;
    let mut found = None;
    for entry in entries {
        if entry.key != id {
            continue;
        }
        if entry.r#type != T::TYPE {
            bail!("crossing store key is occupied by an alien document type");
        }
        if found.is_some() {
            bail!("crossing store contains duplicate records for one key");
        }
        found = Some(rmp_serde::from_slice(&entry.payload)?);
    }
    Ok(found)
}

fn validate_request_shape(value: &PersonaDiscordDeliveryRequest) -> Result<()> {
    let issued = chrono::DateTime::parse_from_rfc3339(&value.issued_at)
        .map_err(|_| anyhow!("Persona delivery issuedAt is not RFC3339"))?;
    let expires = chrono::DateTime::parse_from_rfc3339(&value.expires_at)
        .map_err(|_| anyhow!("Persona delivery expiresAt is not RFC3339"))?;
    if value.schema_version != PERSONA_DISCORD_DELIVERY_REQUEST_SCHEMA_VERSION
        || !valid_crossing_id(&value.request_id, false)
        || !valid_crossing_id(&value.effect_document_id, false)
        || !valid_crossing_id(&value.target_runtime_id, false)
        || !valid_crossing_id(&value.persona_agent_id, false)
        || !valid_crossing_id(&value.channel_id, false)
        || !valid_crossing_id(&value.reply_to_message_id, true)
        || value.content.trim().is_empty()
        || value.content.len() > 1900
        || value.content.contains('\0')
        || value.content_sha256 != sha256_hyphen(value.content.as_bytes())
        || value.issued_at.trim().is_empty()
        || value.expires_at.trim().is_empty()
        || expires <= issued
        || expires - issued > chrono::Duration::seconds(120)
        || value.private_state_exposed
        || !valid_crossing_id(&value.signer_identity_id, false)
    {
        bail!("Persona Discord delivery request violates its fixed contract");
    }
    Ok(())
}

fn valid_crossing_id(value: &str, allow_empty: bool) -> bool {
    (allow_empty || !value.trim().is_empty())
        && value.len() <= 256
        && !value.chars().any(char::is_control)
}

fn validate_receipt_shape(value: &PersonaDiscordDeliveryReceipt) -> Result<()> {
    let evidence = [
        &value.message_id,
        &value.transport,
        &value.crossing_receipt_id,
        &value.receipt_url,
    ];
    let sha_valid = value.request_payload_sha256.len() == 71
        && value.request_payload_sha256.starts_with("sha256-")
        && value.request_payload_sha256[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
    let completed_at_valid = chrono::DateTime::parse_from_rfc3339(&value.completed_at).is_ok();
    let url_bound = value.status != "completed"
        || value
            .receipt_url
            .contains(&format!("/{}/{}", value.channel_id, value.message_id));
    if value.schema_version != PERSONA_DISCORD_DELIVERY_RECEIPT_SCHEMA_VERSION
        || value.receipt_id != value.request_id
        || value.request_id.trim().is_empty()
        || !matches!(value.status.as_str(), "completed" | "failed" | "unknown")
        || value.channel_id.trim().is_empty()
        || !completed_at_valid
        || !sha_valid
        || !url_bound
        || value.provider_identity_id.trim().is_empty()
        || value.private_state_exposed
        || value.provider_signature.len() != 64
        || (value.status == "completed" && evidence.iter().any(|field| field.is_empty()))
        || (value.status != "completed" && evidence.iter().any(|field| !field.is_empty()))
    {
        bail!("Bifrost Persona Discord delivery receipt violates its fixed contract");
    }
    Ok(())
}

fn request_signing_tuple(value: &PersonaDiscordDeliveryRequest) -> impl Serialize + '_ {
    (
        &value.schema_version,
        &value.request_id,
        &value.effect_document_id,
        &value.target_runtime_id,
        &value.persona_agent_id,
        &value.channel_id,
        &value.reply_to_message_id,
        &value.content,
        &value.content_sha256,
        &value.issued_at,
        &value.expires_at,
        value.private_state_exposed,
        &value.signer_identity_id,
    )
}
fn receipt_signing_tuple(value: &PersonaDiscordDeliveryReceipt) -> impl Serialize + '_ {
    (
        &value.schema_version,
        &value.receipt_id,
        &value.request_id,
        &value.request_payload_sha256,
        &value.status,
        &value.channel_id,
        &value.reply_to_message_id,
        &value.message_id,
        &value.transport,
        &value.crossing_receipt_id,
        &value.receipt_url,
        &value.completed_at,
        &value.provider_identity_id,
        value.private_state_exposed,
    )
}
fn sha256_hyphen(bytes: &[u8]) -> String {
    format!("sha256-{:x}", Sha256::digest(bytes))
}
