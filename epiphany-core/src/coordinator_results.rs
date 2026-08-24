use crate::*;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyReorientResultStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct EpiphanyCoordinatorReorientResultSnapshot {
    pub status: EpiphanyReorientResultStatus,
    pub finding: Option<EpiphanyReorientFindingInterpretation>,
    pub note: String,
}
pub fn read_runtime_reorient_result(
    runtime_store_path: &Path,
    job_id: &str,
) -> Result<EpiphanyCoordinatorReorientResultSnapshot> {
    let mut cache = runtime_spine_cache(runtime_store_path)?;
    cache.pull_all_backing_stores()?;
    let job = match cache.get::<EpiphanyRuntimeJob>(job_id)? {
        Some(job) => job,
        None => {
            return Ok(reorient_snapshot(
                EpiphanyReorientResultStatus::Pending,
                None,
                Some(&format!(
                    "Resident runtime job {job_id:?} has not reported typed state yet."
                )),
            ));
        }
    };
    let status = reorient_result_status(&job.status);
    let finding = match status {
        EpiphanyReorientResultStatus::Completed => Some(interpret_runtime_reorient_worker_result(
            &cache
                .get::<EpiphanyRuntimeReorientWorkerResult>(job_id)?
                .ok_or_else(|| {
                    anyhow!(
                        "resident runtime job {job_id:?} completed without its typed reorientation result"
                    )
                })?,
        )),
        EpiphanyReorientResultStatus::Failed => cache
            .get::<EpiphanyRuntimeReorientWorkerResult>(job_id)?
            .as_ref()
            .map(interpret_runtime_reorient_worker_result),
        EpiphanyReorientResultStatus::Pending => None,
    };
    Ok(reorient_snapshot(status, finding, None))
}

fn reorient_snapshot(
    status: EpiphanyReorientResultStatus,
    finding: Option<EpiphanyReorientFindingInterpretation>,
    note_override: Option<&str>,
) -> EpiphanyCoordinatorReorientResultSnapshot {
    let note = note_override
        .map(str::to_string)
        .unwrap_or_else(|| render_reorient_result_note(status, finding.as_ref()));
    EpiphanyCoordinatorReorientResultSnapshot {
        status,
        finding,
        note,
    }
}

fn reorient_result_status(status: &EpiphanyRuntimeJobStatus) -> EpiphanyReorientResultStatus {
    match status {
        EpiphanyRuntimeJobStatus::Queued => EpiphanyReorientResultStatus::Pending,
        EpiphanyRuntimeJobStatus::Completed => EpiphanyReorientResultStatus::Completed,
        EpiphanyRuntimeJobStatus::Failed => EpiphanyReorientResultStatus::Failed,
    }
}

fn render_reorient_result_note(
    status: EpiphanyReorientResultStatus,
    finding: Option<&EpiphanyReorientFindingInterpretation>,
) -> String {
    match status {
        EpiphanyReorientResultStatus::Completed => format!(
            "Reorientation worker completed. Next safe move: {}",
            finding
                .and_then(|finding| finding.next_safe_move.as_deref())
                .unwrap_or("not supplied")
        ),
        EpiphanyReorientResultStatus::Failed => finding
            .and_then(|finding| finding.job_error.as_deref())
            .map(|error| format!("Reorientation worker failed: {error}"))
            .unwrap_or_else(|| "Reorientation worker failed.".to_string()),
        EpiphanyReorientResultStatus::Pending => {
            "Reorientation worker has not produced a result yet.".to_string()
        }
    }
}
