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

#[derive(Debug, Deserialize)]
pub(super) struct RepoPublicationRequest {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    request: RepoPublicationRequestBody,
    antecedents: RepoPublicationAntecedents,
    required_receipts: RepoPublicationReceipts,
    public_export: RepoPublicationExport,
    authority: RepoPublicationAuthority,
}

impl RepoPublicationRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_publication_request.v0"
            && self.safe_action_family == "repo.publication_request"
    }
    pub(super) fn awaits_bifrost_review(&self) -> bool {
        self.request.status == "awaiting-bifrost-review"
            && self.request.requested_owner == "Bifrost"
            && self.request.requested_effect == "publish-redacted-proof-and-route-maintainer-review"
            && !self.request.verification_request_ref.is_empty()
    }
    pub(super) fn has_antecedent_contract(&self) -> bool {
        let a = &self.antecedents;
        a.closure_review_required
            && a.soul_verdict_required
            && a.mind_commit_required
            && a.public_proof_export_required
            && a.private_state_redaction_required
    }
    pub(super) fn has_receipt_contract(&self) -> bool {
        let r = &self.required_receipts;
        r.closure_review == "epiphany.repo_work_closure_review.v0"
            && r.soul_verdict == "epiphany.soul.verification_verdict"
            && r.mind_commit == "epiphany.mind.state_commit_receipt"
            && r.public_proof == "epiphany.repo_work_public_proof_bundle.v0"
            && r.bifrost_publication == "gamecult.bifrost.public_proof_publication_receipt.v0"
            && r.github_publication == "gamecult.github.publication_receipt.v0"
            && r.credit_ledger == "gamecult.bifrost.credit_receipt.v0"
            && r.upstream_sync == "epiphany.repo_work_upstream_main_sync.v0"
    }
    pub(super) fn has_redaction_contract(&self) -> bool {
        let e = &self.public_export;
        e.redacted_only
            && !e.raw_receipts_allowed
            && !e.private_paths_allowed
            && !e.worker_thought_allowed
            && !e.operator_context_allowed
            && e.credit_required
    }
    pub(super) fn has_authority_seals(&self) -> bool {
        let a = &self.authority;
        a.branch_local_only
            && !a.bifrost_publication_authorized
            && !a.github_publication_authorized
            && !a.credit_ledger_authorized
            && !a.merge_authorized
            && !a.upstream_sync_authorized
            && !a.hands_action_authorized
            && !a.service_lifecycle_authority
            && !a.cross_body_mutation_authorized
            && !a.private_verse_rummaging
            && a.maintainer_review_required
    }
}

#[derive(Debug, Deserialize)]
struct RepoPublicationRequestBody {
    status: String,
    requested_owner: String,
    requested_effect: String,
    verification_request_ref: String,
}
#[derive(Debug, Deserialize)]
struct RepoPublicationAntecedents {
    closure_review_required: bool,
    soul_verdict_required: bool,
    mind_commit_required: bool,
    public_proof_export_required: bool,
    private_state_redaction_required: bool,
}
#[derive(Debug, Deserialize)]
struct RepoPublicationReceipts {
    closure_review: String,
    soul_verdict: String,
    mind_commit: String,
    public_proof: String,
    bifrost_publication: String,
    github_publication: String,
    credit_ledger: String,
    upstream_sync: String,
}
#[derive(Debug, Deserialize)]
struct RepoPublicationExport {
    redacted_only: bool,
    raw_receipts_allowed: bool,
    private_paths_allowed: bool,
    worker_thought_allowed: bool,
    operator_context_allowed: bool,
    credit_required: bool,
}
#[derive(Debug, Deserialize)]
struct RepoPublicationAuthority {
    branch_local_only: bool,
    bifrost_publication_authorized: bool,
    github_publication_authorized: bool,
    credit_ledger_authorized: bool,
    merge_authorized: bool,
    upstream_sync_authorized: bool,
    hands_action_authorized: bool,
    service_lifecycle_authority: bool,
    cross_body_mutation_authorized: bool,
    private_verse_rummaging: bool,
    maintainer_review_required: bool,
}

