use super::{
    EpiphanyCrrcAction, EpiphanyCrrcResultStatus, EpiphanyPressure, EpiphanyPressureLevel,
    EpiphanyReorientAction, EpiphanyRoleBoardLane,
};
use crate::EpiphanyCurrentWorkProjection;
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
    LaunchImaginationConsideration,
    WaitForImaginationConsideration,
    LaunchAdmittedModelDirectionConsideration,
    WaitForAdmittedModelDirectionConsideration,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyCoordinatorInput {
    pub mind_present: bool,
    pub should_prepare_compaction: bool,
    pub recommendation: EpiphanyCoordinatorCrrcRecommendation,
    pub current_work: EpiphanyCurrentWorkProjection,
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
    pub mind_present: bool,
    pub pressure: EpiphanyPressure,
    pub recommendation: crate::EpiphanyCrrcRecommendation,
    pub roles: Vec<EpiphanyRoleBoardLane>,
    pub reorient_action: EpiphanyReorientAction,
    pub reorient_result_status: EpiphanyCrrcResultStatus,
    pub current_work: EpiphanyCurrentWorkProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyCoordinatorSourceSignals {
    pub pressure_level: EpiphanyPressureLevel,
    pub should_prepare_compaction: bool,
    pub reorient_action: EpiphanyReorientAction,
    pub crrc_action: EpiphanyCrrcAction,
    pub reorient_result_status: EpiphanyCrrcResultStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyCoordinatorStatus {
    pub decision: EpiphanyCoordinatorDecision,
    pub source_signals: EpiphanyCoordinatorSourceSignals,
    pub roles: Vec<EpiphanyRoleBoardLane>,
}
