use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use epiphany_core::EpiphanyCoordinatorRoleResultStatus;
use epiphany_core::EpiphanyCoordinatorStatusInput;
use epiphany_core::EpiphanyCrrcInput;
use epiphany_core::EpiphanyCrrcReorientAction;
use epiphany_core::EpiphanyCrrcResultStatus;
use epiphany_core::EpiphanyCrrcStateStatus;
use epiphany_core::EpiphanyFreshnessInput;
use epiphany_core::EpiphanyGraphFreshnessStatus;
use epiphany_core::EpiphanyInvalidationStatus;
use epiphany_core::EpiphanyJobStatus;
use epiphany_core::EpiphanyJobsInput;
use epiphany_core::EpiphanyReorientAction;
use epiphany_core::EpiphanyReorientFreshnessStatus;
use epiphany_core::EpiphanyReorientInput;
use epiphany_core::EpiphanyReorientPressureLevel;
use epiphany_core::EpiphanyRetrievalFreshnessStatus;
use epiphany_core::EpiphanyRoleBoardCheckpointSummary;
use epiphany_core::EpiphanyRoleBoardInput;
use epiphany_core::EpiphanyRoleBoardJob;
use epiphany_core::EpiphanyRoleBoardJobStatus;
use epiphany_core::EpiphanyRoleBoardPlanningSummary;
use epiphany_core::EpiphanyRoleFindingInterpretation;
use epiphany_core::EpiphanyRoleResultRoleId;
use epiphany_core::EpiphanyRuntimeJobSnapshot;
use epiphany_core::EpiphanyRuntimeJobStatus;
use epiphany_core::EpiphanySceneInput;
use epiphany_core::EpiphanyTokenUsageSnapshot;
use epiphany_core::derive_coordinator_finding_signals;
use epiphany_core::derive_coordinator_status;
use epiphany_core::derive_freshness;
use epiphany_core::derive_jobs;
use epiphany_core::derive_planning_view;
use epiphany_core::derive_pressure_view;
use epiphany_core::derive_role_board;
use epiphany_core::derive_scene;
use epiphany_core::interpret_runtime_role_worker_result;
use epiphany_core::read_accepted_coordinator_state;
use epiphany_core::recommend_crrc_action;
use epiphany_core::recommend_reorientation;
use epiphany_core::runtime_job_snapshot;
use epiphany_core::runtime_role_worker_result;
use epiphany_state_model::EpiphanyThreadState;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_CARGO_TARGET_DIR: &str = r"C:\Users\Meta\.cargo-target-codex";
const DEFAULT_COORDINATOR_STORE: &str = "state/runtime-spine.msgpack";
const REORIENT_BINDING_ID: &str = "reorient-worker";
const IMAGINATION_BINDING_ID: &str = "imagination-synthesis-worker";
const RESEARCH_BINDING_ID: &str = "research-source-gather-worker";
const MODELING_BINDING_ID: &str = "modeling-checkpoint-worker";
const VERIFICATION_BINDING_ID: &str = "verification-review-worker";
const SEALED_DIRECT_THOUGHT_KEYS: &[&str] = &[
    "rawResult",
    "turns",
    "items",
    "inputTranscript",
    "activeTranscript",
];
const SEALED_LONG_TEXT_KEYS: &[&str] = &["note"];
const MAX_OPERATOR_TEXT_CHARS: usize = 1200;

fn main() -> Result<()> {
    let args = Args::parse()?;
    let status = run_status(&args)?;
    if let Some(result) = &args.result {
        if let Some(parent) = result.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(
            result,
            format!("{}\n", serde_json::to_string_pretty(&status)?),
        )
        .with_context(|| format!("failed to write {}", result.display()))?;
    }
    if args.json {
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        print!("{}", render_status(&status));
    }
    Ok(())
}

#[derive(Debug)]
struct Args {
    thread_id: Option<String>,
    cwd: PathBuf,
    store: PathBuf,
    ephemeral: bool,
    json: bool,
    result: Option<PathBuf>,
    transcript: PathBuf,
    stderr: PathBuf,
    interrupt_binding: Option<String>,
    interrupt_reason: Option<String>,
}

impl Args {
    fn parse() -> Result<Self> {
        let root = env::current_dir().context("failed to resolve current directory")?;
        let mut args = env::args().skip(1);
        let mut parsed = Args {
            thread_id: None,
            cwd: root.clone(),
            store: PathBuf::from(DEFAULT_COORDINATOR_STORE),
            ephemeral: true,
            json: false,
            result: None,
            transcript: root
                .join(".epiphany-status")
                .join("epiphany-mvp-status-transcript.jsonl"),
            stderr: root
                .join(".epiphany-status")
                .join("epiphany-mvp-status-server.stderr.log"),
            interrupt_binding: None,
            interrupt_reason: None,
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--thread-id" => parsed.thread_id = Some(take_string(&mut args, "--thread-id")?),
                "--cwd" => parsed.cwd = take_path(&mut args, "--cwd")?,
                "--store" => parsed.store = take_path(&mut args, "--store")?,
                "--ephemeral" => parsed.ephemeral = true,
                "--no-ephemeral" => parsed.ephemeral = false,
                "--json" => parsed.json = true,
                "--result" => parsed.result = Some(take_path(&mut args, "--result")?),
                "--transcript" => parsed.transcript = take_path(&mut args, "--transcript")?,
                "--stderr" => parsed.stderr = take_path(&mut args, "--stderr")?,
                "--interrupt-binding" => {
                    parsed.interrupt_binding = Some(take_string(&mut args, "--interrupt-binding")?)
                }
                "--interrupt-reason" => {
                    parsed.interrupt_reason = Some(take_string(&mut args, "--interrupt-reason")?)
                }
                _ => return Err(anyhow!("unknown argument: {arg}")),
            }
        }
        Ok(parsed)
    }
}

fn run_status(args: &Args) -> Result<Value> {
    if args.interrupt_binding.is_some() {
        return run_native_interrupt(args);
    }
    run_native_status(args)
}

fn run_native_interrupt(args: &Args) -> Result<Value> {
    let thread_id = args
        .thread_id
        .as_ref()
        .ok_or_else(|| anyhow!("--interrupt-binding requires --thread-id"))?;
    let binding_id = args
        .interrupt_binding
        .as_ref()
        .ok_or_else(|| anyhow!("--interrupt-binding requires a binding id"))?;
    let store = absolute_path(&args.store)?;
    let service = epiphany_core::EpiphanyCoordinatorService::new(&store);
    let state = service
        .state()?
        .ok_or_else(|| anyhow!("cannot interrupt without native coordinator state"))?;
    let result = service.interrupt_job(
        thread_id,
        &state,
        epiphany_core::EpiphanyJobInterruptRequest {
            expected_revision: Some(state.revision),
            binding_id: binding_id.clone(),
            reason: args.interrupt_reason.clone(),
        },
    )?;
    Ok(json!({
        "source": "native",
        "threadId": thread_id,
        "bindingId": result.binding_id,
        "revision": result.epiphany_state.revision,
        "cancelRequested": result.cancel_requested,
        "interruptedThreadIds": result.interrupted_thread_ids,
        "state": result.epiphany_state,
    }))
}

