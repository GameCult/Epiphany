use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use epiphany_core::{
    ChildObservation, CoordinatorLaunch, LaunchedCoordinator, ProcessInstanceIdentity,
    ProcessInstanceObservation, ResidentProviderReadiness, ResidentReadinessRequest,
    ResidentSelfOutcome, ResidentSelfPolicy, ResidentSelfPorts, ResidentSelfPressure,
    ResidentSelfState, acknowledge_resident_self_launch, acquire_resident_process_singleton,
    authenticate_resident_self_policy, bind_runtime_repository_domain,
    bridge_admitted_persona_feedback_to_heartbeat, cancel_resident_self_turn,
    capture_process_instance, complete_resident_self_turn_after_death,
    coordinator_run_receipts, derive_resident_cognition_readiness,
    enqueue_resident_self_pressure, import_bifrost_persona_feedback_deliveries,
    ingest_resident_self_domain_pressure, load_epiphany_cultmesh_swarm_brake,
    load_resident_self_state, observe_process_instance, pending_resident_self_acks,
    prepare_resident_self_launch, publish_resident_provider_readiness,
    recover_receipt_free_dead_coordinator_session,
    resident_cognitive_runtime_id,
    resident_prepared_launch_thread_id, resident_self_child_claim,
    resident_self_grant_has_typed_request, resident_self_local_provider_status,
    resident_self_typed_attempt_exists,
    retain_completed_runtime_sessions,
    retain_coordinator_run_receipts, retain_resident_self_lifecycles,
    settle_resident_self_exited_coordinator, terminate_process_instance,
    validate_persona_feedback_store_separation,
    validate_resident_self_coordinator_receipt_binding,
    validate_resident_self_store_separation, verify_resident_self_grant_fulfillment,
};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    let mut args = Args::parse()?;
    authenticate_resident_self_policy(&mut args.policy)?;
    args.policy.validate()?;
    validate_resident_self_store_separation(&args.state_store, &args.policy)?;
    validate_persona_feedback_store_separation(
        &args.persona_feedback_source_store,
        &args.persona_feedback_store,
        &[
            &args.bifrost_feedback_trust_anchor,
            &args.state_store,
            &args.policy.runtime_store,
            &args.policy.local_verse_store,
            &args.policy.agent_memory_store,
            &args.policy.release_store,
        ],
    )?;
    if args.heartbeat_store == args.state_store {
        return Err(anyhow!(
            "heartbeat and resident Self stores must be physically separate"
        ));
    }
    if matches!(args.command, CommandKind::Status) {
        let now = Utc::now().timestamp_millis().max(0) as u64;
        let projection = derive_resident_cognition_readiness(ResidentReadinessRequest {
            release_store: &args.policy.release_store,
            heartbeat_store: &args.heartbeat_store,
            resident_store: &args.state_store,
            policy: &args.policy,
            release_runtime_id: &args.policy.release_runtime_id,
            release_id: &args.policy.release_id,
            release_witness_sha256: &args.policy.release_witness_sha256,
            now_millis: now,
            freshness_millis: args.provider_freshness_seconds.saturating_mul(1000),
        });
        println!("{}", serde_json::to_string_pretty(&projection)?);
        return Ok(());
    }
    let _singleton = acquire_resident_process_singleton("resident-self", &args.state_store)?;
    if let Some(pressure) = args.pressure.as_ref() {
        enqueue_resident_self_pressure(&args.state_store, pressure)?;
    }
    let mut state = load_resident_self_state(&args.state_store)?;
    let mut ports = NativePorts::new(&args.policy)?;
    match args.command {
        CommandKind::Once => {
            let outcome = cycle(&args, &mut state, &mut ports, false)?;
            retain_runtime_receipts(&args, &state)?;
            retain_resident_self_lifecycles(
                &args.state_store,
                args.retained_closed_lifecycles,
                Utc::now().timestamp_millis().max(0) as u64,
            )?;
            publish_self_readiness(&args)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&summary(&state, &outcome, false))?
            );
        }
        CommandKind::Serve => {
            let shutdown_requested = install_shutdown_signal_owner()?;
            loop {
                let shutting_down = shutdown_requested.load(Ordering::SeqCst);
                let outcome = cycle(&args, &mut state, &mut ports, shutting_down)?;
                retain_runtime_receipts(&args, &state)?;
                retain_resident_self_lifecycles(
                    &args.state_store,
                    args.retained_closed_lifecycles,
                    Utc::now().timestamp_millis().max(0) as u64,
                )?;
                publish_self_readiness(&args)?;
                println!(
                    "{}",
                    serde_json::to_string(&summary(&state, &outcome, shutting_down))?
                );
                if shutting_down && state.active_turn.is_none() {
                    break;
                }
                let seconds = match outcome {
                    ResidentSelfOutcome::Failed => args.policy.failure_backoff_seconds,
                    ResidentSelfOutcome::Completed => args.policy.cooldown_seconds,
                    _ => args.policy.idle_sleep_seconds,
                };
                wait_for_shutdown(&shutdown_requested, Duration::from_secs(seconds.max(1)));
            }
        }
        CommandKind::Status => unreachable!("status returned before actuation setup"),
    }
    Ok(())
}

