use crate::EpiphanyJobLaunchRequest;
use cultcache_rs::DatabaseEntry;
use serde::Deserialize;
use serde::Serialize;

pub const SUBSTRATE_GATE_REPO_ACCESS_REQUEST_TYPE: &str =
    "epiphany.substrate_gate.repo_access_request";
pub const SUBSTRATE_GATE_REPO_ACCESS_REVIEW_TYPE: &str =
    "epiphany.substrate_gate.repo_access_review";
pub const SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE: &str =
    "epiphany.substrate_gate.repo_access_grant_receipt";
pub const SUBSTRATE_GATE_REPO_ACCESS_REFUSAL_RECEIPT_TYPE: &str =
    "epiphany.substrate_gate.repo_access_refusal_receipt";
pub const SUBSTRATE_GATE_REPO_SNAPSHOT_RECEIPT_TYPE: &str =
    "epiphany.substrate_gate.repo_snapshot_receipt";
pub const SUBSTRATE_GATE_REPO_MUTATION_RECEIPT_TYPE: &str =
    "epiphany.substrate_gate.repo_mutation_receipt";
pub const SUBSTRATE_GATE_REPO_ACCESS_REQUEST_SCHEMA_VERSION: &str =
    "epiphany.substrate_gate.repo_access_request.v0";
pub const SUBSTRATE_GATE_REPO_ACCESS_REVIEW_SCHEMA_VERSION: &str =
    "epiphany.substrate_gate.repo_access_review.v0";
pub const SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.substrate_gate.repo_access_grant_receipt.v0";
pub const SUBSTRATE_GATE_REPO_ACCESS_REFUSAL_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.substrate_gate.repo_access_refusal_receipt.v0";
pub const SUBSTRATE_GATE_REPO_SNAPSHOT_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.substrate_gate.repo_snapshot_receipt.v0";
pub const SUBSTRATE_GATE_REPO_MUTATION_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.substrate_gate.repo_mutation_receipt.v0";

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.substrate_gate.repo_access_grant_receipt",
    schema = "SubstrateGateRepoAccessGrantReceipt"
)]
#[non_exhaustive]
pub struct SubstrateGateRepoAccessGrantReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub runtime_job_id: String,
    #[cultcache(key = 3)]
    pub binding_id: String,
    #[cultcache(key = 4)]
    pub role: String,
    #[cultcache(key = 5)]
    pub authority_scope: String,
    #[cultcache(key = 6)]
    pub granted_operations: Vec<String>,
    #[cultcache(key = 7)]
    pub granted_paths: Vec<String>,
    #[cultcache(key = 8)]
    pub granted_at: String,
    #[cultcache(key = 9)]
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubstrateGateCultNetContract {
    pub contract_id: String,
    pub verse_id: String,
    pub document_type: String,
    pub payload_schema_version: String,
    pub authority: String,
    pub operations: Vec<String>,
    pub intent_document_types: Vec<String>,
    pub receipt_document_types: Vec<String>,
    pub notes: Vec<String>,
}

