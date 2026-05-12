mod pressure;

pub use pressure::EpiphanyPressure;
pub use pressure::EpiphanyPressureBasis;
pub use pressure::EpiphanyPressureLevel;
pub use pressure::EpiphanyPressureStatus;
pub use pressure::derive_pressure_view;

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
