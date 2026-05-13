use std::path::Path;

use codex_app_server_protocol::ThreadEpiphanyReorientFinding;
use codex_app_server_protocol::ThreadEpiphanyReorientResultStatus;
use codex_app_server_protocol::ThreadEpiphanyRoleFinding;
use codex_app_server_protocol::ThreadEpiphanyRoleId;
use codex_app_server_protocol::ThreadEpiphanyRoleResultStatus;
use codex_protocol::protocol::EpiphanyRuntimeLink;
use codex_protocol::protocol::EpiphanyThreadState;
use epiphany_core::EpiphanyRuntimeJobSnapshot;
use epiphany_core::EpiphanyRuntimeJobStatus;
use epiphany_core::interpret_reorient_runtime_job_result;
use epiphany_core::interpret_role_runtime_job_result;
use epiphany_core::runtime_job_snapshot;

use crate::results::map_core_role_result_role_id;
use crate::results::map_protocol_reorient_finding;
use crate::results::map_protocol_role_finding;
use crate::results::render_epiphany_reorient_result_note;
use crate::results::render_epiphany_role_result_note;

pub fn role_finding_runtime_result_id(
    finding: &ThreadEpiphanyRoleFinding,
) -> Option<String> {
    finding.runtime_result_id.clone()
}

pub fn role_finding_runtime_job_id(finding: &ThreadEpiphanyRoleFinding) -> Option<String> {
    finding.runtime_job_id.clone()
}

pub fn reorient_finding_runtime_result_id(
    finding: &ThreadEpiphanyReorientFinding,
) -> Option<String> {
    finding.runtime_result_id.clone()
}

pub fn reorient_finding_runtime_job_id(
    finding: &ThreadEpiphanyReorientFinding,
) -> Option<String> {
    finding.runtime_job_id.clone()
}

pub fn load_epiphany_role_result_from_runtime_spine_job(
    job_id: &str,
    runtime_store_path: Option<&Path>,
    role_id: ThreadEpiphanyRoleId,
) -> (
    ThreadEpiphanyRoleResultStatus,
    Option<ThreadEpiphanyRoleFinding>,
    String,
) {
    let Some(runtime_store_path) = runtime_store_path else {
        return (
            ThreadEpiphanyRoleResultStatus::Pending,
            None,
            "Heartbeat activation owns this role specialist; no loaded runtime-spine store is available yet."
                .to_string(),
        );
    };
    let snapshot = match runtime_job_snapshot(runtime_store_path, job_id) {
        Ok(Some(snapshot)) => snapshot,
        Ok(None) => {
            return (
                ThreadEpiphanyRoleResultStatus::Pending,
                None,
                format!(
                    "Heartbeat runtime job {:?} has not reported typed state yet.",
                    job_id
                ),
            );
        }
        Err(err) => {
            return (
                ThreadEpiphanyRoleResultStatus::BackendUnavailable,
                None,
                format!(
                    "Failed to read heartbeat runtime-spine job {:?}: {err}",
                    job_id
                ),
            );
        }
    };
    let status = map_runtime_role_result_status(&snapshot);
    let finding = snapshot.result.as_ref().map(|result| {
        map_protocol_role_finding(
            role_id,
            interpret_role_runtime_job_result(map_core_role_result_role_id(role_id), result),
        )
    });
    let note = render_epiphany_role_result_note(role_id, status, finding.as_ref(), None);
    (status, finding, note)
}

pub fn load_epiphany_reorient_result_from_runtime_spine_job(
    job_id: &str,
    runtime_store_path: Option<&Path>,
) -> (
    ThreadEpiphanyReorientResultStatus,
    Option<ThreadEpiphanyReorientFinding>,
    String,
) {
    let Some(runtime_store_path) = runtime_store_path else {
        return (
            ThreadEpiphanyReorientResultStatus::Pending,
            None,
            "Heartbeat activation owns this reorientation worker; no loaded runtime-spine store is available yet."
                .to_string(),
        );
    };
    let snapshot = match runtime_job_snapshot(runtime_store_path, job_id) {
        Ok(Some(snapshot)) => snapshot,
        Ok(None) => {
            return (
                ThreadEpiphanyReorientResultStatus::Pending,
                None,
                format!(
                    "Heartbeat runtime job {:?} has not reported typed state yet.",
                    job_id
                ),
            );
        }
        Err(err) => {
            return (
                ThreadEpiphanyReorientResultStatus::BackendUnavailable,
                None,
                format!(
                    "Failed to read heartbeat runtime-spine job {:?}: {err}",
                    job_id
                ),
            );
        }
    };
    let status = map_runtime_reorient_result_status(&snapshot);
    let finding = snapshot
        .result
        .as_ref()
        .map(|result| map_protocol_reorient_finding(interpret_reorient_runtime_job_result(result)));
    let note = render_epiphany_reorient_result_note(status, finding.as_ref(), None);
    (status, finding, note)
}

