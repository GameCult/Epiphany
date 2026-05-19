use epiphany_core::EpiphanyGraphFreshness;
use epiphany_core::EpiphanyInvalidationInput;
use epiphany_core::EpiphanyRetrievalFreshness;
pub use epiphany_core::EpiphanyGraphFreshnessStatus;
pub use epiphany_core::EpiphanyInvalidationStatus;
pub use epiphany_core::EpiphanyJobStatus;
pub use epiphany_core::EpiphanyJobView;
pub use epiphany_core::EpiphanyPressure;
pub use epiphany_core::EpiphanyPressureBasis;
pub use epiphany_core::EpiphanyPressureLevel;
pub use epiphany_core::EpiphanyPressureStatus;
pub use epiphany_core::EpiphanyReorientAction;
pub use epiphany_core::EpiphanyReorientCheckpointStatus;
pub use epiphany_core::EpiphanyReorientDecision;
pub use epiphany_core::EpiphanyReorientFreshnessStatus;
pub use epiphany_core::EpiphanyReorientPressureLevel;
pub use epiphany_core::EpiphanyReorientReason;
pub use epiphany_core::EpiphanyReorientStateStatus;
pub use epiphany_core::EpiphanyRetrievalFreshnessStatus;
pub use epiphany_state_model::EpiphanyJobKind;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanySurfaceSource {
    Stored,
    Live,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyFreshnessSurface {
    pub thread_id: String,
    pub source: EpiphanySurfaceSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_revision: Option<u64>,
    pub retrieval: EpiphanyRetrievalFreshness,
    pub graph: EpiphanyGraphFreshness,
    pub watcher: EpiphanyInvalidationInput,
}
