const EPIPHANY_PRESSURE_ELEVATED_PER_MILLE: u32 = 650;
const EPIPHANY_PRESSURE_PREPARE_COMPACTION_PER_MILLE: u32 = 800;
const EPIPHANY_PRESSURE_CRITICAL_PER_MILLE: u32 = 950;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpiphanyTokenUsageSnapshot {
    pub total_tokens: i64,
    pub last_turn_tokens: i64,
    pub model_context_window: Option<i64>,
    pub model_auto_compact_token_limit: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyPressure {
    pub status: EpiphanyPressureStatus,
    pub level: EpiphanyPressureLevel,
    pub basis: EpiphanyPressureBasis,
    pub used_tokens: Option<i64>,
    pub model_context_window: Option<i64>,
    pub model_auto_compact_token_limit: Option<i64>,
    pub remaining_tokens: Option<i64>,
    pub ratio_per_mille: Option<u32>,
    pub should_prepare_compaction: bool,
    pub note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyPressureStatus {
    Unknown,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyPressureLevel {
    Unknown,
    Low,
    Elevated,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyPressureBasis {
    Unknown,
    AutoCompactLimit,
    ModelContextWindow,
}

pub fn derive_pressure_view(info: Option<&EpiphanyTokenUsageSnapshot>) -> EpiphanyPressure {
    let Some(info) = info else {
        return EpiphanyPressure {
            status: EpiphanyPressureStatus::Unknown,
            level: EpiphanyPressureLevel::Unknown,
            basis: EpiphanyPressureBasis::Unknown,
            used_tokens: None,
            model_context_window: None,
            model_auto_compact_token_limit: None,
            remaining_tokens: None,
            ratio_per_mille: None,
            should_prepare_compaction: false,
            note: "No token usage telemetry has been recorded for this thread yet.".to_string(),
        };
    };

    let used_tokens = info.last_turn_tokens.max(0);
    if used_tokens == 0 && info.total_tokens > 0 {
        return EpiphanyPressure {
            status: EpiphanyPressureStatus::Unknown,
            level: EpiphanyPressureLevel::Unknown,
            basis: EpiphanyPressureBasis::Unknown,
            used_tokens: None,
            model_context_window: info.model_context_window,
            model_auto_compact_token_limit: info.model_auto_compact_token_limit,
            remaining_tokens: None,
            ratio_per_mille: None,
            should_prepare_compaction: false,
            note: "Only cumulative token spend is available; CRRC will not infer current context pressure from that.".to_string(),
        };
    }

    let model_context_window = info.model_context_window.filter(|value| *value > 0);
    let model_auto_compact_token_limit = info
        .model_auto_compact_token_limit
        .filter(|value| *value > 0);
    let (basis, limit) = if let Some(limit) = model_auto_compact_token_limit {
        (EpiphanyPressureBasis::AutoCompactLimit, Some(limit))
    } else if let Some(limit) = model_context_window {
        (EpiphanyPressureBasis::ModelContextWindow, Some(limit))
    } else {
        (EpiphanyPressureBasis::Unknown, None)
    };

    let Some(limit) = limit else {
        return EpiphanyPressure {
            status: EpiphanyPressureStatus::Unknown,
            level: EpiphanyPressureLevel::Unknown,
            basis,
            used_tokens: Some(used_tokens),
            model_context_window: info.model_context_window,
            model_auto_compact_token_limit: info.model_auto_compact_token_limit,
            remaining_tokens: None,
            ratio_per_mille: None,
            should_prepare_compaction: false,
            note: "Current context usage is known, but no context window or auto-compact threshold is available."
                .to_string(),
        };
    };

    let ratio_per_mille = ((used_tokens.saturating_mul(1000)) / limit).max(0) as u32;
    let remaining_tokens = limit.saturating_sub(used_tokens);
    let level = match ratio_per_mille {
        ratio if ratio >= EPIPHANY_PRESSURE_CRITICAL_PER_MILLE => EpiphanyPressureLevel::Critical,
        ratio if ratio >= EPIPHANY_PRESSURE_PREPARE_COMPACTION_PER_MILLE => {
            EpiphanyPressureLevel::High
        }
        ratio if ratio >= EPIPHANY_PRESSURE_ELEVATED_PER_MILLE => EpiphanyPressureLevel::Elevated,
        _ => EpiphanyPressureLevel::Low,
    };
    let should_prepare_compaction =
        ratio_per_mille >= EPIPHANY_PRESSURE_PREPARE_COMPACTION_PER_MILLE;
    let note = match basis {
        EpiphanyPressureBasis::AutoCompactLimit => {
            "Pressure is derived from current context usage against the model auto-compact token limit.".to_string()
        }
        EpiphanyPressureBasis::ModelContextWindow => {
            "Pressure is derived from current context usage against the model context window because no auto-compact threshold was recorded.".to_string()
        }
        EpiphanyPressureBasis::Unknown => {
            "Current context usage is known, but no usable pressure limit was recorded.".to_string()
        }
    };

    EpiphanyPressure {
        status: EpiphanyPressureStatus::Ready,
        level,
        basis,
        used_tokens: Some(used_tokens),
        model_context_window: info.model_context_window,
        model_auto_compact_token_limit: info.model_auto_compact_token_limit,
        remaining_tokens: Some(remaining_tokens),
        ratio_per_mille: Some(ratio_per_mille),
        should_prepare_compaction,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_usage_info(
        total_tokens: i64,
        last_tokens: i64,
        model_context_window: Option<i64>,
        model_auto_compact_token_limit: Option<i64>,
    ) -> EpiphanyTokenUsageSnapshot {
        EpiphanyTokenUsageSnapshot {
            total_tokens,
            last_turn_tokens: last_tokens,
            model_context_window,
            model_auto_compact_token_limit,
        }
    }

    #[test]
    fn reports_unknown_without_telemetry() {
        let pressure = derive_pressure_view(None);

        assert_eq!(pressure.status, EpiphanyPressureStatus::Unknown);
        assert_eq!(pressure.level, EpiphanyPressureLevel::Unknown);
        assert_eq!(pressure.basis, EpiphanyPressureBasis::Unknown);
        assert!(!pressure.should_prepare_compaction);
    }

    #[test]
    fn prefers_auto_compact_limit() {
        let info = token_usage_info(1_000, 82, Some(200), Some(100));
        let pressure = derive_pressure_view(Some(&info));

        assert_eq!(pressure.status, EpiphanyPressureStatus::Ready);
        assert_eq!(pressure.level, EpiphanyPressureLevel::High);
        assert_eq!(pressure.basis, EpiphanyPressureBasis::AutoCompactLimit);
        assert_eq!(pressure.ratio_per_mille, Some(820));
        assert_eq!(pressure.remaining_tokens, Some(18));
        assert!(pressure.should_prepare_compaction);
    }

    #[test]
    fn ignores_cumulative_thread_spend() {
        let info = token_usage_info(310, 70, Some(200), Some(100));
        let pressure = derive_pressure_view(Some(&info));

        assert_eq!(pressure.status, EpiphanyPressureStatus::Ready);
        assert_eq!(pressure.level, EpiphanyPressureLevel::Elevated);
        assert_eq!(pressure.used_tokens, Some(70));
        assert_eq!(pressure.ratio_per_mille, Some(700));
        assert!(!pressure.should_prepare_compaction);
    }

    #[test]
    fn refuses_cumulative_only_telemetry() {
        let info = token_usage_info(310, 0, Some(200), Some(100));
        let pressure = derive_pressure_view(Some(&info));

        assert_eq!(pressure.status, EpiphanyPressureStatus::Unknown);
        assert_eq!(pressure.level, EpiphanyPressureLevel::Unknown);
        assert_eq!(pressure.ratio_per_mille, None);
        assert!(!pressure.should_prepare_compaction);
    }

    #[test]
    fn falls_back_to_context_window() {
        let info = token_usage_info(1_000, 150, Some(200), None);
        let pressure = derive_pressure_view(Some(&info));

        assert_eq!(pressure.status, EpiphanyPressureStatus::Ready);
        assert_eq!(pressure.level, EpiphanyPressureLevel::Elevated);
        assert_eq!(pressure.basis, EpiphanyPressureBasis::ModelContextWindow);
        assert_eq!(pressure.ratio_per_mille, Some(750));
        assert!(!pressure.should_prepare_compaction);
    }

    #[test]
    fn compaction_prep_starts_at_eighty_percent() {
        let elevated = token_usage_info(1_000, 79, None, Some(100));
        let high = token_usage_info(1_000, 80, None, Some(100));

        assert!(!derive_pressure_view(Some(&elevated)).should_prepare_compaction);
        assert!(derive_pressure_view(Some(&high)).should_prepare_compaction);
    }
}
