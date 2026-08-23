use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use epiphany_core::EpiphanyCultMeshSwarmBrakeEntry;
use epiphany_core::EpiphanyHeartbeatArtifactRetentionReceipt;
use epiphany_core::GhostlightSceneParticipantSeed;
use epiphany_core::HeartbeatCompleteOptions;
use epiphany_core::HeartbeatHeatUpdateOptions;
use epiphany_core::HeartbeatPumpOptions;
use epiphany_core::HeartbeatQueueMentionOptions;
use epiphany_core::HeartbeatStaleTurnRepairOptions;
use epiphany_core::HeartbeatTickOptions;
use epiphany_core::complete_heartbeat_store;
use epiphany_core::heartbeat_local_provider_status;
use epiphany_core::heartbeat_status_projection;
use epiphany_core::initialize_ghostlight_scene_heartbeat_store;
use epiphany_core::initialize_heartbeat_store;
use epiphany_core::load_epiphany_cultmesh_swarm_brake;
use epiphany_core::load_heartbeat_state_entry;
use epiphany_core::pulse_persona_heartbeat;
use epiphany_core::pulse_resident_self_heartbeat;
use epiphany_core::pump_heartbeat_store;
use epiphany_core::queue_heartbeat_pending_mention_store;
use epiphany_core::recover_stale_heartbeat_store;
use epiphany_core::resident_cognitive_runtime_id;
use epiphany_core::retain_heartbeat_pulse_artifacts;
use epiphany_core::tick_heartbeat_store;
use epiphany_core::update_heartbeat_heat_store;
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
    let mut min_heartbeat_rate = 0.05_f64;
    let mut max_heartbeat_rate = 4.0_f64;
    let mut min_concurrency = 1_usize;
    let mut max_concurrency = 8_usize;
    let mut max_ticks = 8_usize;
    let mut coordinator_action: Option<String> = None;
    let mut target_role: Option<String> = None;
    let mut urgency = 0.75_f64;
    let mut urgency_explicit = false;
    let mut schedule_id = "epiphany-heartbeat".to_string();
    let mut source_scene_ref = "epiphany/coordinator".to_string();
    let mut defer_completion = false;
    let mut role: Option<String> = None;
    let mut action_id: Option<String> = None;
    let mut limit = 8_usize;
    let mut resident_self_store: Option<PathBuf> = None;
    let mut resident_runtime_store: Option<PathBuf> = None;
    let mut profile = "epiphany".to_string();
    let mut scene_id = "ghostlight.scene".to_string();
    let mut scene_participants = Vec::<GhostlightSceneParticipantSeed>::new();
    let mut heat_scope = "global".to_string();
    let mut heat_selector = String::new();
    let mut heat_multiplier = 1.0_f64;
    let mut heat_id: Option<String> = None;
    let mut heat_label: Option<String> = None;
    let mut heat_reason: Option<String> = None;
    let mut heat_expires_after: Option<f64> = None;
    let mut heat_clear = false;
    let mut channel_id: Option<String> = None;
    let mut message_id: Option<String> = None;
    let mut author_id: Option<String> = None;
    let mut author_name: Option<String> = None;
    let mut content: Option<String> = None;
    let mut visible_prompt: Option<String> = None;
    let mut reply_to_message_id: Option<String> = None;
    let mut source_surface = "operator".to_string();
    let mut mention_id: Option<String> = None;
    let mut source_visibility = "private".to_string();
    let mut data_classification = "operator_input".to_string();
    let mut model_provider_id = "operator-configured".to_string();
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
            "--min-heartbeat-rate" => {
                min_heartbeat_rate = next_value(&mut args, "--min-heartbeat-rate")?.parse()?
            }
            "--max-heartbeat-rate" => {
                max_heartbeat_rate = next_value(&mut args, "--max-heartbeat-rate")?.parse()?
            }
            "--min-concurrency" => {
                min_concurrency = next_value(&mut args, "--min-concurrency")?.parse()?
            }
            "--max-concurrency" => {
                max_concurrency = next_value(&mut args, "--max-concurrency")?.parse()?
            }
            "--max-ticks" => max_ticks = next_value(&mut args, "--max-ticks")?.parse()?,
            "--coordinator-action" => {
                coordinator_action = Some(next_value(&mut args, "--coordinator-action")?)
            }
            "--target-role" => target_role = Some(next_value(&mut args, "--target-role")?),
            "--urgency" => {
                urgency = next_value(&mut args, "--urgency")?.parse()?;
                urgency_explicit = true;
            }
            "--schedule-id" => schedule_id = next_value(&mut args, "--schedule-id")?,
            "--source-scene-ref" => source_scene_ref = next_value(&mut args, "--source-scene-ref")?,
            "--defer-completion" => defer_completion = true,
            "--role" => role = Some(next_value(&mut args, "--role")?),
            "--action-id" => action_id = Some(next_value(&mut args, "--action-id")?),
            "--limit" => limit = next_value(&mut args, "--limit")?.parse()?,
            "--resident-self-store" => {
                resident_self_store = Some(next_path(&mut args, "--resident-self-store")?)
            }
            "--resident-runtime-store" => {
                resident_runtime_store = Some(next_path(&mut args, "--resident-runtime-store")?)
            }
            "--profile" => profile = next_value(&mut args, "--profile")?,
            "--scene-id" => scene_id = next_value(&mut args, "--scene-id")?,
            "--scene-participant" => scene_participants.push(parse_scene_participant(
                &next_value(&mut args, "--scene-participant")?,
            )?),
            "--scope" => heat_scope = next_value(&mut args, "--scope")?,
            "--selector" => heat_selector = next_value(&mut args, "--selector")?,
            "--multiplier" => heat_multiplier = next_value(&mut args, "--multiplier")?.parse()?,
            "--id" => heat_id = Some(next_value(&mut args, "--id")?),
            "--label" => heat_label = Some(next_value(&mut args, "--label")?),
            "--reason" => heat_reason = Some(next_value(&mut args, "--reason")?),
            "--expires-after" => {
                heat_expires_after = Some(next_value(&mut args, "--expires-after")?.parse()?)
            }
            "--clear" => heat_clear = true,
            "--channel-id" => channel_id = Some(next_value(&mut args, "--channel-id")?),
            "--message-id" => message_id = Some(next_value(&mut args, "--message-id")?),
            "--author-id" => author_id = Some(next_value(&mut args, "--author-id")?),
            "--author-name" => author_name = Some(next_value(&mut args, "--author-name")?),
            "--content" => content = Some(next_value(&mut args, "--content")?),
            "--visible-prompt" => visible_prompt = Some(next_value(&mut args, "--visible-prompt")?),
            "--reply-to-message-id" => {
                reply_to_message_id = Some(next_value(&mut args, "--reply-to-message-id")?)
            }
            "--source-surface" => source_surface = next_value(&mut args, "--source-surface")?,
            "--mention-id" => mention_id = Some(next_value(&mut args, "--mention-id")?),
            "--source-visibility" => {
                source_visibility = next_value(&mut args, "--source-visibility")?
            }
            "--data-classification" => {
                data_classification = next_value(&mut args, "--data-classification")?
            }
            "--model-provider-id" => {
                model_provider_id = next_value(&mut args, "--model-provider-id")?
            }
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
            let state = match profile.as_str() {
                "epiphany" => initialize_heartbeat_store(&store_path, target_heartbeat_rate)?,
                "ghostlight-scene" => initialize_ghostlight_scene_heartbeat_store(
                    &store_path,
                    target_heartbeat_rate,
                    scene_id.clone(),
                    scene_participants,
                )?,
                other => return Err(anyhow!("unknown heartbeat profile {other:?}")),
            };
            println!(
                "{}",
                serde_json::json!({
                    "ok": true,
                    "command": "init",
                    "profile": profile,
                    "storeFile": store_path,
                    "schemaVersion": state.schema_version,
                    "participants": state.participants.len(),
                    "history": state.history.len(),
                })
            );
        }
        "tick" => {
            assert_swarm_brake_allows_heartbeat(
                &local_verse_store,
                release_runtime_id.as_deref(),
                "tick",
            )?;
            let store_path = store_path.ok_or_else(|| anyhow!("tick requires --store"))?;
            let artifact_dir =
                artifact_dir.ok_or_else(|| anyhow!("tick requires --artifact-dir"))?;
            let tick_schedule_id = schedule_id.clone();
            let mut result = tick_heartbeat_store(
                &store_path,
                &artifact_dir,
                HeartbeatTickOptions {
                    target_heartbeat_rate,
                    coordinator_action,
                    target_role,
                    urgency,
                    schedule_id: tick_schedule_id,
                    source_scene_ref,
                    defer_completion,
                    resident_self_store: resident_self_store.clone(),
                },
            )?;
            println!("{}", result);
        }
        "complete" => {
            let store_path = store_path.ok_or_else(|| anyhow!("complete requires --store"))?;
            let artifact_dir =
                artifact_dir.ok_or_else(|| anyhow!("complete requires --artifact-dir"))?;
            let role = role.ok_or_else(|| anyhow!("complete requires --role"))?;
            let result = complete_heartbeat_store(
                &store_path,
                &artifact_dir,
                HeartbeatCompleteOptions { role, action_id },
            )?;
            println!("{}", result);
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
                    reason: heat_reason.unwrap_or_else(|| {
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
        "pump" => {
            assert_swarm_brake_allows_heartbeat(
                &local_verse_store,
                release_runtime_id.as_deref(),
                "pump",
            )?;
            let store_path = store_path.ok_or_else(|| anyhow!("pump requires --store"))?;
            let artifact_dir =
                artifact_dir.ok_or_else(|| anyhow!("pump requires --artifact-dir"))?;
            let result = pump_heartbeat_store(
                &store_path,
                &artifact_dir,
                HeartbeatPumpOptions {
                    base_heartbeat_rate: target_heartbeat_rate,
                    min_heartbeat_rate,
                    max_heartbeat_rate,
                    min_concurrency,
                    max_concurrency,
                    max_ticks,
                    external_urgency: if urgency_explicit { urgency } else { 0.0 },
                    coordinator_action,
                    target_role,
                    schedule_id,
                    source_scene_ref,
                    resident_self_store,
                },
            )?;
            println!("{}", result);
        }
        "heat" => {
            assert_swarm_brake_allows_heartbeat(
                &local_verse_store,
                release_runtime_id.as_deref(),
                "heat",
            )?;
            let store_path = store_path.ok_or_else(|| anyhow!("heat requires --store"))?;
            let result = update_heartbeat_heat_store(
                &store_path,
                HeartbeatHeatUpdateOptions {
                    scope: heat_scope,
                    selector: heat_selector,
                    multiplier: heat_multiplier,
                    id: heat_id,
                    label: heat_label,
                    reason: heat_reason,
                    expires_after_scene_clock: heat_expires_after,
                    clear: heat_clear,
                },
            )?;
            println!("{}", result);
        }
        "queue-mention" => {
            assert_swarm_brake_allows_heartbeat(
                &local_verse_store,
                release_runtime_id.as_deref(),
                "queue-mention",
            )?;
            let store_path = store_path.ok_or_else(|| anyhow!("queue-mention requires --store"))?;
            let target_role_id = role.unwrap_or_else(|| "Persona".to_string());
            let result = queue_heartbeat_pending_mention_store(
                &store_path,
                HeartbeatQueueMentionOptions {
                    target_role_id,
                    source_surface,
                    channel_id: channel_id
                        .ok_or_else(|| anyhow!("queue-mention requires --channel-id"))?,
                    message_id: message_id
                        .ok_or_else(|| anyhow!("queue-mention requires --message-id"))?,
                    author_id: author_id
                        .ok_or_else(|| anyhow!("queue-mention requires --author-id"))?,
                    author_name,
                    content: content.ok_or_else(|| anyhow!("queue-mention requires --content"))?,
                    visible_prompt: visible_prompt
                        .ok_or_else(|| anyhow!("queue-mention requires --visible-prompt"))?,
                    reply_to_message_id,
                    queued_at: None,
                    mention_id,
                    source_visibility,
                    data_classification,
                    model_provider_id,
                    model_provider_disclosure_allowed: true,
                },
            )?;
            println!("{}", result);
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
        "usage: epiphany-heartbeat-store init --store <path> [--profile epiphany|ghostlight-scene] [--scene-id <id>] [--scene-participant <seed>]\n",
        "       epiphany-heartbeat-store tick --store <path> --artifact-dir <path> [options]\n",
        "       epiphany-heartbeat-store pump --store <path> --artifact-dir <path> [options]\n",
        "       epiphany-heartbeat-store heat --store <path> [options]\n",
        "       epiphany-heartbeat-store complete --store <path> --artifact-dir <path> --role <role> [--action-id <id>]\n",
        "       epiphany-heartbeat-store repair-stale --store <path> --artifact-dir <path> [options]\n",
        "       epiphany-heartbeat-store retain-artifacts --store <path> --artifact-dir <path> [--retain-pulse-artifacts <n>] [--retention-batch-size <n>]\n",
        "       epiphany-heartbeat-store queue-mention --store <path> [options]\n",
        "       epiphany-heartbeat-store status --store <path> [--artifact-dir <path>]\n",
        "       epiphany-heartbeat-store serve --store <path> --artifact-dir <path> [--local-verse-store <path>] [--resident-self-store <path> --resident-runtime-store <path>] [--interval-seconds <n>] [--max-iterations <n>]"
    )))
}

