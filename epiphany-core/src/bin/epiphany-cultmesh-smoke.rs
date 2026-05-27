use anyhow::Result;
use epiphany_core::EPIPHANY_CULTMESH_INTERNAL_TIER;
use epiphany_core::EPIPHANY_CULTMESH_INTERNAL_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION;
use epiphany_core::EpiphanyCultMeshStatusEntry;
use epiphany_core::default_epiphany_cultmesh_operator_status;
use epiphany_core::load_epiphany_cultmesh_operator_status;
use epiphany_core::load_epiphany_cultmesh_status;
use epiphany_core::write_epiphany_cultmesh_body_contracts;
use epiphany_core::write_epiphany_cultmesh_eyes_contracts;
use epiphany_core::write_epiphany_cultmesh_global_room_policies;
use epiphany_core::write_epiphany_cultmesh_hands_contracts;
use epiphany_core::write_epiphany_cultmesh_life_contracts;
use epiphany_core::write_epiphany_cultmesh_mind_contracts;
use epiphany_core::write_epiphany_cultmesh_operator_status;
use epiphany_core::write_epiphany_cultmesh_soul_contracts;
use epiphany_core::write_epiphany_cultmesh_status;
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
    write_epiphany_cultmesh_body_contracts(&store, "epiphany-cultmesh-smoke")?;
    write_epiphany_cultmesh_eyes_contracts(&store, "epiphany-cultmesh-smoke")?;
    write_epiphany_cultmesh_hands_contracts(&store, "epiphany-cultmesh-smoke")?;
    write_epiphany_cultmesh_soul_contracts(&store, "epiphany-cultmesh-smoke")?;
    write_epiphany_cultmesh_life_contracts(&store, "epiphany-cultmesh-smoke")?;
    let operator_status = default_epiphany_cultmesh_operator_status(
        "epiphany-cultmesh-smoke",
        "2026-05-27T00:00:00Z",
    );
    write_epiphany_cultmesh_operator_status(&store, operator_status.clone())?;
    let loaded = load_epiphany_cultmesh_status(&store, "epiphany-cultmesh-smoke")?;
    if loaded != Some(status) {
        anyhow::bail!("CultMesh smoke failed to round-trip Epiphany status document");
    }
    let loaded_operator_status =
        load_epiphany_cultmesh_operator_status(&store, "epiphany-cultmesh-smoke")?;
    if loaded_operator_status != Some(operator_status) {
        anyhow::bail!("CultMesh smoke failed to round-trip Epiphany operator status document");
    }

    println!(
        "epiphany cultmesh smoke ok: {}",
        store.canonicalize()?.display()
    );
    Ok(())
}
