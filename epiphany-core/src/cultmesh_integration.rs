use crate::default_continuity_cultnet_contracts;
use crate::default_eyes_cultnet_contracts;
use crate::default_hands_cultnet_contracts;
use crate::default_mind_cultnet_contracts;
use crate::default_soul_cultnet_contracts;
use crate::default_substrate_gate_cultnet_contracts;
use anyhow::Result;
use anyhow::anyhow;
use cultcache_rs::DatabaseEntry;
use cultmesh_rs::CultMesh;
use cultmesh_rs::CultMeshNode;
use cultmesh_rs::CultMeshNodeOptions;
use cultmesh_rs::cultmesh_documents;
use serde_json::Value;
use std::path::Path;

pub const EPIPHANY_CULTMESH_STATUS_TYPE: &str = "epiphany.cultmesh.status";
pub const EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION: &str = "epiphany.cultmesh.status.v0";
pub const EPIPHANY_CULTMESH_STATUS_KEY: &str = "epiphany-local/status";
pub const EPIPHANY_CULTMESH_OPERATOR_STATUS_TYPE: &str = "epiphany.cultmesh.operator_status";
pub const EPIPHANY_CULTMESH_OPERATOR_STATUS_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.operator_status.v0";
pub const EPIPHANY_CULTMESH_OPERATOR_STATUS_KEY: &str = "epiphany-local/operator-status";
pub const EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_TYPE: &str = "epiphany.cultmesh.operator_snapshot";
pub const EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.operator_snapshot.v0";
pub const EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_LATEST_KEY: &str =
    "epiphany-local/operator-snapshot/latest";
pub const EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_TYPE: &str =
    "epiphany.cultmesh.operator_run_intent";
pub const EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.operator_run_intent.v0";
pub const EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_LATEST_KEY: &str =
    "epiphany-local/operator-run-intent/latest";
pub const EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_TYPE: &str =
    "epiphany.cultmesh.operator_run_receipt";
pub const EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.operator_run_receipt.v0";
pub const EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_LATEST_KEY: &str =
    "epiphany-local/operator-run-receipt/latest";
pub const EPIPHANY_CULTMESH_VERSE_POLICY_TYPE: &str = "epiphany.cultmesh.verse_policy";
pub const EPIPHANY_CULTMESH_VERSE_POLICY_SCHEMA_VERSION: &str = "epiphany.cultmesh.verse_policy.v0";
pub const EPIPHANY_CULTMESH_GLOBAL_ROOM_POLICY_TYPE: &str = "epiphany.cultmesh.global_room_policy";
pub const EPIPHANY_CULTMESH_GLOBAL_ROOM_POLICY_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.global_room_policy.v0";
pub const EPIPHANY_CULTMESH_MIND_CONTRACT_TYPE: &str = "epiphany.cultmesh.mind_contract";
pub const EPIPHANY_CULTMESH_MIND_CONTRACT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.mind_contract.v0";
pub const EPIPHANY_CULTMESH_SUBSTRATE_GATE_CONTRACT_TYPE: &str =
    "epiphany.cultmesh.substrate_gate_contract";
pub const EPIPHANY_CULTMESH_SUBSTRATE_GATE_CONTRACT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.substrate_gate_contract.v0";
pub const EPIPHANY_CULTMESH_EYES_CONTRACT_TYPE: &str = "epiphany.cultmesh.eyes_contract";
pub const EPIPHANY_CULTMESH_EYES_CONTRACT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.eyes_contract.v0";
pub const EPIPHANY_CULTMESH_HANDS_CONTRACT_TYPE: &str = "epiphany.cultmesh.hands_contract";
pub const EPIPHANY_CULTMESH_HANDS_CONTRACT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.hands_contract.v0";
pub const EPIPHANY_CULTMESH_SOUL_CONTRACT_TYPE: &str = "epiphany.cultmesh.soul_contract";
pub const EPIPHANY_CULTMESH_SOUL_CONTRACT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.soul_contract.v0";
pub const EPIPHANY_CULTMESH_CONTINUITY_CONTRACT_TYPE: &str =
    "epiphany.cultmesh.continuity_contract";