fn retain_runtime_receipts(args: &Args, state: &ResidentSelfState) -> Result<()> {
    if !runtime_receipt_retention_allowed(
        state.active_turn.is_some(),
        state.prepared_launch.is_some(),
    ) {
        return Ok(());
    }
    let mut preserved = pending_resident_self_acks(&args.state_store)?
        .into_iter()
        .map(|ack| ack.coordinator_receipt_id)
        .collect::<BTreeSet<_>>();
    if let Some(receipt_id) = &state.last_coordinator_receipt_id {
        preserved.insert(receipt_id.clone());
    }
    let retained_at = Utc::now().to_rfc3339();
    retain_completed_runtime_sessions(
        &args.policy.runtime_store,
        args.retained_completed_runtime_sessions,
        &preserved,
        &retained_at,
    )?;
    retain_coordinator_run_receipts(
        &args.policy.runtime_store,
        args.retained_coordinator_receipts,
        &preserved,
        &retained_at,
    )?;
    Ok(())
}

fn runtime_receipt_retention_allowed(has_active_turn: bool, has_prepared_launch: bool) -> bool {
    !has_active_turn && !has_prepared_launch
}

fn publish_self_readiness(args: &Args) -> Result<()> {
    let process = capture_process_instance(std::process::id())?;
    publish_resident_provider_readiness(
        &args.state_store,
        ResidentProviderReadiness {
            schema_version: epiphany_core::RESIDENT_PROVIDER_READINESS_SCHEMA_VERSION.into(),
            provider: "resident-self".into(),
            runtime_id: args.policy.release_runtime_id.clone(),
            release_id: args.policy.release_id.clone(),
            release_witness_sha256: args.policy.release_witness_sha256.clone(),
            source_commit: args.policy.release_commit.clone(),
            publisher_sequence: 0,
            observed_at_millis: Utc::now().timestamp_millis().max(0) as u64,
            process_id: process.process_id,
            process_creation_token: process.creation_token,
            process_executable_path: process.executable_path.display().to_string(),
            status: resident_self_local_provider_status(&args.state_store, &args.policy).into(),
            private_state_exposed: false,
        },
    )?;
    Ok(())
}

fn run_feedback_import_if_released(
    brake_engaged: bool,
    import: impl FnOnce() -> Result<()>,
) -> Result<()> {
    if brake_engaged {
        return Ok(());
    }
    import()
}

fn run_cycle_ingress_if_running(
    shutdown_requested: bool,
    ingress: impl FnOnce() -> Result<()>,
) -> Result<()> {
    if shutdown_requested {
        return Ok(());
    }
    ingress()
}

fn install_shutdown_signal_owner() -> Result<Arc<AtomicBool>> {
    let requested = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&requested);
    ctrlc::try_set_handler(move || {
        signal.store(true, Ordering::SeqCst);
    })
    .context("failed to install resident Self shutdown signal owner")?;
    Ok(requested)
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

fn cancelled_turn_status(
    shutdown_requested: bool,
    brake_engaged: bool,
    timed_out: bool,
) -> &'static str {
    if shutdown_requested {
        "shutdown-cancelled"
    } else if brake_engaged {
        "brake-cancelled"
    } else if timed_out {
        "timed-out"
    } else {
        "process-failed"
    }
}