fn run_native_status(args: &Args) -> Result<Value> {
    let cwd = absolute_path(&args.cwd)?;
    let store_path = absolute_path(&args.store)?;
    let runtime_store_path = store_path.clone();
    let state = if store_path.exists() {
        read_accepted_coordinator_state(&store_path)
            .with_context(|| format!("failed to load {}", store_path.display()))?
    } else {
        None
    };
    let thread_id = args
        .thread_id
        .clone()
        .unwrap_or_else(|| "native-local".to_string());
    let state_ref = state.as_ref();
    let loaded = state_ref.is_some();

    let scene = derive_scene(EpiphanySceneInput {
        state: state_ref,
        loaded,
        reorient_binding_id: REORIENT_BINDING_ID,
    });
    let pressure = derive_pressure_view(None::<&EpiphanyTokenUsageSnapshot>);
    let freshness = derive_freshness(EpiphanyFreshnessInput {
        state: state_ref,
        retrieval_override: None,
        watcher: None,
    });
    let reorient_pressure_level = match pressure.level {
        epiphany_core::EpiphanyPressureLevel::Unknown => EpiphanyReorientPressureLevel::Unknown,
        epiphany_core::EpiphanyPressureLevel::Low => EpiphanyReorientPressureLevel::Low,
        epiphany_core::EpiphanyPressureLevel::Elevated => EpiphanyReorientPressureLevel::Medium,
        epiphany_core::EpiphanyPressureLevel::High => EpiphanyReorientPressureLevel::High,
        epiphany_core::EpiphanyPressureLevel::Critical => EpiphanyReorientPressureLevel::Critical,
    };
    let (reorient_state_status, reorient_decision) =
        recommend_reorientation(EpiphanyReorientInput {
            checkpoint: state_ref.and_then(|state| state.investigation_checkpoint.as_ref()),
            state_present: state_ref.is_some(),
            pressure_level: reorient_pressure_level,
            retrieval_status: reorient_retrieval_status(freshness.retrieval.status),
            retrieval_dirty_paths: freshness.retrieval.dirty_paths.clone(),
            graph_status: reorient_graph_status(freshness.graph.status),
            graph_dirty_paths: freshness.graph.dirty_paths.clone(),
            watcher_status: reorient_watcher_status(freshness.watcher.status),
            watcher_changed_paths: freshness.watcher.changed_paths.clone(),
            watcher_graph_node_ids: freshness.watcher.graph_node_ids.clone(),
            active_frontier_node_ids: freshness.watcher.active_frontier_node_ids.clone(),
            watched_root: freshness.watcher.watched_root.clone().or(Some(cwd.clone())),
        });
    let planning = derive_planning_view(state_ref);
    let mut jobs = derive_jobs(EpiphanyJobsInput {
        state: state_ref,
        retrieval_override: None,
    });
    reconcile_native_runtime_jobs(&mut jobs, state_ref, &runtime_store_path);
    let state_status = if state_ref.is_some() {
        EpiphanyCrrcStateStatus::Ready
    } else {
        EpiphanyCrrcStateStatus::Missing
    };
    let reorient_action = match reorient_decision.action {
        EpiphanyReorientAction::Resume => EpiphanyCrrcReorientAction::Resume,
        EpiphanyReorientAction::Regather => EpiphanyCrrcReorientAction::Regather,
    };
    let result_status =
        native_reorient_result_status(state_ref, &runtime_store_path, REORIENT_BINDING_ID);
    let modeling_result_status =
        native_role_result_status(state_ref, &runtime_store_path, MODELING_BINDING_ID);
    let research_result_status =
        native_role_result_status(state_ref, &runtime_store_path, RESEARCH_BINDING_ID);
    let verification_result_status =
        native_role_result_status(state_ref, &runtime_store_path, VERIFICATION_BINDING_ID);
    let recommendation = recommend_crrc_action(EpiphanyCrrcInput {
        loaded,
        state_status,
        should_prepare_compaction: pressure.should_prepare_compaction,
        reorient_action,
        result_status,
        checkpoint_present: state_ref
            .and_then(|state| state.investigation_checkpoint.as_ref())
            .is_some(),
        finding_present: false,
        finding_accepted: false,
    });
    let role_jobs = jobs
        .iter()
        .map(|job| EpiphanyRoleBoardJob {
            id: job.id.clone(),
            owner_role: job.owner_role.clone(),
            status: role_job_status(job.status),
            progress_note: job.progress_note.clone(),
            blocking_reason: job.blocking_reason.clone(),
        })
        .collect::<Vec<_>>();
    let roles = derive_role_board(EpiphanyRoleBoardInput {
        state_present: state_ref.is_some(),
        planning: EpiphanyRoleBoardPlanningSummary {
            capture_count: planning.summary.capture_count as usize,
            backlog_item_count: planning.summary.backlog_item_count as usize,
            roadmap_stream_count: planning.summary.roadmap_stream_count as usize,
            objective_draft_count: planning.summary.objective_draft_count as usize,
        },
        checkpoint: state_ref.and_then(|state| {
            state.investigation_checkpoint.as_ref().map(|checkpoint| {
                EpiphanyRoleBoardCheckpointSummary {
                    disposition: Some(format!("{:?}", checkpoint.disposition)),
                    next_action: checkpoint.next_action.clone(),
                }
            })
        }),
        reorient_next_action: reorient_decision.next_action.clone(),
        jobs: role_jobs.clone(),
        crrc_action: recommendation.action,
        crrc_recommended_scene_action: recommendation
            .recommended_scene_action
            .map(epiphany_core::crrc_scene_action_to_coordinator_scene_action),
        crrc_reason: recommendation.reason.clone(),
        reorient_decision_action: format!("{:?}", reorient_decision.action),
        pressure_level: format!("{:?}", pressure.level),
        reorient_result_status: result_status,
        reorient_job: role_jobs
            .iter()
            .find(|job| job.id == REORIENT_BINDING_ID)
            .cloned(),
        imagination_binding_id: IMAGINATION_BINDING_ID.to_string(),
        research_binding_id: RESEARCH_BINDING_ID.to_string(),
        modeling_binding_id: MODELING_BINDING_ID.to_string(),
        verification_binding_id: VERIFICATION_BINDING_ID.to_string(),
        reorient_owner_role: "epiphany-reorienter".to_string(),
        imagination_owner_role: "epiphany-imagination".to_string(),
        research_owner_role: "epiphany-eyes".to_string(),
    });
    let research_finding = native_role_finding(
        state_ref,
        &runtime_store_path,
        RESEARCH_BINDING_ID,
        EpiphanyRoleResultRoleId::Research,
    );
    let modeling_finding = native_role_finding(
        state_ref,
        &runtime_store_path,
        MODELING_BINDING_ID,
        EpiphanyRoleResultRoleId::Modeling,
    );
    let verification_finding = native_role_finding(
        state_ref,
        &runtime_store_path,
        VERIFICATION_BINDING_ID,
        EpiphanyRoleResultRoleId::Verification,
    );
    let finding_signals = derive_coordinator_finding_signals(
        state_ref,
        research_finding.as_ref(),
        modeling_finding.as_ref(),
        verification_finding.as_ref(),
        None,
    );
    let coordinator = derive_coordinator_status(EpiphanyCoordinatorStatusInput {
        state_status,
        checkpoint_present: state_ref
            .and_then(|state| state.investigation_checkpoint.as_ref())
            .is_some(),
        pressure: pressure.clone(),
        recommendation: recommendation.clone(),
        roles: roles.clone(),
        reorient_action: reorient_decision.action,
        research_result_status,
        modeling_result_status,
        verification_result_status,
        reorient_result_status: result_status,
        research_result_accepted: finding_signals.research_result_accepted,
        research_result_reviewable: finding_signals.research_result_reviewable,
        modeling_result_requests_regather: finding_signals.modeling_result_requests_regather,
        modeling_result_accepted: finding_signals.modeling_result_accepted,
        modeling_result_reviewable: finding_signals.modeling_result_reviewable,
        modeling_result_failure_reviewed: finding_signals.modeling_result_failure_reviewed,
        modeling_result_accepted_after_verification: finding_signals
            .modeling_result_accepted_after_verification,
        implementation_evidence_after_verification: finding_signals
            .implementation_evidence_after_verification,
        verification_result_cites_implementation_evidence: finding_signals
            .verification_result_cites_implementation_evidence,
        verification_result_covers_current_modeling: finding_signals
            .verification_result_covers_current_modeling,
        verification_result_accepted: finding_signals.verification_result_accepted,
        verification_result_failure_reviewed: finding_signals.verification_result_failure_reviewed,
        verification_result_allows_implementation: finding_signals
            .verification_result_allows_implementation,
        verification_result_needs_evidence: finding_signals.verification_result_needs_evidence,
        reorient_finding_accepted: finding_signals.reorient_finding_accepted,
    });
    let coordinator_json = coordinator_status_json(&coordinator)?;
    let tool_invocations = native_tool_invocation_surface(&runtime_store_path)?;

    let root = env::current_dir().context("failed to resolve current directory")?;
    let native_aux = native_auxiliary_status(&root)?;
    let status = json!({
        "threadId": thread_id,
        "read": {
            "source": "native",
            "threadStateStore": store_path,
            "statePresent": state_ref.is_some(),
        },
        "view": {
            "source": "native",
            "scene": scene,
            "pressure": pressure,
            "jobs": jobs,
            "roles": {
                "threadId": thread_id,
                "source": "native",
                "stateStatus": crrc_state_status_text(state_status),
                "roles": roles,
            },
            "planning": planning,
            "reorient": {
                "threadId": thread_id,
                "source": "native",
                "stateStatus": reorient_state_status,
                "decision": reorient_decision,
            },
            "crrc": {
                "threadId": thread_id,
                "source": "native",
                "recommendation": recommendation,
            },
            "coordinator": coordinator_json,
            "tools": tool_invocations.clone(),
        },
        "scene": {
            "threadId": thread_id,
            "scene": scene,
        },
        "pressure": {
            "threadId": thread_id,
            "source": "native",
            "pressure": pressure,
        },
        "reorient": {
            "threadId": thread_id,
            "source": "native",
            "stateStatus": reorient_state_status,
            "decision": reorient_decision,
        },
        "jobs": {
            "threadId": thread_id,
            "source": "native",
            "jobs": jobs,
        },
        "roles": {
            "threadId": thread_id,
            "source": "native",
            "stateStatus": crrc_state_status_text(state_status),
            "roles": roles,
        },
        "planning": planning,
        "roleResults": {
            "imagination": native_role_result(state_ref, &runtime_store_path, IMAGINATION_BINDING_ID),
            "research": native_role_result(state_ref, &runtime_store_path, RESEARCH_BINDING_ID),
            "modeling": native_role_result(state_ref, &runtime_store_path, MODELING_BINDING_ID),
            "verification": native_role_result(state_ref, &runtime_store_path, VERIFICATION_BINDING_ID),
        },
        "reorientResult": native_reorient_result(state_ref, &runtime_store_path, REORIENT_BINDING_ID),
        "crrc": {
            "threadId": thread_id,
            "source": "native",
            "recommendation": recommendation,
        },
        "coordinator": coordinator_json,
        "tools": tool_invocations,
        "heartbeat": native_aux.heartbeat,
        "persona": native_aux.persona,
        "bifrostBridge": native_aux.bifrost_bridge,
        "voidMemory": native_aux.void_memory,
    });
    Ok(sanitize_for_operator(status))
}

