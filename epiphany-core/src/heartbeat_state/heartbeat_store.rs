use super::EpiphanyHeartbeatCognitionEntry;
use super::EpiphanyHeartbeatStaleTurnRepairReceipt;
use super::EpiphanyHeartbeatStateEntry;
use super::HEARTBEAT_ARENA_MAINTENANCE;
use super::HEARTBEAT_ARENA_SCENE;
use super::HEARTBEAT_COGNITION_KEY;
use super::HEARTBEAT_COGNITION_SCHEMA_VERSION;
use super::HEARTBEAT_STALE_TURN_REPAIR_LATEST_KEY;
use super::HEARTBEAT_STALE_TURN_REPAIR_SCHEMA_VERSION;
use super::HEARTBEAT_STATE_KEY;
use super::HEARTBEAT_STATE_SCHEMA_VERSION;
use super::LegacyHeartbeatStateWithCognition;
use super::PARTICIPANT_KIND_AGENT;
use super::PARTICIPANT_KIND_CHARACTER;
use super::now_iso;
use super::participant_arena;
use super::participant_kind;
use anyhow::Result;
use anyhow::anyhow;
use cultcache_rs::SingleFileMessagePackBackingStore;
use cultcache_rs::{CacheBackingStore, CultCache, CultCacheEnvelope, DatabaseEntry};
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;

pub fn heartbeat_state_cache(store_path: impl AsRef<Path>) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<EpiphanyHeartbeatStateEntry>()?;
    cache.register_entry_type::<EpiphanyHeartbeatCognitionEntry>()?;
    cache.register_entry_type::<EpiphanyHeartbeatStaleTurnRepairReceipt>()?;
    import_owned_heartbeat_envelopes(&mut cache, store_path.as_ref(), false)?;
    Ok(cache)
}

fn legacy_heartbeat_state_cache(store_path: impl AsRef<Path>) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<LegacyHeartbeatStateWithCognition>()?;
    cache.register_entry_type::<EpiphanyHeartbeatCognitionEntry>()?;
    cache.register_entry_type::<EpiphanyHeartbeatStaleTurnRepairReceipt>()?;
    import_owned_heartbeat_envelopes(&mut cache, store_path.as_ref(), true)?;
    Ok(cache)
}

