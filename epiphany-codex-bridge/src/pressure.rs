use epiphany_core::EpiphanyPressure;
use epiphany_core::EpiphanyPressureStatus as CoreEpiphanyPressureStatus;
pub use epiphany_core::EpiphanyTokenUsageSnapshot;
use epiphany_core::derive_pressure_view;

pub fn derive_epiphany_pressure(snapshot: Option<&EpiphanyTokenUsageSnapshot>) -> EpiphanyPressure {
    derive_pressure_view(snapshot)
}

pub fn should_run_epiphany_pre_compaction_checkpoint_intervention(
    pressure: &EpiphanyPressure,
) -> bool {
    pressure.status == CoreEpiphanyPressureStatus::Ready && pressure.should_prepare_compaction
}

pub fn render_epiphany_pre_compaction_checkpoint_intervention(
    pressure: &EpiphanyPressure,
) -> String {
    epiphany_core::render_epiphany_pre_compaction_checkpoint_intervention(pressure)
}