struct NativeAuxiliaryStatus {
    heartbeat: Value,
    persona: Value,
    bifrost_bridge: Value,
    void_memory: Value,
}

fn native_auxiliary_status(root: &Path) -> Result<NativeAuxiliaryStatus> {
    let heartbeat_dir = root.join(".epiphany-heartbeats");
    let persona_dir = root.join(".epiphany-persona");
    let heartbeat = native_json(
        "epiphany-heartbeat-store",
        &[
            "status",
            "--store",
            "state/agent-heartbeats.msgpack",
            "--artifact-dir",
            &heartbeat_dir.to_string_lossy(),
            "--limit",
            "8",
        ],
    )?;
    let latest_discord_persona = native_json(
        "epiphany-persona-discord",
        &[
            "latest",
            "--artifact-dir",
            &persona_dir.to_string_lossy(),
            "--limit",
            "8",
        ],
    )
    .unwrap_or_else(
        |error| json!({"status": "error", "error": error.to_string(), "latestArtifacts": []}),
    );
    let latest_other_persona = native_json(
        "epiphany-persona-other",
        &[
            "latest",
            "--artifact-dir",
            &persona_dir.to_string_lossy(),
            "--limit",
            "8",
        ],
    )
    .unwrap_or_else(
        |error| json!({"status": "error", "error": error.to_string(), "latestArtifacts": []}),
    );
    let latest_reddit_persona = native_json(
        "epiphany-persona-reddit",
        &[
            "latest",
            "--artifact-dir",
            &persona_dir.to_string_lossy(),
            "--limit",
            "8",
        ],
    )
    .unwrap_or_else(
        |error| json!({"status": "error", "error": error.to_string(), "latestArtifacts": []}),
    );
    let mut latest_artifacts = latest_discord_persona
        .get("latestArtifacts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    latest_artifacts.extend(
        latest_reddit_persona
            .get("latestArtifacts")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
    );
    latest_artifacts.extend(
        latest_other_persona
            .get("latestArtifacts")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
    );
    let persona = json!({
        "status": "ready",
        "artifactDir": persona_dir,
        "latestArtifacts": latest_artifacts,
        "availableActions": ["personaBubble", "characterTurn", "discordPersonaPost", "redditPersonaPost", "otherWorldPersonaRequest"],
    });
    let bifrost_bridge = bifrost_bridge_readiness(root);
    let void_memory = native_json(
        "epiphany-void-memory",
        &["status", "--config", "state/void-memory.toml"],
    )
    .unwrap_or_else(|error| json!({"ok": false, "error": error.to_string()}));

    Ok(NativeAuxiliaryStatus {
        heartbeat,
        persona,
        bifrost_bridge,
        void_memory,
    })
}

