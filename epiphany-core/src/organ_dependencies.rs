use serde::Deserialize;
use serde::Serialize;

pub const EPIPHANY_ORGAN_DEPENDENCY_SCHEMA_VERSION: &str = "epiphany.organ_dependency.v0";

pub const EPIPHANY_STANDING_ORGANS: [&str; 8] = [
    "self",
    "face",
    "imagination",
    "eyes",
    "body",
    "hands",
    "soul",
    "life",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyOrganDependency {
    pub schema_version: String,
    pub organ_id: String,
    pub depends_on: Vec<String>,
    pub contract: String,
}

pub fn default_organ_dependencies_for(organ_id: &str) -> EpiphanyOrganDependency {
    let normalized = organ_id.trim().to_ascii_lowercase();
    EpiphanyOrganDependency {
        schema_version: EPIPHANY_ORGAN_DEPENDENCY_SCHEMA_VERSION.to_string(),
        organ_id: normalized.clone(),
        depends_on: EPIPHANY_STANDING_ORGANS
            .iter()
            .filter(|candidate| **candidate != normalized)
            .map(|candidate| (*candidate).to_string())
            .collect(),
        contract: "Every sub-agent depends on the other organs: Self routes, Face speaks, Imagination projects futures/scenes, Eyes seeks evidence, Body gates substrate access, Hands acts, Soul verifies, and Life preserves continuity.".to_string(),
    }
}

pub fn default_organ_dependency_matrix() -> Vec<EpiphanyOrganDependency> {
    EPIPHANY_STANDING_ORGANS
        .iter()
        .map(|organ| default_organ_dependencies_for(organ))
        .collect()
}

pub fn render_organ_dependency(dependency: &EpiphanyOrganDependency) -> String {
    let depends_on = if dependency.depends_on.is_empty() {
        "- none".to_string()
    } else {
        dependency
            .depends_on
            .iter()
            .map(|organ| format!("- {organ}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        "Organ: {}\nDepends on:\n{}\nContract: {}",
        dependency.organ_id, depends_on, dependency.contract
    )
}

pub fn render_organ_dependencies(dependencies: &[EpiphanyOrganDependency]) -> String {
    if dependencies.is_empty() {
        return "- no organ dependency records supplied".to_string();
    }
    dependencies
        .iter()
        .map(render_organ_dependency)
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_standing_organ_depends_on_every_other_standing_organ() {
        let matrix = default_organ_dependency_matrix();
        assert_eq!(matrix.len(), EPIPHANY_STANDING_ORGANS.len());
        for dependency in matrix {
            assert_eq!(
                dependency.depends_on.len(),
                EPIPHANY_STANDING_ORGANS.len() - 1
            );
            assert!(!dependency.depends_on.contains(&dependency.organ_id));
            for organ in EPIPHANY_STANDING_ORGANS {
                if organ != dependency.organ_id {
                    assert!(dependency.depends_on.contains(&organ.to_string()));
                }
            }
        }
    }
}
