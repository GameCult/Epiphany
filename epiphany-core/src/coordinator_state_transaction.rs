use crate::EpiphanyThreadStateEntry;
use crate::THREAD_STATE_KEY;
use crate::coordinator_acceptance_cache;
use crate::read_accepted_coordinator_state;
use anyhow::{Result, anyhow};
use cultcache_rs::{CultCache, CultCacheEnvelope};
use epiphany_state_model::EpiphanyThreadState;
use std::path::Path;

pub(crate) fn open_coordinator_state_transaction(
    store: &Path,
    expected_state: &EpiphanyThreadState,
) -> Result<CultCache> {
    if let Some(persisted) = read_accepted_coordinator_state(store)?
        && persisted != *expected_state
    {
        return Err(anyhow!(
            "authoritative coordinator state changed before transaction commit"
        ));
    }
    coordinator_acceptance_cache(store)
}

pub(crate) fn commit_coordinator_state_transaction(
    cache: &mut CultCache,
    thread_id: &str,
    next_state: &EpiphanyThreadState,
    mut companion_envelopes: Vec<CultCacheEnvelope>,
) -> Result<EpiphanyThreadState> {
    if companion_envelopes
        .iter()
        .any(|envelope| envelope.key == THREAD_STATE_KEY)
    {
        return Err(anyhow!(
            "coordinator state transaction companions cannot write the canonical state key"
        ));
    }
    let state_entry = EpiphanyThreadStateEntry::from_state(thread_id, next_state)?;
    let (state_envelope, _) = cache.prepare_entry(THREAD_STATE_KEY, &state_entry)?;
    companion_envelopes.insert(0, state_envelope);
    cache.put_prepared_batch(companion_envelopes)?;
    Ok(next_state.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EpiphanyThreadStateEntry;

    #[test]
    fn companion_cannot_impersonate_canonical_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("state.cc");
        let state = EpiphanyThreadState::default();
        let mut cache = coordinator_acceptance_cache(&store)?;
        let counterfeit = EpiphanyThreadStateEntry::from_state("counterfeit", &state)?;
        let envelope = cache.prepare_entry(THREAD_STATE_KEY, &counterfeit)?.0;
        let error =
            commit_coordinator_state_transaction(&mut cache, "thread-1", &state, vec![envelope])
                .expect_err("companion state writer must be refused");
        assert!(error.to_string().contains("canonical state key"));
        assert!(read_accepted_coordinator_state(&store)?.is_none());
        Ok(())
    }

    #[test]
    fn canonical_state_key_has_no_second_production_writer() {
        for (name, source) in [
            ("coordinator_state", include_str!("coordinator_state.rs")),
            ("coordinator_launch", include_str!("coordinator_launch.rs")),
            (
                "coordinator_acceptance",
                include_str!("coordinator_acceptance.rs"),
            ),
            ("thread_state_store", include_str!("thread_state_store.rs")),
        ] {
            let production = source.split("#[cfg(test)]").next().unwrap_or(source);
            assert!(
                !production.contains("prepare_entry(THREAD_STATE_KEY")
                    && !production.contains("put(THREAD_STATE_KEY"),
                "{name} regained canonical state write authority"
            );
        }
        let public_surface = include_str!("lib.rs");
        assert!(!public_surface.contains("write_thread_state_entry"));
        assert!(!public_surface.contains("write_thread_state;"));
        assert!(!public_surface.contains("commit_state_with_mind_witness"));
    }
}
