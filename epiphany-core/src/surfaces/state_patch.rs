use epiphany_state_model::EpiphanyAcceptanceReceipt;
use epiphany_state_model::EpiphanyChurnState;
use epiphany_state_model::EpiphanyEvidenceRecord;
use epiphany_state_model::EpiphanyGraphCheckpoint;
use epiphany_state_model::EpiphanyGraphFrontier;
use epiphany_state_model::EpiphanyGraphs;
use epiphany_state_model::EpiphanyInvariant;
use epiphany_state_model::EpiphanyInvestigationCheckpoint;
use epiphany_state_model::EpiphanyJobBinding;
use epiphany_state_model::EpiphanyModeState;
use epiphany_state_model::EpiphanyObservation;
use epiphany_state_model::EpiphanyPlanningState;
use epiphany_state_model::EpiphanyRuntimeLink;
use epiphany_state_model::EpiphanyScratchPad;
use epiphany_state_model::EpiphanySubgoal;
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
