use std::path::PathBuf;

use codex_app_server_protocol::ThreadEpiphanyReorientFinding;
use codex_app_server_protocol::ThreadEpiphanyRoleFinding;
use codex_app_server_protocol::ThreadEpiphanyRoleId;
use codex_app_server_protocol::ThreadEpiphanyStateUpdatedField;
use codex_app_server_protocol::ThreadEpiphanyUpdatePatch;
use codex_core::EpiphanyStateUpdate;
use codex_protocol::protocol::EpiphanyCodeRef;
use codex_protocol::protocol::EpiphanyInvestigationCheckpoint;
use codex_protocol::protocol::EpiphanyInvestigationDisposition;
use codex_protocol::protocol::EpiphanyScratchPad;
use epiphany_core::EpiphanyReorientAcceptanceFinding;
use epiphany_core::EpiphanyRoleAcceptanceFinding;
use epiphany_core::EpiphanyRoleStatePatchDocument;
use epiphany_core::build_reorient_acceptance_bundle;
use epiphany_core::build_role_acceptance_bundle;
use epiphany_core::imagination_role_state_patch_policy_errors;
use epiphany_core::modeling_role_state_patch_policy_errors;

use crate::results::map_core_role_result_role_id;
use crate::runtime_results::reorient_finding_runtime_job_id;
use crate::runtime_results::reorient_finding_runtime_result_id;
use crate::runtime_results::role_finding_runtime_job_id;
use crate::runtime_results::role_finding_runtime_result_id;

pub struct RoleAcceptanceUpdate {
    pub patch: ThreadEpiphanyUpdatePatch,
    pub changed_fields: Vec<ThreadEpiphanyStateUpdatedField>,
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
}

pub struct ReorientAcceptanceUpdate {
    pub scratch: Option<EpiphanyScratchPad>,
    pub investigation_checkpoint: Option<EpiphanyInvestigationCheckpoint>,
    pub receipt: codex_protocol::protocol::EpiphanyAcceptanceReceipt,
    pub observation: codex_protocol::protocol::EpiphanyObservation,
    pub evidence: codex_protocol::protocol::EpiphanyEvidenceRecord,
    pub changed_fields: Vec<ThreadEpiphanyStateUpdatedField>,
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
}

pub fn parse_role_finding_state_patch(
    finding: &ThreadEpiphanyRoleFinding,
) -> Result<ThreadEpiphanyUpdatePatch, String> {
    finding
        .state_patch
        .clone()
        .ok_or_else(|| "completed role finding did not include a reviewable statePatch".to_string())
}

pub fn core_state_patch_from_protocol(
    patch: &ThreadEpiphanyUpdatePatch,
) -> EpiphanyRoleStatePatchDocument {
    EpiphanyRoleStatePatchDocument {
        objective: patch.objective.clone(),
        active_subgoal_id: patch.active_subgoal_id.clone(),
        subgoals: patch.subgoals.clone(),
        invariants: patch.invariants.clone(),
        graphs: patch.graphs.clone(),
        graph_frontier: patch.graph_frontier.clone(),
        graph_checkpoint: patch.graph_checkpoint.clone(),
        scratch: patch.scratch.clone(),
        investigation_checkpoint: patch.investigation_checkpoint.clone(),
        job_bindings: patch.job_bindings.clone(),
        acceptance_receipts: patch.acceptance_receipts.clone(),
        runtime_links: patch.runtime_links.clone(),
        observations: patch.observations.clone(),
        evidence: patch.evidence.clone(),
        churn: patch.churn.clone(),
        mode: patch.mode.clone(),
        planning: patch.planning.clone(),
    }
}

pub fn state_update_from_thread_patch(
    expected_revision: Option<u64>,
    patch: ThreadEpiphanyUpdatePatch,
) -> EpiphanyStateUpdate {
    EpiphanyStateUpdate {
        expected_revision,
        objective: patch.objective,
        active_subgoal_id: patch.active_subgoal_id,
        subgoals: patch.subgoals,
        invariants: patch.invariants,
        graphs: patch.graphs,
        graph_frontier: patch.graph_frontier,
        graph_checkpoint: patch.graph_checkpoint,
        scratch: patch.scratch,
        investigation_checkpoint: patch.investigation_checkpoint,
        job_bindings: patch.job_bindings,
        acceptance_receipts: patch.acceptance_receipts,
        runtime_links: patch.runtime_links,
        observations: patch.observations,
        evidence: patch.evidence,
        churn: patch.churn,
        mode: patch.mode,
        planning: patch.planning,
    }
}

pub fn imagination_role_accept_patch_errors(patch: &ThreadEpiphanyUpdatePatch) -> Vec<String> {
    imagination_role_state_patch_policy_errors(&core_state_patch_from_protocol(patch))
}

pub fn modeling_role_accept_patch_errors(patch: &ThreadEpiphanyUpdatePatch) -> Vec<String> {
    modeling_role_state_patch_policy_errors(&core_state_patch_from_protocol(patch))
}

