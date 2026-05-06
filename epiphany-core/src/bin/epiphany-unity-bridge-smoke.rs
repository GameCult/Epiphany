use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PROJECT_VERSION: &str = "6000.1.10f1";
const WRONG_VERSION: &str = "6000.4.2f1";

fn main() -> Result<()> {
    let result = run_smoke()?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn run_smoke() -> Result<Value> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let workspace = root.join(".epiphany-smoke").join("unity-bridge-workspace");
    let artifact_root = root.join(".epiphany-smoke").join("unity-bridge-artifacts");
    let fake_roots = root.join(".epiphany-smoke").join("unity-bridge-editors");
    let result_path = root
        .join(".epiphany-smoke")
        .join("unity-bridge-smoke-result.json");
    prepare_project(&workspace, false)?;
    reset_path(&artifact_root)?;
    reset_path(&fake_roots)?;

    let wrong_editor = create_fake_editor(&fake_roots, WRONG_VERSION)?;
    let missing = run_bridge(
        &root,
        &workspace,
        &artifact_root,
        &fake_roots,
        &["inspect"],
        true,
    )?;
    require(
        missing["status"] == "missingEditor",
        "wrong-only Hub root should not satisfy pin",
    )?;
    require(
        missing["projectVersion"] == PROJECT_VERSION,
        "inspection should report the project-pinned version",
    )?;
    require_artifact(&missing, "unity-bridge-summary.json")?;
    require_artifact(&missing, "unity-bridge-inspection.md")?;

    let exact_editor = create_fake_editor(&fake_roots, PROJECT_VERSION)?;
    let ready = run_bridge(
        &root,
        &workspace,
        &artifact_root,
        &fake_roots,
        &["inspect"],
        true,
    )?;
    require(
        ready["status"] == "ready",
        "exact editor should satisfy pin",
    )?;
    require(
        ready["editorPath"] == exact_editor.to_string_lossy().as_ref(),
        "inspection should resolve exact editor",
    )?;
    require(
        ready
            .pointer("/editorBridge/exists")
            .and_then(Value::as_bool)
            == Some(false),
        "fresh smoke project should not have bridge package yet",
    )?;

    let planned = run_bridge(
        &root,
        &workspace,
        &artifact_root,
        &fake_roots,
        &[
            "run",
            "--dry-run",
            "--",
            "-executeMethod",
            "Epiphany.SmokeProbe",
        ],
        true,
    )?;
    require(
        planned["status"] == "ready",
        "dry run should still require exact editor",
    )?;
    require(
        planned["runStatus"] == "planned",
        "dry run should plan instead of executing",
    )?;
    let command = value_array(&planned, "command")?;
    require(
        command.first().and_then(Value::as_str) == Some(exact_editor.to_str().unwrap_or_default()),
        "command should use exact editor path",
    )?;
    require_command_contains(command, "-batchmode")?;
    require_command_contains(command, "-quit")?;
    require_command_contains(command, "-projectPath")?;
    require_command_contains(command, "-logFile")?;
    require_command_contains(command, &workspace.to_string_lossy())?;
    require(
        !command
            .iter()
            .any(|item| item.as_str() == Some(wrong_editor.to_str().unwrap_or_default())),
        "command must not use wrong editor",
    )?;
    require_artifact(&planned, "unity-command.json")?;

    let missing_package = run_bridge(
        &root,
        &workspace,
        &artifact_root,
        &fake_roots,
        &[
            "probe",
            "--dry-run",
            "--operation",
            "scene-facts",
            "--scene",
            "Assets/Scenes/Main.unity",
        ],
        false,
    )?;
    require(
        missing_package["status"] == "missingEditorBridgePackage",
        "named probes should require the resident editor package",
    )?;
    require(
        missing_package["runStatus"] == "blocked",
        "missing package probe should be blocked",
    )?;

    prepare_project(&workspace, true)?;
    let bridged = run_bridge(
        &root,
        &workspace,
        &artifact_root,
        &fake_roots,
        &["inspect"],
        true,
    )?;
    require(
        bridged["status"] == "ready",
        "bridge project should still resolve exact editor",
    )?;
    require(
        bridged
            .pointer("/editorBridge/exists")
            .and_then(Value::as_bool)
            == Some(true),
        "resident editor bridge should be detected",
    )?;

    let scene_probe = run_bridge(
        &root,
        &workspace,
        &artifact_root,
        &fake_roots,
        &[
            "probe",
            "--dry-run",
            "--operation",
            "scene-facts",
            "--scene",
            "Assets/Scenes/Main.unity",
            "--max-objects",
            "25",
            "--max-properties",
            "12",
        ],
        true,
    )?;
    require(
        scene_probe["runStatus"] == "planned",
        "scene probe should plan under dry run",
    )?;
    let scene_command = value_array(&scene_probe, "command")?;
    require(
        scene_command.first().and_then(Value::as_str)
            == Some(exact_editor.to_str().unwrap_or_default()),
        "scene probe should use exact editor path",
    )?;
    require_command_contains(scene_command, "-executeMethod")?;
    require_command_contains(
        scene_command,
        "GameCult.Epiphany.Unity.EpiphanyEditorBridge.RunProbe",
    )?;
    require_command_contains(scene_command, "-epiphanyArtifactDir")?;
    let scene_artifact_path = path_value(&scene_probe, "artifactPath")?;
    require_command_contains(scene_command, &scene_artifact_path.to_string_lossy())?;
    require_command_contains(scene_command, "-epiphanyOperation")?;
    require_command_contains(scene_command, "scene-facts")?;
    require_command_contains(scene_command, "-epiphanyScene")?;
    require_command_contains(scene_command, "Assets/Scenes/Main.unity")?;
    require_command_contains(scene_command, "-epiphanyMaxObjects")?;
    require_command_contains(scene_command, "25")?;
    let scene_artifact = read_command_artifact(&scene_probe)?;
    require(
        scene_artifact["operation"] == "scene-facts",
        "scene command artifact should name operation",
    )?;
    require(
        array_contains(scene_artifact.get("expectedArtifacts"), "scene-facts.json"),
        "scene facts artifact should be expected",
    )?;

    let compilation = run_bridge(
        &root,
        &workspace,
        &artifact_root,
        &fake_roots,
        &["check-compilation", "--dry-run"],
        true,
    )?;
    require(
        compilation["runStatus"] == "planned",
        "compilation probe should plan under dry run",
    )?;
    let compilation_artifact = read_command_artifact(&compilation)?;
    require(
        array_contains(
            compilation_artifact.get("expectedArtifacts"),
            "compilation.json",
        ),
        "compilation artifact should be expected",
    )?;

    let tests = run_bridge(
        &root,
        &workspace,
        &artifact_root,
        &fake_roots,
        &[
            "run-tests",
            "--dry-run",
            "--platform",
            "editmode",
            "--filter",
            "SmokeSuite",
        ],
        true,
    )?;
    require(
        tests["runStatus"] == "planned",
        "Unity test run should plan under dry run",
    )?;
    let test_command = value_array(&tests, "command")?;
    require_command_contains(test_command, "-runTests")?;
    require_command_contains(test_command, "-testPlatform")?;
    require_command_contains(test_command, "editmode")?;
    require_command_contains(test_command, "-testFilter")?;
    require_command_contains(test_command, "SmokeSuite")?;
    let tests_artifact = read_command_artifact(&tests)?;
    require(
        array_contains(tests_artifact.get("expectedArtifacts"), "test-results.xml"),
        "test results should be expected",
    )?;

    let blocked = run_bridge(
        &root,
        &workspace,
        &artifact_root,
        &fake_roots.join("missing"),
        &[
            "run",
            "--dry-run",
            "--",
            "-executeMethod",
            "Epiphany.SmokeProbe",
        ],
        false,
    )?;
    require(
        blocked["runStatus"] == "blocked",
        "missing editor run should be blocked",
    )?;

    let result = json!({
        "workspace": workspace,
        "artifactRoot": artifact_root,
        "missingStatus": missing["status"],
        "readyStatus": ready["status"],
        "plannedStatus": planned["runStatus"],
        "missingPackageStatus": missing_package["status"],
        "sceneProbeStatus": scene_probe["runStatus"],
        "testStatus": tests["runStatus"],
        "blockedStatus": blocked["runStatus"],
        "resolvedEditor": ready["editorPath"],
    });
    write_json(&result_path, &result)?;
    Ok(result)
}

