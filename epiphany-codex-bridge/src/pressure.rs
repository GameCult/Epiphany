use epiphany_core::EpiphanyPressure;
pub use epiphany_core::EpiphanyTokenUsageSnapshot;
use epiphany_core::derive_pressure_view;

pub fn derive_epiphany_pressure(snapshot: Option<&EpiphanyTokenUsageSnapshot>) -> EpiphanyPressure {
    derive_pressure_view(snapshot)
}
