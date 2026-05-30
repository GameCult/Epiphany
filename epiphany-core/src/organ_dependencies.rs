use serde::Deserialize;
use serde::Serialize;

use crate::continuity_gateway::{
    CONTINUITY_RECOVERY_RECEIPT_TYPE, CONTINUITY_REFUSAL_RECEIPT_TYPE,
};
use crate::eyes_gateway::{EYES_EVIDENCE_PACKET_TYPE, EYES_EVIDENCE_REFUSAL_RECEIPT_TYPE};
use crate::hands_gateway::{
    HANDS_ACTION_REFUSAL_RECEIPT_TYPE, HANDS_COMMIT_RECEIPT_TYPE, HANDS_PATCH_RECEIPT_TYPE,
    HANDS_ROLLBACK_RECEIPT_TYPE,
};
use crate::mind_gateway::{
    MIND_GATEWAY_REVIEW_TYPE, MIND_STATE_COMMIT_RECEIPT_TYPE, MIND_STATE_REJECTION_RECEIPT_TYPE,
};
use crate::soul_gateway::{
    SOUL_REGRESSION_RECEIPT_TYPE, SOUL_REVIEW_RECEIPT_TYPE, SOUL_VERDICT_RECEIPT_TYPE,
    SOUL_VERIFICATION_REFUSAL_RECEIPT_TYPE,
};
use crate::substrate_gate::{
    SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE, SUBSTRATE_GATE_REPO_ACCESS_REFUSAL_RECEIPT_TYPE,
    SUBSTRATE_GATE_REPO_MUTATION_RECEIPT_TYPE, SUBSTRATE_GATE_REPO_SNAPSHOT_RECEIPT_TYPE,
};

pub const EPIPHANY_ORGAN_DEPENDENCY_SCHEMA_VERSION: &str = "epiphany.organ_dependency.v0";
pub const EPIPHANY_LAUNCH_ORGAN_CONTRACT_SCHEMA_VERSION: &str = "epiphany.launch_organ_contract.v0";