pub fn epiphany_modeling_finding_has_reviewable_state_patch(
    finding: &ThreadEpiphanyRoleFinding,
) -> bool {
    finding.role_id == ThreadEpiphanyRoleId::Modeling
        && finding
            .state_patch
            .as_ref()
            .is_some_and(|patch| modeling_role_accept_patch_errors(patch).is_empty())
}

pub fn epiphany_imagination_finding_has_reviewable_state_patch(
    finding: &ThreadEpiphanyRoleFinding,
) -> bool {
    finding.role_id == ThreadEpiphanyRoleId::Imagination
        && finding
            .state_patch
            .as_ref()
            .is_some_and(|patch| imagination_role_accept_patch_errors(patch).is_empty())
}

pub fn build_role_acceptance_update(
    role_id: ThreadEpiphanyRoleId,
    binding_id: &str,
    finding: &ThreadEpiphanyRoleFinding,
    accepted_evidence_id: String,
    accepted_observation_id: String,
    accepted_at: String,
) -> Result<RoleAcceptanceUpdate, String> {
    let mut patch = match role_id {
        ThreadEpiphanyRoleId::Imagination => {
            let patch = parse_role_finding_state_patch(finding)?;
            let patch_errors = imagination_role_accept_patch_errors(&patch);
            if !patch_errors.is_empty() {
                return Err(format!(
                    "imagination role state patch is not acceptable: {}",
                    patch_errors.join("; ")
                ));
            }
            patch
        }
        ThreadEpiphanyRoleId::Modeling => {
            let patch = parse_role_finding_state_patch(finding)?;
            let patch_errors = modeling_role_accept_patch_errors(&patch);
            if !patch_errors.is_empty() {
                return Err(format!(
                    "modeling role state patch is not acceptable: {}",
                    patch_errors.join("; ")
                ));
            }
            patch
        }
        ThreadEpiphanyRoleId::Verification => ThreadEpiphanyUpdatePatch::default(),
        ThreadEpiphanyRoleId::Implementation | ThreadEpiphanyRoleId::Reorientation => {
            return Err(format!(
                "role {:?} cannot be accepted through roleAccept",
                role_id
            ));
        }
    };

    let projected_fields = epiphany_update_patch_changed_fields(&patch)
        .into_iter()
        .map(|field| format!("{field:?}"))
        .collect();
    let acceptance_bundle = build_role_acceptance_bundle(
        binding_id,
        EpiphanyRoleAcceptanceFinding {
            role_id: map_core_role_result_role_id(role_id),
            verdict: finding.verdict.clone(),
            summary: finding.summary.clone(),
            next_safe_move: finding.next_safe_move.clone(),
            files_inspected: finding.files_inspected.clone(),
            runtime_result_id: role_finding_runtime_result_id(finding),
            runtime_job_id: role_finding_runtime_job_id(finding),
            projected_fields,
        },
        accepted_evidence_id,
        accepted_observation_id,
        accepted_at,
    )?;
    let accepted_receipt_id = acceptance_bundle.accepted_receipt_id.clone();
    let accepted_observation_id = acceptance_bundle.accepted_observation_id.clone();
    let accepted_evidence_id = acceptance_bundle.accepted_evidence_id.clone();
    patch.evidence.push(acceptance_bundle.evidence);
    patch.observations.push(acceptance_bundle.observation);
    patch.acceptance_receipts.push(acceptance_bundle.receipt);
    let changed_fields = epiphany_update_patch_changed_fields(&patch);

    Ok(RoleAcceptanceUpdate {
        patch,
        changed_fields,
        accepted_receipt_id,
        accepted_observation_id,
        accepted_evidence_id,
    })
}

pub fn build_reorient_acceptance_update(
    binding_id: &str,
    finding: &ThreadEpiphanyReorientFinding,
    accepted_evidence_id: String,
    accepted_observation_id: String,
    accepted_at: String,
    update_scratch: bool,
    update_investigation_checkpoint: bool,
    checkpoint: Option<EpiphanyInvestigationCheckpoint>,
) -> Result<ReorientAcceptanceUpdate, String> {
    let acceptance_bundle = build_reorient_acceptance_bundle(
        binding_id,
        EpiphanyReorientAcceptanceFinding {
            mode: finding.mode.clone(),
            summary: finding.summary.clone(),
            next_safe_move: finding.next_safe_move.clone(),
            checkpoint_still_valid: finding.checkpoint_still_valid,
            files_inspected: finding.files_inspected.clone(),
            runtime_result_id: reorient_finding_runtime_result_id(finding),
            runtime_job_id: reorient_finding_runtime_job_id(finding),
        },
        accepted_evidence_id,
        accepted_observation_id,
        accepted_at,
        update_scratch,
        update_investigation_checkpoint.then_some(checkpoint).flatten(),
    )?;
    let accepted_receipt_id = acceptance_bundle.accepted_receipt_id.clone();
    let accepted_observation_id = acceptance_bundle.accepted_observation_id.clone();
    let accepted_evidence_id = acceptance_bundle.accepted_evidence_id.clone();
    let scratch = acceptance_bundle.scratch;
    let investigation_checkpoint = acceptance_bundle.investigation_checkpoint;

    let mut changed_fields = vec![
        ThreadEpiphanyStateUpdatedField::AcceptanceReceipts,
        ThreadEpiphanyStateUpdatedField::Observations,
        ThreadEpiphanyStateUpdatedField::Evidence,
    ];
    if scratch.is_some() {
        changed_fields.push(ThreadEpiphanyStateUpdatedField::Scratch);
    }
    if investigation_checkpoint.is_some() {
        changed_fields.push(ThreadEpiphanyStateUpdatedField::InvestigationCheckpoint);
    }

    Ok(ReorientAcceptanceUpdate {
        scratch,
        investigation_checkpoint,
        receipt: acceptance_bundle.receipt,
        observation: acceptance_bundle.observation,
        evidence: acceptance_bundle.evidence,
        changed_fields,
        accepted_receipt_id,
        accepted_observation_id,
        accepted_evidence_id,
    })
}