pub const EPIPHANY_CULTMESH_CONTINUITY_CONTRACT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.continuity_contract.v0";
pub const EPIPHANY_CULTMESH_INTERNAL_VERSE_ID: &str = "epiphany-internal";
pub const EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID: &str = "gamecult-local";
pub const EPIPHANY_CULTMESH_GLOBAL_VERSE_ID: &str = "epiphany-global";
pub const EPIPHANY_CULTMESH_INTERNAL_TIER: &str = "internal";
pub const EPIPHANY_CULTMESH_LOCAL_AREA_TIER: &str = "local-area";
pub const EPIPHANY_CULTMESH_GLOBAL_TIER: &str = "global";

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.status",
    schema = "EpiphanyCultMeshStatusEntry"
)]
pub struct EpiphanyCultMeshStatusEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub app_id: String,
    #[cultcache(key = 4)]
    pub note: String,
    #[cultcache(key = 5, default)]
    pub verse_tier: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.operator_status",
    schema = "EpiphanyCultMeshOperatorStatusEntry"
)]
pub struct EpiphanyCultMeshOperatorStatusEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub surface_id: String,
    #[cultcache(key = 4)]
    pub status: String,
    #[cultcache(key = 5)]
    pub generated_at_utc: String,
    #[cultcache(key = 6)]
    pub summary: String,
    #[cultcache(key = 7)]
    pub codex_bridge_role: String,
    #[cultcache(key = 8)]
    pub epiphany_authority_role: String,
    #[cultcache(key = 9)]
    pub prompt_authority: String,
    #[cultcache(key = 10)]
    pub native_authorities: Vec<String>,
    #[cultcache(key = 11)]
    pub quarantined_surfaces: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.operator_snapshot",
    schema = "EpiphanyCultMeshOperatorSnapshotEntry"
)]
pub struct EpiphanyCultMeshOperatorSnapshotEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub snapshot_id: String,
    #[cultcache(key = 4)]
    pub generated_at_utc: String,
    #[cultcache(key = 5)]
    pub source_mode: String,
    #[cultcache(key = 6)]
    pub source_path: String,
    #[cultcache(key = 7)]
    pub thread_id: String,
    #[cultcache(key = 8)]
    pub status: String,
    #[cultcache(key = 9)]
    pub state_status: String,
    #[cultcache(key = 10)]
    pub coordinator_action: String,
    #[cultcache(key = 11)]
    pub crrc_action: String,
    #[cultcache(key = 12)]
    pub pressure_level: String,
    #[cultcache(key = 13)]
    pub reorient_action: String,
    #[cultcache(key = 14)]
    pub next_action: String,
    #[cultcache(key = 15)]
    pub artifact_refs: Vec<String>,
    #[cultcache(key = 16)]
    pub available_actions: Vec<String>,
    #[cultcache(key = 17)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.operator_run_intent",
    schema = "EpiphanyCultMeshOperatorRunIntentEntry"
)]
pub struct EpiphanyCultMeshOperatorRunIntentEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub run_id: String,
    #[cultcache(key = 4)]
    pub requested_at_utc: String,
    #[cultcache(key = 5)]
    pub mode: String,
    #[cultcache(key = 6)]
    pub root: String,
    #[cultcache(key = 7)]
    pub workspace: String,
    #[cultcache(key = 8)]
    pub thread_id: String,
    #[cultcache(key = 9)]
    pub codex_home: String,
    #[cultcache(key = 10)]
    pub target_dir: String,
    #[cultcache(key = 11)]
    pub max_steps: u32,
    #[cultcache(key = 12)]
    pub timeout_seconds: u32,
    #[cultcache(key = 13)]
    pub auto_review: bool,
    #[cultcache(key = 14)]
    pub no_ephemeral: bool,
    #[cultcache(key = 15)]
    pub artifact_root: String,
    #[cultcache(key = 16)]
    pub dogfood_root: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.operator_run_receipt",
    schema = "EpiphanyCultMeshOperatorRunReceiptEntry"
)]
pub struct EpiphanyCultMeshOperatorRunReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub run_id: String,
    #[cultcache(key = 4)]
    pub completed_at_utc: String,
    #[cultcache(key = 5)]
    pub mode: String,
    #[cultcache(key = 6)]
    pub status: String,
    #[cultcache(key = 7)]
    pub result_path: String,
    #[cultcache(key = 8)]
    pub artifact_root: String,
    #[cultcache(key = 9)]
    pub dogfood_root: String,
    #[cultcache(key = 10)]
    pub operator_snapshot_store: String,
    #[cultcache(key = 11)]
    pub operator_snapshot_id: String,
    #[cultcache(key = 12)]
    pub artifact_refs: Vec<String>,
    #[cultcache(key = 13)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.verse_policy",
    schema = "EpiphanyCultMeshVersePolicyEntry"
)]
pub struct EpiphanyCultMeshVersePolicyEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub verse_id: String,
    #[cultcache(key = 2)]
    pub tier: String,
    #[cultcache(key = 3)]
    pub purpose: String,
    #[cultcache(key = 4)]
    pub transport_scope: String,
    #[cultcache(key = 5)]
    pub trust_boundary: String,
    #[cultcache(key = 6)]
    pub private_state_allowed: bool,
    #[cultcache(key = 7)]
    pub untrusted_ingress_allowed: bool,
    #[cultcache(key = 8)]
    pub yggdrasil_tunnel_allowed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.global_room_policy",
    schema = "EpiphanyCultMeshGlobalRoomPolicyEntry"
)]
pub struct EpiphanyCultMeshGlobalRoomPolicyEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub room_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub topic: String,
    #[cultcache(key = 4)]
    pub purpose: String,
    #[cultcache(key = 5)]
    pub posting_policy: String,
    #[cultcache(key = 6)]
    pub threaded: bool,
    #[cultcache(key = 7)]
    pub face_posting_allowed: bool,
    #[cultcache(key = 8)]
    pub untrusted_ingress_allowed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.mind_contract",
    schema = "EpiphanyCultMeshMindContractEntry"
)]
pub struct EpiphanyCultMeshMindContractEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub contract_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub document_type: String,
    #[cultcache(key = 4)]
    pub payload_schema_version: String,
    #[cultcache(key = 5)]
    pub authority: String,
    #[cultcache(key = 6)]
    pub operations: Vec<String>,
    #[cultcache(key = 7)]
    pub intent_document_types: Vec<String>,
    #[cultcache(key = 8)]
    pub receipt_document_types: Vec<String>,
    #[cultcache(key = 9)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.substrate_gate_contract",
    schema = "EpiphanyCultMeshSubstrateGateContractEntry"
)]
pub struct EpiphanyCultMeshSubstrateGateContractEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub contract_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub document_type: String,
    #[cultcache(key = 4)]
    pub payload_schema_version: String,
    #[cultcache(key = 5)]
    pub authority: String,
    #[cultcache(key = 6)]
    pub operations: Vec<String>,
    #[cultcache(key = 7)]
    pub intent_document_types: Vec<String>,
    #[cultcache(key = 8)]
    pub receipt_document_types: Vec<String>,
    #[cultcache(key = 9)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.eyes_contract",
    schema = "EpiphanyCultMeshEyesContractEntry"
)]
pub struct EpiphanyCultMeshEyesContractEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub contract_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub document_type: String,
    #[cultcache(key = 4)]
    pub payload_schema_version: String,
    #[cultcache(key = 5)]
    pub authority: String,
    #[cultcache(key = 6)]
    pub operations: Vec<String>,
    #[cultcache(key = 7)]
    pub intent_document_types: Vec<String>,
    #[cultcache(key = 8)]
    pub receipt_document_types: Vec<String>,
    #[cultcache(key = 9)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.hands_contract",
    schema = "EpiphanyCultMeshHandsContractEntry"
)]
pub struct EpiphanyCultMeshHandsContractEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub contract_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub document_type: String,
    #[cultcache(key = 4)]
    pub payload_schema_version: String,
    #[cultcache(key = 5)]
    pub authority: String,
    #[cultcache(key = 6)]
    pub operations: Vec<String>,
    #[cultcache(key = 7)]
    pub intent_document_types: Vec<String>,
    #[cultcache(key = 8)]
    pub receipt_document_types: Vec<String>,
    #[cultcache(key = 9)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.soul_contract",
    schema = "EpiphanyCultMeshSoulContractEntry"
)]
pub struct EpiphanyCultMeshSoulContractEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub contract_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub document_type: String,
    #[cultcache(key = 4)]
    pub payload_schema_version: String,
    #[cultcache(key = 5)]
    pub authority: String,
    #[cultcache(key = 6)]
    pub operations: Vec<String>,
    #[cultcache(key = 7)]
    pub intent_document_types: Vec<String>,
    #[cultcache(key = 8)]
    pub receipt_document_types: Vec<String>,
    #[cultcache(key = 9)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.continuity_contract",
    schema = "EpiphanyCultMeshContinuityContractEntry"
)]
pub struct EpiphanyCultMeshContinuityContractEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub contract_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub document_type: String,
    #[cultcache(key = 4)]
    pub payload_schema_version: String,
    #[cultcache(key = 5)]
    pub authority: String,
    #[cultcache(key = 6)]
    pub operations: Vec<String>,
    #[cultcache(key = 7)]
    pub intent_document_types: Vec<String>,
    #[cultcache(key = 8)]
    pub receipt_document_types: Vec<String>,
    #[cultcache(key = 9)]
    pub notes: Vec<String>,
}

