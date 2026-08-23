use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::DateTime;
use chrono::Utc;
use serde_json::Value;
use std::fs;
use std::path::Path;

mod heartbeat_documents;
mod heartbeat_projection;
mod heartbeat_retention;
mod heartbeat_roles;
mod heartbeat_store;
pub use heartbeat_documents::*;
pub use heartbeat_projection::heartbeat_status_projection;
use heartbeat_projection::history_event_json;
use heartbeat_projection::schedule_participant_json;
use heartbeat_projection::selection_policy_json;
pub use heartbeat_retention::retain_heartbeat_pulse_artifacts;
use heartbeat_roles::ROLE_ORDER;
pub use heartbeat_roles::default_heartbeat_state;
use heartbeat_roles::default_participant;
pub use heartbeat_roles::initialize_heartbeat_store;
pub use heartbeat_store::commit_heartbeat_state_transaction;
pub use heartbeat_store::heartbeat_state_cache;
pub use heartbeat_store::load_heartbeat_state_entry;
pub use heartbeat_store::load_heartbeat_state_transaction;
pub use heartbeat_store::load_latest_heartbeat_stale_turn_repair_receipt;
pub use heartbeat_store::validate_heartbeat_state;
pub use heartbeat_store::write_heartbeat_stale_turn_repair_receipt;
pub use heartbeat_store::write_heartbeat_state_entry;

pub(super) const HEARTBEAT_ARENA_MAINTENANCE: &str = "maintenance";
pub(super) const PARTICIPANT_KIND_AGENT: &str = "agent";

#[derive(Clone, Debug, PartialEq)]
struct HeartbeatAction {
    action_id: String,
    action_type: &'static str,
    action_scale: &'static str,
    base_recovery: f64,
    initiative_cost: f64,
    interruptibility: f64,
    commitment: f64,
    local_affordance_basis: Vec<String>,
}

