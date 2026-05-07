use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use epiphany_core::load_agent_memory_entry_for_role;
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
    let workspace = root
        .join(".epiphany-smoke")
        .join("repo-personality-workspace");
    let artifacts = root
        .join(".epiphany-smoke")
        .join("repo-personality-artifacts");
    reset_path(&workspace)?;
    reset_path(&artifacts)?;
    let cult_repo = create_repo(
        &workspace.join("CultTiny"),
        &[
            (
                "AGENTS.md",
                "Prefer typed schemas and tiny public contracts.\n",
            ),
            ("src/lib.rs", "pub fn schema_contract() {}\n"),
            (
                "docs/architecture.md",
                "# Architecture\nCultTiny stores typed contract surfaces.\n",
            ),
            (
                "research/prior-art.md",
                "# Prior Art\nPrefer known schema formats before invention.\n",
            ),
            (
                "tests/contract_smoke.rs",
                "#[test] fn contract_smoke() {}\n",
            ),
            ("schemas/message.schema.json", "{\"type\":\"object\"}\n"),
        ],
        "Add schema contract smoke",
    )?;
    let lore_repo = create_repo(
        &workspace.join("LoreTiny"),
        &[
            (
                "AGENTS.md",
                "Respect canon, source notes, and public voice.\n",
            ),
            ("state/map.yaml", "objective: preserve canon\n"),
            ("notes/fresh-workspace-handoff.md", "Read the map first.\n"),
            ("Chronicles/City.md", "# City\nCanon material.\n"),
        ],
        "Polish canon handoff",
    )?;

    let scout = run_personality(
        &root,
        &[
            "scout",
            "--root",
            workspace.to_str().unwrap_or_default(),
            "--artifact-dir",
            artifacts.join("scout").to_str().unwrap_or_default(),
        ],
    )?;
    require(scout["repoCount"] == 2, "scout should find two git repos")?;
    let baseline = path_value(&scout, "store")?;
    require(baseline.exists(), "scout should write typed baseline store")?;

    let projection = run_personality(
        &root,
        &[
            "project",
            "--repo",
            cult_repo.to_str().unwrap_or_default(),
            "--baseline",
            baseline.to_str().unwrap_or_default(),
            "--artifact-dir",
            artifacts.join("cult-project").to_str().unwrap_or_default(),
        ],
    )?;
    require(
        projection["profile"]["roleProjectionCount"] == 8,
        "project should emit all standing role projections",
    )?;
    let store = path_value(&projection, "store")?;
    require(
        store.exists(),
        "project should write typed projection store",
    )?;

    let packet = run_personality(
        &root,
        &[
            "agent-packet",
            "--store",
            store.to_str().unwrap_or_default(),
            "--artifact-dir",
            artifacts.join("cult-packet").to_str().unwrap_or_default(),
        ],
    )?;
    require(
        packet["roleProjectionCount"] == 8,
        "agent packet should carry eight role projections",
    )?;
    let packet_path = path_value(&packet, "packetPath")?;
    let prompt_path = path_value(&packet, "promptPath")?;
    let packet_summary_path = path_value(&packet, "summaryPath")?;
    require(
        packet_path.exists(),
        "agent packet should write packet json",
    )?;
    require(
        prompt_path.exists(),
        "agent packet should write prompt markdown",
    )?;
    require(
        packet_summary_path.exists(),
        "agent packet should write summary markdown",
    )?;

    let memory_packet = run_personality(
        &root,
        &[
            "memory-packet",
            "--store",
            store.to_str().unwrap_or_default(),
            "--artifact-dir",
            artifacts
                .join("cult-memory-packet")
                .to_str()
                .unwrap_or_default(),
        ],
    )?;
    require(
        memory_packet["memorySourceCount"].as_u64().unwrap_or(0) > 0,
        "memory packet should carry source excerpts",
    )?;
    require(
        memory_packet["roleDistillerCount"] == 8,
        "memory packet should carry all role distiller lanes",
    )?;
    let memory_packet_path = path_value(&memory_packet, "packetPath")?;
    let memory_prompt_path = path_value(&memory_packet, "promptPath")?;
    let memory_summary_path = path_value(&memory_packet, "summaryPath")?;
    require(
        memory_packet_path.exists(),
        "memory packet should write packet json",
    )?;
    require(
        memory_prompt_path.exists(),
        "memory packet should write prompt markdown",
    )?;
    require(
        memory_summary_path.exists(),
        "memory packet should write summary markdown",
    )?;

    let trajectory_packet = run_personality(
        &root,
        &[
            "trajectory-packet",
            "--store",
            store.to_str().unwrap_or_default(),
            "--artifact-dir",
            artifacts
                .join("cult-trajectory-packet")
                .to_str()
                .unwrap_or_default(),
        ],
    )?;
    require(
        trajectory_packet["trajectoryThemeCount"]
            .as_u64()
            .unwrap_or(0)
            > 0,
        "trajectory packet should carry trajectory themes",
    )?;
    let trajectory_packet_path = path_value(&trajectory_packet, "packetPath")?;
    let trajectory_prompt_path = path_value(&trajectory_packet, "promptPath")?;
    let trajectory_summary_path = path_value(&trajectory_packet, "summaryPath")?;
    require(
        trajectory_packet_path.exists(),
        "trajectory packet should write packet json",
    )?;
    require(
        trajectory_prompt_path.exists(),
        "trajectory packet should write prompt markdown",
    )?;
    require(
        trajectory_summary_path.exists(),
        "trajectory packet should write summary markdown",
    )?;

    let init_store = artifacts.join("repo-initialization.msgpack");
    let startup = run_personality(
        &root,
        &[
            "startup",
            "--repo",
            cult_repo.to_str().unwrap_or_default(),
            "--baseline",
            baseline.to_str().unwrap_or_default(),
            "--artifact-dir",
            artifacts.join("startup-first").to_str().unwrap_or_default(),
            "--init-store",
            init_store.to_str().unwrap_or_default(),
        ],
    )?;
    require(
        startup["action"] == "reviewInitializationPackets",
        "startup should request birth packet review before accepted records exist",
    )?;
    require(
        startup["generatedPackets"].as_array().map_or(0, Vec::len) == 3,
        "startup should generate trajectory, personality, and memory packets",
    )?;
    let startup_trajectory_packet = packet_for_kind(&startup, "repo-trajectory")?;
    let startup_personality_packet = packet_for_kind(&startup, "repo-personality")?;
    let startup_memory_packet = packet_for_kind(&startup, "repo-memory")?;
    require(
        startup_trajectory_packet.exists(),
        "startup should write trajectory packet",
    )?;
    require(
        startup_personality_packet.exists(),
        "startup should write personality packet",
    )?;
    require(
        startup_memory_packet.exists(),
        "startup should write memory packet",
    )?;
    let birth_runner = run_birth_runner(
        &root,
        &[
            "--repo",
            cult_repo.to_str().unwrap_or_default(),
            "--baseline",
            baseline.to_str().unwrap_or_default(),
            "--artifact-dir",
            artifacts
                .join("birth-runner-plan")
                .to_str()
                .unwrap_or_default(),
            "--init-store",
            artifacts
                .join("birth-runner-plan")
                .join("repo-initialization.msgpack")
                .to_str()
                .unwrap_or_default(),
            "--agent-store",
            root.join("state")
                .join("agents.msgpack")
                .to_str()
                .unwrap_or_default(),
            "--heartbeat-store",
            root.join("state")
                .join("agent-heartbeats.msgpack")
                .to_str()
                .unwrap_or_default(),
            "--mode",
            "plan",
        ],
    )?;
    require(
        birth_runner["schemaVersion"] == "epiphany.repo_birth_runner.v0",
        "birth runner should return its schema",
    )?;
    require(
        birth_runner["executions"].as_array().map_or(0, Vec::len) == 3,
        "birth runner plan should expose all startup-only specialist executions",
    )?;
    require(
        birth_runner["executions"]
            .as_array()
            .into_iter()
            .flatten()
            .all(|execution| execution["heartbeatParticipant"].is_null()),
        "birth runner executions should not be heartbeat participants",
    )?;
    let heartbeat_store = artifacts.join("startup-heartbeats.msgpack");
    run_heartbeat(
        &root,
        &[
            "init",
            "--store",
            heartbeat_store.to_str().unwrap_or_default(),
        ],
    )?;
    let trajectory_result_path = artifacts.join("startup-trajectory-result.json");
    fs::write(
        &trajectory_result_path,
        serde_json::to_vec_pretty(&json!({
            "verdict": "ready-for-review",
            "summary": "Smoke trajectory birth distillation.",
            "confidence": 0.88,
            "selfImage": "CultTiny is becoming a typed-contract workspace that prefers receipts to swagger.",
            "trajectoryNarrative": "The tiny history and docs both lean toward explicit schema boundaries, known formats, and reviewable proof over improvising private protocol sludge.",
            "implicitGoals": ["Keep future expansion grounded in explicit schemas and receipts."],
            "antiGoals": ["Do not drift into ad hoc protocol glue just because the fixture is small."],
            "roleBiases": [{
                "roleId": "coordinator",
                "bias": "Self should treat typed-schema growth as part of the repo's grain.",
                "trajectorySignals": ["systems_formalization", "protocol_intolerance"],
                "behavioralEffect": "Challenge loose glue early.",
                "risk": "Could become stiff if later work needs play.",
                "evidenceRefs": ["AGENTS.md", "docs/architecture.md"]
            }],
            "selfPatchCandidates": [{
                "roleId": "coordinator",
                "selfPatch": {
                    "agentId": "epiphany.self",
                    "reason": "Trajectory birth should teach Self that CultTiny has been moving toward typed receipts and explicit contracts.",
                    "semanticMemories": [{
                        "memoryId": "mem-self-startup-trajectory-smoke",
                        "summary": "CultTiny's trajectory favors typed contract growth and evidence-backed explicitness over casual glue.",
                        "salience": 0.74,
                        "confidence": 0.87
                    }]
                }
            }],
            "initializationRecord": {
                "repoId": "culttiny",
                "distillerKind": "repo-trajectory",
                "acceptedOnce": true
            },
            "doNotMutate": [],
            "nextSafeMove": "Self reviews the trajectory petition before it colors later memory drift."
        }))?,
    )?;
    let accepted_trajectory = run_personality(
        &root,
        &[
            "accept-init",
            "--init-store",
            init_store.to_str().unwrap_or_default(),
            "--packet",
            startup_trajectory_packet.to_str().unwrap_or_default(),
            "--kind",
            "repo-trajectory",
            "--accepted-by",
            "smoke-self",
            "--summary",
            "Smoke accepted repo trajectory birth packet after review.",
            "--result",
            trajectory_result_path.to_str().unwrap_or_default(),
            "--agent-store",
            root.join("state")
                .join("agents.msgpack")
                .to_str()
                .unwrap_or_default(),
            "--apply-self-patches",
            "false",
        ],
    )?;
    require(
        accepted_trajectory["record"]["kind"] == "repo-trajectory",
        "accept-init should record trajectory birth",
    )?;
    let accepted_personality = run_personality(
        &root,
        &[
            "accept-init",
            "--init-store",
            init_store.to_str().unwrap_or_default(),
            "--packet",
            startup_personality_packet.to_str().unwrap_or_default(),
            "--kind",
            "repo-personality",
            "--accepted-by",
            "smoke-self",
            "--summary",
            "Smoke accepted repo personality birth packet after review.",
            "--agent-store",
            root.join("state")
                .join("agents.msgpack")
                .to_str()
                .unwrap_or_default(),
            "--apply-trait-seeds",
            "true",
            "--heartbeat-store",
            heartbeat_store.to_str().unwrap_or_default(),
            "--apply-heartbeat-seeds",
            "true",
        ],
    )?;
    require(
        accepted_personality["record"]["kind"] == "repo-personality",
        "accept-init should record personality birth",
    )?;
    require(
        accepted_personality["heartbeatSeeds"]["applied"]
            .as_u64()
            .unwrap_or_default()
            > 0,
        "accept-init should apply heartbeat seed mutations",
    )?;
    require(
        accepted_personality["traitLatticePersistence"]["applied"]
            .as_u64()
            .unwrap_or_default()
            > 0,
        "accept-init should stamp newborn canonical trait lattice seeds",
    )?;
    let coordinator =
        load_agent_memory_entry_for_role(root.join("state").join("agents.msgpack"), "coordinator")?
            .ok_or_else(|| anyhow!("startup smoke lost coordinator role memory entry"))?;
    require(
        !coordinator
            .agent
            .canonical_state
            .underlying_organization
            .contains_key("baseline"),
        "personality accept-init should replace baseline canonical traits for coordinator",
    )?;
    require(
        coordinator
            .agent
            .canonical_state
            .underlying_organization
            .contains_key("routing_discipline"),
        "personality accept-init should seed coordinator routing_discipline trait",
    )?;
    let agent_store = artifacts.join("startup-agents.msgpack");
    fs::copy(root.join("state").join("agents.msgpack"), &agent_store)
        .context("failed to copy role memory store for startup smoke")?;
    let memory_result_path = artifacts.join("startup-memory-result.json");
    fs::write(
        &memory_result_path,
        serde_json::to_vec_pretty(&json!({
            "verdict": "ready-for-review",
            "summary": "Smoke newborn memory distillation.",
            "confidence": 0.9,
            "roleMemoryPatches": [{
                "roleId": "modeling",
                "roleName": "Body",
                "verdict": "ready-for-review",
                "selfPatch": {
                    "agentId": "epiphany.body",
                    "reason": "Birth memory should teach Body that repo initialization memory is reviewed and typed.",
                    "semanticMemories": [{
                        "memoryId": "mem-body-startup-birth-memory-smoke",
                        "summary": "Repo memory birth packets can produce role-specific selfPatch candidates that are applied only after Self review.",
                        "salience": 0.72,
                        "confidence": 0.88
                    }]
                },
                "sourceRefs": ["AGENTS.md"],
                "whyThisBelongsInMemory": "Body needs the startup memory route to stay source-grounded.",
                "stalenessRisk": "smoke fixture",
                "doNotStore": []
            }]
        }))?,
    )?;
    let accepted_memory = run_personality(
        &root,
        &[
            "accept-init",
            "--init-store",
            init_store.to_str().unwrap_or_default(),
            "--packet",
            startup_memory_packet.to_str().unwrap_or_default(),
            "--kind",
            "repo-memory",
            "--accepted-by",
            "smoke-self",
            "--summary",
            "Smoke accepted repo memory birth packet after review.",
            "--result",
            memory_result_path.to_str().unwrap_or_default(),
            "--agent-store",
            agent_store.to_str().unwrap_or_default(),
            "--apply-self-patches",
            "true",
        ],
    )?;
    require(
        accepted_memory["record"]["kind"] == "repo-memory",
        "accept-init should record memory birth",
    )?;
    require(
        accepted_memory["selfPersistence"]["applied"] == 1,
        "accept-init should apply reviewed memory selfPatch candidates",
    )?;
    let startup_after_accept = run_personality(
        &root,
        &[
            "startup",
            "--repo",
            cult_repo.to_str().unwrap_or_default(),
            "--baseline",
            baseline.to_str().unwrap_or_default(),
            "--artifact-dir",
            artifacts
                .join("startup-after-accept")
                .to_str()
                .unwrap_or_default(),
            "--init-store",
            init_store.to_str().unwrap_or_default(),
        ],
    )?;
    require(
        startup_after_accept["action"] == "continueStartup",
        "startup should not rerun birth packets after accepted records exist",
    )?;
    require(
        startup_after_accept["generatedPackets"]
            .as_array()
            .is_some_and(Vec::is_empty),
        "startup should generate no packets after accepted records exist",
    )?;

    let status = run_personality(
        &root,
        &["status", "--store", store.to_str().unwrap_or_default()],
    )?;
    require(
        status["reports"] == 1,
        "projection store should have one report",
    )?;
    require(
        status["profiles"] == 1,
        "projection store should have one profile",
    )?;
    require(
        status["trajectoryReports"] == 1,
        "projection store should have one trajectory report",
    )?;
    require(
        status["roleProjections"] == 8,
        "projection store should have eight role projections",
    )?;

    Ok(json!({
        "workspace": workspace,
        "artifactRoot": artifacts,
        "cultRepo": cult_repo,
        "loreRepo": lore_repo,
        "baseline": baseline,
        "projectionStore": store,
        "packetPath": packet_path,
        "promptPath": prompt_path,
        "packetSummaryPath": packet_summary_path,
        "trajectoryPacketPath": trajectory_packet_path,
        "trajectoryPromptPath": trajectory_prompt_path,
        "trajectorySummaryPath": trajectory_summary_path,
        "memoryPacketPath": memory_packet_path,
        "memoryPromptPath": memory_prompt_path,
        "memorySummaryPath": memory_summary_path,
        "initStore": init_store,
        "agentStore": agent_store,
        "heartbeatStore": heartbeat_store,
        "birthRunnerSummary": artifacts.join("birth-runner-plan").join("birth-runner-summary.json"),
        "startupTrajectoryPacket": startup_trajectory_packet,
        "startupPersonalityPacket": startup_personality_packet,
        "startupMemoryPacket": startup_memory_packet,
        "startupTrajectoryResult": trajectory_result_path,
        "startupMemoryResult": memory_result_path,
        "startupAfterAcceptAction": startup_after_accept["action"],
        "repoCount": scout["repoCount"],
        "trajectoryReports": status["trajectoryReports"],
        "roleProjections": status["roleProjections"],
    }))
}

