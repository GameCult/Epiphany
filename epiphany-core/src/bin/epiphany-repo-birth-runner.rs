use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::Utc;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_CODEX_BIN: &str = "codex";
const DEFAULT_TIMEOUT_SECONDS: u64 = 900;

fn main() -> Result<()> {
    let args = Args::parse()?;
    let summary = run(args)?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

#[derive(Clone, Debug)]
struct Args {
    repo: PathBuf,
    baseline: PathBuf,
    artifact_dir: PathBuf,
    init_store: PathBuf,
    agent_store: PathBuf,
    heartbeat_store: PathBuf,
    codex_bin: String,
    mode: String,
    model: Option<String>,
    timeout_seconds: u64,
    auto_accept: bool,
}

impl Args {
    fn parse() -> Result<Self> {
        let root = env::current_dir().context("failed to resolve current dir")?;
        let mut args = env::args().skip(1);
        let mut parsed = Args {
            repo: root.clone(),
            baseline: root
                .join(".epiphany-imports")
                .join("repo-personality-terrain")
                .join("baseline.msgpack"),
            artifact_dir: root.join(".epiphany-birth-runner"),
            init_store: root.join("state").join("repo-initialization.msgpack"),
            agent_store: root.join("state").join("agents.msgpack"),
            heartbeat_store: root.join("state").join("agent-heartbeats.msgpack"),
            codex_bin: DEFAULT_CODEX_BIN.to_string(),
            mode: "plan".to_string(),
            model: None,
            timeout_seconds: DEFAULT_TIMEOUT_SECONDS,
            auto_accept: false,
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--repo" => parsed.repo = take_path(&mut args, "--repo")?,
                "--baseline" => parsed.baseline = take_path(&mut args, "--baseline")?,
                "--artifact-dir" => parsed.artifact_dir = take_path(&mut args, "--artifact-dir")?,
                "--init-store" => parsed.init_store = take_path(&mut args, "--init-store")?,
                "--agent-store" => parsed.agent_store = take_path(&mut args, "--agent-store")?,
                "--heartbeat-store" => {
                    parsed.heartbeat_store = take_path(&mut args, "--heartbeat-store")?;
                }
                "--codex-bin" => parsed.codex_bin = take_string(&mut args, "--codex-bin")?,
                "--mode" => parsed.mode = take_string(&mut args, "--mode")?,
                "--model" => parsed.model = Some(take_string(&mut args, "--model")?),
                "--timeout-seconds" => {
                    parsed.timeout_seconds =
                        take_string(&mut args, "--timeout-seconds")?.parse()?;
                }
                "--auto-accept" => parsed.auto_accept = true,
                other => return Err(anyhow!("unknown argument {other:?}")),
            }
        }
        if !matches!(parsed.mode.as_str(), "plan" | "run") {
            return Err(anyhow!("--mode must be plan or run"));
        }
        Ok(parsed)
    }
}

