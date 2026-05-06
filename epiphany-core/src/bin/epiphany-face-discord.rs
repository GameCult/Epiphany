use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::SecondsFormat;
use reqwest::blocking::Client;
use serde_json::Value;
use std::cmp::Reverse;
use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

const CHAT_SCHEMA_VERSION: &str = "epiphany.face_chat.v0";
const BUBBLE_SCHEMA_VERSION: &str = "epiphany.face_bubble.v0";
const DISCORD_API: &str = "https://discord.com/api/v10";

#[derive(Clone, Debug, Default)]
struct FaceConfig {
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
    let mut config_path = PathBuf::from("state/face-discord.toml");
    let mut artifact_dir = PathBuf::from(".epiphany-face");
    let mut content: Option<String> = None;
    let mut channel_id: Option<String> = None;
    let mut source = "epiphany/face".to_string();
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
                "draft",
                "drafted without posting",
            )?
        }
        "bubble" => {
            let content = read_content(content)?;
            run_bubble(&content, &artifact_dir, &source, &status, &mood)?
        }
        "post" => {
            let config = load_config(&config_path)?;
            let content = read_content(content)?;
            run_post(
                &content,
                &config,
                &artifact_dir,
                channel_id,
                persona_name,
                persona_avatar_url,
                reply_to_message_id,
            )?
        }
        "latest" => serde_json::json!({
            "ok": true,
            "artifactDir": artifact_dir,
            "latestArtifacts": latest_face_artifacts(&artifact_dir, limit),
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
    config: &FaceConfig,
    artifact_dir: &Path,
    status: &str,
    reason: &str,
) -> Result<Value> {
    ensure_content(content, "Face chat")?;
    let path = write_draft(content, config, artifact_dir, status, reason)?;
    Ok(serde_json::json!({
        "ok": status != "blocked",
        "posted": false,
        "draftPath": path,
    }))
}

fn run_bubble(
    content: &str,
    artifact_dir: &Path,
    source: &str,
    status: &str,
    mood: &str,
) -> Result<Value> {
    ensure_content(content, "Face bubble")?;
    let payload = bubble_payload(content, source, status, mood);
    let path = artifact_dir.join(format!("face-bubble-{}-{}.json", now_stamp(), short_id()));
    write_json(&path, &payload)?;
    Ok(serde_json::json!({
        "ok": true,
        "posted": false,
        "bubblePath": path,
        "bubble": payload,
    }))
}

fn run_post(
    content: &str,
    config: &FaceConfig,
    artifact_dir: &Path,
    channel_id: Option<String>,
    persona_name: Option<String>,
    persona_avatar_url: Option<String>,
    reply_to_message_id: Option<String>,
) -> Result<Value> {
    ensure_content(content, "Face chat")?;
    let configured_channel_id = allowed_channel_id(config);
    let requested_channel_id = channel_id.or_else(|| configured_channel_id.clone());
    let Some(configured_channel_id) = configured_channel_id else {
        let path = write_draft(
            content,
            config,
            artifact_dir,
            "blocked",
            "missing #aquarium channel id",
        )?;
        return Ok(serde_json::json!({
            "ok": false,
            "posted": false,
            "blocked": "missing-channel-id",
            "draftPath": path,
        }));
    };
    if requested_channel_id.as_deref() != Some(configured_channel_id.as_str()) {
        let path = write_draft(
            content,
            config,
            artifact_dir,
            "blocked",
            "requested channel does not match configured #aquarium channel id",
        )?;
        return Ok(serde_json::json!({
            "ok": false,
            "posted": false,
            "blocked": "wrong-channel",
            "draftPath": path,
        }));
    }
    let Some(token) = bot_token(config) else {
        let path = write_draft(
            content,
            config,
            artifact_dir,
            "blocked",
            "missing Discord bot token",
        )?;
        return Ok(serde_json::json!({
            "ok": false,
            "posted": false,
            "blocked": "missing-token",
            "draftPath": path,
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
    }))
}

#[derive(Clone, Debug)]
struct FacePersona {
    name: String,
    avatar_url: Option<String>,
}

#[derive(Clone, Debug)]
struct PostedDiscordMessage {
    message_id: String,
    transport: &'static str,
}

fn resolve_persona(
    config: &FaceConfig,
    persona_name: Option<String>,
    persona_avatar_url: Option<String>,
) -> Result<Option<FacePersona>> {
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
    Ok(Some(FacePersona {
        name: name.chars().take(80).collect(),
        avatar_url,
    }))
}

fn post_discord_message(
    token: &str,
    channel_id: &str,
    content: &str,
    reply_to_message_id: Option<&str>,
    persona: Option<&FacePersona>,
    config: &FaceConfig,
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
        .header("User-Agent", "EpiphanyFace/0.1")
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
    persona: &FacePersona,
    config: &FaceConfig,
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
        .header("User-Agent", "EpiphanyFace/0.1")
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
        .header("User-Agent", "EpiphanyFace/0.1")
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
    persona: &FacePersona,
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
        .header("User-Agent", "EpiphanyFace/0.1")
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
    let temp_dir = scoped_temp_dir("epiphany-face-discord-smoke")?;
    let config = FaceConfig {
        allowed_channel_name: "#aquarium".to_string(),
        allowed_channel_id: None,
        allowed_channel_id_env: Some("EPIPHANY_FACE_AQUARIUM_CHANNEL_ID_TEST".to_string()),
        bot_token_env: Some("DISCORD_BOT_TOKEN_TEST".to_string()),
        persona_name: Some("Smoke Face".to_string()),
        persona_avatar_url: Some("https://example.invalid/face.png".to_string()),
        webhook_cache_path: Some(temp_dir.join("webhook-cache.json")),
    };
    unsafe {
        env::remove_var("EPIPHANY_FACE_AQUARIUM_CHANNEL_ID_TEST");
    }
    let draft = run_draft(
        "Face notices Body and Soul disagree about evidence shape.",
        &config,
        &temp_dir,
        "draft",
        "drafted without posting",
    )?;
    let bubble = run_bubble(
        "Face opens an Aquarium bubble even while Discord is unavailable.",
        &temp_dir,
        "smoke/face",
        "ready",
        "attentive",
    )?;
    let blocked = run_post(
        "Face should not post without a configured #aquarium channel id.",
        &config,
        &temp_dir,
        None,
        None,
        None,
        None,
    )?;
    unsafe {
        env::set_var("EPIPHANY_FACE_AQUARIUM_CHANNEL_ID_TEST", "123");
    }
    let wrong = run_post(
        "Face should not post outside #aquarium.",
        &config,
        &temp_dir,
        Some("456".to_string()),
        Some("Wrong Face".to_string()),
        None,
        None,
    )?;
    let ok = draft["ok"] == true
        && bubble["ok"] == true
        && bubble["bubble"]["schema_version"] == BUBBLE_SCHEMA_VERSION
        && bubble["bubble"]["bubble"]["requiresDiscord"] == false
        && blocked["ok"] == false
        && blocked["blocked"] == "missing-channel-id"
        && wrong["ok"] == false
        && wrong["blocked"] == "wrong-channel";
    let result = serde_json::json!({
        "ok": ok,
        "draft": draft,
        "bubble": bubble,
        "blocked": blocked,
        "wrongChannel": wrong,
    });
    let _ = fs::remove_dir_all(&temp_dir);
    Ok(result)
}

fn write_draft(
    content: &str,
    config: &FaceConfig,
    artifact_dir: &Path,
    status: &str,
    reason: &str,
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
    });
    let path = artifact_dir.join(format!("face-chat-{}-{}.json", now_stamp(), short_id()));
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
        "role_id": "face",
        "agent_id": "face",
        "display_name": "Face",
        "mood": mood,
        "content": content.trim(),
        "bubble": {
            "kind": "agent-chat",
            "anchorRoleId": "face",
            "opensIn": "aquarium",
            "requiresDiscord": false,
            "ttlSeconds": 90,
        },
    })
}

fn latest_face_artifacts(artifact_dir: &Path, limit: usize) -> Vec<Value> {
    let Ok(read_dir) = fs::read_dir(artifact_dir) else {
        return Vec::new();
    };
    let mut paths = read_dir
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("face-") && name.ends_with(".json"))
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

fn persona_webhook_cache_path(config: &FaceConfig, artifact_dir: &Path) -> PathBuf {
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

fn load_config(path: &Path) -> Result<FaceConfig> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    if path
        .extension()
        .is_some_and(|extension| extension == "json")
    {
        let payload: Value = serde_json::from_str(&raw)
            .with_context(|| format!("failed to decode {}", path.display()))?;
        return Ok(FaceConfig {
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
    Ok(FaceConfig {
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

fn allowed_channel_id(config: &FaceConfig) -> Option<String> {
    config
        .allowed_channel_id
        .clone()
        .or_else(|| config.allowed_channel_id_env.as_ref().and_then(env_value))
}

fn bot_token(config: &FaceConfig) -> Option<String> {
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
        "usage: epiphany-face-discord <draft|bubble|post|latest|smoke> [--config <path>] [--artifact-dir <path>] [--content <text-or-path>]"
    ))
}
