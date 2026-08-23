use anyhow::{Context, Result, anyhow};
use epiphany_core::{
    EpiphanyAgentPassContinuationAction, EpiphanyCoordinatorStatus, EpiphanyCoordinatorStatusInput,
    EpiphanyCrrcAction, EpiphanyCrrcRecommendation, EpiphanyCrrcResultStatus,
    EpiphanyCrrcSceneAction, EpiphanyCurrentWorkProjection, EpiphanyReorientAction,
    EpiphanyRoleBoardInput, EpiphanyRoleResultRoleId, EpiphanyRuntimeJobStatus,
    EpiphanySceneInput, EpiphanyTokenUsageSnapshot, RepoFrontierPlanningLifecycle,
    RepoFrontierPlanningLifecycleStage, RepoFrontierResearchLifecycle,
    RepoFrontierResearchLifecycleStage, derive_planning_view, derive_pressure_view,
    derive_role_board, derive_scene, runtime_job_snapshot,
};
use epiphany_self_policy::derive_coordinator_status;
use serde_json::{Value, json};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

const DEFAULT_COORDINATOR_STORE: &str = "state/runtime-spine.msgpack";
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
    store: PathBuf,
    json: bool,
    result: Option<PathBuf>,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut args = env::args().skip(1);
        let mut parsed = Self {
            thread_id: None,
            store: PathBuf::from(DEFAULT_COORDINATOR_STORE),
            json: false,
            result: None,
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--thread-id" => parsed.thread_id = Some(take_string(&mut args, "--thread-id")?),
                "--store" => parsed.store = take_path(&mut args, "--store")?,
                "--json" => parsed.json = true,
                "--result" => parsed.result = Some(take_path(&mut args, "--result")?),
                _ => return Err(anyhow!("unknown argument: {arg}")),
            }
        }
        Ok(parsed)
    }
}

fn run_status(args: &Args) -> Result<Value> {
    run_native_status(args)
}

pub fn native_coordinator_json(runtime_store: &Path, thread_id: &str) -> Result<Value> {
    run_native_status(
        &Args {
            thread_id: Some(thread_id.to_string()),
            store: runtime_store.to_path_buf(),
            json: true,
            result: None,
        },
    )
}

