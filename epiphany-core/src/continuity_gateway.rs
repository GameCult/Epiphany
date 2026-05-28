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
pub const CONTINUITY_SLEEP_DISTILLATION_SCHEMA_VERSION: &str = "epiphany.continuity.sleep_distillation.v0";
pub const CONTINUITY_RECOVERY_RECEIPT_SCHEMA_VERSION: &str = "epiphany.continuity.recovery_receipt.v0";
pub const CONTINUITY_STALE_TURN_REPAIR_SCHEMA_VERSION: &str = "epiphany.continuity.stale_turn_repair.v0";
pub const CONTINUITY_REFUSAL_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.continuity.refusal_receipt.v0";

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
}
