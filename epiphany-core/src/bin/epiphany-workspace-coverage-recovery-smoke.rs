use anyhow::{Context, Result, anyhow, bail};
use epiphany_core::{
    ProcessInstanceObservation, RuntimeSpineInitOptions,
    WorkspaceCoverageManagedProcessLaunchEntry, authenticate_workspace_coverage_recovery_receipt,
    bind_repository_body, bind_runtime_to_agent_memory_swarm,
    current_workspace_coverage_recovery_target, ensure_agent_memory_swarm_identity,
    initialize_runtime_spine, load_latest_workspace_coverage_managed_process_launch,
    observe_process_instance, observe_repository_body,
    process_identity_from_workspace_coverage_launch, seed_epiphany_local_verse_context,
    workspace_coverage_execution_collection,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

const RUNTIME_ID: &str = "workspace-coverage-recovery-smoke";
const SERVICE_ID: &str = "epiphany-workspace-coverage-projector-service";
const SMOKE_SOURCE: &[u8] = include_bytes!("epiphany-workspace-coverage-recovery-smoke.rs");

fn main() -> Result<()> {
    let smoke_id = Uuid::new_v4().to_string();
    let root = std::env::temp_dir().join(format!("epiphany-workspace-coverage-{smoke_id}"));
    fs::create_dir_all(&root)?;
    let repo = root.join("repo");
    let runtime = root.join("runtime.ccmp");
    let agents = root.join("agents.ccmp");
    let body = root.join("body.ccmp");
    let verse = root.join("verse.ccmp");
    let proof_path = root.join("proof.json");
    let qdrant_url = std::env::var("EPIPHANY_SMOKE_QDRANT_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:6333".into());
    let ollama_url = std::env::var("EPIPHANY_SMOKE_OLLAMA_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:11434".into());
    let model = std::env::var("EPIPHANY_SMOKE_OLLAMA_MODEL")
        .unwrap_or_else(|_| "qwen3-embedding:0.6b".into());
    require_http_ok(&format!("{qdrant_url}/collections"))?;
    let mut qdrant_cleanup = QdrantCollectionCleanup {
        base_url: qdrant_url.clone(),
        owned: BTreeSet::new(),
    };
    require_http_ok(&format!("{ollama_url}/api/tags"))?;
    prewarm_ollama(&ollama_url, &model)?;
    eprintln!("smoke: endpoints ready");
    create_fixture_repo(&repo)?;
    initialize_runtime_spine(
        &runtime,
        RuntimeSpineInitOptions {
            runtime_id: RUNTIME_ID.into(),
            display_name: "workspace coverage recovery smoke".into(),
            created_at: chrono::Utc::now().to_rfc3339(),
        },
    )?;
    ensure_agent_memory_swarm_identity(&agents, &format!("smoke-{smoke_id}"))?;
    bind_runtime_to_agent_memory_swarm(&runtime, &agents, &chrono::Utc::now().to_rfc3339())?;
    bind_repository_body(&repo, &body, &runtime, &format!("smoke-{smoke_id}"))?;
    observe_repository_body(&repo, &body, &runtime)?;
    seed_epiphany_local_verse_context(&verse, RUNTIME_ID, chrono::Utc::now().to_rfc3339())?;
    let _process_cleanup = ManagedProcessCleanup {
        verse: verse.clone(),
    };

    let supervisor = sibling_binary("epiphany-daemon-supervisor")?;
    run_supervisor(
        &supervisor,
        &[
            "workspace-coverage-projector-service-policy",
            "--store",
            path(&verse)?,
            "--runtime-id",
            RUNTIME_ID,
            "--daemon-id",
            "epiphany-daemon-supervisor",
            "--runtime-store",
            path(&runtime)?,
            "--qdrant-url",
            &qdrant_url,
            "--ollama-base-url",
            &ollama_url,
            "--ollama-model",
            &model,
            "--loop-interval-seconds",
            "30",
            "--stdout-artifact",
            path(&root.join("projector.stdout.log"))?,
            "--stderr-artifact",
            path(&root.join("projector.stderr.log"))?,
        ],
    )?;
    eprintln!("smoke: policy persisted");
    run_supervisor(&supervisor, reconcile_args(&verse, &runtime)?)?;
    eprintln!("smoke: initial launch persisted");

    let first = wait_for_claim(&runtime, 1, Duration::from_secs(90))?;
    qdrant_cleanup
        .owned
        .insert(workspace_coverage_execution_collection(
            &first.plan_id,
            &first.claim_id,
            first.claim_epoch,
        )?);
    eprintln!(
        "smoke: initial claim acquired at epoch {}",
        first.claim_epoch
    );
    let first_launch = load_latest_workspace_coverage_managed_process_launch(&verse, RUNTIME_ID)?
        .context("initial managed launch disappeared")?;
    if first.managed_process_launch_id != first_launch.launch_id {
        bail!("initial claim disagrees with latest managed launch");
    }
    kill_process(&first_launch)?;
    wait_for_process_exit(&first_launch, Duration::from_secs(20))?;
    eprintln!("smoke: initial process killed");

    let recovery_output = run_supervisor(&supervisor, reconcile_args(&verse, &runtime)?)?;
    let recovery_json: Value = serde_json::from_str(&recovery_output)
        .context("supervisor recovery output was not JSON")?;
    if recovery_json.get("status").and_then(Value::as_str) != Some("recovered") {
        bail!("supervisor did not recover workspace coverage: {recovery_output}");
    }
    let recovery_id = required_json_string(&recovery_json, "recoveryReceiptId")?;
    let recovery_digest = required_json_string(&recovery_json, "recoveryReceiptDigest")?;
    eprintln!("smoke: supervisor recovery returned");
    let second = wait_for_claim(&runtime, 2, Duration::from_secs(20))?;
    qdrant_cleanup
        .owned
        .insert(workspace_coverage_execution_collection(
            &second.plan_id,
            &second.claim_id,
            second.claim_epoch,
        )?);
    if second.claim_id == first.claim_id
        || second.claim_epoch != first.claim_epoch + 1
        || second.managed_process_launch_id == first.managed_process_launch_id
    {
        bail!("recovered Body authority did not advance exactly once");
    }
    let replacement = load_latest_workspace_coverage_managed_process_launch(&verse, RUNTIME_ID)?
        .context("replacement managed launch disappeared")?;
    authenticate_workspace_coverage_recovery_receipt(
        &body,
        &verse,
        RUNTIME_ID,
        epiphany_core::open_default_host_identity()?.entry(),
        recovery_id,
        recovery_digest,
    )?;
    kill_process(&replacement)?;
    qdrant_cleanup.cleanup_and_verify()?;

    let executable_sha256 = hash_file(&std::env::current_exe()?)?;
    let smoke_source_sha256 = hash_bytes(SMOKE_SOURCE);
    let source_head = git_output(&["rev-parse", "HEAD"])?;
    let source_diff_sha256 = hash_bytes(&git_bytes(&["diff", "--binary", "HEAD"])?);

    let proof = json!({
        "schemaVersion": "epiphany.workspace_coverage_recovery_smoke.v0",
        "smokeId": smoke_id,
        "status": "passed",
        "runtimeId": RUNTIME_ID,
        "initialClaimId": first.claim_id,
        "initialClaimEpoch": first.claim_epoch,
        "initialLaunchId": first.managed_process_launch_id,
        "recoveredClaimId": second.claim_id,
        "recoveredClaimEpoch": second.claim_epoch,
        "replacementLaunchId": second.managed_process_launch_id,
        "recoveryReceiptId": recovery_id,
        "recoveryReceiptDigest": recovery_digest,
        "executableSha256": executable_sha256,
        "smokeSourceSha256": smoke_source_sha256,
        "sourceGitHead": source_head,
        "sourceDiffSha256": source_diff_sha256,
        "ownedCollectionsRemoved": true,
        "bodyStore": body,
        "verseStore": verse,
        "privateStateExposed": false
    });
    fs::write(&proof_path, serde_json::to_vec_pretty(&proof)?)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": "passed",
            "proof": proof_path,
            "recoveryReceiptId": recovery_id,
            "initialEpoch": first.claim_epoch,
            "recoveredEpoch": second.claim_epoch,
            "privateStateExposed": false
        }))?
    );
    Ok(())
}

