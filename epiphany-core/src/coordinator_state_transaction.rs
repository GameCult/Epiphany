use crate::{
    EpiphanyThreadStateEntry, THREAD_STATE_KEY, THREAD_STATE_TYPE, coordinator_acceptance_cache,
};
use anyhow::{Result, anyhow};
use cultcache_rs::{
    CacheBackingStore, CultCache, CultCacheEnvelope, SingleFileMessagePackBackingStore,
};
use epiphany_state_model::EpiphanyThreadState;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

pub(crate) struct CoordinatorStateTransaction {
    cache: CultCache,
    store: PathBuf,
    opening_snapshot: Vec<CultCacheEnvelope>,
    expected_state_envelope: Option<CultCacheEnvelope>,
}

impl Deref for CoordinatorStateTransaction {
    type Target = CultCache;

    fn deref(&self) -> &Self::Target {
        &self.cache
    }
}

impl DerefMut for CoordinatorStateTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cache
    }
}

pub(crate) fn open_coordinator_state_transaction(
    store: &Path,
    expected_state: &EpiphanyThreadState,
) -> Result<CoordinatorStateTransaction> {
    let cache = coordinator_acceptance_cache(store)?;
    let opening_snapshot = cache.snapshot_envelopes();
    let expected_state_envelope = opening_snapshot
        .iter()
        .find(|entry| entry.r#type == THREAD_STATE_TYPE && entry.key == THREAD_STATE_KEY)
        .cloned();
    if let Some(envelope) = &expected_state_envelope {
        let entry: EpiphanyThreadStateEntry = rmp_serde::from_slice(&envelope.payload)?;
        if entry.state()? != *expected_state {
            return Err(anyhow!(
                "authoritative coordinator state changed before transaction opened"
            ));
        }
    }
    Ok(CoordinatorStateTransaction {
        cache,
        store: store.to_path_buf(),
        opening_snapshot,
        expected_state_envelope,
    })
}

fn same_immutable_content(left: &CultCacheEnvelope, right: &CultCacheEnvelope) -> bool {
    left.r#type == right.r#type
        && left.key == right.key
        && left.payload == right.payload
        && left.schema_id == right.schema_id
}

pub(crate) fn commit_coordinator_state_transaction(
    transaction: &mut CoordinatorStateTransaction,
    thread_id: &str,
    next_state: &EpiphanyThreadState,
    companion_envelopes: Vec<CultCacheEnvelope>,
    captured_replacements: Vec<CultCacheEnvelope>,
) -> Result<EpiphanyThreadState> {
    if companion_envelopes
        .iter()
        .chain(&captured_replacements)
        .any(|envelope| envelope.r#type == THREAD_STATE_TYPE && envelope.key == THREAD_STATE_KEY)
    {
        return Err(anyhow!(
            "coordinator state transaction companions cannot write the canonical state identity"
        ));
    }

    let state_entry = EpiphanyThreadStateEntry::from_state(thread_id, next_state)?;
    let (state_envelope, _) = transaction
        .cache
        .prepare_entry(THREAD_STATE_KEY, &state_entry)?;
    let mut expected = transaction
        .expected_state_envelope
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let mut replacements = vec![state_envelope];

    for companion in companion_envelopes {
        let opening = transaction
            .opening_snapshot
            .iter()
            .find(|entry| entry.r#type == companion.r#type && entry.key == companion.key);
        if let Some(existing) = opening {
            if !same_immutable_content(existing, &companion) {
                return Err(anyhow!(
                    "coordinator state transaction companion identity collision at ({:?}, {:?})",
                    companion.r#type,
                    companion.key
                ));
            }
            expected.push(existing.clone());
            replacements.push(existing.clone());
        } else {
            replacements.push(companion);
        }
    }
    for replacement in captured_replacements {
        let existing = transaction
            .opening_snapshot
            .iter()
            .find(|entry| entry.r#type == replacement.r#type && entry.key == replacement.key)
            .ok_or_else(|| {
                anyhow!(
                    "coordinator state transaction captured replacement is absent at ({:?}, {:?})",
                    replacement.r#type,
                    replacement.key
                )
            })?;
        expected.push(existing.clone());
        replacements.push(replacement);
    }

    let backing = SingleFileMessagePackBackingStore::new(&transaction.store);
    if !backing.compare_and_swap_batch(&expected, replacements)? {
        return Err(anyhow!(
            "coordinator state transaction lost its exact atomic compare-and-swap"
        ));
    }
    Ok(next_state.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn companion(
        transaction: &mut CoordinatorStateTransaction,
        key: &str,
        thread_id: &str,
        state: &EpiphanyThreadState,
    ) -> Result<CultCacheEnvelope> {
        let entry = EpiphanyThreadStateEntry::from_state(thread_id, state)?;
        Ok(transaction.prepare_entry(key, &entry)?.0)
    }

    #[test]
    fn companion_cannot_impersonate_canonical_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("state.cc");
        let state = EpiphanyThreadState::default();
        let mut transaction = open_coordinator_state_transaction(&store, &state)?;
        let counterfeit = EpiphanyThreadStateEntry::from_state("counterfeit", &state)?;
        let envelope = transaction.prepare_entry(THREAD_STATE_KEY, &counterfeit)?.0;
        let error = commit_coordinator_state_transaction(
            &mut transaction,
            "thread-1",
            &state,
            vec![envelope],
            Vec::new(),
        )
        .expect_err("companion state writer must be refused");
        assert!(error.to_string().contains("canonical state identity"));
        assert!(
            SingleFileMessagePackBackingStore::new(&store)
                .pull_all()?
                .is_empty()
        );
        Ok(())
    }

    #[test]
    fn transactions_opened_on_the_same_absent_state_have_one_winner() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("state.cc");
        let state = EpiphanyThreadState::default();
        let mut first = open_coordinator_state_transaction(&store, &state)?;
        let mut second = open_coordinator_state_transaction(&store, &state)?;

        commit_coordinator_state_transaction(
            &mut first,
            "thread-1",
            &state,
            Vec::new(),
            Vec::new(),
        )?;
        let error = commit_coordinator_state_transaction(
            &mut second,
            "thread-1",
            &state,
            Vec::new(),
            Vec::new(),
        )
        .expect_err("only one transaction may seed an absent canonical identity");
        assert!(error.to_string().contains("compare-and-swap"));
        Ok(())
    }

    #[test]
    fn absent_store_accepts_an_imported_nondefault_seed_once() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("state.cc");
        let mut imported = EpiphanyThreadState::default();
        imported.revision = 9;
        let mut transaction = open_coordinator_state_transaction(&store, &imported)?;
        commit_coordinator_state_transaction(
            &mut transaction,
            "thread-import",
            &imported,
            Vec::new(),
            Vec::new(),
        )?;
        Ok(())
    }

    #[test]
    fn identical_companion_retry_preserves_the_persisted_envelope() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("state.cc");
        let state = EpiphanyThreadState::default();
        let mut first = open_coordinator_state_transaction(&store, &state)?;
        let first_companion = companion(&mut first, "immutable-companion", "thread-1", &state)?;
        commit_coordinator_state_transaction(
            &mut first,
            "thread-1",
            &state,
            vec![first_companion],
            Vec::new(),
        )?;
        let backing = SingleFileMessagePackBackingStore::new(&store);
        let persisted_before = backing
            .pull_all()?
            .into_iter()
            .find(|entry| entry.key == "immutable-companion")
            .expect("companion persisted");

        let mut retry = open_coordinator_state_transaction(&store, &state)?;
        let retry_companion = companion(&mut retry, "immutable-companion", "thread-1", &state)?;
        commit_coordinator_state_transaction(
            &mut retry,
            "thread-1",
            &state,
            vec![retry_companion],
            Vec::new(),
        )?;
        let persisted_after = backing
            .pull_all()?
            .into_iter()
            .find(|entry| entry.key == "immutable-companion")
            .expect("companion remains persisted");
        assert_eq!(persisted_after, persisted_before);
        Ok(())
    }

    #[test]
    fn conflicting_companion_retry_writes_nothing() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("state.cc");
        let state = EpiphanyThreadState::default();
        let mut first = open_coordinator_state_transaction(&store, &state)?;
        let first_companion = companion(&mut first, "immutable-companion", "thread-1", &state)?;
        commit_coordinator_state_transaction(
            &mut first,
            "thread-1",
            &state,
            vec![first_companion],
            Vec::new(),
        )?;
        let bytes_before = std::fs::read(&store)?;

        let mut collision = open_coordinator_state_transaction(&store, &state)?;
        let conflicting = companion(
            &mut collision,
            "immutable-companion",
            "different-thread",
            &state,
        )?;
        let error = commit_coordinator_state_transaction(
            &mut collision,
            "thread-1",
            &state,
            vec![conflicting],
            Vec::new(),
        )
        .expect_err("immutable companion identity collision must be refused");
        assert!(error.to_string().contains("identity collision"));
        assert_eq!(std::fs::read(&store)?, bytes_before);
        Ok(())
    }

    #[test]
    fn captured_replacement_has_one_exact_snapshot_winner() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("state.cc");
        let state = EpiphanyThreadState::default();
        let mut seed = open_coordinator_state_transaction(&store, &state)?;
        let original = companion(&mut seed, "mutable-companion", "original", &state)?;
        commit_coordinator_state_transaction(
            &mut seed,
            "thread-1",
            &state,
            vec![original],
            Vec::new(),
        )?;

        let mut first = open_coordinator_state_transaction(&store, &state)?;
        let mut second = open_coordinator_state_transaction(&store, &state)?;
        let first_replacement = companion(&mut first, "mutable-companion", "winner", &state)?;
        let second_replacement = companion(&mut second, "mutable-companion", "loser", &state)?;
        commit_coordinator_state_transaction(
            &mut first,
            "thread-1",
            &state,
            Vec::new(),
            vec![first_replacement],
        )?;
        let error = commit_coordinator_state_transaction(
            &mut second,
            "thread-1",
            &state,
            Vec::new(),
            vec![second_replacement],
        )
        .expect_err("a captured replacement cannot overwrite a newer envelope");
        assert!(error.to_string().contains("compare-and-swap"));

        let cache = coordinator_acceptance_cache(&store)?;
        let persisted = cache.get_required::<EpiphanyThreadStateEntry>("mutable-companion")?;
        assert_eq!(persisted.thread_id, "winner");
        Ok(())
    }

    #[test]
    fn companion_identity_is_polymorphic_type_and_key() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("state.cc");
        let state = EpiphanyThreadState::default();
        let mut transaction = open_coordinator_state_transaction(&store, &state)?;
        let typed = companion(&mut transaction, "shared-key", "thread-1", &state)?;
        let other_type = CultCacheEnvelope {
            key: "shared-key".into(),
            r#type: "test.other_type".into(),
            payload: vec![1, 2, 3],
            stored_at: "2026-07-14T00:00:00Z".into(),
            schema_id: Some("test.other_type.v0".into()),
        };
        commit_coordinator_state_transaction(
            &mut transaction,
            "thread-1",
            &state,
            vec![typed, other_type],
            Vec::new(),
        )?;
        let shared = SingleFileMessagePackBackingStore::new(&store)
            .pull_all()?
            .into_iter()
            .filter(|entry| entry.key == "shared-key")
            .collect::<Vec<_>>();
        assert_eq!(shared.len(), 2);
        assert_ne!(shared[0].r#type, shared[1].r#type);
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