pub const EPIPHANY_STANDING_ORGANS: [&str; 7] = [
    "self",
    "face",
    "imagination",
    "eyes",
    "proprioception",
    "hands",
    "soul",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyOrganDependency {
    pub schema_version: String,
    pub organ_id: String,
    pub depends_on: Vec<String>,
    pub contract: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyLaunchOrganContract {
    pub schema_version: String,
    pub authority_scope: String,
    pub document_kind: String,
    pub output_contract_id: String,
    pub owner_organ: String,
    pub dependencies: Vec<EpiphanyOrganDependency>,
    #[serde(default)]
    pub receipt_proof_profiles: Vec<EpiphanyReceiptProofProfile>,
    pub required_receipt_document_types: Vec<String>,
    pub contract: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyReceiptEffectKind {
    #[default]
    StateAdmission,
    EvidencePromotion,
    RepoAction,
    Verification,
    ContinuityRecovery,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyReceiptProofProfile {
    pub effect_kind: EpiphanyReceiptEffectKind,
    pub owner_organ: String,
    pub required_before_promotion_document_types: Vec<String>,
    pub terminal_receipt_document_types: Vec<String>,
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
        contract: "Every sub-agent depends on the other sub-agents: Self routes, Face speaks, Imagination projects futures/scenes, Eyes seeks evidence, Proprioception models the Body, Hands acts through Substrate Gate grants, and Soul verifies. Continuity is protocol machinery, not a sub-agent identity.".to_string(),
    }
}

pub fn default_organ_dependency_matrix() -> Vec<EpiphanyOrganDependency> {
    EPIPHANY_STANDING_ORGANS
        .iter()
        .map(|organ| default_organ_dependencies_for(organ))
        .collect()
}

pub fn default_launch_organ_contract(
    authority_scope: &str,
    document_kind: &str,
    output_contract_id: &str,
) -> EpiphanyLaunchOrganContract {
    EpiphanyLaunchOrganContract {
        schema_version: EPIPHANY_LAUNCH_ORGAN_CONTRACT_SCHEMA_VERSION.to_string(),
        authority_scope: authority_scope.to_string(),
        document_kind: document_kind.to_string(),
        output_contract_id: output_contract_id.to_string(),
        owner_organ: owner_organ_for_authority_scope(authority_scope).to_string(),
        dependencies: default_organ_dependency_matrix(),
        receipt_proof_profiles: default_receipt_proof_profiles(),
        required_receipt_document_types: default_launch_required_receipts(),
        contract: "A worker launch is not naked task cargo: it carries the sub-agent dependency matrix, a receipt document catalogue, and effect-specific proof profiles. Mind gates state effects, Substrate Gate gates repo access, Eyes supplies evidence, Proprioception models the Body, Hands records action, Soul verifies, and Continuity protocols preserve recovery across rupture.".to_string(),
    }
}

pub fn default_receipt_proof_profiles() -> Vec<EpiphanyReceiptProofProfile> {
    vec![
        EpiphanyReceiptProofProfile {
            effect_kind: EpiphanyReceiptEffectKind::StateAdmission,
            owner_organ: "mind".to_string(),
            required_before_promotion_document_types: vec![MIND_GATEWAY_REVIEW_TYPE.to_string()],
            terminal_receipt_document_types: vec![
                MIND_STATE_COMMIT_RECEIPT_TYPE.to_string(),
                MIND_STATE_REJECTION_RECEIPT_TYPE.to_string(),
            ],
            contract: "Durable state admission requires Mind review before promotion and a Mind commit or rejection receipt after the decision resolves.".to_string(),
        },
        EpiphanyReceiptProofProfile {
            effect_kind: EpiphanyReceiptEffectKind::EvidencePromotion,
            owner_organ: "eyes".to_string(),
            required_before_promotion_document_types: vec![
                SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE.to_string(),
                EYES_EVIDENCE_PACKET_TYPE.to_string(),
                MIND_GATEWAY_REVIEW_TYPE.to_string(),
            ],
            terminal_receipt_document_types: vec![
                MIND_STATE_COMMIT_RECEIPT_TYPE.to_string(),
                MIND_STATE_REJECTION_RECEIPT_TYPE.to_string(),
                EYES_EVIDENCE_REFUSAL_RECEIPT_TYPE.to_string(),
            ],
            contract: "A source-dependent truth claim needs scoped substrate access, an Eyes evidence packet, and Mind admission before it becomes durable state.".to_string(),
        },
        EpiphanyReceiptProofProfile {
            effect_kind: EpiphanyReceiptEffectKind::RepoAction,
            owner_organ: "hands".to_string(),
            required_before_promotion_document_types: vec![
                SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE.to_string(),
                HANDS_PATCH_RECEIPT_TYPE.to_string(),
                SOUL_VERDICT_RECEIPT_TYPE.to_string(),
                MIND_GATEWAY_REVIEW_TYPE.to_string(),
            ],
            terminal_receipt_document_types: vec![
                SUBSTRATE_GATE_REPO_MUTATION_RECEIPT_TYPE.to_string(),
                HANDS_COMMIT_RECEIPT_TYPE.to_string(),
                HANDS_ROLLBACK_RECEIPT_TYPE.to_string(),
                HANDS_ACTION_REFUSAL_RECEIPT_TYPE.to_string(),
                SOUL_REGRESSION_RECEIPT_TYPE.to_string(),
                SOUL_VERIFICATION_REFUSAL_RECEIPT_TYPE.to_string(),
                MIND_STATE_COMMIT_RECEIPT_TYPE.to_string(),
                MIND_STATE_REJECTION_RECEIPT_TYPE.to_string(),
            ],
            contract: "A repository mutation needs substrate grant, Hands action proof, Soul verification, and Mind state admission before it can be called true.".to_string(),
        },
        EpiphanyReceiptProofProfile {
            effect_kind: EpiphanyReceiptEffectKind::Verification,
            owner_organ: "soul".to_string(),
            required_before_promotion_document_types: vec![
                SOUL_VERDICT_RECEIPT_TYPE.to_string(),
                MIND_GATEWAY_REVIEW_TYPE.to_string(),
            ],
            terminal_receipt_document_types: vec![
                SOUL_REVIEW_RECEIPT_TYPE.to_string(),
                SOUL_REGRESSION_RECEIPT_TYPE.to_string(),
                SOUL_VERIFICATION_REFUSAL_RECEIPT_TYPE.to_string(),
                MIND_STATE_COMMIT_RECEIPT_TYPE.to_string(),
                MIND_STATE_REJECTION_RECEIPT_TYPE.to_string(),
            ],
            contract: "Verification claims require Soul verdict proof before Mind admits the claim into durable state.".to_string(),
        },
        EpiphanyReceiptProofProfile {
            effect_kind: EpiphanyReceiptEffectKind::ContinuityRecovery,
            owner_organ: "continuity".to_string(),
            required_before_promotion_document_types: vec![
                CONTINUITY_RECOVERY_RECEIPT_TYPE.to_string(),
                MIND_GATEWAY_REVIEW_TYPE.to_string(),
            ],
            terminal_receipt_document_types: vec![
                MIND_STATE_COMMIT_RECEIPT_TYPE.to_string(),
                MIND_STATE_REJECTION_RECEIPT_TYPE.to_string(),
                CONTINUITY_REFUSAL_RECEIPT_TYPE.to_string(),
            ],
            contract: "Recovery and rupture-crossing claims require Continuity proof plus Mind admission; transcript residue is not durable memory.".to_string(),
        },
    ]
}

pub fn default_launch_required_receipts() -> Vec<String> {
    unique_strings(vec![
        MIND_GATEWAY_REVIEW_TYPE,
        MIND_STATE_COMMIT_RECEIPT_TYPE,
        MIND_STATE_REJECTION_RECEIPT_TYPE,
        SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE,
        SUBSTRATE_GATE_REPO_ACCESS_REFUSAL_RECEIPT_TYPE,
        SUBSTRATE_GATE_REPO_SNAPSHOT_RECEIPT_TYPE,
        SUBSTRATE_GATE_REPO_MUTATION_RECEIPT_TYPE,
        EYES_EVIDENCE_PACKET_TYPE,
        EYES_EVIDENCE_REFUSAL_RECEIPT_TYPE,
        HANDS_PATCH_RECEIPT_TYPE,
        HANDS_COMMIT_RECEIPT_TYPE,
        HANDS_ROLLBACK_RECEIPT_TYPE,
        HANDS_ACTION_REFUSAL_RECEIPT_TYPE,
        SOUL_VERDICT_RECEIPT_TYPE,
        SOUL_REGRESSION_RECEIPT_TYPE,
        SOUL_REVIEW_RECEIPT_TYPE,
        SOUL_VERIFICATION_REFUSAL_RECEIPT_TYPE,
        CONTINUITY_RECOVERY_RECEIPT_TYPE,
        CONTINUITY_REFUSAL_RECEIPT_TYPE,
    ])
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

fn owner_organ_for_authority_scope(authority_scope: &str) -> &'static str {
    let normalized = authority_scope.trim().to_ascii_lowercase();
    if normalized.contains("face") {
        "face"
    } else if normalized.contains("imagination") {
        "imagination"
    } else if normalized.contains("eyes") || normalized.contains("evidence") {
        "eyes"
    } else if normalized.contains("modeling") || normalized.contains("proprioception") {
        "proprioception"
    } else if normalized.contains("hands") || normalized.contains("implementation") {
        "hands"
    } else if normalized.contains("verification") || normalized.contains("soul") {
        "soul"
    } else if normalized.contains("reorient") || normalized.contains("continuity") {
        "continuity"
    } else {
        "self"
    }
}

fn unique_strings(items: Vec<&'static str>) -> Vec<String> {
    let mut out = Vec::new();
    for item in items {
        let item = item.to_string();
        if !out.contains(&item) {
            out.push(item);
        }
    }
    out
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

    #[test]
    fn launch_contract_carries_all_organs_and_gateway_receipts() {
        let contract = default_launch_organ_contract(
            "epiphany.role.verification",
            "role",
            "epiphany.worker.role_result.v0",
        );
        assert_eq!(contract.owner_organ, "soul");
        assert_eq!(contract.dependencies.len(), EPIPHANY_STANDING_ORGANS.len());
        assert!(
            contract
                .required_receipt_document_types
                .contains(&MIND_GATEWAY_REVIEW_TYPE.to_string())
        );
        assert!(
            contract
                .required_receipt_document_types
                .contains(&SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE.to_string())
        );
        assert!(
            contract
                .required_receipt_document_types
                .contains(&SOUL_VERDICT_RECEIPT_TYPE.to_string())
        );
        assert!(contract.receipt_proof_profiles.iter().any(|profile| {
            profile.effect_kind == EpiphanyReceiptEffectKind::StateAdmission
                && profile
                    .required_before_promotion_document_types
                    .contains(&MIND_GATEWAY_REVIEW_TYPE.to_string())
                && profile
                    .terminal_receipt_document_types
                    .contains(&MIND_STATE_COMMIT_RECEIPT_TYPE.to_string())
        }));
        assert!(contract.receipt_proof_profiles.iter().any(|profile| {
            profile.effect_kind == EpiphanyReceiptEffectKind::RepoAction
                && profile
                    .required_before_promotion_document_types
                    .contains(&HANDS_PATCH_RECEIPT_TYPE.to_string())
                && profile
                    .required_before_promotion_document_types
                    .contains(&SOUL_VERDICT_RECEIPT_TYPE.to_string())
        }));
        assert!(contract.contract.contains("not naked task cargo"));
    }
}
