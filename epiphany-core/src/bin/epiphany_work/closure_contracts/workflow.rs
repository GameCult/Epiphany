#[derive(Debug, Deserialize)]
pub(super) struct RepoWorkOrder {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    work_order: RepoWorkOrderBody,
    antecedents: RepoWorkOrderAntecedents,
    required_receipts: RepoWorkOrderReceipts,
    scope: RepoWorkOrderScope,
    authority: RepoWorkOrderAuthority,
}

impl RepoWorkOrder {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_work_order.v0"
            && self.safe_action_family == "repo.work_order"
    }

    pub(super) fn awaits_hands_review(&self) -> bool {
        self.work_order.status == "awaiting-hands-review"
            && self.work_order.requested_owner == "Hands"
            && self.work_order.requested_effect == "branch-local-implementation"
    }

    pub(super) fn has_antecedent_contract(&self) -> bool {
        !self.antecedents.objective_draft_ref.is_empty()
            && !self.antecedents.adoption_request_ref.is_empty()
            && !self.antecedents.scheduling_request_ref.is_empty()
            && self.antecedents.mind_adoption_required
            && self.antecedents.self_queue_selection_required
    }

    pub(super) fn has_receipt_contract(&self) -> bool {
        let r = &self.required_receipts;
        r.substrate_gate == "epiphany.substrate_gate.grant"
            && r.hands_intent == "epiphany.hands.action_intent"
            && r.hands_review == "epiphany.hands.action_review"
            && r.hands_patch == "epiphany.hands.patch_receipt"
            && r.hands_command == "epiphany.hands.command_receipt"
            && r.hands_commit == "epiphany.hands.commit_receipt"
            && r.soul_verdict == "epiphany.soul.verification_verdict"
            && r.mind_commit == "epiphany.mind.state_commit_receipt"
    }

    pub(super) fn has_bounded_scope(&self) -> bool {
        self.scope.branch_required
            && self.scope.allowed_branch_prefix == "epiphany/"
            && self.scope.max_changed_paths == 3
            && self.scope.requires_reviewable_diff
    }

    pub(super) fn has_authority_seals(&self) -> bool {
        let a = &self.authority;
        a.branch_local_only
            && !a.substrate_access_authorized
            && !a.hands_action_authorized
            && !a.shell_command_authorized
            && !a.commit_authorized
            && !a.publication_authorized
            && !a.cross_body_mutation_authorized
            && a.bifrost_publication_required
    }
}

#[derive(Debug, Deserialize)]
struct RepoWorkOrderBody {
    status: String,
    requested_owner: String,
    requested_effect: String,
}
#[derive(Debug, Deserialize)]
struct RepoWorkOrderAntecedents {
    objective_draft_ref: String,
    adoption_request_ref: String,
    scheduling_request_ref: String,
    mind_adoption_required: bool,
    self_queue_selection_required: bool,
}
#[derive(Debug, Deserialize)]
struct RepoWorkOrderReceipts {
    substrate_gate: String,
    hands_intent: String,
    hands_review: String,
    hands_patch: String,
    hands_command: String,
    hands_commit: String,
    soul_verdict: String,
    mind_commit: String,
}
#[derive(Debug, Deserialize)]
struct RepoWorkOrderScope {
    branch_required: bool,
    allowed_branch_prefix: String,
    max_changed_paths: u64,
    requires_reviewable_diff: bool,
}
#[derive(Debug, Deserialize)]
struct RepoWorkOrderAuthority {
    branch_local_only: bool,
    substrate_access_authorized: bool,
    hands_action_authorized: bool,
    shell_command_authorized: bool,
    commit_authorized: bool,
    publication_authorized: bool,
    cross_body_mutation_authorized: bool,
    bifrost_publication_required: bool,
}

pub(super) fn parse_repo_work_order(text: &str) -> Result<RepoWorkOrder> {
    toml::from_str(text).context("work order is not valid typed TOML")
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoSchedulingRequest {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    request: RepoSchedulingBody,
    queue: RepoSchedulingQueue,
    required_receipts: RepoSchedulingReceipts,
    authority: RepoSchedulingAuthority,
}

impl RepoSchedulingRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_scheduling_request.v0"
            && self.safe_action_family == "repo.scheduling_request"
    }

    pub(super) fn awaits_mind_adoption(&self) -> bool {
        let r = &self.request;
        r.status == "awaiting-mind-adoption"
            && r.requested_scheduler == "Self"
            && r.mind_adoption_receipt_required
            && r.self_may_schedule_after_mind_only
            && r.queue_run_allowed_after_adoption
    }

    pub(super) fn has_bounded_queue_contract(&self) -> bool {
        let q = &self.queue;
        q.target_gate == "repo-work-queue"
            && q.preferred_next_safe_family == "repo.task_card"
            && q.max_items_per_pulse == 1
            && q.requires_epiphany_branch
            && q.publish_blocker == "bifrost-publication-missing"
    }

    pub(super) fn has_receipt_contract(&self) -> bool {
        let r = &self.required_receipts;
        r.mind_review == "epiphany.mind.gateway_review"
            && r.mind_commit == "epiphany.mind.state_commit_receipt"
            && r.expected_self_receipt == "epiphany.repo_work_queue_selection.v0"
    }

    pub(super) fn has_authority_seals(&self) -> bool {
        let a = &self.authority;
        a.branch_local_only
            && !a.self_scheduling_authorized
            && !a.queue_mutation_authorized
            && !a.hands_action_authorized
            && !a.publication_authorized
            && !a.cross_body_mutation_authorized
            && a.mind_adoption_required
            && a.bifrost_publication_required
    }
}