fn run(args: Args) -> Result<Value> {
    fs::create_dir_all(&args.artifact_dir)
        .with_context(|| format!("failed to create {}", args.artifact_dir.display()))?;
    let startup_dir = args.artifact_dir.join("startup");
    fs::create_dir_all(&startup_dir)?;
    let startup = run_repo_personality_json(&[
        "startup".to_string(),
        "--repo".to_string(),
        args.repo.display().to_string(),
        "--baseline".to_string(),
        args.baseline.display().to_string(),
        "--artifact-dir".to_string(),
        startup_dir.display().to_string(),
        "--init-store".to_string(),
        args.init_store.display().to_string(),
    ])?;
    let packets = startup["generatedPackets"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut executions = Vec::new();
    for packet in packets {
        let kind = packet["kind"].as_str().unwrap_or("unknown");
        let packet_path = path_from_value(&packet["packetPath"])?;
        let execution_dir = args.artifact_dir.join(sanitize(kind));
        fs::create_dir_all(&execution_dir)?;
        let packet_value = read_json(&packet_path)?;
        let prompt_path = execution_dir.join("prompt.md");
        let schema_path = execution_dir.join("output-schema.json");
        let result_path = execution_dir.join("result.json");
        let stdout_path = execution_dir.join("codex.stdout.log");
        let stderr_path = execution_dir.join("codex.stderr.log");
        let prompt = render_specialist_prompt(kind, &packet_value)?;
        let schema = output_schema_for_kind(kind, &packet_value)?;
        write_text(&prompt_path, &prompt)?;
        write_json(&schema_path, &schema)?;
        let mut execution = json!({
            "kind": kind,
            "birthOnly": true,
            "executionOwner": "repo-initialization-startup-runner",
            "heartbeatParticipant": Value::Null,
            "packetPath": packet_path,
            "promptPath": prompt_path,
            "schemaPath": schema_path,
            "resultPath": result_path,
            "stdoutPath": stdout_path,
            "stderrPath": stderr_path,
            "status": "planned",
            "acceptCommand": accept_command(kind, &args, &packet_path, &result_path),
        });
        if args.mode == "run" {
            let run_result = run_codex_specialist(
                &args,
                &prompt,
                &schema_path,
                &result_path,
                &stdout_path,
                &stderr_path,
            )?;
            execution["status"] = run_result["status"].clone();
            execution["exitCode"] = run_result["exitCode"].clone();
            execution["timedOut"] = run_result["timedOut"].clone();
            if result_path.exists() {
                execution["result"] = read_json(&result_path).unwrap_or_else(|err| {
                    json!({
                        "status": "invalid-json",
                        "error": err.to_string(),
                    })
                });
            }
            if args.auto_accept && execution["status"] == "completed" {
                execution["acceptance"] = run_repo_personality_json(&accept_args(
                    kind,
                    &args,
                    &packet_path,
                    &result_path,
                ))?;
            }
        }
        executions.push(execution);
    }
    let summary = json!({
        "schemaVersion": "epiphany.repo_birth_runner.v0",
        "createdAt": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "mode": args.mode,
        "repo": args.repo,
        "baseline": args.baseline,
        "artifactDir": args.artifact_dir,
        "initStore": args.init_store,
        "agentStore": args.agent_store,
        "heartbeatStore": args.heartbeat_store,
        "startup": startup,
        "executions": executions,
        "requiresReview": !args.auto_accept,
        "nextSafeMove": if executions.is_empty() {
            "Birth initialization records are present; continue startup."
        } else if args.mode == "plan" {
            "Review planned startup-only birth executions, then rerun with --mode run."
        } else if args.auto_accept {
            "Review auto-accepted smoke output and continue startup."
        } else {
            "Review result.json for each birth specialist, then run the shown accept-init command."
        }
    });
    write_json(
        &args.artifact_dir.join("birth-runner-summary.json"),
        &summary,
    )?;
    Ok(summary)
}

fn run_codex_specialist(
    args: &Args,
    prompt: &str,
    schema_path: &Path,
    result_path: &Path,
    stdout_path: &Path,
    stderr_path: &Path,
) -> Result<Value> {
    let mut command = Command::new(&args.codex_bin);
    command
        .arg("exec")
        .arg("--cd")
        .arg(&args.repo)
        .arg("--sandbox")
        .arg("read-only")
        .arg("--skip-git-repo-check")
        .arg("--output-schema")
        .arg(schema_path)
        .arg("--output-last-message")
        .arg(result_path);
    if let Some(model) = &args.model {
        command.arg("--model").arg(model);
    }
    command.arg("-");
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().context("failed to spawn codex exec")?;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| anyhow!("failed to open codex stdin"))?
        .write_all(prompt.as_bytes())?;
    let deadline = Instant::now() + Duration::from_secs(args.timeout_seconds);
    loop {
        if let Some(status) = child.try_wait()? {
            let output = child.wait_with_output()?;
            write_bytes(stdout_path, &output.stdout)?;
            write_bytes(stderr_path, &output.stderr)?;
            return Ok(json!({
                "status": if status.success() { "completed" } else { "failed" },
                "exitCode": status.code(),
                "timedOut": false,
            }));
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output()?;
            write_bytes(stdout_path, &output.stdout)?;
            write_bytes(stderr_path, &output.stderr)?;
            return Ok(json!({
                "status": "timeout",
                "exitCode": Value::Null,
                "timedOut": true,
            }));
        }
        thread::sleep(Duration::from_millis(250));
    }
}

fn render_specialist_prompt(kind: &str, packet: &Value) -> Result<String> {
    Ok(format!(
        "{}\n\n# Startup-Only Birth Packet\n\nYou are executing exactly one repo initialization birth specialist packet. Do not edit files. Do not mutate state. Return only JSON that matches the provided schema. The coordinator/Self will review and decide whether to accept the result.\n\n```json\n{}\n```\n",
        packet["prompt"].as_str().unwrap_or(match kind {
            "repo-trajectory" => "Act as the Epiphany Repo Trajectory Distiller.",
            "repo-personality" => "Act as the Epiphany Repo Personality Distiller.",
            "repo-memory" => "Act as the Epiphany Repo Memory Distiller.",
            _ => "Act as a startup-only Epiphany birth specialist.",
        }),
        serde_json::to_string_pretty(packet)?
    ))
}

