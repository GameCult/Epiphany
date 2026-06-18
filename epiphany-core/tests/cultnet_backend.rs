use anyhow::Result;
use cultcache_rs::CultCache;
use cultcache_rs::SingleFileMessagePackBackingStore;
use cultnet_rs::CultNetDocumentBinding;
use cultnet_rs::CultNetDocumentRegistry;
use epiphany_core::EpiphanyAgentMemoryEntry;
use epiphany_core::agent_memory_status;
use epiphany_core::migrate_agent_memory_json_dir_to_cultcache;

#[test]
fn epiphany_agent_memory_can_move_over_cultnet_documents() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let agent_dir = temp.path().join("agents");
    let source_store = temp.path().join("source-agents.msgpack");
    let target_store = temp.path().join("target-agents.msgpack");
    write_agent_fixture(&agent_dir)?;
    migrate_agent_memory_json_dir_to_cultcache(&agent_dir, &source_store)?;

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
    assert_eq!(applied.len(), 7);
    assert!(
        target_cache
            .get::<EpiphanyAgentMemoryEntry>("coordinator")?
            .is_some()
    );

    let status = agent_memory_status(&target_store)?;
    assert_eq!(status["ok"], true);
    Ok(())
}

fn write_agent_fixture(agent_dir: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(agent_dir)?;
    for (role_id, agent_id, filename) in [
        (
            "imagination",
            "epiphany.imagination",
            "imagination.agent-state.json",
        ),
        ("modeling", "epiphany.modeling", "modeling.agent-state.json"),
        ("verification", "epiphany.soul", "soul.agent-state.json"),
        ("implementation", "epiphany.hands", "hands.agent-state.json"),
        ("research", "epiphany.eyes", "eyes.agent-state.json"),
        ("Persona", "epiphany.Persona", "Persona.agent-state.json"),
        ("coordinator", "epiphany.self", "self.agent-state.json"),
    ] {
        std::fs::write(
            agent_dir.join(filename),
            sample_agent_json(agent_id, &format!("Agent {role_id}")),
        )?;
    }
    Ok(())
}

fn sample_agent_json(agent_id: &str, name: &str) -> String {
    serde_json::json!({
        "schema_version": "ghostlight.agent_state.v0",
        "world": {
            "world_id": "epiphany-agent-memory",
            "setting": "Epiphany local harness role memory",
            "time": {"label": "standing memory"},
            "canon_context": ["Organ-state records preserve lane identity."]
        },
        "agents": [{
            "agent_id": agent_id,
            "identity": {
                "name": name,
                "roles": ["test"],
                "origin": "test lane",
                "public_description": "Test role memory.",
                "private_notes": []
            },
            "canonical_state": {
                "underlying_organization": {"test": {"mean": 0.5, "plasticity": 0.5, "current_activation": 0.5}},
                "stable_dispositions": {"test": {"mean": 0.5, "plasticity": 0.5, "current_activation": 0.5}},
                "behavioral_dimensions": {"test": {"mean": 0.5, "plasticity": 0.5, "current_activation": 0.5}},
                "presentation_strategy": {"test": {"mean": 0.5, "plasticity": 0.5, "current_activation": 0.5}},
                "voice_style": {"test": {"mean": 0.5, "plasticity": 0.5, "current_activation": 0.5}},
                "situational_state": {"test": {"mean": 0.5, "plasticity": 0.5, "current_activation": 0.5}},
                "values": [{
                    "value_id": "value-test",
                    "label": "Test value.",
                    "priority": 0.5,
                    "unforgivable_if_betrayed": false
                }]
            },
            "goals": [{
                "goal_id": "goal-test",
                "description": "Keep the test role valid.",
                "scope": "life",
                "priority": 0.5,
                "emotional_stake": "Broken memory makes broken agents.",
                "blockers": [],
                "status": "active"
            }],
            "memories": {
                "episodic": [],
                "semantic": [{
                    "memory_id": "mem-test",
                    "summary": "Test memory.",
                    "salience": 0.5,
                    "confidence": 0.5
                }],
                "relationship_summaries": []
            },
            "perceived_state_overlays": []
        }],
        "relationships": [],
        "events": [],
        "scenes": []
    })
    .to_string()
}
