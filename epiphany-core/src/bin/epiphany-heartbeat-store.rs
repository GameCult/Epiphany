use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use epiphany_core::EpiphanyCultMeshSwarmBrakeEntry;
use epiphany_core::EpiphanyHeartbeatArtifactRetentionReceipt;
use epiphany_core::HeartbeatStaleTurnRepairOptions;
use epiphany_core::heartbeat_local_provider_status;
use epiphany_core::heartbeat_status_projection;
use epiphany_core::initialize_heartbeat_store;
use epiphany_core::load_epiphany_cultmesh_swarm_brake;
use epiphany_core::load_heartbeat_state_entry;
use epiphany_core::pulse_persona_heartbeat;
use epiphany_core::pulse_resident_self_heartbeat;
use epiphany_core::recover_stale_heartbeat_store;
use epiphany_core::resident_cognitive_runtime_id;
use epiphany_core::retain_heartbeat_pulse_artifacts;
use epiphany_core::{
    ResidentProviderReadiness, acquire_resident_process_singleton,
    authenticate_epiphany_packaged_release, capture_process_instance,
    publish_resident_provider_readiness,
};
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return usage();
    };
    let mut store_path: Option<PathBuf> = None;
    let mut artifact_dir: Option<PathBuf> = None;
    let mut local_verse_store: Option<PathBuf> = None;
    let mut target_heartbeat_rate = 1.0_f64;
    let mut schedule_id = "epiphany-heartbeat".to_string();
    let mut source_scene_ref = "epiphany/coordinator".to_string();
    let mut limit = 8_usize;
    let mut resident_self_store: Option<PathBuf> = None;
    let mut resident_runtime_store: Option<PathBuf> = None;
    let mut repair_reason: Option<String> = None;
    let mut max_age_seconds = 1800_i64;
    let mut now_utc: Option<String> = None;
    let mut interval_seconds = 120_u64;
    let mut max_iterations = 0_u64;
    let mut retain_pulse_artifacts = 256_usize;
    let mut retention_batch_size = 64_usize;
    let mut release_store: Option<PathBuf> = None;
    let mut release_runtime_id: Option<String> = None;
    let mut release_id: Option<String> = None;
    let mut release_witness_sha256: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--store" => store_path = Some(next_path(&mut args, "--store")?),
            "--artifact-dir" => artifact_dir = Some(next_path(&mut args, "--artifact-dir")?),
            "--local-verse-store" => {
                local_verse_store = Some(next_path(&mut args, "--local-verse-store")?)
            }
            "--target-heartbeat-rate" => {
                target_heartbeat_rate = next_value(&mut args, "--target-heartbeat-rate")?.parse()?
            }
            "--schedule-id" => schedule_id = next_value(&mut args, "--schedule-id")?,
            "--source-scene-ref" => source_scene_ref = next_value(&mut args, "--source-scene-ref")?,
            "--limit" => limit = next_value(&mut args, "--limit")?.parse()?,
            "--resident-self-store" => {
                resident_self_store = Some(next_path(&mut args, "--resident-self-store")?)
            }
            "--resident-runtime-store" => {
                resident_runtime_store = Some(next_path(&mut args, "--resident-runtime-store")?)
            }
            "--reason" => repair_reason = Some(next_value(&mut args, "--reason")?),
            "--max-age-seconds" => {
                max_age_seconds = next_value(&mut args, "--max-age-seconds")?.parse()?
            }
            "--now-utc" => now_utc = Some(next_value(&mut args, "--now-utc")?),
            "--interval-seconds" => {
                interval_seconds = next_value(&mut args, "--interval-seconds")?.parse()?
            }
            "--max-iterations" => {
                max_iterations = next_value(&mut args, "--max-iterations")?.parse()?
            }
            "--retain-pulse-artifacts" => {
                retain_pulse_artifacts =
                    next_value(&mut args, "--retain-pulse-artifacts")?.parse()?
            }
            "--retention-batch-size" => {
                retention_batch_size = next_value(&mut args, "--retention-batch-size")?.parse()?
            }
            "--release-store" => release_store = Some(next_path(&mut args, "--release-store")?),
            "--release-runtime-id" => {
                release_runtime_id = Some(next_value(&mut args, "--release-runtime-id")?)
            }
            "--release-id" => release_id = Some(next_value(&mut args, "--release-id")?),
            "--release-witness-sha256" => {
                release_witness_sha256 = Some(next_value(&mut args, "--release-witness-sha256")?)
            }
            _ => return Err(anyhow!("unknown argument {arg:?}")),
        }
    }

    match command.as_str() {
        "init" => {
            let store_path = store_path.ok_or_else(|| anyhow!("init requires --store"))?;
            let state = initialize_heartbeat_store(&store_path, target_heartbeat_rate)?;
            println!(
                "{}",
                serde_json::json!({
                    "ok": true,
                    "command": "init",
                    "storeFile": store_path,
                    "schemaVersion": state.schema_version,
                    "participants": state.participants.len(),
                    "history": state.history.len(),
                })
            );
        }
        "repair-stale" => {
            let store_path = store_path.ok_or_else(|| anyhow!("repair-stale requires --store"))?;
            let artifact_dir =
                artifact_dir.ok_or_else(|| anyhow!("repair-stale requires --artifact-dir"))?;
            let result = recover_stale_heartbeat_store(
                &store_path,
                &artifact_dir,
                HeartbeatStaleTurnRepairOptions {
                    max_age_seconds,
                    now_utc,
                    reason: repair_reason.unwrap_or_else(|| {
                        "Operator requested stale heartbeat turn recovery.".to_string()
                    }),
                },
            )?;
            println!("{}", result);
        }
        "retain-artifacts" => {
            let store_path =
                store_path.ok_or_else(|| anyhow!("retain-artifacts requires --store"))?;
            let artifact_dir =
                artifact_dir.ok_or_else(|| anyhow!("retain-artifacts requires --artifact-dir"))?;
            let receipt = retain_heartbeat_pulse_artifacts(
                &store_path,
                &artifact_dir,
                retain_pulse_artifacts,
                retention_batch_size,
                &chrono::Utc::now().to_rfc3339(),
            )?;
            println!(
                "{}",
                serde_json::json!({
                    "schemaVersion": "epiphany.heartbeat.artifact_retention_command.v0",
                    "status": if receipt.is_some() { "completed" } else { "within-bound" },
                    "receipt": receipt.as_ref().map(retention_receipt_projection),
                    "privateStateExposed": false
                })
            );
        }
        "status" => {
            let store_path = store_path.ok_or_else(|| anyhow!("status requires --store"))?;
            if let Some(artifact_dir) = artifact_dir {
                println!(
                    "{}",
                    heartbeat_status_projection(
                        &store_path,
                        artifact_dir,
                        target_heartbeat_rate,
                        limit
                    )?
                );
            } else {
                let state = load_heartbeat_state_entry(&store_path)?;
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "command": "status",
                        "store": store_path,
                        "present": state.is_some(),
                        "schemaVersion": state.as_ref().map(|value| value.schema_version.as_str()),
                        "participants": state.as_ref().map(|value| value.participants.len()),
                        "history": state.as_ref().map(|value| value.history.len()),
                    })
                );
            }
        }
        "serve" => {
            if interval_seconds == 0 {
                return Err(anyhow!(
                    "serve requires --interval-seconds greater than zero"
                ));
            }
            let store_path = store_path.ok_or_else(|| anyhow!("serve requires --store"))?;
            let _singleton = acquire_resident_process_singleton("heartbeat", &store_path)?;
            let artifact_dir =
                artifact_dir.ok_or_else(|| anyhow!("serve requires --artifact-dir"))?;
            let provider_release = if resident_self_store.is_some() {
                let release_store = release_store
                    .as_deref()
                    .ok_or_else(|| anyhow!("resident heartbeat serve requires --release-store"))?;
                let runtime_id = release_runtime_id.as_deref().ok_or_else(|| {
                    anyhow!("resident heartbeat serve requires --release-runtime-id")
                })?;
                let release_id = release_id
                    .as_deref()
                    .ok_or_else(|| anyhow!("resident heartbeat serve requires --release-id"))?;
                let witness = release_witness_sha256.as_deref().ok_or_else(|| {
                    anyhow!("resident heartbeat serve requires --release-witness-sha256")
                })?;
                Some(authenticate_epiphany_packaged_release(
                    release_store,
                    runtime_id,
                    release_id,
                    witness,
                )?)
            } else {
                None
            };
            let provider_process = if provider_release.is_some() {
                Some(capture_process_instance(std::process::id())?)
            } else {
                None
            };
            let brake_runtime_id = heartbeat_brake_runtime_id(
                &resident_self_store,
                &resident_runtime_store,
                release_runtime_id.as_deref(),
            )?;
            fs::create_dir_all(&artifact_dir).with_context(|| {
                format!(
                    "failed to create heartbeat serve artifact directory {}",
                    artifact_dir.display()
                )
            })?;
            let mut completed_iterations = highest_pulse_artifact_sequence(&artifact_dir)?;
            let mut refused_pulses = 0_u64;
            let shutdown_requested = install_shutdown_signal_owner()?;
            loop {
                if heartbeat_serve_should_stop(
                    &shutdown_requested,
                    max_iterations,
                    completed_iterations,
                ) {
                    break;
                }
                let iteration = completed_iterations + 1;
                if let (Some(release), Some(process)) = (&provider_release, &provider_process) {
                    publish_resident_provider_readiness(
                        &store_path,
                        ResidentProviderReadiness {
                            schema_version:
                                epiphany_core::RESIDENT_PROVIDER_READINESS_SCHEMA_VERSION.into(),
                            provider: "heartbeat".into(),
                            runtime_id: release.runtime_id.clone(),
                            release_id: release.release_id.clone(),
                            release_witness_sha256: release_witness_sha256
                                .clone()
                                .expect("authenticated witness"),
                            source_commit: release.source_commit_sha.clone(),
                            publisher_sequence: 0,
                            observed_at_millis: chrono::Utc::now().timestamp_millis().max(0) as u64,
                            process_id: process.process_id,
                            process_creation_token: process.creation_token,
                            process_executable_path: process.executable_path.display().to_string(),
                            status: heartbeat_local_provider_status(
                                &store_path,
                                resident_self_store.as_deref().expect("resident store"),
                            )
                            .into(),
                            private_state_exposed: false,
                        },
                    )?;
                }
                let iteration_dir = artifact_dir.join(format!("pulse-{iteration:06}"));
                let brake = active_swarm_brake(&local_verse_store, brake_runtime_id.as_deref())?;
                if let Some(resident_store) = resident_self_store.as_deref() {
                    let pulse = pulse_resident_self_heartbeat(
                        &store_path,
                        resident_store,
                        &iteration_dir,
                        brake.is_some(),
                        &format!("{schedule_id}.serve-{iteration:06}"),
                        &source_scene_ref,
                    )?;
                    if pulse.status != "idle" {
                        completed_iterations = iteration;
                        refused_pulses += u64::from(brake.is_some());
                        let retention = retain_heartbeat_pulse_artifacts(
                            &store_path,
                            &artifact_dir,
                            retain_pulse_artifacts,
                            retention_batch_size,
                            &chrono::Utc::now().to_rfc3339(),
                        )?;
                        println!(
                            "{}",
                            serde_json::json!({
                                "schemaVersion":"epiphany.heartbeat.serve_pulse.v0", "status":pulse.status,
                                "owner":"heartbeat", "iteration":iteration, "artifactDir":iteration_dir,
                                "acknowledgedTerminalId":pulse.acknowledged_terminal_id, "grantId":pulse.grant_id,
                                "brakeId":brake.as_ref().map(|value| &value.brake_id),
                                "retention":retention.as_ref().map(retention_receipt_projection),
                                "privateStateExposed":false
                            })
                        );
                        if max_iterations > 0 && completed_iterations >= max_iterations {
                            break;
                        }
                        wait_for_shutdown(
                            &shutdown_requested,
                            Duration::from_secs(interval_seconds),
                        );
                        continue;
                    }
                }
                if let Some(brake) = brake {
                    completed_iterations = iteration;
                    refused_pulses += 1;
                    let retention = retain_heartbeat_pulse_artifacts(
                        &store_path,
                        &artifact_dir,
                        retain_pulse_artifacts,
                        retention_batch_size,
                        &chrono::Utc::now().to_rfc3339(),
                    )?;
                    println!(
                        "{}",
                        serde_json::json!({
                            "schemaVersion": "epiphany.heartbeat.serve_pulse.v0",
                            "status": "refused-by-swarm-brake",
                            "owner": "heartbeat",
                            "lifecycleOwner": "Idunn",
                            "iteration": iteration,
                            "artifactDir": iteration_dir,
                            "brakeId": brake.brake_id,
                            "brakeScope": brake.scope,
                            "reason": brake.reason,
                            "retention": retention.as_ref().map(retention_receipt_projection),
                            "privateStateExposed": false
                        })
                    );
                    if max_iterations > 0 && completed_iterations >= max_iterations {
                        break;
                    }
                    wait_for_shutdown(&shutdown_requested, Duration::from_secs(interval_seconds));
                    continue;
                }
                let persona_pulse = pulse_persona_heartbeat(
                    &store_path,
                    &iteration_dir,
                    &format!("{schedule_id}.persona-{iteration:06}"),
                    &source_scene_ref,
                    false,
                )?;
                if persona_pulse["status"] != "idle" {
                    completed_iterations = iteration;
                    let retention = retain_heartbeat_pulse_artifacts(
                        &store_path,
                        &artifact_dir,
                        retain_pulse_artifacts,
                        retention_batch_size,
                        &chrono::Utc::now().to_rfc3339(),
                    )?;
                    println!(
                        "{}",
                        serde_json::json!({
                            "pulse": persona_pulse,
                            "retention": retention.as_ref().map(retention_receipt_projection),
                            "privateStateExposed": false
                        })
                    );
                    if max_iterations > 0 && completed_iterations >= max_iterations {
                        break;
                    }
                    wait_for_shutdown(&shutdown_requested, Duration::from_secs(interval_seconds));
                    continue;
                }
                completed_iterations = iteration;
                let retention = retain_heartbeat_pulse_artifacts(
                    &store_path,
                    &artifact_dir,
                    retain_pulse_artifacts,
                    retention_batch_size,
                    &chrono::Utc::now().to_rfc3339(),
                )?;
                println!(
                    "{}",
                    serde_json::json!({
                        "schemaVersion": "epiphany.heartbeat.serve_pulse.v0",
                        "status": "idle",
                        "owner": "heartbeat",
                        "lifecycleOwner": "Idunn",
                        "iteration": iteration,
                        "artifactDir": iteration_dir,
                        "retention": retention.as_ref().map(retention_receipt_projection),
                        "privateStateExposed": false
                    })
                );
                if max_iterations > 0 && completed_iterations >= max_iterations {
                    break;
                }
                wait_for_shutdown(&shutdown_requested, Duration::from_secs(interval_seconds));
            }
            println!(
                "{}",
                serde_json::json!({
                    "schemaVersion": "epiphany.heartbeat.serve_receipt.v0",
                    "status": "stopped-cleanly",
                    "owner": "heartbeat",
                    "lifecycleOwner": "Idunn",
                    "store": store_path,
                    "artifactDir": artifact_dir,
                    "intervalSeconds": interval_seconds,
                    "completedIterations": completed_iterations,
                    "refusedPulses": refused_pulses,
                    "bounded": max_iterations > 0,
                    "shutdownRequested": shutdown_requested.load(Ordering::SeqCst),
                    "privateStateExposed": false
                })
            );
        }
        _ => return usage(),
    }

    Ok(())
}

