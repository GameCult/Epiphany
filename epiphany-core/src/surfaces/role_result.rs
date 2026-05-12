#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyRoleResultRoleId {
    Implementation,
    Imagination,
    Modeling,
    Verification,
    Reorientation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyRoleSelfPersistenceStatus {
    Missing,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyRoleSelfPersistenceReview {
    pub status: EpiphanyRoleSelfPersistenceStatus,
    pub target_agent_id: Option<String>,
    pub target_path: Option<String>,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EpiphanyRoleFindingInterpretation {
    pub verdict: Option<String>,
    pub summary: Option<String>,
    pub next_safe_move: Option<String>,
    pub checkpoint_summary: Option<String>,
    pub scratch_summary: Option<String>,
    pub files_inspected: Vec<String>,
    pub frontier_node_ids: Vec<String>,
    pub evidence_ids: Vec<String>,
    pub artifact_refs: Vec<String>,
    pub runtime_result_id: Option<String>,
    pub runtime_job_id: Option<String>,
    pub open_questions: Vec<String>,
    pub evidence_gaps: Vec<String>,
    pub risks: Vec<String>,
    pub state_patch: Option<serde_json::Value>,
    pub self_patch: Option<serde_json::Value>,
    pub self_persistence: Option<EpiphanyRoleSelfPersistenceReview>,
    pub job_error: Option<String>,
    pub item_error: Option<String>,
}

pub fn interpret_role_finding(
    role_id: EpiphanyRoleResultRoleId,
    raw_result: &serde_json::Value,
    state_patch_parse_error: Option<String>,
    job_error: Option<String>,
    item_error: Option<String>,
) -> EpiphanyRoleFindingInterpretation {
    let state_patch = raw_result.get("statePatch").cloned();
    let self_patch = raw_result.get("selfPatch").cloned();
    let self_persistence = self_patch
        .as_ref()
        .map(|patch| review_role_self_patch(role_id, patch));
    let item_error = match role_id {
        EpiphanyRoleResultRoleId::Imagination => merge_item_error(
            item_error,
            imagination_role_state_patch_error(
                raw_result,
                state_patch.as_ref(),
                state_patch_parse_error,
            ),
        ),
        EpiphanyRoleResultRoleId::Modeling => merge_item_error(
            item_error,
            modeling_role_state_patch_error(
                raw_result,
                state_patch.as_ref(),
                state_patch_parse_error,
            ),
        ),
        EpiphanyRoleResultRoleId::Implementation
        | EpiphanyRoleResultRoleId::Verification
        | EpiphanyRoleResultRoleId::Reorientation => item_error,
    };

    EpiphanyRoleFindingInterpretation {
        verdict: json_string_field(raw_result, "verdict"),
        summary: json_string_field(raw_result, "summary"),
        next_safe_move: json_string_field(raw_result, "nextSafeMove"),
        checkpoint_summary: json_string_field(raw_result, "checkpointSummary"),
        scratch_summary: json_string_field(raw_result, "scratchSummary"),
        files_inspected: json_string_array_field(raw_result, "filesInspected"),
        frontier_node_ids: json_string_array_field(raw_result, "frontierNodeIds"),
        evidence_ids: json_string_array_field(raw_result, "evidenceIds"),
        artifact_refs: json_string_array_field(raw_result, "artifactRefs"),
        runtime_result_id: json_string_field(raw_result, "runtimeResultId"),
        runtime_job_id: json_string_field(raw_result, "runtimeJobId"),
        open_questions: json_string_array_field(raw_result, "openQuestions"),
        evidence_gaps: json_string_array_field(raw_result, "evidenceGaps"),
        risks: json_string_array_field(raw_result, "risks"),
        state_patch,
        self_patch,
        self_persistence,
        job_error,
        item_error,
    }
}

pub fn role_self_memory_target(role_id: EpiphanyRoleResultRoleId) -> (&'static str, &'static str) {
    match role_id {
        EpiphanyRoleResultRoleId::Imagination => (
            "epiphany.imagination",
            "state/agents/imagination.agent-state.json",
        ),
        EpiphanyRoleResultRoleId::Modeling => {
            ("epiphany.body", "state/agents/body.agent-state.json")
        }
        EpiphanyRoleResultRoleId::Verification => {
            ("epiphany.soul", "state/agents/soul.agent-state.json")
        }
        EpiphanyRoleResultRoleId::Implementation => {
            ("epiphany.hands", "state/agents/hands.agent-state.json")
        }
        EpiphanyRoleResultRoleId::Reorientation => {
            ("epiphany.life", "state/agents/life.agent-state.json")
        }
    }
}

pub fn review_role_self_patch(
    role_id: EpiphanyRoleResultRoleId,
    patch: &serde_json::Value,
) -> EpiphanyRoleSelfPersistenceReview {
    let (expected_agent_id, target_path) = role_self_memory_target(role_id);
    let mut reasons = Vec::new();
    let Some(object) = patch.as_object() else {
        return EpiphanyRoleSelfPersistenceReview {
            status: EpiphanyRoleSelfPersistenceStatus::Rejected,
            target_agent_id: Some(expected_agent_id.to_string()),
            target_path: Some(target_path.to_string()),
            reasons: vec!["selfPatch must be a JSON object".to_string()],
        };
    };

    let agent_id = object.get("agentId").and_then(serde_json::Value::as_str);
    match agent_id {
        Some(id) if id == expected_agent_id => {}
        Some(id) => reasons.push(format!(
            "selfPatch agentId {id:?} does not match this lane; expected {expected_agent_id:?}"
        )),
        None => reasons.push(format!(
            "selfPatch must include agentId {expected_agent_id:?}"
        )),
    }
    match object.get("reason").and_then(serde_json::Value::as_str) {
        Some(reason) if reason.trim().len() >= 16 && reason.len() <= 800 => {}
        Some(_) => reasons.push(
            "selfPatch reason must be a bounded explanation of at least 16 characters".to_string(),
        ),
        None => reasons.push(
            "selfPatch must include a reason explaining why this improves the lane".to_string(),
        ),
    }

    let allowed = [
        "agentId",
        "reason",
        "evidenceIds",
        "semanticMemories",
        "episodicMemories",
        "relationshipMemories",
        "goals",
        "values",
        "privateNotes",
    ];
    let forbidden = [
        "statePatch",
        "objective",
        "activeSubgoalId",
        "subgoals",
        "invariants",
        "graphs",
        "graphFrontier",
        "graphCheckpoint",
        "scratch",
        "investigationCheckpoint",
        "jobBindings",
        "planning",
        "churn",
        "mode",
        "codeEdits",
        "files",
        "authorityScope",
        "backendJobId",
        "rawResult",
    ];
    for key in object.keys() {
        if forbidden.contains(&key.as_str()) {
            reasons.push(format!(
                "selfPatch field {key:?} is project truth or authority; use statePatch, roleAccept, or another explicit control surface instead"
            ));
        } else if !allowed.contains(&key.as_str()) {
            reasons.push(format!(
                "selfPatch field {key:?} is not part of the bounded memory mutation contract"
            ));
        }
    }

    let mut mutation_count = 0usize;
    mutation_count += review_self_patch_memories(&mut reasons, object, "semanticMemories");
    mutation_count += review_self_patch_memories(&mut reasons, object, "episodicMemories");
    mutation_count += review_self_patch_memories(&mut reasons, object, "relationshipMemories");
    mutation_count += review_self_patch_goals(&mut reasons, object);
    mutation_count += review_self_patch_values(&mut reasons, object);
    mutation_count += review_self_patch_private_notes(&mut reasons, object);
    review_self_patch_string_array(&mut reasons, object, "evidenceIds", 16, 160);
    if mutation_count == 0 {
        reasons.push(
            "selfPatch must contain at least one semantic memory, episodic memory, relationship memory, goal, value, or private note"
                .to_string(),
        );
    }

    EpiphanyRoleSelfPersistenceReview {
        status: if reasons.is_empty() {
            EpiphanyRoleSelfPersistenceStatus::Accepted
        } else {
            EpiphanyRoleSelfPersistenceStatus::Rejected
        },
        target_agent_id: Some(expected_agent_id.to_string()),
        target_path: Some(target_path.to_string()),
        reasons,
    }
}

pub fn modeling_role_state_patch_policy_errors(patch: &serde_json::Value) -> Vec<String> {
    let mut errors = Vec::new();
    if has_field(patch, "objective") {
        errors
            .push("objective changes are not allowed through modeling role acceptance".to_string());
    }
    if has_field(patch, "activeSubgoalId") || has_field(patch, "subgoals") {
        errors.push("subgoal changes are not allowed through modeling role acceptance".to_string());
    }
    if has_field(patch, "invariants") {
        errors
            .push("invariant changes are not allowed through modeling role acceptance".to_string());
    }
    if has_field(patch, "jobBindings") {
        errors.push(
            "job binding changes are not allowed through modeling role acceptance".to_string(),
        );
    }
    if non_empty_array_field(patch, "acceptanceReceipts") {
        errors.push(
            "acceptance receipt changes are owned by roleAccept, not worker statePatch".to_string(),
        );
    }
    if non_empty_array_field(patch, "runtimeLinks") {
        errors.push(
            "runtime link changes are owned by launch/read-back surfaces, not worker statePatch"
                .to_string(),
        );
    }
    if has_field(patch, "planning") {
        errors
            .push("planning changes are not allowed through modeling role acceptance".to_string());
    }
    if has_field(patch, "churn") || has_field(patch, "mode") {
        errors.push(
            "churn or mode changes are not allowed through modeling role acceptance".to_string(),
        );
    }
    if !has_field(patch, "graphs")
        && !has_field(patch, "graphFrontier")
        && !has_field(patch, "graphCheckpoint")
        && !has_field(patch, "scratch")
        && !has_field(patch, "investigationCheckpoint")
    {
        errors.push(
            "statePatch must include a modeling field: graphs, graphFrontier, graphCheckpoint, scratch, or investigationCheckpoint"
                .to_string(),
        );
    }
    errors
}

pub fn imagination_role_state_patch_policy_errors(patch: &serde_json::Value) -> Vec<String> {
    let mut errors = Vec::new();
    if has_field(patch, "objective") {
        errors.push(
            "objective changes are not allowed through imagination role acceptance".to_string(),
        );
    }
    if has_field(patch, "activeSubgoalId") || has_field(patch, "subgoals") {
        errors.push(
            "subgoal changes are not allowed through imagination role acceptance".to_string(),
        );
    }
    if has_field(patch, "invariants") {
        errors.push(
            "invariant changes are not allowed through imagination role acceptance".to_string(),
        );
    }
    if has_field(patch, "graphs")
        || has_field(patch, "graphFrontier")
        || has_field(patch, "graphCheckpoint")
        || has_field(patch, "investigationCheckpoint")
    {
        errors.push(
            "graph or checkpoint changes are not allowed through imagination role acceptance"
                .to_string(),
        );
    }
    if has_field(patch, "scratch") {
        errors.push(
            "scratch changes are not allowed through imagination role acceptance".to_string(),
        );
    }
    if has_field(patch, "jobBindings") {
        errors.push(
            "job binding changes are not allowed through imagination role acceptance".to_string(),
        );
    }
    if non_empty_array_field(patch, "acceptanceReceipts") {
        errors.push(
            "acceptance receipt changes are owned by roleAccept, not worker statePatch".to_string(),
        );
    }
    if non_empty_array_field(patch, "runtimeLinks") {
        errors.push(
            "runtime link changes are owned by launch/read-back surfaces, not worker statePatch"
                .to_string(),
        );
    }
    if has_field(patch, "churn") || has_field(patch, "mode") {
        errors.push(
            "churn or mode changes are not allowed through imagination role acceptance".to_string(),
        );
    }
    let Some(planning) = patch.get("planning") else {
        errors.push("statePatch must include planning changes".to_string());
        return errors;
    };
    let objective_drafts = get_field(planning, "objectiveDrafts", "objective_drafts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    if objective_drafts.is_empty() {
        errors.push("planning patch must include at least one objective draft".to_string());
    }
    if !objective_drafts.iter().any(|draft| {
        draft
            .get("status")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|status| status.eq_ignore_ascii_case("draft"))
    }) {
        errors.push(
            "planning patch must include at least one objective draft with status draft"
                .to_string(),
        );
    }
    for draft in &objective_drafts {
        let id = draft
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("<missing>");
        if !draft
            .get("acceptanceCriteria")
            .or_else(|| draft.get("acceptance_criteria"))
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| !items.is_empty())
        {
            errors.push(format!(
                "planning objective draft {:?} must include acceptance criteria",
                id
            ));
        }
        if !draft
            .get("reviewGates")
            .or_else(|| draft.get("review_gates"))
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| !items.is_empty())
        {
            errors.push(format!(
                "planning objective draft {:?} must include review gates",
                id
            ));
        }
    }
    errors
}

