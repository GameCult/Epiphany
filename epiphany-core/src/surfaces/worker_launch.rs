use epiphany_state_model::EpiphanyChurnState;
use epiphany_state_model::EpiphanyCodeRef;
use epiphany_state_model::EpiphanyEvidenceRecord;
use epiphany_state_model::EpiphanyGraphCheckpoint;
use epiphany_state_model::EpiphanyGraphFrontier;
use epiphany_state_model::EpiphanyGraphs;
use epiphany_state_model::EpiphanyInvariant;
use epiphany_state_model::EpiphanyInvestigationCheckpoint;
use epiphany_state_model::EpiphanyInvestigationDisposition;
use epiphany_state_model::EpiphanyJobKind;
use epiphany_state_model::EpiphanyObservation;
use epiphany_state_model::EpiphanyPlanningState;
use epiphany_state_model::EpiphanyScratchPad;
use epiphany_state_model::EpiphanySubgoal;
use epiphany_state_model::EpiphanyThreadState;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;

use crate::EpiphanyJobLaunchRequest;
use crate::EpiphanyReorientAction;
use crate::EpiphanyReorientDecision;
use crate::EpiphanyReorientFreshnessStatus;
use crate::EpiphanyReorientPressureLevel;
use crate::EpiphanyReorientReason;

pub const ROLE_WORKER_OUTPUT_CONTRACT_ID: &str = "epiphany.worker.role_result.v1";
pub const REORIENT_WORKER_OUTPUT_CONTRACT_ID: &str = "epiphany.worker.reorient_result.v0";
pub const REPO_FRONTIER_PROPOSAL_MODELING_CONTEXT_SCHEMA_VERSION: &str =
    "epiphany.worker.repo_frontier_proposal_modeling_context.v0";
pub const REPO_FRONTIER_PROPOSAL_MODELING_CONTEXT_CONTRACT: &str =
    "epiphany.repo_frontier_proposal_modeling_context.v0";
pub const REPO_MODEL_CLAIM_REPAIR_CONTEXT_SCHEMA_VERSION: &str =
    "epiphany.worker.repo_model_claim_repair_context.v0";
pub const REPO_MODEL_CLAIM_REPAIR_CONTEXT_CONTRACT: &str =
    "epiphany.repo_model_claim_repair_context.v0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "documentKind")]
pub enum EpiphanyWorkerLaunchDocument {
    Role(EpiphanyRoleWorkerLaunchDocument),
    Reorient(EpiphanyReorientWorkerLaunchDocument),
}