fn install_shutdown_signal_owner() -> Result<Arc<AtomicBool>> {
    let requested = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&requested);
    ctrlc::try_set_handler(move || {
        signal.store(true, Ordering::SeqCst);
    })
    .context("failed to install Heartbeat shutdown signal owner")?;
    Ok(requested)
}

fn heartbeat_serve_should_stop(
    shutdown_requested: &AtomicBool,
    max_iterations: u64,
    completed_iterations: u64,
) -> bool {
    shutdown_requested.load(Ordering::SeqCst)
        || (max_iterations > 0 && completed_iterations >= max_iterations)
}

fn wait_for_shutdown(requested: &AtomicBool, duration: Duration) {
    let deadline = std::time::Instant::now() + duration;
    while !requested.load(Ordering::SeqCst) {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        thread::sleep(remaining.min(Duration::from_millis(50)));
    }
}

fn next_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(next_value(args, name)?))
}

fn retention_receipt_projection(
    receipt: &EpiphanyHeartbeatArtifactRetentionReceipt,
) -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": receipt.schema_version,
        "receiptId": receipt.receipt_id,
        "planId": receipt.plan_id,
        "status": receipt.status,
        "deletedDirectories": receipt.deleted_directories,
        "deletedFileCount": receipt.deleted_file_count,
        "deletedByteCount": receipt.deleted_byte_count,
        "completedAtUtc": receipt.completed_at_utc,
        "privateStateExposed": receipt.private_state_exposed,
    })
}

