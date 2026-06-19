use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::Utc;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        std::process::exit(2);
    };
    let result = match command.as_str() {
        "online" => run_online(parse_online_args(args)?),
        other => Err(anyhow!("unknown epiphany-swarm command {other:?}")),
    }?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Clone, Debug)]
struct OnlineArgs {
    workspace: PathBuf,
    epiphany_root: PathBuf,
    local_verse_store: Option<PathBuf>,
    state_dir: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    runtime_id: Option<String>,
    init_receipt: Option<PathBuf>,
    agent_template_store: Option<PathBuf>,
}

fn parse_online_args(args: impl Iterator<Item = String>) -> Result<OnlineArgs> {
    let mut workspace = None;
    let mut epiphany_root = None;
    let mut local_verse_store = None;
    let mut state_dir = None;
    let mut artifact_dir = None;
    let mut runtime_id = None;
    let mut init_receipt = None;
    let mut agent_template_store = None;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--epiphany-root" => epiphany_root = Some(take_path(&mut args, "--epiphany-root")?),
            "--local-verse-store" | "--store" => {
                local_verse_store = Some(take_path(&mut args, "--local-verse-store")?);
            }
            "--state-dir" => state_dir = Some(take_path(&mut args, "--state-dir")?),
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--runtime-id" => runtime_id = Some(take_string(&mut args, "--runtime-id")?),
            "--init-receipt" => init_receipt = Some(take_path(&mut args, "--init-receipt")?),
            "--agent-template-store" => {
                agent_template_store = Some(take_path(&mut args, "--agent-template-store")?);
            }
            other => return Err(anyhow!("unexpected online argument {other:?}")),
        }
    }
    Ok(OnlineArgs {
        workspace: workspace.context("missing --workspace")?,
        epiphany_root: epiphany_root
            .unwrap_or(env::current_dir().context("failed to resolve current directory")?),
        local_verse_store,
        state_dir,
        artifact_dir,
        runtime_id,
        init_receipt,
        agent_template_store,
    })
}

