//! Core-owned semantic state contracts.

use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct EpiphanySubgoal {
    pub id: String,
    pub title: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct EpiphanyInvariant {
    pub id: String,
    pub description: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct EpiphanyCodeRef {
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EpiphanyInvestigationDisposition {
    #[default]
    ResumeReady,
    RegatherRequired,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct EpiphanyInvestigationCheckpoint {
    pub checkpoint_id: String,
    pub kind: String,
    pub disposition: EpiphanyInvestigationDisposition,
    pub focus: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captured_at_turn_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub open_questions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub code_refs: Vec<EpiphanyCodeRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct EpiphanyEvidenceRecord {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub code_refs: Vec<EpiphanyCodeRef>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct EpiphanyModeState {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<EpiphanyModeKind>,
}

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Default,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct EpiphanyMemoryGraphSnapshot {
    pub graph_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub domains: Vec<EpiphanyMemoryDomain>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<EpiphanyMemoryNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<EpiphanyMemoryEdge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frontier: Vec<RepoFrontierItem>,
}

/// A durable piece of unfinished repository anatomy. This is Modeling-owned state,
/// not a scheduler job or a presentation card.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct RepoFrontierItem {
    pub id: String,
    pub migration_body: String,
    pub question: String,
    pub gap: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_claim_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Canonical repository-relative path ceiling for this wound. This is the
    /// scope that downstream Planning may narrow and Hands may eventually
    /// change; inspected files and evidence provenance belong elsewhere.
    pub repository_scope: Vec<String>,
    pub recommended_next_organ: String,
    /// The exact plan admitted by Mind for this frontier item. Imagination may
    /// propose this payload, but only the dedicated model transition may make
    /// it part of Modeling-owned truth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adopted_plan: Option<RepoFrontierAdoptedPlan>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependency_item_ids: Vec<String>,
    #[serde(default)]
    pub status: RepoFrontierStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    /// Exact immutable public sources selected upstream for Eyes. These are
    /// causal retrieval inputs, not evidence produced by Modeling.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub public_source_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retired_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct RepoFrontierAdoptedPlan {
    pub planning_request_id: String,
    pub result_id: String,
    pub job_id: String,
    pub candidate_id: String,
    pub candidate_sha256: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub safe_paths: Vec<String>,
    pub action: String,
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checks: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stop_conditions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rollback_steps: Vec<String>,
    pub commit_message: String,
}

impl RepoFrontierAdoptedPlan {
    pub fn effective_action(&self) -> &str {
        self.action.as_str()
    }

    pub fn effective_command(&self) -> &str {
        self.command.as_str()
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RepoFrontierStatus {
    #[default]
    Proposed,
    Active,
    Blocked,
    Resolved,
    Retired,
    Superseded,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct EpiphanyMemoryDomain {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub lifecycle: EpiphanyMemoryLifecycle,
}

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum EpiphanyMemoryLifecycle {
    #[default]
    Observed,
    Proposed,
    Accepted,
    Retired,
    Stale,
}

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum EpiphanyMemoryNodeKind {
    #[default]
    Domain,
    Module,
    Crate,
    Binary,
    Schema,
    RuntimeContract,
    Adapter,
    TestSeam,
    StateStore,
    Summary,
    Other,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct EpiphanyMemoryNode {
    pub id: String,
    pub domain_id: String,
    pub kind: EpiphanyMemoryNodeKind,
    pub title: String,
    pub claim: String,
    pub question: String,
    pub tension: String,
    pub action_implication: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub anchors: Vec<EpiphanyMemoryAnchor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_hashes: Vec<String>,
    #[serde(default)]
    pub lifecycle: EpiphanyMemoryLifecycle,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum EpiphanyMemoryEdgeKind {
    #[default]
    Owns,
    Reads,
    Writes,
    Derives,
    Adapts,
    Persists,
    Launches,
    Verifies,
    Supports,
    Contradicts,
    Grounds,
    Triggers,
    DependsOn,
    Other,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct EpiphanyMemoryEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub kind: EpiphanyMemoryEdgeKind,
    pub claim: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub anchors: Vec<EpiphanyMemoryAnchor>,
    #[serde(default)]
    pub lifecycle: EpiphanyMemoryLifecycle,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct EpiphanyMemoryAnchor {
    pub id: String,
    pub kind: String,
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_ref: Option<EpiphanyCodeRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}