fn outcome_after_cancel(shutdown_requested: bool) -> ResidentSelfOutcome {
    if shutdown_requested {
        ResidentSelfOutcome::Braked
    } else {
        ResidentSelfOutcome::Failed
    }
}

fn exact_resident_coordinator_receipt(
    runtime_store: &std::path::Path,
    lease: &epiphany_core::ResidentSelfTurnLease,
) -> Result<Option<epiphany_core::EpiphanyCoordinatorRunReceipt>> {
    let session_id = epiphany_core::coordinator_run_session_id(
        &lease.turn_id,
        Some(&lease.launch_digest),
    )?;
    let mut receipts = coordinator_run_receipts(runtime_store)?
        .into_iter()
        .filter(|receipt| receipt.session_id == session_id)
        .collect::<Vec<_>>();
    if receipts.len() > 1 {
        return Err(anyhow!(
            "resident coordinator session has multiple terminal receipts"
        ));
    }
    let Some(receipt) = receipts.pop() else {
        return Ok(None);
    };
    validate_resident_self_coordinator_receipt_binding(lease, &receipt)?;
    Ok(Some(receipt))
}

fn settle_receipt_free_dead_coordinator(
    args: &Args,
    lease: &epiphany_core::ResidentSelfTurnLease,
    observation: ChildObservation,
    shutdown_requested: bool,
    brake_engaged: bool,
    timed_out: bool,
    now: u64,
) -> Result<ResidentSelfOutcome> {
    let typed = resident_self_grant_has_typed_request(&args.state_store, &lease.grant_id)?;
    if typed {
        match verify_resident_self_grant_fulfillment(
            &args.state_store,
            &args.policy.runtime_store,
            &lease.grant_id,
        )? {
            epiphany_core::ResidentSelfGrantFulfillment::Fulfilled => {
                let recovery = recover_receipt_free_dead_coordinator_session(
                    &args.state_store,
                    &args.policy.runtime_store,
                    lease,
                    observation,
                    now,
                )?
                .ok_or_else(|| {
                    anyhow!(
                        "typed fulfillment cannot predate its coordinator runtime incarnation"
                    )
                })?;
                complete_resident_self_turn_after_death(
                    &args.state_store,
                    lease,
                    &recovery,
                    now,
                    args.policy.cooldown_seconds,
                )?;
                return Ok(ResidentSelfOutcome::Completed);
            }
            epiphany_core::ResidentSelfGrantFulfillment::Pending => {
                if resident_self_typed_attempt_exists(
                    &args.state_store,
                    &args.policy.runtime_store,
                    &lease.grant_id,
                )? {
                    return Ok(ResidentSelfOutcome::AwaitingFulfillment);
                }
            }
        }
    }
    let recovery = recover_receipt_free_dead_coordinator_session(
        &args.state_store,
        &args.policy.runtime_store,
        lease,
        observation,
        now,
    )?;
    cancel_resident_self_turn(
        &args.state_store,
        lease,
        cancelled_turn_status(shutdown_requested, brake_engaged, timed_out),
        if recovery.is_some() {
            "exact coordinator process died after atomic opening and before a terminal receipt"
        } else {
            "exact coordinator process died before atomic runtime opening"
        },
        now,
    )?;
    Ok(outcome_after_cancel(shutdown_requested))
}

