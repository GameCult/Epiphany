use anyhow::Result;
use anyhow::anyhow;
use cultcache_rs::CultCache;
use cultcache_rs::DatabaseEntry;
use cultcache_rs::SingleFileMessagePackBackingStore;
use epiphany_state_model::EpiphanyThreadState;
use std::path::Path;

pub const THREAD_STATE_TYPE: &str = "epiphany.thread_state";
pub const THREAD_STATE_KEY: &str = "default";
pub const THREAD_STATE_SCHEMA_VERSION: &str = "epiphany.thread_state.v0";

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(type = "epiphany.thread_state", schema = "EpiphanyThreadStateEntry")]
pub struct EpiphanyThreadStateEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub thread_id: String,
    #[cultcache(key = 2)]
    pub state_msgpack: Vec<u8>,
}

impl EpiphanyThreadStateEntry {
    pub fn from_state(thread_id: impl Into<String>, state: &EpiphanyThreadState) -> Result<Self> {
        let state_msgpack = rmp_serde::to_vec_named(state)?;
        Ok(Self {
            schema_version: THREAD_STATE_SCHEMA_VERSION.to_string(),
            thread_id: thread_id.into(),
            state_msgpack,
        })
    }

    pub fn state(&self) -> Result<EpiphanyThreadState> {
        rmp_serde::from_slice(&self.state_msgpack)
            .map_err(|error| anyhow!("failed to decode Epiphany thread state MessagePack: {error}"))
    }
}

pub fn thread_state_cache(store_path: impl AsRef<Path>) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<EpiphanyThreadStateEntry>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(store_path.as_ref()));
    cache.pull_all_backing_stores()?;
    Ok(cache)
}

pub fn load_thread_state_entry(
    store_path: impl AsRef<Path>,
) -> Result<Option<EpiphanyThreadStateEntry>> {
    let cache = thread_state_cache(store_path)?;
    cache.get::<EpiphanyThreadStateEntry>(THREAD_STATE_KEY)
}

pub fn load_thread_state(store_path: impl AsRef<Path>) -> Result<Option<EpiphanyThreadState>> {
    let Some(entry) = load_thread_state_entry(store_path)? else {
        return Ok(None);
    };
    validate_thread_state_entry(&entry)?;
    Ok(Some(entry.state()?))
}

pub fn write_thread_state_entry(
    store_path: impl AsRef<Path>,
    entry: &EpiphanyThreadStateEntry,
) -> Result<EpiphanyThreadStateEntry> {
    validate_thread_state_entry(entry)?;
    let mut cache = thread_state_cache(store_path)?;
    cache.put(THREAD_STATE_KEY, entry)
}

pub fn write_thread_state(
    store_path: impl AsRef<Path>,
    thread_id: impl Into<String>,
    state: &EpiphanyThreadState,
) -> Result<EpiphanyThreadStateEntry> {
    let entry = EpiphanyThreadStateEntry::from_state(thread_id, state)?;
    write_thread_state_entry(store_path, &entry)
}

pub fn validate_thread_state_entry(entry: &EpiphanyThreadStateEntry) -> Result<()> {
    if entry.schema_version != THREAD_STATE_SCHEMA_VERSION {
        return Err(anyhow!(
            "thread state schema_version is {:?}, expected {:?}",
            entry.schema_version,
            THREAD_STATE_SCHEMA_VERSION
        ));
    }
    entry.state()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_state_model::EpiphanyGraph;
    use epiphany_state_model::EpiphanyGraphNode;
    use epiphany_state_model::EpiphanyGraphs;

    #[test]
    fn thread_state_round_trips_through_cultcache() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("thread-state.msgpack");
        let state = EpiphanyThreadState {
            revision: 7,
            objective: Some("Keep the graph native.".to_string()),
            graphs: EpiphanyGraphs {
                architecture: EpiphanyGraph {
                    nodes: vec![EpiphanyGraphNode {
                        id: "node-memory-graph".to_string(),
                        title: "Memory graph".to_string(),
                        purpose: "Own native graph memory.".to_string(),
                        status: Some("accepted".to_string()),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        write_thread_state(&store, "thread-1", &state)?;
        let loaded = load_thread_state(&store)?.expect("thread state should load");

        assert_eq!(loaded.revision, 7);
        assert_eq!(loaded.graphs.architecture.nodes.len(), 1);
        Ok(())
    }
}
