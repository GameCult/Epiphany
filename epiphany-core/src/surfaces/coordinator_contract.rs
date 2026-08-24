use super::EpiphanyCrrcAction;
use crate::EpiphanyCurrentWorkProjection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyCoordinatorRoleStatus {
    Ready,
    Needed,
    Running,
    Review,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyCoordinatorAction {
    PrepareCheckpoint,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyCoordinatorInput {
    pub mind_present: bool,
    pub crrc_action: EpiphanyCrrcAction,
    pub current_work: EpiphanyCurrentWorkProjection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyCoordinatorDecision {
    pub action: EpiphanyCoordinatorAction,
    pub reason: String,
}
