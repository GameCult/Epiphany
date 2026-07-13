use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::SecondsFormat;
use epiphany_core::EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_PERSONA_SPEECH_AUDIT_SCHEMA_VERSION;
use epiphany_core::EpiphanyCultMeshPersonaSpeechAuditEntry;
use epiphany_core::load_latest_epiphany_cultmesh_persona_speech_audit;
use epiphany_core::write_epiphany_cultmesh_persona_speech_audit;
use serde_json::Value;
use std::cmp::Reverse;
use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

const SPEECH_AUDIT_SCHEMA_VERSION: &str = "epiphany.persona_speech_audit.v0";

#[derive(Clone, Debug, Default)]
struct PersonaOtherConfig {
    surface_name: String,
    target_locator: String,
    persona_name: Option<String>,
    bifrost_bridge_cli_path: Option<PathBuf>,
    bifrost_identity: Option<String>,
    bifrost_source_kind: Option<String>,
    bifrost_authority_ref: Option<String>,
    heimdall_capability_ref_env: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PersonaSpeechAudit {
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
    #[serde(rename = "sameTargetPostCount")]
    same_target_crossing_count: usize,
    audit_path: PathBuf,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
enum PersonaOtherLifecycle {
    DraftCandidate,
    Blocked,
    CrossingRequested,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BifrostOtherRequestProvenance {
    bifrost_identity: String,
    source_kind: String,
    source_id: String,
    authority_reference: String,
    epiphany_run_id: String,
    epiphany_lane_id: String,
    epiphany_agent_identity: String,
    heimdall_capability_ref: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BifrostOtherRequestReceipt {
    action: String,
    ok: bool,
    surface_name: String,
    target_locator: String,
    title: String,
    content: String,
    bridge_action_id: String,
    receipt_url: String,
    provenance: BifrostOtherRequestProvenance,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PersonaOtherRequestArtifact {
    schema_version: String,
    lifecycle: PersonaOtherLifecycle,
    reason: String,
    created_at: String,
    surface_name: String,
    target_locator: String,
    title: String,
    persona_name: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    speech_audit: Option<PersonaSpeechAudit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bifrost_request_receipt: Option<BifrostOtherRequestReceipt>,
    private_state_exposed: bool,
}

#[derive(Clone, Debug)]
struct RecentPersonaSpeech {
    opening_key: String,
    topic_key: String,
    target: Option<String>,
    crossing_recorded: bool,
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return usage();
    };
    let mut config_path = PathBuf::from("state/persona-other.toml");
    let mut artifact_dir = PathBuf::from(".epiphany-persona");
    let mut cultmesh_store = PathBuf::from(".epiphany-run/cultmesh/persona-speech.ccmp");
    let mut runtime_id = "epiphany-local".to_string();
    let mut surface_name: Option<String> = None;
    let mut target_locator: Option<String> = None;
    let mut title: Option<String> = None;
    let mut content: Option<String> = None;
    let mut persona_name: Option<String> = None;
    let mut limit = 8_usize;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => config_path = next_path(&mut args, "--config")?,
            "--artifact-dir" => artifact_dir = next_path(&mut args, "--artifact-dir")?,
            "--cultmesh-store" => cultmesh_store = next_path(&mut args, "--cultmesh-store")?,
            "--runtime-id" => runtime_id = next_value(&mut args, "--runtime-id")?,
            "--surface-name" => surface_name = Some(next_value(&mut args, "--surface-name")?),
            "--target-locator" => target_locator = Some(next_value(&mut args, "--target-locator")?),
            "--title" => title = Some(next_value(&mut args, "--title")?),
            "--content" => content = Some(next_value(&mut args, "--content")?),
            "--persona-name" => persona_name = Some(next_value(&mut args, "--persona-name")?),
            "--limit" => limit = next_value(&mut args, "--limit")?.parse()?,
            _ => return Err(anyhow!("unknown argument {arg:?}")),
        }
    }

    let result = match command.as_str() {
        "draft" => {
            let config = load_config(&config_path)?;
            let content = read_required_text(content, "Persona outside-world request")?;
            run_draft(
                &content,
                &config,
                &artifact_dir,
                &cultmesh_store,
                &runtime_id,
                surface_name,
                target_locator,
                title,
                persona_name,
            )?
        }
        "request" => {
            let config = load_config(&config_path)?;
            let content = read_required_text(content, "Persona outside-world request")?;
            run_request(
                &content,
                &config,
                &artifact_dir,
                &cultmesh_store,
                &runtime_id,
                surface_name,
                target_locator,
                title,
                persona_name,
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

#[allow(clippy::too_many_arguments)]
fn run_draft(
    content: &str,
    config: &PersonaOtherConfig,
    artifact_dir: &Path,
    cultmesh_store: &Path,
    runtime_id: &str,
    surface_name: Option<String>,
    target_locator: Option<String>,
    title: Option<String>,
    persona_name: Option<String>,
) -> Result<Value> {
    let target = resolve_target(config, surface_name, target_locator)?;
    let audit = audit_persona_speech(
        content,
        artifact_dir,
        cultmesh_store,
        runtime_id,
        &target.surface_name,
        &target.target_locator,
        false,
    )?;
    let path = write_request_artifact(
        content,
        config,
        &target,
        artifact_dir,
        PersonaOtherLifecycle::DraftCandidate,
        "drafted without crossing Bifrost",
        Some(&audit),
        None,
        title.as_deref(),
        persona_name.as_deref(),
    )?;
    Ok(serde_json::json!({
        "ok": true,
        "requested": false,
        "draftPath": path,
        "surfaceName": target.surface_name,
        "targetLocator": target.target_locator,
        "speechAudit": audit,
    }))
}

#[allow(clippy::too_many_arguments)]
fn run_request(
    content: &str,
    config: &PersonaOtherConfig,
    artifact_dir: &Path,
    cultmesh_store: &Path,
    runtime_id: &str,
    surface_name: Option<String>,
    target_locator: Option<String>,
    title: Option<String>,
    persona_name: Option<String>,
) -> Result<Value> {
    let target = resolve_target(config, surface_name, target_locator)?;
    let audit = audit_persona_speech(
        content,
        artifact_dir,
        cultmesh_store,
        runtime_id,
        &target.surface_name,
        &target.target_locator,
        true,
    )?;
    if audit.decision == "blocked" {
        let path = write_request_artifact(
            content,
            config,
            &target,
            artifact_dir,
            PersonaOtherLifecycle::Blocked,
            &format!("speech audit blocked: {}", audit.reasons.join("; ")),
            Some(&audit),
            None,
            title.as_deref(),
            persona_name.as_deref(),
        )?;
        return Ok(serde_json::json!({
            "ok": false,
            "blocked": "speech-audit",
            "artifactPath": path,
            "speechAudit": audit,
        }));
    }
    let Some(bridge_cli) = bifrost_bridge_cli_path(config) else {
        let path = write_request_artifact(
            content,
            config,
            &target,
            artifact_dir,
            PersonaOtherLifecycle::Blocked,
            "missing Bifrost bridge CLI for outside-world Persona request",
            Some(&audit),
            None,
            title.as_deref(),
            persona_name.as_deref(),
        )?;
        return Ok(serde_json::json!({
            "ok": false,
            "blocked": "missing-bifrost-bridge",
            "artifactPath": path,
            "speechAudit": audit,
        }));
    };
    let Some(bifrost_identity) = required_bifrost_identity(config) else {
        let path = write_request_artifact(
            content,
            config,
            &target,
            artifact_dir,
            PersonaOtherLifecycle::Blocked,
            "missing Bifrost identity for outside-world Persona request",
            Some(&audit),
            None,
            title.as_deref(),
            persona_name.as_deref(),
        )?;
        return Ok(serde_json::json!({
            "ok": false,
            "blocked": "missing-bifrost-identity",
            "artifactPath": path,
            "speechAudit": audit,
        }));
    };
    let Some(heimdall_capability_ref) = required_heimdall_capability_ref(config) else {
        let path = write_request_artifact(
            content,
            config,
            &target,
            artifact_dir,
            PersonaOtherLifecycle::Blocked,
            "missing Heimdall-backed capability/account reference for outside-world Persona request",
            Some(&audit),
            None,
            title.as_deref(),
            persona_name.as_deref(),
        )?;
        return Ok(serde_json::json!({
            "ok": false,
            "blocked": "missing-heimdall-capability-ref",
            "artifactPath": path,
            "speechAudit": audit,
        }));
    };
    if !is_heimdall_surface_reference(&heimdall_capability_ref, &target.surface_name) {
        let path = write_request_artifact(
            content,
            config,
            &target,
            artifact_dir,
            PersonaOtherLifecycle::Blocked,
            &format!(
                "Heimdall reference must be shaped for {} public Persona crossings: heimdall:{}:*",
                target.surface_name, target.surface_name
            ),
            Some(&audit),
            None,
            title.as_deref(),
            persona_name.as_deref(),
        )?;
        return Ok(serde_json::json!({
            "ok": false,
            "blocked": "wrong-heimdall-capability-surface",
            "artifactPath": path,
            "speechAudit": audit,
        }));
    }

    let receipt = request_bifrost_other(
        &bridge_cli,
        config,
        &bifrost_identity,
        &heimdall_capability_ref,
        &target,
        content,
        title.as_deref(),
        persona_name.as_deref(),
        &audit,
    )?;
    let path = write_request_artifact(
        content,
        config,
        &target,
        artifact_dir,
        PersonaOtherLifecycle::CrossingRequested,
        "recorded through Bifrost other-request",
        Some(&audit),
        Some(&receipt),
        title.as_deref(),
        persona_name.as_deref(),
    )?;
    Ok(serde_json::json!({
        "ok": true,
        "requested": true,
        "transport": "bifrost.other-request",
        "surfaceName": target.surface_name,
        "targetLocator": target.target_locator,
        "artifactPath": path,
        "bifrostBridgeReceipt": receipt,
        "speechAudit": audit,
    }))
}

fn request_bifrost_other(
    bridge_cli_path: &Path,
    config: &PersonaOtherConfig,
    bifrost_identity: &str,
    heimdall_capability_ref: &str,
    target: &ResolvedTarget,
    content: &str,
    title: Option<&str>,
    persona_name: Option<&str>,
    audit: &PersonaSpeechAudit,
) -> Result<BifrostOtherRequestReceipt> {
    let title = title
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("Persona request for {}", target.surface_name));
    let persona_name = persona_name
        .or(config.persona_name.as_deref())
        .unwrap_or("Epiphany Persona");
    let mut command = Command::new(bifrost_node_executable());
    command.arg(bridge_cli_path);
    command.arg("other-request");
    command.arg("--surface-name").arg(&target.surface_name);
    command.arg("--target-locator").arg(&target.target_locator);
    command.arg("--title").arg(title);
    command.arg("--content").arg(content);
    command.arg("--identity").arg(bifrost_identity);
    command.arg("--source-kind").arg(
        config
            .bifrost_source_kind
            .as_deref()
            .unwrap_or("epiphany_persona_other"),
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
    command
        .arg("--heimdall-capability-ref")
        .arg(heimdall_capability_ref);
    if let Some(run_id) = env::var("EPIPHANY_RUN_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        command.arg("--epiphany-run-id").arg(run_id);
    }
    command.arg("--persona-name").arg(persona_name);

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
            "Bifrost bridge other-request failed with status {}: {}{}{}",
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
    let receipt: BifrostOtherRequestReceipt = serde_json::from_str(strip_bom(&stdout))
        .context("Bifrost bridge stdout was not a typed other-request receipt")?;
    if !receipt.ok
        || receipt.action != "other-request"
        || receipt.surface_name != target.surface_name
        || receipt.target_locator != target.target_locator
        || receipt.provenance.bifrost_identity != bifrost_identity
        || receipt.provenance.source_id != audit.audit_id
        || receipt.provenance.heimdall_capability_ref != heimdall_capability_ref
        || receipt.provenance.epiphany_lane_id != "Persona"
        || receipt.provenance.epiphany_agent_identity != "epiphany.Persona"
        || receipt.bridge_action_id.trim().is_empty()
        || receipt.receipt_url.trim().is_empty()
    {
        return Err(anyhow!(
            "Bifrost other-request receipt did not bind the requested crossing"
        ));
    }
    Ok(receipt)
}

fn audit_persona_speech(
    content: &str,
    artifact_dir: &Path,
    cultmesh_store: &Path,
    runtime_id: &str,
    surface_name: &str,
    target_locator: &str,
    public_request: bool,
) -> Result<PersonaSpeechAudit> {
    let opening_key = opening_key(content);
    let topic_key = topic_key(content);
    let target_key = format!("{surface_name}:{target_locator}");
    let recent = recent_persona_speech(artifact_dir, 12);
    let repeated_opening_count = recent
        .iter()
        .filter(|speech| !opening_key.is_empty() && speech.opening_key == opening_key)
        .count();
    let repeated_topic_count = recent
        .iter()
        .filter(|speech| !topic_key.is_empty() && speech.topic_key == topic_key)
        .count();
    let same_target_crossing_count = recent
        .iter()
        .filter(|speech| speech.crossing_recorded && speech.target.as_deref() == Some(&target_key))
        .count();
    let mut reasons = Vec::new();
    if public_request && repeated_opening_count >= 2 {
        reasons.push("repeated-opening".to_string());
    }
    if public_request && repeated_topic_count >= 3 {
        reasons.push("repeated-topic".to_string());
    }
    if public_request && same_target_crossing_count >= 3 {
        reasons.push("target-saturation".to_string());
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
        "Persona-other-speech-audit-{}-{}.json",
        now_stamp(),
        audit_id
    ));
    let action_kind = if public_request { "post" } else { "draft" }.to_string();
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
        requested_public_target: Some(target_key.clone()),
        recent_window_count: recent.len(),
        repeated_opening_count,
        repeated_topic_count,
        same_target_crossing_count,
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
            action_kind: audit.action_kind.clone(),
            decision: audit.decision.clone(),
            content_fingerprint: audit.content_fingerprint.clone(),
            opening_key: audit.opening_key.clone(),
            topic_key: audit.topic_key.clone(),
            requested_channel_id: target_key,
            recent_window_count: audit.recent_window_count as u32,
            repeated_opening_count: audit.repeated_opening_count as u32,
            repeated_topic_count: audit.repeated_topic_count as u32,
            same_channel_post_count: audit.same_target_crossing_count as u32,
            reasons: audit.reasons.clone(),
            artifact_ref: audit.audit_path.display().to_string(),
            created_at_utc: audit.created_at.clone(),
            private_state_exposed: false,
            notes: vec![
                "Persona future-surface speech audit is parent-side mouth policy; it stores fingerprints and counters, not raw private thought.".to_string(),
                "Unknown outside-world crossings belong to Bifrost other-request until the surface earns a named actuator.".to_string(),
            ],
        },
    )?;
    Ok(audit)
}

fn run_smoke() -> Result<Value> {
    let temp_dir = scoped_temp_dir("epiphany-persona-other-smoke")?;
    let cultmesh_store = temp_dir.join("persona-speech.ccmp");
    let runtime_id = "epiphany-persona-other-smoke";
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
  surfaceName: opt("--surface-name"),
  targetLocator: opt("--target-locator"),
  title: opt("--title"),
  content: opt("--content"),
  bridgeActionId: "bridge-action-other-smoke",
  receiptUrl: "bifrost://bridge/actions/bridge-action-other-smoke",
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
    let config = PersonaOtherConfig {
        surface_name: "bluesky".to_string(),
        target_locator: "at://did:example/app.bsky.feed.post/root".to_string(),
        persona_name: Some("Smoke Persona".to_string()),
        bifrost_bridge_cli_path: Some(fake_bridge),
        bifrost_identity: Some("epiphany.Persona".to_string()),
        bifrost_source_kind: Some("epiphany_persona_other".to_string()),
        bifrost_authority_ref: Some("epiphany.persona_speech_audit".to_string()),
        heimdall_capability_ref_env: Some("HEIMDALL_CAPABILITY_REF_TEST".to_string()),
    };
    let no_bridge_config = PersonaOtherConfig {
        bifrost_bridge_cli_path: None,
        ..config.clone()
    };
    let missing_capability_config = PersonaOtherConfig {
        heimdall_capability_ref_env: Some("HEIMDALL_CAPABILITY_REF_MISSING_FOR_SMOKE".to_string()),
        ..config.clone()
    };
    let wrong_surface_config = PersonaOtherConfig {
        heimdall_capability_ref_env: Some(
            "HEIMDALL_CAPABILITY_REF_WRONG_SURFACE_FOR_SMOKE".to_string(),
        ),
        ..config.clone()
    };
    unsafe {
        env::set_var(
            "HEIMDALL_CAPABILITY_REF_TEST",
            "heimdall:bluesky:capability:smoke-persona",
        );
        env::set_var(
            "HEIMDALL_CAPABILITY_REF_WRONG_SURFACE_FOR_SMOKE",
            "heimdall:discord:capability:smoke-persona",
        );
        env::remove_var("HEIMDALL_CAPABILITY_REF_MISSING_FOR_SMOKE");
        env::set_var("EPIPHANY_RUN_ID", "epiphany-run-other-smoke");
    }

    let draft = run_draft(
        "Persona drafts a future-surface request without crossing Bifrost.",
        &config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        None,
        None,
        Some("Draft future surface".to_string()),
        None,
    )?;
    let missing_bridge = run_request(
        "Persona should not reach a future surface without Bifrost configured.",
        &no_bridge_config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        None,
        None,
        None,
        None,
    )?;
    let missing_capability = run_request(
        "Persona should not reach a future surface without Heimdall-backed capability proof.",
        &missing_capability_config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        None,
        None,
        None,
        None,
    )?;
    let wrong_surface_capability = run_request(
        "Persona should not reach Bluesky with Discord Heimdall-backed capability proof.",
        &wrong_surface_config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        None,
        None,
        None,
        None,
    )?;
    let bridged = run_request(
        "Persona requests a future-surface crossing only through Bifrost.",
        &config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
        None,
        None,
        Some("Smoke future surface".to_string()),
        None,
    )?;
    seed_requested_artifact(
        "Repeated future surface rite",
        &config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
    )?;
    seed_requested_artifact(
        "Repeated future surface rite",
        &config,
        &temp_dir,
        &cultmesh_store,
        runtime_id,
    )?;
    let repeated = run_request(
        "Repeated future surface rite again.",
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
            .context("Persona other smoke expected latest CultMesh audit")?;
    let draft_artifact = read_artifact(&draft["draftPath"])?;
    let bridged_artifact = read_artifact(&bridged["artifactPath"])?;
    let blocked_artifact = read_artifact(&missing_bridge["artifactPath"])?;
    let base_keys = [
        "schemaVersion",
        "lifecycle",
        "reason",
        "createdAt",
        "surfaceName",
        "targetLocator",
        "title",
        "personaName",
        "content",
        "speechAudit",
        "privateStateExposed",
    ];
    let requested_keys = [
        "schemaVersion",
        "lifecycle",
        "reason",
        "createdAt",
        "surfaceName",
        "targetLocator",
        "title",
        "personaName",
        "content",
        "speechAudit",
        "bifrostRequestReceipt",
        "privateStateExposed",
    ];
    let ok = draft["ok"] == true
        && exact_root_keys(&draft_artifact, &base_keys)
        && draft_artifact["lifecycle"] == "draft-candidate"
        && exact_root_keys(&blocked_artifact, &base_keys)
        && blocked_artifact["lifecycle"] == "blocked"
        && exact_root_keys(&bridged_artifact, &requested_keys)
        && bridged_artifact["lifecycle"] == "crossing-requested"
        && bridged_artifact.get("status").is_none()
        && bridged_artifact.get("published").is_none()
        && bridged_artifact["bifrostRequestReceipt"]["action"] == "other-request"
        && missing_bridge["ok"] == false
        && missing_bridge["blocked"] == "missing-bifrost-bridge"
        && missing_capability["ok"] == false
        && missing_capability["blocked"] == "missing-heimdall-capability-ref"
        && wrong_surface_capability["ok"] == false
        && wrong_surface_capability["blocked"] == "wrong-heimdall-capability-surface"
        && bridged["ok"] == true
        && bridged["transport"] == "bifrost.other-request"
        && bridged["bifrostBridgeReceipt"]["action"] == "other-request"
        && bridged["bifrostBridgeReceipt"]["surfaceName"] == "bluesky"
        && bridged["bifrostBridgeReceipt"]["provenance"]["bifrostIdentity"] == "epiphany.Persona"
        && bridged["bifrostBridgeReceipt"]["provenance"]["heimdallCapabilityRef"]
            == "heimdall:bluesky:capability:smoke-persona"
        && repeated["ok"] == false
        && repeated["blocked"] == "speech-audit"
        && repeated["speechAudit"]["reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| reason == "target-saturation"))
        && latest_cultmesh_audit.decision == "blocked"
        && !latest_cultmesh_audit.private_state_exposed;
    let result = serde_json::json!({
        "ok": ok,
        "draft": draft,
        "missingBridge": missing_bridge,
        "missingCapability": missing_capability,
        "wrongSurfaceCapability": wrong_surface_capability,
        "bridged": bridged,
        "repeatedSpeech": repeated,
        "latestCultMeshSpeechAudit": latest_cultmesh_audit,
    });
    let _ = fs::remove_dir_all(&temp_dir);
    Ok(result)
}

fn read_artifact(path: &Value) -> Result<Value> {
    let path = path
        .as_str()
        .context("smoke result omitted artifact path")?;
    serde_json::from_str(&fs::read_to_string(path)?)
        .with_context(|| format!("failed to decode smoke artifact {path}"))
}

fn exact_root_keys(value: &Value, expected: &[&str]) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.len() == expected.len() && expected.iter().all(|key| object.contains_key(*key))
}

fn seed_requested_artifact(
    content: &str,
    config: &PersonaOtherConfig,
    artifact_dir: &Path,
    cultmesh_store: &Path,
    runtime_id: &str,
) -> Result<()> {
    let target = resolve_target(config, None, None)?;
    let audit = audit_persona_speech(
        content,
        artifact_dir,
        cultmesh_store,
        runtime_id,
        &target.surface_name,
        &target.target_locator,
        false,
    )?;
    write_request_artifact(
        content,
        config,
        &target,
        artifact_dir,
        PersonaOtherLifecycle::CrossingRequested,
        "seed prior future-surface request for speech-audit smoke",
        Some(&audit),
        None,
        None,
        None,
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_request_artifact(
    content: &str,
    config: &PersonaOtherConfig,
    target: &ResolvedTarget,
    artifact_dir: &Path,
    lifecycle: PersonaOtherLifecycle,
    reason: &str,
    speech_audit: Option<&PersonaSpeechAudit>,
    bridge_receipt: Option<&BifrostOtherRequestReceipt>,
    title: Option<&str>,
    persona_name: Option<&str>,
) -> Result<PathBuf> {
    fs::create_dir_all(artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    let path = artifact_dir.join(format!("Persona-other-{}-{}.json", now_stamp(), short_id()));
    let persona_name = persona_name
        .or(config.persona_name.as_deref())
        .unwrap_or("Epiphany Persona");
    let payload = PersonaOtherRequestArtifact {
        schema_version: "epiphany.persona_other_request.v0".to_string(),
        lifecycle,
        reason: reason.to_string(),
        created_at: now_iso(),
        surface_name: target.surface_name.clone(),
        target_locator: target.target_locator.clone(),
        title: title
            .unwrap_or("Persona outside-world bridge request")
            .to_string(),
        persona_name: persona_name.to_string(),
        content: content.to_string(),
        speech_audit: speech_audit.cloned(),
        bifrost_request_receipt: bridge_receipt.cloned(),
        private_state_exposed: false,
    };
    write_json(&path, &serde_json::to_value(payload)?)?;
    Ok(path)
}

fn latest_persona_artifacts(artifact_dir: &Path, limit: usize) -> Vec<Value> {
    let mut entries = fs::read_dir(artifact_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with("Persona-other-") && name.ends_with(".json"))
        })
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            let modified = metadata.modified().ok()?;
            Some((Reverse(modified), entry.path()))
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries
        .into_iter()
        .take(limit)
        .filter_map(|(_, path)| {
            let raw = fs::read_to_string(&path).ok()?;
            let payload: Value = serde_json::from_str(&raw).ok()?;
            Some(serde_json::json!({
                "path": path,
                "lifecycle": payload.get("lifecycle").or_else(|| payload.get("status")),
                "surfaceName": payload.get("surfaceName"),
                "targetLocator": payload.get("targetLocator"),
                "privateStateExposed": payload.get("privateStateExposed"),
            }))
        })
        .collect()
}

fn recent_persona_speech(artifact_dir: &Path, limit: usize) -> Vec<RecentPersonaSpeech> {
    let mut entries = fs::read_dir(artifact_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with("Persona-other-") && name.ends_with(".json"))
        })
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            let modified = metadata.modified().ok()?;
            Some((Reverse(modified), entry.path()))
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries
        .into_iter()
        .take(limit)
        .filter_map(|(_, path)| {
            let raw = fs::read_to_string(path).ok()?;
            let payload: Value = serde_json::from_str(&raw).ok()?;
            let audit = payload.get("speechAudit")?;
            Some(RecentPersonaSpeech {
                opening_key: audit["openingKey"].as_str().unwrap_or_default().to_string(),
                topic_key: audit["topicKey"].as_str().unwrap_or_default().to_string(),
                target: audit["requestedPublicTarget"].as_str().map(str::to_string),
                crossing_recorded: payload["lifecycle"].as_str() == Some("crossing-requested")
                    || payload["status"].as_str() == Some("requested"),
            })
        })
        .collect()
}

#[derive(Clone, Debug)]
struct ResolvedTarget {
    surface_name: String,
    target_locator: String,
}

fn resolve_target(
    config: &PersonaOtherConfig,
    surface_name: Option<String>,
    target_locator: Option<String>,
) -> Result<ResolvedTarget> {
    let surface_name = normalize_surface_name(
        surface_name
            .as_deref()
            .unwrap_or(config.surface_name.as_str()),
    );
    if surface_name.is_empty() {
        return Err(anyhow!("outside-world surface name is required"));
    }
    let target_locator = target_locator
        .or_else(|| {
            (!config.target_locator.trim().is_empty()).then(|| config.target_locator.clone())
        })
        .unwrap_or_else(|| format!("{surface_name}:unspecified"));
    Ok(ResolvedTarget {
        surface_name,
        target_locator: target_locator.trim().to_string(),
    })
}

fn load_config(path: &Path) -> Result<PersonaOtherConfig> {
    if !path.exists() {
        return Ok(PersonaOtherConfig {
            surface_name: "future-surface".to_string(),
            target_locator: "future-surface:unspecified".to_string(),
            ..PersonaOtherConfig::default()
        });
    }
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    if path
        .extension()
        .is_some_and(|extension| extension == "json")
    {
        let payload: Value = serde_json::from_str(&raw)
            .with_context(|| format!("failed to decode {}", path.display()))?;
        return Ok(PersonaOtherConfig {
            surface_name: payload["surface_name"]
                .as_str()
                .map(normalize_surface_name)
                .unwrap_or_else(|| "future-surface".to_string()),
            target_locator: payload["target_locator"]
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| "future-surface:unspecified".to_string()),
            persona_name: payload["persona_name"].as_str().map(str::to_string),
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
    Ok(PersonaOtherConfig {
        surface_name: toml_string(&raw, "surface_name")
            .map(normalize_surface_name)
            .unwrap_or_else(|| "future-surface".to_string()),
        target_locator: toml_string(&raw, "target_locator")
            .unwrap_or_else(|| "future-surface:unspecified".to_string()),
        persona_name: toml_string(&raw, "persona_name"),
        bifrost_bridge_cli_path: toml_string(&raw, "bifrost_bridge_cli_path").map(PathBuf::from),
        bifrost_identity: toml_string(&raw, "bifrost_identity"),
        bifrost_source_kind: toml_string(&raw, "bifrost_source_kind"),
        bifrost_authority_ref: toml_string(&raw, "bifrost_authority_ref"),
        heimdall_capability_ref_env: toml_string(&raw, "heimdall_capability_ref_env"),
    })
}

fn bifrost_bridge_cli_path(config: &PersonaOtherConfig) -> Option<PathBuf> {
    config.bifrost_bridge_cli_path.clone().or_else(|| {
        env::var("EPIPHANY_PERSONA_BIFROST_BRIDGE_CLI")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    })
}

fn required_bifrost_identity(config: &PersonaOtherConfig) -> Option<String> {
    config
        .bifrost_identity
        .clone()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            env::var("BIFROST_IDENTITY")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
}

fn required_heimdall_capability_ref(config: &PersonaOtherConfig) -> Option<String> {
    config
        .heimdall_capability_ref_env
        .as_ref()
        .and_then(env_value)
        .or_else(|| {
            env::var("HEIMDALL_CAPABILITY_REF")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .or_else(|| {
            env::var("BIFROST_HEIMDALL_CAPABILITY_REF")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
}

fn is_heimdall_surface_reference(reference: &str, surface: &str) -> bool {
    let reference = reference.trim().to_ascii_lowercase();
    let surface = normalize_surface_name(surface);
    reference.starts_with("heimdall:") && reference.contains(&format!(":{surface}:"))
}

fn normalize_surface_name(value: impl AsRef<str>) -> String {
    value
        .as_ref()
        .trim()
        .trim_matches('/')
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
        .collect::<String>()
        .to_ascii_lowercase()
}

fn bifrost_node_executable() -> String {
    env::var("EPIPHANY_BIFROST_NODE")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "node".to_string())
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

fn read_required_text(value: Option<String>, label: &str) -> Result<String> {
    let Some(value) = value else {
        return Err(anyhow!("{label} is required"));
    };
    if let Some(path) = value.strip_prefix('@') {
        return fs::read_to_string(path).with_context(|| format!("failed to read {path}"));
    }
    if value == "-" {
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .context("failed to read stdin")?;
        return Ok(buffer);
    }
    Ok(value)
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
    fs::create_dir(&path).with_context(|| format!("failed to claim {}", path.display()))?;
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
        "usage: epiphany-persona-other <draft|request|latest|smoke> [--config <path>] [--artifact-dir <path>] [--surface-name <name>] [--target-locator <locator>] [--title <text>] [--content <text-or-path>]"
    ))
}
