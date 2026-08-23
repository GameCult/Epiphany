use crate::EpiphanyReorientFindingInterpretation;
use cultcache_rs::DatabaseEntry;

pub const CONTINUITY_RECOVERY_RECEIPT_TYPE: &str = "epiphany.continuity.recovery_receipt";
pub const CONTINUITY_RECOVERY_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.continuity.recovery_receipt.v0";

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.continuity.recovery_receipt",
    schema = "ContinuityRecoveryReceipt"
)]
pub struct ContinuityRecoveryReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub source_result_id: String,
    #[cultcache(key = 3)]
    pub source_job_id: String,
    #[cultcache(key = 4)]
    pub binding_id: String,
    #[cultcache(key = 5)]
    pub mode: String,
    #[cultcache(key = 6)]
    pub checkpoint_still_valid: String,
    #[cultcache(key = 7)]
    pub summary: String,
    #[cultcache(key = 8)]
    pub next_safe_move: String,
    #[cultcache(key = 9)]
    pub files_inspected: Vec<String>,
    #[cultcache(key = 10)]
    pub emitted_at: String,
    #[cultcache(key = 11)]
    pub contract: String,
}

pub fn continuity_recovery_receipt_from_reorient_finding(
    receipt_id: String,
    binding_id: String,
    finding: &EpiphanyReorientFindingInterpretation,
    emitted_at: String,
) -> ContinuityRecoveryReceipt {
    ContinuityRecoveryReceipt {
        schema_version: CONTINUITY_RECOVERY_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id,
        source_result_id: finding.runtime_result_id.clone().unwrap_or_default(),
        source_job_id: finding.runtime_job_id.clone().unwrap_or_default(),
        binding_id,
        mode: finding.mode.clone().unwrap_or_else(|| "unknown".to_string()),
        checkpoint_still_valid: finding
            .checkpoint_still_valid
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        summary: finding.summary.clone().unwrap_or_default(),
        next_safe_move: finding.next_safe_move.clone().unwrap_or_default(),
        files_inspected: finding.files_inspected.clone(),
        emitted_at,
        contract: "Continuity recovery emitted from a reviewed reorientation finding; it proves what survived rupture before Mind admits recovery state.".to_string(),
    }
}