fn highest_pulse_artifact_sequence(artifact_root: &Path) -> Result<u64> {
    let mut highest = 0_u64;
    for entry in fs::read_dir(artifact_root)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            return Err(anyhow!("heartbeat artifact root contains a symlink"));
        }
        if !file_type.is_dir() {
            continue;
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow!("heartbeat artifact directory name is not UTF-8"))?;
        let digits = name
            .strip_prefix("pulse-")
            .filter(|digits| digits.len() == 6 && digits.bytes().all(|byte| byte.is_ascii_digit()))
            .ok_or_else(|| {
                anyhow!("heartbeat artifact root contains unknown directory {name:?}")
            })?;
        highest = highest.max(digits.parse()?);
    }
    if highest >= 999_999 {
        return Err(anyhow!("heartbeat pulse artifact sequence is exhausted"));
    }
    Ok(highest)
}

fn next_value(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("{name} requires a value"))
}

fn usage() -> Result<()> {
    Err(anyhow!(concat!(
        "usage: epiphany-heartbeat-store init --store <path> [--target-heartbeat-rate <n>]\n",
        "       epiphany-heartbeat-store repair-stale --store <path> --artifact-dir <path> [options]\n",
        "       epiphany-heartbeat-store retain-artifacts --store <path> --artifact-dir <path> [--retain-pulse-artifacts <n>] [--retention-batch-size <n>]\n",
        "       epiphany-heartbeat-store status --store <path> [--artifact-dir <path>]\n",
        "       epiphany-heartbeat-store serve --store <path> --artifact-dir <path> [--local-verse-store <path>] [--resident-self-store <path> --resident-runtime-store <path>] [--interval-seconds <n>] [--max-iterations <n>]"
    )))
}

