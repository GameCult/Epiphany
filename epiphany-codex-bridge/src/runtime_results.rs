use std::path::Path;

use epiphany_core::EpiphanyCoordinatorRoleResultStatus as CoreEpiphanyCoordinatorRoleResultStatus;
use epiphany_core::EpiphanyCrrcResultStatus as CoreEpiphanyCrrcResultStatus;
use epiphany_core::EpiphanyLaunchOrganContract;
use epiphany_core::EpiphanyReceiptEffectKind;
use epiphany_core::EpiphanyReorientFindingInterpretation;
use epiphany_core::EpiphanyRoleFindingInterpretation;
use epiphany_core::EpiphanyRoleResultRoleId;
use epiphany_core::MIND_GATEWAY_REVIEW_TYPE;
use epiphany_core::runtime_worker_launch_request;
use epiphany_state_model::EpiphanyRuntimeLink;
use epiphany_state_model::EpiphanyThreadState;

use crate::error::EpiphanyBridgeError;
use crate::error::Result as BridgeResult;

pub type EpiphanyRoleResultSnapshot = epiphany_core::EpiphanyCoordinatorRoleResultSnapshot;
pub type EpiphanyReorientResultSnapshot = epiphany_core::EpiphanyCoordinatorReorientResultSnapshot;

pub fn load_core_epiphany_role_result_from_runtime_spine_job(
    job_id: &str,
    runtime_store_path: Option<&Path>,
    role_id: EpiphanyRoleResultRoleId,
) -> EpiphanyRoleResultSnapshot {
    epiphany_core::read_runtime_role_result(runtime_store_path, job_id, role_id)
}

pub fn load_core_epiphany_reorient_result_from_runtime_spine_job(
    job_id: &str,
    runtime_store_path: Option<&Path>,
) -> EpiphanyReorientResultSnapshot {
    epiphany_core::read_runtime_reorient_result(runtime_store_path, job_id)
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
            "epiphany-runtime-results-{}.msgpack",
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
