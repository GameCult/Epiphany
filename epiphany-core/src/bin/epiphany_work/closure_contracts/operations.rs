#[derive(Debug, Deserialize)]
pub(super) struct RepoDeploymentConfig {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    pub(super) deployment: RepoDeploymentSettings,
    pub(super) cultmesh: RepoDeploymentCultMesh,
    pub(super) required_receipts: RepoDeploymentRequiredReceipts,
    pub(super) authority: RepoDeploymentAuthority,
}

impl RepoDeploymentConfig {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_deployment_config.v0"
            && self.safe_action_family == "repo.deployment_config"
    }

    pub(super) fn has_idunn_trigger_contract(&self) -> bool {
        let deployment = &self.deployment;
        !deployment.enabled
            && deployment.owner == "Idunn"
            && deployment.trigger == "git-push-observed-by-idunn"
            && deployment.watched_ref == "refs/heads/main"
            && deployment.deployment_script_ref == "deploy/idunn-deploy.ps1"
            && deployment.deployment_script_hash_required
            && deployment.deployment_script_review_required
            && deployment.host_access_policy_ref_required
            && !deployment.secret_values_embedded
            && deployment.rollback_plan_ref_required
            && deployment.aftercare_checks_required
            && deployment.idunn_receipt_required
            && deployment.aftercare_audit_required
    }

    pub(super) fn has_cultmesh_contract(&self) -> bool {
        let cultmesh = &self.cultmesh;
        cultmesh.local_verse == "gamecult-local"
            && cultmesh.capability_family == "gamecult.idunn.deployment"
            && cultmesh.intent_contract == "gamecult.idunn.deployment_intent.v0"
            && cultmesh.receipt_contract == "gamecult.idunn.deployment_receipt.v0"
            && cultmesh.aftercare_contract == "gamecult.idunn.deployment_aftercare_audit.v0"
            && cultmesh.daemon_owns_execution
    }

    pub(super) fn has_required_receipt_contract(&self) -> bool {
        let receipts = &self.required_receipts;
        receipts.mind_adoption == "epiphany.repo_work_mind_adoption_decision.v0"
            && receipts.soul_review == "epiphany.repo_work_closure_review.v0"
            && receipts.maintainer_review == "gamecult.maintainer.review_receipt.v0"
            && receipts.secret_policy == "epiphany.repo_secret_policy_request.v0"
            && receipts.idunn_deployment == "gamecult.idunn.deployment_receipt.v0"
            && receipts.aftercare_audit == "gamecult.idunn.deployment_aftercare_audit.v0"
    }

    pub(super) fn has_authority_seals(&self) -> bool {
        let authority = &self.authority;
        authority.configuration_only
            && !authority.direct_deployment_authority
            && !authority.direct_ssh_authority
            && !authority.direct_git_push_authority
            && !authority.direct_service_lifecycle_authority
            && !authority.direct_hands_authority
            && !authority.publication_authorized
            && !authority.merge_authorized
            && !authority.cross_body_mutation_authorized
            && !authority.private_verse_rummaging
            && authority.idunn_deployment_authority_required
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDeploymentSettings {
    pub(super) enabled: bool,
    pub(super) owner: String,
    pub(super) trigger: String,
    pub(super) watched_ref: String,
    pub(super) deployment_script_ref: String,
    pub(super) deployment_script_hash_required: bool,
    pub(super) deployment_script_review_required: bool,
    pub(super) host_access_policy_ref_required: bool,
    pub(super) secret_values_embedded: bool,
    pub(super) rollback_plan_ref_required: bool,
    pub(super) aftercare_checks_required: bool,
    pub(super) idunn_receipt_required: bool,
    pub(super) aftercare_audit_required: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDeploymentCultMesh {
    pub(super) local_verse: String,
    pub(super) capability_family: String,
    pub(super) intent_contract: String,
    pub(super) receipt_contract: String,
    pub(super) aftercare_contract: String,
    pub(super) daemon_owns_execution: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDeploymentRequiredReceipts {
    pub(super) mind_adoption: String,
    pub(super) soul_review: String,
    pub(super) maintainer_review: String,
    pub(super) secret_policy: String,
    pub(super) idunn_deployment: String,
    pub(super) aftercare_audit: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDeploymentAuthority {
    pub(super) configuration_only: bool,
    pub(super) direct_deployment_authority: bool,
    pub(super) direct_ssh_authority: bool,
    pub(super) direct_git_push_authority: bool,
    pub(super) direct_service_lifecycle_authority: bool,
    pub(super) direct_hands_authority: bool,
    pub(super) publication_authorized: bool,
    pub(super) merge_authorized: bool,
    pub(super) cross_body_mutation_authorized: bool,
    pub(super) private_verse_rummaging: bool,
    pub(super) idunn_deployment_authority_required: bool,
}

pub(super) fn parse_repo_deployment_config(text: &str) -> Result<RepoDeploymentConfig> {
    toml::from_str(text).context("deployment config is not valid typed TOML")
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoSecretPolicyRequest {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    pub(super) request: RepoSecretPolicyRequestBody,
    pub(super) antecedents: RepoSecretPolicyAntecedents,
    pub(super) required_receipts: RepoSecretPolicyReceipts,
    pub(super) security_packet: RepoSecretPolicyPacket,
    pub(super) authority: RepoSecretPolicyAuthority,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoSecretPolicyRequestBody {
    pub(super) status: String,
    pub(super) routing_owner: String,
    pub(super) required_reviewers: Vec<String>,
    pub(super) policy_admission_owner: String,
    pub(super) requested_effect: String,
    pub(super) requires_secret_inventory_without_values: bool,
    pub(super) requires_write_permission_scope: bool,
    pub(super) requires_public_private_export_boundary: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoSecretPolicyAntecedents {
    pub(super) source_grounding_required: bool,
    pub(super) soul_review_required: bool,
    pub(super) mind_adoption_required: bool,
    pub(super) maintainer_review_required: bool,
    pub(super) bifrost_publication_review_required: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoSecretPolicyReceipts {
    pub(super) source_grounding: String,
    pub(super) soul_review: String,
    pub(super) mind_adoption: String,
    pub(super) maintainer_review: String,
    pub(super) bifrost_publication_review: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoSecretPolicyPacket {
    pub(super) requires_secret_locations_without_values: bool,
    pub(super) requires_credential_owner: bool,
    pub(super) requires_write_scope_matrix: bool,
    pub(super) requires_public_export_redaction_rules: bool,
    pub(super) requires_deployment_authority_owner: bool,
    pub(super) requires_incident_rollback_plan: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoSecretPolicyAuthority {
    pub(super) direct_secret_access_authority: bool,
    pub(super) secret_value_materialization: bool,
    pub(super) write_permission_authority: bool,
    pub(super) deployment_authority: bool,
    pub(super) publication_authorized: bool,
    pub(super) merge_authorized: bool,
    pub(super) service_lifecycle_authority: bool,
    pub(super) cross_body_mutation_authorized: bool,
    pub(super) private_verse_rummaging: bool,
    pub(super) maintainer_security_review_required: bool,
    pub(super) soul_security_verification_required: bool,
    pub(super) mind_policy_admission_required: bool,
    pub(super) bifrost_publication_review_required: bool,
}

impl RepoSecretPolicyRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_secret_policy_request.v0"
            && self.safe_action_family == "repo.secret_policy_request"
    }

    pub(super) fn awaits_security_review(&self) -> bool {
        let request = &self.request;
        request.status == "awaiting-security-review"
            && request.routing_owner == "Self"
            && request.required_reviewers == ["Maintainer", "Soul", "Mind", "Bifrost"]
            && request.policy_admission_owner == "Mind"
            && request.requested_effect == "review-repo-secret-and-write-permission-policy"
            && request.requires_secret_inventory_without_values
            && request.requires_write_permission_scope
            && request.requires_public_private_export_boundary
    }

    pub(super) fn has_review_antecedents(&self) -> bool {
        let antecedents = &self.antecedents;
        antecedents.source_grounding_required
            && antecedents.soul_review_required
            && antecedents.mind_adoption_required
            && antecedents.maintainer_review_required
            && antecedents.bifrost_publication_review_required
    }

    pub(super) fn has_receipt_contract(&self) -> bool {
        let receipts = &self.required_receipts;
        receipts.source_grounding == "epiphany.eyes.evidence_packet"
            && receipts.soul_review == "epiphany.repo_work_closure_review.v0"
            && receipts.mind_adoption == "epiphany.repo_work_mind_adoption_decision.v0"
            && receipts.maintainer_review == "gamecult.maintainer.review_receipt.v0"
            && receipts.bifrost_publication_review
                == "gamecult.bifrost.publication_review_receipt.v0"
    }

    pub(super) fn has_security_packet_contract(&self) -> bool {
        let packet = &self.security_packet;
        packet.requires_secret_locations_without_values
            && packet.requires_credential_owner
            && packet.requires_write_scope_matrix
            && packet.requires_public_export_redaction_rules
            && packet.requires_deployment_authority_owner
            && packet.requires_incident_rollback_plan
    }

    pub(super) fn has_authority_seals(&self) -> bool {
        let authority = &self.authority;
        !authority.direct_secret_access_authority
            && !authority.secret_value_materialization
            && !authority.write_permission_authority
            && !authority.deployment_authority
            && !authority.publication_authorized
            && !authority.merge_authorized
            && !authority.service_lifecycle_authority
            && !authority.cross_body_mutation_authorized
            && !authority.private_verse_rummaging
            && authority.maintainer_security_review_required
            && authority.soul_security_verification_required
            && authority.mind_policy_admission_required
            && authority.bifrost_publication_review_required
    }
}

pub(super) fn parse_repo_secret_policy_request(text: &str) -> Result<RepoSecretPolicyRequest> {
    toml::from_str(text).context("secret policy request is not valid typed TOML")
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDependencyPolicyRequest {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    pub(super) request: RepoDependencyPolicyRequestBody,
    pub(super) antecedents: RepoDependencyPolicyAntecedents,
    pub(super) required_receipts: RepoDependencyPolicyReceipts,
    pub(super) dependency_packet: RepoDependencyPolicyPacket,
    pub(super) authority: RepoDependencyPolicyAuthority,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDependencyPolicyRequestBody {
    pub(super) status: String,
    pub(super) routing_owner: String,
    pub(super) required_reviewers: Vec<String>,
    pub(super) policy_admission_owner: String,
    pub(super) requested_effect: String,
    pub(super) requires_manifest_inventory: bool,
    pub(super) requires_lockfile_policy: bool,
    pub(super) requires_package_manager_command_review: bool,
    pub(super) requires_network_fetch_policy: bool,
    pub(super) requires_vulnerability_review: bool,
    pub(super) requires_license_review: bool,
    pub(super) requires_rollback_plan: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDependencyPolicyAntecedents {
    pub(super) source_grounding_required: bool,
    pub(super) eyes_evidence_required: bool,
    pub(super) soul_review_required: bool,
    pub(super) mind_adoption_required: bool,
    pub(super) maintainer_review_required: bool,
    pub(super) bifrost_publication_review_required: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDependencyPolicyReceipts {
    pub(super) source_grounding: String,
    pub(super) soul_review: String,
    pub(super) mind_adoption: String,
    pub(super) maintainer_review: String,
    pub(super) bifrost_publication_review: String,
    pub(super) dependency_audit: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDependencyPolicyPacket {
    pub(super) requires_manifest_paths: bool,
    pub(super) requires_lockfile_paths: bool,
    pub(super) requires_package_manager_commands: bool,
    pub(super) requires_vulnerability_sources: bool,
    pub(super) requires_license_constraints: bool,
    pub(super) requires_vendored_code_policy: bool,
    pub(super) requires_update_cadence: bool,
    pub(super) requires_private_state_redaction_check: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDependencyPolicyAuthority {
    pub(super) direct_dependency_update_authority: bool,
    pub(super) direct_package_install_authority: bool,
    pub(super) direct_lockfile_mutation_authority: bool,
    pub(super) direct_network_fetch_authority: bool,
    pub(super) direct_ci_mutation_authority: bool,
    pub(super) direct_hands_authority: bool,
    pub(super) publication_authorized: bool,
    pub(super) merge_authorized: bool,
    pub(super) deployment_authority: bool,
    pub(super) service_lifecycle_authority: bool,
    pub(super) cross_body_mutation_authorized: bool,
    pub(super) private_verse_rummaging: bool,
    pub(super) maintainer_dependency_review_required: bool,
    pub(super) soul_dependency_verification_required: bool,
    pub(super) mind_policy_admission_required: bool,
    pub(super) bifrost_publication_review_required: bool,
    pub(super) supply_chain_audit_required: bool,
}

impl RepoDependencyPolicyRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_dependency_policy_request.v0"
            && self.safe_action_family == "repo.dependency_policy_request"
    }

    pub(super) fn awaits_review(&self) -> bool {
        let request = &self.request;
        request.status == "awaiting-dependency-policy-review"
            && request.routing_owner == "Self"
            && request.required_reviewers == ["Maintainer", "Soul", "Mind", "Bifrost"]
            && request.policy_admission_owner == "Mind"
            && request.requested_effect == "review-repo-dependency-and-supply-chain-policy"
            && request.requires_manifest_inventory
            && request.requires_lockfile_policy
            && request.requires_package_manager_command_review
            && request.requires_network_fetch_policy
            && request.requires_vulnerability_review
            && request.requires_license_review
            && request.requires_rollback_plan
    }

    pub(super) fn has_antecedents(&self) -> bool {
        let value = &self.antecedents;
        value.source_grounding_required
            && value.eyes_evidence_required
            && value.soul_review_required
            && value.mind_adoption_required
            && value.maintainer_review_required
            && value.bifrost_publication_review_required
    }

    pub(super) fn has_receipt_contract(&self) -> bool {
        let value = &self.required_receipts;
        value.source_grounding == "epiphany.eyes.evidence_packet"
            && value.soul_review == "epiphany.repo_work_closure_review.v0"
            && value.mind_adoption == "epiphany.repo_work_mind_adoption_decision.v0"
            && value.maintainer_review == "gamecult.maintainer.review_receipt.v0"
            && value.bifrost_publication_review == "gamecult.bifrost.publication_review_receipt.v0"
            && value.dependency_audit == "gamecult.supply_chain.dependency_audit_receipt.v0"
    }

    pub(super) fn has_packet_contract(&self) -> bool {
        let value = &self.dependency_packet;
        value.requires_manifest_paths
            && value.requires_lockfile_paths
            && value.requires_package_manager_commands
            && value.requires_vulnerability_sources
            && value.requires_license_constraints
            && value.requires_vendored_code_policy
            && value.requires_update_cadence
            && value.requires_private_state_redaction_check
    }

    pub(super) fn has_authority_seals(&self) -> bool {
        let value = &self.authority;
        !value.direct_dependency_update_authority
            && !value.direct_package_install_authority
            && !value.direct_lockfile_mutation_authority
            && !value.direct_network_fetch_authority
            && !value.direct_ci_mutation_authority
            && !value.direct_hands_authority
            && !value.publication_authorized
            && !value.merge_authorized
            && !value.deployment_authority
            && !value.service_lifecycle_authority
            && !value.cross_body_mutation_authorized
            && !value.private_verse_rummaging
            && value.maintainer_dependency_review_required
            && value.soul_dependency_verification_required
            && value.mind_policy_admission_required
            && value.bifrost_publication_review_required
            && value.supply_chain_audit_required
    }
}

pub(super) fn parse_repo_dependency_policy_request(
    text: &str,
) -> Result<RepoDependencyPolicyRequest> {
    toml::from_str(text).context("dependency policy request is not valid typed TOML")
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDeploymentRequest {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    pub(super) request: RepoDeploymentRequestBody,
    pub(super) antecedents: RepoDeploymentRequestAntecedents,
    pub(super) required_receipts: RepoDeploymentRequestReceipts,
    pub(super) deployment_packet: RepoDeploymentRequestPacket,
    pub(super) authority: RepoDeploymentRequestAuthority,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDeploymentRequestBody {
    pub(super) status: String,
    pub(super) routing_owner: String,
    pub(super) required_reviewers: Vec<String>,
    pub(super) execution_owner: String,
    pub(super) requested_effect: String,
    pub(super) deployment_trigger: String,
    pub(super) requires_explicit_deployment_policy: bool,
    pub(super) requires_idunn_receipt: bool,
    pub(super) requires_aftercare_audit: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDeploymentRequestAntecedents {
    pub(super) source_grounding_required: bool,
    pub(super) mind_adoption_required: bool,
    pub(super) soul_review_required: bool,
    pub(super) maintainer_review_required: bool,
    pub(super) secret_policy_review_required: bool,
    pub(super) bifrost_publication_review_required: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDeploymentRequestReceipts {
    pub(super) source_grounding: String,
    pub(super) mind_adoption: String,
    pub(super) soul_review: String,
    pub(super) maintainer_review: String,
    pub(super) secret_policy: String,
    pub(super) bifrost_publication_review: String,
    pub(super) idunn_deployment: String,
    pub(super) aftercare_audit: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDeploymentRequestPacket {
    pub(super) requires_target_environment: bool,
    pub(super) requires_git_ref_or_branch: bool,
    pub(super) requires_deployment_script_ref: bool,
    pub(super) requires_script_hash_or_review_ref: bool,
    pub(super) requires_host_access_policy_ref: bool,
    pub(super) requires_secret_policy_ref: bool,
    pub(super) requires_rollback_plan: bool,
    pub(super) requires_aftercare_checks: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDeploymentRequestAuthority {
    pub(super) direct_deployment_authority: bool,
    pub(super) direct_ssh_authority: bool,
    pub(super) direct_git_push_authority: bool,
    pub(super) direct_service_lifecycle_authority: bool,
    pub(super) direct_hands_authority: bool,
    pub(super) publication_authorized: bool,
    pub(super) merge_authorized: bool,
    pub(super) cross_body_mutation_authorized: bool,
    pub(super) private_verse_rummaging: bool,
    pub(super) idunn_deployment_authority_required: bool,
}

impl RepoDeploymentRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_deployment_request.v0"
            && self.safe_action_family == "repo.deployment_request"
    }

    pub(super) fn awaits_idunn_review(&self) -> bool {
        let value = &self.request;
        value.status == "awaiting-idunn-review"
            && value.routing_owner == "Self"
            && value.required_reviewers == ["Maintainer", "Soul", "Mind", "Bifrost"]
            && value.execution_owner == "Idunn"
            && value.requested_effect == "review-repo-deployment-trigger-and-script"
            && value.deployment_trigger == "git-push-observed-by-idunn"
            && value.requires_explicit_deployment_policy
            && value.requires_idunn_receipt
            && value.requires_aftercare_audit
    }

    pub(super) fn has_antecedents(&self) -> bool {
        let value = &self.antecedents;
        value.source_grounding_required
            && value.mind_adoption_required
            && value.soul_review_required
            && value.maintainer_review_required
            && value.secret_policy_review_required
            && value.bifrost_publication_review_required
    }

    pub(super) fn has_receipt_contract(&self) -> bool {
        let value = &self.required_receipts;
        value.source_grounding == "epiphany.eyes.evidence_packet"
            && value.mind_adoption == "epiphany.repo_work_mind_adoption_decision.v0"
            && value.soul_review == "epiphany.repo_work_closure_review.v0"
            && value.maintainer_review == "gamecult.maintainer.review_receipt.v0"
            && value.secret_policy == "epiphany.repo_secret_policy_request.v0"
            && value.bifrost_publication_review == "gamecult.bifrost.publication_review_receipt.v0"
            && value.idunn_deployment == "gamecult.idunn.deployment_receipt.v0"
            && value.aftercare_audit == "gamecult.idunn.deployment_aftercare_audit.v0"
    }

    pub(super) fn has_packet_contract(&self) -> bool {
        let value = &self.deployment_packet;
        value.requires_target_environment
            && value.requires_git_ref_or_branch
            && value.requires_deployment_script_ref
            && value.requires_script_hash_or_review_ref
            && value.requires_host_access_policy_ref
            && value.requires_secret_policy_ref
            && value.requires_rollback_plan
            && value.requires_aftercare_checks
    }

    pub(super) fn has_authority_seals(&self) -> bool {
        let value = &self.authority;
        !value.direct_deployment_authority
            && !value.direct_ssh_authority
            && !value.direct_git_push_authority
            && !value.direct_service_lifecycle_authority
            && !value.direct_hands_authority
            && !value.publication_authorized
            && !value.merge_authorized
            && !value.cross_body_mutation_authorized
            && !value.private_verse_rummaging
            && value.idunn_deployment_authority_required
    }
}

pub(super) fn parse_repo_deployment_request(text: &str) -> Result<RepoDeploymentRequest> {
    toml::from_str(text).context("deployment request is not valid typed TOML")
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoDoctrineUpdateRequest {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    request: RepoDoctrineRequestBody,
    antecedents: std::collections::BTreeMap<String, bool>,
    required_receipts: std::collections::BTreeMap<String, String>,
    doctrine_packet: std::collections::BTreeMap<String, bool>,
    authority: std::collections::BTreeMap<String, bool>,
}
impl RepoDoctrineUpdateRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_doctrine_update_request.v0"
            && self.safe_action_family == "repo.doctrine_update_request"
    }
    pub(super) fn has_coherent_routing(&self) -> bool {
        let r = &self.request;
        r.status == "awaiting-doctrine-review"
            && r.routing_owner == "Self"
            && r.required_reviewers == ["Maintainer", "Mind", "Soul"]
            && r.doctrine_admission_owner == "Mind"
            && r.mutation_owner == "Hands"
            && r.requested_effect == "review-repo-agent-doctrine-update"
            && r.doctrine_target == "AGENTS.md"
            && r.requires_source_grounding
            && r.maintainer_review_required
    }
    pub(super) fn has_antecedent_contract(&self) -> bool {
        [
            "persona_or_human_feedback_required",
            "imagination_plan_required",
            "mind_adoption_required",
            "soul_review_required",
            "maintainer_review_required",
        ]
        .iter()
        .all(|key| self.antecedents.get(*key) == Some(&true))
    }
    pub(super) fn has_receipt_contract(&self) -> bool {
        [
            (
                "imagination_plan",
                "epiphany.repo_work_imagination_action_items_receipt.v0",
            ),
            (
                "mind_adoption",
                "epiphany.repo_work_mind_adoption_decision.v0",
            ),
            ("soul_review", "epiphany.repo_work_closure_review.v0"),
            ("maintainer_review", "gamecult.maintainer.review_receipt.v0"),
            ("hands_commit", "epiphany.hands.commit_receipt"),
        ]
        .iter()
        .all(|(key, value)| self.required_receipts.get(*key).is_some_and(|v| v == value))
    }
    pub(super) fn has_packet_contract(&self) -> bool {
        [
            "requires_current_doctrine_ref",
            "requires_proposed_change_summary",
            "requires_invariant_impact",
            "requires_rehydration_impact",
            "requires_rollback_plan",
            "requires_private_state_redaction_check",
        ]
        .iter()
        .all(|key| self.doctrine_packet.get(*key) == Some(&true))
    }
    pub(super) fn has_authority_seals(&self) -> bool {
        [
            "direct_doctrine_mutation_authority",
            "direct_hands_authority",
            "direct_mind_state_commit",
            "publication_authorized",
            "merge_authorized",
            "service_lifecycle_authority",
            "cross_body_mutation_authorized",
            "private_verse_rummaging",
        ]
        .iter()
        .all(|key| self.authority.get(*key) == Some(&false))
            && [
                "maintainer_review_required",
                "mind_admission_required",
                "hands_receipts_required",
            ]
            .iter()
            .all(|key| self.authority.get(*key) == Some(&true))
    }
}
#[derive(Debug, Deserialize)]
struct RepoDoctrineRequestBody {
    status: String,
    routing_owner: String,
    required_reviewers: Vec<String>,
    doctrine_admission_owner: String,
    mutation_owner: String,
    requested_effect: String,
    doctrine_target: String,
    requires_source_grounding: bool,
    maintainer_review_required: bool,
}
pub(super) fn parse_repo_doctrine_update_request(text: &str) -> Result<RepoDoctrineUpdateRequest> {
    toml::from_str(text).context("doctrine update request is not valid typed TOML")
}
