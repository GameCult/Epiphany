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

#[derive(Debug, Clone, Default)]
pub struct EpiphanyStateUpdate {
    pub expected_revision: Option<u64>,
    pub objective: Option<String>,
    pub active_subgoal_id: Option<String>,
    pub subgoals: Option<Vec<EpiphanySubgoal>>,
    pub invariants: Option<Vec<EpiphanyInvariant>>,
    pub graphs: Option<EpiphanyGraphs>,
    pub graph_frontier: Option<EpiphanyGraphFrontier>,
    pub graph_checkpoint: Option<EpiphanyGraphCheckpoint>,
    pub scratch: Option<EpiphanyScratchPad>,
    pub investigation_checkpoint: Option<EpiphanyInvestigationCheckpoint>,
    pub job_bindings: Option<Vec<EpiphanyJobBinding>>,
    pub acceptance_receipts: Vec<EpiphanyAcceptanceReceipt>,
    pub runtime_links: Vec<EpiphanyRuntimeLink>,
    pub observations: Vec<EpiphanyObservation>,
    pub evidence: Vec<EpiphanyEvidenceRecord>,
    pub churn: Option<EpiphanyChurnState>,
    pub mode: Option<EpiphanyModeState>,
    pub planning: Option<EpiphanyPlanningState>,
}

impl EpiphanyStateUpdate {
    pub fn is_empty(&self) -> bool {
        self.objective.is_none()
            && self.active_subgoal_id.is_none()
            && self.subgoals.is_none()
            && self.invariants.is_none()
            && self.graphs.is_none()
            && self.graph_frontier.is_none()
            && self.graph_checkpoint.is_none()
            && self.scratch.is_none()
            && self.investigation_checkpoint.is_none()
            && self.job_bindings.is_none()
            && self.acceptance_receipts.is_empty()
            && self.runtime_links.is_empty()
            && self.observations.is_empty()
            && self.evidence.is_empty()
            && self.churn.is_none()
            && self.mode.is_none()
            && self.planning.is_none()
    }
}
