use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() -> Result<()> {
    let args = Args::parse()?;
    let result = run_smoke(&args)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Debug)]
struct Args {
    artifact_root: PathBuf,
    coordinator_exe: Option<PathBuf>,
}

impl Args {
    fn parse() -> Result<Self> {
        let root = env::current_dir().context("failed to resolve current dir")?;
        let mut parsed = Self {
            artifact_root: root.join(".epiphany-smoke").join("mvp-coordinator"),
            coordinator_exe: None,
        };
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--app-server" => {
                    let _ = take_path(&mut args, "--app-server")?;
                }
                "--artifact-root" => {
                    parsed.artifact_root = take_path(&mut args, "--artifact-root")?;
                }
                "--coordinator-exe" => {
                    parsed.coordinator_exe = Some(take_path(&mut args, "--coordinator-exe")?);
                }
                other => return Err(anyhow!("unknown argument: {other}")),
            }
        }
        Ok(parsed)
    }
}

fn run_smoke(args: &Args) -> Result<Value> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let artifact_root = absolute_path(&args.artifact_root)?;
    reset_artifact_root(&root, &artifact_root)?;
    let coordinator = match &args.coordinator_exe {
        Some(path) => absolute_path(path)?,
        None => sibling_exe("epiphany-mvp-coordinator")?,
    };
    ensure_coordinator_built(&root, &coordinator)?;

    let cold = run_coordinator(
        &root,
        &coordinator,
        &artifact_root,
        RunOptions {
            name: "cold",
            mode: "plan",
            max_steps: 1,
        },
    )?;
    require(
        cold.pointer("/finalAction/action").and_then(Value::as_str) == Some("prepareCheckpoint"),
        "cold start should stop at prepareCheckpoint",
    )?;
    require_artifacts(&cold)?;
    require_operator_safe(&cold, "$")?;

    let legacy_flags = [
        "--bootstrap-smoke-state",
        "--bootstrap-local-state",
        "--bootstrap-objective",
        "--simulate-high-pressure",
        "--simulate-continue-implementation",
        "--simulate-source-drift",
        "--dry-compact",
    ];
    for flag in legacy_flags {
        let rejected = Command::new(&coordinator)
            .current_dir(&root)
            .arg(flag)
            .output()
            .with_context(|| format!("failed to check rejected coordinator flag {flag}"))?;
        require(
            !rejected.status.success()
                && String::from_utf8_lossy(&rejected.stderr).contains("unknown argument"),
            &format!("production coordinator accepted fixture-only flag {flag}"),
        )?;
    }

    let rejected = Command::new(&coordinator)
        .current_dir(&root)
        .arg("--artifact-dir")
        .arg(artifact_root.join("rejected-private-store-completion"))
        .arg("--test-complete-backend")
        .output()
        .context("failed to run rejected completion check")?;
    let rejected_stderr = String::from_utf8_lossy(&rejected.stderr);
    require(
        !rejected.status.success() && rejected_stderr.contains("CultNet job-result API"),
        "native coordinator should reject direct backend-completion mutation",
    )?;

    let result = json!({
        "artifactRoot": artifact_root,
        "coldAction": cold["finalAction"]["action"],
        "rejectedFixtureFlags": legacy_flags,
        "directBackendCompletionRejected": true,
        "note": "The smoke owns only its .epiphany-smoke fixture body. Production coordinator state and Hands authority come from typed runtime evidence.",
    });
    write_json(
        &artifact_root.join("coordinator-smoke-summary.json"),
        &result,
    )?;
    Ok(result)
}

struct RunOptions {
    name: &'static str,
    mode: &'static str,
    max_steps: usize,
}