fn bifrost_bridge_readiness(root: &Path) -> Value {
    let bifrost_root = root
        .parent()
        .map(|parent| parent.join("Bifrost"))
        .unwrap_or_else(|| root.join("../Bifrost"));
    let advertisement_tool = bifrost_root
        .join("tools")
        .join("provider-advertisement.mjs");
    if !advertisement_tool.exists() {
        return json!({
            "status": "unavailable",
            "owner": "Bifrost",
            "tool": advertisement_tool,
            "note": "Bifrost provider advertisement tool was not found; Epiphany cannot infer outside-world bridge readiness locally.",
            "privateStateExposed": false,
        });
    }

    let output = Command::new("node")
        .arg(&advertisement_tool)
        .arg("print-surface")
        .current_dir(&bifrost_root)
        .output();
    let Ok(output) = output else {
        return json!({
            "status": "unavailable",
            "owner": "Bifrost",
            "tool": advertisement_tool,
            "note": "Failed to invoke Bifrost provider advertisement tool.",
            "privateStateExposed": false,
        });
    };
    if !output.status.success() {
        return json!({
            "status": "unavailable",
            "owner": "Bifrost",
            "tool": advertisement_tool,
            "note": format!("Bifrost provider advertisement failed: {}", String::from_utf8_lossy(&output.stderr)),
            "privateStateExposed": false,
        });
    }

    let parsed: Value = match serde_json::from_slice(&output.stdout) {
        Ok(value) => value,
        Err(error) => {
            return json!({
                "status": "unavailable",
                "owner": "Bifrost",
                "tool": advertisement_tool,
                "note": format!("Bifrost provider advertisement returned invalid JSON: {error}"),
                "privateStateExposed": false,
            });
        }
    };
    let bridge = &parsed["stats"]["bridge"];
    let surfaces = bridge["surfaces"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|surface| {
            let ready = surface["ready"].as_bool().unwrap_or(false);
            let prepared = surface["prepared"].as_bool().unwrap_or(false);
            json!({
                "id": surface["id"].clone(),
                "label": surface["label"].clone(),
                "status": if ready { "live" } else if prepared { "prepared" } else { "missing" },
                "ready": ready,
                "prepared": prepared,
                "authority": surface["authority"].clone(),
                "credentialSource": surface["credentialSource"].clone(),
                "note": surface["note"].clone(),
            })
        })
        .collect::<Vec<_>>();
    let ready_count = surfaces
        .iter()
        .filter(|surface| surface["ready"].as_bool().unwrap_or(false))
        .count();
    let prepared_count = surfaces
        .iter()
        .filter(|surface| {
            surface["prepared"].as_bool().unwrap_or(false)
                && !surface["ready"].as_bool().unwrap_or(false)
        })
        .count();
    json!({
        "status": if bridge["ready"].as_bool().unwrap_or(false) { "live" } else if bridge["prepared"].as_bool().unwrap_or(false) { "prepared" } else { "unavailable" },
        "owner": "Bifrost",
        "authority": "Bifrost owns outside-world crossing gates, bridge receipts, and readiness projection; Heimdall owns provider OAuth, account links, and capability truth; Epiphany consumes this as read-only bridge sight.",
        "source": "Bifrost provider advertisement print-surface",
        "tool": advertisement_tool,
        "generatedAt": parsed["stats"]["generatedAt"].clone(),
        "ready": bridge["ready"].clone(),
        "prepared": bridge["prepared"].clone(),
        "readySurfaceCount": ready_count,
        "preparedSurfaceCount": prepared_count,
        "surfaceCount": surfaces.len(),
        "surfaces": surfaces,
        "privateStateExposed": false,
    })
}

fn native_role_result(
    state: Option<&EpiphanyThreadState>,
    runtime_store: &Path,
    binding_id: &str,
) -> Value {
    let Some(job_id) = latest_runtime_job_id_for_binding(state, binding_id) else {
        return json!({
            "source": "native",
            "status": "backendMissing",
            "bindingId": binding_id,
            "note": "No runtime-spine job is linked to this native role binding.",
        });
    };
    match runtime_job_snapshot(runtime_store, job_id) {
        Ok(Some(snapshot)) => {
            let status = map_runtime_role_result_status(&snapshot);
            let mut result = json!({
                "source": "native",
                "status": status,
                "bindingId": binding_id,
                "runtimeJobId": job_id,
                "note": native_role_result_note(status, &snapshot),
            });
            if matches!(
                status,
                EpiphanyCoordinatorRoleResultStatus::Failed
                    | EpiphanyCoordinatorRoleResultStatus::Cancelled
            ) && let Some(receipt) = snapshot.result
            {
                result["finding"] = json!({
                    "verdict": receipt.verdict,
                    "summary": receipt.summary,
                    "nextSafeMove": empty_string_as_null(&receipt.next_safe_move),
                    "runtimeResultId": receipt.result_id,
                    "runtimeJobId": receipt.job_id,
                    "evidenceIds": receipt.evidence_refs,
                    "artifactRefs": receipt.artifact_refs,
                    "jobError": receipt.summary,
                });
            }
            result
        }
        Ok(None) => json!({
            "source": "native",
            "status": "backendMissing",
            "bindingId": binding_id,
            "runtimeJobId": job_id,
            "note": "The linked runtime-spine job is missing.",
        }),
        Err(error) => json!({
            "source": "native",
            "status": "backendUnavailable",
            "bindingId": binding_id,
            "runtimeJobId": job_id,
            "note": format!("Failed to read linked runtime-spine job: {error}"),
        }),
    }
}

fn native_reorient_result(
    state: Option<&EpiphanyThreadState>,
    runtime_store: &Path,
    binding_id: &str,
) -> Value {
    let Some(job_id) = latest_runtime_job_id_for_binding(state, binding_id) else {
        return json!({
            "source": "native",
            "status": "backendMissing",
            "bindingId": binding_id,
            "note": "No runtime-spine job is linked to this native reorientation binding.",
        });
    };
    match runtime_job_snapshot(runtime_store, job_id) {
        Ok(Some(snapshot)) => {
            let status = map_runtime_reorient_result_status(&snapshot);
            let mut result = json!({
                "source": "native",
                "status": status,
                "bindingId": binding_id,
                "runtimeJobId": job_id,
                "note": native_reorient_result_note(status, &snapshot),
            });
            if matches!(
                status,
                EpiphanyCrrcResultStatus::Failed | EpiphanyCrrcResultStatus::Cancelled
            ) && let Some(receipt) = snapshot.result
            {
                result["finding"] = json!({
                    "summary": receipt.summary,
                    "nextSafeMove": empty_string_as_null(&receipt.next_safe_move),
                    "runtimeResultId": receipt.result_id,
                    "runtimeJobId": receipt.job_id,
                    "evidenceIds": receipt.evidence_refs,
                    "artifactRefs": receipt.artifact_refs,
                    "jobError": receipt.summary,
                });
            }
            result
        }
        Ok(None) => json!({
            "source": "native",
            "status": "backendMissing",
            "bindingId": binding_id,
            "runtimeJobId": job_id,
            "note": "The linked runtime-spine job is missing.",
        }),
        Err(error) => json!({
            "source": "native",
            "status": "backendUnavailable",
            "bindingId": binding_id,
            "runtimeJobId": job_id,
            "note": format!("Failed to read linked runtime-spine job: {error}"),
        }),
    }
}

