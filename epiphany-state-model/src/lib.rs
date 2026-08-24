use std::path::PathBuf;

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;

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
pub struct EpiphanyMemoryGraphSnapshot {
    pub graph_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyMemoryDomain>")]
    pub domains: Vec<EpiphanyMemoryDomain>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyMemoryNode>")]
    pub nodes: Vec<EpiphanyMemoryNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyMemoryEdge>")]
    pub edges: Vec<EpiphanyMemoryEdge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<RepoFrontierItem>")]
    pub frontier: Vec<RepoFrontierItem>,
}

/// A durable piece of unfinished repository anatomy. This is Modeling-owned state,
/// not a scheduler job or a presentation card.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct RepoFrontierItem {
    pub id: String,
    pub migration_body: String,
    pub question: String,
    pub gap: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub target_claim_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    /// Canonical repository-relative path ceiling for this wound. This is the
    /// scope that downstream Planning may narrow and Hands may eventually
    /// change; inspected files and evidence provenance belong elsewhere.
    pub repository_scope: Vec<String>,
    pub recommended_next_organ: String,
    /// The exact plan admitted by Mind for this frontier item. Imagination may
    /// propose this payload, but only the dedicated model transition may make
    /// it part of Modeling-owned truth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "RepoFrontierAdoptedPlan | null")]
    pub adopted_plan: Option<RepoFrontierAdoptedPlan>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub dependency_item_ids: Vec<String>,
    #[serde(default)]
    pub status: RepoFrontierStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub evidence_refs: Vec<String>,
    /// Exact immutable public sources selected upstream for Eyes. These are
    /// causal retrieval inputs, not evidence produced by Modeling.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub public_source_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub retired_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub superseded_by: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct RepoFrontierAdoptedPlan {
    pub planning_request_id: String,
    pub result_id: String,
    pub job_id: String,
    pub candidate_id: String,
    pub candidate_sha256: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub safe_paths: Vec<String>,
    pub action: String,
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub checks: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub stop_conditions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum RepoFrontierStatus {
    #[default]
    Proposed,
    Active,
    Blocked,
    Resolved,
    Retired,
    Superseded,
}

pub fn reorient_checkpoint_from_admitted_repo_model(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    obligation_id: &str,
) -> EpiphanyInvestigationCheckpoint {
    EpiphanyInvestigationCheckpoint {
        checkpoint_id: obligation_id.to_string(),
        kind: "admitted_repo_model".to_string(),
        disposition: EpiphanyInvestigationDisposition::ResumeReady,
        focus: "Current Mind-admitted RepoModel frontier".to_string(),
        summary: Some(
            "Derived from the authenticated current keyed RepoModel view.".to_string(),
        ),
        next_action: Some("Resume from the current Mind-admitted RepoModel frontier.".to_string()),
        captured_at_turn_id: None,
        open_questions: Vec::new(),
        code_refs: snapshot
            .frontier
            .iter()
            .filter(|item| item.status == RepoFrontierStatus::Active)
            .flat_map(|item| item.repository_scope.iter())
            .map(|path| EpiphanyCodeRef {
                path: path.into(),
                start_line: None,
                end_line: None,
                symbol: None,
                note: Some("Current admitted RepoModel frontier scope".to_string()),
            })
            .collect(),
        evidence_ids: Vec::new(),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyMemoryDomain {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub description: Option<String>,
    #[serde(default)]
    pub lifecycle: EpiphanyMemoryLifecycle,
}

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, JsonSchema, TS, Default,
)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EpiphanyMemoryLifecycle {
    #[default]
    Observed,
    Proposed,
    Accepted,
    Retired,
    Stale,
}

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, JsonSchema, TS, Default,
)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
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
    #[ts(type = "Array<EpiphanyMemoryAnchor>")]
    pub anchors: Vec<EpiphanyMemoryAnchor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub source_hashes: Vec<String>,
    #[serde(default)]
    pub lifecycle: EpiphanyMemoryLifecycle,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub updated_at: Option<String>,
}

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, JsonSchema, TS, Default,
)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyMemoryEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub kind: EpiphanyMemoryEdgeKind,
    pub claim: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<EpiphanyMemoryAnchor>")]
    pub anchors: Vec<EpiphanyMemoryAnchor>,
    #[serde(default)]
    pub lifecycle: EpiphanyMemoryLifecycle,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, TS, Default)]
pub struct EpiphanyMemoryAnchor {
    pub id: String,
    pub kind: String,
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "EpiphanyCodeRef | null")]
    pub code_ref: Option<EpiphanyCodeRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub evidence_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub source_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub note: Option<String>,
}
