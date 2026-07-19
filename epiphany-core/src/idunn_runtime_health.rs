use anyhow::{Context, Result, anyhow, bail};
use cultcache_rs::DatabaseEntry;
use cultnet_rs::{
    CultNetMessage, CultNetRawDocumentRecord, CultNetRawPayloadEncoding,
    CultNetRudpSocketTransportConnection, CultNetRudpSocketTransportOptions, CultNetWireContract,
    GameCultProviderHealthIdentity, IdunnSignedDaemonHealthPurpose, IdunnSignedDaemonHealthRecord,
    ServiceIdentitySigner, encode_cultnet_message_to_vec,
};
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

pub const EPIPHANY_IDUNN_RUNTIME_HEALTH_CONTRACT: &str = "epiphany.cultnet-rudp-runtime-health";
pub const CULTNET_RUDP_PROTOCOL_ID: &str = "cultnet.transport.rudp.v0";
const IDUNN_HEALTH_RUDP_CONNECTION_ID: u32 = 0x1d0d_0001;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdunnDaemonHealthDocument {
    pub daemon_id: String,
    pub state: String,
    pub detail: String,
    pub observed_at: String,
    pub health_contract: String,
    pub publication_source: String,
    pub transport: String,
}

pub fn sign_epiphany_runtime_health(
    health: IdunnDaemonHealthDocument,
    source_runtime_id: &str,
    release_id: &str,
    release_witness_sha256: &str,
    source_commit: &str,
    deployment_request_id: &str,
    publisher_incarnation_id: &str,
    publisher_sequence: u64,
    signer: &ServiceIdentitySigner<GameCultProviderHealthIdentity>,
) -> Result<IdunnSignedDaemonHealthRecord> {
    validate_health_document(&health)?;
    let observed_at_unix_millis = chrono::DateTime::parse_from_rfc3339(&health.observed_at)?
        .timestamp_millis()
        .try_into()
        .context("runtime health observation precedes Unix epoch")?;
    let mut record = IdunnSignedDaemonHealthRecord {
        schema_version: "idunn.signed_daemon_health.v1".into(),
        daemon_id: health.daemon_id,
        health_contract: health.health_contract,
        source_runtime_id: source_runtime_id.into(),
        state: health.state,
        detail: health.detail,
        signer_identity_id: signer.entry().identity_id.clone(),
        publisher_incarnation_id: publisher_incarnation_id.into(),
        publisher_sequence,
        observed_at_unix_millis,
        release_id: Some(release_id.into()),
        release_witness_sha256: Some(release_witness_sha256.into()),
        source_commit: Some(source_commit.into()),
        deployment_id: Some(deployment_request_id.into()),
        signature_algorithm: "ed25519".into(),
        signature: Vec::new(),
        private_state_exposed: false,
    };
    validate_unsigned_record(&record)?;
    let canonical = canonical_unsigned_record(&record)?;
    let proof = signer.sign::<IdunnSignedDaemonHealthPurpose>(&canonical);
    if proof.identity_id != record.signer_identity_id {
        bail!("provider-health signer identity disagrees with record")
    }
    record.signature = proof.signature;
    record.validate()?;
    Ok(record)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpiphanyAggregateRuntimeHealthInput {
    pub daemon_id: String,
    pub health_contract: String,
    pub observed_at: String,
    pub release_authenticated: bool,
    pub expected_service_count: usize,
    pub terminal_current_service_count: usize,
    pub warming_service_count: usize,
    pub contradictions: Vec<String>,
}

pub fn derive_epiphany_aggregate_runtime_health(
    input: EpiphanyAggregateRuntimeHealthInput,
) -> Result<IdunnDaemonHealthDocument> {
    require_id(&input.daemon_id, "Idunn daemon id")?;
    require_id(&input.health_contract, "Idunn health contract")?;
    require_id(&input.observed_at, "Idunn health observation time")?;
    if input.expected_service_count == 0 {
        bail!("aggregate runtime health requires at least one managed service");
    }
    if input.terminal_current_service_count > input.expected_service_count
        || input.warming_service_count > input.expected_service_count
        || input
            .terminal_current_service_count
            .saturating_add(input.warming_service_count)
            > input.expected_service_count
    {
        bail!("managed service health counts exceed the expected service count");
    }
    let (state, detail) = if !input.contradictions.is_empty() {
        ("failed", "runtime-contradiction")
    } else if !input.release_authenticated {
        ("failed", "release-authentication-failed")
    } else if input.terminal_current_service_count == input.expected_service_count {
        ("active", "managed-services-current")
    } else if input.warming_service_count > 0
        && input
            .terminal_current_service_count
            .saturating_add(input.warming_service_count)
            == input.expected_service_count
    {
        ("warming", "managed-services-warming")
    } else {
        ("degraded", "managed-services-reconciling")
    };
    Ok(IdunnDaemonHealthDocument {
        daemon_id: input.daemon_id,
        state: state.into(),
        detail: detail.into(),
        observed_at: input.observed_at,
        health_contract: input.health_contract,
        publication_source: "daemon-published".into(),
        transport: CULTNET_RUDP_PROTOCOL_ID.into(),
    })
}

pub fn publish_idunn_daemon_health_rudp(
    endpoint: SocketAddr,
    source_runtime_id: &str,
    signed: &IdunnSignedDaemonHealthRecord,
) -> Result<()> {
    signed.validate()?;
    require_id(source_runtime_id, "health publisher runtime id")?;
    if source_runtime_id != signed.source_runtime_id {
        bail!("health transport source disagrees with signed source runtime");
    }
    let payload = rmp_serde::to_vec(signed).context("encoding canonical signed Idunn health")?;
    let decoded: IdunnSignedDaemonHealthRecord = rmp_serde::from_slice(&payload)?;
    if decoded != *signed || rmp_serde::to_vec(&decoded)? != payload {
        bail!("signed Idunn health encoding is noncanonical");
    }
    let message = CultNetMessage::DocumentPutRaw {
        message_id: format!(
            "epiphany-signed-health:{}:{}:{}",
            signed.daemon_id, signed.publisher_incarnation_id, signed.publisher_sequence
        ),
        document: CultNetRawDocumentRecord {
            schema_id: IdunnSignedDaemonHealthRecord::TYPE.into(),
            record_key: signed.daemon_id.clone(),
            stored_at: chrono::DateTime::from_timestamp_millis(
                signed.observed_at_unix_millis.try_into()?,
            )
            .context("signed health observation time is invalid")?
            .to_rfc3339(),
            payload_encoding: CultNetRawPayloadEncoding::Messagepack,
            payload,
            source_runtime_id: Some(source_runtime_id.into()),
            source_agent_id: Some(signed.signer_identity_id.clone()),
            source_role: Some("signed-daemon-health-publisher".into()),
            tags: Some(vec![CULTNET_RUDP_PROTOCOL_ID.into()]),
        },
    };
    let bind = if endpoint.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };
    let socket = UdpSocket::bind(bind)
        .with_context(|| format!("binding Epiphany Idunn RUDP sender at {bind}"))?;
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    let mut transport =
        CultNetRudpSocketTransportConnection::new(CultNetRudpSocketTransportOptions::client(
            source_runtime_id,
            socket,
            endpoint,
            IDUNN_HEALTH_RUDP_CONNECTION_ID,
        ))?;
    transport.connect(Vec::new())?;
    let deadline = Instant::now() + Duration::from_millis(500);
    while !transport.connected() {
        let _ = transport.receive_once()?;
        transport.poll_resends()?;
        if Instant::now() >= deadline {
            return Err(anyhow!(
                "timed out connecting Epiphany health publisher to {endpoint}"
            ));
        }
    }
    transport.send(
        "schema",
        encode_cultnet_message_to_vec(&message, CultNetWireContract::CultNetSchemaV0)?,
    )?;
    Ok(())
}