fn assert_swarm_brake_allows_heartbeat(
    local_verse_store: &Option<PathBuf>,
    runtime_id: Option<&str>,
    command: &str,
) -> Result<()> {
    if let Some(brake) = active_swarm_brake(local_verse_store, runtime_id)? {
        anyhow::bail!(
            "epiphany-heartbeat-store {command} refusing to run: local Verse swarm brake engaged; scope={}; protected={}; affected={}; reason={}",
            brake.scope,
            brake.protected_surfaces.join(","),
            brake.affected_clusters.join(","),
            brake.reason
        );
    }
    Ok(())
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
    fn retention_receipt_projection_names_operator_fields() {
        let projection = retention_receipt_projection(&EpiphanyHeartbeatArtifactRetentionReceipt {
            schema_version: "epiphany.heartbeat.artifact_retention_receipt.v0".into(),
            receipt_id: "receipt-1".into(),
            plan_id: "plan-1".into(),
            status: "completed".into(),
            deleted_directories: vec!["pulse-000001".into()],
            deleted_file_count: 2,
            deleted_byte_count: 3,
            completed_at_utc: "2026-08-08T21:00:00Z".into(),
            private_state_exposed: false,
        });
        assert_eq!(projection["receiptId"], "receipt-1");
        assert_eq!(projection["deletedDirectories"][0], "pulse-000001");
        assert_eq!(projection["deletedFileCount"], 2);
        assert!(projection.as_array().is_none());
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

fn parse_scene_participant(raw: &str) -> Result<GhostlightSceneParticipantSeed> {
    let parts = raw.split('|').collect::<Vec<_>>();
    if parts.len() != 6 {
        return Err(anyhow!(
            "--scene-participant must be id|name|speed|reaction|threshold|constraint;constraint"
        ));
    }
    Ok(GhostlightSceneParticipantSeed {
        agent_id: parts[0].trim().to_string(),
        display_name: parts[1].trim().to_string(),
        initiative_speed: parts[2].trim().parse()?,
        reaction_bias: parts[3].trim().parse()?,
        interrupt_threshold: parts[4].trim().parse()?,
        constraints: parts[5]
            .split(';')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string)
            .collect(),
    })
}