fn tick_heartbeat_store(
    store_path: impl AsRef<Path>,
    artifact_dir: impl AsRef<Path>,
    options: HeartbeatTickOptions,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    if options.resident_self_store.as_deref() == Some(store_path) {
        return Err(anyhow!(
            "heartbeat and resident Self require physically separate CultCache stores"
        ));
    }
    let (loaded_state, expected_state) = load_heartbeat_state_transaction(store_path)?;
    let mut state =
        loaded_state.unwrap_or_else(|| default_heartbeat_state(options.target_heartbeat_rate));
    if options.target_heartbeat_rate > 0.0 {
        state.target_heartbeat_rate = options.target_heartbeat_rate;
    }
    patch_missing_participants(&mut state);
    if let Some(resident_store) = options.resident_self_store.as_deref()
        && crate::pending_resident_self_pressure(resident_store)?
        && let Some(pending) = state
            .participants
            .iter()
            .find(|participant| participant.role_id == "coordinator")
            .and_then(|participant| participant.pending_turn.as_ref())
    {
        crate::heartbeat_issue_resident_self_grant(
            resident_store,
            &pending.schedule_id,
            &pending.action_id,
            Utc::now().timestamp_millis().max(0) as u64,
        )?;
    }
    let mut resident_ack_to_consume = None;
    if let Some(resident_store) = options.resident_self_store.as_deref()
        && let Some(pending) = state
            .participants
            .iter()
            .find(|participant| participant.role_id == "coordinator")
            .and_then(|participant| participant.pending_turn.as_ref())
        && let Some(ack) = crate::pending_resident_self_ack_for(
            resident_store,
            &pending.schedule_id,
            &pending.action_id,
        )?
    {
        let index = participant_index_by_role(&state, "coordinator")?;
        complete_pending_turn(&mut state, index)?;
        resident_ack_to_consume = Some((resident_store.to_path_buf(), ack.ack_id));
    }
    let resident_pressure_pending =
        if let Some(resident_store) = options.resident_self_store.as_deref() {
            crate::pending_resident_self_pressure(resident_store)?
        } else {
            false
        };
    let result = tick_once(&mut state, &options, resident_pressure_pending)?;
    commit_heartbeat_state_transaction(store_path, expected_state, &state)?;
    if result["event"]["selectedRole"] == "coordinator"
        && let Some(resident_store) = options.resident_self_store.as_deref()
    {
        let schedule_id = result["event"]["schedule_id"]
            .as_str()
            .or_else(|| result["event"]["scheduleId"].as_str())
            .unwrap_or(&options.schedule_id);
        let action_id = result["event"]["action_id"]
            .as_str()
            .or_else(|| result["event"]["actionId"].as_str())
            .unwrap_or_default();
        crate::heartbeat_issue_resident_self_grant(
            resident_store,
            schedule_id,
            action_id,
            Utc::now().timestamp_millis().max(0) as u64,
        )?;
    }
    if let Some((resident_store, ack_id)) = resident_ack_to_consume {
        crate::heartbeat_consume_resident_self_ack(
            &resident_store,
            &ack_id,
            Utc::now().timestamp_millis().max(0) as u64,
        )?;
    }

    let artifact_dir = artifact_dir.as_ref();
    fs::create_dir_all(artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    write_json_artifact(
        artifact_dir.join(format!("{}.initiative.json", options.schedule_id)),
        &result["schedule"],
    )?;
    write_json_artifact(
        artifact_dir.join(format!("{}.event.json", options.schedule_id)),
        &result["event"],
    )?;

    Ok(serde_json::json!({
        "ok": true,
        "storeFile": store_path,
        "artifactDir": artifact_dir,
        "stateFile": null,
        "event": result["event"].clone(),
        "schedule": result["schedule"].clone(),
    }))
}

pub fn reconcile_resident_self_heartbeat_ack(
    heartbeat_store: impl AsRef<Path>,
    resident_store: impl AsRef<Path>,
) -> Result<Option<String>> {
    let heartbeat_store = heartbeat_store.as_ref();
    let resident_store = resident_store.as_ref();
    if heartbeat_store == resident_store {
        return Err(anyhow!("heartbeat and resident Self stores must differ"));
    }
    let (state, expected) = load_heartbeat_state_transaction(heartbeat_store)?;
    let Some(mut state) = state else {
        return Ok(None);
    };
    let Some((index, pending)) = state
        .participants
        .iter()
        .enumerate()
        .find(|(_, participant)| participant.role_id == "coordinator")
        .and_then(|(index, participant)| {
            participant
                .pending_turn
                .clone()
                .map(|pending| (index, pending))
        })
    else {
        // Recovery for a crash after heartbeat completion committed but before the
        // acknowledgement was marked consumed in the separate resident store.
        let completed = crate::pending_resident_self_acks(resident_store)?
            .into_iter()
            .find(|ack| {
                state.history.iter().any(|event| {
                    event.selected_role == "coordinator"
                        && event.schedule_id == ack.heartbeat_schedule_id
                        && event.action_id == ack.heartbeat_action_id
                        && event.turn_status.as_deref() == Some("completed")
                })
            });
        if let Some(ack) = completed {
            crate::heartbeat_consume_resident_self_ack(
                resident_store,
                &ack.ack_id,
                Utc::now().timestamp_millis().max(0) as u64,
            )?;
            return Ok(Some(ack.ack_id));
        }
        return Ok(None);
    };
    let Some(ack) = crate::pending_resident_self_ack_for(
        resident_store,
        &pending.schedule_id,
        &pending.action_id,
    )?
    else {
        return Ok(None);
    };
    let completed = complete_pending_turn(&mut state, index)?;
    let participant = &state.participants[index];
    state.history.push(HeartbeatHistoryEvent {
        ts: now_iso(),
        schedule_id: completed.schedule_id.clone(),
        selected_role: participant.role_id.clone(),
        selected_agent_id: participant.agent_id.clone(),
        action_id: completed.action_id.clone(),
        action_type: completed.action_type.clone(),
        arena: participant_arena(participant).to_string(),
        participant_kind: participant_kind(participant).to_string(),
        action_scale: completed.action_scale.clone(),
        coordinator_action: None,
        target_role: None,
        work_role: None,
        scene_clock: Some(state.scene_clock),
        next_ready_at: Some(participant.next_ready_at),
        turn_status: Some("completed".into()),
        cooldown_started_after_completion: Some(true),
    });
    trim_history(&mut state);
    commit_heartbeat_state_transaction(heartbeat_store, expected, &state)?;
    crate::heartbeat_consume_resident_self_ack(
        resident_store,
        &ack.ack_id,
        Utc::now().timestamp_millis().max(0) as u64,
    )?;
    Ok(Some(ack.ack_id))
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResidentSelfHeartbeatPulse {
    pub status: String,
    pub acknowledged_terminal_id: Option<String>,
    pub grant_id: Option<String>,
}

pub fn pulse_resident_self_heartbeat(
    heartbeat_store: impl AsRef<Path>,
    resident_store: impl AsRef<Path>,
    artifact_dir: impl AsRef<Path>,
    brake_engaged: bool,
    schedule_id: &str,
    source_scene_ref: &str,
) -> Result<ResidentSelfHeartbeatPulse> {
    let heartbeat_store = heartbeat_store.as_ref();
    let resident_store = resident_store.as_ref();
    let acknowledged_terminal_id =
        reconcile_resident_self_heartbeat_ack(heartbeat_store, resident_store)?;
    if brake_engaged {
        return Ok(ResidentSelfHeartbeatPulse {
            status: "braked-after-ack-reconciliation".into(),
            acknowledged_terminal_id,
            grant_id: None,
        });
    }
    let state = load_heartbeat_state_entry(heartbeat_store)?;
    let active = state
        .as_ref()
        .and_then(|state| {
            state
                .participants
                .iter()
                .find(|participant| participant.role_id == "coordinator")
        })
        .and_then(|participant| participant.pending_turn.as_ref())
        .filter(|turn| turn.status == "running")
        .cloned();
    if let Some(active) = active {
        if crate::pending_resident_self_pressure(resident_store)? {
            let grant = crate::heartbeat_issue_resident_self_grant(
                resident_store,
                &active.schedule_id,
                &active.action_id,
                Utc::now().timestamp_millis().max(0) as u64,
            )?;
            return Ok(ResidentSelfHeartbeatPulse {
                status: "recovered-committed-grant".into(),
                acknowledged_terminal_id,
                grant_id: grant.map(|grant| grant.grant_id),
            });
        }
        return Ok(ResidentSelfHeartbeatPulse {
            status: "active-coordinator-turn".into(),
            acknowledged_terminal_id,
            grant_id: None,
        });
    }
    if !crate::pending_resident_self_pressure(resident_store)? {
        return Ok(ResidentSelfHeartbeatPulse {
            status: "idle".into(),
            acknowledged_terminal_id,
            grant_id: None,
        });
    }
    tick_heartbeat_store(
        heartbeat_store,
        artifact_dir,
        HeartbeatTickOptions {
            target_heartbeat_rate: 1.0,
            schedule_id: schedule_id.into(),
            source_scene_ref: source_scene_ref.into(),
            resident_self_store: Some(resident_store.to_path_buf()),
        },
    )?;
    let grant = crate::pending_resident_self_grant(resident_store)?
        .ok_or_else(|| anyhow!("heartbeat selected resident Self pressure but emitted no grant"))?;
    Ok(ResidentSelfHeartbeatPulse {
        status: "granted".into(),
        acknowledged_terminal_id,
        grant_id: Some(grant.grant_id),
    })
}

pub fn recover_stale_heartbeat_store(
    store_path: impl AsRef<Path>,
    artifact_dir: impl AsRef<Path>,
    options: HeartbeatStaleTurnRepairOptions,
) -> Result<Value> {
    if options.max_age_seconds < 0 {
        return Err(anyhow!(
            "stale heartbeat repair max_age_seconds must be non-negative"
        ));
    }
    if options.reason.trim().is_empty() {
        return Err(anyhow!("stale heartbeat repair requires a reason"));
    }
    let store_path = store_path.as_ref();
    let mut state = load_heartbeat_state_entry(store_path)?.ok_or_else(|| {
        anyhow!(
            "CultCache store {} has no heartbeat state entry",
            store_path.display()
        )
    })?;
    let repaired_at = options.now_utc.clone().unwrap_or_else(now_iso);
    let repaired_at_time = parse_heartbeat_time("repair time", &repaired_at)?;
    let mut receipts = Vec::new();

    for participant in &mut state.participants {
        let pending = match participant.pending_turn.clone() {
            Some(pending) if pending.status == "running" => pending,
            _ => continue,
        };
        let started_at = parse_heartbeat_time("pending turn start time", &pending.started_at)?;
        let stale_age_seconds = repaired_at_time
            .signed_duration_since(started_at)
            .num_seconds();
        if stale_age_seconds < options.max_age_seconds {
            continue;
        }
        participant.pending_turn = None;
        participant.current_load = 0.0;
        participant.next_ready_at = round6(state.scene_clock.max(pending.started_scene_clock));
        let receipt = EpiphanyHeartbeatStaleTurnRepairReceipt {
            schema_version: HEARTBEAT_STALE_TURN_REPAIR_SCHEMA_VERSION.to_string(),
            receipt_id: format!(
                "heartbeat-stale-turn-repair-{}-{}",
                participant.role_id,
                now_stamp()
            ),
            repaired_at_utc: repaired_at.clone(),
            role_id: participant.role_id.clone(),
            agent_id: participant.agent_id.clone(),
            action_id: pending.action_id.clone(),
            schedule_id: pending.schedule_id.clone(),
            started_at_utc: pending.started_at.clone(),
            stale_age_seconds,
            reason: options.reason.clone(),
            resulting_status: "repaired".to_string(),
            next_ready_at: participant.next_ready_at,
            private_state_exposed: false,
            notes: vec![
                "Stale heartbeat repair is a Continuity-facing operator-safe receipt, not silent scheduler cleanup.".to_string(),
                "The repaired lane becomes schedulable only through normal heartbeat selection after the receipt is written.".to_string(),
            ],
        };
        let event = HeartbeatHistoryEvent {
            ts: repaired_at.clone(),
            schedule_id: pending.schedule_id.clone(),
            selected_role: participant.role_id.clone(),
            selected_agent_id: participant.agent_id.clone(),
            action_id: pending.action_id.clone(),
            action_type: pending.action_type.clone(),
            arena: participant_arena(participant).to_string(),
            participant_kind: participant_kind(participant).to_string(),
            action_scale: pending.action_scale.clone(),
            coordinator_action: None,
            target_role: None,
            work_role: None,
            scene_clock: Some(state.scene_clock),
            next_ready_at: Some(participant.next_ready_at),
            turn_status: Some("stale_repaired".to_string()),
            cooldown_started_after_completion: Some(false),
        };
        state.history.push(event);
        receipts.push(receipt);
    }

    if receipts.is_empty() {
        return Ok(serde_json::json!({
            "ok": true,
            "storeFile": store_path,
            "repaired": 0,
            "receipts": [],
            "reviewNotes": [
                "No running heartbeat turns exceeded the stale repair threshold."
            ],
        }));
    }

    trim_history(&mut state);
    write_heartbeat_state_entry(store_path, &state)?;
    let mut written_receipts = Vec::new();
    for receipt in receipts {
        written_receipts.push(write_heartbeat_stale_turn_repair_receipt(
            store_path, &receipt,
        )?);
    }

    let artifact_dir = artifact_dir.as_ref();
    fs::create_dir_all(artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    write_json_artifact(
        artifact_dir.join(format!("heartbeat-stale-repair-{}.json", now_stamp())),
        &serde_json::json!({
            "schemaVersion": "epiphany.heartbeat.stale_repair_artifact.v0",
            "storeFile": store_path,
            "repairedAtUtc": repaired_at,
            "maxAgeSeconds": options.max_age_seconds,
            "receipts": written_receipts,
        }),
    )?;

    Ok(serde_json::json!({
        "ok": true,
        "storeFile": store_path,
        "repaired": written_receipts.len(),
        "receipts": written_receipts,
    }))
}

fn tick_once(
    state: &mut EpiphanyHeartbeatStateEntry,
    options: &HeartbeatTickOptions,
    resident_pressure_pending: bool,
) -> Result<Value> {
    let rate = state.target_heartbeat_rate.max(0.001);
    let work_role = resident_pressure_pending.then_some("coordinator");
    let urgency = if resident_pressure_pending { 1.0 } else { 0.0 };
    let (selected_index, selection_kind, selection_reason) =
        select_participant(state, work_role, urgency, resident_pressure_pending)?;
    let selected = state.participants[selected_index].clone();
    let action = action_for_selection(state, &selected, work_role, rate);
    let scene_clock = state.scene_clock.max(selected.next_ready_at);
    let recovery = action.base_recovery
        / selected
            .initiative_speed
            .max(state.selection_policy.minimum_speed);
    let pending = HeartbeatPendingTurn {
        status: "running".to_string(),
        schedule_id: options.schedule_id.clone(),
        action_id: action.action_id.clone(),
        action_type: action.action_type.to_string(),
        action_scale: action.action_scale.to_string(),
        local_affordance_basis: action.local_affordance_basis.clone(),
        started_at: now_iso(),
        started_scene_clock: round6(scene_clock),
        base_recovery: round6(action.base_recovery),
        recovery: round6(recovery),
        cooldown_policy: "after_turn_completion".to_string(),
        completed_at: None,
        completed_scene_clock: None,
        next_ready_at: None,
        initiative_frozen: true,
        initiative_freeze_reason: Some(
            "Participant has an active heartbeat turn; initiative cannot queue it again until the turn completes."
                .to_string(),
        ),
    };
    state.participants[selected_index].pending_turn = Some(pending.clone());
    state.participants[selected_index].last_action_id = Some(action.action_id.clone());
    state.participants[selected_index].last_woke_at = Some(now_iso());
    state.participants[selected_index].current_load =
        round6((state.participants[selected_index].current_load * 0.75).clamp(0.0, 1.0));
    state.scene_clock = round6(scene_clock);
    let selected_after = state.participants[selected_index].clone();
    let event = HeartbeatHistoryEvent {
        ts: now_iso(),
        schedule_id: options.schedule_id.clone(),
        selected_role: selected_after.role_id.clone(),
        selected_agent_id: selected_after.agent_id.clone(),
        action_id: action.action_id.clone(),
        action_type: action.action_type.to_string(),
        arena: participant_arena(&selected_after).to_string(),
        participant_kind: participant_kind(&selected_after).to_string(),
        action_scale: action.action_scale.to_string(),
        coordinator_action: None,
        target_role: work_role.map(str::to_string),
        work_role: work_role.map(str::to_string),
        scene_clock: Some(state.scene_clock),
        next_ready_at: Some(selected_after.next_ready_at),
        turn_status: Some("running".to_string()),
        cooldown_started_after_completion: Some(true),
    };
    state.history.push(event.clone());
    trim_history(state);

    let readiness_snapshot = readiness_snapshot(state, work_role, urgency);
    let schedule = serde_json::json!({
        "schema_version": INITIATIVE_SCHEMA_VERSION,
        "schedule_id": options.schedule_id,
        "source_scene_ref": options.source_scene_ref,
        "scene_clock": state.scene_clock,
        "participants": state.participants.iter().map(schedule_participant_json).collect::<Vec<_>>(),
        "action_catalog": [{
            "action_id": action.action_id,
            "actor_id": selected_after.agent_id,
            "arena": participant_arena(&selected_after),
            "participant_kind": participant_kind(&selected_after),
            "action_type": action.action_type,
            "action_scale": action.action_scale,
            "base_recovery": action.base_recovery,
            "initiative_cost": action.initiative_cost,
            "interruptibility": action.interruptibility,
            "commitment": action.commitment,
            "local_affordance_basis": action.local_affordance_basis,
        }],
        "reaction_windows": if work_role.is_some() {
            serde_json::json!([{
                "window_id": format!("{}.pending-work", options.schedule_id),
                "trigger_event_ref": options.source_scene_ref,
                "urgency": urgency,
                "eligible_actor_ids": [selected_after.agent_id.clone()],
                "allowed_action_scales": ["short", "standard"],
                "expires_at": round6(state.scene_clock + 1.0),
                "notes": "Pending coordinator work can pull its owning lane forward only if readiness clears threshold."
            }])
        } else {
            serde_json::json!([])
        },
        "selection_policy": selection_policy_json(&state.selection_policy),
        "next_actor_selection": {
            "selection_kind": selection_kind,
            "selected_actor_id": selected_after.agent_id,
            "selected_action_ids": [event.action_id.clone()],
            "scene_clock_after_selection": state.scene_clock,
            "selection_reason": selection_reason,
            "override_reason": null,
            "readiness_snapshot": readiness_snapshot,
        },
        "review_notes": [
            "Epiphany heartbeat records Resident Self scheduling receipts.",
            "A selected idle lane records scheduling opportunity; it does not invent project work or mutate durable memory.",
            "When no coordinator work is active, cooldown and sleep pacing keep the swarm from thrashing."
        ],
    });
    Ok(serde_json::json!({
        "event": history_event_json(event),
        "schedule": schedule,
    }))
}

fn complete_pending_turn(
    state: &mut EpiphanyHeartbeatStateEntry,
    participant_index: usize,
) -> Result<HeartbeatPendingTurn> {
    let pending = state.participants[participant_index]
        .pending_turn
        .clone()
        .ok_or_else(|| {
            anyhow!(
                "{} has no running heartbeat turn",
                state.participants[participant_index].role_id
            )
        })?;
    if pending.status != "running" {
        return Err(anyhow!(
            "{} has no running heartbeat turn",
            state.participants[participant_index].role_id
        ));
    }
    let scene_clock = state.scene_clock.max(pending.started_scene_clock);
    state.participants[participant_index].next_ready_at = round6(scene_clock + pending.recovery);
    state.participants[participant_index].last_finished_at = Some(now_iso());
    let mut completed = pending;
    completed.status = "completed".to_string();
    completed.completed_at = state.participants[participant_index]
        .last_finished_at
        .clone();
    completed.completed_scene_clock = Some(round6(scene_clock));
    completed.next_ready_at = Some(state.participants[participant_index].next_ready_at);
    state.participants[participant_index].pending_turn = None;
    Ok(completed)
}

fn select_participant(
    state: &EpiphanyHeartbeatStateEntry,
    work_role: Option<&str>,
    urgency: f64,
    force_work_role: bool,
) -> Result<(usize, &'static str, String)> {
    let active: Vec<usize> = state
        .participants
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            (item.status == "active" && !is_turn_pending(item)).then_some(index)
        })
        .collect();
    if active.is_empty() {
        return Err(anyhow!("heartbeat has no active participants"));
    }
    if let Some(work_role) = work_role {
        let index = participant_index_by_role(state, work_role)?;
        let candidate = &state.participants[index];
        if is_turn_pending(candidate) {
            let pending = candidate.pending_turn.as_ref();
            return Err(anyhow!(
                "{} already has running heartbeat turn {}; complete it before scheduling another",
                candidate.display_name,
                pending
                    .map(|item| item.action_id.as_str())
                    .unwrap_or("unknown")
            ));
        }
        if force_work_role && candidate.status == "active" {
            return Ok((
                index,
                "admitted_work",
                format!(
                    "Admitted {work_role} work selected its owning role before idle initiative."
                ),
            ));
        }
        let reaction_readiness = candidate.reaction_bias * urgency - candidate.current_load;
        if candidate.status == "active" && reaction_readiness >= candidate.interrupt_threshold {
            return Ok((
                index,
                "reaction_interrupt",
                format!(
                    "{} won a heartbeat reaction window for pending {} work.",
                    candidate.display_name, work_role
                ),
            ));
        }
    }
    let selected = active
        .into_iter()
        .min_by(|left, right| {
            let left_item = &state.participants[*left];
            let right_item = &state.participants[*right];
            left_item
                .next_ready_at
                .total_cmp(&right_item.next_ready_at)
                .then_with(|| {
                    right_item
                        .initiative_speed
                        .total_cmp(&left_item.initiative_speed)
                })
                .then_with(|| left_item.agent_id.cmp(&right_item.agent_id))
        })
        .expect("active participant exists");
    Ok((
        selected,
        "scheduled_turn",
        "No pending work cleared a reaction threshold; earliest ready active lane won the heartbeat slot."
            .to_string(),
    ))
}

fn action_for_selection(
    state: &EpiphanyHeartbeatStateEntry,
    selected: &HeartbeatParticipant,
    work_role: Option<&str>,
    target_heartbeat_rate: f64,
) -> HeartbeatAction {
    let minimum_rate = state.pacing_policy.minimum_effective_rate.max(0.001);
    if Some(selected.role_id.as_str()) == work_role {
        let heartbeat_rate = target_heartbeat_rate.max(minimum_rate);
        let action_id = format!("heartbeat.{}.work", selected.role_id);
        return HeartbeatAction {
            action_id,
            action_type: "role_work",
            action_scale: "standard",
            base_recovery: state.pacing_policy.work_base_recovery / heartbeat_rate,
            initiative_cost: 4.0,
            interruptibility: 0.45,
            commitment: 0.65,
            local_affordance_basis: vec![
                format!("Wake {} for admitted Resident Self pressure.", selected.display_name),
                "Heartbeat slots control opportunity, not project authority.".to_string(),
                "Cooldown starts only after the heartbeat turn completes, so an unfinished sub-agent thread cannot be heartbeaten again.".to_string(),
            ],
        };
    }
    let sleep_multiplier = state
        .pacing_policy
        .sleep_heartbeat_rate_multiplier
        .max(minimum_rate);
    let heartbeat_rate = (target_heartbeat_rate * sleep_multiplier).max(minimum_rate);
    let action_id = format!("heartbeat.{}.idle", selected.role_id);
    HeartbeatAction {
        action_id,
        action_type: "idle",
        action_scale: "short",
        base_recovery: state.pacing_policy.idle_base_recovery / heartbeat_rate,
        initiative_cost: 1.0,
        interruptibility: 0.9,
        commitment: 0.25,
        local_affordance_basis: vec![
            format!(
            "{} has no actionable lane work; preserve the idle scheduling slot without inventing work.",
            selected.display_name
        ),
            "Heartbeat slots control opportunity, not project authority.".to_string(),
            "When no coordinator work is active, cooldown and sleep pacing keep the swarm from thrashing.".to_string(),
        ],
    }
}

fn patch_missing_participants(state: &mut EpiphanyHeartbeatStateEntry) {
    let present: Vec<String> = state
        .participants
        .iter()
        .map(|item| item.role_id.clone())
        .collect();
    for role_id in ROLE_ORDER {
        if !present.iter().any(|present| present == role_id) {
            state.participants.push(default_participant(role_id));
        }
    }
    for participant in &mut state.participants {
        participant
            .pending_turn
            .get_or_insert_with(|| HeartbeatPendingTurn {
                status: String::new(),
                ..HeartbeatPendingTurn::default()
            });
        if participant
            .pending_turn
            .as_ref()
            .is_some_and(|turn| turn.status.is_empty())
        {
            participant.pending_turn = None;
        }
    }
}

fn participant_index_by_role(state: &EpiphanyHeartbeatStateEntry, role_id: &str) -> Result<usize> {
    state
        .participants
        .iter()
        .position(|item| item.role_id == role_id)
        .ok_or_else(|| anyhow!("heartbeat participant role {:?} is missing", role_id))
}

fn is_turn_pending(participant: &HeartbeatParticipant) -> bool {
    participant
        .pending_turn
        .as_ref()
        .is_some_and(|turn| turn.status == "running")
}

fn readiness_snapshot(
    state: &EpiphanyHeartbeatStateEntry,
    work_role: Option<&str>,
    urgency: f64,
) -> Vec<Value> {
    state
        .participants
        .iter()
        .filter(|item| item.status == "active")
        .map(|item| {
            let pending = is_turn_pending(item);
            let eligible =
                !pending && Some(item.role_id.as_str()) == work_role && work_role.is_some();
            let reaction_readiness =
                eligible.then_some(round6(item.reaction_bias * urgency - item.current_load));
            serde_json::json!({
                "agent_id": item.agent_id,
                "arena": participant_arena(item),
                "participant_kind": participant_kind(item),
                "next_ready_at": item.next_ready_at,
                "initiative_frozen": pending,
                "freeze_reason": pending.then_some("running_heartbeat_turn"),
                "reaction_readiness": reaction_readiness,
                "eligible_for_reaction": eligible,
            })
        })
        .collect()
}

pub(super) fn participant_arena(participant: &HeartbeatParticipant) -> &str {
    if participant.arena.trim().is_empty() {
        HEARTBEAT_ARENA_MAINTENANCE
    } else {
        participant.arena.as_str()
    }
}

pub(super) fn participant_kind(participant: &HeartbeatParticipant) -> &str {
    if participant.participant_kind.trim().is_empty() {
        PARTICIPANT_KIND_AGENT
    } else {
        participant.participant_kind.as_str()
    }
}

fn trim_history(state: &mut EpiphanyHeartbeatStateEntry) {
    let len = state.history.len();
    if len > 128 {
        state.history.drain(0..(len - 128));
    }
}

fn write_json_artifact(path: impl AsRef<Path>, value: &Value) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))
        .with_context(|| format!("failed to write {}", path.display()))
}

