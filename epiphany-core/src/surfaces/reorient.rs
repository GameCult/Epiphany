use serde::{Deserialize, Serialize};

/// Display projection of an accepted keyed continuity decision.
///
/// This enum does not decide whether Reorientation should run. The keyed
/// current-work owner in `reorientation_work` owns launch, wait, review, and
/// admission behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyReorientAction {
    Resume,
    Regather,
}
