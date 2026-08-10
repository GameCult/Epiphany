use std::path::Path;

use anyhow::{Result, anyhow};
use cultcache_rs::{
    CacheBackingStore, CultCache, CultCacheEnvelope, DatabaseEntry,
    SingleFileMessagePackBackingStore,
};
use cultnet_rs::{GameCultServiceTrustAnchorRecord, ServiceIdentitySigner};
use sha2::{Digest, Sha256};

use crate::agent_memory::SelfPatchMemory;
use crate::heartbeat_state::{
    PERSONA_CONVERSATION_RETENTION_HEAD_SCHEMA_VERSION,
    PERSONA_CONVERSATION_RETENTION_PLAN_SCHEMA_VERSION, PersonaConversationRetentionEnvelope,
    PersonaConversationRetentionHead, PersonaConversationRetentionMember,
    PersonaConversationRetentionPlan, commit_heartbeat_state_transaction,
    load_heartbeat_state_transaction,
};
use crate::{
    AgentSelfPatch, EpiphanyPersonaDeliveryRequestIdentity, PersonaInterpreterEffect,
    PersonaInterpreterEffectDocument, PersonaModelStageReceipt, PersonaModelTerminalReceipt,
    PersonaTurnRequest, PersonaTurnTerminalOptions, apply_agent_self_patch_document,
    complete_persona_turn_request_store, insert_persona_discord_delivery_request,
    load_persona_discord_delivery_receipt, runtime_spine_cache,
    sign_persona_discord_delivery_request, verify_persona_discord_delivery_receipt,
};

pub const PERSONA_DISCORD_DELIVERY_EVIDENCE_SCHEMA_VERSION: &str =
    "epiphany.persona_discord_delivery_evidence.v0";
pub const PERSONA_CONVERSATION_EXECUTION_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.persona_conversation_execution_receipt.v0";
pub const PERSONA_EFFECT_EXECUTION_INTENT_SCHEMA_VERSION: &str =
    "epiphany.persona_effect_execution_intent.v0";
pub const PERSONA_CONVERSATION_STORE_RETIREMENT_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.persona_conversation_store_retirement_receipt.v0";

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.persona_effect_execution_intent.v0",
    schema = "PersonaEffectExecutionIntent"
)]
pub struct PersonaEffectExecutionIntent {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub intent_id: String,
    #[cultcache(key = 2)]
    pub request_id: String,
    #[cultcache(key = 3)]
    pub effect_document_id: String,
    #[cultcache(key = 4)]
    pub effect_index: u64,
    #[cultcache(key = 5)]
    pub effect_kind: String,
    #[cultcache(key = 6)]
    pub status: String,
    #[cultcache(key = 7)]
    pub updated_at: String,
    #[cultcache(key = 8)]
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.persona_discord_delivery_evidence.v0",
    schema = "PersonaDiscordDeliveryEvidence"
)]
pub struct PersonaDiscordDeliveryEvidence {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub evidence_id: String,
    #[cultcache(key = 2)]
    pub effect_document_id: String,
    #[cultcache(key = 3)]
    pub channel_id: String,
    #[cultcache(key = 4, default)]
    pub reply_to_message_id: Option<String>,
    #[cultcache(key = 5)]
    pub message_id: String,
    #[cultcache(key = 6)]
    pub transport: String,
    #[cultcache(key = 7)]
    pub crossing_receipt_id: String,
    #[cultcache(key = 8)]
    pub receipt_url: String,
    #[cultcache(key = 9)]
    pub bridge_receipt_sha256: String,
    #[cultcache(key = 10)]
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.persona_conversation_execution_receipt.v0",
    schema = "PersonaConversationExecutionReceipt"
)]
pub struct PersonaConversationExecutionReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub request_id: String,
    #[cultcache(key = 3)]
    pub effect_document_id: String,
    #[cultcache(key = 4)]
    pub outcome: String,
    #[cultcache(key = 5)]
    pub state_effect_status: String,
    #[cultcache(key = 6, default)]
    pub state_effect_reasons: Vec<String>,
    #[cultcache(key = 7, default)]
    pub delivery_evidence_ids: Vec<String>,
    #[cultcache(key = 8, default)]
    pub heartbeat_terminal_receipt_id: Option<String>,
    #[cultcache(key = 9)]
    pub private_state_exposed: bool,
}