fn native_role_result_status(
    state: Option<&EpiphanyThreadState>,
    runtime_store: &Path,
    binding_id: &str,
) -> EpiphanyCoordinatorRoleResultStatus {
    let Some(job_id) = latest_runtime_job_id_for_binding(state, binding_id) else {
        return EpiphanyCoordinatorRoleResultStatus::BackendMissing;
    };
    match runtime_job_snapshot(runtime_store, job_id) {
        Ok(Some(snapshot)) => map_runtime_role_result_status(&snapshot),
        Ok(None) => EpiphanyCoordinatorRoleResultStatus::BackendMissing,
        Err(_) => EpiphanyCoordinatorRoleResultStatus::BackendUnavailable,
    }
}

fn native_role_finding(
    state: Option<&EpiphanyThreadState>,
    runtime_store: &Path,
    binding_id: &str,
    role_id: EpiphanyRoleResultRoleId,
) -> Option<EpiphanyRoleFindingInterpretation> {
    let job_id = latest_runtime_job_id_for_binding(state, binding_id)?;
    runtime_role_worker_result(runtime_store, job_id)
        .ok()
        .flatten()
        .map(|result| interpret_runtime_role_worker_result(role_id, &result))
}

fn native_reorient_result_status(
    state: Option<&EpiphanyThreadState>,
    runtime_store: &Path,
    binding_id: &str,
) -> EpiphanyCrrcResultStatus {
    let Some(job_id) = latest_runtime_job_id_for_binding(state, binding_id) else {
        return EpiphanyCrrcResultStatus::BackendMissing;
    };
    match runtime_job_snapshot(runtime_store, job_id) {
        Ok(Some(snapshot)) => map_runtime_reorient_result_status(&snapshot),
        Ok(None) => EpiphanyCrrcResultStatus::BackendMissing,
        Err(_) => EpiphanyCrrcResultStatus::BackendUnavailable,
    }
}

fn reconcile_native_runtime_jobs(
    jobs: &mut [epiphany_core::EpiphanyJobView],
    state: Option<&EpiphanyThreadState>,
    runtime_store: &Path,
) {
    for job in jobs {
        let Some(runtime_job_id) = job
            .runtime_job_id
            .clone()
            .or_else(|| latest_runtime_job_id_for_binding(state, &job.id).map(str::to_string))
        else {
            continue;
        };
        let Ok(Some(snapshot)) = runtime_job_snapshot(runtime_store, &runtime_job_id) else {
            continue;
        };
        job.runtime_job_id = Some(runtime_job_id);
        job.status = map_runtime_job_status(snapshot.job.status);
        job.progress_note = Some(snapshot.job.summary.clone());
    }
}

fn latest_runtime_job_id_for_binding<'a>(
    state: Option<&'a EpiphanyThreadState>,
    binding_id: &str,
) -> Option<&'a str> {
    state?
        .runtime_links
        .iter()
        .find(|link| link.binding_id == binding_id && !link.runtime_job_id.trim().is_empty())
        .map(|link| link.runtime_job_id.as_str())
}

fn reorient_retrieval_status(
    status: EpiphanyRetrievalFreshnessStatus,
) -> EpiphanyReorientFreshnessStatus {
    match status {
        EpiphanyRetrievalFreshnessStatus::Missing
        | EpiphanyRetrievalFreshnessStatus::Unavailable => EpiphanyReorientFreshnessStatus::Unknown,
        EpiphanyRetrievalFreshnessStatus::Ready => EpiphanyReorientFreshnessStatus::Clean,
        EpiphanyRetrievalFreshnessStatus::Stale => EpiphanyReorientFreshnessStatus::Stale,
        EpiphanyRetrievalFreshnessStatus::Indexing => EpiphanyReorientFreshnessStatus::Dirty,
    }
}

fn reorient_graph_status(status: EpiphanyGraphFreshnessStatus) -> EpiphanyReorientFreshnessStatus {
    match status {
        EpiphanyGraphFreshnessStatus::Missing => EpiphanyReorientFreshnessStatus::Unknown,
        EpiphanyGraphFreshnessStatus::Ready => EpiphanyReorientFreshnessStatus::Clean,
        EpiphanyGraphFreshnessStatus::Stale => EpiphanyReorientFreshnessStatus::Stale,
    }
}

fn reorient_watcher_status(status: EpiphanyInvalidationStatus) -> EpiphanyReorientFreshnessStatus {
    match status {
        EpiphanyInvalidationStatus::Unavailable => EpiphanyReorientFreshnessStatus::Unknown,
        EpiphanyInvalidationStatus::Clean => EpiphanyReorientFreshnessStatus::Clean,
        EpiphanyInvalidationStatus::Changed => EpiphanyReorientFreshnessStatus::Changed,
    }
}

fn map_runtime_role_result_status(
    snapshot: &EpiphanyRuntimeJobSnapshot,
) -> EpiphanyCoordinatorRoleResultStatus {
    match snapshot.job.status {
        EpiphanyRuntimeJobStatus::Queued => EpiphanyCoordinatorRoleResultStatus::Pending,
        EpiphanyRuntimeJobStatus::Running | EpiphanyRuntimeJobStatus::WaitingForReview => {
            EpiphanyCoordinatorRoleResultStatus::Running
        }
        EpiphanyRuntimeJobStatus::Completed => {
            if snapshot.result.is_some() {
                EpiphanyCoordinatorRoleResultStatus::Completed
            } else {
                EpiphanyCoordinatorRoleResultStatus::Pending
            }
        }
        EpiphanyRuntimeJobStatus::Failed => EpiphanyCoordinatorRoleResultStatus::Failed,
        EpiphanyRuntimeJobStatus::Cancelled => EpiphanyCoordinatorRoleResultStatus::Cancelled,
    }
}

