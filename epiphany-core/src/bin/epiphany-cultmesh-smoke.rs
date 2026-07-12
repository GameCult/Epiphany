use anyhow::Result;
use epiphany_core::EPIPHANY_CULTMESH_INTERNAL_TIER;
use epiphany_core::EPIPHANY_CULTMESH_INTERNAL_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION;
use epiphany_core::EpiphanyCultMeshStatusEntry;
use epiphany_core::epiphany_cultmesh_operator_snapshot_from_status_json;
use epiphany_core::load_epiphany_cultmesh_status;
use epiphany_core::load_latest_epiphany_cultmesh_operator_snapshot;
use epiphany_core::write_epiphany_cultmesh_continuity_contracts;
use epiphany_core::write_epiphany_cultmesh_eyes_contracts;
use epiphany_core::write_epiphany_cultmesh_global_room_policies;
use epiphany_core::write_epiphany_cultmesh_hands_contracts;
use epiphany_core::write_epiphany_cultmesh_mind_contracts;
use epiphany_core::write_epiphany_cultmesh_operator_snapshot;
use epiphany_core::write_epiphany_cultmesh_soul_contracts;
use epiphany_core::write_epiphany_cultmesh_status;
use epiphany_core::write_epiphany_cultmesh_substrate_gate_contracts;
use epiphany_core::write_epiphany_cultmesh_verse_policies;
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut store = PathBuf::from(".epiphany-smoke");
    store.push("cultmesh");
    std::fs::create_dir_all(&store)?;
    store.push("epiphany-local.ccmp");

    let status = EpiphanyCultMeshStatusEntry {
        schema_version: EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION.to_string(),
        runtime_id: "epiphany-cultmesh-smoke".to_string(),
        verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
        verse_tier: EPIPHANY_CULTMESH_INTERNAL_TIER.to_string(),
        app_id: "epiphany".to_string(),
        note: "Epiphany wrote this through CultMesh, not direct CultNet plumbing.".to_string(),
    };

    write_epiphany_cultmesh_status(&store, status.clone())?;
    write_epiphany_cultmesh_verse_policies(&store, "epiphany-cultmesh-smoke")?;
    write_epiphany_cultmesh_global_room_policies(&store, "epiphany-cultmesh-smoke")?;
    write_epiphany_cultmesh_mind_contracts(&store, "epiphany-cultmesh-smoke")?;
    write_epiphany_cultmesh_substrate_gate_contracts(&store, "epiphany-cultmesh-smoke")?;
    write_epiphany_cultmesh_eyes_contracts(&store, "epiphany-cultmesh-smoke")?;
    write_epiphany_cultmesh_hands_contracts(&store, "epiphany-cultmesh-smoke")?;
    write_epiphany_cultmesh_soul_contracts(&store, "epiphany-cultmesh-smoke")?;
    write_epiphany_cultmesh_continuity_contracts(&store, "epiphany-cultmesh-smoke")?;
    let operator_snapshot = epiphany_cultmesh_operator_snapshot_from_status_json(
        "epiphany-cultmesh-smoke",
        "cultmesh-smoke-status",
        "2026-05-27T00:00:00Z",
        "status",
        ".epiphany-smoke/cultmesh/status.json",
        &serde_json::json!({
            "threadId": "thread-smoke",
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
            }
        }),
    )?;
    write_epiphany_cultmesh_operator_snapshot(&store, operator_snapshot.clone())?;
    let loaded = load_epiphany_cultmesh_status(&store, "epiphany-cultmesh-smoke")?;
    if loaded != Some(status) {
        anyhow::bail!("CultMesh smoke failed to round-trip Epiphany status document");
    }
    let loaded_operator_snapshot =
        load_latest_epiphany_cultmesh_operator_snapshot(&store, "epiphany-cultmesh-smoke")?;
    if loaded_operator_snapshot != Some(operator_snapshot) {
        anyhow::bail!("CultMesh smoke failed to round-trip Epiphany operator snapshot document");
    }

    println!(
        "epiphany cultmesh smoke ok: {}",
        store.canonicalize()?.display()
    );
    Ok(())
}