fn create_fixture_repo(repo: &Path) -> Result<()> {
    fs::create_dir_all(repo)?;
    run(Command::new("git")
        .arg("init")
        .arg("-b")
        .arg("main")
        .current_dir(repo))?;
    for file_index in 0..160 {
        let mut text = String::new();
        for line in 0..120 {
            text.push_str(&format!(
                "pub fn smoke_{file_index}_{line}() -> usize {{ {} }}\n",
                file_index + line
            ));
        }
        fs::write(repo.join(format!("body_{file_index:03}.rs")), text)?;
    }
    run(Command::new("git").arg("add").arg(".").current_dir(repo))?;
    Ok(())
}

fn reconcile_args(verse: &Path, runtime: &Path) -> Result<Vec<String>> {
    Ok(vec![
        "managed-service-reconcile".into(),
        "--store".into(),
        path(verse)?.into(),
        "--runtime-id".into(),
        RUNTIME_ID.into(),
        "--daemon-id".into(),
        "epiphany-daemon-supervisor".into(),
        "--service-id".into(),
        SERVICE_ID.into(),
        "--runtime-store".into(),
        path(runtime)?.into(),
        "--force".into(),
    ])
}

fn wait_for_claim(
    runtime: &Path,
    minimum_epoch: u64,
    timeout: Duration,
) -> Result<epiphany_core::WorkspaceCoverageRecoveryTarget> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(target) = current_workspace_coverage_recovery_target(runtime)?
            && target.claim_epoch >= minimum_epoch
        {
            return Ok(target);
        }
        thread::sleep(Duration::from_millis(20));
    }
    Err(anyhow!(
        "workspace coverage claim epoch {minimum_epoch} did not appear"
    ))
}

