use std::path::Path;

use codex_app_server_protocol::ThreadEpiphanyRoleFinding;
use codex_app_server_protocol::ThreadEpiphanyRoleId;
use codex_app_server_protocol::ThreadEpiphanyUpdatePatch;
use codex_core::CodexThread;
use codex_core::latest_epiphany_state_from_rollout_items;
use codex_protocol::protocol::EpiphanyThreadState;
use epiphany_core::EpiphanyRoleStatePatchDocument;

use super::epiphany_mutation_routes::imagination_role_accept_patch_errors;
use super::epiphany_mutation_routes::modeling_role_accept_patch_errors;
use super::read_rollout_items_from_rollout;

pub(super) async fn live_thread_epiphany_state(
    thread: &CodexThread,
) -> Option<EpiphanyThreadState> {
    let mut epiphany_state = thread.epiphany_state().await;
    if let Some(state) = epiphany_state.as_mut()
        && state.retrieval.is_none()
    {
        state.retrieval = Some(thread.epiphany_retrieval_state().await);
    }
    epiphany_state
}

pub(super) async fn client_visible_live_thread_epiphany_state(
    thread: &CodexThread,
    fallback: EpiphanyThreadState,
) -> EpiphanyThreadState {
    live_thread_epiphany_state(thread).await.unwrap_or(fallback)
}

pub(super) fn core_state_patch_from_protocol(
    patch: &ThreadEpiphanyUpdatePatch,
) -> EpiphanyRoleStatePatchDocument {
    EpiphanyRoleStatePatchDocument {
        objective: patch.objective.clone(),
        active_subgoal_id: patch.active_subgoal_id.clone(),
        subgoals: patch.subgoals.clone(),
        invariants: patch.invariants.clone(),
        graphs: patch.graphs.clone(),
        graph_frontier: patch.graph_frontier.clone(),
        graph_checkpoint: patch.graph_checkpoint.clone(),
        scratch: patch.scratch.clone(),
        investigation_checkpoint: patch.investigation_checkpoint.clone(),
        job_bindings: patch.job_bindings.clone(),
        acceptance_receipts: patch.acceptance_receipts.clone(),
        runtime_links: patch.runtime_links.clone(),
        observations: patch.observations.clone(),
        evidence: patch.evidence.clone(),
        churn: patch.churn.clone(),
        mode: patch.mode.clone(),
        planning: patch.planning.clone(),
    }
}

pub(super) fn epiphany_modeling_finding_has_reviewable_state_patch(
    finding: &ThreadEpiphanyRoleFinding,
) -> bool {
    finding.role_id == ThreadEpiphanyRoleId::Modeling
        && finding
            .state_patch
            .as_ref()
            .is_some_and(|patch| modeling_role_accept_patch_errors(patch).is_empty())
}

#[cfg(test)]
pub(super) fn epiphany_imagination_finding_has_reviewable_state_patch(
    finding: &ThreadEpiphanyRoleFinding,
) -> bool {
    finding.role_id == ThreadEpiphanyRoleId::Imagination
        && finding
            .state_patch
            .as_ref()
            .is_some_and(|patch| imagination_role_accept_patch_errors(patch).is_empty())
}

pub(super) async fn load_epiphany_state_from_rollout_path(
    rollout_path: &Path,
) -> std::result::Result<Option<EpiphanyThreadState>, String> {
    let items = read_rollout_items_from_rollout(rollout_path)
        .await
        .map_err(|err| {
            format!(
                "failed to load rollout `{}` for Epiphany state: {err}",
                rollout_path.display()
            )
        })?;
    Ok(latest_epiphany_state_from_rollout_items(&items))
}
