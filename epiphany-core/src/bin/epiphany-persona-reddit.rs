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

const POST_SCHEMA_VERSION: &str = "epiphany.persona_reddit_post.v0";
const SPEECH_AUDIT_SCHEMA_VERSION: &str = "epiphany.persona_speech_audit.v0";

#[derive(Clone, Debug, Default)]
struct PersonaRedditConfig {
    subreddit: String,
    persona_name: Option<String>,
    persona_flair_id: Option<String>,
    persona_flair_text: Option<String>,
    bifrost_bridge_cli_path: Option<PathBuf>,
    bifrost_identity: Option<String>,
    bifrost_source_kind: Option<String>,
    bifrost_authority_ref: Option<String>,
    heimdall_capability_ref_env: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersonaSpeechAudit {
    #[serde(rename = "schema_version")]
    schema_version: String,
    audit_id: String,
    created_at: String,
    action_kind: String,
    decision: String,
    reasons: Vec<String>,
    content_fingerprint: String,
    opening_key: String,
    topic_key: String,
    requested_public_target: Option<String>,
    recent_window_count: usize,
    repeated_opening_count: usize,
    repeated_topic_count: usize,
    same_target_post_count: usize,
    audit_path: PathBuf,
}

#[derive(Clone, Debug)]
struct RecentPersonaSpeech {
    opening_key: String,
    topic_key: String,
    subreddit: Option<String>,
    posted: bool,
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return usage();
    };
    let mut config_path = PathBuf::from("state/persona-reddit.toml");
    let mut artifact_dir = PathBuf::from(".epiphany-persona");
    let mut cultmesh_store = PathBuf::from(".epiphany-run/cultmesh/persona-speech.ccmp");
    let mut runtime_id = "epiphany-local".to_string();
    let mut title: Option<String> = None;
    let mut content: Option<String> = None;
    let mut subreddit: Option<String> = None;
    let mut persona_name: Option<String> = None;
    let mut persona_flair_id: Option<String> = None;
    let mut persona_flair_text: Option<String> = None;
    let mut limit = 8_usize;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => config_path = next_path(&mut args, "--config")?,
            "--artifact-dir" => artifact_dir = next_path(&mut args, "--artifact-dir")?,
            "--cultmesh-store" => cultmesh_store = next_path(&mut args, "--cultmesh-store")?,
            "--runtime-id" => runtime_id = next_value(&mut args, "--runtime-id")?,
            "--title" => title = Some(next_value(&mut args, "--title")?),
            "--content" => content = Some(next_value(&mut args, "--content")?),
            "--subreddit" => subreddit = Some(next_value(&mut args, "--subreddit")?),
            "--persona-name" => persona_name = Some(next_value(&mut args, "--persona-name")?),
            "--persona-flair-id" => {
                persona_flair_id = Some(next_value(&mut args, "--persona-flair-id")?)
            }
            "--persona-flair-text" => {
                persona_flair_text = Some(next_value(&mut args, "--persona-flair-text")?)
            }
            "--limit" => limit = next_value(&mut args, "--limit")?.parse()?,
            _ => return Err(anyhow!("unknown argument {arg:?}")),
        }
    }

    let result = match command.as_str() {
        "draft" => {
            let config = load_config(&config_path)?;
            let title = read_required_text(title, "Persona Reddit title")?;
            let content = read_required_text(content, "Persona Reddit post")?;
            run_draft(
                &title,
                &content,
                &config,
                &artifact_dir,
                &cultmesh_store,
                &runtime_id,
                "draft",
                "drafted without posting",
            )?
        }
        "post" => {
            let config = load_config(&config_path)?;
            let title = read_required_text(title, "Persona Reddit title")?;
            let content = read_required_text(content, "Persona Reddit post")?;
            run_post(
                &title,
                &content,
                &config,
                &artifact_dir,
                &cultmesh_store,
                &runtime_id,
                subreddit,
                persona_name,
                persona_flair_id,
                persona_flair_text,
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
    title: &str,
    content: &str,
    config: &PersonaRedditConfig,
    artifact_dir: &Path,
    cultmesh_store: &Path,
    runtime_id: &str,
    status: &str,
    reason: &str,
) -> Result<Value> {
    ensure_post(title, content)?;
    let audit = audit_persona_speech(
        title,
        content,
        artifact_dir,
        cultmesh_store,
        runtime_id,
        Some(&config.subreddit),
        false,
    )?;
    let path = write_post_artifact(
        title,
        content,
        config,
        artifact_dir,
        status,
        reason,
        Some(&audit),
        None,
    )?;
    Ok(serde_json::json!({
        "ok": status != "blocked",
        "posted": false,
        "draftPath": path,
        "speechAudit": audit,
    }))
}

fn run_post(
    title: &str,
    content: &str,
    config: &PersonaRedditConfig,
    artifact_dir: &Path,
    cultmesh_store: &Path,
    runtime_id: &str,
    subreddit: Option<String>,
    persona_name: Option<String>,
    persona_flair_id: Option<String>,
    persona_flair_text: Option<String>,
) -> Result<Value> {
    ensure_post(title, content)?;
    let target_subreddit = clean_subreddit(subreddit.unwrap_or_else(|| config.subreddit.clone()));
    let audit = audit_persona_speech(
        title,
        content,
        artifact_dir,
        cultmesh_store,
        runtime_id,
        Some(&target_subreddit),
        true,
    )?;
    if audit.decision == "blocked" {
        let path = write_post_artifact(
            title,
            content,
            config,
            artifact_dir,
            "blocked",
            &format!("speech audit blocked: {}", audit.reasons.join("; ")),
            Some(&audit),
            None,
        )?;
        return Ok(serde_json::json!({
            "ok": false,
            "posted": false,
            "blocked": "speech-audit",
            "draftPath": path,
            "speechAudit": audit,
        }));
    }
    if target_subreddit.is_empty() {
        let path = write_post_artifact(
            title,
            content,
            config,
            artifact_dir,
            "blocked",
            "missing Bifrost-owned subreddit",
            Some(&audit),
            None,
        )?;
        return Ok(serde_json::json!({
            "ok": false,
            "posted": false,
            "blocked": "missing-subreddit",
            "draftPath": path,
            "speechAudit": audit,
        }));
    }
    let Some(bridge_cli_path) = bifrost_bridge_cli_path(config) else {
        let path = write_post_artifact(
            title,
            content,
            config,
            artifact_dir,
            "blocked",
            "missing Bifrost bridge CLI path",
            Some(&audit),
            None,
        )?;
        return Ok(serde_json::json!({
            "ok": false,
            "posted": false,
            "blocked": "missing-bifrost-bridge",
            "draftPath": path,
            "speechAudit": audit,
        }));
    };
    let receipt = post_bifrost_reddit_thread(
        &bridge_cli_path,
        config,
        &target_subreddit,
        title,
        content,
        persona_name.as_deref(),
        persona_flair_id.as_deref(),
        persona_flair_text.as_deref(),
        &audit,
    )?;
    let path = write_post_artifact(
        title,
        content,
        config,
        artifact_dir,
        "posted",
        "posted through Bifrost reddit-post",
        Some(&audit),
        Some(&receipt),
    )?;
    Ok(serde_json::json!({
        "ok": true,
        "posted": true,
        "transport": "bifrost.reddit-post",
        "subreddit": target_subreddit,
        "thingId": receipt["thingId"].as_str().or_else(|| receipt["externalReceiptId"].as_str()),
        "url": receipt["url"].as_str(),
        "bifrostBridgeReceipt": receipt,
        "draftPath": path,
        "speechAudit": audit,
    }))
}

fn post_bifrost_reddit_thread(
    bridge_cli_path: &Path,
    config: &PersonaRedditConfig,
    subreddit: &str,
    title: &str,
    content: &str,
    persona_name: Option<&str>,
    persona_flair_id: Option<&str>,
    persona_flair_text: Option<&str>,
    audit: &PersonaSpeechAudit,
) -> Result<Value> {
    let persona_name = persona_name
        .or(config.persona_name.as_deref())
        .unwrap_or("Epiphany Persona");
    let persona_flair_id = persona_flair_id.or(config.persona_flair_id.as_deref());
    let persona_flair_text = persona_flair_text
        .or(config.persona_flair_text.as_deref())
        .unwrap_or(persona_name);

    let mut command = Command::new(bifrost_node_executable());
    command.arg(bridge_cli_path);
    command.arg("reddit-post");
    command.arg("--subreddit").arg(subreddit);
    command.arg("--title").arg(title);
    command.arg("--content").arg(content);
    command.arg("--persona-name").arg(persona_name);
    command.arg("--persona-flair-text").arg(persona_flair_text);
    if let Some(flair_id) = persona_flair_id {
        command.arg("--persona-flair-id").arg(flair_id);
    }
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
            .unwrap_or("epiphany_persona_reddit"),
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
            "Bifrost bridge Reddit post failed with status {}: {}{}{}",
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
    serde_json::from_str(strip_bom(&stdout)).context("Bifrost bridge stdout was not JSON")
}

fn audit_persona_speech(
    title: &str,
    content: &str,
    artifact_dir: &Path,
    cultmesh_store: &Path,
    runtime_id: &str,
    subreddit: Option<&str>,
    public_post: bool,
) -> Result<PersonaSpeechAudit> {
    let combined = format!("{title}\n\n{content}");
    let opening_key = opening_key(title);
    let topic_key = topic_key(&combined);
    let recent = recent_persona_speech(artifact_dir, 12);
    let repeated_opening_count = recent
        .iter()
        .filter(|speech| !opening_key.is_empty() && speech.opening_key == opening_key)
        .count();
    let repeated_topic_count = recent
        .iter()
        .filter(|speech| !topic_key.is_empty() && speech.topic_key == topic_key)
        .count();
    let same_target_post_count = subreddit
        .map(|subreddit| {
            recent
                .iter()
                .filter(|speech| speech.posted && speech.subreddit.as_deref() == Some(subreddit))
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
    if public_post && same_target_post_count >= 3 {
        reasons.push("subreddit-saturation".to_string());
    }
    if title.trim().len() > 280 {
        reasons.push("title-too-long-for-reddit".to_string());
    }
    if content.trim().len() > 9000 {
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
        "Persona-reddit-speech-audit-{}-{}.json",
        now_stamp(),
        audit_id
    ));
    let audit = PersonaSpeechAudit {
        schema_version: SPEECH_AUDIT_SCHEMA_VERSION.to_string(),
        audit_id,
        created_at: now_iso(),
        action_kind: "post".to_string(),
        decision,
        reasons,
        content_fingerprint: content_fingerprint(&combined),
        opening_key,
        topic_key,
        requested_public_target: subreddit.map(str::to_string),
        recent_window_count: recent.len(),
        repeated_opening_count,
        repeated_topic_count,
        same_target_post_count,
        audit_path,
    };
    write_json(&audit.audit_path, &serde_json::to_value(&audit)?)?;
    write_epiphany_cultmesh_persona_speech_audit(
        cultmesh_store,
        EpiphanyCultMeshPersonaSpeechAuditEntry {
            schema_version: EPIPHANY_CULTMESH_PERSONA_SPEECH_AUDIT_SCHEMA_VERSION.to_string(),
            audit_id: audit.audit_id.clone(),
            runtime_id: runtime_id.to_string(),
            verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
            persona_agent_id: "epiphany.Persona".to_string(),
            action_kind: "post".to_string(),
            decision: audit.decision.clone(),
            content_fingerprint: audit.content_fingerprint.clone(),
            opening_key: audit.opening_key.clone(),
            topic_key: audit.topic_key.clone(),
            requested_channel_id: audit.requested_public_target.clone().unwrap_or_default(),
            recent_window_count: audit.recent_window_count as u32,
            repeated_opening_count: audit.repeated_opening_count as u32,
            repeated_topic_count: audit.repeated_topic_count as u32,
            same_channel_post_count: audit.same_target_post_count as u32,
            reasons: audit.reasons.clone(),
            artifact_ref: audit.audit_path.display().to_string(),
            created_at_utc: audit.created_at.clone(),
            private_state_exposed: false,
            notes: vec![
                "Persona Reddit speech audit is parent-side mouth policy; it stores fingerprints and counters, not raw private thought.".to_string(),
                "Reddit crossing belongs to Bifrost; Epiphany records eligibility before bridge transport.".to_string(),
            ],
        },
    )?;
    Ok(audit)
}

fn run_smoke() -> Result<Value> {
    let temp_dir = scoped_temp_dir("epiphany-persona-reddit-smoke")?;
    let cultmesh_store = temp_dir.join("persona-speech.ccmp");
    let runtime_id = "epiphany-persona-reddit-smoke";
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
  ok: true,
  subreddit: opt("--subreddit"),
  title: opt("--title"),
  personaName: opt("--persona-name"),
  personaFlairText: opt("--persona-flair-text"),
  thingId: "t3_epiphanysmoke",
  url: `https://reddit.com/r/${opt("--subreddit")}/comments/t3_epiphanysmoke/epiphany_smoke/`,
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
    let config = PersonaRedditConfig {
        subreddit: "GameCultOrg".to_string(),
        persona_name: Some("Smoke Persona".to_string()),
        persona_flair_id: None,
        persona_flair_text: Some("Smoke Persona".to_string()),
        bifrost_bridge_cli_path: Some(fake_bridge),
        bifrost_identity: Some("epiphany.Persona".to_string()),
        bifrost_source_kind: Some("epiphany_persona_reddit".to_string()),
        bifrost_authority_ref: Some("epiphany.persona_speech_audit".to_string()),
        heimdall_capability_ref_env: Some("HEIMDALL_CAPABILITY_REF_TEST".to_string()),
    };
    let no_bridge_config = PersonaRedditConfig {
        bifrost_bridge_cli_path: None,
        ..config.clone()
    };
    unsafe {
        env::set_var("HEIMDALL_CAPABILITY_REF_TEST", "heimdall-reddit-smoke");
        env::set_var("EPIPHANY_RUN_ID", "epiphany-run-reddit-smoke");
    }
    let draft = run_draft(
        "Epiphany smoke post",
        "Persona drafts a Reddit organizing post without crossing the bridge.",
        &config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        "draft",
        "drafted without posting",
    )?;
    let missing_bridge = run_post(
        "Epiphany missing bridge",
        "Persona should not reach Reddit without Bifrost configured.",
        &no_bridge_config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        None,
        None,
        None,
        None,
    )?;
    let bridged = run_post(
        "Epiphany crosses through Bifrost",
        "Persona posts to Reddit only through the Bifrost bridge.",
        &config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        None,
        None,
        None,
        None,
    )?;
    let seed = audit_persona_speech(
        "Repeated Reddit rite",
        "Modeling and Soul keep circling the same public proof seam.",
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        Some("GameCultOrg"),
        false,
    )?;
    write_post_artifact(
        "Repeated Reddit rite",
        "Modeling and Soul keep circling the same public proof seam.",
        &config,
        &temp_dir,
        "posted",
        "seed prior Reddit post for speech-audit smoke",
        Some(&seed),
        None,
    )?;
    write_post_artifact(
        "Repeated Reddit rite",
        "Modeling and Soul keep circling the same public proof seam.",
        &config,
        &temp_dir,
        "posted",
        "seed second prior Reddit post for speech-audit smoke",
        Some(&seed),
        None,
    )?;
    let repeated = run_post(
        "Repeated Reddit rite",
        "Modeling and Soul keep circling the same public proof seam again.",
        &config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        None,
        None,
        None,
        None,
    )?;
    let latest_cultmesh_audit =
        load_latest_epiphany_cultmesh_persona_speech_audit(&cultmesh_store, runtime_id)?
            .context("Persona Reddit smoke expected latest CultMesh audit")?;
    let ok = draft["ok"] == true
        && missing_bridge["ok"] == false
        && missing_bridge["blocked"] == "missing-bifrost-bridge"
        && bridged["ok"] == true
        && bridged["transport"] == "bifrost.reddit-post"
        && bridged["bifrostBridgeReceipt"]["provenance"]["bifrostIdentity"] == "epiphany.Persona"
        && bridged["bifrostBridgeReceipt"]["provenance"]["heimdallCapabilityRef"]
            == "heimdall-reddit-smoke"
        && repeated["ok"] == false
        && repeated["blocked"] == "speech-audit"
        && repeated["speechAudit"]["reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| reason == "repeated-opening"))
        && latest_cultmesh_audit.decision == "blocked"
        && !latest_cultmesh_audit.private_state_exposed;
    let result = serde_json::json!({
        "ok": ok,
        "draft": draft,
        "missingBridge": missing_bridge,
        "bridged": bridged,
        "repeatedSpeech": repeated,
        "latestCultMeshSpeechAudit": latest_cultmesh_audit,
    });
    let _ = fs::remove_dir_all(&temp_dir);
    Ok(result)
}

fn write_post_artifact(
    title: &str,
    content: &str,
    config: &PersonaRedditConfig,
    artifact_dir: &Path,
    status: &str,
    reason: &str,
    speech_audit: Option<&PersonaSpeechAudit>,
    bridge_receipt: Option<&Value>,
) -> Result<PathBuf> {
    let payload = serde_json::json!({
        "schema_version": POST_SCHEMA_VERSION,
        "created_at": now_iso(),
        "status": status,
        "reason": reason,
        "subreddit": config.subreddit,
        "persona_name": config.persona_name,
        "persona_flair_id": config.persona_flair_id,
        "persona_flair_text": config.persona_flair_text,
        "title": title.trim(),
        "content": content.trim(),
        "speechAudit": speech_audit,
        "bifrostBridgeReceipt": bridge_receipt,
    });
    let path = artifact_dir.join(format!(
        "Persona-reddit-{}-{}.json",
        now_stamp(),
        short_id()
    ));
    write_json(&path, &payload)?;
    Ok(path)
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
                .is_some_and(|name| name.starts_with("Persona-reddit-") && name.ends_with(".json"))
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
                "title": payload.get("title"),
                "subreddit": payload.get("subreddit"),
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
                .is_some_and(|name| name.starts_with("Persona-reddit-") && name.ends_with(".json"))
        })
        .filter_map(|path| Some((path.metadata().ok()?.modified().ok()?, path)))
        .collect::<Vec<_>>();
    paths.sort_by_key(|item| Reverse(item.0));
    paths
        .into_iter()
        .take(limit)
        .filter_map(|(_, path)| {
            let payload: Value = serde_json::from_str(&fs::read_to_string(&path).ok()?).ok()?;
            let title = payload["title"].as_str()?.to_string();
            let content = payload["content"].as_str().unwrap_or_default().to_string();
            Some(RecentPersonaSpeech {
                opening_key: opening_key(&title),
                topic_key: topic_key(&format!("{title}\n\n{content}")),
                subreddit: payload["subreddit"].as_str().map(str::to_string),
                posted: payload["status"].as_str() == Some("posted"),
            })
        })
        .collect()
}

fn ensure_post(title: &str, content: &str) -> Result<()> {
    if title.trim().is_empty() {
        return Err(anyhow!("Persona Reddit title is empty"));
    }
    if content.trim().is_empty() {
        return Err(anyhow!("Persona Reddit content is empty"));
    }
    Ok(())
}

fn read_required_text(value: Option<String>, label: &str) -> Result<String> {
    let Some(value) = value else {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .context("failed to read stdin")?;
        ensure_nonempty(&text, label)?;
        return Ok(text);
    };
    let path = PathBuf::from(&value);
    let text = if path.exists() {
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?
    } else {
        value
    };
    ensure_nonempty(&text, label)?;
    Ok(text)
}

fn ensure_nonempty(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{label} is empty"));
    }
    Ok(())
}

fn load_config(path: &Path) -> Result<PersonaRedditConfig> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    if path
        .extension()
        .is_some_and(|extension| extension == "json")
    {
        let payload: Value = serde_json::from_str(&raw)
            .with_context(|| format!("failed to decode {}", path.display()))?;
        return Ok(PersonaRedditConfig {
            subreddit: payload["subreddit"]
                .as_str()
                .map(clean_subreddit)
                .unwrap_or_else(|| "GameCultOrg".to_string()),
            persona_name: payload["persona_name"].as_str().map(str::to_string),
            persona_flair_id: payload["persona_flair_id"].as_str().map(str::to_string),
            persona_flair_text: payload["persona_flair_text"].as_str().map(str::to_string),
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
    Ok(PersonaRedditConfig {
        subreddit: toml_string(&raw, "subreddit")
            .map(|value| clean_subreddit(value))
            .unwrap_or_else(|| "GameCultOrg".to_string()),
        persona_name: toml_string(&raw, "persona_name"),
        persona_flair_id: toml_string(&raw, "persona_flair_id"),
        persona_flair_text: toml_string(&raw, "persona_flair_text"),
        bifrost_bridge_cli_path: toml_string(&raw, "bifrost_bridge_cli_path").map(PathBuf::from),
        bifrost_identity: toml_string(&raw, "bifrost_identity"),
        bifrost_source_kind: toml_string(&raw, "bifrost_source_kind"),
        bifrost_authority_ref: toml_string(&raw, "bifrost_authority_ref"),
        heimdall_capability_ref_env: toml_string(&raw, "heimdall_capability_ref_env"),
    })
}

fn bifrost_bridge_cli_path(config: &PersonaRedditConfig) -> Option<PathBuf> {
    config.bifrost_bridge_cli_path.clone().or_else(|| {
        env::var("EPIPHANY_PERSONA_BIFROST_BRIDGE_CLI")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    })
}

fn bifrost_node_executable() -> String {
    env::var("EPIPHANY_BIFROST_NODE")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "node".to_string())
}

fn clean_subreddit(value: impl AsRef<str>) -> String {
    value
        .as_ref()
        .trim()
        .trim_start_matches("r/")
        .trim_matches('/')
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '-')
        .collect()
}

fn env_value(name: &String) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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

fn opening_key(content: &str) -> String {
    content
        .split_whitespace()
        .take(6)
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
    tokens.into_iter().take(8).collect::<Vec<_>>().join("|")
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

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))
        .with_context(|| format!("failed to write {}", path.display()))
}

fn strip_bom(value: &str) -> &str {
    value.strip_prefix('\u{feff}').unwrap_or(value)
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
        "usage: epiphany-persona-reddit <draft|post|latest|smoke> [--config <path>] [--artifact-dir <path>] [--title <text>] [--content <text-or-path>]"
    ))
}
