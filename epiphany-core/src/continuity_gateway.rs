use crate::EpiphanyReorientFindingInterpretation;
use cultcache_rs::DatabaseEntry;
use serde::Deserialize;
use serde::Serialize;

pub const CONTINUITY_PACKET_TYPE: &str = "epiphany.continuity.packet";
pub const CONTINUITY_COMPACTION_CHECKPOINT_TYPE: &str = "epiphany.continuity.compaction_checkpoint";
pub const CONTINUITY_SLEEP_DISTILLATION_TYPE: &str = "epiphany.continuity.sleep_distillation";
pub const CONTINUITY_RECOVERY_RECEIPT_TYPE: &str = "epiphany.continuity.recovery_receipt";
pub const CONTINUITY_STALE_TURN_REPAIR_TYPE: &str = "epiphany.continuity.stale_turn_repair";
pub const CONTINUITY_REFUSAL_RECEIPT_TYPE: &str = "epiphany.continuity.refusal_receipt";
pub const CONTINUITY_PACKET_SCHEMA_VERSION: &str = "epiphany.continuity.packet.v0";
pub const CONTINUITY_COMPACTION_CHECKPOINT_SCHEMA_VERSION: &str =
    "epiphany.continuity.compaction_checkpoint.v0";
pub const CONTINUITY_SLEEP_DISTILLATION_SCHEMA_VERSION: &str =
    "epiphany.continuity.sleep_distillation.v0";
pub const CONTINUITY_RECOVERY_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.continuity.recovery_receipt.v0";
pub const CONTINUITY_STALE_TURN_REPAIR_SCHEMA_VERSION: &str =
    "epiphany.continuity.stale_turn_repair.v0";
pub const CONTINUITY_REFUSAL_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.continuity.refusal_receipt.v0";

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuityCultNetContract {
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

pub fn default_continuity_cultnet_contracts() -> Vec<ContinuityCultNetContract> {
    vec![
        ContinuityCultNetContract {
            contract_id: "epiphany.continuity.review".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: CONTINUITY_PACKET_TYPE.to_string(),
            payload_schema_version: CONTINUITY_PACKET_SCHEMA_VERSION.to_string(),
            authority: "continuity".to_string(),
            operations: vec![
                "intentSubmit".to_string(),
                "receiptWatch".to_string(),
                "snapshot".to_string(),
            ],
            intent_document_types: vec![CONTINUITY_PACKET_TYPE.to_string()],
            receipt_document_types: vec![
                CONTINUITY_COMPACTION_CHECKPOINT_TYPE.to_string(),
                CONTINUITY_SLEEP_DISTILLATION_TYPE.to_string(),
                CONTINUITY_RECOVERY_RECEIPT_TYPE.to_string(),
                CONTINUITY_STALE_TURN_REPAIR_TYPE.to_string(),
                CONTINUITY_REFUSAL_RECEIPT_TYPE.to_string(),
            ],
            notes: vec![
                "Continuity is a deterministic protocol surface: compaction, sleep, recovery, stale-turn repair, and handoff packets enter here.".to_string(),
                "Continuity preserves what survives rupture; Mind decides which continuity material becomes durable state.".to_string(),
            ],
        },
        ContinuityCultNetContract {
            contract_id: "epiphany.continuity.compaction_checkpoint.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: CONTINUITY_COMPACTION_CHECKPOINT_TYPE.to_string(),
            payload_schema_version: CONTINUITY_COMPACTION_CHECKPOINT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Compaction checkpoints preserve the hot lesson before context rupture, not after the blackout has already eaten it.".to_string(),
            ],
        },
        ContinuityCultNetContract {
            contract_id: "epiphany.continuity.sleep_distillation.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: CONTINUITY_SLEEP_DISTILLATION_TYPE.to_string(),
            payload_schema_version: CONTINUITY_SLEEP_DISTILLATION_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Sleep distillation receipts separate durable lessons from rumination residue before Mind adoption.".to_string(),
            ],
        },
        ContinuityCultNetContract {
            contract_id: "epiphany.continuity.recovery.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: CONTINUITY_RECOVERY_RECEIPT_TYPE.to_string(),
            payload_schema_version: CONTINUITY_RECOVERY_RECEIPT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Recovery receipts explain what survived, what was rehydrated, and what must be regathered instead of imagined.".to_string(),
            ],
        },
        ContinuityCultNetContract {
            contract_id: "epiphany.continuity.stale_turn_repair.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: CONTINUITY_STALE_TURN_REPAIR_TYPE.to_string(),
            payload_schema_version: CONTINUITY_STALE_TURN_REPAIR_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Stale-turn repair receipts close or recover abandoned work without pretending the old turn completed cleanly.".to_string(),
            ],
        },
    ]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continuity_contracts_make_internal_verse_the_continuity_gate() {
        let contracts = default_continuity_cultnet_contracts();
        let continuity = contracts
            .iter()
            .find(|contract| contract.contract_id == "epiphany.continuity.review")
            .expect("continuity review contract");

        assert_eq!(continuity.verse_id, "epiphany-internal");
        assert_eq!(continuity.authority, "continuity");
        assert!(
            continuity
                .notes
                .iter()
                .any(|note| note.contains("deterministic protocol surface"))
        );
        assert!(
            continuity
                .receipt_document_types
                .contains(&CONTINUITY_RECOVERY_RECEIPT_TYPE.to_string())
        );
    }

    #[test]
    fn reorient_finding_builds_continuity_recovery_receipt() {
        let finding = EpiphanyReorientFindingInterpretation {
            mode: Some("resume".to_string()),
            summary: Some("Checkpoint remains valid.".to_string()),
            next_safe_move: Some("Continue bounded implementation.".to_string()),
            checkpoint_still_valid: Some(true),
            files_inspected: vec!["state/map.yaml".to_string()],
            frontier_node_ids: Vec::new(),
            evidence_ids: Vec::new(),
            artifact_refs: Vec::new(),
            runtime_result_id: Some("result-1".to_string()),
            runtime_job_id: Some("job-1".to_string()),
            job_error: None,
            item_error: None,
        };
        let receipt = continuity_recovery_receipt_from_reorient_finding(
            "continuity-recovery-1".to_string(),
            "reorientation-worker".to_string(),
            &finding,
            "2026-05-30T00:00:00Z".to_string(),
        );

        assert_eq!(receipt.source_result_id, "result-1");
        assert_eq!(receipt.source_job_id, "job-1");
        assert_eq!(receipt.mode, "resume");
        assert_eq!(receipt.checkpoint_still_valid, "true");
    }
}
