use crate::EpiphanyCoordinatorRoleResultStatus;
use crate::EpiphanyCrrcResultStatus;
use crate::EpiphanyReorientFindingInterpretation;
use crate::EpiphanyRoleFindingInterpretation;
use crate::EpiphanyRoleResultRoleId;
use crate::EpiphanyRoleSelfPersistenceReview;
use crate::EpiphanyRoleSelfPersistenceStatus;
use crate::EpiphanyRuntimeJobResult;
use crate::EpiphanyRuntimeJobSnapshot;
use crate::EpiphanyRuntimeJobStatus;
use crate::EpiphanyStateUpdate;
use crate::EpiphanyStateUpdatedField;
use crate::apply_epiphany_state_update;
use crate::epiphany_state_update_validation_errors;
use crate::interpret_runtime_reorient_worker_result;
use crate::interpret_runtime_role_worker_result;
use crate::load_thread_state;
use crate::runtime_job_snapshot;
use crate::runtime_reorient_worker_result;
use crate::runtime_role_worker_result;
use crate::write_thread_state;
use anyhow::Result;
use anyhow::anyhow;
use epiphany_state_model::EpiphanyThreadState;
use std::path::Path;
use std::path::PathBuf;

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

#[derive(Debug, Clone)]
pub struct EpiphanyCoordinatorRoleResultSnapshot {
    pub status: EpiphanyCoordinatorRoleResultStatus,
    pub finding: Option<EpiphanyRoleFindingInterpretation>,
    pub note: String,
}

#[derive(Debug, Clone)]
pub struct EpiphanyCoordinatorReorientResultSnapshot {
    pub status: EpiphanyCrrcResultStatus,
    pub finding: Option<EpiphanyReorientFindingInterpretation>,
    pub note: String,
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

pub fn read_role_result_snapshot(
    state: Option<&EpiphanyThreadState>,
    runtime_store_path: Option<&Path>,
    role_id: EpiphanyRoleResultRoleId,
    binding_id: &str,
) -> EpiphanyCoordinatorRoleResultSnapshot {
    let Some(state) = state else {
        return role_snapshot(
            role_id,
            EpiphanyCoordinatorRoleResultStatus::MissingState,
            None,
            None,
        );
    };
    if let Some(link) = latest_runtime_link(state, binding_id) {
        return read_runtime_role_result(runtime_store_path, &link.runtime_job_id, role_id);
    }
    if state
        .job_bindings
        .iter()
        .any(|binding| binding.id == binding_id)
    {
        role_snapshot(
            role_id,
            EpiphanyCoordinatorRoleResultStatus::BackendUnavailable,
            None,
            Some(
                "Role binding has no runtime-spine job id; launch a runtime-linked role worker for typed results.",
            ),
        )
    } else {
        role_snapshot(
            role_id,
            EpiphanyCoordinatorRoleResultStatus::MissingBinding,
            None,
            None,
        )
    }
}

pub fn read_reorient_result_snapshot(
    state: Option<&EpiphanyThreadState>,
    runtime_store_path: Option<&Path>,
    binding_id: &str,
) -> EpiphanyCoordinatorReorientResultSnapshot {
    let Some(state) = state else {
        return reorient_snapshot(EpiphanyCrrcResultStatus::MissingState, None, None);
    };
    if let Some(link) = latest_runtime_link(state, binding_id) {
        return read_runtime_reorient_result(runtime_store_path, &link.runtime_job_id);
    }
    if state
        .job_bindings
        .iter()
        .any(|binding| binding.id == binding_id)
    {
        reorient_snapshot(
            EpiphanyCrrcResultStatus::BackendUnavailable,
            None,
            Some(
                "Reorientation binding has no runtime-spine job id; launch a runtime-linked reorient worker for typed results.",
            ),
        )
    } else {
        reorient_snapshot(EpiphanyCrrcResultStatus::MissingBinding, None, None)
    }
}

fn latest_runtime_link<'a>(
    state: &'a EpiphanyThreadState,
    binding_id: &str,
) -> Option<&'a epiphany_state_model::EpiphanyRuntimeLink> {
    state
        .runtime_links
        .iter()
        .find(|link| link.binding_id == binding_id && !link.runtime_job_id.trim().is_empty())
}

