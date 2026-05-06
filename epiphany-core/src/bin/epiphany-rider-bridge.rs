use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use serde_json::json;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const RIDER_EXECUTABLE_NAMES: &[&str] = &["rider64.exe", "rider.exe", "rider.bat", "rider"];

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        usage()?;
        return Ok(());
    };
    match run_command(&command, args.collect()) {
        Ok((code, value)) => {
            println!("{}", serde_json::to_string_pretty(&value)?);
            std::process::exit(code);
        }
        Err(error) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "error",
                    "error": error.to_string(),
                }))?
            );
            std::process::exit(1);
        }
    }
}

fn run_command(command: &str, args: Vec<String>) -> Result<(i32, serde_json::Value)> {
    match command {
        "status" => {
            let options = RiderOptions::parse(args)?;
            Ok((0, status_rider(&options)?))
        }
        "context" => {
            let options = RiderOptions::parse(args)?;
            Ok((0, context_rider(&options)?))
        }
        "open-ref" => {
            let options = RiderOptions::parse(args)?;
            Ok((0, open_ref_rider(&options)?))
        }
        "guidance" => {
            let options = RiderOptions::parse(args)?;
            Ok((
                0,
                json!({ "guidance": bridge_guidance(&options.project_root)? }),
            ))
        }
        _ => usage(),
    }
}

fn usage() -> Result<(i32, serde_json::Value)> {
    Err(anyhow!(
        "usage: epiphany-rider-bridge <status|context|open-ref|guidance> --project-root <path>"
    ))
}

#[derive(Debug)]
struct RiderOptions {
    project_root: PathBuf,
    artifact_root: PathBuf,
    artifact_dir: Option<PathBuf>,
    packet: Option<PathBuf>,
    file: Option<PathBuf>,
    line: Option<u32>,
    column: Option<u32>,
    selection_start: Option<u32>,
    selection_end: Option<u32>,
    symbol_name: Option<String>,
    symbol_kind: Option<String>,
    symbol_namespace: Option<String>,
    launch: bool,
}

impl RiderOptions {
    fn parse(args: Vec<String>) -> Result<Self> {
        let root = repo_root()?;
        let mut options = Self {
            project_root: env::current_dir()?,
            artifact_root: root.join(".epiphany-gui").join("rider"),
            artifact_dir: None,
            packet: None,
            file: None,
            line: None,
            column: None,
            selection_start: None,
            selection_end: None,
            symbol_name: None,
            symbol_kind: None,
            symbol_namespace: None,
            launch: false,
        };
        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--project-root" => options.project_root = next_path(&mut iter, &arg)?,
                "--artifact-root" => options.artifact_root = next_path(&mut iter, &arg)?,
                "--artifact-dir" => options.artifact_dir = Some(next_path(&mut iter, &arg)?),
                "--packet" => options.packet = Some(next_path(&mut iter, &arg)?),
                "--file" => options.file = Some(next_path(&mut iter, &arg)?),
                "--line" => options.line = Some(next_value(&mut iter, &arg)?.parse()?),
                "--column" => options.column = Some(next_value(&mut iter, &arg)?.parse()?),
                "--selection-start" => {
                    options.selection_start = Some(next_value(&mut iter, &arg)?.parse()?)
                }
                "--selection-end" => {
                    options.selection_end = Some(next_value(&mut iter, &arg)?.parse()?)
                }
                "--symbol-name" => options.symbol_name = Some(next_value(&mut iter, &arg)?),
                "--symbol-kind" => options.symbol_kind = Some(next_value(&mut iter, &arg)?),
                "--symbol-namespace" => {
                    options.symbol_namespace = Some(next_value(&mut iter, &arg)?)
                }
                "--launch" => options.launch = true,
                _ => return Err(anyhow!("unknown argument {arg:?}")),
            }
        }
        Ok(options)
    }
}

fn next_value(iter: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    iter.next()
        .ok_or_else(|| anyhow!("{name} requires a value"))
}

fn next_path(iter: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(next_value(iter, name)?))
}