pub(super) fn parse_repo_publication_request(text: &str) -> Result<RepoPublicationRequest> {
    toml::from_str(text).context("publication request is not valid typed TOML")
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoSyncRequest {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    request: RepoSyncRequestBody,
    antecedents: RepoSyncAntecedents,
    required_receipts: RepoSyncReceipts,
    sync_proof: RepoSyncProof,
    authority: RepoSyncAuthority,
}
impl RepoSyncRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_sync_request.v0"
            && self.safe_action_family == "repo.sync_request"
    }
    pub(super) fn awaits_upstream_proof(&self) -> bool {
        self.request.status == "awaiting-upstream-main-proof"
            && self.request.requested_owner == "Bifrost"
            && self.request.requested_effect == "prove-published-commit-contained-by-upstream-main"
            && !self.request.publication_request_ref.is_empty()
    }
    pub(super) fn has_antecedent_contract(&self) -> bool {
        let a = &self.antecedents;
        a.publication_receipt_required
            && a.github_publication_required
            && a.maintainer_review_required
            && a.credit_ledger_required
            && a.public_proof_required
    }
    pub(super) fn has_receipt_contract(&self) -> bool {
        let r = &self.required_receipts;
        r.bifrost_publication == "gamecult.bifrost.public_proof_publication_receipt.v0"
            && r.github_publication == "gamecult.github.publication_receipt.v0"
            && r.maintainer_review == "gamecult.maintainer.review_receipt.v0"
            && r.credit_ledger == "gamecult.bifrost.credit_receipt.v0"
            && r.upstream_sync == "epiphany.repo_work_upstream_main_sync.v0"
            && r.ancestry_proof == "git.merge_base_is_ancestor"
    }
    pub(super) fn has_proof_contract(&self) -> bool {
        let p = &self.sync_proof;
        p.target_ref == "origin/main"
            && p.requires_fetch_before_check
            && p.requires_merge_base_ancestor_check
            && p.requires_clean_public_proof_readback
            && p.records_upstream_main_sha
    }
    pub(super) fn has_authority_seals(&self) -> bool {
        let a = &self.authority;
        a.branch_local_only
            && !a.merge_authorized
            && !a.push_authorized
            && !a.upstream_sync_authorized
            && !a.github_publication_authorized
            && !a.credit_ledger_authorized
            && !a.hands_action_authorized
            && !a.service_lifecycle_authority
            && !a.cross_body_mutation_authorized
            && !a.private_verse_rummaging
            && a.operator_or_maintainer_authority_required
    }
}
#[derive(Debug, Deserialize)]
struct RepoSyncRequestBody {
    status: String,
    requested_owner: String,
    requested_effect: String,
    publication_request_ref: String,
}
#[derive(Debug, Deserialize)]
struct RepoSyncAntecedents {
    publication_receipt_required: bool,
    github_publication_required: bool,
    maintainer_review_required: bool,
    credit_ledger_required: bool,
    public_proof_required: bool,
}
#[derive(Debug, Deserialize)]
struct RepoSyncReceipts {
    bifrost_publication: String,
    github_publication: String,
    maintainer_review: String,
    credit_ledger: String,
    upstream_sync: String,
    ancestry_proof: String,
}
#[derive(Debug, Deserialize)]
struct RepoSyncProof {
    target_ref: String,
    requires_fetch_before_check: bool,
    requires_merge_base_ancestor_check: bool,
    requires_clean_public_proof_readback: bool,
    records_upstream_main_sha: bool,
}
#[derive(Debug, Deserialize)]
struct RepoSyncAuthority {
    branch_local_only: bool,
    merge_authorized: bool,
    push_authorized: bool,
    upstream_sync_authorized: bool,
    github_publication_authorized: bool,
    credit_ledger_authorized: bool,
    hands_action_authorized: bool,
    service_lifecycle_authority: bool,
    cross_body_mutation_authorized: bool,
    private_verse_rummaging: bool,
    operator_or_maintainer_authority_required: bool,
}
pub(super) fn parse_repo_sync_request(text: &str) -> Result<RepoSyncRequest> {
    toml::from_str(text).context("sync request is not valid typed TOML")
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoPrRequest {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    request: RepoPrRequestBody,
    antecedents: RepoPrAntecedents,
    required_receipts: RepoPrReceipts,
    pr_packet: RepoPrPacket,
    authority: RepoPrAuthority,
}
impl RepoPrRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_pr_request.v0"
            && self.safe_action_family == "repo.pr_request"
    }
    pub(super) fn awaits_owned_review(&self) -> bool {
        let r = &self.request;
        r.status == "awaiting-pr-publication-review"
            && r.requested_owner == "Bifrost/GitHub"
            && r.requested_effect
                == "open-or-update-review-pr-from-redacted-proof-and-maintainer-context"
            && !r.maintainer_review_request_ref.is_empty()
            && !r.publication_request_ref.is_empty()
            && !r.sync_request_ref.is_empty()
    }
    pub(super) fn has_antecedent_contract(&self) -> bool {
        let a = &self.antecedents;
        a.closure_review_required
            && a.soul_verdict_required
            && a.mind_commit_required
            && a.public_proof_required
            && a.maintainer_review_required
            && a.bifrost_publication_required
            && a.credit_ledger_required
    }
    pub(super) fn has_receipt_contract(&self) -> bool {
        let r = &self.required_receipts;
        r.closure_review == "epiphany.repo_work_closure_review.v0"
            && r.soul_verdict == "epiphany.soul.verification_verdict"
            && r.mind_commit == "epiphany.mind.state_commit_receipt"
            && r.public_proof == "epiphany.repo_work_public_proof_bundle.v0"
            && r.maintainer_review == "gamecult.maintainer.review_receipt.v0"
            && r.bifrost_publication == "gamecult.bifrost.public_proof_publication_receipt.v0"
            && r.credit_ledger == "gamecult.bifrost.credit_receipt.v0"
            && r.pr_publication == "gamecult.github.pull_request_publication_receipt.v0"
    }
    pub(super) fn has_packet_contract(&self) -> bool {
        let p = &self.pr_packet;
        p.base_ref == "origin/main"
            && p.requires_branch_name
            && p.requires_title
            && p.requires_body
            && p.requires_changed_path_list
            && p.requires_public_proof_ref
            && p.requires_maintainer_review_ref
            && p.requires_credit_ref
            && p.requires_private_state_redaction_check
    }
    pub(super) fn has_authority_seals(&self) -> bool {
        let a = &self.authority;
        a.branch_local_only
            && !a.github_pr_authorized
            && !a.branch_push_authorized
            && !a.merge_authorized
            && !a.publication_authorized
            && !a.upstream_sync_authorized
            && !a.hands_action_authorized
            && !a.service_lifecycle_authority
            && !a.cross_body_mutation_authorized
            && !a.private_verse_rummaging
            && a.bifrost_or_maintainer_authority_required
    }
}
#[derive(Debug, Deserialize)]
struct RepoPrRequestBody {
    status: String,
    requested_owner: String,
    requested_effect: String,
    maintainer_review_request_ref: String,
    publication_request_ref: String,
    sync_request_ref: String,
}
#[derive(Debug, Deserialize)]
struct RepoPrAntecedents {
    closure_review_required: bool,
    soul_verdict_required: bool,
    mind_commit_required: bool,
    public_proof_required: bool,
    maintainer_review_required: bool,
    bifrost_publication_required: bool,
    credit_ledger_required: bool,
}
#[derive(Debug, Deserialize)]
struct RepoPrReceipts {
    closure_review: String,
    soul_verdict: String,
    mind_commit: String,
    public_proof: String,
    maintainer_review: String,
    bifrost_publication: String,
    credit_ledger: String,
    pr_publication: String,
}
#[derive(Debug, Deserialize)]
struct RepoPrPacket {
    base_ref: String,
    requires_branch_name: bool,
    requires_title: bool,
    requires_body: bool,
    requires_changed_path_list: bool,
    requires_public_proof_ref: bool,
    requires_maintainer_review_ref: bool,
    requires_credit_ref: bool,
    requires_private_state_redaction_check: bool,
}
#[derive(Debug, Deserialize)]
struct RepoPrAuthority {
    branch_local_only: bool,
    github_pr_authorized: bool,
    branch_push_authorized: bool,
    merge_authorized: bool,
    publication_authorized: bool,
    upstream_sync_authorized: bool,
    hands_action_authorized: bool,
    service_lifecycle_authority: bool,
    cross_body_mutation_authorized: bool,
    private_verse_rummaging: bool,
    bifrost_or_maintainer_authority_required: bool,
}
pub(super) fn parse_repo_pr_request(text: &str) -> Result<RepoPrRequest> {
    toml::from_str(text).context("PR request is not valid typed TOML")
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoMaintainerReviewRequest {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    request: RepoMaintainerReviewBody,
    antecedents: RepoMaintainerReviewAntecedents,
    required_receipts: RepoMaintainerReviewReceipts,
    review_packet: RepoMaintainerReviewPacket,
    authority: RepoMaintainerReviewAuthority,
}
impl RepoMaintainerReviewRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_maintainer_review_request.v0"
            && self.safe_action_family == "repo.maintainer_review_request"
    }
    pub(super) fn awaits_review(&self) -> bool {
        let r = &self.request;
        r.status == "awaiting-maintainer-review"
            && r.requested_owner == "Maintainer"
            && r.requested_effect == "review-redacted-proof-and-branch-diff"
            && !r.verification_request_ref.is_empty()
            && !r.publication_request_ref.is_empty()
    }
    pub(super) fn has_antecedent_contract(&self) -> bool {
        let a = &self.antecedents;
        a.closure_review_required
            && a.soul_verdict_required
            && a.mind_commit_required
            && a.public_proof_required
            && a.bifrost_publication_request_required
    }
    pub(super) fn has_receipt_contract(&self) -> bool {
        let r = &self.required_receipts;
        r.closure_review == "epiphany.repo_work_closure_review.v0"
            && r.soul_verdict == "epiphany.soul.verification_verdict"
            && r.mind_commit == "epiphany.mind.state_commit_receipt"
            && r.public_proof == "epiphany.repo_work_public_proof_bundle.v0"
            && r.maintainer_review == "gamecult.maintainer.review_receipt.v0"
            && r.bifrost_publication == "gamecult.bifrost.public_proof_publication_receipt.v0"
    }
    pub(super) fn has_review_packet(&self) -> bool {
        let p = &self.review_packet;
        p.requires_reviewer_identity
            && p.requires_review_verdict
            && p.allowed_verdicts
                == [
                    "approved",
                    "changes-requested",
                    "rejected",
                    "needs-human-context",
                ]
            && p.requires_changed_path_list
            && p.requires_public_proof_ref
            && p.requires_private_state_redaction_check
    }
    pub(super) fn has_authority_seals(&self) -> bool {
        let a = &self.authority;
        a.branch_local_only
            && !a.maintainer_approval_authorized
            && !a.merge_authorized
            && !a.push_authorized
            && !a.publication_authorized
            && !a.upstream_sync_authorized
            && !a.hands_action_authorized
            && !a.service_lifecycle_authority
            && !a.cross_body_mutation_authorized
            && !a.private_verse_rummaging
            && a.human_or_maintainer_response_required
    }
}
#[derive(Debug, Deserialize)]
struct RepoMaintainerReviewBody {
    status: String,
    requested_owner: String,
    requested_effect: String,
    verification_request_ref: String,
    publication_request_ref: String,
}
#[derive(Debug, Deserialize)]
struct RepoMaintainerReviewAntecedents {
    closure_review_required: bool,
    soul_verdict_required: bool,
    mind_commit_required: bool,
    public_proof_required: bool,
    bifrost_publication_request_required: bool,
}
#[derive(Debug, Deserialize)]
struct RepoMaintainerReviewReceipts {
    closure_review: String,
    soul_verdict: String,
    mind_commit: String,
    public_proof: String,
    maintainer_review: String,
    bifrost_publication: String,
}
#[derive(Debug, Deserialize)]
struct RepoMaintainerReviewPacket {
    requires_reviewer_identity: bool,
    requires_review_verdict: bool,
    allowed_verdicts: Vec<String>,
    requires_changed_path_list: bool,
    requires_public_proof_ref: bool,
    requires_private_state_redaction_check: bool,
}
#[derive(Debug, Deserialize)]
struct RepoMaintainerReviewAuthority {
    branch_local_only: bool,
    maintainer_approval_authorized: bool,
    merge_authorized: bool,
    push_authorized: bool,
    publication_authorized: bool,
    upstream_sync_authorized: bool,
    hands_action_authorized: bool,
    service_lifecycle_authority: bool,
    cross_body_mutation_authorized: bool,
    private_verse_rummaging: bool,
    human_or_maintainer_response_required: bool,
}
pub(super) fn parse_repo_maintainer_review_request(
    text: &str,
) -> Result<RepoMaintainerReviewRequest> {
    toml::from_str(text).context("maintainer review request is not valid typed TOML")
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoReadinessReviewRequest {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    request: RepoReadinessReviewBody,
    antecedents: std::collections::BTreeMap<String, bool>,
    required_receipts: std::collections::BTreeMap<String, String>,
    readiness_packet: RepoReadinessPacket,
    authority: std::collections::BTreeMap<String, bool>,
}

impl RepoReadinessReviewRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_readiness_review_request.v0"
            && self.safe_action_family == "repo.readiness_review_request"
    }

    pub(super) fn has_coherent_routing(&self) -> bool {
        let r = &self.request;
        r.status == "awaiting-mvp-readiness-review"
            && r.routing_owner == "Self"
            && r.required_reviewers == ["Maintainer", "Soul", "Mind", "Bifrost"]
            && r.readiness_approval_owner == "none"
            && r.requested_effect == "review-redacted-repo-swarm-mvp-proof-bundle"
            && r.review_is_advisory_until_maintainer_or_bifrost_acceptance
    }

    pub(super) fn has_antecedent_contract(&self) -> bool {
        [
            "repo_init_required",
            "swarm_online_required",
            "persona_intake_required",
            "imagination_plan_required",
            "self_queue_run_required",
            "hands_commit_required",
            "soul_closure_required",
            "modeling_map_update_required",
            "mind_admission_required",
            "public_proof_required",
            "bifrost_publication_required",
            "upstream_main_sync_required",
            "idunn_lifecycle_readiness_required",
            "tool_directory_readiness_required",
            "private_state_redaction_required",
        ]
        .iter()
        .all(|key| self.antecedents.get(*key) == Some(&true))
    }

    pub(super) fn has_receipt_contract(&self) -> bool {
        [
            ("repo_init", "epiphany.repo_swarm_init_receipt.v0"),
            ("swarm_online", "epiphany.repo_swarm_online_receipt.v0"),
            ("persona_speech_audit", "epiphany.persona_speech_audit.v0"),
            (
                "imagination_action_items",
                "epiphany.repo_work_imagination_action_items_receipt.v0",
            ),
            ("queue_run", "epiphany.repo_work_queue_run_receipt.v0"),
            ("hands_commit", "epiphany.hands.commit_receipt"),
            ("closure_review", "epiphany.repo_work_closure_review.v0"),
            ("soul_verdict", "epiphany.soul.verification_verdict"),
            ("modeling_map", "epiphany.repo_work_map_entry.v0"),
            ("mind_commit", "epiphany.mind.state_commit_receipt"),
            ("public_proof", "epiphany.repo_work_public_proof_bundle.v0"),
            (
                "bifrost_publication",
                "gamecult.bifrost.public_proof_publication_receipt.v0",
            ),
            (
                "upstream_sync",
                "epiphany.repo_work_upstream_sync_receipt.v0",
            ),
            ("idunn_lifecycle", "epiphany.repo_work_service_audit.v0"),
            (
                "tool_directory",
                "epiphany.cultmesh.daemon_tool_directory.v0",
            ),
        ]
        .iter()
        .all(|(key, value)| self.required_receipts.get(*key).is_some_and(|v| v == value))
    }

    pub(super) fn has_packet_contract(&self) -> bool {
        let p = &self.readiness_packet;
        p.requires_proof_bundle_ref
            && p.requires_changed_path_list
            && p.requires_branch_name
            && p.requires_upstream_main_ref
            && p.requires_public_proof_ref
            && p.requires_bifrost_ledger_ref
            && p.requires_idunn_lifecycle_ref
            && p.requires_tool_directory_ref
            && p.requires_redaction_report
            && p.requires_reviewer_identity
            && p.allowed_verdicts
                == [
                    "ready",
                    "ready-with-caveats",
                    "not-ready",
                    "needs-human-review",
                ]
    }

    pub(super) fn has_authority_seals(&self) -> bool {
        let denied = [
            "readiness_approval_authorized",
            "durable_state_commit_authorized",
            "publication_authorized",
            "bifrost_publication_authorized",
            "github_pr_authorized",
            "merge_authorized",
            "upstream_sync_authorized",
            "deployment_authority",
            "service_lifecycle_authority",
            "hands_action_authorized",
            "cross_body_mutation_authorized",
            "private_verse_rummaging",
        ];
        denied
            .iter()
            .all(|key| self.authority.get(*key) == Some(&false))
            && self
                .authority
                .get("maintainer_soul_mind_or_bifrost_review_required")
                == Some(&true)
    }
}

#[derive(Debug, Deserialize)]
struct RepoReadinessReviewBody {
    status: String,
    routing_owner: String,
    required_reviewers: Vec<String>,
    readiness_approval_owner: String,
    requested_effect: String,
    review_is_advisory_until_maintainer_or_bifrost_acceptance: bool,
}

#[derive(Debug, Deserialize)]
struct RepoReadinessPacket {
    requires_proof_bundle_ref: bool,
    requires_changed_path_list: bool,
    requires_branch_name: bool,
    requires_upstream_main_ref: bool,
    requires_public_proof_ref: bool,
    requires_bifrost_ledger_ref: bool,
    requires_idunn_lifecycle_ref: bool,
    requires_tool_directory_ref: bool,
    requires_redaction_report: bool,
    requires_reviewer_identity: bool,
    allowed_verdicts: Vec<String>,
}

pub(super) fn parse_repo_readiness_review_request(
    text: &str,
) -> Result<RepoReadinessReviewRequest> {
    toml::from_str(text).context("readiness review request is not valid typed TOML")
}

include!("external_governance_tests.rs");
