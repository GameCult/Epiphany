use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::SecondsFormat;
use epiphany_core::EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_PERSONA_SPEECH_AUDIT_SCHEMA_VERSION;
use epiphany_core::EpiphanyCultMeshPersonaSpeechAuditEntry;
use epiphany_core::load_latest_epiphany_cultmesh_persona_speech_audit;
use epiphany_core::write_epiphany_cultmesh_persona_speech_audit;
use reqwest::blocking::Client;
use serde_json::Value;
use std::cmp::Reverse;
use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

const CHAT_SCHEMA_VERSION: &str = "epiphany.persona_chat.v0";
const BUBBLE_SCHEMA_VERSION: &str = "epiphany.persona_bubble.v0";
const SPEECH_AUDIT_SCHEMA_VERSION: &str = "epiphany.persona_speech_audit.v0";
const DISCORD_API: &str = "https://discord.com/api/v10";

#[derive(Clone, Debug, Default)]
struct PersonaConfig {
    allowed_channel_name: String,
    allowed_channel_id: Option<String>,
    allowed_channel_id_env: Option<String>,
    bot_token_env: Option<String>,
    persona_name: Option<String>,
    persona_avatar_url: Option<String>,
    webhook_cache_path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return usage();
    };
    let mut config_path = PathBuf::from("state/persona-discord.toml");
    let mut artifact_dir = PathBuf::from(".epiphany-persona");
    let mut cultmesh_store = PathBuf::from(".epiphany-run/cultmesh/persona-speech.ccmp");
    let mut runtime_id = "epiphany-local".to_string();
    let mut content: Option<String> = None;
    let mut channel_id: Option<String> = None;
    let mut source = "epiphany/Persona".to_string();
    let mut status = "ready".to_string();
    let mut mood = "attentive".to_string();
    let mut limit = 8_usize;
    let mut persona_name: Option<String> = None;
    let mut persona_avatar_url: Option<String> = None;
    let mut reply_to_message_id: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => config_path = next_path(&mut args, "--config")?,
            "--artifact-dir" => artifact_dir = next_path(&mut args, "--artifact-dir")?,
            "--cultmesh-store" => cultmesh_store = next_path(&mut args, "--cultmesh-store")?,
            "--runtime-id" => runtime_id = next_value(&mut args, "--runtime-id")?,
            "--content" => content = Some(next_value(&mut args, "--content")?),
            "--channel-id" => channel_id = Some(next_value(&mut args, "--channel-id")?),
            "--source" => source = next_value(&mut args, "--source")?,
            "--status" => status = next_value(&mut args, "--status")?,
            "--mood" => mood = next_value(&mut args, "--mood")?,
            "--limit" => limit = next_value(&mut args, "--limit")?.parse()?,
            "--persona-name" => persona_name = Some(next_value(&mut args, "--persona-name")?),
            "--persona-avatar-url" => {
                persona_avatar_url = Some(next_value(&mut args, "--persona-avatar-url")?)
            }
            "--reply-to-message-id" => {
                reply_to_message_id = Some(next_value(&mut args, "--reply-to-message-id")?)
            }
            _ => return Err(anyhow!("unknown argument {arg:?}")),
        }
    }

    let result = match command.as_str() {
        "draft" => {
            let config = load_config(&config_path)?;
            let content = read_content(content)?;
            run_draft(
                &content,
                &config,
                &artifact_dir,
                &cultmesh_store,
                &runtime_id,
                "draft",
                "drafted without posting",
            )?
        }
        "bubble" => {
            let content = read_content(content)?;
            run_bubble(
                &content,
                &artifact_dir,
                &cultmesh_store,
                &runtime_id,
                &source,
                &status,
                &mood,
            )?
        }
        "post" => {
            let config = load_config(&config_path)?;
            let content = read_content(content)?;
            run_post(
                &content,
                &config,
                &artifact_dir,
                &cultmesh_store,
                &runtime_id,
                channel_id,
                persona_name,
                persona_avatar_url,
                reply_to_message_id,
            )?
        }
        "latest" => serde_json::json!({
            "ok": true,
            "artifactDir": artifact_dir,
            "latestArtifacts": latest_persona_artifacts(&artifact_dir, limit),
        }),
        "smoke" => run_smoke()?,
        _ => return usage(),
    };
    let ok = result["ok"].as_bool().unwrap_or(false);
    println!("{}", serde_json::to_string_pretty(&result)?);
    if !ok {
        std::process::exit(1);
    }
    Ok(())
}

