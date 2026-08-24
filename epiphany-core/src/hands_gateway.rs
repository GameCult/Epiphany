use cultcache_rs::DatabaseEntry;

pub const HANDS_COMMAND_RECEIPT_TYPE: &str = "epiphany.hands.command_receipt";
pub const HANDS_PATCH_RECEIPT_TYPE: &str = "epiphany.hands.patch_receipt";
pub const HANDS_COMMIT_RECEIPT_TYPE: &str = "epiphany.hands.commit_receipt";
#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.hands.action_intent", schema = "HandsActionIntent")]
pub struct HandsActionIntent {
    #[cultcache(key = 0)]
    pub intent_id: String,
    #[cultcache(key = 1)]
    pub runtime_job_id: String,
    #[cultcache(key = 2)]
    pub requested_paths: Vec<String>,
    #[cultcache(key = 3)]
    pub substrate_gate_grant_receipt_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.hands.patch_receipt", schema = "HandsPatchReceipt")]
pub struct HandsPatchReceipt {
    #[cultcache(key = 0)]
    pub receipt_id: String,
    #[cultcache(key = 1)]
    pub intent_id: String,
    #[cultcache(key = 2)]
    pub changed_paths: Vec<String>,
    #[cultcache(key = 3)]
    pub summary: String,
    #[cultcache(key = 4)]
    pub emitted_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.hands.command_receipt",
    schema = "HandsCommandReceipt"
)]
pub struct HandsCommandReceipt {
    #[cultcache(key = 0)]
    pub receipt_id: String,
    #[cultcache(key = 1)]
    pub intent_id: String,
    #[cultcache(key = 2)]
    pub command: String,
    #[cultcache(key = 3)]
    pub exit_code: String,
    #[cultcache(key = 4)]
    pub stdout_artifact: String,
    #[cultcache(key = 5)]
    pub stderr_artifact: String,
    #[cultcache(key = 6)]
    pub summary: String,
    #[cultcache(key = 7)]
    pub emitted_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.hands.commit_receipt", schema = "HandsCommitReceipt")]
pub struct HandsCommitReceipt {
    #[cultcache(key = 0)]
    pub receipt_id: String,
    #[cultcache(key = 1)]
    pub intent_id: String,
    #[cultcache(key = 2)]
    pub commit_sha: String,
    #[cultcache(key = 3)]
    pub branch: String,
    #[cultcache(key = 4)]
    pub changed_paths: Vec<String>,
    #[cultcache(key = 5)]
    pub summary: String,
    #[cultcache(key = 6)]
    pub emitted_at: String,
}

pub fn hands_patch_receipt_for_intent(
    receipt_id: String,
    intent: &HandsActionIntent,
    changed_paths: Vec<String>,
    summary: String,
    emitted_at: String,
) -> HandsPatchReceipt {
    HandsPatchReceipt {
        receipt_id,
        intent_id: intent.intent_id.clone(),
        changed_paths,
        summary,
        emitted_at,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn hands_command_receipt_for_intent(
    receipt_id: String,
    intent: &HandsActionIntent,
    command: String,
    exit_code: String,
    stdout_artifact: String,
    stderr_artifact: String,
    summary: String,
    emitted_at: String,
) -> HandsCommandReceipt {
    HandsCommandReceipt {
        receipt_id,
        intent_id: intent.intent_id.clone(),
        command,
        exit_code,
        stdout_artifact,
        stderr_artifact,
        summary,
        emitted_at,
    }
}

pub fn hands_commit_receipt_for_intent(
    receipt_id: String,
    intent: &HandsActionIntent,
    commit_sha: String,
    branch: String,
    changed_paths: Vec<String>,
    summary: String,
    emitted_at: String,
) -> HandsCommitReceipt {
    HandsCommitReceipt {
        receipt_id,
        intent_id: intent.intent_id.clone(),
        commit_sha,
        branch,
        changed_paths,
        summary,
        emitted_at,
    }
}