cultmesh_documents!(EpiphanyCultMeshDocuments {
    EpiphanyCultMeshStatusEntry => EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION,
    EpiphanyCultMeshOperatorStatusEntry => EPIPHANY_CULTMESH_OPERATOR_STATUS_SCHEMA_VERSION,
    EpiphanyCultMeshOperatorSnapshotEntry => EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_SCHEMA_VERSION,
    EpiphanyCultMeshOperatorRunIntentEntry => EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION,
    EpiphanyCultMeshOperatorRunReceiptEntry => EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshVersePolicyEntry => EPIPHANY_CULTMESH_VERSE_POLICY_SCHEMA_VERSION,
    EpiphanyCultMeshGlobalRoomPolicyEntry => EPIPHANY_CULTMESH_GLOBAL_ROOM_POLICY_SCHEMA_VERSION,
    EpiphanyCultMeshMindContractEntry => EPIPHANY_CULTMESH_MIND_CONTRACT_SCHEMA_VERSION,
    EpiphanyCultMeshSubstrateGateContractEntry => EPIPHANY_CULTMESH_SUBSTRATE_GATE_CONTRACT_SCHEMA_VERSION,
    EpiphanyCultMeshEyesContractEntry => EPIPHANY_CULTMESH_EYES_CONTRACT_SCHEMA_VERSION,
    EpiphanyCultMeshHandsContractEntry => EPIPHANY_CULTMESH_HANDS_CONTRACT_SCHEMA_VERSION,
    EpiphanyCultMeshSoulContractEntry => EPIPHANY_CULTMESH_SOUL_CONTRACT_SCHEMA_VERSION,
    EpiphanyCultMeshContinuityContractEntry => EPIPHANY_CULTMESH_CONTINUITY_CONTRACT_SCHEMA_VERSION,
});

pub fn open_epiphany_cultmesh_node(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<CultMeshNode> {
    CultMesh::create_node(
        store_path,
        EpiphanyCultMeshDocuments,
        CultMeshNodeOptions {
            runtime_id: runtime_id.into(),
            ..CultMeshNodeOptions::default()
        },
    )
}

pub fn write_epiphany_cultmesh_status(
    store_path: impl AsRef<Path>,
    status: EpiphanyCultMeshStatusEntry,
) -> Result<EpiphanyCultMeshStatusEntry> {
    let mut node = open_epiphany_cultmesh_node(&store_path, status.runtime_id.clone())?;
    let written = node.put(EPIPHANY_CULTMESH_STATUS_KEY, &status)?;
    node.flush()?;
    Ok(written)
}

pub fn load_epiphany_cultmesh_status(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshStatusEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_STATUS_KEY)
}

pub fn default_epiphany_cultmesh_operator_status(
    runtime_id: impl Into<String>,
    generated_at_utc: impl Into<String>,
) -> EpiphanyCultMeshOperatorStatusEntry {
    EpiphanyCultMeshOperatorStatusEntry {
        schema_version: EPIPHANY_CULTMESH_OPERATOR_STATUS_SCHEMA_VERSION.to_string(),
        runtime_id: runtime_id.into(),
        verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
        surface_id: "epiphany.operator.status".to_string(),
        status: "ready".to_string(),
        generated_at_utc: generated_at_utc.into(),
        summary:
            "Epiphany operator status is native typed state; Codex is bridge transport, not policy owner."
                .to_string(),
        codex_bridge_role:
            "relatively vanilla Codex may provide OpenAI auth/model transport, streaming, and Codex-native app-server affordances."
                .to_string(),
        epiphany_authority_role:
            "Epiphany owns state, processes, prompts, scheduler decisions, organ contracts, and mutation law."
                .to_string(),
        prompt_authority:
            "Codex prompt machinery must not inject doctrine, role instructions, state law, or coordinator policy into Epiphany agents."
                .to_string(),
        native_authorities: vec![
            "CultCache typed documents".to_string(),
            "CultMesh local Verse store".to_string(),
            "CultNet read/mutation/event contracts".to_string(),
            "Epiphany organ receipt gates".to_string(),
        ],
        quarantined_surfaces: vec![
            "Rider bridge".to_string(),
            "Unity bridge".to_string(),
        ],
    }
}