fn map_runtime_role_result_status(
    snapshot: &EpiphanyRuntimeJobSnapshot,
) -> ThreadEpiphanyRoleResultStatus {
    match snapshot.job.status {
        EpiphanyRuntimeJobStatus::Queued => ThreadEpiphanyRoleResultStatus::Pending,
        EpiphanyRuntimeJobStatus::Running | EpiphanyRuntimeJobStatus::WaitingForReview => {
            ThreadEpiphanyRoleResultStatus::Running
        }
        EpiphanyRuntimeJobStatus::Completed => {
            if snapshot.result.is_some() {
                ThreadEpiphanyRoleResultStatus::Completed
            } else {
                ThreadEpiphanyRoleResultStatus::Pending
            }
        }
        EpiphanyRuntimeJobStatus::Failed => ThreadEpiphanyRoleResultStatus::Failed,
        EpiphanyRuntimeJobStatus::Cancelled => ThreadEpiphanyRoleResultStatus::Cancelled,
    }
}

fn map_runtime_reorient_result_status(
    snapshot: &EpiphanyRuntimeJobSnapshot,
) -> ThreadEpiphanyReorientResultStatus {
    match snapshot.job.status {
        EpiphanyRuntimeJobStatus::Queued => ThreadEpiphanyReorientResultStatus::Pending,
        EpiphanyRuntimeJobStatus::Running | EpiphanyRuntimeJobStatus::WaitingForReview => {
            ThreadEpiphanyReorientResultStatus::Running
        }
        EpiphanyRuntimeJobStatus::Completed => {
            if snapshot.result.is_some() {
                ThreadEpiphanyReorientResultStatus::Completed
            } else {
                ThreadEpiphanyReorientResultStatus::Pending
            }
        }
        EpiphanyRuntimeJobStatus::Failed => ThreadEpiphanyReorientResultStatus::Failed,
        EpiphanyRuntimeJobStatus::Cancelled => ThreadEpiphanyReorientResultStatus::Cancelled,
    }
}

pub async fn load_epiphany_role_result_snapshot(
    state: &EpiphanyThreadState,
    runtime_store_path: Option<&Path>,
    role_id: ThreadEpiphanyRoleId,
    binding_id: &str,
) -> (
    ThreadEpiphanyRoleResultStatus,
    Option<ThreadEpiphanyRoleFinding>,
    String,
) {
    if let Some(link) = latest_epiphany_runtime_link_for_binding(state, binding_id) {
        return load_epiphany_role_result_from_runtime_spine_job(
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
        return (
            ThreadEpiphanyRoleResultStatus::MissingBinding,
            None,
            "No matching Epiphany role specialist binding exists.".to_string(),
        );
    }
    (
        ThreadEpiphanyRoleResultStatus::BackendUnavailable,
        None,
        "Role binding has no runtime-spine job id; launch a runtime-linked role worker for typed results.".to_string(),
    )
}

pub async fn load_epiphany_reorient_result_snapshot(
    state: Option<&EpiphanyThreadState>,
    runtime_store_path: Option<&Path>,
    binding_id: &str,
) -> (
    ThreadEpiphanyReorientResultStatus,
    Option<ThreadEpiphanyReorientFinding>,
    String,
) {
    let Some(state) = state else {
        return (
            ThreadEpiphanyReorientResultStatus::MissingState,
            None,
            "No authoritative Epiphany state exists for this thread.".to_string(),
        );
    };
    if let Some(link) = latest_epiphany_runtime_link_for_binding(state, binding_id) {
        return load_epiphany_reorient_result_from_runtime_spine_job(
            link.runtime_job_id.as_str(),
            runtime_store_path,
        );
    }

    if !state
        .job_bindings
        .iter()
        .any(|binding| binding.id == binding_id)
    {
        return (
            ThreadEpiphanyReorientResultStatus::MissingBinding,
            None,
            "No matching Epiphany reorientation worker binding exists.".to_string(),
        );
    }
    (
        ThreadEpiphanyReorientResultStatus::BackendUnavailable,
        None,
        "Reorientation binding has no runtime-spine job id; launch a runtime-linked reorient worker for typed results.".to_string(),
    )
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
