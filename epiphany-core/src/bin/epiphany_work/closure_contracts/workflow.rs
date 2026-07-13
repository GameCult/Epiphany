#[derive(Debug, Deserialize)]
pub(super) struct RepoVerificationRequest {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    request: RepoVerificationBody,
    antecedents: RepoVerificationAntecedents,
    required_receipts: RepoVerificationReceipts,
    checks: RepoVerificationChecks,
    authority: RepoVerificationAuthority,
}
impl RepoVerificationRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_verification_request.v0"
            && self.safe_action_family == "repo.verification_request"
    }
    pub(super) fn awaits_soul_review(&self) -> bool {
        let r = &self.request;
        r.status == "awaiting-soul-review"
            && r.requested_owner == "Soul"
            && r.requested_effect == "verify-branch-local-hands-work"
            && !r.work_order_ref.is_empty()
    }
    pub(super) fn has_antecedent_contract(&self) -> bool {
        let a = &self.antecedents;
        a.substrate_gate_required
            && a.hands_intent_required
            && a.hands_review_required
            && a.hands_patch_required
            && a.hands_command_required
            && a.hands_commit_required
            && a.work_order_required
    }
    pub(super) fn has_receipt_contract(&self) -> bool {
        let r = &self.required_receipts;
        r.hands_patch == "epiphany.hands.patch_receipt"
            && r.hands_command == "epiphany.hands.command_receipt"
            && r.hands_commit == "epiphany.hands.commit_receipt"
            && r.soul_verdict == "epiphany.soul.verification_verdict"
            && r.closure_review == "epiphany.repo_work_closure_review.v0"
            && r.mind_review == "epiphany.mind.gateway_review"
            && r.mind_commit == "epiphany.mind.state_commit_receipt"
    }
    pub(super) fn has_check_contract(&self) -> bool {
        self.checks.required
            == [
                "declared-paths-match-commit",
                "hands-receipts-present",
                "visible-diff-supports-summary",
                "no-private-state-exposure",
                "publication-remains-gated",
            ]
            && self.checks.model_verdict_allowed
            && self.checks.failure_blocks_mind_admission
    }
    pub(super) fn has_authority_seals(&self) -> bool {
        let a = &self.authority;
        a.branch_local_only
            && !a.soul_verdict_authorized
            && !a.state_commit_authorized
            && !a.hands_action_authorized
            && !a.rerun_authorized
            && !a.publication_authorized
            && !a.merge_authorized
            && !a.service_lifecycle_authority
            && !a.cross_body_mutation_authorized
            && !a.private_verse_rummaging
            && a.bifrost_publication_required
    }
}
#[derive(Debug, Deserialize)]
struct RepoVerificationBody {
    status: String,
    requested_owner: String,
    requested_effect: String,
    work_order_ref: String,
}
#[derive(Debug, Deserialize)]
struct RepoVerificationAntecedents {
    substrate_gate_required: bool,
    hands_intent_required: bool,
    hands_review_required: bool,
    hands_patch_required: bool,
    hands_command_required: bool,
    hands_commit_required: bool,
    work_order_required: bool,
}
#[derive(Debug, Deserialize)]
struct RepoVerificationReceipts {
    hands_patch: String,
    hands_command: String,
    hands_commit: String,
    soul_verdict: String,
    closure_review: String,
    mind_review: String,
    mind_commit: String,
}
#[derive(Debug, Deserialize)]
struct RepoVerificationChecks {
    required: Vec<String>,
    model_verdict_allowed: bool,
    failure_blocks_mind_admission: bool,
}
#[derive(Debug, Deserialize)]
struct RepoVerificationAuthority {
    branch_local_only: bool,
    soul_verdict_authorized: bool,
    state_commit_authorized: bool,
    hands_action_authorized: bool,
    rerun_authorized: bool,
    publication_authorized: bool,
    merge_authorized: bool,
    service_lifecycle_authority: bool,
    cross_body_mutation_authorized: bool,
    private_verse_rummaging: bool,
    bifrost_publication_required: bool,
}
pub(super) fn parse_repo_verification_request(text: &str) -> Result<RepoVerificationRequest> {
    toml::from_str(text).context("verification request is not valid typed TOML")
}

include!("workflow_tests.rs");