impl EpiphanyWorkerLaunchDocument {
    pub fn output_contract_id(&self) -> &'static str {
        match self {
            Self::Role(_) => ROLE_WORKER_OUTPUT_CONTRACT_ID,
            Self::Reorient(_) => REORIENT_WORKER_OUTPUT_CONTRACT_ID,
        }
    }

    pub fn document_kind(&self) -> &'static str {
        match self {
            Self::Role(_) => "role",
            Self::Reorient(_) => "reorient",
        }
    }

    pub fn thread_id(&self) -> &str {
        match self {
            Self::Role(document) => &document.thread_id,
            Self::Reorient(document) => &document.thread_id,
        }
    }

    pub fn dynamic_prompt_context(&self) -> Option<&str> {
        match self {
            Self::Role(document) => document.dynamic_prompt_context.as_deref(),
            Self::Reorient(document) => document.dynamic_prompt_context.as_deref(),
        }
        .map(str::trim)
        .filter(|context| !context.is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyRoleWorkerLaunchDocument {
    pub thread_id: String,
    pub role_id: String,
    pub state_revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamic_prompt_context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal_modeling_context: Option<RepoFrontierProposalModelingContextProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_repair_context: Option<RepoModelClaimRepairContextProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_subgoal_id: Option<String>,
    #[serde(default)]
    pub active_subgoals: Vec<EpiphanySubgoal>,
    #[serde(default)]
    pub active_graph_node_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub investigation_checkpoint: Option<EpiphanyInvestigationCheckpoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scratch: Option<EpiphanyScratchPad>,
    #[serde(default)]
    pub invariants: Vec<EpiphanyInvariant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graphs: Option<EpiphanyGraphs>,
    #[serde(default)]
    pub recent_evidence: Vec<EpiphanyEvidenceRecord>,
    #[serde(default)]
    pub recent_observations: Vec<EpiphanyObservation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_frontier: Option<EpiphanyGraphFrontier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_checkpoint: Option<EpiphanyGraphCheckpoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planning: Option<EpiphanyPlanningState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub churn: Option<EpiphanyChurnState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyReorientWorkerLaunchDocument {
    pub thread_id: String,
    pub mode: String,
    pub checkpoint_id: String,
    pub checkpoint_kind: String,
    pub checkpoint_disposition: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamic_prompt_context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_focus: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_summary: Option<String>,
    pub checkpoint_next_action: String,
    #[serde(default)]
    pub checkpoint_open_questions: Vec<String>,
    #[serde(default)]
    pub checkpoint_evidence_ids: Vec<String>,
    #[serde(default)]
    pub checkpoint_code_refs: Vec<EpiphanyCodeRef>,
    #[serde(default)]
    pub decision_reasons: Vec<String>,
    pub decision_note: String,
    pub pressure_level: String,
    pub retrieval_status: String,
    pub graph_status: String,
    pub watcher_status: String,
    #[serde(default)]
    pub checkpoint_dirty_paths: Vec<String>,
    #[serde(default)]
    pub checkpoint_changed_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scratch: Option<EpiphanyScratchPad>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graphs: Option<EpiphanyGraphs>,
    #[serde(default)]
    pub recent_evidence: Vec<EpiphanyEvidenceRecord>,
    #[serde(default)]
    pub recent_observations: Vec<EpiphanyObservation>,
    #[serde(default)]
    pub active_frontier_node_ids: Vec<String>,
    #[serde(default)]
    pub linked_subgoal_ids: Vec<String>,
    #[serde(default)]
    pub linked_graph_node_ids: Vec<String>,
}

pub struct EpiphanyReorientLaunchRequestInput<'a> {
    pub thread_id: &'a str,
    pub expected_revision: Option<u64>,
    pub max_runtime_seconds: Option<u64>,
    pub binding_id: &'a str,
    pub owner_role: &'a str,
    pub instruction: String,
    pub state: &'a EpiphanyThreadState,
    pub checkpoint: &'a EpiphanyInvestigationCheckpoint,
    pub decision: &'a EpiphanyReorientDecision,
    pub dynamic_prompt_context: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoFrontierProposalModelingContextProjection {
    pub schema_version: String,
    pub contract: String,
    pub request_id: String,
    pub proposal_id: String,
    pub proposal_payload_sha256: String,
    pub runtime_id: String,
    pub thread_id: String,
    pub repository: String,
    pub workspace: String,
    pub source_kind: crate::RepoFrontierProposalSourceKind,
    pub source_actor: String,
    pub source_ref: String,
    pub title: String,
    pub body: String,
    pub desired_outcome: String,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub scope_hints: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    pub private_state_included: bool,
    pub model_revision: u64,
    pub model_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoModelClaimRepairContextProjection {
    pub schema_version: String,
    pub contract: String,
    pub request_id: String,
    pub challenge_id: String,
    pub challenge_sha256: String,
    pub eyes_evidence_packet_id: String,
    pub eyes_evidence_packet_sha256: String,
    pub source_result_id: String,
    pub source_job_id: String,
    pub original_admission_receipt_id: String,
    pub current_admission_receipt_id: String,
    pub model_revision: u64,
    pub model_hash: String,
    pub target_claim_id: String,
    pub target_claim_sha256: String,
    pub runtime_id: String,
    pub thread_id: String,
    pub affected_frontier: Vec<crate::RepoModelClaimRepairFrontierRef>,
    pub requested_at: String,
}

impl RepoModelClaimRepairContextProjection {
    pub(crate) fn from_request(request: &crate::RepoModelClaimRepairRequest) -> Self {
        Self {
            schema_version: REPO_MODEL_CLAIM_REPAIR_CONTEXT_SCHEMA_VERSION.into(),
            contract: REPO_MODEL_CLAIM_REPAIR_CONTEXT_CONTRACT.into(),
            request_id: request.request_id.clone(),
            challenge_id: request.challenge_id.clone(),
            challenge_sha256: request.challenge_sha256.clone(),
            eyes_evidence_packet_id: request.eyes_evidence_packet_id.clone(),
            eyes_evidence_packet_sha256: request.eyes_evidence_packet_sha256.clone(),
            source_result_id: request.source_result_id.clone(),
            source_job_id: request.source_job_id.clone(),
            original_admission_receipt_id: request.original_admission_receipt_id.clone(),
            current_admission_receipt_id: request.current_admission_receipt_id.clone(),
            model_revision: request.model_revision,
            model_hash: request.model_hash.clone(),
            target_claim_id: request.target_claim_id.clone(),
            target_claim_sha256: request.target_claim_sha256.clone(),
            runtime_id: request.runtime_id.clone(),
            thread_id: request.thread_id.clone(),
            affected_frontier: request.affected_frontier.clone(),
            requested_at: request.requested_at.clone(),
        }
    }
}

pub fn build_reorient_job_launch_request(
    input: EpiphanyReorientLaunchRequestInput<'_>,
) -> EpiphanyJobLaunchRequest {
    let linked_subgoal_ids = active_subgoal_ids(input.state);
    let linked_graph_node_ids = unique_strings(
        active_graph_node_ids(input.state)
            .into_iter()
            .chain(input.decision.active_frontier_node_ids.iter().cloned()),
    );
    let launch_document =
        EpiphanyWorkerLaunchDocument::Reorient(build_reorient_worker_launch_document(
            input.thread_id,
            input.state,
            input.checkpoint,
            input.decision,
            linked_subgoal_ids.clone(),
            linked_graph_node_ids.clone(),
            input.dynamic_prompt_context,
        ));
    let output_contract_id = launch_document.output_contract_id().to_string();
    let organ_launch_contract = crate::default_launch_organ_contract(
        reorient_launch_authority_scope(input.decision.action),
        launch_document.document_kind(),
        &output_contract_id,
    );

    EpiphanyJobLaunchRequest {
        expected_revision: input.expected_revision,
        binding_id: input.binding_id.to_string(),
        kind: EpiphanyJobKind::Specialist,
        scope: reorient_launch_scope(input.decision.action).to_string(),
        owner_role: input.owner_role.to_string(),
        authority_scope: reorient_launch_authority_scope(input.decision.action).to_string(),
        linked_subgoal_ids,
        linked_graph_node_ids,
        instruction: input.instruction,
        launch_document,
        output_contract_id,
        organ_launch_contract,
        max_runtime_seconds: input.max_runtime_seconds,
        proposal_modeling_request_id: None,
        claim_repair_request_id: None,
    }
}

fn build_reorient_worker_launch_document(
    thread_id: &str,
    state: &EpiphanyThreadState,
    checkpoint: &EpiphanyInvestigationCheckpoint,
    decision: &EpiphanyReorientDecision,
    linked_subgoal_ids: Vec<String>,
    linked_graph_node_ids: Vec<String>,
    dynamic_prompt_context: Option<String>,
) -> EpiphanyReorientWorkerLaunchDocument {
    EpiphanyReorientWorkerLaunchDocument {
        thread_id: thread_id.to_string(),
        mode: reorient_action_label(decision.action).to_string(),
        checkpoint_id: checkpoint.checkpoint_id.clone(),
        checkpoint_kind: checkpoint.kind.clone(),
        checkpoint_disposition: investigation_disposition_label(checkpoint.disposition).to_string(),
        dynamic_prompt_context,
        checkpoint_focus: Some(checkpoint.focus.clone()),
        checkpoint_summary: checkpoint.summary.clone(),
        checkpoint_next_action: checkpoint
            .next_action
            .clone()
            .unwrap_or_else(|| decision.next_action.clone()),
        checkpoint_open_questions: checkpoint.open_questions.clone(),
        checkpoint_evidence_ids: checkpoint.evidence_ids.clone(),
        checkpoint_code_refs: checkpoint.code_refs.clone(),
        decision_reasons: decision
            .reasons
            .iter()
            .map(|reason| reorient_reason_label(*reason).to_string())
            .collect(),
        decision_note: decision.note.clone(),
        pressure_level: reorient_pressure_level_label(decision.pressure_level).to_string(),
        retrieval_status: reorient_freshness_status_label(decision.retrieval_status).to_string(),
        graph_status: reorient_freshness_status_label(decision.graph_status).to_string(),
        watcher_status: reorient_freshness_status_label(decision.watcher_status).to_string(),
        checkpoint_dirty_paths: decision
            .checkpoint_dirty_paths
            .iter()
            .map(path_to_display_string)
            .collect(),
        checkpoint_changed_paths: decision
            .checkpoint_changed_paths
            .iter()
            .map(path_to_display_string)
            .collect(),
        scratch: state.scratch.clone(),
        graphs: Some(state.graphs.clone()),
        recent_evidence: state.recent_evidence.iter().take(8).cloned().collect(),
        recent_observations: state.observations.iter().take(8).cloned().collect(),
        active_frontier_node_ids: decision.active_frontier_node_ids.clone(),
        linked_subgoal_ids,
        linked_graph_node_ids,
    }
}

fn active_subgoal_ids(state: &EpiphanyThreadState) -> Vec<String> {
    state
        .active_subgoal_id
        .clone()
        .map(|id| vec![id])
        .unwrap_or_default()
}

fn active_graph_node_ids(state: &EpiphanyThreadState) -> Vec<String> {
    state
        .graph_frontier
        .as_ref()
        .map(|frontier| frontier.active_node_ids.clone())
        .unwrap_or_default()
}

fn unique_strings(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut unique = Vec::new();
    for value in values {
        if !unique.iter().any(|existing| existing == &value) {
            unique.push(value);
        }
    }
    unique
}

fn path_to_display_string(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().to_string()
}

fn reorient_launch_authority_scope(action: EpiphanyReorientAction) -> &'static str {
    match action {
        EpiphanyReorientAction::Resume => "epiphany.reorient.resume",
        EpiphanyReorientAction::Regather => "epiphany.reorient.regather",
    }
}

fn reorient_launch_scope(action: EpiphanyReorientAction) -> &'static str {
    match action {
        EpiphanyReorientAction::Resume => "reorient-guided checkpoint resume",
        EpiphanyReorientAction::Regather => "reorient-guided checkpoint regather",
    }
}

fn reorient_action_label(action: EpiphanyReorientAction) -> &'static str {
    match action {
        EpiphanyReorientAction::Resume => "resume",
        EpiphanyReorientAction::Regather => "regather",
    }
}

fn investigation_disposition_label(disposition: EpiphanyInvestigationDisposition) -> &'static str {
    match disposition {
        EpiphanyInvestigationDisposition::ResumeReady => "resume_ready",
        EpiphanyInvestigationDisposition::RegatherRequired => "regather_required",
    }
}

