use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::Utc;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<()> {
    let args = Args::parse()?;
    let result = run_smoke(args)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Clone, Debug)]
struct Args {
    root: PathBuf,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut root = env::current_dir().context("failed to resolve current directory")?;
        let mut args = env::args().skip(1).peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--root" => root = take_path(&mut args, "--root")?,
                other => return Err(anyhow!("unexpected argument {other:?}")),
            }
        }
        Ok(Self { root })
    }
}

fn run_smoke(args: Args) -> Result<Value> {
    let root = args
        .root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.root.display()))?;
    let smoke_root = root.join(".epiphany-smoke");
    let manifest = root.join("epiphany-core").join("Cargo.toml");
    if !manifest.exists() {
        return Err(anyhow!(
            "could not find epiphany-core manifest at {}",
            manifest.display()
        ));
    }
    fs::create_dir_all(&smoke_root)
        .with_context(|| format!("failed to create {}", smoke_root.display()))?;
    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let smoke_dir = smoke_root.join(format!("daemon-survival-rehearsal-{stamp}"));
    fs::create_dir(&smoke_dir)
        .with_context(|| format!("failed to claim fresh smoke dir {}", smoke_dir.display()))?;

    let local_verse = smoke_dir.join("local-verse.ccmp");
    let runtime_id = "epiphany-local";
    let daemon_id = "epiphany-daemon-self";
    let scheduler_id = "epiphany-daemon-survival-rehearsal";
    let restart_command = if cfg!(windows) {
        "powershell.exe"
    } else {
        "sh"
    };
    let restart_args = if cfg!(windows) {
        vec!["-NoProfile", "-Command", "exit 0"]
    } else {
        vec!["-c", "exit 0"]
    };

    let mut policy_args = vec![
        "policy",
        "--store",
        path_str(&local_verse)?,
        "--runtime-id",
        runtime_id,
        "--daemon-id",
        daemon_id,
        "--scheduler-id",
        scheduler_id,
        "--restart-command",
        restart_command,
        "--cwd",
        path_str(&root)?,
        "--cooldown-seconds",
        "0",
        "--reconcile-interval-seconds",
        "0",
        "--heartbeat-stale-seconds",
        "3600",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();
    for arg in &restart_args {
        policy_args.push("--restart-arg".to_string());
        policy_args.push((*arg).to_string());
    }
    let policy = cargo_json(
        &manifest,
        "epiphany-daemon-supervisor",
        &string_refs(&policy_args),
        &root,
    )?;

    let serve_args = vec![
        "serve",
        "--store",
        path_str(&local_verse)?,
        "--runtime-id",
        runtime_id,
        "--daemon-id",
        daemon_id,
        "--scheduler-id",
        scheduler_id,
        "--loop-interval-seconds",
        "0",
        "--max-iterations",
        "2",
        "--force",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();
    let serve = cargo_json(
        &manifest,
        "epiphany-daemon-supervisor",
        &string_refs(&serve_args),
        &root,
    )?;

    require_eq(&policy, &["status"], "written")?;
    require_eq(&policy, &["daemonId"], daemon_id)?;
    require_bool(&policy, &["privateStateExposed"], false)?;
    require_eq(&serve, &["status"], "serveComplete")?;
    require_u64(&serve, &["iterations"], 2)?;
    require_bool(&serve, &["privateStateExposed"], false)?;
    let outputs = serve
        .get("outputs")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("serve output missing outputs array"))?;
    if outputs.len() != 2 {
        return Err(anyhow!("expected 2 serve outputs, got {}", outputs.len()));
    }
    for (idx, output) in outputs.iter().enumerate() {
        require_eq(output, &["status"], "tickComplete")?;
        require_eq(output, &["schedulerId"], scheduler_id)?;
        require_u64(output, &["iteration"], (idx + 1) as u64)?;
        require_u64(output, &["outcomeCount"], 1)?;
        require_u64(output, &["restartedCount"], 1)?;
        require_u64(output, &["refusedCount"], 0)?;
        require_bool(output, &["privateStateExposed"], false)?;
        let outcomes = output
            .get("outcomes")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow!("tick output missing outcomes array"))?;
        let outcome = outcomes
            .first()
            .ok_or_else(|| anyhow!("tick output missing first outcome"))?;
        require_eq(outcome, &["status"], "restarted")?;
        require_eq(outcome, &["daemonId"], daemon_id)?;
        require_bool(outcome, &["privateStateExposed"], false)?;
    }

    let receipt_directory = cargo_json(
        &manifest,
        "epiphany-verse-query",
        &[
            "receipt-directory",
            "--store",
            path_str(&local_verse)?,
            "--runtime-id",
            runtime_id,
        ],
        &root,
    )?;
    require_bool(&receipt_directory, &["privateStateExposed"], false)?;
    let scheduler_row = rows(&receipt_directory)
        .iter()
        .find(|row| row.get("family").and_then(Value::as_str) == Some("scheduler"))
        .cloned()
        .ok_or_else(|| anyhow!("receipt directory did not expose scheduler row"))?;
    require_eq(&scheduler_row, &["status"], "tickComplete")?;
    require_eq(&scheduler_row, &["route"], daemon_id)?;
    require_bool(&scheduler_row, &["privateStateExposed"], false)?;

    let summary = json!({
        "schemaVersion": "epiphany.daemon_survival_rehearsal_smoke.v0",
        "status": "ok",
        "smokeDir": smoke_dir,
        "localVerseStore": local_verse,
        "runtimeId": runtime_id,
        "daemonId": daemon_id,
        "schedulerId": scheduler_id,
        "policyStatus": policy["status"],
        "policyId": policy["policyId"],
        "serveStatus": serve["status"],
        "serveIterations": serve["iterations"],
        "schedulerReceiptId": scheduler_row["latestId"],
        "schedulerReceiptStatus": scheduler_row["status"],
        "restartCommand": restart_command,
        "restartArgs": restart_args,
        "boundedProofMode": true,
        "serviceManagerMutated": false,
        "requiresElevatedAuthority": false,
        "unattendedDaemonSurvivalRehearsed": true,
        "privateStateExposed": false,
    });
    write_json(&smoke_dir.join("summary.json"), &summary)?;
    Ok(summary)
}