fn cycle(
    args: &Args,
    state: &mut ResidentSelfState,
    ports: &mut NativePorts,
    shutdown_requested: bool,
) -> Result<ResidentSelfOutcome> {
    let now = Utc::now().timestamp_millis().max(0) as u64;
    let brake_engaged = ports.brake_engaged()?;
    run_feedback_import_if_released(brake_engaged || shutdown_requested, || {
        import_bifrost_persona_feedback_deliveries(
            &args.persona_feedback_source_store,
            &args.persona_feedback_store,
            &args.bifrost_feedback_trust_anchor,
            &args.policy.release_runtime_id,
            &args.feedback_target_repository,
            &args.feedback_target_persona,
        )?;
        bridge_admitted_persona_feedback_to_heartbeat(
            &args.persona_feedback_store,
            &args.heartbeat_store,
            &args.policy.release_runtime_id,
            &args.policy.model_provider,
            &args.persona_model_allowed_data_classifications,
        )?;
        Ok(())
    })?;
    run_cycle_ingress_if_running(shutdown_requested, || {
        bind_runtime_repository_domain(
            &args.policy.runtime_store,
            &args.feedback_target_repository,
            &Utc::now().to_rfc3339(),
        )?;
        ingest_resident_self_domain_pressure(
            &args.state_store,
            &args.policy.runtime_store,
            &args.persona_feedback_store,
            &args.policy.release_runtime_id,
            &args.feedback_target_repository,
            &args.policy.workspace.display().to_string(),
            now,
        )?;
        Ok(())
    })?;
    *state = load_resident_self_state(&args.state_store)?;
    if let Some(prepared) = state.prepared_launch.clone() {
        if let Some(claim) = resident_self_child_claim(&args.state_store, &prepared.preparation_id)?
        {
            acknowledge_resident_self_launch(
                &args.state_store,
                &prepared.preparation_id,
                &LaunchedCoordinator {
                    process_id: claim.process_id,
                    process_creation_token: claim.process_creation_token,
                    process_executable_path: claim.executable_path,
                },
                claim.claimed_at_millis,
            )?;
            *state = load_resident_self_state(&args.state_store)?;
            return Ok(ResidentSelfOutcome::Running);
        }
        if shutdown_requested || ports.brake_engaged()? {
            return Ok(ResidentSelfOutcome::Braked);
        }
        let launch = CoordinatorLaunch {
            turn_id: resident_prepared_launch_thread_id(&prepared)?,
            wake: epiphany_core::ResidentSelfWake::Explicit {
                objective: prepared.grant.objective.clone(),
            },
            argv: prepared.argv.clone(),
        };
        let process = ports.launch_coordinator(&launch)?;
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while resident_self_child_claim(&args.state_store, &prepared.preparation_id)?.is_none() {
            if std::time::Instant::now() >= deadline {
                return Err(anyhow!(
                    "retried coordinator did not claim prepared authority; preparation remains fail-closed"
                ));
            }
            thread::sleep(Duration::from_millis(50));
        }
        acknowledge_resident_self_launch(
            &args.state_store,
            &prepared.preparation_id,
            &process,
            now,
        )?;
        *state = load_resident_self_state(&args.state_store)?;
        return Ok(ResidentSelfOutcome::Launched);
    }
    if let Some(lease) = state.active_turn.clone() {
        let brake_engaged = ports.brake_engaged()?;
        let timed_out = now.saturating_sub(lease.started_at_millis)
            > args.policy.turn_timeout_seconds.saturating_mul(1000);
        if shutdown_requested || brake_engaged || timed_out {
            if ports.observe_child(&lease)? == ChildObservation::Running {
                ports.request_child_stop(&lease)?;
                return Ok(ResidentSelfOutcome::Draining);
            }
        }
        return match ports.observe_child(&lease)? {
            ChildObservation::Running => Ok(ResidentSelfOutcome::Running),
            observation => {
                let receipt =
                    exact_resident_coordinator_receipt(&args.policy.runtime_store, &lease)?;
                if let Some(receipt) = receipt.as_ref() {
                    let outcome = settle_resident_self_exited_coordinator(
                        &args.state_store,
                        &args.policy.runtime_store,
                        &lease,
                        receipt,
                        shutdown_requested,
                        brake_engaged,
                        timed_out,
                        now,
                        args.policy.cooldown_seconds,
                    )?;
                    *state = load_resident_self_state(&args.state_store)?;
                    return Ok(outcome);
                }
                let outcome = settle_receipt_free_dead_coordinator(
                    args,
                    &lease,
                    observation,
                    shutdown_requested,
                    brake_engaged,
                    timed_out,
                    now,
                )?;
                *state = load_resident_self_state(&args.state_store)?;
                Ok(outcome)
            }
        };
    }
    if shutdown_requested || ports.brake_engaged()? {
        return Ok(ResidentSelfOutcome::Braked);
    }
    let Some(prepared) = prepare_resident_self_launch(&args.state_store, &args.policy, now)? else {
        return Ok(ResidentSelfOutcome::Sleeping);
    };
    let launch = CoordinatorLaunch {
        turn_id: resident_prepared_launch_thread_id(&prepared)?,
        wake: epiphany_core::ResidentSelfWake::Explicit {
            objective: prepared.grant.objective.clone(),
        },
        argv: prepared.argv.clone(),
    };
    let process = ports.launch_coordinator(&launch)?;
    let claim_deadline = std::time::Instant::now() + Duration::from_secs(10);
    while resident_self_child_claim(&args.state_store, &prepared.preparation_id)?.is_none() {
        if std::time::Instant::now() >= claim_deadline {
            return Err(anyhow!(
                "coordinator did not claim prepared authority before bootstrap deadline; preparation remains fail-closed"
            ));
        }
        thread::sleep(Duration::from_millis(50));
    }
    acknowledge_resident_self_launch(&args.state_store, &prepared.preparation_id, &process, now)?;
    *state = load_resident_self_state(&args.state_store)?;
    Ok(ResidentSelfOutcome::Launched)
}

