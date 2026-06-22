use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use chrono::SecondsFormat;
use epiphany_core::load_latest_epiphany_cultmesh_persona_speech_audit;
use epiphany_core::write_epiphany_cultmesh_persona_speech_audit;
use epiphany_core::EpiphanyCultMeshPersonaSpeechAuditEntry;
use epiphany_core::EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_PERSONA_SPEECH_AUDIT_SCHEMA_VERSION;
use serde_json::Value;
use std::cmp::Reverse;
use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

const CHAT_SCHEMA_VERSION: &str = "epiphany.persona_chat.v0";
const BUBBLE_SCHEMA_VERSION: &str = "epiphany.persona_bubble.v0";
const SPEECH_AUDIT_SCHEMA_VERSION: &str = "epiphany.persona_speech_audit.v0";

#[derive(Clone, Debug, Default)]
struct PersonaConfig {
    allowed_channel_name: String,
    allowed_channel_id: Option<String>,
    allowed_channel_id_env: Option<String>,
    persona_name: Option<String>,
    persona_avatar_url: Option<String>,
    bifrost_bridge_cli_path: Option<PathBuf>,
    bifrost_identity: Option<String>,
    bifrost_source_kind: Option<String>,
    bifrost_authority_ref: Option<String>,
    heimdall_capability_ref_env: Option<String>,
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
    let Some(bridge_cli_path) = bifrost_bridge_cli_path(config) else {
        let path = write_draft(
            content,
            config,
            artifact_dir,
            "blocked",
            "missing Bifrost bridge CLI path",
            Some(&audit),
        )?;
        return Ok(serde_json::json!({
            "ok": false,
            "posted": false,
            "blocked": "missing-bifrost-bridge",
            "draftPath": path,
            "speechAudit": audit,
        }));
    };
    let persona = resolve_persona(config, persona_name, persona_avatar_url)?;
    let posted = post_bifrost_discord_message(
        &bridge_cli_path,
        config,
        &configured_channel_id,
        content.trim(),
        reply_to_message_id.as_deref(),
        persona.as_ref(),
        &audit,
    )?;
    let path = write_draft(
        content,
        config,
        artifact_dir,
        "posted",
        &format!(
            "posted message {} through {}",
            posted.message_id, posted.transport
        ),
        Some(&audit),
    )?;
    Ok(serde_json::json!({
        "ok": true,
        "posted": true,
        "messageId": posted.message_id,
        "transport": posted.transport,
        "receiptUrl": posted.receipt_url,
        "externalReceiptId": posted.external_receipt_id,
        "bifrostBridgeReceipt": posted.bridge_receipt,
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
    transport: String,
    receipt_url: Option<String>,
    external_receipt_id: Option<String>,
    bridge_receipt: Value,
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

fn post_bifrost_discord_message(
    bridge_cli_path: &Path,
    config: &PersonaConfig,
    channel_id: &str,
    content: &str,
    reply_to_message_id: Option<&str>,
    persona: Option<&PersonaPersona>,
    audit: &PersonaSpeechAudit,
) -> Result<PostedDiscordMessage> {
    let mut command = Command::new(bifrost_node_executable());
    command.arg(bridge_cli_path);
    command.arg("discord-post");
    command.arg("--channel-id").arg(channel_id);
    command.arg("--content").arg(content);
    command.arg("--identity").arg(
        config
            .bifrost_identity
            .as_deref()
            .unwrap_or("epiphany.Persona"),
    );
    command.arg("--source-kind").arg(
        config
            .bifrost_source_kind
            .as_deref()
            .unwrap_or("epiphany_persona_speech"),
    );
    command.arg("--source-id").arg(&audit.audit_id);
    command.arg("--authority-ref").arg(
        config
            .bifrost_authority_ref
            .as_deref()
            .unwrap_or("epiphany.persona_speech_audit"),
    );
    command.arg("--epiphany-lane-id").arg("Persona");
    command
        .arg("--epiphany-agent-identity")
        .arg("epiphany.Persona");
    if let Some(run_id) = env::var("EPIPHANY_RUN_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        command.arg("--epiphany-run-id").arg(run_id);
    }
    if let Some(ref_name) = &config.heimdall_capability_ref_env {
        if let Some(value) = env_value(ref_name) {
            command.arg("--heimdall-capability-ref").arg(value);
        }
    }
    if let Some(message_id) = reply_to_message_id {
        command.arg("--reply-to-message-id").arg(message_id);
    }
    if let Some(persona) = persona {
        command.arg("--persona-name").arg(&persona.name);
        if let Some(avatar_url) = &persona.avatar_url {
            command.arg("--persona-avatar-url").arg(avatar_url);
        }
    }

    let output = command.output().with_context(|| {
        format!(
            "failed to invoke Bifrost bridge CLI at {}",
            bridge_cli_path.display()
        )
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow!(
            "Bifrost bridge Discord post failed with status {}: {}{}{}",
            output.status,
            stderr.trim(),
            if stdout.trim().is_empty() {
                ""
            } else {
                "\nstdout: "
            },
            stdout.trim()
        ));
    }
    let stdout = String::from_utf8(output.stdout).context("Bifrost bridge stdout was not UTF-8")?;
    let receipt: Value =
        serde_json::from_str(strip_bom(&stdout)).context("Bifrost bridge stdout was not JSON")?;
    let message_id = receipt["messageId"]
        .as_str()
        .or_else(|| receipt["externalReceiptId"].as_str())
        .unwrap_or("bifrost-discord-post")
        .to_string();
    Ok(PostedDiscordMessage {
        message_id,
        transport: "bifrost.discord-post".to_string(),
        receipt_url: receipt["url"].as_str().map(str::to_string),
        external_receipt_id: receipt["externalReceiptId"].as_str().map(str::to_string),
        bridge_receipt: receipt,
    })
}

fn bifrost_node_executable() -> String {
    env::var("EPIPHANY_BIFROST_NODE")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "node".to_string())
}

fn run_smoke() -> Result<Value> {
    let temp_dir = scoped_temp_dir("epiphany-persona-discord-smoke")?;
    let cultmesh_store = temp_dir.join("persona-speech.ccmp");
    let runtime_id = "epiphany-persona-smoke";
    let config = PersonaConfig {
        allowed_channel_name: "#aquarium".to_string(),
        allowed_channel_id: None,
        allowed_channel_id_env: Some("EPIPHANY_PERSONA_AQUARIUM_CHANNEL_ID_TEST".to_string()),
        persona_name: Some("Smoke Persona".to_string()),
        persona_avatar_url: Some("https://example.invalid/Persona.png".to_string()),
        bifrost_bridge_cli_path: None,
        bifrost_identity: Some("epiphany.Persona".to_string()),
        bifrost_source_kind: Some("epiphany_persona_speech".to_string()),
        bifrost_authority_ref: Some("epiphany.persona_speech_audit".to_string()),
        heimdall_capability_ref_env: Some("HEIMDALL_CAPABILITY_REF_TEST".to_string()),
    };
    let fake_bridge = temp_dir.join("fake-bifrost-bridge.mjs");
    fs::write(
        &fake_bridge,
        r#"
const args = process.argv.slice(2);
function opt(name) {
  const i = args.indexOf(name);
  return i >= 0 && i + 1 < args.length ? args[i + 1] : "";
}
console.log(JSON.stringify({
  action: args[0],
  messageId: "bridge-message-123",
  externalReceiptId: "bridge-message-123",
  url: `https://discord.com/channels/test/${opt("--channel-id")}/bridge-message-123`,
  provenance: {
    bifrostIdentity: opt("--identity"),
    sourceKind: opt("--source-kind"),
    sourceId: opt("--source-id"),
    authorityReference: opt("--authority-ref"),
    epiphanyRunId: opt("--epiphany-run-id"),
    epiphanyLaneId: opt("--epiphany-lane-id"),
    epiphanyAgentIdentity: opt("--epiphany-agent-identity"),
    heimdallCapabilityRef: opt("--heimdall-capability-ref")
  }
}));
"#,
    )
    .with_context(|| format!("failed to write {}", fake_bridge.display()))?;
    let mut bridge_config = config.clone();
    bridge_config.bifrost_bridge_cli_path = Some(fake_bridge);
    unsafe {
        env::remove_var("EPIPHANY_PERSONA_AQUARIUM_CHANNEL_ID_TEST");
        env::set_var("HEIMDALL_CAPABILITY_REF_TEST", "heimdall-capability-smoke");
        env::set_var("EPIPHANY_RUN_ID", "epiphany-run-smoke");
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
    let missing_bridge = run_post(
        "Persona should not cross Discord without Bifrost bridge configuration.",
        &config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        Some("123".to_string()),
        None,
        None,
        None,
    )?;
    let bridged = run_post(
        "Persona crosses to Discord only through Bifrost.",
        &bridge_config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        Some("123".to_string()),
        Some("Smoke Persona".to_string()),
        Some("https://example.invalid/Persona.png".to_string()),
        None,
    )?;
    let wrong = run_post(
        "Persona should not post outside #aquarium.",
        &bridge_config,
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
        &bridge_config,
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
        && missing_bridge["ok"] == false
        && missing_bridge["blocked"] == "missing-bifrost-bridge"
        && bridged["ok"] == true
        && bridged["transport"] == "bifrost.discord-post"
        && bridged["bifrostBridgeReceipt"]["provenance"]["bifrostIdentity"] == "epiphany.Persona"
        && bridged["bifrostBridgeReceipt"]["provenance"]["heimdallCapabilityRef"]
            == "heimdall-capability-smoke"
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
        "missingBridge": missing_bridge,
        "bridged": bridged,
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
            persona_name: payload["persona_name"].as_str().map(str::to_string),
            persona_avatar_url: payload["persona_avatar_url"].as_str().map(str::to_string),
            bifrost_bridge_cli_path: payload["bifrost_bridge_cli_path"]
                .as_str()
                .map(PathBuf::from),
            bifrost_identity: payload["bifrost_identity"].as_str().map(str::to_string),
            bifrost_source_kind: payload["bifrost_source_kind"].as_str().map(str::to_string),
            bifrost_authority_ref: payload["bifrost_authority_ref"]
                .as_str()
                .map(str::to_string),
            heimdall_capability_ref_env: payload["heimdall_capability_ref_env"]
                .as_str()
                .map(str::to_string),
        });
    }
    Ok(PersonaConfig {
        allowed_channel_name: toml_string(&raw, "allowed_channel_name")
            .unwrap_or_else(|| "#aquarium".to_string()),
        allowed_channel_id: toml_string(&raw, "allowed_channel_id"),
        allowed_channel_id_env: toml_string(&raw, "allowed_channel_id_env"),
        persona_name: toml_string(&raw, "persona_name"),
        persona_avatar_url: toml_string(&raw, "persona_avatar_url"),
        bifrost_bridge_cli_path: toml_string(&raw, "bifrost_bridge_cli_path").map(PathBuf::from),
        bifrost_identity: toml_string(&raw, "bifrost_identity"),
        bifrost_source_kind: toml_string(&raw, "bifrost_source_kind"),
        bifrost_authority_ref: toml_string(&raw, "bifrost_authority_ref"),
        heimdall_capability_ref_env: toml_string(&raw, "heimdall_capability_ref_env"),
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

fn bifrost_bridge_cli_path(config: &PersonaConfig) -> Option<PathBuf> {
    config.bifrost_bridge_cli_path.clone().or_else(|| {
        env::var("EPIPHANY_PERSONA_BIFROST_BRIDGE_CLI")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    })
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
