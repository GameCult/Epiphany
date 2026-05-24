use crate::default_mind_cultnet_contracts;
use anyhow::Result;
use cultcache_rs::DatabaseEntry;
use cultmesh_rs::CultMesh;
use cultmesh_rs::CultMeshNode;
use cultmesh_rs::CultMeshNodeOptions;
use cultmesh_rs::cultmesh_documents;
use std::path::Path;

pub const EPIPHANY_CULTMESH_STATUS_TYPE: &str = "epiphany.cultmesh.status";
pub const EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION: &str = "epiphany.cultmesh.status.v0";
pub const EPIPHANY_CULTMESH_STATUS_KEY: &str = "epiphany-local/status";
pub const EPIPHANY_CULTMESH_VERSE_POLICY_TYPE: &str = "epiphany.cultmesh.verse_policy";
pub const EPIPHANY_CULTMESH_VERSE_POLICY_SCHEMA_VERSION: &str = "epiphany.cultmesh.verse_policy.v0";
pub const EPIPHANY_CULTMESH_GLOBAL_ROOM_POLICY_TYPE: &str = "epiphany.cultmesh.global_room_policy";
pub const EPIPHANY_CULTMESH_GLOBAL_ROOM_POLICY_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.global_room_policy.v0";
pub const EPIPHANY_CULTMESH_MIND_CONTRACT_TYPE: &str = "epiphany.cultmesh.mind_contract";
pub const EPIPHANY_CULTMESH_MIND_CONTRACT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.mind_contract.v0";
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

cultmesh_documents!(EpiphanyCultMeshDocuments {
    EpiphanyCultMeshStatusEntry => EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION,
    EpiphanyCultMeshVersePolicyEntry => EPIPHANY_CULTMESH_VERSE_POLICY_SCHEMA_VERSION,
    EpiphanyCultMeshGlobalRoomPolicyEntry => EPIPHANY_CULTMESH_GLOBAL_ROOM_POLICY_SCHEMA_VERSION,
    EpiphanyCultMeshMindContractEntry => EPIPHANY_CULTMESH_MIND_CONTRACT_SCHEMA_VERSION,
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

pub fn epiphany_cultmesh_verse_policies() -> Vec<EpiphanyCultMeshVersePolicyEntry> {
    vec![
        EpiphanyCultMeshVersePolicyEntry {
            schema_version: EPIPHANY_CULTMESH_VERSE_POLICY_SCHEMA_VERSION.to_string(),
            verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
            tier: EPIPHANY_CULTMESH_INTERNAL_TIER.to_string(),
            purpose: "Sub-agent typed state: heartbeat, role dossiers, runtime-spine jobs, private receipts, and other Epiphany-owned organs.".to_string(),
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
}