fn summary(
    state: &ResidentSelfState,
    outcome: &ResidentSelfOutcome,
    shutdown_requested: bool,
) -> serde_json::Value {
    json!({
        "schemaVersion": "epiphany.resident_self.operator_projection.v0",
        "status": outcome.operator_status(),
        "revision": state.revision,
        "activeTurnId": state.active_turn.as_ref().map(|turn| &turn.turn_id),
        "shutdownRequested": shutdown_requested,
        "nextEligibleAtMillis": state.next_eligible_at_millis,
        "wakeAuthority": "standard heartbeat consumes typed operator, admitted Modeling-map direction consideration, Persona feedback, or Imagination proposal pressure and emits one single-consumption Self grant",
        "preparedRecovery": if state.prepared_launch.is_some() { "fail-closed-awaiting-exact-child-claim-or-witnessed-recovery" } else { "not-required" },
        "authority": "Self may launch one bounded coordinator turn; it cannot directly invoke model/tools, mutate Mind/Hands, review, release, or deploy",
        "privateStateExposed": false
    })
}

#[derive(Clone, Copy)]
enum CommandKind {
    Once,
    Serve,
    Status,
}

struct Args {
    command: CommandKind,
    state_store: PathBuf,
    heartbeat_store: PathBuf,
    provider_freshness_seconds: u64,
    retained_closed_lifecycles: usize,
    retained_coordinator_receipts: usize,
    retained_completed_runtime_sessions: usize,
    persona_feedback_source_store: PathBuf,
    persona_feedback_store: PathBuf,
    bifrost_feedback_trust_anchor: PathBuf,
    feedback_target_repository: String,
    feedback_target_persona: String,
    persona_model_allowed_data_classifications: Vec<String>,
    policy: ResidentSelfPolicy,
    pressure: Option<ResidentSelfPressure>,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut it = env::args().skip(1);
        let command = match it.next().as_deref() {
            Some("once") => CommandKind::Once,
            Some("serve") => CommandKind::Serve,
            Some("status") => CommandKind::Status,
            _ => {
                return Err(anyhow!(
                    "usage: epiphany-swarm <once|serve|status> with exact absolute packaged paths"
                ));
            }
        };
        let mut value = BTreeMap::new();
        let mut objective = None;
        while let Some(flag) = it.next() {
            let raw = it
                .next()
                .ok_or_else(|| anyhow!("missing value for {flag}"))?;
            if flag == "--objective" {
                objective = Some(raw);
            } else {
                value.insert(flag, raw);
            }
        }
        let path = |name: &str| -> Result<PathBuf> {
            Ok(PathBuf::from(
                value.get(name).ok_or_else(|| anyhow!("missing {name}"))?,
            ))
        };
        let u64v = |name: &str, default: u64| -> Result<u64> {
            value.get(name).map_or(Ok(default), |v| {
                v.parse().with_context(|| format!("invalid {name}"))
            })
        };
        let release_store = path("--release-store")?;
        let policy = ResidentSelfPolicy {
            workspace: path("--workspace")?,
            coordinator_bin: release_store.clone(),
            model_runtime_bin: release_store.clone(),
            tool_adapter_bin: release_store.clone(),
            runtime_store: path("--runtime-store")?,
            local_verse_store: path("--local-verse-store")?,
            agent_memory_store: path("--agent-memory-store")?,
            artifact_root: path("--artifact-root")?,
            codex_home: path("--codex-home")?,
            mcp_config: path("--mcp-config")?,
            model_provider: value
                .get("--model-provider")
                .cloned()
                .ok_or_else(|| anyhow!("missing --model-provider"))?,
            max_steps: u64v("--max-steps", 4)?,
            turn_timeout_seconds: u64v("--turn-timeout-seconds", 600)?,
            cooldown_seconds: u64v("--cooldown-seconds", 60)?,
            idle_sleep_seconds: u64v("--idle-sleep-seconds", 15)?,
            failure_backoff_seconds: u64v("--failure-backoff-seconds", 60)?,
            release_commit: String::new(),
            release_manifest_digest: String::new(),
            release_store,
            release_runtime_id: value
                .get("--release-runtime-id")
                .cloned()
                .ok_or_else(|| anyhow!("missing --release-runtime-id"))?,
            release_id: value
                .get("--release-id")
                .cloned()
                .ok_or_else(|| anyhow!("missing --release-id"))?,
            release_witness_sha256: value
                .get("--release-witness-sha256")
                .cloned()
                .ok_or_else(|| anyhow!("missing --release-witness-sha256"))?,
        };
        let pressure = objective.map(|objective| ResidentSelfPressure {
            schema_version: epiphany_core::RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
            pressure_id: value
                .get("--pressure-id")
                .cloned()
                .unwrap_or_else(|| format!("operator-pressure-{}", Utc::now().timestamp_millis())),
            kind: "operator-objective".into(),
            provenance_ref: "cli://epiphany-swarm/operator-objective".into(),
            objective,
            created_at_millis: Utc::now().timestamp_millis().max(0) as u64,
            status: "pending".into(),
            consumed_by_grant_id: None,
            private_state_exposed: false,
        });
        Ok(Self {
            command,
            state_store: path("--state-store")?,
            heartbeat_store: path("--heartbeat-store")?,
            provider_freshness_seconds: u64v("--provider-freshness-seconds", 180)?,
            retained_closed_lifecycles: u64v("--retained-closed-lifecycles", 256)?
                .try_into()
                .context("--retained-closed-lifecycles exceeds platform size")?,
            retained_coordinator_receipts: u64v("--retained-coordinator-receipts", 256)?
                .try_into()
                .context("--retained-coordinator-receipts exceeds platform size")?,
            retained_completed_runtime_sessions: u64v(
                "--retained-completed-runtime-sessions",
                256,
            )?
            .try_into()
            .context("--retained-completed-runtime-sessions exceeds platform size")?,
            persona_feedback_source_store: path("--persona-feedback-source-store")?,
            persona_feedback_store: path("--persona-feedback-store")?,
            bifrost_feedback_trust_anchor: path("--bifrost-feedback-trust-anchor")?,
            feedback_target_repository: value
                .get("--feedback-target-repository")
                .cloned()
                .ok_or_else(|| anyhow!("missing --feedback-target-repository"))?,
            feedback_target_persona: value
                .get("--feedback-target-persona")
                .cloned()
                .ok_or_else(|| anyhow!("missing --feedback-target-persona"))?,
            persona_model_allowed_data_classifications: value
                .get("--persona-model-allowed-data-classifications")
                .ok_or_else(|| anyhow!("missing --persona-model-allowed-data-classifications"))?
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect(),
            policy,
            pressure,
        })
    }
}