fn repo_root() -> Result<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| anyhow!("epiphany-core has no parent repo root"))?
        .to_path_buf())
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn artifact_dir(root: &Path, action: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(root
        .canonicalize()
        .unwrap_or_else(|_| root.to_path_buf())
        .join(format!("rider-{action}-{nanos}-{}", std::process::id())))
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))
        .with_context(|| format!("failed to write {}", path.display()))
}

fn write_text(path: &Path, value: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, value).with_context(|| format!("failed to write {}", path.display()))
}

fn maybe_resolve(path: &Path) -> String {
    clean_windows_path(
        &path
            .canonicalize()
            .unwrap_or_else(|_| path.to_path_buf())
            .display()
            .to_string(),
    )
}

fn clean_windows_path(value: &str) -> String {
    value.strip_prefix(r"\\?\").unwrap_or(value).to_string()
}

fn is_inside(path: &Path, root: &Path) -> bool {
    let Ok(path) = path.canonicalize() else {
        return false;
    };
    let Ok(root) = root.canonicalize() else {
        return false;
    };
    path.starts_with(root)
}

fn relative_or_absolute(path: &Path, root: &Path) -> String {
    match (path.canonicalize(), root.canonicalize()) {
        (Ok(path), Ok(root)) => path
            .strip_prefix(root)
            .map(|value| value.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.display().to_string()),
        _ => path.display().to_string(),
    }
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for path in paths {
        let key = maybe_resolve(&path).to_lowercase();
        if seen.insert(key) {
            result.push(path);
        }
    }
    result
}

fn rider_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(value) = env::var("EPIPHANY_RIDER_ROOTS") {
        roots.extend(env::split_paths(&value));
    }
    for (variable, fallback) in [
        ("ProgramFiles", r"C:\Program Files".to_string()),
        ("ProgramFiles(x86)", r"C:\Program Files (x86)".to_string()),
        (
            "LOCALAPPDATA",
            dirs_home()
                .join("AppData")
                .join("Local")
                .display()
                .to_string(),
        ),
    ] {
        let base = env::var(variable).unwrap_or(fallback);
        roots.push(PathBuf::from(base).join("JetBrains"));
    }
    dedupe_paths(roots)
}

fn dirs_home() -> PathBuf {
    env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn discover_rider_installations() -> Result<Vec<serde_json::Value>> {
    let mut candidates = Vec::new();
    if let Ok(value) = env::var("EPIPHANY_RIDER_PATH") {
        candidates.push(PathBuf::from(value));
    }
    if let Some(path_var) = env::var_os("PATH") {
        for dir in env::split_paths(&path_var) {
            for name in RIDER_EXECUTABLE_NAMES {
                candidates.push(dir.join(name));
            }
        }
    }
    for root in rider_search_roots() {
        if !root.exists() {
            continue;
        }
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if !path.is_dir() || !name.contains("Rider") {
                continue;
            }
            for exe in RIDER_EXECUTABLE_NAMES {
                candidates.push(path.join("bin").join(exe));
            }
        }
    }
    let mut installations = Vec::new();
    for path in dedupe_paths(candidates) {
        if !path.exists() {
            continue;
        }
        let version_hint = path
            .components()
            .rev()
            .filter_map(|part| part.as_os_str().to_str())
            .find(|part| part.contains("Rider"))
            .map(ToOwned::to_owned);
        installations.push(json!({
            "path": maybe_resolve(&path),
            "exists": true,
            "versionHint": version_hint,
        }));
    }
    Ok(installations)
}

fn discover_solution(project_root: &Path) -> Result<serde_json::Value> {
    let root = project_root.canonicalize()?;
    let mut top_level = Vec::new();
    for entry in fs::read_dir(&root)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) == Some("sln") {
            top_level.push(path);
        }
    }
    top_level.sort();
    if !top_level.is_empty() {
        return Ok(json!({
            "status": "ready",
            "path": top_level[0],
            "candidates": top_level.iter().take(8).collect::<Vec<_>>(),
        }));
    }
    let mut candidates = Vec::new();
    collect_solutions(&root, &root, &mut candidates)?;
    Ok(json!({
        "status": if candidates.is_empty() { "missingSolution" } else { "ready" },
        "path": candidates.first(),
        "candidates": candidates.iter().take(8).collect::<Vec<_>>(),
    }))
}

