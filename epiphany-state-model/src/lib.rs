use std::path::PathBuf;

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS)]
pub struct EpiphanyStateItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_id: Option<String>,
    pub state: EpiphanyThreadState,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyThreadState {
    pub revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub objective: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub active_subgoal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanySubgoal>")]
    pub subgoals: Vec<EpiphanySubgoal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyInvariant>")]
    pub invariants: Vec<EpiphanyInvariant>,
    #[serde(default, skip_serializing_if = "EpiphanyGraphs::is_empty")]
    #[ts(type = "EpiphanyGraphs")]
    pub graphs: EpiphanyGraphs,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "EpiphanyGraphFrontier | null")]
    pub graph_frontier: Option<EpiphanyGraphFrontier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "EpiphanyGraphCheckpoint | null")]
    pub graph_checkpoint: Option<EpiphanyGraphCheckpoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "EpiphanyRetrievalState | null")]
    pub retrieval: Option<EpiphanyRetrievalState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "EpiphanyScratchPad | null")]
    pub scratch: Option<EpiphanyScratchPad>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "EpiphanyInvestigationCheckpoint | null")]
    pub investigation_checkpoint: Option<EpiphanyInvestigationCheckpoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyJobBinding>")]
    pub job_bindings: Vec<EpiphanyJobBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyAcceptanceReceipt>")]
    pub acceptance_receipts: Vec<EpiphanyAcceptanceReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyRuntimeLink>")]
    pub runtime_links: Vec<EpiphanyRuntimeLink>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyObservation>")]
    pub observations: Vec<EpiphanyObservation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyEvidenceRecord>")]
    pub recent_evidence: Vec<EpiphanyEvidenceRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "EpiphanyChurnState | null")]
    pub churn: Option<EpiphanyChurnState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "EpiphanyModeState | null")]
    pub mode: Option<EpiphanyModeState>,
    #[serde(default, skip_serializing_if = "EpiphanyPlanningState::is_empty")]
    #[ts(type = "EpiphanyPlanningState")]
    pub planning: EpiphanyPlanningState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub last_updated_turn_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyAcceptanceReceipt {
    pub id: String,
    pub result_id: String,
    pub job_id: String,
    pub binding_id: String,
    pub surface: String,
    pub role_id: String,
    pub status: String,
    pub accepted_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub accepted_observation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub accepted_evidence_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub summary: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyRuntimeLink {
    pub id: String,
    pub binding_id: String,
    pub surface: String,
    pub role_id: String,
    pub authority_scope: String,
    pub runtime_job_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub runtime_result_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub linked_subgoal_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub linked_graph_node_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanySubgoal {
    pub id: String,
    pub title: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub summary: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyInvariant {
    pub id: String,
    pub description: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub rationale: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyGraphs {
    #[serde(default, skip_serializing_if = "EpiphanyGraph::is_empty")]
    pub architecture: EpiphanyGraph,
    #[serde(default, skip_serializing_if = "EpiphanyGraph::is_empty")]
    pub dataflow: EpiphanyGraph,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<EpiphanyGraphLink>,
}

impl EpiphanyGraphs {
    pub fn is_empty(&self) -> bool {
        self.architecture.is_empty() && self.dataflow.is_empty() && self.links.is_empty()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyGraph {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<EpiphanyGraphNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<EpiphanyGraphEdge>,
}

impl EpiphanyGraph {
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyGraphNode {
    pub id: String,
    pub title: String,
    pub purpose: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub mechanism: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub metaphor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyCodeRef>")]
    pub code_refs: Vec<EpiphanyCodeRef>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyGraphEdge {
    pub source_id: String,
    pub target_id: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub mechanism: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyCodeRef>")]
    pub code_refs: Vec<EpiphanyCodeRef>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyGraphLink {
    pub dataflow_node_id: String,
    pub architecture_node_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub relationship: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyCodeRef>")]
    pub code_refs: Vec<EpiphanyCodeRef>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyCodeRef {
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub start_line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub end_line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub note: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyGraphFrontier {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_node_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_edge_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub open_question_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub open_gap_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dirty_paths: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyGraphCheckpoint {
    pub checkpoint_id: String,
    pub graph_revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub frontier_node_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub open_question_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub open_gap_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyRetrievalState {
    pub workspace_root: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub index_revision: Option<String>,
    #[serde(default)]
    pub status: EpiphanyRetrievalStatus,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[ts(type = "boolean")]
    pub semantic_available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "bigint | null")]
    pub last_indexed_at_unix_seconds: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub indexed_file_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub indexed_chunk_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyRetrievalShardSummary>")]
    pub shards: Vec<EpiphanyRetrievalShardSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub dirty_paths: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EpiphanyRetrievalStatus {
    #[default]
    Unavailable,
    Ready,
    Stale,
    Indexing,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyRetrievalShardSummary {
    pub shard_id: String,
    pub path_prefix: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub indexed_file_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub indexed_chunk_count: Option<u32>,
    #[serde(default)]
    pub status: EpiphanyRetrievalStatus,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[ts(type = "boolean")]
    pub exact_available: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[ts(type = "boolean")]
    pub semantic_available: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyScratchPad {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub hypothesis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub next_probe: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub notes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EpiphanyInvestigationDisposition {
    #[default]
    ResumeReady,
    RegatherRequired,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyInvestigationCheckpoint {
    pub checkpoint_id: String,
    pub kind: String,
    pub disposition: EpiphanyInvestigationDisposition,
    pub focus: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub next_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub captured_at_turn_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub open_questions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyCodeRef>")]
    pub code_refs: Vec<EpiphanyCodeRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub evidence_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EpiphanyJobKind {
    Indexing,
    Remap,
    Verification,
    Specialist,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS)]
pub struct EpiphanyJobBinding {
    pub id: String,
    pub kind: EpiphanyJobKind,
    pub scope: String,
    pub owner_role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub authority_scope: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub linked_subgoal_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub linked_graph_node_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub blocking_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyObservation {
    pub id: String,
    pub summary: String,
    pub source_kind: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub code_refs: Vec<EpiphanyCodeRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyEvidenceRecord {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub code_refs: Vec<EpiphanyCodeRef>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyChurnState {
    pub understanding_status: String,
    pub diff_pressure: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub graph_freshness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub warning: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub unexplained_writes: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyModeState {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "ModeKind | null")]
    pub kind: Option<EpiphanyModeKind>,
}

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, JsonSchema, TS, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum EpiphanyModeKind {
    Plan,
    #[default]
    #[serde(
        alias = "code",
        alias = "pair_programming",
        alias = "execute",
        alias = "custom"
    )]
    Default,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyPlanningState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub workspace_root: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyPlanningCapture>")]
    pub captures: Vec<EpiphanyPlanningCapture>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyBacklogItem>")]
    pub backlog_items: Vec<EpiphanyBacklogItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyRoadmapStream>")]
    pub roadmap_streams: Vec<EpiphanyRoadmapStream>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyObjectiveDraft>")]
    pub objective_drafts: Vec<EpiphanyObjectiveDraft>,
}

impl EpiphanyPlanningState {
    pub fn is_empty(&self) -> bool {
        self.workspace_root.is_none()
            && self.captures.is_empty()
            && self.backlog_items.is_empty()
            && self.roadmap_streams.is_empty()
            && self.objective_drafts.is_empty()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyPlanningCapture {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub body: Option<String>,
    pub confidence: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub speaker: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub tags: Vec<String>,
    pub source: EpiphanyPlanningSourceRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyPlanningSourceRef {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub external_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub repo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub issue_number: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "bigint | null")]
    pub database_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub state: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub milestone: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub assignees: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub closed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "boolean | null")]
    pub is_pull_request: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub imported_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyBacklogItem {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub summary: String,
    pub status: String,
    pub horizon: String,
    pub priority: EpiphanyPlanningPriority,
    pub confidence: String,
    pub product_area: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub lane_hints: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub dependencies: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub blockers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub acceptance_sketch: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyPlanningSourceRef>")]
    pub source_refs: Vec<EpiphanyPlanningSourceRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub duplicate_of: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyPlanningPriority {
    pub value: String,
    pub rationale: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub impact: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub urgency: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub confidence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub effort: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub unblocks: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub blocks: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub reason: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyRoadmapStream {
    pub id: String,
    pub title: String,
    pub purpose: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub item_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub near_term_focus: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub blocked_by: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub review_cadence: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyObjectiveDraft {
    pub id: String,
    pub title: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub source_item_ids: Vec<String>,
    pub scope: EpiphanyObjectiveDraftScope,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub acceptance_criteria: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub evidence_required: Vec<String>,
    pub lane_plan: EpiphanyObjectiveDraftLanePlan,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub dependencies: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub risks: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub review_gates: Vec<String>,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyObjectiveDraftScope {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub includes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub excludes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyObjectiveDraftLanePlan {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub imagination: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub eyes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub hands: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub soul: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub life: Option<String>,
}
