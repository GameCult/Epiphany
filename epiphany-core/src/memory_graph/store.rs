use super::EpiphanyMemoryGraphSnapshot;
use super::validate_memory_graph_snapshot;
use anyhow::Result;
use anyhow::anyhow;
use cultcache_rs::CultCache;
use cultcache_rs::DatabaseEntry;
use cultcache_rs::SingleFileMessagePackBackingStore;
use std::path::Path;

pub const MEMORY_GRAPH_TYPE: &str = "epiphany.memory_graph";
pub const MEMORY_GRAPH_KEY: &str = "default";
pub const MEMORY_GRAPH_SCHEMA_VERSION: &str = "epiphany.memory_graph.v0";

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(type = "epiphany.memory_graph", schema = "EpiphanyMemoryGraphEntry")]
pub struct EpiphanyMemoryGraphEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub graph_id: String,
    #[cultcache(key = 2)]
    pub snapshot_msgpack: Vec<u8>,
}

impl EpiphanyMemoryGraphEntry {
    pub fn from_snapshot(snapshot: &EpiphanyMemoryGraphSnapshot) -> Result<Self> {
        let snapshot_msgpack = rmp_serde::to_vec_named(snapshot)?;
        Ok(Self {
            schema_version: MEMORY_GRAPH_SCHEMA_VERSION.to_string(),
            graph_id: snapshot.graph_id.clone(),
            snapshot_msgpack,
        })
    }

    pub fn snapshot(&self) -> Result<EpiphanyMemoryGraphSnapshot> {
        rmp_serde::from_slice(&self.snapshot_msgpack)
            .map_err(|error| anyhow!("failed to decode memory graph snapshot MessagePack: {error}"))
    }
}

pub fn memory_graph_cache(store_path: impl AsRef<Path>) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<EpiphanyMemoryGraphEntry>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(store_path.as_ref()));
    cache.pull_all_backing_stores()?;
    Ok(cache)
}

pub fn load_memory_graph_entry(
    store_path: impl AsRef<Path>,
) -> Result<Option<EpiphanyMemoryGraphEntry>> {
    let cache = memory_graph_cache(store_path)?;
    cache.get::<EpiphanyMemoryGraphEntry>(MEMORY_GRAPH_KEY)
}

pub fn load_memory_graph_snapshot(
    store_path: impl AsRef<Path>,
) -> Result<Option<EpiphanyMemoryGraphSnapshot>> {
    let Some(entry) = load_memory_graph_entry(store_path)? else {
        return Ok(None);
    };
    validate_memory_graph_entry(&entry)?;
    Ok(Some(entry.snapshot()?))
}

pub fn write_memory_graph_entry(
    store_path: impl AsRef<Path>,
    entry: &EpiphanyMemoryGraphEntry,
) -> Result<EpiphanyMemoryGraphEntry> {
    validate_memory_graph_entry(entry)?;
    let mut cache = memory_graph_cache(store_path)?;
    cache.put(MEMORY_GRAPH_KEY, entry)
}

pub fn write_memory_graph_snapshot(
    store_path: impl AsRef<Path>,
    snapshot: &EpiphanyMemoryGraphSnapshot,
) -> Result<EpiphanyMemoryGraphEntry> {
    let entry = EpiphanyMemoryGraphEntry::from_snapshot(snapshot)?;
    write_memory_graph_entry(store_path, &entry)
}

pub fn validate_memory_graph_entry(entry: &EpiphanyMemoryGraphEntry) -> Result<()> {
    if entry.schema_version != MEMORY_GRAPH_SCHEMA_VERSION {
        return Err(anyhow!(
            "memory graph schema_version is {:?}, expected {:?}",
            entry.schema_version,
            MEMORY_GRAPH_SCHEMA_VERSION
        ));
    }
    let snapshot = entry.snapshot()?;
    if entry.graph_id != snapshot.graph_id {
        return Err(anyhow!(
            "memory graph entry graph_id {:?} does not match snapshot graph_id {:?}",
            entry.graph_id,
            snapshot.graph_id
        ));
    }
    let errors = validate_memory_graph_snapshot(&snapshot);
    if !errors.is_empty() {
        let message = errors
            .iter()
            .map(|error| format!("{}: {}", error.path, error.message))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(anyhow!("memory graph validation failed: {message}"));
    }
    Ok(())
}
