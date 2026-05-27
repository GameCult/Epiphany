use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyViewLens {
    Scene,
    Jobs,
    Roles,
    Planning,
    Pressure,
    Reorient,
    Crrc,
    Coordinator,
}

pub fn default_epiphany_view_lenses() -> Vec<EpiphanyViewLens> {
    vec![
        EpiphanyViewLens::Scene,
        EpiphanyViewLens::Jobs,
        EpiphanyViewLens::Roles,
        EpiphanyViewLens::Planning,
        EpiphanyViewLens::Pressure,
        EpiphanyViewLens::Reorient,
        EpiphanyViewLens::Crrc,
        EpiphanyViewLens::Coordinator,
    ]
}

pub fn epiphany_view_needs_jobs(lenses: &[EpiphanyViewLens]) -> bool {
    lenses.contains(&EpiphanyViewLens::Jobs)
        || lenses.contains(&EpiphanyViewLens::Roles)
        || lenses.contains(&EpiphanyViewLens::Crrc)
        || lenses.contains(&EpiphanyViewLens::Coordinator)
}

pub fn epiphany_view_needs_reorientation_inputs(lenses: &[EpiphanyViewLens]) -> bool {
    lenses.contains(&EpiphanyViewLens::Roles)
        || lenses.contains(&EpiphanyViewLens::Reorient)
        || lenses.contains(&EpiphanyViewLens::Crrc)
        || lenses.contains(&EpiphanyViewLens::Coordinator)
}

pub fn epiphany_view_needs_pressure(lenses: &[EpiphanyViewLens]) -> bool {
    lenses.contains(&EpiphanyViewLens::Pressure) || epiphany_view_needs_reorientation_inputs(lenses)
}

pub fn epiphany_view_needs_runtime_store(lenses: &[EpiphanyViewLens]) -> bool {
    lenses.contains(&EpiphanyViewLens::Roles)
        || lenses.contains(&EpiphanyViewLens::Crrc)
        || lenses.contains(&EpiphanyViewLens::Coordinator)
}
