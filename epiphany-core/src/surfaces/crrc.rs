use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyCrrcResultStatus {
    MissingState,
    MissingBinding,
    BackendUnavailable,
    BackendMissing,
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}