/// Advances one reserved Persona turn across the signed Epiphany→Bifrost
/// request/receipt boundary. `None` means a request is durably pending.
#[allow(clippy::too_many_arguments)]
pub fn poll_persona_discord_crossing(
    runtime_store: &Path,
    heartbeat_store: &Path,
    agent_store: &Path,
    cultmesh_store: &Path,
    runtime_id: &str,
    request_store: &Path,
    receipt_store: &Path,
    signer: &ServiceIdentitySigner<EpiphanyPersonaDeliveryRequestIdentity>,
    receipt_anchor: &GameCultServiceTrustAnchorRecord,
    request_id: &str,
    effect_document_id: &str,
) -> Result<Option<PersonaConversationExecutionReceipt>> {
    let receipt_id = format!("persona-conversation:{request_id}");
    if let Some(existing) =
        runtime_document::<PersonaConversationExecutionReceipt>(runtime_store, &receipt_id)?
    {
        return Ok(Some(existing));
    }
    let request = load_reserved_request(heartbeat_store, request_id)?;
    let effects =
        runtime_document::<PersonaInterpreterEffectDocument>(runtime_store, effect_document_id)?
            .ok_or_else(|| anyhow!("Persona Interpreter effect document is missing"))?;
    validate_model_terminal(runtime_store, &request, &effects)?;
    require_persona_effects_unbraked(cultmesh_store, runtime_id)?;
    let say = effects
        .effects
        .iter()
        .enumerate()
        .find_map(|(index, effect)| {
            if let PersonaInterpreterEffect::Say {
                channel_id,
                reply_to_message_id,
                content,
                ..
            } = effect
            {
                Some((index, channel_id, reply_to_message_id, content))
            } else {
                None
            }
        });
    let Some((index, channel_id, reply_to_message_id, content)) = say else {
        // Non-speech turns retain the existing typed state-admission path.
        let (state_status, reasons) = match admit_state_notes(
            runtime_store,
            agent_store,
            cultmesh_store,
            runtime_id,
            &request,
            &effects,
        ) {
            Ok(value) => value,
            Err(error) if error.to_string().contains("quarantined") => {
                return terminalize_local_effect_quarantine(
                    runtime_store,
                    heartbeat_store,
                    &request,
                    &effects,
                    error,
                );
            }
            Err(error) => return Err(error),
        };
        let outcome = if effects
            .effects
            .iter()
            .all(|effect| matches!(effect, PersonaInterpreterEffect::Drop { .. }))
        {
            "dropped"
        } else {
            "silence"
        };
        let terminal = complete_persona_turn_request_store(
            heartbeat_store,
            PersonaTurnTerminalOptions {
                request_id: request_id.into(),
                outcome: outcome.into(),
                delivery_evidence: None,
                blocked_evidence: None,
            },
        )?;
        let receipt = PersonaConversationExecutionReceipt {
            schema_version: PERSONA_CONVERSATION_EXECUTION_RECEIPT_SCHEMA_VERSION.into(),
            receipt_id,
            request_id: request_id.into(),
            effect_document_id: effect_document_id.into(),
            outcome: outcome.into(),
            state_effect_status: state_status,
            state_effect_reasons: reasons,
            delivery_evidence_ids: vec![],
            heartbeat_terminal_receipt_id: Some(terminal.receipt_id),
            private_state_exposed: false,
        };
        put_runtime_document(runtime_store, &receipt.receipt_id, &receipt)?;
        return Ok(Some(receipt));
    };
    let crossing_request_id = format!("persona-discord:{request_id}:{index}");
    let reply = resolve_reply_target(&request, reply_to_message_id.as_deref())?.unwrap_or_default();
    let crossing = if let Some(existing) =
        crate::load_persona_discord_delivery_request(request_store, &crossing_request_id)?
    {
        existing
    } else {
        if let Err(error) = admit_state_notes(
            runtime_store,
            agent_store,
            cultmesh_store,
            runtime_id,
            &request,
            &effects,
        ) {
            if error.to_string().contains("quarantined") {
                return terminalize_local_effect_quarantine(
                    runtime_store,
                    heartbeat_store,
                    &request,
                    &effects,
                    error,
                );
            }
            return Err(error);
        }
        begin_effect(runtime_store, &request, &effects, index, "say")?;
        let issued = chrono::Utc::now();
        let signed = sign_persona_discord_delivery_request(
            signer,
            crossing_request_id.clone(),
            effects.document_id.clone(),
            runtime_id.into(),
            request.agent_id.clone(),
            channel_id.clone(),
            reply.clone(),
            content.clone(),
            issued.to_rfc3339(),
            (issued + chrono::Duration::seconds(90)).to_rfc3339(),
        )?;
        require_persona_effects_unbraked(cultmesh_store, runtime_id)?;
        insert_persona_discord_delivery_request(request_store, &signed)?;
        signed
    };
    let Some(delivery) =
        load_persona_discord_delivery_receipt(receipt_store, &crossing_request_id)?
    else {
        return Ok(None);
    };
    verify_persona_discord_delivery_receipt(&delivery, &crossing, receipt_anchor)?;
    let signed_receipt_sha256 =
        format!("sha256:{:x}", Sha256::digest(rmp_serde::to_vec(&delivery)?));
    let (outcome, blocked_evidence) = match delivery.status.as_str() {
        "completed" => ("delivered", None),
        "failed" | "unknown" => (
            "blocked",
            Some(crate::PersonaTurnBlockedEvidence {
                evidence_source: "bifrost_crossing".into(),
                crossing_status: delivery.status.clone(),
                reason: format!(
                    "Signed Bifrost Discord receipt {} for request {} terminated as {} and cannot be retried automatically",
                    delivery.receipt_id, delivery.request_id, delivery.status
                ),
                crossing_receipt_id: (!delivery.crossing_receipt_id.trim().is_empty())
                    .then(|| delivery.crossing_receipt_id.clone()),
                bridge_receipt_sha256: Some(signed_receipt_sha256.clone()),
            }),
        ),
        _ => unreachable!(),
    };
    let evidence = (outcome == "delivered").then(|| PersonaDiscordDeliveryEvidence {
        schema_version: PERSONA_DISCORD_DELIVERY_EVIDENCE_SCHEMA_VERSION.into(),
        evidence_id: format!(
            "persona-delivery:{}:{}",
            effects.document_id, delivery.message_id
        ),
        effect_document_id: effects.document_id.clone(),
        channel_id: delivery.channel_id.clone(),
        reply_to_message_id: (!delivery.reply_to_message_id.is_empty())
            .then(|| delivery.reply_to_message_id.clone()),
        message_id: delivery.message_id.clone(),
        transport: delivery.transport.clone(),
        crossing_receipt_id: delivery.crossing_receipt_id.clone(),
        receipt_url: delivery.receipt_url.clone(),
        bridge_receipt_sha256: signed_receipt_sha256,
        private_state_exposed: false,
    });
    if let Some(value) = &evidence {
        put_runtime_document(runtime_store, &value.evidence_id, value)?;
    }
    let terminal = complete_persona_turn_request_store(
        heartbeat_store,
        PersonaTurnTerminalOptions {
            request_id: request_id.into(),
            outcome: outcome.into(),
            delivery_evidence: evidence.clone(),
            blocked_evidence,
        },
    )?;
    let intent_id = format!("persona-effect-intent:{}:{index}", effects.document_id);
    if let Some(mut intent) =
        runtime_document::<PersonaEffectExecutionIntent>(runtime_store, &intent_id)?
    {
        finish_effect(runtime_store, &mut intent)?;
    }
    let receipt = PersonaConversationExecutionReceipt {
        schema_version: PERSONA_CONVERSATION_EXECUTION_RECEIPT_SCHEMA_VERSION.into(),
        receipt_id,
        request_id: request_id.into(),
        effect_document_id: effect_document_id.into(),
        outcome: outcome.into(),
        state_effect_status: "admitted_before_delivery_request".into(),
        state_effect_reasons: vec![],
        delivery_evidence_ids: evidence
            .iter()
            .map(|value| value.evidence_id.clone())
            .collect(),
        heartbeat_terminal_receipt_id: Some(terminal.receipt_id),
        private_state_exposed: false,
    };
    put_runtime_document(runtime_store, &receipt.receipt_id, &receipt)?;
    Ok(Some(receipt))
}

fn terminalize_local_effect_quarantine(
    runtime_store: &Path,
    heartbeat_store: &Path,
    request: &PersonaTurnRequest,
    effects: &PersonaInterpreterEffectDocument,
    error: anyhow::Error,
) -> Result<Option<PersonaConversationExecutionReceipt>> {
    let terminal = complete_persona_turn_request_store(
        heartbeat_store,
        PersonaTurnTerminalOptions {
            request_id: request.request_id.clone(),
            outcome: "blocked".into(),
            delivery_evidence: None,
            blocked_evidence: Some(crate::PersonaTurnBlockedEvidence {
                evidence_source: "local_effect".into(),
                crossing_status: "unknown".into(),
                reason: error.to_string(),
                crossing_receipt_id: None,
                bridge_receipt_sha256: None,
            }),
        },
    )?;
    let receipt = PersonaConversationExecutionReceipt {
        schema_version: PERSONA_CONVERSATION_EXECUTION_RECEIPT_SCHEMA_VERSION.into(),
        receipt_id: format!("persona-conversation:{}", request.request_id),
        request_id: request.request_id.clone(),
        effect_document_id: effects.document_id.clone(),
        outcome: "blocked".into(),
        state_effect_status: "quarantined_ambiguous_local_effect".into(),
        state_effect_reasons: vec![error.to_string()],
        delivery_evidence_ids: vec![],
        heartbeat_terminal_receipt_id: Some(terminal.receipt_id),
        private_state_exposed: false,
    };
    put_runtime_document(runtime_store, &receipt.receipt_id, &receipt)?;
    Ok(Some(receipt))
}

