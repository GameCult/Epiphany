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
            run_post(&content, &config, &artifact_dir, channel_id)?
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
    let response = Client::new()
        .post(format!(
            "{DISCORD_API}/channels/{configured_channel_id}/messages"
        ))
        .header("Authorization", format!("Bot {token}"))
        .header("User-Agent", "EpiphanyFace/0.1")
        .json(&serde_json::json!({ "content": content.trim() }))
        .send()
        .context("Discord post failed")?;
    if !response.status().is_success() {
        return Err(anyhow!(
            "Discord post failed with HTTP {}: {}",
            response.status(),
            response.text().unwrap_or_default()
        ));
    }
    let payload: Value = response.json().context("Discord response was not JSON")?;
    let path = write_draft(
        content,
        config,
        artifact_dir,
        "posted",
        &format!(
            "posted message {}",
            payload
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        ),
    )?;
    Ok(serde_json::json!({
        "ok": true,
        "posted": true,
        "messageId": payload.get("id"),
        "draftPath": path,
    }))
}

fn run_smoke() -> Result<Value> {
    let temp_dir = scoped_temp_dir("epiphany-face-discord-smoke")?;
    let config = FaceConfig {
        allowed_channel_name: "#aquarium".to_string(),
        allowed_channel_id: None,
        allowed_channel_id_env: Some("EPIPHANY_FACE_AQUARIUM_CHANNEL_ID_TEST".to_string()),
        bot_token_env: Some("DISCORD_BOT_TOKEN_TEST".to_string()),
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
    )?;
    unsafe {
        env::set_var("EPIPHANY_FACE_AQUARIUM_CHANNEL_ID_TEST", "123");
    }
    let wrong = run_post(
        "Face should not post outside #aquarium.",
        &config,
        &temp_dir,
        Some("456".to_string()),
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
        });
    }
    Ok(FaceConfig {
        allowed_channel_name: toml_string(&raw, "allowed_channel_name")
            .unwrap_or_else(|| "#aquarium".to_string()),
        allowed_channel_id: toml_string(&raw, "allowed_channel_id"),
        allowed_channel_id_env: toml_string(&raw, "allowed_channel_id_env"),
        bot_token_env: toml_string(&raw, "bot_token_env"),
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