struct NativePorts<'a> {
    policy: &'a ResidentSelfPolicy,
    cognitive_runtime_id: String,
    children: BTreeMap<u32, Child>,
}

impl<'a> NativePorts<'a> {
    fn new(policy: &'a ResidentSelfPolicy) -> Result<Self> {
        Ok(Self {
            policy,
            cognitive_runtime_id: resident_cognitive_runtime_id(&policy.runtime_store)?,
            children: BTreeMap::new(),
        })
    }
}

impl ResidentSelfPorts for NativePorts<'_> {
    fn brake_engaged(&mut self) -> Result<bool> {
        resident_self_brake_engaged(
            &self.policy.local_verse_store,
            &self.cognitive_runtime_id,
        )
    }

    fn observe_child(
        &mut self,
        lease: &epiphany_core::ResidentSelfTurnLease,
    ) -> Result<ChildObservation> {
        if let Some(child) = self.children.get_mut(&lease.process_id) {
            return Ok(match child.try_wait()? {
                Some(status) => ChildObservation::Exited(status.code().unwrap_or(-1)),
                None => ChildObservation::Running,
            });
        }
        let expected = ProcessInstanceIdentity {
            process_id: lease.process_id,
            creation_token: lease.process_creation_token,
            created_at_rfc3339: None,
            executable_path: lease.process_executable_path.clone(),
        };
        Ok(match observe_process_instance(&expected) {
            ProcessInstanceObservation::ExactAlive => ChildObservation::Running,
            ProcessInstanceObservation::ExactExited { exit_code } => {
                ChildObservation::Exited(exit_code.map(|v| v as i32).unwrap_or(-1))
            }
            ProcessInstanceObservation::Missing | ProcessInstanceObservation::Replaced { .. } => {
                ChildObservation::Missing
            }
            // Uncertainty cannot prove the exact incarnation dead, so it cannot admit a replacement.
            ProcessInstanceObservation::Inaccessible
            | ProcessInstanceObservation::Indeterminate { .. } => ChildObservation::Running,
        })
    }

    fn request_child_stop(&mut self, lease: &epiphany_core::ResidentSelfTurnLease) -> Result<()> {
        terminate_process_instance(&ProcessInstanceIdentity {
            process_id: lease.process_id,
            creation_token: lease.process_creation_token,
            created_at_rfc3339: None,
            executable_path: lease.process_executable_path.clone(),
        })
    }

    fn launch_coordinator(&mut self, launch: &CoordinatorLaunch) -> Result<LaunchedCoordinator> {
        let mut command = Command::new(&self.policy.coordinator_bin);
        command
            .args(&launch.argv)
            .current_dir(&self.policy.workspace);
        let child = command.spawn().with_context(|| {
            format!("failed to launch {}", self.policy.coordinator_bin.display())
        })?;
        let process_id = child.id();
        let identity = capture_process_instance(process_id)?;
        self.children.insert(process_id, child);
        Ok(LaunchedCoordinator {
            process_id,
            process_creation_token: identity.creation_token,
            process_executable_path: identity.executable_path,
        })
    }

    fn coordinator_receipt_since(
        &mut self,
        turn_id: &str,
        started_at_millis: u64,
    ) -> Result<Option<String>> {
        let mut receipts = coordinator_run_receipts(&self.policy.runtime_store)?;
        receipts.retain(|receipt| {
            receipt.thread_id == turn_id
                && chrono::DateTime::parse_from_rfc3339(&receipt.created_at)
                    .map(|at| at.timestamp_millis().max(0) as u64 >= started_at_millis)
                    .unwrap_or(false)
        });
        receipts.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then(a.receipt_id.cmp(&b.receipt_id))
        });
        Ok(receipts.last().map(|receipt| receipt.receipt_id.clone()))
    }
}