fn modeling_role_state_patch_error(
    raw_result: &serde_json::Value,
    state_patch: Option<&serde_json::Value>,
    parse_error: Option<String>,
) -> Option<String> {
    let Some(value) = raw_result.get("statePatch") else {
        return Some("modeling result is not reviewable: missing required statePatch".to_string());
    };
    if let Some(error) = parse_error {
        return Some(format!(
            "modeling result is not reviewable: invalid statePatch ({error})"
        ));
    }
    let errors = modeling_role_state_patch_policy_errors(state_patch.unwrap_or(value));
    if errors.is_empty() {
        None
    } else {
        Some(format!(
            "modeling result is not reviewable: {}",
            errors.join("; ")
        ))
    }
}

fn imagination_role_state_patch_error(
    raw_result: &serde_json::Value,
    state_patch: Option<&serde_json::Value>,
    parse_error: Option<String>,
) -> Option<String> {
    let Some(value) = raw_result.get("statePatch") else {
        return Some(
            "imagination result is not reviewable: missing required statePatch".to_string(),
        );
    };
    if let Some(error) = parse_error {
        return Some(format!(
            "imagination result is not reviewable: invalid statePatch ({error})"
        ));
    }
    let errors = imagination_role_state_patch_policy_errors(state_patch.unwrap_or(value));
    if errors.is_empty() {
        None
    } else {
        Some(format!(
            "imagination result is not reviewable: {}",
            errors.join("; ")
        ))
    }
}

