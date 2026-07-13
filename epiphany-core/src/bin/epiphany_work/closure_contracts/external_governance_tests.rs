#[cfg(test)]
mod external_governance_tests {
    use super::*;

    #[test]
    fn readiness_comment_cannot_counterfeit_approval_seal() {
        let text = r#"
schema_version = "epiphany.repo_readiness_review_request.v0"
safe_action_family = "repo.readiness_review_request"
summary = "summary"
private_state_exposed = false
[request]
status = "awaiting-mvp-readiness-review"
routing_owner = "Self"
required_reviewers = ["Maintainer", "Soul", "Mind", "Bifrost"]
readiness_approval_owner = "Maintainer"
requested_effect = "review-redacted-repo-swarm-mvp-proof-bundle"
maintainer_readiness_acceptance_required = true
bifrost_publication_review_required = true
[antecedents]
[required_receipts]
[readiness_packet]
requires_proof_bundle_ref = false
requires_changed_path_list = false
requires_branch_name = false
requires_upstream_main_ref = false
requires_public_proof_ref = false
requires_bifrost_ledger_ref = false
requires_idunn_lifecycle_ref = false
requires_tool_directory_ref = false
requires_redaction_report = false
requires_reviewer_identity = false
allowed_verdicts = []
[authority]
# readiness_approval_authorized = false
readiness_approval_authorized = true
"#;
        let request = parse_repo_readiness_review_request(text).expect("fixture is typed TOML");
        assert!(!request.has_authority_seals());
    }

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
routing_owner = "Self"
accounting_owner = "Bifrost"
review_evidence_owner = "Maintainer"
spend_receipt_required = true
review_load_receipt_required = true
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
bifrost_accounting_required = true
maintainer_review_evidence_required = true
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
routing_owner = "Self"
acceptance_owner = "Maintainer"
accounting_owner = "Bifrost"
acceptance_receipt_required = true
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
maintainer_acceptance_authority_required = true
bifrost_accounting_required = true
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

    #[test]
    fn publication_comment_cannot_counterfeit_bifrost_seal() {
        let text = r#"
schema_version = "epiphany.repo_publication_request.v0"
safe_action_family = "repo.publication_request"
summary = "summary"
private_state_exposed = false
[request]
status = "awaiting-bifrost-review"
requested_owner = "Bifrost"
requested_effect = "publish-redacted-proof-and-route-maintainer-review"
verification_request_ref = "verification"
[antecedents]
closure_review_required = true
soul_verdict_required = true
mind_commit_required = true
public_proof_export_required = true
private_state_redaction_required = true
[required_receipts]
closure_review = "epiphany.repo_work_closure_review.v0"
soul_verdict = "epiphany.soul.verification_verdict"
mind_commit = "epiphany.mind.state_commit_receipt"
public_proof = "epiphany.repo_work_public_proof_bundle.v0"
bifrost_publication = "gamecult.bifrost.public_proof_publication_receipt.v0"
github_publication = "gamecult.github.publication_receipt.v0"
credit_ledger = "gamecult.bifrost.credit_receipt.v0"
upstream_sync = "epiphany.repo_work_upstream_main_sync.v0"
[public_export]
redacted_only = true
raw_receipts_allowed = false
private_paths_allowed = false
worker_thought_allowed = false
operator_context_allowed = false
credit_required = true
[authority]
branch_local_only = true
# bifrost_publication_authorized = false
bifrost_publication_authorized = true
github_publication_authorized = false
credit_ledger_authorized = false
merge_authorized = false
upstream_sync_authorized = false
hands_action_authorized = false
service_lifecycle_authority = false
cross_body_mutation_authorized = false
private_verse_rummaging = false
maintainer_review_required = true
"#;
        let request = parse_repo_publication_request(text).expect("fixture is typed TOML");
        assert!(!request.has_authority_seals());
    }

    #[test]
    fn sync_comment_cannot_counterfeit_push_seal() {
        let text = r#"
schema_version = "epiphany.repo_sync_request.v0"
safe_action_family = "repo.sync_request"
summary = "summary"
private_state_exposed = false
[request]
status = "awaiting-upstream-main-proof"
requested_owner = "Bifrost"
requested_effect = "prove-published-commit-contained-by-upstream-main"
publication_request_ref = "publication"
[antecedents]
publication_receipt_required = true
github_publication_required = true
maintainer_review_required = true
credit_ledger_required = true
public_proof_required = true
[required_receipts]
bifrost_publication = "gamecult.bifrost.public_proof_publication_receipt.v0"
github_publication = "gamecult.github.publication_receipt.v0"
maintainer_review = "gamecult.maintainer.review_receipt.v0"
credit_ledger = "gamecult.bifrost.credit_receipt.v0"
upstream_sync = "epiphany.repo_work_upstream_main_sync.v0"
ancestry_proof = "git.merge_base_is_ancestor"
[sync_proof]
target_ref = "origin/main"
requires_fetch_before_check = true
requires_merge_base_ancestor_check = true
requires_clean_public_proof_readback = true
records_upstream_main_sha = true
[authority]
branch_local_only = true
merge_authorized = false
# push_authorized = false
push_authorized = true
upstream_sync_authorized = false
github_publication_authorized = false
credit_ledger_authorized = false
hands_action_authorized = false
service_lifecycle_authority = false
cross_body_mutation_authorized = false
private_verse_rummaging = false
operator_or_maintainer_authority_required = true
"#;
        let request = parse_repo_sync_request(text).expect("fixture is typed TOML");
        assert!(!request.has_authority_seals());
    }

