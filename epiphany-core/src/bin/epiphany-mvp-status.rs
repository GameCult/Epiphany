use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use epiphany_core::EpiphanyCoordinatorRoleResultStatus;
use epiphany_core::EpiphanyCoordinatorStatusInput;
use epiphany_core::EpiphanyCrrcInput;
use epiphany_core::EpiphanyCrrcReorientAction;
use epiphany_core::EpiphanyCrrcResultStatus;
use epiphany_core::EpiphanyCrrcStateStatus;
use epiphany_core::EpiphanyJobStatus;
use epiphany_core::EpiphanyJobsInput;
use epiphany_core::EpiphanyReorientAction;
use epiphany_core::EpiphanyReorientFreshnessStatus;
use epiphany_core::EpiphanyReorientInput;
use epiphany_core::EpiphanyReorientPressureLevel;
use epiphany_core::EpiphanyRoleBoardCheckpointSummary;
use epiphany_core::EpiphanyRoleBoardInput;
use epiphany_core::EpiphanyRoleBoardJob;
use epiphany_core::EpiphanyRoleBoardJobStatus;
use epiphany_core::EpiphanyRoleBoardPlanningSummary;
use epiphany_core::EpiphanyRuntimeJobSnapshot;
use epiphany_core::EpiphanyRuntimeJobStatus;
use epiphany_core::EpiphanySceneInput;
use epiphany_core::EpiphanyTokenUsageSnapshot;
use epiphany_core::derive_coordinator_finding_signals;
use epiphany_core::derive_coordinator_status;
use epiphany_core::derive_jobs;
use epiphany_core::derive_planning_view;
use epiphany_core::derive_pressure_view;
use epiphany_core::derive_role_board;
use epiphany_core::derive_scene;
use epiphany_core::load_thread_state_entry;
use epiphany_core::recommend_crrc_action;
use epiphany_core::recommend_reorientation;
use epiphany_core::runtime_job_snapshot;
use epiphany_state_model::EpiphanyThreadState;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_APP_SERVER: &str = r"C:\Users\Meta\.cargo-target-codex\debug\codex-app-server.exe";
const DEFAULT_CARGO_TARGET_DIR: &str = r"C:\Users\Meta\.cargo-target-codex";
const DEFAULT_THREAD_STATE_STORE: &str = "state/thread-state.msgpack";
const DEFAULT_RUNTIME_STORE: &str = "state/runtime-spine.msgpack";
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
        if args.source == StatusSource::Codex {
            write_transcript_telemetry(&args.transcript, &result.with_extension("telemetry.json"))?;
        }
    }
    if args.source == StatusSource::Codex {
        write_transcript_telemetry(
            &args.transcript,
            &args.transcript.with_extension("telemetry.json"),
        )?;
    }
    if args.json {
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        print!("{}", render_status(&status));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatusSource {
    Native,
    Codex,
}

#[derive(Debug)]
struct Args {
    source: StatusSource,
    app_server: PathBuf,
    codex_home: PathBuf,
    thread_id: Option<String>,
    cwd: PathBuf,
    thread_state_store: PathBuf,
    runtime_store: PathBuf,
    ephemeral: bool,
    json: bool,
    result: Option<PathBuf>,
    transcript: PathBuf,
    stderr: PathBuf,
}

impl Args {
    fn parse() -> Result<Self> {
        let root = env::current_dir().context("failed to resolve current directory")?;
        let mut args = env::args().skip(1);
        let mut parsed = Args {
            source: StatusSource::Codex,
            app_server: PathBuf::from(DEFAULT_APP_SERVER),
            codex_home: env::var_os("CODEX_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir().join(".codex")),
            thread_id: None,
            cwd: root.clone(),
            thread_state_store: PathBuf::from(DEFAULT_THREAD_STATE_STORE),
            runtime_store: PathBuf::from(DEFAULT_RUNTIME_STORE),
            ephemeral: true,
            json: false,
            result: None,
            transcript: root
                .join(".epiphany-status")
                .join("epiphany-mvp-status-transcript.jsonl"),
            stderr: root
                .join(".epiphany-status")
                .join("epiphany-mvp-status-server.stderr.log"),
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--source" => parsed.source = take_source(&mut args, "--source")?,
                "--native" => parsed.source = StatusSource::Native,
                "--codex" => parsed.source = StatusSource::Codex,
                "--app-server" => parsed.app_server = take_path(&mut args, "--app-server")?,
                "--codex-home" => parsed.codex_home = take_path(&mut args, "--codex-home")?,
                "--thread-id" => parsed.thread_id = Some(take_string(&mut args, "--thread-id")?),
                "--cwd" => parsed.cwd = take_path(&mut args, "--cwd")?,
                "--thread-state-store" => {
                    parsed.thread_state_store = take_path(&mut args, "--thread-state-store")?
                }
                "--runtime-store" => {
                    parsed.runtime_store = take_path(&mut args, "--runtime-store")?
                }
                "--ephemeral" => parsed.ephemeral = true,
                "--no-ephemeral" => parsed.ephemeral = false,
                "--json" => parsed.json = true,
                "--result" => parsed.result = Some(take_path(&mut args, "--result")?),
                "--transcript" => parsed.transcript = take_path(&mut args, "--transcript")?,
                "--stderr" => parsed.stderr = take_path(&mut args, "--stderr")?,
                _ => return Err(anyhow!("unknown argument: {arg}")),
            }
        }
        Ok(parsed)
    }
}

fn run_status(args: &Args) -> Result<Value> {
    match args.source {
        StatusSource::Native => run_native_status(args),
        StatusSource::Codex => run_codex_status(args),
    }
}

fn run_native_status(args: &Args) -> Result<Value> {
    let cwd = absolute_path(&args.cwd)?;
    let store_path = absolute_path(&args.thread_state_store)?;
    let runtime_store_path = absolute_path(&args.runtime_store)?;
    let state_entry = if store_path.exists() {
        load_thread_state_entry(&store_path)
            .with_context(|| format!("failed to load {}", store_path.display()))?
    } else {
        None
    };
    let thread_id = args
        .thread_id
        .clone()
        .or_else(|| state_entry.as_ref().map(|entry| entry.thread_id.clone()))
        .unwrap_or_else(|| "native-local".to_string());
    let state = state_entry
        .as_ref()
        .map(|entry| entry.state())
        .transpose()
        .context("failed to decode native Epiphany thread state")?;
    let state_ref = state.as_ref();
    let loaded = state_ref.is_some();

    let scene = derive_scene(EpiphanySceneInput {
        state: state_ref,
        loaded,
        reorient_binding_id: REORIENT_BINDING_ID,
    });
    let pressure = derive_pressure_view(None::<&EpiphanyTokenUsageSnapshot>);
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
            retrieval_status: EpiphanyReorientFreshnessStatus::Unknown,
            retrieval_dirty_paths: Vec::new(),
            graph_status: EpiphanyReorientFreshnessStatus::Unknown,
            graph_dirty_paths: Vec::new(),
            watcher_status: EpiphanyReorientFreshnessStatus::Unknown,
            watcher_changed_paths: Vec::new(),
            watcher_graph_node_ids: Vec::new(),
            active_frontier_node_ids: state_ref
                .and_then(|state| state.graph_frontier.as_ref())
                .map(|frontier| frontier.active_node_ids.clone())
                .unwrap_or_default(),
            watched_root: Some(cwd.clone()),
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
    let finding_signals = derive_coordinator_finding_signals(state_ref, None, None, None, None);
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
        modeling_result_accepted_after_verification: finding_signals
            .modeling_result_accepted_after_verification,
        implementation_evidence_after_verification: finding_signals
            .implementation_evidence_after_verification,
        verification_result_cites_implementation_evidence: finding_signals
            .verification_result_cites_implementation_evidence,
        verification_result_covers_current_modeling: finding_signals
            .verification_result_covers_current_modeling,
        verification_result_accepted: finding_signals.verification_result_accepted,
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
        "face": native_aux.face,
        "voidMemory": native_aux.void_memory,
    });
    Ok(sanitize_for_operator(status))
}

