use std::path::Path;

use epiphany_core::EpiphanyCoordinatorRoleResultStatus as CoreEpiphanyCoordinatorRoleResultStatus;
use epiphany_core::EpiphanyCrrcResultStatus as CoreEpiphanyCrrcResultStatus;
use epiphany_core::EpiphanyLaunchOrganContract;
use epiphany_core::EpiphanyReceiptEffectKind;
use epiphany_core::EpiphanyReorientFindingInterpretation;
use epiphany_core::EpiphanyRoleFindingInterpretation;
use epiphany_core::EpiphanyRoleResultRoleId;
use epiphany_core::EpiphanyRuntimeJobResult;
use epiphany_core::EpiphanyRuntimeJobSnapshot;
use epiphany_core::EpiphanyRuntimeJobStatus;
use epiphany_core::MIND_GATEWAY_REVIEW_TYPE;
use epiphany_core::interpret_runtime_reorient_worker_result;
use epiphany_core::interpret_runtime_role_worker_result;
use epiphany_core::runtime_job_snapshot;
use epiphany_core::runtime_reorient_worker_result;
use epiphany_core::runtime_role_worker_result;
use epiphany_core::runtime_worker_launch_request;
use epiphany_state_model::EpiphanyRuntimeLink;
use epiphany_state_model::EpiphanyThreadState;

use crate::error::EpiphanyBridgeError;
use crate::error::Result as BridgeResult;
use crate::results::render_core_reorient_result_note;
use crate::results::render_core_role_result_note;

#[derive(Debug, Clone)]
pub struct EpiphanyRoleResultSnapshot {
    pub status: CoreEpiphanyCoordinatorRoleResultStatus,
    pub finding: Option<EpiphanyRoleFindingInterpretation>,
    pub completed_at: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone)]
pub struct EpiphanyReorientResultSnapshot {
    pub status: CoreEpiphanyCrrcResultStatus,
    pub finding: Option<EpiphanyReorientFindingInterpretation>,
    pub note: String,
}

pub fn load_core_epiphany_role_result_from_runtime_spine_job(
    job_id: &str,
    runtime_store_path: Option<&Path>,
    role_id: EpiphanyRoleResultRoleId,
) -> EpiphanyRoleResultSnapshot {
    let Some(runtime_store_path) = runtime_store_path else {
        return EpiphanyRoleResultSnapshot {
            status: CoreEpiphanyCoordinatorRoleResultStatus::Pending,
            finding: None,
            completed_at: None,
            note: "Heartbeat activation owns this role specialist; no loaded runtime-spine store is available yet."
                .to_string(),
        };
    };
    let snapshot = match runtime_job_snapshot(runtime_store_path, job_id) {
        Ok(Some(snapshot)) => snapshot,
        Ok(None) => {
            return EpiphanyRoleResultSnapshot {
                status: CoreEpiphanyCoordinatorRoleResultStatus::Pending,
                finding: None,
                completed_at: None,
                note: format!(
                    "Heartbeat runtime job {:?} has not reported typed state yet.",
                    job_id
                ),
            };
        }
        Err(err) => {
            return EpiphanyRoleResultSnapshot {
                status: CoreEpiphanyCoordinatorRoleResultStatus::BackendUnavailable,
                finding: None,
                completed_at: None,
                note: format!(
                    "Failed to read heartbeat runtime-spine job {:?}: {err}",
                    job_id
                ),
            };
        }
    };
    let status = map_runtime_role_result_status(&snapshot);
    let finding = match status {
        CoreEpiphanyCoordinatorRoleResultStatus::Completed => {
            match runtime_role_worker_result(runtime_store_path, job_id) {
                Ok(Some(result)) => Some(interpret_runtime_role_worker_result(role_id, &result)),
                Ok(None) => {
                    return EpiphanyRoleResultSnapshot {
                        status: CoreEpiphanyCoordinatorRoleResultStatus::BackendUnavailable,
                        finding: None,
                        completed_at: snapshot
                            .result
                            .as_ref()
                            .map(|result| result.completed_at.clone()),
                        note: format!(
                            "Heartbeat runtime job {:?} completed without an EpiphanyRuntimeRoleWorkerResult typed document; generic lifecycle receipts are not reviewable findings.",
                            job_id
                        ),
                    };
                }
                Err(err) => {
                    return EpiphanyRoleResultSnapshot {
                        status: CoreEpiphanyCoordinatorRoleResultStatus::BackendUnavailable,
                        finding: None,
                        completed_at: snapshot
                            .result
                            .as_ref()
                            .map(|result| result.completed_at.clone()),
                        note: format!(
                            "Failed to read typed role worker result for heartbeat runtime job {:?}: {err}",
                            job_id
                        ),
                    };
                }
            }
        }
        CoreEpiphanyCoordinatorRoleResultStatus::Failed
        | CoreEpiphanyCoordinatorRoleResultStatus::Cancelled => snapshot
            .result
            .as_ref()
            .map(interpret_runtime_role_lifecycle_failure),
        _ => None,
    };
    let item_error = finding
        .as_ref()
        .and_then(|finding| finding.job_error.as_deref());
    let note = render_core_role_result_note(role_id, status, finding.as_ref(), item_error);
    EpiphanyRoleResultSnapshot {
        status,
        finding,
        completed_at: snapshot
            .result
            .as_ref()
            .map(|result| result.completed_at.clone()),
        note,
    }
}

