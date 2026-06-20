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
        "run" | "run-queue" | "pulse" => run_swarm(parse_run_args(args)?),
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

#[derive(Clone, Debug)]
struct RunArgs {
    workspace: PathBuf,
    epiphany_root: PathBuf,
    local_verse_store: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    runtime_id: Option<String>,
    online_receipt: Option<PathBuf>,
    until: String,
    max_iterations: u64,
    max_items: u64,
    loop_interval_seconds: u64,
    cooldown_seconds: u64,
    active_timeout_seconds: u64,
    dry_run: bool,
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

fn parse_run_args(args: impl Iterator<Item = String>) -> Result<RunArgs> {
    let mut workspace = None;
    let mut epiphany_root = None;
    let mut local_verse_store = None;
    let mut artifact_dir = None;
    let mut runtime_id = None;
    let mut online_receipt = None;
    let mut until = "blocked-or-published".to_string();
    let mut max_iterations = 8_u64;
    let mut max_items = 1_u64;
    let mut loop_interval_seconds = 0_u64;
    let mut cooldown_seconds = 0_u64;
    let mut active_timeout_seconds = 900_u64;
    let mut dry_run = false;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--epiphany-root" => epiphany_root = Some(take_path(&mut args, "--epiphany-root")?),
            "--local-verse-store" | "--store" => {
                local_verse_store = Some(take_path(&mut args, "--local-verse-store")?);
            }
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--runtime-id" => runtime_id = Some(take_string(&mut args, "--runtime-id")?),
            "--online-receipt" => online_receipt = Some(take_path(&mut args, "--online-receipt")?),
            "--until" => until = take_string(&mut args, "--until")?,
            "--max-iterations" | "--max-pulses" => {
                max_iterations = take_u64(&mut args, "--max-iterations")?;
            }
            "--max-items" => max_items = take_u64(&mut args, "--max-items")?,
            "--loop-interval-seconds" | "--interval-seconds" => {
                loop_interval_seconds = take_u64(&mut args, "--loop-interval-seconds")?;
            }
            "--cooldown-seconds" => cooldown_seconds = take_u64(&mut args, "--cooldown-seconds")?,
            "--active-timeout-seconds" => {
                active_timeout_seconds = take_u64(&mut args, "--active-timeout-seconds")?;
            }
            "--dry-run" | "--no-execute" => dry_run = true,
            other => return Err(anyhow!("unexpected run argument {other:?}")),
        }
    }
    Ok(RunArgs {
        workspace: workspace.context("missing --workspace")?,
        epiphany_root: epiphany_root
            .unwrap_or(env::current_dir().context("failed to resolve current directory")?),
        local_verse_store,
        artifact_dir,
        runtime_id,
        online_receipt,
        until,
        max_iterations,
        max_items,
        loop_interval_seconds,
        cooldown_seconds,
        active_timeout_seconds,
        dry_run,
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

fn run_swarm(args: RunArgs) -> Result<Value> {
    if args.max_iterations == 0 {
        return Err(anyhow!(
            "epiphany-swarm run requires --max-iterations greater than 0"
        ));
    }
    if args.max_items == 0 {
        return Err(anyhow!(
            "epiphany-swarm run requires --max-items greater than 0"
        ));
    }
    if args.until != "blocked-or-published" && args.until != "blocked-or-noop" {
        return Err(anyhow!(
            "unsupported --until {:?}; supported values are blocked-or-published and blocked-or-noop",
            args.until
        ));
    }

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
    let online_receipt_path = args.online_receipt.clone().unwrap_or_else(|| {
        workspace
            .join(".epiphany")
            .join("swarm-online")
            .join("repo-swarm-online-receipt.json")
    });
    let online_receipt = read_json(&online_receipt_path).with_context(
        || "repo swarm online receipt is required; run epiphany-swarm online first",
    )?;
    let local_verse_store = args.local_verse_store.clone().unwrap_or_else(|| {
        path_from_json(&online_receipt, &["localVerseStore"])
            .unwrap_or_else(|| workspace.join(".epiphany").join("local-verse.ccmp"))
    });
    let runtime_id = args.runtime_id.clone().unwrap_or_else(|| {
        string_from_json(&online_receipt, &["runtimeId"])
            .unwrap_or_else(|| "repo-swarm-local".to_string())
    });
    let artifact_dir = args
        .artifact_dir
        .clone()
        .unwrap_or_else(|| workspace.join(".epiphany").join("swarm-run"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;

    let started_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let mut iterations = Vec::new();
    let mut stop_reason = "max-iterations-reached".to_string();
    for pulse_index in 1..=args.max_iterations {
        let queue_run = cargo_json(
            &manifest_path,
            "epiphany-work",
            &queue_run_args(
                &args,
                &workspace,
                &epiphany_root,
                &local_verse_store,
                &runtime_id,
            ),
        )
        .with_context(|| format!("epiphany-work queue-run pulse {pulse_index} failed"))?;
        let queue_status = queue_run
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let actionable_count = queue_run
            .get("actionableCount")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let iteration = json!({
            "pulse": pulse_index,
            "queueRunStatus": queue_status,
            "actionableCount": actionable_count,
            "queueCount": queue_run["queueCount"],
            "selectedRows": queue_run["selectedRows"],
            "outputs": queue_run["outputs"],
            "receiptPath": queue_run["receiptPath"],
            "privateStateExposed": queue_run["privateStateExposed"]
        });
        iterations.push(iteration);
        if queue_status == "blocked-or-noop" || actionable_count == 0 {
            stop_reason = "blocked-or-noop".to_string();
            break;
        }
        if queue_status == "would-advance" && args.dry_run {
            stop_reason = "dry-run-preview-complete".to_string();
            break;
        }
        if pulse_index < args.max_iterations && args.loop_interval_seconds > 0 {
            std::thread::sleep(std::time::Duration::from_secs(args.loop_interval_seconds));
        }
    }
    let completed_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let status = if stop_reason == "max-iterations-reached" {
        "paused-at-iteration-limit"
    } else if stop_reason == "dry-run-preview-complete" {
        "dry-run-preview"
    } else {
        "blocked-or-noop"
    };
    let next_safe_move = match stop_reason.as_str() {
        "blocked-or-noop" => {
            "Inspect repo-work overview blockers, then route planning, closure, publication, or sync through the owning organ."
        }
        "dry-run-preview-complete" => {
            "Rerun epiphany-swarm run without --dry-run to advance the selected branch-local queue row."
        }
        _ => {
            "Rerun epiphany-swarm run only while repo-work queue rows remain tick-actionable and authority gates stay branch-local."
        }
    };
    let receipt_path = artifact_dir.join("repo-swarm-run-receipt.json");
    let receipt = json!({
        "schemaVersion": "epiphany.repo_swarm_run_receipt.v0",
        "createdAt": completed_at,
        "startedAt": started_at,
        "completedAt": completed_at,
        "status": status,
        "stopReason": stop_reason,
        "workspace": workspace,
        "runtimeId": runtime_id,
        "localVerseStore": local_verse_store,
        "onlineReceiptPath": online_receipt_path,
        "until": args.until,
        "scheduler": {
            "owner": "Self",
            "mouth": "epiphany-swarm run",
            "delegatesQueueSelectionTo": "epiphany-work queue-run",
            "delegatesActuationTo": "epiphany-work tick",
            "maxIterations": args.max_iterations,
            "maxItems": args.max_items,
            "loopIntervalSeconds": args.loop_interval_seconds,
            "cooldownSeconds": args.cooldown_seconds,
            "activeTimeoutSeconds": args.active_timeout_seconds,
            "dryRun": args.dry_run
        },
        "iterations": iterations,
        "iterationCount": iterations.len(),
        "authority": {
            "branchLocalOnly": true,
            "publicationAuthorized": false,
            "mergeAuthorized": false,
            "serviceLifecycleAuthorized": false,
            "crossRepoMutationAuthorized": false,
            "privateStateExposed": false
        },
        "nextSafeMove": next_safe_move,
        "privateStateExposed": false
    });
    write_json(&receipt_path, &receipt)?;
    Ok(json!({
        "schemaVersion": "epiphany.repo_swarm_run.v0",
        "status": receipt["status"],
        "stopReason": receipt["stopReason"],
        "workspace": receipt["workspace"],
        "runtimeId": receipt["runtimeId"],
        "localVerseStore": receipt["localVerseStore"],
        "receiptPath": receipt_path,
        "iterationCount": receipt["iterationCount"],
        "iterations": receipt["iterations"],
        "authority": receipt["authority"],
        "privateStateExposed": false,
        "nextSafeMove": receipt["nextSafeMove"],
    }))
}

fn queue_run_args(
    args: &RunArgs,
    workspace: &Path,
    epiphany_root: &Path,
    local_verse_store: &Path,
    runtime_id: &str,
) -> Vec<String> {
    let mut queue_args = vec![
        "queue-run".to_string(),
        "--workspace".to_string(),
        workspace.display().to_string(),
        "--epiphany-root".to_string(),
        epiphany_root.display().to_string(),
        "--local-verse-store".to_string(),
        local_verse_store.display().to_string(),
        "--runtime-id".to_string(),
        runtime_id.to_string(),
        "--max-items".to_string(),
        args.max_items.to_string(),
        "--cooldown-seconds".to_string(),
        args.cooldown_seconds.to_string(),
        "--active-timeout-seconds".to_string(),
        args.active_timeout_seconds.to_string(),
    ];
    if args.dry_run {
        queue_args.push("--dry-run".to_string());
    }
    queue_args
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

fn string_from_json(value: &Value, path: &[&str]) -> Option<String> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    cursor.as_str().map(ToString::to_string)
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

fn take_u64(args: &mut impl Iterator<Item = String>, name: &str) -> Result<u64> {
    let raw = take_string(args, name)?;
    raw.parse::<u64>()
        .with_context(|| format!("invalid integer for {name}: {raw:?}"))
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
        "usage: epiphany-swarm <online|run> ...\n\
         online --workspace <repo> [--epiphany-root <path>] [--store <local-verse.ccmp>] [--state-dir <path>] [--artifact-dir <path>] [--runtime-id <id>] [--init-receipt <path>] [--agent-template-store <agents.msgpack>]\n\
         run --workspace <repo> [--epiphany-root <path>] [--store <local-verse.ccmp>] [--runtime-id <id>] [--until blocked-or-published] [--max-iterations <n>] [--max-items <n>] [--dry-run]"
    );
}