    #[test]
    fn pr_comment_cannot_counterfeit_github_seal() {
        let text = r#"
schema_version = "epiphany.repo_pr_request.v0"
safe_action_family = "repo.pr_request"
summary = "summary"
private_state_exposed = false
[request]
status = "awaiting-pr-publication-review"
routing_owner = "Self"
publication_owner = "Bifrost"
execution_owner = "Hands"
provider = "GitHub"
provider_receipt_required = true
requested_effect = "open-or-update-review-pr-from-redacted-proof-and-maintainer-context"
maintainer_review_request_ref = "review"
publication_request_ref = "publication"
sync_request_ref = "sync"
[antecedents]
closure_review_required = true
soul_verdict_required = true
mind_commit_required = true
public_proof_required = true
maintainer_review_required = true
bifrost_publication_required = true
credit_ledger_required = true
[required_receipts]
closure_review = "epiphany.repo_work_closure_review.v0"
soul_verdict = "epiphany.soul.verification_verdict"
mind_commit = "epiphany.mind.state_commit_receipt"
public_proof = "epiphany.repo_work_public_proof_bundle.v0"
maintainer_review = "gamecult.maintainer.review_receipt.v0"
bifrost_publication = "gamecult.bifrost.public_proof_publication_receipt.v0"
credit_ledger = "gamecult.bifrost.credit_receipt.v0"
pr_publication = "gamecult.github.pull_request_publication_receipt.v0"
[pr_packet]
base_ref = "origin/main"
requires_branch_name = true
requires_title = true
requires_body = true
requires_changed_path_list = true
requires_public_proof_ref = true
requires_maintainer_review_ref = true
requires_credit_ref = true
requires_private_state_redaction_check = true
[authority]
branch_local_only = true
# github_pr_authorized = false
github_pr_authorized = true
branch_push_authorized = false
merge_authorized = false
publication_authorized = false
upstream_sync_authorized = false
hands_action_authorized = false
service_lifecycle_authority = false
cross_body_mutation_authorized = false
private_verse_rummaging = false
maintainer_review_required = true
bifrost_publication_gate_required = true
hands_execution_required = true
github_provider_receipt_required = true
"#;
        let request = parse_repo_pr_request(text).expect("fixture is typed TOML");
        assert!(!request.has_authority_seals());
    }

    #[test]
    fn maintainer_comment_cannot_counterfeit_approval_seal() {
        let text = r#"
schema_version = "epiphany.repo_maintainer_review_request.v0"
safe_action_family = "repo.maintainer_review_request"
summary = "summary"
private_state_exposed = false
[request]
status = "awaiting-maintainer-review"
requested_owner = "Maintainer"
requested_effect = "review-redacted-proof-and-branch-diff"
verification_request_ref = "verification"
publication_request_ref = "publication"
[antecedents]
closure_review_required = true
soul_verdict_required = true
mind_commit_required = true
public_proof_required = true
bifrost_publication_request_required = true
[required_receipts]
closure_review = "epiphany.repo_work_closure_review.v0"
soul_verdict = "epiphany.soul.verification_verdict"
mind_commit = "epiphany.mind.state_commit_receipt"
public_proof = "epiphany.repo_work_public_proof_bundle.v0"
maintainer_review = "gamecult.maintainer.review_receipt.v0"
bifrost_publication = "gamecult.bifrost.public_proof_publication_receipt.v0"
[review_packet]
requires_reviewer_identity = true
requires_review_verdict = true
allowed_verdicts = ["approved", "changes-requested", "rejected", "needs-human-context"]
requires_changed_path_list = true
requires_public_proof_ref = true
requires_private_state_redaction_check = true
[authority]
branch_local_only = true
# maintainer_approval_authorized = false
maintainer_approval_authorized = true
merge_authorized = false
push_authorized = false
publication_authorized = false
upstream_sync_authorized = false
hands_action_authorized = false
service_lifecycle_authority = false
cross_body_mutation_authorized = false
private_verse_rummaging = false
human_or_maintainer_response_required = true
"#;
        let request = parse_repo_maintainer_review_request(text).expect("fixture is typed TOML");
        assert!(!request.has_authority_seals());
    }
}
