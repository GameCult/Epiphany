use anyhow::{Context, Result, anyhow, bail};
use cultnet_rs::{
    CultNetMessage, CultNetRawDocumentRecord, CultNetRawPayloadEncoding,
    CultNetRudpSocketTransportConnection, CultNetRudpSocketTransportOptions, CultNetWireContract,
    encode_cultnet_message_to_vec,
};
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

pub const IDUNN_DAEMON_HEALTH_TYPE: &str = "idunn.daemon_health";
pub const IDUNN_DAEMON_HEALTH_SCHEMA_VERSION: &str = "idunn.daemon_health.v1";
pub const EPIPHANY_SIGNED_RUNTIME_HEALTH_TYPE: &str = "epiphany.idunn_signed_runtime_health";
pub const EPIPHANY_SIGNED_RUNTIME_HEALTH_SCHEMA_VERSION: &str =
    "epiphany.idunn_signed_runtime_health.v0";
pub const EPIPHANY_IDUNN_RUNTIME_HEALTH_CONTRACT: &str = "epiphany.cultnet-rudp-runtime-health";
pub const CULTNET_RUDP_PROTOCOL_ID: &str = "cultnet.transport.rudp.v0";
// Shared Idunn daemon-health RUDP contract. This must match the Idunn ingress
// constant; a private Epiphany connection id cannot complete the handshake.
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanySignedRuntimeHealthDocument {
    pub schema_version: String,
    pub health: IdunnDaemonHealthDocument,
    pub source_runtime_id: String,
    pub release_id: String,
    pub release_witness_sha256: String,
    pub source_commit: String,
    pub deployment_request_id: String,
    pub publisher_incarnation_id: String,
    pub publisher_sequence: u64,
    pub publisher_process_id: u32,
    pub publisher_process_creation_token: u64,
    pub publisher_process_created_at: String,
    pub publisher_executable_path: String,
    pub signer_identity_id: String,
    pub signature_algorithm: String,
    pub signature: Vec<u8>,
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
    process: &crate::ProcessInstanceIdentity,
    signer: &crate::HostIdentitySigner,
) -> Result<EpiphanySignedRuntimeHealthDocument> {
    validate_health_document(&health)?;
    let mut document = EpiphanySignedRuntimeHealthDocument {
        schema_version: EPIPHANY_SIGNED_RUNTIME_HEALTH_SCHEMA_VERSION.into(),
        health,
        source_runtime_id: source_runtime_id.into(),
        release_id: release_id.into(),
        release_witness_sha256: release_witness_sha256.into(),
        source_commit: source_commit.into(),
        deployment_request_id: deployment_request_id.into(),
        publisher_incarnation_id: publisher_incarnation_id.into(),
        publisher_sequence,
        publisher_process_id: process.process_id,
        publisher_process_creation_token: process.creation_token,
        publisher_process_created_at: process
            .created_at_rfc3339
            .clone()
            .context("runtime health publisher process creation time is unavailable")?,
        publisher_executable_path: process.executable_path.display().to_string(),
        signer_identity_id: signer.entry().identity_id.clone(),
        signature_algorithm: "ed25519".into(),
        signature: Vec::new(),
    };
    validate_signed_health_shape(&document, false)?;
    let statement = signed_health_statement(&document)?;
    document.signature = signer
        .sign(EPIPHANY_SIGNED_RUNTIME_HEALTH_TYPE, &statement)?
        .signature;
    validate_signed_health_shape(&document, true)?;
    Ok(document)
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
        (
            "failed",
            format!(
                "Epiphany runtime authority is contradictory: {}",
                input.contradictions.join("; ")
            ),
        )
    } else if !input.release_authenticated {
        (
            "failed",
            "Epiphany packaged release authentication failed.".to_string(),
        )
    } else if input.terminal_current_service_count == input.expected_service_count {
        (
            "active",
            format!(
                "Epiphany packaged release and all {} managed services have terminal current authority.",
                input.expected_service_count
            ),
        )
    } else if input.warming_service_count > 0
        && input
            .terminal_current_service_count
            .saturating_add(input.warming_service_count)
            == input.expected_service_count
    {
        (
            "warming",
            format!(
                "Epiphany is advancing authenticated durable work: {} terminal current, {} warming, {} expected.",
                input.terminal_current_service_count,
                input.warming_service_count,
                input.expected_service_count
            ),
        )
    } else {
        (
            "degraded",
            format!(
                "Epiphany is reconciling managed services: {} terminal current, {} warming, {} expected.",
                input.terminal_current_service_count,
                input.warming_service_count,
                input.expected_service_count
            ),
        )
    };

    Ok(IdunnDaemonHealthDocument {
        daemon_id: input.daemon_id,
        state: state.to_string(),
        detail,
        observed_at: input.observed_at,
        health_contract: input.health_contract,
        publication_source: "daemon-published".to_string(),
        transport: CULTNET_RUDP_PROTOCOL_ID.to_string(),
    })
}

