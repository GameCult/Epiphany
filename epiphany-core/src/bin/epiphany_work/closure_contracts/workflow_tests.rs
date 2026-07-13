#[cfg(test)]
mod workflow_tests {
    use super::*;

    #[test]
    fn verification_comment_cannot_counterfeit_soul_verdict_seal() {
        let text = r#"
schema_version = "epiphany.repo_verification_request.v0"
safe_action_family = "repo.verification_request"
summary = "summary"
private_state_exposed = false
[request]
status = "awaiting-soul-review"
requested_owner = "Soul"
requested_effect = "verify-branch-local-hands-work"
work_order_ref = "work-order"
[antecedents]
substrate_gate_required = true
hands_intent_required = true
hands_review_required = true
hands_patch_required = true
hands_command_required = true
hands_commit_required = true
work_order_required = true
[required_receipts]
hands_patch = "epiphany.hands.patch_receipt"
hands_command = "epiphany.hands.command_receipt"
hands_commit = "epiphany.hands.commit_receipt"
soul_verdict = "epiphany.soul.verification_verdict"
closure_review = "epiphany.repo_work_closure_review.v0"
mind_review = "epiphany.mind.gateway_review"
mind_commit = "epiphany.mind.state_commit_receipt"
[checks]
required = ["declared-paths-match-commit", "hands-receipts-present", "visible-diff-supports-summary", "no-private-state-exposure", "publication-remains-gated"]
model_verdict_allowed = true
failure_blocks_mind_admission = true
[authority]
branch_local_only = true
# soul_verdict_authorized = false
soul_verdict_authorized = true
state_commit_authorized = false
hands_action_authorized = false
rerun_authorized = false
publication_authorized = false
merge_authorized = false
service_lifecycle_authority = false
cross_body_mutation_authorized = false
private_verse_rummaging = false
bifrost_publication_required = true
"#;
        let request = parse_repo_verification_request(text).expect("fixture is typed TOML");
        assert!(!request.has_authority_seals());
    }
}