fn run_supervisor<I, S>(supervisor: &Path, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let command_id = Uuid::new_v4();
    let stdout_path = std::env::temp_dir().join(format!("epiphany-smoke-{command_id}.stdout"));
    let stderr_path = std::env::temp_dir().join(format!("epiphany-smoke-{command_id}.stderr"));
    let stdout = fs::File::create(&stdout_path)?;
    let stderr = fs::File::create(&stderr_path)?;
    let status = Command::new(supervisor)
        .args(args)
        .env("EPIPHANY_WORKSPACE_COVERAGE_SMOKE_DIAGNOSTICS", "1")
        .env("EPIPHANY_OLLAMA_TIMEOUT_MS", "120000")
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .status()?;
    let stdout = fs::read_to_string(&stdout_path)?;
    let stderr = fs::read_to_string(&stderr_path)?;
    let _ = fs::remove_file(stdout_path);
    let _ = fs::remove_file(stderr_path);
    if !status.success() {
        bail!("supervisor failed: {stderr}");
    }
    Ok(stdout)
}

fn run(command: &mut Command) -> Result<()> {
    let output = command.output()?;
    if !output.status.success() {
        bail!(
            "command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

fn sibling_binary(name: &str) -> Result<PathBuf> {
    let file = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.into()
    };
    let path = std::env::current_exe()?.with_file_name(file);
    if !path.is_file() {
        bail!("required sibling binary is absent: {}", path.display());
    }
    Ok(path)
}

fn path(value: &Path) -> Result<&str> {
    value.to_str().context("smoke path is not UTF-8")
}

fn required_json_string<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .with_context(|| format!("recovery output lacks {key}"))
}

fn require_http_ok(url: &str) -> Result<()> {
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        bail!("required endpoint is unavailable: {url}");
    }
    Ok(())
}

fn prewarm_ollama(base_url: &str, model: &str) -> Result<()> {
    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?
        .post(format!("{base_url}/api/embed"))
        .json(&json!({"model": model, "input": ["epiphany live smoke warmup"]}))
        .send()?;
    if !response.status().is_success() {
        bail!("Ollama smoke warmup failed with {}", response.status());
    }
    Ok(())
}

struct ManagedProcessCleanup {
    verse: PathBuf,
}

struct QdrantCollectionCleanup {
    base_url: String,
    owned: BTreeSet<String>,
}

