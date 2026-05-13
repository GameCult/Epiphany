use chrono::SecondsFormat;
use chrono::Utc;
use codex_app_server_protocol::ThreadEpiphanyReorientFinding;
use codex_app_server_protocol::ThreadEpiphanyRoleFinding;
use codex_app_server_protocol::ThreadEpiphanyRoleId;
use codex_app_server_protocol::ThreadEpiphanyStateUpdatedField;
use codex_app_server_protocol::ThreadEpiphanyUpdatePatch;
use codex_core::CodexThread;
use codex_core::EpiphanyPromotionInput;
use codex_core::evaluate_promotion;
use codex_protocol::error::CodexErr;
use codex_protocol::protocol::EpiphanyEvidenceRecord;
use codex_protocol::protocol::EpiphanyThreadState;

use crate::mutation::build_reorient_acceptance_update;
use crate::mutation::build_role_acceptance_update;
use crate::mutation::epiphany_promote_changed_fields;
use crate::mutation::epiphany_update_patch_changed_fields;
use crate::mutation::state_update_from_thread_patch;
use crate::mutation::thread_epiphany_patch_has_state_replacements;
use crate::launch::epiphany_role_label;
use crate::runtime_results::load_completed_epiphany_reorient_finding;
use crate::runtime_results::load_completed_epiphany_role_finding;
use crate::state::client_visible_live_thread_epiphany_state;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EpiphanyThreadUpdateApplied {
    pub revision: u64,
    pub changed_fields: Vec<ThreadEpiphanyStateUpdatedField>,
    pub epiphany_state: EpiphanyThreadState,
}

#[derive(Debug, Clone)]
pub enum EpiphanyThreadPromoteApplied {
    Rejected { reasons: Vec<String> },
    Accepted(EpiphanyThreadUpdateApplied),
}

#[derive(Debug, Clone)]
pub struct EpiphanyRoleAcceptApplied {
    pub revision: u64,
    pub changed_fields: Vec<ThreadEpiphanyStateUpdatedField>,
    pub epiphany_state: EpiphanyThreadState,
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
    pub applied_patch: ThreadEpiphanyUpdatePatch,
    pub finding: ThreadEpiphanyRoleFinding,
}

#[derive(Debug, Clone)]
pub struct EpiphanyReorientAcceptApplied {
    pub revision: u64,
    pub changed_fields: Vec<ThreadEpiphanyStateUpdatedField>,
    pub epiphany_state: EpiphanyThreadState,
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
    pub finding: ThreadEpiphanyReorientFinding,
}

pub async fn apply_thread_epiphany_update(
    thread: &CodexThread,
    expected_revision: Option<u64>,
    patch: ThreadEpiphanyUpdatePatch,
) -> Result<EpiphanyThreadUpdateApplied, CodexErr> {
    let changed_fields = epiphany_update_patch_changed_fields(&patch);
    let update = state_update_from_thread_patch(expected_revision, patch);
    let epiphany_state = thread.epiphany_update_state(update).await?;
    let epiphany_state = client_visible_live_thread_epiphany_state(thread, epiphany_state).await;
    Ok(EpiphanyThreadUpdateApplied {
        revision: epiphany_state.revision,
        changed_fields,
        epiphany_state,
    })
}

pub async fn apply_thread_epiphany_promote(
    thread: &CodexThread,
    expected_revision: Option<u64>,
    patch: ThreadEpiphanyUpdatePatch,
    verifier_evidence: EpiphanyEvidenceRecord,
) -> Result<EpiphanyThreadPromoteApplied, CodexErr> {
    let decision = evaluate_promotion(EpiphanyPromotionInput {
        has_state_replacements: thread_epiphany_patch_has_state_replacements(&patch),
        active_subgoal_id: patch.active_subgoal_id.clone(),
        subgoals: patch.subgoals.clone(),
        invariants: patch.invariants.clone(),
        graphs: patch.graphs.clone(),
        graph_frontier: patch.graph_frontier.clone(),
        graph_checkpoint: patch.graph_checkpoint.clone(),
        investigation_checkpoint: patch.investigation_checkpoint.clone(),
        churn: patch.churn.clone(),
        observations: patch.observations.clone(),
        evidence: patch.evidence.clone(),
        verifier_evidence: verifier_evidence.clone(),
    });
    if !decision.accepted {
        return Ok(EpiphanyThreadPromoteApplied::Rejected {
            reasons: decision.reasons,
        });
    }

    let changed_fields = epiphany_promote_changed_fields(&patch);
    let mut patch = patch;
    patch.evidence.push(verifier_evidence);
    let update = state_update_from_thread_patch(expected_revision, patch);
    let epiphany_state = thread.epiphany_update_state(update).await?;
    let epiphany_state = client_visible_live_thread_epiphany_state(thread, epiphany_state).await;
    Ok(EpiphanyThreadPromoteApplied::Accepted(
        EpiphanyThreadUpdateApplied {
            revision: epiphany_state.revision,
            changed_fields,
            epiphany_state,
        },
    ))
}

