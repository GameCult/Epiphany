use anyhow::{Context, Result, anyhow, bail};
use chrono::Utc;
use cultcache_rs::{
    CacheBackingStore, CultCache, DatabaseEntry, SingleFileMessagePackBackingStore,
};
use cultnet_rs::{
    CultNetMessage, CultNetRawDocumentRecord, CultNetRawPayloadEncoding,
    CultNetRudpSocketTransportConnection, CultNetRudpSocketTransportOptions, CultNetWireContract,
    GameCultServiceTrustAnchorRecord, decode_cultnet_message_from_slice,
    encode_cultnet_message_to_vec, query_read_only_raw_snapshot,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::net::{SocketAddr, UdpSocket};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::{
    BIFROST_OPERATOR_COMMAND_DELIVERY_TYPE, BifrostOperatorCommandAdmission, HostIdentitySigner,
    HostIdentityTrustAnchorEntry, IdunnProviderHealthAdmission, OperatorCommand,
    OperatorCommandPolicy, OperatorCommandResult, ProviderReleaseBinding, RequiredProviderHealth,
    admit_and_execute_bifrost_operator_command, admit_required_idunn_provider_health,
    load_resident_self_state, pending_repo_frontier_plan_reviews,
    read_idunn_provider_health_trust_anchor, required_idunn_provider_health_query,
    resident_self_pressures, verify_idunn_provider_health_candidate,
};

pub const EPIPHANY_OPERATOR_COMMAND_RESULT_RECEIPT_TYPE: &str =
    "epiphany.operator_command.sealed_result";
pub const EPIPHANY_OPERATOR_COMMAND_RESULT_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.operator_command.sealed_result.v1";
const RESULT_SIGNING_PURPOSE: &str = "epiphany.operator-command.sealed-result.v1";
pub const EPIPHANY_OPERATOR_COMMAND_RUDP_CONNECTION_ID: u32 = 0xe91f_0001;
pub const EPIPHANY_OPERATOR_COMMAND_SERVICE_HEALTH_SCHEMA_VERSION: &str =
    "epiphany.operator_command.service_health.v0";
const SERVICE_HEALTH_KEY: &str = "operator-command-service";
const SERVICE_HEALTH_SIGNING_PURPOSE: &str = "epiphany.operator-command.service-health.v0";
const IDUNN_PUBLIC_HEALTH_QUERY_CONNECTION_ID: u32 = 0x1d0d_0002;

#[derive(Clone, Debug)]
pub struct OperatorCommandServiceConfig {
    pub command_store: PathBuf,
    pub local_verse_store: PathBuf,
    pub resident_self_store: PathBuf,
    pub runtime_store: PathBuf,
    pub policy: OperatorCommandPolicy,
    pub trusted_bifrost_identity: HostIdentityTrustAnchorEntry,
    pub provider_health: OperatorStatusProviderHealthConfig,
}

#[derive(Clone, Debug)]
pub struct OperatorStatusProviderHealthConfig {
    pub query_endpoint: SocketAddr,
    pub idunn_runtime_id: String,
    pub trust_anchor_store: PathBuf,
    pub admission_store: PathBuf,
    pub max_local_age_millis: u64,
    pub deployment_id: String,
    pub release_id: String,
    pub release_witness_sha256: String,
    pub source_commit: String,
}

#[derive(Clone, Debug)]
pub struct OperatorCommandServiceHealthConfig {
    pub store: PathBuf,
    pub bind: String,
    pub release_id: String,
    pub release_witness_sha256: String,
    pub source_commit: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.operator_command.service_health",
    schema = "EpiphanyOperatorCommandServiceHealth"
)]
pub struct EpiphanyOperatorCommandServiceHealth {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub release_id: String,
    #[cultcache(key = 3)]
    pub release_witness_sha256: String,
    #[cultcache(key = 4)]
    pub source_commit: String,
    #[cultcache(key = 5)]
    pub bind: String,
    #[cultcache(key = 6)]
    pub config_sha256: String,
    #[cultcache(key = 7)]
    pub bifrost_identity_id: String,
    #[cultcache(key = 8)]
    pub executor_identity_id: String,
    #[cultcache(key = 9)]
    pub observed_at: String,
    #[cultcache(key = 10)]
    pub observed_at_millis: u64,
    #[cultcache(key = 11)]
    pub process_id: u32,
    #[cultcache(key = 12)]
    pub process_creation_token: u64,
    #[cultcache(key = 13)]
    pub process_executable_path: String,
    #[cultcache(key = 14)]
    pub private_state_exposed: bool,
    #[cultcache(key = 15)]
    pub executor_signature: Vec<u8>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_v2: Option<EpiphanyOperatorStatusV2>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EpiphanyOperatorStatusProviderV2 {
    pub daemon_id: String,
    pub health_contract: String,
    pub availability: String,
    pub unavailable_reason: String,
    pub state: Option<String>,
    pub reason_code: Option<String>,
    pub provider_observed_at_unix_millis: Option<u64>,
    pub evaluated_at_unix_millis: Option<u64>,
    pub expires_at_unix_millis: Option<u64>,
    pub release_id: Option<String>,
    pub release_witness_sha256: Option<String>,
    pub source_commit: Option<String>,
    pub deployment_id: Option<String>,
    pub projection_sha256: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EpiphanyOperatorStatusV2 {
    pub schema_version: String,
    pub release_id: String,
    pub release_witness_sha256: String,
    pub source_commit: String,
    pub deployment_id: String,
    pub coordinator_snapshot: String,
    pub coordinator_state: String,
    pub coordinator_action: String,
    pub brake_status: String,
    pub resident_status: String,
    pub pressure_count: usize,
    pub pending_review_count: usize,
    pub provider_set_status: String,
    pub providers: Vec<EpiphanyOperatorStatusProviderV2>,
    pub private_state_exposed: bool,
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
            status_v2: None,
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
    pub protocol_file: String,
    pub admission_sha256: String,
    pub sealed_result_sha256: String,
    pub protocol_sha256: String,
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
    let mut wire_result = EpiphanyOperatorCommandWireResult::from(&result);
    if matches!(admission.packet.command, OperatorCommand::Status) {
        wire_result.schema_version = "epiphany.operator_command.status_result.v2".into();
        wire_result.status_v2 = Some(build_operator_status_v2(config, &result)?);
    }
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

fn required_status_providers(
    config: &OperatorStatusProviderHealthConfig,
) -> Vec<RequiredProviderHealth> {
    vec![
        RequiredProviderHealth {
            daemon_id: "yggdrasil-epiphany".into(),
            health_contract: crate::EPIPHANY_IDUNN_RUNTIME_HEALTH_CONTRACT.into(),
            release_binding: ProviderReleaseBinding::Exact {
                release_id: config.release_id.clone(),
                release_witness_sha256: config.release_witness_sha256.clone(),
                source_commit: config.source_commit.clone(),
                deployment_id: config.deployment_id.clone(),
            },
        },
        RequiredProviderHealth {
            daemon_id: "yggdrasil-bifrost-persona-feedback".into(),
            health_contract: "bifrost.cultnet-rudp-persona-feedback-health".into(),
            release_binding: ProviderReleaseBinding::Forbidden,
        },
    ]
}

fn build_operator_status_v2(
    config: &OperatorCommandServiceConfig,
    result: &OperatorCommandResult,
) -> Result<EpiphanyOperatorStatusV2> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .try_into()?;
    let required = required_status_providers(&config.provider_health);
    let (providers, complete) = match query_and_admit_status_providers(config, &required, now) {
        Ok(value) => value,
        Err(_) => (
            required
                .iter()
                .map(|item| {
                    unavailable_provider(item, "authenticated-provider-snapshot-unavailable")
                })
                .collect(),
            false,
        ),
    };
    let resident = load_resident_self_state(&config.resident_self_store)?;
    Ok(EpiphanyOperatorStatusV2 {
        schema_version: "epiphany.operator.status.v2".into(),
        release_id: config.provider_health.release_id.clone(),
        release_witness_sha256: config.provider_health.release_witness_sha256.clone(),
        source_commit: config.provider_health.source_commit.clone(),
        deployment_id: config.provider_health.deployment_id.clone(),
        coordinator_snapshot: result.operator_status.clone(),
        coordinator_state: result.state_status.clone(),
        coordinator_action: result.coordinator_action.clone(),
        brake_status: result.brake_status.clone(),
        resident_status: if resident.active_turn.is_some() {
            "resident-self/running"
        } else {
            "resident-self/idle"
        }
        .into(),
        pressure_count: resident_self_pressures(&config.resident_self_store)?.len(),
        pending_review_count: pending_repo_frontier_plan_reviews(&config.runtime_store, 25)?.len(),
        provider_set_status: if complete {
            "complete-authenticated"
        } else {
            "incomplete"
        }
        .into(),
        providers,
        private_state_exposed: false,
    })
}

fn query_and_admit_status_providers(
    config: &OperatorCommandServiceConfig,
    required: &[RequiredProviderHealth],
    now: u64,
) -> Result<(Vec<EpiphanyOperatorStatusProviderV2>, bool)> {
    let anchor: GameCultServiceTrustAnchorRecord =
        read_idunn_provider_health_trust_anchor(&config.provider_health.trust_anchor_store)?;
    let query = required_idunn_provider_health_query(
        format!("operator-status-{now}"),
        required,
        &config.provider_health.idunn_runtime_id,
    )?;
    let records = query_read_only_raw_snapshot(&query, |request| {
        exchange_idunn_public_snapshot(
            config.provider_health.query_endpoint,
            &config.policy.runtime_id,
            request,
        )
    })?;
    let mut projected = Vec::with_capacity(required.len());
    let mut verified_records = Vec::with_capacity(required.len());
    for requirement in required {
        let Some(record) = records
            .iter()
            .find(|record| record.record_key == requirement.record_key())
        else {
            projected.push(unavailable_provider(
                requirement,
                "required-provider-record-missing",
            ));
            continue;
        };
        match verify_idunn_provider_health_candidate(
            requirement,
            record,
            &anchor,
            &config.provider_health.idunn_runtime_id,
            now,
            config.provider_health.max_local_age_millis,
        ) {
            Ok(value) => {
                projected.push(available_provider(&value));
                verified_records.push(record.clone());
            }
            Err(_) => projected.push(unavailable_provider(
                requirement,
                "required-provider-record-invalid",
            )),
        }
    }
    let complete = verified_records.len() == required.len();
    if complete {
        admit_required_idunn_provider_health(
            &config.provider_health.admission_store,
            required,
            &verified_records,
            &anchor,
            &config.provider_health.idunn_runtime_id,
            now,
            config.provider_health.max_local_age_millis,
        )?;
    }
    Ok((projected, complete))
}

fn exchange_idunn_public_snapshot(
    endpoint: SocketAddr,
    runtime_id: &str,
    request: CultNetMessage,
) -> Result<CultNetMessage> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    let mut transport =
        CultNetRudpSocketTransportConnection::new(CultNetRudpSocketTransportOptions::client(
            runtime_id,
            socket,
            endpoint,
            IDUNN_PUBLIC_HEALTH_QUERY_CONNECTION_ID,
        ))?;
    transport.connect(Vec::new())?;
    let deadline = Instant::now() + Duration::from_secs(2);
    while !transport.connected() {
        let _ = transport.receive_once()?;
        transport.poll_resends()?;
        if Instant::now() >= deadline {
            bail!("Idunn public provider-health handshake unavailable");
        }
    }
    transport.send(
        "schema",
        encode_cultnet_message_to_vec(&request, CultNetWireContract::CultNetSchemaV0)?,
    )?;
    loop {
        if let Some(frame) = transport.receive_once()? {
            if frame.channel_id != "schema" {
                bail!("Idunn public provider-health response channel is invalid");
            }
            return decode_cultnet_message_from_slice(
                &frame.payload,
                CultNetWireContract::CultNetSchemaV0,
            );
        }
        transport.poll_resends()?;
        if Instant::now() >= deadline {
            bail!("Idunn public provider-health response unavailable");
        }
    }
}

fn available_provider(value: &IdunnProviderHealthAdmission) -> EpiphanyOperatorStatusProviderV2 {
    EpiphanyOperatorStatusProviderV2 {
        daemon_id: value.daemon_id.clone(),
        health_contract: value.health_contract.clone(),
        availability: "authenticated-current".into(),
        unavailable_reason: String::new(),
        state: Some(value.provider_state.clone()),
        reason_code: Some(value.reason_code.clone()),
        provider_observed_at_unix_millis: Some(value.provider_observed_at_unix_millis),
        evaluated_at_unix_millis: Some(value.evaluated_at_unix_millis),
        expires_at_unix_millis: Some(value.expires_at_unix_millis),
        release_id: value.release_id.clone(),
        release_witness_sha256: value.release_witness_sha256.clone(),
        source_commit: value.source_commit.clone(),
        deployment_id: value.deployment_id.clone(),
        projection_sha256: Some(value.projection_sha256.clone()),
    }
}

fn unavailable_provider(
    value: &RequiredProviderHealth,
    reason: &str,
) -> EpiphanyOperatorStatusProviderV2 {
    EpiphanyOperatorStatusProviderV2 {
        daemon_id: value.daemon_id.clone(),
        health_contract: value.health_contract.clone(),
        availability: "unavailable".into(),
        unavailable_reason: reason.into(),
        state: None,
        reason_code: None,
        provider_observed_at_unix_millis: None,
        evaluated_at_unix_millis: None,
        expires_at_unix_millis: None,
        release_id: None,
        release_witness_sha256: None,
        source_commit: None,
        deployment_id: None,
        projection_sha256: None,
    }
}

fn operator_status_v2_migration_fixture() -> EpiphanyOperatorStatusV2 {
    let requirements = required_status_providers(&OperatorStatusProviderHealthConfig {
        query_endpoint: "127.0.0.1:1".parse().expect("fixed fixture endpoint"),
        idunn_runtime_id: "idunn-yggdrasil".into(),
        trust_anchor_store: PathBuf::new(),
        admission_store: PathBuf::new(),
        max_local_age_millis: 30_000,
        deployment_id: "deployment-fixture".into(),
        release_id: "release-fixture".into(),
        release_witness_sha256: format!("sha256-{}", "a".repeat(64)),
        source_commit: "b".repeat(40),
    });
    EpiphanyOperatorStatusV2 {
        schema_version: "epiphany.operator.status.v2".into(),
        release_id: "release-fixture".into(),
        release_witness_sha256: format!("sha256-{}", "a".repeat(64)),
        source_commit: "b".repeat(40),
        deployment_id: "deployment-fixture".into(),
        coordinator_snapshot: "coordinator-snapshot/sleeping".into(),
        coordinator_state: "coordinator-state/ready".into(),
        coordinator_action: "coordinator-action/none".into(),
        brake_status: "engaged".into(),
        resident_status: "resident-self/idle".into(),
        pressure_count: 0,
        pending_review_count: 0,
        provider_set_status: "incomplete".into(),
        providers: requirements
            .iter()
            .map(|value| unavailable_provider(value, "required-provider-record-missing"))
            .collect(),
        private_state_exposed: false,
    }
}

pub fn serve_operator_command_rudp(
    socket: UdpSocket,
    config: &OperatorCommandServiceConfig,
    signer: &HostIdentitySigner,
    health: &OperatorCommandServiceHealthConfig,
) -> Result<()> {
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    let mut transport =
        CultNetRudpSocketTransportConnection::new(CultNetRudpSocketTransportOptions::server(
            &config.policy.runtime_id,
            socket,
            EPIPHANY_OPERATOR_COMMAND_RUDP_CONNECTION_ID,
        ))?;
    serve_operator_command_rudp_loop(&mut transport, config, signer, Some(health), None, None)
}

fn serve_operator_command_rudp_loop(
    transport: &mut CultNetRudpSocketTransportConnection,
    config: &OperatorCommandServiceConfig,
    signer: &HostIdentitySigner,
    health: Option<&OperatorCommandServiceHealthConfig>,
    iteration_limit: Option<usize>,
    fixed_now: Option<&str>,
) -> Result<()> {
    let mut iterations = 0usize;
    let mut last_health = None::<Instant>;
    loop {
        if iteration_limit.is_some_and(|limit| iterations >= limit) {
            return Ok(());
        }
        iterations += 1;
        if let Some(health) = health {
            if last_health.is_none_or(|at| at.elapsed() >= Duration::from_secs(5)) {
                publish_operator_command_service_health(&health.store, config, signer, health)?;
                last_health = Some(Instant::now());
            }
        }
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

fn service_config_sha256(
    config: &OperatorCommandServiceConfig,
    health: &OperatorCommandServiceHealthConfig,
) -> Result<String> {
    let bytes = rmp_serde::to_vec(&(
        config.policy.runtime_id.as_str(),
        config.policy.discord_guild_id.as_str(),
        &config.policy.allowed_channel_ids,
        &config.policy.actor_capabilities,
        config.policy.max_ttl_seconds,
        config.trusted_bifrost_identity.identity_id.as_str(),
        (
            config.provider_health.query_endpoint,
            config.provider_health.idunn_runtime_id.as_str(),
            &config.provider_health.trust_anchor_store,
            &config.provider_health.admission_store,
            config.provider_health.max_local_age_millis,
            config.provider_health.deployment_id.as_str(),
            config.provider_health.release_id.as_str(),
            config.provider_health.release_witness_sha256.as_str(),
            config.provider_health.source_commit.as_str(),
        ),
        health.bind.as_str(),
        health.release_id.as_str(),
        health.release_witness_sha256.as_str(),
        health.source_commit.as_str(),
    ))?;
    Ok(format!("sha256-{:x}", Sha256::digest(bytes)))
}

fn service_health_signing_payload(value: &EpiphanyOperatorCommandServiceHealth) -> Result<Vec<u8>> {
    rmp_serde::to_vec(&(
        value.schema_version.as_str(),
        value.runtime_id.as_str(),
        value.release_id.as_str(),
        value.release_witness_sha256.as_str(),
        value.source_commit.as_str(),
        value.bind.as_str(),
        value.config_sha256.as_str(),
        value.bifrost_identity_id.as_str(),
        value.executor_identity_id.as_str(),
        value.observed_at.as_str(),
        value.observed_at_millis,
        value.process_id,
        value.process_creation_token,
        value.process_executable_path.as_str(),
        value.private_state_exposed,
    ))
    .map_err(Into::into)
}

pub fn publish_operator_command_service_health(
    store: &Path,
    config: &OperatorCommandServiceConfig,
    signer: &HostIdentitySigner,
    health: &OperatorCommandServiceHealthConfig,
) -> Result<EpiphanyOperatorCommandServiceHealth> {
    let observed_at_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .try_into()
        .context("operator health time exceeds u64")?;
    let process = crate::capture_process_instance(std::process::id())?;
    let mut value = EpiphanyOperatorCommandServiceHealth {
        schema_version: EPIPHANY_OPERATOR_COMMAND_SERVICE_HEALTH_SCHEMA_VERSION.into(),
        runtime_id: config.policy.runtime_id.clone(),
        release_id: health.release_id.clone(),
        release_witness_sha256: health.release_witness_sha256.clone(),
        source_commit: health.source_commit.clone(),
        bind: health.bind.clone(),
        config_sha256: service_config_sha256(config, health)?,
        bifrost_identity_id: config.trusted_bifrost_identity.identity_id.clone(),
        executor_identity_id: signer.entry().identity_id.clone(),
        observed_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        observed_at_millis,
        process_id: process.process_id,
        process_creation_token: process.creation_token,
        process_executable_path: process.executable_path.display().to_string(),
        private_state_exposed: false,
        executor_signature: Vec::new(),
    };
    value.executor_signature = signer
        .sign(
            SERVICE_HEALTH_SIGNING_PURPOSE,
            &service_health_signing_payload(&value)?,
        )?
        .signature;
    let backing = SingleFileMessagePackBackingStore::new(store);
    let existing = backing.pull_all()?.into_iter().find(|entry| {
        entry.r#type == <EpiphanyOperatorCommandServiceHealth as DatabaseEntry>::TYPE
            && entry.key == SERVICE_HEALTH_KEY
    });
    let mut cache = CultCache::new();
    cache.register_entry_type::<EpiphanyOperatorCommandServiceHealth>()?;
    let (replacement, _) = cache.prepare_entry(SERVICE_HEALTH_KEY, &value)?;
    let committed = match existing {
        Some(expected) => backing.compare_and_swap_entry(&expected, replacement)?,
        None => backing.insert_entry_if_absent(replacement)?,
    };
    if !committed {
        bail!("operator service health lost exact CAS");
    }
    Ok(value)
}

pub fn authenticate_operator_command_service_health(
    store: &Path,
    runtime_id: &str,
    release_id: &str,
    release_witness_sha256: &str,
    source_commit: &str,
    bind: &str,
    executor: &HostIdentityTrustAnchorEntry,
    now_millis: u64,
    freshness_millis: u64,
) -> Result<EpiphanyOperatorCommandServiceHealth> {
    let mut matches = SingleFileMessagePackBackingStore::new(store)
        .pull_all()?
        .into_iter()
        .filter(|entry| {
            entry.r#type == <EpiphanyOperatorCommandServiceHealth as DatabaseEntry>::TYPE
                && entry.key == SERVICE_HEALTH_KEY
        });
    let envelope = matches
        .next()
        .context("operator service health is absent")?;
    if matches.next().is_some() {
        bail!("operator service health has duplicate owner state");
    }
    let value: EpiphanyOperatorCommandServiceHealth = rmp_serde::from_slice(&envelope.payload)?;
    if value.schema_version != EPIPHANY_OPERATOR_COMMAND_SERVICE_HEALTH_SCHEMA_VERSION
        || value.private_state_exposed
        || value.runtime_id != runtime_id
        || value.release_id != release_id
        || value.release_witness_sha256 != release_witness_sha256
        || value.source_commit != source_commit
        || value.bind != bind
        || !value.config_sha256.starts_with("sha256-")
        || value.config_sha256.len() != 71
        || value.bifrost_identity_id.trim().is_empty()
        || value.executor_identity_id != executor.identity_id
        || value.process_id == 0
        || value.process_creation_token == 0
        || value.process_executable_path.trim().is_empty()
        || value.observed_at_millis > now_millis
        || now_millis.saturating_sub(value.observed_at_millis) > freshness_millis
    {
        bail!("operator service health is stale or not bound to the exact release/config/identity");
    }
    crate::verify_host_identity_trust_anchor_signature(
        executor,
        SERVICE_HEALTH_SIGNING_PURPOSE,
        &service_health_signing_payload(&value)?,
        &crate::HostIdentitySignature {
            identity_id: value.executor_identity_id.clone(),
            signature: value.executor_signature.clone(),
        },
    )?;
    Ok(value)
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
            provider_health: OperatorStatusProviderHealthConfig {
                query_endpoint: "127.0.0.1:1".parse()?,
                idunn_runtime_id: "idunn-interop-fixture".into(),
                trust_anchor_store: private.join("idunn-anchor.cc"),
                admission_store: private.join("provider-admission.cc"),
                max_local_age_millis: 30_000,
                deployment_id: "deployment-fixture".into(),
                release_id: "release-fixture".into(),
                release_witness_sha256: format!("sha256-{}", "a".repeat(64)),
                source_commit: "b".repeat(40),
            },
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
        let protocol = serde_json::json!({
            "schemaVersion": "epiphany.operator_command.protocol_fixture.v1",
            "admissionSchemaVersion": crate::BIFROST_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION,
            "resultSchemaVersion": crate::OPERATOR_COMMAND_RESULT_SCHEMA_VERSION,
            "sealedResultSchemaVersion": EPIPHANY_OPERATOR_COMMAND_RESULT_RECEIPT_SCHEMA_VERSION,
            "admissionSigningPurpose": crate::operator_command_admission_signing_purpose(),
            "resultSigningPurpose": RESULT_SIGNING_PURPOSE,
            "commands": [
                serde_json::to_value(crate::OperatorCommand::Status)?,
                serde_json::to_value(crate::OperatorCommand::Sleep { reason: "Bounded sleep reason".into() })?,
                serde_json::to_value(crate::OperatorCommand::Wake)?,
                serde_json::to_value(crate::OperatorCommand::Directive { objective: "Bounded operator objective".into() })?,
                serde_json::to_value(crate::OperatorCommand::Reviews)?,
                serde_json::to_value(crate::OperatorCommand::Review {
                    mind_request_id: "mind-request-fixture-1".into(),
                    candidate_id: "candidate-fixture-1".into(),
                    candidate_sha256: "1".repeat(64),
                    expected_model_revision: 41,
                    expected_model_hash: "2".repeat(64),
                    decision: crate::RepoFrontierPlanDecision::Adopt,
                })?,
            ],
            "reviewDecisions": [
                serde_json::to_value(crate::RepoFrontierPlanDecision::Adopt)?,
                serde_json::to_value(crate::RepoFrontierPlanDecision::Refuse)?,
                serde_json::to_value(crate::RepoFrontierPlanDecision::Hold)?,
            ],
            "boundedReviewResult": serde_json::to_value(EpiphanyOperatorCommandWireResult {
                schema_version: crate::OPERATOR_COMMAND_RESULT_SCHEMA_VERSION.into(),
                result_id: "operator-result-fixture-reviews".into(),
                command_id: "fixture-command-reviews".into(),
                packet_sha256: format!("sha256-{}", "3".repeat(64)),
                target_runtime_id: "epiphany-interop-fixture".into(),
                disposition: "observed".into(),
                consequence_kind: "mind-review-candidates".into(),
                consequence_ref: String::new(),
                completed_at: "2026-07-19T12:00:01Z".into(),
                private_state_exposed: false,
                operator_status: String::new(),
                state_status: String::new(),
                coordinator_action: String::new(),
                brake_status: String::new(),
                detail: "1 pending Mind review candidate(s)".into(),
                reviews: vec![crate::RepoFrontierPlanReviewSummary {
                    mind_request_id: "mind-request-fixture-1".into(),
                    candidate_id: "candidate-fixture-1".into(),
                    candidate_sha256: "1".repeat(64),
                    model_revision: 41,
                    model_hash: "2".repeat(64),
                    frontier_item_id: "frontier-fixture-1".into(),
                    requested_at: "2026-07-19T11:59:00Z".into(),
                }],
                review_candidate_id: String::new(),
                review_decision: String::new(),
                status_v2: None,
            })?,
            "operatorStatusV2MigrationFixture": serde_json::to_value(
                operator_status_v2_migration_fixture()
            )?,
        });
        let protocol_bytes = serde_json::to_vec_pretty(&protocol)?;
        std::fs::write(output.join("protocol.json"), &protocol_bytes)?;
        let manifest = OperatorCommandInteropFixtureManifest {
            schema_version: "epiphany.operator_command.interop_fixture.v0".into(),
            admission_file: "operator-admission.msgpack".into(),
            bifrost_raw_trust_anchor_file: "bifrost-anchor.msgpack".into(),
            executor_raw_trust_anchor_file: "executor-anchor.msgpack".into(),
            executor_cultcache_trust_anchor_file: "executor-anchor.cc".into(),
            sealed_result_file: "sealed-result.msgpack".into(),
            protocol_file: "protocol.json".into(),
            admission_sha256: format!("sha256-{:x}", Sha256::digest(&admission_bytes)),
            sealed_result_sha256: format!("sha256-{:x}", Sha256::digest(&receipt_bytes)),
            protocol_sha256: format!("sha256-{:x}", Sha256::digest(&protocol_bytes)),
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

    #[test]
    fn status_v2_golden_is_bounded_and_names_exact_provider_policies() {
        let value = operator_status_v2_migration_fixture();
        assert_eq!(value.schema_version, "epiphany.operator.status.v2");
        assert_eq!(value.provider_set_status, "incomplete");
        assert_eq!(value.providers.len(), 2);
        assert_eq!(value.providers[0].daemon_id, "yggdrasil-epiphany");
        assert_eq!(
            value.providers[1].daemon_id,
            "yggdrasil-bifrost-persona-feedback"
        );
        assert!(value.providers.iter().all(|provider| {
            provider.availability == "unavailable"
                && provider.state.is_none()
                && provider.projection_sha256.is_none()
        }));
        let json = serde_json::to_string(&value).unwrap();
        for forbidden in ["signature", "privateKey", "detail", "\\\\", ":\\"] {
            assert!(
                !json.contains(forbidden),
                "leaked forbidden field {forbidden}"
            );
        }
        assert!(!value.private_state_exposed);
    }

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
                provider_health: OperatorStatusProviderHealthConfig {
                    query_endpoint: "127.0.0.1:1".parse()?,
                    idunn_runtime_id: "idunn-yggdrasil".into(),
                    trust_anchor_store: root.join("idunn-anchor.cc"),
                    admission_store: root.join("provider-admission.cc"),
                    max_local_age_millis: 30_000,
                    deployment_id: "deployment-test".into(),
                    release_id: "release-test".into(),
                    release_witness_sha256: format!("sha256-{}", "a".repeat(64)),
                    source_commit: "b".repeat(40),
                },
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
    fn signed_service_health_binds_release_config_identity_and_freshness() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let (config, executor, _) = fixture(temp.path())?;
        let executor_anchor = crate::export_host_identity_trust_anchor(
            &executor,
            &temp.path().join("executor-anchor.cc"),
        )?;
        let health_config = OperatorCommandServiceHealthConfig {
            store: temp.path().join("operator-health.cc"),
            bind: "127.0.0.1:17874".into(),
            release_id: "release-1".into(),
            release_witness_sha256: format!("sha256-{}", "a".repeat(64)),
            source_commit: "b".repeat(40),
        };
        let published = publish_operator_command_service_health(
            &health_config.store,
            &config,
            &executor,
            &health_config,
        )?;
        assert_eq!(
            authenticate_operator_command_service_health(
                &health_config.store,
                &config.policy.runtime_id,
                &health_config.release_id,
                &health_config.release_witness_sha256,
                &health_config.source_commit,
                &health_config.bind,
                &executor_anchor,
                published.observed_at_millis,
                15_000,
            )?,
            published
        );
        assert!(
            authenticate_operator_command_service_health(
                &health_config.store,
                &config.policy.runtime_id,
                "alien-release",
                &health_config.release_witness_sha256,
                &health_config.source_commit,
                &health_config.bind,
                &executor_anchor,
                published.observed_at_millis,
                15_000,
            )
            .is_err()
        );
        assert!(
            authenticate_operator_command_service_health(
                &health_config.store,
                &config.policy.runtime_id,
                &health_config.release_id,
                &health_config.release_witness_sha256,
                &health_config.source_commit,
                &health_config.bind,
                &executor_anchor,
                published.observed_at_millis + 15_001,
                15_000,
            )
            .is_err()
        );
        Ok(())
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
                None,
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
        let protocol_bytes = std::fs::read(temp.path().join(&manifest.protocol_file))?;
        assert_eq!(
            manifest.protocol_sha256,
            format!("sha256-{:x}", Sha256::digest(&protocol_bytes))
        );
        let protocol: serde_json::Value = serde_json::from_slice(&protocol_bytes)?;
        assert_eq!(
            protocol["commands"]
                .as_array()
                .expect("fixture commands")
                .iter()
                .map(|command| command["kind"].as_str().expect("command kind"))
                .collect::<Vec<_>>(),
            vec!["status", "sleep", "wake", "directive", "reviews", "review"]
        );
        let review = &protocol["commands"][5];
        assert!(review.get("mindRequestId").is_some());
        assert!(review.get("expectedModelRevision").is_some());
        assert!(review.get("mind_request_id").is_none());
        assert_eq!(
            protocol["boundedReviewResult"]["reviews"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(
            protocol["boundedReviewResult"]["reviews"][0]
                .get("proposalText")
                .is_none()
        );
        Ok(())
    }
}
