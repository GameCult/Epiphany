use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::Utc;
use serde_json::Value;
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;
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
    smoke_root: PathBuf,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut root = env::current_dir().context("failed to resolve current directory")?;
        let mut smoke_root = root.join(".epiphany-smoke");
        let mut args = env::args().skip(1).peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--root" => root = take_path(&mut args, "--root")?,
                "--smoke-root" => smoke_root = take_path(&mut args, "--smoke-root")?,
                other => return Err(anyhow!("unexpected argument {other:?}")),
            }
        }
        Ok(Self { root, smoke_root })
    }
}

fn run_smoke(args: Args) -> Result<Value> {
    let root = args
        .root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.root.display()))?;
    let manifest = root.join("epiphany-core").join("Cargo.toml");
    if !manifest.exists() {
        return Err(anyhow!(
            "could not find epiphany-core manifest at {}",
            manifest.display()
        ));
    }
    fs::create_dir_all(&args.smoke_root)
        .with_context(|| format!("failed to create {}", args.smoke_root.display()))?;
    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let smoke_dir = args
        .smoke_root
        .join(format!("repo-work-service-lifecycle-{stamp}"));
    if smoke_dir.exists() {
        fs::remove_dir_all(&smoke_dir)
            .with_context(|| format!("failed to clear {}", smoke_dir.display()))?;
    }
    fs::create_dir_all(&smoke_dir)
        .with_context(|| format!("failed to create {}", smoke_dir.display()))?;

    let repo = smoke_dir.join("repo-body");
    fs::create_dir_all(&repo).with_context(|| format!("failed to create {}", repo.display()))?;
    run_command(Command::new("git").arg("init").current_dir(&repo), "git init")?;
    let local_verse = repo.join(".epiphany").join("local-verse.ccmp");
    let runbook_path = smoke_dir.join("epiphany-repo-work-queue-runner.ps1");
    let service_id = "epiphany-repo-work-queue-runner";
    let scheduler_id = "epiphany-repo-work-queue-runner";
    let runtime_id = "epiphany-local";
    let daemon_id = "epiphany-daemon-self";
    let repo_work_runtime_id = "repo-work-service-lifecycle-smoke";
    let epiphany_work_bin = root
        .join("epiphany-core")
        .join("target")
        .join("debug")
        .join(if cfg!(windows) {
            "epiphany-work.exe"
        } else {
            "epiphany-work"
        });
    let service_args = vec![
        "queue-run".to_string(),
        "--workspace".to_string(),
        path_string(&repo)?,
        "--epiphany-root".to_string(),
        path_string(&root)?,
        "--local-verse-store".to_string(),
        path_string(&local_verse)?,
        "--runtime-id".to_string(),
        repo_work_runtime_id.to_string(),
        "--max-items".to_string(),
        "1".to_string(),
        "--dry-run".to_string(),
    ];
    let plan = daemon_supervisor(
        &manifest,
        &root,
        &local_verse,
        &[
            "service-plan",
            "--store",
            path_str(&local_verse)?,
            "--runtime-id",
            runtime_id,
            "--daemon-id",
            daemon_id,
            "--scheduler-id",
            scheduler_id,
            "--service-id",
            service_id,
            "--service-command",
            path_str(&epiphany_work_bin)?,
            "--reason",
            "Idunn-owned lifecycle artifact for Self-owned repo work queue-run pulse.",
            "--cwd",
            path_str(&root)?,
        ],
        &service_args,
    )?;
    let runbook = daemon_supervisor(
        &manifest,
        &root,
        &local_verse,
        &[
            "service-runbook",
            "--store",
            path_str(&local_verse)?,
            "--runtime-id",
            runtime_id,
            "--daemon-id",
            daemon_id,
            "--scheduler-id",
            scheduler_id,
            "--service-id",
            service_id,
            "--service-command",
            path_str(&epiphany_work_bin)?,
            "--reason",
            "Idunn-owned lifecycle artifact for Self-owned repo work queue-run pulse.",
            "--cwd",
            path_str(&root)?,
            "--runbook-path",
            path_str(&runbook_path)?,
        ],
        &service_args,
    )?;
    let runbook_receipts = receipt_directory(&manifest, &root, &local_verse, runtime_id)?;
    let runbook_rows = rows(&runbook_receipts);
    let runbook_row = runbook_rows.iter().find(|row| {
        row.get("family").and_then(Value::as_str) == Some("service-lifecycle")
            && row.get("serviceId").and_then(Value::as_str) == Some(service_id)
            && row
                .get("serviceRoute")
                .and_then(Value::as_str)
                == Some("epiphany-repo-work-queue-runner::runbook")
    });
    let Some(runbook_row) = runbook_row else {
        return Err(anyhow!(
            "receipt directory did not expose runbook service-lifecycle row for {service_id}"
        ));
    };
    require_eq(runbook_row, &["status"], "written")?;
    require_eq(runbook_row, &["artifactStatus"], "present")?;

    let launch = daemon_supervisor(
        &manifest,
        &root,
        &local_verse,
        &[
            "service-launch",
            "--store",
            path_str(&local_verse)?,
            "--runtime-id",
            runtime_id,
            "--daemon-id",
            daemon_id,
            "--scheduler-id",
            scheduler_id,
            "--service-id",
            service_id,
            "--service-command",
            path_str(&epiphany_work_bin)?,
            "--reason",
            "Idunn-owned lifecycle launch proof for Self-owned repo work queue-run pulse.",
            "--cwd",
            path_str(&root)?,
            "--wait-child",
        ],
        &service_args,
    )?;
    let launch_receipts = receipt_directory(&manifest, &root, &local_verse, runtime_id)?;
    let launch_rows = rows(&launch_receipts);
    let launch_row = launch_rows.iter().find(|row| {
        row.get("family").and_then(Value::as_str) == Some("service-lifecycle")
            && row.get("serviceId").and_then(Value::as_str) == Some(service_id)
            && row
                .get("serviceRoute")
                .and_then(Value::as_str)
                == Some("epiphany-repo-work-queue-runner::launch")
    });
    let Some(launch_row) = launch_row else {
        return Err(anyhow!(
            "receipt directory did not expose launch service-lifecycle row for {service_id}"
        ));
    };
    require_eq(launch_row, &["status"], "completed")?;

    let audit = daemon_supervisor(
        &manifest,
        &root,
        &local_verse,
        &[
            "repo-work-service-audit",
            "--store",
            path_str(&local_verse)?,
            "--runtime-id",
            runtime_id,
            "--daemon-id",
            daemon_id,
            "--scheduler-id",
            scheduler_id,
            "--service-id",
            service_id,
            "--service-command",
            path_str(&epiphany_work_bin)?,
            "--reason",
            "Idunn-owned readiness audit for Self-owned repo work queue-run pulse.",
            "--cwd",
            path_str(&root)?,
        ],
        &service_args,
    )?;

    require_eq(&plan, &["status"], "planned")?;
    require_eq(&runbook, &["status"], "written")?;
    require_eq(&launch, &["status"], "completed")?;
    require_eq(&audit, &["status"], "complete")?;
    require_eq(&plan, &["serviceId"], service_id)?;
    require_eq(&runbook, &["serviceId"], service_id)?;
    require_eq(&launch, &["serviceId"], service_id)?;
    require_eq(&audit, &["serviceId"], service_id)?;
    require_bool(&plan, &["privateStateExposed"], false)?;
    require_bool(&runbook, &["privateStateExposed"], false)?;
    require_bool(&launch, &["privateStateExposed"], false)?;
    require_bool(&audit, &["privateStateExposed"], false)?;
    require_i64(&launch, &["exitCode"], 0)?;
    require_i64(&audit, &["launchExitCode"], 0)?;
    require_text_array_contains(&plan, &["args"], "queue-run")?;
    require_text_array_contains(&plan, &["args"], "--dry-run")?;
    require_text_array_contains(&plan, &["args"], path_str(&repo)?)?;
    require_text_array_contains(&runbook, &["args"], "queue-run")?;
    require_text_array_contains(&runbook, &["args"], "--dry-run")?;
    require_eq(&audit, &["planStatus"], "planned")?;
    require_eq(&audit, &["runbookStatus"], "written")?;
    require_eq(&audit, &["runbookArtifactStatus"], "present")?;
    require_eq(&audit, &["launchStatus"], "completed")?;

    let runbook_text = fs::read_to_string(&runbook_path)
        .with_context(|| format!("failed to read {}", runbook_path.display()))?;
    require_text(
        &runbook_text,
        "Generated by epiphany-daemon-supervisor service-runbook",
    )?;
    require_text(&runbook_text, "Start-Process")?;
    require_text(&runbook_text, "-WindowStyle Hidden")?;
    require_text(&runbook_text, "queue-run")?;
    require_text(&runbook_text, "--dry-run")?;
    let runbook_sha256 = sha256_file(&runbook_path)?;

    let audit_receipts = receipt_directory(&manifest, &root, &local_verse, runtime_id)?;
    let audit_rows = rows(&audit_receipts);
    let audit_row = audit_rows.iter().find(|row| {
        row.get("family").and_then(Value::as_str) == Some("service-lifecycle")
            && row.get("serviceId").and_then(Value::as_str) == Some(service_id)
            && row
                .get("serviceRoute")
                .and_then(Value::as_str)
                == Some("epiphany-repo-work-queue-runner::repo-work-service-audit")
    });
    let Some(audit_row) = audit_row else {
        return Err(anyhow!(
            "receipt directory did not expose audit service-lifecycle row for {service_id}"
        ));
    };
    require_eq(audit_row, &["status"], "complete")?;

    let summary = json!({
        "schemaVersion": "epiphany.repo_work_service_lifecycle_smoke.v0",
        "status": "ok",
        "smokeDir": smoke_dir,
        "repo": repo,
        "localVerseStore": local_verse,
        "serviceId": service_id,
        "schedulerId": scheduler_id,
        "daemonId": daemon_id,
        "planStatus": plan["status"],
        "planReceiptId": plan["receiptId"],
        "runbookStatus": runbook["status"],
        "runbookReceiptId": runbook["receiptId"],
        "runbookPath": runbook_path,
        "runbookSha256": runbook_sha256,
        "launchStatus": launch["status"],
        "launchReceiptId": launch["receiptId"],
        "launchExitCode": launch["exitCode"],
        "auditStatus": audit["status"],
        "auditReceiptId": audit["receiptId"],
        "auditPlanStatus": audit["planStatus"],
        "auditRunbookStatus": audit["runbookStatus"],
        "auditRunbookArtifactStatus": audit["runbookArtifactStatus"],
        "auditLaunchStatus": audit["launchStatus"],
        "auditLaunchExitCode": audit["launchExitCode"],
        "serviceCommand": plan["command"],
        "serviceArgs": plan["args"],
        "lifecycleOwner": "Idunn",
        "hostedBody": "repo-work",
        "mutatesServiceManager": false,
        "launchesService": true,
        "waitChild": true,
        "requiresElevatedAuthority": false,
        "privateStateExposed": false,
    });
    write_json(&smoke_dir.join("summary.json"), &summary)?;
    Ok(summary)
}