fn canonical_unsigned_record(record: &IdunnSignedDaemonHealthRecord) -> Result<Vec<u8>> {
    let mut unsigned = record.clone();
    unsigned.signature.clear();
    validate_unsigned_record(&unsigned)?;
    rmp_serde::to_vec(&unsigned).context("encoding canonical unsigned Idunn health")
}

fn validate_unsigned_record(record: &IdunnSignedDaemonHealthRecord) -> Result<()> {
    if !record.signature.is_empty() {
        bail!("unsigned signing record already contains a signature");
    }
    let mut signed_shape = record.clone();
    signed_shape.signature = vec![0; 64];
    signed_shape.validate()?;
    uuid::Uuid::parse_str(&record.publisher_incarnation_id)
        .context("publisher incarnation id must be UUID")?;
    Ok(())
}

fn validate_health_document(health: &IdunnDaemonHealthDocument) -> Result<()> {
    require_id(&health.daemon_id, "Idunn daemon id")?;
    require_id(&health.health_contract, "Idunn health contract")?;
    require_id(&health.observed_at, "Idunn health observation time")?;
    if !matches!(
        health.state.as_str(),
        "active" | "warming" | "degraded" | "failed"
    ) {
        bail!("unsupported Idunn daemon health state");
    }
    if !matches!(
        health.detail.as_str(),
        "runtime-contradiction"
            | "release-authentication-failed"
            | "managed-services-current"
            | "managed-services-warming"
            | "managed-services-reconciling"
    ) {
        bail!("Idunn daemon health detail is not a bounded generic reason");
    }
    if health.publication_source != "daemon-published"
        || health.transport != CULTNET_RUDP_PROTOCOL_ID
    {
        bail!("Idunn daemon health publication authority is invalid");
    }
    chrono::DateTime::parse_from_rfc3339(&health.observed_at)?;
    Ok(())
}

