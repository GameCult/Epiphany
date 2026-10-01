use anyhow::{Result, anyhow};
use cultcache_rs::{
    CacheBackingStore, CultCacheEnvelope, PushAllOptions, RedbMessagePackBackingStore,
    SingleFileMessagePackBackingStore,
};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub(crate) struct RuntimeSpineBackingStore {
    path: PathBuf,
}

enum SelectedBackingStore {
    Snapshot(SingleFileMessagePackBackingStore),
    Keyed(RedbMessagePackBackingStore),
}

impl RuntimeSpineBackingStore {
    pub(crate) fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    fn selected(&self) -> Result<SelectedBackingStore> {
        match self
            .path
            .extension()
            .and_then(|extension| extension.to_str())
        {
            Some("cc" | "msgpack") => Ok(SelectedBackingStore::Snapshot(
                SingleFileMessagePackBackingStore::new(&self.path),
            )),
            Some("redb") => Ok(SelectedBackingStore::Keyed(
                RedbMessagePackBackingStore::new(&self.path)?,
            )),
            extension => Err(anyhow!(
                "runtime spine store must use an explicit .cc, .msgpack, or .redb extension; got {:?}",
                extension
            )),
        }
    }

    pub(crate) fn compare_and_swap_batch(
        &self,
        expected: &[CultCacheEnvelope],
        replacements: Vec<CultCacheEnvelope>,
    ) -> Result<bool> {
        match self.selected()? {
            SelectedBackingStore::Snapshot(store) => {
                store.compare_and_swap_batch(expected, replacements)
            }
            SelectedBackingStore::Keyed(store) => {
                store.compare_and_swap_batch(expected, replacements)
            }
        }
    }

    pub(crate) fn replace_and_delete_if_snapshot_unchanged(
        &self,
        expected_snapshot: &[CultCacheEnvelope],
        replacements: Vec<CultCacheEnvelope>,
        deletions: &[CultCacheEnvelope],
    ) -> Result<bool> {
        match self.selected()? {
            SelectedBackingStore::Snapshot(store) => store
                .replace_and_delete_if_snapshot_unchanged(
                    expected_snapshot,
                    replacements,
                    deletions,
                ),
            SelectedBackingStore::Keyed(store) => store.replace_and_delete_if_snapshot_unchanged(
                expected_snapshot,
                replacements,
                deletions,
            ),
        }
    }
}

impl CacheBackingStore for RuntimeSpineBackingStore {
    fn pull_all(&self) -> Result<Vec<CultCacheEnvelope>> {
        match self.selected()? {
            SelectedBackingStore::Snapshot(store) => store.pull_all(),
            SelectedBackingStore::Keyed(store) => store.pull_all(),
        }
    }

    fn push(&mut self, entry: &CultCacheEnvelope) -> Result<()> {
        match self.selected()? {
            SelectedBackingStore::Snapshot(mut store) => store.push(entry),
            SelectedBackingStore::Keyed(mut store) => store.push(entry),
        }
    }

    fn delete(&mut self, entry: &CultCacheEnvelope) -> Result<()> {
        match self.selected()? {
            SelectedBackingStore::Snapshot(mut store) => store.delete(entry),
            SelectedBackingStore::Keyed(mut store) => store.delete(entry),
        }
    }

    fn push_all(&mut self, entries: &[CultCacheEnvelope], options: PushAllOptions) -> Result<()> {
        match self.selected()? {
            SelectedBackingStore::Snapshot(mut store) => store.push_all(entries, options),
            SelectedBackingStore::Keyed(mut store) => store.push_all(entries, options),
        }
    }
}

pub(crate) fn runtime_spine_backing_store(path: &Path) -> Result<RuntimeSpineBackingStore> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("cc" | "msgpack" | "redb") => {
            #[cfg(test)]
            record_backing_store_resolution(path);
            Ok(RuntimeSpineBackingStore::new(path))
        }
        extension => Err(anyhow!(
            "runtime spine store must use an explicit .cc, .msgpack, or .redb extension; got {:?}",
            extension
        )),
    }
}

/// Every resolution this process has made, in order, recorded only under test.
///
/// A handle carries nothing but its path today, so a second handle to the same
/// path is behaviourally invisible and no assertion about files or contents can
/// see it. The `.redb` arm makes that difference real: redb permits one
/// writable handle per path. This log is the probe that lets a caller pin
/// "resolved once and reused" rather than the much weaker "only one store was
/// written".
#[cfg(test)]
static RESOLVED_BACKING_STORES: std::sync::Mutex<Vec<PathBuf>> =
    std::sync::Mutex::new(Vec::new());

#[cfg(test)]
fn record_backing_store_resolution(path: &Path) {
    RESOLVED_BACKING_STORES
        .lock()
        .expect("the backing-store resolution log is never poisoned")
        .push(path.to_path_buf());
}

/// How many times this path has been resolved into a handle. Tests run in
/// parallel, so a caller compares this around its own call against its own
/// temporary path rather than reading a global total.
#[cfg(test)]
pub(crate) fn backing_store_resolutions(path: &Path) -> usize {
    RESOLVED_BACKING_STORES
        .lock()
        .expect("the backing-store resolution log is never poisoned")
        .iter()
        .filter(|resolved| resolved.as_path() == path)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(payload: u8) -> CultCacheEnvelope {
        CultCacheEnvelope {
            key: "state".to_string(),
            r#type: "epiphany.test".to_string(),
            payload: vec![payload],
            stored_at: "2026-08-08T00:00:00Z".to_string(),
            schema_id: Some("epiphany.test".to_string()),
        }
    }

    #[test]
    fn explicit_extensions_select_one_backend_and_preserve_cas() -> Result<()> {
        let temp = tempfile::tempdir()?;
        for file_name in ["runtime.cc", "runtime.redb"] {
            let path = temp.path().join(file_name);
            let store = RuntimeSpineBackingStore::new(path);
            let first = envelope(1);
            let second = envelope(2);
            assert!(store.compare_and_swap_batch(&[], vec![first.clone()])?);
            assert_eq!(store.pull_all()?, vec![first.clone()], "{file_name}");
            assert!(store.compare_and_swap_batch(&[first.clone()], vec![second.clone()])?);
            assert!(!store.compare_and_swap_batch(&[first], vec![envelope(3)])?);
            assert_eq!(store.pull_all()?, vec![second]);
        }
        assert!(runtime_spine_backing_store(&temp.path().join("runtime.bin")).is_err());
        Ok(())
    }
}