fn run_native_status(args: &Args) -> Result<Value> {
    let store_path = absolute_path(&args.store)?;
    let thread_id = args
        .thread_id
        .clone()
        .unwrap_or_else(|| "native-local".to_string());

    let mind = if store_path.exists() {
        Some(
            epiphany_core::assemble_mind_view(&store_path).with_context(|| {
                format!("failed to assemble keyed Mind {}", store_path.display())
            })?,
        )
    } else {
        None
    };
    let current_work = if mind.is_some() {
        epiphany_core::project_current_work(&store_path)
            .context("failed to derive current work from keyed Mind and runtime receipts")?
    } else {
        empty_current_work()
    };
    let pressure = derive_pressure_view(None::<&EpiphanyTokenUsageSnapshot>);
    let latest_reorientation_decision = mind
        .as_ref()
        .and_then(|mind| mind.reorientation_decisions.last());
    let reorientation_work = current_work.reorientation.as_ref();
    let reorient_action = latest_reorientation_decision
        .map(|decision| {
            if decision.mode == "regather" {
                EpiphanyReorientAction::Regather
            } else {
                EpiphanyReorientAction::Resume
            }
        })
        .unwrap_or(EpiphanyReorientAction::Resume);
    let reorient_result_status = keyed_reorientation_result_status(
        &store_path,
        reorientation_work,
        latest_reorientation_decision.is_some(),
    )?;
    let recommendation = keyed_reorientation_recommendation(
        reorientation_work,
        latest_reorientation_decision,
        pressure.should_prepare_compaction,
    );

    let scene = derive_scene(EpiphanySceneInput {
        mind: mind.as_ref(),
        loaded: mind.is_some(),
        reorientation_work_present: reorientation_work.is_some(),
    });
    let planning = derive_planning_view(mind.as_ref());
    let roles = derive_role_board(EpiphanyRoleBoardInput {
        mind_present: mind.is_some(),
        current_work: current_work.clone(),
    });
    let coordinator = derive_coordinator_status(EpiphanyCoordinatorStatusInput {
        mind_present: mind.is_some(),
        pressure: pressure.clone(),
        recommendation: recommendation.clone(),
        roles: roles.clone(),
        reorient_action,
        reorient_result_status,
        current_work: current_work.clone(),
    });
    let coordinator_json = coordinator_status_json(&coordinator)?;

    let research_lifecycle = mind.as_ref().map(|_| current_work.research.clone());
    let planning_lifecycle = mind
        .as_ref()
        .map(|_| current_work.frontier_planning.clone());
    let planning_eligibility = mind
        .as_ref()
        .map(|_| epiphany_core::runtime_repo_frontier_planning_eligibility(&store_path))
        .transpose()?;
    let frontier_relinquishment = mind
        .as_ref()
        .map(|_| epiphany_core::runtime_latest_repo_frontier_relinquishment(&store_path))
        .transpose()?
        .flatten();
    let body_job_id = current_work
        .body_modeling
        .as_ref()
        .and_then(|work| work.attempt.job_id.as_deref());
    let modeling_job_id = current_work
        .proposal_modeling
        .as_ref()
        .and_then(|work| work.attempt.job_id.as_deref())
        .or(body_job_id)
        .or(current_work
            .frontier_verdict_modeling
            .as_ref()
            .and_then(|work| work.attempt.job_id.as_deref()));
    let imagination_job_id = planning_lifecycle
        .as_ref()
        .and_then(|lifecycle| lifecycle.imagination_job_id.as_deref())
        .or(current_work
            .imagination_considerations
            .first()
            .and_then(|work| work.attempt.job_id.as_deref()))
        .or(current_work
            .admitted_model_direction_consideration
            .as_ref()
            .and_then(|work| work.attempt.job_id.as_deref()));
    let role_results = json!({
        "imagination": native_role_result(&store_path, imagination_job_id, EpiphanyRoleResultRoleId::Imagination),
        "research": native_role_result(
            &store_path,
            research_lifecycle.as_ref().and_then(|lifecycle| lifecycle.worker_job_id.as_deref()),
            EpiphanyRoleResultRoleId::Research,
        ),
        "modeling": native_role_result(&store_path, modeling_job_id, EpiphanyRoleResultRoleId::Modeling),
        "verification": native_role_result(
            &store_path,
            current_work.verification.as_ref().and_then(|work| work.attempt.job_id.as_deref()),
            EpiphanyRoleResultRoleId::Verification,
        ),
    });
    let tools = if mind.is_some() {
        native_tool_invocation_surface(&store_path)?
    } else {
        json!({"source": "native", "status": "missingMind"})
    };
    let reorient_result = keyed_reorientation_result(
        &store_path,
        reorientation_work,
        latest_reorientation_decision,
    )?;
    Ok(sanitize_for_operator(json!({
        "threadId": thread_id,
        "read": {
            "source": "native",
            "mindStore": store_path,
            "mindPresent": mind.is_some(),
            "projectionDigest": mind.as_ref().map(|mind| mind.projection_digest.as_str()),
        },
        "scene": {"threadId": thread_id, "scene": scene},
        "pressure": {"threadId": thread_id, "source": "native", "pressure": pressure},
        "reorient": {
            "threadId": thread_id,
            "source": "native",
            "decision": {
                "action": reorient_action,
                "nextAction": latest_reorientation_decision
                    .map(|decision| decision.next_safe_move.as_str())
                    .unwrap_or("No unresolved keyed reorientation obligation."),
                "requestId": reorientation_work.map(|work| work.request.request_id.as_str()),
                "authority": "keyedMindCurrentWork",
            },
        },
        "roles": {"threadId": thread_id, "source": "native", "roles": roles},
        "planning": planning,
        "currentWork": current_work,
        "frontierPlanning": {
            "lifecycle": planning_lifecycle,
            "eligibility": planning_eligibility,
        },
        "frontierRelinquishment": frontier_relinquishment,
        "roleResults": role_results,
        "reorientResult": reorient_result,
        "crrc": {"threadId": thread_id, "source": "native", "recommendation": recommendation},
        "coordinator": coordinator_json,
        "tools": tools,
    })))
}

fn empty_current_work() -> EpiphanyCurrentWorkProjection {
    EpiphanyCurrentWorkProjection {
        mind_projection_digest: String::new(),
        body_modeling: None,
        research: RepoFrontierResearchLifecycle {
            stage: RepoFrontierResearchLifecycleStage::Terminal,
            frontier_item_id: None,
            request_id: None,
            worker_job_id: None,
        },
        frontier_planning: RepoFrontierPlanningLifecycle {
            stage: RepoFrontierPlanningLifecycleStage::Unavailable,
            planning_request_id: None,
            imagination_job_id: None,
            imagination_result_id: None,
            mind_request_id: None,
            mind_job_id: None,
            mind_result_id: None,
            decision_id: None,
        },
        proposal_modeling: None,
        frontier_verdict_modeling: None,
        verification: None,
        reorientation: None,
        imagination_considerations: Vec::new(),
        admitted_model_direction_consideration: None,
        hands_frontier_ready: false,
    }
}

