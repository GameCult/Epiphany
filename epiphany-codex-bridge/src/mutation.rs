use codex_app_server_protocol::ThreadEpiphanyRoleId;
use codex_app_server_protocol::ThreadEpiphanyStateUpdatedField;
use codex_app_server_protocol::ThreadEpiphanyStateUpdatedNotification;
use codex_app_server_protocol::ThreadEpiphanyStateUpdatedSource;
use codex_app_server_protocol::ThreadEpiphanyUpdatePatch;
use epiphany_core::EpiphanyReorientAcceptanceFinding;
use epiphany_core::EpiphanyReorientFindingInterpretation as CoreEpiphanyReorientFinding;
use epiphany_core::EpiphanyRoleAcceptanceFinding;
use epiphany_core::EpiphanyRoleFindingInterpretation as CoreEpiphanyRoleFinding;
use epiphany_core::EpiphanyRoleStatePatchDocument;
use epiphany_core::EpiphanyStateUpdate;
use epiphany_core::build_reorient_acceptance_bundle;
use epiphany_core::build_role_acceptance_bundle;
use epiphany_core::imagination_role_state_patch_policy_errors;
use epiphany_core::modeling_role_state_patch_policy_errors;
use epiphany_state_model::EpiphanyInvestigationCheckpoint;

use crate::results::map_core_role_result_role_id;

pub struct RoleAcceptanceUpdate {
    pub patch: ThreadEpiphanyUpdatePatch,
    pub changed_fields: Vec<ThreadEpiphanyStateUpdatedField>,
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
}

pub struct ReorientAcceptanceUpdate {
    pub state_update: EpiphanyStateUpdate,
    pub changed_fields: Vec<ThreadEpiphanyStateUpdatedField>,
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
}

pub fn parse_core_role_finding_state_patch(
    finding: &CoreEpiphanyRoleFinding,
) -> Result<EpiphanyRoleStatePatchDocument, String> {
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

pub fn protocol_patch_from_core(
    patch: EpiphanyRoleStatePatchDocument,
) -> ThreadEpiphanyUpdatePatch {
    ThreadEpiphanyUpdatePatch {
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

pub fn epiphany_state_updated_notification(
    thread_id: String,
    source: ThreadEpiphanyStateUpdatedSource,
    revision: u64,
    changed_fields: Vec<ThreadEpiphanyStateUpdatedField>,
    epiphany_state: epiphany_state_model::EpiphanyThreadState,
) -> ThreadEpiphanyStateUpdatedNotification {
    ThreadEpiphanyStateUpdatedNotification {
        thread_id,
        source,
        revision,
        changed_fields,
        epiphany_state,
    }
}

pub fn imagination_role_accept_patch_errors(patch: &ThreadEpiphanyUpdatePatch) -> Vec<String> {
    imagination_role_state_patch_policy_errors(&core_state_patch_from_protocol(patch))
}

pub fn modeling_role_accept_patch_errors(patch: &ThreadEpiphanyUpdatePatch) -> Vec<String> {
    modeling_role_state_patch_policy_errors(&core_state_patch_from_protocol(patch))
}

pub fn build_role_acceptance_update(
    role_id: ThreadEpiphanyRoleId,
    binding_id: &str,
    finding: &CoreEpiphanyRoleFinding,
    accepted_evidence_id: String,
    accepted_observation_id: String,
    accepted_at: String,
) -> Result<RoleAcceptanceUpdate, String> {
    let mut core_patch = match role_id {
        ThreadEpiphanyRoleId::Imagination => {
            let patch = parse_core_role_finding_state_patch(finding)?;
            let patch_errors = imagination_role_state_patch_policy_errors(&patch);
            if !patch_errors.is_empty() {
                return Err(format!(
                    "imagination role state patch is not acceptable: {}",
                    patch_errors.join("; ")
                ));
            }
            patch
        }
        ThreadEpiphanyRoleId::Modeling => {
            let patch = parse_core_role_finding_state_patch(finding)?;
            let patch_errors = modeling_role_state_patch_policy_errors(&patch);
            if !patch_errors.is_empty() {
                return Err(format!(
                    "modeling role state patch is not acceptable: {}",
                    patch_errors.join("; ")
                ));
            }
            patch
        }
        ThreadEpiphanyRoleId::Verification => EpiphanyRoleStatePatchDocument::default(),
        ThreadEpiphanyRoleId::Implementation | ThreadEpiphanyRoleId::Reorientation => {
            return Err(format!(
                "role {:?} cannot be accepted through roleAccept",
                role_id
            ));
        }
    };

    let patch_for_projection = protocol_patch_from_core(core_patch.clone());
    let projected_fields = epiphany_update_patch_changed_fields(&patch_for_projection)
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
            runtime_result_id: finding.runtime_result_id.clone(),
            runtime_job_id: finding.runtime_job_id.clone(),
            projected_fields,
        },
        accepted_evidence_id,
        accepted_observation_id,
        accepted_at,
    )?;
    let accepted_receipt_id = acceptance_bundle.accepted_receipt_id.clone();
    let accepted_observation_id = acceptance_bundle.accepted_observation_id.clone();
    let accepted_evidence_id = acceptance_bundle.accepted_evidence_id.clone();
    core_patch.evidence.push(acceptance_bundle.evidence);
    core_patch.observations.push(acceptance_bundle.observation);
    core_patch
        .acceptance_receipts
        .push(acceptance_bundle.receipt);
    let patch = protocol_patch_from_core(core_patch);
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
    expected_revision: Option<u64>,
    binding_id: &str,
    finding: &CoreEpiphanyReorientFinding,
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
            runtime_result_id: finding.runtime_result_id.clone(),
            runtime_job_id: finding.runtime_job_id.clone(),
        },
        accepted_evidence_id,
        accepted_observation_id,
        accepted_at,
        update_scratch,
        update_investigation_checkpoint
            .then_some(checkpoint)
            .flatten(),
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
        state_update: EpiphanyStateUpdate {
            expected_revision,
            scratch,
            investigation_checkpoint,
            acceptance_receipts: vec![acceptance_bundle.receipt],
            observations: vec![acceptance_bundle.observation],
            evidence: vec![acceptance_bundle.evidence],
            ..Default::default()
        },
        changed_fields,
        accepted_receipt_id,
        accepted_observation_id,
        accepted_evidence_id,
    })
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
