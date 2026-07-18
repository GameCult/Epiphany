use anyhow::{Result, anyhow, bail};
use cultcache_rs::{CultCache, DatabaseEntry, SingleFileMessagePackBackingStore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::{
    HostIdentitySignature, HostIdentityTrustAnchorEntry,
    verify_host_identity_trust_anchor_signature,
};

pub const BIFROST_PERSONA_FEEDBACK_ADMISSION_SCHEMA_VERSION: &str =
    "bifrost.persona_feedback.delivery.v0";
pub const BIFROST_PERSONA_FEEDBACK_DELIVERY_TYPE: &str = "bifrost.persona_feedback.delivery";
pub const LOCAL_PERSONA_FEEDBACK_SCHEMA_VERSION: &str =
    "epiphany.persona_feedback.admitted_pressure.v0";
const SIGNING_PURPOSE: &str = "bifrost.persona-feedback.delivery.v0";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaFeedbackPacket {
    pub feedback_id: String,
    pub source_event_id: String,
    pub source_actor_id: String,
    pub actor_classification: String,
    pub actor_link_refs: Vec<String>,
    pub discord_guild_id: String,
    pub discord_channel_id: String,
    pub discord_message_id: String,
    pub target_runtime_id: String,
    pub target_repository: String,
    pub target_persona_id: String,
    pub source_room_id: String,
    pub feedback_text: String,
    pub content_sha256: String,
    pub source_discussion_refs: Vec<String>,
    pub source_visibility: String,
    pub data_classification: String,
    pub private_state_included: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BifrostPersonaFeedbackAdmission {
    pub schema_version: String,
    pub admission_id: String,
    pub packet: PersonaFeedbackPacket,
    pub packet_sha256: String,
    pub source_observer_id: String,
    pub source_observer_runtime_id: String,
    pub provider: String,
    pub bifrost_admission_receipt_id: String,
    pub authority: String,
    pub provider_identity_id: String,
    pub provider_signature: Vec<u8>,
}

impl DatabaseEntry for BifrostPersonaFeedbackAdmission {
    const TYPE: &'static str = BIFROST_PERSONA_FEEDBACK_DELIVERY_TYPE;
    const SCHEMA_NAME: &'static str = "BifrostPersonaFeedbackAdmission";
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.persona_feedback.admitted_pressure.v0",
    schema = "LocalAdmittedPersonaFeedback"
)]
pub struct LocalAdmittedPersonaFeedback {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub feedback_id: String,
    #[cultcache(key = 2)]
    pub admission_id: String,
    #[cultcache(key = 3)]
    pub source_actor_id: String,
    #[cultcache(key = 4)]
    pub source_provider: String,
    #[cultcache(key = 5)]
    pub target_runtime_id: String,
    #[cultcache(key = 6)]
    pub target_repository: String,
    #[cultcache(key = 7)]
    pub target_persona_id: String,
    #[cultcache(key = 8)]
    pub source_room_id: String,
    #[cultcache(key = 9)]
    pub feedback_text: String,
    #[cultcache(key = 10)]
    pub source_discussion_refs: Vec<String>,
    #[cultcache(key = 11)]
    pub bifrost_provider_identity_id: String,
    #[cultcache(key = 12)]
    pub authority: String,
    #[cultcache(key = 13)]
    pub private_state_exposed: bool,
    #[cultcache(key = 14)]
    pub source_visibility: String,
    #[cultcache(key = 15)]
    pub data_classification: String,
    #[cultcache(key = 16)]
    pub packet_sha256: String,
    #[cultcache(key = 17)]
    pub source_observer_id: String,
    #[cultcache(key = 18)]
    pub source_observer_runtime_id: String,
}

pub fn persona_feedback_packet_sha256(packet: &PersonaFeedbackPacket) -> Result<String> {
    Ok(format!(
        "sha256-{:x}",
        Sha256::digest(rmp_serde::to_vec(packet)?)
    ))
}

pub fn persona_feedback_admission_signing_payload(
    admission: &BifrostPersonaFeedbackAdmission,
) -> Result<Vec<u8>> {
    rmp_serde::to_vec(&(
        admission.schema_version.as_str(),
        admission.admission_id.as_str(),
        &admission.packet,
        admission.packet_sha256.as_str(),
        admission.source_observer_id.as_str(),
        admission.source_observer_runtime_id.as_str(),
        admission.provider.as_str(),
        admission.bifrost_admission_receipt_id.as_str(),
        admission.authority.as_str(),
        admission.provider_identity_id.as_str(),
    ))
    .map_err(Into::into)
}

pub fn persona_feedback_admission_signing_purpose() -> &'static str {
    SIGNING_PURPOSE
}