/// Repairs the local execution projection after the heartbeat terminal commit
/// won a crash race. The heartbeat terminal remains authority; this function
/// only finishes matching typed intents and restores its derived receipt.
pub fn reconcile_terminal_persona_conversation(
    runtime_store: &Path,
    heartbeat_store: &Path,
    request_id: &str,
) -> Result<Option<PersonaConversationExecutionReceipt>> {
    let receipt_id = format!("persona-conversation:{request_id}");
    if let Some(existing) = runtime_document(runtime_store, &receipt_id)? {
        return Ok(Some(existing));
    }
    let state = crate::heartbeat_state::load_heartbeat_state_entry(heartbeat_store)?
        .ok_or_else(|| anyhow!("heartbeat state is missing"))?;
    let Some(request) = state
        .persona_turn_requests
        .into_iter()
        .find(|value| value.request_id == request_id)
    else {
        return Ok(None);
    };
    let Some(terminal) = request.terminal_receipt.as_ref() else {
        return Ok(None);
    };
    let effect_document_id = format!("persona-effects:{request_id}");
    let Some(effects) =
        runtime_document::<PersonaInterpreterEffectDocument>(runtime_store, &effect_document_id)?
    else {
        return Ok(None);
    };
    validate_model_terminal(runtime_store, &request, &effects)?;
    for index in 0..effects.effects.len() {
        let intent_id = format!("persona-effect-intent:{}:{index}", effects.document_id);
        if let Some(mut intent) =
            runtime_document::<PersonaEffectExecutionIntent>(runtime_store, &intent_id)?
        {
            if intent.status == "started" {
                finish_effect(runtime_store, &mut intent)?;
            }
        }
    }
    let receipt = PersonaConversationExecutionReceipt {
        schema_version: PERSONA_CONVERSATION_EXECUTION_RECEIPT_SCHEMA_VERSION.into(),
        receipt_id,
        request_id: request_id.into(),
        effect_document_id,
        outcome: terminal.outcome.clone(),
        state_effect_status: "reconciled_from_heartbeat_terminal".into(),
        state_effect_reasons: vec![],
        delivery_evidence_ids: terminal.delivery_evidence_id.clone().into_iter().collect(),
        heartbeat_terminal_receipt_id: Some(terminal.receipt_id.clone()),
        private_state_exposed: false,
    };
    put_runtime_document(runtime_store, &receipt.receipt_id, &receipt)?;
    Ok(Some(receipt))
}

pub fn persona_model_terminal_exists(runtime_store: &Path, request_id: &str) -> Result<bool> {
    Ok(runtime_document::<PersonaModelTerminalReceipt>(
        runtime_store,
        &format!("persona-terminal:{request_id}"),
    )?
    .is_some())
}

pub fn persona_delivery_receipt_exists_for_turn(
    receipt_store: &Path,
    request_id: &str,
) -> Result<bool> {
    for index in 0..16 {
        if crate::load_persona_discord_delivery_receipt(
            receipt_store,
            &format!("persona-discord:{request_id}:{index}"),
        )?
        .is_some()
        {
            return Ok(true);
        }
    }
    Ok(false)
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.persona_conversation_store_retirement_receipt.v0",
    schema = "PersonaConversationStoreRetirementReceipt"
)]
pub struct PersonaConversationStoreRetirementReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub store_role: String,
    #[cultcache(key = 3)]
    pub plan_id: String,
    #[cultcache(key = 4)]
    pub deleted_envelope_count: u64,
    #[cultcache(key = 5)]
    pub deleted_envelopes_sha256: String,
    #[cultcache(key = 6)]
    pub completed_at: String,
    #[cultcache(key = 7)]
    pub private_state_exposed: bool,
}

pub fn pending_persona_discord_delivery_request_for_turn(
    request_store: &Path,
    request_id: &str,
) -> Result<Option<crate::PersonaDiscordDeliveryRequest>> {
    for index in 0..16 {
        if let Some(request) = crate::load_persona_discord_delivery_request(
            request_store,
            &format!("persona-discord:{request_id}:{index}"),
        )? {
            return Ok(Some(request));
        }
    }
    Ok(None)
}

#[allow(clippy::too_many_arguments)]
pub fn retain_terminal_persona_conversations(
    runtime_store: &Path,
    heartbeat_store: &Path,
    request_store: &Path,
    receipt_store: &Path,
    receipt_anchor: &GameCultServiceTrustAnchorRecord,
    retain_turns: usize,
    retained_at: &str,
) -> Result<Option<PersonaConversationRetentionHead>> {
    chrono::DateTime::parse_from_rfc3339(retained_at)
        .map_err(|_| anyhow!("Persona conversation retention time must be RFC3339"))?;
    let (loaded, expected) = load_heartbeat_state_transaction(heartbeat_store)?;
    let mut state = loaded.ok_or_else(|| anyhow!("heartbeat state is missing"))?;
    if state.persona_conversation_retention_plan.is_none() {
        let mut terminal = state
            .persona_turn_requests
            .iter()
            .filter(|request| request.terminal_receipt.is_some())
            .collect::<Vec<_>>();
        terminal.sort_by(|left, right| left.reserved_at.cmp(&right.reserved_at));
        let oldest_live = state
            .persona_turn_requests
            .iter()
            .filter(|request| request.terminal_receipt.is_none())
            .map(|request| request.reserved_at.as_str())
            .min();
        let eligible = terminal
            .into_iter()
            .filter(|request| {
                oldest_live.is_none_or(|frontier| request.reserved_at.as_str() < frontier)
            })
            .filter_map(|request| {
                build_persona_retention_member(
                    runtime_store,
                    request_store,
                    receipt_store,
                    receipt_anchor,
                    request,
                )
                .transpose()
            })
            .collect::<Result<Vec<_>>>()?;
        let retire_count = eligible.len().saturating_sub(retain_turns);
        if retire_count == 0 {
            return Ok(None);
        }
        let members = eligible.into_iter().take(retire_count).collect::<Vec<_>>();
        let mut digest = Sha256::new();
        digest.update(b"epiphany.persona-conversation-retention-plan.v0\0");
        digest.update(rmp_serde::to_vec(&members)?);
        let plan = PersonaConversationRetentionPlan {
            schema_version: PERSONA_CONVERSATION_RETENTION_PLAN_SCHEMA_VERSION.into(),
            plan_id: format!("sha256:{:x}", digest.finalize()),
            members,
            planned_at: retained_at.into(),
            private_state_exposed: false,
        };
        state.persona_conversation_retention_plan = Some(plan);
        commit_heartbeat_state_transaction(heartbeat_store, expected, &state)?;
    }
    reconcile_persona_conversation_retention(
        runtime_store,
        heartbeat_store,
        request_store,
        receipt_store,
        retained_at,
    )
    .map(Some)
}