fn heartbeat_brake_runtime_id(
    resident_self_store: &Option<PathBuf>,
    resident_runtime_store: &Option<PathBuf>,
    release_runtime_id: Option<&str>,
) -> Result<Option<String>> {
    if resident_self_store.is_some() {
        let runtime_store = resident_runtime_store
            .as_deref()
            .ok_or_else(|| anyhow!("resident heartbeat serve requires --resident-runtime-store"))?;
        return Ok(Some(resident_cognitive_runtime_id(runtime_store)?));
    }
    Ok(release_runtime_id.map(str::to_string))
}

fn active_swarm_brake(
    local_verse_store: &Option<PathBuf>,
    runtime_id: Option<&str>,
) -> Result<Option<EpiphanyCultMeshSwarmBrakeEntry>> {
    let Some(local_verse_store) = local_verse_store else {
        return Ok(None);
    };
    if !local_verse_store.exists() {
        return Ok(None);
    }
    let runtime_id = runtime_id
        .ok_or_else(|| anyhow!("heartbeat local Verse brake lookup requires runtime identity"))?;
    Ok(
        load_epiphany_cultmesh_swarm_brake(local_verse_store, runtime_id)?
            .filter(|brake| brake.status == "engaged"),
    )
}

#[cfg(test)]
mod brake_tests {
    use super::*;

