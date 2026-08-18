use std::{env, net::SocketAddr, path::PathBuf, thread, time::Duration};

use anyhow::{Context, Result, anyhow};
use cultcache_rs::{CacheBackingStore, DatabaseEntry, SingleFileMessagePackBackingStore};
use epiphany_core::{
    EpiphanyAgentMemoryEntry, EpiphanyHeartbeatStateEntry, EpiphanyMindDocumentVersion,
    EpiphanyMindPersonaPassInputDocument, PersonaIdentity, PersonaProjectorInput,
    PersonaRepoActivity, PersonaSocialAffordance, PersonaTranscriptMessage, PersonaTurnRequest,
    PersonaTurnTerminalOptions, admit_persona_pass_input, assemble_mind_view,
    complete_persona_turn_request_store, default_organ_dependencies_for,
    exchange_persona_discord_delivery_rudp, load_admitted_persona_pass_input,
    load_agent_memory_entry_for_role, load_epiphany_cultmesh_swarm_brake,
    load_heartbeat_state_entry, load_persona_discord_receipt_anchor,
    load_persona_discord_service_anchor, open_persona_discord_request_identity,
    pending_persona_discord_delivery_request_for_turn, persona_delivery_receipt_exists_for_turn,
    persona_model_terminal_exists, poll_persona_discord_crossing,
    reconcile_terminal_persona_conversation, retain_terminal_persona_conversations,
    semantic_memory_recall_from_heartbeat_action, validate_persona_discord_request_anchor,
};
use epiphany_openai_runtime::{
    NativePersonaModelRunner, PersonaModelExecutionPlan, execute_persona_model_turn,
};

#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::parse()?;
    loop {
        let worked = poll_once(&options).await?;
        if options.once {
            break;
        }
        if !worked {
            thread::sleep(Duration::from_millis(options.poll_ms));
        }
    }
    Ok(())
}

async fn poll_once(options: &Options) -> Result<bool> {
    let brake = load_epiphany_cultmesh_swarm_brake(&options.cultmesh_store, &options.runtime_id)?
        .ok_or_else(|| anyhow!("Persona service requires canonical brake state"))?;
    if brake.status != "released" {
        return Ok(false);
    }
    let receipt_anchor = load_persona_discord_receipt_anchor(&options.mouth_receipt_anchor)?;
    retain_terminal_persona_conversations(
        &options.runtime_store,
        &options.heartbeat_store,
        &options.mouth_request_store,
        &options.mouth_receipt_store,
        &receipt_anchor,
        options.retained_terminal_conversations,
        &chrono::Utc::now().to_rfc3339(),
    )?;
    let Some(state) = load_heartbeat_state_entry(&options.heartbeat_store)? else {
        return Ok(false);
    };
    for request in state
        .persona_turn_requests
        .iter()
        .filter(|request| request.terminal_receipt.is_some())
    {
        reconcile_terminal_persona_conversation(
            &options.runtime_store,
            &options.heartbeat_store,
            &request.request_id,
        )?;
    }
    let mut candidates = state
        .persona_turn_requests
        .into_iter()
        .filter(|request| request.status == "reserved" && request.terminal_receipt.is_none())
        .collect::<Vec<_>>();
    candidates.sort_by_key(|request| {
        if persona_delivery_receipt_exists_for_turn(
            &options.mouth_receipt_store,
            &request.request_id,
        )
        .unwrap_or(false)
        {
            0
        } else if !persona_model_terminal_exists(&options.runtime_store, &request.request_id)
            .unwrap_or(false)
        {
            1
        } else {
            2
        }
    });
    let Some(request) = candidates.into_iter().next() else {
        return Ok(false);
    };
    ensure_persona_pass_input_admitted(options, &request)?;
    let plan = PersonaModelExecutionPlan::from_admitted_input(
        &options.runtime_store,
        &request.request_id,
        options.provider.clone(),
        options.model.clone(),
        options.cultmesh_store.clone(),
        options.runtime_id.clone(),
    )?;
    let mut runner = NativePersonaModelRunner {
        store_path: options.runtime_store.clone(),
        codex_home: options.codex_home.clone(),
        provider: options.provider.clone(),
        model: options.model.clone(),
    };
    let model_terminal = match execute_persona_model_turn(&plan, &mut runner).await {
        Ok(receipt) => receipt,
        Err(error) if error.to_string().contains("braked") => return Ok(false),
        Err(error) => {
            complete_persona_turn_request_store(
                &options.heartbeat_store,
                PersonaTurnTerminalOptions {
                    request_id: request.request_id,
                    outcome: "failed".into(),
                    delivery_evidence: None,
                    blocked_evidence: None,
                },
            )?;
            return Err(error);
        }
    };
    let signer = open_persona_discord_request_identity(&options.mouth_identity_store)?;
    let request_anchor = load_persona_discord_service_anchor(&options.mouth_request_anchor)?;
    validate_persona_discord_request_anchor(&request_anchor, &options.runtime_id)?;
    if request_anchor.signer_identity_id != signer.entry().identity_id {
        return Err(anyhow!(
            "Persona mouth identity does not match its root-bound request anchor"
        ));
    }
    let mut result = poll_persona_discord_crossing(
        &options.runtime_store,
        &options.heartbeat_store,
        &options.cultmesh_store,
        &options.runtime_id,
        &options.mouth_request_store,
        &options.mouth_receipt_store,
        &signer,
        &receipt_anchor,
        &request.request_id,
        &model_terminal.effect_document_id,
    )?;
    if result.is_none() {
        let crossing = pending_persona_discord_delivery_request_for_turn(
            &options.mouth_request_store,
            &request.request_id,
        )?
        .ok_or_else(|| anyhow!("Persona crossing is pending without a durable request"))?;
        match exchange_persona_discord_delivery_rudp(
            options.mouth_rudp,
            &options.runtime_id,
            &options.mouth_request_store,
            &options.mouth_receipt_store,
            &crossing,
            &receipt_anchor,
            Duration::from_millis(options.mouth_timeout_ms),
        ) {
            Ok(_) => {
                result = poll_persona_discord_crossing(
                    &options.runtime_store,
                    &options.heartbeat_store,
                    &options.cultmesh_store,
                    &options.runtime_id,
                    &options.mouth_request_store,
                    &options.mouth_receipt_store,
                    &signer,
                    &receipt_anchor,
                    &request.request_id,
                    &model_terminal.effect_document_id,
                )?;
            }
            Err(error) => eprintln!("Persona delivery remains durably pending: {error:#}"),
        }
    }
    Ok(result.is_some())
}