fn create_repo(root: &Path, files: &[(&str, &str)], commit_message: &str) -> Result<PathBuf> {
    fs::create_dir_all(root)?;
    run_git(root, &["init"])?;
    run_git(
        root,
        &["config", "user.email", "epiphany-smoke@example.invalid"],
    )?;
    run_git(root, &["config", "user.name", "Epiphany Smoke"])?;
    for (path, content) in files {
        let path = root.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
    }
    run_git(root, &["add", "."])?;
    run_git(root, &["commit", "-m", commit_message])?;
    Ok(root.to_path_buf())
}

fn run_personality(root: &Path, args: &[&str]) -> Result<Value> {
    let exe = native_personality_exe()?;
    ensure_built(root, "epiphany-repo-personality", &exe)?;
    let output = Command::new(exe)
        .current_dir(root)
        .args(args)
        .output()
        .context("failed to run epiphany-repo-personality")?;
    require(
        output.status.success(),
        &format!(
            "epiphany-repo-personality failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ),
    )?;
    serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "epiphany-repo-personality did not return JSON: {}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn run_birth_runner(root: &Path, args: &[&str]) -> Result<Value> {
    let exe = native_birth_runner_exe()?;
    ensure_built(root, "epiphany-repo-birth-runner", &exe)?;
    let output = Command::new(exe)
        .current_dir(root)
        .args(args)
        .output()
        .context("failed to run epiphany-repo-birth-runner")?;
    require(
        output.status.success(),
        &format!(
            "epiphany-repo-birth-runner failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ),
    )?;
    serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "epiphany-repo-birth-runner did not return JSON: {}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn run_heartbeat(root: &Path, args: &[&str]) -> Result<Value> {
    let exe = native_heartbeat_exe()?;
    ensure_built(root, "epiphany-heartbeat-store", &exe)?;
    let output = Command::new(exe)
        .current_dir(root)
        .args(args)
        .output()
        .context("failed to run epiphany-heartbeat-store")?;
    require(
        output.status.success(),
        &format!(
            "epiphany-heartbeat-store failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ),
    )?;
    serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "epiphany-heartbeat-store did not return JSON: {}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn native_personality_exe() -> Result<PathBuf> {
    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .unwrap_or_else(|| r"C:\Users\Meta\.cargo-target-codex".into());
    Ok(PathBuf::from(target_dir).join("debug").join(format!(
        "epiphany-repo-personality{}",
        env::consts::EXE_SUFFIX
    )))
}

fn native_birth_runner_exe() -> Result<PathBuf> {
    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .unwrap_or_else(|| r"C:\Users\Meta\.cargo-target-codex".into());
    Ok(PathBuf::from(target_dir).join("debug").join(format!(
        "epiphany-repo-birth-runner{}",
        env::consts::EXE_SUFFIX
    )))
}

fn native_heartbeat_exe() -> Result<PathBuf> {
    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .unwrap_or_else(|| r"C:\Users\Meta\.cargo-target-codex".into());
    Ok(PathBuf::from(target_dir).join("debug").join(format!(
        "epiphany-heartbeat-store{}",
        env::consts::EXE_SUFFIX
    )))
}

fn ensure_built(root: &Path, bin: &str, exe: &Path) -> Result<()> {
    let status = Command::new("cargo")
        .current_dir(root)
        .arg("build")
        .arg("--manifest-path")
        .arg(root.join("epiphany-core").join("Cargo.toml"))
        .arg("--bin")
        .arg(bin)
        .status()
        .with_context(|| format!("failed to build {bin}"))?;
    require(
        status.success() && exe.exists(),
        &format!("{bin} executable was not built at {}", exe.display()),
    )
}

fn run_git(repo: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    require(
        output.status.success(),
        &format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

fn reset_path(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).with_context(|| format!("failed to reset {}", path.display()))?;
    }
    fs::create_dir_all(path).with_context(|| format!("failed to create {}", path.display()))
}

fn path_value(value: &Value, key: &str) -> Result<PathBuf> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("missing path value {key}"))
}

fn packet_for_kind(value: &Value, kind: &str) -> Result<PathBuf> {
    value["generatedPackets"]
        .as_array()
        .and_then(|packets| {
            packets
                .iter()
                .find(|packet| packet["kind"] == kind)
                .and_then(|packet| packet["packetPath"].as_str())
        })
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("missing generated packet for {kind}"))
}

fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(anyhow!(message.to_string()))
    }
}
