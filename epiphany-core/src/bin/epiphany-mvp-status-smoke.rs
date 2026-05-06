use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_APP_SERVER: &str = r"C:\Users\Meta\.cargo-target-codex\debug\codex-app-server.exe";

fn main() -> Result<()> {
    let args = Args::parse()?;
    let result = run_smoke(&args)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Debug)]
struct Args {
    app_server: PathBuf,
    codex_home: PathBuf,
    result: PathBuf,
    transcript: PathBuf,
    stderr: PathBuf,
    rendered: PathBuf,
    status_exe: Option<PathBuf>,
}

impl Args {
    fn parse() -> Result<Self> {
        let root = env::current_dir().context("failed to resolve current dir")?;
        let mut parsed = Self {
            app_server: PathBuf::from(DEFAULT_APP_SERVER),
            codex_home: root.join(".epiphany-smoke").join("mvp-status-codex-home"),
            result: root
                .join(".epiphany-smoke")
                .join("mvp-status-smoke-result.json"),
            transcript: root
                .join(".epiphany-smoke")
                .join("mvp-status-smoke-transcript.jsonl"),
            stderr: root
                .join(".epiphany-smoke")
                .join("mvp-status-smoke-server.stderr.log"),
            rendered: root
                .join(".epiphany-smoke")
                .join("mvp-status-smoke-rendered.txt"),
            status_exe: None,
        };
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--app-server" => parsed.app_server = take_path(&mut args, "--app-server")?,
                "--codex-home" => parsed.codex_home = take_path(&mut args, "--codex-home")?,
                "--result" => parsed.result = take_path(&mut args, "--result")?,
                "--transcript" => parsed.transcript = take_path(&mut args, "--transcript")?,
                "--stderr" => parsed.stderr = take_path(&mut args, "--stderr")?,
                "--rendered" => parsed.rendered = take_path(&mut args, "--rendered")?,
                "--status-exe" => parsed.status_exe = Some(take_path(&mut args, "--status-exe")?),
                other => return Err(anyhow!("unknown argument: {other}")),
            }
        }
        Ok(parsed)
    }
}

fn run_smoke(args: &Args) -> Result<Value> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let app_server = absolute_path(&args.app_server)?;
    if !app_server.exists() {
        return Err(anyhow!(
            "codex app-server binary not found: {}",
            app_server.display()
        ));
    }
    let codex_home = absolute_path(&args.codex_home)?;
    let result_path = absolute_path(&args.result)?;
    let transcript_path = absolute_path(&args.transcript)?;
    let stderr_path = absolute_path(&args.stderr)?;
    let rendered_path = absolute_path(&args.rendered)?;
    reset_smoke_paths(
        &root,
        &[
            codex_home.clone(),
            result_path.clone(),
            transcript_path.clone(),
            stderr_path.clone(),
            rendered_path.clone(),
        ],
    )?;
    let status_exe = match &args.status_exe {
        Some(path) => absolute_path(path)?,
        None => sibling_exe("epiphany-mvp-status")?,
    };
    ensure_status_built(&root, &status_exe)?;

    let status = run_status_json(
        &root,
        &status_exe,
        &app_server,
        &codex_home,
        &result_path,
        &transcript_path,
        &stderr_path,
    )?;
    let rendered = run_status_rendered(
        &root,
        &status_exe,
        &app_server,
        &codex_home,
        &transcript_path.with_file_name("mvp-status-smoke-render-transcript.jsonl"),
        &stderr_path.with_file_name("mvp-status-smoke-render-server.stderr.log"),
    )?;
    if let Some(parent) = rendered_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&rendered_path, &rendered)?;

    validate_status(&status, &rendered)?;
    let result = json!({
        "threadId": status["threadId"],
        "recommendation": status["crrc"]["recommendation"],
        "roles": status["roles"],
        "roleResults": status["roleResults"],
        "planning": status["planning"],
        "heartbeat": status["heartbeat"],
        "face": status["face"],
        "stateStatus": status["scene"]["scene"]["stateStatus"],
        "availableActions": status["scene"]["scene"]["availableActions"],
        "rendered": rendered,
    });
    write_json(&result_path, &result)?;
    Ok(result)
}