impl QdrantCollectionCleanup {
    fn cleanup_and_verify(&mut self) -> Result<()> {
        let client = reqwest::blocking::Client::new();
        let names = self.owned.iter().cloned().collect::<Vec<_>>();
        for name in names {
            let response = client
                .delete(format!("{}/collections/{name}", self.base_url))
                .send()?;
            if !response.status().is_success()
                && response.status() != reqwest::StatusCode::NOT_FOUND
            {
                bail!("Qdrant refused fixture collection deletion for {name}");
            }
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                let status = client
                    .get(format!("{}/collections/{name}", self.base_url))
                    .send()?
                    .status();
                if status == reqwest::StatusCode::NOT_FOUND {
                    self.owned.remove(&name);
                    break;
                }
                if Instant::now() >= deadline {
                    bail!("fixture collection {name} remained after deletion");
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
        Ok(())
    }
}

impl Drop for QdrantCollectionCleanup {
    fn drop(&mut self) {
        let client = reqwest::blocking::Client::new();
        for name in &self.owned {
            let _ = client
                .delete(format!("{}/collections/{name}", self.base_url))
                .send();
        }
    }
}

impl Drop for ManagedProcessCleanup {
    fn drop(&mut self) {
        if let Ok(Some(launch)) =
            load_latest_workspace_coverage_managed_process_launch(&self.verse, RUNTIME_ID)
        {
            let _ = kill_process(&launch);
        }
    }
}

#[cfg(windows)]
fn kill_native_process(pid: u32) -> Result<()> {
    let status = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if !status.success() {
        bail!("taskkill failed for PID {pid}");
    }
    Ok(())
}

#[cfg(unix)]
fn kill_native_process(pid: u32) -> Result<()> {
    let status = Command::new("kill")
        .args(["-KILL", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if !status.success() {
        bail!("kill failed for PID {pid}");
    }
    Ok(())
}

fn kill_process(launch: &WorkspaceCoverageManagedProcessLaunchEntry) -> Result<()> {
    let expected = process_identity_from_workspace_coverage_launch(launch);
    match observe_process_instance(&expected) {
        ProcessInstanceObservation::ExactAlive => kill_native_process(expected.process_id),
        ProcessInstanceObservation::ExactExited { .. } | ProcessInstanceObservation::Missing => {
            bail!("managed launch {} already exited", launch.launch_id)
        }
        ProcessInstanceObservation::Replaced { .. } => {
            bail!(
                "managed launch {} PID has been reused; refusing kill",
                launch.launch_id
            )
        }
        ProcessInstanceObservation::Inaccessible => {
            bail!(
                "managed launch {} is inaccessible; refusing kill",
                launch.launch_id
            )
        }
        ProcessInstanceObservation::Indeterminate { reason } => {
            bail!(
                "managed launch {} is indeterminate: {reason}",
                launch.launch_id
            )
        }
    }
}

fn wait_for_process_exit(
    launch: &WorkspaceCoverageManagedProcessLaunchEntry,
    timeout: Duration,
) -> Result<()> {
    let expected = process_identity_from_workspace_coverage_launch(launch);
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        match observe_process_instance(&expected) {
            ProcessInstanceObservation::ExactAlive => {}
            ProcessInstanceObservation::ExactExited { .. }
            | ProcessInstanceObservation::Missing => return Ok(()),
            ProcessInstanceObservation::Replaced { .. } => {
                bail!(
                    "managed launch {} PID was reused while awaiting exit",
                    launch.launch_id
                )
            }
            ProcessInstanceObservation::Inaccessible => {
                bail!("managed launch {} became inaccessible", launch.launch_id)
            }
            ProcessInstanceObservation::Indeterminate { reason } => {
                bail!(
                    "managed launch {} became indeterminate: {reason}",
                    launch.launch_id
                )
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    bail!("managed launch {} did not exit", launch.launch_id)
}

fn hash_file(path: &Path) -> Result<String> {
    Ok(hash_bytes(&fs::read(path)?))
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("sha256-{:x}", Sha256::digest(bytes))
}

fn git_bytes(args: &[&str]) -> Result<Vec<u8>> {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .context("epiphany-core has no repository parent")?;
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()?;
    if !output.status.success() {
        bail!(
            "git provenance command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(output.stdout)
}

fn git_output(args: &[&str]) -> Result<String> {
    Ok(String::from_utf8(git_bytes(args)?)?.trim().to_string())
}
