use std::{env, path::PathBuf, process::Command, thread, time::Duration};

use anyhow::{Context, Result, anyhow};
use epiphany_core::{
    PersonaIdentity, PersonaProjectorInput, PersonaRepoActivity, PersonaSocialAffordance,
    PersonaTranscriptMessage, PersonaTurnTerminalOptions, complete_persona_turn_request_store,
    default_organ_dependencies_for, load_agent_memory_entry_for_role,
    load_epiphany_cultmesh_swarm_brake, load_heartbeat_state_entry,
    load_persona_discord_receipt_anchor, load_persona_discord_service_anchor,
    open_persona_discord_request_identity, persona_delivery_receipt_exists_for_turn,
    persona_model_terminal_exists, poll_persona_discord_crossing,
    reconcile_terminal_persona_conversation, semantic_memory_recall_from_heartbeat_action,
    validate_persona_discord_request_anchor,
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
    let plan = PersonaModelExecutionPlan {
        turn_id: request.request_id.clone(),
        provider: options.provider.clone(),
        model: options.model.clone(),
        projector_input: PersonaProjectorInput {
            identity,
            memory,
            semantic_memory_recall: semantic_recall.clone(),
            pending_mentions: request.mentions.clone(),
            repo_activity: vec![observe_repo_activity(
                &options.repo_root,
                &options.repo_name,
            )?],
            social_affordances,
            organ_dependencies: vec![default_organ_dependencies_for("Persona")],
        },
        transcript,
        allowed_channel_ids: request
            .mentions
            .iter()
            .map(|mention| mention.channel_id.clone())
            .collect(),
        dynamic_semantic_memory_recall: semantic_recall,
        cultmesh_store: options.cultmesh_store.clone(),
        runtime_id: options.runtime_id.clone(),
    };
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
    let receipt_anchor = load_persona_discord_receipt_anchor(&options.mouth_receipt_anchor)?;
    let result = poll_persona_discord_crossing(
        &options.runtime_store,
        &options.heartbeat_store,
        &options.agent_store,
        &options.cultmesh_store,
        &options.runtime_id,
        &options.mouth_request_store,
        &options.mouth_receipt_store,
        &signer,
        &receipt_anchor,
        &request.request_id,
        &model_terminal.effect_document_id,
    )?;
    Ok(result.is_some())
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
    repo_root: PathBuf,
    mouth_request_store: PathBuf,
    mouth_receipt_store: PathBuf,
    mouth_identity_store: PathBuf,
    mouth_request_anchor: PathBuf,
    mouth_receipt_anchor: PathBuf,
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
            repo_root: path("--repo-root")?,
            mouth_request_store: path("--mouth-request-store")?,
            mouth_receipt_store: path("--mouth-receipt-store")?,
            mouth_identity_store: path("--mouth-identity-store")?,
            mouth_request_anchor: path("--mouth-request-anchor")?,
            mouth_receipt_anchor: path("--mouth-receipt-anchor")?,
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

fn observe_repo_activity(repo_root: &PathBuf, repo_name: &str) -> Result<PersonaRepoActivity> {
    let head = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["log", "-1", "--format=%H %s"])
        .output()
        .context("failed to inspect Persona home-repo head")?;
    let status = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["status", "--short"])
        .output()
        .context("failed to inspect Persona home-repo status")?;
    if !head.status.success() || !status.status.success() {
        return Err(anyhow!("Persona home-repo activity is unavailable"));
    }
    let head_text = String::from_utf8(head.stdout)?.trim().to_string();
    let changed = String::from_utf8(status.stdout)?.lines().count();
    Ok(PersonaRepoActivity {
        repo_name: repo_name.into(),
        summary: format!("Current body head: {head_text}; {changed} worktree path(s) changed."),
        refs: head_text
            .split_whitespace()
            .next()
            .map(|value| format!("git:{value}"))
            .into_iter()
            .collect(),
    })
}
