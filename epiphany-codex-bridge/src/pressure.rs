use codex_app_server_protocol::ThreadEpiphanyPressure;
use epiphany_core::EpiphanyPressure;
use epiphany_core::EpiphanyPressureLevel as CoreEpiphanyPressureLevel;
use epiphany_core::EpiphanyPressureStatus as CoreEpiphanyPressureStatus;
pub use epiphany_core::EpiphanyTokenUsageSnapshot;
use epiphany_core::derive_pressure_view;

use crate::launch::epiphany_agent_prompt_with_memory;
use crate::launch::epiphany_specialist_prompt_config;
use crate::protocol_edge::protocol_pressure_from_core;

pub fn derive_epiphany_pressure(snapshot: Option<&EpiphanyTokenUsageSnapshot>) -> EpiphanyPressure {
    derive_pressure_view(snapshot)
}

pub fn map_epiphany_pressure(
    snapshot: Option<&EpiphanyTokenUsageSnapshot>,
) -> ThreadEpiphanyPressure {
    protocol_pressure_from_core(derive_epiphany_pressure(snapshot))
}

pub fn should_run_epiphany_pre_compaction_checkpoint_intervention(
    pressure: &EpiphanyPressure,
) -> bool {
    pressure.status == CoreEpiphanyPressureStatus::Ready && pressure.should_prepare_compaction
}

pub fn render_epiphany_pre_compaction_checkpoint_intervention(
    pressure: &EpiphanyPressure,
) -> String {
    let usage = match (
        pressure.used_tokens,
        pressure.remaining_tokens,
        pressure.ratio_per_mille,
    ) {
        (Some(used), Some(remaining), Some(ratio)) => format!(
            "{used} tokens used, {remaining} remaining, {}.{}% of the selected limit",
            ratio / 10,
            ratio % 10
        ),
        (Some(used), _, _) => format!("{used} tokens used"),
        _ => "token usage known only as a pressure threshold crossing".to_string(),
    };
    let template = epiphany_agent_prompt_with_memory(
        &epiphany_specialist_prompt_config()
            .crrc
            .pre_compaction_checkpoint_intervention,
    );
    template
        .trim()
        .replace(
            "{pressure_level}",
            core_pressure_level_label(pressure.level),
        )
        .replace("{usage}", &usage)
}

fn core_pressure_level_label(level: CoreEpiphanyPressureLevel) -> &'static str {
    match level {
        CoreEpiphanyPressureLevel::Unknown => "unknown",
        CoreEpiphanyPressureLevel::Low => "low",
        CoreEpiphanyPressureLevel::Elevated => "elevated",
        CoreEpiphanyPressureLevel::High => "high",
        CoreEpiphanyPressureLevel::Critical => "critical",
    }
}
