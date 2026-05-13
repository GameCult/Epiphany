use epiphany_state_model::EpiphanyChurnState;
use epiphany_state_model::EpiphanyCodeRef;
use epiphany_state_model::EpiphanyEvidenceRecord;
use epiphany_state_model::EpiphanyGraphCheckpoint;
use epiphany_state_model::EpiphanyGraphFrontier;
use epiphany_state_model::EpiphanyGraphs;
use epiphany_state_model::EpiphanyInvariant;
use epiphany_state_model::EpiphanyInvestigationCheckpoint;
use epiphany_state_model::EpiphanyObservation;
use epiphany_state_model::EpiphanyPlanningState;
use epiphany_state_model::EpiphanyScratchPad;
use epiphany_state_model::EpiphanySubgoal;
use serde::Deserialize;
use serde::Serialize;

pub const ROLE_WORKER_OUTPUT_CONTRACT_ID: &str = "epiphany.worker.role_result.v0";
pub const REORIENT_WORKER_OUTPUT_CONTRACT_ID: &str = "epiphany.worker.reorient_result.v0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "documentKind")]
pub enum EpiphanyWorkerLaunchDocument {
    Role(EpiphanyRoleWorkerLaunchDocument),
    Reorient(EpiphanyReorientWorkerLaunchDocument),
}

impl EpiphanyWorkerLaunchDocument {
    pub fn output_contract_id(&self) -> &'static str {
        match self {
            Self::Role(_) => ROLE_WORKER_OUTPUT_CONTRACT_ID,
            Self::Reorient(_) => REORIENT_WORKER_OUTPUT_CONTRACT_ID,
        }
    }

    pub fn thread_id(&self) -> &str {
        match self {
            Self::Role(document) => &document.thread_id,
            Self::Reorient(document) => &document.thread_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyRoleWorkerLaunchDocument {
    pub thread_id: String,
    pub role_id: String,
    pub state_revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_subgoal_id: Option<String>,
    #[serde(default)]
    pub active_subgoals: Vec<EpiphanySubgoal>,
    #[serde(default)]
    pub active_graph_node_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub investigation_checkpoint: Option<EpiphanyInvestigationCheckpoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scratch: Option<EpiphanyScratchPad>,
    #[serde(default)]
    pub invariants: Vec<EpiphanyInvariant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graphs: Option<EpiphanyGraphs>,
    #[serde(default)]
    pub recent_evidence: Vec<EpiphanyEvidenceRecord>,
    #[serde(default)]
    pub recent_observations: Vec<EpiphanyObservation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_frontier: Option<EpiphanyGraphFrontier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_checkpoint: Option<EpiphanyGraphCheckpoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planning: Option<EpiphanyPlanningState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub churn: Option<EpiphanyChurnState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyReorientWorkerLaunchDocument {
    pub thread_id: String,
    pub mode: String,
    pub checkpoint_id: String,
    pub checkpoint_kind: String,
    pub checkpoint_disposition: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_focus: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_summary: Option<String>,
    pub checkpoint_next_action: String,
    #[serde(default)]
    pub checkpoint_open_questions: Vec<String>,
    #[serde(default)]
    pub checkpoint_evidence_ids: Vec<String>,
    #[serde(default)]
    pub checkpoint_code_refs: Vec<EpiphanyCodeRef>,
    #[serde(default)]
    pub decision_reasons: Vec<String>,
    pub decision_note: String,
    pub pressure_level: String,
    pub retrieval_status: String,
    pub graph_status: String,
    pub watcher_status: String,
    #[serde(default)]
    pub checkpoint_dirty_paths: Vec<String>,
    #[serde(default)]
    pub checkpoint_changed_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scratch: Option<EpiphanyScratchPad>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graphs: Option<EpiphanyGraphs>,
    #[serde(default)]
    pub recent_evidence: Vec<EpiphanyEvidenceRecord>,
    #[serde(default)]
    pub recent_observations: Vec<EpiphanyObservation>,
    #[serde(default)]
    pub active_frontier_node_ids: Vec<String>,
    #[serde(default)]
    pub linked_subgoal_ids: Vec<String>,
    #[serde(default)]
    pub linked_graph_node_ids: Vec<String>,
}