fn run_draft(
    content: &str,
    config: &PersonaConfig,
    artifact_dir: &Path,
    cultmesh_store: &Path,
    runtime_id: &str,
    status: &str,
    reason: &str,
) -> Result<Value> {
    ensure_content(content, "Persona chat")?;
    let audit = audit_persona_speech(
        content,
        PersonaSpeechActionKind::Draft,
        artifact_dir,
        cultmesh_store,
        runtime_id,
        allowed_channel_id(config).as_deref(),
        false,
    )?;
    let path = write_draft(content, config, artifact_dir, status, reason, Some(&audit))?;
    Ok(serde_json::json!({
        "ok": status != "blocked",
        "posted": false,
        "draftPath": path,
        "speechAudit": audit,
    }))
}

fn run_bubble(
    content: &str,
    artifact_dir: &Path,
    cultmesh_store: &Path,
    runtime_id: &str,
    source: &str,
    status: &str,
    mood: &str,
) -> Result<Value> {
    ensure_content(content, "Persona bubble")?;
    let audit = audit_persona_speech(
        content,
        PersonaSpeechActionKind::Bubble,
        artifact_dir,
        cultmesh_store,
        runtime_id,
        None,
        false,
    )?;
    let payload = bubble_payload(content, source, status, mood);
    let path = artifact_dir.join(format!(
        "Persona-bubble-{}-{}.json",
        now_stamp(),
        short_id()
    ));
    write_json(&path, &payload)?;
    Ok(serde_json::json!({
        "ok": true,
        "posted": false,
        "bubblePath": path,
        "bubble": payload,
        "speechAudit": audit,
    }))
}

fn run_post(
    content: &str,
    config: &PersonaConfig,
    artifact_dir: &Path,
    cultmesh_store: &Path,
    runtime_id: &str,
    channel_id: Option<String>,
    persona_name: Option<String>,
    persona_avatar_url: Option<String>,
    reply_to_message_id: Option<String>,
) -> Result<Value> {
    ensure_content(content, "Persona chat")?;
    let configured_channel_id = allowed_channel_id(config);
    let requested_channel_id = channel_id.or_else(|| configured_channel_id.clone());
    let audit = audit_persona_speech(
        content,
        PersonaSpeechActionKind::Post,
        artifact_dir,
        cultmesh_store,
        runtime_id,
        requested_channel_id.as_deref(),
        true,
    )?;
    if audit.decision == "blocked" {
        let path = write_draft(
            content,
            config,
            artifact_dir,
            "blocked",
            &format!("speech audit blocked: {}", audit.reasons.join("; ")),
            Some(&audit),
        )?;
        return Ok(serde_json::json!({
            "ok": false,
            "posted": false,
            "blocked": "speech-audit",
            "draftPath": path,
            "speechAudit": audit,
        }));
    }
    let Some(configured_channel_id) = configured_channel_id else {
        let path = write_draft(
            content,
            config,
            artifact_dir,
            "blocked",
            "missing #aquarium channel id",
            Some(&audit),
        )?;
        return Ok(serde_json::json!({
            "ok": false,
            "posted": false,
            "blocked": "missing-channel-id",
            "draftPath": path,
            "speechAudit": audit,
        }));
    };
    if requested_channel_id.as_deref() != Some(configured_channel_id.as_str()) {
        let path = write_draft(
            content,
            config,
            artifact_dir,
            "blocked",
            "requested channel does not match configured #aquarium channel id",
            Some(&audit),
        )?;
        return Ok(serde_json::json!({
            "ok": false,
            "posted": false,
            "blocked": "wrong-channel",
            "draftPath": path,
            "speechAudit": audit,
        }));
    }
    let Some(token) = bot_token(config) else {
        let path = write_draft(
            content,
            config,
            artifact_dir,
            "blocked",
            "missing Discord bot token",
            Some(&audit),
        )?;
        return Ok(serde_json::json!({
            "ok": false,
            "posted": false,
            "blocked": "missing-token",
            "draftPath": path,
            "speechAudit": audit,
        }));
    };
    let persona = resolve_persona(config, persona_name, persona_avatar_url)?;
    let posted = post_discord_message(
        &token,
        &configured_channel_id,
        content.trim(),
        reply_to_message_id.as_deref(),
        persona.as_ref(),
        config,
        artifact_dir,
    )?;
    let path = write_draft(
        content,
        config,
        artifact_dir,
        "posted",
        &format!(
            "posted message {} via {}",
            posted.message_id, posted.transport
        ),
        Some(&audit),
    )?;
    Ok(serde_json::json!({
        "ok": true,
        "posted": true,
        "messageId": posted.message_id,
        "transport": posted.transport,
        "persona": persona.map(|persona| serde_json::json!({
            "name": persona.name,
            "avatarUrl": persona.avatar_url,
        })),
        "draftPath": path,
        "speechAudit": audit,
    }))
}

#[derive(Clone, Debug)]
struct PersonaPersona {
    name: String,
    avatar_url: Option<String>,
}