#[derive(Debug, Deserialize)]
struct RepoSchedulingBody {
    status: String,
    requested_scheduler: String,
    mind_adoption_receipt_required: bool,
    self_may_schedule_after_mind_only: bool,
    queue_run_allowed_after_adoption: bool,
}
#[derive(Debug, Deserialize)]
struct RepoSchedulingQueue {
    target_gate: String,
    preferred_next_safe_family: String,
    max_items_per_pulse: u64,
    requires_epiphany_branch: bool,
    publish_blocker: String,
}
#[derive(Debug, Deserialize)]
struct RepoSchedulingReceipts {
    mind_review: String,
    mind_commit: String,
    expected_self_receipt: String,
}
#[derive(Debug, Deserialize)]
struct RepoSchedulingAuthority {
    branch_local_only: bool,
    self_scheduling_authorized: bool,
    queue_mutation_authorized: bool,
    hands_action_authorized: bool,
    publication_authorized: bool,
    cross_body_mutation_authorized: bool,
    mind_adoption_required: bool,
    bifrost_publication_required: bool,
}

pub(super) fn parse_repo_scheduling_request(text: &str) -> Result<RepoSchedulingRequest> {
    toml::from_str(text).context("scheduling request is not valid typed TOML")
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoAdoptionRequest {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    request: RepoAdoptionBody,
    decision_contract: RepoAdoptionDecisionContract,
    inputs: RepoAdoptionInputs,
    authority: RepoAdoptionAuthority,
}

impl RepoAdoptionRequest {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_adoption_request.v0"
            && self.safe_action_family == "repo.adoption_request"
    }

    pub(super) fn awaits_mind_review(&self) -> bool {
        let r = &self.request;
        r.status == "awaiting-mind-review"
            && r.requested_decision == "adopt-or-refuse-objective-draft"
            && r.mind_review_required
            && r.mind_state_commit_required
            && r.self_scheduling_after_mind_only
    }

    pub(super) fn has_decision_contract(&self) -> bool {
        let d = &self.decision_contract;
        d.allowed_verdicts == ["adopted", "refused", "needs-more-consensus"]
            && d.requires_review_finding
            && d.requires_receipt == "epiphany.mind.gateway_review"
            && d.requires_commit_receipt_if_adopted == "epiphany.mind.state_commit_receipt"
            && d.does_not_modify_state
    }

    pub(super) fn has_input_contract(&self) -> bool {
        self.inputs.objective_draft_required
            && self.inputs.consensus_brief_required
            && self
                .inputs
                .public_discussion_refs
                .iter()
                .all(|value| !value.trim().is_empty())
            && self
                .inputs
                .candidate_action_refs
                .iter()
                .all(|value| !value.trim().is_empty())
    }

    pub(super) fn has_authority_seals(&self) -> bool {
        let a = &self.authority;
        a.branch_local_only
            && !a.objective_adoption_authorized
            && !a.state_commit_authorized
            && !a.self_scheduling_authorized
            && !a.hands_action_authorized
            && !a.publication_authorized
            && !a.cross_body_mutation_authorized
    }
}

#[derive(Debug, Deserialize)]
struct RepoAdoptionBody {
    status: String,
    requested_decision: String,
    mind_review_required: bool,
    mind_state_commit_required: bool,
    self_scheduling_after_mind_only: bool,
}
#[derive(Debug, Deserialize)]
struct RepoAdoptionDecisionContract {
    allowed_verdicts: Vec<String>,
    requires_review_finding: bool,
    requires_receipt: String,
    requires_commit_receipt_if_adopted: String,
    does_not_modify_state: bool,
}
#[derive(Debug, Deserialize)]
struct RepoAdoptionInputs {
    public_discussion_refs: Vec<String>,
    candidate_action_refs: Vec<String>,
    objective_draft_required: bool,
    consensus_brief_required: bool,
}
#[derive(Debug, Deserialize)]
struct RepoAdoptionAuthority {
    branch_local_only: bool,
    objective_adoption_authorized: bool,
    state_commit_authorized: bool,
    self_scheduling_authorized: bool,
    hands_action_authorized: bool,
    publication_authorized: bool,
    cross_body_mutation_authorized: bool,
}

pub(super) fn parse_repo_adoption_request(text: &str) -> Result<RepoAdoptionRequest> {
    toml::from_str(text).context("adoption request is not valid typed TOML")
}

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