fn reorient_reason_label(reason: EpiphanyReorientReason) -> &'static str {
    match reason {
        EpiphanyReorientReason::MissingState => "missingState",
        EpiphanyReorientReason::MissingCheckpoint => "missingCheckpoint",
        EpiphanyReorientReason::CheckpointRequestedRegather => "checkpointRequestedRegather",
        EpiphanyReorientReason::CheckpointPathsDirty => "checkpointPathsDirty",
        EpiphanyReorientReason::CheckpointPathsChanged => "checkpointPathsChanged",
        EpiphanyReorientReason::FrontierChanged => "frontierChanged",
        EpiphanyReorientReason::UnanchoredCheckpointWhileStateStale => {
            "unanchoredCheckpointWhileStateStale"
        }
        EpiphanyReorientReason::CheckpointReady => "checkpointReady",
    }
}

fn reorient_pressure_level_label(level: EpiphanyReorientPressureLevel) -> &'static str {
    match level {
        EpiphanyReorientPressureLevel::Unknown => "unknown",
        EpiphanyReorientPressureLevel::Low => "low",
        EpiphanyReorientPressureLevel::Medium => "medium",
        EpiphanyReorientPressureLevel::High => "high",
        EpiphanyReorientPressureLevel::Critical => "critical",
    }
}

