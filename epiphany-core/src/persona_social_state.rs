use anyhow::{Result, anyhow};
use cultcache_rs::{
    CacheBackingStore, CultCache, CultCacheEnvelope, DatabaseEntry,
    SingleFileMessagePackBackingStore,
};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

pub const PERSONA_TURN_REQUEST_SCHEMA_VERSION: &str = "epiphany.persona_turn_request.v1";
pub const PERSONA_TURN_TERMINAL_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.persona_turn_terminal_receipt.v1";
pub const PERSONA_CONVERSATION_RETENTION_HEAD_SCHEMA_VERSION: &str =
    "epiphany.persona_conversation_retention_head.v1";
pub const PERSONA_CONVERSATION_RETENTION_PLAN_SCHEMA_VERSION: &str =
    "epiphany.persona_conversation_retention_plan.v1";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaSocialMention {
    pub id: String,
    pub target_role_id: String,
    pub target_agent_id: String,
    pub source_surface: String,
    pub channel_id: String,
    pub message_id: String,
    pub author_id: String,
    #[serde(default)]
    pub author_name: Option<String>,
    pub content: String,
    pub visible_prompt: String,
    #[serde(default)]
    pub reply_to_message_id: Option<String>,
    pub queued_at: String,
    #[serde(default)]
    pub source_visibility: String,
    #[serde(default)]
    pub data_classification: String,
    #[serde(default)]
    pub model_provider_id: String,
    #[serde(default)]
    pub model_provider_disclosure_allowed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaTurnRequest {
    pub schema_version: String,
    pub request_id: String,
    pub schedule_id: String,
    pub action_id: String,
    pub role_id: String,
    pub agent_id: String,
    pub status: String,
    pub reserved_at: String,
    pub mentions: Vec<PersonaSocialMention>,
    #[serde(default)]
    pub terminal_receipt: Option<PersonaTurnTerminalReceipt>,
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaTurnTerminalReceipt {
    pub schema_version: String,
    pub receipt_id: String,
    pub request_id: String,
    pub schedule_id: String,
    pub action_id: String,
    pub outcome: String,
    pub mention_disposition: String,
    pub mention_ids: Vec<String>,
    pub mention_cargo_sha256: String,
    #[serde(default)]
    pub delivery_evidence_id: Option<String>,
    #[serde(default)]
    pub crossing_receipt_id: Option<String>,
    #[serde(default)]
    pub bridge_receipt_sha256: Option<String>,
    #[serde(default)]
    pub blocked_crossing_status: Option<String>,
    #[serde(default)]
    pub blocked_reason: Option<String>,
    pub completed_at: String,
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PersonaTurnTerminalOptions {
    pub request_id: String,
    pub outcome: String,
    pub delivery_evidence: Option<crate::PersonaDiscordDeliveryEvidence>,
    pub blocked_evidence: Option<PersonaTurnBlockedEvidence>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaConversationRetentionHead {
    pub schema_version: String,
    pub revision: u64,
    pub retired_turn_count: u64,
    pub through_reserved_at: String,
    pub chained_digest: String,
    pub retained_at: String,
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaConversationRetentionEnvelope {
    pub envelope_type: String,
    pub key: String,
    pub envelope_sha256: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaConversationRetentionMember {
    pub request_id: String,
    pub reserved_at: String,
    pub completed_at: String,
    pub outcome: String,
    pub terminal_receipt_sha256: String,
    pub runtime_envelopes: Vec<PersonaConversationRetentionEnvelope>,
    pub crossing_request_envelopes: Vec<PersonaConversationRetentionEnvelope>,
    pub crossing_receipt_envelopes: Vec<PersonaConversationRetentionEnvelope>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaConversationRetentionPlan {
    pub schema_version: String,
    pub plan_id: String,
    pub members: Vec<PersonaConversationRetentionMember>,
    pub planned_at: String,
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PersonaTurnBlockedEvidence {
    pub evidence_source: String,
    pub crossing_status: String,
    pub reason: String,
    pub crossing_receipt_id: Option<String>,
    pub bridge_receipt_sha256: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaBlockedConversationPressure {
    pub schema_version: String,
    pub quarantine_id: String,
    pub request_id: String,
    pub terminal_receipt_id: String,
    pub crossing_status: String,
    pub evidence_source: String,
    pub reason: String,
    pub mentions: Vec<PersonaSocialMention>,
    pub mention_cargo_sha256: String,
    #[serde(default)]
    pub crossing_receipt_id: Option<String>,
    #[serde(default)]
    pub bridge_receipt_sha256: Option<String>,
    pub quarantined_at: String,
    pub private_state_exposed: bool,
}

pub const PERSONA_SOCIAL_MENTION_TYPE: &str = "epiphany.persona.social_mention.v1";
pub const PERSONA_SOCIAL_TURN_REQUEST_TYPE: &str = "epiphany.persona.turn_request.v1";
pub const PERSONA_SOCIAL_TURN_TERMINAL_TYPE: &str = "epiphany.persona.turn_terminal.v1";
pub const PERSONA_SOCIAL_QUARANTINE_TYPE: &str = "epiphany.persona.quarantine.v1";
pub const PERSONA_SOCIAL_RETENTION_HEAD_TYPE: &str = "epiphany.persona.retention_head.v1";
pub const PERSONA_SOCIAL_RETENTION_PLAN_TYPE: &str = "epiphany.persona.retention_plan.v1";
pub const PERSONA_SOCIAL_RETENTION_HEAD_KEY: &str = "persona";
pub const PERSONA_SOCIAL_RETENTION_PLAN_KEY: &str = "active";

#[derive(Clone, Debug, PartialEq)]
pub struct PersonaSocialQueueMentionOptions {
    pub target_role_id: String,
    pub source_surface: String,
    pub channel_id: String,
    pub message_id: String,
    pub author_id: String,
    pub author_name: Option<String>,
    pub content: String,
    pub visible_prompt: String,
    pub reply_to_message_id: Option<String>,
    pub queued_at: Option<String>,
    pub mention_id: Option<String>,
    pub source_visibility: String,
    pub data_classification: String,
    pub model_provider_id: String,
    pub model_provider_disclosure_allowed: bool,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.persona.social_mention.v1",
    schema = "PersonaSocialMentionDocument"
)]
pub struct PersonaSocialMentionDocument {
    #[cultcache(key = 0)]
    pub status: String,
    #[cultcache(key = 1)]
    pub mention: PersonaSocialMention,
    #[cultcache(key = 2)]
    pub terminal_receipt_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.persona.turn_request.v1",
    schema = "PersonaSocialTurnRequestDocument"
)]
pub struct PersonaSocialTurnRequestDocument {
    #[cultcache(key = 0)]
    pub request: PersonaTurnRequest,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.persona.turn_terminal.v1",
    schema = "PersonaSocialTurnTerminalDocument"
)]
pub struct PersonaSocialTurnTerminalDocument {
    #[cultcache(key = 0)]
    pub receipt: PersonaTurnTerminalReceipt,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.persona.quarantine.v1",
    schema = "PersonaSocialQuarantineDocument"
)]
pub struct PersonaSocialQuarantineDocument {
    #[cultcache(key = 0)]
    pub pressure: PersonaBlockedConversationPressure,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.persona.retention_head.v1",
    schema = "PersonaSocialRetentionHeadDocument"
)]
pub struct PersonaSocialRetentionHeadDocument {
    #[cultcache(key = 0)]
    pub head: PersonaConversationRetentionHead,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.persona.retention_plan.v1",
    schema = "PersonaSocialRetentionPlanDocument"
)]
pub struct PersonaSocialRetentionPlanDocument {
    #[cultcache(key = 0)]
    pub status: String,
    #[cultcache(key = 1)]
    pub plan: PersonaConversationRetentionPlan,
}

pub fn persona_social_cache(store_path: impl AsRef<Path>) -> Result<CultCache> {
    let mut cache = CultCache::new();
    register_persona_social_types(&mut cache)?;
    let mut identities = HashSet::new();
    for envelope in SingleFileMessagePackBackingStore::new(store_path.as_ref()).pull_all()? {
        if !is_persona_social_type(&envelope.r#type) {
            continue;
        }
        if !identities.insert((envelope.r#type.clone(), envelope.key.clone())) {
            return Err(anyhow!(
                "Persona social store contains duplicate document type {:?} key {:?}",
                envelope.r#type,
                envelope.key
            ));
        }
        load_persona_social_envelope(&mut cache, envelope)?;
    }
    Ok(cache)
}

pub fn queue_persona_social_mention(
    store_path: impl AsRef<Path>,
    options: PersonaSocialQueueMentionOptions,
) -> Result<serde_json::Value> {
    if options.target_role_id != "Persona" {
        return Err(anyhow!(
            "Persona social mention target role must be Persona"
        ));
    }
    validate_mention_text("content", &options.content, 4, 1200)?;
    validate_mention_text("visible_prompt", &options.visible_prompt, 4, 1200)?;
    for (label, value) in [
        ("source_surface", &options.source_surface),
        ("channel_id", &options.channel_id),
        ("message_id", &options.message_id),
        ("author_id", &options.author_id),
        ("source_visibility", &options.source_visibility),
        ("data_classification", &options.data_classification),
        ("model_provider_id", &options.model_provider_id),
    ] {
        validate_mention_text(label, value, 1, 240)?;
    }
    if !options.model_provider_disclosure_allowed {
        return Err(anyhow!(
            "pending mention is not authorized for disclosure to the configured model provider"
        ));
    }
    let mention_id = options.mention_id.clone().unwrap_or_else(|| {
        stable_pending_mention_id(
            &options.target_role_id,
            &options.channel_id,
            &options.message_id,
            &options.visible_prompt,
        )
    });
    let document = PersonaSocialMentionDocument {
        status: "pending".into(),
        mention: PersonaSocialMention {
            id: mention_id.clone(),
            target_role_id: options.target_role_id.clone(),
            target_agent_id: "epiphany.Persona".into(),
            source_surface: options.source_surface,
            channel_id: options.channel_id,
            message_id: options.message_id,
            author_id: options.author_id,
            author_name: options.author_name,
            content: options.content,
            visible_prompt: options.visible_prompt,
            reply_to_message_id: options.reply_to_message_id,
            queued_at: options.queued_at.unwrap_or_else(now_iso),
            source_visibility: options.source_visibility,
            data_classification: options.data_classification,
            model_provider_id: options.model_provider_id,
            model_provider_disclosure_allowed: true,
        },
        terminal_receipt_id: None,
    };
    let store_path = store_path.as_ref();
    let cache = persona_social_cache(store_path)?;
    if let Some(existing) = cache.get::<PersonaSocialMentionDocument>(&mention_id)? {
        if existing != document {
            return Err(anyhow!(
                "Persona social mention identity collision for {mention_id:?}"
            ));
        }
        return Ok(serde_json::json!({
            "ok": true,
            "queued": false,
            "reason": "duplicate-pending-mention",
            "mentionId": mention_id,
            "pendingMentionCount": pending_persona_mentions(store_path)?.len(),
        }));
    }
    let envelope = cache.prepare_entry(&mention_id, &document)?.0;
    if !persona_social_backing(store_path).compare_and_swap_batch(&[], vec![envelope])? {
        let current =
            persona_social_cache(store_path)?.get::<PersonaSocialMentionDocument>(&mention_id)?;
        if current.as_ref() != Some(&document) {
            return Err(anyhow!(
                "Persona social mention lost exact insert compare-and-swap"
            ));
        }
    }
    Ok(serde_json::json!({
        "ok": true,
        "queued": true,
        "mentionId": mention_id,
        "targetRoleId": options.target_role_id,
        "pendingMentionCount": pending_persona_mentions(store_path)?.len(),
    }))
}

pub fn pulse_persona_social(
    store_path: impl AsRef<Path>,
    brake_engaged: bool,
) -> Result<serde_json::Value> {
    if brake_engaged {
        return Ok(serde_json::json!({
            "schemaVersion": "epiphany.persona_social_pulse.v1",
            "status": "refused-by-swarm-brake",
            "privateStateExposed": false,
        }));
    }
    let store_path = store_path.as_ref();
    let cache = persona_social_cache(store_path)?;
    let mut mention_documents = cache
        .get_all::<PersonaSocialMentionDocument>()?
        .into_iter()
        .filter(|document| document.status == "pending")
        .collect::<Vec<_>>();
    mention_documents.sort_by(|left, right| {
        left.mention
            .queued_at
            .cmp(&right.mention.queued_at)
            .then_with(|| left.mention.id.cmp(&right.mention.id))
    });
    if persona_turn_requests(store_path)?
        .iter()
        .any(|request| request.terminal_receipt.is_none())
    {
        return Ok(serde_json::json!({
            "schemaVersion": "epiphany.persona_social_pulse.v1",
            "status": "already-running",
            "privateStateExposed": false,
        }));
    }
    if mention_documents.is_empty() {
        return Ok(serde_json::json!({
            "schemaVersion": "epiphany.persona_social_pulse.v1",
            "status": "idle",
            "privateStateExposed": false,
        }));
    }
    let mut schedule_digest = sha2::Sha256::new();
    schedule_digest.update(b"epiphany.persona-social-schedule.v1\0");
    schedule_digest.update(rmp_serde::to_vec(&mention_documents)?);
    let schedule_id = format!("persona-schedule:sha256:{:x}", schedule_digest.finalize());
    let request_id = format!("persona-turn:{schedule_id}");
    let request = PersonaTurnRequest {
        schema_version: PERSONA_TURN_REQUEST_SCHEMA_VERSION.into(),
        request_id: request_id.clone(),
        schedule_id: schedule_id.clone(),
        action_id: "persona.turn".into(),
        role_id: "Persona".into(),
        agent_id: "epiphany.Persona".into(),
        status: "reserved".into(),
        reserved_at: now_iso(),
        mentions: mention_documents
            .iter()
            .map(|document| document.mention.clone())
            .collect(),
        terminal_receipt: None,
        private_state_exposed: false,
    };
    if let Some(head) = persona_retention_head(store_path)? {
        let frontier = chrono::DateTime::parse_from_rfc3339(&head.through_reserved_at)
            .map_err(|_| anyhow!("Persona conversation retention frontier is invalid"))?;
        let reserved = chrono::DateTime::parse_from_rfc3339(&request.reserved_at)
            .map_err(|_| anyhow!("Persona turn reservation time is invalid"))?;
        if reserved <= frontier {
            return Err(anyhow!(
                "Persona turn reservation is at or behind the retired replay frontier"
            ));
        }
    }
    let mut expected = Vec::new();
    let mut writes = Vec::new();
    for mut document in mention_documents {
        expected.push(
            persona_social_envelope::<PersonaSocialMentionDocument>(&cache, &document.mention.id)
                .ok_or_else(|| anyhow!("pending Persona mention envelope disappeared"))?,
        );
        document.status = "reserved".into();
        writes.push(cache.prepare_entry(&document.mention.id, &document)?.0);
    }
    writes.push(
        cache
            .prepare_entry(
                &request_id,
                &PersonaSocialTurnRequestDocument {
                    request: request.clone(),
                },
            )?
            .0,
    );
    if !persona_social_backing(store_path).compare_and_swap_batch(&expected, writes)? {
        return Err(anyhow!(
            "Persona social pulse lost exact mention reservation compare-and-swap"
        ));
    }
    Ok(serde_json::json!({
        "schemaVersion": "epiphany.persona_social_pulse.v1",
        "status": "reserved",
        "requestId": request_id,
        "schedule": {
            "schema_version": "epiphany.persona_social_schedule.v1",
            "schedule_id": schedule_id,
            "source_ref": "persona-social:pending",
            "action_catalog": [{
                "action_id": request.action_id,
                "actor_id": request.agent_id,
                "action_type": "persona_turn",
                "pending_mentions": request.mentions,
            }],
        },
        "privateStateExposed": false,
    }))
}

pub fn complete_persona_social_turn(
    store_path: impl AsRef<Path>,
    options: PersonaTurnTerminalOptions,
) -> Result<PersonaTurnTerminalReceipt> {
    let mention_disposition = match options.outcome.as_str() {
        "delivered" | "silence" | "dropped" => "consumed",
        "failed" => "retained",
        "blocked" => "quarantined",
        other => {
            return Err(anyhow!(
                "unsupported Persona turn terminal outcome {other:?}"
            ));
        }
    };
    let store_path = store_path.as_ref();
    let cache = persona_social_cache(store_path)?;
    let request_document = cache
        .get::<PersonaSocialTurnRequestDocument>(&options.request_id)?
        .ok_or_else(|| anyhow!("Persona turn request {:?} is missing", options.request_id))?;
    let request = request_document.request;
    if let Some(existing) = cache
        .get::<PersonaSocialTurnTerminalDocument>(&options.request_id)?
        .map(|document| document.receipt)
    {
        if existing.outcome == options.outcome {
            return Ok(existing);
        }
        return Err(anyhow!(
            "Persona turn request {:?} already terminated as {:?}",
            options.request_id,
            existing.outcome
        ));
    }
    let delivery_evidence = options.delivery_evidence.as_ref();
    let blocked_evidence = options.blocked_evidence.as_ref();
    if options.outcome == "delivered" {
        let evidence = delivery_evidence.ok_or_else(|| {
            anyhow!("delivered Persona turn requires typed Discord delivery evidence")
        })?;
        if evidence.schema_version != crate::PERSONA_DISCORD_DELIVERY_EVIDENCE_SCHEMA_VERSION
            || evidence.private_state_exposed
            || evidence.evidence_id.trim().is_empty()
            || evidence.message_id.trim().is_empty()
            || evidence.crossing_receipt_id.trim().is_empty()
            || evidence.bridge_receipt_sha256.trim().is_empty()
            || !request
                .mentions
                .iter()
                .any(|mention| mention.channel_id == evidence.channel_id)
        {
            return Err(anyhow!(
                "Discord delivery evidence is not bound to the Persona turn"
            ));
        }
    } else if delivery_evidence.is_some() {
        return Err(anyhow!(
            "non-delivered Persona terminal outcome must not carry delivery evidence"
        ));
    }
    if options.outcome == "blocked" {
        let evidence = blocked_evidence
            .ok_or_else(|| anyhow!("blocked Persona turn requires typed crossing evidence"))?;
        if !matches!(
            evidence.evidence_source.as_str(),
            "bifrost_crossing" | "local_effect"
        ) || !matches!(evidence.crossing_status.as_str(), "unknown" | "failed")
            || (evidence.evidence_source == "local_effect"
                && (evidence.crossing_receipt_id.is_some()
                    || evidence.bridge_receipt_sha256.is_some()))
            || evidence.reason.trim().is_empty()
            || evidence
                .crossing_receipt_id
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
            || evidence
                .bridge_receipt_sha256
                .as_deref()
                .is_some_and(|value| {
                    !value.strip_prefix("sha256:").is_some_and(|digest| {
                        digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
                    })
                })
        {
            return Err(anyhow!("blocked Persona crossing evidence is invalid"));
        }
    } else if blocked_evidence.is_some() {
        return Err(anyhow!(
            "non-blocked Persona terminal outcome must not carry blocked evidence"
        ));
    }
    let receipt = PersonaTurnTerminalReceipt {
        schema_version: PERSONA_TURN_TERMINAL_RECEIPT_SCHEMA_VERSION.into(),
        receipt_id: format!("{}:terminal", request.request_id),
        request_id: request.request_id.clone(),
        schedule_id: request.schedule_id.clone(),
        action_id: request.action_id.clone(),
        outcome: options.outcome,
        mention_disposition: mention_disposition.into(),
        mention_ids: request
            .mentions
            .iter()
            .map(|mention| mention.id.clone())
            .collect(),
        mention_cargo_sha256: format!(
            "sha256-{:x}",
            sha2::Sha256::digest(rmp_serde::to_vec(&request.mentions)?)
        ),
        delivery_evidence_id: delivery_evidence.map(|evidence| evidence.evidence_id.clone()),
        crossing_receipt_id: delivery_evidence.map(|evidence| evidence.crossing_receipt_id.clone()),
        bridge_receipt_sha256: delivery_evidence
            .map(|evidence| evidence.bridge_receipt_sha256.clone()),
        blocked_crossing_status: blocked_evidence.map(|evidence| evidence.crossing_status.clone()),
        blocked_reason: blocked_evidence.map(|evidence| evidence.reason.clone()),
        completed_at: now_iso(),
        private_state_exposed: false,
    };
    let request_envelope =
        persona_social_envelope::<PersonaSocialTurnRequestDocument>(&cache, &request.request_id)
            .ok_or_else(|| anyhow!("Persona turn request envelope disappeared"))?;
    let mut expected = vec![request_envelope.clone()];
    let mut writes = vec![
        request_envelope,
        cache
            .prepare_entry(
                &request.request_id,
                &PersonaSocialTurnTerminalDocument {
                    receipt: receipt.clone(),
                },
            )?
            .0,
    ];
    for mention in &request.mentions {
        let mut document = cache
            .get::<PersonaSocialMentionDocument>(&mention.id)?
            .ok_or_else(|| anyhow!("Persona reserved mention {:?} is missing", mention.id))?;
        if document.status != "reserved" || document.mention != *mention {
            return Err(anyhow!(
                "Persona reserved mention {:?} changed before terminalization",
                mention.id
            ));
        }
        expected.push(
            persona_social_envelope::<PersonaSocialMentionDocument>(&cache, &mention.id)
                .ok_or_else(|| anyhow!("Persona mention envelope disappeared"))?,
        );
        document.status = match mention_disposition {
            "retained" => "pending",
            "consumed" => "consumed",
            "quarantined" => "quarantined",
            _ => unreachable!("closed mention disposition"),
        }
        .into();
        document.terminal_receipt_id = Some(receipt.receipt_id.clone());
        writes.push(cache.prepare_entry(&mention.id, &document)?.0);
    }
    if mention_disposition == "quarantined" {
        let evidence = blocked_evidence.expect("validated blocked evidence");
        let pressure = PersonaBlockedConversationPressure {
            schema_version: "epiphany.persona_blocked_conversation_pressure.v1".into(),
            quarantine_id: format!("{}:quarantine", request.request_id),
            request_id: request.request_id.clone(),
            terminal_receipt_id: receipt.receipt_id.clone(),
            crossing_status: evidence.crossing_status.clone(),
            evidence_source: evidence.evidence_source.clone(),
            reason: evidence.reason.clone(),
            mentions: request.mentions.clone(),
            mention_cargo_sha256: receipt.mention_cargo_sha256.clone(),
            crossing_receipt_id: evidence.crossing_receipt_id.clone(),
            bridge_receipt_sha256: evidence.bridge_receipt_sha256.clone(),
            quarantined_at: receipt.completed_at.clone(),
            private_state_exposed: false,
        };
        let quarantine_id = pressure.quarantine_id.clone();
        writes.push(
            cache
                .prepare_entry(
                    &quarantine_id,
                    &PersonaSocialQuarantineDocument { pressure },
                )?
                .0,
        );
    }
    if !persona_social_backing(store_path).compare_and_swap_batch(&expected, writes)? {
        return Err(anyhow!(
            "Persona terminal decision lost exact document compare-and-swap"
        ));
    }
    Ok(receipt)
}

pub fn pending_persona_mentions(store_path: impl AsRef<Path>) -> Result<Vec<PersonaSocialMention>> {
    let cache = persona_social_cache(store_path)?;
    let mut mentions = cache
        .get_all::<PersonaSocialMentionDocument>()?
        .into_iter()
        .filter(|document| document.status == "pending")
        .map(|document| document.mention)
        .collect::<Vec<_>>();
    mentions.sort_by(|left, right| {
        left.queued_at
            .cmp(&right.queued_at)
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(mentions)
}

pub fn persona_turn_requests(store_path: impl AsRef<Path>) -> Result<Vec<PersonaTurnRequest>> {
    let cache = persona_social_cache(store_path)?;
    let terminals = cache
        .get_all::<PersonaSocialTurnTerminalDocument>()?
        .into_iter()
        .map(|document| (document.receipt.request_id.clone(), document.receipt))
        .collect::<BTreeMap<_, _>>();
    let mut requests = cache
        .get_all::<PersonaSocialTurnRequestDocument>()?
        .into_iter()
        .map(|document| {
            let mut request = document.request;
            if let Some(terminal) = terminals.get(&request.request_id) {
                request.status = "terminal".into();
                request.terminal_receipt = Some(terminal.clone());
                request.mentions.clear();
            }
            request
        })
        .collect::<Vec<_>>();
    requests.sort_by(|left, right| {
        left.reserved_at
            .cmp(&right.reserved_at)
            .then_with(|| left.request_id.cmp(&right.request_id))
    });
    Ok(requests)
}

pub fn persona_turn_request(
    store_path: impl AsRef<Path>,
    request_id: &str,
) -> Result<Option<PersonaTurnRequest>> {
    Ok(persona_turn_requests(store_path)?
        .into_iter()
        .find(|request| request.request_id == request_id))
}

pub fn persona_turn_request_source(
    store_path: impl AsRef<Path>,
    request_id: &str,
) -> Result<crate::EpiphanyMindDocumentVersion> {
    let cache = persona_social_cache(store_path)?;
    let envelope = persona_social_envelope::<PersonaSocialTurnRequestDocument>(&cache, request_id)
        .ok_or_else(|| anyhow!("Persona social request {request_id:?} is missing"))?;
    crate::EpiphanyMindDocumentVersion::from_envelope("epiphany-persona-social", &envelope)
}

pub fn persona_retention_head(
    store_path: impl AsRef<Path>,
) -> Result<Option<PersonaConversationRetentionHead>> {
    Ok(persona_social_cache(store_path)?
        .get::<PersonaSocialRetentionHeadDocument>(PERSONA_SOCIAL_RETENTION_HEAD_KEY)?
        .map(|document| document.head))
}

pub fn persona_retention_plan(
    store_path: impl AsRef<Path>,
) -> Result<Option<PersonaConversationRetentionPlan>> {
    Ok(persona_social_cache(store_path)?
        .get::<PersonaSocialRetentionPlanDocument>(PERSONA_SOCIAL_RETENTION_PLAN_KEY)?
        .filter(|document| document.status == "active")
        .map(|document| document.plan))
}

pub fn put_persona_retention_plan(
    store_path: impl AsRef<Path>,
    plan: &PersonaConversationRetentionPlan,
) -> Result<()> {
    let store_path = store_path.as_ref();
    let cache = persona_social_cache(store_path)?;
    if let Some(existing) =
        cache.get::<PersonaSocialRetentionPlanDocument>(PERSONA_SOCIAL_RETENTION_PLAN_KEY)?
    {
        if existing.status == "active" {
            if existing.plan == *plan {
                return Ok(());
            }
            return Err(anyhow!(
                "a different Persona retention plan is already active"
            ));
        }
    }
    let expected = persona_social_envelope::<PersonaSocialRetentionPlanDocument>(
        &cache,
        PERSONA_SOCIAL_RETENTION_PLAN_KEY,
    )
    .into_iter()
    .collect::<Vec<_>>();
    let replacement = cache
        .prepare_entry(
            PERSONA_SOCIAL_RETENTION_PLAN_KEY,
            &PersonaSocialRetentionPlanDocument {
                status: "active".into(),
                plan: plan.clone(),
            },
        )?
        .0;
    if !persona_social_backing(store_path).compare_and_swap_batch(&expected, vec![replacement])? {
        return Err(anyhow!(
            "Persona retention plan lost exact compare-and-swap"
        ));
    }
    Ok(())
}

pub fn complete_persona_retention_plan(
    store_path: impl AsRef<Path>,
    plan: &PersonaConversationRetentionPlan,
    head: &PersonaConversationRetentionHead,
) -> Result<()> {
    let store_path = store_path.as_ref();
    let cache = persona_social_cache(store_path)?;
    let current = cache
        .get::<PersonaSocialRetentionPlanDocument>(PERSONA_SOCIAL_RETENTION_PLAN_KEY)?
        .ok_or_else(|| anyhow!("Persona retention plan is missing"))?;
    if current.status == "completed"
        && current.plan == *plan
        && persona_retention_head(store_path)?.as_ref() == Some(head)
    {
        return Ok(());
    }
    if current.status != "active" || current.plan != *plan {
        return Err(anyhow!("Persona retention plan changed before completion"));
    }
    let mut expected = vec![
        persona_social_envelope::<PersonaSocialRetentionPlanDocument>(
            &cache,
            PERSONA_SOCIAL_RETENTION_PLAN_KEY,
        )
        .ok_or_else(|| anyhow!("Persona retention plan envelope disappeared"))?,
    ];
    if let Some(envelope) = persona_social_envelope::<PersonaSocialRetentionHeadDocument>(
        &cache,
        PERSONA_SOCIAL_RETENTION_HEAD_KEY,
    ) {
        expected.push(envelope);
    }
    let writes = vec![
        cache
            .prepare_entry(
                PERSONA_SOCIAL_RETENTION_PLAN_KEY,
                &PersonaSocialRetentionPlanDocument {
                    status: "completed".into(),
                    plan: plan.clone(),
                },
            )?
            .0,
        cache
            .prepare_entry(
                PERSONA_SOCIAL_RETENTION_HEAD_KEY,
                &PersonaSocialRetentionHeadDocument { head: head.clone() },
            )?
            .0,
    ];
    if !persona_social_backing(store_path).compare_and_swap_batch(&expected, writes)? {
        return Err(anyhow!(
            "Persona retention completion lost exact compare-and-swap"
        ));
    }
    Ok(())
}

pub(crate) fn persona_social_envelope<T: DatabaseEntry>(
    cache: &CultCache,
    key: &str,
) -> Option<CultCacheEnvelope> {
    cache
        .snapshot_envelopes()
        .into_iter()
        .find(|envelope| envelope.r#type == T::TYPE && envelope.key == key)
}

pub(crate) fn persona_social_backing(
    store_path: impl AsRef<Path>,
) -> SingleFileMessagePackBackingStore {
    SingleFileMessagePackBackingStore::new(store_path.as_ref())
}

fn register_persona_social_types(cache: &mut CultCache) -> Result<()> {
    cache.register_entry_type::<PersonaSocialMentionDocument>()?;
    cache.register_entry_type::<PersonaSocialTurnRequestDocument>()?;
    cache.register_entry_type::<PersonaSocialTurnTerminalDocument>()?;
    cache.register_entry_type::<PersonaSocialQuarantineDocument>()?;
    cache.register_entry_type::<PersonaSocialRetentionHeadDocument>()?;
    cache.register_entry_type::<PersonaSocialRetentionPlanDocument>()?;
    Ok(())
}

fn is_persona_social_type(document_type: &str) -> bool {
    [
        PERSONA_SOCIAL_MENTION_TYPE,
        PERSONA_SOCIAL_TURN_REQUEST_TYPE,
        PERSONA_SOCIAL_TURN_TERMINAL_TYPE,
        PERSONA_SOCIAL_QUARANTINE_TYPE,
        PERSONA_SOCIAL_RETENTION_HEAD_TYPE,
        PERSONA_SOCIAL_RETENTION_PLAN_TYPE,
    ]
    .contains(&document_type)
}

fn load_persona_social_envelope(cache: &mut CultCache, envelope: CultCacheEnvelope) -> Result<()> {
    match envelope.r#type.as_str() {
        PERSONA_SOCIAL_MENTION_TYPE => {
            cache.load_envelope::<PersonaSocialMentionDocument>(envelope)?;
        }
        PERSONA_SOCIAL_TURN_REQUEST_TYPE => {
            cache.load_envelope::<PersonaSocialTurnRequestDocument>(envelope)?;
        }
        PERSONA_SOCIAL_TURN_TERMINAL_TYPE => {
            cache.load_envelope::<PersonaSocialTurnTerminalDocument>(envelope)?;
        }
        PERSONA_SOCIAL_QUARANTINE_TYPE => {
            cache.load_envelope::<PersonaSocialQuarantineDocument>(envelope)?;
        }
        PERSONA_SOCIAL_RETENTION_HEAD_TYPE => {
            cache.load_envelope::<PersonaSocialRetentionHeadDocument>(envelope)?;
        }
        PERSONA_SOCIAL_RETENTION_PLAN_TYPE => {
            cache.load_envelope::<PersonaSocialRetentionPlanDocument>(envelope)?;
        }
        _ => unreachable!("caller filtered Persona social document types"),
    }
    Ok(())
}

fn validate_mention_text(label: &str, value: &str, min_len: usize, max_len: usize) -> Result<()> {
    let trimmed = value.trim();
    if trimmed.len() < min_len || value.len() > max_len {
        return Err(anyhow!(
            "pending mention {label} must be between {min_len} and {max_len} UTF-8 bytes"
        ));
    }
    Ok(())
}

fn stable_pending_mention_id(
    role_id: &str,
    channel_id: &str,
    message_id: &str,
    visible_prompt: &str,
) -> String {
    let mut hash = 5381_u64;
    for byte in format!("{role_id}\0{channel_id}\0{message_id}\0{visible_prompt}").as_bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(*byte as u64);
    }
    format!("mention-{hash:016x}")
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}