pub fn write_epiphany_cultmesh_operator_status(
    store_path: impl AsRef<Path>,
    status: EpiphanyCultMeshOperatorStatusEntry,
) -> Result<EpiphanyCultMeshOperatorStatusEntry> {
    let mut node = open_epiphany_cultmesh_node(&store_path, status.runtime_id.clone())?;
    let written = node.put(EPIPHANY_CULTMESH_OPERATOR_STATUS_KEY, &status)?;
    node.flush()?;
    Ok(written)
}

pub fn load_epiphany_cultmesh_operator_status(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshOperatorStatusEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_OPERATOR_STATUS_KEY)
}

pub fn epiphany_cultmesh_operator_snapshot_from_status_json(
    runtime_id: impl Into<String>,
    snapshot_id: impl Into<String>,
    generated_at_utc: impl Into<String>,
    source_mode: impl Into<String>,
    source_path: impl Into<String>,
    status_json: &Value,
) -> Result<EpiphanyCultMeshOperatorSnapshotEntry> {
    let source_path = source_path.into();
    let state_status = pointer_text(status_json, "/scene/scene/stateStatus", "unknown");
    let crrc_action = pointer_text(status_json, "/crrc/recommendation/action", "unknown");
    let reorient_action = pointer_text(status_json, "/reorient/decision/action", "unknown");
    let operator_status = if state_status == "missing" || crrc_action == "regatherManually" {
        "needs-regather"
    } else {
        "ready"
    };
    let mut artifact_refs = Vec::new();
    if !source_path.trim().is_empty() {
        artifact_refs.push(source_path.clone());
    }

    Ok(EpiphanyCultMeshOperatorSnapshotEntry {
        schema_version: EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_SCHEMA_VERSION.to_string(),
        runtime_id: runtime_id.into(),
        verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
        snapshot_id: snapshot_id.into(),
        generated_at_utc: generated_at_utc.into(),
        source_mode: source_mode.into(),
        source_path,
        thread_id: pointer_text(status_json, "/threadId", "missing"),
        status: operator_status.to_string(),
        state_status,
        coordinator_action: pointer_text(status_json, "/coordinator/action", "none"),
        crrc_action,
        pressure_level: pointer_text(status_json, "/pressure/pressure/level", "unknown"),
        reorient_action,
        next_action: pointer_text(status_json, "/reorient/decision/nextAction", "none"),
        artifact_refs,
        available_actions: pointer_string_array(status_json, "/scene/scene/availableActions")?,
        notes: vec![
            "Snapshot is derived from the operator-safe MVP status artifact; raw JSON remains an edge artifact, not internal state.".to_string(),
            "Codex app-server remains compatibility transport for this source until the status surface is native end to end.".to_string(),
        ],
    })
}

pub fn write_epiphany_cultmesh_operator_snapshot(
    store_path: impl AsRef<Path>,
    snapshot: EpiphanyCultMeshOperatorSnapshotEntry,
) -> Result<EpiphanyCultMeshOperatorSnapshotEntry> {
    let mut node = open_epiphany_cultmesh_node(&store_path, snapshot.runtime_id.clone())?;
    let snapshot_key = epiphany_cultmesh_operator_snapshot_key(&snapshot.snapshot_id);
    let written = node.put(snapshot_key.as_str(), &snapshot)?;
    node.put(EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_LATEST_KEY, &written)?;
    node.flush()?;
    Ok(written)
}

pub fn load_epiphany_cultmesh_operator_snapshot(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    snapshot_id: impl AsRef<str>,
) -> Result<Option<EpiphanyCultMeshOperatorSnapshotEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let snapshot_key = epiphany_cultmesh_operator_snapshot_key(snapshot_id.as_ref());
    node.get(snapshot_key.as_str())
}

pub fn load_latest_epiphany_cultmesh_operator_snapshot(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshOperatorSnapshotEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_LATEST_KEY)
}

pub fn write_epiphany_cultmesh_operator_run_intent(
    store_path: impl AsRef<Path>,
    intent: EpiphanyCultMeshOperatorRunIntentEntry,
) -> Result<EpiphanyCultMeshOperatorRunIntentEntry> {
    let mut node = open_epiphany_cultmesh_node(&store_path, intent.runtime_id.clone())?;
    let intent_key = epiphany_cultmesh_operator_run_intent_key(&intent.run_id);
    let written = node.put(intent_key.as_str(), &intent)?;
    node.put(EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_LATEST_KEY, &written)?;
    node.flush()?;
    Ok(written)
}

pub fn load_latest_epiphany_cultmesh_operator_run_intent(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshOperatorRunIntentEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_LATEST_KEY)
}

pub fn write_epiphany_cultmesh_operator_run_receipt(
    store_path: impl AsRef<Path>,
    receipt: EpiphanyCultMeshOperatorRunReceiptEntry,
) -> Result<EpiphanyCultMeshOperatorRunReceiptEntry> {
    let mut node = open_epiphany_cultmesh_node(&store_path, receipt.runtime_id.clone())?;
    let receipt_key = epiphany_cultmesh_operator_run_receipt_key(&receipt.run_id);
    let written = node.put(receipt_key.as_str(), &receipt)?;
    node.put(EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_LATEST_KEY, &written)?;
    node.flush()?;
    Ok(written)
}

pub fn load_latest_epiphany_cultmesh_operator_run_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshOperatorRunReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_LATEST_KEY)
}

