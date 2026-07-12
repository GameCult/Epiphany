use cultcache_rs::DatabaseEntry;

pub const REPO_WORK_MODELING_FINDING_TYPE: &str = "epiphany.modeling.repo_work_finding";
pub const REPO_WORK_MODELING_FINDING_SCHEMA_VERSION: &str =
    "epiphany.modeling.repo_work_finding.v0";

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
}
