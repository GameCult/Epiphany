use super::EpiphanyHeartbeatArtifactRetentionPlan;
use super::EpiphanyHeartbeatArtifactRetentionReceipt;
use super::EpiphanyHeartbeatStaleTurnRepairReceipt;
use super::EpiphanyHeartbeatStateEntry;
use super::HEARTBEAT_ARENA_MAINTENANCE;
use super::HEARTBEAT_STALE_TURN_REPAIR_LATEST_KEY;
use super::HEARTBEAT_STALE_TURN_REPAIR_SCHEMA_VERSION;
use super::HEARTBEAT_STATE_KEY;
use super::HEARTBEAT_STATE_SCHEMA_VERSION;
use super::PARTICIPANT_KIND_AGENT;
use super::participant_arena;
use super::participant_kind;
use anyhow::Result;
use anyhow::anyhow;
use cultcache_rs::SingleFileMessagePackBackingStore;
use cultcache_rs::{CacheBackingStore, CultCache, CultCacheEnvelope, DatabaseEntry};
use std::collections::HashSet;
use std::path::Path;

pub fn heartbeat_state_cache(store_path: impl AsRef<Path>) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<EpiphanyHeartbeatStateEntry>()?;
    cache.register_entry_type::<EpiphanyHeartbeatStaleTurnRepairReceipt>()?;
    cache.register_entry_type::<EpiphanyHeartbeatArtifactRetentionPlan>()?;
    cache.register_entry_type::<EpiphanyHeartbeatArtifactRetentionReceipt>()?;
    import_owned_heartbeat_envelopes(&mut cache, store_path.as_ref())?;
    Ok(cache)
}

