use codex_app_server_protocol::ThreadEpiphanyPressure;
use codex_app_server_protocol::ThreadEpiphanyPressureBasis;
use codex_app_server_protocol::ThreadEpiphanyPressureLevel;
use codex_app_server_protocol::ThreadEpiphanyPressureStatus;
use codex_protocol::protocol::TokenUsageInfo as CoreTokenUsageInfo;
use epiphany_core::EpiphanyPressure;
use epiphany_core::EpiphanyPressureBasis as CoreEpiphanyPressureBasis;
use epiphany_core::EpiphanyPressureLevel as CoreEpiphanyPressureLevel;
use epiphany_core::EpiphanyPressureStatus as CoreEpiphanyPressureStatus;
use epiphany_core::derive_pressure_view;

use crate::launch::epiphany_agent_prompt_with_memory;
use crate::launch::epiphany_specialist_prompt_config;
use crate::launch::pressure_level_label;

pub fn map_epiphany_pressure(info: Option<&CoreTokenUsageInfo>) -> ThreadEpiphanyPressure {
    map_core_epiphany_pressure(derive_pressure_view(info))
}

fn map_core_epiphany_pressure(pressure: EpiphanyPressure) -> ThreadEpiphanyPressure {
    ThreadEpiphanyPressure {
        status: match pressure.status {
            CoreEpiphanyPressureStatus::Unknown => ThreadEpiphanyPressureStatus::Unknown,
            CoreEpiphanyPressureStatus::Ready => ThreadEpiphanyPressureStatus::Ready,
        },
        level: match pressure.level {
            CoreEpiphanyPressureLevel::Unknown => ThreadEpiphanyPressureLevel::Unknown,
            CoreEpiphanyPressureLevel::Low => ThreadEpiphanyPressureLevel::Low,
            CoreEpiphanyPressureLevel::Elevated => ThreadEpiphanyPressureLevel::Elevated,
            CoreEpiphanyPressureLevel::High => ThreadEpiphanyPressureLevel::High,
            CoreEpiphanyPressureLevel::Critical => ThreadEpiphanyPressureLevel::Critical,
        },
        basis: match pressure.basis {
            CoreEpiphanyPressureBasis::Unknown => ThreadEpiphanyPressureBasis::Unknown,
            CoreEpiphanyPressureBasis::AutoCompactLimit => {
                ThreadEpiphanyPressureBasis::AutoCompactLimit
            }
            CoreEpiphanyPressureBasis::ModelContextWindow => {
                ThreadEpiphanyPressureBasis::ModelContextWindow
            }
        },
        used_tokens: pressure.used_tokens,
        model_context_window: pressure.model_context_window,
        model_auto_compact_token_limit: pressure.model_auto_compact_token_limit,
        remaining_tokens: pressure.remaining_tokens,
        ratio_per_mille: pressure.ratio_per_mille,
        should_prepare_compaction: pressure.should_prepare_compaction,
        note: pressure.note,
    }
}

pub fn should_run_epiphany_pre_compaction_checkpoint_intervention(
    pressure: &ThreadEpiphanyPressure,
) -> bool {
    pressure.status == ThreadEpiphanyPressureStatus::Ready && pressure.should_prepare_compaction
}

pub fn render_epiphany_pre_compaction_checkpoint_intervention(
    pressure: &ThreadEpiphanyPressure,
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
        .replace("{pressure_level}", pressure_level_label(pressure.level))
        .replace("{usage}", &usage)
}