fn run_online(args: OnlineArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let epiphany_root = args
        .epiphany_root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.epiphany_root.display()))?;
    let manifest_path = epiphany_root.join("epiphany-core").join("Cargo.toml");
    if !manifest_path.exists() {
        return Err(anyhow!(
            "could not find epiphany-core manifest at {}",
            manifest_path.display()
        ));
    }
    let init_receipt_path = args.init_receipt.unwrap_or_else(|| {
        workspace
            .join(".epiphany")
            .join("repo-init")
            .join("repo-swarm-init-receipt.json")
    });
    let init_receipt = read_json(&init_receipt_path)
        .with_context(|| format!("repo swarm init receipt is required; run epiphany-repo init first or pass --init-receipt"))?;
    let state_dir = args.state_dir.unwrap_or_else(|| {
        path_from_json(&init_receipt, &["stores", "stateDir"])
            .unwrap_or_else(|| workspace.join(".epiphany").join("state"))
    });
    let artifact_dir = args
        .artifact_dir
        .unwrap_or_else(|| workspace.join(".epiphany").join("swarm-online"));
    let local_verse_store = args
        .local_verse_store
        .unwrap_or_else(|| workspace.join(".epiphany").join("local-verse.ccmp"));
    let agent_store = state_dir.join("agents.msgpack");
    let runtime_id = args.runtime_id.unwrap_or_else(|| {
        format!(
            "repo-swarm-{}",
            workspace
                .file_name()
                .and_then(|name| name.to_str())
                .map(sanitize)
                .unwrap_or_else(|| "workspace".to_string())
        )
    });
    fs::create_dir_all(&state_dir)
        .with_context(|| format!("failed to create {}", state_dir.display()))?;
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    if let Some(parent) = local_verse_store.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let agent_seed = ensure_agent_store(
        &agent_store,
        args.agent_template_store
            .unwrap_or_else(|| epiphany_root.join("state").join("agents.msgpack")),
    )?;
    let soa_refresh = cargo_json(
        &manifest_path,
        "epiphany-agent-memory-store",
        &[
            "refresh-soa".to_string(),
            "--store".to_string(),
            agent_store.display().to_string(),
        ],
    )?;

    let seed = verse_json(
        &manifest_path,
        "seed-compact",
        &local_verse_store,
        &runtime_id,
        &[],
    )?;
    let agent_state = verse_json(
        &manifest_path,
        "agent-state",
        &local_verse_store,
        &runtime_id,
        &[
            "--agent-store".to_string(),
            agent_store.display().to_string(),
        ],
    )?;
    let agent_state_report = verse_json(
        &manifest_path,
        "agent-state-report",
        &local_verse_store,
        &runtime_id,
        &[],
    )?;
    let topology = verse_json(
        &manifest_path,
        "cluster-topology",
        &local_verse_store,
        &runtime_id,
        &[],
    )?;
    let liveness = verse_json(
        &manifest_path,
        "swarm-status",
        &local_verse_store,
        &runtime_id,
        &[],
    )?;
    let tool_directory = verse_json(
        &manifest_path,
        "tool-directory",
        &local_verse_store,
        &runtime_id,
        &[],
    )?;
    let overview = verse_json(
        &manifest_path,
        "swarm-overview",
        &local_verse_store,
        &runtime_id,
        &[],
    )?;

    let receipt = json!({
        "schemaVersion": "epiphany.repo_swarm_online_receipt.v0",
        "createdAt": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "workspace": workspace,
        "runtimeId": runtime_id,
        "initReceiptPath": init_receipt_path,
        "localVerseStore": local_verse_store,
        "stateDir": state_dir,
        "agentStore": agent_store,
        "agentSeed": agent_seed,
        "seed": seed,
        "soaRefresh": compact_fields(&soa_refresh, &["rowCount", "roleIds", "ok"]),
        "agentState": compact_fields(&agent_state, &["status", "summaryId", "rowCount", "privateStateExposed"]),
        "agentStateReport": compact_fields(&agent_state_report, &["status", "agentCount", "roleIds", "privateStateExposed"]),
        "topology": compact_fields(&topology, &["status", "clusterCount", "privateVerseCount", "daemonCount", "publicDiscussionClusterCount", "privateStateExposed"]),
        "liveness": compact_fields(&liveness, &["status", "daemonCount", "nonReadyCount", "privateStateExposed"]),
        "toolDirectory": summarize_tool_directory(&tool_directory),
        "overview": compact_fields(&overview, &["status", "agentCount", "daemonCount", "toolCount", "privateStateExposed", "recommendedWrapperMode", "recommendedWrapperCommand"]),
        "proofBundle": {
            "repoLocalAgentStateSoa": agent_state_report["summaryId"],
            "clusterTopology": topology["schemaVersion"],
            "idunnLiveness": liveness["schemaVersion"],
            "globalToolDirectory": tool_directory["schemaVersion"],
            "gjallarOverview": overview["schemaVersion"],
            "privateStateExposed": false
        },
        "nextSafeMove": "Accept a Persona or Bifrost work item into typed Imagination/Self action pressure; the next missing command is epiphany-work accept --workspace <repo>."
    });
    let receipt_path = artifact_dir.join("repo-swarm-online-receipt.json");
    write_json(&receipt_path, &receipt)?;
    Ok(json!({
        "schemaVersion": "epiphany.repo_swarm_online.v0",
        "status": overview.get("status").cloned().unwrap_or_else(|| json!("ok")),
        "workspace": receipt["workspace"],
        "runtimeId": receipt["runtimeId"],
        "localVerseStore": receipt["localVerseStore"],
        "receiptPath": receipt_path,
        "agentSeed": receipt["agentSeed"],
        "agentState": receipt["agentState"],
        "topology": receipt["topology"],
        "liveness": receipt["liveness"],
        "toolDirectory": receipt["toolDirectory"],
        "overview": receipt["overview"],
        "privateStateExposed": false,
        "nextSafeMove": receipt["nextSafeMove"],
    }))
}