fn reorient_freshness_status_label(status: EpiphanyReorientFreshnessStatus) -> &'static str {
    match status {
        EpiphanyReorientFreshnessStatus::Unknown => "unknown",
        EpiphanyReorientFreshnessStatus::Clean => "clean",
        EpiphanyReorientFreshnessStatus::Dirty => "dirty",
        EpiphanyReorientFreshnessStatus::Stale => "stale",
        EpiphanyReorientFreshnessStatus::Changed => "changed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_state_model::EpiphanyCodeRef;
    use epiphany_state_model::EpiphanyGraphFrontier;
    use std::path::PathBuf;

    #[test]
    fn reorient_launch_request_is_composed_from_typed_state() {
        let state = EpiphanyThreadState {
            revision: 7,
            active_subgoal_id: Some("sg-1".to_string()),
            graph_frontier: Some(EpiphanyGraphFrontier {
                active_node_ids: vec!["node-a".to_string()],
                active_edge_ids: Vec::new(),
                open_question_ids: Vec::new(),
                open_gap_ids: Vec::new(),
                dirty_paths: Vec::new(),
            }),
            ..Default::default()
        };
        let checkpoint = EpiphanyInvestigationCheckpoint {
            checkpoint_id: "ix-1".to_string(),
            kind: "implementation".to_string(),
            disposition: EpiphanyInvestigationDisposition::RegatherRequired,
            focus: "runtime seam".to_string(),
            summary: Some("resume here".to_string()),
            next_action: None,
            captured_at_turn_id: Some("turn-1".to_string()),
            open_questions: Vec::new(),
            code_refs: vec![EpiphanyCodeRef {
                path: PathBuf::from("src/lib.rs"),
                start_line: None,
                end_line: None,
                symbol: None,
                note: None,
            }],
            evidence_ids: vec!["ev-1".to_string()],
        };
        let decision = EpiphanyReorientDecision {
            action: EpiphanyReorientAction::Regather,
            checkpoint_status: crate::EpiphanyReorientCheckpointStatus::RegatherRequired,
            checkpoint_id: Some("ix-1".to_string()),
            pressure_level: EpiphanyReorientPressureLevel::High,
            retrieval_status: EpiphanyReorientFreshnessStatus::Stale,
            graph_status: EpiphanyReorientFreshnessStatus::Changed,
            watcher_status: EpiphanyReorientFreshnessStatus::Dirty,
            reasons: vec![EpiphanyReorientReason::FrontierChanged],
            checkpoint_dirty_paths: vec![PathBuf::from("src/lib.rs")],
            checkpoint_changed_paths: vec![PathBuf::from("src/main.rs")],
            active_frontier_node_ids: vec!["node-b".to_string()],
            next_action: "Regather the seam.".to_string(),
            note: "continuity moved".to_string(),
        };

        let request = build_reorient_job_launch_request(EpiphanyReorientLaunchRequestInput {
            thread_id: "thread-1",
            expected_revision: Some(state.revision),
            max_runtime_seconds: Some(60),
            binding_id: "reorient-worker",
            owner_role: "epiphany-reorient",
            instruction: "do the reorientation".to_string(),
            state: &state,
            checkpoint: &checkpoint,
            decision: &decision,
            dynamic_prompt_context: Some(
                "<epiphany_dynamic_context>\ncheckpoint weather\n</epiphany_dynamic_context>"
                    .to_string(),
            ),
        });

        assert_eq!(request.binding_id, "reorient-worker");
        assert_eq!(request.scope, "reorient-guided checkpoint regather");
        assert_eq!(request.authority_scope, "epiphany.reorient.regather");
        assert_eq!(request.linked_subgoal_ids, vec!["sg-1"]);
        assert_eq!(
            request.linked_graph_node_ids,
            vec!["node-a".to_string(), "node-b".to_string()]
        );
        let EpiphanyWorkerLaunchDocument::Reorient(document) = request.launch_document else {
            panic!("expected reorient launch document");
        };
        assert_eq!(document.mode, "regather");
        assert_eq!(document.checkpoint_next_action, "Regather the seam.");
        assert_eq!(document.decision_reasons, vec!["frontierChanged"]);
        assert_eq!(document.pressure_level, "high");
        assert_eq!(document.retrieval_status, "stale");
        assert_eq!(document.graph_status, "changed");
        assert_eq!(document.watcher_status, "dirty");
        assert_eq!(
            document.dynamic_prompt_context.as_deref(),
            Some("<epiphany_dynamic_context>\ncheckpoint weather\n</epiphany_dynamic_context>")
        );
    }
}