pub fn default_substrate_gate_cultnet_contracts() -> Vec<SubstrateGateCultNetContract> {
    vec![
        SubstrateGateCultNetContract {
            contract_id: "epiphany.substrate_gate.repo_access.review".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: SUBSTRATE_GATE_REPO_ACCESS_REQUEST_TYPE.to_string(),
            payload_schema_version: SUBSTRATE_GATE_REPO_ACCESS_REQUEST_SCHEMA_VERSION.to_string(),
            authority: "substrateGate".to_string(),
            operations: vec![
                "intentSubmit".to_string(),
                "receiptWatch".to_string(),
                "snapshot".to_string(),
            ],
            intent_document_types: vec![SUBSTRATE_GATE_REPO_ACCESS_REQUEST_TYPE.to_string()],
            receipt_document_types: vec![
                SUBSTRATE_GATE_REPO_ACCESS_REVIEW_TYPE.to_string(),
                SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE.to_string(),
                SUBSTRATE_GATE_REPO_ACCESS_REFUSAL_RECEIPT_TYPE.to_string(),
            ],
            notes: vec![
                "Substrate Gate is the repo access protocol: workers request repository reads, indexing, commands, edits, and bridge operations here.".to_string(),
                "Hands may mutate files only after Substrate Gate grants scoped access; Eyes may inspect only through scoped Substrate Gate read/index grants.".to_string(),
            ],
        },
        SubstrateGateCultNetContract {
            contract_id: "epiphany.substrate_gate.repo_access.review_receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: SUBSTRATE_GATE_REPO_ACCESS_REVIEW_TYPE.to_string(),
            payload_schema_version: SUBSTRATE_GATE_REPO_ACCESS_REVIEW_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Substrate Gate reviews explain which repo paths, operations, commands, and bridge surfaces were granted or refused.".to_string(),
            ],
        },
        SubstrateGateCultNetContract {
            contract_id: "epiphany.substrate_gate.repo_snapshot.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: SUBSTRATE_GATE_REPO_SNAPSHOT_RECEIPT_TYPE.to_string(),
            payload_schema_version: SUBSTRATE_GATE_REPO_SNAPSHOT_RECEIPT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Repo snapshots are evidence projections from Substrate Gate access; they do not grant future access by existing.".to_string(),
            ],
        },
        SubstrateGateCultNetContract {
            contract_id: "epiphany.substrate_gate.repo_mutation.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: SUBSTRATE_GATE_REPO_MUTATION_RECEIPT_TYPE.to_string(),
            payload_schema_version: SUBSTRATE_GATE_REPO_MUTATION_RECEIPT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Repo mutation receipts prove Substrate Gate granted the scoped substrate touch before Hands changed files or ran repo-affecting commands.".to_string(),
            ],
        },
    ]
}

pub fn substrate_gate_repo_access_grant_for_launch(
    receipt_id: String,
    runtime_job_id: String,
    request: &EpiphanyJobLaunchRequest,
    granted_at: String,
) -> SubstrateGateRepoAccessGrantReceipt {
    SubstrateGateRepoAccessGrantReceipt {
        schema_version: SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id,
        runtime_job_id,
        binding_id: request.binding_id.clone(),
        role: request.owner_role.clone(),
        authority_scope: request.authority_scope.clone(),
        granted_operations: vec!["read".to_string(), "snapshot".to_string()],
        granted_paths: vec![".".to_string()],
        granted_at,
        contract: "Substrate Gate granted scoped repository read/snapshot access for this worker launch; mutation remains forbidden without a separate mutation receipt.".to_string(),
    }
}

pub fn substrate_gate_repo_work_planning_grant(
    receipt_id: String,
    runtime_job_id: String,
    granted_paths: Vec<String>,
    granted_at: String,
) -> SubstrateGateRepoAccessGrantReceipt {
    SubstrateGateRepoAccessGrantReceipt {
        schema_version: SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id,
        runtime_job_id,
        binding_id: "repo-work-runner".to_string(),
        role: "epiphany-hands".to_string(),
        authority_scope: "repo.branch_local_work".to_string(),
        granted_operations: vec!["read".to_string(), "snapshot".to_string()],
        granted_paths,
        granted_at,
        contract: "Substrate Gate grants read/snapshot access for repo-work planning only; mutation awaits an approved Hands review.".to_string(),
    }
}