fn native_role_result(
    runtime_store: &Path,
    job_id: Option<&str>,
    role_id: EpiphanyRoleResultRoleId,
) -> Value {
    let Some(job_id) = job_id else {
        return json!({"source": "native", "status": "noCurrentWork"});
    };
    let snapshot = epiphany_core::read_runtime_role_result(Some(runtime_store), job_id, role_id);
    let mut result = json!({
        "source": "native",
        "status": snapshot.status,
        "runtimeJobId": job_id,
        "note": snapshot.note,
    });
    if let Some(finding) = snapshot.finding {
        result["finding"] = json!(finding);
    }
    result
}

fn keyed_reorientation_result_status(
    runtime_store: &Path,
    work: Option<&epiphany_core::EpiphanyReorientationWorkProjection>,
    accepted_decision_present: bool,
) -> Result<EpiphanyCrrcResultStatus> {
    let Some(work) = work else {
        return Ok(if accepted_decision_present {
            EpiphanyCrrcResultStatus::Completed
        } else {
            EpiphanyCrrcResultStatus::MissingBinding
        });
    };
    let Some(job_id) = work.attempt.job_id.as_deref() else {
        return Ok(EpiphanyCrrcResultStatus::MissingBinding);
    };
    let snapshot = runtime_job_snapshot(runtime_store, job_id)?
        .ok_or_else(|| anyhow!("keyed reorientation work lost its runtime job"))?;
    Ok(match snapshot.job.status {
        EpiphanyRuntimeJobStatus::Queued => EpiphanyCrrcResultStatus::Pending,
        EpiphanyRuntimeJobStatus::Running | EpiphanyRuntimeJobStatus::WaitingForReview => {
            EpiphanyCrrcResultStatus::Running
        }
        EpiphanyRuntimeJobStatus::Completed => EpiphanyCrrcResultStatus::Completed,
        EpiphanyRuntimeJobStatus::Failed => EpiphanyCrrcResultStatus::Failed,
        EpiphanyRuntimeJobStatus::Cancelled => EpiphanyCrrcResultStatus::Cancelled,
    })
}

fn keyed_reorientation_recommendation(
    work: Option<&epiphany_core::EpiphanyReorientationWorkProjection>,
    accepted: Option<&epiphany_core::EpiphanyMindReorientationDecisionDocument>,
    should_prepare_compaction: bool,
) -> EpiphanyCrrcRecommendation {
    let build = |action, scene, reason: &str| EpiphanyCrrcRecommendation {
        action,
        recommended_scene_action: scene,
        reason: reason.into(),
    };
    if let Some(work) = work {
        return match work.attempt.action {
            EpiphanyAgentPassContinuationAction::Launch => build(
                EpiphanyCrrcAction::LaunchReorientWorker,
                Some(EpiphanyCrrcSceneAction::ReorientLaunch),
                "The exact keyed continuity request has no live attempt.",
            ),
            EpiphanyAgentPassContinuationAction::Wait => build(
                EpiphanyCrrcAction::WaitForReorientWorker,
                Some(EpiphanyCrrcSceneAction::ReorientResult),
                "The exact keyed continuity attempt is still live.",
            ),
            EpiphanyAgentPassContinuationAction::Review => build(
                EpiphanyCrrcAction::ReviewReorientResult,
                Some(EpiphanyCrrcSceneAction::ReorientResult),
                "The exact keyed continuity result awaits admission.",
            ),
        };
    }
    if should_prepare_compaction {
        return build(
            EpiphanyCrrcAction::LaunchReorientWorker,
            Some(EpiphanyCrrcSceneAction::ReorientLaunch),
            "Context pressure requires a new keyed continuity request.",
        );
    }
    if accepted.is_some_and(|decision| decision.mode == "regather") {
        return build(
            EpiphanyCrrcAction::RegatherManually,
            Some(EpiphanyCrrcSceneAction::Reorient),
            "The accepted keyed continuity decision requires explicit regather.",
        );
    }
    build(
        EpiphanyCrrcAction::Continue,
        Some(EpiphanyCrrcSceneAction::Reorient),
        "No unresolved keyed reorientation obligation exists.",
    )
}