pub fn role_finding_summary(finding: &ThreadEpiphanyRoleFinding) -> String {
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

pub fn reorient_finding_code_refs(
    finding: &ThreadEpiphanyReorientFinding,
) -> Vec<EpiphanyCodeRef> {
    finding
        .files_inspected
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

pub fn reorient_finding_scratch(
    binding_id: &str,
    finding: &ThreadEpiphanyReorientFinding,
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

pub fn reorient_finding_investigation_checkpoint(
    checkpoint: &EpiphanyInvestigationCheckpoint,
    evidence_id: &str,
    code_refs: &[EpiphanyCodeRef],
    finding: &ThreadEpiphanyReorientFinding,
) -> EpiphanyInvestigationCheckpoint {
    let mut checkpoint = checkpoint.clone();
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

pub fn epiphany_job_launch_changed_fields() -> Vec<ThreadEpiphanyStateUpdatedField> {
    vec![
        ThreadEpiphanyStateUpdatedField::JobBindings,
        ThreadEpiphanyStateUpdatedField::RuntimeLinks,
    ]
}

pub fn thread_epiphany_patch_has_state_replacements(patch: &ThreadEpiphanyUpdatePatch) -> bool {
    patch.objective.is_some()
        || patch.active_subgoal_id.is_some()
        || patch.subgoals.is_some()
        || patch.invariants.is_some()
        || patch.graphs.is_some()
        || patch.graph_frontier.is_some()
        || patch.graph_checkpoint.is_some()
        || patch.scratch.is_some()
        || patch.investigation_checkpoint.is_some()
        || patch.job_bindings.is_some()
        || !patch.acceptance_receipts.is_empty()
        || !patch.runtime_links.is_empty()
        || patch.churn.is_some()
        || patch.mode.is_some()
        || patch.planning.is_some()
}

pub fn epiphany_update_patch_changed_fields(
    patch: &ThreadEpiphanyUpdatePatch,
) -> Vec<ThreadEpiphanyStateUpdatedField> {
    let mut fields = Vec::new();
    if patch.objective.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::Objective);
    }
    if patch.active_subgoal_id.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::ActiveSubgoalId);
    }
    if patch.subgoals.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::Subgoals);
    }
    if patch.invariants.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::Invariants);
    }
    if patch.graphs.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::Graphs);
    }
    if patch.graph_frontier.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::GraphFrontier);
    }
    if patch.graph_checkpoint.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::GraphCheckpoint);
    }
    if patch.scratch.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::Scratch);
    }
    if patch.investigation_checkpoint.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::InvestigationCheckpoint);
    }
    if patch.job_bindings.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::JobBindings);
    }
    if !patch.acceptance_receipts.is_empty() {
        fields.push(ThreadEpiphanyStateUpdatedField::AcceptanceReceipts);
    }
    if !patch.runtime_links.is_empty() {
        fields.push(ThreadEpiphanyStateUpdatedField::RuntimeLinks);
    }
    if !patch.observations.is_empty() {
        fields.push(ThreadEpiphanyStateUpdatedField::Observations);
    }
    if !patch.evidence.is_empty() {
        fields.push(ThreadEpiphanyStateUpdatedField::Evidence);
    }
    if patch.churn.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::Churn);
    }
    if patch.mode.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::Mode);
    }
    if patch.planning.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::Planning);
    }
    fields
}

pub fn epiphany_promote_changed_fields(
    patch: &ThreadEpiphanyUpdatePatch,
) -> Vec<ThreadEpiphanyStateUpdatedField> {
    let mut fields = epiphany_update_patch_changed_fields(patch);
    if !fields.contains(&ThreadEpiphanyStateUpdatedField::Evidence) {
        fields.push(ThreadEpiphanyStateUpdatedField::Evidence);
    }
    fields
}