fn build_persona_retention_member(
    runtime_store: &Path,
    request_store: &Path,
    receipt_store: &Path,
    receipt_anchor: &GameCultServiceTrustAnchorRecord,
    request: &PersonaTurnRequest,
) -> Result<Option<PersonaConversationRetentionMember>> {
    let terminal = request
        .terminal_receipt
        .as_ref()
        .ok_or_else(|| anyhow!("Persona retention candidate is not terminal"))?;
    if !matches!(
        terminal.outcome.as_str(),
        "delivered" | "silence" | "dropped"
    ) {
        return Ok(None);
    }
    let receipt_id = format!("persona-conversation:{}", request.request_id);
    let Some(conversation) =
        runtime_document::<PersonaConversationExecutionReceipt>(runtime_store, &receipt_id)?
    else {
        return Ok(None);
    };
    if conversation.request_id != request.request_id
        || conversation.outcome != terminal.outcome
        || conversation.heartbeat_terminal_receipt_id.as_deref()
            != Some(terminal.receipt_id.as_str())
        || conversation.private_state_exposed
    {
        return Err(anyhow!(
            "Persona conversation retention receipt binding is invalid"
        ));
    }
    let Some(effects) = runtime_document::<PersonaInterpreterEffectDocument>(
        runtime_store,
        &conversation.effect_document_id,
    )?
    else {
        return Ok(None);
    };
    validate_model_terminal(runtime_store, request, &effects)?;
    let model_terminal_id = format!("persona-terminal:{}", request.request_id);
    let model_terminal =
        runtime_document::<PersonaModelTerminalReceipt>(runtime_store, &model_terminal_id)?
            .ok_or_else(|| anyhow!("Persona model terminal disappeared during retention"))?;
    let mut runtime_ids = vec![
        (
            <PersonaConversationExecutionReceipt as DatabaseEntry>::TYPE,
            receipt_id,
        ),
        (
            <PersonaInterpreterEffectDocument as DatabaseEntry>::TYPE,
            effects.document_id.clone(),
        ),
        (
            <PersonaModelTerminalReceipt as DatabaseEntry>::TYPE,
            model_terminal_id,
        ),
    ];
    runtime_ids.extend(
        model_terminal
            .stage_receipt_ids
            .iter()
            .cloned()
            .map(|id| (<PersonaModelStageReceipt as DatabaseEntry>::TYPE, id)),
    );
    for (index, effect) in effects.effects.iter().enumerate() {
        if matches!(effect, PersonaInterpreterEffect::Drop { .. }) {
            continue;
        }
        let id = format!("persona-effect-intent:{}:{index}", effects.document_id);
        let Some(intent) = runtime_document::<PersonaEffectExecutionIntent>(runtime_store, &id)?
        else {
            return Ok(None);
        };
        if intent.status != "completed" || intent.private_state_exposed {
            return Ok(None);
        }
        runtime_ids.push((<PersonaEffectExecutionIntent as DatabaseEntry>::TYPE, id));
    }
    if let Some(evidence_id) = &terminal.delivery_evidence_id {
        let Some(evidence) =
            runtime_document::<PersonaDiscordDeliveryEvidence>(runtime_store, evidence_id)?
        else {
            return Ok(None);
        };
        if conversation.delivery_evidence_ids != [evidence_id.clone()]
            || evidence.private_state_exposed
        {
            return Err(anyhow!("Persona retained delivery evidence is invalid"));
        }
        runtime_ids.push((
            <PersonaDiscordDeliveryEvidence as DatabaseEntry>::TYPE,
            evidence_id.clone(),
        ));
    } else if terminal.outcome == "delivered" || !conversation.delivery_evidence_ids.is_empty() {
        return Err(anyhow!(
            "delivered Persona retention candidate lacks evidence"
        ));
    }
    let mut runtime_cache = runtime_spine_cache(runtime_store)?;
    runtime_cache.pull_all_backing_stores()?;
    let runtime_snapshot = runtime_cache.snapshot_envelopes();
    let runtime_envelopes = retention_envelopes(&runtime_snapshot, &runtime_ids)?;

    let mut crossing_request_envelopes = Vec::new();
    let mut crossing_receipt_envelopes = Vec::new();
    if terminal.outcome == "delivered" {
        let say_index = effects
            .effects
            .iter()
            .position(|effect| matches!(effect, PersonaInterpreterEffect::Say { .. }))
            .ok_or_else(|| anyhow!("delivered Persona turn has no speech effect"))?;
        let crossing_id = format!("persona-discord:{}:{say_index}", request.request_id);
        let Some(crossing_request) =
            crate::load_persona_discord_delivery_request(request_store, &crossing_id)?
        else {
            return Ok(None);
        };
        let Some(crossing_receipt) =
            crate::load_persona_discord_delivery_receipt(receipt_store, &crossing_id)?
        else {
            return Ok(None);
        };
        verify_persona_discord_delivery_receipt(
            &crossing_receipt,
            &crossing_request,
            receipt_anchor,
        )?;
        if crossing_receipt.status != "completed" {
            return Ok(None);
        }
        crossing_request_envelopes = retention_envelopes(
            &SingleFileMessagePackBackingStore::new(request_store).pull_all()?,
            &[(
                <crate::PersonaDiscordDeliveryRequest as DatabaseEntry>::TYPE,
                crossing_id.clone(),
            )],
        )?;
        crossing_receipt_envelopes = retention_envelopes(
            &SingleFileMessagePackBackingStore::new(receipt_store).pull_all()?,
            &[(
                <crate::PersonaDiscordDeliveryReceipt as DatabaseEntry>::TYPE,
                crossing_id,
            )],
        )?;
    }
    Ok(Some(PersonaConversationRetentionMember {
        request_id: request.request_id.clone(),
        reserved_at: request.reserved_at.clone(),
        completed_at: terminal.completed_at.clone(),
        outcome: terminal.outcome.clone(),
        terminal_receipt_sha256: digest_value(terminal)?,
        runtime_envelopes,
        crossing_request_envelopes,
        crossing_receipt_envelopes,
    }))
}

