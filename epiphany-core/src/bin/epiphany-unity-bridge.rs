use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use serde_json::json;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const PROJECT_VERSION_RELATIVE: &str = "ProjectSettings/ProjectVersion.txt";
const UNITY_EDITOR_RELATIVE: &[&str] = &["Editor", "Unity.exe"];
const EDITOR_BRIDGE_RELATIVE: &str = "Assets/Editor/Epiphany/EpiphanyEditorBridge.cs";
const EDITOR_BRIDGE_EXECUTE_METHOD: &str = "GameCult.Epiphany.Unity.EpiphanyEditorBridge.RunProbe";
const FORBIDDEN_EXTRA_ARGS: &[&str] = &["-batchmode", "-quit", "-projectpath", "-logfile"];

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return Err(anyhow!(
            "usage: epiphany-unity-bridge <inspect|run|check-compilation|probe|run-tests|guidance>"
        ));
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
    let options = UnityOptions::parse(args)?;
    let result = match command {
        "inspect" => inspect_unity(&options)?,
        "run" => run_unity(&options)?,
        "check-compilation" => run_named_probe(&options, "check-compilation")?,
        "probe" => {
            let operation = options
                .operation
                .as_deref()
                .ok_or_else(|| anyhow!("probe requires --operation"))?;
            run_named_probe(&options, operation)?
        }
        "run-tests" => test_unity(&options)?,
        "guidance" => json!({ "guidance": bridge_guidance(&options.project_path)? }),
        _ => return Err(anyhow!("unsupported command: {command}")),
    };
    let blocked = result["runStatus"].as_str() == Some("blocked");
    Ok((if blocked { 2 } else { 0 }, result))
}

#[derive(Debug)]
struct UnityOptions {
    project_path: PathBuf,
    artifact_root: PathBuf,
    artifact_dir: Option<PathBuf>,
    label: String,
    timeout_seconds: u64,
    dry_run: bool,
    unity_args: Vec<String>,
    operation: Option<String>,
    scene: Option<String>,
    asset: Option<String>,
    guid: Option<String>,
    max_objects: Option<u32>,
    max_properties: Option<u32>,
    max_dependencies: Option<u32>,
    platform: String,
    filter: Option<String>,
}

