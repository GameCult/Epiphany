use anyhow::{Context, Result, anyhow, bail};
use cultnet_rs::{
    CultNetMessage, CultNetRawDocumentRecord, CultNetRawPayloadEncoding,
    CultNetRudpSocketTransportConnection, CultNetRudpSocketTransportOptions, CultNetWireContract,
    decode_cultnet_message_from_slice, encode_cultnet_message_to_vec,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::net::UdpSocket;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{
    BIFROST_OPERATOR_COMMAND_DELIVERY_TYPE, BifrostOperatorCommandAdmission, HostIdentitySigner,
    HostIdentityTrustAnchorEntry, OperatorCommandPolicy, OperatorCommandResult,
    admit_and_execute_bifrost_operator_command,
};

pub const EPIPHANY_OPERATOR_COMMAND_RESULT_RECEIPT_TYPE: &str =
    "epiphany.operator_command.sealed_result";
pub const EPIPHANY_OPERATOR_COMMAND_RESULT_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.operator_command.sealed_result.v1";
const RESULT_SIGNING_PURPOSE: &str = "epiphany.operator-command.sealed-result.v1";
pub const EPIPHANY_OPERATOR_COMMAND_RUDP_CONNECTION_ID: u32 = 0xe91f_0001;

#[derive(Clone, Debug)]
pub struct OperatorCommandServiceConfig {
    pub command_store: PathBuf,
    pub local_verse_store: PathBuf,
    pub resident_self_store: PathBuf,
    pub runtime_store: PathBuf,
    pub policy: OperatorCommandPolicy,
    pub trusted_bifrost_identity: HostIdentityTrustAnchorEntry,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EpiphanyOperatorCommandResultReceipt {
    pub schema_version: String,
    pub result: EpiphanyOperatorCommandWireResult,
    pub result_payload_sha256: String,
    pub command_id: String,
    pub packet_sha256: String,
    pub target_runtime_id: String,
    pub completed_at: String,
    pub provider_identity_id: String,
    pub private_state_exposed: bool,
    pub executor_signature: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EpiphanyOperatorCommandWireResult {
    pub schema_version: String,
    pub result_id: String,
    pub command_id: String,
    pub packet_sha256: String,
    pub target_runtime_id: String,
    pub disposition: String,
    pub consequence_kind: String,
    pub consequence_ref: String,
    pub completed_at: String,
    pub private_state_exposed: bool,
    pub operator_status: String,
    pub state_status: String,
    pub coordinator_action: String,
    pub brake_status: String,
    pub detail: String,
    pub reviews: Vec<crate::RepoFrontierPlanReviewSummary>,
    pub review_candidate_id: String,
    pub review_decision: String,
}

impl From<&OperatorCommandResult> for EpiphanyOperatorCommandWireResult {
    fn from(value: &OperatorCommandResult) -> Self {
        Self {
            schema_version: value.schema_version.clone(),
            result_id: value.result_id.clone(),
            command_id: value.command_id.clone(),
            packet_sha256: value.packet_sha256.clone(),
            target_runtime_id: value.target_runtime_id.clone(),
            disposition: value.disposition.clone(),
            consequence_kind: value.consequence_kind.clone(),
            consequence_ref: value.consequence_ref.clone(),
            completed_at: value.completed_at.clone(),
            private_state_exposed: value.private_state_exposed,
            operator_status: value.operator_status.clone(),
            state_status: value.state_status.clone(),
            coordinator_action: value.coordinator_action.clone(),
            brake_status: value.brake_status.clone(),
            detail: value.detail.clone(),
            reviews: value.reviews.clone(),
            review_candidate_id: value.review_candidate_id.clone(),
            review_decision: value.review_decision.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorCommandServiceReadiness {
    pub schema_version: String,
    pub status: String,
    pub runtime_id: String,
    pub bind: String,
    pub executor_identity_id: String,
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperatorCommandInteropFixtureManifest {
    pub schema_version: String,
    pub admission_file: String,
    pub bifrost_raw_trust_anchor_file: String,
    pub executor_raw_trust_anchor_file: String,
    pub executor_cultcache_trust_anchor_file: String,
    pub sealed_result_file: String,
    pub admission_sha256: String,
    pub sealed_result_sha256: String,
    pub admission_signing_purpose: String,
    pub result_signing_purpose: String,
    pub rudp_connection_id: u32,
    pub private_state_exposed: bool,
}

pub fn operator_command_result_receipt_signing_purpose() -> &'static str {
    RESULT_SIGNING_PURPOSE
}

pub fn operator_command_result_receipt_signing_payload(
    receipt: &EpiphanyOperatorCommandResultReceipt,
) -> Result<Vec<u8>> {
    rmp_serde::to_vec(&(
        receipt.schema_version.as_str(),
        &receipt.result,
        receipt.result_payload_sha256.as_str(),
        receipt.command_id.as_str(),
        receipt.packet_sha256.as_str(),
        receipt.target_runtime_id.as_str(),
        receipt.completed_at.as_str(),
        receipt.provider_identity_id.as_str(),
        receipt.private_state_exposed,
    ))
    .map_err(Into::into)
}

pub fn execute_operator_command_admission(
    config: &OperatorCommandServiceConfig,
    signer: &HostIdentitySigner,
    admission_payload: &[u8],
    now: &str,
) -> Result<EpiphanyOperatorCommandResultReceipt> {
    let admission: BifrostOperatorCommandAdmission = rmp_serde::from_slice(admission_payload)
        .context("operator service rejected malformed or non-strict Bifrost admission")?;
    let result = admit_and_execute_bifrost_operator_command(
        &config.command_store,
        &config.local_verse_store,
        &config.resident_self_store,
        &config.runtime_store,
        &admission,
        &config.trusted_bifrost_identity,
        &config.policy,
        now,
    )?;
    let wire_result = EpiphanyOperatorCommandWireResult::from(&result);
    let result_payload = rmp_serde::to_vec_named(&wire_result)?;
    let mut receipt = EpiphanyOperatorCommandResultReceipt {
        schema_version: EPIPHANY_OPERATOR_COMMAND_RESULT_RECEIPT_SCHEMA_VERSION.into(),
        result_payload_sha256: format!("sha256-{:x}", Sha256::digest(&result_payload)),
        command_id: result.command_id.clone(),
        packet_sha256: result.packet_sha256.clone(),
        target_runtime_id: result.target_runtime_id.clone(),
        completed_at: result.completed_at.clone(),
        provider_identity_id: signer.entry().identity_id.clone(),
        private_state_exposed: false,
        executor_signature: Vec::new(),
        result: wire_result,
    };
    receipt.executor_signature = signer
        .sign(
            RESULT_SIGNING_PURPOSE,
            &operator_command_result_receipt_signing_payload(&receipt)?,
        )?
        .signature;
    Ok(receipt)
}

pub fn serve_operator_command_rudp(
    socket: UdpSocket,
    config: &OperatorCommandServiceConfig,
    signer: &HostIdentitySigner,
) -> Result<()> {
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    let mut transport =
        CultNetRudpSocketTransportConnection::new(CultNetRudpSocketTransportOptions::server(
            &config.policy.runtime_id,
            socket,
            EPIPHANY_OPERATOR_COMMAND_RUDP_CONNECTION_ID,
        ))?;
    serve_operator_command_rudp_loop(&mut transport, config, signer, None, None)
}

fn serve_operator_command_rudp_loop(
    transport: &mut CultNetRudpSocketTransportConnection,
    config: &OperatorCommandServiceConfig,
    signer: &HostIdentitySigner,
    iteration_limit: Option<usize>,
    fixed_now: Option<&str>,
) -> Result<()> {
    let mut iterations = 0usize;
    loop {
        if iteration_limit.is_some_and(|limit| iterations >= limit) {
            return Ok(());
        }
        iterations += 1;
        match transport.receive_once() {
            Ok(Some(frame)) => {
                let response = match fixed_now {
                    Some(now) => process_wire_frame_at(config, signer, &frame.payload, now),
                    None => process_wire_frame(config, signer, &frame.payload),
                };
                if let Ok(message) = response {
                    match encode_cultnet_message_to_vec(
                        &message,
                        CultNetWireContract::CultNetSchemaV0,
                    ) {
                        Ok(payload) => {
                            if let Err(error) = transport.send("schema", payload) {
                                eprintln!("operator service reply transport error: {error:#}");
                            }
                        }
                        Err(error) => eprintln!("operator service reply encoding error: {error:#}"),
                    }
                }
            }
            Ok(None) => {}
            Err(error) => {
                eprintln!("operator service discarded hostile transport frame: {error:#}")
            } // Malformed, foreign, unsigned, expired-new, and capability-invalid
              // admissions receive no signed application reply. A receipt would
              // falsely turn unauthenticated material into an Epiphany statement.
              // Authenticated owner precondition failures are instead returned by
              // the core as sealed `refused` results.
        }
        if let Err(error) = transport.poll_resends() {
            eprintln!("operator service resend transport error: {error:#}");
        }
    }
}

fn process_wire_frame(
    config: &OperatorCommandServiceConfig,
    signer: &HostIdentitySigner,
    payload: &[u8],
) -> Result<CultNetMessage> {
    process_wire_frame_at(config, signer, payload, &chrono::Utc::now().to_rfc3339())
}

fn process_wire_frame_at(
    config: &OperatorCommandServiceConfig,
    signer: &HostIdentitySigner,
    payload: &[u8],
    now: &str,
) -> Result<CultNetMessage> {
    let message = decode_cultnet_message_from_slice(payload, CultNetWireContract::CultNetSchemaV0)
        .context("operator service rejected non-CultNet payload")?;
    let CultNetMessage::DocumentPutRaw { document, .. } = message else {
        bail!("operator service accepts only one raw typed admission document");
    };
    if document.schema_id != BIFROST_OPERATOR_COMMAND_DELIVERY_TYPE
        || document.payload_encoding != CultNetRawPayloadEncoding::Messagepack
        || document.source_role.as_deref() != Some("bifrost-operator-admission")
    {
        bail!("operator service rejected foreign wire document authority");
    }
    let admission: BifrostOperatorCommandAdmission =
        rmp_serde::from_slice(&document.payload).context("operator wire admission is malformed")?;
    if document.record_key != admission.admission_id
        || document.source_runtime_id.as_deref()
            != Some(admission.source_observer_runtime_id.as_str())
    {
        bail!("operator wire envelope substituted admission identity");
    }
    let receipt = execute_operator_command_admission(config, signer, &document.payload, now)?;
    Ok(CultNetMessage::DocumentPutRaw {
        message_id: format!("operator-result-{}", receipt.command_id),
        document: CultNetRawDocumentRecord {
            schema_id: EPIPHANY_OPERATOR_COMMAND_RESULT_RECEIPT_TYPE.into(),
            record_key: receipt.command_id.clone(),
            stored_at: receipt.completed_at.clone(),
            payload_encoding: CultNetRawPayloadEncoding::Messagepack,
            payload: rmp_serde::to_vec_named(&receipt)?,
            source_runtime_id: Some(config.policy.runtime_id.clone()),
            source_agent_id: Some(receipt.provider_identity_id.clone()),
            source_role: Some("epiphany-operator-command-executor".into()),
            tags: Some(vec!["cultnet.transport.rudp.v0".into()]),
        },
    })
}

pub fn read_operator_command_trust_anchor(path: &Path) -> Result<HostIdentityTrustAnchorEntry> {
    // This is the Bifrost-owned boundary artifact: one raw compact MessagePack
    // six-tuple. Epiphany's canonical exported `.cc` anchor is a CultCache
    // envelope and intentionally is not accepted by this ingress reader.
    rmp_serde::from_slice(&std::fs::read(path)?)
        .map_err(|error| anyhow!("operator Bifrost trust anchor is malformed: {error}"))
}

pub fn write_operator_command_interop_fixture(
    output: &Path,
) -> Result<OperatorCommandInteropFixtureManifest> {
    std::fs::create_dir_all(output)?;
    let private = output.join(".fixture-private");
    if private.exists() {
        bail!("operator interop fixture private workspace already exists");
    }
    std::fs::create_dir(&private)?;
    let generated = (|| -> Result<OperatorCommandInteropFixtureManifest> {
        let bifrost = crate::enroll_host_identity_at(&private.join("bifrost.cc"))?;
        let executor = crate::enroll_host_identity_at(&private.join("executor.cc"))?;
        let bifrost_anchor =
            crate::export_host_identity_trust_anchor(&bifrost, &private.join("bifrost-anchor.cc"))?;
        let executor_anchor = crate::export_host_identity_trust_anchor(
            &executor,
            &output.join("executor-anchor.cc"),
        )?;
        let config = OperatorCommandServiceConfig {
            command_store: private.join("commands.cc"),
            local_verse_store: private.join("verse.cc"),
            resident_self_store: private.join("resident.cc"),
            runtime_store: private.join("runtime.cc"),
            policy: OperatorCommandPolicy {
                runtime_id: "epiphany-interop-fixture".into(),
                discord_guild_id: "fixture-guild".into(),
                allowed_channel_ids: vec!["fixture-ops".into()],
                actor_capabilities: std::collections::BTreeMap::from([(
                    "fixture-actor".into(),
                    vec![crate::OperatorCapability::Sleep],
                )]),
                max_ttl_seconds: 60,
            },
            trusted_bifrost_identity: bifrost_anchor.clone(),
        };
        let packet = crate::OperatorCommandPacket {
            command_id: "fixture-command-1".into(),
            nonce: "fixture-nonce-1".into(),
            source_event_id: "fixture-event-1".into(),
            source_actor_id: "fixture-actor".into(),
            discord_guild_id: "fixture-guild".into(),
            discord_channel_id: "fixture-ops".into(),
            discord_message_id: "fixture-message-1".into(),
            target_runtime_id: "epiphany-interop-fixture".into(),
            issued_at: "2026-07-19T12:00:00Z".into(),
            expires_at: "2026-07-19T12:01:00Z".into(),
            command: crate::OperatorCommand::Sleep {
                reason: "Interop fixture".into(),
            },
        };
        let mut admission = BifrostOperatorCommandAdmission {
            schema_name: BIFROST_OPERATOR_COMMAND_DELIVERY_TYPE.into(),
            schema_version: crate::BIFROST_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION.into(),
            admission_id: "fixture-admission-1".into(),
            packet_sha256: crate::operator_command_packet_sha256(&packet)?,
            packet,
            source_observer_id: "voidbot".into(),
            source_observer_runtime_id: "fixture-bifrost".into(),
            provider: "bifrost".into(),
            bifrost_admission_receipt_id: "fixture-bifrost-receipt-1".into(),
            authority: "exact_operator_command_only".into(),
            provider_identity_id: bifrost.entry().identity_id.clone(),
            provider_signature: Vec::new(),
        };
        admission.provider_signature = bifrost
            .sign(
                crate::operator_command_admission_signing_purpose(),
                &crate::operator_command_admission_signing_payload(&admission)?,
            )?
            .signature;
        let admission_bytes = rmp_serde::to_vec_named(&admission)?;
        let receipt = execute_operator_command_admission(
            &config,
            &executor,
            &admission_bytes,
            "2026-07-19T12:00:01Z",
        )?;
        let receipt_bytes = rmp_serde::to_vec_named(&receipt)?;
        std::fs::write(output.join("operator-admission.msgpack"), &admission_bytes)?;
        std::fs::write(
            output.join("bifrost-anchor.msgpack"),
            rmp_serde::to_vec(&bifrost_anchor)?,
        )?;
        std::fs::write(
            output.join("executor-anchor.msgpack"),
            rmp_serde::to_vec(&executor_anchor)?,
        )?;
        std::fs::write(output.join("sealed-result.msgpack"), &receipt_bytes)?;
        let manifest = OperatorCommandInteropFixtureManifest {
            schema_version: "epiphany.operator_command.interop_fixture.v0".into(),
            admission_file: "operator-admission.msgpack".into(),
            bifrost_raw_trust_anchor_file: "bifrost-anchor.msgpack".into(),
            executor_raw_trust_anchor_file: "executor-anchor.msgpack".into(),
            executor_cultcache_trust_anchor_file: "executor-anchor.cc".into(),
            sealed_result_file: "sealed-result.msgpack".into(),
            admission_sha256: format!("sha256-{:x}", Sha256::digest(&admission_bytes)),
            sealed_result_sha256: format!("sha256-{:x}", Sha256::digest(&receipt_bytes)),
            admission_signing_purpose: crate::operator_command_admission_signing_purpose().into(),
            result_signing_purpose: RESULT_SIGNING_PURPOSE.into(),
            rudp_connection_id: EPIPHANY_OPERATOR_COMMAND_RUDP_CONNECTION_ID,
            private_state_exposed: false,
        };
        std::fs::write(
            output.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest)?,
        )?;
        Ok(manifest)
    })();
    let cleanup = std::fs::remove_dir_all(&private);
    generated.and_then(|manifest| {
        cleanup.context("failed to remove interop fixture private signing material")?;
        Ok(manifest)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OperatorCapability, OperatorCommand, OperatorCommandPacket};
    use cultcache_rs::{CacheBackingStore, SingleFileMessagePackBackingStore};
    use std::collections::BTreeMap;
    use std::thread;
    use std::time::Instant;

    fn fixture(
        root: &Path,
    ) -> Result<(
        OperatorCommandServiceConfig,
        HostIdentitySigner,
        HostIdentitySigner,
    )> {
        let bifrost = crate::enroll_host_identity_at(&root.join("bifrost.cc"))?;
        let anchor = crate::export_host_identity_trust_anchor(&bifrost, &root.join("anchor.cc"))?;
        let executor = crate::enroll_host_identity_at(&root.join("executor.cc"))?;
        Ok((
            OperatorCommandServiceConfig {
                command_store: root.join("commands.cc"),
                local_verse_store: root.join("verse.cc"),
                resident_self_store: root.join("resident.cc"),
                runtime_store: root.join("runtime.cc"),
                policy: OperatorCommandPolicy {
                    runtime_id: "epiphany-yggdrasil".into(),
                    discord_guild_id: "guild".into(),
                    allowed_channel_ids: vec!["ops".into()],
                    actor_capabilities: BTreeMap::from([(
                        "actor".into(),
                        vec![OperatorCapability::Sleep],
                    )]),
                    max_ttl_seconds: 60,
                },
                trusted_bifrost_identity: anchor,
            },
            executor,
            bifrost,
        ))
    }

    fn admission(signer: &HostIdentitySigner) -> Result<BifrostOperatorCommandAdmission> {
        let packet = OperatorCommandPacket {
            command_id: "command-1".into(),
            nonce: "nonce-1".into(),
            source_event_id: "event-1".into(),
            source_actor_id: "actor".into(),
            discord_guild_id: "guild".into(),
            discord_channel_id: "ops".into(),
            discord_message_id: "message-1".into(),
            target_runtime_id: "epiphany-yggdrasil".into(),
            issued_at: "2026-07-19T12:00:00Z".into(),
            expires_at: "2026-07-19T12:01:00Z".into(),
            command: OperatorCommand::Sleep {
                reason: "Supervised sleep.".into(),
            },
        };
        let mut value = BifrostOperatorCommandAdmission {
            schema_name: BIFROST_OPERATOR_COMMAND_DELIVERY_TYPE.into(),
            schema_version: crate::BIFROST_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION.into(),
            admission_id: "admission-1".into(),
            packet_sha256: crate::operator_command_packet_sha256(&packet)?,
            packet,
            source_observer_id: "voidbot".into(),
            source_observer_runtime_id: "voidbot-yggdrasil".into(),
            provider: "bifrost".into(),
            bifrost_admission_receipt_id: "receipt-1".into(),
            authority: "exact_operator_command_only".into(),
            provider_identity_id: signer.entry().identity_id.clone(),
            provider_signature: Vec::new(),
        };
        value.provider_signature = signer
            .sign(
                crate::operator_command_admission_signing_purpose(),
                &crate::operator_command_admission_signing_payload(&value)?,
            )?
            .signature;
        Ok(value)
    }

    #[test]
    fn seals_exact_result_and_replays_same_receipt() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let (config, executor, bifrost) = fixture(temp.path())?;
        let payload = rmp_serde::to_vec_named(&admission(&bifrost)?)?;
        let first = execute_operator_command_admission(
            &config,
            &executor,
            &payload,
            "2026-07-19T12:00:01Z",
        )?;
        let reopened_executor = crate::open_host_identity_at(&temp.path().join("executor.cc"))?;
        let replay = execute_operator_command_admission(
            &config,
            &reopened_executor,
            &payload,
            "2026-07-19T12:02:00Z",
        )?;
        assert_eq!(first, replay);
        assert!(!first.private_state_exposed);
        crate::verify_host_identity_signature(
            executor.entry(),
            RESULT_SIGNING_PURPOSE,
            &operator_command_result_receipt_signing_payload(&first)?,
            &crate::HostIdentitySignature {
                identity_id: first.provider_identity_id.clone(),
                signature: first.executor_signature.clone(),
            },
        )?;
        Ok(())
    }

    #[test]
    fn strict_wire_rejects_tamper_before_command_store_write() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let (config, executor, bifrost) = fixture(temp.path())?;
        let mut value = serde_json::to_value(admission(&bifrost)?)?;
        value
            .as_object_mut()
            .unwrap()
            .insert("argv".into(), serde_json::json!(["whoami"]));
        let payload = rmp_serde::to_vec_named(&value)?;
        assert!(
            execute_operator_command_admission(
                &config,
                &executor,
                &payload,
                "2026-07-19T12:00:01Z"
            )
            .is_err()
        );
        assert!(!config.command_store.exists());
        Ok(())
    }

    #[test]
    fn cultnet_wire_accepts_only_exact_typed_admission_and_returns_sealed_receipt() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let (config, executor, bifrost) = fixture(temp.path())?;
        let admission = admission(&bifrost)?;
        let request = CultNetMessage::DocumentPutRaw {
            message_id: "request-1".into(),
            document: CultNetRawDocumentRecord {
                schema_id: BIFROST_OPERATOR_COMMAND_DELIVERY_TYPE.into(),
                record_key: admission.admission_id.clone(),
                stored_at: admission.packet.issued_at.clone(),
                payload_encoding: CultNetRawPayloadEncoding::Messagepack,
                payload: rmp_serde::to_vec_named(&admission)?,
                source_runtime_id: Some(admission.source_observer_runtime_id.clone()),
                source_agent_id: Some(admission.provider_identity_id.clone()),
                source_role: Some("bifrost-operator-admission".into()),
                tags: Some(vec!["cultnet.transport.rudp.v0".into()]),
            },
        };
        let encoded =
            encode_cultnet_message_to_vec(&request, CultNetWireContract::CultNetSchemaV0)?;
        let response = process_wire_frame_at(&config, &executor, &encoded, "2026-07-19T12:00:01Z")?;
        let CultNetMessage::DocumentPutRaw { document, .. } = response else {
            panic!("non-document response")
        };
        assert_eq!(
            document.schema_id,
            EPIPHANY_OPERATOR_COMMAND_RESULT_RECEIPT_TYPE
        );
        let receipt: EpiphanyOperatorCommandResultReceipt =
            rmp_serde::from_slice(&document.payload)?;
        assert_eq!(receipt.provider_identity_id, executor.entry().identity_id);
        assert_eq!(receipt.command_id, admission.packet.command_id);
        assert_eq!(receipt.packet_sha256, admission.packet_sha256);
        assert!(!receipt.private_state_exposed);

        let mut alien = request;
        let CultNetMessage::DocumentPutRaw { document, .. } = &mut alien else {
            unreachable!()
        };
        document.source_role = Some("persona-feedback".into());
        let before = std::fs::read(&config.command_store)?;
        assert!(
            process_wire_frame_at(
                &config,
                &executor,
                &encode_cultnet_message_to_vec(&alien, CultNetWireContract::CultNetSchemaV0)?,
                "2026-07-19T12:00:02Z"
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&config.command_store)?, before);
        Ok(())
    }

    #[test]
    fn valid_command_succeeds_after_hostile_udp_on_same_service_instance() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let (config, executor, bifrost) = fixture(temp.path())?;
        let admission = admission(&bifrost)?;
        let socket = UdpSocket::bind("127.0.0.1:0")?;
        socket.set_read_timeout(Some(Duration::from_millis(20)))?;
        let target = socket.local_addr()?;
        let attacker = UdpSocket::bind("127.0.0.1:0")?;
        attacker.send_to(b"not-cultnet-rudp", target)?;
        let server = thread::spawn(move || -> Result<()> {
            let mut transport = CultNetRudpSocketTransportConnection::new(
                CultNetRudpSocketTransportOptions::server(
                    &config.policy.runtime_id,
                    socket,
                    EPIPHANY_OPERATOR_COMMAND_RUDP_CONNECTION_ID,
                ),
            )?;
            serve_operator_command_rudp_loop(
                &mut transport,
                &config,
                &executor,
                Some(100),
                Some("2026-07-19T12:00:01Z"),
            )
        });

        let request = CultNetMessage::DocumentPutRaw {
            message_id: "post-hostile-request".into(),
            document: CultNetRawDocumentRecord {
                schema_id: BIFROST_OPERATOR_COMMAND_DELIVERY_TYPE.into(),
                record_key: admission.admission_id.clone(),
                stored_at: admission.packet.issued_at.clone(),
                payload_encoding: CultNetRawPayloadEncoding::Messagepack,
                payload: rmp_serde::to_vec_named(&admission)?,
                source_runtime_id: Some(admission.source_observer_runtime_id.clone()),
                source_agent_id: Some(admission.provider_identity_id.clone()),
                source_role: Some("bifrost-operator-admission".into()),
                tags: Some(vec!["cultnet.transport.rudp.v0".into()]),
            },
        };
        let client_socket = UdpSocket::bind("127.0.0.1:0")?;
        client_socket.set_read_timeout(Some(Duration::from_millis(20)))?;
        let mut client =
            CultNetRudpSocketTransportConnection::new(CultNetRudpSocketTransportOptions::client(
                "fixture-bifrost",
                client_socket,
                target,
                EPIPHANY_OPERATOR_COMMAND_RUDP_CONNECTION_ID,
            ))?;
        client.connect(Vec::new())?;
        let deadline = Instant::now() + Duration::from_secs(2);
        while !client.connected() {
            let _ = client.receive_once()?;
            client.poll_resends()?;
            if Instant::now() >= deadline {
                bail!("post-hostile client handshake timed out");
            }
        }
        client.send(
            "schema",
            encode_cultnet_message_to_vec(&request, CultNetWireContract::CultNetSchemaV0)?,
        )?;
        let deadline = Instant::now() + Duration::from_secs(2);
        let response = loop {
            if let Some(frame) = client.receive_once()? {
                break decode_cultnet_message_from_slice(
                    &frame.payload,
                    CultNetWireContract::CultNetSchemaV0,
                )?;
            }
            client.poll_resends()?;
            if Instant::now() >= deadline {
                bail!("post-hostile sealed response timed out");
            }
        };
        let CultNetMessage::DocumentPutRaw { document, .. } = response else {
            bail!("post-hostile service returned non-document response");
        };
        assert_eq!(
            document.schema_id,
            EPIPHANY_OPERATOR_COMMAND_RESULT_RECEIPT_TYPE
        );
        let receipt: EpiphanyOperatorCommandResultReceipt =
            rmp_serde::from_slice(&document.payload)?;
        assert_eq!(receipt.command_id, admission.packet.command_id);
        assert_eq!(receipt.packet_sha256, admission.packet_sha256);
        server
            .join()
            .map_err(|_| anyhow!("operator service test thread panicked"))??;
        Ok(())
    }

    #[test]
    fn rust_fixture_contains_verifiable_bifrost_and_executor_bytes() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let manifest = write_operator_command_interop_fixture(temp.path())?;
        assert_eq!(
            manifest.rudp_connection_id,
            EPIPHANY_OPERATOR_COMMAND_RUDP_CONNECTION_ID
        );
        assert!(!temp.path().join(".fixture-private").exists());

        let admission_bytes = std::fs::read(temp.path().join(&manifest.admission_file))?;
        let admission: BifrostOperatorCommandAdmission = rmp_serde::from_slice(&admission_bytes)?;
        let bifrost_anchor: HostIdentityTrustAnchorEntry = rmp_serde::from_slice(&std::fs::read(
            temp.path().join(&manifest.bifrost_raw_trust_anchor_file),
        )?)?;
        crate::verify_host_identity_trust_anchor_signature(
            &bifrost_anchor,
            crate::operator_command_admission_signing_purpose(),
            &crate::operator_command_admission_signing_payload(&admission)?,
            &crate::HostIdentitySignature {
                identity_id: admission.provider_identity_id.clone(),
                signature: admission.provider_signature.clone(),
            },
        )?;

        let receipt_bytes = std::fs::read(temp.path().join(&manifest.sealed_result_file))?;
        let receipt: EpiphanyOperatorCommandResultReceipt = rmp_serde::from_slice(&receipt_bytes)?;
        let executor_anchor: HostIdentityTrustAnchorEntry = rmp_serde::from_slice(&std::fs::read(
            temp.path().join(&manifest.executor_raw_trust_anchor_file),
        )?)?;
        let envelopes = SingleFileMessagePackBackingStore::new(
            &temp
                .path()
                .join(&manifest.executor_cultcache_trust_anchor_file),
        )
        .pull_all()?;
        assert_eq!(envelopes.len(), 1);
        let canonical_anchor: HostIdentityTrustAnchorEntry =
            rmp_serde::from_slice(&envelopes[0].payload)?;
        assert_eq!(canonical_anchor, executor_anchor);
        crate::verify_host_identity_trust_anchor_signature(
            &executor_anchor,
            RESULT_SIGNING_PURPOSE,
            &operator_command_result_receipt_signing_payload(&receipt)?,
            &crate::HostIdentitySignature {
                identity_id: receipt.provider_identity_id.clone(),
                signature: receipt.executor_signature.clone(),
            },
        )?;
        let wire_result = rmp_serde::to_vec_named(&receipt.result)?;
        assert_eq!(
            receipt.result_payload_sha256,
            format!("sha256-{:x}", Sha256::digest(wire_result))
        );
        Ok(())
    }
}
