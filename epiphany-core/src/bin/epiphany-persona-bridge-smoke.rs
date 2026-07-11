use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const REQUIRED_BINS: &[&str] = &[
    "epiphany-bifrost-bridge-status-smoke",
    "epiphany-persona-discord",
    "epiphany-persona-reddit",
    "epiphany-persona-other",
];

fn main() -> Result<()> {
    let args = Args::parse()?;
    let result = run_smoke(&args)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Debug)]
struct Args {
    result: PathBuf,
}

impl Args {
    fn parse() -> Result<Self> {
        let root = env::current_dir().context("failed to resolve current dir")?;
        let mut parsed = Self {
            result: root
                .join(".epiphany-smoke")
                .join("persona-bridge-smoke.json"),
        };
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--result" => parsed.result = take_path(&mut args, "--result")?,
                other => return Err(anyhow!("unknown argument: {other}")),
            }
        }
        Ok(parsed)
    }
}

fn run_smoke(args: &Args) -> Result<Value> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    ensure_bins_built(&root)?;

    let readiness = run_json(&root, "epiphany-bifrost-bridge-status-smoke", &[])?;
    let discord = run_json(&root, "epiphany-persona-discord", &["smoke"])?;
    let reddit = run_json(&root, "epiphany-persona-reddit", &["smoke"])?;
    let other = run_json(&root, "epiphany-persona-other", &["smoke"])?;

    validate_readiness(&readiness)?;
    validate_persona_surface(
        "discord",
        &discord,
        "bifrost.discord-post",
        "heimdall:discord:",
    )?;
    validate_persona_surface("reddit", &reddit, "bifrost.reddit-post", "heimdall:reddit:")?;
    validate_persona_surface(
        "other",
        &other,
        "bifrost.other-request",
        "heimdall:bluesky:",
    )?;
    require(
        other
            .pointer("/bridged/bifrostBridgeReceipt/surfaceName")
            .and_then(Value::as_str)
            == Some("bluesky"),
        "future-surface Persona bridge smoke should name the Bifrost target surface",
    )?;

    let result = json!({
        "status": "ok",
        "readiness": bridge_summary(&readiness),
        "surfaces": [
            persona_summary("discord", &discord),
            persona_summary("reddit", &reddit),
            persona_summary("other", &other),
        ],
        "privateStateExposed": false,
        "note": "Aggregate proof only: Bifrost owns outside-world crossings and receipts; Epiphany owns Persona mouth eligibility and consumes readiness sight.",
    });
    write_json(&absolute_path(&args.result)?, &result)?;
    Ok(result)
}

