use crate::coordinator_results::latest_runtime_link;
use crate::coordinator_state::changed_fields;
use crate::*;
use epiphany_state_model::EpiphanyThreadState;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpiphanyCoordinatorAdmissionError {
    InvalidRequest(String),
    Store(String),
}

impl std::fmt::Display for EpiphanyCoordinatorAdmissionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequest(message) | Self::Store(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for EpiphanyCoordinatorAdmissionError {}

/// The Epiphany-native owner of coordinator state.
///
/// Codex compatibility surfaces may translate requests into calls on this
/// service, but they do not receive a persistence hook or retain an independent
/// state opinion.
pub struct EpiphanyRoleAcceptanceUpdate {
    pub state_update: EpiphanyStateUpdate,
    pub applied_patch: EpiphanyRoleStatePatchDocument,
    pub changed_fields: Vec<EpiphanyStateUpdatedField>,
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
    pub mind_review: MindGatewayReview,
}

pub struct EpiphanyReorientAcceptanceUpdate {
    pub state_update: EpiphanyStateUpdate,
    pub changed_fields: Vec<EpiphanyStateUpdatedField>,
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
    pub mind_review: MindGatewayReview,
}

pub fn completed_role_finding(
    runtime_store_path: Option<&Path>,
    state: &EpiphanyThreadState,
    role_id: EpiphanyRoleResultRoleId,
    binding_id: &str,
) -> Result<EpiphanyRoleFindingInterpretation, EpiphanyCoordinatorAdmissionError> {
    let Some(link) = latest_runtime_link(state, binding_id) else {
        return if state
            .job_bindings
            .iter()
            .any(|binding| binding.id == binding_id)
        {
            Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(
                "role findings without runtime-spine results are unsupported; accept only typed runtime-spine results".to_string(),
            ))
        } else {
            Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
                "epiphany role binding {binding_id:?} was not found"
            )))
        };
    };
    let snapshot = read_runtime_role_result(runtime_store_path, &link.runtime_job_id, role_id);
    if snapshot.status != EpiphanyCoordinatorRoleResultStatus::Completed {
        return Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
            "cannot accept role result while worker status is {:?}",
            snapshot.status
        )));
    }
    let store = runtime_store_path.ok_or_else(|| {
        EpiphanyCoordinatorAdmissionError::InvalidRequest(
            "cannot accept completed role worker without a loaded runtime-spine store".to_string(),
        )
    })?;
    load_launch_organ_contract(store, &link.runtime_job_id, "role")?;
    snapshot.finding.ok_or_else(|| {
        EpiphanyCoordinatorAdmissionError::InvalidRequest(
            "cannot accept completed role worker because no typed runtime-spine result was recorded"
                .to_string(),
        )
    })
}

pub fn completed_reorient_finding(
    runtime_store_path: Option<&Path>,
    state: &EpiphanyThreadState,
    binding_id: &str,
) -> Result<EpiphanyReorientFindingInterpretation, EpiphanyCoordinatorAdmissionError> {
    let Some(link) = latest_runtime_link(state, binding_id) else {
        return if state
            .job_bindings
            .iter()
            .any(|binding| binding.id == binding_id)
        {
            Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(
                "reorientation findings without runtime-spine results are unsupported; accept only typed runtime-spine results".to_string(),
            ))
        } else {
            Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
                "epiphany reorientation binding {binding_id:?} was not found"
            )))
        };
    };
    let snapshot = read_runtime_reorient_result(runtime_store_path, &link.runtime_job_id);
    if snapshot.status != EpiphanyCrrcResultStatus::Completed {
        return Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
            "cannot accept reorientation result while worker status is {:?}",
            snapshot.status
        )));
    }
    let store = runtime_store_path.ok_or_else(|| {
        EpiphanyCoordinatorAdmissionError::InvalidRequest(
            "cannot accept completed reorientation worker without a loaded runtime-spine store"
                .to_string(),
        )
    })?;
    load_launch_organ_contract(store, &link.runtime_job_id, "reorient")?;
    snapshot.finding.ok_or_else(|| {
        EpiphanyCoordinatorAdmissionError::InvalidRequest(
            "cannot accept completed reorientation worker because no typed runtime-spine result was recorded"
                .to_string(),
        )
    })
}

pub fn load_launch_organ_contract(
    runtime_store_path: &Path,
    job_id: &str,
    expected_document_kind: &str,
) -> Result<EpiphanyLaunchOrganContract, EpiphanyCoordinatorAdmissionError> {
    let request = runtime_worker_launch_request(runtime_store_path, job_id).map_err(|error| {
        EpiphanyCoordinatorAdmissionError::Store(format!(
            "failed to read worker launch request for runtime job {job_id:?}: {error}"
        ))
    })?;
    let request = request.ok_or_else(|| {
        EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
            "cannot accept runtime job {job_id:?} without its typed worker launch request"
        ))
    })?;
    if request.document_kind != expected_document_kind {
        return Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
            "cannot accept runtime job {job_id:?}: launch document kind {:?} does not match expected {expected_document_kind:?}",
            request.document_kind
        )));
    }
    if request.organ_launch_contract.dependencies.is_empty()
        || request
            .organ_launch_contract
            .receipt_proof_profiles
            .is_empty()
    {
        return Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
            "cannot accept runtime job {job_id:?}: worker launch request has no organ dependency/proof-profile contract"
        )));
    }
    if !request
        .organ_launch_contract
        .receipt_proof_profiles
        .iter()
        .any(|profile| {
            profile.effect_kind == EpiphanyReceiptEffectKind::StateAdmission
                && profile
                    .required_before_promotion_document_types
                    .iter()
                    .any(|document_type| document_type == MIND_GATEWAY_REVIEW_TYPE)
        })
    {
        return Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
            "cannot accept runtime job {job_id:?}: worker launch contract has no state-admission proof profile requiring Mind review"
        )));
    }
    Ok(request.organ_launch_contract)
}