fn map_runtime_reorient_result_status(
    snapshot: &EpiphanyRuntimeJobSnapshot,
) -> EpiphanyCrrcResultStatus {
    match snapshot.job.status {
        EpiphanyRuntimeJobStatus::Queued => EpiphanyCrrcResultStatus::Pending,
        EpiphanyRuntimeJobStatus::Running | EpiphanyRuntimeJobStatus::WaitingForReview => {
            EpiphanyCrrcResultStatus::Running
        }
        EpiphanyRuntimeJobStatus::Completed => {
            if snapshot.result.is_some() {
                EpiphanyCrrcResultStatus::Completed
            } else {
                EpiphanyCrrcResultStatus::Pending
            }
        }
        EpiphanyRuntimeJobStatus::Failed => EpiphanyCrrcResultStatus::Failed,
        EpiphanyRuntimeJobStatus::Cancelled => EpiphanyCrrcResultStatus::Cancelled,
    }
}

fn map_runtime_job_status(status: EpiphanyRuntimeJobStatus) -> EpiphanyJobStatus {
    match status {
        EpiphanyRuntimeJobStatus::Queued => EpiphanyJobStatus::Pending,
        EpiphanyRuntimeJobStatus::Running | EpiphanyRuntimeJobStatus::WaitingForReview => {
            EpiphanyJobStatus::Running
        }
        EpiphanyRuntimeJobStatus::Completed => EpiphanyJobStatus::Completed,
        EpiphanyRuntimeJobStatus::Failed => EpiphanyJobStatus::Failed,
        EpiphanyRuntimeJobStatus::Cancelled => EpiphanyJobStatus::Cancelled,
    }
}

fn native_role_result_note(
    status: EpiphanyCoordinatorRoleResultStatus,
    snapshot: &EpiphanyRuntimeJobSnapshot,
) -> String {
    match status {
        EpiphanyCoordinatorRoleResultStatus::Failed => {
            format!("Role runtime job failed: {}", snapshot.job.summary)
        }
        EpiphanyCoordinatorRoleResultStatus::Cancelled => {
            format!("Role runtime job was cancelled: {}", snapshot.job.summary)
        }
        EpiphanyCoordinatorRoleResultStatus::Completed => {
            "Role runtime job has a terminal lifecycle receipt; use typed worker results for reviewable findings.".to_string()
        }
        EpiphanyCoordinatorRoleResultStatus::Running => {
            "Role runtime job is still running.".to_string()
        }
        EpiphanyCoordinatorRoleResultStatus::Pending => {
            "Role runtime job has not produced a terminal receipt yet.".to_string()
        }
        EpiphanyCoordinatorRoleResultStatus::BackendUnavailable => {
            "The bound runtime backend is unavailable.".to_string()
        }
        EpiphanyCoordinatorRoleResultStatus::BackendMissing => {
            "The bound runtime backend job or item is missing.".to_string()
        }
        EpiphanyCoordinatorRoleResultStatus::MissingState => {
            "No authoritative Epiphany state exists for this thread.".to_string()
        }
        EpiphanyCoordinatorRoleResultStatus::MissingBinding => {
            "No matching Epiphany role specialist binding exists.".to_string()
        }
    }
}

fn native_reorient_result_note(
    status: EpiphanyCrrcResultStatus,
    snapshot: &EpiphanyRuntimeJobSnapshot,
) -> String {
    match status {
        EpiphanyCrrcResultStatus::Failed => {
            format!("Reorientation runtime job failed: {}", snapshot.job.summary)
        }
        EpiphanyCrrcResultStatus::Cancelled => {
            format!("Reorientation runtime job was cancelled: {}", snapshot.job.summary)
        }
        EpiphanyCrrcResultStatus::Completed => {
            "Reorientation runtime job has a terminal lifecycle receipt; use typed worker results for reviewable findings.".to_string()
        }
        EpiphanyCrrcResultStatus::Running => {
            "Reorientation runtime job is still running.".to_string()
        }
        EpiphanyCrrcResultStatus::Pending => {
            "Reorientation runtime job has not produced a terminal receipt yet.".to_string()
        }
        EpiphanyCrrcResultStatus::BackendUnavailable => {
            "The bound runtime backend is unavailable.".to_string()
        }
        EpiphanyCrrcResultStatus::BackendMissing => {
            "The bound runtime backend job or item is missing.".to_string()
        }
        EpiphanyCrrcResultStatus::MissingState => {
            "No authoritative Epiphany state exists for this thread.".to_string()
        }
        EpiphanyCrrcResultStatus::MissingBinding => {
            "No matching Epiphany reorientation binding exists.".to_string()
        }
    }
}

fn empty_string_as_null(value: &str) -> Value {
    if value.trim().is_empty() {
        Value::Null
    } else {
        json!(value)
    }
}

fn native_tool_invocation_surface(runtime_store: &Path) -> Result<Value> {
    let spine_status = epiphany_core::runtime_spine_status(runtime_store)?;
    let invocations = epiphany_core::runtime_tool_invocation_statuses(runtime_store)?;
    Ok(json!({
        "source": "native",
        "runtimeStore": runtime_store.display().to_string(),
        "summary": {
            "present": spine_status.present,
            "intentCount": spine_status.tool_invocation_intents,
            "pendingCount": spine_status.pending_tool_invocations,
            "receiptCount": spine_status.tool_invocation_receipts,
        },
        "invocations": invocations,
    }))
}

fn coordinator_status_json(status: &epiphany_core::EpiphanyCoordinatorStatus) -> Result<Value> {
    let mut value = serde_json::to_value(status).context("failed to encode coordinator status")?;
    let decision = value
        .get("decision")
        .cloned()
        .ok_or_else(|| anyhow!("coordinator status encoded without decision"))?;
    if let (Value::Object(root), Value::Object(decision)) = (&mut value, decision) {
        for (key, item) in decision {
            root.entry(key).or_insert(item);
        }
    }
    Ok(value)
}

fn crrc_state_status_text(status: EpiphanyCrrcStateStatus) -> &'static str {
    match status {
        EpiphanyCrrcStateStatus::Missing => "missing",
        EpiphanyCrrcStateStatus::Ready => "ready",
    }
}

fn role_job_status(status: epiphany_core::EpiphanyJobStatus) -> EpiphanyRoleBoardJobStatus {
    match status {
        epiphany_core::EpiphanyJobStatus::Idle => EpiphanyRoleBoardJobStatus::Idle,
        epiphany_core::EpiphanyJobStatus::Needed => EpiphanyRoleBoardJobStatus::Needed,
        epiphany_core::EpiphanyJobStatus::Pending => EpiphanyRoleBoardJobStatus::Pending,
        epiphany_core::EpiphanyJobStatus::Running => EpiphanyRoleBoardJobStatus::Running,
        epiphany_core::EpiphanyJobStatus::Completed => EpiphanyRoleBoardJobStatus::Completed,
        epiphany_core::EpiphanyJobStatus::Failed => EpiphanyRoleBoardJobStatus::Failed,
        epiphany_core::EpiphanyJobStatus::Cancelled => EpiphanyRoleBoardJobStatus::Cancelled,
        epiphany_core::EpiphanyJobStatus::Blocked => EpiphanyRoleBoardJobStatus::Blocked,
        epiphany_core::EpiphanyJobStatus::Unavailable => EpiphanyRoleBoardJobStatus::Unavailable,
    }
}