pub async fn apply_thread_epiphany_role_accept(
    thread: &CodexThread,
    role_id: ThreadEpiphanyRoleId,
    expected_revision: Option<u64>,
    binding_id: &str,
) -> Result<EpiphanyRoleAcceptApplied, CodexErr> {
    let state = thread.epiphany_state().await.ok_or_else(|| {
        CodexErr::InvalidRequest(
            "cannot accept an Epiphany role finding without authoritative Epiphany state"
                .to_string(),
        )
    })?;
    validate_expected_revision(expected_revision, state.revision)?;

    let finding = load_completed_epiphany_role_finding(thread, &state, role_id, binding_id).await?;
    let accepted_prefix = epiphany_role_label(role_id);
    let acceptance_update = build_role_acceptance_update(
        role_id,
        binding_id,
        &finding,
        format!("ev-{accepted_prefix}-{}", Uuid::new_v4()),
        format!("obs-{accepted_prefix}-{}", Uuid::new_v4()),
        Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    )
    .map_err(CodexErr::InvalidRequest)?;

    let accepted_receipt_id = acceptance_update.accepted_receipt_id.clone();
    let accepted_observation_id = acceptance_update.accepted_observation_id.clone();
    let accepted_evidence_id = acceptance_update.accepted_evidence_id.clone();
    let changed_fields = acceptance_update.changed_fields.clone();
    let applied_patch = acceptance_update.patch.clone();
    let epiphany_state = thread
        .epiphany_update_state(state_update_from_thread_patch(
            expected_revision,
            acceptance_update.patch,
        ))
        .await?;
    let epiphany_state = client_visible_live_thread_epiphany_state(thread, epiphany_state).await;

    Ok(EpiphanyRoleAcceptApplied {
        revision: epiphany_state.revision,
        changed_fields,
        epiphany_state,
        accepted_receipt_id,
        accepted_observation_id,
        accepted_evidence_id,
        applied_patch,
        finding,
    })
}

pub async fn apply_thread_epiphany_reorient_accept(
    thread: &CodexThread,
    expected_revision: Option<u64>,
    binding_id: &str,
    update_scratch: bool,
    update_investigation_checkpoint: bool,
) -> Result<EpiphanyReorientAcceptApplied, CodexErr> {
    let state = thread.epiphany_state().await.ok_or_else(|| {
        CodexErr::InvalidRequest(
            "cannot accept a reorientation finding without authoritative Epiphany state"
                .to_string(),
        )
    })?;
    validate_expected_revision(expected_revision, state.revision)?;
    if update_investigation_checkpoint && state.investigation_checkpoint.is_none() {
        return Err(CodexErr::InvalidRequest(
            "cannot update investigation checkpoint because this thread has no durable checkpoint"
                .to_string(),
        ));
    }

    let finding = load_completed_epiphany_reorient_finding(thread, &state, binding_id).await?;
    let acceptance_update = build_reorient_acceptance_update(
        expected_revision,
        binding_id,
        &finding,
        format!("ev-reorient-{}", Uuid::new_v4()),
        format!("obs-reorient-{}", Uuid::new_v4()),
        Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        update_scratch,
        update_investigation_checkpoint,
        state.investigation_checkpoint.clone(),
    )
    .map_err(CodexErr::InvalidRequest)?;

    let accepted_receipt_id = acceptance_update.accepted_receipt_id.clone();
    let accepted_observation_id = acceptance_update.accepted_observation_id.clone();
    let accepted_evidence_id = acceptance_update.accepted_evidence_id.clone();
    let changed_fields = acceptance_update.changed_fields.clone();
    let epiphany_state = thread
        .epiphany_update_state(acceptance_update.state_update)
        .await?;
    let epiphany_state = client_visible_live_thread_epiphany_state(thread, epiphany_state).await;

    Ok(EpiphanyReorientAcceptApplied {
        revision: epiphany_state.revision,
        changed_fields,
        epiphany_state,
        accepted_receipt_id,
        accepted_observation_id,
        accepted_evidence_id,
        finding,
    })
}

fn validate_expected_revision(
    expected_revision: Option<u64>,
    actual_revision: u64,
) -> Result<(), CodexErr> {
    if let Some(expected_revision) = expected_revision
        && actual_revision != expected_revision
    {
        return Err(CodexErr::InvalidRequest(format!(
            "epiphany state revision mismatch: expected {expected_revision}, found {actual_revision}"
        )));
    }
    Ok(())
}