#[derive(Clone, Debug)]
struct PostedDiscordMessage {
    message_id: String,
    transport: &'static str,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum PersonaSpeechActionKind {
    Draft,
    Bubble,
    Post,
}

impl PersonaSpeechActionKind {
    fn as_str(self) -> &'static str {
        match self {
            PersonaSpeechActionKind::Draft => "draft",
            PersonaSpeechActionKind::Bubble => "bubble",
            PersonaSpeechActionKind::Post => "post",
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersonaSpeechAudit {
    #[serde(rename = "schema_version")]
    schema_version: String,
    audit_id: String,
    created_at: String,
    action_kind: PersonaSpeechActionKind,
    decision: String,
    reasons: Vec<String>,
    content_fingerprint: String,
    opening_key: String,
    topic_key: String,
    requested_channel_id: Option<String>,
    recent_window_count: usize,
    repeated_opening_count: usize,
    repeated_topic_count: usize,
    same_channel_post_count: usize,
    audit_path: PathBuf,
}

#[derive(Clone, Debug)]
struct RecentPersonaSpeech {
    opening_key: String,
    topic_key: String,
    channel_id: Option<String>,
    action_kind: PersonaSpeechActionKind,
}

fn audit_persona_speech(
    content: &str,
    action_kind: PersonaSpeechActionKind,
    artifact_dir: &Path,
    cultmesh_store: &Path,
    runtime_id: &str,
    requested_channel_id: Option<&str>,
    public_post: bool,
) -> Result<PersonaSpeechAudit> {
    let opening_key = opening_key(content);
    let topic_key = topic_key(content);
    let recent = recent_persona_speech(artifact_dir, 12);
    let repeated_opening_count = recent
        .iter()
        .filter(|speech| !opening_key.is_empty() && speech.opening_key == opening_key)
        .count();
    let repeated_topic_count = recent
        .iter()
        .filter(|speech| !topic_key.is_empty() && speech.topic_key == topic_key)
        .count();
    let same_channel_post_count = requested_channel_id
        .map(|channel_id| {
            recent
                .iter()
                .filter(|speech| {
                    speech.action_kind == PersonaSpeechActionKind::Post
                        && speech.channel_id.as_deref() == Some(channel_id)
                })
                .count()
        })
        .unwrap_or(0);
    let mut reasons = Vec::new();
    if public_post && repeated_opening_count >= 2 {
        reasons.push("repeated-opening".to_string());
    }
    if public_post && repeated_topic_count >= 3 {
        reasons.push("repeated-topic".to_string());
    }
    if public_post && same_channel_post_count >= 4 {
        reasons.push("channel-saturation".to_string());
    }
    if content.trim().len() > 1800 {
        reasons.push("content-too-long-for-public-persona".to_string());
    }
    let decision = if reasons.is_empty() {
        "eligible"
    } else {
        "blocked"
    }
    .to_string();
    let audit_id = format!("persona-speech-audit-{}", short_id());
    let audit_path = artifact_dir.join(format!(
        "Persona-speech-audit-{}-{}.json",
        now_stamp(),
        audit_id
    ));
    let audit = PersonaSpeechAudit {
        schema_version: SPEECH_AUDIT_SCHEMA_VERSION.to_string(),
        audit_id,
        created_at: now_iso(),
        action_kind,
        decision,
        reasons,
        content_fingerprint: content_fingerprint(content),
        opening_key,
        topic_key,
        requested_channel_id: requested_channel_id.map(str::to_string),
        recent_window_count: recent.len(),
        repeated_opening_count,
        repeated_topic_count,
        same_channel_post_count,
        audit_path,
    };
    write_json(&audit.audit_path, &serde_json::to_value(&audit)?)?;
    let cultmesh_entry = persona_speech_audit_cultmesh_entry(&audit, runtime_id);
    write_epiphany_cultmesh_persona_speech_audit(cultmesh_store, cultmesh_entry)?;
    Ok(audit)
}

fn persona_speech_audit_cultmesh_entry(
    audit: &PersonaSpeechAudit,
    runtime_id: &str,
) -> EpiphanyCultMeshPersonaSpeechAuditEntry {
    EpiphanyCultMeshPersonaSpeechAuditEntry {
        schema_version: EPIPHANY_CULTMESH_PERSONA_SPEECH_AUDIT_SCHEMA_VERSION.to_string(),
        audit_id: audit.audit_id.clone(),
        runtime_id: runtime_id.to_string(),
        verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
        persona_agent_id: "epiphany.Persona".to_string(),
        action_kind: audit.action_kind.as_str().to_string(),
        decision: audit.decision.clone(),
        content_fingerprint: audit.content_fingerprint.clone(),
        opening_key: audit.opening_key.clone(),
        topic_key: audit.topic_key.clone(),
        requested_channel_id: audit.requested_channel_id.clone().unwrap_or_default(),
        recent_window_count: audit.recent_window_count as u32,
        repeated_opening_count: audit.repeated_opening_count as u32,
        repeated_topic_count: audit.repeated_topic_count as u32,
        same_channel_post_count: audit.same_channel_post_count as u32,
        reasons: audit.reasons.clone(),
        artifact_ref: audit.audit_path.display().to_string(),
        created_at_utc: audit.created_at.clone(),
        private_state_exposed: false,
        notes: vec![
            "Persona speech audit is parent-side mouth policy; it stores fingerprints and counters, not raw Persona prose.".to_string(),
            "Public speech remains blocked when repetition or saturation trips before Discord transport work.".to_string(),
        ],
    }
}

fn resolve_persona(
    config: &PersonaConfig,
    persona_name: Option<String>,
    persona_avatar_url: Option<String>,
) -> Result<Option<PersonaPersona>> {
    let name = trim_optional(persona_name).or_else(|| trim_optional(config.persona_name.clone()));
    let avatar_url = trim_optional(persona_avatar_url)
        .or_else(|| trim_optional(config.persona_avatar_url.clone()));
    if name.is_none() && avatar_url.is_none() {
        return Ok(None);
    }
    let Some(name) = name else {
        return Err(anyhow!(
            "persona-name is required when posting with a persona avatar"
        ));
    };
    Ok(Some(PersonaPersona {
        name: name.chars().take(80).collect(),
        avatar_url,
    }))
}

fn post_discord_message(
    token: &str,
    channel_id: &str,
    content: &str,
    reply_to_message_id: Option<&str>,
    persona: Option<&PersonaPersona>,
    config: &PersonaConfig,
    artifact_dir: &Path,
) -> Result<PostedDiscordMessage> {
    if let Some(persona) = persona {
        return post_discord_persona_message(
            token,
            channel_id,
            content,
            reply_to_message_id,
            persona,
            config,
            artifact_dir,
        );
    }

    let response = Client::new()
        .post(format!("{DISCORD_API}/channels/{channel_id}/messages"))
        .header("Authorization", format!("Bot {token}"))
        .header("User-Agent", "EpiphanyPersona/0.1")
        .json(&serde_json::json!({
            "content": content,
            "message_reference": reply_to_message_id.map(|message_id| serde_json::json!({
                "message_id": message_id,
                "fail_if_not_exists": false,
            })),
            "allowed_mentions": {"parse": []},
        }))
        .send()
        .context("Discord post failed")?;
    let payload = decode_discord_json(response, "Discord message post")?;
    Ok(PostedDiscordMessage {
        message_id: required_discord_id(&payload, "Discord message post")?,
        transport: "bot",
    })
}

fn post_discord_persona_message(
    token: &str,
    channel_id: &str,
    content: &str,
    reply_to_message_id: Option<&str>,
    persona: &PersonaPersona,
    config: &PersonaConfig,
    artifact_dir: &Path,
) -> Result<PostedDiscordMessage> {
    let target = resolve_webhook_target(token, channel_id)?;
    let cache_path = persona_webhook_cache_path(config, artifact_dir);
    let mut webhook = cached_persona_webhook(&cache_path, &target.webhook_channel_id)?;
    if webhook.is_none() {
        let created = create_persona_webhook(token, &target.webhook_channel_id)?;
        write_cached_persona_webhook(&cache_path, &target.webhook_channel_id, &created)?;
        webhook = Some(created);
    }
    let webhook = webhook.context("persona webhook should exist")?;
    match execute_persona_webhook(&webhook, &target, content, reply_to_message_id, persona) {
        Ok(posted) => Ok(posted),
        Err(error) if is_stale_webhook_error(&error) => {
            clear_cached_persona_webhook(&cache_path, &target.webhook_channel_id)?;
            let refreshed = create_persona_webhook(token, &target.webhook_channel_id)?;
            write_cached_persona_webhook(&cache_path, &target.webhook_channel_id, &refreshed)?;
            execute_persona_webhook(&refreshed, &target, content, reply_to_message_id, persona)
        }
        Err(error) => Err(error),
    }
}

#[derive(Clone, Debug)]
struct WebhookTarget {
    webhook_channel_id: String,
    thread_id: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CachedWebhookRecord {
    id: String,
    token: String,
    channel_id: String,
    name: String,
    created_at: String,
}

fn resolve_webhook_target(token: &str, channel_id: &str) -> Result<WebhookTarget> {
    let response = Client::new()
        .get(format!("{DISCORD_API}/channels/{channel_id}"))
        .header("Authorization", format!("Bot {token}"))
        .header("User-Agent", "EpiphanyPersona/0.1")
        .send()
        .context("Discord channel lookup failed")?;
    let payload = decode_discord_json(response, "Discord channel lookup")?;
    let channel_type = payload["type"]
        .as_i64()
        .ok_or_else(|| anyhow!("Discord channel lookup returned no numeric type"))?;
    let id = required_discord_id(&payload, "Discord channel lookup")?;
    if matches!(channel_type, 10 | 11 | 12) {
        let parent_id = payload["parent_id"].as_str().ok_or_else(|| {
            anyhow!("Discord thread {id} has no parent channel for webhook routing")
        })?;
        return Ok(WebhookTarget {
            webhook_channel_id: parent_id.to_string(),
            thread_id: Some(id),
        });
    }
    Ok(WebhookTarget {
        webhook_channel_id: id,
        thread_id: None,
    })
}

fn create_persona_webhook(token: &str, channel_id: &str) -> Result<CachedWebhookRecord> {
    let response = Client::new()
        .post(format!("{DISCORD_API}/channels/{channel_id}/webhooks"))
        .header("Authorization", format!("Bot {token}"))
        .header("User-Agent", "EpiphanyPersona/0.1")
        .json(&serde_json::json!({"name": "Epiphany Persona Pipe"}))
        .send()
        .context("Discord webhook creation failed")?;
    let payload = decode_discord_json(response, "Discord webhook creation")?;
    let id = required_discord_id(&payload, "Discord webhook creation")?;
    let token = payload["token"]
        .as_str()
        .ok_or_else(|| anyhow!("Discord webhook creation returned no executable token"))?;
    Ok(CachedWebhookRecord {
        id,
        token: token.to_string(),
        channel_id: channel_id.to_string(),
        name: "Epiphany Persona Pipe".to_string(),
        created_at: now_iso(),
    })
}

fn execute_persona_webhook(
    webhook: &CachedWebhookRecord,
    target: &WebhookTarget,
    content: &str,
    reply_to_message_id: Option<&str>,
    persona: &PersonaPersona,
) -> Result<PostedDiscordMessage> {
    let mut url = format!(
        "{DISCORD_API}/webhooks/{}/{}?wait=true",
        webhook.id, webhook.token
    );
    if let Some(thread_id) = &target.thread_id {
        url.push_str("&thread_id=");
        url.push_str(thread_id);
    }
    let response = Client::new()
        .post(url)
        .header("User-Agent", "EpiphanyPersona/0.1")
        .json(&serde_json::json!({
            "content": content,
            "username": persona.name,
            "avatar_url": persona.avatar_url,
            "message_reference": reply_to_message_id.map(|message_id| serde_json::json!({
                "message_id": message_id,
                "fail_if_not_exists": false,
            })),
            "allowed_mentions": {"parse": []},
        }))
        .send()
        .context("Discord webhook execution failed")?;
    let payload = decode_discord_json(response, "Discord webhook execution")?;
    Ok(PostedDiscordMessage {
        message_id: required_discord_id(&payload, "Discord webhook execution")?,
        transport: "webhook",
    })
}

fn run_smoke() -> Result<Value> {
    let temp_dir = scoped_temp_dir("epiphany-persona-discord-smoke")?;
    let cultmesh_store = temp_dir.join("persona-speech.ccmp");
    let runtime_id = "epiphany-persona-smoke";
    let config = PersonaConfig {
        allowed_channel_name: "#aquarium".to_string(),
        allowed_channel_id: None,
        allowed_channel_id_env: Some("EPIPHANY_PERSONA_AQUARIUM_CHANNEL_ID_TEST".to_string()),
        bot_token_env: Some("DISCORD_BOT_TOKEN_TEST".to_string()),
        persona_name: Some("Smoke Persona".to_string()),
        persona_avatar_url: Some("https://example.invalid/Persona.png".to_string()),
        webhook_cache_path: Some(temp_dir.join("webhook-cache.json")),
    };
    unsafe {
        env::remove_var("EPIPHANY_PERSONA_AQUARIUM_CHANNEL_ID_TEST");
    }
    let draft = run_draft(
        "Persona notices Modeling and Soul disagree about evidence shape.",
        &config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        "draft",
        "drafted without posting",
    )?;
    let bubble = run_bubble(
        "Persona opens an Aquarium bubble even while Discord is unavailable.",
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        "smoke/Persona",
        "ready",
        "attentive",
    )?;
    let blocked = run_post(
        "Persona should not post without a configured #aquarium channel id.",
        &config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        None,
        None,
        None,
        None,
    )?;
    unsafe {
        env::set_var("EPIPHANY_PERSONA_AQUARIUM_CHANNEL_ID_TEST", "123");
    }
    let wrong = run_post(
        "Persona should not post outside #aquarium.",
        &config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        Some("456".to_string()),
        Some("Wrong Persona".to_string()),
        None,
        None,
    )?;
    let repeated_audit_seed = audit_persona_speech(
        "Rite noted: Modeling and Soul keep circling the same evidence seam.",
        PersonaSpeechActionKind::Post,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        Some("123"),
        false,
    )?;
    write_draft(
        "Rite noted: Modeling and Soul keep circling the same evidence seam.",
        &config,
        &temp_dir,
        "posted",
        "seed prior posted Persona output for speech-audit smoke",
        Some(&repeated_audit_seed),
    )?;
    write_draft(
        "Rite noted: Modeling and Soul keep circling the same evidence seam.",
        &config,
        &temp_dir,
        "posted",
        "seed second prior posted Persona output for speech-audit smoke",
        Some(&repeated_audit_seed),
    )?;
    let repeated = run_post(
        "Rite noted: Modeling and Soul keep circling the same evidence seam again.",
        &config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        Some("123".to_string()),
        None,
        None,
        None,
    )?;
    let latest_cultmesh_audit =
        load_latest_epiphany_cultmesh_persona_speech_audit(&cultmesh_store, runtime_id)?
            .context("Persona speech audit smoke expected latest CultMesh audit")?;
    let ok = draft["ok"] == true
        && bubble["ok"] == true
        && bubble["bubble"]["schema_version"] == BUBBLE_SCHEMA_VERSION
        && bubble["bubble"]["bubble"]["requiresDiscord"] == false
        && blocked["ok"] == false
        && blocked["blocked"] == "missing-channel-id"
        && wrong["ok"] == false
        && wrong["blocked"] == "wrong-channel"
        && repeated["ok"] == false
        && repeated["blocked"] == "speech-audit"
        && repeated["speechAudit"]["decision"] == "blocked"
        && repeated["speechAudit"]["reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| reason == "repeated-opening"))
        && latest_cultmesh_audit.decision == "blocked"
        && latest_cultmesh_audit
            .reasons
            .iter()
            .any(|reason| reason == "repeated-opening")
        && !latest_cultmesh_audit.private_state_exposed;
    let result = serde_json::json!({
        "ok": ok,
        "draft": draft,
        "bubble": bubble,
        "blocked": blocked,
        "wrongChannel": wrong,
        "repeatedSpeech": repeated,
        "latestCultMeshSpeechAudit": latest_cultmesh_audit,
    });
    let _ = fs::remove_dir_all(&temp_dir);
    Ok(result)
}

fn write_draft(
    content: &str,
    config: &PersonaConfig,
    artifact_dir: &Path,
    status: &str,
    reason: &str,
    speech_audit: Option<&PersonaSpeechAudit>,
) -> Result<PathBuf> {
    let payload = serde_json::json!({
        "schema_version": CHAT_SCHEMA_VERSION,
        "created_at": now_iso(),
        "status": status,
        "reason": reason,
        "allowed_channel_name": if config.allowed_channel_name.is_empty() { "#aquarium" } else { &config.allowed_channel_name },
        "allowed_channel_id": allowed_channel_id(config),
        "persona_name": config.persona_name,
        "persona_avatar_url": config.persona_avatar_url,
        "content": content.trim(),
        "speechAudit": speech_audit,
    });
    let path = artifact_dir.join(format!("Persona-chat-{}-{}.json", now_stamp(), short_id()));
    write_json(&path, &payload)?;
    Ok(path)
}

fn bubble_payload(content: &str, source: &str, status: &str, mood: &str) -> Value {
    serde_json::json!({
        "schema_version": BUBBLE_SCHEMA_VERSION,
        "created_at": now_iso(),
        "status": status,
        "source": source,
        "target": "aquarium",
        "role_id": "Persona",
        "agent_id": "Persona",
        "display_name": "Persona",
        "mood": mood,
        "content": content.trim(),
        "bubble": {
            "kind": "agent-chat",
            "anchorRoleId": "Persona",
            "opensIn": "aquarium",
            "requiresDiscord": false,
            "ttlSeconds": 90,
        },
    })
}

fn latest_persona_artifacts(artifact_dir: &Path, limit: usize) -> Vec<Value> {
    let Ok(read_dir) = fs::read_dir(artifact_dir) else {
        return Vec::new();
    };
    let mut paths = read_dir
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("Persona-") && name.ends_with(".json"))
        })
        .filter_map(|path| Some((path.metadata().ok()?.modified().ok()?, path)))
        .collect::<Vec<_>>();
    paths.sort_by_key(|item| Reverse(item.0));
    paths
        .into_iter()
        .take(limit)
        .filter_map(|(modified, path)| {
            let payload: Value = serde_json::from_str(&fs::read_to_string(&path).ok()?).ok()?;
            let modified_at: chrono::DateTime<chrono::Utc> = modified.into();
            Some(serde_json::json!({
                "path": path,
                "name": path.file_name().and_then(|name| name.to_str()),
                "modifiedAt": modified_at.to_rfc3339_opts(SecondsFormat::Secs, true),
                "schemaVersion": payload.get("schema_version"),
                "status": payload.get("status"),
                "reason": payload.get("reason"),
                "content": payload.get("content"),
                "bubble": payload.get("bubble"),
                "source": payload.get("source"),
            }))
        })
        .collect()
}

fn recent_persona_speech(artifact_dir: &Path, limit: usize) -> Vec<RecentPersonaSpeech> {
    let Ok(read_dir) = fs::read_dir(artifact_dir) else {
        return Vec::new();
    };
    let mut paths = read_dir
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with("Persona-chat-") || name.starts_with("Persona-bubble-")
                })
        })
        .filter_map(|path| Some((path.metadata().ok()?.modified().ok()?, path)))
        .collect::<Vec<_>>();
    paths.sort_by_key(|item| Reverse(item.0));
    paths
        .into_iter()
        .take(limit)
        .filter_map(|(_, path)| {
            let payload: Value = serde_json::from_str(&fs::read_to_string(&path).ok()?).ok()?;
            let content = payload["content"]
                .as_str()
                .or_else(|| payload["bubble"]["content"].as_str())?
                .to_string();
            let status = payload["status"].as_str().unwrap_or_default();
            let action_kind = if payload["schema_version"] == BUBBLE_SCHEMA_VERSION {
                PersonaSpeechActionKind::Bubble
            } else if status == "posted" {
                PersonaSpeechActionKind::Post
            } else {
                PersonaSpeechActionKind::Draft
            };
            Some(RecentPersonaSpeech {
                opening_key: opening_key(&content),
                topic_key: topic_key(&content),
                channel_id: payload["allowed_channel_id"].as_str().map(str::to_string),
                action_kind,
            })
        })
        .collect()
}