pub fn acceptance_launch_contract_for_binding(
    runtime_store_path: &Path,
    state: &EpiphanyThreadState,
    binding_id: &str,
    expected_document_kind: &str,
) -> Result<EpiphanyLaunchOrganContract, EpiphanyCoordinatorAdmissionError> {
    let link = latest_runtime_link(state, binding_id).ok_or_else(|| {
        EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
            "cannot prove receipt profile for binding {binding_id:?}: no runtime link exists"
        ))
    })?;
    load_launch_organ_contract(
        runtime_store_path,
        &link.runtime_job_id,
        expected_document_kind,
    )
}

pub fn enforce_acceptance_receipt_proofs(
    contract: &EpiphanyLaunchOrganContract,
    claimed_effects: &[EpiphanyReceiptEffectKind],
    available_document_types: &[String],
    enforceable_document_types: &[String],
) -> Result<(), EpiphanyCoordinatorAdmissionError> {
    let evaluations = evaluate_receipt_proof_profiles(
        contract,
        claimed_effects,
        available_document_types,
        enforceable_document_types,
    );
    let errors = receipt_proof_evaluation_errors(&evaluations);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
            "receipt proof profile rejected state admission: {}",
            errors.join("; ")
        )))
    }
}

pub fn role_acceptance_claimed_effects(
    role_id: EpiphanyRoleResultRoleId,
    changed_fields: &[EpiphanyStateUpdatedField],
) -> Vec<EpiphanyReceiptEffectKind> {
    let mut effects = vec![EpiphanyReceiptEffectKind::StateAdmission];
    if changed_fields.iter().any(|field| {
        matches!(
            field,
            EpiphanyStateUpdatedField::Evidence | EpiphanyStateUpdatedField::Observations
        )
    }) {
        effects.push(EpiphanyReceiptEffectKind::EvidencePromotion);
    }
    if role_id == EpiphanyRoleResultRoleId::Verification {
        effects.push(EpiphanyReceiptEffectKind::Verification);
    }
    effects
}

pub fn role_acceptance_enforceable_receipts(role_id: EpiphanyRoleResultRoleId) -> Vec<String> {
    let mut receipts = vec![MIND_GATEWAY_REVIEW_TYPE.to_string()];
    if role_id == EpiphanyRoleResultRoleId::Research {
        receipts.push(SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE.to_string());
        receipts.push(EYES_EVIDENCE_PACKET_TYPE.to_string());
    } else if role_id == EpiphanyRoleResultRoleId::Verification {
        receipts.push(SOUL_VERDICT_RECEIPT_TYPE.to_string());
    }
    receipts
}

pub fn reorient_acceptance_claimed_effects() -> Vec<EpiphanyReceiptEffectKind> {
    vec![
        EpiphanyReceiptEffectKind::StateAdmission,
        EpiphanyReceiptEffectKind::ContinuityRecovery,
    ]
}

