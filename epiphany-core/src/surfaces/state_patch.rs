use codex_protocol::protocol::EpiphanyAcceptanceReceipt;
use codex_protocol::protocol::EpiphanyChurnState;
use codex_protocol::protocol::EpiphanyEvidenceRecord;
use codex_protocol::protocol::EpiphanyGraphCheckpoint;
use codex_protocol::protocol::EpiphanyGraphFrontier;
use codex_protocol::protocol::EpiphanyGraphs;
use codex_protocol::protocol::EpiphanyInvariant;
use codex_protocol::protocol::EpiphanyInvestigationCheckpoint;
use codex_protocol::protocol::EpiphanyJobBinding;
use codex_protocol::protocol::EpiphanyModeState;
use codex_protocol::protocol::EpiphanyObservation;
use codex_protocol::protocol::EpiphanyPlanningState;
use codex_protocol::protocol::EpiphanyRuntimeLink;
use codex_protocol::protocol::EpiphanyScratchPad;
use codex_protocol::protocol::EpiphanySubgoal;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyRoleStatePatchDocument {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_subgoal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subgoals: Option<Vec<EpiphanySubgoal>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invariants: Option<Vec<EpiphanyInvariant>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graphs: Option<EpiphanyGraphs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_frontier: Option<EpiphanyGraphFrontier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_checkpoint: Option<EpiphanyGraphCheckpoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scratch: Option<EpiphanyScratchPad>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub investigation_checkpoint: Option<EpiphanyInvestigationCheckpoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_bindings: Option<Vec<EpiphanyJobBinding>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acceptance_receipts: Vec<EpiphanyAcceptanceReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runtime_links: Vec<EpiphanyRuntimeLink>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observations: Vec<EpiphanyObservation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<EpiphanyEvidenceRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub churn: Option<EpiphanyChurnState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<EpiphanyModeState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planning: Option<EpiphanyPlanningState>,
}