fn collect_solutions(root: &Path, current: &Path, candidates: &mut Vec<PathBuf>) -> Result<()> {
    if candidates.len() >= 8 {
        return Ok(());
    }
    for entry in fs::read_dir(current)? {
        let path = entry?.path();
        let rel = path.strip_prefix(root).unwrap_or(&path);
        if rel.components().any(|component| {
            let value = component.as_os_str().to_string_lossy();
            value.starts_with('.')
                || matches!(
                    value.as_ref(),
                    "Library" | "Temp" | "obj" | "bin" | "node_modules"
                )
        }) {
            continue;
        }
        if path.is_dir() {
            collect_solutions(root, &path, candidates)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("sln") {
            candidates.push(path);
        }
    }
    Ok(())
}

fn run_git(project_root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn vcs_summary(project_root: &Path) -> serde_json::Value {
    let branch = run_git(project_root, &["rev-parse", "--abbrev-ref", "HEAD"]);
    let status = run_git(project_root, &["status", "--porcelain=v1"]);
    let changed = run_git(project_root, &["diff", "--name-only"]);
    let staged = run_git(project_root, &["diff", "--cached", "--name-only"]);
    let mut changed_files: Vec<String> = changed
        .as_deref()
        .unwrap_or_default()
        .lines()
        .map(ToOwned::to_owned)
        .collect();
    if let Some(status) = &status {
        for line in status.lines() {
            let path = line.get(3..).unwrap_or_default().trim().to_string();
            if !path.is_empty() && !changed_files.contains(&path) {
                changed_files.push(path);
            }
        }
    }
    json!({
        "status": if branch.is_some() { "ready" } else { "notGit" },
        "branch": branch,
        "dirty": status.as_deref().is_some_and(|value| !value.is_empty()),
        "changedFiles": changed_files,
        "stagedFiles": staged.as_deref().unwrap_or_default().lines().collect::<Vec<_>>(),
        "changedRangesKnown": branch.is_some(),
    })
}

fn render_status(summary: &serde_json::Value) -> String {
    format!(
        "# Rider Bridge Status\n\n- status: {}\n- workspace: `{}`\n- solution: `{}`\n- rider: `{}`\n- installations: {}\n- branch: {}\n- dirty: {}\n- note: {}\n\nRider is a source-context organ. This artifact is an operator-safe projection, not durable Epiphany truth.\n",
        summary["status"].as_str().unwrap_or("unknown"),
        summary["workspace"].as_str().unwrap_or("unknown"),
        summary["solutionPath"].as_str().unwrap_or("none"),
        summary["riderPath"].as_str().unwrap_or("missing"),
        summary["installationCount"].as_u64().unwrap_or_default(),
        summary["vcs"]["branch"].as_str().unwrap_or("unknown"),
        summary["vcs"]["dirty"].as_bool().unwrap_or(false),
        summary["note"].as_str().unwrap_or(""),
    )
}

fn render_context(packet: &serde_json::Value, summary: &serde_json::Value) -> String {
    format!(
        "# Rider Context Packet\n\n- status: {}\n- project: `{}`\n- solution: `{}`\n- file: `{}`\n- selection: {}-{}\n- symbol: {}\n- artifact: `{}`\n\nModeling may use this packet as scratch/context. It is not accepted map state until reviewed.\n",
        summary["status"].as_str().unwrap_or("unknown"),
        packet["projectRoot"].as_str().unwrap_or("unknown"),
        packet["solutionPath"].as_str().unwrap_or("none"),
        packet["filePath"].as_str().unwrap_or("none"),
        packet["selection"]["startLine"]
            .as_i64()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        packet["selection"]["endLine"]
            .as_i64()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        packet["symbol"]["name"].as_str().unwrap_or("none"),
        summary["artifactPath"].as_str().unwrap_or("unknown"),
    )
}

fn status_rider(options: &RiderOptions) -> Result<serde_json::Value> {
    let project_root = options.project_root.canonicalize()?;
    let directory = options
        .artifact_dir
        .clone()
        .map(Ok)
        .unwrap_or_else(|| artifact_dir(&options.artifact_root, "inspect"))?;
    let installations = discover_rider_installations()?;
    let solution = discover_solution(&project_root)?;
    let first_rider = installations.first();
    let status = if first_rider.is_some() {
        "ready"
    } else {
        "missingRider"
    };
    let note = if first_rider.is_some() {
        "Rider executable is available for explicit source navigation and context capture."
    } else {
        "No Rider executable was found. Install Rider or set EPIPHANY_RIDER_PATH."
    };
    let summary = json!({
        "kind": "riderBridgeStatus",
        "status": status,
        "workspace": project_root,
        "solutionPath": solution["path"],
        "solutionStatus": solution["status"],
        "solutionCandidates": solution["candidates"],
        "riderPath": first_rider.and_then(|value| value["path"].as_str()),
        "installationCount": installations.len(),
        "installations": installations,
        "searchRoots": rider_search_roots(),
        "vcs": vcs_summary(&project_root),
        "capturedAt": now_iso(),
        "artifactPath": directory,
        "note": note,
    });
    write_json(&directory.join("rider-bridge-summary.json"), &summary)?;
    write_text(
        &directory.join("rider-bridge-status.md"),
        &render_status(&summary),
    )?;
    Ok(summary)
}

fn packet_from_options(options: &RiderOptions, project_root: &Path) -> Result<serde_json::Value> {
    if let Some(packet) = &options.packet {
        let value: serde_json::Value = serde_json::from_str(&fs::read_to_string(packet)?)?;
        if !value.is_object() {
            return Err(anyhow!("context packet must be a JSON object"));
        }
        return Ok(value);
    }
    let solution = discover_solution(project_root)?;
    let file_path = options
        .file
        .as_ref()
        .map(|path| path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
    if let Some(file_path) = &file_path
        && !is_inside(file_path, project_root)
    {
        return Err(anyhow!(
            "context file is outside project root: {}",
            file_path.display()
        ));
    }
    Ok(json!({
        "kind": "riderContext",
        "capturedAt": now_iso(),
        "projectRoot": project_root,
        "solutionPath": solution["path"],
        "filePath": file_path.as_ref().map(|path| relative_or_absolute(path, project_root)),
        "caret": options.line.map(|line| json!({ "line": line, "column": options.column })),
        "selection": if options.selection_start.is_some() || options.selection_end.is_some() {
            json!({ "startLine": options.selection_start, "endLine": options.selection_end })
        } else {
            serde_json::Value::Null
        },
        "symbol": options.symbol_name.as_ref().map(|name| json!({
            "name": name,
            "kind": options.symbol_kind,
            "namespace": options.symbol_namespace,
        })),
        "vcs": vcs_summary(project_root),
    }))
}

fn context_rider(options: &RiderOptions) -> Result<serde_json::Value> {
    let project_root = options.project_root.canonicalize()?;
    let directory = options
        .artifact_dir
        .clone()
        .map(Ok)
        .unwrap_or_else(|| artifact_dir(&options.artifact_root, "context"))?;
    let packet = packet_from_options(options, &project_root)?;
    let mut status_options = RiderOptions {
        ..options.clone_for_status(directory.clone())
    };
    status_options.artifact_dir = Some(directory.clone());
    let mut summary = status_rider(&status_options)?;
    summary["kind"] = json!("riderContextCapture");
    summary["status"] = json!("captured");
    summary["contextPath"] = json!(directory.join("rider-context.json"));
    summary["filePath"] = packet["filePath"].clone();
    summary["symbol"] = packet["symbol"].clone();
    write_json(&directory.join("rider-context.json"), &packet)?;
    write_json(&directory.join("rider-bridge-summary.json"), &summary)?;
    write_text(
        &directory.join("rider-context.md"),
        &render_context(&packet, &summary),
    )?;
    Ok(summary)
}

impl RiderOptions {
    fn clone_for_status(&self, directory: PathBuf) -> Self {
        Self {
            project_root: self.project_root.clone(),
            artifact_root: self.artifact_root.clone(),
            artifact_dir: Some(directory),
            packet: None,
            file: None,
            line: None,
            column: None,
            selection_start: None,
            selection_end: None,
            symbol_name: None,
            symbol_kind: None,
            symbol_namespace: None,
            launch: false,
        }
    }
}

fn open_ref_rider(options: &RiderOptions) -> Result<serde_json::Value> {
    let project_root = options.project_root.canonicalize()?;
    let directory = options
        .artifact_dir
        .clone()
        .map(Ok)
        .unwrap_or_else(|| artifact_dir(&options.artifact_root, "open-ref"))?;
    let status_options = options.clone_for_status(directory.clone());
    let status = status_rider(&status_options)?;
    let file_path = options
        .file
        .as_ref()
        .ok_or_else(|| anyhow!("open-ref requires --file"))?
        .canonicalize()?;
    if !is_inside(&file_path, &project_root) {
        return Err(anyhow!(
            "code ref is outside project root: {}",
            file_path.display()
        ));
    }
    let rider_path = status["riderPath"].as_str();
    let mut command = Vec::<String>::new();
    if let Some(path) = rider_path {
        command.push(path.to_string());
        command.push(file_path.display().to_string());
        if let Some(line) = options.line {
            command.push("--line".to_string());
            command.push(line.to_string());
        }
    }
    if let Some(path) = rider_path
        && options.launch
    {
        let mut launch = Command::new(path);
        launch.arg(&file_path);
        if let Some(line) = options.line {
            launch.arg("--line").arg(line.to_string());
        }
        launch.current_dir(&project_root).spawn()?;
    }
    let summary = json!({
        "kind": "riderOpenRef",
        "status": if rider_path.is_some() { if options.launch { "launched" } else { "planned" } } else { "missingRider" },
        "workspace": status["workspace"],
        "solutionPath": status["solutionPath"],
        "solutionStatus": status["solutionStatus"],
        "solutionCandidates": status["solutionCandidates"],
        "riderPath": status["riderPath"],
        "installationCount": status["installationCount"],
        "installations": status["installations"],
        "searchRoots": status["searchRoots"],
        "vcs": status["vcs"],
        "capturedAt": status["capturedAt"],
        "artifactPath": directory,
        "note": status["note"],
        "filePath": file_path,
        "line": options.line,
        "column": options.column,
        "command": command,
        "launched": options.launch && rider_path.is_some(),
    });
    write_json(&directory.join("rider-open-ref.json"), &summary)?;
    write_json(&directory.join("rider-bridge-summary.json"), &summary)?;
    write_text(
        &directory.join("rider-bridge-status.md"),
        &render_status(&summary),
    )?;
    Ok(summary)
}

fn bridge_guidance(project_root: &Path) -> Result<String> {
    let root = repo_root()?;
    let installations = discover_rider_installations()?;
    let solution = discover_solution(project_root)?;
    let status_command = format!(
        "`epiphany-rider-bridge status --project-root {}`",
        project_root.display()
    );
    if let Some(first) = installations.first() {
        let solution_text = solution["path"].as_str().unwrap_or("no solution found yet");
        Ok(format!(
            "- Rider bridge: Rider is available at `{}` and solution context is `{}`. Use {} for an auditable source/IDE status receipt, and use `context --file <path> --selection-start <line> --selection-end <line> --symbol-name <name>` when the future plugin or operator sends a bounded source slice. Rider facts are scratch/context until modeling or verification accepts them.",
            first["path"].as_str().unwrap_or("unknown"),
            solution_text,
            status_command,
        ))
    } else {
        Ok(format!(
            "- Rider bridge: no Rider executable was found. Use {} to write an auditable missing-IDE artifact, or set EPIPHANY_RIDER_PATH. Do not pretend IDE diagnostics, changed ranges, or source navigation were captured until the bridge has a receipt. Native bridge binary is rooted at `{}`.",
            status_command,
            root.display(),
        ))
    }
}