fn retention_envelopes(
    snapshot: &[CultCacheEnvelope],
    identities: &[(&str, String)],
) -> Result<Vec<PersonaConversationRetentionEnvelope>> {
    identities
        .iter()
        .map(|(envelope_type, key)| {
            let envelope = snapshot
                .iter()
                .find(|envelope| envelope.r#type == *envelope_type && envelope.key == *key)
                .ok_or_else(|| {
                    anyhow!("Persona retention envelope {envelope_type:?}/{key:?} is missing")
                })?;
            Ok(PersonaConversationRetentionEnvelope {
                envelope_type: (*envelope_type).into(),
                key: key.clone(),
                envelope_sha256: digest_value(envelope)?,
            })
        })
        .collect()
}

fn reconcile_persona_conversation_retention(
    runtime_store: &Path,
    heartbeat_store: &Path,
    request_store: &Path,
    receipt_store: &Path,
    retained_at: &str,
) -> Result<PersonaConversationRetentionHead> {
    let state = crate::heartbeat_state::load_heartbeat_state_entry(heartbeat_store)?
        .ok_or_else(|| anyhow!("heartbeat state is missing"))?;
    let plan = state
        .persona_conversation_retention_plan
        .clone()
        .ok_or_else(|| anyhow!("Persona conversation retention plan is missing"))?;
    let runtime_members = plan
        .members
        .iter()
        .flat_map(|member| member.runtime_envelopes.iter())
        .cloned()
        .collect::<Vec<_>>();
    delete_runtime_retention_envelopes(
        runtime_store,
        &plan.plan_id,
        &plan.planned_at,
        &runtime_members,
    )?;
    delete_snapshot_retention_envelopes(
        request_store,
        "crossing-request",
        &plan.plan_id,
        &plan.planned_at,
        &plan
            .members
            .iter()
            .flat_map(|member| member.crossing_request_envelopes.iter())
            .cloned()
            .collect::<Vec<_>>(),
    )?;
    delete_snapshot_retention_envelopes(
        receipt_store,
        "crossing-receipt",
        &plan.plan_id,
        &plan.planned_at,
        &plan
            .members
            .iter()
            .flat_map(|member| member.crossing_receipt_envelopes.iter())
            .cloned()
            .collect::<Vec<_>>(),
    )?;

    let (loaded, expected) = load_heartbeat_state_transaction(heartbeat_store)?;
    let mut state = loaded.ok_or_else(|| anyhow!("heartbeat state is missing"))?;
    if state.persona_conversation_retention_plan.as_ref() != Some(&plan) {
        return Err(anyhow!(
            "Persona conversation retention plan changed before commit"
        ));
    }
    let ids = plan
        .members
        .iter()
        .map(|member| member.request_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for member in &plan.members {
        let request = state
            .persona_turn_requests
            .iter()
            .find(|request| request.request_id == member.request_id)
            .ok_or_else(|| anyhow!("Persona retention turn disappeared before commit"))?;
        if digest_value(
            request
                .terminal_receipt
                .as_ref()
                .ok_or_else(|| anyhow!("Persona retention turn lost terminal receipt"))?,
        )? != member.terminal_receipt_sha256
        {
            return Err(anyhow!(
                "Persona terminal receipt changed before retention commit"
            ));
        }
    }
    state
        .persona_turn_requests
        .retain(|request| !ids.contains(request.request_id.as_str()));
    let prior = state.persona_conversation_retention_head.as_ref();
    let mut digest = Sha256::new();
    digest.update(b"epiphany.persona-conversation-retention-head.v0\0");
    digest.update(
        prior
            .map(|head| head.chained_digest.as_str())
            .unwrap_or("persona-conversation-retention-root")
            .as_bytes(),
    );
    digest.update(plan.plan_id.as_bytes());
    digest.update(rmp_serde::to_vec(&plan.members)?);
    let retired_through = plan
        .members
        .iter()
        .map(|member| member.reserved_at.as_str())
        .max()
        .unwrap_or_default();
    let through_reserved_at = prior
        .map(|head| head.through_reserved_at.as_str())
        .into_iter()
        .chain(std::iter::once(retired_through))
        .max()
        .unwrap_or_default()
        .to_string();
    let head = PersonaConversationRetentionHead {
        schema_version: PERSONA_CONVERSATION_RETENTION_HEAD_SCHEMA_VERSION.into(),
        revision: prior.map_or(1, |head| head.revision.saturating_add(1)),
        retired_turn_count: prior.map_or(plan.members.len() as u64, |head| {
            head.retired_turn_count
                .saturating_add(plan.members.len() as u64)
        }),
        through_reserved_at,
        chained_digest: format!("sha256:{:x}", digest.finalize()),
        retained_at: retained_at.into(),
        private_state_exposed: false,
    };
    state.persona_conversation_retention_head = Some(head.clone());
    state.persona_conversation_retention_plan = None;
    commit_heartbeat_state_transaction(heartbeat_store, expected, &state)?;
    Ok(head)
}

fn delete_runtime_retention_envelopes(
    store: &Path,
    plan_id: &str,
    completed_at: &str,
    members: &[PersonaConversationRetentionEnvelope],
) -> Result<()> {
    let mut cache = runtime_spine_cache(store)?;
    cache.pull_all_backing_stores()?;
    let snapshot = cache.snapshot_envelopes();
    let deletions = matching_retention_envelopes(&snapshot, members)?;
    let receipt = store_retirement_receipt("runtime", plan_id, completed_at, members)?;
    if deletions.is_empty() {
        require_store_retirement_receipt(&snapshot, &receipt)?;
        return Ok(());
    }
    let (replacement, _) = cache.prepare_entry(&receipt.receipt_id, &receipt)?;
    if !crate::runtime_store_backend::runtime_spine_backing_store(store)?
        .replace_and_delete_if_snapshot_unchanged(&snapshot, vec![replacement], &deletions)?
    {
        return Err(anyhow!("Persona runtime retention lost its snapshot fence"));
    }
    Ok(())
}

fn delete_snapshot_retention_envelopes(
    store: &Path,
    store_role: &str,
    plan_id: &str,
    completed_at: &str,
    members: &[PersonaConversationRetentionEnvelope],
) -> Result<()> {
    if members.is_empty() {
        return Ok(());
    }
    let backing = SingleFileMessagePackBackingStore::new(store);
    let snapshot = backing.pull_all()?;
    let deletions = matching_retention_envelopes(&snapshot, members)?;
    let receipt = store_retirement_receipt(store_role, plan_id, completed_at, members)?;
    if deletions.is_empty() {
        require_store_retirement_receipt(&snapshot, &receipt)?;
        return Ok(());
    }
    let mut cache = CultCache::new();
    cache.register_entry_type::<PersonaConversationStoreRetirementReceipt>()?;
    let (replacement, _) = cache.prepare_entry(&receipt.receipt_id, &receipt)?;
    if !backing.replace_and_delete_if_snapshot_unchanged(
        &snapshot,
        vec![replacement],
        &deletions,
    )? {
        return Err(anyhow!(
            "Persona crossing retention lost its snapshot fence"
        ));
    }
    Ok(())
}

fn require_store_retirement_receipt(
    snapshot: &[CultCacheEnvelope],
    expected: &PersonaConversationStoreRetirementReceipt,
) -> Result<()> {
    let envelope = snapshot
        .iter()
        .find(|envelope| {
            envelope.r#type == <PersonaConversationStoreRetirementReceipt as DatabaseEntry>::TYPE
                && envelope.key == expected.receipt_id
        })
        .ok_or_else(|| {
            anyhow!("Persona detail rows disappeared without a typed retirement receipt")
        })?;
    let mut cache = CultCache::new();
    cache.register_entry_type::<PersonaConversationStoreRetirementReceipt>()?;
    cache.load_envelope::<PersonaConversationStoreRetirementReceipt>(envelope.clone())?;
    if cache
        .get::<PersonaConversationStoreRetirementReceipt>(&expected.receipt_id)?
        .as_ref()
        != Some(expected)
    {
        return Err(anyhow!(
            "Persona store retirement receipt does not match the pending plan"
        ));
    }
    Ok(())
}

fn store_retirement_receipt(
    store_role: &str,
    plan_id: &str,
    completed_at: &str,
    members: &[PersonaConversationRetentionEnvelope],
) -> Result<PersonaConversationStoreRetirementReceipt> {
    Ok(PersonaConversationStoreRetirementReceipt {
        schema_version: PERSONA_CONVERSATION_STORE_RETIREMENT_RECEIPT_SCHEMA_VERSION.into(),
        receipt_id: format!("persona-conversation-retirement:{store_role}"),
        store_role: store_role.into(),
        plan_id: plan_id.into(),
        deleted_envelope_count: members.len() as u64,
        deleted_envelopes_sha256: digest_value(&members)?,
        completed_at: completed_at.into(),
        private_state_exposed: false,
    })
}

fn matching_retention_envelopes(
    snapshot: &[CultCacheEnvelope],
    members: &[PersonaConversationRetentionEnvelope],
) -> Result<Vec<CultCacheEnvelope>> {
    let mut deletions = Vec::new();
    for member in members {
        if let Some(envelope) = snapshot
            .iter()
            .find(|envelope| envelope.r#type == member.envelope_type && envelope.key == member.key)
        {
            if digest_value(envelope)? != member.envelope_sha256 {
                return Err(anyhow!("Persona retention envelope changed after planning"));
            }
            deletions.push(envelope.clone());
        }
    }
    Ok(deletions)
}

fn digest_value<T: serde::Serialize>(value: &T) -> Result<String> {
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(rmp_serde::to_vec(value)?)
    ))
}

fn admit_state_notes(
    runtime_store: &Path,
    agent_store: &Path,
    cultmesh_store: &Path,
    runtime_id: &str,
    request: &PersonaTurnRequest,
    document: &PersonaInterpreterEffectDocument,
) -> Result<(String, Vec<String>)> {
    let mut semantic = Vec::new();
    let mut relationships = Vec::new();
    let mut pending = Vec::new();
    let mut journals = Vec::new();
    for (index, effect) in document.effects.iter().enumerate() {
        let PersonaInterpreterEffect::StateNote {
            memory_kind,
            summary,
            confidence,
            ..
        } = effect
        else {
            continue;
        };
        let journal = begin_effect(runtime_store, request, document, index, "state_note")?;
        if journal.status == "completed" {
            continue;
        }
        journals.push(journal);
        let memory = SelfPatchMemory {
            memory_id: stable_memory_id(&document.document_id, index),
            summary: summary.clone(),
            salience: 0.7,
            confidence: confidence.unwrap_or(0.7),
            linked_event_ids: Some(vec![document.document_id.clone()]),
            linked_relationship_id: None,
        };
        match memory_kind.as_str() {
            "memory" => semantic.push(memory),
            "social_read" | "bond" => relationships.push(memory),
            other => pending.push(format!(
                "state_note kind {other:?} awaits a coherent typed Persona-state mapping"
            )),
        }
    }
    if semantic.is_empty() && relationships.is_empty() {
        for journal in &mut journals {
            finish_effect(runtime_store, journal)?;
        }
        return Ok((
            if pending.is_empty() {
                "none"
            } else {
                "pending"
            }
            .into(),
            pending,
        ));
    }
    let patch = AgentSelfPatch {
        agent_id: Some(request.agent_id.clone()),
        reason: Some(
            "Persona Interpreter proposed bounded memory effects after a completed natural turn."
                .into(),
        ),
        evidence_ids: Some(vec![document.document_id.clone()]),
        semantic_memories: (!semantic.is_empty()).then_some(semantic),
        relationship_memories: (!relationships.is_empty()).then_some(relationships),
        ..Default::default()
    };
    require_persona_effects_unbraked(cultmesh_store, runtime_id)?;
    let review = apply_agent_self_patch_document(&request.role_id, patch, agent_store)?;
    if review.status == "accepted" && review.applied == Some(true) {
        for journal in &mut journals {
            finish_effect(runtime_store, journal)?;
        }
        Ok((
            if pending.is_empty() {
                "admitted"
            } else {
                "partially_admitted"
            }
            .into(),
            pending,
        ))
    } else {
        pending.extend(review.reasons);
        for journal in &mut journals {
            finish_effect(runtime_store, journal)?;
        }
        Ok(("pending".into(), pending))
    }
}

fn validate_model_terminal(
    runtime_store: &Path,
    request: &PersonaTurnRequest,
    effects: &PersonaInterpreterEffectDocument,
) -> Result<()> {
    let terminal_id = format!("persona-terminal:{}", effects.turn_id);
    let terminal = runtime_document::<PersonaModelTerminalReceipt>(runtime_store, &terminal_id)?
        .ok_or_else(|| anyhow!("exact Persona model terminal receipt is missing"))?;
    let effect_digest = format!("sha256:{:x}", Sha256::digest(serde_json::to_vec(effects)?));
    if terminal.receipt_id != terminal_id
        || terminal.turn_id != effects.turn_id
        || effects.turn_id != request.request_id
        || effects.identity_id != request.agent_id
        || terminal.effect_document_id != effects.document_id
        || terminal.identity_id != request.agent_id
        || terminal.effect_document_sha256 != effect_digest
        || terminal.private_state_exposed
        || terminal.stage_receipt_ids.len() != 3
        || terminal.stage_output_sha256.len() != 3
    {
        return Err(anyhow!("Persona model terminal binding is invalid"));
    }
    for (index, expected) in ["projector", "persona", "interpreter"]
        .into_iter()
        .enumerate()
    {
        let expected_receipt_id = format!("persona-stage:{}:{expected}", effects.turn_id);
        let receipt_id = &terminal.stage_receipt_ids[index];
        if receipt_id != &expected_receipt_id {
            return Err(anyhow!("Persona {expected} stage receipt id is invalid"));
        }
        let receipt = runtime_document::<PersonaModelStageReceipt>(runtime_store, receipt_id)?
            .ok_or_else(|| anyhow!("Persona {expected} stage receipt is missing"))?;
        if receipt.receipt_id != expected_receipt_id
            || receipt.stage != expected
            || receipt.turn_id != effects.turn_id
            || receipt.request_id != format!("persona:{}:{expected}", effects.turn_id)
            || receipt.provider.is_empty()
            || receipt.model.is_empty()
            || !valid_sha256(&receipt.prompt_sha256)
            || !receipt.output_sha256.starts_with("sha256:")
            || !valid_sha256(&receipt.output_sha256)
            || receipt.output_sha256 != terminal.stage_output_sha256[index]
            || receipt.private_output_ref.is_empty()
            || receipt.private_state_exposed
        {
            return Err(anyhow!(
                "Persona {expected} stage digest binding is invalid"
            ));
        }
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn begin_effect(
    runtime_store: &Path,
    request: &PersonaTurnRequest,
    document: &PersonaInterpreterEffectDocument,
    index: usize,
    kind: &str,
) -> Result<PersonaEffectExecutionIntent> {
    let id = format!("persona-effect-intent:{}:{index}", document.document_id);
    if let Some(mut existing) =
        runtime_document::<PersonaEffectExecutionIntent>(runtime_store, &id)?
    {
        if existing.request_id != request.request_id
            || existing.effect_document_id != document.document_id
            || existing.effect_index != index as u64
            || existing.effect_kind != kind
            || existing.private_state_exposed
        {
            return Err(anyhow!("Persona effect intent binding is invalid"));
        }
        if existing.status == "completed" || (existing.status == "started" && kind == "say") {
            return Ok(existing);
        }
        if existing.status == "started" {
            existing.status = "quarantined_ambiguous_local_effect".into();
            existing.updated_at = chrono::Utc::now().to_rfc3339();
            put_runtime_document(runtime_store, &id, &existing)?;
        }
        return Err(anyhow!(
            "Persona effect {id} is quarantined and requires review"
        ));
    }
    let intent = PersonaEffectExecutionIntent {
        schema_version: PERSONA_EFFECT_EXECUTION_INTENT_SCHEMA_VERSION.into(),
        intent_id: id.clone(),
        request_id: request.request_id.clone(),
        effect_document_id: document.document_id.clone(),
        effect_index: index as u64,
        effect_kind: kind.into(),
        status: "started".into(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        private_state_exposed: false,
    };
    put_runtime_document(runtime_store, &id, &intent)?;
    Ok(intent)
}

fn finish_effect(runtime_store: &Path, intent: &mut PersonaEffectExecutionIntent) -> Result<()> {
    intent.status = "completed".into();
    intent.updated_at = chrono::Utc::now().to_rfc3339();
    put_runtime_document(runtime_store, &intent.intent_id, intent)
}

fn require_persona_effects_unbraked(store: &Path, runtime_id: &str) -> Result<()> {
    let brake = crate::load_epiphany_cultmesh_swarm_brake(store, runtime_id)?
        .ok_or_else(|| anyhow!("Persona effects refuse to run without canonical brake state"))?;
    if brake.status != "released" {
        return Err(anyhow!("Persona effects are braked: {}", brake.reason));
    }
    Ok(())
}

fn resolve_reply_target(
    request: &PersonaTurnRequest,
    requested: Option<&str>,
) -> Result<Option<String>> {
    if let Some(id) = requested {
        if !request.mentions.iter().any(|mention| {
            mention.message_id == id || mention.reply_to_message_id.as_deref() == Some(id)
        }) {
            return Err(anyhow!(
                "Persona SAY reply target is outside the reserved mention set"
            ));
        }
        return Ok(Some(id.to_string()));
    }
    Ok(request
        .mentions
        .last()
        .map(|mention| mention.message_id.clone()))
}

fn load_reserved_request(store: &Path, request_id: &str) -> Result<PersonaTurnRequest> {
    let state = crate::heartbeat_state::load_heartbeat_state_entry(store)?
        .ok_or_else(|| anyhow!("heartbeat state is missing"))?;
    let request = state
        .persona_turn_requests
        .into_iter()
        .find(|request| request.request_id == request_id)
        .ok_or_else(|| anyhow!("reserved Persona turn request is missing"))?;
    if request.status != "reserved" || request.terminal_receipt.is_some() {
        return Err(anyhow!("Persona turn request is not reserved"));
    }
    Ok(request)
}

fn runtime_document<T: DatabaseEntry>(store: &Path, key: &str) -> Result<Option<T>> {
    let mut cache = runtime_spine_cache(store)?;
    cache.pull_all_backing_stores()?;
    cache.get(key)
}
fn put_runtime_document<T: DatabaseEntry>(store: &Path, key: &str, value: &T) -> Result<()> {
    let mut cache = runtime_spine_cache(store)?;
    cache.pull_all_backing_stores()?;
    cache.put(key, value)?;
    Ok(())
}
fn stable_memory_id(document_id: &str, index: usize) -> String {
    format!(
        "mem-persona-{:x}",
        Sha256::digest(format!("{document_id}:{index}").as_bytes())
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unused_anchor() -> GameCultServiceTrustAnchorRecord {
        GameCultServiceTrustAnchorRecord {
            schema_version: cultnet_rs::GAMECULT_SERVICE_TRUST_ANCHOR_SCHEMA.into(),
            trust_anchor_id: "unused".into(),
            service_id: "unused".into(),
            runtime_id: "unused".into(),
            signer_identity_id: "unused".into(),
            signer_public_key: vec![],
            signature_algorithm: "ed25519".into(),
            signing_purpose: "unused".into(),
            signed_schema: "unused".into(),
            binding_authority: "root".into(),
            bound_at_unix_millis: 1,
            expires_at_unix_millis: None,
            private_state_exposed: false,
        }
    }

    fn seed_closed_silence_turn(
        runtime_store: &Path,
        state: &mut crate::heartbeat_state::EpiphanyHeartbeatStateEntry,
        suffix: &str,
        reserved_at: &str,
    ) -> Result<()> {
        let request_id = format!("turn-{suffix}");
        let effect_id = format!("persona-effects:{request_id}");
        let stage_ids = ["projector", "persona", "interpreter"]
            .map(|stage| format!("persona-stage:{request_id}:{stage}"));
        let terminal = crate::heartbeat_state::PersonaTurnTerminalReceipt {
            schema_version: crate::heartbeat_state::PERSONA_TURN_TERMINAL_RECEIPT_SCHEMA_VERSION
                .into(),
            receipt_id: format!("{request_id}:terminal"),
            request_id: request_id.clone(),
            schedule_id: format!("schedule-{suffix}"),
            action_id: format!("action-{suffix}"),
            outcome: "dropped".into(),
            mention_disposition: "consumed".into(),
            mention_ids: vec![],
            mention_cargo_sha256: format!("sha256-{}", "0".repeat(64)),
            completed_at: reserved_at.into(),
            private_state_exposed: false,
            ..Default::default()
        };
        state.persona_turn_requests.push(PersonaTurnRequest {
            schema_version: crate::heartbeat_state::PERSONA_TURN_REQUEST_SCHEMA_VERSION.into(),
            request_id: request_id.clone(),
            schedule_id: format!("schedule-{suffix}"),
            action_id: format!("action-{suffix}"),
            role_id: "Persona".into(),
            agent_id: "epiphany.Persona".into(),
            status: "terminal".into(),
            reserved_at: reserved_at.into(),
            mentions: vec![],
            semantic_memory_recall: serde_json::Value::Null,
            terminal_receipt: Some(terminal.clone()),
            private_state_exposed: false,
        });
        let effects = PersonaInterpreterEffectDocument {
            schema_version: crate::PERSONA_INTERPRETER_EFFECT_DOCUMENT_SCHEMA_VERSION.into(),
            document_id: effect_id.clone(),
            turn_id: request_id.clone(),
            identity_id: "epiphany.Persona".into(),
            interpreter_request_id: format!("persona:{request_id}:interpreter"),
            created_at: reserved_at.into(),
            effects: vec![PersonaInterpreterEffect::Drop {
                reason: "quiet".into(),
            }],
            private_state_exposed: false,
        };
        put_runtime_document(runtime_store, &effect_id, &effects)?;
        let outputs = ["1", "2", "3"].map(|digit| format!("sha256:{}", digit.repeat(64)));
        for (index, stage) in ["projector", "persona", "interpreter"].iter().enumerate() {
            put_runtime_document(
                runtime_store,
                &stage_ids[index],
                &PersonaModelStageReceipt {
                    schema_version: crate::PERSONA_MODEL_STAGE_RECEIPT_SCHEMA_VERSION.into(),
                    receipt_id: stage_ids[index].clone(),
                    turn_id: request_id.clone(),
                    stage: (*stage).into(),
                    request_id: format!("persona:{request_id}:{stage}"),
                    output_sha256: outputs[index].clone(),
                    private_output_ref: format!("model-events:persona:{request_id}:{stage}"),
                    completed_at: reserved_at.into(),
                    private_state_exposed: false,
                    provider: "test".into(),
                    model: "test".into(),
                    prompt_sha256: format!("sha256:{}", "a".repeat(64)),
                },
            )?;
        }
        put_runtime_document(
            runtime_store,
            &format!("persona-terminal:{request_id}"),
            &PersonaModelTerminalReceipt {
                schema_version: crate::PERSONA_MODEL_TERMINAL_RECEIPT_SCHEMA_VERSION.into(),
                receipt_id: format!("persona-terminal:{request_id}"),
                turn_id: request_id.clone(),
                identity_id: "epiphany.Persona".into(),
                effect_document_id: effect_id.clone(),
                stage_receipt_ids: stage_ids.to_vec(),
                completed_at: reserved_at.into(),
                private_state_exposed: false,
                downstream_status: "effects_pending_mind_admission_and_mouth_routing".into(),
                effect_document_sha256: format!(
                    "sha256:{:x}",
                    Sha256::digest(serde_json::to_vec(&effects)?)
                ),
                stage_output_sha256: outputs.to_vec(),
            },
        )?;
        put_runtime_document(
            runtime_store,
            &format!("persona-conversation:{request_id}"),
            &PersonaConversationExecutionReceipt {
                schema_version: PERSONA_CONVERSATION_EXECUTION_RECEIPT_SCHEMA_VERSION.into(),
                receipt_id: format!("persona-conversation:{request_id}"),
                request_id: request_id.clone(),
                effect_document_id: effect_id,
                outcome: "dropped".into(),
                state_effect_status: "none".into(),
                state_effect_reasons: vec![],
                delivery_evidence_ids: vec![],
                heartbeat_terminal_receipt_id: Some(terminal.receipt_id),
                private_state_exposed: false,
            },
        )?;
        Ok(())
    }

    fn request() -> PersonaTurnRequest {
        PersonaTurnRequest {
            request_id: "turn-1".into(),
            ..Default::default()
        }
    }
    fn document() -> PersonaInterpreterEffectDocument {
        PersonaInterpreterEffectDocument {
            schema_version: crate::PERSONA_INTERPRETER_EFFECT_DOCUMENT_SCHEMA_VERSION.into(),
            document_id: "persona-effects:turn-1".into(),
            turn_id: "turn-1".into(),
            identity_id: "epiphany.Persona".into(),
            interpreter_request_id: "interpreter-1".into(),
            created_at: "2026-07-21T00:00:00Z".into(),
            effects: vec![],
            private_state_exposed: false,
        }
    }

    #[test]
    fn started_speech_intent_resumes_but_ambiguous_local_mutation_quarantines() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime.cc");
        let speech = begin_effect(&store, &request(), &document(), 0, "say")?;
        assert_eq!(speech.status, "started");
        assert_eq!(
            begin_effect(&store, &request(), &document(), 0, "say")?.intent_id,
            speech.intent_id
        );

        begin_effect(&store, &request(), &document(), 1, "state_note")?;
        assert!(begin_effect(&store, &request(), &document(), 1, "state_note").is_err());
        let quarantined = runtime_document::<PersonaEffectExecutionIntent>(
            &store,
            "persona-effect-intent:persona-effects:turn-1:1",
        )?
        .unwrap();
        assert_eq!(quarantined.status, "quarantined_ambiguous_local_effect");
        Ok(())
    }

    #[test]
    fn terminal_retention_bounds_detail_and_leaves_an_irreversible_replay_frontier() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let runtime = temp.path().join("runtime.cc");
        let heartbeat = temp.path().join("heartbeat.cc");
        let requests = temp.path().join("requests.cc");
        let receipts = temp.path().join("receipts.cc");
        let mut state = crate::heartbeat_state::default_heartbeat_state(1.0);
        state.persona_turn_requests.push(PersonaTurnRequest {
            schema_version: crate::heartbeat_state::PERSONA_TURN_REQUEST_SCHEMA_VERSION.into(),
            request_id: "turn-failed".into(),
            schedule_id: "schedule-failed".into(),
            action_id: "action-failed".into(),
            role_id: "Persona".into(),
            agent_id: "epiphany.Persona".into(),
            status: "terminal".into(),
            reserved_at: "2026-08-09T23:59:59Z".into(),
            mentions: vec![],
            semantic_memory_recall: serde_json::Value::Null,
            terminal_receipt: Some(crate::heartbeat_state::PersonaTurnTerminalReceipt {
                schema_version:
                    crate::heartbeat_state::PERSONA_TURN_TERMINAL_RECEIPT_SCHEMA_VERSION.into(),
                receipt_id: "turn-failed:terminal".into(),
                request_id: "turn-failed".into(),
                schedule_id: "schedule-failed".into(),
                action_id: "action-failed".into(),
                outcome: "failed".into(),
                mention_disposition: "retained".into(),
                mention_ids: vec!["mention-failed".into()],
                mention_cargo_sha256: format!("sha256-{}", "f".repeat(64)),
                completed_at: "2026-08-09T23:59:59Z".into(),
                private_state_exposed: false,
                ..Default::default()
            }),
            private_state_exposed: false,
        });
        seed_closed_silence_turn(&runtime, &mut state, "old", "2026-08-10T00:00:00Z")?;
        seed_closed_silence_turn(&runtime, &mut state, "new", "2026-08-10T00:00:01Z")?;
        crate::heartbeat_state::write_heartbeat_state_entry(&heartbeat, &state)?;

        let head = retain_terminal_persona_conversations(
            &runtime,
            &heartbeat,
            &requests,
            &receipts,
            &unused_anchor(),
            1,
            "2026-08-10T00:01:00Z",
        )?
        .expect("oldest closed turn should retire");
        assert_eq!(head.retired_turn_count, 1);
        assert_eq!(head.through_reserved_at, "2026-08-10T00:00:00Z");
        let retained = crate::heartbeat_state::load_heartbeat_state_entry(&heartbeat)?.unwrap();
        assert_eq!(retained.persona_turn_requests.len(), 2);
        assert!(retained.persona_turn_requests.iter().any(|request| {
            request.request_id == "turn-failed"
                && request
                    .terminal_receipt
                    .as_ref()
                    .is_some_and(|receipt| receipt.mention_disposition == "retained")
        }));
        assert!(
            retained
                .persona_turn_requests
                .iter()
                .any(|request| request.request_id == "turn-new")
        );
        assert!(retained.persona_conversation_retention_plan.is_none());
        assert!(
            runtime_document::<PersonaConversationExecutionReceipt>(
                &runtime,
                "persona-conversation:turn-old"
            )?
            .is_none()
        );
        assert!(
            runtime_document::<PersonaConversationExecutionReceipt>(
                &runtime,
                "persona-conversation:turn-new"
            )?
            .is_some()
        );
        assert!(
            retain_terminal_persona_conversations(
                &runtime,
                &heartbeat,
                &requests,
                &receipts,
                &unused_anchor(),
                1,
                "2026-08-10T00:02:00Z",
            )?
            .is_none()
        );

        let mut hostile = retained;
        hostile.persona_turn_requests.push(PersonaTurnRequest {
            schema_version: crate::heartbeat_state::PERSONA_TURN_REQUEST_SCHEMA_VERSION.into(),
            request_id: "turn-old".into(),
            schedule_id: "schedule-old".into(),
            action_id: "action-old".into(),
            role_id: "Persona".into(),
            agent_id: "epiphany.Persona".into(),
            status: "reserved".into(),
            reserved_at: "2026-08-10T00:00:00Z".into(),
            mentions: vec![],
            semantic_memory_recall: serde_json::Value::Null,
            terminal_receipt: None,
            private_state_exposed: false,
        });
        assert!(crate::heartbeat_state::validate_heartbeat_state(&hostile).is_err());
        Ok(())
    }

    #[test]
    fn retry_requires_the_exact_store_cleanup_receipt_when_detail_is_already_absent() -> Result<()>
    {
        let temp = tempfile::tempdir()?;
        let runtime = temp.path().join("runtime.cc");
        let receipt = PersonaConversationExecutionReceipt {
            schema_version: PERSONA_CONVERSATION_EXECUTION_RECEIPT_SCHEMA_VERSION.into(),
            receipt_id: "persona-conversation:retry".into(),
            request_id: "retry".into(),
            effect_document_id: "persona-effects:retry".into(),
            outcome: "dropped".into(),
            state_effect_status: "none".into(),
            state_effect_reasons: vec![],
            delivery_evidence_ids: vec![],
            heartbeat_terminal_receipt_id: Some("retry:terminal".into()),
            private_state_exposed: false,
        };
        put_runtime_document(&runtime, &receipt.receipt_id, &receipt)?;
        let mut cache = runtime_spine_cache(&runtime)?;
        cache.pull_all_backing_stores()?;
        let members = retention_envelopes(
            &cache.snapshot_envelopes(),
            &[(
                <PersonaConversationExecutionReceipt as DatabaseEntry>::TYPE,
                receipt.receipt_id.clone(),
            )],
        )?;
        delete_runtime_retention_envelopes(
            &runtime,
            "plan-retry",
            "2026-08-10T00:00:00Z",
            &members,
        )?;
        delete_runtime_retention_envelopes(
            &runtime,
            "plan-retry",
            "2026-08-10T00:00:00Z",
            &members,
        )?;

        let hostile = temp.path().join("hostile.cc");
        assert!(
            delete_runtime_retention_envelopes(
                &hostile,
                "plan-retry",
                "2026-08-10T00:00:00Z",
                &members,
            )
            .is_err()
        );
        Ok(())
    }
}
