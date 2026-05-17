#[derive(Default)]
pub struct EpiphanyCheckpointInterventionState {
    last_intervention_turn_id: Option<String>,
    pending_compaction_turn_id: Option<String>,
}

impl EpiphanyCheckpointInterventionState {
    pub fn record_intervention(&mut self, turn_id: &str) -> bool {
        if self.last_intervention_turn_id.as_deref() == Some(turn_id) {
            return false;
        }
        self.last_intervention_turn_id = Some(turn_id.to_string());
        true
    }

    pub fn mark_pending_compaction(&mut self, turn_id: &str) {
        self.pending_compaction_turn_id = Some(turn_id.to_string());
    }

    pub fn take_pending_compaction(&mut self, turn_id: &str) -> bool {
        if self.pending_compaction_turn_id.as_deref() != Some(turn_id) {
            return false;
        }
        self.pending_compaction_turn_id = None;
        true
    }

    pub fn clear_pending_compaction(&mut self) {
        self.pending_compaction_turn_id = None;
    }
}

#[cfg(test)]
mod tests {
    use super::EpiphanyCheckpointInterventionState;

    #[test]
    fn pending_compaction_is_turn_scoped() {
        let mut state = EpiphanyCheckpointInterventionState::default();

        assert!(state.record_intervention("turn-a"));
        assert!(!state.record_intervention("turn-a"));
        assert!(!state.take_pending_compaction("turn-a"));

        state.mark_pending_compaction("turn-a");

        assert!(!state.take_pending_compaction("turn-b"));
        assert!(state.take_pending_compaction("turn-a"));
        assert!(!state.take_pending_compaction("turn-a"));
    }

    #[test]
    fn pending_compaction_can_be_cleared() {
        let mut state = EpiphanyCheckpointInterventionState::default();

        state.mark_pending_compaction("turn-a");
        state.clear_pending_compaction();

        assert!(!state.take_pending_compaction("turn-a"));
    }
}