fn merge_item_error(item_error: Option<String>, extra_error: Option<String>) -> Option<String> {
    match (item_error, extra_error) {
        (Some(existing), Some(extra)) => Some(format!("{existing}; {extra}")),
        (Some(existing), None) => Some(existing),
        (None, Some(extra)) => Some(extra),
        (None, None) => None,
    }
}

fn json_string_field(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn json_string_array_field(value: &serde_json::Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn has_field(value: &serde_json::Value, key: &str) -> bool {
    value.get(key).is_some_and(|field| !field.is_null())
}

fn get_field<'a>(
    value: &'a serde_json::Value,
    camel_key: &str,
    snake_key: &str,
) -> Option<&'a serde_json::Value> {
    value.get(camel_key).or_else(|| value.get(snake_key))
}

fn non_empty_array_field(value: &serde_json::Value, key: &str) -> bool {
    value
        .get(key)
        .and_then(serde_json::Value::as_array)
        .is_some_and(|items| !items.is_empty())
}

fn review_self_patch_memories(
    reasons: &mut Vec<String>,
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> usize {
    let Some(value) = object.get(field) else {
        return 0;
    };
    let Some(items) = value.as_array() else {
        reasons.push(format!("selfPatch {field} must be an array"));
        return 0;
    };
    if items.len() > 8 {
        reasons.push(format!("selfPatch {field} may contain at most 8 records"));
    }
    let mut valid = 0usize;
    for (index, item) in items.iter().enumerate() {
        let Some(record) = item.as_object() else {
            reasons.push(format!("selfPatch {field}[{index}] must be an object"));
            continue;
        };
        review_self_patch_id(reasons, record, field, index, "memoryId", "mem-");
        review_self_patch_summary(reasons, record, field, index, "summary", 600);
        review_self_patch_unit(reasons, record, field, index, "salience");
        review_self_patch_unit(reasons, record, field, index, "confidence");
        valid += 1;
    }
    valid
}

fn review_self_patch_goals(
    reasons: &mut Vec<String>,
    object: &serde_json::Map<String, serde_json::Value>,
) -> usize {
    let Some(value) = object.get("goals") else {
        return 0;
    };
    let Some(items) = value.as_array() else {
        reasons.push("selfPatch goals must be an array".to_string());
        return 0;
    };
    if items.len() > 6 {
        reasons.push("selfPatch goals may contain at most 6 records".to_string());
    }
    let mut valid = 0usize;
    for (index, item) in items.iter().enumerate() {
        let Some(record) = item.as_object() else {
            reasons.push(format!("selfPatch goals[{index}] must be an object"));
            continue;
        };
        review_self_patch_id(reasons, record, "goals", index, "goalId", "goal-");
        review_self_patch_summary(reasons, record, "goals", index, "description", 700);
        review_self_patch_string(reasons, record, "goals", index, "scope", 80);
        review_self_patch_unit(reasons, record, "goals", index, "priority");
        review_self_patch_string(reasons, record, "goals", index, "emotionalStake", 400);
        review_self_patch_string(reasons, record, "goals", index, "status", 80);
        valid += 1;
    }
    valid
}

fn review_self_patch_values(
    reasons: &mut Vec<String>,
    object: &serde_json::Map<String, serde_json::Value>,
) -> usize {
    let Some(value) = object.get("values") else {
        return 0;
    };
    let Some(items) = value.as_array() else {
        reasons.push("selfPatch values must be an array".to_string());
        return 0;
    };
    if items.len() > 6 {
        reasons.push("selfPatch values may contain at most 6 records".to_string());
    }
    let mut valid = 0usize;
    for (index, item) in items.iter().enumerate() {
        let Some(record) = item.as_object() else {
            reasons.push(format!("selfPatch values[{index}] must be an object"));
            continue;
        };
        review_self_patch_id(reasons, record, "values", index, "valueId", "value-");
        review_self_patch_summary(reasons, record, "values", index, "label", 240);
        review_self_patch_unit(reasons, record, "values", index, "priority");
        if !record
            .get("unforgivableIfBetrayed")
            .is_some_and(serde_json::Value::is_boolean)
        {
            reasons.push(format!(
                "selfPatch values[{index}].unforgivableIfBetrayed must be a boolean"
            ));
        }
        valid += 1;
    }
    valid
}

fn review_self_patch_private_notes(
    reasons: &mut Vec<String>,
    object: &serde_json::Map<String, serde_json::Value>,
) -> usize {
    let Some(value) = object.get("privateNotes") else {
        return 0;
    };
    let Some(items) = value.as_array() else {
        reasons.push("selfPatch privateNotes must be an array".to_string());
        return 0;
    };
    if items.len() > 6 {
        reasons.push("selfPatch privateNotes may contain at most 6 records".to_string());
    }
    for (index, item) in items.iter().enumerate() {
        match item.as_str() {
            Some(text) if !text.trim().is_empty() && text.len() <= 600 => {}
            _ => reasons.push(format!(
                "selfPatch privateNotes[{index}] must be non-empty text under 600 characters"
            )),
        }
    }
    items.len()
}

fn review_self_patch_string_array(
    reasons: &mut Vec<String>,
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    max_items: usize,
    max_len: usize,
) {
    let Some(value) = object.get(field) else {
        return;
    };
    let Some(items) = value.as_array() else {
        reasons.push(format!("selfPatch {field} must be an array"));
        return;
    };
    if items.len() > max_items {
        reasons.push(format!(
            "selfPatch {field} may contain at most {max_items} records"
        ));
    }
    for (index, item) in items.iter().enumerate() {
        match item.as_str() {
            Some(text) if !text.trim().is_empty() && text.len() <= max_len => {}
            _ => reasons.push(format!(
                "selfPatch {field}[{index}] must be non-empty text under {max_len} characters"
            )),
        }
    }
}

fn review_self_patch_id(
    reasons: &mut Vec<String>,
    record: &serde_json::Map<String, serde_json::Value>,
    collection: &str,
    index: usize,
    field: &str,
    prefix: &str,
) {
    match record.get(field).and_then(serde_json::Value::as_str) {
        Some(id)
            if id.starts_with(prefix)
                && id.len() <= 120
                && id
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || "-_.".contains(ch)) => {}
        Some(_) => reasons.push(format!(
            "selfPatch {collection}[{index}].{field} must start with {prefix:?}, avoid whitespace, and stay under 120 characters"
        )),
        None => reasons.push(format!(
            "selfPatch {collection}[{index}].{field} is required"
        )),
    }
}

