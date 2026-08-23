use cultcache_rs::DatabaseEntry;

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
pub const SUBSTRATE_GATE_SOURCE_READ_OPERATION: &str = "read";
pub const SUBSTRATE_GATE_PUBLIC_SOURCE_READ_OPERATION: &str = "publicSourceRead";

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

pub fn substrate_gate_repo_access_grant_for_worker(
    receipt_id: String,
    runtime_job_id: String,
    binding_id: String,
    role: String,
    authority_scope: String,
    allow_public_source_read: bool,
    granted_at: String,
) -> SubstrateGateRepoAccessGrantReceipt {
    let mut granted_operations = vec![
        SUBSTRATE_GATE_SOURCE_READ_OPERATION.to_string(),
        "snapshot".to_string(),
    ];
    if allow_public_source_read {
        granted_operations.push(SUBSTRATE_GATE_PUBLIC_SOURCE_READ_OPERATION.to_string());
    }
    SubstrateGateRepoAccessGrantReceipt {
        schema_version: SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id,
        runtime_job_id,
        binding_id,
        role,
        authority_scope,
        granted_operations,
        granted_paths: vec![".".to_string()],
        granted_at,
        contract: "Substrate Gate granted the exact worker launch its named governed source operations; public-source reads are Eyes-only and mutation remains forbidden without a separate mutation receipt.".to_string(),
    }
}

pub fn substrate_gate_operation_for_governed_tool(
    server: &str,
    tool_name: &str,
) -> Option<&'static str> {
    match (server, tool_name) {
        (
            "epiphany_source",
            "read_file" | "directory_inventory" | "git_show" | "read_hands_receipt",
        )
        | ("epiphany_state", "resident_grant_lifecycle") => {
            Some(SUBSTRATE_GATE_SOURCE_READ_OPERATION)
        }
        ("epiphany_public", "github_file") => Some(SUBSTRATE_GATE_PUBLIC_SOURCE_READ_OPERATION),
        _ => None,
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
    fn substrate_gate_grant_for_worker_is_read_only() {
        let grant = substrate_gate_repo_access_grant_for_worker(
            "grant-1".to_string(),
            "job-1".to_string(),
            "research-source-gather-worker".to_string(),
            "epiphany-eyes".to_string(),
            "epiphany.role.research".to_string(),
            true,
            "2026-05-30T00:00:00Z".to_string(),
        );
        assert!(grant.granted_operations.contains(&"read".to_string()));
        assert!(
            grant
                .granted_operations
                .contains(&"publicSourceRead".to_string())
        );
        assert!(!grant.granted_operations.contains(&"write".to_string()));
        assert!(grant.contract.contains("mutation remains forbidden"));
    }

    #[test]
    fn governed_tools_map_to_fixed_grant_operations() {
        assert_eq!(
            substrate_gate_operation_for_governed_tool("epiphany_source", "read_file"),
            Some("read")
        );
        assert_eq!(
            substrate_gate_operation_for_governed_tool("epiphany_public", "github_file"),
            Some("publicSourceRead")
        );
        assert_eq!(
            substrate_gate_operation_for_governed_tool("epiphany_public", "arbitrary_url"),
            None
        );
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
