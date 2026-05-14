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
    pub state_patch: Option<EpiphanyRoleStatePatchDocument>,
    pub self_patch: Option<AgentSelfPatch>,
    pub self_persistence: Option<EpiphanyRoleSelfPersistenceReview>,
    pub job_error: Option<String>,
    pub item_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyRoleAcceptanceFinding {
    pub role_id: EpiphanyRoleResultRoleId,
    pub verdict: Option<String>,
    pub summary: Option<String>,
    pub next_safe_move: Option<String>,
    pub files_inspected: Vec<String>,
    pub runtime_result_id: Option<String>,
    pub runtime_job_id: Option<String>,
    pub projected_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyReorientAcceptanceFinding {
    pub mode: Option<String>,
    pub summary: Option<String>,
    pub next_safe_move: Option<String>,
    pub checkpoint_still_valid: Option<bool>,
    pub files_inspected: Vec<String>,
    pub runtime_result_id: Option<String>,
    pub runtime_job_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyReorientFindingInterpretation {
    pub mode: Option<String>,
    pub summary: Option<String>,
    pub next_safe_move: Option<String>,
    pub checkpoint_still_valid: Option<bool>,
    pub files_inspected: Vec<String>,
    pub frontier_node_ids: Vec<String>,
    pub evidence_ids: Vec<String>,
    pub artifact_refs: Vec<String>,
    pub runtime_result_id: Option<String>,
    pub runtime_job_id: Option<String>,
    pub job_error: Option<String>,
    pub item_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyAcceptanceBundle {
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
    pub evidence: EpiphanyEvidenceRecord,
    pub observation: EpiphanyObservation,
    pub receipt: EpiphanyAcceptanceReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyReorientAcceptanceBundle {
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
    pub evidence: EpiphanyEvidenceRecord,
    pub observation: EpiphanyObservation,
    pub receipt: EpiphanyAcceptanceReceipt,
    pub scratch: Option<EpiphanyScratchPad>,
    pub investigation_checkpoint: Option<EpiphanyInvestigationCheckpoint>,
}

pub fn interpret_role_finding(
    role_id: EpiphanyRoleResultRoleId,
    raw_result: &serde_json::Value,
    _legacy_state_patch_parse_error: Option<String>,
    job_error: Option<String>,
    item_error: Option<String>,
) -> EpiphanyRoleFindingInterpretation {
    let state_patch_result = raw_result
        .get("statePatch")
        .cloned()
        .map(serde_json::from_value::<EpiphanyRoleStatePatchDocument>);
    let state_patch = state_patch_result
        .as_ref()
        .and_then(|result| result.as_ref().ok().cloned());
    let state_patch_parse_error = state_patch_result
        .as_ref()
        .and_then(|result| result.as_ref().err().map(ToString::to_string));
    let (self_patch, self_persistence) = raw_result
        .get("selfPatch")
        .map(|patch| decode_role_self_patch(role_id, patch))
        .unwrap_or((None, None));
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

pub fn interpret_reorient_finding(
    raw_result: &serde_json::Value,
    job_error: Option<String>,
    item_error: Option<String>,
) -> EpiphanyReorientFindingInterpretation {
    EpiphanyReorientFindingInterpretation {
        mode: json_string_field(raw_result, "mode"),
        summary: json_string_field(raw_result, "summary"),
        next_safe_move: json_string_field(raw_result, "nextSafeMove"),
        checkpoint_still_valid: raw_result
            .get("checkpointStillValid")
            .and_then(serde_json::Value::as_bool),
        files_inspected: json_string_array_field(raw_result, "filesInspected"),
        frontier_node_ids: json_string_array_field(raw_result, "frontierNodeIds"),
        evidence_ids: json_string_array_field(raw_result, "evidenceIds"),
        artifact_refs: json_string_array_field(raw_result, "artifactRefs"),
        runtime_result_id: json_string_field(raw_result, "runtimeResultId"),
        runtime_job_id: json_string_field(raw_result, "runtimeJobId"),
        job_error,
        item_error,
    }
}

pub fn interpret_runtime_role_worker_result(
    role_id: EpiphanyRoleResultRoleId,
    result: &crate::EpiphanyRuntimeRoleWorkerResult,
) -> EpiphanyRoleFindingInterpretation {
    let state_patch_result = result.state_patch();
    let self_patch_result = result.self_patch();
    let state_patch = state_patch_result
        .as_ref()
        .ok()
        .and_then(|patch| patch.clone());
    let self_patch = self_patch_result
        .as_ref()
        .ok()
        .and_then(|patch| patch.clone());
    let item_error = merge_item_error(
        result.item_error.clone(),
        state_patch_result.as_ref().err().map(ToString::to_string),
    );
    let item_error = merge_item_error(
        item_error,
        self_patch_result.as_ref().err().map(ToString::to_string),
    );
    EpiphanyRoleFindingInterpretation {
        verdict: Some(result.verdict.clone()),
        summary: Some(result.summary.clone()),
        next_safe_move: empty_string_as_none(&result.next_safe_move),
        checkpoint_summary: result.checkpoint_summary.clone(),
        scratch_summary: result.scratch_summary.clone(),
        files_inspected: result.files_inspected.clone(),
        frontier_node_ids: result.frontier_node_ids.clone(),
        evidence_ids: result.evidence_ids.clone(),
        artifact_refs: result.artifact_refs.clone(),
        runtime_result_id: Some(result.result_id.clone()),
        runtime_job_id: Some(result.job_id.clone()),
        open_questions: result.open_questions.clone(),
        evidence_gaps: result.evidence_gaps.clone(),
        risks: result.risks.clone(),
        state_patch: state_patch.clone(),
        self_patch,
        self_persistence: None,
        job_error: None,
        item_error: match role_id {
            EpiphanyRoleResultRoleId::Modeling => merge_item_error(
                item_error,
                modeling_role_state_patch_error(
                    &serde_json::Value::Null,
                    state_patch.as_ref(),
                    None,
                ),
            ),
            EpiphanyRoleResultRoleId::Imagination => merge_item_error(
                item_error,
                imagination_role_state_patch_error(
                    &serde_json::Value::Null,
                    state_patch.as_ref(),
                    None,
                ),
            ),
            EpiphanyRoleResultRoleId::Implementation
            | EpiphanyRoleResultRoleId::Verification
            | EpiphanyRoleResultRoleId::Reorientation => item_error,
        },
    }
}

pub fn interpret_runtime_reorient_worker_result(
    result: &crate::EpiphanyRuntimeReorientWorkerResult,
) -> EpiphanyReorientFindingInterpretation {
    EpiphanyReorientFindingInterpretation {
        mode: Some(result.mode.clone()),
        summary: Some(result.summary.clone()),
        next_safe_move: Some(result.next_safe_move.clone()),
        checkpoint_still_valid: result.checkpoint_still_valid,
        files_inspected: result.files_inspected.clone(),
        frontier_node_ids: result.frontier_node_ids.clone(),
        evidence_ids: result.evidence_ids.clone(),
        artifact_refs: result.artifact_refs.clone(),
        runtime_result_id: Some(result.result_id.clone()),
        runtime_job_id: Some(result.job_id.clone()),
        job_error: None,
        item_error: result.item_error.clone(),
    }
}

pub fn build_role_acceptance_bundle(
    binding_id: &str,
    role: EpiphanyRoleAcceptanceFinding,
    accepted_evidence_id: String,
    accepted_observation_id: String,
    accepted_at: String,
) -> Result<EpiphanyAcceptanceBundle, String> {
    let accepted_kind = role_accepted_kind(role.role_id)?;
    let accepted_prefix = role_label(role.role_id)?;
    let runtime_result_id = role
        .runtime_result_id
        .clone()
        .ok_or_else(|| "cannot accept role finding without a runtimeResultId".to_string())?;
    let runtime_job_id = role
        .runtime_job_id
        .clone()
        .ok_or_else(|| "cannot accept role finding without a runtimeJobId".to_string())?;
    let accepted_receipt_id = format!("accept-{accepted_prefix}-{runtime_result_id}");
    let code_refs = role_finding_code_refs(role.role_id, &role.files_inspected);
    let evidence = EpiphanyEvidenceRecord {
        id: accepted_evidence_id.clone(),
        kind: accepted_kind.to_string(),
        status: "accepted".to_string(),
        summary: role_finding_summary(&role),
        code_refs: code_refs.clone(),
    };
    let observation = EpiphanyObservation {
        id: accepted_observation_id.clone(),
        summary: role_finding_observation_summary(&role),
        source_kind: accepted_kind.to_string(),
        status: "accepted".to_string(),
        code_refs,
        evidence_ids: vec![accepted_evidence_id.clone()],
    };
    let receipt = EpiphanyAcceptanceReceipt {
        id: accepted_receipt_id.clone(),
        result_id: runtime_result_id,
        job_id: runtime_job_id,
        binding_id: binding_id.to_string(),
        surface: "roleAccept".to_string(),
        role_id: accepted_prefix.to_string(),
        status: "accepted".to_string(),
        accepted_at,
        accepted_observation_id: Some(accepted_observation_id.clone()),
        accepted_evidence_id: Some(accepted_evidence_id.clone()),
        summary: role.summary.clone(),
    };
    Ok(EpiphanyAcceptanceBundle {
        accepted_receipt_id,
        accepted_observation_id,
        accepted_evidence_id,
        evidence,
        observation,
        receipt,
    })
}

pub fn build_reorient_acceptance_bundle(
    binding_id: &str,
    finding: EpiphanyReorientAcceptanceFinding,
    accepted_evidence_id: String,
    accepted_observation_id: String,
    accepted_at: String,
    update_scratch: bool,
    investigation_checkpoint: Option<EpiphanyInvestigationCheckpoint>,
) -> Result<EpiphanyReorientAcceptanceBundle, String> {
    let runtime_result_id = finding.runtime_result_id.clone().ok_or_else(|| {
        "cannot accept reorientation finding without a runtimeResultId".to_string()
    })?;
    let runtime_job_id = finding
        .runtime_job_id
        .clone()
        .ok_or_else(|| "cannot accept reorientation finding without a runtimeJobId".to_string())?;
    let accepted_receipt_id = format!("accept-reorient-{runtime_result_id}");
    let code_refs = reorient_finding_code_refs(&finding.files_inspected);
    let evidence = EpiphanyEvidenceRecord {
        id: accepted_evidence_id.clone(),
        kind: "reorient_result".to_string(),
        status: "accepted".to_string(),
        summary: reorient_finding_summary(&finding),
        code_refs: code_refs.clone(),
    };
    let observation = EpiphanyObservation {
        id: accepted_observation_id.clone(),
        summary: reorient_finding_observation_summary(&finding),
        source_kind: "reorient_result".to_string(),
        status: "accepted".to_string(),
        code_refs: code_refs.clone(),
        evidence_ids: vec![accepted_evidence_id.clone()],
    };
    let scratch = update_scratch.then(|| reorient_finding_scratch(binding_id, &finding));
    let investigation_checkpoint = investigation_checkpoint.map(|checkpoint| {
        reorient_finding_investigation_checkpoint(
            checkpoint,
            accepted_evidence_id.as_str(),
            code_refs.as_slice(),
            &finding,
        )
    });
    let receipt = EpiphanyAcceptanceReceipt {
        id: accepted_receipt_id.clone(),
        result_id: runtime_result_id,
        job_id: runtime_job_id,
        binding_id: binding_id.to_string(),
        surface: "reorientAccept".to_string(),
        role_id: "reorientation".to_string(),
        status: "accepted".to_string(),
        accepted_at,
        accepted_observation_id: Some(accepted_observation_id.clone()),
        accepted_evidence_id: Some(accepted_evidence_id.clone()),
        summary: finding.summary.clone(),
    };
    Ok(EpiphanyReorientAcceptanceBundle {
        accepted_receipt_id,
        accepted_observation_id,
        accepted_evidence_id,
        evidence,
        observation,
        receipt,
        scratch,
        investigation_checkpoint,
    })
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

pub fn role_accepted_kind(role_id: EpiphanyRoleResultRoleId) -> Result<&'static str, String> {
    match role_id {
        EpiphanyRoleResultRoleId::Imagination => Ok("planning_synthesis"),
        EpiphanyRoleResultRoleId::Modeling => Ok("modeling_result"),
        EpiphanyRoleResultRoleId::Verification => Ok("verification_result"),
        EpiphanyRoleResultRoleId::Implementation | EpiphanyRoleResultRoleId::Reorientation => Err(
            format!("role {role_id:?} cannot be accepted through roleAccept"),
        ),
    }
}

pub fn role_label(role_id: EpiphanyRoleResultRoleId) -> Result<&'static str, String> {
    match role_id {
        EpiphanyRoleResultRoleId::Imagination => Ok("imagination"),
        EpiphanyRoleResultRoleId::Modeling => Ok("modeling"),
        EpiphanyRoleResultRoleId::Verification => Ok("verification"),
        EpiphanyRoleResultRoleId::Implementation | EpiphanyRoleResultRoleId::Reorientation => Err(
            format!("role {role_id:?} cannot be accepted through roleAccept"),
        ),
    }
}

pub fn review_role_self_patch(
    role_id: EpiphanyRoleResultRoleId,
    patch: &serde_json::Value,
) -> EpiphanyRoleSelfPersistenceReview {
    let (expected_agent_id, target_path) = role_self_memory_target(role_id);
    let reasons = match decode_agent_self_patch(patch) {
        Ok(patch) => review_agent_self_patch_contract(expected_agent_id, &patch),
        Err(reason) => vec![reason],
    };

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

fn decode_role_self_patch(
    role_id: EpiphanyRoleResultRoleId,
    patch: &serde_json::Value,
) -> (
    Option<AgentSelfPatch>,
    Option<EpiphanyRoleSelfPersistenceReview>,
) {
    let review = review_role_self_patch(role_id, patch);
    let decoded = if review.status == EpiphanyRoleSelfPersistenceStatus::Accepted {
        decode_agent_self_patch(patch).ok()
    } else {
        None
    };
    (decoded, Some(review))
}

pub fn modeling_role_state_patch_policy_errors(
    patch: &EpiphanyRoleStatePatchDocument,
) -> Vec<String> {
    let mut errors = Vec::new();
    if patch.objective.is_some() {
        errors
            .push("objective changes are not allowed through modeling role acceptance".to_string());
    }
    if patch.active_subgoal_id.is_some() || patch.subgoals.is_some() {
        errors.push("subgoal changes are not allowed through modeling role acceptance".to_string());
    }
    if patch.invariants.is_some() {
        errors
            .push("invariant changes are not allowed through modeling role acceptance".to_string());
    }
    if patch.job_bindings.is_some() {
        errors.push(
            "job binding changes are not allowed through modeling role acceptance".to_string(),
        );
    }
    if !patch.acceptance_receipts.is_empty() {
        errors.push(
            "acceptance receipt changes are owned by roleAccept, not worker statePatch".to_string(),
        );
    }
    if !patch.runtime_links.is_empty() {
        errors.push(
            "runtime link changes are owned by launch/read-back surfaces, not worker statePatch"
                .to_string(),
        );
    }
    if patch.planning.is_some() {
        errors
            .push("planning changes are not allowed through modeling role acceptance".to_string());
    }
    if patch.churn.is_some() || patch.mode.is_some() {
        errors.push(
            "churn or mode changes are not allowed through modeling role acceptance".to_string(),
        );
    }
    if patch.graphs.is_none()
        && patch.graph_frontier.is_none()
        && patch.graph_checkpoint.is_none()
        && patch.scratch.is_none()
        && patch.investigation_checkpoint.is_none()
    {
        errors.push(
            "statePatch must include a modeling field: graphs, graphFrontier, graphCheckpoint, scratch, or investigationCheckpoint"
                .to_string(),
        );
    }
    errors
}

pub fn imagination_role_state_patch_policy_errors(
    patch: &EpiphanyRoleStatePatchDocument,
) -> Vec<String> {
    let mut errors = Vec::new();
    if patch.objective.is_some() {
        errors.push(
            "objective changes are not allowed through imagination role acceptance".to_string(),
        );
    }
    if patch.active_subgoal_id.is_some() || patch.subgoals.is_some() {
        errors.push(
            "subgoal changes are not allowed through imagination role acceptance".to_string(),
        );
    }
    if patch.invariants.is_some() {
        errors.push(
            "invariant changes are not allowed through imagination role acceptance".to_string(),
        );
    }
    if patch.graphs.is_some()
        || patch.graph_frontier.is_some()
        || patch.graph_checkpoint.is_some()
        || patch.investigation_checkpoint.is_some()
    {
        errors.push(
            "graph or checkpoint changes are not allowed through imagination role acceptance"
                .to_string(),
        );
    }
    if patch.scratch.is_some() {
        errors.push(
            "scratch changes are not allowed through imagination role acceptance".to_string(),
        );
    }
    if patch.job_bindings.is_some() {
        errors.push(
            "job binding changes are not allowed through imagination role acceptance".to_string(),
        );
    }
    if !patch.acceptance_receipts.is_empty() {
        errors.push(
            "acceptance receipt changes are owned by roleAccept, not worker statePatch".to_string(),
        );
    }
    if !patch.runtime_links.is_empty() {
        errors.push(
            "runtime link changes are owned by launch/read-back surfaces, not worker statePatch"
                .to_string(),
        );
    }
    if patch.churn.is_some() || patch.mode.is_some() {
        errors.push(
            "churn or mode changes are not allowed through imagination role acceptance".to_string(),
        );
    }
    let Some(planning) = patch.planning.as_ref() else {
        errors.push("statePatch must include planning changes".to_string());
        return errors;
    };
    if planning.objective_drafts.is_empty() {
        errors.push("planning patch must include at least one objective draft".to_string());
    }
    if !planning
        .objective_drafts
        .iter()
        .any(|draft| draft.status.eq_ignore_ascii_case("draft"))
    {
        errors.push(
            "planning patch must include at least one objective draft with status draft"
                .to_string(),
        );
    }
    for draft in &planning.objective_drafts {
        if draft.acceptance_criteria.is_empty() {
            errors.push(format!(
                "planning objective draft {:?} must include acceptance criteria",
                draft.id
            ));
        }
        if draft.review_gates.is_empty() {
            errors.push(format!(
                "planning objective draft {:?} must include review gates",
                draft.id
            ));
        }
    }
    errors
}

fn modeling_role_state_patch_error(
    raw_result: &serde_json::Value,
    state_patch: Option<&EpiphanyRoleStatePatchDocument>,
    parse_error: Option<String>,
) -> Option<String> {
    if raw_result.get("statePatch").is_none() {
        return Some("modeling result is not reviewable: missing required statePatch".to_string());
    };
    if let Some(error) = parse_error {
        return Some(format!(
            "modeling result is not reviewable: invalid statePatch ({error})"
        ));
    }
    let Some(state_patch) = state_patch else {
        return Some("modeling result is not reviewable: invalid statePatch".to_string());
    };
    let errors = modeling_role_state_patch_policy_errors(state_patch);
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
    state_patch: Option<&EpiphanyRoleStatePatchDocument>,
    parse_error: Option<String>,
) -> Option<String> {
    if raw_result.get("statePatch").is_none() {
        return Some(
            "imagination result is not reviewable: missing required statePatch".to_string(),
        );
    };
    if let Some(error) = parse_error {
        return Some(format!(
            "imagination result is not reviewable: invalid statePatch ({error})"
        ));
    }
    let Some(state_patch) = state_patch else {
        return Some("imagination result is not reviewable: invalid statePatch".to_string());
    };
    let errors = imagination_role_state_patch_policy_errors(state_patch);
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

fn empty_string_as_none(value: &str) -> Option<String> {
    (!value.trim().is_empty()).then(|| value.to_string())
}

fn role_finding_summary(finding: &EpiphanyRoleAcceptanceFinding) -> String {
    let summary = finding
        .summary
        .clone()
        .unwrap_or_else(|| "Role worker returned a structured finding.".to_string());
    if let Some(next_safe_move) = finding.next_safe_move.as_deref() {
        format!("{summary} Next safe move: {next_safe_move}")
    } else {
        summary
    }
}

fn role_finding_observation_summary(finding: &EpiphanyRoleAcceptanceFinding) -> String {
    let verdict = finding.verdict.as_deref().unwrap_or("unknown");
    let changed = if finding.projected_fields.is_empty() {
        "no typed state fields projected".to_string()
    } else {
        finding.projected_fields.join(", ")
    };
    format!(
        "Accepted {:?} role result with verdict {verdict}; projected fields: {changed}. {}",
        finding.role_id,
        role_finding_summary(finding)
    )
}

fn role_finding_code_refs(
    role_id: EpiphanyRoleResultRoleId,
    files_inspected: &[String],
) -> Vec<EpiphanyCodeRef> {
    files_inspected
        .iter()
        .filter(|path| !path.trim().is_empty())
        .map(|path| EpiphanyCodeRef {
            path: PathBuf::from(path),
            start_line: None,
            end_line: None,
            symbol: None,
            note: Some(format!("Inspected by accepted {role_id:?} role worker.")),
        })
        .collect()
}

fn reorient_finding_summary(finding: &EpiphanyReorientAcceptanceFinding) -> String {
    let summary = finding
        .summary
        .clone()
        .unwrap_or_else(|| "Reorientation worker returned a structured finding.".to_string());
    if let Some(next_safe_move) = finding.next_safe_move.as_deref() {
        format!("{summary} Next safe move: {next_safe_move}")
    } else {
        summary
    }
}

fn reorient_finding_observation_summary(finding: &EpiphanyReorientAcceptanceFinding) -> String {
    let mode = finding.mode.as_deref().unwrap_or("unknown");
    let validity = match finding.checkpoint_still_valid {
        Some(true) => "checkpoint still valid",
        Some(false) => "checkpoint requires regather",
        None => "checkpoint validity not reported",
    };
    format!(
        "Accepted {mode} reorientation result: {validity}. {}",
        reorient_finding_summary(finding)
    )
}

fn reorient_finding_code_refs(files_inspected: &[String]) -> Vec<EpiphanyCodeRef> {
    files_inspected
        .iter()
        .filter(|path| !path.trim().is_empty())
        .map(|path| EpiphanyCodeRef {
            path: PathBuf::from(path),
            start_line: None,
            end_line: None,
            symbol: None,
            note: Some("Inspected by accepted reorientation worker.".to_string()),
        })
        .collect()
}

fn reorient_finding_scratch(
    binding_id: &str,
    finding: &EpiphanyReorientAcceptanceFinding,
) -> EpiphanyScratchPad {
    let mode = finding.mode.as_deref().unwrap_or("unknown");
    let checkpoint_validity = match finding.checkpoint_still_valid {
        Some(true) => "valid",
        Some(false) => "invalid",
        None => "unknown",
    };
    EpiphanyScratchPad {
        summary: finding.summary.clone(),
        hypothesis: Some(format!(
            "Accepted {mode} reorientation finding from {binding_id}; checkpoint validity is {checkpoint_validity}."
        )),
        next_probe: finding.next_safe_move.clone(),
        notes: vec![format!(
            "Files inspected: {}",
            if finding.files_inspected.is_empty() {
                "none reported".to_string()
            } else {
                finding.files_inspected.join(", ")
            }
        )],
    }
}

fn reorient_finding_investigation_checkpoint(
    mut checkpoint: EpiphanyInvestigationCheckpoint,
    evidence_id: &str,
    code_refs: &[EpiphanyCodeRef],
    finding: &EpiphanyReorientAcceptanceFinding,
) -> EpiphanyInvestigationCheckpoint {
    checkpoint.summary = finding.summary.clone().or(checkpoint.summary);
    checkpoint.next_action = finding.next_safe_move.clone().or(checkpoint.next_action);
    checkpoint.disposition = EpiphanyInvestigationDisposition::ResumeReady;
    if !checkpoint
        .evidence_ids
        .iter()
        .any(|existing| existing == evidence_id)
    {
        checkpoint.evidence_ids.push(evidence_id.to_string());
    }
    for code_ref in code_refs {
        if !checkpoint
            .code_refs
            .iter()
            .any(|existing| existing.path == code_ref.path)
        {
            checkpoint.code_refs.push(code_ref.clone());
        }
    }
    checkpoint
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
    fn projects_reorient_structured_output() {
        let finding = interpret_reorient_finding(
            &serde_json::json!({
                "mode": "resume",
                "summary": "Checkpoint is still warm.",
                "nextSafeMove": "Continue the bounded cut.",
                "checkpointStillValid": true,
                "filesInspected": ["state/map.yaml"],
                "frontierNodeIds": ["node-1"],
                "evidenceIds": ["ev-1"],
                "artifactRefs": ["artifact:reorient"],
                "runtimeResultId": "result-1",
                "runtimeJobId": "job-1"
            }),
            None,
            None,
        );

        assert_eq!(finding.mode.as_deref(), Some("resume"));
        assert_eq!(finding.checkpoint_still_valid, Some(true));
        assert_eq!(finding.frontier_node_ids, vec!["node-1"]);
        assert_eq!(finding.runtime_job_id.as_deref(), Some("job-1"));
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
                        "objective_drafts": [{
                            "id": "draft-imagination-test",
                            "title": "Draft from imagination",
                            "summary": "Reviewable objective draft.",
                            "scope": {"includes": ["typed state"], "excludes": []},
                            "status": "draft",
                            "lane_plan": {},
                            "acceptance_criteria": ["A criterion"],
                            "review_gates": ["human review"]
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

    #[test]
    fn builds_role_acceptance_bundle_from_runtime_identity() {
        let bundle = build_role_acceptance_bundle(
            "binding-modeling",
            EpiphanyRoleAcceptanceFinding {
                role_id: EpiphanyRoleResultRoleId::Modeling,
                verdict: Some("checkpoint-ready".to_string()),
                summary: Some("The map is source-grounded.".to_string()),
                next_safe_move: Some("Run verification.".to_string()),
                files_inspected: vec!["src/lib.rs".to_string(), "".to_string()],
                runtime_result_id: Some("result-123".to_string()),
                runtime_job_id: Some("job-123".to_string()),
                projected_fields: vec!["Graphs".to_string(), "InvestigationCheckpoint".to_string()],
            },
            "ev-modeling-test".to_string(),
            "obs-modeling-test".to_string(),
            "2026-05-12T00:00:00Z".to_string(),
        )
        .expect("role acceptance bundle should build");

        assert_eq!(bundle.accepted_receipt_id, "accept-modeling-result-123");
        assert_eq!(bundle.evidence.kind, "modeling_result");
        assert_eq!(bundle.observation.source_kind, "modeling_result");
        assert_eq!(bundle.receipt.result_id, "result-123");
        assert_eq!(bundle.receipt.job_id, "job-123");
        assert_eq!(bundle.receipt.surface, "roleAccept");
        assert_eq!(bundle.receipt.role_id, "modeling");
        assert!(
            bundle
                .observation
                .summary
                .contains("Graphs, InvestigationCheckpoint")
        );
        assert_eq!(bundle.evidence.code_refs.len(), 1);
        assert_eq!(
            bundle.evidence.code_refs[0].path,
            PathBuf::from("src/lib.rs")
        );
        assert!(
            bundle.evidence.code_refs[0]
                .note
                .as_deref()
                .is_some_and(|note| note.contains("Modeling role worker"))
        );
    }

    #[test]
    fn builds_reorient_acceptance_bundle_with_scratch_and_checkpoint() {
        let checkpoint = EpiphanyInvestigationCheckpoint {
            checkpoint_id: "checkpoint-1".to_string(),
            kind: "implementation".to_string(),
            disposition: EpiphanyInvestigationDisposition::RegatherRequired,
            focus: "runtime acceptance".to_string(),
            summary: Some("Old summary".to_string()),
            next_action: Some("Old move".to_string()),
            captured_at_turn_id: Some("turn-1".to_string()),
            open_questions: vec!["Which result owns acceptance?".to_string()],
            code_refs: vec![EpiphanyCodeRef {
                path: PathBuf::from("src/old.rs"),
                start_line: None,
                end_line: None,
                symbol: None,
                note: None,
            }],
            evidence_ids: vec!["ev-old".to_string()],
        };

        let bundle = build_reorient_acceptance_bundle(
            "reorient-worker",
            EpiphanyReorientAcceptanceFinding {
                mode: Some("resume".to_string()),
                summary: Some("The checkpoint can resume.".to_string()),
                next_safe_move: Some("Continue the migration.".to_string()),
                checkpoint_still_valid: Some(true),
                files_inspected: vec!["src/new.rs".to_string()],
                runtime_result_id: Some("result-reorient".to_string()),
                runtime_job_id: Some("job-reorient".to_string()),
            },
            "ev-reorient-test".to_string(),
            "obs-reorient-test".to_string(),
            "2026-05-12T00:00:00Z".to_string(),
            true,
            Some(checkpoint),
        )
        .expect("reorient acceptance bundle should build");

        assert_eq!(
            bundle.accepted_receipt_id,
            "accept-reorient-result-reorient"
        );
        assert_eq!(bundle.receipt.surface, "reorientAccept");
        assert_eq!(bundle.evidence.kind, "reorient_result");
        assert!(
            bundle
                .observation
                .summary
                .contains("checkpoint still valid")
        );
        let scratch = bundle.scratch.expect("scratch should be updated");
        assert_eq!(
            scratch.summary.as_deref(),
            Some("The checkpoint can resume.")
        );
        assert_eq!(
            scratch.next_probe.as_deref(),
            Some("Continue the migration.")
        );
        let checkpoint = bundle
            .investigation_checkpoint
            .expect("checkpoint should be updated");
        assert_eq!(
            checkpoint.disposition,
            EpiphanyInvestigationDisposition::ResumeReady
        );
        assert_eq!(
            checkpoint.summary.as_deref(),
            Some("The checkpoint can resume.")
        );
        assert!(
            checkpoint
                .evidence_ids
                .iter()
                .any(|id| id == "ev-reorient-test")
        );
        assert!(checkpoint.code_refs.iter().any(|code_ref| {
            code_ref.path == PathBuf::from("src/new.rs")
                && code_ref
                    .note
                    .as_deref()
                    .is_some_and(|note| note.contains("reorientation worker"))
        }));
    }
}
use super::state_patch::EpiphanyRoleStatePatchDocument;
use crate::agent_memory::AgentSelfPatch;
use crate::agent_memory::decode_agent_self_patch;
use crate::agent_memory::review_agent_self_patch_contract;
use epiphany_state_model::EpiphanyAcceptanceReceipt;
use epiphany_state_model::EpiphanyCodeRef;
use epiphany_state_model::EpiphanyEvidenceRecord;
use epiphany_state_model::EpiphanyInvestigationCheckpoint;
use epiphany_state_model::EpiphanyInvestigationDisposition;
use epiphany_state_model::EpiphanyObservation;
use epiphany_state_model::EpiphanyScratchPad;
use std::path::PathBuf;
