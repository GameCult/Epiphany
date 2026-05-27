use epiphany_core::EpiphanyCoordinatorRoleResultStatus as CoreEpiphanyCoordinatorRoleResultStatus;
use epiphany_core::EpiphanyCrrcResultStatus as CoreEpiphanyCrrcResultStatus;
use epiphany_core::EpiphanyReorientFindingInterpretation;
use epiphany_core::EpiphanyRoleFindingInterpretation;
use epiphany_core::EpiphanyRoleResultRoleId;
use epiphany_core::EpiphanyRoleSelfPersistenceReview as CoreEpiphanyRoleSelfPersistenceReview;
use epiphany_core::EpiphanyRoleSelfPersistenceStatus as CoreEpiphanyRoleSelfPersistenceStatus;

pub fn render_core_role_result_note(
    role_id: EpiphanyRoleResultRoleId,
    status: CoreEpiphanyCoordinatorRoleResultStatus,
    finding: Option<&EpiphanyRoleFindingInterpretation>,
    item_error: Option<&str>,
) -> String {
    match status {
        CoreEpiphanyCoordinatorRoleResultStatus::Completed => {
            if let Some(finding) = finding {
                let next = finding.next_safe_move.as_deref().unwrap_or("not supplied");
                let self_note =
                    render_core_self_persistence_note(finding.self_persistence.as_ref())
                        .map(|note| format!(" {note}"))
                        .unwrap_or_default();
                if let Some(item_error) = finding.item_error.as_deref().or(item_error) {
                    format!(
                        "{:?} role specialist completed, but the finding needs repair: {item_error}. Next safe move: {next}.{self_note}",
                        role_id
                    )
                } else {
                    format!(
                        "{:?} role specialist completed. Next safe move: {next}.{self_note}",
                        role_id
                    )
                }
            } else {
                format!(
                    "{:?} role specialist completed, but no structured result was recorded.",
                    role_id
                )
            }
        }
        CoreEpiphanyCoordinatorRoleResultStatus::Failed => item_error
            .map(|error| format!("{:?} role specialist failed: {error}", role_id))
            .unwrap_or_else(|| format!("{:?} role specialist failed.", role_id)),
        CoreEpiphanyCoordinatorRoleResultStatus::Cancelled => {
            format!(
                "{:?} role specialist was cancelled before producing a result.",
                role_id
            )
        }
        CoreEpiphanyCoordinatorRoleResultStatus::Running => {
            format!("{:?} role specialist is still running.", role_id)
        }
        CoreEpiphanyCoordinatorRoleResultStatus::Pending => {
            format!(
                "{:?} role specialist has not produced a result yet.",
                role_id
            )
        }
        CoreEpiphanyCoordinatorRoleResultStatus::MissingState => {
            "No authoritative Epiphany state exists for this thread.".to_string()
        }
        CoreEpiphanyCoordinatorRoleResultStatus::MissingBinding => {
            "No matching Epiphany role specialist binding exists.".to_string()
        }
        CoreEpiphanyCoordinatorRoleResultStatus::BackendUnavailable => {
            "The bound runtime backend is unavailable.".to_string()
        }
        CoreEpiphanyCoordinatorRoleResultStatus::BackendMissing => {
            "The bound runtime backend job or item is missing.".to_string()
        }
    }
}

fn render_core_self_persistence_note(
    review: Option<&CoreEpiphanyRoleSelfPersistenceReview>,
) -> Option<String> {
    let review = review?;
    match review.status {
        CoreEpiphanyRoleSelfPersistenceStatus::Missing => None,
        CoreEpiphanyRoleSelfPersistenceStatus::Accepted => Some(format!(
            "Self persistence request is acceptable for {}.",
            review
                .target_agent_id
                .as_deref()
                .unwrap_or("the role memory file")
        )),
        CoreEpiphanyRoleSelfPersistenceStatus::Rejected => {
            let reasons = if review.reasons.is_empty() {
                "no reason recorded".to_string()
            } else {
                review.reasons.join("; ")
            };
            Some(format!("Self persistence request was refused: {reasons}."))
        }
    }
}

pub fn render_core_reorient_result_note(
    status: CoreEpiphanyCrrcResultStatus,
    finding: Option<&EpiphanyReorientFindingInterpretation>,
    item_error: Option<&str>,
) -> String {
    match status {
        CoreEpiphanyCrrcResultStatus::Completed => {
            if let Some(finding) = finding {
                let next = finding.next_safe_move.as_deref().unwrap_or("not supplied");
                format!("Reorientation worker completed. Next safe move: {next}")
            } else {
                "Reorientation worker completed, but no structured result was recorded.".to_string()
            }
        }
        CoreEpiphanyCrrcResultStatus::Failed => item_error
            .map(|error| format!("Reorientation worker failed: {error}"))
            .unwrap_or_else(|| "Reorientation worker failed.".to_string()),
        CoreEpiphanyCrrcResultStatus::Cancelled => {
            "Reorientation worker was cancelled before producing a result.".to_string()
        }
        CoreEpiphanyCrrcResultStatus::Running => {
            "Reorientation worker is still running.".to_string()
        }
        CoreEpiphanyCrrcResultStatus::Pending => {
            "Reorientation worker has not produced a result yet.".to_string()
        }
        CoreEpiphanyCrrcResultStatus::MissingState => {
            "No authoritative Epiphany state exists for this thread.".to_string()
        }
        CoreEpiphanyCrrcResultStatus::MissingBinding => {
            "No matching Epiphany reorientation worker binding exists.".to_string()
        }
        CoreEpiphanyCrrcResultStatus::BackendUnavailable => {
            "The bound runtime backend is unavailable.".to_string()
        }
        CoreEpiphanyCrrcResultStatus::BackendMissing => {
            "The bound runtime backend job or item is missing.".to_string()
        }
    }
}
