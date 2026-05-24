use anyhow::Result;
use epiphany_core::EPIPHANY_CULTMESH_LOCAL_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION;
use epiphany_core::EpiphanyCultMeshStatusEntry;
use epiphany_core::load_epiphany_cultmesh_status;
use epiphany_core::write_epiphany_cultmesh_status;
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut store = PathBuf::from(".epiphany-smoke");
    store.push("cultmesh");
    std::fs::create_dir_all(&store)?;
    store.push("epiphany-local.ccmp");

    let status = EpiphanyCultMeshStatusEntry {
        schema_version: EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION.to_string(),
        runtime_id: "epiphany-cultmesh-smoke".to_string(),
        verse_id: EPIPHANY_CULTMESH_LOCAL_VERSE_ID.to_string(),
        app_id: "epiphany".to_string(),
        note: "Epiphany wrote this through CultMesh, not direct CultNet plumbing.".to_string(),
    };

    write_epiphany_cultmesh_status(&store, status.clone())?;
    let loaded = load_epiphany_cultmesh_status(&store, "epiphany-cultmesh-smoke")?;
    if loaded != Some(status) {
        anyhow::bail!("CultMesh smoke failed to round-trip Epiphany status document");
    }

    println!(
        "epiphany cultmesh smoke ok: {}",
        store.canonicalize()?.display()
    );
    Ok(())
}
