use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StatusRequest {
    thread_id: Option<String>,
    cwd: Option<String>,
    codex_home: Option<String>,
    app_server: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OperatorActionResult {
    action: String,
    artifact_path: String,
    summary: String,
    thread_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactBundle {
    name: String,
    path: String,
    files: Vec<String>,
    summary_path: Option<String>,
    final_status_path: Option<String>,
    comparison_path: Option<String>,
    modified_millis: Option<u128>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OperatorSnapshot {
    generated_at: String,
    repo_root: String,
    status: Value,
    artifacts: Vec<ArtifactBundle>,
}

#[tauri::command]
fn load_operator_snapshot(request: Option<StatusRequest>) -> Result<OperatorSnapshot, String> {
    let repo_root = repo_root()?;
    let status = load_status(&repo_root, request.unwrap_or_default())?;
    let artifacts = list_artifacts(&repo_root)?;
    Ok(OperatorSnapshot {
        generated_at: unix_millis().to_string(),
        repo_root: repo_root.display().to_string(),
        status,
        artifacts,
    })
}

#[tauri::command]
fn run_operator_action(
    action: String,
    request: Option<StatusRequest>,
) -> Result<OperatorActionResult, String> {
    let repo_root = repo_root()?;
    let request = request.unwrap_or_default();
    match action.as_str() {
        "statusSnapshot" => run_status_snapshot(&repo_root, request),
        "coordinatorPlan" => run_coordinator_plan(&repo_root, request),
        "launchModeling"
        | "readModelingResult"
        | "acceptModeling"
        | "launchVerification"
        | "readVerificationResult"
        | "launchReorient"
        | "readReorientResult"
        | "acceptReorient"
        | "prepareCheckpoint" => run_gui_action_bridge(&repo_root, request, action),
        _ => Err(format!("unknown operator action: {action}")),
    }
}

fn load_status(repo_root: &Path, request: StatusRequest) -> Result<Value, String> {
    let python = find_python()?;
    let status_script = repo_root.join("tools").join("epiphany_mvp_status.py");
    let workspace = request
        .cwd
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.to_path_buf());
    let codex_home = request
        .codex_home
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join(".epiphany-gui").join("codex-home"));
    let transcript = repo_root
        .join(".epiphany-gui")
        .join("status-transcript.jsonl");
    let stderr = repo_root
        .join(".epiphany-gui")
        .join("status-server.stderr.log");

    let mut command = Command::new(python);
    command
        .current_dir(repo_root)
        .arg(status_script)
        .arg("--json")
        .arg("--cwd")
        .arg(workspace)
        .arg("--codex-home")
        .arg(codex_home)
        .arg("--transcript")
        .arg(transcript)
        .arg("--stderr")
        .arg(stderr)
        .arg("--no-ephemeral");

    if let Some(thread_id) = request.thread_id {
        command.arg("--thread-id").arg(thread_id);
    }
    if let Some(app_server) = request.app_server {
        command.arg("--app-server").arg(app_server);
    }

    let output = command
        .output()
        .map_err(|err| format!("failed to run Epiphany status bridge: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Epiphany status bridge exited with {}: {}",
            output.status, stderr
        ));
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|err| format!("failed to parse Epiphany status JSON: {err}"))
}

fn run_status_snapshot(
    repo_root: &Path,
    request: StatusRequest,
) -> Result<OperatorActionResult, String> {
    let python = find_python()?;
    let artifact_root = repo_root
        .join(".epiphany-gui")
        .join("status-snapshots")
        .join(unix_millis().to_string());
    fs::create_dir_all(&artifact_root)
        .map_err(|err| format!("failed to create status artifact dir: {err}"))?;
    let result_path = artifact_root.join("status.json");
    let transcript_path = artifact_root.join("transcript.jsonl");
    let stderr_path = artifact_root.join("server.stderr.log");
    let workspace = request
        .cwd
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.to_path_buf());
    let codex_home = request
        .codex_home
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join(".epiphany-gui").join("codex-home"));

    let mut command = Command::new(python);
    command
        .current_dir(repo_root)
        .arg(repo_root.join("tools").join("epiphany_mvp_status.py"))
        .arg("--json")
        .arg("--cwd")
        .arg(workspace)
        .arg("--codex-home")
        .arg(codex_home)
        .arg("--result")
        .arg(&result_path)
        .arg("--transcript")
        .arg(transcript_path)
        .arg("--stderr")
        .arg(stderr_path)
        .arg("--no-ephemeral");
    if let Some(thread_id) = request.thread_id {
        command.arg("--thread-id").arg(thread_id);
    }
    if let Some(app_server) = request.app_server {
        command.arg("--app-server").arg(app_server);
    }
    run_command(command, "status snapshot")?;
    Ok(OperatorActionResult {
        action: "statusSnapshot".to_string(),
        artifact_path: artifact_root.display().to_string(),
        summary: "Status snapshot written.".to_string(),
        thread_id: None,
    })
}

fn run_coordinator_plan(
    repo_root: &Path,
    request: StatusRequest,
) -> Result<OperatorActionResult, String> {
    let python = find_python()?;
    let artifact_dir = repo_root
        .join(".epiphany-dogfood")
        .join(format!("gui-coordinator-plan-{}", unix_millis()));
    let workspace = request
        .cwd
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.to_path_buf());
    let codex_home = request
        .codex_home
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join(".epiphany-gui").join("codex-home"));

    let mut command = Command::new(python);
    command
        .current_dir(repo_root)
        .arg(repo_root.join("tools").join("epiphany_mvp_coordinator.py"))
        .arg("--mode")
        .arg("plan")
        .arg("--max-steps")
        .arg("2")
        .arg("--cwd")
        .arg(workspace)
        .arg("--codex-home")
        .arg(codex_home)
        .arg("--artifact-dir")
        .arg(&artifact_dir);
    if let Some(thread_id) = request.thread_id {
        command.arg("--thread-id").arg(thread_id);
    }
    if let Some(app_server) = request.app_server {
        command.arg("--app-server").arg(app_server);
    }
    run_command(command, "coordinator plan")?;
    Ok(OperatorActionResult {
        action: "coordinatorPlan".to_string(),
        artifact_path: artifact_dir.display().to_string(),
        summary: "Coordinator plan artifact written.".to_string(),
        thread_id: None,
    })
}

