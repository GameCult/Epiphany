use anyhow::{Context, Result, bail};
use chrono::Utc;
use epiphany_core::*;
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::{BufRead, BufReader},
    net::TcpListener,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use uuid::Uuid;

const RUNTIME: &str = "semantic-recovery-smoke";
const PROJECTOR: &str = "epiphany-memory-semantic-projector";
const SERVICE: &str = "epiphany-memory-semantic-projector-service";

fn main() -> Result<()> {
    let root = env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| {
        env::temp_dir().join(format!("epiphany-semantic-recovery-{}", Uuid::new_v4()))
    });
    if root.exists() {
        bail!("fixture root already exists: {}", root.display());
    }
    fs::create_dir_all(&root)?;
    let proof_path = root.join("proof.json");
    let result = run(&root);
    if let Err(error) = &result {
        fs::write(
            &proof_path,
            serde_json::to_vec_pretty(&json!({
                "schemaVersion":"epiphany.semantic_recovery_smoke_proof.v1", "status":"failed",
                "fixtureRoot":root, "error":format!("{error:#}"), "liveCanonicalInputsUsed":false
            }))?,
        )?;
    }
    result
}

fn run(root: &Path) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    let swarm = format!("smoke-{id}");
    let mind = root.join("mind.cc");
    let modeling = root.join("modeling.cc");
    let verse = root.join("verse.cc");
    seed_mind(root, &mind, &swarm)?;
    seed_modeling(&modeling, &mind, &swarm)?;
    let input = agent_memory_semantic_projection_input(&mind)?;
    let modeling_input = runtime_modeling_semantic_projection_input(&modeling)?;

    let exe_dir = env::current_exe()?
        .parent()
        .context("smoke executable has no directory")?
        .to_path_buf();
    let projector = sibling(&exe_dir, PROJECTOR);
    let supervisor = sibling(&exe_dir, "epiphany-daemon-supervisor");
    for path in [&projector, &supervisor] {
        if !path.is_file() {
            bail!("required sibling executable missing: {}", path.display());
        }
    }
    let receipt_id = Uuid::new_v4().to_string();
    let qdrant =
        env::var("EPIPHANY_QDRANT_URL").unwrap_or_else(|_| "http://127.0.0.1:16333".into());
    let ollama =
        env::var("EPIPHANY_OLLAMA_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".into());
    let model = env::var("EPIPHANY_OLLAMA_MODEL").unwrap_or_else(|_| "qwen3-embedding:0.6b".into());
    let mind_collection = format!("epiphany_smoke_mind_{}", id.replace('-', ""));
    let modeling_collection = format!("epiphany_smoke_modeling_{}", id.replace('-', ""));
    for collection in [&mind_collection, &modeling_collection] {
        if collection_exists(&qdrant, collection)? {
            bail!("refused fixture collection collision: {collection}");
        }
    }
    let mut collections = FixtureCollections::new(
        qdrant.clone(),
        vec![mind_collection.clone(), modeling_collection.clone()],
    );
    let stalled = StalledHttpEndpoint::new()?;
    let old_args = vec![
        "pulse".into(),
        "--agent-store".into(),
        s(&mind),
        "--runtime-store".into(),
        s(&modeling),
        "--local-verse-store".into(),
        s(&verse),
        "--runtime-id".into(),
        RUNTIME.into(),
        "--interval-seconds".into(),
        "1".into(),
        "--qdrant-url".into(),
        qdrant.clone(),
        "--ollama-base-url".into(),
        stalled.url(),
        "--ollama-model".into(),
        model.clone(),
    ];
    let mut old_child = ChildGuard(
        Command::new(&projector)
            .args(&old_args)
            .env("EPIPHANY_MIND_QDRANT_COLLECTION", &mind_collection)
            .env("EPIPHANY_MODELING_QDRANT_COLLECTION", &modeling_collection)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()?,
    );
    let old = wait_for_running_claim(&mind, &input, Duration::from_secs(15))?;
    stop(&mut old_child);
    let old_pid = old_child.id();
    let args = vec![
        "serve".into(),
        "--agent-store".into(),
        s(&mind),
        "--runtime-store".into(),
        s(&modeling),
        "--local-verse-store".into(),
        s(&verse),
        "--runtime-id".into(),
        RUNTIME.into(),
        "--interval-seconds".into(),
        "3".into(),
        "--qdrant-url".into(),
        qdrant.clone(),
        "--ollama-base-url".into(),
        ollama.clone(),
        "--ollama-model".into(),
        model,
    ];
    let policy = EpiphanyCultMeshManagedServicePolicyEntry {
        schema_version: EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION.into(),
        policy_id: "managed-service-policy-epiphany-memory-semantic-projector-service".into(),
        service_id: SERVICE.into(),
        owner_daemon_id: "epiphany-daemon-supervisor".into(),
        command: s(&projector),
        args: args.clone(),
        cwd: None,
        enabled: true,
        restart_mode: "always".into(),
        cooldown_seconds: 0,
        backoff_multiplier: 1,
        stdout_artifact: s(&root.join("projector.stdout.log")),
        stderr_artifact: s(&root.join("projector.stderr.log")),
        updated_at_utc: Utc::now().to_rfc3339(),
        private_state_exposed: false,
        notes: vec!["GUID-scoped recovery smoke".into()],
    };
    write_epiphany_cultmesh_semantic_projector_service_policy(&verse, RUNTIME, policy.clone())?;
    let (_, policy_digest) =
        load_epiphany_cultmesh_managed_service_policy_with_digest(&verse, RUNTIME, SERVICE)?
            .context("policy absent")?;
    let mut child = ChildGuard(
        Command::new(&projector)
            .args(&args)
            .env("EPIPHANY_STARTUP_LIFECYCLE_RECEIPT_ID", &receipt_id)
            .env("EPIPHANY_MIND_QDRANT_COLLECTION", &mind_collection)
            .env("EPIPHANY_MODELING_QDRANT_COLLECTION", &modeling_collection)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?,
    );
    let pid = child.id();
    let executable_sha256 = format!("sha256-{:x}", Sha256::digest(fs::read(&projector)?));
    let completed = Utc::now();
    write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &verse,
        RUNTIME,
        EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry {
            schema_version: EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION
                .into(),
            receipt_id: receipt_id.clone(),
            service_id: SERVICE.into(),
            scheduler_id: "epiphany-daemon-supervisor".into(),
            runtime_id: RUNTIME.into(),
            daemon_selector: "epiphany-daemon-supervisor".into(),
            action: "launch".into(),
            status: "launched".into(),
            command: s(&projector),
            args: args.clone(),
            cwd: None,
            process_id: Some(pid),
            exit_code: None,
            started_at_utc: (completed - chrono::Duration::seconds(1)).to_rfc3339(),
            completed_at_utc: Some(completed.to_rfc3339()),
            operator_artifact_ref: format!("fixture://{id}/launch"),
            private_state_exposed: false,
            notes: vec![],
            executable_sha256,
            preflight_witness_id: String::new(),
            required_document_types: vec![],
            schema_preflight_passed: false,
            schema_catalog_sha256: String::new(),
            managed_policy_id: policy.policy_id,
            managed_policy_digest: policy_digest,
            provider_daemon_id: PROJECTOR.into(),
            startup_correlation_id: receipt_id.clone(),
        },
    )?;
    let stdout = child
        .stdout
        .take()
        .context("projector stdout unavailable")?;
    let (tx, rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let _ = tx.send(line);
        }
    });
    let first = next_pulse(&rx, Duration::from_secs(20))?;
    let heartbeat = first["heartbeatId"]
        .as_str()
        .context("pulse heartbeat absent")?;
    if first["providerStatus"] != "ready" {
        stop(&mut child);
        bail!("projector did not establish ready recovery witness: {first}");
    }

    let unrelated_id = Uuid::new_v4().to_string();
    write_epiphany_cultmesh_daemon_heartbeat_event(
        &verse,
        RUNTIME,
        EpiphanyCultMeshDaemonHeartbeatEventEntry {
            schema_version: EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_SCHEMA_VERSION.into(),
            heartbeat_id: unrelated_id.clone(),
            daemon_id: PROJECTOR.into(),
            cluster_id: "local".into(),
            provider_incarnation: first["providerIncarnation"].as_str().unwrap().into(),
            sequence: 999,
            status: "ready".into(),
            heartbeat_at: Utc::now().to_rfc3339(),
            private_state_exposed: false,
            startup_lifecycle_receipt_id: String::new(),
        },
    )?;
    let before_wrong = file_sha256(&mind)?;
    let wrong = supervisor_recover(
        &supervisor,
        &verse,
        &mind,
        &old.claim_id,
        &receipt_id,
        &unrelated_id,
    )?;
    if wrong.success {
        stop(&mut child);
        bail!("uncorrelated heartbeat authorized recovery");
    }
    if file_sha256(&mind)? != before_wrong {
        stop(&mut child);
        bail!("wrong heartbeat mutated canonical fixture store");
    }
    let recovered_process = supervisor_recover(
        &supervisor,
        &verse,
        &mind,
        &old.claim_id,
        &receipt_id,
        heartbeat,
    )?;
    if !recovered_process.success {
        stop(&mut child);
        bail!("exact recovery failed: {}", recovered_process.stderr);
    }
    let recovered: Value = serde_json::from_str(&recovered_process.stdout)?;
    if recovered["epoch"] != old.claim_epoch + 1 {
        stop(&mut child);
        bail!("recovered epoch did not advance exactly once");
    }
    let before_reuse = file_sha256(&mind)?;
    let reuse = supervisor_recover(
        &supervisor,
        &verse,
        &mind,
        &old.claim_id,
        &receipt_id,
        heartbeat,
    )?;
    if reuse.success {
        stop(&mut child);
        bail!("recovery evidence was reusable");
    }
    if file_sha256(&mind)? != before_reuse {
        stop(&mut child);
        bail!("single-use refusal mutated canonical fixture store");
    }
    let atomic = inspect_memory_semantic_recovery_for_smoke(
        &mind,
        &input,
        &old.executor_id,
        &old.executor_incarnation,
    )?;
    if atomic.claim_epoch != 2
        || atomic.claim_status != "running"
        || atomic.attempt_status != "running"
        || !atomic.recovery_authorization_consumed
        || !atomic.abandoned_attempt_failed
        || atomic.old_owner_authenticates_current_claim
    {
        stop(&mut child);
        bail!("recovery atomic state is invalid: {atomic:?}");
    }
    let deadline = Instant::now() + Duration::from_secs(60);
    let terminal = loop {
        if let Some(receipt) =
            load_memory_semantic_projection_success(&mind, input.obligation(), input.source_head())?
        {
            break receipt;
        }
        if Instant::now() > deadline {
            stop(&mut child);
            bail!("recovered successor did not reach terminal success");
        }
        let _ = next_pulse(&rx, Duration::from_secs(5));
    };
    stop(&mut child);
    collections.cleanup()?;
    let _readiness = load_memory_semantic_projection_readiness(&mind, &input)?
        .context("terminal success lacks readiness")?;
    let query_eligible = memory_semantic_projection_query_eligible(
        input.obligation(),
        input.source_head(),
        &terminal,
    );
    if !query_eligible {
        bail!("terminal receipt is not query eligible");
    }
    let proof = json!({"schemaVersion":"epiphany.semantic_recovery_smoke_proof.v1","status":"passed","fixtureId":id,"fixtureRoot":root,
        "liveCanonicalInputsUsed":false,"fixtureStores":{"mind":{"path":mind,"sourceId":input.source_head().canonical_source_id},"modeling":{"path":modeling,"sourceId":modeling_input.source_head().canonical_source_id},"verse":{"path":verse,"runtimeId":RUNTIME}},
        "processBoundary":{"supervisor":supervisor,"projector":projector,"abandonedProjectorPid":old_pid,"replacementProjectorPid":pid},
        "fencing":{"wrongHeartbeat":wrong,"wrongHeartbeatCanonicalBytesUnchanged":true,"singleUse":reuse,"singleUseCanonicalBytesUnchanged":true,"predecessorClaimId":old.claim_id,"predecessorEpoch":old.claim_epoch,"atomicRecovery":atomic},
        "successor":{"authorizationId":recovered["authorizationId"],"epoch":recovered["epoch"],"receiptId":terminal.receipt_id,"queryEligible":query_eligible,"readinessPresent":true},
        "cleanup":{"mindCollection":mind_collection,"modelingCollection":modeling_collection,"preflightAbsent":true,"deleted":true,"verifiedAbsent":true}});
    fs::write(root.join("proof.json"), serde_json::to_vec_pretty(&proof)?)?;
    println!("{}", serde_json::to_string_pretty(&proof)?);
    Ok(())
}