pub fn render_status(status: &Value) -> String {
    let scene = &status["scene"]["scene"];
    let pressure = &status["pressure"]["pressure"];
    let reorient = &status["reorient"]["decision"];
    let result = &status["reorientResult"];
    let recommendation = &status["crrc"]["recommendation"];
    let coordinator = &status["coordinator"];
    let heartbeat = &status["heartbeat"];
    let persona = &status["persona"];
    let bifrost_bridge = &status["bifrostBridge"];
    let planning_response = &status["planning"];
    let planning_summary = &planning_response["summary"];
    let checkpoint = &scene["investigationCheckpoint"];
    let latest_heartbeat = heartbeat["latestEvent"].clone();
    let latest_persona = persona["latestArtifacts"]
        .as_array()
        .and_then(|items| items.first())
        .cloned()
        .unwrap_or_else(|| json!({}));

    let mut lines = vec![
        "Epiphany MVP Status".to_string(),
        format!("Thread: {}", text(&status["threadId"])),
        format!(
            "State: {} rev {} ({})",
            text(&scene["stateStatus"]),
            maybe(&scene["revision"], "none"),
            text(&scene["source"])
        ),
        String::new(),
        "Recommendation".to_string(),
        format!("- action: {}", text(&recommendation["action"])),
        format!(
            "- scene action: {}",
            maybe(&recommendation["recommendedSceneAction"], "none")
        ),
        format!("- reason: {}", text(&recommendation["reason"])),
        format!(
            "- coordinator: {} ({})",
            maybe(&coordinator["action"], "none"),
            maybe(&coordinator["targetRole"], "none")
        ),
        String::new(),
        "Continuity".to_string(),
        format!(
            "- pressure: {} ({}, prepare={})",
            text(&pressure["level"]),
            text(&pressure["status"]),
            bool_text(&pressure["shouldPrepareCompaction"])
        ),
        format!(
            "- reorient: {} via {}",
            text(&reorient["action"]),
            list_text(&reorient["reasons"], "none")
        ),
        format!("- next: {}", text(&reorient["nextAction"])),
        format!(
            "- result: {} for {}",
            text(&result["status"]),
            text(&result["bindingId"])
        ),
        String::new(),
        "Heartbeat".to_string(),
        format!(
            "- status: {}; clock {}; rate {}",
            maybe(&heartbeat["status"], "none"),
            maybe(&heartbeat["sceneClock"], "none"),
            maybe(&heartbeat["targetHeartbeatRate"], "none")
        ),
        format!(
            "- latest: {} / {} / {}",
            maybe(&latest_heartbeat["selectedRole"], "none"),
            maybe(&latest_heartbeat["actionType"], "none"),
            maybe(&latest_heartbeat["coordinatorAction"], "none")
        ),
        String::new(),
        "Persona".to_string(),
        format!(
            "- latest artifact: {}",
            maybe(&latest_persona["name"], "none")
        ),
        format!(
            "- latest content: {}",
            maybe(&latest_persona["content"], "none")
        ),
        format!(
            "- Bifrost bridge: {} ({}/{})",
            maybe(&bifrost_bridge["status"], "unavailable"),
            maybe(&bifrost_bridge["readySurfaceCount"], "0"),
            maybe(&bifrost_bridge["surfaceCount"], "0")
        ),
        format!(
            "- bridge surfaces: {}",
            bifrost_bridge_surface_text(bifrost_bridge)
        ),
        String::new(),
        "Planning".to_string(),
        format!(
            "- state: {} rev {}",
            maybe(&planning_response["stateStatus"], "none"),
            maybe(&planning_response["stateRevision"], "none")
        ),
        format!(
            "- captures: {} (pending {}, github {})",
            maybe(&planning_summary["captureCount"], "0"),
            maybe(&planning_summary["pendingCaptureCount"], "0"),
            maybe(&planning_summary["githubIssueCaptureCount"], "0")
        ),
        format!(
            "- backlog: {} (ready {})",
            maybe(&planning_summary["backlogItemCount"], "0"),
            maybe(&planning_summary["readyBacklogItemCount"], "0")
        ),
        format!(
            "- roadmap streams: {}; objective drafts: {} (draft {})",
            maybe(&planning_summary["roadmapStreamCount"], "0"),
            maybe(&planning_summary["objectiveDraftCount"], "0"),
            maybe(&planning_summary["draftObjectiveCount"], "0")
        ),
        format!(
            "- active objective: {}",
            maybe(&planning_summary["activeObjective"], "none")
        ),
        format!("- note: {}", maybe(&planning_summary["note"], "none")),
        String::new(),
        "Role Lanes".to_string(),
    ];
    if let Some(roles) = status["roles"]["roles"].as_array() {
        for lane in roles {
            lines.push(format!(
                "- {}: {} ({}) - {}",
                text(&lane["title"]),
                text(&lane["status"]),
                text(&lane["ownerRole"]),
                text(&lane["note"])
            ));
        }
    }
    lines.extend([String::new(), "Role Findings".to_string()]);
    for (role_id, label) in [
        ("imagination", "Imagination / Planning"),
        ("research", "Eyes / Research"),
        ("modeling", "Modeling / Checkpoint"),
        ("verification", "Verification / Review"),
    ] {
        let role_result = &status["roleResults"][role_id];
        lines.push(format!(
            "- {}: {} for {}",
            label,
            maybe(&role_result["status"], "none"),
            maybe(&role_result["bindingId"], "none")
        ));
    }
    lines.extend([
        String::new(),
        "Checkpoint".to_string(),
        format!("- id: {}", maybe(&checkpoint["checkpointId"], "none")),
        format!(
            "- disposition: {}",
            maybe(&checkpoint["disposition"], "none")
        ),
        format!("- focus: {}", maybe(&checkpoint["focus"], "none")),
        format!("- next: {}", maybe(&checkpoint["nextAction"], "none")),
        String::new(),
        "Jobs".to_string(),
    ]);
    if let Some(jobs) = status["jobs"]["jobs"].as_array() {
        if jobs.is_empty() {
            lines.push("- none".to_string());
        } else {
            for job in jobs {
                lines.push(format!(
                    "- {}: {} {}, {} [{}]",
                    text(&job["id"]),
                    text(&job["status"]),
                    text(&job["kind"]),
                    text(&job["ownerRole"]),
                    text(&job["scope"])
                ));
            }
        }
    }
    let tools = &status["tools"];
    let tool_summary = &tools["summary"];
    lines.extend([
        String::new(),
        "Tools".to_string(),
        format!(
            "- intents: {} pending: {} receipts: {}",
            maybe(&tool_summary["intentCount"], "0"),
            maybe(&tool_summary["pendingCount"], "0"),
            maybe(&tool_summary["receiptCount"], "0")
        ),
        format!("- runtime store: {}", maybe(&tools["runtimeStore"], "none")),
    ]);
    if let Some(invocations) = tools["invocations"].as_array() {
        if invocations.is_empty() {
            lines.push("- latest: none".to_string());
        } else {
            for invocation in invocations.iter().rev().take(5) {
                lines.push(format!(
                    "- {}: {} {}/{} ({})",
                    maybe(&invocation["intentId"], "unknown"),
                    maybe(&invocation["status"], "unknown"),
                    maybe(&invocation["server"], "unknown"),
                    maybe(&invocation["toolName"], "unknown"),
                    maybe(&invocation["adapter"], "unknown")
                ));
            }
        }
    }
    lines.extend([
        String::new(),
        "Available Actions".to_string(),
        format!("- {}", list_text(&scene["availableActions"], "none")),
    ]);
    format!("{}\n", lines.join("\n"))
}

