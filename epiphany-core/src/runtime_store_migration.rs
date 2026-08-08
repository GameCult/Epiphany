use anyhow::{Result, anyhow};
use cultcache_rs::{
    CacheBackingStore, CultCacheEnvelope, DatabaseEntry, RedbMessagePackBackingStore,
    SingleFileMessagePackBackingStore,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub const RUNTIME_STORE_MIGRATION_RECEIPT_TYPE: &str = "epiphany.runtime_store_migration_receipt";
pub const RUNTIME_STORE_MIGRATION_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.runtime_store_migration_receipt.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStoreMigrationReceipt {
    pub schema_version: String,
    pub migration_id: String,
    pub source_path: String,
    pub source_sha256: String,
    pub source_entry_count: usize,
    pub source_envelope_set_sha256: String,
    pub destination_path: String,
    pub migrated_at: String,
}

impl DatabaseEntry for RuntimeStoreMigrationReceipt {
    const TYPE: &'static str = RUNTIME_STORE_MIGRATION_RECEIPT_TYPE;
    const SCHEMA_NAME: &'static str = "RuntimeStoreMigrationReceipt";
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn sorted_envelopes(entries: &[CultCacheEnvelope]) -> Vec<CultCacheEnvelope> {
    let mut sorted = entries.to_vec();
    sorted.sort_by(|left, right| (&left.r#type, &left.key).cmp(&(&right.r#type, &right.key)));
    sorted
}

fn envelope_set_sha256(entries: &[CultCacheEnvelope]) -> Result<String> {
    Ok(sha256(&rmp_serde::to_vec_named(&sorted_envelopes(
        entries,
    ))?))
}

pub fn migrate_runtime_store_to_redb(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    migrated_at: &str,
) -> Result<RuntimeStoreMigrationReceipt> {
    chrono::DateTime::parse_from_rfc3339(migrated_at)
        .map_err(|_| anyhow!("migration timestamp must be RFC3339"))?;
    let source = source.as_ref();
    let destination = destination.as_ref();
    if source.extension().and_then(|value| value.to_str()) != Some("cc") {
        return Err(anyhow!("migration source must be a legacy .cc snapshot"));
    }
    if destination.extension().and_then(|value| value.to_str()) != Some("redb") {
        return Err(anyhow!("migration destination must use .redb"));
    }
    if destination.exists() {
        return Err(anyhow!("migration destination already exists"));
    }

    let source_bytes = fs::read(source)?;
    let source_sha256 = sha256(&source_bytes);
    let source_store = SingleFileMessagePackBackingStore::new(source);
    let entries = source_store.pull_all()?;
    let identities = entries
        .iter()
        .map(|entry| (&entry.r#type, &entry.key))
        .collect::<BTreeSet<_>>();
    if identities.len() != entries.len() {
        return Err(anyhow!(
            "legacy runtime snapshot contains duplicate typed identities"
        ));
    }
    if entries
        .iter()
        .any(|entry| entry.r#type == RUNTIME_STORE_MIGRATION_RECEIPT_TYPE)
    {
        return Err(anyhow!(
            "legacy runtime snapshot already contains a migration receipt"
        ));
    }

    let set_sha256 = envelope_set_sha256(&entries)?;
    let migration_id = format!("runtime-store-migration-{source_sha256}");
    let receipt = RuntimeStoreMigrationReceipt {
        schema_version: RUNTIME_STORE_MIGRATION_RECEIPT_SCHEMA_VERSION.to_string(),
        migration_id: migration_id.clone(),
        source_path: source.canonicalize()?.display().to_string(),
        source_sha256: source_sha256.clone(),
        source_entry_count: entries.len(),
        source_envelope_set_sha256: set_sha256.clone(),
        destination_path: PathBuf::from(destination).display().to_string(),
        migrated_at: migrated_at.to_string(),
    };
    let receipt_envelope = CultCacheEnvelope {
        key: migration_id,
        r#type: RUNTIME_STORE_MIGRATION_RECEIPT_TYPE.to_string(),
        payload: rmp_serde::to_vec_named(&receipt)?,
        stored_at: migrated_at.to_string(),
        schema_id: Some(RUNTIME_STORE_MIGRATION_RECEIPT_SCHEMA_VERSION.to_string()),
    };
    let destination_store = RedbMessagePackBackingStore::new(destination)?;
    let mut writes = entries.clone();
    writes.push(receipt_envelope.clone());
    if !destination_store.compare_and_swap_batch(&[], writes)? {
        return Err(anyhow!(
            "keyed runtime migration lost its empty-destination CAS"
        ));
    }
    let mut migrated = destination_store.pull_all()?;
    let persisted_receipt = migrated
        .iter()
        .find(|entry| entry.r#type == RUNTIME_STORE_MIGRATION_RECEIPT_TYPE)
        .cloned()
        .ok_or_else(|| anyhow!("keyed runtime migration lost its receipt"))?;
    if persisted_receipt != receipt_envelope {
        return Err(anyhow!(
            "keyed runtime migration receipt changed during persistence"
        ));
    }
    migrated.retain(|entry| entry.r#type != RUNTIME_STORE_MIGRATION_RECEIPT_TYPE);
    if sorted_envelopes(&migrated) != sorted_envelopes(&entries)
        || envelope_set_sha256(&migrated)? != set_sha256
    {
        return Err(anyhow!(
            "keyed runtime migration failed exact envelope equivalence"
        ));
    }
    if sha256(&fs::read(source)?) != source_sha256 {
        return Err(anyhow!("legacy runtime snapshot changed during migration"));
    }
    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_is_exact_seals_source_and_refuses_destination_reuse() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let source = temp.path().join("runtime.cc");
        let destination = temp.path().join("runtime.redb");
        let entry = CultCacheEnvelope {
            key: "state".to_string(),
            r#type: "epiphany.test".to_string(),
            payload: vec![1, 2, 3],
            stored_at: "2026-08-08T00:00:00Z".to_string(),
            schema_id: Some("epiphany.test".to_string()),
        };
        let source_store = SingleFileMessagePackBackingStore::new(&source);
        assert!(source_store.compare_and_swap_batch(&[], vec![entry.clone()])?);
        let before = fs::read(&source)?;
        let receipt = migrate_runtime_store_to_redb(&source, &destination, "2026-08-08T01:00:00Z")?;
        assert_eq!(receipt.source_entry_count, 1);
        assert_eq!(fs::read(&source)?, before);
        assert!(
            migrate_runtime_store_to_redb(&source, &destination, "2026-08-08T02:00:00Z").is_err()
        );
        let rows = RedbMessagePackBackingStore::new(&destination)?.pull_all()?;
        assert!(rows.contains(&entry));
        assert_eq!(
            rows.iter()
                .filter(|row| row.r#type == RUNTIME_STORE_MIGRATION_RECEIPT_TYPE)
                .count(),
            1
        );
        Ok(())
    }
}
