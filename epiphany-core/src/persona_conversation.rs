use std::path::Path;

use anyhow::{Result, anyhow};
use cultcache_rs::DatabaseEntry;
use cultnet_rs::{GameCultServiceTrustAnchorRecord, ServiceIdentitySigner};
use sha2::{Digest, Sha256};

use crate::agent_memory::SelfPatchMemory;
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
}