pub fn load_core_epiphany_reorient_result_from_runtime_spine_job(
    job_id: &str,
    runtime_store_path: Option<&Path>,
) -> EpiphanyReorientResultSnapshot {
    let Some(runtime_store_path) = runtime_store_path else {
        return EpiphanyReorientResultSnapshot {
            status: CoreEpiphanyCrrcResultStatus::Pending,
            finding: None,
            note: "Heartbeat activation owns this reorientation worker; no loaded runtime-spine store is available yet."
                .to_string(),
        };
    };
    let snapshot = match runtime_job_snapshot(runtime_store_path, job_id) {
        Ok(Some(snapshot)) => snapshot,
        Ok(None) => {
            return EpiphanyReorientResultSnapshot {
                status: CoreEpiphanyCrrcResultStatus::Pending,
                finding: None,
                note: format!(
                    "Heartbeat runtime job {:?} has not reported typed state yet.",
                    job_id
                ),
            };
        }
        Err(err) => {
            return EpiphanyReorientResultSnapshot {
                status: CoreEpiphanyCrrcResultStatus::BackendUnavailable,
                finding: None,
                note: format!(
                    "Failed to read heartbeat runtime-spine job {:?}: {err}",
                    job_id
                ),
            };
        }
    };
    let status = map_runtime_reorient_result_status(&snapshot);
    let finding = match status {
        CoreEpiphanyCrrcResultStatus::Completed => {
            match runtime_reorient_worker_result(runtime_store_path, job_id) {
                Ok(Some(result)) => Some(interpret_runtime_reorient_worker_result(&result)),
                Ok(None) => {
                    return EpiphanyReorientResultSnapshot {
                        status: CoreEpiphanyCrrcResultStatus::BackendUnavailable,
                        finding: None,
                        note: format!(
                            "Heartbeat runtime job {:?} completed without an EpiphanyRuntimeReorientWorkerResult typed document; generic lifecycle receipts are not reviewable findings.",
                            job_id
                        ),
                    };
                }
                Err(err) => {
                    return EpiphanyReorientResultSnapshot {
                        status: CoreEpiphanyCrrcResultStatus::BackendUnavailable,
                        finding: None,
                        note: format!(
                            "Failed to read typed reorientation worker result for heartbeat runtime job {:?}: {err}",
                            job_id
                        ),
                    };
                }
            }
        }
        CoreEpiphanyCrrcResultStatus::Failed | CoreEpiphanyCrrcResultStatus::Cancelled => snapshot
            .result
            .as_ref()
            .map(interpret_runtime_reorient_lifecycle_failure),
        _ => None,
    };
    let item_error = finding
        .as_ref()
        .and_then(|finding| finding.job_error.as_deref());
    let note = render_core_reorient_result_note(status, finding.as_ref(), item_error);
    EpiphanyReorientResultSnapshot {
        status,
        finding,
        note,
    }
}