fn output_schema_for_kind(kind: &str, packet: &Value) -> Result<Value> {
    let expected = packet.get("expectedOutput").cloned().unwrap_or(Value::Null);
    let schema = match kind {
        "repo-trajectory" => json!({
            "type": "object",
            "required": ["verdict", "summary", "confidence", "selfImage", "trajectoryNarrative", "implicitGoals", "antiGoals", "roleBiases", "selfPatchCandidates", "initializationRecord", "doNotMutate", "nextSafeMove"],
            "properties": {
                "verdict": {"type": "string", "enum": ["ready-for-review", "needs-more-history", "reject"]},
                "summary": {"type": "string"},
                "confidence": {"type": "number"},
                "selfImage": {"type": "string"},
                "trajectoryNarrative": {"type": "string"},
                "implicitGoals": {"type": "array", "items": {"type": "string"}},
                "antiGoals": {"type": "array", "items": {"type": "string"}},
                "roleBiases": {"type": "array", "items": {"type": "object"}},
                "selfPatchCandidates": {"type": "array", "items": {"type": "object"}},
                "initializationRecord": {"type": "object"},
                "doNotMutate": {"type": "array", "items": {"type": "string"}},
                "nextSafeMove": {"type": "string"}
            },
            "additionalProperties": true,
            "xEpiphanyExpectedOutput": expected,
        }),
        "repo-personality" => json!({
            "type": "object",
            "required": ["verdict", "summary", "confidence", "roleQuirks", "selfPatchCandidates", "initializationRecord", "doNotMutate", "nextSafeMove"],
            "properties": {
                "verdict": {"type": "string", "enum": ["ready-for-review", "needs-more-terrain", "reject"]},
                "summary": {"type": "string"},
                "confidence": {"type": "number"},
                "roleQuirks": {"type": "array", "items": {"type": "object"}},
                "selfPatchCandidates": {"type": "array", "items": {"type": "object"}},
                "initializationRecord": {"type": "object"},
                "doNotMutate": {"type": "array", "items": {"type": "string"}},
                "nextSafeMove": {"type": "string"}
            },
            "additionalProperties": true,
            "xEpiphanyExpectedOutput": expected,
        }),
        "repo-memory" => json!({
            "type": "object",
            "required": ["verdict", "summary", "confidence", "roleMemoryPatches", "initializationRecord", "doNotMutate", "nextSafeMove"],
            "properties": {
                "verdict": {"type": "string", "enum": ["ready-for-review", "needs-more-terrain", "reject"]},
                "summary": {"type": "string"},
                "confidence": {"type": "number"},
                "roleMemoryPatches": {"type": "array", "items": {"type": "object"}},
                "globalMemoryCandidates": {"type": "array", "items": {"type": "object"}},
                "initializationRecord": {"type": "object"},
                "doNotMutate": {"type": "array", "items": {"type": "string"}},
                "nextSafeMove": {"type": "string"}
            },
            "additionalProperties": true,
            "xEpiphanyExpectedOutput": expected,
        }),
        other => return Err(anyhow!("unsupported birth specialist kind {other:?}")),
    };
    Ok(schema)
}

fn accept_command(kind: &str, args: &Args, packet_path: &Path, result_path: &Path) -> Vec<String> {
    accept_args(kind, args, packet_path, result_path)
}

fn accept_args(kind: &str, args: &Args, packet_path: &Path, result_path: &Path) -> Vec<String> {
    let mut out = vec![
        "accept-init".to_string(),
        "--init-store".to_string(),
        args.init_store.display().to_string(),
        "--packet".to_string(),
        packet_path.display().to_string(),
        "--kind".to_string(),
        kind.to_string(),
        "--accepted-by".to_string(),
        "Self".to_string(),
        "--summary".to_string(),
        format!("Accepted {kind} birth specialist result after review."),
        "--result".to_string(),
        result_path.display().to_string(),
        "--agent-store".to_string(),
        args.agent_store.display().to_string(),
        "--apply-self-patches".to_string(),
        "true".to_string(),
    ];
    if kind == "repo-personality" {
        out.extend([
            "--heartbeat-store".to_string(),
            args.heartbeat_store.display().to_string(),
            "--apply-heartbeat-seeds".to_string(),
            "true".to_string(),
        ]);
    }
    out
}

fn run_repo_personality_json(args: &[String]) -> Result<Value> {
    let exe = native_exe("epiphany-repo-personality");
    let output = Command::new(&exe)
        .args(args)
        .output()
        .with_context(|| format!("failed to run {}", exe.display()))?;
    if !output.status.success() {
        return Err(anyhow!(
            "epiphany-repo-personality failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("epiphany-repo-personality returned invalid JSON"))
}

fn native_exe(name: &str) -> PathBuf {
    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .unwrap_or_else(|| r"C:\Users\Meta\.cargo-target-codex".into());
    PathBuf::from(target_dir)
        .join("debug")
        .join(format!("{name}{}", env::consts::EXE_SUFFIX))
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

fn write_text(path: &Path, value: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, value).with_context(|| format!("failed to write {}", path.display()))
}

fn write_bytes(path: &Path, value: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, value).with_context(|| format!("failed to write {}", path.display()))
}

fn path_from_value(value: &Value) -> Result<PathBuf> {
    value
        .as_str()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("expected path string, got {value}"))
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect()
}

fn take_string(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("{name} requires a value"))
}

fn take_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(take_string(args, name)?))
}
