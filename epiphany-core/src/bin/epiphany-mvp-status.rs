use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_APP_SERVER: &str = r"C:\Users\Meta\.cargo-target-codex\debug\codex-app-server.exe";
const DEFAULT_CARGO_TARGET_DIR: &str = r"C:\Users\Meta\.cargo-target-codex";
const SEALED_DIRECT_THOUGHT_KEYS: &[&str] = &[
    "rawResult",
    "turns",
    "items",
    "inputTranscript",
    "activeTranscript",
];

fn main() -> Result<()> {
    let args = Args::parse()?;
    let status = run_status(&args)?;
    if let Some(result) = &args.result {
        if let Some(parent) = result.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(
            result,
            format!("{}\n", serde_json::to_string_pretty(&status)?),
        )
        .with_context(|| format!("failed to write {}", result.display()))?;
        write_transcript_telemetry(&args.transcript, &result.with_extension("telemetry.json"))?;
    }
    write_transcript_telemetry(
        &args.transcript,
        &args.transcript.with_extension("telemetry.json"),
    )?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        print!("{}", render_status(&status));
    }
    Ok(())
}

#[derive(Debug)]
struct Args {
    app_server: PathBuf,
    codex_home: PathBuf,
    thread_id: Option<String>,
    cwd: PathBuf,
    ephemeral: bool,
    json: bool,
    result: Option<PathBuf>,
    transcript: PathBuf,
    stderr: PathBuf,
}

impl Args {
    fn parse() -> Result<Self> {
        let root = env::current_dir().context("failed to resolve current directory")?;
        let mut args = env::args().skip(1);
        let mut parsed = Args {
            app_server: PathBuf::from(DEFAULT_APP_SERVER),
            codex_home: env::var_os("CODEX_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir().join(".codex")),
            thread_id: None,
            cwd: root.clone(),
            ephemeral: true,
            json: false,
            result: None,
            transcript: root
                .join(".epiphany-status")
                .join("epiphany-mvp-status-transcript.jsonl"),
            stderr: root
                .join(".epiphany-status")
                .join("epiphany-mvp-status-server.stderr.log"),
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--app-server" => parsed.app_server = take_path(&mut args, "--app-server")?,
                "--codex-home" => parsed.codex_home = take_path(&mut args, "--codex-home")?,
                "--thread-id" => parsed.thread_id = Some(take_string(&mut args, "--thread-id")?),
                "--cwd" => parsed.cwd = take_path(&mut args, "--cwd")?,
                "--ephemeral" => parsed.ephemeral = true,
                "--no-ephemeral" => parsed.ephemeral = false,
                "--json" => parsed.json = true,
                "--result" => parsed.result = Some(take_path(&mut args, "--result")?),
                "--transcript" => parsed.transcript = take_path(&mut args, "--transcript")?,
                "--stderr" => parsed.stderr = take_path(&mut args, "--stderr")?,
                _ => return Err(anyhow!("unknown argument: {arg}")),
            }
        }
        Ok(parsed)
    }
}