pub fn read_runtime_role_result(
    runtime_store_path: Option<&Path>,
    job_id: &str,
    role_id: EpiphanyRoleResultRoleId,
) -> EpiphanyCoordinatorRoleResultSnapshot {
    let Some(runtime_store_path) = runtime_store_path else {
        return role_snapshot(
            role_id,
            EpiphanyCoordinatorRoleResultStatus::Pending,
            None,
            Some(
                "Heartbeat activation owns this role specialist; no loaded runtime-spine store is available yet.",
            ),
        );
    };
    let snapshot = match runtime_job_snapshot(runtime_store_path, job_id) {
        Ok(Some(snapshot)) => snapshot,
        Ok(None) => {
            return role_snapshot(
                role_id,
                EpiphanyCoordinatorRoleResultStatus::Pending,
                None,
                Some(&format!(
                    "Heartbeat runtime job {job_id:?} has not reported typed state yet."
                )),
            );
        }
        Err(error) => {
            return role_snapshot(
                role_id,
                EpiphanyCoordinatorRoleResultStatus::BackendUnavailable,
                None,
                Some(&format!(
                    "Failed to read heartbeat runtime-spine job {job_id:?}: {error}"
                )),
            );
        }
    };
    let status = role_result_status(&snapshot);
    let finding = match status {
        EpiphanyCoordinatorRoleResultStatus::Completed => {
            match runtime_role_worker_result(runtime_store_path, job_id) {
                Ok(Some(result)) => Some(interpret_runtime_role_worker_result(role_id, &result)),
                Ok(None) => {
                    return role_snapshot(
                        role_id,
                        EpiphanyCoordinatorRoleResultStatus::BackendUnavailable,
                        None,
                        Some(&format!(
                            "Heartbeat runtime job {job_id:?} completed without an EpiphanyRuntimeRoleWorkerResult typed document; generic lifecycle receipts are not reviewable findings."
                        )),
                    );
                }
                Err(error) => {
                    return role_snapshot(
                        role_id,
                        EpiphanyCoordinatorRoleResultStatus::BackendUnavailable,
                        None,
                        Some(&format!(
                            "Failed to read typed role worker result for heartbeat runtime job {job_id:?}: {error}"
                        )),
                    );
                }
            }
        }
        EpiphanyCoordinatorRoleResultStatus::Failed
        | EpiphanyCoordinatorRoleResultStatus::Cancelled => snapshot
            .result
            .as_ref()
            .map(interpret_role_lifecycle_failure),
        _ => None,
    };
    role_snapshot(role_id, status, finding, None)
}

pub fn read_runtime_reorient_result(
    runtime_store_path: Option<&Path>,
    job_id: &str,
) -> EpiphanyCoordinatorReorientResultSnapshot {
    let Some(runtime_store_path) = runtime_store_path else {
        return reorient_snapshot(
            EpiphanyCrrcResultStatus::Pending,
            None,
            Some(
                "Heartbeat activation owns this reorientation worker; no loaded runtime-spine store is available yet.",
            ),
        );
    };
    let snapshot = match runtime_job_snapshot(runtime_store_path, job_id) {
        Ok(Some(snapshot)) => snapshot,
        Ok(None) => {
            return reorient_snapshot(
                EpiphanyCrrcResultStatus::Pending,
                None,
                Some(&format!(
                    "Heartbeat runtime job {job_id:?} has not reported typed state yet."
                )),
            );
        }
        Err(error) => {
            return reorient_snapshot(
                EpiphanyCrrcResultStatus::BackendUnavailable,
                None,
                Some(&format!(
                    "Failed to read heartbeat runtime-spine job {job_id:?}: {error}"
                )),
            );
        }
    };
    let status = reorient_result_status(&snapshot);
    let finding = match status {
        EpiphanyCrrcResultStatus::Completed => {
            match runtime_reorient_worker_result(runtime_store_path, job_id) {
                Ok(Some(result)) => Some(interpret_runtime_reorient_worker_result(&result)),
                Ok(None) => {
                    return reorient_snapshot(
                        EpiphanyCrrcResultStatus::BackendUnavailable,
                        None,
                        Some(&format!(
                            "Heartbeat runtime job {job_id:?} completed without an EpiphanyRuntimeReorientWorkerResult typed document; generic lifecycle receipts are not reviewable findings."
                        )),
                    );
                }
                Err(error) => {
                    return reorient_snapshot(
                        EpiphanyCrrcResultStatus::BackendUnavailable,
                        None,
                        Some(&format!(
                            "Failed to read typed reorientation worker result for heartbeat runtime job {job_id:?}: {error}"
                        )),
                    );
                }
            }
        }
        EpiphanyCrrcResultStatus::Failed | EpiphanyCrrcResultStatus::Cancelled => snapshot
            .result
            .as_ref()
            .map(interpret_reorient_lifecycle_failure),
        _ => None,
    };
    reorient_snapshot(status, finding, None)
}

