use codex_app_server_protocol::ThreadEpiphanyReorientFinding;
use codex_app_server_protocol::ThreadEpiphanyReorientResultStatus;
use codex_app_server_protocol::ThreadEpiphanyRoleFinding;
use codex_app_server_protocol::ThreadEpiphanyRoleId;
use codex_app_server_protocol::ThreadEpiphanyRoleResultStatus;
use codex_app_server_protocol::ThreadEpiphanyRoleSelfPersistenceReview;
use codex_app_server_protocol::ThreadEpiphanyRoleSelfPersistenceStatus;
use codex_app_server_protocol::ThreadEpiphanyUpdatePatch;
use epiphany_core::EpiphanyReorientFindingInterpretation;
use epiphany_core::EpiphanyRoleFindingInterpretation;
use epiphany_core::EpiphanyRoleResultRoleId;
use epiphany_core::EpiphanyRoleSelfPersistenceReview as CoreEpiphanyRoleSelfPersistenceReview;
use epiphany_core::EpiphanyRoleSelfPersistenceStatus as CoreEpiphanyRoleSelfPersistenceStatus;
use epiphany_core::EpiphanyRoleStatePatchDocument;

pub fn map_protocol_reorient_finding(
    finding: EpiphanyReorientFindingInterpretation,
) -> ThreadEpiphanyReorientFinding {
    ThreadEpiphanyReorientFinding {
        mode: finding.mode,
        summary: finding.summary,
        next_safe_move: finding.next_safe_move,
        checkpoint_still_valid: finding.checkpoint_still_valid,
        files_inspected: finding.files_inspected,
        frontier_node_ids: finding.frontier_node_ids,
        evidence_ids: finding.evidence_ids,
        artifact_refs: finding.artifact_refs,
        runtime_result_id: finding.runtime_result_id,
        runtime_job_id: finding.runtime_job_id,
        job_error: finding.job_error,
        item_error: finding.item_error,
    }
}

pub fn map_protocol_role_finding(
    role_id: ThreadEpiphanyRoleId,
    finding: EpiphanyRoleFindingInterpretation,
) -> ThreadEpiphanyRoleFinding {
    let state_patch = finding.state_patch.map(map_protocol_update_patch);
    ThreadEpiphanyRoleFinding {
        role_id,
        verdict: finding.verdict,
        summary: finding.summary,
        next_safe_move: finding.next_safe_move,
        checkpoint_summary: finding.checkpoint_summary,
        scratch_summary: finding.scratch_summary,
        files_inspected: finding.files_inspected,
        frontier_node_ids: finding.frontier_node_ids,
        evidence_ids: finding.evidence_ids,
        artifact_refs: finding.artifact_refs,
        runtime_result_id: finding.runtime_result_id,
        runtime_job_id: finding.runtime_job_id,
        open_questions: finding.open_questions,
        evidence_gaps: finding.evidence_gaps,
        risks: finding.risks,
        state_patch,
        self_patch: finding.self_patch.map(|patch| {
            serde_json::to_value(patch)
                .expect("AgentSelfPatch is a serializable protocol projection")
        }),
        self_persistence: finding
            .self_persistence
            .map(map_protocol_role_self_persistence_review),
        job_error: finding.job_error,
        item_error: finding.item_error,
    }
}

fn map_protocol_update_patch(patch: EpiphanyRoleStatePatchDocument) -> ThreadEpiphanyUpdatePatch {
    ThreadEpiphanyUpdatePatch {
        objective: patch.objective,
        active_subgoal_id: patch.active_subgoal_id,
        subgoals: patch.subgoals,
        invariants: patch.invariants,
        graphs: patch.graphs,
        graph_frontier: patch.graph_frontier,
        graph_checkpoint: patch.graph_checkpoint,
        scratch: patch.scratch,
        investigation_checkpoint: patch.investigation_checkpoint,
        job_bindings: patch.job_bindings,
        acceptance_receipts: patch.acceptance_receipts,
        runtime_links: patch.runtime_links,
        observations: patch.observations,
        evidence: patch.evidence,
        churn: patch.churn,
        mode: patch.mode,
        planning: patch.planning,
    }
}

pub fn map_core_role_result_role_id(
    role_id: ThreadEpiphanyRoleId,
) -> EpiphanyRoleResultRoleId {
    match role_id {
        ThreadEpiphanyRoleId::Implementation => EpiphanyRoleResultRoleId::Implementation,
        ThreadEpiphanyRoleId::Imagination => EpiphanyRoleResultRoleId::Imagination,
        ThreadEpiphanyRoleId::Modeling => EpiphanyRoleResultRoleId::Modeling,
        ThreadEpiphanyRoleId::Verification => EpiphanyRoleResultRoleId::Verification,
        ThreadEpiphanyRoleId::Reorientation => EpiphanyRoleResultRoleId::Reorientation,
    }
}