fn run_status_json(
    root: &Path,
    status_exe: &Path,
    app_server: &Path,
    codex_home: &Path,
    result_path: &Path,
    transcript_path: &Path,
    stderr_path: &Path,
) -> Result<Value> {
    let output = Command::new(status_exe)
        .current_dir(root)
        .arg("--app-server")
        .arg(app_server)
        .arg("--codex-home")
        .arg(codex_home)
        .arg("--cwd")
        .arg(root)
        .arg("--transcript")
        .arg(transcript_path)
        .arg("--stderr")
        .arg(stderr_path)
        .arg("--result")
        .arg(result_path)
        .arg("--json")
        .output()
        .context("failed to run native status JSON command")?;
    if !output.status.success() {
        return Err(anyhow!(
            "native status JSON command failed: {}{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    serde_json::from_slice(&output.stdout).context("failed to decode native status JSON")
}

fn run_status_rendered(
    root: &Path,
    status_exe: &Path,
    app_server: &Path,
    codex_home: &Path,
    transcript_path: &Path,
    stderr_path: &Path,
) -> Result<String> {
    let output = Command::new(status_exe)
        .current_dir(root)
        .arg("--app-server")
        .arg(app_server)
        .arg("--codex-home")
        .arg(codex_home)
        .arg("--cwd")
        .arg(root)
        .arg("--transcript")
        .arg(transcript_path)
        .arg("--stderr")
        .arg(stderr_path)
        .output()
        .context("failed to run native status rendered command")?;
    if !output.status.success() {
        return Err(anyhow!(
            "native status rendered command failed: {}{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    String::from_utf8(output.stdout).context("rendered status was not UTF-8")
}

fn validate_status(status: &Value, rendered: &str) -> Result<()> {
    require(
        status
            .pointer("/scene/scene/stateStatus")
            .and_then(Value::as_str)
            == Some("missing"),
        "fresh status smoke should honestly report missing Epiphany state",
    )?;
    require(
        status
            .pointer("/crrc/recommendation/action")
            .and_then(Value::as_str)
            == Some("regatherManually"),
        "fresh status smoke should recommend manual regather without state",
    )?;
    require(
        array_contains(status.pointer("/scene/scene/availableActions"), "crrc"),
        "status view should expose the CRRC action in the scene",
    )?;
    require(
        array_contains(status.pointer("/scene/scene/availableActions"), "roles"),
        "status view should expose the role ownership action in the scene",
    )?;
    require(
        rendered.contains("Epiphany MVP Status") && rendered.contains("Recommendation"),
        "rendered status should include the operator view headings",
    )?;
    require(
        rendered.contains("regatherManually"),
        "rendered status should expose the CRRC recommendation",
    )?;
    let lane_ids = status
        .pointer("/roles/roles")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("roles surface missing roles"))?
        .iter()
        .filter_map(|lane| lane.get("id").and_then(Value::as_str))
        .collect::<Vec<_>>();
    require(
        lane_ids
            == [
                "implementation",
                "imagination",
                "modeling",
                "verification",
                "reorientation",
            ],
        "roles surface should expose the five MVP role lanes",
    )?;
    require(
        status
            .pointer("/roles/note")
            .and_then(Value::as_str)
            .is_some_and(|note| note.starts_with("Role ownership is derived read-only")),
        "roles surface should declare read-only derived ownership",
    )?;
    require(
        status
            .pointer("/planning/stateStatus")
            .and_then(Value::as_str)
            == Some("missing")
            && status
                .pointer("/planning/summary/captureCount")
                .and_then(Value::as_u64)
                == Some(0),
        "fresh status smoke should expose empty planning state honestly",
    )?;
    require(
        rendered.contains("Planning") && rendered.contains("captures: 0"),
        "rendered status should expose the planning section",
    )?;
    require(
        rendered.contains("Role Lanes") && rendered.contains("Verification / Review"),
        "rendered status should expose the role lane section",
    )?;
    require(
        rendered.contains("Role Findings")
            && status
                .pointer("/roleResults/imagination/status")
                .and_then(Value::as_str)
                == Some("missingState")
            && status
                .pointer("/roleResults/modeling/status")
                .and_then(Value::as_str)
                == Some("missingState")
            && status
                .pointer("/roleResults/verification/status")
                .and_then(Value::as_str)
                == Some("missingState"),
        "rendered status should expose fixed role result read-back status",
    )?;
    require(
        status
            .pointer("/heartbeat/schema_version")
            .and_then(Value::as_str)
            == Some("epiphany.agent_heartbeat_status.v0"),
        "status view should expose heartbeat initiative status for Aquarium",
    )?;
    let face_actions = status
        .pointer("/face/availableActions")
        .and_then(Value::as_array);
    require(
        face_actions.is_some_and(|items| {
            items.len() == 1 && items.first().and_then(Value::as_str) == Some("faceBubble")
        }),
        "status view should expose Face bubble action for Aquarium",
    )?;
    require(
        rendered.contains("Heartbeat") && rendered.contains("Face"),
        "rendered status should expose heartbeat and Face sections",
    )?;
    Ok(())
}

fn reset_smoke_paths(root: &Path, paths: &[PathBuf]) -> Result<()> {
    let smoke_root = root.join(".epiphany-smoke").canonicalize().or_else(|_| {
        let smoke_root = root.join(".epiphany-smoke");
        fs::create_dir_all(&smoke_root)?;
        smoke_root.canonicalize()
    })?;
    for path in paths {
        if !path.exists() {
            continue;
        }
        let resolved = path.canonicalize()?;
        if resolved == smoke_root || !resolved.starts_with(&smoke_root) {
            return Err(anyhow!(
                "refusing to delete non-smoke path: {}",
                path.display()
            ));
        }
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn ensure_status_built(root: &Path, status_exe: &Path) -> Result<()> {
    if status_exe.exists() {
        return Ok(());
    }
    let status = Command::new("cargo")
        .current_dir(root)
        .arg("build")
        .arg("--manifest-path")
        .arg(root.join("epiphany-core").join("Cargo.toml"))
        .arg("--bin")
        .arg("epiphany-mvp-status")
        .status()
        .context("failed to build epiphany-mvp-status")?;
    require(
        status.success() && status_exe.exists(),
        &format!(
            "native status binary was not built: {}",
            status_exe.display()
        ),
    )
}

fn sibling_exe(name: &str) -> Result<PathBuf> {
    let mut exe = env::current_exe().context("failed to resolve current executable")?;
    exe.set_file_name(format!("{name}{}", env::consts::EXE_SUFFIX));
    Ok(exe)
}

fn array_contains(value: Option<&Value>, needle: &str) -> bool {
    value
        .and_then(Value::as_array)
        .is_some_and(|items| items.iter().any(|item| item.as_str() == Some(needle)))
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
