use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use epiphany_core::{
    ChildObservation, CoordinatorLaunch, LaunchedCoordinator, ProcessInstanceIdentity,
    ProcessInstanceObservation, ResidentProviderReadiness, ResidentReadinessRequest,
    ResidentSelfOutcome, ResidentSelfPolicy, ResidentSelfPorts, ResidentSelfPressure,
    ResidentSelfState, acknowledge_resident_self_launch, authenticate_resident_self_policy,
    bind_runtime_repository_domain, cancel_resident_self_turn, capture_process_instance,
    complete_resident_self_turn, coordinator_run_receipts, derive_resident_cognition_readiness,
    enqueue_resident_self_pressure, import_bifrost_persona_feedback_deliveries,
    ingest_resident_self_domain_pressure, load_epiphany_cultmesh_swarm_brake,
    load_resident_self_state, observe_process_instance, prepare_resident_self_launch,
    publish_resident_provider_readiness, resident_self_child_claim,
    resident_self_local_provider_status, terminate_process_instance,
    validate_bifrost_persona_feedback_source, validate_persona_feedback_store_separation,
    validate_resident_self_store_separation,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;
use std::process::{Child, Command};
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
    // Status is non-actuating sight. In particular it must not open the
    // provider-owned delivery store: CultCache read locking can create or
    // rewrite the sibling lock file, which would let a root-run verifier
    // poison the service identity's later access. The resident importer owns
    // source authentication immediately before it can consume deliveries.
    validate_feedback_source_for_command(&args.command, || {
        validate_bifrost_persona_feedback_source(
            &args.persona_feedback_source_store,
            &args.bifrost_feedback_trust_anchor,
            &args.policy.release_runtime_id,
            &args.feedback_target_repository,
            &args.feedback_target_persona,
        )
        .map(|_| ())
    })?;
    if let Some(pressure) = args.pressure.as_ref() {
        enqueue_resident_self_pressure(&args.state_store, pressure)?;
    }
    let mut state = load_resident_self_state(&args.state_store)?;
    let mut ports = NativePorts::new(&args.policy);
    match args.command {
        CommandKind::Once => {
            let outcome = cycle(&args, &mut state, &mut ports)?;
            publish_self_readiness(&args)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&summary(&state, &outcome))?
            );
        }
        CommandKind::Serve => loop {
            let outcome = cycle(&args, &mut state, &mut ports)?;
            publish_self_readiness(&args)?;
            println!("{}", serde_json::to_string(&summary(&state, &outcome))?);
            let seconds = match outcome {
                ResidentSelfOutcome::Failed => args.policy.failure_backoff_seconds,
                ResidentSelfOutcome::Completed => args.policy.cooldown_seconds,
                _ => args.policy.idle_sleep_seconds,
            };
            thread::sleep(Duration::from_secs(seconds.max(1)));
        },
        CommandKind::Status => unreachable!("status returned before actuation setup"),
    }
    Ok(())
}

