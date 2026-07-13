use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct RepoToolRequest {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    pub(super) request: RepoToolRequestBody,
    pub(super) cultmesh: RepoToolRequestCultMesh,
    pub(super) odin: RepoToolRequestOdin,
    pub(super) authority: RepoToolRequestAuthority,
}

impl RepoToolRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_tool_request.v0"
            && self.safe_action_family == "repo.tool_request"
    }

    pub(super) fn has_request_contract(&self) -> bool {
        self.request.target_directory == "gamecult-local/daemon-tool-directory"
            && self.request.target_capability == "daemon-tool-capability:selected-by-review"
            && self.request.operation == "submitTypedToolIntent"
    }

    pub(super) fn has_cultmesh_contract(&self) -> bool {
        self.cultmesh.intent_contract == "epiphany.cultmesh.daemon_tool_invocation_intent.v0"
            && self.cultmesh.receipt_contract
                == "epiphany.cultmesh.daemon_tool_invocation_receipt.v0"
            && self.cultmesh.host_daemon_owns_execution
            && !self.cultmesh.requester_owns_request
            && self.cultmesh.requires_host_liveness_ready
            && self.cultmesh.requires_cultmesh_receipts
    }

    pub(super) fn has_odin_contract(&self) -> bool {
        self.odin.discoverable
            && self.odin.preserves_provider_ownership
            && !self.odin.private_verse_passthrough
    }

    pub(super) fn has_authority_seals(&self) -> bool {
        !self.authority.direct_tool_execution
            && !self.authority.arbitrary_shell_authority
            && !self.authority.hands_action_authority
            && !self.authority.state_commit_authority
            && !self.authority.publication_authority
            && !self.authority.service_lifecycle_authority
            && !self.authority.cross_body_mutation_authority
            && !self.authority.private_verse_rummaging
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoToolRequestBody {
    target_directory: String,
    target_capability: String,
    operation: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoToolRequestCultMesh {
    intent_contract: String,
    receipt_contract: String,
    host_daemon_owns_execution: bool,
    requester_owns_request: bool,
    requires_host_liveness_ready: bool,
    requires_cultmesh_receipts: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoToolRequestOdin {
    discoverable: bool,
    preserves_provider_ownership: bool,
    private_verse_passthrough: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoToolRequestAuthority {
    pub(super) direct_tool_execution: bool,
    arbitrary_shell_authority: bool,
    hands_action_authority: bool,
    state_commit_authority: bool,
    publication_authority: bool,
    service_lifecycle_authority: bool,
    cross_body_mutation_authority: bool,
    private_verse_rummaging: bool,
}

pub(super) fn parse_repo_tool_request(text: &str) -> Result<RepoToolRequest> {
    toml::from_str(text).context("tool request is not valid typed TOML")
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoMetricsRequest {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    request: RepoMetricsRequestBody,
    antecedents: RepoMetricsAntecedents,
    required_receipts: RepoMetricsReceipts,
    metrics_packet: RepoMetricsPacket,
    authority: RepoMetricsAuthority,
}

impl RepoMetricsRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_metrics_request.v0"
            && self.safe_action_family == "repo.metrics_request"
    }

    pub(super) fn awaits_owned_review(&self) -> bool {
        self.request.status == "awaiting-metrics-review"
            && self.request.requested_owner == "Bifrost/Maintainer"
            && self.request.requested_effect == "record-compute-review-and-artifact-accounting"
            && !self.request.publication_request_ref.is_empty()
            && !self.request.credit_request_ref.is_empty()
            && !self.request.artifact_acceptance_request_ref.is_empty()
    }

    pub(super) fn has_antecedent_contract(&self) -> bool {
        let a = &self.antecedents;
        a.closure_review_required
            && a.soul_verdict_required
            && a.mind_commit_required
            && a.public_proof_required
            && a.accepted_artifact_required
            && a.credit_request_required
    }

    pub(super) fn has_receipt_contract(&self) -> bool {
        let r = &self.required_receipts;
        r.closure_review == "epiphany.repo_work_closure_review.v0"
            && r.soul_verdict == "epiphany.soul.verification_verdict"
            && r.mind_commit == "epiphany.mind.state_commit_receipt"
            && r.public_proof == "epiphany.repo_work_public_proof_bundle.v0"
            && r.accepted_artifact == "gamecult.artifact.acceptance_receipt.v0"
            && r.model_spend == "gamecult.metrics.model_spend_receipt.v0"
            && r.review_load == "gamecult.metrics.review_load_receipt.v0"
            && r.credit_readback == "gamecult.bifrost.credit_readback_receipt.v0"
    }

    pub(super) fn has_metrics_packet(&self) -> bool {
        let p = &self.metrics_packet;
        p.requires_model_call_count
            && p.requires_token_or_cost_summary
            && p.requires_review_minutes_or_count
            && p.requires_accepted_artifact_ref
            && p.requires_public_proof_ref
            && p.requires_credit_readback_ref
            && p.requires_private_state_redaction_check
    }

    pub(super) fn has_authority_seals(&self) -> bool {
        let a = &self.authority;
        a.branch_local_only
            && !a.metrics_ledger_authorized
            && !a.spend_authorized
            && !a.review_load_authorized
            && !a.credit_ledger_authorized
            && !a.github_pr_authorized
            && !a.merge_authorized
            && !a.publication_authorized
            && !a.upstream_sync_authorized
            && !a.hands_action_authorized
            && !a.service_lifecycle_authority
            && !a.cross_body_mutation_authorized
            && !a.private_verse_rummaging
            && a.bifrost_or_maintainer_metrics_authority_required
    }
}

#[derive(Debug, Deserialize)]
struct RepoMetricsRequestBody {
    status: String,
    requested_owner: String,
    requested_effect: String,
    publication_request_ref: String,
    credit_request_ref: String,
    artifact_acceptance_request_ref: String,
}

#[derive(Debug, Deserialize)]
struct RepoMetricsAntecedents {
    closure_review_required: bool,
    soul_verdict_required: bool,
    mind_commit_required: bool,
    public_proof_required: bool,
    accepted_artifact_required: bool,
    credit_request_required: bool,
}

#[derive(Debug, Deserialize)]
struct RepoMetricsReceipts {
    closure_review: String,
    soul_verdict: String,
    mind_commit: String,
    public_proof: String,
    accepted_artifact: String,
    model_spend: String,
    review_load: String,
    credit_readback: String,
}

#[derive(Debug, Deserialize)]
struct RepoMetricsPacket {
    requires_model_call_count: bool,
    requires_token_or_cost_summary: bool,
    requires_review_minutes_or_count: bool,
    requires_accepted_artifact_ref: bool,
    requires_public_proof_ref: bool,
    requires_credit_readback_ref: bool,
    requires_private_state_redaction_check: bool,
}

#[derive(Debug, Deserialize)]
struct RepoMetricsAuthority {
    branch_local_only: bool,
    metrics_ledger_authorized: bool,
    spend_authorized: bool,
    review_load_authorized: bool,
    credit_ledger_authorized: bool,
    github_pr_authorized: bool,
    merge_authorized: bool,
    publication_authorized: bool,
    upstream_sync_authorized: bool,
    hands_action_authorized: bool,
    service_lifecycle_authority: bool,
    cross_body_mutation_authorized: bool,
    private_verse_rummaging: bool,
    bifrost_or_maintainer_metrics_authority_required: bool,
}

pub(super) fn parse_repo_metrics_request(text: &str) -> Result<RepoMetricsRequest> {
    toml::from_str(text).context("metrics request is not valid typed TOML")
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoArtifactAcceptanceRequest {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    request: RepoArtifactAcceptanceRequestBody,
    antecedents: RepoArtifactAcceptanceAntecedents,
    required_receipts: RepoArtifactAcceptanceReceipts,
    artifact_packet: RepoArtifactAcceptancePacket,
    authority: RepoArtifactAcceptanceAuthority,
}

impl RepoArtifactAcceptanceRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_artifact_acceptance_request.v0"
            && self.safe_action_family == "repo.artifact_acceptance_request"
    }

    pub(super) fn awaits_owned_review(&self) -> bool {
        let r = &self.request;
        r.status == "awaiting-artifact-acceptance-review"
            && r.requested_owner == "Maintainer/Bifrost"
            && r.requested_effect == "record-accepted-artifact-for-reviewed-branch-work"
            && !r.verification_request_ref.is_empty()
            && !r.maintainer_review_request_ref.is_empty()
            && !r.publication_request_ref.is_empty()
    }

    pub(super) fn has_antecedent_contract(&self) -> bool {
        let a = &self.antecedents;
        a.closure_review_required
            && a.soul_verdict_required
            && a.mind_commit_required
            && a.public_proof_required
            && a.maintainer_review_required
            && a.hands_commit_required
    }

    pub(super) fn has_receipt_contract(&self) -> bool {
        let r = &self.required_receipts;
        r.closure_review == "epiphany.repo_work_closure_review.v0"
            && r.soul_verdict == "epiphany.soul.verification_verdict"
            && r.mind_commit == "epiphany.mind.state_commit_receipt"
            && r.public_proof == "epiphany.repo_work_public_proof_bundle.v0"
            && r.maintainer_review == "gamecult.maintainer.review_receipt.v0"
            && r.hands_commit == "epiphany.hands.commit_receipt"
            && r.accepted_artifact == "gamecult.artifact.acceptance_receipt.v0"
    }

    pub(super) fn has_artifact_packet(&self) -> bool {
        let p = &self.artifact_packet;
        p.requires_artifact_ref
            && p.requires_commit_sha
            && p.requires_changed_path_list
            && p.requires_review_verdict
            && p.requires_public_proof_ref
            && p.requires_acceptance_rationale
            && p.requires_private_state_redaction_check
    }

    pub(super) fn has_authority_seals(&self) -> bool {
        let a = &self.authority;
        a.branch_local_only
            && !a.artifact_acceptance_authorized
            && !a.credit_ledger_authorized
            && !a.github_pr_authorized
            && !a.merge_authorized
            && !a.publication_authorized
            && !a.upstream_sync_authorized
            && !a.hands_action_authorized
            && !a.service_lifecycle_authority
            && !a.cross_body_mutation_authorized
            && !a.private_verse_rummaging
            && a.maintainer_or_bifrost_acceptance_authority_required
    }
}

#[derive(Debug, Deserialize)]
struct RepoArtifactAcceptanceRequestBody {
    status: String,
    requested_owner: String,
    requested_effect: String,
    verification_request_ref: String,
    maintainer_review_request_ref: String,
    publication_request_ref: String,
}

#[derive(Debug, Deserialize)]
struct RepoArtifactAcceptanceAntecedents {
    closure_review_required: bool,
    soul_verdict_required: bool,
    mind_commit_required: bool,
    public_proof_required: bool,
    maintainer_review_required: bool,
    hands_commit_required: bool,
}

#[derive(Debug, Deserialize)]
struct RepoArtifactAcceptanceReceipts {
    closure_review: String,
    soul_verdict: String,
    mind_commit: String,
    public_proof: String,
    maintainer_review: String,
    hands_commit: String,
    accepted_artifact: String,
}

#[derive(Debug, Deserialize)]
struct RepoArtifactAcceptancePacket {
    requires_artifact_ref: bool,
    requires_commit_sha: bool,
    requires_changed_path_list: bool,
    requires_review_verdict: bool,
    requires_public_proof_ref: bool,
    requires_acceptance_rationale: bool,
    requires_private_state_redaction_check: bool,
}

#[derive(Debug, Deserialize)]
struct RepoArtifactAcceptanceAuthority {
    branch_local_only: bool,
    artifact_acceptance_authorized: bool,
    credit_ledger_authorized: bool,
    github_pr_authorized: bool,
    merge_authorized: bool,
    publication_authorized: bool,
    upstream_sync_authorized: bool,
    hands_action_authorized: bool,
    service_lifecycle_authority: bool,
    cross_body_mutation_authorized: bool,
    private_verse_rummaging: bool,
    maintainer_or_bifrost_acceptance_authority_required: bool,
}

pub(super) fn parse_repo_artifact_acceptance_request(
    text: &str,
) -> Result<RepoArtifactAcceptanceRequest> {
    toml::from_str(text).context("artifact acceptance request is not valid typed TOML")
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoCreditRequest {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    request: RepoCreditRequestBody,
    antecedents: RepoCreditAntecedents,
    required_receipts: RepoCreditReceipts,
    credit_packet: RepoCreditPacket,
    authority: RepoCreditAuthority,
}

impl RepoCreditRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_credit_request.v0"
            && self.safe_action_family == "repo.credit_request"
    }
    pub(super) fn awaits_bifrost_review(&self) -> bool {
        let r = &self.request;
        r.status == "awaiting-bifrost-credit-review"
            && r.requested_owner == "Bifrost"
            && r.requested_effect == "record-credit-for-redacted-proof-and-accepted-artifact"
            && !r.publication_request_ref.is_empty()
            && !r.maintainer_review_request_ref.is_empty()
            && !r.pr_request_ref.is_empty()
    }
    pub(super) fn has_antecedent_contract(&self) -> bool {
        let a = &self.antecedents;
        a.closure_review_required
            && a.soul_verdict_required
            && a.mind_commit_required
            && a.public_proof_required
            && a.maintainer_review_required
            && a.accepted_artifact_required
            && a.authorship_context_required
    }
    pub(super) fn has_receipt_contract(&self) -> bool {
        let r = &self.required_receipts;
        r.closure_review == "epiphany.repo_work_closure_review.v0"
            && r.soul_verdict == "epiphany.soul.verification_verdict"
            && r.mind_commit == "epiphany.mind.state_commit_receipt"
            && r.public_proof == "epiphany.repo_work_public_proof_bundle.v0"
            && r.maintainer_review == "gamecult.maintainer.review_receipt.v0"
            && r.accepted_artifact == "gamecult.artifact.acceptance_receipt.v0"
            && r.credit_ledger == "gamecult.bifrost.credit_receipt.v0"
            && r.credit_readback == "gamecult.bifrost.credit_readback_receipt.v0"
    }
    pub(super) fn has_credit_packet(&self) -> bool {
        let p = &self.credit_packet;
        p.requires_author_identity
            && p.requires_reviewer_identity
            && p.requires_accepted_artifact_ref
            && p.requires_public_proof_ref
            && p.requires_changed_path_list
            && p.requires_credit_ledger_target
            && p.requires_private_state_redaction_check
    }
    pub(super) fn has_authority_seals(&self) -> bool {
        let a = &self.authority;
        a.branch_local_only
            && !a.credit_ledger_authorized
            && !a.bifrost_publication_authorized
            && !a.github_pr_authorized
            && !a.merge_authorized
            && !a.publication_authorized
            && !a.upstream_sync_authorized
            && !a.hands_action_authorized
            && !a.service_lifecycle_authority
            && !a.cross_body_mutation_authorized
            && !a.private_verse_rummaging
            && a.bifrost_credit_authority_required
    }
}

#[derive(Debug, Deserialize)]
struct RepoCreditRequestBody {
    status: String,
    requested_owner: String,
    requested_effect: String,
    publication_request_ref: String,
    maintainer_review_request_ref: String,
    pr_request_ref: String,
}
#[derive(Debug, Deserialize)]
struct RepoCreditAntecedents {
    closure_review_required: bool,
    soul_verdict_required: bool,
    mind_commit_required: bool,
    public_proof_required: bool,
    maintainer_review_required: bool,
    accepted_artifact_required: bool,
    authorship_context_required: bool,
}
#[derive(Debug, Deserialize)]
struct RepoCreditReceipts {
    closure_review: String,
    soul_verdict: String,
    mind_commit: String,
    public_proof: String,
    maintainer_review: String,
    accepted_artifact: String,
    credit_ledger: String,
    credit_readback: String,
}
#[derive(Debug, Deserialize)]
struct RepoCreditPacket {
    requires_author_identity: bool,
    requires_reviewer_identity: bool,
    requires_accepted_artifact_ref: bool,
    requires_public_proof_ref: bool,
    requires_changed_path_list: bool,
    requires_credit_ledger_target: bool,
    requires_private_state_redaction_check: bool,
}
#[derive(Debug, Deserialize)]
struct RepoCreditAuthority {
    branch_local_only: bool,
    credit_ledger_authorized: bool,
    bifrost_publication_authorized: bool,
    github_pr_authorized: bool,
    merge_authorized: bool,
    publication_authorized: bool,
    upstream_sync_authorized: bool,
    hands_action_authorized: bool,
    service_lifecycle_authority: bool,
    cross_body_mutation_authorized: bool,
    private_verse_rummaging: bool,
    bifrost_credit_authority_required: bool,
}

pub(super) fn parse_repo_credit_request(text: &str) -> Result<RepoCreditRequest> {
    toml::from_str(text).context("credit request is not valid typed TOML")
}

#[cfg(test)]
mod tool_request_tests {
    use super::*;

    #[test]
    fn comment_cannot_counterfeit_direct_execution_seal() {
        let text = r#"
schema_version = "epiphany.repo_tool_request.v0"
safe_action_family = "repo.tool_request"
summary = "summary"
private_state_exposed = false
[request]
target_directory = "gamecult-local/daemon-tool-directory"
target_capability = "daemon-tool-capability:selected-by-review"
operation = "submitTypedToolIntent"
[cultmesh]
intent_contract = "epiphany.cultmesh.daemon_tool_invocation_intent.v0"
receipt_contract = "epiphany.cultmesh.daemon_tool_invocation_receipt.v0"
host_daemon_owns_execution = true
requester_owns_request = false
requires_host_liveness_ready = true
requires_cultmesh_receipts = true
[odin]
discoverable = true
preserves_provider_ownership = true
private_verse_passthrough = false
[authority]
# direct_tool_execution = false
direct_tool_execution = true
arbitrary_shell_authority = false
hands_action_authority = false
state_commit_authority = false
publication_authority = false
service_lifecycle_authority = false
cross_body_mutation_authority = false
private_verse_rummaging = false
"#;
        let request = parse_repo_tool_request(text).expect("fixture is typed TOML");
        assert!(request.authority.direct_tool_execution);
        assert!(!request.has_authority_seals());
    }

    #[test]
    fn metrics_comment_cannot_counterfeit_spend_seal() {
        let text = r#"
schema_version = "epiphany.repo_metrics_request.v0"
safe_action_family = "repo.metrics_request"
summary = "summary"
private_state_exposed = false
[request]
status = "awaiting-metrics-review"
requested_owner = "Bifrost/Maintainer"
requested_effect = "record-compute-review-and-artifact-accounting"
publication_request_ref = "publication"
credit_request_ref = "credit"
artifact_acceptance_request_ref = "artifact"
[antecedents]
closure_review_required = true
soul_verdict_required = true
mind_commit_required = true
public_proof_required = true
accepted_artifact_required = true
credit_request_required = true
[required_receipts]
closure_review = "epiphany.repo_work_closure_review.v0"
soul_verdict = "epiphany.soul.verification_verdict"
mind_commit = "epiphany.mind.state_commit_receipt"
public_proof = "epiphany.repo_work_public_proof_bundle.v0"
accepted_artifact = "gamecult.artifact.acceptance_receipt.v0"
model_spend = "gamecult.metrics.model_spend_receipt.v0"
review_load = "gamecult.metrics.review_load_receipt.v0"
credit_readback = "gamecult.bifrost.credit_readback_receipt.v0"
[metrics_packet]
requires_model_call_count = true
requires_token_or_cost_summary = true
requires_review_minutes_or_count = true
requires_accepted_artifact_ref = true
requires_public_proof_ref = true
requires_credit_readback_ref = true
requires_private_state_redaction_check = true
[authority]
branch_local_only = true
metrics_ledger_authorized = false
# spend_authorized = false
spend_authorized = true
review_load_authorized = false
credit_ledger_authorized = false
github_pr_authorized = false
merge_authorized = false
publication_authorized = false
upstream_sync_authorized = false
hands_action_authorized = false
service_lifecycle_authority = false
cross_body_mutation_authorized = false
private_verse_rummaging = false
bifrost_or_maintainer_metrics_authority_required = true
"#;
        let request = parse_repo_metrics_request(text).expect("fixture is typed TOML");
        assert!(!request.has_authority_seals());
    }

    #[test]
    fn artifact_comment_cannot_counterfeit_acceptance_seal() {
        let text = r#"
schema_version = "epiphany.repo_artifact_acceptance_request.v0"
safe_action_family = "repo.artifact_acceptance_request"
summary = "summary"
private_state_exposed = false
[request]
status = "awaiting-artifact-acceptance-review"
requested_owner = "Maintainer/Bifrost"
requested_effect = "record-accepted-artifact-for-reviewed-branch-work"
verification_request_ref = "verification"
maintainer_review_request_ref = "review"
publication_request_ref = "publication"
[antecedents]
closure_review_required = true
soul_verdict_required = true
mind_commit_required = true
public_proof_required = true
maintainer_review_required = true
hands_commit_required = true
[required_receipts]
closure_review = "epiphany.repo_work_closure_review.v0"
soul_verdict = "epiphany.soul.verification_verdict"
mind_commit = "epiphany.mind.state_commit_receipt"
public_proof = "epiphany.repo_work_public_proof_bundle.v0"
maintainer_review = "gamecult.maintainer.review_receipt.v0"
hands_commit = "epiphany.hands.commit_receipt"
accepted_artifact = "gamecult.artifact.acceptance_receipt.v0"
[artifact_packet]
requires_artifact_ref = true
requires_commit_sha = true
requires_changed_path_list = true
requires_review_verdict = true
requires_public_proof_ref = true
requires_acceptance_rationale = true
requires_private_state_redaction_check = true
[authority]
branch_local_only = true
# artifact_acceptance_authorized = false
artifact_acceptance_authorized = true
credit_ledger_authorized = false
github_pr_authorized = false
merge_authorized = false
publication_authorized = false
upstream_sync_authorized = false
hands_action_authorized = false
service_lifecycle_authority = false
cross_body_mutation_authorized = false
private_verse_rummaging = false
maintainer_or_bifrost_acceptance_authority_required = true
"#;
        let request = parse_repo_artifact_acceptance_request(text).expect("fixture is typed TOML");
        assert!(!request.has_authority_seals());
    }

    #[test]
    fn credit_comment_cannot_counterfeit_ledger_seal() {
        let text = r#"
schema_version = "epiphany.repo_credit_request.v0"
safe_action_family = "repo.credit_request"
summary = "summary"
private_state_exposed = false
[request]
status = "awaiting-bifrost-credit-review"
requested_owner = "Bifrost"
requested_effect = "record-credit-for-redacted-proof-and-accepted-artifact"
publication_request_ref = "publication"
maintainer_review_request_ref = "review"
pr_request_ref = "pr"
[antecedents]
closure_review_required = true
soul_verdict_required = true
mind_commit_required = true
public_proof_required = true
maintainer_review_required = true
accepted_artifact_required = true
authorship_context_required = true
[required_receipts]
closure_review = "epiphany.repo_work_closure_review.v0"
soul_verdict = "epiphany.soul.verification_verdict"
mind_commit = "epiphany.mind.state_commit_receipt"
public_proof = "epiphany.repo_work_public_proof_bundle.v0"
maintainer_review = "gamecult.maintainer.review_receipt.v0"
accepted_artifact = "gamecult.artifact.acceptance_receipt.v0"
credit_ledger = "gamecult.bifrost.credit_receipt.v0"
credit_readback = "gamecult.bifrost.credit_readback_receipt.v0"
[credit_packet]
requires_author_identity = true
requires_reviewer_identity = true
requires_accepted_artifact_ref = true
requires_public_proof_ref = true
requires_changed_path_list = true
requires_credit_ledger_target = true
requires_private_state_redaction_check = true
[authority]
branch_local_only = true
# credit_ledger_authorized = false
credit_ledger_authorized = true
bifrost_publication_authorized = false
github_pr_authorized = false
merge_authorized = false
publication_authorized = false
upstream_sync_authorized = false
hands_action_authorized = false
service_lifecycle_authority = false
cross_body_mutation_authorized = false
private_verse_rummaging = false
bifrost_credit_authority_required = true
"#;
        let request = parse_repo_credit_request(text).expect("fixture is typed TOML");
        assert!(!request.has_authority_seals());
    }
}

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
    pub(super) requested_owner: String,
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
    pub(super) maintainer_or_soul_security_authority_required: bool,
}