fn role_snapshot(
    role_id: EpiphanyRoleResultRoleId,
    status: EpiphanyCoordinatorRoleResultStatus,
    finding: Option<EpiphanyRoleFindingInterpretation>,
    note_override: Option<&str>,
) -> EpiphanyCoordinatorRoleResultSnapshot {
    let note = note_override.map(str::to_string).unwrap_or_else(|| {
        render_role_result_note(
            role_id,
            status,
            finding.as_ref(),
            finding
                .as_ref()
                .and_then(|finding| finding.job_error.as_deref()),
        )
    });
    EpiphanyCoordinatorRoleResultSnapshot {
        status,
        finding,
        note,
    }
}

fn reorient_snapshot(
    status: EpiphanyCrrcResultStatus,
    finding: Option<EpiphanyReorientFindingInterpretation>,
    note_override: Option<&str>,
) -> EpiphanyCoordinatorReorientResultSnapshot {
    let note = note_override.map(str::to_string).unwrap_or_else(|| {
        render_reorient_result_note(
            status,
            finding.as_ref(),
            finding
                .as_ref()
                .and_then(|finding| finding.job_error.as_deref()),
        )
    });
    EpiphanyCoordinatorReorientResultSnapshot {
        status,
        finding,
        note,
    }
}

fn interpret_role_lifecycle_failure(
    result: &EpiphanyRuntimeJobResult,
) -> EpiphanyRoleFindingInterpretation {
    EpiphanyRoleFindingInterpretation {
        verdict: Some(result.verdict.clone()),
        summary: Some(result.summary.clone()),
        next_safe_move: nonempty(&result.next_safe_move),
        checkpoint_summary: None,
        scratch_summary: None,
        files_inspected: Vec::new(),
        frontier_node_ids: Vec::new(),
        evidence_ids: result.evidence_refs.clone(),
        artifact_refs: result.artifact_refs.clone(),
        runtime_result_id: Some(result.result_id.clone()),
        runtime_job_id: Some(result.job_id.clone()),
        open_questions: Vec::new(),
        evidence_gaps: Vec::new(),
        risks: Vec::new(),
        state_patch: None,
        self_patch: None,
        self_persistence: None,
        job_error: Some(result.summary.clone()),
        item_error: None,
    }
}

fn interpret_reorient_lifecycle_failure(
    result: &EpiphanyRuntimeJobResult,
) -> EpiphanyReorientFindingInterpretation {
    EpiphanyReorientFindingInterpretation {
        mode: None,
        summary: Some(result.summary.clone()),
        next_safe_move: nonempty(&result.next_safe_move),
        checkpoint_still_valid: None,
        files_inspected: Vec::new(),
        frontier_node_ids: Vec::new(),
        evidence_ids: result.evidence_refs.clone(),
        artifact_refs: result.artifact_refs.clone(),
        runtime_result_id: Some(result.result_id.clone()),
        runtime_job_id: Some(result.job_id.clone()),
        job_error: Some(result.summary.clone()),
        item_error: None,
    }
}

fn role_result_status(
    snapshot: &EpiphanyRuntimeJobSnapshot,
) -> EpiphanyCoordinatorRoleResultStatus {
    match snapshot.job.status {
        EpiphanyRuntimeJobStatus::Queued => EpiphanyCoordinatorRoleResultStatus::Pending,
        EpiphanyRuntimeJobStatus::Running | EpiphanyRuntimeJobStatus::WaitingForReview => {
            EpiphanyCoordinatorRoleResultStatus::Running
        }
        EpiphanyRuntimeJobStatus::Completed if snapshot.result.is_some() => {
            EpiphanyCoordinatorRoleResultStatus::Completed
        }
        EpiphanyRuntimeJobStatus::Completed => EpiphanyCoordinatorRoleResultStatus::Pending,
        EpiphanyRuntimeJobStatus::Failed => EpiphanyCoordinatorRoleResultStatus::Failed,
        EpiphanyRuntimeJobStatus::Cancelled => EpiphanyCoordinatorRoleResultStatus::Cancelled,
    }
}

fn reorient_result_status(snapshot: &EpiphanyRuntimeJobSnapshot) -> EpiphanyCrrcResultStatus {
    match snapshot.job.status {
        EpiphanyRuntimeJobStatus::Queued => EpiphanyCrrcResultStatus::Pending,
        EpiphanyRuntimeJobStatus::Running | EpiphanyRuntimeJobStatus::WaitingForReview => {
            EpiphanyCrrcResultStatus::Running
        }
        EpiphanyRuntimeJobStatus::Completed if snapshot.result.is_some() => {
            EpiphanyCrrcResultStatus::Completed
        }
        EpiphanyRuntimeJobStatus::Completed => EpiphanyCrrcResultStatus::Pending,
        EpiphanyRuntimeJobStatus::Failed => EpiphanyCrrcResultStatus::Failed,
        EpiphanyRuntimeJobStatus::Cancelled => EpiphanyCrrcResultStatus::Cancelled,
    }
}

