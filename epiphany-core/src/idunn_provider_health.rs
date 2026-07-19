use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use cultcache_rs::{
    CultCache, CultCacheEnvelope, DatabaseEntry, SingleFileMessagePackBackingStore,
};
use cultnet_rs::{
    CultNetRawDocumentRecord, CultNetRawPayloadEncoding, CultNetRawSnapshotQuery,
    CultNetSnapshotRecordExpectation, CultNetSnapshotSourceExpectation,
    GameCultServiceTrustAnchorRecord, IDUNN_AUTHENTICATED_PROVIDER_HEALTH_PROJECTION_SCHEMA,
    IDUNN_AUTHENTICATED_PROVIDER_HEALTH_PROJECTION_SIGNING_PURPOSE,
    IdunnAuthenticatedProviderHealthProjectionPurpose,
    IdunnAuthenticatedProviderHealthProjectionRecord, IdunnServiceIdentity,
    ServiceIdentitySignature, verify_service_identity_signature_with_public_key,
};
use sha2::{Digest, Sha256};

pub const EPIPHANY_IDUNN_PROVIDER_HEALTH_ADMISSION_SCHEMA: &str =
    "epiphany.idunn_provider_health_admission.v1";
pub const EPIPHANY_IDUNN_PROVIDER_HEALTH_ADMISSION_TYPE: &str =
    "epiphany.idunn_provider_health_admission";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderReleaseBinding {
    Forbidden,
    Exact {
        release_id: String,
        release_witness_sha256: String,
        source_commit: String,
        deployment_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequiredProviderHealth {
    pub daemon_id: String,
    pub health_contract: String,
    pub release_binding: ProviderReleaseBinding,
}

impl RequiredProviderHealth {
    pub fn record_key(&self) -> String {
        provider_health_record_key(&self.daemon_id, &self.health_contract)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.idunn_provider_health_admission",
    schema = "epiphany.idunn_provider_health_admission.v1"
)]
pub struct IdunnProviderHealthAdmission {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub admission_id: String,
    #[cultcache(key = 2)]
    pub daemon_id: String,
    #[cultcache(key = 3)]
    pub health_contract: String,
    #[cultcache(key = 4)]
    pub provider_state: String,
    #[cultcache(key = 5)]
    pub reason_code: String,
    #[cultcache(key = 6)]
    pub projection_id: String,
    #[cultcache(key = 7)]
    pub projection_sha256: String,
    #[cultcache(key = 8)]
    pub projection_incarnation_id: String,
    #[cultcache(key = 9)]
    pub projection_sequence: u64,
    #[cultcache(key = 10)]
    pub evaluated_at_unix_millis: u64,
    #[cultcache(key = 11)]
    pub expires_at_unix_millis: u64,
    #[cultcache(key = 12)]
    pub provider_incarnation_id: String,
    #[cultcache(key = 13)]
    pub provider_sequence: u64,
    #[cultcache(key = 14)]
    pub provider_observed_at_unix_millis: u64,
    #[cultcache(key = 15)]
    pub signed_health_sha256: String,
    #[cultcache(key = 16)]
    pub idunn_runtime_id: String,
    #[cultcache(key = 17)]
    pub idunn_signer_identity_id: String,
    #[cultcache(key = 18)]
    pub trust_anchor_id: String,
    #[cultcache(key = 19)]
    pub trust_anchor_sha256: String,
    #[cultcache(key = 20)]
    pub release_id: Option<String>,
    #[cultcache(key = 21)]
    pub release_witness_sha256: Option<String>,
    #[cultcache(key = 22)]
    pub source_commit: Option<String>,
    #[cultcache(key = 23)]
    pub deployment_id: Option<String>,
    #[cultcache(key = 24)]
    pub admitted_at_unix_millis: u64,
    #[cultcache(key = 25)]
    pub private_state_exposed: bool,
    #[cultcache(key = 26)]
    pub trust_anchor_bound_at_unix_millis: u64,
}

pub fn provider_health_record_key(daemon_id: &str, health_contract: &str) -> String {
    format!(
        "provider-health:{:x}",
        Sha256::digest([daemon_id.as_bytes(), b"\0", health_contract.as_bytes()].concat())
    )
}

/// Build the exact read-only request expected from Idunn's public store. The
/// returned query rejects extra identities and source-runtime substitution;
/// admission below additionally refuses any missing required identity.
pub fn required_idunn_provider_health_query(
    message_id: impl Into<String>,
    required: &[RequiredProviderHealth],
    expected_idunn_runtime_id: &str,
) -> Result<CultNetRawSnapshotQuery> {
    validate_required_set(required)?;
    if expected_idunn_runtime_id.trim().is_empty() {
        bail!("expected Idunn runtime id is empty");
    }
    CultNetRawSnapshotQuery::new(
        message_id,
        required
            .iter()
            .map(|requirement| CultNetSnapshotRecordExpectation {
                schema_id: IDUNN_AUTHENTICATED_PROVIDER_HEALTH_PROJECTION_SCHEMA.into(),
                record_key: requirement.record_key(),
                source: CultNetSnapshotSourceExpectation {
                    runtime_id: Some(expected_idunn_runtime_id.into()),
                    agent_id: None,
                    role: None,
                    tags: None,
                },
            })
            .collect(),
    )
}

/// Verify a complete required snapshot, then atomically advance its durable
/// local admissions. Invalid, missing, replayed, or substituted input performs
/// no write and therefore cannot erase the last admitted truth.
pub fn admit_required_idunn_provider_health(
    admission_store: &Path,
    required: &[RequiredProviderHealth],
    records: &[CultNetRawDocumentRecord],
    trust_anchor: &GameCultServiceTrustAnchorRecord,
    expected_idunn_runtime_id: &str,
    now_unix_millis: u64,
    max_local_age_millis: u64,
) -> Result<Vec<IdunnProviderHealthAdmission>> {
    validate_required_set(required)?;
    validate_anchor(trust_anchor, expected_idunn_runtime_id, now_unix_millis)?;
    if max_local_age_millis == 0 {
        bail!("local provider-health TTL is zero");
    }

    let requirements = required
        .iter()
        .map(|item| (item.record_key(), item))
        .collect::<BTreeMap<_, _>>();
    let mut incoming = BTreeMap::new();
    for record in records {
        let requirement = requirements
            .get(&record.record_key)
            .ok_or_else(|| anyhow!("Idunn snapshot contains an unknown provider-health record"))?;
        if incoming.contains_key(&record.record_key) {
            bail!("Idunn snapshot contains a duplicate provider-health record");
        }
        let admission = verify_projection(
            record,
            requirement,
            trust_anchor,
            expected_idunn_runtime_id,
            now_unix_millis,
            max_local_age_millis,
        )?;
        incoming.insert(record.record_key.clone(), admission);
    }
    if incoming.len() != requirements.len() {
        let missing = requirements
            .keys()
            .filter(|key| !incoming.contains_key(*key))
            .cloned()
            .collect::<Vec<_>>();
        bail!("Idunn snapshot is missing required provider-health records: {missing:?}");
    }

    let backing = SingleFileMessagePackBackingStore::new(admission_store);
    for _ in 0..8 {
        let snapshot = backing.pull_all_read_only_snapshot()?;
        let existing = load_existing_admissions(&snapshot)?;
        let mut expected = Vec::new();
        let mut replacements = Vec::new();
        let mut final_values = BTreeMap::new();
        for requirement in required {
            let key = requirement.record_key();
            let next = incoming
                .get(&key)
                .ok_or_else(|| anyhow!("verified provider-health set lost a required record"))?;
            match existing.get(&key) {
                None => replacements.push(admission_envelope(next)?),
                Some((envelope, current))
                    if current.projection_sha256 == next.projection_sha256 =>
                {
                    expected.push(envelope.clone());
                    replacements.push(envelope.clone());
                    final_values.insert(key, current.clone());
                    continue;
                }
                Some((envelope, current)) => {
                    validate_monotonic_successor(current, next)?;
                    expected.push(envelope.clone());
                    replacements.push(admission_envelope(next)?);
                }
            }
            final_values.insert(key, next.clone());
        }
        if backing.compare_and_swap_batch(&expected, replacements)? {
            return order_admission_map(required, &final_values);
        }
    }
    bail!("Idunn provider-health admission lost repeated cross-process contention")
}

fn verify_projection(
    record: &CultNetRawDocumentRecord,
    requirement: &RequiredProviderHealth,
    anchor: &GameCultServiceTrustAnchorRecord,
    expected_idunn_runtime_id: &str,
    now: u64,
    max_local_age: u64,
) -> Result<IdunnProviderHealthAdmission> {
    if record.schema_id != IDUNN_AUTHENTICATED_PROVIDER_HEALTH_PROJECTION_SCHEMA
        || record.payload_encoding != CultNetRawPayloadEncoding::Messagepack
        || record.record_key != requirement.record_key()
        || record.source_runtime_id.as_deref() != Some(expected_idunn_runtime_id)
    {
        bail!("Idunn provider-health record schema, key, encoding, or source runtime is invalid");
    }
    let projection: IdunnAuthenticatedProviderHealthProjectionRecord =
        rmp_serde::from_slice(&record.payload)
            .context("Idunn provider-health payload is malformed MessagePack")?;
    if rmp_serde::to_vec(&projection)? != record.payload {
        bail!("Idunn provider-health payload is not canonical MessagePack");
    }
    projection.validate()?;
    if projection.projection_id != record.record_key
        || projection.daemon_id != requirement.daemon_id
        || projection.health_contract != requirement.health_contract
        || projection.idunn_runtime_id != expected_idunn_runtime_id
        || projection.idunn_signer_identity_id != anchor.signer_identity_id
        || projection.private_state_exposed
    {
        bail!(
            "Idunn provider-health projection identity, pair, runtime, signer, or privacy is invalid"
        );
    }
    if projection.evaluated_at_unix_millis > now
        || projection.expires_at_unix_millis <= now
        || now.saturating_sub(projection.evaluated_at_unix_millis) > max_local_age
        || anchor.bound_at_unix_millis > projection.evaluated_at_unix_millis
        || anchor
            .expires_at_unix_millis
            .is_some_and(|expires| expires <= projection.evaluated_at_unix_millis)
        || anchor
            .expires_at_unix_millis
            .is_some_and(|expires| projection.expires_at_unix_millis > expires)
    {
        bail!(
            "Idunn provider-health projection is future, expired, outside local TTL, or outside anchor lifetime"
        );
    }
    validate_release_binding(&projection, &requirement.release_binding)?;

    let proof = ServiceIdentitySignature {
        identity_id: projection.idunn_signer_identity_id.clone(),
        signature: projection.signature.clone(),
    };
    let mut unsigned = projection.clone();
    unsigned.signature.clear();
    verify_service_identity_signature_with_public_key::<
        IdunnServiceIdentity,
        IdunnAuthenticatedProviderHealthProjectionPurpose,
    >(
        &anchor.signer_public_key,
        &rmp_serde::to_vec(&unsigned)?,
        &proof,
    )
    .context("Idunn provider-health signature is invalid")?;

    Ok(IdunnProviderHealthAdmission {
        schema_version: EPIPHANY_IDUNN_PROVIDER_HEALTH_ADMISSION_SCHEMA.into(),
        admission_id: requirement.record_key(),
        daemon_id: projection.daemon_id,
        health_contract: projection.health_contract,
        provider_state: projection.provider_state,
        reason_code: projection.reason_code,
        projection_id: projection.projection_id,
        projection_sha256: sha256(&record.payload),
        projection_incarnation_id: projection.projection_incarnation_id,
        projection_sequence: projection.projection_sequence,
        evaluated_at_unix_millis: projection.evaluated_at_unix_millis,
        expires_at_unix_millis: projection.expires_at_unix_millis,
        provider_incarnation_id: projection.provider_incarnation_id,
        provider_sequence: projection.provider_sequence,
        provider_observed_at_unix_millis: projection.provider_observed_at_unix_millis,
        signed_health_sha256: projection.signed_health_sha256,
        idunn_runtime_id: projection.idunn_runtime_id,
        idunn_signer_identity_id: projection.idunn_signer_identity_id,
        trust_anchor_id: anchor.trust_anchor_id.clone(),
        trust_anchor_sha256: sha256(&rmp_serde::to_vec(anchor)?),
        release_id: projection.release_id,
        release_witness_sha256: projection.release_witness_sha256,
        source_commit: projection.source_commit,
        deployment_id: projection.deployment_id,
        admitted_at_unix_millis: now,
        private_state_exposed: false,
        trust_anchor_bound_at_unix_millis: anchor.bound_at_unix_millis,
    })
}

fn validate_anchor(
    anchor: &GameCultServiceTrustAnchorRecord,
    expected_runtime: &str,
    now: u64,
) -> Result<()> {
    anchor.validate()?;
    if expected_runtime.trim().is_empty()
        || anchor.runtime_id != expected_runtime
        || anchor.service_id != "idunn"
        || anchor.signed_schema != IDUNN_AUTHENTICATED_PROVIDER_HEALTH_PROJECTION_SCHEMA
        || anchor.signing_purpose != IDUNN_AUTHENTICATED_PROVIDER_HEALTH_PROJECTION_SIGNING_PURPOSE
        || anchor.bound_at_unix_millis > now
        || anchor
            .expires_at_unix_millis
            .is_some_and(|expires| expires <= now)
        || anchor.private_state_exposed
    {
        bail!("root-pinned Idunn trust anchor profile, runtime, lifetime, or privacy is invalid");
    }
    Ok(())
}

fn validate_release_binding(
    projection: &IdunnAuthenticatedProviderHealthProjectionRecord,
    policy: &ProviderReleaseBinding,
) -> Result<()> {
    match policy {
        ProviderReleaseBinding::Forbidden => {
            if projection.release_id.is_some()
                || projection.release_witness_sha256.is_some()
                || projection.source_commit.is_some()
                || projection.deployment_id.is_some()
            {
                bail!("provider-health projection carries forbidden release authority");
            }
        }
        ProviderReleaseBinding::Exact {
            release_id,
            release_witness_sha256,
            source_commit,
            deployment_id,
        } => {
            if projection.release_id.as_ref() != Some(release_id)
                || projection.release_witness_sha256.as_ref() != Some(release_witness_sha256)
                || projection.source_commit.as_ref() != Some(source_commit)
                || projection.deployment_id.as_ref() != Some(deployment_id)
            {
                bail!(
                    "provider-health projection release lineage is not the exact required release"
                );
            }
        }
    }
    Ok(())
}

fn validate_monotonic_successor(
    current: &IdunnProviderHealthAdmission,
    next: &IdunnProviderHealthAdmission,
) -> Result<()> {
    if current.daemon_id != next.daemon_id
        || current.health_contract != next.health_contract
        || current.idunn_runtime_id != next.idunn_runtime_id
    {
        bail!("provider-health successor changes admitted pair or Idunn runtime");
    }
    let same_anchor = current.idunn_signer_identity_id == next.idunn_signer_identity_id
        && current.trust_anchor_id == next.trust_anchor_id
        && current.trust_anchor_sha256 == next.trust_anchor_sha256
        && current.trust_anchor_bound_at_unix_millis == next.trust_anchor_bound_at_unix_millis;
    if !same_anchor {
        if next.trust_anchor_bound_at_unix_millis <= current.trust_anchor_bound_at_unix_millis
            || next.trust_anchor_bound_at_unix_millis <= current.admitted_at_unix_millis
            || next.trust_anchor_bound_at_unix_millis <= current.evaluated_at_unix_millis
            || next.evaluated_at_unix_millis < next.trust_anchor_bound_at_unix_millis
        {
            bail!("root Idunn trust-anchor rotation is stale or precedes its signed projection");
        }
        return Ok(());
    }
    if current.projection_incarnation_id == next.projection_incarnation_id {
        if next.projection_sequence <= current.projection_sequence
            || next.evaluated_at_unix_millis < current.evaluated_at_unix_millis
        {
            bail!("provider-health projection sequence or evaluation time replayed");
        }
    } else if next.evaluated_at_unix_millis <= current.evaluated_at_unix_millis {
        bail!("new Idunn projection incarnation does not advance evaluation time");
    }
    if current.provider_incarnation_id == next.provider_incarnation_id {
        if next.provider_sequence < current.provider_sequence
            || next.provider_observed_at_unix_millis < current.provider_observed_at_unix_millis
            || (next.provider_sequence == current.provider_sequence
                && (next.provider_observed_at_unix_millis
                    != current.provider_observed_at_unix_millis
                    || next.signed_health_sha256 != current.signed_health_sha256))
        {
            bail!("provider statement sequence, observation, or digest replayed");
        }
    } else if next.provider_observed_at_unix_millis <= current.provider_observed_at_unix_millis {
        bail!("new provider incarnation does not advance observation time");
    }
    Ok(())
}

fn validate_required_set(required: &[RequiredProviderHealth]) -> Result<()> {
    if required.is_empty() {
        bail!("required provider-health set is empty");
    }
    let mut pairs = BTreeSet::new();
    let mut keys = BTreeSet::new();
    for item in required {
        if item.daemon_id.trim().is_empty() || item.health_contract.trim().is_empty() {
            bail!("required provider-health pair contains an empty identity");
        }
        if !pairs.insert((item.daemon_id.clone(), item.health_contract.clone()))
            || !keys.insert(item.record_key())
        {
            bail!("required provider-health set contains a duplicate pair");
        }
    }
    Ok(())
}

fn load_existing_admissions(
    entries: &[CultCacheEnvelope],
) -> Result<BTreeMap<String, (CultCacheEnvelope, IdunnProviderHealthAdmission)>> {
    let mut output = BTreeMap::new();
    for entry in entries {
        if entry.r#type != EPIPHANY_IDUNN_PROVIDER_HEALTH_ADMISSION_TYPE
            || entry.schema_id.as_deref() != Some(EPIPHANY_IDUNN_PROVIDER_HEALTH_ADMISSION_TYPE)
        {
            bail!(
                "Idunn provider-health admission store contains foreign state type={:?} schema={:?}",
                entry.r#type,
                entry.schema_id
            );
        }
        let admission: IdunnProviderHealthAdmission = rmp_serde::from_slice(&entry.payload)?;
        if rmp_serde::to_vec(&admission)? != entry.payload
            || entry.key != admission.admission_id
            || admission.schema_version != EPIPHANY_IDUNN_PROVIDER_HEALTH_ADMISSION_SCHEMA
            || admission.private_state_exposed
            || admission.trust_anchor_bound_at_unix_millis == 0
        {
            bail!("stored Idunn provider-health admission is malformed or noncanonical");
        }
        if output
            .insert(entry.key.clone(), (entry.clone(), admission))
            .is_some()
        {
            bail!("Idunn provider-health admission store contains duplicate identity");
        }
    }
    Ok(output)
}

fn admission_envelope(admission: &IdunnProviderHealthAdmission) -> Result<CultCacheEnvelope> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<IdunnProviderHealthAdmission>()?;
    Ok(cache.prepare_entry(&admission.admission_id, admission)?.0)
}

fn order_admission_map(
    required: &[RequiredProviderHealth],
    values: &BTreeMap<String, IdunnProviderHealthAdmission>,
) -> Result<Vec<IdunnProviderHealthAdmission>> {
    required
        .iter()
        .map(|item| {
            values
                .get(&item.record_key())
                .cloned()
                .ok_or_else(|| anyhow!("admitted set lost required provider-health record"))
        })
        .collect()
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256-{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cultnet_rs::{
        IdunnAuthenticatedProviderHealthProjectionPurpose, ServiceIdentitySigner,
        enroll_service_identity_at,
    };

    const NOW: u64 = 1_800_000_000_000;
    const RUNTIME: &str = "idunn-yggdrasil";

    fn required(daemon: &str) -> RequiredProviderHealth {
        RequiredProviderHealth {
            daemon_id: daemon.into(),
            health_contract: "idunn.signed_daemon_health.v1".into(),
            release_binding: ProviderReleaseBinding::Forbidden,
        }
    }

    fn anchor(
        signer: &ServiceIdentitySigner<IdunnServiceIdentity>,
    ) -> GameCultServiceTrustAnchorRecord {
        GameCultServiceTrustAnchorRecord {
            schema_version: cultnet_rs::GAMECULT_SERVICE_TRUST_ANCHOR_SCHEMA.into(),
            trust_anchor_id: "root/idunn/provider-health".into(),
            service_id: "idunn".into(),
            runtime_id: RUNTIME.into(),
            signer_identity_id: signer.entry().identity_id.clone(),
            signer_public_key: signer.entry().public_key.clone(),
            signature_algorithm: "ed25519".into(),
            signing_purpose: IDUNN_AUTHENTICATED_PROVIDER_HEALTH_PROJECTION_SIGNING_PURPOSE.into(),
            signed_schema: IDUNN_AUTHENTICATED_PROVIDER_HEALTH_PROJECTION_SCHEMA.into(),
            binding_authority: "root".into(),
            bound_at_unix_millis: NOW - 10_000,
            expires_at_unix_millis: Some(NOW + 100_000),
            private_state_exposed: false,
        }
    }

    fn projection(
        signer: &ServiceIdentitySigner<IdunnServiceIdentity>,
        requirement: &RequiredProviderHealth,
        projection_sequence: u64,
        provider_sequence: u64,
    ) -> IdunnAuthenticatedProviderHealthProjectionRecord {
        let mut value = IdunnAuthenticatedProviderHealthProjectionRecord {
            schema_version: IDUNN_AUTHENTICATED_PROVIDER_HEALTH_PROJECTION_SCHEMA.into(),
            projection_id: requirement.record_key(),
            daemon_id: requirement.daemon_id.clone(),
            health_contract: requirement.health_contract.clone(),
            provider_state: "active".into(),
            reason_code: cultnet_rs::IDUNN_PROVIDER_ACTIVE_REASON.into(),
            provider_observed_at_unix_millis: NOW - 3_000 + provider_sequence,
            admitted_at_unix_millis: NOW - 2_000 + provider_sequence,
            evaluated_at_unix_millis: NOW - 1_000 + projection_sequence,
            trust_binding_id: "root/provider".into(),
            trust_binding_sha256: format!("sha256-{}", "a".repeat(64)),
            signed_health_sha256: format!("sha256-{:064x}", provider_sequence),
            authenticated_admission_sha256: format!("sha256-{}", "c".repeat(64)),
            provider_signer_identity_id: "provider-key".into(),
            provider_incarnation_id: "provider/boot/1".into(),
            provider_sequence,
            release_id: None,
            release_witness_sha256: None,
            source_commit: None,
            deployment_id: None,
            idunn_runtime_id: RUNTIME.into(),
            idunn_signer_identity_id: signer.entry().identity_id.clone(),
            projection_incarnation_id: "idunn/boot/1".into(),
            projection_sequence,
            signature_algorithm: "ed25519".into(),
            signature: Vec::new(),
            private_state_exposed: false,
            expires_at_unix_millis: NOW + 60_000,
        };
        resign(signer, &mut value);
        value
    }

    fn resign(
        signer: &ServiceIdentitySigner<IdunnServiceIdentity>,
        value: &mut IdunnAuthenticatedProviderHealthProjectionRecord,
    ) {
        value.signature.clear();
        value.signature = signer
            .sign::<IdunnAuthenticatedProviderHealthProjectionPurpose>(
                &rmp_serde::to_vec(value).unwrap(),
            )
            .signature;
    }

    fn record(
        projection: &IdunnAuthenticatedProviderHealthProjectionRecord,
    ) -> CultNetRawDocumentRecord {
        CultNetRawDocumentRecord {
            schema_id: IDUNN_AUTHENTICATED_PROVIDER_HEALTH_PROJECTION_SCHEMA.into(),
            record_key: projection.projection_id.clone(),
            stored_at: "2027-01-15T08:00:00Z".into(),
            payload_encoding: CultNetRawPayloadEncoding::Messagepack,
            payload: rmp_serde::to_vec(projection).unwrap(),
            source_runtime_id: Some(RUNTIME.into()),
            source_agent_id: Some("idunn".into()),
            source_role: Some("provider-health-projection".into()),
            tags: Some(vec!["public-health".into()]),
        }
    }

    #[test]
    fn required_query_and_admission_accept_exact_complete_set() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let signer =
            enroll_service_identity_at::<IdunnServiceIdentity>(&temp.path().join("id.cc"))?;
        let requirements = vec![required("epiphany"), required("bifrost")];
        let query = required_idunn_provider_health_query("status-1", &requirements, RUNTIME)?;
        let records = requirements
            .iter()
            .map(|item| record(&projection(&signer, item, 1, 1)))
            .collect::<Vec<_>>();
        let accepted = query.accept_response(cultnet_rs::CultNetMessage::SnapshotResponseRaw {
            message_id: "status-1".into(),
            documents: records,
        })?;
        let admitted = admit_required_idunn_provider_health(
            &temp.path().join("admission.cc"),
            &requirements,
            &accepted,
            &anchor(&signer),
            RUNTIME,
            NOW,
            10_000,
        )?;
        assert_eq!(admitted.len(), 2);
        assert!(admitted.iter().all(|item| item.provider_state == "active"));
        Ok(())
    }

    #[test]
    fn hostile_projection_substitutions_cannot_erase_prior_admission() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let signer =
            enroll_service_identity_at::<IdunnServiceIdentity>(&temp.path().join("id.cc"))?;
        let requirement = required("epiphany");
        let store = temp.path().join("admission.cc");
        let valid = record(&projection(&signer, &requirement, 1, 1));
        admit_required_idunn_provider_health(
            &store,
            std::slice::from_ref(&requirement),
            std::slice::from_ref(&valid),
            &anchor(&signer),
            RUNTIME,
            NOW,
            10_000,
        )?;
        let baseline = std::fs::read(&store)?;

        let mut attacks = Vec::new();
        let mut forged = valid.clone();
        forged.source_runtime_id = Some("mallory".into());
        attacks.push(forged);
        let mut forged = valid.clone();
        forged.payload.push(0);
        attacks.push(forged);
        let mut unsigned: IdunnAuthenticatedProviderHealthProjectionRecord =
            rmp_serde::from_slice(&valid.payload)?;
        unsigned.signature[0] ^= 1;
        attacks.push(record(&unsigned));
        let mut private: IdunnAuthenticatedProviderHealthProjectionRecord =
            rmp_serde::from_slice(&valid.payload)?;
        private.private_state_exposed = true;
        attacks.push(record(&private));
        let mut wrong_reason: IdunnAuthenticatedProviderHealthProjectionRecord =
            rmp_serde::from_slice(&valid.payload)?;
        wrong_reason.reason_code = "release_drift".into();
        resign(&signer, &mut wrong_reason);
        attacks.push(record(&wrong_reason));
        let mut expired: IdunnAuthenticatedProviderHealthProjectionRecord =
            rmp_serde::from_slice(&valid.payload)?;
        expired.expires_at_unix_millis = NOW;
        resign(&signer, &mut expired);
        attacks.push(record(&expired));
        let mut future: IdunnAuthenticatedProviderHealthProjectionRecord =
            rmp_serde::from_slice(&valid.payload)?;
        future.evaluated_at_unix_millis = NOW + 1;
        future.expires_at_unix_millis = NOW + 60_000;
        resign(&signer, &mut future);
        attacks.push(record(&future));
        let mut forbidden_release: IdunnAuthenticatedProviderHealthProjectionRecord =
            rmp_serde::from_slice(&valid.payload)?;
        forbidden_release.release_id = Some("release-1".into());
        forbidden_release.release_witness_sha256 = Some(format!("sha256-{}", "d".repeat(64)));
        forbidden_release.source_commit = Some("e".repeat(40));
        forbidden_release.deployment_id = Some("deployment-1".into());
        resign(&signer, &mut forbidden_release);
        attacks.push(record(&forbidden_release));
        for attack in attacks {
            assert!(
                admit_required_idunn_provider_health(
                    &store,
                    std::slice::from_ref(&requirement),
                    &[attack],
                    &anchor(&signer),
                    RUNTIME,
                    NOW,
                    10_000,
                )
                .is_err()
            );
            assert_eq!(std::fs::read(&store)?, baseline);
        }

        let mut expired_anchor = anchor(&signer);
        expired_anchor.expires_at_unix_millis = Some(NOW);
        assert!(
            admit_required_idunn_provider_health(
                &store,
                std::slice::from_ref(&requirement),
                std::slice::from_ref(&valid),
                &expired_anchor,
                RUNTIME,
                NOW,
                10_000,
            )
            .is_err()
        );
        let mut wrong_purpose = anchor(&signer);
        wrong_purpose.signing_purpose = "caller-selected-purpose".into();
        assert!(
            admit_required_idunn_provider_health(
                &store,
                std::slice::from_ref(&requirement),
                std::slice::from_ref(&valid),
                &wrong_purpose,
                RUNTIME,
                NOW,
                10_000,
            )
            .is_err()
        );
        let mut late_anchor = anchor(&signer);
        late_anchor.bound_at_unix_millis = NOW - 100;
        assert!(
            admit_required_idunn_provider_health(
                &store,
                std::slice::from_ref(&requirement),
                std::slice::from_ref(&valid),
                &late_anchor,
                RUNTIME,
                NOW,
                10_000,
            )
            .is_err()
        );
        assert!(
            admit_required_idunn_provider_health(
                &store,
                std::slice::from_ref(&requirement),
                std::slice::from_ref(&valid),
                &anchor(&signer),
                RUNTIME,
                NOW + 20_000,
                10_000,
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&store)?, baseline);
        Ok(())
    }

    #[test]
    fn required_set_refuses_duplicate_missing_unknown_and_release_substitution() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let signer =
            enroll_service_identity_at::<IdunnServiceIdentity>(&temp.path().join("id.cc"))?;
        let requirement = required("epiphany");
        assert!(
            required_idunn_provider_health_query(
                "duplicate",
                &[requirement.clone(), requirement.clone()],
                RUNTIME,
            )
            .is_err()
        );
        assert!(
            admit_required_idunn_provider_health(
                &temp.path().join("missing.cc"),
                std::slice::from_ref(&requirement),
                &[],
                &anchor(&signer),
                RUNTIME,
                NOW,
                10_000,
            )
            .is_err()
        );
        let unknown_requirement = required("odin");
        let unknown = record(&projection(&signer, &unknown_requirement, 1, 1));
        assert!(
            admit_required_idunn_provider_health(
                &temp.path().join("unknown.cc"),
                std::slice::from_ref(&requirement),
                &[unknown],
                &anchor(&signer),
                RUNTIME,
                NOW,
                10_000,
            )
            .is_err()
        );
        let mut exact = requirement.clone();
        exact.release_binding = ProviderReleaseBinding::Exact {
            release_id: "release-1".into(),
            release_witness_sha256: format!("sha256-{}", "d".repeat(64)),
            source_commit: "e".repeat(40),
            deployment_id: "deployment-1".into(),
        };
        assert!(
            admit_required_idunn_provider_health(
                &temp.path().join("release.cc"),
                std::slice::from_ref(&exact),
                &[record(&projection(&signer, &exact, 1, 1))],
                &anchor(&signer),
                RUNTIME,
                NOW,
                10_000,
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn monotonic_cas_accepts_successor_and_rejects_replays_without_write() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let signer =
            enroll_service_identity_at::<IdunnServiceIdentity>(&temp.path().join("id.cc"))?;
        let requirement = required("epiphany");
        let store = temp.path().join("admission.cc");
        let first = record(&projection(&signer, &requirement, 1, 1));
        admit_required_idunn_provider_health(
            &store,
            std::slice::from_ref(&requirement),
            &[first],
            &anchor(&signer),
            RUNTIME,
            NOW,
            10_000,
        )?;
        let second = record(&projection(&signer, &requirement, 2, 2));
        let accepted = admit_required_idunn_provider_health(
            &store,
            std::slice::from_ref(&requirement),
            &[second],
            &anchor(&signer),
            RUNTIME,
            NOW + 10,
            10_000,
        )?;
        assert_eq!(accepted[0].projection_sequence, 2);
        let baseline = std::fs::read(&store)?;
        let replay = record(&projection(&signer, &requirement, 1, 1));
        assert!(
            admit_required_idunn_provider_health(
                &store,
                std::slice::from_ref(&requirement),
                &[replay],
                &anchor(&signer),
                RUNTIME,
                NOW + 20,
                10_000
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&store)?, baseline);
        Ok(())
    }

    #[test]
    fn root_anchor_rotation_requires_newer_root_sight_and_resets_continuity_once() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let old_signer =
            enroll_service_identity_at::<IdunnServiceIdentity>(&temp.path().join("old-id.cc"))?;
        let new_signer =
            enroll_service_identity_at::<IdunnServiceIdentity>(&temp.path().join("new-id.cc"))?;
        let attacker =
            enroll_service_identity_at::<IdunnServiceIdentity>(&temp.path().join("attacker.cc"))?;
        let requirement = required("epiphany");
        let store = temp.path().join("admission.cc");
        let first = record(&projection(&old_signer, &requirement, 5, 5));
        admit_required_idunn_provider_health(
            &store,
            std::slice::from_ref(&requirement),
            &[first],
            &anchor(&old_signer),
            RUNTIME,
            NOW,
            10_000,
        )?;

        let mut new_anchor = anchor(&new_signer);
        new_anchor.bound_at_unix_millis = NOW + 100;
        new_anchor.expires_at_unix_millis = Some(NOW + 200_000);
        let mut rotated = projection(&new_signer, &requirement, 1, 1);
        rotated.provider_observed_at_unix_millis = NOW;
        rotated.admitted_at_unix_millis = NOW + 50;
        rotated.evaluated_at_unix_millis = NOW + 101;
        rotated.expires_at_unix_millis = NOW + 80_000;
        rotated.projection_incarnation_id = "idunn/boot/rotated".into();
        rotated.provider_incarnation_id = "provider/boot/rotated".into();
        resign(&new_signer, &mut rotated);
        let accepted = admit_required_idunn_provider_health(
            &store,
            std::slice::from_ref(&requirement),
            &[record(&rotated)],
            &new_anchor,
            RUNTIME,
            NOW + 200,
            10_000,
        )?;
        assert_eq!(
            accepted[0].idunn_signer_identity_id,
            new_anchor.signer_identity_id
        );
        let baseline = std::fs::read(&store)?;

        let mut old_after_rotation = projection(&old_signer, &requirement, 99, 99);
        old_after_rotation.evaluated_at_unix_millis = NOW + 150;
        old_after_rotation.expires_at_unix_millis = NOW + 60_000;
        resign(&old_signer, &mut old_after_rotation);
        assert!(
            admit_required_idunn_provider_health(
                &store,
                std::slice::from_ref(&requirement),
                &[record(&old_after_rotation)],
                &anchor(&old_signer),
                RUNTIME,
                NOW + 200,
                10_000,
            )
            .is_err()
        );

        let mut self_key = rotated.clone();
        self_key.idunn_signer_identity_id = attacker.entry().identity_id.clone();
        resign(&attacker, &mut self_key);
        assert!(
            admit_required_idunn_provider_health(
                &store,
                std::slice::from_ref(&requirement),
                &[record(&self_key)],
                &new_anchor,
                RUNTIME,
                NOW + 200,
                10_000,
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&store)?, baseline);
        Ok(())
    }
}
