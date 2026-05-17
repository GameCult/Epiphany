use super::ContextualUserFragment;
use codex_protocol::protocol::EPIPHANY_STATE_CLOSE_TAG;
use codex_protocol::protocol::EPIPHANY_STATE_OPEN_TAG;
use codex_protocol::protocol::EpiphanyThreadState;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EpiphanyStateInstructions {
    text: String,
}

impl EpiphanyStateInstructions {
    pub(crate) fn from_state(state: &EpiphanyThreadState) -> Self {
        Self {
            text: epiphany_core::render_epiphany_state(state),
        }
    }
}

impl ContextualUserFragment for EpiphanyStateInstructions {
    const ROLE: &'static str = "developer";
    const START_MARKER: &'static str = EPIPHANY_STATE_OPEN_TAG;
    const END_MARKER: &'static str = EPIPHANY_STATE_CLOSE_TAG;

    fn body(&self) -> String {
        self.text.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::EpiphanyStateInstructions;
    use crate::context::ContextualUserFragment;
    use codex_protocol::protocol::EpiphanyThreadState;

    #[test]
    fn epiphany_state_fragment_wraps_native_rendered_state() {
        let state = EpiphanyThreadState {
            revision: 7,
            objective: Some("Map the actual turn loop.".to_string()),
            ..Default::default()
        };

        let rendered = EpiphanyStateInstructions::from_state(&state).render();

        assert!(rendered.contains("<epiphany_state>"));
        assert!(rendered.contains("- Revision: 7"));
        assert!(rendered.contains("Map the actual turn loop."));
        assert!(rendered.contains("</epiphany_state>"));
    }
}