fn require_id(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        bail!("{label} is empty, oversized, or contains control characters");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cultnet_rs::{
        ServiceIdentityProfile, ServiceIdentitySignature, ServiceSignaturePurpose,
        decode_cultnet_message_from_slice, enroll_service_identity_at,
        verify_service_identity_signature,
    };
    use std::thread;

    struct WrongPurpose;
    impl ServiceSignaturePurpose<GameCultProviderHealthIdentity> for WrongPurpose {
        const PURPOSE: &'static [u8] = b"wrong-purpose";
    }

    fn input(current: usize) -> EpiphanyAggregateRuntimeHealthInput {
        EpiphanyAggregateRuntimeHealthInput {
            daemon_id: "yggdrasil-epiphany".into(),
            health_contract: EPIPHANY_IDUNN_RUNTIME_HEALTH_CONTRACT.into(),
            observed_at: "2026-07-16T00:00:00Z".into(),
            release_authenticated: true,
            expected_service_count: 2,
            terminal_current_service_count: current,
            warming_service_count: 0,
            contradictions: Vec::new(),
        }
    }

    fn signed_health() -> (
        tempfile::TempDir,
        ServiceIdentitySigner<GameCultProviderHealthIdentity>,
        IdunnSignedDaemonHealthRecord,
    ) {
        let root = tempfile::tempdir().unwrap();
        let signer = enroll_service_identity_at::<GameCultProviderHealthIdentity>(
            &root.path().join("provider.cc"),
        )
        .unwrap();
        let record = sign_epiphany_runtime_health(
            derive_epiphany_aggregate_runtime_health(input(2)).unwrap(),
            "epiphany-yggdrasil",
            "release-test",
            &format!("sha256-{}", "a".repeat(64)),
            &"b".repeat(40),
            "deploy-request-test",
            "00000000-0000-4000-8000-000000000001",
            1,
            &signer,
        )
        .unwrap();
        (root, signer, record)
    }

    #[test]
    fn aggregate_health_uses_closed_nonprivate_reasons() {
        assert_eq!(
            derive_epiphany_aggregate_runtime_health(input(2))
                .unwrap()
                .detail,
            "managed-services-current"
        );
        let mut warming = input(1);
        warming.warming_service_count = 1;
        assert_eq!(
            derive_epiphany_aggregate_runtime_health(warming)
                .unwrap()
                .detail,
            "managed-services-warming"
        );
        let mut failed = input(2);
        failed
            .contradictions
            .push("secret path and process detail".into());
        assert_eq!(
            derive_epiphany_aggregate_runtime_health(failed)
                .unwrap()
                .detail,
            "runtime-contradiction"
        );
    }

    #[test]
    fn canonical_record_binds_exact_release_and_process_continuity() {
        let (_root, signer, record) = signed_health();
        assert_eq!(record.source_runtime_id, "epiphany-yggdrasil");
        assert_eq!(record.release_id.as_deref(), Some("release-test"));
        assert_eq!(record.deployment_id.as_deref(), Some("deploy-request-test"));
        assert_eq!(record.publisher_sequence, 1);
        assert!(!record.private_state_exposed);
        let statement = canonical_unsigned_record(&record).unwrap();
        let proof = ServiceIdentitySignature {
            identity_id: record.signer_identity_id.clone(),
            signature: record.signature.clone(),
        };
        verify_service_identity_signature::<
            GameCultProviderHealthIdentity,
            IdunnSignedDaemonHealthPurpose,
        >(&signer.trust_anchor().unwrap(), &statement, &proof)
        .unwrap();
        assert_eq!(&statement[..3], &[0xdc, 0, 17]);
    }

    #[test]
    fn wrong_purpose_key_mutation_and_noncanonical_encoding_fail() {
        let (_root, signer, record) = signed_health();
        let statement = canonical_unsigned_record(&record).unwrap();
        let wrong = signer.sign::<WrongPurpose>(&statement);
        assert!(
            verify_service_identity_signature::<
                GameCultProviderHealthIdentity,
                IdunnSignedDaemonHealthPurpose,
            >(&signer.trust_anchor().unwrap(), &statement, &wrong)
            .is_err()
        );
        let other_root = tempfile::tempdir().unwrap();
        let other = enroll_service_identity_at::<GameCultProviderHealthIdentity>(
            &other_root.path().join("other.cc"),
        )
        .unwrap();
        let proof = ServiceIdentitySignature {
            identity_id: record.signer_identity_id.clone(),
            signature: record.signature.clone(),
        };
        assert!(
            verify_service_identity_signature::<
                GameCultProviderHealthIdentity,
                IdunnSignedDaemonHealthPurpose,
            >(&other.trust_anchor().unwrap(), &statement, &proof)
            .is_err()
        );
        let mut mutated = record.clone();
        mutated.state = "failed".into();
        assert!(
            verify_service_identity_signature::<
                GameCultProviderHealthIdentity,
                IdunnSignedDaemonHealthPurpose,
            >(
                &signer.trust_anchor().unwrap(),
                &canonical_unsigned_record(&mutated).unwrap(),
                &proof
            )
            .is_err()
        );
        let named = rmp_serde::to_vec(&serde_json::json!({
            "schema_version": record.schema_version,
            "daemon_id": record.daemon_id,
            "health_contract": record.health_contract,
            "source_runtime_id": record.source_runtime_id,
            "state": record.state,
            "detail": record.detail,
            "signer_identity_id": record.signer_identity_id,
            "publisher_incarnation_id": record.publisher_incarnation_id,
            "publisher_sequence": record.publisher_sequence,
            "observed_at_unix_millis": record.observed_at_unix_millis,
            "release_id": record.release_id,
            "release_witness_sha256": record.release_witness_sha256,
            "source_commit": record.source_commit,
            "deployment_id": record.deployment_id,
            "signature_algorithm": record.signature_algorithm,
            "signature": [],
            "private_state_exposed": record.private_state_exposed,
        }))
        .unwrap();
        assert!(
            verify_service_identity_signature::<
                GameCultProviderHealthIdentity,
                IdunnSignedDaemonHealthPurpose,
            >(&signer.trust_anchor().unwrap(), &named, &proof)
            .is_err()
        );
    }

    #[test]
    fn sequence_and_incarnation_are_strict() {
        let (_root, signer, record) = signed_health();
        let health = derive_epiphany_aggregate_runtime_health(input(2)).unwrap();
        assert!(
            sign_epiphany_runtime_health(
                health.clone(),
                "epiphany-yggdrasil",
                "r",
                &format!("sha256-{}", "a".repeat(64)),
                &"b".repeat(40),
                "d",
                "bad",
                2,
                &signer
            )
            .is_err()
        );
        assert!(
            sign_epiphany_runtime_health(
                health,
                "epiphany-yggdrasil",
                "r",
                &format!("sha256-{}", "a".repeat(64)),
                &"b".repeat(40),
                "d",
                &record.publisher_incarnation_id,
                0,
                &signer
            )
            .is_err()
        );
    }

    #[test]
    fn rudp_publisher_emits_exact_canonical_tuple() {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        let endpoint = socket.local_addr().unwrap();
        let server = thread::spawn(move || {
            let mut transport = CultNetRudpSocketTransportConnection::new(
                CultNetRudpSocketTransportOptions::server(
                    "idunn-test",
                    socket,
                    IDUNN_HEALTH_RUDP_CONNECTION_ID,
                ),
            )
            .unwrap();
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                if let Some(frame) = transport.receive_once().unwrap() {
                    return decode_cultnet_message_from_slice(
                        &frame.payload,
                        CultNetWireContract::CultNetSchemaV0,
                    )
                    .unwrap();
                }
                transport.poll_resends().unwrap();
                assert!(Instant::now() < deadline);
            }
        });
        let (_root, _signer, record) = signed_health();
        publish_idunn_daemon_health_rudp(endpoint, "epiphany-yggdrasil", &record).unwrap();
        let CultNetMessage::DocumentPutRaw { document, .. } = server.join().unwrap() else {
            panic!("wrong message")
        };
        assert_eq!(document.schema_id, IdunnSignedDaemonHealthRecord::TYPE);
        assert_eq!(
            document.source_role.as_deref(),
            Some("signed-daemon-health-publisher")
        );
        let decoded: IdunnSignedDaemonHealthRecord =
            rmp_serde::from_slice(&document.payload).unwrap();
        assert_eq!(decoded, record);
        assert_eq!(rmp_serde::to_vec(&decoded).unwrap(), document.payload);
    }

    #[test]
    fn identity_profile_is_not_host_identity() {
        assert_eq!(
            GameCultProviderHealthIdentity::TRUST_ANCHOR_SCHEMA,
            "gamecult.provider_health_identity.trust_anchor.v1"
        );
        assert_ne!(
            GameCultProviderHealthIdentity::ID_DOMAIN,
            b"epiphany.host-incarnation.identity.v0\0"
        );
    }
}
