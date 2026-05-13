//! Compatibility marker for the disabled Codex skill watcher.

pub(crate) struct SkillsWatcher;

impl SkillsWatcher {
    pub(crate) fn noop() -> Self {
        Self
    }
}