fn map_protocol_role_self_persistence_review(
    review: CoreEpiphanyRoleSelfPersistenceReview,
) -> ThreadEpiphanyRoleSelfPersistenceReview {
    ThreadEpiphanyRoleSelfPersistenceReview {
        status: match review.status {
            CoreEpiphanyRoleSelfPersistenceStatus::Missing => {
                ThreadEpiphanyRoleSelfPersistenceStatus::Missing
            }
            CoreEpiphanyRoleSelfPersistenceStatus::Accepted => {
                ThreadEpiphanyRoleSelfPersistenceStatus::Accepted
            }
            CoreEpiphanyRoleSelfPersistenceStatus::Rejected => {
                ThreadEpiphanyRoleSelfPersistenceStatus::Rejected
            }
        },
        target_agent_id: review.target_agent_id,
        target_path: review.target_path,
        reasons: review.reasons,
    }
}

pub fn render_epiphany_role_result_note(
    role_id: ThreadEpiphanyRoleId,
    status: ThreadEpiphanyRoleResultStatus,
    finding: Option<&ThreadEpiphanyRoleFinding>,
    item_error: Option<&str>,
) -> String {
    match status {
        ThreadEpiphanyRoleResultStatus::Completed => {
            if let Some(finding) = finding {
                let next = finding.next_safe_move.as_deref().unwrap_or("not supplied");
                let self_note =
                    render_epiphany_self_persistence_note(finding.self_persistence.as_ref())
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
        ThreadEpiphanyRoleResultStatus::Failed => item_error
            .map(|error| format!("{:?} role specialist failed: {error}", role_id))
            .unwrap_or_else(|| format!("{:?} role specialist failed.", role_id)),
        ThreadEpiphanyRoleResultStatus::Cancelled => {
            format!(
                "{:?} role specialist was cancelled before producing a result.",
                role_id
            )
        }
        ThreadEpiphanyRoleResultStatus::Running => {
            format!("{:?} role specialist is still running.", role_id)
        }
        ThreadEpiphanyRoleResultStatus::Pending => {
            format!(
                "{:?} role specialist has not produced a result yet.",
                role_id
            )
        }
        ThreadEpiphanyRoleResultStatus::MissingState => {
            "No authoritative Epiphany state exists for this thread.".to_string()
        }
        ThreadEpiphanyRoleResultStatus::MissingBinding => {
            "No matching Epiphany role specialist binding exists.".to_string()
        }
        ThreadEpiphanyRoleResultStatus::BackendUnavailable => {
            "The bound runtime backend is unavailable.".to_string()
        }
        ThreadEpiphanyRoleResultStatus::BackendMissing => {
            "The bound runtime backend job or item is missing.".to_string()
        }
    }
}

fn render_epiphany_self_persistence_note(
    review: Option<&ThreadEpiphanyRoleSelfPersistenceReview>,
) -> Option<String> {
    let review = review?;
    match review.status {
        ThreadEpiphanyRoleSelfPersistenceStatus::Missing => None,
        ThreadEpiphanyRoleSelfPersistenceStatus::Accepted => Some(format!(
            "Self persistence request is acceptable for {}.",
            review
                .target_agent_id
                .as_deref()
                .unwrap_or("the role memory file")
        )),
        ThreadEpiphanyRoleSelfPersistenceStatus::Rejected => {
            let reasons = if review.reasons.is_empty() {
                "no reason recorded".to_string()
            } else {
                review.reasons.join("; ")
            };
            Some(format!("Self persistence request was refused: {reasons}."))
        }
    }
}

pub fn render_epiphany_reorient_result_note(
    status: ThreadEpiphanyReorientResultStatus,
    finding: Option<&ThreadEpiphanyReorientFinding>,
    item_error: Option<&str>,
) -> String {
    match status {
        ThreadEpiphanyReorientResultStatus::Completed => {
            if let Some(finding) = finding {
                let next = finding.next_safe_move.as_deref().unwrap_or("not supplied");
                format!("Reorientation worker completed. Next safe move: {next}")
            } else {
                "Reorientation worker completed, but no structured result was recorded.".to_string()
            }
        }
        ThreadEpiphanyReorientResultStatus::Failed => item_error
            .map(|error| format!("Reorientation worker failed: {error}"))
            .unwrap_or_else(|| "Reorientation worker failed.".to_string()),
        ThreadEpiphanyReorientResultStatus::Cancelled => {
            "Reorientation worker was cancelled before producing a result.".to_string()
        }
        ThreadEpiphanyReorientResultStatus::Running => {
            "Reorientation worker is still running.".to_string()
        }
        ThreadEpiphanyReorientResultStatus::Pending => {
            "Reorientation worker has not produced a result yet.".to_string()
        }
        ThreadEpiphanyReorientResultStatus::MissingState => {
            "No authoritative Epiphany state exists for this thread.".to_string()
        }
        ThreadEpiphanyReorientResultStatus::MissingBinding => {
            "No matching Epiphany reorientation worker binding exists.".to_string()
        }
        ThreadEpiphanyReorientResultStatus::BackendUnavailable => {
            "The bound runtime backend is unavailable.".to_string()
        }
        ThreadEpiphanyReorientResultStatus::BackendMissing => {
            "The bound runtime backend job or item is missing.".to_string()
        }
    }
}