fn seed_mind(root: &Path, store: &Path, swarm: &str) -> Result<()> {
    let dir = root.join("agents");
    fs::create_dir(&dir)?;
    for (role, agent, file) in [
        (
            "imagination",
            "epiphany.imagination",
            "imagination.agent-state.json",
        ),
        ("modeling", "epiphany.modeling", "modeling.agent-state.json"),
        ("verification", "epiphany.soul", "soul.agent-state.json"),
        ("implementation", "epiphany.hands", "hands.agent-state.json"),
        ("research", "epiphany.eyes", "eyes.agent-state.json"),
        ("Persona", "epiphany.Persona", "Persona.agent-state.json"),
        ("coordinator", "epiphany.self", "self.agent-state.json"),
    ] {
        let j = json!({"schema_version":"ghostlight.agent_state.v1","world":{"world_id":"fixture","setting":"isolated fixture","time":{"label":"now"},"canon_context":["fixture"]},"agents":[{"agent_id":agent,"identity":{"name":role,"roles":["fixture"],"origin":"fixture","public_description":"fixture","private_notes":[]},"canonical_state":{"underlying_organization":{"x":{"mean":0.5,"plasticity":0.5,"current_activation":0.5}},"stable_dispositions":{"x":{"mean":0.5,"plasticity":0.5,"current_activation":0.5}},"behavioral_dimensions":{"x":{"mean":0.5,"plasticity":0.5,"current_activation":0.5}},"presentation_strategy":{"x":{"mean":0.5,"plasticity":0.5,"current_activation":0.5}},"voice_style":{"x":{"mean":0.5,"plasticity":0.5,"current_activation":0.5}},"situational_state":{"x":{"mean":0.5,"plasticity":0.5,"current_activation":0.5}},"values":[{"value_id":"v","label":"fixture","priority":0.5,"unforgivable_if_betrayed":false}]},"goals":[{"goal_id":"g","description":"fixture","scope":"life","priority":0.5,"emotional_stake":"fixture","blockers":[],"status":"active"}],"memories":{"episodic":[],"semantic":[],"relationship_summaries":[]},"perceived_state_overlays":[]}],"relationships":[],"events":[],"scenes":[]});
        let mut j = j;
        j["schema_version"] = json!("ghostlight.agent_state.v0");
        fs::write(dir.join(file), serde_json::to_vec(&j)?)?;
    }
    migrate_agent_memory_json_dir_to_cultcache(dir, store)?;
    ensure_agent_memory_swarm_identity(store, swarm)?;
    admit_legacy_agent_memory_generation(store)?;
    Ok(())
}
fn seed_modeling(store: &Path, mind: &Path, swarm: &str) -> Result<()> {
    let at = Utc::now().to_rfc3339();
    initialize_runtime_spine(
        store,
        RuntimeSpineInitOptions {
            runtime_id: format!("runtime-{swarm}"),
            display_name: "fixture".into(),
            created_at: at.clone(),
        },
    )?;
    bind_runtime_to_agent_memory_swarm(store, mind, &at)?;
    let mut graph = EpiphanyMemoryGraphSnapshot {
        schema_version: Some(MEMORY_GRAPH_SCHEMA_VERSION.into()),
        graph_id: format!("model-{swarm}"),
        model_revision: 1,
        ..Default::default()
    };
    graph.model_hash = memory_graph_model_hash(&graph)?;
    ensure_runtime_repo_model(store, store.with_extension("absent"), &graph, &at)?;
    Ok(())
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessEvidence {
    success: bool,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    stderr_class: String,
}

fn supervisor_recover(
    exe: &Path,
    verse: &Path,
    mind: &Path,
    claim: &str,
    receipt: &str,
    heartbeat: &str,
) -> Result<ProcessEvidence> {
    let out = Command::new(exe)
        .args([
            "semantic-recover",
            "--store",
            &s(verse),
            "--runtime-id",
            RUNTIME,
            "--agent-store",
            &s(mind),
            "--expected-claim-id",
            claim,
            "--receipt-id",
            receipt,
            "--provider-heartbeat-id",
            heartbeat,
        ])
        .output()?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let stderr_class = stderr
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("none")
        .chars()
        .take(240)
        .collect();
    Ok(ProcessEvidence {
        success: out.status.success(),
        exit_code: out.status.code(),
        stdout,
        stderr,
        stderr_class,
    })
}

fn wait_for_running_claim(
    store: &Path,
    input: &MemorySemanticProjectionInput,
    timeout: Duration,
) -> Result<SemanticRecoverySmokeInspection> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Ok(row) =
            inspect_memory_semantic_recovery_for_smoke(store, input, "absent", "absent")
        {
            if row.claim_status == "running" && row.claim_epoch == 1 {
                return Ok(row);
            }
        }
        if Instant::now() >= deadline {
            bail!("real projector did not persist its running epoch-1 claim");
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn collection_exists(base: &str, name: &str) -> Result<bool> {
    let url = format!("{}/collections/{}", base.trim_end_matches('/'), name);
    let status = reqwest::blocking::Client::new().get(url).send()?.status();
    if status.is_success() {
        return Ok(true);
    }
    if status.as_u16() == 404 {
        return Ok(false);
    }
    bail!("Qdrant collection preflight returned {status} for {name}")
}

fn file_sha256(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}

struct StalledHttpEndpoint {
    url: String,
    running: Arc<AtomicBool>,
}
impl StalledHttpEndpoint {
    fn new() -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let url = format!("http://{}", listener.local_addr()?);
        let running = Arc::new(AtomicBool::new(true));
        let alive = running.clone();
        thread::spawn(move || {
            let mut held = Vec::new();
            while alive.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => held.push(stream),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5))
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self { url, running })
    }
    fn url(&self) -> String {
        self.url.clone()
    }
}
impl Drop for StalledHttpEndpoint {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
    }
}
fn next_pulse(rx: &std::sync::mpsc::Receiver<String>, timeout: Duration) -> Result<Value> {
    let deadline = Instant::now() + timeout;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        let line = rx
            .recv_timeout(left)
            .context("timed out waiting for projector pulse")?;
        if let Ok(v) = serde_json::from_str::<Value>(&line) {
            return Ok(v);
        }
    }
}
fn cleanup_collection(base: &str, name: &str) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/collections/{}", base.trim_end_matches('/'), name);
    let response = client.delete(&url).send()?;
    if !response.status().is_success() && response.status().as_u16() != 404 {
        bail!(
            "failed to delete fixture collection {name}: {}",
            response.status()
        )
    }
    let verification = client.get(&url).send()?.status();
    if verification.as_u16() != 404 {
        bail!(
            "fixture collection cleanup verification expected HTTP 404 for {name}, observed {verification}"
        )
    }
    Ok(())
}
fn stop(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

struct ChildGuard(Child);
impl Deref for ChildGuard {
    type Target = Child;
    fn deref(&self) -> &Child {
        &self.0
    }
}
impl DerefMut for ChildGuard {
    fn deref_mut(&mut self) -> &mut Child {
        &mut self.0
    }
}
impl Drop for ChildGuard {
    fn drop(&mut self) {
        stop(&mut self.0);
    }
}

struct FixtureCollections {
    base: String,
    names: Vec<String>,
    cleaned: bool,
}
impl FixtureCollections {
    fn new(base: String, names: Vec<String>) -> Self {
        Self {
            base,
            names,
            cleaned: false,
        }
    }
    fn cleanup(&mut self) -> Result<()> {
        for name in &self.names {
            cleanup_collection(&self.base, name)?;
        }
        self.cleaned = true;
        Ok(())
    }
}
impl Drop for FixtureCollections {
    fn drop(&mut self) {
        if !self.cleaned {
            for name in &self.names {
                let _ = cleanup_collection(&self.base, name);
            }
        }
    }
}
fn sibling(dir: &Path, name: &str) -> PathBuf {
    dir.join(if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.into()
    })
}
fn s(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    #[test]
    fn fixture_has_no_canonical_store_or_default_collection_mouth() {
        let source = include_str!("epiphany-semantic-recovery-smoke.rs");
        let forbidden = [
            ["state/", "agents.msgpack"].concat(),
            ["state\\", "agents.msgpack"].concat(),
            [".epiphany", "-run"].concat(),
            ["epiphany_mind", "_v1"].concat(),
            ["epiphany_modeling", "_v1"].concat(),
        ];
        for forbidden in forbidden {
            assert!(
                !source.contains(&forbidden),
                "forbidden live surface: {forbidden}"
            );
        }
        let mutation_mouth = ["seed_abandoned_memory_semantic_projection", "_for_smoke"].concat();
        assert!(!source.contains(&mutation_mouth));
        assert!(source.contains("inspect_memory_semantic_recovery_for_smoke"));
        assert!(source.contains("Command::new(&projector)"));
        assert!(source.contains("Command::new(exe)"));
        let manifest = include_str!("../../Cargo.toml");
        assert!(manifest.contains("required-features = [\"semantic-recovery-smoke\"]"));
    }
}
