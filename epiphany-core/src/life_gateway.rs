use serde::Deserialize;
use serde::Serialize;

pub const LIFE_CONTINUITY_PACKET_TYPE: &str = "epiphany.life.continuity_packet";
pub const LIFE_COMPACTION_CHECKPOINT_TYPE: &str = "epiphany.life.compaction_checkpoint";
pub const LIFE_SLEEP_DISTILLATION_TYPE: &str = "epiphany.life.sleep_distillation";
pub const LIFE_RECOVERY_RECEIPT_TYPE: &str = "epiphany.life.recovery_receipt";
pub const LIFE_STALE_TURN_REPAIR_TYPE: &str = "epiphany.life.stale_turn_repair";
pub const LIFE_CONTINUITY_REFUSAL_RECEIPT_TYPE: &str = "epiphany.life.continuity_refusal_receipt";
pub const LIFE_CONTINUITY_PACKET_SCHEMA_VERSION: &str = "epiphany.life.continuity_packet.v0";
pub const LIFE_COMPACTION_CHECKPOINT_SCHEMA_VERSION: &str =
    "epiphany.life.compaction_checkpoint.v0";
pub const LIFE_SLEEP_DISTILLATION_SCHEMA_VERSION: &str = "epiphany.life.sleep_distillation.v0";
pub const LIFE_RECOVERY_RECEIPT_SCHEMA_VERSION: &str = "epiphany.life.recovery_receipt.v0";
pub const LIFE_STALE_TURN_REPAIR_SCHEMA_VERSION: &str = "epiphany.life.stale_turn_repair.v0";
pub const LIFE_CONTINUITY_REFUSAL_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.life.continuity_refusal_receipt.v0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifeCultNetContract {
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

pub fn default_life_cultnet_contracts() -> Vec<LifeCultNetContract> {
    vec![
        LifeCultNetContract {
            contract_id: "epiphany.life.continuity.review".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: LIFE_CONTINUITY_PACKET_TYPE.to_string(),
            payload_schema_version: LIFE_CONTINUITY_PACKET_SCHEMA_VERSION.to_string(),
            authority: "life".to_string(),
            operations: vec![
                "intentSubmit".to_string(),
                "receiptWatch".to_string(),
                "snapshot".to_string(),
            ],
            intent_document_types: vec![LIFE_CONTINUITY_PACKET_TYPE.to_string()],
            receipt_document_types: vec![
                LIFE_COMPACTION_CHECKPOINT_TYPE.to_string(),
                LIFE_SLEEP_DISTILLATION_TYPE.to_string(),
                LIFE_RECOVERY_RECEIPT_TYPE.to_string(),
                LIFE_STALE_TURN_REPAIR_TYPE.to_string(),
                LIFE_CONTINUITY_REFUSAL_RECEIPT_TYPE.to_string(),
            ],
            notes: vec![
                "Life is the continuity organ: compaction, sleep, recovery, stale-turn repair, and handoff packets enter here.".to_string(),
                "Life preserves what survives rupture; Mind decides which continuity material becomes durable state.".to_string(),
            ],
        },
        LifeCultNetContract {
            contract_id: "epiphany.life.compaction_checkpoint.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: LIFE_COMPACTION_CHECKPOINT_TYPE.to_string(),
            payload_schema_version: LIFE_COMPACTION_CHECKPOINT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Compaction checkpoints preserve the hot lesson before context rupture, not after the blackout has already eaten it.".to_string(),
            ],
        },
        LifeCultNetContract {
            contract_id: "epiphany.life.sleep_distillation.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: LIFE_SLEEP_DISTILLATION_TYPE.to_string(),
            payload_schema_version: LIFE_SLEEP_DISTILLATION_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Sleep distillation receipts separate durable lessons from rumination residue before Mind adoption.".to_string(),
            ],
        },
        LifeCultNetContract {
            contract_id: "epiphany.life.recovery.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: LIFE_RECOVERY_RECEIPT_TYPE.to_string(),
            payload_schema_version: LIFE_RECOVERY_RECEIPT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Recovery receipts explain what survived, what was rehydrated, and what must be regathered instead of imagined.".to_string(),
            ],
        },
        LifeCultNetContract {
            contract_id: "epiphany.life.stale_turn_repair.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: LIFE_STALE_TURN_REPAIR_TYPE.to_string(),
            payload_schema_version: LIFE_STALE_TURN_REPAIR_SCHEMA_VERSION.to_string(),
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
    fn life_contracts_make_internal_verse_the_continuity_gate() {
        let contracts = default_life_cultnet_contracts();
        let continuity = contracts
            .iter()
            .find(|contract| contract.contract_id == "epiphany.life.continuity.review")
            .expect("life continuity review contract");

        assert_eq!(continuity.verse_id, "epiphany-internal");
        assert_eq!(continuity.authority, "life");
        assert!(
            continuity
                .notes
                .iter()
                .any(|note| note.contains("continuity organ"))
        );
        assert!(
            continuity
                .receipt_document_types
                .contains(&LIFE_RECOVERY_RECEIPT_TYPE.to_string())
        );
    }
}
