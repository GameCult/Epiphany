use anyhow::Result;
use cultcache_rs::CultCache;
use cultcache_rs::SingleFileMessagePackBackingStore;
use cultnet_rs::CultNetDocumentBinding;
use cultnet_rs::CultNetDocumentRegistry;
use epiphany_core::EpiphanyAgentMemoryEntry;
use epiphany_core::agent_memory_status;

#[test]
fn epiphany_agent_memory_can_move_over_cultnet_documents() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let source_store = temp.path().join("source-agents.msgpack");
    let target_store = temp.path().join("target-agents.msgpack");
    std::fs::copy("../state/agents.msgpack", &source_store)?;

    let mut source_cache = CultCache::new();
    source_cache.register_entry_type::<EpiphanyAgentMemoryEntry>()?;
    source_cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(&source_store));
    source_cache.pull_all_backing_stores()?;

    let mut registry = CultNetDocumentRegistry::new();
    registry.register(
        CultNetDocumentBinding::for_entry::<EpiphanyAgentMemoryEntry>(Some(
            "ghostlight.agent_state.v0".to_string(),
        )),
    );
    let snapshot = registry.create_snapshot_response(
        &source_cache,
        "epiphany-agent-memory-snapshot",
        None,
        None,
    )?;

    let mut target_cache = CultCache::new();
    target_cache.register_entry_type::<EpiphanyAgentMemoryEntry>()?;
    target_cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(&target_store));
    target_cache.pull_all_backing_stores()?;
    let applied = registry
        .apply_snapshot_response::<EpiphanyAgentMemoryEntry>(&mut target_cache, &snapshot)?;
    assert_eq!(applied.len(), 8);
    assert!(
        target_cache
            .get::<EpiphanyAgentMemoryEntry>("coordinator")?
            .is_some()
    );

    let status = agent_memory_status(&target_store)?;
    assert_eq!(status["ok"], true);
    Ok(())
}