impl UnityOptions {
    fn parse(args: Vec<String>) -> Result<Self> {
        let root = repo_root()?;
        let mut options = Self {
            project_path: env::current_dir()?,
            artifact_root: root.join(".epiphany-gui").join("runtime"),
            artifact_dir: None,
            label: "unity-bridge-run".to_string(),
            timeout_seconds: 600,
            dry_run: false,
            unity_args: Vec::new(),
            operation: None,
            scene: None,
            asset: None,
            guid: None,
            max_objects: None,
            max_properties: None,
            max_dependencies: None,
            platform: "editmode".to_string(),
            filter: None,
        };
        let mut iter = args.into_iter().peekable();
        while let Some(arg) = iter.next() {
            if arg == "--" {
                options.unity_args.extend(iter);
                break;
            }
            match arg.as_str() {
                "--project-path" => options.project_path = next_path(&mut iter, &arg)?,
                "--artifact-root" => options.artifact_root = next_path(&mut iter, &arg)?,
                "--artifact-dir" => options.artifact_dir = Some(next_path(&mut iter, &arg)?),
                "--label" => options.label = next_value(&mut iter, &arg)?,
                "--timeout-seconds" => {
                    options.timeout_seconds = next_value(&mut iter, &arg)?.parse()?
                }
                "--dry-run" => options.dry_run = true,
                "--operation" => options.operation = Some(next_value(&mut iter, &arg)?),
                "--scene" => options.scene = Some(next_value(&mut iter, &arg)?),
                "--asset" => options.asset = Some(next_value(&mut iter, &arg)?),
                "--guid" => options.guid = Some(next_value(&mut iter, &arg)?),
                "--max-objects" => {
                    options.max_objects = Some(next_value(&mut iter, &arg)?.parse()?)
                }
                "--max-properties" => {
                    options.max_properties = Some(next_value(&mut iter, &arg)?.parse()?)
                }
                "--max-dependencies" => {
                    options.max_dependencies = Some(next_value(&mut iter, &arg)?.parse()?)
                }
                "--platform" => options.platform = next_value(&mut iter, &arg)?,
                "--filter" => options.filter = Some(next_value(&mut iter, &arg)?),
                _ => options.unity_args.push(arg),
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

fn clean_windows_path(value: &str) -> String {
    value.strip_prefix(r"\\?\").unwrap_or(value).to_string()
}

fn path_string(path: &Path) -> String {
    clean_windows_path(&path.display().to_string())
}

fn maybe_resolve(path: &Path) -> String {
    path_string(&path.canonicalize().unwrap_or_else(|_| path.to_path_buf()))
}

fn artifact_dir(root: &Path, action: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(root
        .canonicalize()
        .unwrap_or_else(|_| root.to_path_buf())
        .join(format!("unity-{action}-{nanos}-{}", std::process::id())))
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

fn read_unity_project_version(project_path: &Path) -> serde_json::Value {
    let version_path = project_path.join(PROJECT_VERSION_RELATIVE);
    let mut result = json!({
        "path": path_string(&version_path),
        "exists": version_path.exists(),
        "editorVersion": null,
        "editorVersionWithRevision": null,
    });
    if !version_path.exists() {
        result["status"] = json!("missingProjectVersion");
        result["note"] = json!("ProjectSettings/ProjectVersion.txt was not found.");
        return result;
    }
    let Ok(raw) = fs::read_to_string(&version_path) else {
        result["status"] = json!("missingProjectVersion");
        result["note"] = json!("ProjectVersion.txt could not be read.");
        return result;
    };
    for line in raw.lines() {
        if let Some(value) = line.strip_prefix("m_EditorVersion:") {
            result["editorVersion"] = json!(value.trim());
        } else if let Some(value) = line.strip_prefix("m_EditorVersionWithRevision:") {
            result["editorVersionWithRevision"] = json!(value.trim());
        }
    }
    if result["editorVersion"]
        .as_str()
        .is_some_and(|value| !value.is_empty())
    {
        result["status"] = json!("ready");
    } else {
        result["status"] = json!("missingProjectVersion");
        result["note"] = json!("ProjectVersion.txt did not contain m_EditorVersion.");
    }
    result
}

fn default_unity_editor_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(value) = env::var("EPIPHANY_UNITY_EDITOR_ROOTS") {
        roots.extend(env::split_paths(&value));
    }
    roots.push(
        PathBuf::from(env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".to_string()))
            .join("Unity")
            .join("Hub")
            .join("Editor"),
    );
    roots.push(
        PathBuf::from(
            env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".to_string()),
        )
        .join("Unity")
        .join("Hub")
        .join("Editor"),
    );
    dedupe_paths(roots)
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

fn unity_editor_relative() -> PathBuf {
    UNITY_EDITOR_RELATIVE.iter().collect()
}

fn editor_path_for_version(root: &Path, version: &str) -> PathBuf {
    if root.file_name().and_then(|value| value.to_str()) == Some(version) {
        root.join(unity_editor_relative())
    } else {
        root.join(version).join(unity_editor_relative())
    }
}

fn installed_unity_editors(roots: &[PathBuf]) -> Result<Vec<serde_json::Value>> {
    let mut editors = Vec::new();
    let rel = unity_editor_relative();
    for root in roots {
        if !root.exists() {
            continue;
        }
        let direct = root.join(&rel);
        if direct.exists() {
            editors.push(json!({
                "version": root.file_name().and_then(|value| value.to_str()),
                "root": path_string(root),
                "editorPath": path_string(&direct),
            }));
            continue;
        }
        for entry in fs::read_dir(root)? {
            let child = entry?.path();
            if !child.is_dir() {
                continue;
            }
            let editor = child.join(&rel);
            if editor.exists() {
                editors.push(json!({
                    "version": child.file_name().and_then(|value| value.to_str()),
                    "root": path_string(&child),
                    "editorPath": path_string(&editor),
                }));
            }
        }
    }
    Ok(editors)
}

fn detect_editor_bridge(project_path: &Path) -> serde_json::Value {
    let package_path = project_path.join(EDITOR_BRIDGE_RELATIVE);
    json!({
        "kind": "epiphanyUnityEditorBridge",
        "path": path_string(&package_path),
        "relativePath": EDITOR_BRIDGE_RELATIVE,
        "exists": package_path.exists(),
        "executeMethod": EDITOR_BRIDGE_EXECUTE_METHOD,
    })
}

fn resolve_unity_editor(project_path: &Path) -> Result<serde_json::Value> {
    let project_path = project_path
        .canonicalize()
        .unwrap_or_else(|_| project_path.to_path_buf());
    let version = read_unity_project_version(&project_path);
    let roots = default_unity_editor_roots();
    let editors = installed_unity_editors(&roots)?;
    let mut result = json!({
        "kind": "unity",
        "projectPath": path_string(&project_path),
        "generatedAt": now_iso(),
        "versionFile": version,
        "searchRoots": roots.iter().map(|path| path_string(path)).collect::<Vec<_>>(),
        "installedEditors": editors,
        "projectVersion": null,
        "projectVersionWithRevision": null,
        "editorPath": null,
        "status": "missingProjectVersion",
        "note": "ProjectSettings/ProjectVersion.txt was not found or did not pin a Unity editor.",
        "editorBridge": detect_editor_bridge(&project_path),
    });
    result["projectVersion"] = result["versionFile"]["editorVersion"].clone();
    result["projectVersionWithRevision"] =
        result["versionFile"]["editorVersionWithRevision"].clone();
    let Some(project_version) = result["projectVersion"].as_str().map(ToOwned::to_owned) else {
        return Ok(result);
    };
    if project_version.is_empty() {
        return Ok(result);
    }
    let exact_candidates: Vec<PathBuf> = roots
        .iter()
        .map(|root| editor_path_for_version(root, &project_version))
        .collect();
    let exact = exact_candidates.iter().find(|path| path.exists());
    result["candidatePaths"] = json!(
        exact_candidates
            .iter()
            .map(|path| path_string(path))
            .collect::<Vec<_>>()
    );
    if let Some(exact) = exact {
        result["status"] = json!("ready");
        result["editorPath"] = json!(path_string(exact));
        result["note"] = json!(format!(
            "Project pins Unity {project_version}; exact editor resolved at {}.",
            path_string(exact)
        ));
    } else {
        let installed_versions = result["installedEditors"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|item| item["version"].as_str())
            .collect::<Vec<_>>()
            .join(", ");
        result["status"] = json!("missingEditor");
        result["note"] = json!(format!(
            "Project pins Unity {project_version}, but no exact editor was found. Installed Hub versions: {}.",
            if installed_versions.is_empty() {
                "none detected"
            } else {
                &installed_versions
            }
        ));
    }
    Ok(result)
}

fn render_inspection(summary: &serde_json::Value) -> String {
    let installed_lines = summary["installedEditors"]
        .as_array()
        .filter(|items| !items.is_empty())
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    format!(
                        "- {}: {}",
                        item["version"].as_str().unwrap_or("unknown"),
                        item["editorPath"].as_str().unwrap_or("unknown")
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec!["- none detected".to_string()]);
    let candidate_lines = summary["candidatePaths"]
        .as_array()
        .filter(|items| !items.is_empty())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(|item| format!("- {item}"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec!["- none".to_string()]);
    format!(
        "# Unity Runtime Bridge\n\nStatus: {}\nProject: {}\nProject version: {}\nEditor path: {}\n\nInstalled Hub editors:\n{}\n\nExact candidate paths:\n{}\n\nNote: {}\n",
        summary["status"].as_str().unwrap_or("unknown"),
        summary["projectPath"].as_str().unwrap_or("unknown"),
        summary["projectVersion"].as_str().unwrap_or("none"),
        summary["editorPath"].as_str().unwrap_or("none"),
        installed_lines.join("\n"),
        candidate_lines.join("\n"),
        summary["note"].as_str().unwrap_or(""),
    )
}

fn write_inspection_artifacts(directory: &Path, summary: &serde_json::Value) -> Result<()> {
    write_json(&directory.join("unity-bridge-summary.json"), summary)?;
    write_text(
        &directory.join("unity-bridge-inspection.md"),
        &render_inspection(summary),
    )
}

fn validate_unity_args(values: &[String]) -> Result<()> {
    let lowered: HashSet<String> = values.iter().map(|value| value.to_lowercase()).collect();
    let forbidden = FORBIDDEN_EXTRA_ARGS
        .iter()
        .filter(|value| lowered.contains(**value))
        .copied()
        .collect::<Vec<_>>();
    if !forbidden.is_empty() {
        return Err(anyhow!(
            "Unity bridge owns batchmode, quit, projectPath, and logFile; remove extra args: {}",
            forbidden.join(", ")
        ));
    }
    Ok(())
}

fn build_unity_command(
    summary: &serde_json::Value,
    extra_args: &[String],
    log_path: Option<&Path>,
) -> Result<Vec<String>> {
    let editor_path = summary["editorPath"]
        .as_str()
        .ok_or_else(|| anyhow!("Unity editor path is unavailable."))?;
    let project_path = summary["projectPath"]
        .as_str()
        .ok_or_else(|| anyhow!("Unity project path is unavailable."))?;
    validate_unity_args(extra_args)?;
    let mut command = vec![
        editor_path.to_string(),
        "-batchmode".to_string(),
        "-quit".to_string(),
        "-projectPath".to_string(),
        project_path.to_string(),
    ];
    if let Some(log_path) = log_path {
        command.extend(["-logFile".to_string(), path_string(log_path)]);
    }
    command.extend(extra_args.iter().cloned());
    Ok(command)
}

fn editor_bridge_ready(summary: &serde_json::Value) -> bool {
    summary["editorBridge"]["exists"].as_bool() == Some(true)
}

fn named_probe_extra_args(
    options: &UnityOptions,
    operation: &str,
    directory: &Path,
) -> Vec<String> {
    let mut args = vec![
        "-executeMethod".to_string(),
        EDITOR_BRIDGE_EXECUTE_METHOD.to_string(),
        "-epiphanyArtifactDir".to_string(),
        path_string(directory),
        "-epiphanyOperation".to_string(),
        operation.to_string(),
    ];
    for (value, flag) in [
        (
            options.scene.as_ref().map(ToString::to_string),
            "-epiphanyScene",
        ),
        (
            options.asset.as_ref().map(ToString::to_string),
            "-epiphanyAsset",
        ),
        (
            options.guid.as_ref().map(ToString::to_string),
            "-epiphanyGuid",
        ),
        (
            options.max_objects.map(|value| value.to_string()),
            "-epiphanyMaxObjects",
        ),
        (
            options.max_properties.map(|value| value.to_string()),
            "-epiphanyMaxProperties",
        ),
        (
            options.max_dependencies.map(|value| value.to_string()),
            "-epiphanyMaxDependencies",
        ),
    ] {
        if let Some(value) = value {
            args.extend([flag.to_string(), value]);
        }
    }
    args
}

fn expected_artifacts(operation: &str) -> Vec<&'static str> {
    match operation {
        "inspect-project" => vec![
            "project-facts.json",
            "unity-probe-result.json",
            "unity-probe-result.md",
        ],
        "check-compilation" => vec![
            "compilation.json",
            "unity-probe-result.json",
            "unity-probe-result.md",
        ],
        "scene-facts" => vec![
            "scene-facts.json",
            "unity-probe-result.json",
            "unity-probe-result.md",
        ],
        "prefab-facts" => vec![
            "prefab-facts.json",
            "unity-probe-result.json",
            "unity-probe-result.md",
        ],
        "serialized-object" => vec![
            "serialized-object-facts.json",
            "unity-probe-result.json",
            "unity-probe-result.md",
        ],
        "reference-search" => vec![
            "reference-search.json",
            "unity-probe-result.json",
            "unity-probe-result.md",
        ],
        "run-tests" => vec!["test-results.xml", "unity.log"],
        _ => Vec::new(),
    }
}

fn probe_description(operation: &str) -> &'static str {
    match operation {
        "inspect-project" => "Capture Unity project/editor/build/render-pipeline facts.",
        "check-compilation" => {
            "Check whether the editor bridge can run after Unity script compilation."
        }
        "scene-facts" => {
            "Capture scene hierarchy, components, prefab links, and serialized fields."
        }
        "prefab-facts" => {
            "Capture prefab hierarchy, nested instances, overrides, and serialized fields."
        }
        "serialized-object" => "Capture SerializedObject properties for one asset.",
        "reference-search" => "Search Unity asset dependencies for references to a GUID or asset.",
        _ => "Run a named Unity editor bridge probe.",
    }
}

fn write_command_artifact(
    directory: &Path,
    command: &[String],
    dry_run: bool,
    operation: &str,
    expected_artifacts: Vec<&str>,
) -> Result<()> {
    write_json(
        &directory.join("unity-command.json"),
        &json!({
            "command": command,
            "dryRun": dry_run,
            "operation": operation,
            "expectedArtifacts": expected_artifacts,
        }),
    )
}

fn run_unity(options: &UnityOptions) -> Result<serde_json::Value> {
    let directory = options
        .artifact_dir
        .clone()
        .map(Ok)
        .unwrap_or_else(|| artifact_dir(&options.artifact_root, "run"))?;
    let project_path = options
        .project_path
        .canonicalize()
        .unwrap_or_else(|_| options.project_path.clone());
    let mut summary = resolve_unity_editor(&project_path)?;
    summary["artifactPath"] = json!(path_string(&directory));
    summary["operation"] = json!("run");
    summary["label"] = json!(options.label);
    summary["unityArgs"] = json!(options.unity_args);
    if summary["status"].as_str() != Some("ready") {
        summary["runStatus"] = json!("blocked");
        summary["note"] = json!(format!(
            "{} Runtime execution refused; install the exact pinned editor or use inspect artifacts as evidence of the missing runtime.",
            summary["note"].as_str().unwrap_or("")
        ));
        write_inspection_artifacts(&directory, &summary)?;
        return Ok(summary);
    }
    let log_path = directory.join("unity.log");
    let command = build_unity_command(&summary, &options.unity_args, Some(&log_path))?;
    summary["command"] = json!(command);
    summary["logPath"] = json!(path_string(&log_path));
    if options.dry_run {
        summary["runStatus"] = json!("planned");
        summary["returncode"] = serde_json::Value::Null;
        write_inspection_artifacts(&directory, &summary)?;
        write_command_artifact(
            &directory,
            summary["command"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                .collect::<Vec<_>>()
                .as_slice(),
            true,
            "run",
            Vec::new(),
        )?;
        return Ok(summary);
    }
    execute_command(
        &mut summary,
        &directory,
        &project_path,
        options.timeout_seconds,
    )?;
    write_inspection_artifacts(&directory, &summary)?;
    write_command_artifact(
        &directory,
        summary["command"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>()
            .as_slice(),
        false,
        "run",
        Vec::new(),
    )?;
    Ok(summary)
}

fn run_named_probe(options: &UnityOptions, operation: &str) -> Result<serde_json::Value> {
    let allowed = [
        "inspect-project",
        "check-compilation",
        "scene-facts",
        "prefab-facts",
        "serialized-object",
        "reference-search",
    ];
    if !allowed.contains(&operation) {
        return Err(anyhow!("unsupported probe operation: {operation}"));
    }
    let directory =
        options.artifact_dir.clone().map(Ok).unwrap_or_else(|| {
            artifact_dir(&options.artifact_root, &operation.replace('-', "_"))
        })?;
    let project_path = options
        .project_path
        .canonicalize()
        .unwrap_or_else(|_| options.project_path.clone());
    let mut summary = resolve_unity_editor(&project_path)?;
    summary["artifactPath"] = json!(path_string(&directory));
    summary["operation"] = json!(operation);
    summary["probe"] = json!({
        "description": probe_description(operation),
        "expectedArtifacts": expected_artifacts(operation),
    });
    if summary["status"].as_str() != Some("ready") {
        summary["runStatus"] = json!("blocked");
        summary["note"] = json!(format!(
            "{} Runtime/editor probe refused; install the exact pinned editor or use inspect artifacts as evidence of the missing runtime.",
            summary["note"].as_str().unwrap_or("")
        ));
        write_inspection_artifacts(&directory, &summary)?;
        return Ok(summary);
    }
    if !editor_bridge_ready(&summary) {
        let bridge_path = summary["editorBridge"]["path"]
            .as_str()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| path_string(&project_path.join(EDITOR_BRIDGE_RELATIVE)));
        summary["status"] = json!("missingEditorBridgePackage");
        summary["runStatus"] = json!("blocked");
        summary["note"] = json!(format!(
            "The Epiphany Unity editor package is missing. Add {bridge_path} before running editor-resident probes."
        ));
        write_inspection_artifacts(&directory, &summary)?;
        return Ok(summary);
    }
    let extra_args = named_probe_extra_args(options, operation, &directory);
    let log_path = directory.join("unity.log");
    let command = build_unity_command(&summary, &extra_args, Some(&log_path))?;
    summary["command"] = json!(command);
    summary["logPath"] = json!(path_string(&log_path));
    if options.dry_run {
        summary["runStatus"] = json!("planned");
        summary["returncode"] = serde_json::Value::Null;
        write_inspection_artifacts(&directory, &summary)?;
        write_command_artifact(
            &directory,
            summary["command"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                .collect::<Vec<_>>()
                .as_slice(),
            true,
            operation,
            expected_artifacts(operation),
        )?;
        return Ok(summary);
    }
    execute_command(
        &mut summary,
        &directory,
        &project_path,
        options.timeout_seconds,
    )?;
    write_inspection_artifacts(&directory, &summary)?;
    write_command_artifact(
        &directory,
        summary["command"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>()
            .as_slice(),
        false,
        operation,
        expected_artifacts(operation),
    )?;
    Ok(summary)
}

fn test_unity(options: &UnityOptions) -> Result<serde_json::Value> {
    let directory = options
        .artifact_dir
        .clone()
        .map(Ok)
        .unwrap_or_else(|| artifact_dir(&options.artifact_root, "run_tests"))?;
    let project_path = options
        .project_path
        .canonicalize()
        .unwrap_or_else(|_| options.project_path.clone());
    let mut summary = resolve_unity_editor(&project_path)?;
    summary["artifactPath"] = json!(path_string(&directory));
    summary["operation"] = json!("run-tests");
    summary["testPlatform"] = json!(options.platform);
    summary["testFilter"] = json!(options.filter);
    if summary["status"].as_str() != Some("ready") {
        summary["runStatus"] = json!("blocked");
        summary["note"] = json!(format!(
            "{} Unity tests refused; install the exact pinned editor or use inspect artifacts as evidence of the missing runtime.",
            summary["note"].as_str().unwrap_or("")
        ));
        write_inspection_artifacts(&directory, &summary)?;
        return Ok(summary);
    }
    let log_path = directory.join("unity.log");
    let test_results = directory.join("test-results.xml");
    let mut extra_args = vec![
        "-runTests".to_string(),
        "-testPlatform".to_string(),
        options.platform.clone(),
        "-testResults".to_string(),
        path_string(&test_results),
    ];
    if let Some(filter) = &options.filter {
        extra_args.extend(["-testFilter".to_string(), filter.clone()]);
    }
    let command = build_unity_command(&summary, &extra_args, Some(&log_path))?;
    summary["command"] = json!(command);
    summary["logPath"] = json!(path_string(&log_path));
    summary["testResultsPath"] = json!(path_string(&test_results));
    if options.dry_run {
        summary["runStatus"] = json!("planned");
        summary["returncode"] = serde_json::Value::Null;
        write_inspection_artifacts(&directory, &summary)?;
        write_command_artifact(
            &directory,
            summary["command"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                .collect::<Vec<_>>()
                .as_slice(),
            true,
            "run-tests",
            expected_artifacts("run-tests"),
        )?;
        return Ok(summary);
    }
    execute_command(
        &mut summary,
        &directory,
        &project_path,
        options.timeout_seconds,
    )?;
    write_inspection_artifacts(&directory, &summary)?;
    write_command_artifact(
        &directory,
        summary["command"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>()
            .as_slice(),
        false,
        "run-tests",
        expected_artifacts("run-tests"),
    )?;
    Ok(summary)
}

fn execute_command(
    summary: &mut serde_json::Value,
    directory: &Path,
    project_path: &Path,
    timeout_seconds: u64,
) -> Result<()> {
    let command = summary["command"]
        .as_array()
        .ok_or_else(|| anyhow!("summary command is not an array"))?
        .iter()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    let (program, args) = command
        .split_first()
        .ok_or_else(|| anyhow!("empty Unity command"))?;
    let stdout_path = directory.join("unity-stdout.log");
    let stderr_path = directory.join("unity-stderr.log");
    fs::create_dir_all(directory)?;
    let started = Instant::now();
    let mut child = Command::new(program)
        .args(args)
        .current_dir(project_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    let timeout = Duration::from_secs(timeout_seconds);
    loop {
        if let Some(status) = child.try_wait()? {
            let output = child.wait_with_output()?;
            fs::write(&stdout_path, output.stdout)?;
            fs::write(&stderr_path, output.stderr)?;
            let code = status.code();
            summary["runStatus"] = json!(if status.success() { "passed" } else { "failed" });
            summary["returncode"] = json!(code);
            summary["durationSeconds"] =
                json!((started.elapsed().as_millis() as f64 / 1000.0).round());
            summary["stdoutPath"] = json!(path_string(&stdout_path));
            summary["stderrPath"] = json!(path_string(&stderr_path));
            return Ok(());
        }
        if started.elapsed() > timeout {
            let _ = child.kill();
            let output = child.wait_with_output()?;
            if !output.stdout.is_empty() {
                fs::write(&stdout_path, output.stdout)?;
                summary["stdoutPath"] = json!(path_string(&stdout_path));
            }
            if !output.stderr.is_empty() {
                fs::write(&stderr_path, output.stderr)?;
                summary["stderrPath"] = json!(path_string(&stderr_path));
            }
            summary["runStatus"] = json!("timedOut");
            summary["returncode"] = serde_json::Value::Null;
            summary["durationSeconds"] =
                json!((started.elapsed().as_millis() as f64 / 1000.0).round());
            summary["note"] = json!(format!(
                "Unity command timed out after {timeout_seconds} seconds."
            ));
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn inspect_unity(options: &UnityOptions) -> Result<serde_json::Value> {
    let directory = options
        .artifact_dir
        .clone()
        .map(Ok)
        .unwrap_or_else(|| artifact_dir(&options.artifact_root, "inspect"))?;
    let mut summary = resolve_unity_editor(&options.project_path)?;
    summary["artifactPath"] = json!(path_string(&directory));
    summary["operation"] = json!("inspect");
    write_inspection_artifacts(&directory, &summary)?;
    Ok(summary)
}

fn bridge_guidance(project_path: &Path) -> Result<String> {
    let summary = resolve_unity_editor(project_path)?;
    let inspect_command = format!(
        "`epiphany-unity-bridge inspect --project-path {}`",
        project_path.display()
    );
    if summary["status"].as_str() == Some("ready") {
        if editor_bridge_ready(&summary) {
            Ok(format!(
                "- Unity bridge: project pins {} and the exact editor resolved to `{}`. If Unity execution is needed, use named bridge operations such as `epiphany-unity-bridge probe --project-path {} --operation scene-facts --scene <Assets/...unity>` or `epiphany-unity-bridge check-compilation --project-path {}`. The bridge owns -batchmode, -quit, -projectPath, -logFile, -executeMethod, and artifacts. Do not invoke `Unity`, `Unity.exe`, default installs, or PATH-resolved editors directly, and do not refactor Unity-owned scene/prefab state as raw text when the editor bridge can inspect it.",
                summary["projectVersion"].as_str().unwrap_or("unknown"),
                summary["editorPath"].as_str().unwrap_or("unknown"),
                project_path.display(),
                project_path.display(),
            ))
        } else {
            Ok(format!(
                "- Unity bridge: project pins {} and the exact editor resolved to `{}`, but the resident editor package is missing. Add `{}` before claiming scene, prefab, or runtime facts. Do not invoke `Unity`, `Unity.exe`, default installs, or PATH-resolved editors directly.",
                summary["projectVersion"].as_str().unwrap_or("unknown"),
                summary["editorPath"].as_str().unwrap_or("unknown"),
                EDITOR_BRIDGE_RELATIVE,
            ))
        }
    } else {
        Ok(format!(
            "- Unity bridge: {} Run {} to write an auditable runtime evidence artifact. Do not invoke `Unity`, `Unity.exe`, default installs, or PATH-resolved editors directly; if runtime parity is needed, stop with the bridge artifact as the evidence gap.",
            summary["note"]
                .as_str()
                .unwrap_or("Unity bridge is blocked."),
            inspect_command,
        ))
    }
}
