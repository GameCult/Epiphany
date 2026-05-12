mod coordinator;
mod crrc;
mod pressure;
mod role_board;

pub use coordinator::EpiphanyCoordinatorAction;
pub use coordinator::EpiphanyCoordinatorAutomationAction;
pub use coordinator::EpiphanyCoordinatorCrrcRecommendation;
pub use coordinator::EpiphanyCoordinatorDecision;
pub use coordinator::EpiphanyCoordinatorInput;
pub use coordinator::EpiphanyCoordinatorRoleId;
pub use coordinator::EpiphanyCoordinatorRoleLane;
pub use coordinator::EpiphanyCoordinatorRoleResultStatus;
pub use coordinator::EpiphanyCoordinatorRoleStatus;
pub use coordinator::EpiphanyCoordinatorSceneAction;
pub use coordinator::EpiphanyCoordinatorSignals;
pub use coordinator::coordinator_automation_action;
pub use coordinator::crrc_scene_action_to_coordinator_scene_action;
pub use coordinator::recommend_coordinator_action;
pub use coordinator::select_coordinator_automation_action;
pub use crrc::EpiphanyCrrcAction;
pub use crrc::EpiphanyCrrcInput;
pub use crrc::EpiphanyCrrcRecommendation;
pub use crrc::EpiphanyCrrcReorientAction;
pub use crrc::EpiphanyCrrcResultStatus;
pub use crrc::EpiphanyCrrcSceneAction;
pub use crrc::EpiphanyCrrcStateStatus;
pub use crrc::recommend_crrc_action;
pub use pressure::EpiphanyPressure;
pub use pressure::EpiphanyPressureBasis;
pub use pressure::EpiphanyPressureLevel;
pub use pressure::EpiphanyPressureStatus;
pub use pressure::derive_pressure_view;
pub use role_board::EpiphanyRoleBoardCheckpointSummary;
pub use role_board::EpiphanyRoleBoardInput;
pub use role_board::EpiphanyRoleBoardJob;
pub use role_board::EpiphanyRoleBoardJobStatus;
pub use role_board::EpiphanyRoleBoardLane;
pub use role_board::EpiphanyRoleBoardPlanningSummary;
pub use role_board::derive_role_board;
pub use role_board::render_role_board_note;
pub use role_board::reorientation_role_status;
pub use role_board::role_board_job_status_to_role_status;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyViewInput {
    pub pressure: Option<EpiphanyPressure>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyViewLens {
    Pressure,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EpiphanyViewOutput {
    pub pressure: Option<EpiphanyPressure>,
}