fn epiphany_cultmesh_operator_snapshot_key(snapshot_id: &str) -> String {
    format!("epiphany-local/operator-snapshot/{snapshot_id}")
}

fn epiphany_cultmesh_operator_run_intent_key(run_id: &str) -> String {
    format!("epiphany-local/operator-run-intent/{run_id}")
}

fn epiphany_cultmesh_operator_run_receipt_key(run_id: &str) -> String {
    format!("epiphany-local/operator-run-receipt/{run_id}")
}

fn pointer_text(value: &Value, pointer: &str, fallback: &str) -> String {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn pointer_string_array(value: &Value, pointer: &str) -> Result<Vec<String>> {
    let Some(items) = value.pointer(pointer) else {
        return Ok(Vec::new());
    };
    let items = items
        .as_array()
        .ok_or_else(|| anyhow!("{pointer} must be an array when present"))?;
    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToString::to_string)
                .ok_or_else(|| anyhow!("{pointer} must contain only strings"))
        })
        .collect()
}

pub fn epiphany_cultmesh_verse_policies() -> Vec<EpiphanyCultMeshVersePolicyEntry> {
    vec![
        EpiphanyCultMeshVersePolicyEntry {
            schema_version: EPIPHANY_CULTMESH_VERSE_POLICY_SCHEMA_VERSION.to_string(),
            verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
            tier: EPIPHANY_CULTMESH_INTERNAL_TIER.to_string(),
            purpose: "Sub-agent typed state: heartbeat, organ-state records, runtime-spine jobs, private receipts, and other Epiphany-owned organs.".to_string(),
            transport_scope: "single-host or trusted localhost mesh".to_string(),
            trust_boundary: "private Epiphany instance boundary".to_string(),
            private_state_allowed: true,
            untrusted_ingress_allowed: false,
            yggdrasil_tunnel_allowed: false,
        },
        EpiphanyCultMeshVersePolicyEntry {
            schema_version: EPIPHANY_CULTMESH_VERSE_POLICY_SCHEMA_VERSION.to_string(),
            verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
            tier: EPIPHANY_CULTMESH_LOCAL_AREA_TIER.to_string(),
            purpose: "Trusted GameCult local-area sharing across projects, including operator-approved tunnels to services on Yggdrasil.".to_string(),
            transport_scope: "LAN plus explicit GameCult tunnel endpoints".to_string(),
            trust_boundary: "trusted GameCult project/runtime boundary".to_string(),
            private_state_allowed: false,
            untrusted_ingress_allowed: false,
            yggdrasil_tunnel_allowed: true,
        },
        EpiphanyCultMeshVersePolicyEntry {
            schema_version: EPIPHANY_CULTMESH_VERSE_POLICY_SCHEMA_VERSION.to_string(),
            verse_id: EPIPHANY_CULTMESH_GLOBAL_VERSE_ID.to_string(),
            tier: EPIPHANY_CULTMESH_GLOBAL_TIER.to_string(),
            purpose: "Untrusted public surfaces: public dreams, questions, hypotheses, invitations, lineage, ingress receipts, and adoption receipts.".to_string(),
            transport_scope: "public internet".to_string(),
            trust_boundary: "untrusted public boundary".to_string(),
            private_state_allowed: false,
            untrusted_ingress_allowed: true,
            yggdrasil_tunnel_allowed: false,
        },
    ]
}