pub fn admit_bifrost_persona_feedback(
    feedback_store: &Path,
    admission: &BifrostPersonaFeedbackAdmission,
    trusted_bifrost_identity: &HostIdentityTrustAnchorEntry,
    expected_runtime_id: &str,
    expected_repository: &str,
    expected_persona_id: &str,
) -> Result<LocalAdmittedPersonaFeedback> {
    validate_bifrost_persona_feedback_admission(
        admission,
        trusted_bifrost_identity,
        expected_runtime_id,
        expected_repository,
        expected_persona_id,
    )?;
    let packet = &admission.packet;

    let local = LocalAdmittedPersonaFeedback {
        schema_version: LOCAL_PERSONA_FEEDBACK_SCHEMA_VERSION.into(),
        feedback_id: packet.feedback_id.clone(),
        admission_id: admission.admission_id.clone(),
        source_actor_id: packet.source_actor_id.clone(),
        source_provider: admission.provider.clone(),
        target_runtime_id: packet.target_runtime_id.clone(),
        target_repository: packet.target_repository.clone(),
        target_persona_id: packet.target_persona_id.clone(),
        source_room_id: packet.source_room_id.clone(),
        feedback_text: packet.feedback_text.clone(),
        source_discussion_refs: packet.source_discussion_refs.clone(),
        bifrost_provider_identity_id: admission.provider_identity_id.clone(),
        authority: "resident-pressure-only".into(),
        private_state_exposed: false,
        source_visibility: packet.source_visibility.clone(),
        data_classification: packet.data_classification.clone(),
        packet_sha256: admission.packet_sha256.clone(),
        source_observer_id: admission.source_observer_id.clone(),
        source_observer_runtime_id: admission.source_observer_runtime_id.clone(),
    };
    let node = crate::open_epiphany_cultmesh_node(feedback_store, expected_runtime_id.to_string())?;
    let mut cache = CultCache::new();
    cache.register_entry_type::<LocalAdmittedPersonaFeedback>()?;
    let (entry, _) = cache.prepare_entry(&local.feedback_id, &local)?;
    if !SingleFileMessagePackBackingStore::new(feedback_store).insert_entry_if_absent(entry)? {
        let existing = node.get::<LocalAdmittedPersonaFeedback>(&local.feedback_id)?;
        if existing.as_ref() != Some(&local) {
            return Err(anyhow!("admitted Persona feedback identity collision"));
        }
    }
    Ok(local)
}

