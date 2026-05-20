use std::path::Path;

use codex_app_server_protocol::ThreadEpiphanyReorientResultStatus;
use codex_app_server_protocol::ThreadEpiphanyRoleId;
use codex_app_server_protocol::ThreadEpiphanyRoleResultStatus;
use epiphany_core::EpiphanyCoordinatorRoleResultStatus as CoreEpiphanyCoordinatorRoleResultStatus;
use epiphany_core::EpiphanyCrrcResultStatus as CoreEpiphanyCrrcResultStatus;
use epiphany_core::EpiphanyReorientFindingInterpretation;
use epiphany_core::EpiphanyRoleFindingInterpretation;
use epiphany_core::EpiphanyRuntimeJobSnapshot;
use epiphany_core::EpiphanyRuntimeJobStatus;
use epiphany_core::interpret_runtime_reorient_worker_result;
use epiphany_core::interpret_runtime_role_worker_result;
use epiphany_core::runtime_job_snapshot;
use epiphany_core::runtime_reorient_worker_result;
use epiphany_core::runtime_role_worker_result;
use epiphany_state_model::EpiphanyRuntimeLink;
use epiphany_state_model::EpiphanyThreadState;

use crate::error::EpiphanyBridgeError;
use crate::error::Result as BridgeResult;
use crate::results::map_core_role_result_role_id;
use crate::results::render_core_reorient_result_note;
use crate::results::render_core_role_result_note;