fn run_json(root: &Path, bin_name: &str, args: &[&str]) -> Result<Value> {
    let output = Command::new(sibling_exe(bin_name)?)
        .current_dir(root)
        .args(args)
        .output()
        .with_context(|| format!("failed to run {bin_name}"))?;
    if !output.status.success() {
        return Err(anyhow!(
            "{bin_name} failed: {}{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("failed to decode {bin_name} JSON output"))
}

fn validate_readiness(value: &Value) -> Result<()> {
    require(
        value.get("status").and_then(Value::as_str) == Some("ok"),
        "Bifrost readiness smoke should pass",
    )?;
    require(
        value
            .pointer("/bifrostBridge/owner")
            .and_then(Value::as_str)
            == Some("Bifrost"),
        "Bifrost readiness smoke should name Bifrost as owner",
    )?;
    require(
        value
            .pointer("/bifrostBridge/privateStateExposed")
            .and_then(Value::as_bool)
            == Some(false),
        "Bifrost readiness smoke should seal private state",
    )?;
    require(
        surface_is(value, "github", &["live"])
            && surface_is(value, "other", &["live"])
            && surface_is(value, "patron", &["live"])
            && surface_is(value, "discord", &["live", "prepared"])
            && surface_is(value, "reddit", &["live", "prepared"]),
        "Bifrost readiness smoke should expose governed bridge surfaces",
    )
}

fn validate_persona_surface(
    name: &str,
    value: &Value,
    expected_transport: &str,
    expected_heimdall_prefix: &str,
) -> Result<()> {
    require(
        value.get("ok").and_then(Value::as_bool) == Some(true),
        &format!("{name} Persona smoke should pass"),
    )?;
    require(
        value
            .pointer("/missingBridge/blocked")
            .and_then(Value::as_str)
            == Some("missing-bifrost-bridge"),
        &format!("{name} Persona should stop without Bifrost bridge configuration"),
    )?;
    require(
        value
            .pointer("/missingCapability/blocked")
            .and_then(Value::as_str)
            == Some("missing-heimdall-capability-ref"),
        &format!("{name} Persona should stop without Heimdall capability proof"),
    )?;
    require(
        value
            .pointer("/wrongSurfaceCapability/blocked")
            .and_then(Value::as_str)
            == Some("wrong-heimdall-capability-surface"),
        &format!("{name} Persona should stop wrong-surface Heimdall proof"),
    )?;
    require(
        value.pointer("/bridged/transport").and_then(Value::as_str) == Some(expected_transport),
        &format!("{name} Persona should cross only through Bifrost transport"),
    )?;
    require(
        value
            .pointer("/bridged/bifrostBridgeReceipt/provenance/bifrostIdentity")
            .and_then(Value::as_str)
            == Some("epiphany.Persona"),
        &format!("{name} Persona bridge receipt should carry Bifrost identity"),
    )?;
    let heimdall_ref = value
        .pointer("/bridged/bifrostBridgeReceipt/provenance/heimdallCapabilityRef")
        .and_then(Value::as_str)
        .unwrap_or("");
    require(
        heimdall_ref.starts_with(expected_heimdall_prefix),
        &format!("{name} Persona bridge receipt should carry target-shaped Heimdall proof"),
    )?;
    require(
        value
            .pointer("/latestCultMeshSpeechAudit")
            .and_then(Value::as_array)
            .and_then(|row| row.get(18))
            .and_then(Value::as_bool)
            == Some(false),
        &format!("{name} Persona speech audit mirror should seal private state"),
    )
}

fn bridge_summary(value: &Value) -> Value {
    json!({
        "status": value.pointer("/bifrostBridge/status"),
        "readySurfaceCount": value.pointer("/bifrostBridge/readySurfaceCount"),
        "preparedSurfaceCount": value.pointer("/bifrostBridge/preparedSurfaceCount"),
        "surfaceCount": value.pointer("/bifrostBridge/surfaceCount"),
        "surfaces": value.pointer("/bifrostBridge/surfaces"),
    })
}

fn persona_summary(name: &str, value: &Value) -> Value {
    json!({
        "surface": name,
        "transport": value.pointer("/bridged/transport"),
        "bifrostIdentity": value.pointer("/bridged/bifrostBridgeReceipt/provenance/bifrostIdentity"),
        "heimdallCapabilityRef": value.pointer("/bridged/bifrostBridgeReceipt/provenance/heimdallCapabilityRef"),
        "missingBridge": value.pointer("/missingBridge/blocked"),
        "missingCapability": value.pointer("/missingCapability/blocked"),
        "wrongSurfaceCapability": value.pointer("/wrongSurfaceCapability/blocked"),
        "speechAuditPrivateStateExposed": value
            .pointer("/latestCultMeshSpeechAudit")
            .and_then(Value::as_array)
            .and_then(|row| row.get(18))
            .cloned()
            .unwrap_or(Value::Null),
    })
}

fn surface_is(value: &Value, id: &str, allowed: &[&str]) -> bool {
    value
        .pointer("/bifrostBridge/surfaces")
        .and_then(Value::as_array)
        .and_then(|surfaces| {
            surfaces
                .iter()
                .find(|surface| surface.get("id").and_then(Value::as_str) == Some(id))
        })
        .and_then(|surface| surface.get("status").and_then(Value::as_str))
        .is_some_and(|status| allowed.contains(&status))
}

fn ensure_bins_built(root: &Path) -> Result<()> {
    let mut missing = Vec::new();
    for bin in REQUIRED_BINS {
        if !sibling_exe(bin)?.exists() {
            missing.push(*bin);
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    let mut command = Command::new("cargo");
    command
        .current_dir(root)
        .arg("build")
        .arg("--manifest-path")
        .arg(root.join("epiphany-core").join("Cargo.toml"));
    for bin in missing {
        command.arg("--bin").arg(bin);
    }
    let status = command
        .status()
        .context("failed to build Persona bridge smoke dependencies")?;
    require(
        status.success(),
        "failed to build Persona bridge smoke dependencies",
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