fn daemon_supervisor(
    manifest: &Path,
    cwd: &Path,
    _store: &Path,
    base_args: &[&str],
    service_args: &[String],
) -> Result<Value> {
    let mut args = base_args
        .iter()
        .map(|arg| (*arg).to_string())
        .collect::<Vec<_>>();
    for service_arg in service_args {
        args.push("--service-arg".to_string());
        args.push(service_arg.clone());
    }
    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    cargo_json(manifest, "epiphany-daemon-supervisor", &arg_refs, cwd)
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

fn receipt_directory(
    manifest: &Path,
    root: &Path,
    local_verse: &Path,
    runtime_id: &str,
) -> Result<Value> {
    cargo_json(
        manifest,
        "epiphany-verse-query",
        &[
            "receipt-directory",
            "--store",
            path_str(local_verse)?,
            "--runtime-id",
            runtime_id,
        ],
        root,
    )
}

fn rows(value: &Value) -> Vec<Value> {
    value
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn run_command(command: &mut Command, label: &str) -> Result<()> {
    let output = command
        .output()
        .with_context(|| format!("failed to run {label}"))?;
    if !output.status.success() {
        return Err(anyhow!(
            "{label} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
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

fn require_i64(value: &Value, path: &[&str], expected: i64) -> Result<()> {
    let actual = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_i64);
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

fn require_text_array_contains(value: &Value, path: &[&str], expected: &str) -> Result<()> {
    let values = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("expected {} to be an array", path.join(".")))?;
    if values.iter().any(|value| value.as_str() == Some(expected)) {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to contain {expected:?}, got {:?}",
            path.join("."),
            values
        ))
    }
}

fn require_text(haystack: &str, needle: &str) -> Result<()> {
    if haystack.contains(needle) {
        Ok(())
    } else {
        Err(anyhow!("expected text to contain {needle:?}"))
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

fn path_string(path: &Path) -> Result<String> {
    Ok(path_str(path)?.to_string())
}