fn cargo_json(manifest: &Path, bin: &str, args: &[&str], cwd: &Path) -> Result<Value> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(manifest)
        .arg("--bin")
        .arg(bin)
        .arg("--")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to run cargo bin {bin}"))?;
    if !output.status.success() {
        return Err(anyhow!(
            "{bin} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "{bin} did not emit JSON on stdout:\n{}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn string_refs(args: &[String]) -> Vec<&str> {
    args.iter().map(String::as_str).collect()
}

fn rows(value: &Value) -> Vec<&Value> {
    value
        .get("rows")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().collect())
        .unwrap_or_default()
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn require_eq(value: &Value, path: &[&str], expected: &str) -> Result<()> {
    let actual = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_str)
        .unwrap_or("<missing>");
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to be {expected:?}, got {actual:?}",
            path.join(".")
        ))
    }
}

fn require_bool(value: &Value, path: &[&str], expected: bool) -> Result<()> {
    let actual = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_bool);
    if actual == Some(expected) {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to be {expected}, got {:?}",
            path.join("."),
            actual
        ))
    }
}

fn require_u64(value: &Value, path: &[&str], expected: u64) -> Result<()> {
    let actual = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_u64);
    if actual == Some(expected) {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to be {expected}, got {:?}",
            path.join("."),
            actual
        ))
    }
}

fn take_path(
    args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    flag: &str,
) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .with_context(|| format!("missing value for {flag}"))
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| anyhow!("path is not valid UTF-8: {}", path.display()))
}
