use crate::*;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct EpiphanyCoordinatorReorientResultSnapshot {
    pub status: EpiphanyCrrcResultStatus,
    pub finding: Option<EpiphanyReorientFindingInterpretation>,
    pub note: String,
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
                "Resident Self owns this reorientation worker; no loaded runtime-spine store is available yet.",
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
                    "Resident runtime job {job_id:?} has not reported typed state yet."
                )),
            );
        }
        Err(error) => {
            return reorient_snapshot(
                EpiphanyCrrcResultStatus::BackendUnavailable,
                None,
                Some(&format!(
                    "Failed to read resident runtime-spine job {job_id:?}: {error}"
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
                            "Resident runtime job {job_id:?} completed without an EpiphanyRuntimeReorientWorkerResult typed document; generic lifecycle receipts are not reviewable findings."
                        )),
                    );
                }
                Err(error) => {
                    return reorient_snapshot(
                        EpiphanyCrrcResultStatus::BackendUnavailable,
                        None,
                        Some(&format!(
                            "Failed to read typed reorientation worker result for resident runtime job {job_id:?}: {error}"
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
        evidence_ids: Vec::new(),
        artifact_refs: Vec::new(),
        runtime_result_id: Some(result.result_id.clone()),
        runtime_job_id: Some(result.job_id.clone()),
        job_error: Some(result.summary.clone()),
        item_error: None,
    }
}

fn reorient_result_status(snapshot: &EpiphanyRuntimeJobSnapshot) -> EpiphanyCrrcResultStatus {
    match snapshot.job.status {
        EpiphanyRuntimeJobStatus::Queued => EpiphanyCrrcResultStatus::Pending,
        EpiphanyRuntimeJobStatus::Completed if snapshot.result.is_some() => {
            EpiphanyCrrcResultStatus::Completed
        }
        EpiphanyRuntimeJobStatus::Completed => EpiphanyCrrcResultStatus::Pending,
        EpiphanyRuntimeJobStatus::Failed => EpiphanyCrrcResultStatus::Failed,
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
