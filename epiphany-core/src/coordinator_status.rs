use crate::{
    EpiphanyCoordinatorInput, EpiphanyCurrentWorkProjection, EpiphanyRoleBoardInput,
    RepoFrontierPlanningLifecycle, RepoFrontierPlanningLifecycleStage,
    RepoFrontierResearchLifecycle, RepoFrontierResearchLifecycleStage, derive_role_board,
};
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::{
    env,
    path::{Path, PathBuf},
};

const SEALED_DIRECT_THOUGHT_KEYS: &[&str] = &[
    "rawResult",
    "turns",
    "items",
    "inputTranscript",
    "activeTranscript",
];
const SEALED_LONG_TEXT_KEYS: &[&str] = &["note"];
const MAX_OPERATOR_TEXT_CHARS: usize = 1200;

pub fn native_coordinator_json(runtime_store: &Path, thread_id: &str) -> Result<Value> {
    let store_path = absolute_path(runtime_store)?;

    let projected_work = store_path
        .exists()
        .then(|| {
            crate::project_current_work(&store_path)
                .context("failed to derive current work from keyed Mind and runtime receipts")
        })
        .transpose()?;
    let mind_present = projected_work.is_some();
    let projection_digest = projected_work
        .as_ref()
        .map(|work| work.mind_projection_digest.clone());
    let current_work = projected_work.unwrap_or_else(empty_current_work);

    let roles = derive_role_board(EpiphanyRoleBoardInput {
        mind_present,
        current_work: current_work.clone(),
    });
    let coordinator = crate::recommend_coordinator_action(EpiphanyCoordinatorInput {
        mind_present,
        current_work: current_work.clone(),
    });
    Ok(sanitize_for_operator(json!({
        "threadId": thread_id,
        "read": {
            "source": "native",
            "mindStore": store_path,
            "mindPresent": mind_present,
            "projectionDigest": projection_digest,
        },
        "roles": {"threadId": thread_id, "source": "native", "roles": roles},
        "currentWork": current_work,
        "coordinator": coordinator,
    })))
}

fn empty_current_work() -> EpiphanyCurrentWorkProjection {
    EpiphanyCurrentWorkProjection {
        mind_projection_digest: String::new(),
        operator_regather_required: false,
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
