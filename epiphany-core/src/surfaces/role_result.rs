use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyRoleResultRoleId {
    Implementation,
    Imagination,
    Research,
    Modeling,
    Verification,
    Reorientation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    pub research_decision: Option<crate::EpiphanyResearchDecision>,
    pub repo_model_mutation_proposal: Option<crate::EpiphanyRepoModelMutationProposal>,
    pub job_error: Option<String>,
    pub item_error: Option<String>,
    pub verification_request_id: Option<String>,
    pub frontier_route_id: Option<String>,
    pub proposal_modeling_request_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

pub fn interpret_runtime_role_worker_result(
    role_id: EpiphanyRoleResultRoleId,
    result: &crate::EpiphanyRuntimeRoleWorkerResult,
) -> EpiphanyRoleFindingInterpretation {
    let research_decision_result = result.research_decision();
    let repo_model_mutation_proposal_result = result.repo_model_mutation_proposal();
    let research_decision = research_decision_result
        .as_ref()
        .ok()
        .and_then(|decision| decision.clone());
    let repo_model_mutation_proposal = repo_model_mutation_proposal_result
        .as_ref()
        .ok()
        .and_then(|patch| patch.clone());
    let item_error = merge_item_error(
        result.item_error.clone(),
        research_decision_result
            .as_ref()
            .err()
            .map(ToString::to_string),
    );
    let item_error = merge_item_error(
        item_error,
        repo_model_mutation_proposal_result
            .as_ref()
            .err()
            .map(ToString::to_string),
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
        research_decision: research_decision.clone(),
        repo_model_mutation_proposal: repo_model_mutation_proposal.clone(),
        job_error: None,
        item_error: match role_id {
            EpiphanyRoleResultRoleId::Modeling => merge_item_error(
                item_error,
                modeling_role_result_error(
                    result.repo_model_mutation_proposal_msgpack.is_some(),
                    repo_model_mutation_proposal.as_ref(),
                    None,
                ),
            ),
            EpiphanyRoleResultRoleId::Imagination => item_error,
            EpiphanyRoleResultRoleId::Research => merge_item_error(
                item_error,
                research_role_decision_error(
                    result.research_decision_msgpack.is_some(),
                    research_decision.as_ref(),
                    None,
                ),
            ),
            EpiphanyRoleResultRoleId::Implementation
            | EpiphanyRoleResultRoleId::Verification
            | EpiphanyRoleResultRoleId::Reorientation => item_error,
        },
        verification_request_id: result.verification_request_id.clone(),
        frontier_route_id: result.frontier_route_id.clone(),
        proposal_modeling_request_id: result.proposal_modeling_request_id.clone(),
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

fn modeling_role_result_error(
    repo_model_mutation_proposal_present: bool,
    repo_model_mutation_proposal: Option<&crate::EpiphanyRepoModelMutationProposal>,
    repo_model_mutation_proposal_parse_error: Option<String>,
) -> Option<String> {
    if let Some(error) = repo_model_mutation_proposal_parse_error {
        return Some(format!(
            "modeling result is not reviewable: invalid RepoModel mutation proposal ({error})"
        ));
    }
    if repo_model_mutation_proposal_present && repo_model_mutation_proposal.is_none() {
        return Some(
            "modeling result is not reviewable: invalid RepoModel mutation proposal".to_string(),
        );
    }
    if let Some(proposal) = repo_model_mutation_proposal
        && let Err(error) = proposal.validate()
    {
        return Some(format!(
            "modeling result is not reviewable: invalid RepoModel mutation proposal ({error})"
        ));
    }
    None
}

fn research_role_decision_error(
    decision_present: bool,
    decision: Option<&crate::EpiphanyResearchDecision>,
    parse_error: Option<String>,
) -> Option<String> {
    if !decision_present {
        return Some(
            "research result is not reviewable: missing required researchDecision".to_string(),
        );
    }
    if let Some(error) = parse_error {
        return Some(format!(
            "research result is not reviewable: invalid researchDecision ({error})"
        ));
    }
    let Some(decision) = decision else {
        return Some("research result is not reviewable: invalid researchDecision".to_string());
    };
    decision
        .validate()
        .err()
        .map(|error| format!("research result is not reviewable: {error}"))
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