fn ensure_agent_store(agent_store: &Path, template_store: PathBuf) -> Result<Value> {
    if agent_store.exists() {
        return Ok(json!({
            "status": "existing",
            "agentStore": agent_store,
            "templateStore": Value::Null,
            "privateStateExposed": false,
        }));
    }
    if !template_store.exists() {
        return Err(anyhow!(
            "repo-local agent store is absent and template store {} does not exist",
            template_store.display()
        ));
    }
    if let Some(parent) = agent_store.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&template_store, agent_store).with_context(|| {
        format!(
            "failed to seed {} from {}",
            agent_store.display(),
            template_store.display()
        )
    })?;
    Ok(json!({
        "status": "seeded-from-template",
        "agentStore": agent_store,
        "templateStore": template_store,
        "scaffold": "repo-local standing faculty bootstrap; birth accept-init may refine this store after review",
        "privateStateExposed": false,
    }))
}

fn verse_json(
    manifest_path: &Path,
    command: &str,
    store: &Path,
    runtime_id: &str,
    extra: &[String],
) -> Result<Value> {
    let mut args = vec![
        command.to_string(),
        "--store".to_string(),
        store.display().to_string(),
        "--runtime-id".to_string(),
        runtime_id.to_string(),
    ];
    args.extend(extra.iter().cloned());
    cargo_json(manifest_path, "epiphany-verse-query", &args)
        .with_context(|| format!("epiphany-verse-query {command} failed"))
}

fn cargo_json(manifest_path: &Path, bin_name: &str, args: &[String]) -> Result<Value> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--bin")
        .arg(bin_name)
        .arg("--")
        .args(args)
        .output()
        .with_context(|| format!("failed to spawn cargo run --bin {bin_name}"))?;
    if !output.status.success() {
        return Err(anyhow!(
            "cargo run --bin {bin_name} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("{bin_name} returned invalid JSON"))
}

fn compact_fields(value: &Value, fields: &[&str]) -> Value {
    let mut out = serde_json::Map::new();
    for field in fields {
        out.insert(
            (*field).to_string(),
            value.get(*field).cloned().unwrap_or(Value::Null),
        );
    }
    Value::Object(out)
}

fn summarize_tool_directory(value: &Value) -> Value {
    let tools = value
        .get("tools")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let all_agents_tool_count = tools
        .iter()
        .filter(|tool| {
            tool.get("availableToAllAgents")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .count();
    let private_state_exposed = tools.iter().any(|tool| {
        tool.get("privateStateExposed")
            .and_then(Value::as_bool)
            .unwrap_or(true)
    });
    json!({
        "status": value.get("status").cloned().unwrap_or(Value::Null),
        "toolCount": value.get("toolCount").cloned().unwrap_or_else(|| json!(tools.len())),
        "allAgentsToolCount": all_agents_tool_count,
        "privateStateExposed": private_state_exposed,
    })
}

fn path_from_json(value: &Value, path: &[&str]) -> Option<PathBuf> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    cursor.as_str().map(PathBuf::from)
}

fn read_json(path: &Path) -> Result<Value> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("failed to decode {}", path.display()))
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn ensure_git_repo(workspace: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .with_context(|| format!("failed to run git in {}", workspace.display()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!("{} is not a git repository", workspace.display()))
    }
}

fn take_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(take_string(args, name)?))
}

fn take_string(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("missing value for {name}"))
}

fn sanitize(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if sanitized.is_empty() {
        "workspace".to_string()
    } else {
        sanitized
    }
}

fn print_usage() {
    eprintln!(
        "usage: epiphany-swarm online --workspace <repo> [--epiphany-root <path>] [--store <local-verse.ccmp>] [--state-dir <path>] [--artifact-dir <path>] [--runtime-id <id>] [--init-receipt <path>] [--agent-template-store <agents.msgpack>]"
    );
}
