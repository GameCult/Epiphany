use crate::*;
use anyhow::{Result, anyhow};
use epiphany_state_model::EpiphanyThreadState;
use std::path::Path;

pub fn interrupt_coordinator_job(
    store: &Path,
    thread_id: &str,
    state: &EpiphanyThreadState,
    request: EpiphanyJobInterruptRequest,
) -> Result<EpiphanyJobInterruptResult> {
    if request.binding_id.trim().is_empty() {
        return Err(anyhow!(
            "epiphany job interrupt binding_id must be non-empty"
        ));
    }
    if let Some(expected) = request.expected_revision
        && expected != state.revision
    {
        return Err(anyhow!(
            "epiphany state revision mismatch: expected {expected}, found {}",
            state.revision
        ));
    }
    let binding_index = state
        .job_bindings
        .iter()
        .position(|binding| binding.id == request.binding_id)
        .ok_or_else(|| {
            anyhow!(
                "epiphany job binding {:?} was not found",
                request.binding_id
            )
        })?;
    let reason = request
        .reason
        .as_deref()
        .map(str::trim)
        .filter(|reason| !reason.is_empty())
        .unwrap_or(
            "No active heartbeat turn is currently bound; launch explicitly to resume specialist work.",
        );
    let applied = apply_coordinator_state_update(
        store,
        thread_id,
        EpiphanyStateUpdate {
            expected_revision: request.expected_revision,
            job_bindings: Some(clear_epiphany_job_binding_backend(
                state.job_bindings.clone(),
                binding_index,
                reason,
            )),
            ..Default::default()
        },
        None,
    )?;
    Ok(EpiphanyJobInterruptResult {
        epiphany_state: applied.state,
        binding_id: request.binding_id,
        cancel_requested: false,
        interrupted_thread_ids: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_state_model::EpiphanyJobBinding;

    #[test]
    fn native_interrupt_updates_the_canonical_binding_once() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("interrupt.cc");
        let state = EpiphanyThreadState {
            job_bindings: vec![EpiphanyJobBinding {
                id: "modeling-worker".to_string(),
                kind: epiphany_state_model::EpiphanyJobKind::Specialist,
                scope: "model".to_string(),
                owner_role: "modeling".to_string(),
                authority_scope: Some("model".to_string()),
                linked_subgoal_ids: Vec::new(),
                linked_graph_node_ids: Vec::new(),
                blocking_reason: None,
            }],
            ..Default::default()
        };
        let result = interrupt_coordinator_job(
            &store,
            "thread-1",
            &state,
            EpiphanyJobInterruptRequest {
                expected_revision: Some(0),
                binding_id: "modeling-worker".to_string(),
                reason: Some("operator interrupted".to_string()),
            },
        )?;
        assert_eq!(result.epiphany_state.revision, 1);
        assert_eq!(
            result.epiphany_state.job_bindings[0]
                .blocking_reason
                .as_deref(),
            Some("operator interrupted")
        );
        assert!(!result.cancel_requested);
        assert_eq!(read_coordinator_state(&store)?, Some(result.epiphany_state));
        Ok(())
    }
}