fn run_status(args: &Args) -> Result<Value> {
    let app_server = absolute_path(&args.app_server)?;
    let codex_home = absolute_path(&args.codex_home)?;
    let cwd = absolute_path(&args.cwd)?;
    let transcript = absolute_path(&args.transcript)?;
    let stderr = absolute_path(&args.stderr)?;

    if !app_server.exists() {
        return Err(anyhow!(
            "codex app-server binary not found: {}",
            app_server.display()
        ));
    }
    fs::create_dir_all(&codex_home)
        .with_context(|| format!("failed to create {}", codex_home.display()))?;
    let mut client = AppServerClient::start(&app_server, &codex_home, &transcript, &stderr)?;
    client.send(
        "initialize",
        Some(json!({
            "clientInfo": {
                "name": "epiphany-mvp-status",
                "title": "Epiphany MVP Status",
                "version": "0.1.0",
            },
            "capabilities": {"experimentalApi": true},
        })),
        true,
    )?;
    client.send("initialized", None, false)?;

    let thread_id = if let Some(thread_id) = &args.thread_id {
        if !args.ephemeral {
            client.send("thread/resume", Some(json!({"threadId": thread_id})), true)?;
        }
        thread_id.clone()
    } else {
        let started = client.send(
            "thread/start",
            Some(json!({"cwd": cwd, "ephemeral": args.ephemeral})),
            true,
        )?;
        started["thread"]["id"]
            .as_str()
            .ok_or_else(|| anyhow!("thread/start returned no thread id"))?
            .to_string()
    };

    let read = client.send(
        "thread/read",
        Some(json!({"threadId": thread_id, "includeTurns": false})),
        true,
    )?;
    let scene = client.send(
        "thread/epiphany/scene",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let pressure = client.send(
        "thread/epiphany/pressure",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let reorient = client.send(
        "thread/epiphany/reorient",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let jobs = client.send(
        "thread/epiphany/jobs",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let roles = client.send(
        "thread/epiphany/roles",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let planning = client.send(
        "thread/epiphany/planning",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let role_results = json!({
        "imagination": client.send("thread/epiphany/roleResult", Some(json!({"threadId": thread_id, "roleId": "imagination"})), true)?,
        "modeling": client.send("thread/epiphany/roleResult", Some(json!({"threadId": thread_id, "roleId": "modeling"})), true)?,
        "verification": client.send("thread/epiphany/roleResult", Some(json!({"threadId": thread_id, "roleId": "verification"})), true)?,
    });
    let reorient_result = client.send(
        "thread/epiphany/reorientResult",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let crrc = client.send(
        "thread/epiphany/crrc",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let coordinator = client.send(
        "thread/epiphany/coordinator",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let root = env::current_dir().context("failed to resolve current directory")?;
    let heartbeat_dir = root.join(".epiphany-heartbeats");
    let face_dir = root.join(".epiphany-face");
    let heartbeat = native_json(
        "epiphany-heartbeat-store",
        &[
            "status",
            "--store",
            "state/agent-heartbeats.msgpack",
            "--artifact-dir",
            &heartbeat_dir.to_string_lossy(),
            "--limit",
            "8",
        ],
    )?;
    let latest_face = native_json(
        "epiphany-face-discord",
        &[
            "latest",
            "--artifact-dir",
            &face_dir.to_string_lossy(),
            "--limit",
            "8",
        ],
    )
    .unwrap_or_else(
        |error| json!({"status": "error", "error": error.to_string(), "latestArtifacts": []}),
    );
    let face = json!({
        "status": "ready",
        "artifactDir": face_dir,
        "latestArtifacts": latest_face.get("latestArtifacts").cloned().unwrap_or_else(|| json!([])),
        "availableActions": ["faceBubble", "characterTurn", "discordPersonaPost"],
    });
    let void_memory = native_json(
        "epiphany-void-memory",
        &["status", "--config", "state/void-memory.toml"],
    )
    .unwrap_or_else(|error| json!({"ok": false, "error": error.to_string()}));
    let status = json!({
        "threadId": thread_id,
        "read": read,
        "scene": scene,
        "pressure": pressure,
        "reorient": reorient,
        "jobs": jobs,
        "roles": roles,
        "planning": planning,
        "roleResults": role_results,
        "reorientResult": reorient_result,
        "crrc": crrc,
        "coordinator": coordinator,
        "heartbeat": heartbeat,
        "face": face,
        "voidMemory": void_memory,
    });
    Ok(sanitize_for_operator(status))
}

pub struct AppServerClient {
    child: Child,
    stdin: ChildStdin,
    rx: mpsc::Receiver<Value>,
    transcript: Arc<Mutex<File>>,
    notifications: Arc<Mutex<Vec<Value>>>,
    next_id: u64,
}

impl AppServerClient {
    pub fn start(
        app_server: &Path,
        codex_home: &Path,
        transcript_path: &Path,
        stderr_path: &Path,
    ) -> Result<Self> {
        if let Some(parent) = transcript_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        if let Some(parent) = stderr_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let transcript = Arc::new(Mutex::new(
            File::create(transcript_path)
                .with_context(|| format!("failed to create {}", transcript_path.display()))?,
        ));
        let stderr_file = Arc::new(Mutex::new(
            File::create(stderr_path)
                .with_context(|| format!("failed to create {}", stderr_path.display()))?,
        ));
        let mut command = Command::new(app_server);
        command
            .current_dir(
                env::current_dir()?
                    .join("vendor")
                    .join("codex")
                    .join("codex-rs"),
            )
            .env("CODEX_HOME", codex_home)
            .env(
                "CARGO_TARGET_DIR",
                env::var("CARGO_TARGET_DIR")
                    .unwrap_or_else(|_| DEFAULT_CARGO_TARGET_DIR.to_string()),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command
            .spawn()
            .context("failed to spawn codex app-server")?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("app-server stdin unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("app-server stdout unavailable"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("app-server stderr unavailable"))?;
        let (tx, rx) = mpsc::channel();
        let transcript_for_stdout = Arc::clone(&transcript);
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let notifications_for_stdout = Arc::clone(&notifications);
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if line.trim().is_empty() {
                    continue;
                }
                let message = serde_json::from_str::<Value>(&line).unwrap_or_else(
                    |error| json!({"_decode_error": error.to_string(), "raw": line}),
                );
                record(&transcript_for_stdout, "received", &message);
                if message.get("method").is_some()
                    && message.get("id").is_none()
                    && let Ok(mut notifications) = notifications_for_stdout.lock()
                {
                    notifications.push(message.clone());
                }
                let _ = tx.send(message);
            }
        });
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if let Ok(mut file) = stderr_file.lock() {
                    let _ = writeln!(file, "{line}");
                }
            }
        });
        Ok(Self {
            child,
            stdin,
            rx,
            transcript,
            notifications,
            next_id: 1,
        })
    }

    pub fn send(
        &mut self,
        method: &str,
        params: Option<Value>,
        expect_response: bool,
    ) -> Result<Value> {
        let mut message = serde_json::Map::new();
        message.insert("method".to_string(), json!(method));
        let request_id = if expect_response {
            let id = self.next_id;
            self.next_id += 1;
            message.insert("id".to_string(), json!(id));
            Some(id)
        } else {
            None
        };
        if let Some(params) = params {
            message.insert("params".to_string(), params);
        }
        let message = Value::Object(message);
        record(&self.transcript, "sent", &message);
        writeln!(
            self.stdin,
            "{}",
            serde_json::to_string(&message).context("failed to encode request")?
        )
        .context("failed to write app-server request")?;
        self.stdin
            .flush()
            .context("failed to flush app-server stdin")?;
        let Some(request_id) = request_id else {
            return Ok(Value::Null);
        };
        self.wait_for(request_id)
    }

    fn wait_for(&mut self, request_id: u64) -> Result<Value> {
        let deadline = Instant::now() + Duration::from_secs(45);
        while Instant::now() < deadline {
            if let Some(status) = self.child.try_wait()? {
                return Err(anyhow!(
                    "app-server exited with {} before response {}",
                    status,
                    request_id
                ));
            }
            match self.rx.recv_timeout(Duration::from_millis(500)) {
                Ok(message) => {
                    if message.get("id").and_then(Value::as_u64) != Some(request_id) {
                        continue;
                    }
                    if let Some(error) = message.get("error") {
                        return Err(anyhow!("request {request_id} failed: {error}"));
                    }
                    let result = message
                        .get("result")
                        .cloned()
                        .ok_or_else(|| anyhow!("request {request_id} returned no result"))?;
                    if !result.is_object() {
                        return Err(anyhow!(
                            "request {request_id} returned non-object result: {result}"
                        ));
                    }
                    return Ok(result);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(error) => return Err(anyhow!("app-server response channel closed: {error}")),
            }
        }
        Err(anyhow!("timed out waiting for response {request_id}"))
    }

    pub fn notification_count(&self, method: &str, start_index: usize) -> usize {
        self.notifications
            .lock()
            .ok()
            .map(|notifications| {
                notifications
                    .iter()
                    .skip(start_index)
                    .filter(|message| message.get("method").and_then(Value::as_str) == Some(method))
                    .count()
            })
            .unwrap_or(0)
    }

    pub fn notification_len(&self) -> usize {
        self.notifications
            .lock()
            .map(|notifications| notifications.len())
            .unwrap_or(0)
    }

    pub fn require_no_notification(
        &mut self,
        method: &str,
        start_index: usize,
        timeout: Duration,
    ) -> Result<()> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if let Some(status) = self.child.try_wait()? {
                return Err(anyhow!(
                    "app-server exited with {} while checking notification {}",
                    status,
                    method
                ));
            }
            if self.notification_count(method, start_index) > 0 {
                return Err(anyhow!("unexpected notification {method}"));
            }
            thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }

    pub fn wait_for_notification(
        &mut self,
        method: &str,
        start_index: usize,
        timeout: Duration,
    ) -> Result<Value> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if let Some(status) = self.child.try_wait()? {
                return Err(anyhow!(
                    "app-server exited with {} before notification {}",
                    status,
                    method
                ));
            }
            if let Ok(notifications) = self.notifications.lock()
                && let Some(message) = notifications
                    .iter()
                    .skip(start_index)
                    .find(|message| message.get("method").and_then(Value::as_str) == Some(method))
            {
                return Ok(message.clone());
            }
            thread::sleep(Duration::from_millis(100));
        }
        Err(anyhow!("timed out waiting for notification {method}"))
    }
}

impl Drop for AppServerClient {
    fn drop(&mut self) {
        let _ = self.stdin.flush();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn render_status(status: &Value) -> String {
    let scene = &status["scene"]["scene"];
    let pressure = &status["pressure"]["pressure"];
    let reorient = &status["reorient"]["decision"];
    let result = &status["reorientResult"];
    let recommendation = &status["crrc"]["recommendation"];
    let coordinator = &status["coordinator"];
    let heartbeat = &status["heartbeat"];
    let face = &status["face"];
    let planning_response = &status["planning"];
    let planning_summary = &planning_response["summary"];
    let checkpoint = &scene["investigationCheckpoint"];
    let latest_heartbeat = heartbeat["latestEvent"].clone();
    let latest_face = face["latestArtifacts"]
        .as_array()
        .and_then(|items| items.first())
        .cloned()
        .unwrap_or_else(|| json!({}));

    let mut lines = vec![
        "Epiphany MVP Status".to_string(),
        format!("Thread: {}", text(&status["threadId"])),
        format!(
            "State: {} rev {} ({})",
            text(&scene["stateStatus"]),
            maybe(&scene["revision"], "none"),
            text(&scene["source"])
        ),
        String::new(),
        "Recommendation".to_string(),
        format!("- action: {}", text(&recommendation["action"])),
        format!(
            "- scene action: {}",
            maybe(&recommendation["recommendedSceneAction"], "none")
        ),
        format!("- reason: {}", text(&recommendation["reason"])),
        format!(
            "- coordinator: {} ({})",
            maybe(&coordinator["action"], "none"),
            maybe(&coordinator["targetRole"], "none")
        ),
        String::new(),
        "Continuity".to_string(),
        format!(
            "- pressure: {} ({}, prepare={})",
            text(&pressure["level"]),
            text(&pressure["status"]),
            bool_text(&pressure["shouldPrepareCompaction"])
        ),
        format!(
            "- reorient: {} via {}",
            text(&reorient["action"]),
            list_text(&reorient["reasons"], "none")
        ),
        format!("- next: {}", text(&reorient["nextAction"])),
        format!(
            "- result: {} for {}",
            text(&result["status"]),
            text(&result["bindingId"])
        ),
        String::new(),
        "Heartbeat".to_string(),
        format!(
            "- status: {}; clock {}; rate {}",
            maybe(&heartbeat["status"], "none"),
            maybe(&heartbeat["sceneClock"], "none"),
            maybe(&heartbeat["targetHeartbeatRate"], "none")
        ),
        format!(
            "- latest: {} / {} / {}",
            maybe(&latest_heartbeat["selectedRole"], "none"),
            maybe(&latest_heartbeat["actionType"], "none"),
            maybe(&latest_heartbeat["coordinatorAction"], "none")
        ),
        String::new(),
        "Face".to_string(),
        format!("- latest artifact: {}", maybe(&latest_face["name"], "none")),
        format!(
            "- latest content: {}",
            maybe(&latest_face["content"], "none")
        ),
        String::new(),
        "Planning".to_string(),
        format!(
            "- state: {} rev {}",
            maybe(&planning_response["stateStatus"], "none"),
            maybe(&planning_response["stateRevision"], "none")
        ),
        format!(
            "- captures: {} (pending {}, github {})",
            maybe(&planning_summary["captureCount"], "0"),
            maybe(&planning_summary["pendingCaptureCount"], "0"),
            maybe(&planning_summary["githubIssueCaptureCount"], "0")
        ),
        format!(
            "- backlog: {} (ready {})",
            maybe(&planning_summary["backlogItemCount"], "0"),
            maybe(&planning_summary["readyBacklogItemCount"], "0")
        ),
        format!(
            "- roadmap streams: {}; objective drafts: {} (draft {})",
            maybe(&planning_summary["roadmapStreamCount"], "0"),
            maybe(&planning_summary["objectiveDraftCount"], "0"),
            maybe(&planning_summary["draftObjectiveCount"], "0")
        ),
        format!(
            "- active objective: {}",
            maybe(&planning_summary["activeObjective"], "none")
        ),
        format!("- note: {}", maybe(&planning_summary["note"], "none")),
        String::new(),
        "Role Lanes".to_string(),
    ];
    if let Some(roles) = status["roles"]["roles"].as_array() {
        for lane in roles {
            lines.push(format!(
                "- {}: {} ({}) - {}",
                text(&lane["title"]),
                text(&lane["status"]),
                text(&lane["ownerRole"]),
                text(&lane["note"])
            ));
        }
    }
    lines.extend([String::new(), "Role Findings".to_string()]);
    for (role_id, label) in [
        ("imagination", "Imagination / Planning"),
        ("modeling", "Modeling / Checkpoint"),
        ("verification", "Verification / Review"),
    ] {
        let role_result = &status["roleResults"][role_id];
        lines.push(format!(
            "- {}: {} for {}",
            label,
            maybe(&role_result["status"], "none"),
            maybe(&role_result["bindingId"], "none")
        ));
    }
    lines.extend([
        String::new(),
        "Checkpoint".to_string(),
        format!("- id: {}", maybe(&checkpoint["checkpointId"], "none")),
        format!(
            "- disposition: {}",
            maybe(&checkpoint["disposition"], "none")
        ),
        format!("- focus: {}", maybe(&checkpoint["focus"], "none")),
        format!("- next: {}", maybe(&checkpoint["nextAction"], "none")),
        String::new(),
        "Jobs".to_string(),
    ]);
    if let Some(jobs) = status["jobs"]["jobs"].as_array() {
        if jobs.is_empty() {
            lines.push("- none".to_string());
        } else {
            for job in jobs {
                lines.push(format!(
                    "- {}: {} {}, {} [{}]",
                    text(&job["id"]),
                    text(&job["status"]),
                    text(&job["kind"]),
                    text(&job["ownerRole"]),
                    text(&job["scope"])
                ));
            }
        }
    }
    lines.extend([
        String::new(),
        "Available Actions".to_string(),
        format!("- {}", list_text(&scene["availableActions"], "none")),
    ]);
    format!("{}\n", lines.join("\n"))
}

pub fn sanitize_for_operator(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sanitized = serde_json::Map::new();
            for (key, item) in map {
                if SEALED_DIRECT_THOUGHT_KEYS
                    .iter()
                    .any(|candidate| candidate == &key)
                {
                    sanitized.insert(key.clone(), sealed_direct_thought(&key, &item));
                } else {
                    sanitized.insert(key, sanitize_for_operator(item));
                }
            }
            Value::Object(sanitized)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sanitize_for_operator).collect()),
        other => other,
    }
}

fn sealed_direct_thought(key: &str, value: &Value) -> Value {
    let size = match value {
        Value::String(text) => Some(text.chars().count()),
        Value::Array(items) => Some(items.len()),
        Value::Object(map) => Some(map.len()),
        _ => None,
    };
    let mut sealed = json!({
        "sealed": true,
        "key": key,
        "reason": "Operator-safe dogfood views use projected findings and audit receipts; direct agent transcript/thought payloads stay sealed unless the user explicitly requests forensic debugging.",
    });
    if let Some(size) = size {
        sealed["size"] = json!(size);
    }
    sealed
}

pub fn write_transcript_telemetry(transcript: &Path, output: &Path) -> Result<()> {
    let _ = native_json(
        "epiphany-agent-telemetry",
        &[
            &transcript.to_string_lossy(),
            "--output",
            &output.to_string_lossy(),
        ],
    )?;
    Ok(())
}

pub fn native_json(bin_name: &str, args: &[&str]) -> Result<Value> {
    let exe = PathBuf::from(
        env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| DEFAULT_CARGO_TARGET_DIR.to_string()),
    )
    .join("debug")
    .join(format!("{bin_name}.exe"));
    let output = if exe.exists() {
        Command::new(&exe).args(args).output()
    } else {
        let mut command = Command::new("cargo");
        command
            .arg("run")
            .arg("--quiet")
            .arg("--manifest-path")
            .arg("epiphany-core/Cargo.toml")
            .arg("--bin")
            .arg(bin_name)
            .arg("--")
            .args(args)
            .output()
    }
    .with_context(|| format!("failed to run {bin_name}"))?;
    if !output.status.success() {
        return Err(anyhow!(
            "{} failed: {}{}",
            bin_name,
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("{bin_name} returned invalid JSON"))
}

fn record(transcript: &Arc<Mutex<File>>, kind: &str, payload: &Value) {
    if let Ok(mut file) = transcript.lock() {
        let _ = writeln!(file, "{}", json!({kind: payload}));
    }
}

fn take_string(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("{name} requires a value"))
}

fn take_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(take_string(args, name)?))
}

fn home_dir() -> PathBuf {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()
            .context("failed to resolve current directory")?
            .join(path))
    }
}

fn text(value: &Value) -> String {
    value
        .as_str()
        .map(ToString::to_string)
        .unwrap_or_else(|| value.to_string())
}

fn maybe(value: &Value, fallback: &str) -> String {
    if value.is_null() {
        return fallback.to_string();
    }
    if let Some(text) = value.as_str() {
        if text.is_empty() {
            return fallback.to_string();
        }
        return text.to_string();
    }
    value.to_string()
}

fn bool_text(value: &Value) -> &'static str {
    if value.as_bool().unwrap_or(false) {
        "true"
    } else {
        "false"
    }
}

fn list_text(value: &Value, fallback: &str) -> String {
    let Some(items) = value.as_array() else {
        return fallback.to_string();
    };
    if items.is_empty() {
        return fallback.to_string();
    }
    items.iter().map(text).collect::<Vec<_>>().join(", ")
}