pub fn sanitize_for_operator(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sanitized = serde_json::Map::new();
            for (key, item) in map {
                if SEALED_DIRECT_THOUGHT_KEYS
                    .iter()
                    .any(|candidate| candidate == &key)
                {
                    sanitized.insert(key.clone(), sealed_direct_thought(&key, &item));
                } else if SEALED_LONG_TEXT_KEYS
                    .iter()
                    .any(|candidate| candidate == &key)
                    && let Some(text) = item.as_str()
                    && text.chars().count() > MAX_OPERATOR_TEXT_CHARS
                {
                    sanitized.insert(key.clone(), sealed_long_text(&key, text));
                } else {
                    sanitized.insert(key, sanitize_for_operator(item));
                }
            }
            Value::Object(sanitized)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sanitize_for_operator).collect()),
        other => other,
    }
}

fn sealed_direct_thought(key: &str, value: &Value) -> Value {
    let size = match value {
        Value::String(text) => Some(text.chars().count()),
        Value::Array(items) => Some(items.len()),
        Value::Object(map) => Some(map.len()),
        _ => None,
    };
    let mut sealed = json!({
        "sealed": true,
        "key": key,
        "reason": "Operator-safe dogfood views use projected findings and audit receipts; direct agent transcript/thought payloads stay sealed unless the user explicitly requests forensic debugging.",
    });
    if let Some(size) = size {
        sealed["size"] = json!(size);
    }
    sealed
}

fn sealed_long_text(key: &str, text: &str) -> Value {
    let preview: String = text.chars().take(240).collect();
    json!({
        "sealed": true,
        "key": key,
        "reason": "Long operator note sealed to keep status artifacts readable; inspect the sealed transcript or prompt source only during explicit forensic debugging.",
        "size": text.chars().count(),
        "preview": preview,
    })
}

pub fn native_json(bin_name: &str, args: &[&str]) -> Result<Value> {
    let sibling = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .map(|path| path.join(format!("{bin_name}.exe")));
    let configured = PathBuf::from(
        env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| DEFAULT_CARGO_TARGET_DIR.to_string()),
    )
    .join("debug")
    .join(format!("{bin_name}.exe"));
    let exe = sibling.filter(|path| path.exists()).unwrap_or(configured);
    let output = if exe.exists() {
        Command::new(&exe).args(args).output()
    } else {
        let mut command = Command::new("cargo");
        command
            .arg("run")
            .arg("--quiet")
            .arg("--manifest-path")
            .arg("epiphany-core/Cargo.toml")
            .arg("--bin")
            .arg(bin_name)
            .arg("--")
            .args(args)
            .output()
    }
    .with_context(|| format!("failed to run {bin_name}"))?;
    if !output.status.success() {
        return Err(anyhow!(
            "{} failed: {}{}",
            bin_name,
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("{bin_name} returned invalid JSON"))
}

fn take_string(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("{name} requires a value"))
}

fn take_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(take_string(args, name)?))
}

#[cfg(test)]
mod native_interrupt_tests {
    use super::*;
    use epiphany_state_model::{EpiphanyJobBinding, EpiphanyJobKind};

    #[test]
    fn native_operator_contract_has_one_store_and_no_codex_state_field() {
        let source = include_str!("epiphany-mvp-status.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        let compatibility_field = ["epiphany", "State"].concat();
        assert!(!production.contains(&compatibility_field));
        assert!(!production.contains("--thread-state-store"));
        assert!(!production.contains("--runtime-store"));
        assert!(production.contains("--store"));
    }

    #[test]
    fn default_status_interrupt_uses_native_coordinator_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("status-interrupt.cc");
        let service = epiphany_core::EpiphanyCoordinatorService::new(&store);
        service.apply_state_update(
            "thread-1",
            epiphany_core::EpiphanyStateUpdate {
                expected_revision: Some(0),
                job_bindings: Some(vec![EpiphanyJobBinding {
                    id: "modeling-worker".to_string(),
                    kind: EpiphanyJobKind::Specialist,
                    scope: "model".to_string(),
                    owner_role: "modeling".to_string(),
                    authority_scope: Some("model".to_string()),
                    linked_subgoal_ids: Vec::new(),
                    linked_graph_node_ids: Vec::new(),
                    blocking_reason: None,
                }]),
                ..Default::default()
            },
            None,
        )?;
        let args = Args {
            thread_id: Some("thread-1".to_string()),
            cwd: temp.path().to_path_buf(),
            store,
            ephemeral: true,
            json: true,
            result: None,
            transcript: PathBuf::new(),
            stderr: PathBuf::new(),
            interrupt_binding: Some("modeling-worker".to_string()),
            interrupt_reason: Some("native status proof".to_string()),
        };
        let result = run_status(&args)?;
        assert_eq!(result["source"], "native");
        assert_eq!(result["cancelRequested"], false);
        assert_eq!(
            result["state"]["job_bindings"][0]["blocking_reason"],
            "native status proof"
        );
        Ok(())
    }
}

pub fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()
            .context("failed to resolve current directory")?
            .join(path))
    }
}

fn text(value: &Value) -> String {
    value
        .as_str()
        .map(ToString::to_string)
        .unwrap_or_else(|| value.to_string())
}

fn maybe(value: &Value, fallback: &str) -> String {
    if value.is_null() {
        return fallback.to_string();
    }
    if let Some(text) = value.as_str() {
        if text.is_empty() {
            return fallback.to_string();
        }
        return text.to_string();
    }
    value.to_string()
}

fn bool_text(value: &Value) -> &'static str {
    if value.as_bool().unwrap_or(false) {
        "true"
    } else {
        "false"
    }
}

fn list_text(value: &Value, fallback: &str) -> String {
    let Some(items) = value.as_array() else {
        return fallback.to_string();
    };
    if items.is_empty() {
        return fallback.to_string();
    }
    items.iter().map(text).collect::<Vec<_>>().join(", ")
}

fn bifrost_bridge_surface_text(value: &Value) -> String {
    let Some(items) = value["surfaces"].as_array() else {
        return "none".to_string();
    };
    if items.is_empty() {
        return "none".to_string();
    }
    items
        .iter()
        .map(|surface| {
            format!(
                "{}={}",
                maybe(&surface["id"], "unknown"),
                maybe(&surface["status"], "unknown")
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}
