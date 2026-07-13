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