impl RepoSecretPolicyRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_secret_policy_request.v0"
            && self.safe_action_family == "repo.secret_policy_request"
    }

    pub(super) fn awaits_security_review(&self) -> bool {
        let request = &self.request;
        request.status == "awaiting-security-review"
            && request.requested_owner == "Maintainer/Soul/Bifrost"
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
            && authority.maintainer_or_soul_security_authority_required
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
    pub(super) requested_owner: String,
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
    pub(super) maintainer_or_soul_dependency_authority_required: bool,
}

impl RepoDependencyPolicyRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_dependency_policy_request.v0"
            && self.safe_action_family == "repo.dependency_policy_request"
    }

    pub(super) fn awaits_review(&self) -> bool {
        let request = &self.request;
        request.status == "awaiting-dependency-policy-review"
            && request.requested_owner == "Maintainer/Soul/Bifrost"
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
            && value.maintainer_or_soul_dependency_authority_required
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
    pub(super) requested_owner: String,
    pub(super) requested_effect: String,
    pub(super) deployment_trigger: String,
    pub(super) deployment_owner: String,
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
            && value.requested_owner == "Idunn/Maintainer"
            && value.requested_effect == "review-repo-deployment-trigger-and-script"
            && value.deployment_trigger == "git-push-observed-by-idunn"
            && value.deployment_owner == "Idunn"
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