pub fn write_epiphany_cultmesh_verse_policies(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshVersePolicyEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for policy in epiphany_cultmesh_verse_policies() {
        written.push(node.put(policy.verse_id.clone(), &policy)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn epiphany_cultmesh_global_room_policies() -> Vec<EpiphanyCultMeshGlobalRoomPolicyEntry> {
    [
        (
            "dreams",
            "Dreams",
            "Public dreams, symbolic fragments, imaginative pressure, and unfinished possible worlds.",
        ),
        (
            "architecture",
            "Architecture",
            "System design, ownership maps, protocol boundaries, and rejected machine shapes.",
        ),
        (
            "research",
            "Research",
            "Prior art, papers, source-grounded findings, and scout reports.",
        ),
        (
            "faces",
            "Faces",
            "Public Face identity, voice, social surface, and community-facing presence.",
        ),
        (
            "gamecult",
            "GameCult",
            "GameCult project coordination, public receipts, and cross-project questions.",
        ),
        (
            "governance",
            "Governance",
            "Public proposals and governance-adjacent discussion before any Bifrost adoption.",
        ),
    ]
    .into_iter()
    .map(|(slug, topic, purpose)| EpiphanyCultMeshGlobalRoomPolicyEntry {
        schema_version: EPIPHANY_CULTMESH_GLOBAL_ROOM_POLICY_SCHEMA_VERSION.to_string(),
        room_id: format!("epiphany-global/{slug}"),
        verse_id: EPIPHANY_CULTMESH_GLOBAL_VERSE_ID.to_string(),
        topic: topic.to_string(),
        purpose: purpose.to_string(),
        posting_policy:
            "Faces may post public, non-private, citation/provenance-bearing thread roots and replies; local adoption still requires review."
                .to_string(),
        threaded: true,
        face_posting_allowed: true,
        untrusted_ingress_allowed: true,
    })
    .collect()
}

pub fn write_epiphany_cultmesh_global_room_policies(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshGlobalRoomPolicyEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for room in epiphany_cultmesh_global_room_policies() {
        written.push(node.put(room.room_id.clone(), &room)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn epiphany_cultmesh_mind_contracts() -> Vec<EpiphanyCultMeshMindContractEntry> {
    default_mind_cultnet_contracts()
        .into_iter()
        .map(|contract| EpiphanyCultMeshMindContractEntry {
            schema_version: EPIPHANY_CULTMESH_MIND_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id: contract.contract_id,
            verse_id: contract.verse_id,
            document_type: contract.document_type,
            payload_schema_version: contract.payload_schema_version,
            authority: contract.authority,
            operations: contract.operations,
            intent_document_types: contract.intent_document_types,
            receipt_document_types: contract.receipt_document_types,
            notes: contract.notes,
        })
        .collect()
}

pub fn write_epiphany_cultmesh_mind_contracts(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshMindContractEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for contract in epiphany_cultmesh_mind_contracts() {
        written.push(node.put(contract.contract_id.clone(), &contract)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn epiphany_cultmesh_substrate_gate_contracts()
-> Vec<EpiphanyCultMeshSubstrateGateContractEntry> {
    default_substrate_gate_cultnet_contracts()
        .into_iter()
        .map(|contract| EpiphanyCultMeshSubstrateGateContractEntry {
            schema_version: EPIPHANY_CULTMESH_SUBSTRATE_GATE_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id: contract.contract_id,
            verse_id: contract.verse_id,
            document_type: contract.document_type,
            payload_schema_version: contract.payload_schema_version,
            authority: contract.authority,
            operations: contract.operations,
            intent_document_types: contract.intent_document_types,
            receipt_document_types: contract.receipt_document_types,
            notes: contract.notes,
        })
        .collect()
}

pub fn write_epiphany_cultmesh_substrate_gate_contracts(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshSubstrateGateContractEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for contract in epiphany_cultmesh_substrate_gate_contracts() {
        written.push(node.put(contract.contract_id.clone(), &contract)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn epiphany_cultmesh_eyes_contracts() -> Vec<EpiphanyCultMeshEyesContractEntry> {
    default_eyes_cultnet_contracts()
        .into_iter()
        .map(|contract| EpiphanyCultMeshEyesContractEntry {
            schema_version: EPIPHANY_CULTMESH_EYES_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id: contract.contract_id,
            verse_id: contract.verse_id,
            document_type: contract.document_type,
            payload_schema_version: contract.payload_schema_version,
            authority: contract.authority,
            operations: contract.operations,
            intent_document_types: contract.intent_document_types,
            receipt_document_types: contract.receipt_document_types,
            notes: contract.notes,
        })
        .collect()
}

pub fn write_epiphany_cultmesh_eyes_contracts(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshEyesContractEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for contract in epiphany_cultmesh_eyes_contracts() {
        written.push(node.put(contract.contract_id.clone(), &contract)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn epiphany_cultmesh_hands_contracts() -> Vec<EpiphanyCultMeshHandsContractEntry> {
    default_hands_cultnet_contracts()
        .into_iter()
        .map(|contract| EpiphanyCultMeshHandsContractEntry {
            schema_version: EPIPHANY_CULTMESH_HANDS_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id: contract.contract_id,
            verse_id: contract.verse_id,
            document_type: contract.document_type,
            payload_schema_version: contract.payload_schema_version,
            authority: contract.authority,
            operations: contract.operations,
            intent_document_types: contract.intent_document_types,
            receipt_document_types: contract.receipt_document_types,
            notes: contract.notes,
        })
        .collect()
}

pub fn write_epiphany_cultmesh_hands_contracts(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshHandsContractEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for contract in epiphany_cultmesh_hands_contracts() {
        written.push(node.put(contract.contract_id.clone(), &contract)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn epiphany_cultmesh_soul_contracts() -> Vec<EpiphanyCultMeshSoulContractEntry> {
    default_soul_cultnet_contracts()
        .into_iter()
        .map(|contract| EpiphanyCultMeshSoulContractEntry {
            schema_version: EPIPHANY_CULTMESH_SOUL_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id: contract.contract_id,
            verse_id: contract.verse_id,
            document_type: contract.document_type,
            payload_schema_version: contract.payload_schema_version,
            authority: contract.authority,
            operations: contract.operations,
            intent_document_types: contract.intent_document_types,
            receipt_document_types: contract.receipt_document_types,
            notes: contract.notes,
        })
        .collect()
}

pub fn write_epiphany_cultmesh_soul_contracts(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshSoulContractEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for contract in epiphany_cultmesh_soul_contracts() {
        written.push(node.put(contract.contract_id.clone(), &contract)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn epiphany_cultmesh_continuity_contracts() -> Vec<EpiphanyCultMeshContinuityContractEntry> {
    default_continuity_cultnet_contracts()
        .into_iter()
        .map(|contract| EpiphanyCultMeshContinuityContractEntry {
            schema_version: EPIPHANY_CULTMESH_CONTINUITY_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id: contract.contract_id,
            verse_id: contract.verse_id,
            document_type: contract.document_type,
            payload_schema_version: contract.payload_schema_version,
            authority: contract.authority,
            operations: contract.operations,
            intent_document_types: contract.intent_document_types,
            receipt_document_types: contract.receipt_document_types,
            notes: contract.notes,
        })
        .collect()
}

pub fn write_epiphany_cultmesh_continuity_contracts(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshContinuityContractEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for contract in epiphany_cultmesh_continuity_contracts() {
        written.push(node.put(contract.contract_id.clone(), &contract)?);
    }
    node.flush()?;
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn epiphany_status_round_trips_through_cultmesh() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-local.ccmp");
        let status = EpiphanyCultMeshStatusEntry {
            schema_version: EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION.to_string(),
            runtime_id: "epiphany-test".to_string(),
            verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
            verse_tier: EPIPHANY_CULTMESH_INTERNAL_TIER.to_string(),
            app_id: "epiphany".to_string(),
            note: "CultMesh is the local abstraction over CultCache and CultNet.".to_string(),
        };

        write_epiphany_cultmesh_status(&store, status.clone())?;
        assert_eq!(
            load_epiphany_cultmesh_status(&store, "epiphany-test")?,
            Some(status)
        );
        Ok(())
    }

    #[test]
    fn operator_status_round_trips_as_native_cultmesh_document() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-operator-status.ccmp");
        let status =
            default_epiphany_cultmesh_operator_status("epiphany-test", "2026-05-27T00:00:00Z");

        write_epiphany_cultmesh_operator_status(&store, status.clone())?;
        let loaded = load_epiphany_cultmesh_operator_status(&store, "epiphany-test")?;

        assert_eq!(loaded, Some(status));
        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        assert!(
            node.documents()
                .binding(EPIPHANY_CULTMESH_OPERATOR_STATUS_TYPE)
                .is_some()
        );
        Ok(())
    }

    #[test]
    fn operator_snapshot_distills_status_json_into_typed_cultmesh_document() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-operator-snapshot.ccmp");
        let status_json = serde_json::json!({
            "threadId": "thread-test",
            "scene": {
                "scene": {
                    "stateStatus": "missing",
                    "availableActions": ["crrc", "roles"]
                }
            },
            "pressure": {
                "pressure": {
                    "level": "low"
                }
            },
            "reorient": {
                "decision": {
                    "action": "regather",
                    "nextAction": "Regather source context."
                }
            },
            "crrc": {
                "recommendation": {
                    "action": "regatherManually"
                }
            },
            "coordinator": {
                "action": "wait"
            },
            "rawResult": {
                "sealed": true
            }
        });
        let snapshot = epiphany_cultmesh_operator_snapshot_from_status_json(
            "epiphany-test",
            "snapshot-test",
            "2026-05-27T00:00:00Z",
            "status",
            ".epiphany-run/status.json",
            &status_json,
        )?;

        assert_eq!(snapshot.status, "needs-regather");
        assert_eq!(snapshot.thread_id, "thread-test");
        assert_eq!(snapshot.available_actions, vec!["crrc", "roles"]);
        assert_eq!(snapshot.artifact_refs, vec![".epiphany-run/status.json"]);

        write_epiphany_cultmesh_operator_snapshot(&store, snapshot.clone())?;
        assert_eq!(
            load_epiphany_cultmesh_operator_snapshot(&store, "epiphany-test", "snapshot-test")?,
            Some(snapshot.clone())
        );
        assert_eq!(
            load_latest_epiphany_cultmesh_operator_snapshot(&store, "epiphany-test")?,
            Some(snapshot)
        );
        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        assert!(
            node.documents()
                .binding(EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_TYPE)
                .is_some()
        );
        Ok(())
    }

    #[test]
    fn operator_run_intent_and_receipt_round_trip_as_native_cultmesh_documents() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-operator-run.ccmp");
        let intent = EpiphanyCultMeshOperatorRunIntentEntry {
            schema_version: EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION.to_string(),
            runtime_id: "epiphany-test".to_string(),
            verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
            run_id: "run-test".to_string(),
            requested_at_utc: "2026-05-27T00:00:00Z".to_string(),
            mode: "status".to_string(),
            root: "E:\\Projects\\EpiphanyAgent".to_string(),
            workspace: "E:\\Projects\\EpiphanyAgent".to_string(),
            thread_id: String::new(),
            codex_home: "C:\\Users\\Meta\\.codex".to_string(),
            target_dir: "C:\\Users\\Meta\\.cargo-target-codex".to_string(),
            max_steps: 4,
            timeout_seconds: 240,
            auto_review: false,
            no_ephemeral: false,
            artifact_root: ".epiphany-run/run-test".to_string(),
            dogfood_root: ".epiphany-dogfood/run-test".to_string(),
        };
        let receipt = EpiphanyCultMeshOperatorRunReceiptEntry {
            schema_version: EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION.to_string(),
            runtime_id: "epiphany-test".to_string(),
            verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
            run_id: "run-test".to_string(),
            completed_at_utc: "2026-05-27T00:00:01Z".to_string(),
            mode: "status".to_string(),
            status: "completed".to_string(),
            result_path: ".epiphany-run/run-test/status.json".to_string(),
            artifact_root: ".epiphany-run/run-test".to_string(),
            dogfood_root: ".epiphany-dogfood/run-test".to_string(),
            operator_snapshot_store: ".epiphany-run/cultmesh/operator-snapshots.ccmp".to_string(),
            operator_snapshot_id: "run-test-status".to_string(),
            artifact_refs: vec![".epiphany-run/run-test/status.json".to_string()],
            notes: vec!["receipt".to_string()],
        };

        write_epiphany_cultmesh_operator_run_intent(&store, intent.clone())?;
        write_epiphany_cultmesh_operator_run_receipt(&store, receipt.clone())?;

        assert_eq!(
            load_latest_epiphany_cultmesh_operator_run_intent(&store, "epiphany-test")?,
            Some(intent)
        );
        assert_eq!(
            load_latest_epiphany_cultmesh_operator_run_receipt(&store, "epiphany-test")?,
            Some(receipt)
        );
        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        assert!(
            node.documents()
                .binding(EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_TYPE)
                .is_some()
        );
        assert!(
            node.documents()
                .binding(EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_TYPE)
                .is_some()
        );
        Ok(())
    }

    #[test]
    fn builtin_verse_policies_keep_public_and_private_boundaries_apart() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-verses.ccmp");
        let written = write_epiphany_cultmesh_verse_policies(&store, "epiphany-test")?;
        assert_eq!(written.len(), 3);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let internal = node.get_required::<EpiphanyCultMeshVersePolicyEntry>(
            EPIPHANY_CULTMESH_INTERNAL_VERSE_ID,
        )?;
        let local_area = node.get_required::<EpiphanyCultMeshVersePolicyEntry>(
            EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID,
        )?;
        let global = node
            .get_required::<EpiphanyCultMeshVersePolicyEntry>(EPIPHANY_CULTMESH_GLOBAL_VERSE_ID)?;

        assert!(internal.private_state_allowed);
        assert!(!internal.untrusted_ingress_allowed);
        assert!(!local_area.private_state_allowed);
        assert!(local_area.yggdrasil_tunnel_allowed);
        assert!(!global.private_state_allowed);
        assert!(global.untrusted_ingress_allowed);
        Ok(())
    }

    #[test]
    fn global_room_policies_make_public_threaded_rooms_for_faces() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-global-rooms.ccmp");
        let written = write_epiphany_cultmesh_global_room_policies(&store, "epiphany-test")?;
        assert!(written.len() >= 5);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let dreams =
            node.get_required::<EpiphanyCultMeshGlobalRoomPolicyEntry>("epiphany-global/dreams")?;
        let architecture = node.get_required::<EpiphanyCultMeshGlobalRoomPolicyEntry>(
            "epiphany-global/architecture",
        )?;

        assert_eq!(dreams.verse_id, EPIPHANY_CULTMESH_GLOBAL_VERSE_ID);
        assert!(dreams.threaded);
        assert!(dreams.face_posting_allowed);
        assert!(dreams.untrusted_ingress_allowed);
        assert!(architecture.purpose.contains("ownership"));
        Ok(())
    }

    #[test]
    fn mind_contracts_use_verses_to_keep_state_guarded() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-mind-contracts.ccmp");
        let written = write_epiphany_cultmesh_mind_contracts(&store, "epiphany-test")?;
        assert!(written.len() >= 4);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let state_review = node.get_required::<EpiphanyCultMeshMindContractEntry>(
            "epiphany.mind.state_effect.review",
        )?;
        let public_adoption = node.get_required::<EpiphanyCultMeshMindContractEntry>(
            "epiphany.mind.public_adoption.review",
        )?;

        assert_eq!(state_review.verse_id, EPIPHANY_CULTMESH_INTERNAL_VERSE_ID);
        assert_eq!(state_review.authority, "mind");
        assert!(
            state_review
                .notes
                .iter()
                .any(|note| note.contains("persistent state guardian"))
        );
        assert_eq!(public_adoption.verse_id, EPIPHANY_CULTMESH_GLOBAL_VERSE_ID);
        assert!(
            public_adoption
                .notes
                .iter()
                .any(|note| note.contains("thought weather"))
        );
        Ok(())
    }

    #[test]
    fn substrate_gate_contracts_use_verses_to_keep_repo_access_guarded() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-substrate-gate-contracts.ccmp");
        let written = write_epiphany_cultmesh_substrate_gate_contracts(&store, "epiphany-test")?;
        assert!(written.len() >= 4);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let repo_access = node.get_required::<EpiphanyCultMeshSubstrateGateContractEntry>(
            "epiphany.substrate_gate.repo_access.review",
        )?;

        assert_eq!(repo_access.verse_id, EPIPHANY_CULTMESH_INTERNAL_VERSE_ID);
        assert_eq!(repo_access.authority, "substrateGate");
        assert!(
            repo_access
                .notes
                .iter()
                .any(|note| note.contains("repo access protocol"))
        );
        Ok(())
    }

    #[test]
    fn eyes_contracts_use_verses_to_keep_evidence_guarded() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-eyes-contracts.ccmp");
        let written = write_epiphany_cultmesh_eyes_contracts(&store, "epiphany-test")?;
        assert!(written.len() >= 4);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let evidence = node
            .get_required::<EpiphanyCultMeshEyesContractEntry>("epiphany.eyes.evidence.review")?;

        assert_eq!(evidence.verse_id, EPIPHANY_CULTMESH_INTERNAL_VERSE_ID);
        assert_eq!(evidence.authority, "eyes");
        assert!(
            evidence
                .notes
                .iter()
                .any(|note| note.contains("evidence ingress guardian"))
        );
        Ok(())
    }

    #[test]
    fn hands_contracts_use_verses_to_keep_action_guarded() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-hands-contracts.ccmp");
        let written = write_epiphany_cultmesh_hands_contracts(&store, "epiphany-test")?;
        assert!(written.len() >= 5);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let action = node
            .get_required::<EpiphanyCultMeshHandsContractEntry>("epiphany.hands.action.review")?;

        assert_eq!(action.verse_id, EPIPHANY_CULTMESH_INTERNAL_VERSE_ID);
        assert_eq!(action.authority, "hands");
        assert!(
            action
                .notes
                .iter()
                .any(|note| note.contains("action organ"))
        );
        Ok(())
    }

    #[test]
    fn soul_contracts_use_verses_to_keep_verification_guarded() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-soul-contracts.ccmp");
        let written = write_epiphany_cultmesh_soul_contracts(&store, "epiphany-test")?;
        assert!(written.len() >= 5);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let verification = node.get_required::<EpiphanyCultMeshSoulContractEntry>(
            "epiphany.soul.verification.review",
        )?;

        assert_eq!(verification.verse_id, EPIPHANY_CULTMESH_INTERNAL_VERSE_ID);
        assert_eq!(verification.authority, "soul");
        assert!(
            verification
                .notes
                .iter()
                .any(|note| note.contains("verification organ"))
        );
        Ok(())
    }

    #[test]
    fn continuity_contracts_use_verses_to_keep_continuity_guarded() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-continuity-contracts.ccmp");
        let written = write_epiphany_cultmesh_continuity_contracts(&store, "epiphany-test")?;
        assert!(written.len() >= 5);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let continuity = node.get_required::<EpiphanyCultMeshContinuityContractEntry>(
            "epiphany.continuity.review",
        )?;

        assert_eq!(continuity.verse_id, EPIPHANY_CULTMESH_INTERNAL_VERSE_ID);
        assert_eq!(continuity.authority, "continuity");
        assert!(
            continuity
                .notes
                .iter()
                .any(|note| note.contains("deterministic protocol surface"))
        );
        Ok(())
    }
}
