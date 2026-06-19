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
        "init" => run_init(parse_init_args(args)?),
        other => Err(anyhow!("unknown epiphany-repo command {other:?}")),
    }?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Clone, Debug)]
struct InitArgs {
    workspace: PathBuf,
    epiphany_root: PathBuf,
    artifact_dir: Option<PathBuf>,
    state_dir: Option<PathBuf>,
    swarm_id: Option<String>,
    topic: Option<String>,
    mode: String,
    model: Option<String>,
    auto_accept: bool,
    create_branch: bool,
    switch_branch: bool,
}

fn parse_init_args(args: impl Iterator<Item = String>) -> Result<InitArgs> {
    let mut workspace = None;
    let mut epiphany_root = None;
    let mut artifact_dir = None;
    let mut state_dir = None;
    let mut swarm_id = None;
    let mut topic = None;
    let mut mode = "plan".to_string();
    let mut model = None;
    let mut auto_accept = false;
    let mut create_branch = false;
    let mut switch_branch = false;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--epiphany-root" => epiphany_root = Some(take_path(&mut args, "--epiphany-root")?),
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--state-dir" => state_dir = Some(take_path(&mut args, "--state-dir")?),
            "--swarm-id" => swarm_id = Some(take_string(&mut args, "--swarm-id")?),
            "--topic" => topic = Some(take_string(&mut args, "--topic")?),
            "--mode" => mode = take_string(&mut args, "--mode")?,
            "--model" => model = Some(take_string(&mut args, "--model")?),
            "--auto-accept" => auto_accept = true,
            "--create-branch" => create_branch = true,
            "--switch-branch" => {
                create_branch = true;
                switch_branch = true;
            }
            other => return Err(anyhow!("unexpected init argument {other:?}")),
        }
    }
    if !matches!(mode.as_str(), "plan" | "run") {
        return Err(anyhow!("--mode must be plan or run"));
    }
    if auto_accept && mode != "run" {
        return Err(anyhow!("--auto-accept requires --mode run"));
    }
    let workspace = workspace.context("missing --workspace")?;
    let epiphany_root =
        epiphany_root.unwrap_or(env::current_dir().context("failed to resolve current directory")?);
    Ok(InitArgs {
        workspace,
        epiphany_root,
        artifact_dir,
        state_dir,
        swarm_id,
        topic,
        mode,
        model,
        auto_accept,
        create_branch,
        switch_branch,
    })
}

fn run_init(args: InitArgs) -> Result<Value> {
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

    let workspace_name = workspace
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("repo");
    let swarm_id = args
        .swarm_id
        .unwrap_or_else(|| sanitize(workspace_name).max_with_default("repo-swarm"));
    let topic = args
        .topic
        .unwrap_or_else(|| "first-awakening".to_string())
        .pipe(|value| sanitize(&value).max_with_default("first-awakening"));
    let branch_name = format!("epiphany/{swarm_id}/{topic}");
    let artifact_dir = args
        .artifact_dir
        .unwrap_or_else(|| workspace.join(".epiphany").join("repo-init"));
    let state_dir = args
        .state_dir
        .unwrap_or_else(|| workspace.join(".epiphany").join("state"));
    let baseline = state_dir.join("repo-personality-baseline.msgpack");
    let init_store = state_dir.join("repo-initialization.msgpack");
    let agent_store = state_dir.join("agents.msgpack");
    let heartbeat_store = state_dir.join("agent-heartbeats.msgpack");
    let runtime_store = state_dir.join("runtime-spine.msgpack");
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    fs::create_dir_all(&state_dir)
        .with_context(|| format!("failed to create {}", state_dir.display()))?;

    let current_branch = git_output(&workspace, &["branch", "--show-current"])
        .unwrap_or_else(|_| "unknown".to_string());
    let branch_receipt = prepare_branch(
        &workspace,
        &branch_name,
        args.create_branch,
        args.switch_branch,
    )?;
    let birth_summary = run_birth_runner(BirthRunnerArgs {
        manifest_path: &manifest_path,
        repo: &workspace,
        baseline: &baseline,
        artifact_dir: &artifact_dir.join("birth"),
        init_store: &init_store,
        agent_store: &agent_store,
        heartbeat_store: &heartbeat_store,
        runtime_store: &runtime_store,
        mode: &args.mode,
        model: args.model.as_deref(),
        auto_accept: args.auto_accept,
    })?;
    let receipt = json!({
        "schemaVersion": "epiphany.repo_swarm_init_receipt.v0",
        "createdAt": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "workspace": workspace,
        "epiphanyRoot": epiphany_root,
        "swarmId": swarm_id,
        "topic": topic,
        "branch": {
            "name": branch_name,
            "previousBranch": current_branch,
            "receipt": branch_receipt,
        },
        "stores": {
            "stateDir": state_dir,
            "baseline": baseline,
            "initStore": init_store,
            "agentStore": agent_store,
            "heartbeatStore": heartbeat_store,
            "runtimeStore": runtime_store,
        },
        "artifactDir": artifact_dir,
        "birth": birth_summary,
        "autonomyContract": {
            "standingAuthority": "branch-local work inside the owned repo Body",
            "publicationGate": "Bifrost",
            "privateState": "sealed",
            "humanRequiredBefore": ["upstream publication", "merge", "authority escalation"],
        },
        "nextSafeMove": if args.mode == "plan" {
            "Review birth packet prompts and rerun init with --mode run when Self is ready to execute startup-only distillers."
        } else if args.auto_accept {
            "Inspect auto-accepted birth receipts, then bring the repo swarm online."
        } else {
            "Review birth specialist results and run the emitted accept-init commands before epiphany-swarm online."
        }
    });
    let receipt_path = artifact_dir.join("repo-swarm-init-receipt.json");
    write_json(&receipt_path, &receipt)?;
    Ok(json!({
        "schemaVersion": "epiphany.repo_init.v0",
        "mode": "init",
        "workspace": receipt["workspace"],
        "branch": receipt["branch"],
        "stores": receipt["stores"],
        "artifactDir": receipt["artifactDir"],
        "receiptPath": receipt_path,
        "birth": {
            "schemaVersion": birth_summary["schemaVersion"],
            "mode": birth_summary["mode"],
            "executionCount": birth_summary["executions"].as_array().map_or(0, Vec::len),
            "requiresReview": birth_summary["requiresReview"],
            "nextSafeMove": birth_summary["nextSafeMove"],
        },
        "nextSafeMove": receipt["nextSafeMove"],
    }))
}