fn validate_feedback_source_for_command(
    command: &CommandKind,
    validate: impl FnOnce() -> Result<()>,
) -> Result<()> {
    if matches!(command, CommandKind::Status) {
        return Ok(());
    }
    validate()
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

fn cycle(
    args: &Args,
    state: &mut ResidentSelfState,
    ports: &mut NativePorts,
) -> Result<ResidentSelfOutcome> {
    let now = Utc::now().timestamp_millis().max(0) as u64;
    import_bifrost_persona_feedback_deliveries(
        &args.persona_feedback_source_store,
        &args.persona_feedback_store,
        &args.bifrost_feedback_trust_anchor,
        &args.policy.release_runtime_id,
        &args.feedback_target_repository,
        &args.feedback_target_persona,
    )?;
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
        if ports.brake_engaged()? {
            return Ok(ResidentSelfOutcome::Braked);
        }
        let launch = CoordinatorLaunch {
            turn_id: format!("resident-self-turn-{}", prepared.grant.grant_id),
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
        if brake_engaged || timed_out {
            if ports.observe_child(&lease)? == ChildObservation::Running {
                ports.request_child_stop(&lease)?;
                return Ok(ResidentSelfOutcome::Draining);
            }
        }
        return match ports.observe_child(&lease)? {
            ChildObservation::Running => Ok(ResidentSelfOutcome::Running),
            ChildObservation::Exited(0) => {
                let receipt = coordinator_run_receipts(&args.policy.runtime_store)?
                    .into_iter()
                    .find(|receipt| {
                        receipt.thread_id == lease.turn_id
                            && receipt.resident_launch_digest.as_deref()
                                == Some(&lease.launch_digest)
                    })
                    .ok_or_else(|| {
                        anyhow!(
                            "coordinator exited zero without its exact resident terminal receipt"
                        )
                    })?;
                complete_resident_self_turn(&args.state_store, &lease, &receipt, now)?;
                *state = load_resident_self_state(&args.state_store)?;
                Ok(ResidentSelfOutcome::Completed)
            }
            ChildObservation::Exited(code) => {
                cancel_resident_self_turn(
                    &args.state_store,
                    &lease,
                    if brake_engaged {
                        "brake-cancelled"
                    } else if timed_out {
                        "timed-out"
                    } else {
                        "process-failed"
                    },
                    &format!("coordinator terminal observation exit={code}"),
                    now,
                )?;
                *state = load_resident_self_state(&args.state_store)?;
                Ok(ResidentSelfOutcome::Failed)
            }
            ChildObservation::Missing => {
                cancel_resident_self_turn(
                    &args.state_store,
                    &lease,
                    if brake_engaged {
                        "brake-cancelled"
                    } else if timed_out {
                        "timed-out"
                    } else {
                        "process-failed"
                    },
                    "exact coordinator process is missing",
                    now,
                )?;
                *state = load_resident_self_state(&args.state_store)?;
                Ok(ResidentSelfOutcome::Failed)
            }
        };
    }
    if ports.brake_engaged()? {
        return Ok(ResidentSelfOutcome::Braked);
    }
    let Some(prepared) = prepare_resident_self_launch(&args.state_store, &args.policy, now)? else {
        return Ok(ResidentSelfOutcome::Sleeping);
    };
    let launch = CoordinatorLaunch {
        turn_id: format!("resident-self-turn-{}", prepared.grant.grant_id),
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

fn summary(state: &ResidentSelfState, outcome: &ResidentSelfOutcome) -> serde_json::Value {
    json!({
        "schemaVersion": "epiphany.resident_self.operator_projection.v0",
        "status": format!("{outcome:?}").to_ascii_lowercase(),
        "revision": state.revision,
        "activeTurnId": state.active_turn.as_ref().map(|turn| &turn.turn_id),
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
    persona_feedback_source_store: PathBuf,
    persona_feedback_store: PathBuf,
    bifrost_feedback_trust_anchor: PathBuf,
    feedback_target_repository: String,
    feedback_target_persona: String,
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
            policy,
            pressure,
        })
    }
}

struct NativePorts<'a> {
    policy: &'a ResidentSelfPolicy,
    children: BTreeMap<u32, Child>,
}

impl<'a> NativePorts<'a> {
    fn new(policy: &'a ResidentSelfPolicy) -> Self {
        Self {
            policy,
            children: BTreeMap::new(),
        }
    }
}

impl ResidentSelfPorts for NativePorts<'_> {
    fn brake_engaged(&mut self) -> Result<bool> {
        resident_self_brake_engaged(
            &self.policy.local_verse_store,
            &self.policy.release_runtime_id,
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
        if let Some(child) = self.children.get_mut(&lease.process_id) {
            return child.kill().with_context(|| {
                format!(
                    "failed to stop exact coordinator process {}",
                    lease.process_id
                )
            });
        }
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
    fn resident_self_uses_exact_release_runtime_brake_namespace() -> Result<()> {
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
    fn status_never_opens_or_validates_the_bifrost_provider_store() -> Result<()> {
        let touched = std::cell::Cell::new(false);
        validate_feedback_source_for_command(&CommandKind::Status, || {
            touched.set(true);
            anyhow::bail!("poison Bifrost store was touched")
        })?;
        assert!(!touched.get());
        assert!(
            validate_feedback_source_for_command(&CommandKind::Once, || {
                anyhow::bail!("non-status path authenticates provider source")
            })
            .is_err()
        );
        Ok(())
    }
}