fn opening_key(content: &str) -> String {
    content
        .split_whitespace()
        .take(5)
        .map(normalize_token)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn topic_key(content: &str) -> String {
    let mut tokens = content
        .split_whitespace()
        .map(normalize_token)
        .filter(|token| token.len() >= 4 && !persona_stopword(token))
        .collect::<Vec<_>>();
    tokens.sort();
    tokens.dedup();
    tokens.into_iter().take(6).collect::<Vec<_>>().join("|")
}

fn content_fingerprint(content: &str) -> String {
    topic_key(content)
        + "::"
        + &content
            .split_whitespace()
            .map(normalize_token)
            .filter(|token| !token.is_empty())
            .take(24)
            .collect::<Vec<_>>()
            .join("-")
}

fn normalize_token(value: &str) -> String {
    value
        .trim_matches(|ch: char| !ch.is_ascii_alphanumeric())
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
        .collect::<String>()
        .to_ascii_lowercase()
}

fn persona_stopword(value: &str) -> bool {
    matches!(
        value,
        "about"
            | "after"
            | "again"
            | "because"
            | "before"
            | "being"
            | "should"
            | "there"
            | "these"
            | "thing"
            | "through"
            | "under"
            | "without"
            | "would"
    )
}

fn persona_webhook_cache_path(config: &PersonaConfig, artifact_dir: &Path) -> PathBuf {
    config
        .webhook_cache_path
        .clone()
        .unwrap_or_else(|| artifact_dir.join("discord-webhook-cache.json"))
}

fn cached_persona_webhook(path: &Path, channel_id: &str) -> Result<Option<CachedWebhookRecord>> {
    let cache = read_persona_webhook_cache(path)?;
    let Some(value) = cache.get(channel_id) else {
        return Ok(None);
    };
    let record: CachedWebhookRecord = serde_json::from_value(value.clone())
        .with_context(|| format!("cached persona webhook for channel {channel_id} is malformed"))?;
    Ok((record.channel_id == channel_id).then_some(record))
}

fn read_persona_webhook_cache(path: &Path) -> Result<serde_json::Map<String, Value>> {
    match fs::read_to_string(path) {
        Ok(raw) => {
            let value: Value = serde_json::from_str(strip_bom(&raw))
                .with_context(|| format!("failed to decode {}", path.display()))?;
            Ok(value.as_object().cloned().unwrap_or_default())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(serde_json::Map::new()),
        Err(error) => Err(error).with_context(|| format!("failed to read {}", path.display())),
    }
}

fn write_cached_persona_webhook(
    path: &Path,
    channel_id: &str,
    webhook: &CachedWebhookRecord,
) -> Result<()> {
    let mut cache = read_persona_webhook_cache(path)?;
    cache.insert(channel_id.to_string(), serde_json::to_value(webhook)?);
    write_json(path, &Value::Object(cache))
}

fn clear_cached_persona_webhook(path: &Path, channel_id: &str) -> Result<()> {
    let mut cache = read_persona_webhook_cache(path)?;
    if cache.remove(channel_id).is_some() {
        write_json(path, &Value::Object(cache))?;
    }
    Ok(())
}

fn decode_discord_json(response: reqwest::blocking::Response, context: &str) -> Result<Value> {
    let status = response.status();
    let body = response
        .text()
        .with_context(|| format!("{context} response body read failed"))?;
    if !status.is_success() {
        return Err(anyhow!("{context} failed with HTTP {status}: {body}"));
    }
    serde_json::from_str(&body).with_context(|| format!("{context} response was not JSON"))
}

fn required_discord_id(payload: &Value, context: &str) -> Result<String> {
    payload["id"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| anyhow!("{context} returned no id"))
}

fn is_stale_webhook_error(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message.contains("Discord webhook execution failed with HTTP 401")
        || message.contains("Discord webhook execution failed with HTTP 404")
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn strip_bom(value: &str) -> &str {
    value.strip_prefix('\u{feff}').unwrap_or(value)
}

fn load_config(path: &Path) -> Result<PersonaConfig> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    if path
        .extension()
        .is_some_and(|extension| extension == "json")
    {
        let payload: Value = serde_json::from_str(&raw)
            .with_context(|| format!("failed to decode {}", path.display()))?;
        return Ok(PersonaConfig {
            allowed_channel_name: payload["allowed_channel_name"]
                .as_str()
                .unwrap_or("#aquarium")
                .to_string(),
            allowed_channel_id: payload["allowed_channel_id"]
                .as_str()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            allowed_channel_id_env: payload["allowed_channel_id_env"]
                .as_str()
                .map(str::to_string),
            bot_token_env: payload["bot_token_env"].as_str().map(str::to_string),
            persona_name: payload["persona_name"].as_str().map(str::to_string),
            persona_avatar_url: payload["persona_avatar_url"].as_str().map(str::to_string),
            webhook_cache_path: payload["webhook_cache_path"].as_str().map(PathBuf::from),
        });
    }
    Ok(PersonaConfig {
        allowed_channel_name: toml_string(&raw, "allowed_channel_name")
            .unwrap_or_else(|| "#aquarium".to_string()),
        allowed_channel_id: toml_string(&raw, "allowed_channel_id"),
        allowed_channel_id_env: toml_string(&raw, "allowed_channel_id_env"),
        bot_token_env: toml_string(&raw, "bot_token_env"),
        persona_name: toml_string(&raw, "persona_name"),
        persona_avatar_url: toml_string(&raw, "persona_avatar_url"),
        webhook_cache_path: toml_string(&raw, "webhook_cache_path").map(PathBuf::from),
    })
}

fn toml_string(raw: &str, key: &str) -> Option<String> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .find_map(|line| {
            let (left, right) = line.split_once('=')?;
            (left.trim() == key).then(|| parse_quoted(right.trim()))?
        })
}

fn parse_quoted(value: &str) -> Option<String> {
    let value = value.trim();
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_string)
}