fn parse_heartbeat_time(label: &str, value: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|time| time.with_timezone(&Utc))
        .with_context(|| format!("failed to parse {label} {value:?} as RFC3339"))
}

fn now_stamp() -> String {
    chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

pub(super) fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PersonaTurnBlockedEvidence, PersonaTurnTerminalOptions};
    use pretty_assertions::assert_eq;

    #[test]
    fn resident_pressure_selects_self_before_idle_lane_initiative() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let heartbeat_store = temp.path().join("heartbeat.cc");
        let resident_store = temp.path().join("resident.cc");
        let artifact_dir = temp.path().join("artifacts");
        initialize_heartbeat_store(&heartbeat_store, 1.0)?;
        let mut heartbeat = load_heartbeat_state_entry(&heartbeat_store)?.unwrap();
        heartbeat
            .participants
            .iter_mut()
            .find(|participant| participant.role_id == "coordinator")
            .unwrap()
            .next_ready_at = 100.0;
        write_heartbeat_state_entry(&heartbeat_store, &heartbeat)?;
        crate::enqueue_resident_self_pressure(
            &resident_store,
            &crate::ResidentSelfPressure {
                schema_version: crate::RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
                pressure_id: "wake-health".into(),
                kind: "operator-objective".into(),
                provenance_ref: "test://wake-health".into(),
                objective: "Perform a bounded health check.".into(),
                created_at_millis: 1,
                status: "pending".into(),
                consumed_by_grant_id: None,
                private_state_exposed: false,
            },
        )?;

        let pulse = pulse_resident_self_heartbeat(
            &heartbeat_store,
            &resident_store,
            &artifact_dir,
            false,
            "wake-health",
            "test://wake-health",
        )?;

        assert_eq!(pulse.status, "granted");
        assert!(pulse.grant_id.is_some());
        let heartbeat = load_heartbeat_state_entry(&heartbeat_store)?.unwrap();
        assert_eq!(
            heartbeat
                .participants
                .iter()
                .find(|participant| participant.role_id == "coordinator")
                .and_then(|participant| participant.pending_turn.as_ref())
                .map(|turn| turn.action_id.as_str()),
            Some("heartbeat.coordinator.work")
        );
        assert!(heartbeat.participants.iter().all(|participant| {
            participant.role_id == "coordinator" || participant.pending_turn.is_none()
        }));
        Ok(())
    }

    #[test]
    fn stale_heartbeat_repair_receipt_clears_running_turn() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let heartbeat_store = temp.path().join("stale-heartbeats.msgpack");
        let resident_store = temp.path().join("resident.cc");
        let artifact_dir = temp.path().join("artifacts");
        initialize_heartbeat_store(&heartbeat_store, 1.0)?;
        crate::enqueue_resident_self_pressure(
            &resident_store,
            &crate::ResidentSelfPressure {
                schema_version: crate::RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
                pressure_id: "stale-work".into(),
                kind: "operator-objective".into(),
                provenance_ref: "test://stale".into(),
                objective: "Exercise stale grant recovery.".into(),
                created_at_millis: 1,
                status: "pending".into(),
                consumed_by_grant_id: None,
                private_state_exposed: false,
            },
        )?;
        pulse_resident_self_heartbeat(
            &heartbeat_store,
            &resident_store,
            &artifact_dir,
            false,
            "stale-work",
            "test://stale",
        )?;

        let mut state = load_heartbeat_state_entry(&heartbeat_store)?.expect("heartbeat state");
        let coordinator = state
            .participants
            .iter_mut()
            .find(|participant| participant.role_id == "coordinator")
            .expect("coordinator participant exists");
        coordinator
            .pending_turn
            .as_mut()
            .expect("running coordinator turn exists")
            .started_at = "2026-06-17T00:00:00+00:00".to_string();
        write_heartbeat_state_entry(&heartbeat_store, &state)?;

        let repaired = recover_stale_heartbeat_store(
            &heartbeat_store,
            &artifact_dir,
            HeartbeatStaleTurnRepairOptions {
                max_age_seconds: 60,
                now_utc: Some("2026-06-17T00:05:00+00:00".to_string()),
                reason: "Test stale Resident Self grant recovery.".to_string(),
            },
        )?;
        assert_eq!(repaired["repaired"], serde_json::json!(1));
        let receipt = load_latest_heartbeat_stale_turn_repair_receipt(&heartbeat_store)?
            .expect("stale-turn repair receipt exists");
        assert_eq!(receipt.role_id, "coordinator");
        assert_eq!(receipt.action_id, "heartbeat.coordinator.work");
        assert_eq!(receipt.stale_age_seconds, 300);
        assert!(!receipt.private_state_exposed);

        let state = load_heartbeat_state_entry(&heartbeat_store)?.expect("heartbeat state");
        assert!(
            state
                .participants
                .iter()
                .find(|participant| participant.role_id == "coordinator")
                .and_then(|participant| participant.pending_turn.as_ref())
                .is_none()
        );
        Ok(())
    }

    #[test]
    fn pending_persona_mention_is_reserved_until_terminal_delivery() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("Persona-heartbeats.msgpack");
        initialize_heartbeat_store(&store_path, 1.0)?;
        let queued = crate::queue_persona_social_mention(
            &store_path,
            crate::PersonaSocialQueueMentionOptions {
                target_role_id: "Persona".to_string(),
                source_surface: "discord".to_string(),
                channel_id: "aquarium".to_string(),
                message_id: "m1".to_string(),
                author_id: "human".to_string(),
                author_name: Some("Metacrat".to_string()),
                content: "Epiphany, answer this through the Persona membrane.".to_string(),
                visible_prompt: "answer this through the Persona membrane".to_string(),
                reply_to_message_id: None,
                queued_at: Some("2026-05-24T00:00:00+00:00".to_string()),
                mention_id: Some("mention-Persona-test".to_string()),
                source_visibility: "public".to_string(),
                data_classification: "public_feedback".to_string(),
                model_provider_id: "openai-codex".to_string(),
                model_provider_disclosure_allowed: true,
            },
        )?;
        assert_eq!(queued["queued"], true);

        let pulse = crate::pulse_persona_social(&store_path, false)?;

        assert_eq!(pulse["status"], "reserved");
        assert_eq!(
            pulse["schedule"]["action_catalog"][0]["pending_mentions"][0]["id"],
            "mention-Persona-test"
        );
        let mentions = crate::pending_persona_mentions(&store_path)?;
        let requests = crate::persona_turn_requests(&store_path)?;
        assert!(
            mentions.is_empty(),
            "reserved mentions are not pending work"
        );
        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(request.status, "reserved");
        assert_eq!(request.mentions[0].id, "mention-Persona-test");

        let terminal = crate::complete_persona_social_turn(
            &store_path,
            PersonaTurnTerminalOptions {
                request_id: request.request_id.clone(),
                outcome: "dropped".to_string(),
                delivery_evidence: None,
                blocked_evidence: None,
            },
        )?;
        assert_eq!(terminal.mention_disposition, "consumed");
        let replay = crate::complete_persona_social_turn(
            &store_path,
            PersonaTurnTerminalOptions {
                request_id: request.request_id.clone(),
                outcome: "dropped".to_string(),
                delivery_evidence: None,
                blocked_evidence: None,
            },
        )?;
        assert_eq!(replay, terminal);
        assert!(crate::pending_persona_mentions(&store_path)?.is_empty());
        let requests = crate::persona_turn_requests(&store_path)?;
        assert_eq!(requests[0].status, "terminal");
        assert!(requests[0].terminal_receipt.is_some());
        Ok(())
    }

    #[test]
    fn failed_persona_turn_releases_lane_but_retains_mention_for_retry() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("Persona-heartbeats.msgpack");
        initialize_heartbeat_store(&store_path, 1.0)?;
        crate::queue_persona_social_mention(
            &store_path,
            crate::PersonaSocialQueueMentionOptions {
                target_role_id: "Persona".to_string(),
                source_surface: "discord".to_string(),
                channel_id: "aquarium".to_string(),
                message_id: "m-failure".to_string(),
                author_id: "human".to_string(),
                author_name: None,
                content: "Retain this pressure across a failed turn.".to_string(),
                visible_prompt: "retain this pressure across failure".to_string(),
                reply_to_message_id: None,
                queued_at: Some("2026-05-24T00:00:00+00:00".to_string()),
                mention_id: Some("mention-Persona-failure".to_string()),
                source_visibility: "public".to_string(),
                data_classification: "public_feedback".to_string(),
                model_provider_id: "openai-codex".to_string(),
                model_provider_disclosure_allowed: true,
            },
        )?;
        crate::pulse_persona_social(&store_path, false)?;
        let request_id = crate::persona_turn_requests(&store_path)?[0]
            .request_id
            .clone();
        let terminal = crate::complete_persona_social_turn(
            &store_path,
            PersonaTurnTerminalOptions {
                request_id,
                outcome: "failed".to_string(),
                delivery_evidence: None,
                blocked_evidence: None,
            },
        )?;
        assert_eq!(terminal.mention_disposition, "retained");
        assert_eq!(crate::pending_persona_mentions(&store_path)?.len(), 1);
        assert_eq!(
            crate::persona_turn_requests(&store_path)?[0].status,
            "terminal"
        );
        Ok(())
    }

    #[test]
    fn indeterminate_local_effect_quarantines_pressure_and_cannot_reschedule() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("Persona-heartbeats.msgpack");
        initialize_heartbeat_store(&store_path, 1.0)?;
        crate::queue_persona_social_mention(
            &store_path,
            crate::PersonaSocialQueueMentionOptions {
                target_role_id: "Persona".to_string(),
                source_surface: "bifrost-discord".to_string(),
                channel_id: "aquarium".to_string(),
                message_id: "m-unknown".to_string(),
                author_id: "human".to_string(),
                author_name: None,
                content: "Do not answer this twice.".to_string(),
                visible_prompt: "do not answer this twice".to_string(),
                reply_to_message_id: Some("m-unknown".to_string()),
                queued_at: Some("2026-05-24T00:00:00+00:00".to_string()),
                mention_id: Some("mention-Persona-unknown".to_string()),
                source_visibility: "public".to_string(),
                data_classification: "public_feedback".to_string(),
                model_provider_id: "openai-codex".to_string(),
                model_provider_disclosure_allowed: true,
            },
        )?;
        let pulse = crate::pulse_persona_social(&store_path, false)?;
        let request_id = pulse["requestId"].as_str().expect("request id").to_string();
        assert!(
            crate::complete_persona_social_turn(
                &store_path,
                PersonaTurnTerminalOptions {
                    request_id: request_id.clone(),
                    outcome: "blocked".to_string(),
                    delivery_evidence: None,
                    blocked_evidence: None,
                },
            )
            .is_err()
        );
        let terminal = crate::complete_persona_social_turn(
            &store_path,
            PersonaTurnTerminalOptions {
                request_id,
                outcome: "blocked".to_string(),
                delivery_evidence: None,
                blocked_evidence: Some(PersonaTurnBlockedEvidence {
                    evidence_source: "local_effect".to_string(),
                    crossing_status: "unknown".to_string(),
                    reason: "Mind mutation may have occurred before the local journal committed"
                        .to_string(),
                    crossing_receipt_id: None,
                    bridge_receipt_sha256: None,
                }),
            },
        )?;
        assert_eq!(terminal.mention_disposition, "quarantined");
        let next = crate::pulse_persona_social(&store_path, false)?;
        assert_eq!(next["status"], "idle");
        assert!(crate::pending_persona_mentions(&store_path)?.is_empty());
        assert_eq!(crate::persona_turn_requests(&store_path)?.len(), 1);
        let blocked = crate::blocked_persona_pressures(&store_path)?;
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].mentions[0].id, "mention-Persona-unknown");
        Ok(())
    }

    #[test]
    fn persona_social_pulse_reserves_once_and_obeys_brake() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("Persona-heartbeats.msgpack");
        initialize_heartbeat_store(&store_path, 1.0)?;
        crate::queue_persona_social_mention(
            &store_path,
            crate::PersonaSocialQueueMentionOptions {
                target_role_id: "Persona".to_string(),
                source_surface: "bifrost-discord".to_string(),
                channel_id: "aquarium".to_string(),
                message_id: "m-pulse".to_string(),
                author_id: "human".to_string(),
                author_name: None,
                content: "Wake the Persona exactly once.".to_string(),
                visible_prompt: "wake the Persona exactly once".to_string(),
                reply_to_message_id: Some("m-pulse".to_string()),
                queued_at: Some("2026-05-24T00:00:00+00:00".to_string()),
                mention_id: Some("mention-Persona-pulse".to_string()),
                source_visibility: "public".to_string(),
                data_classification: "public_feedback".to_string(),
                model_provider_id: "openai-codex".to_string(),
                model_provider_disclosure_allowed: true,
            },
        )?;
        let braked = crate::pulse_persona_social(&store_path, true)?;
        assert_eq!(braked["status"], "refused-by-swarm-brake");
        assert!(crate::persona_turn_requests(&store_path)?.is_empty());

        let first = crate::pulse_persona_social(&store_path, false)?;
        assert_eq!(first["status"], "reserved");
        let second = crate::pulse_persona_social(&store_path, false)?;
        assert_eq!(second["status"], "already-running");
        assert_eq!(crate::persona_turn_requests(&store_path)?.len(), 1);
        Ok(())
    }
}
