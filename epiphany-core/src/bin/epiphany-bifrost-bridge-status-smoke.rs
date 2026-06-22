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
    status_exe: Option<PathBuf>,
    result: PathBuf,
}

impl Args {
    fn parse() -> Result<Self> {
        let root = env::current_dir().context("failed to resolve current dir")?;
        let mut parsed = Self {
            status_exe: None,
            result: root
                .join(".epiphany-smoke")
                .join("bifrost-bridge-status-smoke.json"),
        };
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--status-exe" => parsed.status_exe = Some(take_path(&mut args, "--status-exe")?),
                "--result" => parsed.result = take_path(&mut args, "--result")?,
                other => return Err(anyhow!("unknown argument: {other}")),
            }
        }
        Ok(parsed)
    }
}

fn run_smoke(args: &Args) -> Result<Value> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let status_exe = match &args.status_exe {
        Some(path) => absolute_path(path)?,
        None => sibling_exe("epiphany-mvp-status")?,
    };
    ensure_status_built(&root, &status_exe)?;
    let status = run_native_status(&root, &status_exe)?;
    let bridge = status
        .get("bifrostBridge")
        .ok_or_else(|| anyhow!("native status did not include bifrostBridge"))?;
    validate_bridge(bridge)?;

    let result = json!({
        "status": "ok",
        "statusExe": status_exe,
        "bifrostBridge": bridge,
        "privateStateExposed": bridge["privateStateExposed"],
        "readySurfaceCount": bridge["readySurfaceCount"],
        "preparedSurfaceCount": bridge["preparedSurfaceCount"],
        "surfaceCount": bridge["surfaceCount"],
        "surfaces": bridge["surfaces"],
    });
    write_json(&absolute_path(&args.result)?, &result)?;
    Ok(result)
}

fn run_native_status(root: &Path, status_exe: &Path) -> Result<Value> {
    let output = Command::new(status_exe)
        .current_dir(root)
        .arg("--native")
        .arg("--json")
        .arg("--cwd")
        .arg(root)
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

fn validate_bridge(bridge: &Value) -> Result<()> {
    require(
        bridge.get("owner").and_then(Value::as_str) == Some("Bifrost"),
        "Bifrost bridge sight should be owned by Bifrost",
    )?;
    require(
        bridge.get("privateStateExposed").and_then(Value::as_bool) == Some(false),
        "Bifrost bridge sight should not expose private state",
    )?;
    require(
        bridge
            .get("surfaceCount")
            .and_then(Value::as_u64)
            .is_some_and(|count| count >= 5),
        "Bifrost bridge sight should expose at least five governed surfaces",
    )?;
    require(
        surface_is(bridge, "github", &["live"])
            && surface_is(bridge, "other", &["live"])
            && surface_is(bridge, "patron", &["live"])
            && surface_is(bridge, "discord", &["live", "prepared"])
            && surface_is(bridge, "reddit", &["live", "prepared"]),
        "Bifrost bridge sight should report GitHub/Other/Patron live and Discord/Reddit live or prepared",
    )
}

fn surface_is(bridge: &Value, id: &str, allowed: &[&str]) -> bool {
    bridge
        .get("surfaces")
        .and_then(Value::as_array)
        .and_then(|surfaces| {
            surfaces
                .iter()
                .find(|surface| surface.get("id").and_then(Value::as_str) == Some(id))
        })
        .and_then(|surface| surface.get("status").and_then(Value::as_str))
        .is_some_and(|surface_status| allowed.contains(&surface_status))
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