pub fn publish_idunn_daemon_health_rudp(
    endpoint: SocketAddr,
    source_runtime_id: &str,
    signed: &EpiphanySignedRuntimeHealthDocument,
) -> Result<()> {
    validate_signed_health_shape(signed, true)?;
    require_id(source_runtime_id, "health publisher runtime id")?;
    if source_runtime_id != signed.source_runtime_id {
        bail!("health transport source disagrees with signed source runtime");
    }
    let message = CultNetMessage::DocumentPutRaw {
        message_id: format!(
            "epiphany-health:{}:{}",
            signed.health.daemon_id,
            signed.health.observed_at.replace(':', "-")
        ),
        document: CultNetRawDocumentRecord {
            schema_id: EPIPHANY_SIGNED_RUNTIME_HEALTH_TYPE.to_string(),
            record_key: signed.health.daemon_id.clone(),
            stored_at: signed.health.observed_at.clone(),
            payload_encoding: CultNetRawPayloadEncoding::Messagepack,
            payload: rmp_serde::to_vec_named(signed)
                .context("encoding signed Epiphany Idunn daemon health")?,
            source_runtime_id: Some(source_runtime_id.to_string()),
            source_agent_id: None,
            source_role: Some("daemon-health-publisher".to_string()),
            tags: Some(vec![CULTNET_RUDP_PROTOCOL_ID.to_string()]),
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
    let payload = encode_cultnet_message_to_vec(&message, CultNetWireContract::CultNetSchemaV0)
        .context("encoding Idunn health CultNet message")?;
    transport
        .send("schema", payload)
        .with_context(|| format!("sending Epiphany Idunn health to {endpoint}"))
}

fn signed_health_statement(document: &EpiphanySignedRuntimeHealthDocument) -> Result<Vec<u8>> {
    let mut unsigned = document.clone();
    unsigned.signature.clear();
    validate_signed_health_shape(&unsigned, false)?;
    rmp_serde::to_vec_named(&unsigned).context("encoding signed runtime health statement")
}

fn validate_signed_health_shape(
    document: &EpiphanySignedRuntimeHealthDocument,
    signed: bool,
) -> Result<()> {
    if document.schema_version != EPIPHANY_SIGNED_RUNTIME_HEALTH_SCHEMA_VERSION {
        bail!("signed Epiphany runtime health schema is invalid");
    }
    validate_health_document(&document.health)?;
    for (label, value) in [
        ("source runtime id", document.source_runtime_id.as_str()),
        ("release id", document.release_id.as_str()),
        ("source commit", document.source_commit.as_str()),
        (
            "deployment request id",
            document.deployment_request_id.as_str(),
        ),
        (
            "publisher incarnation id",
            document.publisher_incarnation_id.as_str(),
        ),
        (
            "publisher process creation time",
            document.publisher_process_created_at.as_str(),
        ),
        (
            "publisher executable path",
            document.publisher_executable_path.as_str(),
        ),
        ("signer identity id", document.signer_identity_id.as_str()),
    ] {
        require_id(value, label)?;
    }
    uuid::Uuid::parse_str(&document.publisher_incarnation_id)
        .context("publisher incarnation id must be UUID")?;
    if document.publisher_sequence == 0
        || document.publisher_process_id == 0
        || document.publisher_process_creation_token == 0
    {
        bail!("signed Epiphany runtime health publisher identity is invalid");
    }
    chrono::DateTime::parse_from_rfc3339(&document.publisher_process_created_at)?;
    let witness = document
        .release_witness_sha256
        .strip_prefix("sha256-")
        .unwrap_or(&document.release_witness_sha256);
    if witness.len() != 64
        || !witness
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        || document.source_commit.len() != 40
        || !document
            .source_commit
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        || document.signature_algorithm != "ed25519"
        || (signed && document.signature.len() != 64)
        || (!signed && !document.signature.is_empty())
    {
        bail!("signed Epiphany runtime health cryptographic shape is invalid");
    }
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
    if health.publication_source != "daemon-published"
        || health.transport != CULTNET_RUDP_PROTOCOL_ID
    {
        bail!("Idunn daemon health publication authority is invalid");
    }
    Ok(())
}

fn require_id(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{label} is required");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cultnet_rs::decode_cultnet_message_from_slice;
    use std::thread;

    fn signed_health(health: IdunnDaemonHealthDocument) -> EpiphanySignedRuntimeHealthDocument {
        let root = tempfile::tempdir().unwrap();
        let signer = crate::enroll_host_identity_at(&root.path().join("host.ccmp")).unwrap();
        sign_epiphany_runtime_health(
            health,
            "epiphany-daemon-supervisor",
            "release-test",
            &format!("sha256-{}", "a".repeat(64)),
            &"b".repeat(40),
            "deploy-request-test",
            "00000000-0000-4000-8000-000000000001",
            1,
            &crate::ProcessInstanceIdentity {
                process_id: 42,
                creation_token: 7,
                created_at_rfc3339: Some("2026-07-16T00:00:00Z".into()),
                executable_path: std::path::PathBuf::from("/srv/epiphany/supervisor"),
            },
            &signer,
        )
        .unwrap()
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

    #[test]
    fn aggregate_health_requires_every_current_lineage() {
        assert_eq!(
            derive_epiphany_aggregate_runtime_health(input(2))
                .unwrap()
                .state,
            "active"
        );
        assert_eq!(
            derive_epiphany_aggregate_runtime_health(input(1))
                .unwrap()
                .state,
            "degraded"
        );
    }

    #[test]
    fn aggregate_health_names_authenticated_nonterminal_work_as_warming() {
        let mut value = input(1);
        value.warming_service_count = 1;
        let health = derive_epiphany_aggregate_runtime_health(value).unwrap();
        assert_eq!(health.state, "warming");
        assert!(health.detail.contains("1 terminal current, 1 warming"));
    }

    #[test]
    fn warming_cannot_hide_a_missing_required_service() {
        let mut value = input(0);
        value.warming_service_count = 1;
        assert_eq!(
            derive_epiphany_aggregate_runtime_health(value)
                .unwrap()
                .state,
            "degraded"
        );
    }

    #[test]
    fn aggregate_health_fails_authenticated_contradictions() {
        let mut value = input(2);
        value
            .contradictions
            .push("semantic child uses stale policy".into());
        let health = derive_epiphany_aggregate_runtime_health(value).unwrap();
        assert_eq!(health.state, "failed");
        assert!(health.detail.contains("stale policy"));
    }

    #[test]
    fn semantic_sight_cannot_substitute_for_runtime_lineage() {
        let health = derive_epiphany_aggregate_runtime_health(input(0)).unwrap();
        assert_ne!(health.state, "active");
    }

    #[test]
    fn health_document_rejects_alien_publication_authority() {
        let mut health = derive_epiphany_aggregate_runtime_health(input(2)).unwrap();
        health.publication_source = "probe".into();
        assert!(validate_health_document(&health).is_err());
    }

    #[test]
    fn rudp_publisher_emits_exact_messagepack_health_document() {
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
                assert!(Instant::now() < deadline, "health frame timed out");
            }
        });
        let health = derive_epiphany_aggregate_runtime_health(input(2)).unwrap();
        let signed = signed_health(health.clone());
        publish_idunn_daemon_health_rudp(endpoint, "epiphany-daemon-supervisor", &signed).unwrap();
        let CultNetMessage::DocumentPutRaw { document, .. } = server.join().unwrap() else {
            panic!("publisher emitted a non-document message")
        };
        assert_eq!(document.schema_id, EPIPHANY_SIGNED_RUNTIME_HEALTH_TYPE);
        assert_eq!(document.record_key, health.daemon_id);
        assert_eq!(
            document.payload_encoding,
            CultNetRawPayloadEncoding::Messagepack
        );
        let decoded: EpiphanySignedRuntimeHealthDocument =
            rmp_serde::from_slice(&document.payload).unwrap();
        assert_eq!(decoded, signed);
    }

    #[test]
    fn rudp_publisher_fails_closed_when_idunn_does_not_accept() {
        let reserved = UdpSocket::bind("127.0.0.1:0").unwrap();
        let endpoint = reserved.local_addr().unwrap();
        drop(reserved);
        let health = signed_health(derive_epiphany_aggregate_runtime_health(input(2)).unwrap());
        assert!(
            publish_idunn_daemon_health_rudp(endpoint, "epiphany-daemon-supervisor", &health,)
                .is_err()
        );
    }

    #[test]
    fn rudp_publisher_uses_the_idunn_daemon_health_connection_contract() {
        assert_eq!(IDUNN_HEALTH_RUDP_CONNECTION_ID, 0x1d0d_0001);
    }
}