fn import_owned_heartbeat_envelopes(cache: &mut CultCache, store_path: &Path) -> Result<()> {
    let state_type = <EpiphanyHeartbeatStateEntry as DatabaseEntry>::TYPE;
    let mut identities = HashSet::new();
    for envelope in SingleFileMessagePackBackingStore::new(store_path).pull_all()? {
        let owned = (envelope.r#type == state_type && envelope.key == HEARTBEAT_STATE_KEY)
            || envelope.r#type == <EpiphanyHeartbeatStaleTurnRepairReceipt as DatabaseEntry>::TYPE;
        let owned = owned
            || envelope.r#type == <EpiphanyHeartbeatArtifactRetentionPlan as DatabaseEntry>::TYPE
            || envelope.r#type
                == <EpiphanyHeartbeatArtifactRetentionReceipt as DatabaseEntry>::TYPE;
        if !owned {
            continue;
        }
        if !identities.insert((envelope.r#type.clone(), envelope.key.clone())) {
            return Err(anyhow!(
                "heartbeat store contains duplicate owner entry type {:?} key {:?}",
                envelope.r#type,
                envelope.key
            ));
        }
        if envelope.r#type == state_type {
            cache.load_envelope::<EpiphanyHeartbeatStateEntry>(envelope)?;
        } else if envelope.r#type == <EpiphanyHeartbeatArtifactRetentionPlan as DatabaseEntry>::TYPE
        {
            cache.load_envelope::<EpiphanyHeartbeatArtifactRetentionPlan>(envelope)?;
        } else if envelope.r#type
            == <EpiphanyHeartbeatArtifactRetentionReceipt as DatabaseEntry>::TYPE
        {
            cache.load_envelope::<EpiphanyHeartbeatArtifactRetentionReceipt>(envelope)?;
        } else {
            cache.load_envelope::<EpiphanyHeartbeatStaleTurnRepairReceipt>(envelope)?;
        }
    }
    Ok(())
}

fn commit_owned_entry<T: DatabaseEntry>(
    store_path: &Path,
    cache: &CultCache,
    key: &str,
    value: &T,
) -> Result<T> {
    let expected = cache
        .snapshot_envelopes()
        .into_iter()
        .find(|entry| entry.r#type == T::TYPE && entry.key == key);
    let (replacement, written) = cache.prepare_entry(key, value)?;
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    let committed = match expected {
        Some(expected) => backing.compare_and_swap_entry(&expected, replacement)?,
        None => backing.insert_entry_if_absent(replacement)?,
    };
    if !committed {
        return Err(anyhow!(
            "heartbeat owner entry lost exact atomic compare-and-swap"
        ));
    }
    Ok(written)
}

pub fn load_heartbeat_state_entry(
    store_path: impl AsRef<Path>,
) -> Result<Option<EpiphanyHeartbeatStateEntry>> {
    let cache = heartbeat_state_cache(store_path)?;
    cache.get::<EpiphanyHeartbeatStateEntry>(HEARTBEAT_STATE_KEY)
}

pub fn write_heartbeat_state_entry(
    store_path: impl AsRef<Path>,
    state: &EpiphanyHeartbeatStateEntry,
) -> Result<EpiphanyHeartbeatStateEntry> {
    validate_heartbeat_state(state)?;
    let store_path = store_path.as_ref();
    let cache = heartbeat_state_cache(store_path)?;
    commit_owned_entry(store_path, &cache, HEARTBEAT_STATE_KEY, state)
}

pub fn load_heartbeat_state_transaction(
    store_path: impl AsRef<Path>,
) -> Result<(
    Option<EpiphanyHeartbeatStateEntry>,
    Option<CultCacheEnvelope>,
)> {
    let cache = heartbeat_state_cache(store_path)?;
    let envelope = cache.snapshot_envelopes().into_iter().find(|entry| {
        entry.r#type == <EpiphanyHeartbeatStateEntry as DatabaseEntry>::TYPE
            && entry.key == HEARTBEAT_STATE_KEY
    });
    Ok((
        cache.get::<EpiphanyHeartbeatStateEntry>(HEARTBEAT_STATE_KEY)?,
        envelope,
    ))
}

pub fn commit_heartbeat_state_transaction(
    store_path: impl AsRef<Path>,
    expected: Option<CultCacheEnvelope>,
    state: &EpiphanyHeartbeatStateEntry,
) -> Result<()> {
    validate_heartbeat_state(state)?;
    let cache = heartbeat_state_cache(store_path.as_ref())?;
    let (replacement, _) = cache.prepare_entry(HEARTBEAT_STATE_KEY, state)?;
    let backing = SingleFileMessagePackBackingStore::new(store_path.as_ref());
    let committed = match expected {
        Some(expected) => backing.compare_and_swap_entry(&expected, replacement)?,
        None => backing.insert_entry_if_absent(replacement)?,
    };
    if !committed {
        return Err(anyhow!(
            "heartbeat state lost exact atomic compare-and-swap"
        ));
    }
    Ok(())
}

pub fn write_heartbeat_stale_turn_repair_receipt(
    store_path: impl AsRef<Path>,
    receipt: &EpiphanyHeartbeatStaleTurnRepairReceipt,
) -> Result<EpiphanyHeartbeatStaleTurnRepairReceipt> {
    validate_heartbeat_stale_turn_repair_receipt(receipt)?;
    let store_path = store_path.as_ref();
    let cache = heartbeat_state_cache(store_path)?;
    let written = receipt.clone();
    let (receipt_entry, _) = cache.prepare_entry(&receipt.receipt_id, receipt)?;
    let (latest_entry, _) = cache.prepare_entry(HEARTBEAT_STALE_TURN_REPAIR_LATEST_KEY, receipt)?;
    let expected = cache
        .snapshot_envelopes()
        .into_iter()
        .filter(|entry| {
            entry.r#type == <EpiphanyHeartbeatStaleTurnRepairReceipt as DatabaseEntry>::TYPE
                && (entry.key == receipt.receipt_id
                    || entry.key == HEARTBEAT_STALE_TURN_REPAIR_LATEST_KEY)
        })
        .collect::<Vec<_>>();
    if !SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&expected, vec![receipt_entry, latest_entry])?
    {
        return Err(anyhow!(
            "heartbeat stale-turn receipt lost exact atomic compare-and-swap"
        ));
    }
    Ok(written)
}

pub fn load_latest_heartbeat_stale_turn_repair_receipt(
    store_path: impl AsRef<Path>,
) -> Result<Option<EpiphanyHeartbeatStaleTurnRepairReceipt>> {
    let cache = heartbeat_state_cache(store_path)?;
    cache.get::<EpiphanyHeartbeatStaleTurnRepairReceipt>(HEARTBEAT_STALE_TURN_REPAIR_LATEST_KEY)
}

pub fn validate_heartbeat_stale_turn_repair_receipt(
    receipt: &EpiphanyHeartbeatStaleTurnRepairReceipt,
) -> Result<()> {
    if receipt.schema_version != HEARTBEAT_STALE_TURN_REPAIR_SCHEMA_VERSION {
        return Err(anyhow!(
            "heartbeat stale-turn repair schema_version is {:?}, expected {:?}",
            receipt.schema_version,
            HEARTBEAT_STALE_TURN_REPAIR_SCHEMA_VERSION
        ));
    }
    if receipt.private_state_exposed {
        return Err(anyhow!(
            "heartbeat stale-turn repair receipt must not expose private state"
        ));
    }
    if receipt.receipt_id.trim().is_empty()
        || receipt.role_id.trim().is_empty()
        || receipt.agent_id.trim().is_empty()
        || receipt.action_id.trim().is_empty()
    {
        return Err(anyhow!(
            "heartbeat stale-turn repair receipt requires receipt, role, agent, and action ids"
        ));
    }
    if receipt.stale_age_seconds < 0 {
        return Err(anyhow!(
            "heartbeat stale-turn repair receipt stale_age_seconds must be non-negative"
        ));
    }
    if receipt.reason.trim().is_empty() {
        return Err(anyhow!(
            "heartbeat stale-turn repair receipt requires a reason"
        ));
    }
    if receipt.resulting_status != "repaired" {
        return Err(anyhow!(
            "heartbeat stale-turn repair receipt resulting_status must be repaired"
        ));
    }
    Ok(())
}

pub fn validate_heartbeat_state(state: &EpiphanyHeartbeatStateEntry) -> Result<()> {
    if state.schema_version != HEARTBEAT_STATE_SCHEMA_VERSION {
        return Err(anyhow!(
            "heartbeat state schema_version is {:?}, expected {:?}",
            state.schema_version,
            HEARTBEAT_STATE_SCHEMA_VERSION
        ));
    }
    if state.participants.is_empty() {
        return Err(anyhow!("heartbeat state has no participants"));
    }
    if state.target_heartbeat_rate < 0.0 {
        return Err(anyhow!(
            "heartbeat target_heartbeat_rate must be non-negative"
        ));
    }
    for participant in &state.participants {
        if participant.agent_id.trim().is_empty() {
            return Err(anyhow!("heartbeat participant has empty agent_id"));
        }
        if participant.role_id.trim().is_empty() {
            return Err(anyhow!(
                "heartbeat participant {} has empty role_id",
                participant.agent_id
            ));
        }
        if participant.initiative_speed <= 0.0 {
            return Err(anyhow!(
                "heartbeat participant {} initiative_speed must be positive",
                participant.agent_id
            ));
        }
        let arena = participant_arena(participant);
        if arena != HEARTBEAT_ARENA_MAINTENANCE {
            return Err(anyhow!(
                "heartbeat participant {} arena {:?} is unsupported",
                participant.agent_id,
                arena
            ));
        }
        let participant_kind = participant_kind(participant);
        if participant_kind != PARTICIPANT_KIND_AGENT {
            return Err(anyhow!(
                "heartbeat participant {} participant_kind {:?} is unsupported",
                participant.agent_id,
                participant_kind
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heartbeat_state::default_heartbeat_state;
    use pretty_assertions::assert_eq;

    #[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
    #[cultcache(type = "test.foreign.readiness", schema = "ForeignReadiness")]
    struct ForeignReadiness {
        #[cultcache(key = 0)]
        status: String,
    }

    #[test]
    fn round_trips_scheduler_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("heartbeats.msgpack");
        let state = default_heartbeat_state(1.0);

        write_heartbeat_state_entry(&store_path, &state)?;
        let loaded = load_heartbeat_state_entry(&store_path)?
            .expect("heartbeat state should round-trip through CultCache");

        assert_eq!(loaded.schema_version, HEARTBEAT_STATE_SCHEMA_VERSION);
        assert_eq!(loaded.participants.len(), state.participants.len());

        Ok(())
    }

    #[test]
    fn heartbeat_owner_reads_and_writes_preserve_foreign_shared_store_rows() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("heartbeats.msgpack");
        let mut foreign_cache = CultCache::new();
        foreign_cache.register_entry_type::<ForeignReadiness>()?;
        let (foreign, _) = foreign_cache.prepare_entry(
            "provider-readiness",
            &ForeignReadiness {
                status: "ready".into(),
            },
        )?;
        let mut backing = SingleFileMessagePackBackingStore::new(&store_path);
        backing.push(&foreign)?;

        assert!(load_heartbeat_state_entry(&store_path)?.is_none());
        write_heartbeat_state_entry(&store_path, &default_heartbeat_state(1.0))?;

        let rows = backing.pull_all()?;
        assert!(rows.contains(&foreign));
        assert!(rows.iter().any(|row| {
            row.r#type == <EpiphanyHeartbeatStateEntry as DatabaseEntry>::TYPE
                && row.key == HEARTBEAT_STATE_KEY
        }));
        Ok(())
    }
}
