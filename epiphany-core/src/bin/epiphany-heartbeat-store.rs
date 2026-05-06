use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use epiphany_core::GhostlightSceneParticipantSeed;
use epiphany_core::HeartbeatCompleteOptions;
use epiphany_core::HeartbeatTickOptions;
use epiphany_core::VoidRoutineOptions;
use epiphany_core::apply_agent_self_patch;
use epiphany_core::complete_heartbeat_store;
use epiphany_core::heartbeat_status_projection;
use epiphany_core::initialize_ghostlight_scene_heartbeat_store;
use epiphany_core::initialize_heartbeat_store;
use epiphany_core::load_heartbeat_state_entry;
use epiphany_core::run_void_routine_store;
use epiphany_core::tick_heartbeat_store;
use epiphany_core::validate_agent_memory_store;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return usage();
    };
    let mut store_path: Option<PathBuf> = None;
    let mut artifact_dir: Option<PathBuf> = None;
    let mut target_heartbeat_rate = 1.0_f64;
    let mut coordinator_action: Option<String> = None;
    let mut target_role: Option<String> = None;
    let mut urgency = 0.75_f64;
    let mut schedule_id = "epiphany-heartbeat".to_string();
    let mut source_scene_ref = "epiphany/coordinator".to_string();
    let mut defer_completion = false;
    let mut role: Option<String> = None;
    let mut action_id: Option<String> = None;
    let mut limit = 8_usize;
    let mut agent_store: Option<PathBuf> = None;
    let mut apply_rumination = false;
    let mut profile = "epiphany".to_string();
    let mut scene_id = "ghostlight.scene".to_string();
    let mut scene_participants = Vec::<GhostlightSceneParticipantSeed>::new();
    let mut source = "epiphany/void-routine".to_string();
    let mut no_dream = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--store" => store_path = Some(next_path(&mut args, "--store")?),
            "--artifact-dir" => artifact_dir = Some(next_path(&mut args, "--artifact-dir")?),
            "--target-heartbeat-rate" => {
                target_heartbeat_rate = next_value(&mut args, "--target-heartbeat-rate")?.parse()?
            }
            "--coordinator-action" => {
                coordinator_action = Some(next_value(&mut args, "--coordinator-action")?)
            }
            "--target-role" => target_role = Some(next_value(&mut args, "--target-role")?),
            "--urgency" => urgency = next_value(&mut args, "--urgency")?.parse()?,
            "--schedule-id" => schedule_id = next_value(&mut args, "--schedule-id")?,
            "--source-scene-ref" => source_scene_ref = next_value(&mut args, "--source-scene-ref")?,
            "--defer-completion" => defer_completion = true,
            "--role" => role = Some(next_value(&mut args, "--role")?),
            "--action-id" => action_id = Some(next_value(&mut args, "--action-id")?),
            "--limit" => limit = next_value(&mut args, "--limit")?.parse()?,
            "--agent-store" => agent_store = Some(next_path(&mut args, "--agent-store")?),
            "--apply-rumination" => apply_rumination = true,
            "--profile" => profile = next_value(&mut args, "--profile")?,
            "--scene-id" => scene_id = next_value(&mut args, "--scene-id")?,
            "--scene-participant" => scene_participants.push(parse_scene_participant(
                &next_value(&mut args, "--scene-participant")?,
            )?),
            "--source" => source = next_value(&mut args, "--source")?,
            "--no-dream" => no_dream = true,
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
            let store_path = store_path.ok_or_else(|| anyhow!("tick requires --store"))?;
            let artifact_dir =
                artifact_dir.ok_or_else(|| anyhow!("tick requires --artifact-dir"))?;
            if let Some(agent_store) = &agent_store {
                let errors = validate_agent_memory_store(agent_store)?;
                if !errors.is_empty() {
                    return Err(anyhow!(
                        "agent memory validation failed: {}",
                        errors.join("; ")
                    ));
                }
            }
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
                },
            )?;
            if apply_rumination {
                let agent_store = agent_store
                    .as_ref()
                    .ok_or_else(|| anyhow!("--apply-rumination requires --agent-store"))?;
                apply_rumination_patch(&mut result, agent_store, &artifact_dir, &schedule_id)?;
            }
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
        "routine" => {
            let store_path = store_path.ok_or_else(|| anyhow!("routine requires --store"))?;
            let artifact_dir =
                artifact_dir.ok_or_else(|| anyhow!("routine requires --artifact-dir"))?;
            if let Some(agent_store) = &agent_store {
                let errors = validate_agent_memory_store(agent_store)?;
                if !errors.is_empty() {
                    return Err(anyhow!(
                        "agent memory validation failed: {}",
                        errors.join("; ")
                    ));
                }
            }
            let result = run_void_routine_store(
                &store_path,
                &artifact_dir,
                VoidRoutineOptions {
                    agent_store,
                    source,
                    allow_dream: !no_dream,
                },
            )?;
            println!("{}", result);
        }
        "smoke" => {
            let agent_store = agent_store.unwrap_or_else(|| PathBuf::from("state/agents.msgpack"));
            let result = run_smoke(&agent_store)?;
            let ok = result["ok"].as_bool().unwrap_or(false);
            println!("{}", serde_json::to_string_pretty(&result)?);
            if !ok {
                std::process::exit(1);
            }
        }
        _ => return usage(),
    }

    Ok(())
}

