use anyhow::{Result, anyhow, bail};
use cultcache_rs::{
    CacheBackingStore, CultCache, DatabaseEntry, SingleFileMessagePackBackingStore,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

use crate::{
    EpiphanyCultMeshSwarmBrakeEntry, HostIdentitySignature, HostIdentityTrustAnchorEntry,
    RepoFrontierPlanDecision, RepoFrontierPlanOperatorReview, RepoFrontierPlanReviewSummary,
    ResidentSelfPressure, commit_operator_repo_frontier_plan_review,
    enqueue_resident_self_pressure, load_epiphany_cultmesh_swarm_brake,
    load_latest_epiphany_cultmesh_operator_snapshot, operator_repo_frontier_plan_review_is_current,
    pending_repo_frontier_plan_reviews, resident_self_pressures,
    verify_host_identity_trust_anchor_signature, write_epiphany_cultmesh_swarm_brake,
};

pub const BIFROST_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION: &str =
    "bifrost.operator_command.delivery.v1";
pub const LEGACY_BIFROST_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION: &str =
    "bifrost.operator_command.delivery.v0";
pub const BIFROST_OPERATOR_COMMAND_DELIVERY_TYPE: &str = "bifrost.operator_command.delivery";
pub const LOCAL_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION: &str =
    "epiphany.operator_command.admitted.v1";
pub const LEGACY_LOCAL_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION: &str =
    "epiphany.operator_command.admitted.v0";
pub const OPERATOR_COMMAND_RESULT_SCHEMA_VERSION: &str = "epiphany.operator_command.result.v1";
const SIGNING_PURPOSE: &str = "bifrost.operator-command.delivery.v1";
const LEGACY_SIGNING_PURPOSE: &str = "bifrost.operator-command.delivery.v0";
const DISCORD_OPERATOR_BRAKE_ID: &str = "epiphany-discord-operator-brake";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorCapability {
    Status,
    Sleep,
    Wake,
    Directive,
    Reviews,
    Review,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum OperatorCommand {
    Status,
    Sleep {
        reason: String,
    },
    Wake,
    Directive {
        objective: String,
    },
    Reviews,
    Review {
        mind_request_id: String,
        candidate_id: String,
        candidate_sha256: String,
        expected_model_revision: u64,
        expected_model_hash: String,
        decision: RepoFrontierPlanDecision,
    },
}

impl OperatorCommand {
    fn capability(&self) -> OperatorCapability {
        match self {
            Self::Status => OperatorCapability::Status,
            Self::Sleep { .. } => OperatorCapability::Sleep,
            Self::Wake => OperatorCapability::Wake,
            Self::Directive { .. } => OperatorCapability::Directive,
            Self::Reviews => OperatorCapability::Reviews,
            Self::Review { .. } => OperatorCapability::Review,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperatorCommandPacket {
    pub command_id: String,
    pub nonce: String,
    pub source_event_id: String,
    pub source_actor_id: String,
    pub discord_guild_id: String,
    pub discord_channel_id: String,
    pub discord_message_id: String,
    pub target_runtime_id: String,
    pub issued_at: String,
    pub expires_at: String,
    pub command: OperatorCommand,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BifrostOperatorCommandAdmission {
    pub schema_name: String,
    pub schema_version: String,
    pub admission_id: String,
    pub packet: OperatorCommandPacket,
    pub packet_sha256: String,
    pub source_observer_id: String,
    pub source_observer_runtime_id: String,
    pub provider: String,
    pub bifrost_admission_receipt_id: String,
    pub authority: String,
    pub provider_identity_id: String,
    pub provider_signature: Vec<u8>,
}

impl DatabaseEntry for BifrostOperatorCommandAdmission {
    const TYPE: &'static str = BIFROST_OPERATOR_COMMAND_DELIVERY_TYPE;
    const SCHEMA_NAME: &'static str = "BifrostOperatorCommandAdmission";
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.operator_command.admitted",
    schema = "LocalAdmittedOperatorCommand"
)]
pub struct LocalAdmittedOperatorCommand {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub command_id: String,
    #[cultcache(key = 2)]
    pub admission_id: String,
    #[cultcache(key = 3)]
    pub nonce: String,
    #[cultcache(key = 4)]
    pub packet_sha256: String,
    #[cultcache(key = 5)]
    pub source_actor_id: String,
    #[cultcache(key = 6)]
    pub capability: String,
    #[cultcache(key = 7)]
    pub target_runtime_id: String,
    #[cultcache(key = 8)]
    pub admitted_at: String,
    #[cultcache(key = 9)]
    pub expires_at: String,
    #[cultcache(key = 10)]
    pub authority: String,
    #[cultcache(key = 11)]
    pub private_state_exposed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorCommandResultDisposition {
    Observed,
    Applied,
    Refused,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.operator_command.result",
    schema = "OperatorCommandResult"
)]
pub struct OperatorCommandResult {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub result_id: String,
    #[cultcache(key = 2)]
    pub command_id: String,
    #[cultcache(key = 3)]
    pub packet_sha256: String,
    #[cultcache(key = 4)]
    pub target_runtime_id: String,
    #[cultcache(key = 5)]
    pub disposition: String,
    #[cultcache(key = 6)]
    pub consequence_kind: String,
    #[cultcache(key = 7)]
    pub consequence_ref: String,
    #[cultcache(key = 8)]
    pub completed_at: String,
    #[cultcache(key = 9)]
    pub private_state_exposed: bool,
    #[cultcache(key = 10)]
    pub operator_status: String,
    #[cultcache(key = 11)]
    pub state_status: String,
    #[cultcache(key = 12)]
    pub coordinator_action: String,
    #[cultcache(key = 13)]
    pub brake_status: String,
    #[cultcache(key = 14)]
    pub detail: String,
    #[cultcache(key = 15, default)]
    pub reviews: Vec<RepoFrontierPlanReviewSummary>,
    #[cultcache(key = 16, default)]
    pub review_candidate_id: String,
    #[cultcache(key = 17, default)]
    pub review_decision: String,
}

#[derive(Clone, Debug)]
pub struct OperatorCommandPolicy {
    pub runtime_id: String,
    pub discord_guild_id: String,
    pub allowed_channel_ids: Vec<String>,
    pub actor_capabilities: BTreeMap<String, Vec<OperatorCapability>>,
    pub max_ttl_seconds: i64,
}

pub fn operator_command_packet_sha256(packet: &OperatorCommandPacket) -> Result<String> {
    Ok(format!(
        "sha256-{:x}",
        Sha256::digest(rmp_serde::to_vec(packet)?)
    ))
}

pub fn operator_command_admission_signing_payload(
    admission: &BifrostOperatorCommandAdmission,
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

pub fn operator_command_admission_signing_purpose() -> &'static str {
    SIGNING_PURPOSE
}

fn command_cache(path: &Path) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<LocalAdmittedOperatorCommand>()?;
    cache.register_entry_type::<OperatorCommandResult>()?;
    if path.exists() {
        for envelope in SingleFileMessagePackBackingStore::new(path).pull_all()? {
            match envelope.r#type.as_str() {
                LocalAdmittedOperatorCommand::TYPE => {
                    cache.load_envelope::<LocalAdmittedOperatorCommand>(envelope)?;
                }
                OperatorCommandResult::TYPE => {
                    cache.load_envelope::<OperatorCommandResult>(envelope)?;
                }
                _ => bail!("operator command store contains foreign authority"),
            };
        }
    }
    Ok(cache)
}

fn validate_admission(
    admission: &BifrostOperatorCommandAdmission,
    anchor: &HostIdentityTrustAnchorEntry,
    policy: &OperatorCommandPolicy,
    now: chrono::DateTime<chrono::Utc>,
    enforce_time: bool,
) -> Result<OperatorCapability> {
    let packet = &admission.packet;
    let issued =
        chrono::DateTime::parse_from_rfc3339(&packet.issued_at)?.with_timezone(&chrono::Utc);
    let expires =
        chrono::DateTime::parse_from_rfc3339(&packet.expires_at)?.with_timezone(&chrono::Utc);
    let capability = packet.command.capability();
    let command_text_valid = match &packet.command {
        OperatorCommand::Sleep { reason } => !reason.trim().is_empty() && reason.len() <= 512,
        OperatorCommand::Directive { objective } => {
            !objective.trim().is_empty() && objective.len() <= 4096
        }
        OperatorCommand::Review {
            mind_request_id,
            candidate_id,
            candidate_sha256,
            expected_model_hash,
            ..
        } => {
            !mind_request_id.trim().is_empty()
                && mind_request_id.len() <= 256
                && !candidate_id.trim().is_empty()
                && candidate_id.len() <= 256
                && candidate_sha256.len() == 64
                && candidate_sha256
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit())
                && expected_model_hash.len() == 64
                && expected_model_hash
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit())
        }
        _ => true,
    };
    let legacy_admission =
        admission.schema_version == LEGACY_BIFROST_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION;
    let admission_version_valid = admission.schema_version
        == BIFROST_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION
        || (legacy_admission
            && !matches!(
                &packet.command,
                OperatorCommand::Reviews | OperatorCommand::Review { .. }
            ));
    if admission.schema_name != BIFROST_OPERATOR_COMMAND_DELIVERY_TYPE
        || !admission_version_valid
        || admission.admission_id.trim().is_empty()
        || packet.command_id.trim().is_empty()
        || packet.nonce.trim().is_empty()
        || packet.source_event_id.trim().is_empty()
        || packet.source_actor_id.trim().is_empty()
        || packet.discord_message_id.trim().is_empty()
        || admission.source_observer_id != "voidbot"
        || admission.source_observer_runtime_id.trim().is_empty()
        || admission.provider != "bifrost"
        || admission.bifrost_admission_receipt_id.trim().is_empty()
        || admission.authority != "exact_operator_command_only"
        || !command_text_valid
        || packet.target_runtime_id != policy.runtime_id
        || packet.discord_guild_id != policy.discord_guild_id
        || !policy
            .allowed_channel_ids
            .contains(&packet.discord_channel_id)
        || (enforce_time && issued > now)
        || (enforce_time && expires < now)
        || expires <= issued
        || (expires - issued).num_seconds() > policy.max_ttl_seconds
        || policy.max_ttl_seconds <= 0
        || !policy
            .actor_capabilities
            .get(&packet.source_actor_id)
            .is_some_and(|caps| caps.contains(&capability))
    {
        bail!("Bifrost operator command violates local command policy");
    }
    if admission.provider_identity_id != anchor.identity_id
        || admission.packet_sha256 != operator_command_packet_sha256(packet)?
    {
        bail!("Bifrost operator command identity or payload binding is invalid");
    }
    verify_host_identity_trust_anchor_signature(
        anchor,
        if legacy_admission {
            LEGACY_SIGNING_PURPOSE
        } else {
            SIGNING_PURPOSE
        },
        &operator_command_admission_signing_payload(admission)?,
        &HostIdentitySignature {
            identity_id: admission.provider_identity_id.clone(),
            signature: admission.provider_signature.clone(),
        },
    )?;
    Ok(capability)
}

pub fn admit_and_execute_bifrost_operator_command(
    command_store: &Path,
    local_verse_store: &Path,
    resident_self_store: &Path,
    runtime_store: &Path,
    admission: &BifrostOperatorCommandAdmission,
    trusted_bifrost_identity: &HostIdentityTrustAnchorEntry,
    policy: &OperatorCommandPolicy,
    now: &str,
) -> Result<OperatorCommandResult> {
    let now_dt = chrono::DateTime::parse_from_rfc3339(now)?.with_timezone(&chrono::Utc);
    let capability =
        validate_admission(admission, trusted_bifrost_identity, policy, now_dt, false)?;
    let packet = &admission.packet;
    let mut admitted = LocalAdmittedOperatorCommand {
        schema_version: if admission.schema_version
            == LEGACY_BIFROST_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION
        {
            LEGACY_LOCAL_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION.into()
        } else {
            LOCAL_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION.into()
        },
        command_id: packet.command_id.clone(),
        admission_id: admission.admission_id.clone(),
        nonce: packet.nonce.clone(),
        packet_sha256: admission.packet_sha256.clone(),
        source_actor_id: packet.source_actor_id.clone(),
        capability: format!("{capability:?}").to_lowercase(),
        target_runtime_id: packet.target_runtime_id.clone(),
        admitted_at: now.into(),
        expires_at: packet.expires_at.clone(),
        authority: "exact-command-only".into(),
        private_state_exposed: false,
    };
    let cache = command_cache(command_store)?;
    let prior_admission = cache.get::<LocalAdmittedOperatorCommand>(&packet.command_id)?;
    let was_admitted = prior_admission.is_some();
    for prior in cache.get_all::<LocalAdmittedOperatorCommand>()? {
        if prior.source_actor_id == admitted.source_actor_id
            && prior.nonce == admitted.nonce
            && prior.command_id != admitted.command_id
        {
            bail!("operator command actor nonce collision");
        }
    }
    if let Some(prior) = prior_admission.as_ref() {
        let mut replay = admitted.clone();
        replay.admitted_at = prior.admitted_at.clone();
        if &replay != prior {
            bail!("operator command identity collision");
        }
        admitted = prior.clone();
    }
    let result_id = format!(
        "operator-command-result-{:x}",
        Sha256::digest(format!("{}:{}", packet.command_id, admission.packet_sha256).as_bytes())
    );
    if let Some(existing) = cache.get::<OperatorCommandResult>(&result_id)? {
        return Ok(existing);
    }
    if !was_admitted {
        validate_admission(admission, trusted_bifrost_identity, policy, now_dt, true)?;
    }
    if cache
        .get::<LocalAdmittedOperatorCommand>(&packet.command_id)?
        .as_ref()
        != Some(&admitted)
    {
        let (entry, _) = cache.prepare_entry(&packet.command_id, &admitted)?;
        if !SingleFileMessagePackBackingStore::new(command_store).insert_entry_if_absent(entry)? {
            bail!("operator command admission collision");
        }
    }
    let (
        disposition,
        consequence_kind,
        consequence_ref,
        operator_status,
        state_status,
        coordinator_action,
        brake_status,
        detail,
        reviews,
        review_candidate_id,
        review_decision,
    ) = match &packet.command {
        OperatorCommand::Status => {
            let snapshot = load_latest_epiphany_cultmesh_operator_snapshot(
                local_verse_store,
                policy.runtime_id.clone(),
            )?;
            let Some(snapshot) = snapshot else {
                return persist_result(
                    command_store,
                    packet,
                    admission,
                    policy,
                    now,
                    result_id,
                    OperatorCommandResultDisposition::Refused,
                    "operator-snapshot",
                    "",
                    "",
                    "",
                    "",
                    "",
                    "operator status snapshot is unavailable",
                    Vec::new(),
                    "",
                    "",
                );
            };
            let brake_status =
                load_epiphany_cultmesh_swarm_brake(local_verse_store, policy.runtime_id.clone())?
                    .map(|brake| brake.status)
                    .unwrap_or_else(|| "absent".into());
            (
                OperatorCommandResultDisposition::Observed,
                "operator-snapshot",
                snapshot.snapshot_id,
                snapshot.status,
                snapshot.state_status,
                snapshot.coordinator_action,
                brake_status,
                String::from("bounded operator-safe status projection"),
                Vec::new(),
                String::new(),
                String::new(),
            )
        }
        OperatorCommand::Sleep { reason } => {
            if let Some(current) =
                load_epiphany_cultmesh_swarm_brake(local_verse_store, policy.runtime_id.clone())?
            {
                let exact_retry = current.brake_id == DISCORD_OPERATOR_BRAKE_ID
                    && current.notes.iter().any(|note| {
                        note == &format!("Authenticated operator command {}", packet.command_id)
                    });
                let current_is_not_older =
                    chrono::DateTime::parse_from_rfc3339(&current.created_at_utc)?
                        >= chrono::DateTime::parse_from_rfc3339(&packet.issued_at)?;
                if (current.status == "engaged" && current.brake_id != DISCORD_OPERATOR_BRAKE_ID)
                    || (!exact_retry && current_is_not_older)
                {
                    return persist_result(
                        command_store,
                        packet,
                        admission,
                        policy,
                        now,
                        result_id,
                        OperatorCommandResultDisposition::Refused,
                        "swarm-brake",
                        "",
                        "",
                        "",
                        "",
                        &current.status,
                        "operator sleep cannot replace a foreign or newer brake generation",
                        Vec::new(),
                        "",
                        "",
                    );
                }
            }
            let brake = EpiphanyCultMeshSwarmBrakeEntry {
                schema_version: "epiphany.cultmesh.swarm_brake.v0".into(),
                brake_id: DISCORD_OPERATOR_BRAKE_ID.into(),
                status: "engaged".into(),
                scope: "all".into(),
                reason: reason.trim().into(),
                operator_agent_id: packet.source_actor_id.clone(),
                affected_clusters: vec![policy.runtime_id.clone()],
                protected_surfaces: vec![
                    "heartbeat.scheduler".into(),
                    "coordinator.run".into(),
                    "persona.public_speech".into(),
                    "daemon.tool_invocation".into(),
                ],
                created_at_utc: packet.issued_at.clone(),
                expires_at_utc: None,
                private_state_exposed: false,
                notes: vec![format!(
                    "Authenticated operator command {}",
                    packet.command_id
                )],
                runtime_id: policy.runtime_id.clone(),
            };
            write_epiphany_cultmesh_swarm_brake(
                local_verse_store,
                policy.runtime_id.clone(),
                brake.clone(),
            )?;
            (
                OperatorCommandResultDisposition::Applied,
                "swarm-brake",
                brake.brake_id,
                String::new(),
                String::new(),
                String::new(),
                "engaged".into(),
                String::from("Discord operator brake engaged"),
                Vec::new(),
                String::new(),
                String::new(),
            )
        }
        OperatorCommand::Wake => {
            let brake =
                load_epiphany_cultmesh_swarm_brake(local_verse_store, policy.runtime_id.clone())?;
            let Some(mut brake) = brake.filter(|brake| brake.brake_id == DISCORD_OPERATOR_BRAKE_ID)
            else {
                return persist_result(
                    command_store,
                    packet,
                    admission,
                    policy,
                    now,
                    result_id,
                    OperatorCommandResultDisposition::Refused,
                    "swarm-brake",
                    "",
                    "",
                    "",
                    "",
                    "engaged",
                    "operator wake cannot release a brake owned by another authority",
                    Vec::new(),
                    "",
                    "",
                );
            };
            brake.status = "released".into();
            brake.reason = "Authenticated operator wake; release brake only.".into();
            brake.operator_agent_id = packet.source_actor_id.clone();
            brake.created_at_utc = packet.issued_at.clone();
            brake.expires_at_utc = None;
            brake.runtime_id = policy.runtime_id.clone();
            write_epiphany_cultmesh_swarm_brake(
                local_verse_store,
                policy.runtime_id.clone(),
                brake.clone(),
            )?;
            (
                OperatorCommandResultDisposition::Applied,
                "swarm-brake",
                brake.brake_id,
                String::new(),
                String::new(),
                String::new(),
                "released".into(),
                String::from("Discord operator brake released; no work scheduled"),
                Vec::new(),
                String::new(),
                String::new(),
            )
        }
        OperatorCommand::Directive { objective } => {
            let pressure_id = format!("operator-command-{}", packet.command_id);
            let pressure = ResidentSelfPressure {
                schema_version: crate::RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
                pressure_id: pressure_id.clone(),
                kind: "operator-objective".into(),
                provenance_ref: format!("cultcache://operator-command/{}", packet.command_id),
                objective: objective.trim().into(),
                created_at_millis: issued_millis(&packet.issued_at)?,
                status: "pending".into(),
                consumed_by_grant_id: None,
                private_state_exposed: false,
            };
            match resident_self_pressures(resident_self_store)?
                .into_iter()
                .find(|p| p.pressure_id == pressure_id)
            {
                Some(existing) if existing == pressure => {}
                Some(_) => {
                    return persist_result(
                        command_store,
                        packet,
                        admission,
                        policy,
                        now,
                        result_id,
                        OperatorCommandResultDisposition::Refused,
                        "resident-self-pressure",
                        &pressure_id,
                        "",
                        "",
                        "",
                        "",
                        "operator directive pressure identity collision",
                        Vec::new(),
                        "",
                        "",
                    );
                }
                None => enqueue_resident_self_pressure(resident_self_store, &pressure)?,
            }
            (
                OperatorCommandResultDisposition::Applied,
                "resident-self-pressure",
                pressure_id,
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::from("directive admitted as pressure only"),
                Vec::new(),
                String::new(),
                String::new(),
            )
        }
        OperatorCommand::Reviews => {
            let reviews = pending_repo_frontier_plan_reviews(runtime_store, 10)?;
            (
                OperatorCommandResultDisposition::Observed,
                "mind-review-candidates",
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                format!("{} pending Mind review candidate(s)", reviews.len()),
                reviews,
                String::new(),
                String::new(),
            )
        }
        OperatorCommand::Review {
            mind_request_id,
            candidate_id,
            candidate_sha256,
            expected_model_revision,
            expected_model_hash,
            decision,
        } => {
            let review = RepoFrontierPlanOperatorReview {
                command_id: packet.command_id.clone(),
                admission_id: admission.admission_id.clone(),
                packet_sha256: admission.packet_sha256.clone(),
                source_actor_id: packet.source_actor_id.clone(),
                mind_request_id: mind_request_id.clone(),
                candidate_id: candidate_id.clone(),
                candidate_sha256: candidate_sha256.clone(),
                expected_model_revision: *expected_model_revision,
                expected_model_hash: expected_model_hash.clone(),
                decision: *decision,
                decided_at: packet.issued_at.clone(),
            };
            if !operator_repo_frontier_plan_review_is_current(runtime_store, &review)? {
                return persist_result(
                    command_store,
                    packet,
                    admission,
                    policy,
                    now,
                    result_id,
                    OperatorCommandResultDisposition::Refused,
                    "mind-review-decision",
                    "",
                    "",
                    "",
                    "",
                    "",
                    "Mind refused stale, terminal, or mismatched review candidate",
                    Vec::new(),
                    candidate_id,
                    &format!("{decision:?}").to_lowercase(),
                );
            }
            let receipt = commit_operator_repo_frontier_plan_review(runtime_store, &review)?;
            (
                OperatorCommandResultDisposition::Applied,
                "mind-review-decision",
                receipt.decision_id,
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::from("Mind committed exact current-candidate disposition"),
                Vec::new(),
                candidate_id.clone(),
                format!("{decision:?}").to_lowercase(),
            )
        }
    };
    persist_result(
        command_store,
        packet,
        admission,
        policy,
        now,
        result_id,
        disposition,
        consequence_kind,
        &consequence_ref,
        &operator_status,
        &state_status,
        &coordinator_action,
        &brake_status,
        &detail,
        reviews,
        &review_candidate_id,
        &review_decision,
    )
}

#[allow(clippy::too_many_arguments)]
fn persist_result(
    command_store: &Path,
    packet: &OperatorCommandPacket,
    admission: &BifrostOperatorCommandAdmission,
    policy: &OperatorCommandPolicy,
    now: &str,
    result_id: String,
    disposition: OperatorCommandResultDisposition,
    consequence_kind: &str,
    consequence_ref: &str,
    operator_status: &str,
    state_status: &str,
    coordinator_action: &str,
    brake_status: &str,
    detail: &str,
    reviews: Vec<RepoFrontierPlanReviewSummary>,
    review_candidate_id: &str,
    review_decision: &str,
) -> Result<OperatorCommandResult> {
    let result = OperatorCommandResult {
        schema_version: OPERATOR_COMMAND_RESULT_SCHEMA_VERSION.into(),
        result_id: result_id.clone(),
        command_id: packet.command_id.clone(),
        packet_sha256: admission.packet_sha256.clone(),
        target_runtime_id: policy.runtime_id.clone(),
        disposition: format!("{disposition:?}").to_lowercase(),
        consequence_kind: consequence_kind.into(),
        consequence_ref: consequence_ref.into(),
        completed_at: now.into(),
        private_state_exposed: false,
        operator_status: operator_status.into(),
        state_status: state_status.into(),
        coordinator_action: coordinator_action.into(),
        brake_status: brake_status.into(),
        detail: detail.into(),
        reviews,
        review_candidate_id: review_candidate_id.into(),
        review_decision: review_decision.into(),
    };
    let refreshed = command_cache(command_store)?;
    let (entry, _) = refreshed.prepare_entry(&result_id, &result)?;
    if !SingleFileMessagePackBackingStore::new(command_store).insert_entry_if_absent(entry)? {
        let existing = command_cache(command_store)?.get::<OperatorCommandResult>(&result_id)?;
        if existing.as_ref() != Some(&result) {
            bail!("operator command result collision");
        }
    }
    Ok(result)
}

fn issued_millis(value: &str) -> Result<u64> {
    let millis = chrono::DateTime::parse_from_rfc3339(value)?.timestamp_millis();
    u64::try_from(millis).map_err(|_| anyhow!("operator command timestamp predates epoch"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cultcache_rs::CultCacheEnvelope;

    fn policy() -> OperatorCommandPolicy {
        OperatorCommandPolicy {
            runtime_id: "epiphany-yggdrasil".into(),
            discord_guild_id: "guild-1".into(),
            allowed_channel_ids: vec!["ops-1".into()],
            actor_capabilities: BTreeMap::from([(
                "operator-1".into(),
                vec![
                    OperatorCapability::Status,
                    OperatorCapability::Sleep,
                    OperatorCapability::Wake,
                    OperatorCapability::Directive,
                    OperatorCapability::Reviews,
                    OperatorCapability::Review,
                ],
            )]),
            max_ttl_seconds: 600,
        }
    }

    fn signed(
        signer: &crate::HostIdentitySigner,
        id: &str,
        command: OperatorCommand,
    ) -> Result<BifrostOperatorCommandAdmission> {
        let packet = OperatorCommandPacket {
            command_id: id.into(),
            nonce: format!("nonce-{id}"),
            source_event_id: format!("event-{id}"),
            source_actor_id: "operator-1".into(),
            discord_guild_id: "guild-1".into(),
            discord_channel_id: "ops-1".into(),
            discord_message_id: format!("message-{id}"),
            target_runtime_id: "epiphany-yggdrasil".into(),
            issued_at: "2026-07-19T11:59:00Z".into(),
            expires_at: "2026-07-19T12:05:00Z".into(),
            command,
        };
        let mut admission = BifrostOperatorCommandAdmission {
            schema_name: BIFROST_OPERATOR_COMMAND_DELIVERY_TYPE.into(),
            schema_version: BIFROST_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION.into(),
            admission_id: format!("admission-{id}"),
            packet_sha256: operator_command_packet_sha256(&packet)?,
            packet,
            source_observer_id: "voidbot".into(),
            source_observer_runtime_id: "voidbot-yggdrasil".into(),
            provider: "bifrost".into(),
            bifrost_admission_receipt_id: format!("receipt-{id}"),
            authority: "exact_operator_command_only".into(),
            provider_identity_id: signer.entry().identity_id.clone(),
            provider_signature: Vec::new(),
        };
        admission.provider_signature = signer
            .sign(
                SIGNING_PURPOSE,
                &operator_command_admission_signing_payload(&admission)?,
            )?
            .signature;
        Ok(admission)
    }

    fn legacy_signed(
        signer: &crate::HostIdentitySigner,
        id: &str,
        command: OperatorCommand,
    ) -> Result<BifrostOperatorCommandAdmission> {
        let mut admission = signed(signer, id, command)?;
        admission.schema_version = LEGACY_BIFROST_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION.into();
        admission.provider_signature = signer
            .sign(
                LEGACY_SIGNING_PURPOSE,
                &operator_command_admission_signing_payload(&admission)?,
            )?
            .signature;
        Ok(admission)
    }

    fn snapshot() -> crate::EpiphanyCultMeshOperatorSnapshotEntry {
        crate::EpiphanyCultMeshOperatorSnapshotEntry {
            schema_version: crate::EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_SCHEMA_VERSION.into(),
            runtime_id: "epiphany-yggdrasil".into(),
            verse_id: crate::EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.into(),
            snapshot_id: "snapshot-1".into(),
            generated_at_utc: "2026-07-19T11:58:00Z".into(),
            source_mode: "native".into(),
            source_path: "cultcache://operator-safe/status".into(),
            thread_id: "thread-1".into(),
            status: "sleeping".into(),
            state_status: "ready".into(),
            coordinator_action: "none".into(),
            crrc_action: "none".into(),
            pressure_level: "low".into(),
            reorient_action: "none".into(),
            next_action: "await pressure".into(),
            artifact_refs: Vec::new(),
            available_actions: vec!["status".into()],
            notes: Vec::new(),
        }
    }

    #[test]
    fn hostile_commands_refuse_without_local_writes() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let signer = crate::enroll_host_identity_at(&temp.path().join("identity.cc"))?;
        let anchor =
            crate::export_host_identity_trust_anchor(&signer, &temp.path().join("anchor.cc"))?;
        for case in ["signature", "runtime", "channel", "actor", "expired"] {
            let root = temp.path().join(case);
            std::fs::create_dir_all(&root)?;
            let (commands, verse, resident) = (
                root.join("commands.cc"),
                root.join("verse.cc"),
                root.join("resident.cc"),
            );
            let mut admission = signed(
                &signer,
                case,
                OperatorCommand::Directive {
                    objective: "Map this domain.".into(),
                },
            )?;
            match case {
                "signature" => admission.provider_signature[0] ^= 1,
                "runtime" => admission.packet.target_runtime_id = "alien".into(),
                "channel" => admission.packet.discord_channel_id = "alien".into(),
                "actor" => admission.packet.source_actor_id = "alien".into(),
                "expired" => admission.packet.expires_at = "2026-07-19T11:59:30Z".into(),
                _ => unreachable!(),
            }
            assert!(
                admit_and_execute_bifrost_operator_command(
                    &commands,
                    &verse,
                    &resident,
                    &resident,
                    &admission,
                    &anchor,
                    &policy(),
                    "2026-07-19T12:00:00Z"
                )
                .is_err()
            );
            assert!(!commands.exists() && !verse.exists() && !resident.exists());
        }
        Ok(())
    }

    #[test]
    fn legacy_v0_delivery_replays_across_v1_cutover_but_cannot_carry_review_commands() -> Result<()>
    {
        let temp = tempfile::tempdir()?;
        let signer = crate::enroll_host_identity_at(&temp.path().join("identity.cc"))?;
        let anchor =
            crate::export_host_identity_trust_anchor(&signer, &temp.path().join("anchor.cc"))?;
        let (commands, verse, resident, runtime) = (
            temp.path().join("commands.cc"),
            temp.path().join("verse.cc"),
            temp.path().join("resident.cc"),
            temp.path().join("runtime.cc"),
        );
        let legacy = legacy_signed(
            &signer,
            "legacy-sleep",
            OperatorCommand::Sleep {
                reason: "Drain the v0 delivery.".into(),
            },
        )?;
        let first = admit_and_execute_bifrost_operator_command(
            &commands,
            &verse,
            &resident,
            &runtime,
            &legacy,
            &anchor,
            &policy(),
            "2026-07-19T12:00:00Z",
        )?;
        let replay = admit_and_execute_bifrost_operator_command(
            &commands,
            &verse,
            &resident,
            &runtime,
            &legacy,
            &anchor,
            &policy(),
            "2026-07-19T12:10:00Z",
        )?;
        assert_eq!(replay, first);
        let review = legacy_signed(&signer, "legacy-review", OperatorCommand::Reviews)?;
        assert!(
            admit_and_execute_bifrost_operator_command(
                &commands,
                &verse,
                &resident,
                &runtime,
                &review,
                &anchor,
                &policy(),
                "2026-07-19T12:00:00Z",
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn persisted_v0_command_ledger_replays_after_v1_cutover() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let signer = crate::enroll_host_identity_at(&temp.path().join("identity.cc"))?;
        let anchor =
            crate::export_host_identity_trust_anchor(&signer, &temp.path().join("anchor.cc"))?;
        let (commands, verse, resident, runtime) = (
            temp.path().join("commands.cc"),
            temp.path().join("verse.cc"),
            temp.path().join("resident.cc"),
            temp.path().join("runtime.cc"),
        );
        let legacy = legacy_signed(&signer, "persisted-v0", OperatorCommand::Status)?;
        let packet = &legacy.packet;
        let result_id = format!(
            "operator-command-result-{:x}",
            Sha256::digest(format!("{}:{}", packet.command_id, legacy.packet_sha256).as_bytes())
        );
        let admitted_at = "2026-07-19T12:00:00Z";
        let old_admission = (
            LEGACY_LOCAL_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION,
            packet.command_id.as_str(),
            legacy.admission_id.as_str(),
            packet.nonce.as_str(),
            legacy.packet_sha256.as_str(),
            packet.source_actor_id.as_str(),
            "status",
            packet.target_runtime_id.as_str(),
            admitted_at,
            packet.expires_at.as_str(),
            "exact-command-only",
            false,
        );
        let old_result = (
            "epiphany.operator_command.result.v0",
            result_id.as_str(),
            packet.command_id.as_str(),
            legacy.packet_sha256.as_str(),
            packet.target_runtime_id.as_str(),
            "observed",
            "operator_snapshot",
            "snapshot-v0",
            admitted_at,
            false,
            "sleeping",
            "ready",
            "none",
            "engaged",
            "persisted v0 result",
        );
        let backing = SingleFileMessagePackBackingStore::new(&commands);
        for (key, ty, payload) in [
            (
                packet.command_id.as_str(),
                LocalAdmittedOperatorCommand::TYPE,
                rmp_serde::to_vec(&old_admission)?,
            ),
            (
                result_id.as_str(),
                OperatorCommandResult::TYPE,
                rmp_serde::to_vec(&old_result)?,
            ),
        ] {
            assert!(backing.insert_entry_if_absent(CultCacheEnvelope {
                key: key.into(),
                r#type: ty.into(),
                payload,
                stored_at: admitted_at.into(),
                schema_id: Some(ty.into()),
            })?);
        }

        let replay = admit_and_execute_bifrost_operator_command(
            &commands,
            &verse,
            &resident,
            &runtime,
            &legacy,
            &anchor,
            &policy(),
            admitted_at,
        )?;
        assert_eq!(replay.schema_version, "epiphany.operator_command.result.v0");
        assert_eq!(replay.consequence_ref, "snapshot-v0");
        assert!(replay.reviews.is_empty());
        assert!(replay.review_candidate_id.is_empty());
        assert!(replay.review_decision.is_empty());
        assert!(!verse.exists() && !resident.exists() && !runtime.exists());
        Ok(())
    }

    #[test]
    fn commands_delegate_without_wake_or_directive_authority_leak() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let signer = crate::enroll_host_identity_at(&temp.path().join("identity.cc"))?;
        let anchor =
            crate::export_host_identity_trust_anchor(&signer, &temp.path().join("anchor.cc"))?;
        let (commands, verse, resident) = (
            temp.path().join("commands.cc"),
            temp.path().join("verse.cc"),
            temp.path().join("resident.cc"),
        );
        crate::write_epiphany_cultmesh_operator_snapshot(&verse, snapshot())?;
        let p = policy();
        let status = signed(&signer, "status", OperatorCommand::Status)?;
        let status_result = admit_and_execute_bifrost_operator_command(
            &commands,
            &verse,
            &resident,
            &resident,
            &status,
            &anchor,
            &p,
            "2026-07-19T12:00:00Z",
        )?;
        assert_eq!(status_result.consequence_ref, "snapshot-1");
        assert_eq!(status_result.operator_status, "sleeping");
        assert_eq!(status_result.state_status, "ready");
        assert!(!resident.exists());
        let sleep = signed(
            &signer,
            "sleep",
            OperatorCommand::Sleep {
                reason: "Move preparation.".into(),
            },
        )?;
        admit_and_execute_bifrost_operator_command(
            &commands,
            &verse,
            &resident,
            &resident,
            &sleep,
            &anchor,
            &p,
            "2026-07-19T12:00:00Z",
        )?;
        let written_brake =
            load_epiphany_cultmesh_swarm_brake(&verse, p.runtime_id.clone())?.unwrap();
        assert_eq!(written_brake.status, "engaged");
        assert_eq!(written_brake.brake_id, DISCORD_OPERATOR_BRAKE_ID);
        assert_eq!(written_brake.scope, "all");
        assert_eq!(
            written_brake.protected_surfaces,
            vec![
                "heartbeat.scheduler",
                "coordinator.run",
                "persona.public_speech",
                "daemon.tool_invocation"
            ]
        );
        assert!(
            !written_brake
                .protected_surfaces
                .iter()
                .any(|surface| matches!(
                    surface.as_str(),
                    "scheduling" | "daemon-pokes" | "operator-runs"
                ))
        );
        let wake = signed(&signer, "wake", OperatorCommand::Wake)?;
        admit_and_execute_bifrost_operator_command(
            &commands,
            &verse,
            &resident,
            &resident,
            &wake,
            &anchor,
            &p,
            "2026-07-19T12:00:01Z",
        )?;
        assert_eq!(
            load_epiphany_cultmesh_swarm_brake(&verse, p.runtime_id.clone())?
                .unwrap()
                .status,
            "released"
        );
        assert!(
            !resident.exists(),
            "wake created pressure, grant, or job authority"
        );
        let directive = signed(
            &signer,
            "directive",
            OperatorCommand::Directive {
                objective: "Map the operator nerve.".into(),
            },
        )?;
        let first = admit_and_execute_bifrost_operator_command(
            &commands,
            &verse,
            &resident,
            &resident,
            &directive,
            &anchor,
            &p,
            "2026-07-19T12:00:02Z",
        )?;
        let pressures = resident_self_pressures(&resident)?;
        assert_eq!(pressures.len(), 1);
        assert_eq!(pressures[0].kind, "operator-objective");
        assert!(pressures[0].consumed_by_grant_id.is_none());
        assert_eq!(
            admit_and_execute_bifrost_operator_command(
                &commands,
                &verse,
                &resident,
                &resident,
                &directive,
                &anchor,
                &p,
                "2026-07-19T12:00:03Z"
            )?,
            first
        );
        assert_eq!(resident_self_pressures(&resident)?.len(), 1);
        Ok(())
    }

    #[test]
    fn wake_cannot_release_the_deployment_brake() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let signer = crate::enroll_host_identity_at(&temp.path().join("identity.cc"))?;
        let anchor =
            crate::export_host_identity_trust_anchor(&signer, &temp.path().join("anchor.cc"))?;
        let (commands, verse, resident) = (
            temp.path().join("commands.cc"),
            temp.path().join("verse.cc"),
            temp.path().join("resident.cc"),
        );
        let mut deployment = crate::default_epiphany_cultmesh_swarm_brake("2026-07-19T11:00:00Z");
        deployment.brake_id = "epiphany-yggdrasil/deployment-brake".into();
        deployment.runtime_id = "epiphany-yggdrasil".into();
        crate::write_epiphany_cultmesh_swarm_brake(&verse, "epiphany-yggdrasil", deployment)?;
        let before = std::fs::read(&verse)?;
        let wake = signed(&signer, "foreign-wake", OperatorCommand::Wake)?;
        let refused = admit_and_execute_bifrost_operator_command(
            &commands,
            &verse,
            &resident,
            &resident,
            &wake,
            &anchor,
            &policy(),
            "2026-07-19T12:00:00Z",
        )?;
        assert_eq!(refused.disposition, "refused");
        assert_eq!(std::fs::read(&verse)?, before);
        assert!(!resident.exists());
        assert_eq!(
            admit_and_execute_bifrost_operator_command(
                &commands,
                &verse,
                &resident,
                &resident,
                &wake,
                &anchor,
                &policy(),
                "2026-07-19T12:00:01Z"
            )?,
            refused
        );
        assert_eq!(std::fs::read(&verse)?, before);
        Ok(())
    }

    #[test]
    fn replay_recovers_after_admission_and_consequence_without_result() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let signer = crate::enroll_host_identity_at(&temp.path().join("identity.cc"))?;
        let anchor =
            crate::export_host_identity_trust_anchor(&signer, &temp.path().join("anchor.cc"))?;
        let (commands, verse, resident) = (
            temp.path().join("commands.cc"),
            temp.path().join("verse.cc"),
            temp.path().join("resident.cc"),
        );
        let admission = signed(
            &signer,
            "crash-recovery",
            OperatorCommand::Directive {
                objective: "Recover the exact pressure.".into(),
            },
        )?;
        let packet = &admission.packet;
        let admitted = LocalAdmittedOperatorCommand {
            schema_version: LOCAL_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION.into(),
            command_id: packet.command_id.clone(),
            admission_id: admission.admission_id.clone(),
            nonce: packet.nonce.clone(),
            packet_sha256: admission.packet_sha256.clone(),
            source_actor_id: packet.source_actor_id.clone(),
            capability: "directive".into(),
            target_runtime_id: packet.target_runtime_id.clone(),
            admitted_at: "2026-07-19T12:00:00Z".into(),
            expires_at: packet.expires_at.clone(),
            authority: "exact-command-only".into(),
            private_state_exposed: false,
        };
        let cache = command_cache(&commands)?;
        let (entry, _) = cache.prepare_entry(&packet.command_id, &admitted)?;
        assert!(SingleFileMessagePackBackingStore::new(&commands).insert_entry_if_absent(entry)?);
        let pressure = ResidentSelfPressure {
            schema_version: crate::RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
            pressure_id: format!("operator-command-{}", packet.command_id),
            kind: "operator-objective".into(),
            provenance_ref: format!("cultcache://operator-command/{}", packet.command_id),
            objective: "Recover the exact pressure.".into(),
            created_at_millis: issued_millis(&packet.issued_at)?,
            status: "pending".into(),
            consumed_by_grant_id: None,
            private_state_exposed: false,
        };
        enqueue_resident_self_pressure(&resident, &pressure)?;
        let result = admit_and_execute_bifrost_operator_command(
            &commands,
            &verse,
            &resident,
            &resident,
            &admission,
            &anchor,
            &policy(),
            "2026-07-19T12:06:00Z",
        )?;
        assert_eq!(result.disposition, "applied");
        assert_eq!(resident_self_pressures(&resident)?, vec![pressure]);
        assert_eq!(
            admit_and_execute_bifrost_operator_command(
                &commands,
                &verse,
                &resident,
                &resident,
                &admission,
                &anchor,
                &policy(),
                "2026-07-19T12:00:04Z"
            )?,
            result
        );
        Ok(())
    }

    #[test]
    fn review_replay_recovers_after_mind_consequence_without_command_result() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let (runtime, planning, _, request) =
            crate::coordinator_launch::tests::operator_review_candidate_fixture(
                temp.path(),
                "operator-command-crash",
            )?;
        let signer = crate::enroll_host_identity_at(&temp.path().join("identity.cc"))?;
        let anchor =
            crate::export_host_identity_trust_anchor(&signer, &temp.path().join("anchor.cc"))?;
        let (commands, verse, resident) = (
            temp.path().join("commands.cc"),
            temp.path().join("verse.cc"),
            temp.path().join("resident.cc"),
        );
        let admission = signed(
            &signer,
            "review-crash-recovery",
            OperatorCommand::Review {
                mind_request_id: request.request_id.clone(),
                candidate_id: request.candidate_id.clone(),
                candidate_sha256: request.candidate_sha256.clone(),
                expected_model_revision: planning.model_revision,
                expected_model_hash: planning.model_hash.clone(),
                decision: RepoFrontierPlanDecision::Hold,
            },
        )?;
        let packet = &admission.packet;
        let admitted = LocalAdmittedOperatorCommand {
            schema_version: LOCAL_OPERATOR_COMMAND_ADMISSION_SCHEMA_VERSION.into(),
            command_id: packet.command_id.clone(),
            admission_id: admission.admission_id.clone(),
            nonce: packet.nonce.clone(),
            packet_sha256: admission.packet_sha256.clone(),
            source_actor_id: packet.source_actor_id.clone(),
            capability: "review".into(),
            target_runtime_id: packet.target_runtime_id.clone(),
            admitted_at: "2026-07-19T12:00:00Z".into(),
            expires_at: packet.expires_at.clone(),
            authority: "exact-command-only".into(),
            private_state_exposed: false,
        };
        let cache = command_cache(&commands)?;
        let (entry, _) = cache.prepare_entry(&packet.command_id, &admitted)?;
        assert!(SingleFileMessagePackBackingStore::new(&commands).insert_entry_if_absent(entry)?);
        let canonical = commit_operator_repo_frontier_plan_review(
            &runtime,
            &RepoFrontierPlanOperatorReview {
                command_id: packet.command_id.clone(),
                admission_id: admission.admission_id.clone(),
                packet_sha256: admission.packet_sha256.clone(),
                source_actor_id: packet.source_actor_id.clone(),
                mind_request_id: request.request_id,
                candidate_id: request.candidate_id,
                candidate_sha256: request.candidate_sha256,
                expected_model_revision: planning.model_revision,
                expected_model_hash: planning.model_hash,
                decision: RepoFrontierPlanDecision::Hold,
                decided_at: packet.issued_at.clone(),
            },
        )?;
        let recovered = admit_and_execute_bifrost_operator_command(
            &commands,
            &verse,
            &resident,
            &runtime,
            &admission,
            &anchor,
            &policy(),
            "2026-07-19T12:00:00Z",
        )?;
        assert_eq!(recovered.disposition, "applied");
        assert_eq!(recovered.consequence_ref, canonical.decision_id);
        assert_eq!(recovered.review_decision, "hold");
        assert_eq!(
            admit_and_execute_bifrost_operator_command(
                &commands,
                &verse,
                &resident,
                &runtime,
                &admission,
                &anchor,
                &policy(),
                "2026-07-19T12:00:01Z",
            )?,
            recovered
        );
        Ok(())
    }
}