pub fn build_native_role_acceptance_update(
    expected_revision: Option<u64>,
    role_id: EpiphanyRoleResultRoleId,
    binding_id: &str,
    finding: &EpiphanyRoleFindingInterpretation,
    accepted_evidence_id: String,
    accepted_observation_id: String,
    accepted_at: String,
) -> Result<EpiphanyRoleAcceptanceUpdate, String> {
    let mut patch = finding.state_patch.clone().unwrap_or_default();
    let errors = match role_id {
        EpiphanyRoleResultRoleId::Imagination => imagination_role_state_patch_policy_errors(&patch),
        EpiphanyRoleResultRoleId::Modeling => modeling_role_state_patch_policy_errors(&patch),
        EpiphanyRoleResultRoleId::Research => research_role_state_patch_policy_errors(&patch),
        EpiphanyRoleResultRoleId::Verification => {
            patch = EpiphanyRoleStatePatchDocument::default();
            Vec::new()
        }
        EpiphanyRoleResultRoleId::Implementation | EpiphanyRoleResultRoleId::Reorientation => {
            return Err(format!(
                "role {role_id:?} cannot be accepted through roleAccept"
            ));
        }
    };
    if role_id != EpiphanyRoleResultRoleId::Verification && finding.state_patch.is_none() {
        return Err("completed role finding did not include a reviewable statePatch".to_string());
    }
    if !errors.is_empty() {
        return Err(format!(
            "{} role state patch is not acceptable: {}",
            role_label_lower(role_id),
            errors.join("; ")
        ));
    }

    let projected_fields = changed_fields_from_patch(&patch)
        .iter()
        .map(|field| format!("{field:?}"))
        .collect();
    let mind_review = mind_review_role_acceptance(binding_id, role_id, finding, &patch);
    mind_review_allows_state(&mind_review)?;
    let bundle = build_role_acceptance_bundle(
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
    let accepted_receipt_id = bundle.accepted_receipt_id.clone();
    let accepted_observation_id = bundle.accepted_observation_id.clone();
    let accepted_evidence_id = bundle.accepted_evidence_id.clone();
    patch.evidence.push(bundle.evidence);
    patch.observations.push(bundle.observation);
    patch.acceptance_receipts.push(bundle.receipt);
    let changed_fields = changed_fields_from_patch(&patch);
    Ok(EpiphanyRoleAcceptanceUpdate {
        state_update: state_update_from_patch(expected_revision, patch.clone()),
        applied_patch: patch,
        changed_fields,
        accepted_receipt_id,
        accepted_observation_id,
        accepted_evidence_id,
        mind_review,
    })
}

pub fn build_native_reorient_acceptance_update(
    expected_revision: Option<u64>,
    binding_id: &str,
    finding: &EpiphanyReorientFindingInterpretation,
    accepted_evidence_id: String,
    accepted_observation_id: String,
    accepted_at: String,
    update_scratch: bool,
    update_investigation_checkpoint: bool,
    checkpoint: Option<epiphany_state_model::EpiphanyInvestigationCheckpoint>,
) -> Result<EpiphanyReorientAcceptanceUpdate, String> {
    let mind_finding = EpiphanyReorientAcceptanceFinding {
        mode: finding.mode.clone(),
        summary: finding.summary.clone(),
        next_safe_move: finding.next_safe_move.clone(),
        checkpoint_still_valid: finding.checkpoint_still_valid,
        files_inspected: finding.files_inspected.clone(),
        runtime_result_id: finding.runtime_result_id.clone(),
        runtime_job_id: finding.runtime_job_id.clone(),
    };
    let mind_review = mind_review_reorient_acceptance(
        binding_id,
        &mind_finding,
        update_scratch,
        update_investigation_checkpoint,
    );
    mind_review_allows_state(&mind_review)?;
    let bundle = build_reorient_acceptance_bundle(
        binding_id,
        mind_finding,
        accepted_evidence_id,
        accepted_observation_id,
        accepted_at,
        update_scratch,
        update_investigation_checkpoint
            .then_some(checkpoint)
            .flatten(),
    )?;
    let accepted_receipt_id = bundle.accepted_receipt_id.clone();
    let accepted_observation_id = bundle.accepted_observation_id.clone();
    let accepted_evidence_id = bundle.accepted_evidence_id.clone();
    let mut changed_fields = vec![
        EpiphanyStateUpdatedField::AcceptanceReceipts,
        EpiphanyStateUpdatedField::Observations,
        EpiphanyStateUpdatedField::Evidence,
    ];
    if bundle.scratch.is_some() {
        changed_fields.push(EpiphanyStateUpdatedField::Scratch);
    }
    if bundle.investigation_checkpoint.is_some() {
        changed_fields.push(EpiphanyStateUpdatedField::InvestigationCheckpoint);
    }
    Ok(EpiphanyReorientAcceptanceUpdate {
        state_update: EpiphanyStateUpdate {
            expected_revision,
            scratch: bundle.scratch,
            investigation_checkpoint: bundle.investigation_checkpoint,
            acceptance_receipts: vec![bundle.receipt],
            observations: vec![bundle.observation],
            evidence: vec![bundle.evidence],
            ..Default::default()
        },
        changed_fields,
        accepted_receipt_id,
        accepted_observation_id,
        accepted_evidence_id,
        mind_review,
    })
}

fn state_update_from_patch(
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

fn changed_fields_from_patch(
    patch: &EpiphanyRoleStatePatchDocument,
) -> Vec<EpiphanyStateUpdatedField> {
    changed_fields(&state_update_from_patch(None, patch.clone()))
}

fn role_label_lower(role_id: EpiphanyRoleResultRoleId) -> &'static str {
    match role_id {
        EpiphanyRoleResultRoleId::Imagination => "imagination",
        EpiphanyRoleResultRoleId::Modeling => "modeling",
        EpiphanyRoleResultRoleId::Research => "research",
        EpiphanyRoleResultRoleId::Verification => "verification",
        EpiphanyRoleResultRoleId::Implementation => "implementation",
        EpiphanyRoleResultRoleId::Reorientation => "reorientation",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_finding_admission_refuses_missing_binding() {
        let error = completed_role_finding(
            None,
            &EpiphanyThreadState::default(),
            EpiphanyRoleResultRoleId::Modeling,
            "modeling-worker",
        )
        .expect_err("missing binding must not become an acceptable finding");
        assert_eq!(
            error,
            EpiphanyCoordinatorAdmissionError::InvalidRequest(
                "epiphany role binding \"modeling-worker\" was not found".to_string()
            )
        );
    }
}