fn keyed_reorientation_result(
    runtime_store: &Path,
    work: Option<&epiphany_core::EpiphanyReorientationWorkProjection>,
    accepted: Option<&epiphany_core::EpiphanyMindReorientationDecisionDocument>,
) -> Result<Value> {
    let Some(work) = work else {
        return Ok(accepted
            .map(|decision| json!({"status": "accepted", "decision": decision}))
            .unwrap_or(Value::Null));
    };
    let Some(job_id) = work.attempt.job_id.as_deref() else {
        return Ok(json!({"status": "unlaunched", "requestId": work.request.request_id}));
    };
    let snapshot = runtime_job_snapshot(runtime_store, job_id)?
        .ok_or_else(|| anyhow!("keyed reorientation work lost its runtime job"))?;
    Ok(json!({
        "status": format!("{:?}", snapshot.job.status).to_ascii_lowercase(),
        "requestId": work.request.request_id,
        "jobId": job_id,
        "result": epiphany_core::runtime_reorient_worker_result(runtime_store, job_id)?,
    }))
}

fn native_tool_invocation_surface(runtime_store: &Path) -> Result<Value> {
    let spine_status = epiphany_core::runtime_spine_status(runtime_store)?;
    Ok(json!({
        "source": "native",
        "runtimeStore": runtime_store.display().to_string(),
        "summary": {
            "present": spine_status.present,
            "intentCount": spine_status.tool_invocation_intents,
            "pendingCount": spine_status.pending_tool_invocations,
            "receiptCount": spine_status.tool_invocation_receipts,
        },
        "invocations": epiphany_core::runtime_tool_invocation_statuses(runtime_store)?,
    }))
}

fn coordinator_status_json(status: &EpiphanyCoordinatorStatus) -> Result<Value> {
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

pub fn render_status(status: &Value) -> String {
    let read = &status["read"];
    let coordinator = &status["coordinator"];
    let current = &status["currentWork"];
    let mut lines = vec![
        "Epiphany Status".to_string(),
        format!(
            "Mind: {}",
            if read["mindPresent"].as_bool() == Some(true) {
                "ready"
            } else {
                "missing"
            }
        ),
        format!("Projection: {}", maybe(&read["projectionDigest"], "none")),
        format!("Coordinator: {}", maybe(&coordinator["action"], "none")),
        format!("Reason: {}", maybe(&coordinator["reason"], "none")),
        format!(
            "Hands ready: {}",
            current["handsFrontierReady"].as_bool().unwrap_or(false)
        ),
        String::new(),
        "Role lanes".to_string(),
    ];
    if let Some(roles) = status["roles"]["roles"].as_array() {
        for lane in roles {
            lines.push(format!(
                "- {}: {} — {}",
                maybe(&lane["title"], "unknown"),
                maybe(&lane["status"], "unknown"),
                maybe(&lane["note"], "")
            ));
        }
    }
    lines.push(String::new());
    lines.join("\n")
}

pub fn sanitize_for_operator(value: Value) -> Value {
    match value {
        Value::Object(values) => {
            let mut sanitized = serde_json::Map::new();
            for (key, item) in values {
                if SEALED_DIRECT_THOUGHT_KEYS.contains(&key.as_str()) {
                    continue;
                }
                if SEALED_LONG_TEXT_KEYS.contains(&key.as_str()) {
                    sanitized.insert(key, truncate_json_text(item));
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

fn truncate_json_text(value: Value) -> Value {
    let Value::String(text) = value else {
        return sanitize_for_operator(value);
    };
    let mut chars = text.chars();
    let truncated = chars
        .by_ref()
        .take(MAX_OPERATOR_TEXT_CHARS)
        .collect::<String>();
    if chars.next().is_some() {
        Value::String(format!("{truncated}…"))
    } else {
        Value::String(text)
    }
}

fn maybe(value: &Value, fallback: &str) -> String {
    value
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| fallback.to_string())
}

fn take_string(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("{flag} requires a value"))
}

fn take_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(take_string(args, flag)?))
}

pub fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Ok(env::current_dir()
        .context("failed to resolve current directory")?
        .join(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_projection_removes_direct_thought() {
        let sanitized = sanitize_for_operator(json!({
            "rawResult": "private",
            "decision": {"action": "continue"},
        }));
        assert!(sanitized.get("rawResult").is_none());
        assert_eq!(sanitized["decision"]["action"], "continue");
    }

}
