use crate::EpiphanyCoordinatorReorientResultSnapshot;
use crate::EpiphanyCoordinatorRoleResultSnapshot;
use crate::EpiphanyCoordinatorRoleResultStatus;
use crate::EpiphanyCrrcResultStatus;
use crate::EpiphanyLaunchOrganContract;
use crate::EpiphanyReceiptEffectKind;
use crate::EpiphanyReorientAcceptanceFinding;
use crate::EpiphanyReorientFindingInterpretation;
use crate::EpiphanyRoleAcceptanceFinding;
use crate::EpiphanyRoleFindingInterpretation;
use crate::EpiphanyRoleResultRoleId;
use crate::EpiphanyRoleStatePatchDocument;
use crate::EpiphanyStateUpdate;
use crate::EpiphanyStateUpdatedField;
use crate::MIND_GATEWAY_REVIEW_TYPE;
use crate::MindGatewayReview;
use crate::apply_epiphany_state_update;
use crate::build_reorient_acceptance_bundle;
use crate::build_role_acceptance_bundle;
use crate::coordinator_results::latest_runtime_link;
use crate::epiphany_state_update_validation_errors;
use crate::imagination_role_state_patch_policy_errors;
use crate::load_thread_state;
use crate::mind_review_allows_state;
use crate::mind_review_reorient_acceptance;
use crate::mind_review_role_acceptance;
use crate::modeling_role_state_patch_policy_errors;
use crate::read_reorient_result_snapshot;
use crate::read_role_result_snapshot;
use crate::read_runtime_reorient_result;
use crate::read_runtime_role_result;
use crate::research_role_state_patch_policy_errors;
use crate::runtime_worker_launch_request;
use crate::write_thread_state;
use anyhow::Result;
use anyhow::anyhow;
use epiphany_state_model::EpiphanyThreadState;
use std::path::Path;
use std::path::PathBuf;

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
#[derive(Debug, Clone)]
pub struct EpiphanyCoordinatorService {
    thread_state_store: PathBuf,
    runtime_spine_store: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EpiphanyCoordinatorStateApplied {
    pub revision: u64,
    pub changed_fields: Vec<EpiphanyStateUpdatedField>,
    pub state: EpiphanyThreadState,
}

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

impl EpiphanyCoordinatorService {
    pub fn new(
        thread_state_store: impl Into<PathBuf>,
        runtime_spine_store: impl Into<PathBuf>,
    ) -> Self {
        Self {
            thread_state_store: thread_state_store.into(),
            runtime_spine_store: runtime_spine_store.into(),
        }
    }

    pub fn thread_state_store(&self) -> &Path {
        &self.thread_state_store
    }

    pub fn runtime_spine_store(&self) -> &Path {
        &self.runtime_spine_store
    }

    pub fn state(&self) -> Result<Option<EpiphanyThreadState>> {
        load_thread_state(&self.thread_state_store)
    }

    pub fn role_result(
        &self,
        role_id: EpiphanyRoleResultRoleId,
        binding_id: &str,
    ) -> Result<EpiphanyCoordinatorRoleResultSnapshot> {
        let state = self.state()?;
        Ok(read_role_result_snapshot(
            state.as_ref(),
            Some(&self.runtime_spine_store),
            role_id,
            binding_id,
        ))
    }

    pub fn reorient_result(
        &self,
        binding_id: &str,
    ) -> Result<EpiphanyCoordinatorReorientResultSnapshot> {
        let state = self.state()?;
        Ok(read_reorient_result_snapshot(
            state.as_ref(),
            Some(&self.runtime_spine_store),
            binding_id,
        ))
    }

    pub fn apply_state_update(
        &self,
        thread_id: &str,
        update: EpiphanyStateUpdate,
        reference_turn_id: Option<String>,
    ) -> Result<EpiphanyCoordinatorStateApplied> {
        let changed_fields = changed_fields(&update);
        let state = apply_coordinator_state_update_to_state(
            &self.state()?.unwrap_or_default(),
            update,
            reference_turn_id,
        )?;
        write_thread_state(&self.thread_state_store, thread_id, &state)?;
        Ok(EpiphanyCoordinatorStateApplied {
            revision: state.revision,
            changed_fields,
            state,
        })
    }
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

pub fn apply_coordinator_state_update_to_state(
    current_state: &EpiphanyThreadState,
    update: EpiphanyStateUpdate,
    reference_turn_id: Option<String>,
) -> Result<EpiphanyThreadState> {
    if update.is_empty() {
        return Err(anyhow!(
            "epiphany update patch must contain at least one mutation"
        ));
    }
    if let Some(expected_revision) = update.expected_revision
        && expected_revision != current_state.revision
    {
        return Err(anyhow!(
            "epiphany state revision mismatch: expected {expected_revision}, found {}",
            current_state.revision
        ));
    }

    let validation_errors = epiphany_state_update_validation_errors(current_state, &update);
    if !validation_errors.is_empty() {
        return Err(anyhow!(
            "invalid epiphany update patch: {}",
            validation_errors.join("; ")
        ));
    }

    let mut next_state = current_state.clone();
    apply_epiphany_state_update(&mut next_state, update, reference_turn_id);
    Ok(next_state)
}

fn changed_fields(update: &EpiphanyStateUpdate) -> Vec<EpiphanyStateUpdatedField> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_service_owns_revision_gated_thread_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let service = EpiphanyCoordinatorService::new(
            temp.path().join("thread-state.msgpack"),
            temp.path().join("runtime-spine.msgpack"),
        );

        let applied = service.apply_state_update(
            "coordinator-session-1",
            EpiphanyStateUpdate {
                expected_revision: Some(0),
                objective: Some("Remove Codex from coordinator authority.".to_string()),
                ..Default::default()
            },
            Some("operator-turn-1".to_string()),
        )?;

        assert_eq!(applied.revision, 1);
        assert_eq!(
            applied.changed_fields,
            vec![EpiphanyStateUpdatedField::Objective]
        );
        assert_eq!(
            service.state()?.and_then(|state| state.objective),
            Some("Remove Codex from coordinator authority.".to_string())
        );
        Ok(())
    }

    #[test]
    fn rejected_revision_cannot_mutate_or_repair_native_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let service = EpiphanyCoordinatorService::new(
            temp.path().join("thread-state.msgpack"),
            temp.path().join("runtime-spine.msgpack"),
        );
        service.apply_state_update(
            "coordinator-session-1",
            EpiphanyStateUpdate {
                expected_revision: Some(0),
                objective: Some("Canonical objective".to_string()),
                ..Default::default()
            },
            None,
        )?;

        let error = service
            .apply_state_update(
                "coordinator-session-1",
                EpiphanyStateUpdate {
                    expected_revision: Some(0),
                    objective: Some("Stale overwrite".to_string()),
                    ..Default::default()
                },
                None,
            )
            .expect_err("stale revision must be refused");

        assert!(error.to_string().contains("revision mismatch"));
        let state = service.state()?.expect("canonical state must remain");
        assert_eq!(state.revision, 1);
        assert_eq!(state.objective.as_deref(), Some("Canonical objective"));
        Ok(())
    }

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
