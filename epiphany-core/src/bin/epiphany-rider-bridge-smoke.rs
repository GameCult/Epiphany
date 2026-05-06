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
    let result = run_smoke()?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn run_smoke() -> Result<Value> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let workspace = root.join(".epiphany-smoke").join("rider-bridge-workspace");
    let artifact_root = root.join(".epiphany-smoke").join("rider-bridge-artifacts");
    let fake_rider_root = root.join(".epiphany-smoke").join("rider-bridge-install");
    let result_path = root
        .join(".epiphany-smoke")
        .join("rider-bridge-smoke-result.json");
    let source = prepare_workspace(&workspace)?;
    reset_path(&artifact_root)?;
    reset_path(&fake_rider_root)?;
    let fake_rider = create_fake_rider(&fake_rider_root)?;

    let status = run_bridge(&root, &workspace, &artifact_root, &fake_rider, &["status"])?;
    require(
        status["status"] == "ready",
        "fake Rider path should make status ready",
    )?;
    require(
        status["riderPath"] == fake_rider.to_string_lossy().as_ref(),
        "status should resolve fake Rider path",
    )?;
    require(
        status
            .get("solutionPath")
            .and_then(Value::as_str)
            .is_some_and(|path| path.ends_with("Aetheria.sln")),
        "status should find the workspace solution",
    )?;
    require_artifact(&status, "rider-bridge-summary.json")?;
    require_artifact(&status, "rider-bridge-status.md")?;

    let context = run_bridge(
        &root,
        &workspace,
        &artifact_root,
        &fake_rider,
        &[
            "context",
            "--file",
            source.to_str().unwrap_or_default(),
            "--selection-start",
            "3",
            "--selection-end",
            "5",
            "--symbol-name",
            "GravityTileRenderer",
            "--symbol-kind",
            "class",
            "--symbol-namespace",
            "Aetheria.Rendering",
        ],
    )?;
    require(
        context["status"] == "captured",
        "context command should capture a packet",
    )?;
    require(
        context["filePath"] == "Assets/Scripts/GravityTileRenderer.cs",
        "context file should be project-relative",
    )?;
    require_artifact(&context, "rider-context.json")?;
    require_artifact(&context, "rider-context.md")?;

    let open_ref = run_bridge(
        &root,
        &workspace,
        &artifact_root,
        &fake_rider,
        &[
            "open-ref",
            "--file",
            source.to_str().unwrap_or_default(),
            "--line",
            "3",
        ],
    )?;
    require(
        open_ref["status"] == "planned",
        "open-ref should plan without launch",
    )?;
    require(
        open_ref
            .get("command")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(Value::as_str)
            == Some(fake_rider.to_str().unwrap_or_default()),
        "open-ref should use discovered Rider path",
    )?;
    require_artifact(&open_ref, "rider-open-ref.json")?;

    let result = json!({
        "workspace": workspace,
        "artifactRoot": artifact_root,
        "status": status["status"],
        "contextStatus": context["status"],
        "openRefStatus": open_ref["status"],
        "riderPath": status["riderPath"],
    });
    write_json(&result_path, &result)?;
    Ok(result)
}

fn run_bridge(
    root: &Path,
    workspace: &Path,
    artifact_root: &Path,
    rider_path: &Path,
    args: &[&str],
) -> Result<Value> {
    let bridge = native_bridge_exe()?;
    ensure_bridge_built(root, "epiphany-rider-bridge", &bridge)?;
    let output = Command::new(bridge)
        .current_dir(root)
        .env("EPIPHANY_RIDER_PATH", rider_path)
        .arg(args[0])
        .arg("--project-root")
        .arg(workspace)
        .arg("--artifact-root")
        .arg(artifact_root)
        .args(&args[1..])
        .output()
        .context("failed to run native rider bridge")?;
    require(
        output.status.success(),
        &format!(
            "rider bridge failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ),
    )?;
    serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "rider bridge did not return JSON: {}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn prepare_workspace(workspace: &Path) -> Result<PathBuf> {
    reset_path(workspace)?;
    fs::write(
        workspace.join("Aetheria.sln"),
        "Microsoft Visual Studio Solution File\n",
    )?;
    let source = workspace
        .join("Assets")
        .join("Scripts")
        .join("GravityTileRenderer.cs");
    fs::create_dir_all(source.parent().expect("source path has parent"))?;
    fs::write(
        &source,
        "namespace Aetheria.Rendering\n{\n    public sealed class GravityTileRenderer\n    {\n    }\n}\n",
    )?;
    Ok(source)
}

fn create_fake_rider(root: &Path) -> Result<PathBuf> {
    let rider = root
        .join("JetBrains Rider 2026.1.0.1")
        .join("bin")
        .join("rider64.exe");
    fs::create_dir_all(rider.parent().expect("rider path has parent"))?;
    fs::write(&rider, "fake rider binary for smoke planning only\n")?;
    Ok(rider)
}

fn require_artifact(summary: &Value, name: &str) -> Result<()> {
    let artifact_path = summary
        .get("artifactPath")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("missing artifactPath"))?;
    require(
        artifact_path.join(name).exists(),
        &format!("missing rider artifact: {name}"),
    )
}

fn native_bridge_exe() -> Result<PathBuf> {
    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .unwrap_or_else(|| r"C:\Users\Meta\.cargo-target-codex".into());
    Ok(PathBuf::from(target_dir)
        .join("debug")
        .join(format!("epiphany-rider-bridge{}", env::consts::EXE_SUFFIX)))
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
