#[derive(Debug, Deserialize)]
pub(super) struct RepoInterpreterBrief {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    interpreter: RepoInterpreterRequest,
    semantic_checks: std::collections::BTreeMap<String, bool>,
    allowed_outputs: RepoInterpreterOutputs,
    required_gates: std::collections::BTreeMap<String, bool>,
    authority: std::collections::BTreeMap<String, bool>,
}

impl RepoInterpreterBrief {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_interpreter_brief.v0"
            && self.safe_action_family == "repo.interpreter_brief"
    }
    pub(super) fn is_imagination_request_for_mind(&self) -> bool {
        let i = &self.interpreter;
        i.status == "awaiting-mind-interpretation"
            && i.authoring_owner == "Imagination"
            && i.requested_interpreter == "Mind"
            && !i.interpretation_admitted
            && i.purpose == "public-pressure-to-action-semantics"
            && i.requires_consensus_readback
            && i.requires_safe_family_choice
            && i.requires_requested_paths
            && i.requires_verification_asks
            && i.requires_evidence_needs
            && i.candidate_actions_non_authoritative
    }
    pub(super) fn has_semantic_checks(&self) -> bool {
        [
            "intent_summary_required",
            "scope_boundary_required",
            "requested_paths_required",
            "verification_required",
            "evidence_required",
            "rollback_required",
            "non_goals_required",
            "open_questions_required",
            "consensus_alignment_required",
        ]
        .iter()
        .all(|key| self.semantic_checks.get(*key) == Some(&true))
    }
    pub(super) fn has_allowed_outputs(&self) -> bool {
        let o = &self.allowed_outputs;
        o.candidate_safe_families
            == [
                "repo.consensus_brief",
                "repo.objective_draft",
                "repo.adoption_request",
                "repo.work_order",
                "repo.verification_request",
                "repo.publication_request",
            ]
            && o.may_request_replanning
            && o.may_request_more_consensus
            && !o.may_adopt_objective
            && !o.may_schedule_work
            && !o.may_touch_substrate
            && !o.may_publish
            && !o.may_deploy
    }
    pub(super) fn has_required_gates(&self) -> bool {
        [
            "imagination_consensus_required",
            "mind_review_required",
            "soul_source_grounding_required",
            "bifrost_publication_review_required",
            "hands_receipt_required_before_state_change",
            "substrate_receipt_required_before_mutation",
            "idunn_receipt_required_before_deployment",
        ]
        .iter()
        .all(|key| self.required_gates.get(*key) == Some(&true))
    }
    pub(super) fn has_authority_seals(&self) -> bool {
        [
            "direct_state_commit_authorized",
            "objective_adoption_authorized",
            "self_scheduling_authorized",
            "substrate_access_authorized",
            "hands_action_authorized",
            "shell_command_authorized",
            "commit_authorized",
            "publication_authorized",
            "deployment_execution_authority",
            "cross_body_mutation_authorized",
            "private_worker_transcripts_allowed",
            "raw_result_payloads_allowed",
            "private_state_exposed",
        ]
        .iter()
        .all(|key| self.authority.get(*key) == Some(&false))
    }
}

#[derive(Debug, Deserialize)]
struct RepoInterpreterRequest {
    status: String,
    authoring_owner: String,
    requested_interpreter: String,
    interpretation_admitted: bool,
    purpose: String,
    requires_consensus_readback: bool,
    requires_safe_family_choice: bool,
    requires_requested_paths: bool,
    requires_verification_asks: bool,
    requires_evidence_needs: bool,
    candidate_actions_non_authoritative: bool,
}
#[derive(Debug, Deserialize)]
struct RepoInterpreterOutputs {
    candidate_safe_families: Vec<String>,
    may_request_replanning: bool,
    may_request_more_consensus: bool,
    may_adopt_objective: bool,
    may_schedule_work: bool,
    may_touch_substrate: bool,
    may_publish: bool,
    may_deploy: bool,
}
pub(super) fn parse_repo_interpreter_brief(text: &str) -> Result<RepoInterpreterBrief> {
    toml::from_str(text).context("interpreter brief is not valid typed TOML")
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoObjectiveDraft {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    draft: RepoObjectiveDraftState,
    acceptance: RepoObjectiveDraftAcceptance,
    inputs: RepoObjectiveDraftInputs,
    authority: RepoObjectiveDraftAuthority,
}

impl RepoObjectiveDraft {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_objective_draft.v0"
            && self.safe_action_family == "repo.objective_draft"
    }

    pub(super) fn remains_imagination_draft(&self) -> bool {
        let d = &self.draft;
        d.status == "review-required"
            && d.owner == "Imagination"
            && d.adoption_gate == "Mind"
            && d.scheduler_gate == "Self"
            && d.publication_gate == "Bifrost"
            && !d.objective_adopted
    }

    pub(super) fn has_acceptance_contract(&self) -> bool {
        self.acceptance.criteria
            == [
                "Mind explicitly accepts or rejects this Objective Draft before Self schedules it.",
                "Self schedules only after Mind adoption and a safe-family action plan exist.",
                "Hands acts only through a later receipt-backed plan and declared path scope.",
                "Bifrost gates publication, credit, and upstream-main sync.",
            ]
    }

    pub(super) fn has_input_contract(&self) -> bool {
        self.inputs.consensus_brief_required
            && self
                .inputs
                .public_discussion_refs
                .iter()
                .all(|v| !v.trim().is_empty())
            && self
                .inputs
                .candidate_action_refs
                .iter()
                .all(|v| !v.trim().is_empty())
    }

    pub(super) fn has_authority_seals(&self) -> bool {
        let a = &self.authority;
        a.branch_local_only
            && !a.objective_adoption_authorized
            && !a.self_scheduling_authorized
            && !a.hands_action_authorized
            && !a.publication_authorized
            && !a.cross_body_mutation_authorized
            && a.mind_adoption_required
            && a.bifrost_publication_required
    }
}

#[derive(Debug, Deserialize)]
struct RepoObjectiveDraftState {
    status: String,
    owner: String,
    adoption_gate: String,
    scheduler_gate: String,
    publication_gate: String,
    objective_adopted: bool,
}
#[derive(Debug, Deserialize)]
struct RepoObjectiveDraftAcceptance {
    criteria: Vec<String>,
}
#[derive(Debug, Deserialize)]
struct RepoObjectiveDraftInputs {
    public_discussion_refs: Vec<String>,
    candidate_action_refs: Vec<String>,
    consensus_brief_required: bool,
}
#[derive(Debug, Deserialize)]
struct RepoObjectiveDraftAuthority {
    branch_local_only: bool,
    objective_adoption_authorized: bool,
    self_scheduling_authorized: bool,
    hands_action_authorized: bool,
    publication_authorized: bool,
    cross_body_mutation_authorized: bool,
    mind_adoption_required: bool,
    bifrost_publication_required: bool,
}

pub(super) fn parse_repo_objective_draft(text: &str) -> Result<RepoObjectiveDraft> {
    toml::from_str(text).context("objective draft is not valid typed TOML")
}

include!("preparation_tests.rs");