fn next_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(next_value(args, name)?))
}

fn next_value(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("{name} requires a value"))
}

fn usage() -> Result<()> {
    Err(anyhow!(
        "usage: epiphany-heartbeat-store init --store <path> [--profile epiphany|ghostlight-scene] [--scene-id <id>] [--scene-participant <id|name|speed|reaction|threshold|constraint;constraint>]\n       epiphany-heartbeat-store tick --store <path> --artifact-dir <path> [--coordinator-action <action>] [--agent-store <path> --apply-rumination] [--defer-completion]\n       epiphany-heartbeat-store complete --store <path> --artifact-dir <path> --role <role> [--action-id <id>]\n       epiphany-heartbeat-store status --store <path> [--artifact-dir <path>]\n       epiphany-heartbeat-store routine --store <path> --artifact-dir <path> [--agent-store <path>] [--source <source>] [--no-dream]\n       epiphany-heartbeat-store smoke [--agent-store <path>]"
    ))
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

fn run_smoke(agent_store: &Path) -> Result<serde_json::Value> {
    let temp_dir = scoped_temp_dir("epiphany-heartbeat-smoke")?;
    let temp_agent_store = temp_dir.join("agents.msgpack");
    let store_path = temp_dir.join("heartbeats.msgpack");
    let artifact_dir = temp_dir.join("artifacts");
    fs::copy(agent_store, &temp_agent_store).with_context(|| {
        format!(
            "failed to copy {} to {}",
            agent_store.display(),
            temp_agent_store.display()
        )
    })?;

    let initial_errors = validate_agent_memory_store(&temp_agent_store)?;
    if !initial_errors.is_empty() {
        let _ = fs::remove_dir_all(&temp_dir);
        return Ok(serde_json::json!({
            "ok": false,
            "phase": "agent-memory-validate",
            "validationErrors": initial_errors,
        }));
    }

    initialize_heartbeat_store(&store_path, 1.0)?;
    let work = tick_heartbeat_store(
        &store_path,
        &artifact_dir,
        HeartbeatTickOptions {
            target_heartbeat_rate: 1.0,
            coordinator_action: Some("continueImplementation".to_string()),
            target_role: None,
            urgency: 0.95,
            schedule_id: "smoke-work".to_string(),
            source_scene_ref: "smoke/coordinator".to_string(),
            defer_completion: true,
        },
    )?;
    let blocked_repeat = tick_heartbeat_store(
        &store_path,
        &artifact_dir,
        HeartbeatTickOptions {
            target_heartbeat_rate: 1.0,
            coordinator_action: Some("continueImplementation".to_string()),
            target_role: None,
            urgency: 0.95,
            schedule_id: "smoke-work".to_string(),
            source_scene_ref: "smoke/coordinator".to_string(),
            defer_completion: true,
        },
    )
    .err()
    .map(|error| error.to_string());

    let action_id = work["event"]["actionId"]
        .as_str()
        .ok_or_else(|| anyhow!("smoke work event has no actionId"))?
        .to_string();
    let completed = complete_heartbeat_store(
        &store_path,
        &artifact_dir,
        HeartbeatCompleteOptions {
            role: "implementation".to_string(),
            action_id: Some(action_id),
        },
    )?;
    let mut idle = tick_heartbeat_store(
        &store_path,
        &artifact_dir,
        HeartbeatTickOptions {
            target_heartbeat_rate: 1.0,
            coordinator_action: None,
            target_role: None,
            urgency: 0.0,
            schedule_id: "smoke-idle".to_string(),
            source_scene_ref: "smoke/idle".to_string(),
            defer_completion: false,
        },
    )?;

    if let Some(patch) = idle["rumination"]["selfPatch"].as_object() {
        let role_id = idle["rumination"]["roleId"]
            .as_str()
            .ok_or_else(|| anyhow!("idle rumination has no roleId"))?
            .to_string();
        let patch = serde_json::Value::Object(patch.clone());
        let applied = apply_agent_self_patch(&role_id, &patch, &temp_agent_store)?;
        idle["rumination"]["result"] = serde_json::to_value(applied)?;
        idle["rumination"]["applied"] = serde_json::Value::Bool(true);
        write_rumination_artifact(&artifact_dir, "smoke-idle", &idle["rumination"])?;
    }

    let validation_errors = validate_agent_memory_store(&temp_agent_store)?;
    let status = heartbeat_status_projection(&store_path, &artifact_dir, 1.0, 8)?;
    let routine = run_void_routine_store(
        &store_path,
        &artifact_dir,
        VoidRoutineOptions {
            agent_store: Some(temp_agent_store.clone()),
            source: "smoke/void-routine".to_string(),
            allow_dream: true,
        },
    )?;
    let routine_status = heartbeat_status_projection(&store_path, &artifact_dir, 1.0, 8)?;
    let initiative_errors = validate_schedule_shape(&work["schedule"])
        .into_iter()
        .chain(validate_schedule_shape(&idle["schedule"]))
        .collect::<Vec<_>>();
    let ok = work["event"]["selectedRole"] == "implementation"
        && work["event"]["turnStatus"] == "running"
        && blocked_repeat
            .as_deref()
            .is_some_and(|message| message.contains("already has running heartbeat turn"))
        && completed["event"]["turnStatus"] == "completed"
        && idle["event"]["actionType"] == "ruminate_memory"
        && idle["event"]["turnStatus"] == "completed"
        && idle["event"]["nextReadyAt"].as_f64().unwrap_or_default()
            > completed["event"]["nextReadyAt"]
                .as_f64()
                .unwrap_or_default()
        && idle["rumination"]["result"]["status"] == "accepted"
        && validation_errors.is_empty()
        && initiative_errors.is_empty()
        && artifact_dir.join("smoke-work.initiative.json").exists()
        && artifact_dir.join("smoke-work.completion.json").exists()
        && artifact_dir.join("smoke-idle.rumination.json").exists()
        && status["participants"].as_array().map(Vec::len) == Some(8)
        && routine["routine"]["schema_version"] == "epiphany.void_routine.v0"
        && routine["routine"]["memoryResonance"]["schema_version"]
            == "epiphany.memory_resonance.v0"
        && routine["routine"]["incubation"]["schema_version"] == "epiphany.incubation.v0"
        && routine["routine"]["thoughtLanes"]["schema_version"] == "epiphany.cognition_lanes.v0"
        && routine["routine"]["bridge"]["schema_version"] == "epiphany.cognition_bridge.v0"
        && routine["routine"]["candidateInterventions"]["schema_version"]
            == "epiphany.candidate_interventions.v0"
        && routine_status["sleepCycle"]["schema_version"] == "epiphany.sleep_cycle.v0";

    let result = serde_json::json!({
        "ok": ok,
        "workEvent": work["event"],
        "blockedRepeat": blocked_repeat,
        "completionEvent": completed["event"],
        "idleEvent": idle["event"],
        "idleRumination": idle["rumination"],
        "voidRoutine": routine["routine"],
        "validationErrors": validation_errors,
        "initiativeErrors": initiative_errors,
    });
    let _ = fs::remove_dir_all(&temp_dir);
    Ok(result)
}

fn apply_rumination_patch(
    result: &mut serde_json::Value,
    agent_store: &Path,
    artifact_dir: &Path,
    schedule_id: &str,
) -> Result<()> {
    if result["rumination"].is_null() {
        return Ok(());
    }
    let Some(patch) = result["rumination"]["selfPatch"].as_object() else {
        return Ok(());
    };
    let role_id = result["rumination"]["roleId"]
        .as_str()
        .ok_or_else(|| anyhow!("rumination has no roleId"))?
        .to_string();
    let patch = serde_json::Value::Object(patch.clone());
    let applied = apply_agent_self_patch(&role_id, &patch, agent_store)?;
    result["rumination"]["result"] = serde_json::to_value(applied)?;
    result["rumination"]["applied"] = serde_json::Value::Bool(true);
    write_rumination_artifact(artifact_dir, schedule_id, &result["rumination"])?;
    Ok(())
}

fn write_rumination_artifact(
    artifact_dir: &Path,
    schedule_id: &str,
    value: &serde_json::Value,
) -> Result<()> {
    fs::create_dir_all(artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    let path = artifact_dir.join(format!("{schedule_id}.rumination.json"));
    fs::write(&path, format!("{}\n", serde_json::to_string_pretty(value)?))
        .with_context(|| format!("failed to write {}", path.display()))
}

fn validate_schedule_shape(schedule: &serde_json::Value) -> Vec<String> {
    let mut errors = Vec::new();
    let required = [
        "schema_version",
        "schedule_id",
        "source_scene_ref",
        "scene_clock",
        "participants",
        "action_catalog",
        "reaction_windows",
        "selection_policy",
        "next_actor_selection",
        "review_notes",
    ];
    for key in required {
        if schedule.get(key).is_none() {
            errors.push(format!("initiative schedule missing {key}"));
        }
    }
    if schedule["schema_version"] != "ghostlight.initiative_schedule.v0" {
        errors.push("initiative schedule has wrong schema_version".to_string());
    }
    if !schedule["scene_clock"].is_number()
        || schedule["scene_clock"]
            .as_f64()
            .is_some_and(|clock| clock < 0.0)
    {
        errors.push("initiative schedule scene_clock must be non-negative number".to_string());
    }
    for participant in schedule["participants"].as_array().into_iter().flatten() {
        if ![
            "active",
            "blocked",
            "withdrawn",
            "incapacitated",
            "offscreen",
        ]
        .contains(&participant["status"].as_str().unwrap_or_default())
        {
            errors.push(format!(
                "participant {:?} has invalid status",
                participant["agent_id"]
            ));
        }
        for key in [
            "initiative_speed",
            "next_ready_at",
            "reaction_bias",
            "interrupt_threshold",
            "current_load",
        ] {
            if !participant[key].is_number() {
                errors.push(format!(
                    "participant {:?} {key} must be numeric",
                    participant["agent_id"]
                ));
            }
        }
    }
    for action in schedule["action_catalog"].as_array().into_iter().flatten() {
        if ![
            "speak",
            "silence",
            "move",
            "gesture",
            "touch_object",
            "block_object",
            "use_object",
            "show_object",
            "withhold_object",
            "transfer_object",
            "spend_resource",
            "attack",
            "wait",
            "mixed",
            "role_work",
            "ruminate_memory",
            "scene_turn",
        ]
        .contains(&action["action_type"].as_str().unwrap_or_default())
        {
            errors.push(format!(
                "action {:?} has invalid action_type",
                action["action_id"]
            ));
        }
        if !["micro", "short", "standard", "major", "committed"]
            .contains(&action["action_scale"].as_str().unwrap_or_default())
        {
            errors.push(format!(
                "action {:?} has invalid action_scale",
                action["action_id"]
            ));
        }
    }
    let selection = &schedule["next_actor_selection"];
    if ![
        "scheduled_turn",
        "reaction_interrupt",
        "coordinator_override",
    ]
    .contains(&selection["selection_kind"].as_str().unwrap_or_default())
    {
        errors.push("next_actor_selection has invalid selection_kind".to_string());
    }
    for snapshot in selection["readiness_snapshot"]
        .as_array()
        .into_iter()
        .flatten()
    {
        if let Some(object) = snapshot.as_object() {
            let extra_keys = object
                .keys()
                .filter(|key| {
                    ![
                        "agent_id",
                        "arena",
                        "participant_kind",
                        "next_ready_at",
                        "reaction_readiness",
                        "eligible_for_reaction",
                    ]
                    .contains(&key.as_str())
                })
                .cloned()
                .collect::<Vec<_>>();
            if !extra_keys.is_empty() {
                errors.push(format!(
                    "readiness snapshot has Ghostlight-incompatible keys: {extra_keys:?}"
                ));
            }
        }
    }
    errors
}

fn scoped_temp_dir(prefix: &str) -> Result<PathBuf> {
    let path = env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&path).with_context(|| format!("failed to create {}", path.display()))?;
    Ok(path)
}
