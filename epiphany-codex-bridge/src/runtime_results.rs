use std::path::Path;

use codex_app_server_protocol::ThreadEpiphanyReorientFinding;
use codex_app_server_protocol::ThreadEpiphanyReorientResultStatus;
use codex_app_server_protocol::ThreadEpiphanyRoleFinding;
use codex_app_server_protocol::ThreadEpiphanyRoleId;
use codex_app_server_protocol::ThreadEpiphanyRoleResultStatus;
use codex_core::CodexThread;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::protocol::EpiphanyRuntimeLink;
use codex_protocol::protocol::EpiphanyThreadState;
use epiphany_core::EpiphanyRuntimeJobSnapshot;
use epiphany_core::EpiphanyRuntimeJobStatus;
use epiphany_core::interpret_runtime_reorient_worker_result;
use epiphany_core::interpret_runtime_role_worker_result;
use epiphany_core::runtime_job_snapshot;
use epiphany_core::runtime_reorient_worker_result;
use epiphany_core::runtime_role_worker_result;

use crate::results::map_core_role_result_role_id;
use crate::results::map_protocol_reorient_finding;
use crate::results::map_protocol_role_finding;
use crate::results::render_epiphany_reorient_result_note;
use crate::results::render_epiphany_role_result_note;

pub fn role_finding_runtime_result_id(finding: &ThreadEpiphanyRoleFinding) -> Option<String> {
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

pub fn reorient_finding_runtime_job_id(finding: &ThreadEpiphanyReorientFinding) -> Option<String> {
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
    let finding = if status == ThreadEpiphanyRoleResultStatus::Completed {
        match runtime_role_worker_result(runtime_store_path, job_id) {
            Ok(Some(result)) => Some(map_protocol_role_finding(
                role_id,
                interpret_runtime_role_worker_result(
                    map_core_role_result_role_id(role_id),
                    &result,
                ),
            )),
            Ok(None) => {
                return (
                    ThreadEpiphanyRoleResultStatus::BackendUnavailable,
                    None,
                    format!(
                        "Heartbeat runtime job {:?} completed without an EpiphanyRuntimeRoleWorkerResult typed document; generic lifecycle receipts are not reviewable findings.",
                        job_id
                    ),
                );
            }
            Err(err) => {
                return (
                    ThreadEpiphanyRoleResultStatus::BackendUnavailable,
                    None,
                    format!(
                        "Failed to read typed role worker result for heartbeat runtime job {:?}: {err}",
                        job_id
                    ),
                );
            }
        }
    } else {
        None
    };
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
    let finding = if status == ThreadEpiphanyReorientResultStatus::Completed {
        match runtime_reorient_worker_result(runtime_store_path, job_id) {
            Ok(Some(result)) => Some(map_protocol_reorient_finding(
                interpret_runtime_reorient_worker_result(&result),
            )),
            Ok(None) => {
                return (
                    ThreadEpiphanyReorientResultStatus::BackendUnavailable,
                    None,
                    format!(
                        "Heartbeat runtime job {:?} completed without an EpiphanyRuntimeReorientWorkerResult typed document; generic lifecycle receipts are not reviewable findings.",
                        job_id
                    ),
                );
            }
            Err(err) => {
                return (
                    ThreadEpiphanyReorientResultStatus::BackendUnavailable,
                    None,
                    format!(
                        "Failed to read typed reorientation worker result for heartbeat runtime job {:?}: {err}",
                        job_id
                    ),
                );
            }
        }
    } else {
        None
    };
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

pub async fn load_completed_epiphany_role_finding(
    thread: &CodexThread,
    state: &EpiphanyThreadState,
    role_id: ThreadEpiphanyRoleId,
    binding_id: &str,
) -> CodexResult<ThreadEpiphanyRoleFinding> {
    if let Some(link) = latest_epiphany_runtime_link_for_binding(state, binding_id) {
        let runtime_store_path = thread.epiphany_runtime_spine_store_path().await;
        let (status, finding, _note) = load_epiphany_role_result_from_runtime_spine_job(
            link.runtime_job_id.as_str(),
            Some(runtime_store_path.as_path()),
            role_id,
        );
        if status != ThreadEpiphanyRoleResultStatus::Completed {
            return Err(CodexErr::InvalidRequest(format!(
                "cannot accept role result while worker status is {:?}",
                status
            )));
        }
        return finding.ok_or_else(|| {
            CodexErr::InvalidRequest(
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
        return Err(CodexErr::InvalidRequest(format!(
            "epiphany role binding {:?} was not found",
            binding_id
        )));
    }

    Err(CodexErr::InvalidRequest(
        "role findings without runtime-spine results are unsupported; accept only typed runtime-spine results"
            .to_string(),
    ))
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

pub async fn load_completed_epiphany_reorient_finding(
    thread: &CodexThread,
    state: &EpiphanyThreadState,
    binding_id: &str,
) -> CodexResult<ThreadEpiphanyReorientFinding> {
    if let Some(link) = latest_epiphany_runtime_link_for_binding(state, binding_id) {
        let runtime_store_path = thread.epiphany_runtime_spine_store_path().await;
        let (status, finding, _note) = load_epiphany_reorient_result_from_runtime_spine_job(
            link.runtime_job_id.as_str(),
            Some(runtime_store_path.as_path()),
        );
        if status != ThreadEpiphanyReorientResultStatus::Completed {
            return Err(CodexErr::InvalidRequest(format!(
                "cannot accept reorientation result while worker status is {:?}",
                status
            )));
        }
        return finding.ok_or_else(|| {
            CodexErr::InvalidRequest(
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
        return Err(CodexErr::InvalidRequest(format!(
            "epiphany reorientation binding {:?} was not found",
            binding_id
        )));
    }

    Err(CodexErr::InvalidRequest(
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

    use codex_app_server_protocol::ThreadEpiphanyReorientResultStatus;
    use codex_app_server_protocol::ThreadEpiphanyRoleId;
    use codex_app_server_protocol::ThreadEpiphanyRoleResultStatus;
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

        let (status, finding, note) = load_epiphany_role_result_from_runtime_spine_job(
            "role-job",
            Some(store.as_path()),
            ThreadEpiphanyRoleId::Modeling,
        );

        assert_eq!(status, ThreadEpiphanyRoleResultStatus::BackendUnavailable);
        assert!(finding.is_none());
        assert!(note.contains("EpiphanyRuntimeRoleWorkerResult typed document"));

        let _ = fs::remove_file(store);
    }

    #[test]
    fn reorient_result_requires_typed_worker_document() {
        let store = temp_store_path();
        seed_completed_lifecycle_result(&store, "reorient-job", "reorientation");

        let (status, finding, note) = load_epiphany_reorient_result_from_runtime_spine_job(
            "reorient-job",
            Some(store.as_path()),
        );

        assert_eq!(
            status,
            ThreadEpiphanyReorientResultStatus::BackendUnavailable
        );
        assert!(finding.is_none());
        assert!(note.contains("EpiphanyRuntimeReorientWorkerResult typed document"));

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