fn run_gui_action_bridge(
    repo_root: &Path,
    request: StatusRequest,
    action: String,
) -> Result<OperatorActionResult, String> {
    let thread_id = request.thread_id.clone();
    let python = find_python()?;
    let artifact_root = repo_root.join(".epiphany-gui").join("actions");
    let workspace = request
        .cwd
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.to_path_buf());
    let codex_home = request
        .codex_home
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join(".epiphany-gui").join("codex-home"));

    let mut command = Command::new(python);
    command
        .current_dir(repo_root)
        .arg(repo_root.join("tools").join("epiphany_gui_action.py"))
        .arg("--action")
        .arg(&action)
        .arg("--cwd")
        .arg(workspace)
        .arg("--codex-home")
        .arg(codex_home)
        .arg("--artifact-root")
        .arg(artifact_root);
    if let Some(thread_id) = thread_id {
        command.arg("--thread-id").arg(thread_id);
    }
    if let Some(app_server) = request.app_server {
        command.arg("--app-server").arg(app_server);
    }
    let value = run_json_command(command, &action)?;
    Ok(OperatorActionResult {
        action,
        artifact_path: json_string(&value, "artifactPath")?,
        summary: json_string(&value, "summary")?,
        thread_id: value
            .get("threadId")
            .and_then(Value::as_str)
            .map(ToString::to_string),
    })
}

fn run_command(mut command: Command, label: &str) -> Result<(), String> {
    let output = command
        .output()
        .map_err(|err| format!("failed to run {label}: {err}"))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("{label} exited with {}: {}", output.status, stderr))
}

fn run_json_command(mut command: Command, label: &str) -> Result<Value, String> {
    let output = command
        .output()
        .map_err(|err| format!("failed to run {label}: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{label} exited with {}: {}", output.status, stderr));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|err| format!("failed to parse {label} JSON: {err}"))
}

fn json_string(value: &Value, key: &str) -> Result<String, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| format!("missing string field in GUI action result: {key}"))
}

fn list_artifacts(repo_root: &Path) -> Result<Vec<ArtifactBundle>, String> {
    let mut bundles = Vec::new();
    collect_artifact_root(&mut bundles, &repo_root.join(".epiphany-dogfood"), "")?;
    collect_artifact_root(
        &mut bundles,
        &repo_root.join(".epiphany-gui").join("actions"),
        "actions/",
    )?;
    collect_artifact_root(
        &mut bundles,
        &repo_root.join(".epiphany-gui").join("status-snapshots"),
        "status/",
    )?;

    bundles.sort_by(|a, b| b.modified_millis.cmp(&a.modified_millis));
    Ok(bundles)
}

fn collect_artifact_root(
    bundles: &mut Vec<ArtifactBundle>,
    root: &Path,
    name_prefix: &str,
) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(&root).map_err(|err| format!("failed to read artifacts: {err}"))? {
        let entry = entry.map_err(|err| format!("failed to read artifact entry: {err}"))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        for file in
            fs::read_dir(&path).map_err(|err| format!("failed to read artifact bundle: {err}"))?
        {
            let file = file.map_err(|err| format!("failed to read artifact file: {err}"))?;
            if file.path().is_file() {
                files.push(file.file_name().to_string_lossy().to_string());
            }
        }
        files.sort();
        let raw_name = entry.file_name().to_string_lossy().to_string();
        let modified_millis = entry
            .metadata()
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(system_time_millis);
        bundles.push(ArtifactBundle {
            name: format!("{name_prefix}{raw_name}"),
            path: path.display().to_string(),
            summary_path: existing_path(&path, "epiphany-dogfood-summary.json")
                .or_else(|| existing_path(&path, "gui-action-summary.json"))
                .or_else(|| existing_path(&path, "status.json")),
            final_status_path: existing_path(&path, "epiphany-final-status.json")
                .or_else(|| existing_path(&path, "after-status.json")),
            comparison_path: existing_path(&path, "comparison.md"),
            files,
            modified_millis,
        });
    }

    Ok(())
}

fn existing_path(root: &Path, name: &str) -> Option<String> {
    let path = root.join(name);
    path.exists().then(|| path.display().to_string())
}

fn repo_root() -> Result<PathBuf, String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "failed to derive repository root from CARGO_MANIFEST_DIR".to_string())
}

fn find_python() -> Result<PathBuf, String> {
    if let Ok(value) = std::env::var("EPIPHANY_PYTHON") {
        let path = PathBuf::from(value);
        if path.exists() {
            return Ok(path);
        }
    }
    let bundled = PathBuf::from(
        r"C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe",
    );
    if bundled.exists() {
        return Ok(bundled);
    }
    Ok(PathBuf::from("python"))
}

fn unix_millis() -> u128 {
    system_time_millis(SystemTime::now()).unwrap_or_default()
}

fn system_time_millis(value: SystemTime) -> Option<u128> {
    value
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_millis())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_operator_snapshot,
            run_operator_action
        ])
        .run(tauri::generate_context!())
        .expect("error while running Epiphany operator");
}