fn import_owned_heartbeat_envelopes(
    cache: &mut CultCache,
    store_path: &Path,
    legacy: bool,
) -> Result<()> {
    let state_type = if legacy {
        <LegacyHeartbeatStateWithCognition as DatabaseEntry>::TYPE
    } else {
        <EpiphanyHeartbeatStateEntry as DatabaseEntry>::TYPE
    };
    let mut identities = HashSet::new();
    for envelope in SingleFileMessagePackBackingStore::new(store_path).pull_all()? {
        let owned = (envelope.r#type == state_type && envelope.key == HEARTBEAT_STATE_KEY)
            || (envelope.r#type == <EpiphanyHeartbeatCognitionEntry as DatabaseEntry>::TYPE
                && envelope.key == HEARTBEAT_COGNITION_KEY)
            || envelope.r#type == <EpiphanyHeartbeatStaleTurnRepairReceipt as DatabaseEntry>::TYPE;
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
            if legacy {
                // The legacy and current state documents intentionally share a
                // polymorphic identity. A current payload is not legacy cargo.
                let _ = cache.load_envelope::<LegacyHeartbeatStateWithCognition>(envelope);
            } else {
                cache.load_envelope::<EpiphanyHeartbeatStateEntry>(envelope)?;
            }
        } else if envelope.r#type == <EpiphanyHeartbeatCognitionEntry as DatabaseEntry>::TYPE {
            cache.load_envelope::<EpiphanyHeartbeatCognitionEntry>(envelope)?;
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

pub fn load_heartbeat_cognition_entry(
    store_path: impl AsRef<Path>,
) -> Result<Option<EpiphanyHeartbeatCognitionEntry>> {
    let store_path = store_path.as_ref();
    let cache = heartbeat_state_cache(store_path)?;
    if let Some(cognition) =
        cache.get::<EpiphanyHeartbeatCognitionEntry>(HEARTBEAT_COGNITION_KEY)?
    {
        return Ok(Some(cognition));
    }
    let legacy_cache = legacy_heartbeat_state_cache(store_path)?;
    let legacy = match legacy_cache.get::<LegacyHeartbeatStateWithCognition>(HEARTBEAT_STATE_KEY) {
        Ok(Some(legacy)) => legacy,
        Ok(None) | Err(_) => return Ok(None),
    };
    legacy_heartbeat_cognition_entry(legacy)
}

pub fn write_heartbeat_cognition_entry(
    store_path: impl AsRef<Path>,
    cognition: &EpiphanyHeartbeatCognitionEntry,
) -> Result<EpiphanyHeartbeatCognitionEntry> {
    validate_heartbeat_cognition(cognition)?;
    let store_path = store_path.as_ref();
    let cache = heartbeat_state_cache(store_path)?;
    commit_owned_entry(store_path, &cache, HEARTBEAT_COGNITION_KEY, cognition)
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

pub fn validate_heartbeat_cognition(cognition: &EpiphanyHeartbeatCognitionEntry) -> Result<()> {
    if cognition.schema_version != HEARTBEAT_COGNITION_SCHEMA_VERSION {
        return Err(anyhow!(
            "heartbeat cognition schema_version is {:?}, expected {:?}",
            cognition.schema_version,
            HEARTBEAT_COGNITION_SCHEMA_VERSION
        ));
    }
    Ok(())
}

fn legacy_heartbeat_cognition_entry(
    legacy: LegacyHeartbeatStateWithCognition,
) -> Result<Option<EpiphanyHeartbeatCognitionEntry>> {
    if legacy.sleep_cycle.is_none()
        && legacy.memory_resonance.is_none()
        && legacy.incubation.is_none()
        && legacy.thought_lanes.is_none()
        && legacy.bridge.is_none()
        && legacy.candidate_interventions.is_none()
        && legacy.appraisals.is_none()
        && legacy.reactions.is_none()
    {
        return Ok(None);
    }
    let routine = legacy.extra.get("voidRoutine");
    let latest_run_id = routine
        .and_then(|value| value.get("lastRunId"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let source = routine
        .and_then(|value| value.get("source"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let updated_at = routine
        .and_then(|value| value.get("lastRunAt"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(now_iso);
    Ok(Some(EpiphanyHeartbeatCognitionEntry {
        schema_version: HEARTBEAT_COGNITION_SCHEMA_VERSION.to_string(),
        updated_at,
        latest_run_id,
        latest_artifact_ref: None,
        source,
        sleep_cycle: decode_legacy_document(legacy.sleep_cycle)?,
        memory_resonance: decode_legacy_document(legacy.memory_resonance)?,
        incubation: decode_legacy_document(legacy.incubation)?,
        thought_lanes: decode_legacy_document(legacy.thought_lanes)?,
        bridge: decode_legacy_document(legacy.bridge)?,
        candidate_interventions: decode_legacy_document(legacy.candidate_interventions)?,
        appraisals: decode_legacy_document(legacy.appraisals)?,
        reactions: decode_legacy_document(legacy.reactions)?,
    }))
}

fn decode_legacy_document<T>(value: Option<Value>) -> Result<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    value
        .map(serde_json::from_value)
        .transpose()
        .map_err(Into::into)
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
    if state.initiative_heat.global_multiplier <= 0.0 {
        return Err(anyhow!(
            "heartbeat initiative_heat global_multiplier must be positive"
        ));
    }
    for multiplier in &state.initiative_heat.multipliers {
        if multiplier.id.trim().is_empty() {
            return Err(anyhow!("heartbeat initiative heat multiplier has empty id"));
        }
        if multiplier.multiplier <= 0.0 {
            return Err(anyhow!(
                "heartbeat initiative heat multiplier {} must be positive",
                multiplier.id
            ));
        }
        if multiplier.selector.trim().is_empty() && multiplier.scope != "all" {
            return Err(anyhow!(
                "heartbeat initiative heat multiplier {} selector is empty for scope {}",
                multiplier.id,
                multiplier.scope
            ));
        }
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
        if !matches!(arena, HEARTBEAT_ARENA_MAINTENANCE | HEARTBEAT_ARENA_SCENE) {
            return Err(anyhow!(
                "heartbeat participant {} arena {:?} is unsupported",
                participant.agent_id,
                arena
            ));
        }
        let participant_kind = participant_kind(participant);
        if !matches!(
            participant_kind,
            PARTICIPANT_KIND_AGENT | PARTICIPANT_KIND_CHARACTER
        ) {
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
    fn round_trips_state_and_cognition_documents() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("heartbeats.msgpack");
        let state = default_heartbeat_state(1.0);

        write_heartbeat_state_entry(&store_path, &state)?;
        let loaded = load_heartbeat_state_entry(&store_path)?
            .expect("heartbeat state should round-trip through CultCache");

        assert_eq!(loaded.schema_version, HEARTBEAT_STATE_SCHEMA_VERSION);
        assert_eq!(loaded.participants.len(), state.participants.len());

        let cognition = EpiphanyHeartbeatCognitionEntry {
            schema_version: HEARTBEAT_COGNITION_SCHEMA_VERSION.to_string(),
            updated_at: "2026-05-17T00:00:00Z".to_string(),
            latest_run_id: Some("run-1".to_string()),
            latest_artifact_ref: None,
            source: Some("unit-test".to_string()),
            sleep_cycle: None,
            memory_resonance: None,
            incubation: None,
            thought_lanes: None,
            bridge: None,
            candidate_interventions: None,
            appraisals: None,
            reactions: None,
        };

        write_heartbeat_cognition_entry(&store_path, &cognition)?;
        let loaded_cognition = load_heartbeat_cognition_entry(&store_path)?
            .expect("heartbeat cognition should round-trip through CultCache");

        assert_eq!(
            loaded_cognition.schema_version,
            HEARTBEAT_COGNITION_SCHEMA_VERSION
        );
        assert_eq!(loaded_cognition.latest_run_id.as_deref(), Some("run-1"));
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