fn ensure_persona_pass_input_admitted(
    options: &Options,
    request: &PersonaTurnRequest,
) -> Result<()> {
    if load_admitted_persona_pass_input(&options.runtime_store, &request.request_id)?.is_some() {
        return Ok(());
    }

    let memory = load_agent_memory_entry_for_role(&options.agent_store, &request.role_id)
        .ok()
        .flatten();
    let identity = PersonaIdentity {
        identity_id: request.agent_id.clone(),
        display_name: options.persona_name.clone(),
        repo_name: options.repo_name.clone(),
        public_description: options.persona_description.clone(),
        jurisdiction: vec![options.repo_name.clone()],
    };
    let transcript = request
        .mentions
        .iter()
        .map(|mention| PersonaTranscriptMessage {
            channel_id: mention.channel_id.clone(),
            message_id: mention.message_id.clone(),
            author_id: mention.author_id.clone(),
            author_name: mention
                .author_name
                .clone()
                .unwrap_or_else(|| mention.author_id.clone()),
            is_agent: false,
            content: mention.content.clone(),
            timestamp: mention.queued_at.clone(),
        })
        .collect::<Vec<_>>();
    let semantic_recall = semantic_memory_recall_from_heartbeat_action(
        &serde_json::json!({"persona_memory_recall": request.semantic_memory_recall}),
    );
    let social_affordances = request
        .mentions
        .iter()
        .map(|mention| PersonaSocialAffordance {
            person_id: mention.author_id.clone(),
            summary: format!(
                "{} directly addressed the Persona in this reserved turn.",
                mention.author_name.as_deref().unwrap_or(&mention.author_id)
            ),
            recent_message_ids: vec![mention.message_id.clone()],
        })
        .collect();
    let mind = assemble_mind_view(&options.runtime_store)?;
    let repo_activity = mind
        .repository_body_observation
        .as_ref()
        .map(|body| PersonaRepoActivity {
            repo_name: options.repo_name.clone(),
            summary: format!(
                "Typed repository Body generation {} at manifest {}.",
                body.generation, body.manifest_root_sha256
            ),
            refs: vec![body.observation_id.clone()],
        })
        .into_iter()
        .collect();
    let has_memory = memory.is_some();
    let projector_input = PersonaProjectorInput {
        identity,
        memory,
        semantic_memory_recall: semantic_recall.clone(),
        pending_mentions: request.mentions.clone(),
        repo_activity,
        social_affordances,
        organ_dependencies: vec![default_organ_dependencies_for("Persona")],
    };
    let heartbeat_envelope = SingleFileMessagePackBackingStore::new(&options.heartbeat_store)
        .pull_all()?
        .into_iter()
        .find(|entry| entry.r#type == EpiphanyHeartbeatStateEntry::TYPE)
        .ok_or_else(|| anyhow!("Persona pass input lost its heartbeat source"))?;
    let heartbeat_source =
        EpiphanyMindDocumentVersion::from_envelope("epiphany-heartbeat", &heartbeat_envelope)?;
    let mut observed_sources = vec![heartbeat_source.clone()];
    if has_memory {
        let agent_envelope = SingleFileMessagePackBackingStore::new(&options.agent_store)
            .pull_all()?
            .into_iter()
            .find(|entry| {
                entry.r#type == EpiphanyAgentMemoryEntry::TYPE && entry.key == request.role_id
            })
            .ok_or_else(|| anyhow!("Persona pass input lost its agent-memory source"))?;
        observed_sources.push(EpiphanyMindDocumentVersion::from_envelope(
            "epiphany-agent-memory",
            &agent_envelope,
        )?);
    }
    if let Some(source) = mind.source_documents.iter().find(|source| {
        source.document_type == epiphany_core::EpiphanyMindRepositoryBodyObservationDocument::TYPE
            && mind
                .repository_body_observation
                .as_ref()
                .is_some_and(|body| source.document_key == body.observation_id)
    }) {
        observed_sources.push(source.clone());
    }
    let document = EpiphanyMindPersonaPassInputDocument {
        turn_id: request.request_id.clone(),
        projector_input,
        transcript,
        allowed_channel_ids: request
            .mentions
            .iter()
            .map(|mention| mention.channel_id.clone())
            .collect(),
        dynamic_semantic_memory_recall: semantic_recall,
        observed_sources,
        admitted_at: request.reserved_at.clone(),
    };
    admit_persona_pass_input(&options.runtime_store, heartbeat_source, &document)?;
    load_admitted_persona_pass_input(&options.runtime_store, &request.request_id)?
        .ok_or_else(|| anyhow!("Persona pass input admission was not durable"))?;
    Ok(())
}

struct Options {
    runtime_store: PathBuf,
    heartbeat_store: PathBuf,
    agent_store: PathBuf,
    cultmesh_store: PathBuf,
    codex_home: PathBuf,
    runtime_id: String,
    provider: String,
    model: String,
    repo_name: String,
    persona_name: String,
    persona_description: String,
    _repo_root: PathBuf,
    mouth_request_store: PathBuf,
    mouth_receipt_store: PathBuf,
    mouth_identity_store: PathBuf,
    mouth_request_anchor: PathBuf,
    mouth_receipt_anchor: PathBuf,
    mouth_rudp: SocketAddr,
    mouth_timeout_ms: u64,
    retained_terminal_conversations: usize,
    poll_ms: u64,
    once: bool,
}
impl Options {
    fn parse() -> Result<Self> {
        let mut values = std::collections::BTreeMap::new();
        let mut once = false;
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            if arg == "--once" {
                once = true;
                continue;
            }
            values.insert(
                arg,
                args.next()
                    .ok_or_else(|| anyhow!("argument requires value"))?,
            );
        }
        let path = |key: &str| {
            values
                .get(key)
                .map(PathBuf::from)
                .ok_or_else(|| anyhow!("{key} is required"))
        };
        Ok(Self {
            runtime_store: path("--runtime-store")?,
            heartbeat_store: path("--heartbeat-store")?,
            agent_store: path("--agent-store")?,
            cultmesh_store: path("--cultmesh-store")?,
            codex_home: path("--codex-home")?,
            _repo_root: path("--repo-root")?,
            mouth_request_store: path("--mouth-request-store")?,
            mouth_receipt_store: path("--mouth-receipt-store")?,
            mouth_identity_store: path("--mouth-identity-store")?,
            mouth_request_anchor: path("--mouth-request-anchor")?,
            mouth_receipt_anchor: path("--mouth-receipt-anchor")?,
            mouth_rudp: value(&values, "--mouth-rudp", "")
                .parse()
                .context("--mouth-rudp must be an IP:port socket address")?,
            mouth_timeout_ms: value(&values, "--mouth-timeout-ms", "10000").parse()?,
            retained_terminal_conversations: value(
                &values,
                "--retained-terminal-conversations",
                "256",
            )
            .parse()?,
            runtime_id: value(&values, "--runtime-id", "epiphany-local"),
            provider: value(&values, "--provider", "openai-codex"),
            model: value(&values, "--model", "gpt-5.4"),
            repo_name: value(&values, "--repo-name", "EpiphanyAgent"),
            persona_name: value(&values, "--persona-name", "Epiphany"),
            persona_description: value(
                &values,
                "--persona-description",
                "The resident Persona of this Epiphany swarm.",
            ),
            poll_ms: value(&values, "--poll-ms", "2000").parse()?,
            once,
        })
    }
}
fn value(values: &std::collections::BTreeMap<String, String>, key: &str, default: &str) -> String {
    values.get(key).cloned().unwrap_or_else(|| default.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admitted_reentry_does_not_reobserve_external_stores() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let runtime_store = temp.path().join("runtime.cc");
        epiphany_core::initialize_runtime_spine(
            &runtime_store,
            epiphany_core::RuntimeSpineInitOptions {
                runtime_id: "persona-reentry".into(),
                display_name: "Persona reentry".into(),
                created_at: "2026-08-18T03:00:00Z".into(),
            },
        )?;
        let mut cache = epiphany_core::runtime_spine_cache(&runtime_store)?;
        cache.pull_all_backing_stores()?;
        let provenance = EpiphanyMindDocumentVersion::from_envelope(
            "epiphany-heartbeat",
            &cache
                .get_envelope::<epiphany_core::EpiphanyRuntimeIdentity>(
                    epiphany_core::RUNTIME_IDENTITY_KEY,
                )?
                .expect("runtime identity"),
        )?;
        let document = EpiphanyMindPersonaPassInputDocument {
            turn_id: "turn-reentry".into(),
            projector_input: PersonaProjectorInput {
                identity: PersonaIdentity {
                    identity_id: "epiphany.Persona".into(),
                    display_name: "Epiphany".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            transcript: Vec::new(),
            allowed_channel_ids: vec!["aquarium".into()],
            dynamic_semantic_memory_recall: String::new(),
            observed_sources: vec![provenance.clone()],
            admitted_at: "2026-08-18T03:00:01Z".into(),
        };
        admit_persona_pass_input(&runtime_store, provenance, &document)?;
        let request = PersonaTurnRequest {
            request_id: document.turn_id.clone(),
            role_id: "Persona".into(),
            agent_id: "epiphany.Persona".into(),
            reserved_at: document.admitted_at.clone(),
            status: "reserved".into(),
            ..Default::default()
        };
        let absent = temp.path().join("intentionally-absent");
        let options = Options {
            runtime_store: runtime_store.clone(),
            heartbeat_store: absent.join("heartbeat.cc"),
            agent_store: absent.join("agent.cc"),
            cultmesh_store: absent.join("cultmesh.cc"),
            codex_home: absent.join("codex-home"),
            runtime_id: "persona-reentry".into(),
            provider: "test".into(),
            model: "test-model".into(),
            repo_name: "Epiphany".into(),
            persona_name: "Epiphany".into(),
            persona_description: String::new(),
            _repo_root: absent.join("repo"),
            mouth_request_store: absent.join("mouth-requests.cc"),
            mouth_receipt_store: absent.join("mouth-receipts.cc"),
            mouth_identity_store: absent.join("mouth-identity.cc"),
            mouth_request_anchor: absent.join("request-anchor.cc"),
            mouth_receipt_anchor: absent.join("receipt-anchor.cc"),
            mouth_rudp: "127.0.0.1:1".parse()?,
            mouth_timeout_ms: 1,
            retained_terminal_conversations: 1,
            poll_ms: 1,
            once: true,
        };

        ensure_persona_pass_input_admitted(&options, &request)?;
        PersonaModelExecutionPlan::from_admitted_input(
            &runtime_store,
            &request.request_id,
            "test",
            "test-model",
            options.cultmesh_store,
            "persona-reentry",
        )?;
        assert!(!absent.exists());
        Ok(())
    }
}