fn resident_self_brake_engaged(
    local_verse_store: &std::path::Path,
    runtime_id: &str,
) -> Result<bool> {
    Ok(
        load_epiphany_cultmesh_swarm_brake(local_verse_store, runtime_id)?
            .is_some_and(|brake| brake.status == "engaged"),
    )
}

#[cfg(test)]
mod brake_tests {
    use super::*;

    #[test]
    fn brake_lookup_uses_the_exact_requested_runtime_namespace() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("verse.cc");
        let mut brake =
            epiphany_core::default_epiphany_cultmesh_swarm_brake("2026-07-18T00:00:00Z");
        brake.status = "engaged".into();
        brake.reason = "test".into();
        epiphany_core::write_epiphany_cultmesh_swarm_brake(&store, "wrong-runtime", brake.clone())?;
        assert!(!resident_self_brake_engaged(&store, "epiphany-yggdrasil")?);
        epiphany_core::write_epiphany_cultmesh_swarm_brake(&store, "epiphany-yggdrasil", brake)?;
        assert!(resident_self_brake_engaged(&store, "epiphany-yggdrasil")?);
        Ok(())
    }

    #[test]
    fn native_resident_uses_mounted_cognitive_runtime_for_braking() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let runtime_store = temp.path().join("runtime.cc");
        let verse_store = temp.path().join("verse.cc");
        epiphany_core::initialize_runtime_spine(
            &runtime_store,
            epiphany_core::RuntimeSpineInitOptions {
                runtime_id: "cognitive-runtime".into(),
                display_name: "Cognitive runtime".into(),
                created_at: "2026-08-10T00:00:00Z".into(),
            },
        )?;
        let mut cognitive_brake =
            epiphany_core::default_epiphany_cultmesh_swarm_brake("2026-08-10T00:00:00Z");
        cognitive_brake.status = "engaged".into();
        cognitive_brake.reason = "cognitive brake".into();
        epiphany_core::write_epiphany_cultmesh_swarm_brake(
            &verse_store,
            "cognitive-runtime",
            cognitive_brake,
        )?;
        let shared = temp.path().to_path_buf();
        let policy = ResidentSelfPolicy {
            workspace: shared.clone(),
            coordinator_bin: shared.join("coordinator"),
            model_runtime_bin: shared.join("model"),
            tool_adapter_bin: shared.join("tool"),
            runtime_store,
            local_verse_store: verse_store,
            agent_memory_store: shared.join("mind.cc"),
            artifact_root: shared.join("artifacts"),
            codex_home: shared.join("codex-home"),
            mcp_config: shared.join("mcp.toml"),
            model_provider: "test".into(),
            max_steps: 1,
            turn_timeout_seconds: 1,
            cooldown_seconds: 1,
            idle_sleep_seconds: 1,
            failure_backoff_seconds: 1,
            release_commit: "release-commit".into(),
            release_manifest_digest: "sha256:release".into(),
            release_store: shared.join("release.cc"),
            release_runtime_id: "deployment-runtime".into(),
            release_id: "sha256:release".into(),
            release_witness_sha256: "sha256:witness".into(),
        };

        let mut ports = NativePorts::new(&policy)?;
        assert_eq!(ports.cognitive_runtime_id, "cognitive-runtime");
        assert!(ports.brake_engaged()?);
        assert!(!resident_self_brake_engaged(
            &policy.local_verse_store,
            &policy.release_runtime_id,
        )?);
        Ok(())
    }

    #[test]
    fn engaged_brake_does_not_touch_the_bifrost_provider_store() -> Result<()> {
        let touched = std::cell::Cell::new(false);
        run_feedback_import_if_released(true, || {
            touched.set(true);
            anyhow::bail!("absent Bifrost delivery store was touched")
        })?;
        assert!(!touched.get());
        assert!(
            run_feedback_import_if_released(false, || {
                anyhow::bail!("released cognition requires Bifrost")
            })
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn shutdown_preempts_other_cancellation_causes() {
        assert_eq!(
            cancelled_turn_status(true, true, true),
            "shutdown-cancelled"
        );
        assert_eq!(cancelled_turn_status(false, true, true), "brake-cancelled");
        assert_eq!(cancelled_turn_status(false, false, true), "timed-out");
        assert_eq!(cancelled_turn_status(false, false, false), "process-failed");
    }

    #[test]
    fn shutdown_cancellation_is_a_braked_outcome_not_a_failure() {
        assert_eq!(outcome_after_cancel(true), ResidentSelfOutcome::Braked);
        assert_eq!(outcome_after_cancel(false), ResidentSelfOutcome::Failed);
    }

    #[test]
    fn terminal_fulfillment_has_explicit_operator_projection() {
        assert_eq!(
            ResidentSelfOutcome::AwaitingFulfillment.operator_status(),
            "awaiting-fulfillment"
        );
    }

    #[test]
    fn shutdown_wait_returns_immediately_when_already_requested() {
        let requested = AtomicBool::new(true);
        let started = std::time::Instant::now();
        wait_for_shutdown(&requested, Duration::from_secs(1));
        assert!(started.elapsed() < Duration::from_millis(100));
    }

    #[test]
    fn shutdown_does_not_admit_new_cycle_ingress() -> Result<()> {
        let touched = std::cell::Cell::new(false);
        run_cycle_ingress_if_running(true, || {
            touched.set(true);
            Ok(())
        })?;
        assert!(!touched.get());
        run_cycle_ingress_if_running(false, || {
            touched.set(true);
            Ok(())
        })?;
        assert!(touched.get());
        Ok(())
    }

    #[test]
    fn runtime_receipt_retention_refuses_active_or_prepared_launch_authority() {
        assert!(runtime_receipt_retention_allowed(false, false));
        assert!(!runtime_receipt_retention_allowed(true, false));
        assert!(!runtime_receipt_retention_allowed(false, true));
        assert!(!runtime_receipt_retention_allowed(true, true));
    }
}
