use super::EpiphanyCrrcAction;
use super::EpiphanyCrrcRecommendation;
use super::EpiphanyCrrcResultStatus;
use super::EpiphanyCrrcStateStatus;
use super::EpiphanyPressure;
use super::EpiphanyPressureLevel;
use super::EpiphanyReorientAction;
use super::EpiphanyRoleBoardLane;
use crate::EpiphanyModelingContinuationAction;
use crate::RepoFrontierPlanningLifecycleStage;
use crate::RepoFrontierResearchContinuationAction;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyCoordinatorRoleId {
    Implementation,
    Imagination,
    Research,
    Modeling,
    Verification,
    Reorientation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyCoordinatorRoleStatus {
    Ready,
    Needed,
    Running,
    Waiting,
    Review,
    Blocked,
    Unavailable,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyCoordinatorRoleResultStatus {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyCoordinatorAction {
    PrepareCheckpoint,
    CompactRehydrateReorient,
    LaunchReorientWorker,
    WaitForReorientWorker,
    ReviewReorientResult,
    RegatherManually,
    LaunchResearch,
    ReviewResearchResult,
    LaunchModeling,
    WaitForModelingResult,
    ReviewModelingResult,
    LaunchVerification,
    ReviewVerificationResult,
    ContinueImplementation,
    AwaitFrontierProposal,
    StartFrontierPlanning,
    LaunchImagination,
    WaitForImaginationResult,
    RequestMindPlanReview,
    LaunchMindPlanReview,
    WaitForMindPlanResult,
    CommitFrontierPlanDecision,
    ReviewFrontierPlanningFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyCoordinatorSceneAction {
    Update,
    Reorient,
    ReorientLaunch,
    ReorientResult,
    ReorientAccept,
    RoleLaunch,
    RoleResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyCoordinatorAutomationAction {
    None,
    CompactRehydrateReorient,
    LaunchReorientWorker,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyCoordinatorCrrcRecommendation {
    pub action: EpiphanyCrrcAction,
    pub recommended_scene_action: Option<EpiphanyCoordinatorSceneAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyCoordinatorSignals {
    pub research_result_status: EpiphanyCoordinatorRoleResultStatus,
    pub modeling_result_status: EpiphanyCoordinatorRoleResultStatus,
    pub verification_result_status: EpiphanyCoordinatorRoleResultStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyCoordinatorSourceSignals {
    pub pressure_level: EpiphanyPressureLevel,
    pub should_prepare_compaction: bool,
    pub reorient_action: EpiphanyReorientAction,
    pub crrc_action: EpiphanyCrrcAction,
    pub research_result_status: EpiphanyCoordinatorRoleResultStatus,
    pub modeling_result_status: EpiphanyCoordinatorRoleResultStatus,
    pub verification_result_status: EpiphanyCoordinatorRoleResultStatus,
    pub reorient_result_status: EpiphanyCrrcResultStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyCoordinatorRoleLane {
    pub id: EpiphanyCoordinatorRoleId,
    pub status: EpiphanyCoordinatorRoleStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyCoordinatorInput {
    pub state_status: EpiphanyCrrcStateStatus,
    pub checkpoint_present: bool,
    pub should_prepare_compaction: bool,
    pub recommendation: EpiphanyCoordinatorCrrcRecommendation,
    pub roles: Vec<EpiphanyCoordinatorRoleLane>,
    pub signals: EpiphanyCoordinatorSignals,
    pub research_result_accepted: bool,
    pub research_result_reviewable: bool,
    pub research_result_failure_reviewed: bool,
    pub modeling_result_accepted_after_research: bool,
    pub modeling_result_requests_regather: bool,
    pub modeling_result_accepted: bool,
    pub modeling_result_reviewable: bool,
    pub modeling_result_failure_reviewed: bool,
    pub modeling_result_proposal_bound: bool,
    pub modeling_result_accepted_after_verification: bool,
    pub implementation_evidence_after_verification: bool,
    pub verification_result_cites_implementation_evidence: bool,
    pub verification_result_covers_current_modeling: bool,
    pub verification_result_accepted: bool,
    pub verification_result_failure_reviewed: bool,
    pub verification_result_allows_implementation: bool,
    pub verification_result_needs_evidence: bool,
    pub reorient_finding_accepted: bool,
    /// True only when the canonical runtime RepoModel has exactly one current
    /// admission and an Active, dependency-ready Hands frontier item.
    pub hands_frontier_ready: bool,
    /// Exact next action projected by the current frontier Research lifecycle.
    /// Legacy role-lane status remains observable but cannot override it.
    pub research_continuation_action: Option<RepoFrontierResearchContinuationAction>,
    /// Read-only projection of the single typed Imagination -> Mind planning
    /// lifecycle. Self may advance it, but only Mind's result can decide adoption.
    pub frontier_planning_stage: RepoFrontierPlanningLifecycleStage,
    /// Exact current action for the oldest unresolved proposal Modeling request.
    pub proposal_modeling_action: Option<EpiphanyModelingContinuationAction>,
    pub body_modeling_work_ready: bool,
    pub body_modeling_review_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyCoordinatorDecision {
    pub action: EpiphanyCoordinatorAction,
    pub target_role: Option<EpiphanyCoordinatorRoleId>,
    pub recommended_scene_action: Option<EpiphanyCoordinatorSceneAction>,
    pub requires_review: bool,
    pub can_auto_run: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyCoordinatorStatusInput {
    pub state_status: EpiphanyCrrcStateStatus,
    pub checkpoint_present: bool,
    pub pressure: EpiphanyPressure,
    pub recommendation: EpiphanyCrrcRecommendation,
    /// True when CRRC's manual-regather pressure is causally newer than the
    /// latest typed frontier relinquishment. Raw CRRC remains observable when
    /// false, but it cannot route Self from stale continuity history.
    pub crrc_regather_current: bool,
    pub roles: Vec<EpiphanyRoleBoardLane>,
    pub reorient_action: EpiphanyReorientAction,
    pub research_result_status: EpiphanyCoordinatorRoleResultStatus,
    pub modeling_result_status: EpiphanyCoordinatorRoleResultStatus,
    pub verification_result_status: EpiphanyCoordinatorRoleResultStatus,
    pub reorient_result_status: EpiphanyCrrcResultStatus,
    pub research_result_accepted: bool,
    pub research_result_reviewable: bool,
    pub research_result_failure_reviewed: bool,
    pub modeling_result_accepted_after_research: bool,
    pub modeling_result_requests_regather: bool,
    pub modeling_result_accepted: bool,
    pub modeling_result_reviewable: bool,
    pub modeling_result_failure_reviewed: bool,
    pub modeling_result_proposal_bound: bool,
    pub modeling_result_accepted_after_verification: bool,
    pub implementation_evidence_after_verification: bool,
    pub verification_result_cites_implementation_evidence: bool,
    pub verification_result_covers_current_modeling: bool,
    pub verification_result_accepted: bool,
    pub verification_result_failure_reviewed: bool,
    pub verification_result_allows_implementation: bool,
    pub verification_result_needs_evidence: bool,
    pub reorient_finding_accepted: bool,
    pub hands_frontier_ready: bool,
    pub research_continuation_action: Option<RepoFrontierResearchContinuationAction>,
    pub frontier_planning_stage: RepoFrontierPlanningLifecycleStage,
    pub proposal_modeling_action: Option<EpiphanyModelingContinuationAction>,
    pub body_modeling_work_ready: bool,
    pub body_modeling_review_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyCoordinatorStatus {
    pub decision: EpiphanyCoordinatorDecision,
    pub source_signals: EpiphanyCoordinatorSourceSignals,
    pub roles: Vec<EpiphanyRoleBoardLane>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EpiphanyCoordinatorFindingSignals {
    pub research_result_accepted: bool,
    pub research_result_reviewable: bool,
    pub research_result_failure_reviewed: bool,
    pub modeling_result_accepted_after_research: bool,
    pub modeling_result_requests_regather: bool,
    pub modeling_result_accepted: bool,
    pub modeling_result_reviewable: bool,
    pub modeling_result_failure_reviewed: bool,
    pub modeling_result_proposal_bound: bool,
    pub modeling_result_accepted_after_verification: bool,
    pub implementation_evidence_after_verification: bool,
    pub verification_result_cites_implementation_evidence: bool,
    pub verification_result_covers_current_modeling: bool,
    pub verification_result_accepted: bool,
    pub verification_result_failure_reviewed: bool,
    pub verification_result_allows_implementation: bool,
    pub verification_result_needs_evidence: bool,
    pub reorient_finding_accepted: bool,
}
