use anyhow::{Context, Result, anyhow};
use epiphany_core::{
    EpiphanyCoordinatorInput, EpiphanyCrrcAction, EpiphanyCurrentWorkProjection,
    EpiphanyRoleBoardInput, RepoFrontierPlanningLifecycle, RepoFrontierPlanningLifecycleStage,
    RepoFrontierResearchLifecycle, RepoFrontierResearchLifecycleStage, derive_role_board,
};
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
    let latest_reorientation_decision = mind
        .as_ref()
        .and_then(|mind| mind.reorientation_decisions.last());
    let crrc_action = if latest_reorientation_decision
        .is_some_and(|decision| decision.mode == "regather")
    {
        EpiphanyCrrcAction::RegatherManually
    } else {
        EpiphanyCrrcAction::Continue
    };

    let roles = derive_role_board(EpiphanyRoleBoardInput {
        mind_present: mind.is_some(),
        current_work: current_work.clone(),
    });
    let coordinator = epiphany_core::recommend_coordinator_action(EpiphanyCoordinatorInput {
        mind_present: mind.is_some(),
        crrc_action,
        current_work: current_work.clone(),
    });
    Ok(sanitize_for_operator(json!({
        "threadId": thread_id,
        "read": {
            "source": "native",
            "mindStore": store_path,
            "mindPresent": mind.is_some(),
            "projectionDigest": mind.as_ref().map(|mind| mind.projection_digest.as_str()),
        },
        "roles": {"threadId": thread_id, "source": "native", "roles": roles},
        "currentWork": current_work,
        "coordinator": coordinator,
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