fn render_role_result_note(
    role_id: EpiphanyRoleResultRoleId,
    status: EpiphanyCoordinatorRoleResultStatus,
    finding: Option<&EpiphanyRoleFindingInterpretation>,
    item_error: Option<&str>,
) -> String {
    match status {
        EpiphanyCoordinatorRoleResultStatus::Completed => finding.map_or_else(
            || format!("{role_id:?} role specialist completed, but no structured result was recorded."),
            |finding| {
                let next = finding.next_safe_move.as_deref().unwrap_or("not supplied");
                let self_note = render_self_persistence_note(finding.self_persistence.as_ref())
                    .map(|note| format!(" {note}"))
                    .unwrap_or_default();
                if let Some(error) = finding.item_error.as_deref().or(item_error) {
                    format!("{role_id:?} role specialist completed, but the finding needs repair: {error}. Next safe move: {next}.{self_note}")
                } else {
                    format!("{role_id:?} role specialist completed. Next safe move: {next}.{self_note}")
                }
            },
        ),
        EpiphanyCoordinatorRoleResultStatus::Failed => item_error
            .map(|error| format!("{role_id:?} role specialist failed: {error}"))
            .unwrap_or_else(|| format!("{role_id:?} role specialist failed.")),
        EpiphanyCoordinatorRoleResultStatus::Cancelled => format!("{role_id:?} role specialist was cancelled before producing a result."),
        EpiphanyCoordinatorRoleResultStatus::Running => format!("{role_id:?} role specialist is still running."),
        EpiphanyCoordinatorRoleResultStatus::Pending => format!("{role_id:?} role specialist has not produced a result yet."),
        EpiphanyCoordinatorRoleResultStatus::MissingState => "No authoritative Epiphany state exists for this thread.".to_string(),
        EpiphanyCoordinatorRoleResultStatus::MissingBinding => "No matching Epiphany role specialist binding exists.".to_string(),
        EpiphanyCoordinatorRoleResultStatus::BackendUnavailable => "The bound runtime backend is unavailable.".to_string(),
        EpiphanyCoordinatorRoleResultStatus::BackendMissing => "The bound runtime backend job or item is missing.".to_string(),
    }
}

fn render_self_persistence_note(
    review: Option<&EpiphanyRoleSelfPersistenceReview>,
) -> Option<String> {
    let review = review?;
    match review.status {
        EpiphanyRoleSelfPersistenceStatus::Missing => None,
        EpiphanyRoleSelfPersistenceStatus::Accepted => Some(format!(
            "Self persistence request is acceptable for {}.",
            review
                .target_agent_id
                .as_deref()
                .unwrap_or("the role memory file")
        )),
        EpiphanyRoleSelfPersistenceStatus::Rejected => Some(format!(
            "Self persistence request was refused: {}.",
            if review.reasons.is_empty() {
                "no reason recorded".to_string()
            } else {
                review.reasons.join("; ")
            }
        )),
    }
}

fn render_reorient_result_note(
    status: EpiphanyCrrcResultStatus,
    finding: Option<&EpiphanyReorientFindingInterpretation>,
    item_error: Option<&str>,
) -> String {
    match status {
        EpiphanyCrrcResultStatus::Completed => finding.map_or_else(
            || "Reorientation worker completed, but no structured result was recorded.".to_string(),
            |finding| {
                format!(
                    "Reorientation worker completed. Next safe move: {}",
                    finding.next_safe_move.as_deref().unwrap_or("not supplied")
                )
            },
        ),
        EpiphanyCrrcResultStatus::Failed => item_error
            .map(|error| format!("Reorientation worker failed: {error}"))
            .unwrap_or_else(|| "Reorientation worker failed.".to_string()),
        EpiphanyCrrcResultStatus::Cancelled => {
            "Reorientation worker was cancelled before producing a result.".to_string()
        }
        EpiphanyCrrcResultStatus::Running => "Reorientation worker is still running.".to_string(),
        EpiphanyCrrcResultStatus::Pending => {
            "Reorientation worker has not produced a result yet.".to_string()
        }
        EpiphanyCrrcResultStatus::MissingState => {
            "No authoritative Epiphany state exists for this thread.".to_string()
        }
        EpiphanyCrrcResultStatus::MissingBinding => {
            "No matching Epiphany reorientation worker binding exists.".to_string()
        }
        EpiphanyCrrcResultStatus::BackendUnavailable => {
            "The bound runtime backend is unavailable.".to_string()
        }
        EpiphanyCrrcResultStatus::BackendMissing => {
            "The bound runtime backend job or item is missing.".to_string()
        }
    }
}

fn nonempty(value: &str) -> Option<String> {
    (!value.trim().is_empty()).then(|| value.to_string())
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
}