    #[test]
    fn heartbeat_serve_stops_for_signal_or_bounded_completion() {
        let requested = AtomicBool::new(false);
        assert!(!heartbeat_serve_should_stop(&requested, 0, 42));
        assert!(!heartbeat_serve_should_stop(&requested, 43, 42));
        assert!(heartbeat_serve_should_stop(&requested, 42, 42));
        requested.store(true, Ordering::SeqCst);
        assert!(heartbeat_serve_should_stop(&requested, 0, 42));
    }

    #[test]
    fn heartbeat_shutdown_interrupts_interval_wait() {
        let requested = AtomicBool::new(true);
        let started = std::time::Instant::now();
        wait_for_shutdown(&requested, Duration::from_secs(1));
        assert!(started.elapsed() < Duration::from_millis(100));
    }

    #[test]
    fn heartbeat_brake_lookup_uses_the_exact_requested_namespace() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("verse.cc");
        let store_arg = Some(store.clone());
        let mut brake =
            epiphany_core::default_epiphany_cultmesh_swarm_brake("2026-07-18T00:00:00Z");
        brake.status = "engaged".into();
        brake.reason = "test".into();
        let mut legacy_node =
            epiphany_core::open_epiphany_cultmesh_node(&store, "legacy-writer-runtime")?;
        legacy_node.put("epiphany-local/swarm-brake", &brake)?;
        legacy_node.flush()?;
        assert!(active_swarm_brake(&store_arg, Some("epiphany-yggdrasil"))?.is_some());
        epiphany_core::write_epiphany_cultmesh_swarm_brake(&store, "wrong-runtime", brake.clone())?;
        assert!(active_swarm_brake(&store_arg, Some("epiphany-yggdrasil"))?.is_none());
        epiphany_core::write_epiphany_cultmesh_swarm_brake(&store, "epiphany-yggdrasil", brake)?;
        assert!(active_swarm_brake(&store_arg, Some("epiphany-yggdrasil"))?.is_some());
        Ok(())
    }

    #[test]
    fn resident_heartbeat_uses_mounted_cognitive_runtime_for_braking() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let runtime_store = temp.path().join("runtime.cc");
        epiphany_core::initialize_runtime_spine(
            &runtime_store,
            epiphany_core::RuntimeSpineInitOptions {
                runtime_id: "cognitive-runtime".into(),
                display_name: "Cognitive runtime".into(),
                created_at: "2026-08-10T00:00:00Z".into(),
            },
        )?;
        let selected = heartbeat_brake_runtime_id(
            &Some(temp.path().join("resident.cc")),
            &Some(runtime_store),
            Some("deployment-runtime"),
        )?;
        assert_eq!(selected.as_deref(), Some("cognitive-runtime"));
        Ok(())
    }

    #[test]
    fn resident_heartbeat_resumes_after_highest_artifact_sequence() -> Result<()> {
        let temp = tempfile::tempdir()?;
        fs::create_dir(temp.path().join("pulse-000003"))?;
        fs::create_dir(temp.path().join("pulse-012557"))?;
        assert_eq!(highest_pulse_artifact_sequence(temp.path())?, 12_557);
        fs::create_dir(temp.path().join("alien"))?;
        assert!(highest_pulse_artifact_sequence(temp.path()).is_err());
        Ok(())
    }
}
