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
    "epiphany.operator_command.sealed_result.v0";
const RESULT_SIGNING_PURPOSE: &str = "epiphany.operator-command.sealed-result.v0";
const RUDP_CONNECTION_ID: u32 = 0xe91f_0001;

#[derive(Clone, Debug)]
pub struct OperatorCommandServiceConfig {
    pub command_store: PathBuf,
    pub local_verse_store: PathBuf,
    pub resident_self_store: PathBuf,
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
            RUDP_CONNECTION_ID,
        ))?;
    loop {
        if let Some(frame) = transport.receive_once()? {
            let response = process_wire_frame(config, signer, &frame.payload);
            if let Ok(message) = response {
                transport.send(
                    "schema",
                    encode_cultnet_message_to_vec(&message, CultNetWireContract::CultNetSchemaV0)?,
                )?;
            }
            // Malformed, foreign, unsigned, expired-new, and capability-invalid
            // admissions receive no signed application reply. A receipt would
            // falsely turn unauthenticated material into an Epiphany statement.
            // Authenticated owner precondition failures are instead returned by
            // the core as sealed `refused` results.
        }
        transport.poll_resends()?;
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
    rmp_serde::from_slice(&std::fs::read(path)?)
        .map_err(|error| anyhow!("operator Bifrost trust anchor is malformed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OperatorCapability, OperatorCommand, OperatorCommandPacket};
    use std::collections::BTreeMap;

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
}