pub fn substrate_gate_coordinator_implementation_grant(
    receipt_id: String,
    runtime_job_id: String,
    granted_paths: Vec<String>,
    granted_at: String,
) -> SubstrateGateRepoAccessGrantReceipt {
    SubstrateGateRepoAccessGrantReceipt {
        schema_version: SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id,
        runtime_job_id,
        binding_id: "implementation-worker".to_string(),
        role: "epiphany-hands".to_string(),
        authority_scope: "epiphany.role.implementation".to_string(),
        granted_operations: vec![
            "read".to_string(),
            "snapshot".to_string(),
            "patch".to_string(),
            "command".to_string(),
            "commit".to_string(),
        ],
        granted_paths,
        granted_at,
        contract: "Substrate Gate grants scoped repository access for a coordinator-approved implementation continuation; every mutation still needs Hands receipts.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substrate_gate_contracts_make_internal_verse_the_repo_access_gate() {
        let contracts = default_substrate_gate_cultnet_contracts();
        let repo_access = contracts
            .iter()
            .find(|contract| contract.contract_id == "epiphany.substrate_gate.repo_access.review")
            .expect("repo access review contract");

        assert_eq!(repo_access.verse_id, "epiphany-internal");
        assert_eq!(repo_access.authority, "substrateGate");
        assert!(
            repo_access
                .notes
                .iter()
                .any(|note| note.contains("repo access protocol"))
        );
        assert!(
            repo_access
                .receipt_document_types
                .contains(&SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE.to_string())
        );
    }

    #[test]
    fn substrate_gate_grant_for_launch_is_read_only() {
        let request = EpiphanyJobLaunchRequest {
            expected_revision: Some(7),
            binding_id: "research-source-gather-worker".to_string(),
            kind: epiphany_state_model::EpiphanyJobKind::Specialist,
            scope: "research".to_string(),
            owner_role: "epiphany-eyes".to_string(),
            authority_scope: "epiphany.role.research".to_string(),
            linked_subgoal_ids: Vec::new(),
            linked_graph_node_ids: Vec::new(),
            instruction: "Gather source evidence.".to_string(),
            launch_document: crate::EpiphanyWorkerLaunchDocument::Role(
                crate::EpiphanyRoleWorkerLaunchDocument {
                    thread_id: "thread-1".to_string(),
                    role_id: "research".to_string(),
                    state_revision: 7,
                    objective: None,
                    dynamic_prompt_context: None,
                    proposal_modeling_context: None,
                    active_subgoal_id: None,
                    active_subgoals: Vec::new(),
                    active_graph_node_ids: Vec::new(),
                    investigation_checkpoint: None,
                    scratch: None,
                    invariants: Vec::new(),
                    graphs: None,
                    recent_evidence: Vec::new(),
                    recent_observations: Vec::new(),
                    graph_frontier: None,
                    graph_checkpoint: None,
                    planning: None,
                    churn: None,
                },
            ),
            output_contract_id: crate::ROLE_WORKER_OUTPUT_CONTRACT_ID.to_string(),
            organ_launch_contract: crate::default_launch_organ_contract(
                "epiphany.role.research",
                "role",
                crate::ROLE_WORKER_OUTPUT_CONTRACT_ID,
            ),
            max_runtime_seconds: None,
            proposal_modeling_request_id: None,
        };
        let grant = substrate_gate_repo_access_grant_for_launch(
            "grant-1".to_string(),
            "job-1".to_string(),
            &request,
            "2026-05-30T00:00:00Z".to_string(),
        );
        assert!(grant.granted_operations.contains(&"read".to_string()));
        assert!(!grant.granted_operations.contains(&"write".to_string()));
        assert!(grant.contract.contains("mutation remains forbidden"));
    }

    #[test]
    fn repo_work_and_implementation_grants_have_fixed_policies() {
        let planning = substrate_gate_repo_work_planning_grant(
            "planning-grant".to_string(),
            "planning-job".to_string(),
            vec!["README.md".to_string()],
            "2026-07-12T00:00:00Z".to_string(),
        );
        assert_eq!(planning.binding_id, "repo-work-runner");
        assert_eq!(planning.granted_operations, vec!["read", "snapshot"]);

        let implementation = substrate_gate_coordinator_implementation_grant(
            "implementation-grant".to_string(),
            "implementation-job".to_string(),
            vec!["src".to_string()],
            "2026-07-12T00:00:00Z".to_string(),
        );
        assert_eq!(implementation.binding_id, "implementation-worker");
        assert_eq!(
            implementation.granted_operations,
            vec!["read", "snapshot", "patch", "command", "commit"]
        );
    }
}