fn interpret_runtime_role_lifecycle_failure(
    result: &EpiphanyRuntimeJobResult,
) -> EpiphanyRoleFindingInterpretation {
    EpiphanyRoleFindingInterpretation {
        verdict: Some(result.verdict.clone()),
        summary: Some(result.summary.clone()),
        next_safe_move: empty_string_as_none(&result.next_safe_move),
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

fn interpret_runtime_reorient_lifecycle_failure(
    result: &EpiphanyRuntimeJobResult,
) -> EpiphanyReorientFindingInterpretation {
    EpiphanyReorientFindingInterpretation {
        mode: None,
        summary: Some(result.summary.clone()),
        next_safe_move: empty_string_as_none(&result.next_safe_move),
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

fn empty_string_as_none(value: &str) -> Option<String> {
    (!value.trim().is_empty()).then(|| value.to_string())
}

fn map_runtime_role_result_status(
    snapshot: &EpiphanyRuntimeJobSnapshot,
) -> CoreEpiphanyCoordinatorRoleResultStatus {
    match snapshot.job.status {
        EpiphanyRuntimeJobStatus::Queued => CoreEpiphanyCoordinatorRoleResultStatus::Pending,
        EpiphanyRuntimeJobStatus::Running | EpiphanyRuntimeJobStatus::WaitingForReview => {
            CoreEpiphanyCoordinatorRoleResultStatus::Running
        }
        EpiphanyRuntimeJobStatus::Completed => {
            if snapshot.result.is_some() {
                CoreEpiphanyCoordinatorRoleResultStatus::Completed
            } else {
                CoreEpiphanyCoordinatorRoleResultStatus::Pending
            }
        }
        EpiphanyRuntimeJobStatus::Failed => CoreEpiphanyCoordinatorRoleResultStatus::Failed,
        EpiphanyRuntimeJobStatus::Cancelled => CoreEpiphanyCoordinatorRoleResultStatus::Cancelled,
    }
}

fn map_runtime_reorient_result_status(
    snapshot: &EpiphanyRuntimeJobSnapshot,
) -> CoreEpiphanyCrrcResultStatus {
    match snapshot.job.status {
        EpiphanyRuntimeJobStatus::Queued => CoreEpiphanyCrrcResultStatus::Pending,
        EpiphanyRuntimeJobStatus::Running | EpiphanyRuntimeJobStatus::WaitingForReview => {
            CoreEpiphanyCrrcResultStatus::Running
        }
        EpiphanyRuntimeJobStatus::Completed => {
            if snapshot.result.is_some() {
                CoreEpiphanyCrrcResultStatus::Completed
            } else {
                CoreEpiphanyCrrcResultStatus::Pending
            }
        }
        EpiphanyRuntimeJobStatus::Failed => CoreEpiphanyCrrcResultStatus::Failed,
        EpiphanyRuntimeJobStatus::Cancelled => CoreEpiphanyCrrcResultStatus::Cancelled,
    }
}

pub async fn load_core_epiphany_role_result_snapshot(
    state: &EpiphanyThreadState,
    runtime_store_path: Option<&Path>,
    role_id: EpiphanyRoleResultRoleId,
    binding_id: &str,
) -> EpiphanyRoleResultSnapshot {
    if let Some(link) = latest_epiphany_runtime_link_for_binding(state, binding_id) {
        return load_core_epiphany_role_result_from_runtime_spine_job(
            link.runtime_job_id.as_str(),
            runtime_store_path,
            role_id,
        );
    }

    if !state
        .job_bindings
        .iter()
        .any(|binding| binding.id == binding_id)
    {
        return EpiphanyRoleResultSnapshot {
            status: CoreEpiphanyCoordinatorRoleResultStatus::MissingBinding,
            finding: None,
            completed_at: None,
            note: "No matching Epiphany role specialist binding exists.".to_string(),
        };
    }
    EpiphanyRoleResultSnapshot {
        status: CoreEpiphanyCoordinatorRoleResultStatus::BackendUnavailable,
        finding: None,
        completed_at: None,
        note: "Role binding has no runtime-spine job id; launch a runtime-linked role worker for typed results.".to_string(),
    }
}

pub fn load_completed_core_epiphany_role_finding(
    runtime_store_path: Option<&Path>,
    state: &EpiphanyThreadState,
    role_id: EpiphanyRoleResultRoleId,
    binding_id: &str,
) -> BridgeResult<EpiphanyRoleFindingInterpretation> {
    if let Some(link) = latest_epiphany_runtime_link_for_binding(state, binding_id) {
        let snapshot = load_core_epiphany_role_result_from_runtime_spine_job(
            link.runtime_job_id.as_str(),
            runtime_store_path,
            role_id,
        );
        if snapshot.status != CoreEpiphanyCoordinatorRoleResultStatus::Completed {
            return Err(EpiphanyBridgeError::InvalidRequest(format!(
                "cannot accept role result while worker status is {:?}",
                snapshot.status
            )));
        }
        let runtime_store_path = runtime_store_path.ok_or_else(|| {
            EpiphanyBridgeError::InvalidRequest(
                "cannot accept completed role worker without a loaded runtime-spine store"
                    .to_string(),
            )
        })?;
        require_launch_organ_contract(runtime_store_path, link.runtime_job_id.as_str(), "role")?;
        return snapshot.finding.ok_or_else(|| {
            EpiphanyBridgeError::InvalidRequest(
                "cannot accept completed role worker because no typed runtime-spine result was recorded"
                    .to_string(),
            )
        });
    }

    if !state
        .job_bindings
        .iter()
        .any(|binding| binding.id == binding_id)
    {
        return Err(EpiphanyBridgeError::InvalidRequest(format!(
            "epiphany role binding {:?} was not found",
            binding_id
        )));
    }

    Err(EpiphanyBridgeError::InvalidRequest(
        "role findings without runtime-spine results are unsupported; accept only typed runtime-spine results"
            .to_string(),
    ))
}

pub async fn load_core_epiphany_reorient_result_snapshot(
    state: Option<&EpiphanyThreadState>,
    runtime_store_path: Option<&Path>,
    binding_id: &str,
) -> EpiphanyReorientResultSnapshot {
    let Some(state) = state else {
        return EpiphanyReorientResultSnapshot {
            status: CoreEpiphanyCrrcResultStatus::MissingState,
            finding: None,
            note: "No authoritative Epiphany state exists for this thread.".to_string(),
        };
    };
    if let Some(link) = latest_epiphany_runtime_link_for_binding(state, binding_id) {
        return load_core_epiphany_reorient_result_from_runtime_spine_job(
            link.runtime_job_id.as_str(),
            runtime_store_path,
        );
    }

    if !state
        .job_bindings
        .iter()
        .any(|binding| binding.id == binding_id)
    {
        return EpiphanyReorientResultSnapshot {
            status: CoreEpiphanyCrrcResultStatus::MissingBinding,
            finding: None,
            note: "No matching Epiphany reorientation worker binding exists.".to_string(),
        };
    }
    EpiphanyReorientResultSnapshot {
        status: CoreEpiphanyCrrcResultStatus::BackendUnavailable,
        finding: None,
        note: "Reorientation binding has no runtime-spine job id; launch a runtime-linked reorient worker for typed results.".to_string(),
    }
}

pub fn load_completed_core_epiphany_reorient_finding(
    runtime_store_path: Option<&Path>,
    state: &EpiphanyThreadState,
    binding_id: &str,
) -> BridgeResult<EpiphanyReorientFindingInterpretation> {
    if let Some(link) = latest_epiphany_runtime_link_for_binding(state, binding_id) {
        let snapshot = load_core_epiphany_reorient_result_from_runtime_spine_job(
            link.runtime_job_id.as_str(),
            runtime_store_path,
        );
        if snapshot.status != CoreEpiphanyCrrcResultStatus::Completed {
            return Err(EpiphanyBridgeError::InvalidRequest(format!(
                "cannot accept reorientation result while worker status is {:?}",
                snapshot.status
            )));
        }
        let runtime_store_path = runtime_store_path.ok_or_else(|| {
            EpiphanyBridgeError::InvalidRequest(
                "cannot accept completed reorientation worker without a loaded runtime-spine store"
                    .to_string(),
            )
        })?;
        require_launch_organ_contract(
            runtime_store_path,
            link.runtime_job_id.as_str(),
            "reorient",
        )?;
        return snapshot.finding.ok_or_else(|| {
            EpiphanyBridgeError::InvalidRequest(
                "cannot accept completed reorientation worker because no typed runtime-spine result was recorded"
                    .to_string(),
            )
        });
    }

    if !state
        .job_bindings
        .iter()
        .any(|binding| binding.id == binding_id)
    {
        return Err(EpiphanyBridgeError::InvalidRequest(format!(
            "epiphany reorientation binding {:?} was not found",
            binding_id
        )));
    }

    Err(EpiphanyBridgeError::InvalidRequest(
        "reorientation findings without runtime-spine results are unsupported; accept only typed runtime-spine results"
            .to_string(),
    ))
}

fn require_launch_organ_contract(
    runtime_store_path: &Path,
    job_id: &str,
    expected_document_kind: &str,
) -> BridgeResult<()> {
    load_launch_organ_contract_for_runtime_job(runtime_store_path, job_id, expected_document_kind)
        .map(|_| ())
}

pub fn load_launch_organ_contract_for_runtime_job(
    runtime_store_path: &Path,
    job_id: &str,
    expected_document_kind: &str,
) -> BridgeResult<EpiphanyLaunchOrganContract> {
    let request = runtime_worker_launch_request(runtime_store_path, job_id).map_err(|err| {
        EpiphanyBridgeError::Fatal(format!(
            "failed to read worker launch request for runtime job {:?}: {err}",
            job_id
        ))
    })?;
    let request = request.ok_or_else(|| {
        EpiphanyBridgeError::InvalidRequest(format!(
            "cannot accept runtime job {:?} without its typed worker launch request",
            job_id
        ))
    })?;
    if request.document_kind != expected_document_kind {
        return Err(EpiphanyBridgeError::InvalidRequest(format!(
            "cannot accept runtime job {:?}: launch document kind {:?} does not match expected {:?}",
            job_id, request.document_kind, expected_document_kind
        )));
    }
    if request.organ_launch_contract.dependencies.is_empty()
        || request
            .organ_launch_contract
            .receipt_proof_profiles
            .is_empty()
    {
        return Err(EpiphanyBridgeError::InvalidRequest(format!(
            "cannot accept runtime job {:?}: worker launch request has no organ dependency/proof-profile contract",
            job_id
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
        return Err(EpiphanyBridgeError::InvalidRequest(format!(
            "cannot accept runtime job {:?}: worker launch contract has no state-admission proof profile requiring Mind review",
            job_id
        )));
    }
    Ok(request.organ_launch_contract)
}

pub fn latest_epiphany_runtime_link_for_binding<'a>(
    state: &'a EpiphanyThreadState,
    binding_id: &str,
) -> Option<&'a EpiphanyRuntimeLink> {
    state
        .runtime_links
        .iter()
        .find(|link| link.binding_id == binding_id && !link.runtime_job_id.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;

    use epiphany_core::EpiphanyCoordinatorRoleResultStatus as CoreEpiphanyCoordinatorRoleResultStatus;
    use epiphany_core::EpiphanyCrrcResultStatus as CoreEpiphanyCrrcResultStatus;
    use epiphany_core::EpiphanyRoleResultRoleId;
    use epiphany_core::RuntimeSpineInitOptions;
    use epiphany_core::RuntimeSpineJobOptions;
    use epiphany_core::RuntimeSpineJobResultOptions;
    use epiphany_core::RuntimeSpineSessionOptions;
    use epiphany_core::complete_runtime_job;
    use epiphany_core::create_runtime_job;
    use epiphany_core::create_runtime_session;
    use epiphany_core::initialize_runtime_spine;

    use super::*;

    const NOW: &str = "2026-05-13T00:00:00Z";

    #[test]
    fn role_result_requires_typed_worker_document() {
        let store = temp_store_path();
        seed_lifecycle_result(&store, "role-job", "modeling", "completed");

        let result = load_core_epiphany_role_result_from_runtime_spine_job(
            "role-job",
            Some(store.as_path()),
            EpiphanyRoleResultRoleId::Modeling,
        );

        assert_eq!(
            result.status,
            CoreEpiphanyCoordinatorRoleResultStatus::BackendUnavailable
        );
        assert!(result.finding.is_none());
        assert!(
            result
                .note
                .contains("EpiphanyRuntimeRoleWorkerResult typed document")
        );

        let _ = fs::remove_file(store);
    }

    #[test]
    fn reorient_result_requires_typed_worker_document() {
        let store = temp_store_path();
        seed_lifecycle_result(&store, "reorient-job", "reorientation", "completed");

        let result = load_core_epiphany_reorient_result_from_runtime_spine_job(
            "reorient-job",
            Some(store.as_path()),
        );

        assert_eq!(
            result.status,
            CoreEpiphanyCrrcResultStatus::BackendUnavailable
        );
        assert!(result.finding.is_none());
        assert!(
            result
                .note
                .contains("EpiphanyRuntimeReorientWorkerResult typed document")
        );

        let _ = fs::remove_file(store);
    }

    #[test]
    fn role_result_projects_failed_lifecycle_receipt_as_terminal() {
        let store = temp_store_path();
        seed_lifecycle_result(&store, "role-job", "modeling", "failed");

        let result = load_core_epiphany_role_result_from_runtime_spine_job(
            "role-job",
            Some(store.as_path()),
            EpiphanyRoleResultRoleId::Modeling,
        );

        assert_eq!(
            result.status,
            CoreEpiphanyCoordinatorRoleResultStatus::Failed
        );
        let finding = result.finding.expect("failed receipt should be visible");
        assert_eq!(finding.verdict.as_deref(), Some("failed"));
        assert_eq!(finding.runtime_job_id.as_deref(), Some("role-job"));
        assert!(
            finding
                .job_error
                .as_deref()
                .is_some_and(|error| error.contains("generic lifecycle result"))
        );
        assert!(result.note.contains("failed"));

        let _ = fs::remove_file(store);
    }

    #[test]
    fn reorient_result_projects_failed_lifecycle_receipt_as_terminal() {
        let store = temp_store_path();
        seed_lifecycle_result(&store, "reorient-job", "reorientation", "failed");

        let result = load_core_epiphany_reorient_result_from_runtime_spine_job(
            "reorient-job",
            Some(store.as_path()),
        );

        assert_eq!(result.status, CoreEpiphanyCrrcResultStatus::Failed);
        let finding = result.finding.expect("failed receipt should be visible");
        assert_eq!(finding.runtime_job_id.as_deref(), Some("reorient-job"));
        assert!(
            finding
                .job_error
                .as_deref()
                .is_some_and(|error| error.contains("generic lifecycle result"))
        );
        assert!(result.note.contains("failed"));

        let _ = fs::remove_file(store);
    }

    fn temp_store_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "epiphany-runtime-results-{}.cc",
            uuid::Uuid::new_v4()
        ))
    }

    fn seed_lifecycle_result(store: &Path, job_id: &str, role: &str, verdict: &str) {
        initialize_runtime_spine(
            store,
            RuntimeSpineInitOptions {
                runtime_id: "test-runtime".to_string(),
                display_name: "Test Runtime".to_string(),
                created_at: NOW.to_string(),
            },
        )
        .expect("initialize runtime");
        create_runtime_session(
            store,
            RuntimeSpineSessionOptions {
                session_id: "session-1".to_string(),
                objective: "test typed result boundary".to_string(),
                created_at: NOW.to_string(),
                coordinator_note: "test".to_string(),
            },
        )
        .expect("create session");
        create_runtime_job(
            store,
            RuntimeSpineJobOptions {
                job_id: job_id.to_string(),
                session_id: "session-1".to_string(),
                role: role.to_string(),
                created_at: NOW.to_string(),
                summary: "generic lifecycle job".to_string(),
                artifact_refs: Vec::new(),
            },
        )
        .expect("create job");
        complete_runtime_job(
            store,
            RuntimeSpineJobResultOptions {
                result_id: format!("result-{job_id}"),
                job_id: job_id.to_string(),
                completed_at: NOW.to_string(),
                verdict: verdict.to_string(),
                summary: "generic lifecycle result".to_string(),
                next_safe_move: "do not review generic lifecycle receipts".to_string(),
                evidence_refs: Vec::new(),
                artifact_refs: Vec::new(),
            },
        )
        .expect("complete job");
    }
}