fn run_codex_status(args: &Args) -> Result<Value> {
    let app_server = absolute_path(&args.app_server)?;
    let codex_home = absolute_path(&args.codex_home)?;
    let cwd = absolute_path(&args.cwd)?;
    let transcript = absolute_path(&args.transcript)?;
    let stderr = absolute_path(&args.stderr)?;

    if !app_server.exists() {
        return Err(anyhow!(
            "codex app-server binary not found: {}",
            app_server.display()
        ));
    }
    fs::create_dir_all(&codex_home)
        .with_context(|| format!("failed to create {}", codex_home.display()))?;
    let mut client = AppServerClient::start(&app_server, &codex_home, &transcript, &stderr)?;
    client.send(
        "initialize",
        Some(json!({
            "clientInfo": {
                "name": "epiphany-mvp-status",
                "title": "Epiphany MVP Status",
                "version": "0.1.0",
            },
            "capabilities": {"experimentalApi": true},
        })),
        true,
    )?;
    client.send("initialized", None, false)?;

    let thread_id = if let Some(thread_id) = &args.thread_id {
        if !args.ephemeral {
            client.send("thread/resume", Some(json!({"threadId": thread_id})), true)?;
        }
        thread_id.clone()
    } else {
        let started = client.send(
            "thread/start",
            Some(json!({"cwd": cwd, "ephemeral": args.ephemeral})),
            true,
        )?;
        started["thread"]["id"]
            .as_str()
            .ok_or_else(|| anyhow!("thread/start returned no thread id"))?
            .to_string()
    };

    let read = client.send(
        "thread/read",
        Some(json!({"threadId": thread_id, "includeTurns": false})),
        true,
    )?;
    let view = client.send(
        "thread/epiphany/view",
        Some(json!({"threadId": thread_id, "lenses": ["scene", "pressure", "jobs", "roles", "planning", "reorient", "crrc", "coordinator"]})),
        true,
    )?;
    let scene = json!({
        "threadId": thread_id,
        "scene": view.get("scene").cloned().unwrap_or_else(|| json!(null)),
    });
    let pressure = json!({
        "threadId": thread_id,
        "source": "live",
        "pressure": view.get("pressure").cloned().unwrap_or_else(|| json!(null)),
    });
    let reorient = view.get("reorient").cloned().unwrap_or_else(|| json!(null));
    let jobs = json!({
        "threadId": thread_id,
        "source": "live",
        "jobs": view.get("jobs").cloned().unwrap_or_else(|| json!([])),
    });
    let roles = view.get("roles").cloned().unwrap_or_else(|| json!(null));
    let planning = view.get("planning").cloned().unwrap_or_else(|| json!(null));
    let role_results = json!({
        "imagination": client.send("thread/epiphany/roleResult", Some(json!({"threadId": thread_id, "roleId": "imagination"})), true)?,
        "research": client.send("thread/epiphany/roleResult", Some(json!({"threadId": thread_id, "roleId": "research"})), true)?,
        "modeling": client.send("thread/epiphany/roleResult", Some(json!({"threadId": thread_id, "roleId": "modeling"})), true)?,
        "verification": client.send("thread/epiphany/roleResult", Some(json!({"threadId": thread_id, "roleId": "verification"})), true)?,
    });
    let reorient_result = client.send(
        "thread/epiphany/reorientResult",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let crrc = view.get("crrc").cloned().unwrap_or_else(|| json!(null));
    let coordinator = view
        .get("coordinator")
        .cloned()
        .unwrap_or_else(|| json!(null));
    let runtime_store_path = absolute_path(&args.runtime_store)?;
    let tool_invocations = native_tool_invocation_surface(&runtime_store_path)?;
    let root = env::current_dir().context("failed to resolve current directory")?;
    let native_aux = native_auxiliary_status(&root)?;
    let status = json!({
        "threadId": thread_id,
        "read": read,
        "view": view,
        "scene": scene,
        "pressure": pressure,
        "reorient": reorient,
        "jobs": jobs,
        "roles": roles,
        "planning": planning,
        "roleResults": role_results,
        "reorientResult": reorient_result,
        "crrc": crrc,
        "coordinator": coordinator,
        "tools": tool_invocations,
        "heartbeat": native_aux.heartbeat,
        "face": native_aux.face,
        "voidMemory": native_aux.void_memory,
    });
    Ok(sanitize_for_operator(status))
}

struct NativeAuxiliaryStatus {
    heartbeat: Value,
    face: Value,
    void_memory: Value,
}

fn native_auxiliary_status(root: &Path) -> Result<NativeAuxiliaryStatus> {
    let heartbeat_dir = root.join(".epiphany-heartbeats");
    let face_dir = root.join(".epiphany-face");
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
    let latest_face = native_json(
        "epiphany-face-discord",
        &[
            "latest",
            "--artifact-dir",
            &face_dir.to_string_lossy(),
            "--limit",
            "8",
        ],
    )
    .unwrap_or_else(
        |error| json!({"status": "error", "error": error.to_string(), "latestArtifacts": []}),
    );
    let face = json!({
        "status": "ready",
        "artifactDir": face_dir,
        "latestArtifacts": latest_face.get("latestArtifacts").cloned().unwrap_or_else(|| json!([])),
        "availableActions": ["faceBubble", "characterTurn", "discordPersonaPost"],
    });
    let void_memory = native_json(
        "epiphany-void-memory",
        &["status", "--config", "state/void-memory.toml"],
    )
    .unwrap_or_else(|error| json!({"ok": false, "error": error.to_string()}));

    Ok(NativeAuxiliaryStatus {
        heartbeat,
        face,
        void_memory,
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

pub struct AppServerClient {
    child: Child,
    stdin: ChildStdin,
    rx: mpsc::Receiver<Value>,
    transcript: Arc<Mutex<File>>,
    notifications: Arc<Mutex<Vec<Value>>>,
    next_id: u64,
}

impl AppServerClient {
    pub fn start(
        app_server: &Path,
        codex_home: &Path,
        transcript_path: &Path,
        stderr_path: &Path,
    ) -> Result<Self> {
        if let Some(parent) = transcript_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        if let Some(parent) = stderr_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let transcript = Arc::new(Mutex::new(
            File::create(transcript_path)
                .with_context(|| format!("failed to create {}", transcript_path.display()))?,
        ));
        let stderr_file = Arc::new(Mutex::new(
            File::create(stderr_path)
                .with_context(|| format!("failed to create {}", stderr_path.display()))?,
        ));
        let mut command = Command::new(app_server);
        command
            .current_dir(
                env::current_dir()?
                    .join("vendor")
                    .join("codex")
                    .join("codex-rs"),
            )
            .env("CODEX_HOME", codex_home)
            .env(
                "CARGO_TARGET_DIR",
                env::var("CARGO_TARGET_DIR")
                    .unwrap_or_else(|_| DEFAULT_CARGO_TARGET_DIR.to_string()),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command
            .spawn()
            .context("failed to spawn codex app-server")?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("app-server stdin unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("app-server stdout unavailable"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("app-server stderr unavailable"))?;
        let (tx, rx) = mpsc::channel();
        let transcript_for_stdout = Arc::clone(&transcript);
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let notifications_for_stdout = Arc::clone(&notifications);
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if line.trim().is_empty() {
                    continue;
                }
                let message = serde_json::from_str::<Value>(&line).unwrap_or_else(
                    |error| json!({"_decode_error": error.to_string(), "raw": line}),
                );
                record(&transcript_for_stdout, "received", &message);
                if message.get("method").is_some()
                    && message.get("id").is_none()
                    && let Ok(mut notifications) = notifications_for_stdout.lock()
                {
                    notifications.push(message.clone());
                }
                let _ = tx.send(message);
            }
        });
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if let Ok(mut file) = stderr_file.lock() {
                    let _ = writeln!(file, "{line}");
                }
            }
        });
        Ok(Self {
            child,
            stdin,
            rx,
            transcript,
            notifications,
            next_id: 1,
        })
    }

    pub fn send(
        &mut self,
        method: &str,
        params: Option<Value>,
        expect_response: bool,
    ) -> Result<Value> {
        let mut message = serde_json::Map::new();
        message.insert("method".to_string(), json!(method));
        let request_id = if expect_response {
            let id = self.next_id;
            self.next_id += 1;
            message.insert("id".to_string(), json!(id));
            Some(id)
        } else {
            None
        };
        if let Some(params) = params {
            message.insert("params".to_string(), params);
        }
        let message = Value::Object(message);
        record(&self.transcript, "sent", &message);
        writeln!(
            self.stdin,
            "{}",
            serde_json::to_string(&message).context("failed to encode request")?
        )
        .context("failed to write app-server request")?;
        self.stdin
            .flush()
            .context("failed to flush app-server stdin")?;
        let Some(request_id) = request_id else {
            return Ok(Value::Null);
        };
        self.wait_for(request_id)
    }

    fn wait_for(&mut self, request_id: u64) -> Result<Value> {
        let deadline = Instant::now() + Duration::from_secs(45);
        while Instant::now() < deadline {
            if let Some(status) = self.child.try_wait()? {
                return Err(anyhow!(
                    "app-server exited with {} before response {}",
                    status,
                    request_id
                ));
            }
            match self.rx.recv_timeout(Duration::from_millis(500)) {
                Ok(message) => {
                    if message.get("id").and_then(Value::as_u64) != Some(request_id) {
                        continue;
                    }
                    if let Some(error) = message.get("error") {
                        return Err(anyhow!("request {request_id} failed: {error}"));
                    }
                    let result = message
                        .get("result")
                        .cloned()
                        .ok_or_else(|| anyhow!("request {request_id} returned no result"))?;
                    if !result.is_object() {
                        return Err(anyhow!(
                            "request {request_id} returned non-object result: {result}"
                        ));
                    }
                    return Ok(result);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(error) => return Err(anyhow!("app-server response channel closed: {error}")),
            }
        }
        Err(anyhow!("timed out waiting for response {request_id}"))
    }

    pub fn notification_count(&self, method: &str, start_index: usize) -> usize {
        self.notifications
            .lock()
            .ok()
            .map(|notifications| {
                notifications
                    .iter()
                    .skip(start_index)
                    .filter(|message| message.get("method").and_then(Value::as_str) == Some(method))
                    .count()
            })
            .unwrap_or(0)
    }

    pub fn notification_len(&self) -> usize {
        self.notifications
            .lock()
            .map(|notifications| notifications.len())
            .unwrap_or(0)
    }

    pub fn require_no_notification(
        &mut self,
        method: &str,
        start_index: usize,
        timeout: Duration,
    ) -> Result<()> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if let Some(status) = self.child.try_wait()? {
                return Err(anyhow!(
                    "app-server exited with {} while checking notification {}",
                    status,
                    method
                ));
            }
            if self.notification_count(method, start_index) > 0 {
                return Err(anyhow!("unexpected notification {method}"));
            }
            thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }

    pub fn wait_for_notification(
        &mut self,
        method: &str,
        start_index: usize,
        timeout: Duration,
    ) -> Result<Value> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if let Some(status) = self.child.try_wait()? {
                return Err(anyhow!(
                    "app-server exited with {} before notification {}",
                    status,
                    method
                ));
            }
            if let Ok(notifications) = self.notifications.lock()
                && let Some(message) = notifications
                    .iter()
                    .skip(start_index)
                    .find(|message| message.get("method").and_then(Value::as_str) == Some(method))
            {
                return Ok(message.clone());
            }
            thread::sleep(Duration::from_millis(100));
        }
        Err(anyhow!("timed out waiting for notification {method}"))
    }
}

