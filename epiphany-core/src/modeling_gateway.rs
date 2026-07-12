use cultcache_rs::DatabaseEntry;

pub const REPO_WORK_MODELING_FINDING_TYPE: &str = "epiphany.modeling.repo_work_finding";
pub const REPO_WORK_MODELING_FINDING_SCHEMA_VERSION: &str =
    "epiphany.modeling.repo_work_finding.v0";
pub const REPO_WORK_MODELING_REQUEST_TYPE: &str = "epiphany.modeling.repo_work_request";
pub const REPO_WORK_MODELING_REQUEST_SCHEMA_VERSION: &str =
    "epiphany.modeling.repo_work_request.v0";
pub const REPO_WORK_MODELING_ROUTE_TYPE: &str = "epiphany.modeling.repo_work_route";
pub const REPO_WORK_MODELING_ROUTE_SCHEMA_VERSION: &str = "epiphany.modeling.repo_work_route.v0";
pub const REPO_WORK_MAP_ENTRY_TYPE: &str = "epiphany.repo_work.map_entry";
pub const REPO_WORK_MAP_ENTRY_SCHEMA_VERSION: &str = "epiphany.repo_work.map_entry.v0";

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.modeling.repo_work_finding",
    schema = "RepoWorkModelingFinding"
)]
pub struct RepoWorkModelingFinding {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub item: String,
    #[cultcache(key = 3)]
    pub model_ref: String,
    #[cultcache(key = 4)]
    pub soul_verdict_receipt_id: String,
    #[cultcache(key = 5)]
    pub verdict: String,
    #[cultcache(key = 6)]
    pub finding: String,
    #[cultcache(key = 7)]
    pub summary: String,
    #[cultcache(key = 8)]
    pub changed_paths: Vec<String>,
    #[cultcache(key = 9)]
    pub commit_sha: String,
    #[cultcache(key = 10)]
    pub emitted_at: String,
    #[cultcache(key = 11)]
    pub private_state_exposed: bool,
    #[cultcache(key = 12)]
    pub contract: String,
    #[cultcache(key = 13)]
    pub request_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.modeling.repo_work_request",
    schema = "RepoWorkModelingRequest"
)]
pub struct RepoWorkModelingRequest {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub item: String,
    #[cultcache(key = 3)]
    pub requester: String,
    #[cultcache(key = 4)]
    pub soul_verdict_receipt_id: String,
    #[cultcache(key = 5)]
    pub commit_sha: String,
    #[cultcache(key = 6)]
    pub changed_paths: Vec<String>,
    #[cultcache(key = 7)]
    pub instruction: String,
    #[cultcache(key = 8)]
    pub requested_at: String,
    #[cultcache(key = 9)]
    pub private_state_exposed: bool,
    #[cultcache(key = 10)]
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.modeling.repo_work_route",
    schema = "RepoWorkModelingRoute"
)]
pub struct RepoWorkModelingRoute {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub route_id: String,
    #[cultcache(key = 2)]
    pub item: String,
    #[cultcache(key = 3)]
    pub generation: u64,
    #[cultcache(key = 4)]
    pub request_id: String,
    #[cultcache(key = 5)]
    pub previous_finding_receipt_id: String,
    #[cultcache(key = 6)]
    pub authority_owner: String,
    #[cultcache(key = 7)]
    pub authority_witness_id: String,
    #[cultcache(key = 8)]
    pub updated_at: String,
    #[cultcache(key = 9)]
    pub private_state_exposed: bool,
    #[cultcache(key = 10)]
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.repo_work.map_entry", schema = "RepoWorkMapEntry")]
pub struct RepoWorkMapEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub map_entry_id: String,
    #[cultcache(key = 2)]
    pub admitted_at: String,
    #[cultcache(key = 3)]
    pub item: String,
    #[cultcache(key = 4)]
    pub branch: String,
    #[cultcache(key = 5)]
    pub changed_paths: Vec<String>,
    #[cultcache(key = 6)]
    pub commit_sha: String,
    #[cultcache(key = 7)]
    pub safe_action_family: String,
    #[cultcache(key = 8)]
    pub modeling_summary: String,
    #[cultcache(key = 9)]
    pub modeling_finding_receipt_id: String,
    #[cultcache(key = 10)]
    pub soul_verdict_receipt_id: String,
    #[cultcache(key = 11)]
    pub mind_gateway_review_id: String,
    #[cultcache(key = 12)]
    pub mind_state_commit_receipt_id: String,
    #[cultcache(key = 13)]
    pub execute_receipt_path: String,
    #[cultcache(key = 14)]
    pub closure_review_path: String,
    #[cultcache(key = 15)]
    pub closure_receipt_path: String,
    #[cultcache(key = 16)]
    pub publication_gate: String,
    #[cultcache(key = 17)]
    pub durable_state_admitted: bool,
    #[cultcache(key = 18)]
    pub private_state_exposed: bool,
    #[cultcache(key = 19)]
    pub modeling_route_id: String,
    #[cultcache(key = 20)]
    pub modeling_generation: u64,
}