fn run_bridge(
    root: &Path,
    workspace: &Path,
    artifact_root: &Path,
    editor_roots: &Path,
    args: &[&str],
    expect_success: bool,
) -> Result<Value> {
    let bridge = native_bridge_exe()?;
    ensure_bridge_built(root, "epiphany-unity-bridge", &bridge)?;
    let output = Command::new(bridge)
        .current_dir(root)
        .env("EPIPHANY_UNITY_EDITOR_ROOTS", editor_roots)
        .arg(args[0])
        .arg("--project-path")
        .arg(workspace)
        .arg("--artifact-root")
        .arg(artifact_root)
        .args(&args[1..])
        .output()
        .context("failed to run native unity bridge")?;
    if expect_success {
        require(
            output.status.success(),
            &format!(
                "bridge should succeed, got {:?}: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            ),
        )?;
    } else {
        require(
            !output.status.success(),
            "bridge should fail for blocked runtime execution",
        )?;
    }
    serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "bridge did not return JSON: {}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn prepare_project(workspace: &Path, include_bridge: bool) -> Result<()> {
    reset_path(workspace)?;
    let settings = workspace.join("ProjectSettings");
    fs::create_dir_all(&settings)?;
    fs::write(
        settings.join("ProjectVersion.txt"),
        format!(
            "m_EditorVersion: {PROJECT_VERSION}\nm_EditorVersionWithRevision: {PROJECT_VERSION} (3c681a6c22ff)\n"
        ),
    )?;
    if include_bridge {
        let bridge = workspace
            .join("Assets")
            .join("Editor")
            .join("Epiphany")
            .join("EpiphanyEditorBridge.cs");
        fs::create_dir_all(bridge.parent().expect("bridge path has parent"))?;
        fs::write(
            bridge,
            "// smoke marker for resident Epiphany editor bridge\n",
        )?;
    }
    Ok(())
}

fn create_fake_editor(root: &Path, version: &str) -> Result<PathBuf> {
    let editor = root.join(version).join("Editor").join("Unity.exe");
    fs::create_dir_all(editor.parent().expect("editor path has parent"))?;
    fs::write(&editor, "fake unity binary for dry-run smoke only\n")?;
    Ok(editor)
}

fn read_command_artifact(summary: &Value) -> Result<Value> {
    let command_path = path_value(summary, "artifactPath")?.join("unity-command.json");
    require(command_path.exists(), "missing unity-command.json")?;
    let bytes = fs::read(&command_path)?;
    serde_json::from_slice(&bytes).context("failed to parse unity-command.json")
}

fn require_artifact(summary: &Value, name: &str) -> Result<()> {
    let artifact_path = path_value(summary, "artifactPath")?;
    require(
        artifact_path.join(name).exists(),
        &format!("missing bridge artifact: {name}"),
    )
}

fn require_command_contains(command: &[Value], value: &str) -> Result<()> {
    require(
        command.iter().any(|item| item.as_str() == Some(value)),
        &format!("command should contain {value:?}: {command:?}"),
    )
}

fn native_bridge_exe() -> Result<PathBuf> {
    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .unwrap_or_else(|| r"C:\Users\Meta\.cargo-target-codex".into());
    Ok(PathBuf::from(target_dir)
        .join("debug")
        .join(format!("epiphany-unity-bridge{}", env::consts::EXE_SUFFIX)))
}

fn ensure_bridge_built(root: &Path, bin: &str, exe: &Path) -> Result<()> {
    if exe.exists() {
        return Ok(());
    }
    let status = Command::new("cargo")
        .current_dir(root)
        .arg("build")
        .arg("--manifest-path")
        .arg(root.join("epiphany-core").join("Cargo.toml"))
        .arg("--bin")
        .arg(bin)
        .status()
        .with_context(|| format!("failed to build native {bin}"))?;
    require(
        status.success() && exe.exists(),
        &format!("native {bin} executable was not built at {}", exe.display()),
    )
}

fn value_array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("missing array field {key}"))
}

fn path_value(value: &Value, key: &str) -> Result<PathBuf> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("missing path field {key}"))
}

fn array_contains(value: Option<&Value>, needle: &str) -> bool {
    value
        .and_then(Value::as_array)
        .is_some_and(|items| items.iter().any(|item| item.as_str() == Some(needle)))
}

fn reset_path(path: &Path) -> Result<()> {
    if path.exists() {
        let root = env::current_dir()?.join(".epiphany-smoke").canonicalize()?;
        let resolved = path.canonicalize()?;
        if resolved == root || !resolved.starts_with(&root) {
            return Err(anyhow!(
                "refusing to reset non-smoke path: {}",
                path.display()
            ));
        }
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)?;
    Ok(())
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
