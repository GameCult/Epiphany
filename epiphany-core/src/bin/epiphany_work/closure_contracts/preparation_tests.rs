#[cfg(test)]
mod preparation_tests {
    use super::*;

    #[test]
    fn objective_draft_comment_cannot_counterfeit_adoption_seal() {
        let text = r#"
schema_version = "epiphany.repo_objective_draft.v0"
safe_action_family = "repo.objective_draft"
summary = "summary"
private_state_exposed = false
[draft]
status = "review-required"
owner = "Imagination"
adoption_gate = "Mind"
scheduler_gate = "Self"
publication_gate = "Bifrost"
objective_adopted = false
[acceptance]
criteria = ["Mind explicitly accepts or rejects this Objective Draft before Self schedules it.", "Self schedules only after Mind adoption and a safe-family action plan exist.", "Hands acts only through a later receipt-backed plan and declared path scope.", "Bifrost gates publication, credit, and upstream-main sync."]
[inputs]
public_discussion_refs = []
candidate_action_refs = []
consensus_brief_required = true
[authority]
branch_local_only = true
# objective_adoption_authorized = false
objective_adoption_authorized = true
self_scheduling_authorized = false
hands_action_authorized = false
publication_authorized = false
cross_body_mutation_authorized = false
mind_adoption_required = true
bifrost_publication_required = true
"#;
        let draft = parse_repo_objective_draft(text).expect("fixture is typed TOML");
        assert!(!draft.has_authority_seals());
    }
}
