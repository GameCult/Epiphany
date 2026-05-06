use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::SecondsFormat;
use epiphany_core::EpiphanyAgentMemoryEntry;
use epiphany_core::GhostlightMemory;
use epiphany_core::load_agent_memory_entry_for_role;
use serde_json::Value;
use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

const CHARACTER_TURN_SCHEMA_VERSION: &str = "epiphany.character_turn_packet.v0";

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return usage();
    };
    let mut role = "face".to_string();
    let mut agent_store = PathBuf::from("state/agents.msgpack");
    let mut artifact_dir = PathBuf::from(".epiphany-character-loop");
    let mut stimulus: Option<String> = None;
    let mut source = "epiphany/character-loop".to_string();
    let mut mode = "public-surface".to_string();
    let mut status = "ready".to_string();
    let mut mood = "attentive".to_string();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--role" => role = next_value(&mut args, "--role")?,
            "--agent-store" => agent_store = next_path(&mut args, "--agent-store")?,
            "--artifact-dir" => artifact_dir = next_path(&mut args, "--artifact-dir")?,
            "--stimulus" => stimulus = Some(next_value(&mut args, "--stimulus")?),
            "--source" => source = next_value(&mut args, "--source")?,
            "--mode" => mode = next_value(&mut args, "--mode")?,
            "--status" => status = next_value(&mut args, "--status")?,
            "--mood" => mood = next_value(&mut args, "--mood")?,
            _ => return Err(anyhow!("unknown argument {arg:?}")),
        }
    }

    let result = match command.as_str() {
        "turn" => {
            let stimulus = read_required_text(stimulus, "stimulus")?;
            run_turn(
                &role,
                &agent_store,
                &artifact_dir,
                &stimulus,
                &source,
                &mode,
                &status,
                &mood,
            )?
        }
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

fn run_turn(
    role: &str,
    agent_store: &Path,
    artifact_dir: &Path,
    stimulus: &str,
    source: &str,
    mode: &str,
    status: &str,
    mood: &str,
) -> Result<Value> {
    if role.trim().is_empty() {
        return Err(anyhow!("--role must not be empty"));
    }
    ensure_content(stimulus, "stimulus")?;
    let entry = load_agent_memory_entry_for_role(agent_store, role)?.ok_or_else(|| {
        anyhow!(
            "{role} agent memory is missing from {}",
            agent_store.display()
        )
    })?;
    let packet = character_turn_packet(&entry, stimulus, source, mode, status, mood);
    let path = artifact_dir.join(format!(
        "character-turn-{}-{}-{}.json",
        sanitize_file_stem(role),
        now_stamp(),
        short_id()
    ));
    write_json(&path, &packet)?;
    Ok(serde_json::json!({
        "ok": true,
        "schemaVersion": CHARACTER_TURN_SCHEMA_VERSION,
        "roleId": role,
        "agentId": entry.agent.agent_id,
        "turnPath": path,
        "turn": packet,
    }))
}

fn character_turn_packet(
    entry: &EpiphanyAgentMemoryEntry,
    stimulus: &str,
    source: &str,
    mode: &str,
    status: &str,
    mood: &str,
) -> Value {
    serde_json::json!({
        "schema_version": CHARACTER_TURN_SCHEMA_VERSION,
        "protocol": {
            "bundle": "epiphany.character_loop",
            "referenceLineage": "Ghostlight-style character-local cognition, rebuilt as Epiphany-native protocol.",
            "mode": mode,
            "roleId": entry.role_id,
            "agentId": entry.agent.agent_id,
            "turnKind": "stimulus_response",
            "responseContract": "Respond from projected local state and visible stimulus; do not leak hidden agent streams or invent omniscient truth.",
        },
        "source": source,
        "createdAt": now_iso(),
        "stimulus": {
            "kind": "operator_or_aquarium_input",
            "content": stimulus,
            "status": status,
            "mood": mood,
        },
        "projectedLocalContext": {
            "identity": {
                "name": entry.agent.identity.name,
                "roles": entry.agent.identity.roles,
                "origin": entry.agent.identity.origin,
                "publicDescription": entry.agent.identity.public_description,
            },
            "world": {
                "worldId": entry.world.world_id,
                "setting": entry.world.setting,
                "time": entry.world.time.label,
                "canonContext": entry.world.canon_context,
            },
            "activeGoals": entry.agent.goals.iter().map(|goal| serde_json::json!({
                "goalId": goal.goal_id,
                "description": goal.description,
                "scope": goal.scope,
                "priority": goal.priority,
                "emotionalStake": goal.emotional_stake,
                "status": goal.status,
            })).collect::<Vec<_>>(),
            "values": entry.agent.canonical_state.values.iter().map(|value| serde_json::json!({
                "valueId": value.value_id,
                "label": value.label,
                "priority": value.priority,
                "unforgivableIfBetrayed": value.unforgivable_if_betrayed,
            })).collect::<Vec<_>>(),
            "semanticMemories": entry.agent.memories.semantic.iter().map(memory_projection).collect::<Vec<_>>(),
            "episodicMemories": entry.agent.memories.episodic.iter().map(memory_projection).collect::<Vec<_>>(),
            "relationshipMemories": entry.agent.memories.relationship_summaries.iter().map(memory_projection).collect::<Vec<_>>(),
            "privateNoteCount": entry.agent.identity.private_notes.len(),
        },
        "allowedOutputs": [
            "bubble",
            "discordAquariumDraft",
            "roleFinding",
            "silence",
            "selfPatch"
        ],
        "guardrails": [
            "Humans talk to Face; other organs expose internals through Aquarium and typed artifacts rather than becoming direct chats.",
            "Use character-local projection: the actor receives its own dossier and visible stimulus, not omniscient hidden state.",
            "If speech or action would be noise, return silence plus bounded rumination or selfPatch.",
            "Posting, state mutation, objective adoption, and semantic acceptance remain separate reviewed Epiphany surfaces."
        ],
        "resultShape": {
            "observedStimulus": "string",
            "localInterpretation": "string",
            "candidateBubble": "string|null",
            "candidateDraft": "string|null",
            "decision": "bubble|draft|finding|silence|blocked",
            "selfPatch": "optional bounded role memory mutation"
        }
    })
}

fn memory_projection(memory: &GhostlightMemory) -> Value {
    serde_json::json!({
        "memoryId": memory.memory_id,
        "summary": memory.summary,
        "salience": memory.salience,
        "confidence": memory.confidence,
    })
}

fn run_smoke() -> Result<Value> {
    let temp_dir = scoped_temp_dir("epiphany-character-loop-smoke")?;
    let artifact_dir = temp_dir.join("artifacts");
    let result = run_turn(
        "face",
        Path::new("state/agents.msgpack"),
        &artifact_dir,
        "Aquarium hover selected Face and asked what the swarm is feeling.",
        "smoke/character-loop",
        "public-surface",
        "ready",
        "attentive",
    )?;
    let path = PathBuf::from(
        result["turnPath"]
            .as_str()
            .ok_or_else(|| anyhow!("smoke result missing turnPath"))?,
    );
    let packet: Value = serde_json::from_str(
        &fs::read_to_string(&path)
            .with_context(|| format!("failed to read smoke artifact {}", path.display()))?,
    )?;
    let ok = packet["schema_version"] == CHARACTER_TURN_SCHEMA_VERSION
        && packet["protocol"]["bundle"] == "epiphany.character_loop"
        && packet["protocol"]["roleId"] == "face"
        && packet["projectedLocalContext"]["identity"]["name"]
            .as_str()
            .is_some_and(|name| !name.trim().is_empty())
        && packet["stimulus"]["content"]
            .as_str()
            .is_some_and(|text| text.contains("Aquarium"));
    Ok(serde_json::json!({
        "ok": ok,
        "turnPath": path,
        "packet": {
            "schemaVersion": packet["schema_version"],
            "bundle": packet["protocol"]["bundle"],
            "roleId": packet["protocol"]["roleId"],
            "agentId": packet["protocol"]["agentId"],
            "identityName": packet["projectedLocalContext"]["identity"]["name"],
            "allowedOutputs": packet["allowedOutputs"],
        }
    }))
}

fn read_required_text(content: Option<String>, label: &str) -> Result<String> {
    let Some(content) = content else {
        let mut stdin = String::new();
        std::io::stdin()
            .read_to_string(&mut stdin)
            .context("failed to read stdin")?;
        ensure_content(&stdin, label)?;
        return Ok(stdin);
    };
    let path = PathBuf::from(&content);
    if path.exists() {
        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read content file {}", path.display()))?;
        ensure_content(&text, label)?;
        return Ok(text);
    }
    ensure_content(&content, label)?;
    Ok(content)
}

fn ensure_content(content: &str, label: &str) -> Result<()> {
    if content.trim().is_empty() {
        return Err(anyhow!("{label} is empty"));
    }
    Ok(())
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("missing value after {flag}"))
}

fn next_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(next_value(args, flag)?))
}

fn write_json(path: &Path, payload: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(payload)?),
    )
    .with_context(|| format!("failed to write {}", path.display()))
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn now_stamp() -> String {
    chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

fn short_id() -> String {
    Uuid::new_v4()
        .to_string()
        .split('-')
        .next()
        .unwrap_or("id")
        .to_string()
}

fn sanitize_file_stem(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn scoped_temp_dir(prefix: &str) -> Result<PathBuf> {
    let dir = env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    Ok(dir)
}

fn usage() -> Result<()> {
    Err(anyhow!(
        "usage: epiphany-character-loop <turn|smoke> [--role <role>] [--agent-store <path>] [--artifact-dir <path>] [--stimulus <text-or-path>] [--source <source>] [--mode <mode>]"
    ))
}
