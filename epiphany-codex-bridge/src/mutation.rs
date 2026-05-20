use epiphany_core::EpiphanyReorientAcceptanceFinding;
use epiphany_core::EpiphanyReorientFindingInterpretation as CoreEpiphanyReorientFinding;
use epiphany_core::EpiphanyRoleAcceptanceFinding;
use epiphany_core::EpiphanyRoleFindingInterpretation as CoreEpiphanyRoleFinding;
use epiphany_core::EpiphanyRoleResultRoleId;
use epiphany_core::EpiphanyRoleStatePatchDocument;
use epiphany_core::EpiphanyStateUpdate;
use epiphany_core::EpiphanyStateUpdatedField;
use epiphany_core::build_reorient_acceptance_bundle;
use epiphany_core::build_role_acceptance_bundle;
use epiphany_core::imagination_role_state_patch_policy_errors;
use epiphany_core::modeling_role_state_patch_policy_errors;
use epiphany_state_model::EpiphanyInvestigationCheckpoint;

pub struct RoleAcceptanceUpdate {
    pub state_update: EpiphanyStateUpdate,
    pub applied_patch: EpiphanyRoleStatePatchDocument,
    pub changed_fields: Vec<EpiphanyStateUpdatedField>,
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
}

pub struct ReorientAcceptanceUpdate {
    pub state_update: EpiphanyStateUpdate,
    pub changed_fields: Vec<EpiphanyStateUpdatedField>,
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

pub fn state_update_from_core_patch(
    expected_revision: Option<u64>,
    patch: EpiphanyRoleStatePatchDocument,
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

pub fn build_role_acceptance_update(
    expected_revision: Option<u64>,
    role_id: EpiphanyRoleResultRoleId,
    binding_id: &str,
    finding: &CoreEpiphanyRoleFinding,
    accepted_evidence_id: String,
    accepted_observation_id: String,
    accepted_at: String,
) -> Result<RoleAcceptanceUpdate, String> {
    let mut core_patch = match role_id {
        EpiphanyRoleResultRoleId::Imagination => {
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
        EpiphanyRoleResultRoleId::Modeling => {
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
        EpiphanyRoleResultRoleId::Verification => EpiphanyRoleStatePatchDocument::default(),
        EpiphanyRoleResultRoleId::Implementation | EpiphanyRoleResultRoleId::Reorientation => {
            return Err(format!(
                "role {:?} cannot be accepted through roleAccept",
                role_id
            ));
        }
    };

    let projected_fields = epiphany_update_patch_changed_fields(&core_patch)
        .into_iter()
        .map(|field| format!("{field:?}"))
        .collect();
    let acceptance_bundle = build_role_acceptance_bundle(
        binding_id,
        EpiphanyRoleAcceptanceFinding {
            role_id,
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
    let applied_patch = core_patch.clone();
    let changed_fields = epiphany_update_patch_changed_fields(&applied_patch);
    let state_update = state_update_from_core_patch(expected_revision, core_patch);

    Ok(RoleAcceptanceUpdate {
        state_update,
        applied_patch,
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
        EpiphanyStateUpdatedField::AcceptanceReceipts,
        EpiphanyStateUpdatedField::Observations,
        EpiphanyStateUpdatedField::Evidence,
    ];
    if scratch.is_some() {
        changed_fields.push(EpiphanyStateUpdatedField::Scratch);
    }
    if investigation_checkpoint.is_some() {
        changed_fields.push(EpiphanyStateUpdatedField::InvestigationCheckpoint);
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

pub fn epiphany_job_launch_changed_fields() -> Vec<EpiphanyStateUpdatedField> {
    vec![
        EpiphanyStateUpdatedField::JobBindings,
        EpiphanyStateUpdatedField::RuntimeLinks,
    ]
}

pub fn thread_epiphany_patch_has_state_replacements(
    patch: &EpiphanyRoleStatePatchDocument,
) -> bool {
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
    patch: &EpiphanyRoleStatePatchDocument,
) -> Vec<EpiphanyStateUpdatedField> {
    let mut fields = Vec::new();
    if patch.objective.is_some() {
        fields.push(EpiphanyStateUpdatedField::Objective);
    }
    if patch.active_subgoal_id.is_some() {
        fields.push(EpiphanyStateUpdatedField::ActiveSubgoalId);
    }
    if patch.subgoals.is_some() {
        fields.push(EpiphanyStateUpdatedField::Subgoals);
    }
    if patch.invariants.is_some() {
        fields.push(EpiphanyStateUpdatedField::Invariants);
    }
    if patch.graphs.is_some() {
        fields.push(EpiphanyStateUpdatedField::Graphs);
    }
    if patch.graph_frontier.is_some() {
        fields.push(EpiphanyStateUpdatedField::GraphFrontier);
    }
    if patch.graph_checkpoint.is_some() {
        fields.push(EpiphanyStateUpdatedField::GraphCheckpoint);
    }
    if patch.scratch.is_some() {
        fields.push(EpiphanyStateUpdatedField::Scratch);
    }
    if patch.investigation_checkpoint.is_some() {
        fields.push(EpiphanyStateUpdatedField::InvestigationCheckpoint);
    }
    if patch.job_bindings.is_some() {
        fields.push(EpiphanyStateUpdatedField::JobBindings);
    }
    if !patch.acceptance_receipts.is_empty() {
        fields.push(EpiphanyStateUpdatedField::AcceptanceReceipts);
    }
    if !patch.runtime_links.is_empty() {
        fields.push(EpiphanyStateUpdatedField::RuntimeLinks);
    }
    if !patch.observations.is_empty() {
        fields.push(EpiphanyStateUpdatedField::Observations);
    }
    if !patch.evidence.is_empty() {
        fields.push(EpiphanyStateUpdatedField::Evidence);
    }
    if patch.churn.is_some() {
        fields.push(EpiphanyStateUpdatedField::Churn);
    }
    if patch.mode.is_some() {
        fields.push(EpiphanyStateUpdatedField::Mode);
    }
    if patch.planning.is_some() {
        fields.push(EpiphanyStateUpdatedField::Planning);
    }
    fields
}

pub fn epiphany_promote_changed_fields(
    patch: &EpiphanyRoleStatePatchDocument,
) -> Vec<EpiphanyStateUpdatedField> {
    let mut fields = epiphany_update_patch_changed_fields(patch);
    if !fields.contains(&EpiphanyStateUpdatedField::Evidence) {
        fields.push(EpiphanyStateUpdatedField::Evidence);
    }
    fields
}
