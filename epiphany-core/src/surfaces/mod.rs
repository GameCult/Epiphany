mod crrc;
mod pressure;

pub use crrc::recommend_crrc_action;
pub use crrc::EpiphanyCrrcAction;
pub use crrc::EpiphanyCrrcInput;
pub use crrc::EpiphanyCrrcRecommendation;
pub use crrc::EpiphanyCrrcReorientAction;
pub use crrc::EpiphanyCrrcResultStatus;
pub use crrc::EpiphanyCrrcSceneAction;
pub use crrc::EpiphanyCrrcStateStatus;
pub use pressure::derive_pressure_view;
pub use pressure::EpiphanyPressure;
pub use pressure::EpiphanyPressureBasis;
pub use pressure::EpiphanyPressureLevel;
pub use pressure::EpiphanyPressureStatus;

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