fn validate_bifrost_persona_feedback_admission(
    admission: &BifrostPersonaFeedbackAdmission,
    trusted_bifrost_identity: &HostIdentityTrustAnchorEntry,
    expected_runtime_id: &str,
    expected_repository: &str,
    expected_persona_id: &str,
) -> Result<()> {
    let packet = &admission.packet;
    if admission.schema_version != BIFROST_PERSONA_FEEDBACK_ADMISSION_SCHEMA_VERSION
        || admission.source_observer_id != "voidbot"
        || admission.source_observer_runtime_id.trim().is_empty()
        || admission.provider != "bifrost"
        || admission.bifrost_admission_receipt_id.trim().is_empty()
        || admission.authority != "feedback_only"
        || admission.admission_id.trim().is_empty()
        || packet.feedback_id.trim().is_empty()
        || packet.source_event_id.trim().is_empty()
        || packet.source_actor_id.trim().is_empty()
        || !matches!(
            packet.actor_classification.as_str(),
            "linked_governance_feedback" | "unlinked_social_feedback"
        )
        || packet.discord_guild_id.trim().is_empty()
        || packet.discord_channel_id.trim().is_empty()
        || packet.discord_message_id.trim().is_empty()
        || !is_sha256(&packet.content_sha256)
        || packet.feedback_text.trim().is_empty()
        || packet.source_discussion_refs.is_empty()
        || !matches!(
            (
                packet.source_visibility.as_str(),
                packet.data_classification.as_str()
            ),
            ("public", "public_feedback")
                | ("organization", "organization_feedback")
                | ("private", "private_feedback")
        )
        || packet.private_state_included
    {
        bail!("Bifrost Persona feedback admission violates its fixed public pressure contract");
    }
    if packet.target_runtime_id != expected_runtime_id
        || packet.target_repository != expected_repository
        || packet.target_persona_id != expected_persona_id
    {
        bail!("Bifrost Persona feedback targets a different runtime, repository, or Persona");
    }
    if admission.provider_identity_id != trusted_bifrost_identity.identity_id {
        bail!("Persona feedback admission is not produced by the trusted Bifrost identity");
    }
    let digest = persona_feedback_packet_sha256(packet)?;
    if admission.packet_sha256 != digest {
        bail!("Persona feedback admission payload binding is invalid");
    }
    verify_host_identity_trust_anchor_signature(
        trusted_bifrost_identity,
        SIGNING_PURPOSE,
        &persona_feedback_admission_signing_payload(admission)?,
        &HostIdentitySignature {
            identity_id: admission.provider_identity_id.clone(),
            signature: admission.provider_signature.clone(),
        },
    )?;
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.strip_prefix("sha256-").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

pub fn admitted_persona_feedback(
    path: &Path,
    runtime_id: &str,
) -> Result<Vec<LocalAdmittedPersonaFeedback>> {
    let node = crate::open_epiphany_cultmesh_node(path, runtime_id.to_string())?;
    let mut feedback = node
        .get_all_with_keys::<LocalAdmittedPersonaFeedback>()?
        .into_iter()
        .map(|(_, value)| value)
        .collect::<Vec<_>>();
    feedback.sort_by(|left, right| left.feedback_id.cmp(&right.feedback_id));
    Ok(feedback)
}

pub fn import_bifrost_persona_feedback_deliveries(
    source_store: &Path,
    feedback_store: &Path,
    trust_anchor_path: &Path,
    expected_runtime_id: &str,
    expected_repository: &str,
    expected_persona_id: &str,
) -> Result<Vec<LocalAdmittedPersonaFeedback>> {
    validate_persona_feedback_store_separation(source_store, feedback_store, &[trust_anchor_path])?;
    let (anchor, deliveries) = validated_bifrost_persona_feedback_deliveries(
        source_store,
        trust_anchor_path,
        expected_runtime_id,
        expected_repository,
        expected_persona_id,
    )?;
    deliveries
        .iter()
        .map(|delivery| {
            admit_bifrost_persona_feedback(
                feedback_store,
                delivery,
                &anchor,
                expected_runtime_id,
                expected_repository,
                expected_persona_id,
            )
        })
        .collect()
}

pub fn validate_persona_feedback_store_separation(
    source_store: &Path,
    feedback_store: &Path,
    protected_stores: &[&Path],
) -> Result<()> {
    if paths_share_storage(source_store, feedback_store)?
        || protected_stores
            .iter()
            .any(|path| paths_share_storage(source_store, path).unwrap_or(true))
    {
        bail!(
            "Bifrost feedback source must be physically separate from local Verse, resident, runtime, Mind, release, and trust stores"
        );
    }
    Ok(())
}

fn paths_share_storage(left: &Path, right: &Path) -> Result<bool> {
    let canonical = |path: &Path| -> Result<std::path::PathBuf> {
        if path.exists() {
            return Ok(path.canonicalize()?);
        }
        let parent = path
            .parent()
            .ok_or_else(|| anyhow!("store path has no parent"))?;
        Ok(parent.canonicalize()?.join(
            path.file_name()
                .ok_or_else(|| anyhow!("store path has no file name"))?,
        ))
    };
    Ok(canonical(left)? == canonical(right)? || crate::same_existing_file(left, right)?)
}

pub fn validate_bifrost_persona_feedback_source(
    source_store: &Path,
    trust_anchor_path: &Path,
    expected_runtime_id: &str,
    expected_repository: &str,
    expected_persona_id: &str,
) -> Result<usize> {
    Ok(validated_bifrost_persona_feedback_deliveries(
        source_store,
        trust_anchor_path,
        expected_runtime_id,
        expected_repository,
        expected_persona_id,
    )?
    .1
    .len())
}

fn validated_bifrost_persona_feedback_deliveries(
    source_store: &Path,
    trust_anchor_path: &Path,
    expected_runtime_id: &str,
    expected_repository: &str,
    expected_persona_id: &str,
) -> Result<(
    HostIdentityTrustAnchorEntry,
    Vec<BifrostPersonaFeedbackAdmission>,
)> {
    let anchor_bytes = std::fs::read(trust_anchor_path)?;
    let anchor: HostIdentityTrustAnchorEntry = rmp_serde::from_slice(&anchor_bytes)
        .map_err(|error| anyhow!("Bifrost Persona feedback trust anchor is malformed: {error}"))?;
    // Bifrost owns this store and replaces its complete snapshot atomically.
    // Epiphany is a read-only consumer; taking CultCache's ordinary shared
    // lock would require write authority merely to create/open the sibling
    // lock file. Decode the provider snapshot without acquiring ownership of
    // its directory, and reject any foreign document family.
    let envelopes =
        SingleFileMessagePackBackingStore::new(source_store).pull_all_read_only_snapshot()?;
    let mut deliveries = envelopes
        .into_iter()
        .map(|entry| {
            if entry.r#type != <BifrostPersonaFeedbackAdmission as DatabaseEntry>::TYPE {
                bail!(
                    "Bifrost Persona feedback source contains foreign document type {:?}",
                    entry.r#type
                );
            }
            rmp_serde::from_slice::<BifrostPersonaFeedbackAdmission>(&entry.payload)
                .map_err(Into::into)
        })
        .collect::<Result<Vec<_>>>()?;
    deliveries.sort_by(|left, right| left.admission_id.cmp(&right.admission_id));
    let mut identities = std::collections::BTreeMap::new();
    for delivery in &deliveries {
        validate_bifrost_persona_feedback_admission(
            delivery,
            &anchor,
            &delivery.packet.target_runtime_id,
            &delivery.packet.target_repository,
            &delivery.packet.target_persona_id,
        )?;
        let identity = (
            delivery.packet_sha256.clone(),
            delivery.admission_id.clone(),
        );
        if identities
            .insert(delivery.packet.feedback_id.clone(), identity.clone())
            .is_some_and(|existing| existing != identity)
        {
            bail!("Bifrost Persona feedback source contains a same-id collision");
        }
    }
    deliveries.retain(|delivery| {
        delivery.packet.target_runtime_id == expected_runtime_id
            && delivery.packet.target_repository == expected_repository
            && delivery.packet.target_persona_id == expected_persona_id
    });
    Ok((anchor, deliveries))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signed_delivery(
        signer: &crate::HostIdentitySigner,
        target_persona: &str,
    ) -> Result<BifrostPersonaFeedbackAdmission> {
        let packet = PersonaFeedbackPacket {
            feedback_id: "feedback-1".into(),
            source_event_id: "discord-event-1".into(),
            source_actor_id: "discord-user-7".into(),
            actor_classification: "unlinked_social_feedback".into(),
            actor_link_refs: Vec::new(),
            discord_guild_id: "guild-1".into(),
            discord_channel_id: "channel-1".into(),
            discord_message_id: "message-1".into(),
            target_runtime_id: "epiphany-yggdrasil".into(),
            target_repository: "GameCult/Epiphany".into(),
            target_persona_id: target_persona.into(),
            source_room_id: "discord://guild-1/channel-1".into(),
            feedback_text: "Please make the map legible.".into(),
            content_sha256:
                "sha256-1d0e810c0f13bb53b93f8d2643f2e5c74fcfe666b6988ddfc85758942f68d3aa".into(),
            source_discussion_refs: vec!["discord://guild-1/channel-1/message-1".into()],
            source_visibility: "public".into(),
            data_classification: "public_feedback".into(),
            private_state_included: false,
        };
        let mut admission = BifrostPersonaFeedbackAdmission {
            schema_version: BIFROST_PERSONA_FEEDBACK_ADMISSION_SCHEMA_VERSION.into(),
            admission_id: "bifrost-admission-1".into(),
            packet_sha256: persona_feedback_packet_sha256(&packet)?,
            packet,
            source_observer_id: "voidbot".into(),
            source_observer_runtime_id: "voidbot-yggdrasil".into(),
            provider: "bifrost".into(),
            bifrost_admission_receipt_id: "bifrost-receipt-1".into(),
            authority: "feedback_only".into(),
            provider_identity_id: signer.entry().identity_id.clone(),
            provider_signature: Vec::new(),
        };
        admission.provider_signature = signer
            .sign(
                SIGNING_PURPOSE,
                &persona_feedback_admission_signing_payload(&admission)?,
            )?
            .signature;
        Ok(admission)
    }

    #[test]
    fn admits_exact_authenticated_delivery_as_pressure_only() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let signer = crate::enroll_host_identity_at(&temp.path().join("bifrost.cc"))?;
        let anchor = crate::export_host_identity_trust_anchor(
            &signer,
            &temp.path().join("bifrost-anchor.cc"),
        )?;
        let delivery = signed_delivery(&signer, "epiphany")?;
        let local = admit_bifrost_persona_feedback(
            &temp.path().join("local.cc"),
            &delivery,
            &anchor,
            "epiphany-yggdrasil",
            "GameCult/Epiphany",
            "epiphany",
        )?;
        assert_eq!(local.source_actor_id, "discord-user-7");
        assert_eq!(local.target_persona_id, "epiphany");
        assert_eq!(local.authority, "resident-pressure-only");
        assert_eq!(
            admitted_persona_feedback(&temp.path().join("local.cc"), "epiphany-yggdrasil")?,
            vec![local]
        );
        Ok(())
    }

    #[test]
    fn refuses_target_payload_and_provider_substitution() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let signer = crate::enroll_host_identity_at(&temp.path().join("bifrost.cc"))?;
        let other = crate::enroll_host_identity_at(&temp.path().join("caller.cc"))?;
        let anchor = crate::export_host_identity_trust_anchor(
            &signer,
            &temp.path().join("bifrost-anchor.cc"),
        )?;
        let other_anchor = crate::export_host_identity_trust_anchor(
            &other,
            &temp.path().join("caller-anchor.cc"),
        )?;
        let delivery = signed_delivery(&signer, "epiphany")?;
        assert!(
            admit_bifrost_persona_feedback(
                &temp.path().join("local.cc"),
                &delivery,
                &anchor,
                "epiphany-yggdrasil",
                "GameCult/Epiphany",
                "Other.Persona",
            )
            .is_err()
        );
        assert!(
            admit_bifrost_persona_feedback(
                &temp.path().join("local.cc"),
                &delivery,
                &other_anchor,
                "epiphany-yggdrasil",
                "GameCult/Epiphany",
                "epiphany",
            )
            .is_err()
        );
        let mut tampered = delivery;
        tampered.packet.feedback_text = "caller-authored substitution".into();
        assert!(
            admit_bifrost_persona_feedback(
                &temp.path().join("local.cc"),
                &tampered,
                &anchor,
                "epiphany-yggdrasil",
                "GameCult/Epiphany",
                "epiphany",
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn imports_signed_provider_store_into_dedicated_feedback_store() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let source_store = temp.path().join("bifrost-delivery.cc");
        let feedback_store = temp.path().join("persona-feedback.cc");
        let anchor_path = temp.path().join("bifrost-anchor.msgpack");
        let signer = crate::enroll_host_identity_at(&temp.path().join("bifrost.cc"))?;
        let anchor = crate::export_host_identity_trust_anchor(
            &signer,
            &temp.path().join("unused-anchor-store.cc"),
        )?;
        std::fs::write(&anchor_path, rmp_serde::to_vec(&anchor)?)?;
        let delivery = signed_delivery(&signer, "epiphany")?;
        let mut source = CultCache::new();
        source.register_entry_type::<BifrostPersonaFeedbackAdmission>()?;
        source.add_generic_backing_store(SingleFileMessagePackBackingStore::new(&source_store));
        source.put(&delivery.admission_id, &delivery)?;
        let source_lock = source_store.with_file_name(format!(
            "{}.lock",
            source_store.file_name().unwrap().to_string_lossy()
        ));
        if source_lock.exists() {
            std::fs::remove_file(&source_lock)?;
        }

        let mut local =
            crate::open_epiphany_cultmesh_node(&feedback_store, "epiphany-yggdrasil".to_string())?;
        local.put(
            "status",
            &crate::EpiphanyCultMeshStatusEntry {
                schema_version: crate::EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION.into(),
                runtime_id: "epiphany-yggdrasil".into(),
                verse_id: "gamecult-local".into(),
                app_id: "epiphany".into(),
                note: "existing local Verse state".into(),
                verse_tier: "local".into(),
            },
        )?;

        let imported = import_bifrost_persona_feedback_deliveries(
            &source_store,
            &feedback_store,
            &anchor_path,
            "epiphany-yggdrasil",
            "GameCult/Epiphany",
            "epiphany",
        )?;
        assert_eq!(imported.len(), 1);
        assert!(
            !source_lock.exists(),
            "read-only provider snapshot must not create a sibling lock"
        );
        assert_eq!(
            admitted_persona_feedback(&feedback_store, "epiphany-yggdrasil")?.len(),
            1
        );
        assert!(
            crate::open_epiphany_cultmesh_node(&feedback_store, "epiphany-yggdrasil".to_string())?
                .get::<crate::EpiphanyCultMeshStatusEntry>("status")?
                .is_some()
        );
        Ok(())
    }
}