fn review_self_patch_summary(
    reasons: &mut Vec<String>,
    record: &serde_json::Map<String, serde_json::Value>,
    collection: &str,
    index: usize,
    field: &str,
    max_len: usize,
) {
    match record.get(field).and_then(serde_json::Value::as_str) {
        Some(text) if !text.trim().is_empty() && text.len() <= max_len => {}
        Some(_) => reasons.push(format!(
            "selfPatch {collection}[{index}].{field} must be non-empty text under {max_len} characters"
        )),
        None => reasons.push(format!(
            "selfPatch {collection}[{index}].{field} is required"
        )),
    }
}

fn review_self_patch_string(
    reasons: &mut Vec<String>,
    record: &serde_json::Map<String, serde_json::Value>,
    collection: &str,
    index: usize,
    field: &str,
    max_len: usize,
) {
    match record.get(field).and_then(serde_json::Value::as_str) {
        Some(text) if !text.trim().is_empty() && text.len() <= max_len => {}
        Some(_) => reasons.push(format!(
            "selfPatch {collection}[{index}].{field} must be non-empty text under {max_len} characters"
        )),
        None => reasons.push(format!(
            "selfPatch {collection}[{index}].{field} is required"
        )),
    }
}

fn review_self_patch_unit(
    reasons: &mut Vec<String>,
    record: &serde_json::Map<String, serde_json::Value>,
    collection: &str,
    index: usize,
    field: &str,
) {
    match record.get(field).and_then(serde_json::Value::as_f64) {
        Some(value) if (0.0..=1.0).contains(&value) => {}
        Some(_) => reasons.push(format!(
            "selfPatch {collection}[{index}].{field} must be between 0 and 1"
        )),
        None => reasons.push(format!(
            "selfPatch {collection}[{index}].{field} is required"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_structured_output() {
        let finding = interpret_role_finding(
            EpiphanyRoleResultRoleId::Verification,
            &serde_json::json!({
                "verdict": "pass",
                "summary": "The evidence covers the bounded slice.",
                "nextSafeMove": "Promote after human review.",
                "filesInspected": ["src/lib.rs"],
                "frontierNodeIds": ["node-1"],
                "evidenceIds": ["ev-1"],
                "artifactRefs": ["artifact:role"],
                "runtimeResultId": "result-1",
                "runtimeJobId": "job-1"
            }),
            None,
            None,
            None,
        );

        assert_eq!(finding.verdict.as_deref(), Some("pass"));
        assert_eq!(finding.files_inspected, vec!["src/lib.rs"]);
        assert_eq!(finding.runtime_result_id.as_deref(), Some("result-1"));
    }

    #[test]
    fn reviews_acceptable_self_patch() {
        let finding = interpret_role_finding(
            EpiphanyRoleResultRoleId::Modeling,
            &serde_json::json!({
                "statePatch": {"scratch": {"summary": "Source-grounded modeling checkpoint."}},
                "selfPatch": {
                    "agentId": "epiphany.body",
                    "reason": "The Body should remember graph growth must stay source-grounded and bounded.",
                    "semanticMemories": [{
                        "memoryId": "mem-body-source-grounded-growth",
                        "summary": "Grow graph and checkpoint state only when source evidence makes the anatomy harder to misread.",
                        "salience": 0.82,
                        "confidence": 0.9
                    }]
                }
            }),
            None,
            None,
            None,
        );

        let review = finding.self_persistence.as_ref().unwrap();
        assert_eq!(review.status, EpiphanyRoleSelfPersistenceStatus::Accepted);
        assert_eq!(review.target_agent_id.as_deref(), Some("epiphany.body"));
    }

    #[test]
    fn rejects_bad_self_patch() {
        let finding = interpret_role_finding(
            EpiphanyRoleResultRoleId::Verification,
            &serde_json::json!({
                "selfPatch": {
                    "agentId": "epiphany.body",
                    "reason": "Too broad.",
                    "graphs": {},
                    "semanticMemories": [{
                        "memoryId": "mem-soul-bad-project-truth",
                        "summary": "Project graph state belongs in memory now.",
                        "salience": 0.7,
                        "confidence": 0.4
                    }]
                }
            }),
            None,
            None,
            None,
        );

        let review = finding.self_persistence.as_ref().unwrap();
        assert_eq!(review.status, EpiphanyRoleSelfPersistenceStatus::Rejected);
        assert!(
            review
                .reasons
                .iter()
                .any(|reason| reason.contains("expected \"epiphany.soul\""))
        );
        assert!(
            review
                .reasons
                .iter()
                .any(|reason| reason.contains("project truth"))
        );
    }

    #[test]
    fn marks_modeling_without_patch_unreviewable() {
        let missing = interpret_role_finding(
            EpiphanyRoleResultRoleId::Modeling,
            &serde_json::json!({"verdict": "checkpoint-ready"}),
            None,
            None,
            None,
        );

        assert!(
            missing
                .item_error
                .as_deref()
                .is_some_and(|error| error.contains("missing required statePatch"))
        );

        let reviewable = interpret_role_finding(
            EpiphanyRoleResultRoleId::Modeling,
            &serde_json::json!({"statePatch": {"scratch": {"summary": "banked"}}}),
            None,
            None,
            None,
        );

        assert!(reviewable.item_error.is_none());
    }

    #[test]
    fn marks_imagination_without_planning_patch_unreviewable() {
        let missing = interpret_role_finding(
            EpiphanyRoleResultRoleId::Imagination,
            &serde_json::json!({"verdict": "draft-ready"}),
            None,
            None,
            None,
        );

        assert!(
            missing
                .item_error
                .as_deref()
                .is_some_and(|error| error.contains("missing required statePatch"))
        );

        let reviewable = interpret_role_finding(
            EpiphanyRoleResultRoleId::Imagination,
            &serde_json::json!({
                "statePatch": {
                    "planning": {
                        "objectiveDrafts": [{
                            "id": "draft-imagination-test",
                            "status": "draft",
                            "acceptanceCriteria": ["A criterion"],
                            "reviewGates": ["human review"]
                        }]
                    }
                }
            }),
            None,
            None,
            None,
        );

        assert!(reviewable.item_error.is_none());
    }
}