#[derive(Debug, Clone)]
pub struct EpiphanyRoleResultSnapshot {
    pub status: CoreEpiphanyCoordinatorRoleResultStatus,
    pub finding: Option<EpiphanyRoleFindingInterpretation>,
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
    role_id: ThreadEpiphanyRoleId,
) -> EpiphanyRoleResultSnapshot {
    let core_role_id = map_core_role_result_role_id(role_id);
    let Some(runtime_store_path) = runtime_store_path else {
        return EpiphanyRoleResultSnapshot {
            status: CoreEpiphanyCoordinatorRoleResultStatus::Pending,
            finding: None,
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
                note: format!(
                    "Failed to read heartbeat runtime-spine job {:?}: {err}",
                    job_id
                ),
            };
        }
    };
    let status = map_runtime_role_result_status(&snapshot);
    let finding = if status == CoreEpiphanyCoordinatorRoleResultStatus::Completed {
        match runtime_role_worker_result(runtime_store_path, job_id) {
            Ok(Some(result)) => Some(interpret_runtime_role_worker_result(core_role_id, &result)),
            Ok(None) => {
                return EpiphanyRoleResultSnapshot {
                    status: CoreEpiphanyCoordinatorRoleResultStatus::BackendUnavailable,
                    finding: None,
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
                    note: format!(
                        "Failed to read typed role worker result for heartbeat runtime job {:?}: {err}",
                        job_id
                    ),
                };
            }
        }
    } else {
        None
    };
    let note = render_core_role_result_note(core_role_id, status, finding.as_ref(), None);
    EpiphanyRoleResultSnapshot {
        status,
        finding,
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
    let finding = if status == CoreEpiphanyCrrcResultStatus::Completed {
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
    } else {
        None
    };
    let note = render_core_reorient_result_note(status, finding.as_ref(), None);
    EpiphanyReorientResultSnapshot {
        status,
        finding,
        note,
    }
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

pub fn map_protocol_role_result_status(
    status: CoreEpiphanyCoordinatorRoleResultStatus,
) -> ThreadEpiphanyRoleResultStatus {
    match status {
        CoreEpiphanyCoordinatorRoleResultStatus::MissingState => {
            ThreadEpiphanyRoleResultStatus::MissingState
        }
        CoreEpiphanyCoordinatorRoleResultStatus::MissingBinding => {
            ThreadEpiphanyRoleResultStatus::MissingBinding
        }
        CoreEpiphanyCoordinatorRoleResultStatus::BackendUnavailable => {
            ThreadEpiphanyRoleResultStatus::BackendUnavailable
        }
        CoreEpiphanyCoordinatorRoleResultStatus::BackendMissing => {
            ThreadEpiphanyRoleResultStatus::BackendMissing
        }
        CoreEpiphanyCoordinatorRoleResultStatus::Pending => ThreadEpiphanyRoleResultStatus::Pending,
        CoreEpiphanyCoordinatorRoleResultStatus::Running => ThreadEpiphanyRoleResultStatus::Running,
        CoreEpiphanyCoordinatorRoleResultStatus::Completed => {
            ThreadEpiphanyRoleResultStatus::Completed
        }
        CoreEpiphanyCoordinatorRoleResultStatus::Failed => ThreadEpiphanyRoleResultStatus::Failed,
        CoreEpiphanyCoordinatorRoleResultStatus::Cancelled => {
            ThreadEpiphanyRoleResultStatus::Cancelled
        }
    }
}

pub fn map_protocol_reorient_result_status(
    status: CoreEpiphanyCrrcResultStatus,
) -> ThreadEpiphanyReorientResultStatus {
    match status {
        CoreEpiphanyCrrcResultStatus::MissingState => {
            ThreadEpiphanyReorientResultStatus::MissingState
        }
        CoreEpiphanyCrrcResultStatus::MissingBinding => {
            ThreadEpiphanyReorientResultStatus::MissingBinding
        }
        CoreEpiphanyCrrcResultStatus::BackendUnavailable => {
            ThreadEpiphanyReorientResultStatus::BackendUnavailable
        }
        CoreEpiphanyCrrcResultStatus::BackendMissing => {
            ThreadEpiphanyReorientResultStatus::BackendMissing
        }
        CoreEpiphanyCrrcResultStatus::Pending => ThreadEpiphanyReorientResultStatus::Pending,
        CoreEpiphanyCrrcResultStatus::Running => ThreadEpiphanyReorientResultStatus::Running,
        CoreEpiphanyCrrcResultStatus::Completed => ThreadEpiphanyReorientResultStatus::Completed,
        CoreEpiphanyCrrcResultStatus::Failed => ThreadEpiphanyReorientResultStatus::Failed,
        CoreEpiphanyCrrcResultStatus::Cancelled => ThreadEpiphanyReorientResultStatus::Cancelled,
    }
}

pub async fn load_core_epiphany_role_result_snapshot(
    state: &EpiphanyThreadState,
    runtime_store_path: Option<&Path>,
    role_id: ThreadEpiphanyRoleId,
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
            note: "No matching Epiphany role specialist binding exists.".to_string(),
        };
    }
    EpiphanyRoleResultSnapshot {
        status: CoreEpiphanyCoordinatorRoleResultStatus::BackendUnavailable,
        finding: None,
        note: "Role binding has no runtime-spine job id; launch a runtime-linked role worker for typed results.".to_string(),
    }
}

pub fn load_completed_core_epiphany_role_finding(
    runtime_store_path: Option<&Path>,
    state: &EpiphanyThreadState,
    role_id: ThreadEpiphanyRoleId,
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

    use codex_app_server_protocol::ThreadEpiphanyRoleId;
    use epiphany_core::EpiphanyCoordinatorRoleResultStatus as CoreEpiphanyCoordinatorRoleResultStatus;
    use epiphany_core::EpiphanyCrrcResultStatus as CoreEpiphanyCrrcResultStatus;
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
        seed_completed_lifecycle_result(&store, "role-job", "modeling");

        let result = load_core_epiphany_role_result_from_runtime_spine_job(
            "role-job",
            Some(store.as_path()),
            ThreadEpiphanyRoleId::Modeling,
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
        seed_completed_lifecycle_result(&store, "reorient-job", "reorientation");

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

    fn temp_store_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "epiphany-runtime-results-{}.msgpack",
            uuid::Uuid::new_v4()
        ))
    }

    fn seed_completed_lifecycle_result(store: &Path, job_id: &str, role: &str) {
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
                verdict: "completed".to_string(),
                summary: "generic lifecycle result".to_string(),
                next_safe_move: "do not review generic lifecycle receipts".to_string(),
                evidence_refs: Vec::new(),
                artifact_refs: Vec::new(),
            },
        )
        .expect("complete job");
    }
}
