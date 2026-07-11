use epiphany_core::EpiphanyReorientFindingInterpretation as CoreEpiphanyReorientFinding;
use epiphany_core::EpiphanyRoleFindingInterpretation as CoreEpiphanyRoleFinding;
use epiphany_core::EpiphanyRoleResultRoleId;
use epiphany_core::EpiphanyRoleStatePatchDocument;
use epiphany_core::EpiphanyStateUpdate;
use epiphany_core::EpiphanyStateUpdatedField;
use epiphany_state_model::EpiphanyInvestigationCheckpoint;

pub type RoleAcceptanceUpdate = epiphany_core::EpiphanyRoleAcceptanceUpdate;
pub type ReorientAcceptanceUpdate = epiphany_core::EpiphanyReorientAcceptanceUpdate;

pub fn build_role_acceptance_update(
    expected_revision: Option<u64>,
    role_id: EpiphanyRoleResultRoleId,
    binding_id: &str,
    finding: &CoreEpiphanyRoleFinding,
    accepted_evidence_id: String,
    accepted_observation_id: String,
    accepted_at: String,
) -> Result<RoleAcceptanceUpdate, String> {
    epiphany_core::build_native_role_acceptance_update(
        expected_revision,
        role_id,
        binding_id,
        finding,
        accepted_evidence_id,
        accepted_observation_id,
        accepted_at,
    )
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
    epiphany_core::build_native_reorient_acceptance_update(
        expected_revision,
        binding_id,
        finding,
        accepted_evidence_id,
        accepted_observation_id,
        accepted_at,
        update_scratch,
        update_investigation_checkpoint,
        checkpoint,
    )
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
    let update = state_update_from_core_patch(None, patch.clone());
    let mut fields = Vec::new();
    if update.objective.is_some() {
        fields.push(EpiphanyStateUpdatedField::Objective);
    }
    if update.active_subgoal_id.is_some() {
        fields.push(EpiphanyStateUpdatedField::ActiveSubgoalId);
    }
    if update.subgoals.is_some() {
        fields.push(EpiphanyStateUpdatedField::Subgoals);
    }
    if update.invariants.is_some() {
        fields.push(EpiphanyStateUpdatedField::Invariants);
    }
    if update.graphs.is_some() {
        fields.push(EpiphanyStateUpdatedField::Graphs);
    }
    if update.graph_frontier.is_some() {
        fields.push(EpiphanyStateUpdatedField::GraphFrontier);
    }
    if update.graph_checkpoint.is_some() {
        fields.push(EpiphanyStateUpdatedField::GraphCheckpoint);
    }
    if update.scratch.is_some() {
        fields.push(EpiphanyStateUpdatedField::Scratch);
    }
    if update.investigation_checkpoint.is_some() {
        fields.push(EpiphanyStateUpdatedField::InvestigationCheckpoint);
    }
    if update.job_bindings.is_some() {
        fields.push(EpiphanyStateUpdatedField::JobBindings);
    }
    if !update.acceptance_receipts.is_empty() {
        fields.push(EpiphanyStateUpdatedField::AcceptanceReceipts);
    }
    if !update.runtime_links.is_empty() {
        fields.push(EpiphanyStateUpdatedField::RuntimeLinks);
    }
    if !update.observations.is_empty() {
        fields.push(EpiphanyStateUpdatedField::Observations);
    }
    if !update.evidence.is_empty() {
        fields.push(EpiphanyStateUpdatedField::Evidence);
    }
    if update.churn.is_some() {
        fields.push(EpiphanyStateUpdatedField::Churn);
    }
    if update.mode.is_some() {
        fields.push(EpiphanyStateUpdatedField::Mode);
    }
    if update.planning.is_some() {
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