impl Drop for AppServerClient {
    fn drop(&mut self) {
        let _ = self.stdin.flush();
        let _ = self.child.kill();
        let _ = self.child.wait();
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
    let face = &status["face"];
    let planning_response = &status["planning"];
    let planning_summary = &planning_response["summary"];
    let checkpoint = &scene["investigationCheckpoint"];
    let latest_heartbeat = heartbeat["latestEvent"].clone();
    let latest_face = face["latestArtifacts"]
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
        "Face".to_string(),
        format!("- latest artifact: {}", maybe(&latest_face["name"], "none")),
        format!(
            "- latest content: {}",
            maybe(&latest_face["content"], "none")
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

pub fn write_transcript_telemetry(transcript: &Path, output: &Path) -> Result<()> {
    let _ = native_json(
        "epiphany-agent-telemetry",
        &[
            &transcript.to_string_lossy(),
            "--output",
            &output.to_string_lossy(),
        ],
    )?;
    Ok(())
}

pub fn native_json(bin_name: &str, args: &[&str]) -> Result<Value> {
    let exe = PathBuf::from(
        env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| DEFAULT_CARGO_TARGET_DIR.to_string()),
    )
    .join("debug")
    .join(format!("{bin_name}.exe"));
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

fn record(transcript: &Arc<Mutex<File>>, kind: &str, payload: &Value) {
    if let Ok(mut file) = transcript.lock() {
        let _ = writeln!(file, "{}", json!({kind: payload}));
    }
}

fn take_string(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("{name} requires a value"))
}

fn take_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(take_string(args, name)?))
}

fn take_source(args: &mut impl Iterator<Item = String>, name: &str) -> Result<StatusSource> {
    match take_string(args, name)?.as_str() {
        "native" => Ok(StatusSource::Native),
        "codex" => Ok(StatusSource::Codex),
        other => Err(anyhow!(
            "{name} must be 'native' or 'codex', received {other:?}"
        )),
    }
}

fn home_dir() -> PathBuf {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
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