struct BirthRunnerArgs<'a> {
    manifest_path: &'a Path,
    repo: &'a Path,
    baseline: &'a Path,
    artifact_dir: &'a Path,
    init_store: &'a Path,
    agent_store: &'a Path,
    heartbeat_store: &'a Path,
    runtime_store: &'a Path,
    mode: &'a str,
    model: Option<&'a str>,
    auto_accept: bool,
}

fn run_birth_runner(args: BirthRunnerArgs<'_>) -> Result<Value> {
    cargo_build_support_bin(args.manifest_path, "epiphany-repo-personality")?;
    let mut command = Command::new("cargo");
    command
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(args.manifest_path)
        .arg("--bin")
        .arg("epiphany-repo-birth-runner")
        .arg("--")
        .arg("--repo")
        .arg(args.repo)
        .arg("--baseline")
        .arg(args.baseline)
        .arg("--artifact-dir")
        .arg(args.artifact_dir)
        .arg("--init-store")
        .arg(args.init_store)
        .arg("--agent-store")
        .arg(args.agent_store)
        .arg("--heartbeat-store")
        .arg(args.heartbeat_store)
        .arg("--runtime-store")
        .arg(args.runtime_store)
        .arg("--mode")
        .arg(args.mode);
    if let Some(model) = args.model {
        command.arg("--model").arg(model);
    }
    if args.auto_accept {
        command.arg("--auto-accept");
    }
    let output = command
        .output()
        .context("failed to spawn epiphany-repo-birth-runner through cargo")?;
    if !output.status.success() {
        return Err(anyhow!(
            "epiphany-repo-birth-runner failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout)
        .context("epiphany-repo-birth-runner returned invalid JSON")
}

fn cargo_build_support_bin(manifest_path: &Path, bin_name: &str) -> Result<()> {
    let output = Command::new("cargo")
        .arg("build")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--bin")
        .arg(bin_name)
        .output()
        .with_context(|| format!("failed to spawn cargo build --bin {bin_name}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "cargo build --bin {bin_name} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn prepare_branch(
    workspace: &Path,
    branch_name: &str,
    create_branch: bool,
    switch_branch: bool,
) -> Result<Value> {
    let exists = git_output(workspace, &["rev-parse", "--verify", branch_name]).is_ok();
    if !create_branch {
        return Ok(json!({
            "schemaVersion": "epiphany.repo_branch_receipt.v0",
            "status": "planned",
            "branchName": branch_name,
            "exists": exists,
            "created": false,
            "switched": false,
            "createCommand": format!("git -C {} switch -c {}", workspace.display(), branch_name),
        }));
    }
    if exists {
        if switch_branch {
            git_ok(workspace, &["switch", branch_name])?;
        }
        return Ok(json!({
            "schemaVersion": "epiphany.repo_branch_receipt.v0",
            "status": if switch_branch { "selected-existing" } else { "exists" },
            "branchName": branch_name,
            "exists": true,
            "created": false,
            "switched": switch_branch,
        }));
    }
    let mut args = vec!["branch", branch_name];
    if switch_branch {
        args = vec!["switch", "-c", branch_name];
    }
    git_ok(workspace, &args)?;
    Ok(json!({
        "schemaVersion": "epiphany.repo_branch_receipt.v0",
        "status": if switch_branch { "created-and-selected" } else { "created" },
        "branchName": branch_name,
        "exists": false,
        "created": true,
        "switched": switch_branch,
    }))
}

fn ensure_git_repo(workspace: &Path) -> Result<()> {
    git_ok(workspace, &["rev-parse", "--show-toplevel"])
        .with_context(|| format!("{} is not a git repository", workspace.display()))
}

fn git_ok(workspace: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn git_output(workspace: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        return Err(anyhow!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn take_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(take_string(args, name)?))
}

fn take_string(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("missing value for {name}"))
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn sanitize(value: &str) -> String {
    value
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
        .join("-")
}

trait NonEmptyDefault {
    fn max_with_default(self, fallback: &str) -> String;
}

impl NonEmptyDefault for String {
    fn max_with_default(self, fallback: &str) -> String {
        if self.is_empty() {
            fallback.to_string()
        } else {
            self
        }
    }
}

trait Pipe: Sized {
    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}

impl<T> Pipe for T {}

fn print_usage() {
    eprintln!(
        "usage: epiphany-repo init --workspace <repo> [--epiphany-root <path>] [--artifact-dir <path>] [--state-dir <path>] [--swarm-id <id>] [--topic <topic>] [--mode plan|run] [--model <model>] [--auto-accept] [--create-branch|--switch-branch]"
    );
}
