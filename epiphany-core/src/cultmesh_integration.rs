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
pub const EPIPHANY_CULTMESH_LOCAL_VERSE_ID: &str = "epiphany-local";

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
}

cultmesh_documents!(EpiphanyCultMeshDocuments {
    EpiphanyCultMeshStatusEntry => EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION,
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
            verse_id: EPIPHANY_CULTMESH_LOCAL_VERSE_ID.to_string(),
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
}