fn allowed_channel_id(config: &PersonaConfig) -> Option<String> {
    config
        .allowed_channel_id
        .clone()
        .or_else(|| config.allowed_channel_id_env.as_ref().and_then(env_value))
}

fn bot_token(config: &PersonaConfig) -> Option<String> {
    config.bot_token_env.as_ref().and_then(env_value)
}

fn env_value(name: &String) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_content(content: Option<String>) -> Result<String> {
    let Some(content) = content else {
        let mut value = String::new();
        std::io::stdin()
            .read_to_string(&mut value)
            .context("failed to read stdin")?;
        return Ok(value);
    };
    let path = PathBuf::from(&content);
    if path.exists() {
        return fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()));
    }
    Ok(content)
}

fn ensure_content(content: &str, label: &str) -> Result<()> {
    if content.trim().is_empty() {
        return Err(anyhow!("{label} content is empty"));
    }
    Ok(())
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))
        .with_context(|| format!("failed to write {}", path.display()))
}

fn now_iso() -> String {
    chrono::Utc::now()
        .to_rfc3339_opts(SecondsFormat::Secs, true)
        .replace('Z', "+00:00")
}

fn now_stamp() -> String {
    chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

fn short_id() -> String {
    Uuid::new_v4().simple().to_string()[..8].to_string()
}

fn scoped_temp_dir(prefix: &str) -> Result<PathBuf> {
    let path = env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&path).with_context(|| format!("failed to create {}", path.display()))?;
    Ok(path)
}

fn next_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(next_value(args, name)?))
}

fn next_value(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("{name} requires a value"))
}

fn usage() -> Result<()> {
    Err(anyhow!(
        "usage: epiphany-persona-discord <draft|bubble|post|latest|smoke> [--config <path>] [--artifact-dir <path>] [--content <text-or-path>]"
    ))
}