fn run_coordinator(
    root: &Path,
    coordinator: &Path,
    artifact_root: &Path,
    options: RunOptions,
) -> Result<Value> {
    let artifact_dir = artifact_root.join(options.name);
    let mut command = Command::new(coordinator);
    command
        .current_dir(root)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .arg("--runtime-store")
        .arg(artifact_dir.join("runtime-spine.msgpack"))
        .arg("--codex-home")
        .arg(artifact_dir.join("codex-home"))
        .arg("--cwd")
        .arg(root)
        .arg("--mode")
        .arg(options.mode)
        .arg("--max-steps")
        .arg(options.max_steps.to_string())
        .arg("--poll-seconds")
        .arg("0.1")
        .arg("--timeout-seconds")
        .arg("3");
    let output = command
        .output()
        .context("failed to run native coordinator")?;
    if !output.status.success() {
        return Err(anyhow!(
            "native coordinator failed: {}{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    serde_json::from_slice(&output.stdout).context("failed to decode coordinator summary")
}

fn require_artifacts(summary: &Value) -> Result<()> {
    let artifact_dir = PathBuf::from(
        summary
            .get("artifactDir")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("summary missing artifactDir"))?,
    );
    for name in [
        "coordinator-summary.json",
        "coordinator-steps.jsonl",
        "coordinator-final-status.json",
        "coordinator-final-status.txt",
        "coordinator-final-action.txt",
        "agent-function-telemetry.json",
        "runtime-spine-status.json",
    ] {
        require(
            artifact_dir.join(name).exists(),
            &format!("missing coordinator artifact {name}"),
        )?;
    }
    require(
        artifact_dir.join("runtime-spine.msgpack").exists(),
        "missing native runtime spine store",
    )?;
    let runtime_status: Value = serde_json::from_str(&fs::read_to_string(
        artifact_dir.join("runtime-spine-status.json"),
    )?)?;
    require(
        runtime_status["present"].as_bool() == Some(true),
        "native runtime spine should be present",
    )?;
    require(
        runtime_status["sessions"].as_u64().unwrap_or(0) >= 1,
        "native runtime spine should record a session",
    )?;
    let telemetry: Value = serde_json::from_str(&fs::read_to_string(
        artifact_dir.join("agent-function-telemetry.json"),
    )?)?;
    require(
        telemetry["appServerCalls"].as_u64() == Some(0)
            && telemetry["transport"].as_str() == Some("cultcache"),
        "native coordinator telemetry must prove zero app-server calls",
    )?;
    Ok(())
}

fn require_operator_safe(value: &Value, path: &str) -> Result<()> {
    match value {
        Value::Object(map) => {
            if let Some(raw_result) = map.get("rawResult") {
                require(
                    raw_result.get("sealed").and_then(Value::as_bool) == Some(true),
                    &format!("{path}.rawResult should be sealed in operator-facing artifacts"),
                )?;
            }
            for (key, item) in map {
                require_operator_safe(item, &format!("{path}.{key}"))?;
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                require_operator_safe(item, &format!("{path}[{index}]"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn reset_artifact_root(root: &Path, path: &Path) -> Result<()> {
    let dogfood_root = root.join(".epiphany-smoke").canonicalize().or_else(|_| {
        let dogfood_root = root.join(".epiphany-smoke");
        fs::create_dir_all(&dogfood_root)?;
        dogfood_root.canonicalize()
    })?;
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let resolved = if path.exists() {
        path.canonicalize()?
    } else {
        parent.canonicalize()?.join(path.file_name().unwrap())
    };
    if resolved == dogfood_root || !resolved.starts_with(&dogfood_root) {
        return Err(anyhow!(
            "refusing to delete non-dogfood artifact root: {}",
            path.display()
        ));
    }
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)?;
    Ok(())
}

fn sibling_exe(name: &str) -> Result<PathBuf> {
    let mut exe = env::current_exe().context("failed to resolve current executable")?;
    exe.set_file_name(format!("{name}{}", env::consts::EXE_SUFFIX));
    Ok(exe)
}

fn ensure_coordinator_built(root: &Path, coordinator: &Path) -> Result<()> {
    ensure_epiphany_bin_built(root, coordinator, "epiphany-mvp-coordinator")
}

fn ensure_epiphany_bin_built(root: &Path, binary: &Path, bin_name: &str) -> Result<()> {
    if binary.exists() {
        return Ok(());
    }
    let status = Command::new("cargo")
        .current_dir(root)
        .arg("build")
        .arg("--manifest-path")
        .arg(root.join("epiphany-core").join("Cargo.toml"))
        .arg("--bin")
        .arg(bin_name)
        .status()
        .with_context(|| format!("failed to build {bin_name}"))?;
    require(
        status.success() && binary.exists(),
        &format!("native binary was not built: {}", binary.display()),
    )
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))?;
    Ok(())
}

fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(anyhow!("{message}"))
    }
}

fn take_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("{name} requires a value"))
}
