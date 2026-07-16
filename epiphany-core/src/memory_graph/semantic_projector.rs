use super::{
    MEMORY_SEMANTIC_PROJECTION_ATTEMPT_SCHEMA_VERSION, MemorySemanticIndexReceipt,
    MemorySemanticProjectionAttempt, MemorySemanticProjectionObligation,
    MemorySemanticProjectionSourceHead, bind_memory_semantic_index_receipt,
    memory_semantic_projection_terminal_success, validate_memory_semantic_projection_attempt,
    validate_memory_semantic_projection_obligation,
};
use anyhow::{Result, anyhow};
use cultcache_rs::{
    CacheBackingStore, CultCache, CultCacheEnvelope, DatabaseEntry,
    SingleFileMessagePackBackingStore,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

pub const MEMORY_SEMANTIC_PROJECTION_CLAIM_SCHEMA_VERSION: &str =
    "gamecult.epiphany.memory_semantic_projection_claim.v1";
pub const MEMORY_SEMANTIC_PROJECTOR_EXECUTOR_GRANT_SCHEMA_VERSION: &str =
    "gamecult.epiphany.memory_semantic_projector_executor_grant.v1";
pub const MEMORY_SEMANTIC_PROJECTOR_RECOVERY_AUTHORIZATION_SCHEMA_VERSION: &str =
    "gamecult.epiphany.memory_semantic_projector_recovery_authorization.v2";

#[derive(Deserialize, Serialize)]
struct LegacyMemorySemanticProjectionAttemptV0 {
    schema_version: String,
    attempt_id: String,
    obligation_id: String,
    started_at: String,
    completed_at: Option<String>,
    status: String,
    error: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct LegacyMemorySemanticProjectionClaimV0 {
    schema_version: String,
    scope_id: String,
    claim_id: String,
    obligation_id: String,
    attempt_id: String,
    executor_id: String,
    epoch: u64,
    status: String,
    claimed_at: String,
    completed_at: Option<String>,
}

/// Retires pre-Idunn claim/attempt pairs from active authority. Their old
/// receipts remain historical and query-ineligible; current obligations can
/// then receive fresh fenced work.
pub fn retire_memory_semantic_projection_claims_v0(
    store_path: impl AsRef<Path>,
) -> Result<Vec<String>> {
    const ATTEMPT_TYPE: &str = "gamecult.epiphany.memory_semantic_projection_attempt";
    const CLAIM_TYPE: &str = "gamecult.epiphany.memory_semantic_projection_claim";
    let backing = SingleFileMessagePackBackingStore::new(store_path.as_ref());
    let opening = backing.pull_all()?;
    let mut retired = Vec::new();
    for claim_envelope in opening.iter().filter(|entry| entry.r#type == CLAIM_TYPE) {
        if rmp_serde::from_slice::<MemorySemanticProjectionClaim>(&claim_envelope.payload).is_ok() {
            continue;
        }
        let claim = rmp_serde::from_slice::<LegacyMemorySemanticProjectionClaimV0>(
            &claim_envelope.payload,
        )?;
        if claim.schema_version != "gamecult.epiphany.memory_semantic_projection_claim.v0" {
            return Err(anyhow!(
                "unsupported legacy semantic claim schema for {:?}",
                claim.claim_id
            ));
        }
        let attempt_envelope = opening
            .iter()
            .find(|entry| entry.r#type == ATTEMPT_TYPE && entry.key == claim.attempt_id)
            .ok_or_else(|| {
                anyhow!(
                    "legacy semantic claim {:?} has no exact attempt",
                    claim.claim_id
                )
            })?;
        let attempt = rmp_serde::from_slice::<LegacyMemorySemanticProjectionAttemptV0>(
            &attempt_envelope.payload,
        )?;
        if attempt.attempt_id != claim.attempt_id || attempt.obligation_id != claim.obligation_id {
            return Err(anyhow!("legacy semantic claim/attempt pair disagrees"));
        }
        if !backing
            .delete_batch_if_unchanged(&[claim_envelope.clone(), attempt_envelope.clone()])?
        {
            return Err(anyhow!(
                "legacy semantic authority changed during explicit retirement"
            ));
        }
        retired.push(claim.claim_id);
    }
    Ok(retired)
}

/// Retires historical v0 attempts whose scope claim was already superseded or
/// overwritten before fenced claim history existed.
pub fn retire_orphaned_memory_semantic_projection_attempts_v0(
    store_path: impl AsRef<Path>,
) -> Result<Vec<String>> {
    const ATTEMPT_TYPE: &str = "gamecult.epiphany.memory_semantic_projection_attempt";
    const CLAIM_TYPE: &str = "gamecult.epiphany.memory_semantic_projection_claim";
    let backing = SingleFileMessagePackBackingStore::new(store_path.as_ref());
    let opening = backing.pull_all()?;
    let current_claim_attempt_ids = opening
        .iter()
        .filter(|entry| entry.r#type == CLAIM_TYPE)
        .filter_map(|entry| {
            rmp_serde::from_slice::<MemorySemanticProjectionClaim>(&entry.payload)
                .ok()
                .map(|claim| claim.attempt_id)
        })
        .collect::<std::collections::BTreeSet<_>>();
    let mut retired = Vec::new();
    for attempt_envelope in opening.iter().filter(|entry| entry.r#type == ATTEMPT_TYPE) {
        if rmp_serde::from_slice::<MemorySemanticProjectionAttempt>(&attempt_envelope.payload)
            .is_ok()
        {
            continue;
        }
        let attempt = rmp_serde::from_slice::<LegacyMemorySemanticProjectionAttemptV0>(
            &attempt_envelope.payload,
        )?;
        if attempt.schema_version != "gamecult.epiphany.memory_semantic_projection_attempt.v0" {
            return Err(anyhow!(
                "unsupported orphaned semantic attempt schema for {:?}",
                attempt.attempt_id
            ));
        }
        if current_claim_attempt_ids.contains(&attempt.attempt_id) {
            continue;
        }
        if !backing.delete_batch_if_unchanged(std::slice::from_ref(attempt_envelope))? {
            return Err(anyhow!(
                "orphaned semantic attempt changed during explicit retirement"
            ));
        }
        retired.push(attempt.attempt_id);
    }
    Ok(retired)
}

/// Retires index receipts whose physical claim namespace is absent from the
/// canonical store. Their Qdrant points are rebuildable cache residue.
pub fn retire_unowned_memory_semantic_index_receipts(
    store_path: impl AsRef<Path>,
) -> Result<Vec<String>> {
    const RECEIPT_TYPE: &str = "gamecult.epiphany.memory_semantic_index_receipt";
    const CLAIM_TYPE: &str = "gamecult.epiphany.memory_semantic_projection_claim";
    let backing = SingleFileMessagePackBackingStore::new(store_path.as_ref());
    let opening = backing.pull_all()?;
    let claim_ids = opening
        .iter()
        .filter(|entry| entry.r#type == CLAIM_TYPE)
        .filter_map(|entry| {
            rmp_serde::from_slice::<MemorySemanticProjectionClaim>(&entry.payload)
                .ok()
                .map(|claim| claim.claim_id)
        })
        .collect::<std::collections::BTreeSet<_>>();
    let mut retired = Vec::new();
    for envelope in opening.iter().filter(|entry| entry.r#type == RECEIPT_TYPE) {
        let receipt = rmp_serde::from_slice::<MemorySemanticIndexReceipt>(&envelope.payload)?;
        if !receipt.claim_id.trim().is_empty()
            && receipt.claim_epoch > 0
            && claim_ids.contains(&receipt.claim_id)
        {
            continue;
        }
        if !backing.delete_batch_if_unchanged(std::slice::from_ref(envelope))? {
            return Err(anyhow!(
                "unowned semantic index receipt changed during explicit retirement"
            ));
        }
        retired.push(receipt.receipt_id);
    }
    Ok(retired)
}

/// Upgrades v0 attempt rows by binding them to the exact persisted claim that
/// already owns their attempt identity. No claim means no migration.
pub fn migrate_memory_semantic_projection_attempts_v0(
    store_path: impl AsRef<Path>,
) -> Result<Vec<String>> {
    const ATTEMPT_TYPE: &str = "gamecult.epiphany.memory_semantic_projection_attempt";
    const CLAIM_TYPE: &str = "gamecult.epiphany.memory_semantic_projection_claim";
    let backing = SingleFileMessagePackBackingStore::new(store_path.as_ref());
    let opening = backing.pull_all()?;
    let claims = opening
        .iter()
        .filter(|entry| entry.r#type == CLAIM_TYPE)
        .map(|entry| rmp_serde::from_slice::<MemorySemanticProjectionClaim>(&entry.payload))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut migrated = Vec::new();
    for envelope in opening.iter().filter(|entry| entry.r#type == ATTEMPT_TYPE) {
        if rmp_serde::from_slice::<MemorySemanticProjectionAttempt>(&envelope.payload).is_ok() {
            continue;
        }
        let legacy =
            rmp_serde::from_slice::<LegacyMemorySemanticProjectionAttemptV0>(&envelope.payload)?;
        if legacy.schema_version != "gamecult.epiphany.memory_semantic_projection_attempt.v0" {
            return Err(anyhow!(
                "unsupported legacy semantic attempt schema for {:?}",
                legacy.attempt_id
            ));
        }
        let claim = claims
            .iter()
            .find(|claim| {
                claim.attempt_id == legacy.attempt_id && claim.obligation_id == legacy.obligation_id
            })
            .ok_or_else(|| {
                anyhow!(
                    "legacy semantic attempt {:?} has no exact owning claim",
                    legacy.attempt_id
                )
            })?;
        let upgraded = MemorySemanticProjectionAttempt {
            schema_version: MEMORY_SEMANTIC_PROJECTION_ATTEMPT_SCHEMA_VERSION.to_string(),
            attempt_id: legacy.attempt_id,
            obligation_id: legacy.obligation_id,
            started_at: legacy.started_at,
            completed_at: legacy.completed_at,
            status: legacy.status,
            error: legacy.error,
            claim_id: claim.claim_id.clone(),
            claim_epoch: claim.epoch,
            executor_id: claim.executor_id.clone(),
            executor_incarnation: claim.executor_incarnation.clone(),
            authority_id: claim.authority_id.clone(),
        };
        validate_memory_semantic_projection_attempt(&upgraded)?;
        let mut replacement: CultCacheEnvelope = envelope.clone();
        replacement.payload = rmp_serde::to_vec_named(&upgraded)?;
        if !backing.compare_and_swap_entry(envelope, replacement)? {
            return Err(anyhow!(
                "semantic attempt changed during explicit migration"
            ));
        }
        migrated.push(upgraded.attempt_id);
    }
    Ok(migrated)
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.memory_semantic_projector_executor_grant",
    schema = "MemorySemanticProjectorExecutorGrant"
)]
pub struct MemorySemanticProjectorExecutorGrant {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub grant_id: String,
    #[cultcache(key = 2)]
    pub scope_id: String,
    #[cultcache(key = 3)]
    pub swarm_id: String,
    #[cultcache(key = 4)]
    pub partition: String,
    #[cultcache(key = 5)]
    pub obligation_id: String,
    #[cultcache(key = 6)]
    pub executor_id: String,
    #[cultcache(key = 7)]
    pub executor_incarnation: String,
    #[cultcache(key = 8)]
    pub purpose: String,
    #[cultcache(key = 9)]
    pub issuer_id: String,
    #[cultcache(key = 10)]
    pub issuer_incarnation: String,
    #[cultcache(key = 11)]
    pub status: String,
    #[cultcache(key = 12)]
    pub issued_at: String,
    #[cultcache(key = 13)]
    pub consumed_at: Option<String>,
    #[cultcache(key = 14)]
    pub resulting_claim_id: Option<String>,
    #[cultcache(key = 15)]
    pub resulting_claim_epoch: Option<u64>,
    #[cultcache(key = 16)]
    pub predecessor_claim_id: Option<String>,
    #[cultcache(key = 17)]
    pub predecessor_claim_epoch: Option<u64>,
    #[cultcache(key = 18)]
    pub predecessor_claim_status: String,
}

#[cfg(test)]
mod authority_tests {
    use super::*;
    use crate::{
        MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION, SEMANTIC_PROJECTION_SCHEMA_VERSION,
    };
    use anyhow::Context;
    use tempfile::tempdir;

    fn obligation() -> MemorySemanticProjectionObligation {
        MemorySemanticProjectionObligation {
            schema_version: MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION.to_string(),
            obligation_id: "obligation-modeling-7".into(),
            swarm_id: "swarm-a".into(),
            partition: "modeling".into(),
            canonical_source_id: "runtime/repo-model".into(),
            source_commit_id: "admission-7".into(),
            graph_id: "repo-model".into(),
            source_generation: 7,
            source_model_hash: "model-hash-7".into(),
            canonical_content_set_hash: "content-set-7".into(),
            projection_schema_version: SEMANTIC_PROJECTION_SCHEMA_VERSION.into(),
            created_at: "2026-07-15T04:00:00Z".into(),
        }
    }

    fn input(
        store: &Path,
        obligation: &MemorySemanticProjectionObligation,
    ) -> Result<MemorySemanticProjectionInput> {
        let envelopes = SingleFileMessagePackBackingStore::new(store).pull_all()?;
        let authority = exact_envelope(
            &envelopes,
            MemorySemanticProjectionObligation::TYPE,
            &obligation.obligation_id,
        )?;
        Ok(MemorySemanticProjectionInput {
            snapshot: super::super::EpiphanyMemoryGraphSnapshot {
                schema_version: Some("v0".to_string()),
                graph_id: obligation.graph_id.clone(),
                model_revision: obligation.source_generation,
                ..Default::default()
            },
            obligation: obligation.clone(),
            authority: MemorySemanticProjectionAuthoritySnapshot {
                head: MemorySemanticProjectionSourceHead {
                    swarm_id: obligation.swarm_id.clone(),
                    partition: obligation.partition.clone(),
                    canonical_source_id: obligation.canonical_source_id.clone(),
                    source_commit_id: obligation.source_commit_id.clone(),
                    graph_id: obligation.graph_id.clone(),
                    source_generation: obligation.source_generation,
                    source_model_hash: obligation.source_model_hash.clone(),
                    canonical_content_set_hash: obligation.canonical_content_set_hash.clone(),
                },
                envelopes: vec![authority],
            },
        })
    }

    #[test]
    fn empty_projection_terminal_succeeds_once_without_becoming_query_eligible() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("empty-terminal.cc");
        let mut obligation = obligation();
        obligation.canonical_content_set_hash = format!("{:x}", Sha256::digest([]));
        let mut cache = semantic_projector_cache(&store)?;
        cache.put(&obligation.obligation_id, &obligation)?;
        let input = input(&store, &obligation)?;
        let acquisition = idunn_acquire_memory_semantic_projection(
            &store,
            &input,
            "executor-a",
            "executor-incarnation-a",
            "execute",
            "idunn-a",
            "2026-07-15T04:01:00Z",
        )?;
        let receipt = MemorySemanticIndexReceipt {
            schema_version: crate::MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "empty-receipt".into(),
            swarm_id: obligation.swarm_id.clone(),
            partition: obligation.partition.clone(),
            collection_name: "epiphany_modeling_v1".into(),
            graph_id: obligation.graph_id.clone(),
            model_revision: obligation.source_generation,
            model_hash: obligation.source_model_hash.clone(),
            embedding_provider_id: "provider".into(),
            embedding_model: "model".into(),
            vector_dimensions: 0,
            indexed_document_count: 0,
            deleted_document_count: 0,
            canonical_content_set_hash: obligation.canonical_content_set_hash.clone(),
            indexed_at: "2026-07-15T04:02:00Z".into(),
            status: "ready".into(),
            obligation_id: obligation.obligation_id.clone(),
            canonical_source_id: obligation.canonical_source_id.clone(),
            source_commit_id: obligation.source_commit_id.clone(),
            source_generation: obligation.source_generation,
            projection_schema_version: obligation.projection_schema_version.clone(),
            claim_id: acquisition.claim.claim_id.clone(),
            claim_epoch: acquisition.claim.epoch,
            observed_vector_binding_root_sha256: format!("{:x}", Sha256::digest([])),
        };
        succeed_memory_semantic_projection_claim(
            &store,
            &acquisition.claim.claim_id,
            &input.authority,
            receipt,
            "2026-07-15T04:02:01Z",
        )?;
        let readiness = load_memory_semantic_projection_readiness(&store, &input)?
            .expect("terminal empty receipt remains authenticated");
        assert!(!super::super::memory_semantic_projection_query_eligible(
            &obligation,
            &input.authority.head,
            &readiness.receipt
        ));
        assert_eq!(
            classify_memory_semantic_projection_for_pulse(&store, &input)?,
            crate::MemorySemanticProjectorPulseClassification::Ready
        );
        assert!(
            idunn_acquire_memory_semantic_projection(
                &store,
                &input,
                "executor-b",
                "executor-incarnation-b",
                "execute",
                "idunn-b",
                "2026-07-15T04:03:00Z",
            )
            .is_err()
        );
        let mut legacy = readiness.receipt;
        legacy.schema_version = "gamecult.epiphany.memory_semantic_index_receipt.v1".into();
        legacy.observed_vector_binding_root_sha256.clear();
        let mut cache = semantic_projector_cache(&store)?;
        cache.put(&legacy.receipt_id, &legacy)?;
        assert_eq!(
            classify_memory_semantic_projection_for_pulse(&store, &input)?,
            crate::MemorySemanticProjectorPulseClassification::Repair
        );
        let scope_id = projection_scope_id(&obligation.swarm_id, &obligation.partition)?;
        cache.pull_all_backing_stores()?;
        let current = cache
            .get::<MemorySemanticProjectionClaim>(&scope_id)?
            .expect("succeeded predecessor");
        let mut forged = current.clone();
        forged.authority_id = "missing-consumed-authority".into();
        cache.put(&scope_id, &forged)?;
        assert!(classify_memory_semantic_projection_for_pulse(&store, &input).is_err());
        assert!(
            idunn_acquire_memory_semantic_projection(
                &store,
                &input,
                "executor-b",
                "executor-incarnation-b",
                "repair",
                "idunn-b",
                "2026-07-15T04:03:00Z",
            )
            .is_err()
        );
        cache.put(&scope_id, &current)?;
        let repair = idunn_acquire_memory_semantic_projection(
            &store,
            &input,
            "executor-b",
            "executor-incarnation-b",
            "repair",
            "idunn-b",
            "2026-07-15T04:03:00Z",
        )?;
        assert_eq!(repair.claim.epoch, acquisition.claim.epoch + 1);
        fail_memory_semantic_projection_claim(
            &store,
            &repair.claim.claim_id,
            "2026-07-15T04:04:00Z",
            "transient embedding failure",
        )?;
        assert_eq!(
            classify_memory_semantic_projection_for_pulse(&store, &input)?,
            crate::MemorySemanticProjectorPulseClassification::Failed
        );
        let retry = idunn_acquire_memory_semantic_projection(
            &store,
            &input,
            "executor-c",
            "executor-incarnation-c",
            "execute",
            "idunn-c",
            "2026-07-15T04:05:00Z",
        )?;
        assert_eq!(retry.claim.epoch, repair.claim.epoch + 1);
        let migrated = MemorySemanticIndexReceipt {
            schema_version: crate::MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "empty-receipt-v2".into(),
            indexed_at: "2026-07-15T04:06:00Z".into(),
            claim_id: retry.claim.claim_id.clone(),
            claim_epoch: retry.claim.epoch,
            observed_vector_binding_root_sha256: format!("{:x}", Sha256::digest([])),
            ..legacy
        };
        succeed_memory_semantic_projection_claim(
            &store,
            &retry.claim.claim_id,
            &input.authority,
            migrated,
            "2026-07-15T04:06:01Z",
        )?;
        assert_eq!(
            classify_memory_semantic_projection_for_pulse(&store, &input)?,
            crate::MemorySemanticProjectorPulseClassification::Ready
        );
        Ok(())
    }

    #[test]
    fn legacy_attempt_migration_binds_exact_owning_claim() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("legacy-attempt.msgpack");
        let obligation = obligation();
        let mut cache = semantic_projector_cache(&store)?;
        cache.put(&obligation.obligation_id, &obligation)?;
        let projection_input = input(&store, &obligation)?;
        let acquisition = idunn_acquire_memory_semantic_projection(
            &store,
            &projection_input,
            "executor-a",
            "executor-a-incarnation",
            "execute",
            "idunn-incarnation-a",
            "2026-07-15T04:01:00Z",
        )?;
        let backing = SingleFileMessagePackBackingStore::new(&store);
        let opening = backing.pull_all()?;
        let attempt_envelope = opening
            .iter()
            .find(|entry| {
                entry.r#type == MemorySemanticProjectionAttempt::TYPE
                    && entry.key == acquisition.claim.attempt_id
            })
            .context("attempt missing")?;
        let attempt: MemorySemanticProjectionAttempt =
            rmp_serde::from_slice(&attempt_envelope.payload)?;
        let legacy = LegacyMemorySemanticProjectionAttemptV0 {
            schema_version: "gamecult.epiphany.memory_semantic_projection_attempt.v0".into(),
            attempt_id: attempt.attempt_id.clone(),
            obligation_id: attempt.obligation_id.clone(),
            started_at: attempt.started_at,
            completed_at: attempt.completed_at,
            status: attempt.status,
            error: attempt.error,
        };
        let mut legacy_envelope = attempt_envelope.clone();
        legacy_envelope.payload = rmp_serde::to_vec_named(&legacy)?;
        assert!(backing.compare_and_swap_entry(attempt_envelope, legacy_envelope)?);

        assert_eq!(
            migrate_memory_semantic_projection_attempts_v0(&store)?,
            vec![attempt.attempt_id.clone()]
        );
        let (_, _, _, upgraded) = load_running_claim(&store, &acquisition.claim.claim_id)?;
        assert_eq!(upgraded.claim_id, acquisition.claim.claim_id);
        assert_eq!(upgraded.claim_epoch, acquisition.claim.epoch);
        assert_eq!(upgraded.executor_id, acquisition.claim.executor_id);
        assert_eq!(
            upgraded.executor_incarnation,
            acquisition.claim.executor_incarnation
        );
        assert_eq!(upgraded.authority_id, acquisition.claim.authority_id);
        assert!(migrate_memory_semantic_projection_attempts_v0(&store)?.is_empty());
        Ok(())
    }

    #[test]
    fn legacy_unfenced_claim_and_attempt_retire_atomically() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("legacy-claim.msgpack");
        let obligation = obligation();
        let mut cache = semantic_projector_cache(&store)?;
        cache.put(&obligation.obligation_id, &obligation)?;
        let projection_input = input(&store, &obligation)?;
        let acquisition = idunn_acquire_memory_semantic_projection(
            &store,
            &projection_input,
            "executor-a",
            "executor-a-incarnation",
            "execute",
            "idunn-incarnation-a",
            "2026-07-15T04:01:00Z",
        )?;
        let backing = SingleFileMessagePackBackingStore::new(&store);
        let opening = backing.pull_all()?;
        let claim_envelope = exact_envelope(
            &opening,
            MemorySemanticProjectionClaim::TYPE,
            &acquisition.claim.scope_id,
        )?;
        let attempt_envelope = exact_envelope(
            &opening,
            MemorySemanticProjectionAttempt::TYPE,
            &acquisition.claim.attempt_id,
        )?;
        let legacy_claim = LegacyMemorySemanticProjectionClaimV0 {
            schema_version: "gamecult.epiphany.memory_semantic_projection_claim.v0".into(),
            scope_id: acquisition.claim.scope_id.clone(),
            claim_id: acquisition.claim.claim_id.clone(),
            obligation_id: acquisition.claim.obligation_id.clone(),
            attempt_id: acquisition.claim.attempt_id.clone(),
            executor_id: acquisition.claim.executor_id.clone(),
            epoch: acquisition.claim.epoch,
            status: acquisition.claim.status.clone(),
            claimed_at: acquisition.claim.claimed_at.clone(),
            completed_at: acquisition.claim.completed_at.clone(),
        };
        let attempt: MemorySemanticProjectionAttempt =
            rmp_serde::from_slice(&attempt_envelope.payload)?;
        let legacy_attempt = LegacyMemorySemanticProjectionAttemptV0 {
            schema_version: "gamecult.epiphany.memory_semantic_projection_attempt.v0".into(),
            attempt_id: attempt.attempt_id,
            obligation_id: attempt.obligation_id,
            started_at: attempt.started_at,
            completed_at: attempt.completed_at,
            status: attempt.status,
            error: attempt.error,
        };
        let mut legacy_claim_envelope = claim_envelope.clone();
        legacy_claim_envelope.payload = rmp_serde::to_vec_named(&legacy_claim)?;
        let mut legacy_attempt_envelope = attempt_envelope.clone();
        legacy_attempt_envelope.payload = rmp_serde::to_vec_named(&legacy_attempt)?;
        assert!(backing.compare_and_swap_batch(
            &[claim_envelope, attempt_envelope],
            vec![legacy_claim_envelope, legacy_attempt_envelope],
        )?);

        assert_eq!(
            retire_memory_semantic_projection_claims_v0(&store)?,
            vec![acquisition.claim.claim_id]
        );
        let observation = observe_memory_semantic_projection(&store, &projection_input)?;
        assert_eq!(observation.status, "pending");
        assert!(retire_memory_semantic_projection_claims_v0(&store)?.is_empty());
        Ok(())
    }

    #[test]
    fn orphaned_legacy_attempt_retires_without_fabricated_claim() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("orphaned-attempt.msgpack");
        let legacy = LegacyMemorySemanticProjectionAttemptV0 {
            schema_version: "gamecult.epiphany.memory_semantic_projection_attempt.v0".into(),
            attempt_id: "orphaned-attempt-1".into(),
            obligation_id: "obligation-modeling-7".into(),
            started_at: "2026-07-15T04:01:00Z".into(),
            completed_at: Some("2026-07-15T04:02:00Z".into()),
            status: "succeeded".into(),
            error: None,
        };
        let mut backing = SingleFileMessagePackBackingStore::new(&store);
        backing.push(&CultCacheEnvelope {
            key: legacy.attempt_id.clone(),
            r#type: MemorySemanticProjectionAttempt::TYPE.into(),
            payload: rmp_serde::to_vec_named(&legacy)?,
            stored_at: "2026-07-15T04:02:00Z".into(),
            schema_id: Some(MemorySemanticProjectionAttempt::TYPE.into()),
        })?;
        assert_eq!(
            retire_orphaned_memory_semantic_projection_attempts_v0(&store)?,
            vec![legacy.attempt_id]
        );
        assert!(backing.pull_all()?.is_empty());
        Ok(())
    }

    #[test]
    fn concurrent_idunn_acquisition_has_one_winner_and_no_issued_litter() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("acquisition.msgpack");
        let obligation = obligation();
        let mut cache = semantic_projector_cache(&store)?;
        cache.put(&obligation.obligation_id, &obligation)?;
        let projection_input = input(&store, &obligation)?;
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
        let mut workers = Vec::new();
        for executor in ["executor-a", "executor-b"] {
            let store = store.clone();
            let projection_input = projection_input.clone();
            let barrier = barrier.clone();
            workers.push(std::thread::spawn(move || {
                barrier.wait();
                idunn_acquire_memory_semantic_projection(
                    store,
                    &projection_input,
                    executor,
                    &format!("{executor}-incarnation"),
                    "execute",
                    "idunn-incarnation-a",
                    "2026-07-15T04:01:00Z",
                )
            }));
        }
        barrier.wait();
        let results = workers
            .into_iter()
            .map(|worker| worker.join().expect("worker"))
            .collect::<Vec<_>>();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        let envelopes = SingleFileMessagePackBackingStore::new(&store).pull_all()?;
        let grants = decode_all::<MemorySemanticProjectorExecutorGrant>(&envelopes)?;
        assert_eq!(grants.len(), 1);
        assert!(grants.iter().all(|grant| grant.status == "consumed"));
        let claim = decode_all::<MemorySemanticProjectionClaim>(&envelopes)?
            .pop()
            .expect("claim");
        authenticate_claim_authority_from_envelopes(&envelopes, &claim)?;
        Ok(())
    }

    #[test]
    fn acquisition_is_epoch_bound_and_timestamp_ordered() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("ordering.msgpack");
        let obligation = obligation();
        let mut cache = semantic_projector_cache(&store)?;
        cache.put(&obligation.obligation_id, &obligation)?;
        let projection_input = input(&store, &obligation)?;
        assert!(
            idunn_acquire_memory_semantic_projection(
                &store,
                &projection_input,
                "executor-a",
                "executor-a-incarnation",
                "execute",
                "idunn-incarnation-a",
                "2026-07-15T03:59:59Z"
            )
            .is_err()
        );
        let acquisition = idunn_acquire_memory_semantic_projection(
            &store,
            &projection_input,
            "executor-a",
            "executor-a-incarnation",
            "execute",
            "idunn-incarnation-a",
            "2026-07-15T04:01:00Z",
        )?;
        assert_eq!(acquisition.grant.predecessor_claim_status, "absent");
        assert_eq!(acquisition.grant.resulting_claim_epoch, Some(1));
        assert_eq!(acquisition.claim.authority_id, acquisition.grant.grant_id);
        fail_memory_semantic_projection_claim(
            &store,
            &acquisition.claim.claim_id,
            "2026-07-15T04:02:00Z",
            "test failure",
        )?;
        let successor = idunn_acquire_memory_semantic_projection(
            &store,
            &projection_input,
            "executor-b",
            "executor-b-incarnation",
            "execute",
            "idunn-incarnation-a",
            "2026-07-15T04:03:00Z",
        )?;
        assert_eq!(
            successor.grant.predecessor_claim_id.as_deref(),
            Some(acquisition.claim.claim_id.as_str())
        );
        assert_eq!(successor.grant.predecessor_claim_epoch, Some(1));
        assert_eq!(successor.grant.predecessor_claim_status, "failed");
        assert_eq!(successor.claim.epoch, 2);
        Ok(())
    }

    #[test]
    fn acquisition_refuses_advanced_non_obligation_authority() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("stale-acquisition.msgpack");
        let obligation = obligation();
        let mut cache = semantic_projector_cache(&store)?;
        cache.put(&obligation.obligation_id, &obligation)?;
        let marker_key = "canonical-head-marker";
        cache.put(marker_key, &obligation)?;
        let mut projection_input = input(&store, &obligation)?;
        let opening = SingleFileMessagePackBackingStore::new(&store).pull_all()?;
        projection_input.authority.envelopes.push(exact_envelope(
            &opening,
            MemorySemanticProjectionObligation::TYPE,
            marker_key,
        )?);
        let mut advanced = obligation.clone();
        advanced.source_generation += 1;
        advanced.source_model_hash = "advanced-model-hash".to_string();
        advanced.canonical_content_set_hash = "advanced-content-hash".to_string();
        cache.put(marker_key, &advanced)?;
        assert!(
            idunn_acquire_memory_semantic_projection(
                &store,
                &projection_input,
                "executor-stale",
                "executor-stale-incarnation",
                "execute",
                "idunn-incarnation-a",
                "2026-07-15T04:01:00Z",
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn sealed_recovery_binds_lifecycle_receipt_and_provider_heartbeat() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("recovery.msgpack");
        let obligation = obligation();
        let mut cache = semantic_projector_cache(&store)?;
        cache.put(&obligation.obligation_id, &obligation)?;
        let projection_input = input(&store, &obligation)?;
        let acquisition = idunn_acquire_memory_semantic_projection(
            &store,
            &projection_input,
            "executor-a",
            "provider-old",
            "execute",
            "idunn-incarnation-a",
            "2026-07-15T04:01:00Z",
        )?;
        let evidence = idunn_semantic_recovery_evidence_from_cultmesh(
            &store,
            &projection_input,
            &acquisition.claim.claim_id,
            "idunn-incarnation-a",
            "managed-policy-1",
            "sha256-policy",
            "launch-receipt-1",
            "sha256-launch-receipt",
            "2026-07-15T04:02:00Z",
            "heartbeat-1",
            "sha256-heartbeat",
            "provider-new",
            "2026-07-15T04:03:00Z",
            "launch-receipt-1",
        )?;
        let (authorization, recovered) = idunn_recover_memory_semantic_projection(
            &store,
            &projection_input,
            &acquisition.claim.claim_id,
            "executor-b",
            "provider-new",
            &evidence,
            "2026-07-15T04:04:00Z",
        )?;
        assert_eq!(authorization.status, "consumed");
        assert_eq!(authorization.managed_service_policy_id, "managed-policy-1");
        assert_eq!(
            authorization.launch_lifecycle_receipt_id,
            "launch-receipt-1"
        );
        assert_eq!(authorization.provider_heartbeat_id, "heartbeat-1");
        assert_eq!(
            authorization.provider_incarnation,
            recovered.executor_incarnation
        );
        assert_eq!(recovered.authority_id, authorization.authorization_id);
        assert!(
            fail_memory_semantic_projection_claim(
                &store,
                &acquisition.claim.claim_id,
                "2026-07-15T04:05:00Z",
                "late old executor"
            )
            .is_err()
        );
        assert!(
            idunn_semantic_recovery_evidence_from_cultmesh(
                &store,
                &projection_input,
                &recovered.claim_id,
                "idunn-incarnation-a",
                "managed-policy-2",
                "sha256-policy-2",
                "launch-receipt-2",
                "sha256-launch-receipt-2",
                "2026-07-15T04:03:00Z",
                "heartbeat-2",
                "sha256-heartbeat-2",
                "provider-new",
                "2026-07-15T04:03:00Z",
                "launch-receipt-2",
            )
            .is_err()
        );
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.memory_semantic_projector_recovery_authorization",
    schema = "MemorySemanticProjectorRecoveryAuthorization"
)]
pub struct MemorySemanticProjectorRecoveryAuthorization {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub authorization_id: String,
    #[cultcache(key = 2)]
    pub scope_id: String,
    #[cultcache(key = 3)]
    pub swarm_id: String,
    #[cultcache(key = 4)]
    pub partition: String,
    #[cultcache(key = 5)]
    pub obligation_id: String,
    #[cultcache(key = 6)]
    pub claim_id: String,
    #[cultcache(key = 7)]
    pub claim_epoch: u64,
    #[cultcache(key = 8)]
    pub attempt_id: String,
    #[cultcache(key = 9)]
    pub abandoned_executor_id: String,
    #[cultcache(key = 10)]
    pub replacement_executor_id: String,
    #[cultcache(key = 11)]
    pub replacement_executor_incarnation: String,
    #[cultcache(key = 12)]
    pub issuer_id: String,
    #[cultcache(key = 13)]
    pub issuer_incarnation: String,
    #[cultcache(key = 14)]
    pub managed_service_policy_id: String,
    #[cultcache(key = 15)]
    pub managed_service_policy_digest: String,
    #[cultcache(key = 16)]
    pub status: String,
    #[cultcache(key = 17)]
    pub issued_at: String,
    #[cultcache(key = 18)]
    pub consumed_at: Option<String>,
    #[cultcache(key = 19)]
    pub resulting_claim_id: Option<String>,
    #[cultcache(key = 20)]
    pub resulting_claim_epoch: Option<u64>,
    #[cultcache(key = 21)]
    pub abandoned_executor_incarnation: String,
    #[cultcache(key = 22)]
    pub launch_lifecycle_receipt_id: String,
    #[cultcache(key = 23)]
    pub launch_lifecycle_receipt_digest: String,
    #[cultcache(key = 24)]
    pub launch_lifecycle_receipt_completed_at: String,
    #[cultcache(key = 25)]
    pub provider_heartbeat_id: String,
    #[cultcache(key = 26)]
    pub provider_heartbeat_digest: String,
    #[cultcache(key = 27)]
    pub provider_incarnation: String,
    #[cultcache(key = 28)]
    pub provider_heartbeat_at: String,
    #[cultcache(key = 29)]
    pub canonical_store_id: String,
    #[cultcache(key = 30)]
    pub startup_correlation_id: String,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.memory_semantic_projection_claim",
    schema = "MemorySemanticProjectionClaim"
)]
pub struct MemorySemanticProjectionClaim {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub scope_id: String,
    #[cultcache(key = 2)]
    pub claim_id: String,
    #[cultcache(key = 3)]
    pub obligation_id: String,
    #[cultcache(key = 4)]
    pub attempt_id: String,
    #[cultcache(key = 5)]
    pub executor_id: String,
    #[cultcache(key = 6)]
    pub epoch: u64,
    #[cultcache(key = 7)]
    pub status: String,
    #[cultcache(key = 8)]
    pub claimed_at: String,
    #[cultcache(key = 9)]
    pub completed_at: Option<String>,
    #[cultcache(key = 10)]
    pub executor_incarnation: String,
    #[cultcache(key = 11)]
    pub authority_kind: String,
    #[cultcache(key = 12)]
    pub authority_id: String,
}

#[allow(dead_code)] // Sealed until Idunn can supply authenticated lifecycle evidence.
#[derive(Clone, Debug)]
pub struct IdunnSemanticRecoveryEvidence {
    issuer_incarnation: String,
    managed_service_policy_id: String,
    managed_service_policy_digest: String,
    launch_lifecycle_receipt_id: String,
    launch_lifecycle_receipt_digest: String,
    launch_lifecycle_receipt_completed_at: String,
    provider_heartbeat_id: String,
    provider_heartbeat_digest: String,
    provider_incarnation: String,
    provider_heartbeat_at: String,
    canonical_store_id: String,
    scope_id: String,
    swarm_id: String,
    partition: String,
    obligation_id: String,
    claim_id: String,
    claim_epoch: u64,
    abandoned_executor_incarnation: String,
    startup_correlation_id: String,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn idunn_semantic_recovery_evidence_from_cultmesh(
    store_path: impl AsRef<Path>,
    input: &MemorySemanticProjectionInput,
    expected_claim_id: &str,
    issuer_incarnation: &str,
    managed_service_policy_id: &str,
    managed_service_policy_digest: &str,
    launch_lifecycle_receipt_id: &str,
    launch_lifecycle_receipt_digest: &str,
    launch_lifecycle_receipt_completed_at: &str,
    provider_heartbeat_id: &str,
    provider_heartbeat_digest: &str,
    provider_incarnation: &str,
    provider_heartbeat_at: &str,
    startup_correlation_id: &str,
) -> Result<IdunnSemanticRecoveryEvidence> {
    validate_memory_semantic_projection_obligation(&input.obligation)?;
    let store_path = store_path.as_ref();
    let envelopes = SingleFileMessagePackBackingStore::new(store_path).pull_all()?;
    let persisted_obligation = decode_one::<MemorySemanticProjectionObligation>(
        &envelopes,
        &input.obligation.obligation_id,
    )?
    .ok_or_else(|| anyhow!("recovery evidence canonical obligation disappeared"))?;
    if persisted_obligation != input.obligation {
        return Err(anyhow!("recovery evidence canonical obligation advanced"));
    }
    for expected in &input.authority.envelopes {
        let persisted = envelopes
            .iter()
            .find(|row| row.r#type == expected.r#type && row.key == expected.key)
            .ok_or_else(|| anyhow!("recovery evidence input authority disappeared"))?;
        if persisted != expected {
            return Err(anyhow!("recovery evidence input authority advanced"));
        }
    }
    let scope_id = projection_scope_id(&input.obligation.swarm_id, &input.obligation.partition)?;
    let claim = decode_one::<MemorySemanticProjectionClaim>(&envelopes, &scope_id)?
        .ok_or_else(|| anyhow!("recovery evidence requires current scope claim"))?;
    validate_memory_semantic_projection_claim(&claim)?;
    if claim.claim_id != expected_claim_id
        || claim.obligation_id != input.obligation.obligation_id
        || claim.status != "running"
    {
        return Err(anyhow!(
            "recovery evidence claim does not match canonical input"
        ));
    }
    authenticate_claim_authority_from_envelopes(&envelopes, &claim)?;
    let evidence = IdunnSemanticRecoveryEvidence {
        issuer_incarnation: issuer_incarnation.to_string(),
        managed_service_policy_id: managed_service_policy_id.to_string(),
        managed_service_policy_digest: managed_service_policy_digest.to_string(),
        launch_lifecycle_receipt_id: launch_lifecycle_receipt_id.to_string(),
        launch_lifecycle_receipt_digest: launch_lifecycle_receipt_digest.to_string(),
        launch_lifecycle_receipt_completed_at: launch_lifecycle_receipt_completed_at.to_string(),
        provider_heartbeat_id: provider_heartbeat_id.to_string(),
        provider_heartbeat_digest: provider_heartbeat_digest.to_string(),
        provider_incarnation: provider_incarnation.to_string(),
        provider_heartbeat_at: provider_heartbeat_at.to_string(),
        canonical_store_id: memory_semantic_projector_store_id(store_path)?,
        scope_id,
        swarm_id: input.obligation.swarm_id.clone(),
        partition: input.obligation.partition.clone(),
        obligation_id: input.obligation.obligation_id.clone(),
        claim_id: claim.claim_id,
        claim_epoch: claim.epoch,
        abandoned_executor_incarnation: claim.executor_incarnation,
        startup_correlation_id: startup_correlation_id.to_string(),
    };
    validate_idunn_semantic_recovery_evidence(&evidence)?;
    Ok(evidence)
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemorySemanticProjectorAcquisition {
    pub grant: MemorySemanticProjectorExecutorGrant,
    pub claim: MemorySemanticProjectionClaim,
}

#[derive(Clone, Debug)]
pub(crate) struct MemorySemanticProjectionAuthoritySnapshot {
    pub(crate) head: MemorySemanticProjectionSourceHead,
    pub(crate) envelopes: Vec<CultCacheEnvelope>,
}

#[derive(Clone, Debug)]
pub struct MemorySemanticProjectionInput {
    pub(crate) snapshot: super::EpiphanyMemoryGraphSnapshot,
    pub(crate) obligation: MemorySemanticProjectionObligation,
    pub(crate) authority: MemorySemanticProjectionAuthoritySnapshot,
}

#[derive(Clone, Debug)]
pub struct MemorySemanticProjectionObservation {
    pub(crate) swarm_id: String,
    pub(crate) partition: String,
    pub(crate) obligation_id: String,
    pub(crate) source_generation: u64,
    pub(crate) canonical_model_hash: String,
    pub(crate) canonical_content_set_hash: String,
    pub(crate) status: String,
    pub(crate) receipt_id: Option<String>,
    pub(crate) indexed_document_count: Option<u32>,
    pub(crate) vector_dimensions: Option<u32>,
    pub(crate) observed_source_at: String,
    pub(crate) query_eligible_display_only: bool,
}

impl MemorySemanticProjectionInput {
    pub fn snapshot(&self) -> &super::EpiphanyMemoryGraphSnapshot {
        &self.snapshot
    }

    pub fn obligation(&self) -> &MemorySemanticProjectionObligation {
        &self.obligation
    }

    pub fn source_head(&self) -> &MemorySemanticProjectionSourceHead {
        &self.authority.head
    }
}

pub fn observe_memory_semantic_projection(
    store_path: impl AsRef<Path>,
    input: &MemorySemanticProjectionInput,
) -> Result<MemorySemanticProjectionObservation> {
    if input.authority.envelopes.is_empty() {
        return Err(anyhow!(
            "semantic projection observation requires canonical authority"
        ));
    }
    validate_memory_semantic_projection_obligation(&input.obligation)?;
    let envelopes = SingleFileMessagePackBackingStore::new(store_path.as_ref()).pull_all()?;
    let persisted_obligation = decode_one::<MemorySemanticProjectionObligation>(
        &envelopes,
        &input.obligation.obligation_id,
    )?
    .ok_or_else(|| anyhow!("semantic projection observation obligation disappeared"))?;
    if persisted_obligation != input.obligation {
        return Err(anyhow!(
            "semantic projection observation obligation identity collision"
        ));
    }
    for expected in &input.authority.envelopes {
        let current = envelopes
            .iter()
            .find(|row| row.r#type == expected.r#type && row.key == expected.key)
            .ok_or_else(|| anyhow!("semantic projection observation authority disappeared"))?;
        if current != expected {
            return Err(anyhow!(
                "semantic projection observation authority advanced"
            ));
        }
    }
    let attempts = decode_all::<MemorySemanticProjectionAttempt>(&envelopes)?;
    let receipts = decode_all::<MemorySemanticIndexReceipt>(&envelopes)?;
    let health = super::derive_memory_semantic_projection_health(
        &input.obligation,
        &input.authority.head,
        &attempts,
        &receipts,
    )?;
    let receipt = health
        .receipt_id
        .as_ref()
        .and_then(|id| receipts.iter().find(|receipt| &receipt.receipt_id == id));
    if health.status == super::MemorySemanticProjectionHealthStatus::Ready {
        let authenticated = load_memory_semantic_projection_success(
            store_path.as_ref(),
            &input.obligation,
            &input.authority.head,
        )?
        .ok_or_else(|| {
            anyhow!("semantic projection ready observation lacks authenticated success")
        })?;
        if receipt.is_none_or(|receipt| receipt != &authenticated) {
            return Err(anyhow!(
                "semantic projection ready observation receipt is not authentic"
            ));
        }
    }
    let observed_source_at = attempts
        .iter()
        .filter(|attempt| attempt.obligation_id == input.obligation.obligation_id)
        .flat_map(|attempt| {
            [
                attempt.started_at.as_str(),
                attempt.completed_at.as_deref().unwrap_or(""),
            ]
        })
        .chain(
            receipt
                .into_iter()
                .map(|receipt| receipt.indexed_at.as_str()),
        )
        .filter(|value| !value.is_empty())
        .max_by_key(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .unwrap_or(input.obligation.created_at.as_str())
        .to_string();
    let status = match health.status {
        super::MemorySemanticProjectionHealthStatus::Pending => "pending",
        super::MemorySemanticProjectionHealthStatus::Failed => "failed",
        super::MemorySemanticProjectionHealthStatus::Stale => {
            return Err(anyhow!(
                "semantic projection observation input is not the current canonical head"
            ));
        }
        super::MemorySemanticProjectionHealthStatus::Ready => "ready",
    };
    let display_receipt = if health.status == super::MemorySemanticProjectionHealthStatus::Ready {
        receipt
    } else {
        None
    };
    Ok(MemorySemanticProjectionObservation {
        swarm_id: input.obligation.swarm_id.clone(),
        partition: input.obligation.partition.clone(),
        obligation_id: input.obligation.obligation_id.clone(),
        source_generation: input.obligation.source_generation,
        canonical_model_hash: input.obligation.source_model_hash.clone(),
        canonical_content_set_hash: input.obligation.canonical_content_set_hash.clone(),
        status: status.to_string(),
        receipt_id: display_receipt.map(|row| row.receipt_id.clone()),
        indexed_document_count: display_receipt.map(|row| row.indexed_document_count),
        vector_dimensions: display_receipt.map(|row| row.vector_dimensions),
        observed_source_at,
        query_eligible_display_only: health.query_eligible,
    })
}

#[cfg(feature = "semantic-recovery-smoke")]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticRecoverySmokeInspection {
    pub claim_id: String,
    pub claim_epoch: u64,
    pub claim_status: String,
    pub executor_id: String,
    pub executor_incarnation: String,
    pub attempt_status: String,
    pub attempt_completed: bool,
    pub recovery_authorization_id: Option<String>,
    pub recovery_authorization_status: Option<String>,
    pub recovery_authorization_consumed: bool,
    pub abandoned_attempt_failed: bool,
    pub old_owner_authenticates_current_claim: bool,
}

/// Read-only, feature-sealed witness for the native recovery smoke. It decodes
/// and validates the same typed authority documents used by production; it
/// cannot acquire, recover, complete, fail, or otherwise mutate a claim.
#[cfg(feature = "semantic-recovery-smoke")]
pub fn inspect_memory_semantic_recovery_for_smoke(
    store_path: impl AsRef<Path>,
    input: &MemorySemanticProjectionInput,
    old_executor_id: &str,
    old_executor_incarnation: &str,
) -> Result<SemanticRecoverySmokeInspection> {
    let store_path = store_path.as_ref();
    observe_memory_semantic_projection(store_path, input)?;
    let envelopes = SingleFileMessagePackBackingStore::new(store_path).pull_all()?;
    let scope_id = projection_scope_id(&input.obligation.swarm_id, &input.obligation.partition)?;
    let claim = decode_one::<MemorySemanticProjectionClaim>(&envelopes, &scope_id)?
        .ok_or_else(|| anyhow!("semantic recovery smoke found no current claim"))?;
    validate_memory_semantic_projection_claim(&claim)?;
    let attempts = decode_all::<MemorySemanticProjectionAttempt>(&envelopes)?;
    for attempt in &attempts {
        validate_memory_semantic_projection_attempt(attempt)?;
    }
    let attempt = attempts
        .iter()
        .find(|attempt| attempt.attempt_id == claim.attempt_id)
        .ok_or_else(|| anyhow!("semantic recovery smoke current claim lost its attempt"))?;
    let authorizations = decode_all::<MemorySemanticProjectorRecoveryAuthorization>(&envelopes)?;
    for authorization in &authorizations {
        validate_memory_semantic_projector_recovery_authorization(authorization)?;
    }
    let authorization = authorizations
        .iter()
        .find(|authorization| authorization.resulting_claim_id.as_deref() == Some(&claim.claim_id));
    let abandoned_attempt_failed = authorization.is_some_and(|authorization| {
        attempts.iter().any(|attempt| {
            attempt.attempt_id == authorization.attempt_id
                && attempt.claim_id == authorization.claim_id
                && attempt.claim_epoch == authorization.claim_epoch
                && attempt.status == "failed"
                && attempt.completed_at.is_some()
        })
    });
    let old_owner_authenticates_current_claim = owned_running_memory_semantic_projection_claim(
        store_path,
        input,
        old_executor_id,
        old_executor_incarnation,
    )?
    .is_some();
    Ok(SemanticRecoverySmokeInspection {
        claim_id: claim.claim_id,
        claim_epoch: claim.epoch,
        claim_status: claim.status,
        executor_id: claim.executor_id,
        executor_incarnation: claim.executor_incarnation,
        attempt_status: attempt.status.clone(),
        attempt_completed: attempt.completed_at.is_some(),
        recovery_authorization_id: authorization.map(|row| row.authorization_id.clone()),
        recovery_authorization_status: authorization.map(|row| row.status.clone()),
        recovery_authorization_consumed: authorization
            .is_some_and(|row| row.status == "consumed" && row.consumed_at.is_some()),
        abandoned_attempt_failed,
        old_owner_authenticates_current_claim,
    })
}

#[cfg(test)]
#[test]
fn semantic_recovery_smoke_exposes_no_raw_mutation_authority() {
    let source = include_str!("semantic_projector.rs");
    assert!(source.contains("pub(crate) fn idunn_acquire_memory_semantic_projection("));
    let public_acquire = ["pub fn idunn_", "acquire_memory_semantic_projection("].concat();
    let fixture_seed = ["seed_abandoned_memory_semantic_projection", "_for_smoke"].concat();
    assert!(!source.contains(&public_acquire));
    assert!(!source.contains(&fixture_seed));
    assert!(source.contains("pub fn inspect_memory_semantic_recovery_for_smoke("));
}

pub(crate) fn projection_scope_id(swarm_id: &str, partition: &str) -> Result<String> {
    if swarm_id.trim().is_empty() || !matches!(partition, "mind" | "modeling") {
        return Err(anyhow!("semantic projection scope identity is invalid"));
    }
    Ok(format!(
        "memory-semantic-scope-{partition}-{:x}",
        Sha256::digest(format!("{swarm_id}|{partition}").as_bytes())
    ))
}

pub(crate) fn memory_semantic_projector_store_id(store_path: &Path) -> Result<String> {
    let canonical = store_path.canonicalize().map_err(|error| {
        anyhow!("semantic projector canonical store identity is unavailable: {error}")
    })?;
    let rendered = canonical.to_string_lossy();
    let identity = if cfg!(windows) {
        rendered.to_lowercase()
    } else {
        rendered.into_owned()
    };
    Ok(format!(
        "memory-semantic-store-{:x}",
        Sha256::digest(identity.as_bytes())
    ))
}

pub fn load_memory_semantic_projection_success(
    store_path: impl AsRef<Path>,
    obligation: &MemorySemanticProjectionObligation,
    head: &MemorySemanticProjectionSourceHead,
) -> Result<Option<MemorySemanticIndexReceipt>> {
    validate_memory_semantic_projection_obligation(obligation)?;
    let envelopes = SingleFileMessagePackBackingStore::new(store_path.as_ref()).pull_all()?;
    let attempts = decode_all::<MemorySemanticProjectionAttempt>(&envelopes)?;
    let receipts = decode_all::<MemorySemanticIndexReceipt>(&envelopes)?;
    let health =
        super::derive_memory_semantic_projection_health(obligation, head, &attempts, &receipts)?;
    if health.status != super::MemorySemanticProjectionHealthStatus::Ready {
        return Ok(None);
    }
    let receipt_id = health
        .receipt_id
        .ok_or_else(|| anyhow!("ready semantic projection health lost receipt identity"))?;
    let receipt = receipts
        .into_iter()
        .find(|receipt| receipt.receipt_id == receipt_id)
        .ok_or_else(|| anyhow!("ready semantic projection receipt disappeared"))?;
    let scope_id = projection_scope_id(&obligation.swarm_id, &obligation.partition)?;
    let claim = decode_one::<MemorySemanticProjectionClaim>(&envelopes, &scope_id)?
        .ok_or_else(|| anyhow!("ready semantic projection lost its scope claim"))?;
    validate_memory_semantic_projection_claim(&claim)?;
    if claim.scope_id != scope_id {
        return Err(anyhow!(
            "semantic projection claim key disagrees with payload"
        ));
    }
    authenticate_claim_authority(store_path.as_ref(), &claim)?;
    if claim.status != "succeeded"
        || claim.obligation_id != obligation.obligation_id
        || receipt.claim_id != claim.claim_id
        || receipt.claim_epoch != claim.epoch
    {
        return Err(anyhow!(
            "ready semantic projection receipt is not authenticated by the succeeded scope claim"
        ));
    }
    let attempt = decode_one::<MemorySemanticProjectionAttempt>(&envelopes, &claim.attempt_id)?
        .ok_or_else(|| anyhow!("ready semantic projection lost its claim attempt"))?;
    validate_memory_semantic_projection_attempt(&attempt)?;
    if attempt.status != "succeeded" || !attempt_authenticates_claim(&attempt, &claim) {
        return Err(anyhow!(
            "ready semantic projection claim is not authenticated by its succeeded attempt"
        ));
    }
    Ok(Some(receipt))
}

pub fn load_memory_semantic_projection_readiness(
    store_path: impl AsRef<Path>,
    input: &MemorySemanticProjectionInput,
) -> Result<Option<super::MemorySemanticProjectionReadiness>> {
    let receipt = load_memory_semantic_projection_success(
        store_path,
        &input.obligation,
        &input.authority.head,
    )?;
    Ok(
        receipt.map(|receipt| super::MemorySemanticProjectionReadiness {
            obligation: input.obligation.clone(),
            current: input.authority.head.clone(),
            receipt,
        }),
    )
}

pub(crate) fn classify_memory_semantic_projection_for_pulse(
    store_path: &Path,
    input: &MemorySemanticProjectionInput,
) -> Result<super::MemorySemanticProjectorPulseClassification> {
    validate_memory_semantic_projection_obligation(&input.obligation)?;
    let envelopes = SingleFileMessagePackBackingStore::new(store_path).pull_all()?;
    let persisted = decode_one::<MemorySemanticProjectionObligation>(
        &envelopes,
        &input.obligation.obligation_id,
    )?;
    if persisted.as_ref() != Some(&input.obligation) {
        return Ok(super::MemorySemanticProjectorPulseClassification::Stale);
    }
    for expected in &input.authority.envelopes {
        if envelopes
            .iter()
            .find(|row| row.r#type == expected.r#type && row.key == expected.key)
            != Some(expected)
        {
            return Ok(super::MemorySemanticProjectorPulseClassification::Stale);
        }
    }
    let scope_id = projection_scope_id(&input.obligation.swarm_id, &input.obligation.partition)?;
    let current_claim = decode_one::<MemorySemanticProjectionClaim>(&envelopes, &scope_id)?;
    if let Some(claim) = &current_claim {
        validate_memory_semantic_projection_claim(&claim)?;
        if claim.scope_id != scope_id {
            return Err(anyhow!("semantic projector pulse claim key disagrees"));
        }
        if claim.obligation_id == input.obligation.obligation_id {
            match claim.status.as_str() {
                "running" => return Ok(super::MemorySemanticProjectorPulseClassification::Running),
                _ => {}
            }
        }
    }
    let health = super::derive_memory_semantic_projection_health(
        &input.obligation,
        &input.authority.head,
        &decode_all::<MemorySemanticProjectionAttempt>(&envelopes)?,
        &decode_all::<MemorySemanticIndexReceipt>(&envelopes)?,
    )?;
    let succeeded_predecessor = current_claim
        .as_ref()
        .map(|claim| {
            authenticated_succeeded_predecessor(&envelopes, claim, &input.obligation.obligation_id)
        })
        .transpose()?
        .unwrap_or(false);
    Ok(match health.status {
        super::MemorySemanticProjectionHealthStatus::Pending => {
            if succeeded_predecessor {
                super::MemorySemanticProjectorPulseClassification::Repair
            } else {
                super::MemorySemanticProjectorPulseClassification::Pending
            }
        }
        super::MemorySemanticProjectionHealthStatus::Failed => {
            if succeeded_predecessor {
                super::MemorySemanticProjectorPulseClassification::Repair
            } else {
                super::MemorySemanticProjectorPulseClassification::Failed
            }
        }
        super::MemorySemanticProjectionHealthStatus::Ready => {
            super::MemorySemanticProjectorPulseClassification::Ready
        }
        super::MemorySemanticProjectionHealthStatus::Stale => {
            super::MemorySemanticProjectorPulseClassification::Stale
        }
    })
}

pub(crate) fn owned_running_memory_semantic_projection_claim(
    store_path: &Path,
    input: &MemorySemanticProjectionInput,
    executor_id: &str,
    executor_incarnation: &str,
) -> Result<Option<String>> {
    let envelopes = SingleFileMessagePackBackingStore::new(store_path).pull_all()?;
    let scope_id = projection_scope_id(&input.obligation.swarm_id, &input.obligation.partition)?;
    let Some(claim) = decode_one::<MemorySemanticProjectionClaim>(&envelopes, &scope_id)? else {
        return Ok(None);
    };
    validate_memory_semantic_projection_claim(&claim)?;
    if claim.status != "running"
        || claim.obligation_id != input.obligation.obligation_id
        || claim.executor_id != executor_id
        || claim.executor_incarnation != executor_incarnation
    {
        return Ok(None);
    }
    authenticate_claim_authority_from_envelopes(&envelopes, &claim)?;
    for expected in &input.authority.envelopes {
        if envelopes
            .iter()
            .find(|row| row.r#type == expected.r#type && row.key == expected.key)
            != Some(expected)
        {
            return Err(anyhow!("owned running claim canonical authority advanced"));
        }
    }
    Ok(Some(claim.claim_id))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn idunn_acquire_memory_semantic_projection(
    store_path: impl AsRef<Path>,
    input: &MemorySemanticProjectionInput,
    executor_id: &str,
    executor_incarnation: &str,
    purpose: &str,
    idunn_incarnation: &str,
    acquired_at: &str,
) -> Result<MemorySemanticProjectorAcquisition> {
    let obligation = &input.obligation;
    validate_memory_semantic_projection_obligation(obligation)?;
    validate_identity_and_time(executor_id, acquired_at)?;
    validate_opaque_identity(executor_incarnation, "executor incarnation")?;
    validate_opaque_identity(idunn_incarnation, "issuer incarnation")?;
    if !matches!(purpose, "execute" | "repair") {
        return Err(anyhow!("semantic projector grant purpose is invalid"));
    }
    let store_path = store_path.as_ref();
    observe_memory_semantic_projection(store_path, input)?;
    let cache = semantic_projector_cache(store_path)?;
    let opening = SingleFileMessagePackBackingStore::new(store_path).pull_all()?;
    let persisted =
        decode_one::<MemorySemanticProjectionObligation>(&opening, &obligation.obligation_id)?
            .ok_or_else(|| anyhow!("semantic projector grant requires persisted obligation"))?;
    if persisted != *obligation {
        return Err(anyhow!("semantic projector grant obligation advanced"));
    }
    ensure_not_before(
        acquired_at,
        &obligation.created_at,
        "acquisition before obligation",
    )?;
    let scope_id = projection_scope_id(&obligation.swarm_id, &obligation.partition)?;
    let current = decode_one::<MemorySemanticProjectionClaim>(&opening, &scope_id)?;
    if let Some(claim) = &current {
        validate_memory_semantic_projection_claim(claim)?;
        if claim.scope_id != scope_id {
            return Err(anyhow!(
                "semantic projector scope key disagrees with claim payload"
            ));
        }
        let attempt = decode_one::<MemorySemanticProjectionAttempt>(&opening, &claim.attempt_id)?
            .ok_or_else(|| anyhow!("semantic projector scope attempt disappeared"))?;
        validate_memory_semantic_projection_attempt(&attempt)?;
        if !attempt_authenticates_claim(&attempt, claim) || attempt.status != claim.status {
            return Err(anyhow!(
                "semantic projector scope claim and attempt disagree"
            ));
        }
        if claim.status == "running" {
            return Err(anyhow!("semantic projector scope is already running"));
        }
        ensure_not_before(
            acquired_at,
            claim.completed_at.as_deref().unwrap_or(&claim.claimed_at),
            "acquisition before predecessor",
        )?;
    }
    let succeeded_predecessor = current
        .as_ref()
        .map(|claim| {
            authenticated_succeeded_predecessor(&opening, claim, &obligation.obligation_id)
        })
        .transpose()?
        .unwrap_or(false);
    match purpose {
        "execute" if succeeded_predecessor => {
            return Err(anyhow!("execute grant cannot authorize succeeded repair"));
        }
        "repair" if !succeeded_predecessor => {
            return Err(anyhow!("repair grant requires exact succeeded claim"));
        }
        _ => {}
    }
    let epoch = current.as_ref().map_or(1, |claim| claim.epoch + 1);
    let grant_id = format!("memory-semantic-executor-grant-{}", uuid::Uuid::new_v4());
    let claim = running_claim(
        &scope_id,
        obligation,
        executor_id,
        executor_incarnation,
        "executor_grant",
        &grant_id,
        epoch,
        acquired_at,
    );
    let attempt = running_attempt(&claim);
    let grant = MemorySemanticProjectorExecutorGrant {
        schema_version: MEMORY_SEMANTIC_PROJECTOR_EXECUTOR_GRANT_SCHEMA_VERSION.to_string(),
        grant_id: grant_id.clone(),
        scope_id: scope_id.clone(),
        swarm_id: obligation.swarm_id.clone(),
        partition: obligation.partition.clone(),
        obligation_id: obligation.obligation_id.clone(),
        executor_id: executor_id.to_string(),
        executor_incarnation: executor_incarnation.to_string(),
        purpose: purpose.to_string(),
        issuer_id: "idunn".to_string(),
        issuer_incarnation: idunn_incarnation.to_string(),
        status: "consumed".to_string(),
        issued_at: acquired_at.to_string(),
        consumed_at: Some(acquired_at.to_string()),
        resulting_claim_id: Some(claim.claim_id.clone()),
        resulting_claim_epoch: Some(claim.epoch),
        predecessor_claim_id: current.as_ref().map(|row| row.claim_id.clone()),
        predecessor_claim_epoch: current.as_ref().map(|row| row.epoch),
        predecessor_claim_status: current
            .as_ref()
            .map_or("absent", |row| row.status.as_str())
            .to_string(),
    };
    validate_memory_semantic_projector_executor_grant(&grant)?;
    if opening.iter().any(|envelope| {
        envelope.r#type == MemorySemanticProjectorExecutorGrant::TYPE
            && envelope.key == grant.grant_id
    }) {
        return Err(anyhow!(
            "semantic projector executor grant identity collision"
        ));
    }
    let obligation_envelope = exact_envelope(
        &opening,
        MemorySemanticProjectionObligation::TYPE,
        &obligation.obligation_id,
    )?;
    let mut expected = vec![obligation_envelope.clone()];
    let mut replacements = vec![obligation_envelope];
    for authority in &input.authority.envelopes {
        if authority.r#type == MemorySemanticProjectionObligation::TYPE
            && authority.key == obligation.obligation_id
        {
            continue;
        }
        let persisted = opening
            .iter()
            .find(|row| row.r#type == authority.r#type && row.key == authority.key)
            .ok_or_else(|| anyhow!("semantic projector acquisition authority disappeared"))?;
        if persisted != authority {
            return Err(anyhow!("semantic projector acquisition authority advanced"));
        }
        expected.push(persisted.clone());
        replacements.push(persisted.clone());
    }
    if current.is_some() {
        let claim = exact_envelope(
            &opening,
            MemorySemanticProjectionClaim::TYPE,
            &grant.scope_id,
        )?;
        expected.push(claim.clone());
        let current = current.as_ref().expect("checked current claim");
        let attempt = exact_envelope(
            &opening,
            MemorySemanticProjectionAttempt::TYPE,
            &current.attempt_id,
        )?;
        expected.push(attempt.clone());
        replacements.push(attempt);
    }
    replacements.push(cache.prepare_entry(&scope_id, &claim)?.0);
    replacements.push(cache.prepare_entry(&attempt.attempt_id, &attempt)?.0);
    replacements.push(cache.prepare_entry(&grant.grant_id, &grant)?.0);
    if !SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&expected, replacements)?
    {
        return Err(anyhow!("semantic projector acquisition lost exact CAS"));
    }
    Ok(MemorySemanticProjectorAcquisition { grant, claim })
}

pub(crate) fn execute_memory_semantic_projection(
    store_path: impl AsRef<Path>,
    input: &MemorySemanticProjectionInput,
    claim_id: &str,
    config: &super::MemorySemanticIndexConfig,
) -> Result<MemorySemanticIndexReceipt> {
    let partition = match input.obligation.partition.as_str() {
        "mind" => super::SemanticPartition::Mind,
        "modeling" => super::SemanticPartition::Modeling,
        _ => {
            return Err(anyhow!(
                "semantic projection obligation partition is invalid"
            ));
        }
    };
    let store_path = store_path.as_ref();
    let started_at = now_rfc3339();
    let (_, _, claim, _) = load_running_claim(store_path, claim_id)?;
    if claim.obligation_id != input.obligation.obligation_id {
        return Err(anyhow!(
            "semantic projection claim does not bind input obligation"
        ));
    }
    authenticate_claim_authority(store_path, &claim)?;
    let raw_receipt = match super::semantic_index::index_memory_semantic_partition(
        &input.snapshot,
        &input.obligation.swarm_id,
        partition,
        &super::semantic_index::MemorySemanticProjectionNamespace {
            obligation_id: claim.obligation_id.clone(),
            claim_id: claim.claim_id.clone(),
            claim_epoch: claim.epoch,
        },
        &started_at,
        config,
    ) {
        Ok(receipt) => receipt,
        Err(error) => {
            let _ = fail_memory_semantic_projection_claim(
                store_path,
                &claim.claim_id,
                &now_rfc3339(),
                &format!("{error:#}"),
            );
            return Err(error);
        }
    };
    match succeed_memory_semantic_projection_claim(
        store_path,
        &claim.claim_id,
        &input.authority,
        raw_receipt,
        &now_rfc3339(),
    ) {
        Ok(receipt) => Ok(receipt),
        Err(error) => {
            let _ = fail_memory_semantic_projection_claim(
                store_path,
                &claim.claim_id,
                &now_rfc3339(),
                &format!("terminal success refused: {error:#}"),
            );
            Err(error)
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code, clippy::too_many_arguments)] // Withheld behind sealed recovery evidence.
pub(crate) fn idunn_recover_memory_semantic_projection(
    store_path: impl AsRef<Path>,
    input: &MemorySemanticProjectionInput,
    expected_claim_id: &str,
    replacement_executor_id: &str,
    replacement_executor_incarnation: &str,
    evidence: &IdunnSemanticRecoveryEvidence,
    recovered_at: &str,
) -> Result<(
    MemorySemanticProjectorRecoveryAuthorization,
    MemorySemanticProjectionClaim,
)> {
    validate_identity_and_time(replacement_executor_id, recovered_at)?;
    validate_opaque_identity(
        replacement_executor_incarnation,
        "replacement executor incarnation",
    )?;
    validate_opaque_identity(&evidence.issuer_incarnation, "issuer incarnation")?;
    validate_idunn_semantic_recovery_evidence(evidence)?;
    if replacement_executor_incarnation != evidence.provider_incarnation {
        return Err(anyhow!(
            "recovery executor incarnation is not the observed provider incarnation"
        ));
    }
    ensure_strictly_before(
        &evidence.launch_lifecycle_receipt_completed_at,
        &evidence.provider_heartbeat_at,
        "provider heartbeat must follow managed-service launch receipt",
    )?;
    ensure_not_before(
        recovered_at,
        &evidence.provider_heartbeat_at,
        "recovery precedes provider heartbeat",
    )?;
    let store_path = store_path.as_ref();
    validate_memory_semantic_projection_obligation(&input.obligation)?;
    if evidence.canonical_store_id != memory_semantic_projector_store_id(store_path)? {
        return Err(anyhow!(
            "recovery evidence belongs to another canonical store"
        ));
    }
    let cache = semantic_projector_cache(store_path)?;
    let opening = SingleFileMessagePackBackingStore::new(store_path).pull_all()?;
    let current = decode_all::<MemorySemanticProjectionClaim>(&opening)?
        .into_iter()
        .find(|claim| claim.claim_id == expected_claim_id)
        .ok_or_else(|| anyhow!("semantic projection recovery claim is missing"))?;
    validate_memory_semantic_projection_claim(&current)?;
    if current.status != "running" {
        return Err(anyhow!(
            "semantic projection recovery requires exact running claim"
        ));
    }
    authenticate_claim_authority_from_envelopes(&opening, &current)?;
    let obligation =
        decode_one::<MemorySemanticProjectionObligation>(&opening, &current.obligation_id)?
            .ok_or_else(|| anyhow!("semantic projection recovery lost obligation"))?;
    if obligation != input.obligation
        || current.obligation_id != input.obligation.obligation_id
        || current.scope_id
            != projection_scope_id(&input.obligation.swarm_id, &input.obligation.partition)?
        || evidence.scope_id != current.scope_id
        || evidence.swarm_id != obligation.swarm_id
        || evidence.partition != obligation.partition
        || evidence.obligation_id != obligation.obligation_id
        || evidence.claim_id != current.claim_id
        || evidence.claim_epoch != current.epoch
        || evidence.abandoned_executor_incarnation != current.executor_incarnation
    {
        return Err(anyhow!(
            "recovery evidence does not bind exact canonical claim input"
        ));
    }
    for expected in &input.authority.envelopes {
        let persisted = opening
            .iter()
            .find(|row| row.r#type == expected.r#type && row.key == expected.key)
            .ok_or_else(|| anyhow!("recovery canonical input authority disappeared"))?;
        if persisted != expected {
            return Err(anyhow!("recovery canonical input authority advanced"));
        }
    }
    let old_attempt = decode_one::<MemorySemanticProjectionAttempt>(&opening, &current.attempt_id)?
        .ok_or_else(|| anyhow!("semantic projection recovery lost running attempt"))?;
    validate_memory_semantic_projection_attempt(&old_attempt)?;
    if old_attempt.status != "running" || !attempt_authenticates_claim(&old_attempt, &current) {
        return Err(anyhow!(
            "semantic projection recovery running attempt disagrees"
        ));
    }
    ensure_not_before(recovered_at, &current.claimed_at, "recovery before claim")?;
    let authorization_id = format!(
        "memory-semantic-recovery-authorization-{:x}",
        Sha256::digest(
            format!(
                "{}|{}|{}|{}",
                evidence.canonical_store_id,
                evidence.managed_service_policy_digest,
                evidence.launch_lifecycle_receipt_digest,
                evidence.provider_heartbeat_digest
            )
            .as_bytes()
        )
    );
    if opening.iter().any(|row| {
        row.r#type == MemorySemanticProjectorRecoveryAuthorization::TYPE
            && row.key == authorization_id
    }) {
        return Err(anyhow!("semantic recovery evidence was already consumed"));
    }
    let next = running_claim(
        &current.scope_id,
        &obligation,
        replacement_executor_id,
        replacement_executor_incarnation,
        "recovery_authorization",
        &authorization_id,
        current.epoch + 1,
        recovered_at,
    );
    let next_attempt = running_attempt(&next);
    let failed_attempt = MemorySemanticProjectionAttempt {
        completed_at: Some(recovered_at.to_string()),
        status: "failed".to_string(),
        error: Some("superseded by authenticated fenced recovery".to_string()),
        ..old_attempt.clone()
    };
    let authorization = MemorySemanticProjectorRecoveryAuthorization {
        schema_version: MEMORY_SEMANTIC_PROJECTOR_RECOVERY_AUTHORIZATION_SCHEMA_VERSION.to_string(),
        authorization_id: authorization_id.clone(),
        scope_id: current.scope_id.clone(),
        swarm_id: obligation.swarm_id.clone(),
        partition: obligation.partition.clone(),
        obligation_id: obligation.obligation_id.clone(),
        claim_id: current.claim_id.clone(),
        claim_epoch: current.epoch,
        attempt_id: current.attempt_id.clone(),
        abandoned_executor_id: current.executor_id.clone(),
        abandoned_executor_incarnation: current.executor_incarnation.clone(),
        replacement_executor_id: replacement_executor_id.to_string(),
        replacement_executor_incarnation: replacement_executor_incarnation.to_string(),
        issuer_id: "idunn".to_string(),
        issuer_incarnation: evidence.issuer_incarnation.clone(),
        managed_service_policy_id: evidence.managed_service_policy_id.clone(),
        managed_service_policy_digest: evidence.managed_service_policy_digest.clone(),
        launch_lifecycle_receipt_id: evidence.launch_lifecycle_receipt_id.clone(),
        launch_lifecycle_receipt_digest: evidence.launch_lifecycle_receipt_digest.clone(),
        launch_lifecycle_receipt_completed_at: evidence
            .launch_lifecycle_receipt_completed_at
            .clone(),
        provider_heartbeat_id: evidence.provider_heartbeat_id.clone(),
        provider_heartbeat_digest: evidence.provider_heartbeat_digest.clone(),
        provider_incarnation: evidence.provider_incarnation.clone(),
        provider_heartbeat_at: evidence.provider_heartbeat_at.clone(),
        canonical_store_id: evidence.canonical_store_id.clone(),
        startup_correlation_id: evidence.startup_correlation_id.clone(),
        status: "consumed".to_string(),
        issued_at: recovered_at.to_string(),
        consumed_at: Some(recovered_at.to_string()),
        resulting_claim_id: Some(next.claim_id.clone()),
        resulting_claim_epoch: Some(next.epoch),
    };
    validate_memory_semantic_projector_recovery_authorization(&authorization)?;
    let mut replacements = vec![
        cache.prepare_entry(&current.scope_id, &next)?.0,
        cache
            .prepare_entry(&failed_attempt.attempt_id, &failed_attempt)?
            .0,
        cache
            .prepare_entry(&next_attempt.attempt_id, &next_attempt)?
            .0,
        cache
            .prepare_entry(&authorization.authorization_id, &authorization)?
            .0,
    ];
    let authority_expected = input
        .authority
        .envelopes
        .iter()
        .filter(|row| {
            !matches!(
                row.r#type.as_str(),
                "gamecult.epiphany.memory_semantic_projection_obligation"
                    | "gamecult.epiphany.memory_semantic_projection_claim"
                    | "gamecult.epiphany.memory_semantic_projection_attempt"
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    replacements.extend(authority_expected.iter().cloned());
    terminal_or_recovery_cas(
        store_path,
        &opening,
        &current,
        &old_attempt,
        replacements,
        &authority_expected,
    )?;
    Ok((authorization, next))
}

pub(crate) fn fail_memory_semantic_projection_claim(
    store_path: impl AsRef<Path>,
    claim_id: &str,
    completed_at: &str,
    error: &str,
) -> Result<MemorySemanticProjectionClaim> {
    if error.trim().is_empty() {
        return Err(anyhow!("semantic projection failure requires an error"));
    }
    chrono::DateTime::parse_from_rfc3339(completed_at)
        .map_err(|_| anyhow!("semantic projection completion time must be RFC3339"))?;
    let store_path = store_path.as_ref();
    let (cache, opening, claim, attempt) = load_running_claim(store_path, claim_id)?;
    ensure_not_before(completed_at, &claim.claimed_at, "failure before claim")?;
    let terminal_claim = MemorySemanticProjectionClaim {
        status: "failed".to_string(),
        completed_at: Some(completed_at.to_string()),
        ..claim.clone()
    };
    let terminal_attempt = MemorySemanticProjectionAttempt {
        status: "failed".to_string(),
        completed_at: Some(completed_at.to_string()),
        error: Some(error.to_string()),
        ..attempt.clone()
    };
    terminal_or_recovery_cas(
        store_path,
        &opening,
        &claim,
        &attempt,
        vec![
            cache.prepare_entry(&claim.scope_id, &terminal_claim)?.0,
            cache
                .prepare_entry(&attempt.attempt_id, &terminal_attempt)?
                .0,
        ],
        &[],
    )?;
    Ok(terminal_claim)
}

pub(crate) fn succeed_memory_semantic_projection_claim(
    store_path: impl AsRef<Path>,
    claim_id: &str,
    authority: &MemorySemanticProjectionAuthoritySnapshot,
    receipt: MemorySemanticIndexReceipt,
    completed_at: &str,
) -> Result<MemorySemanticIndexReceipt> {
    chrono::DateTime::parse_from_rfc3339(completed_at)
        .map_err(|_| anyhow!("semantic projection completion time must be RFC3339"))?;
    if authority.envelopes.is_empty() {
        return Err(anyhow!(
            "semantic projection success requires canonical authority envelopes"
        ));
    }
    let store_path = store_path.as_ref();
    let (cache, opening, claim, attempt) = load_running_claim(store_path, claim_id)?;
    ensure_not_before(completed_at, &claim.claimed_at, "success before claim")?;
    let obligation =
        decode_one::<MemorySemanticProjectionObligation>(&opening, &claim.obligation_id)?
            .ok_or_else(|| anyhow!("semantic projection success lost obligation"))?;
    ensure_not_before(
        &receipt.indexed_at,
        &claim.claimed_at,
        "index receipt before claim",
    )?;
    ensure_not_before(
        completed_at,
        &receipt.indexed_at,
        "success before index receipt",
    )?;
    if receipt.obligation_id != claim.obligation_id
        || receipt.claim_id != claim.claim_id
        || receipt.claim_epoch != claim.epoch
    {
        return Err(anyhow!(
            "semantic projection result belongs to a different physical claim namespace"
        ));
    }
    let receipt = bind_memory_semantic_index_receipt(receipt, &obligation)?;
    if !memory_semantic_projection_terminal_success(&obligation, &authority.head, &receipt) {
        return Err(anyhow!(
            "semantic projection success no longer matches canonical source head"
        ));
    }
    let terminal_claim = MemorySemanticProjectionClaim {
        status: "succeeded".to_string(),
        completed_at: Some(completed_at.to_string()),
        ..claim.clone()
    };
    let terminal_attempt = MemorySemanticProjectionAttempt {
        status: "succeeded".to_string(),
        completed_at: Some(completed_at.to_string()),
        error: None,
        ..attempt.clone()
    };
    let obligation_envelope = exact_envelope(
        &opening,
        "gamecult.epiphany.memory_semantic_projection_obligation",
        &obligation.obligation_id,
    )?;
    let claim_envelope = exact_envelope(
        &opening,
        "gamecult.epiphany.memory_semantic_projection_claim",
        &claim.scope_id,
    )?;
    let attempt_envelope = exact_envelope(
        &opening,
        "gamecult.epiphany.memory_semantic_projection_attempt",
        &attempt.attempt_id,
    )?;
    let mut expected = vec![
        obligation_envelope.clone(),
        claim_envelope,
        attempt_envelope,
    ];
    let mut replacements = vec![
        obligation_envelope,
        cache.prepare_entry(&claim.scope_id, &terminal_claim)?.0,
        cache
            .prepare_entry(&attempt.attempt_id, &terminal_attempt)?
            .0,
        cache.prepare_entry(&receipt.receipt_id, &receipt)?.0,
    ];
    for authority_envelope in &authority.envelopes {
        let current = opening
            .iter()
            .find(|candidate| {
                candidate.r#type == authority_envelope.r#type
                    && candidate.key == authority_envelope.key
            })
            .ok_or_else(|| anyhow!("canonical projection authority envelope disappeared"))?;
        if current != authority_envelope {
            return Err(anyhow!("canonical projection authority head advanced"));
        }
        if !expected.iter().any(|candidate| {
            candidate.r#type == authority_envelope.r#type && candidate.key == authority_envelope.key
        }) {
            expected.push(authority_envelope.clone());
            replacements.push(authority_envelope.clone());
        }
    }
    if !SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&expected, replacements)?
    {
        return Err(anyhow!(
            "semantic projection success lost exact terminal CAS"
        ));
    }
    Ok(receipt)
}

pub(crate) fn validate_memory_semantic_projection_claim(
    claim: &MemorySemanticProjectionClaim,
) -> Result<()> {
    if claim.schema_version != MEMORY_SEMANTIC_PROJECTION_CLAIM_SCHEMA_VERSION
        || claim.scope_id.trim().is_empty()
        || claim.claim_id.trim().is_empty()
        || claim.obligation_id.trim().is_empty()
        || claim.attempt_id.trim().is_empty()
        || claim.executor_id.trim().is_empty()
        || !is_opaque_identity(&claim.executor_incarnation)
        || !matches!(
            claim.authority_kind.as_str(),
            "executor_grant" | "recovery_authorization"
        )
        || !is_opaque_identity(&claim.authority_id)
        || claim.epoch == 0
        || chrono::DateTime::parse_from_rfc3339(&claim.claimed_at).is_err()
    {
        return Err(anyhow!("semantic projection claim is invalid"));
    }
    match claim.status.as_str() {
        "running" if claim.completed_at.is_none() => Ok(()),
        "failed" | "succeeded"
            if claim
                .completed_at
                .as_deref()
                .is_some_and(|time| chrono::DateTime::parse_from_rfc3339(time).is_ok()) =>
        {
            Ok(())
        }
        _ => Err(anyhow!(
            "semantic projection claim terminal fields disagree"
        )),
    }
}

pub fn validate_memory_semantic_projector_executor_grant(
    grant: &MemorySemanticProjectorExecutorGrant,
) -> Result<()> {
    let consumed = grant.status == "consumed"
        && grant.consumed_at.as_deref().is_some_and(valid_rfc3339)
        && grant
            .resulting_claim_id
            .as_deref()
            .is_some_and(is_opaque_identity)
        && grant.resulting_claim_epoch.is_some_and(|epoch| epoch > 0);
    if grant.schema_version != MEMORY_SEMANTIC_PROJECTOR_EXECUTOR_GRANT_SCHEMA_VERSION
        || !grant
            .grant_id
            .starts_with("memory-semantic-executor-grant-")
        || !is_opaque_identity(&grant.grant_id)
        || !is_opaque_identity(&grant.scope_id)
        || !is_opaque_identity(&grant.swarm_id)
        || !matches!(grant.partition.as_str(), "mind" | "modeling")
        || !is_opaque_identity(&grant.obligation_id)
        || !is_opaque_identity(&grant.executor_id)
        || !is_opaque_identity(&grant.executor_incarnation)
        || !matches!(grant.purpose.as_str(), "execute" | "repair")
        || grant.issuer_id != "idunn"
        || !is_opaque_identity(&grant.issuer_incarnation)
        || !valid_rfc3339(&grant.issued_at)
        || grant.consumed_at.as_deref() != Some(grant.issued_at.as_str())
        || !consumed
        || !matches!(
            grant.predecessor_claim_status.as_str(),
            "absent" | "failed" | "succeeded"
        )
        || (grant.predecessor_claim_status == "absent"
            && (grant.predecessor_claim_id.is_some() || grant.predecessor_claim_epoch.is_some()))
        || (grant.predecessor_claim_status != "absent"
            && !(grant
                .predecessor_claim_id
                .as_deref()
                .is_some_and(is_opaque_identity)
                && grant.predecessor_claim_epoch.is_some_and(|epoch| epoch > 0)))
        || (grant.predecessor_claim_status == "absent" && grant.resulting_claim_epoch != Some(1))
        || grant
            .predecessor_claim_epoch
            .is_some_and(|epoch| grant.resulting_claim_epoch != Some(epoch + 1))
        || (grant.purpose == "repair" && grant.predecessor_claim_status != "succeeded")
        || (grant.purpose == "execute" && grant.predecessor_claim_status == "succeeded")
    {
        return Err(anyhow!("semantic projector executor grant is invalid"));
    }
    Ok(())
}

pub fn validate_memory_semantic_projector_recovery_authorization(
    authorization: &MemorySemanticProjectorRecoveryAuthorization,
) -> Result<()> {
    let consumed = authorization.status == "consumed"
        && authorization
            .consumed_at
            .as_deref()
            .is_some_and(valid_rfc3339)
        && authorization
            .resulting_claim_id
            .as_deref()
            .is_some_and(is_opaque_identity)
        && authorization
            .resulting_claim_epoch
            .is_some_and(|epoch| epoch > authorization.claim_epoch);
    if authorization.schema_version
        != MEMORY_SEMANTIC_PROJECTOR_RECOVERY_AUTHORIZATION_SCHEMA_VERSION
        || !authorization
            .authorization_id
            .starts_with("memory-semantic-recovery-authorization-")
        || !is_opaque_identity(&authorization.authorization_id)
        || !is_opaque_identity(&authorization.scope_id)
        || !is_opaque_identity(&authorization.swarm_id)
        || !matches!(authorization.partition.as_str(), "mind" | "modeling")
        || !is_opaque_identity(&authorization.obligation_id)
        || !is_opaque_identity(&authorization.claim_id)
        || authorization.claim_epoch == 0
        || !is_opaque_identity(&authorization.attempt_id)
        || !is_opaque_identity(&authorization.abandoned_executor_id)
        || !is_opaque_identity(&authorization.abandoned_executor_incarnation)
        || !is_opaque_identity(&authorization.replacement_executor_id)
        || !is_opaque_identity(&authorization.replacement_executor_incarnation)
        || authorization.issuer_id != "idunn"
        || !is_opaque_identity(&authorization.issuer_incarnation)
        || !is_opaque_identity(&authorization.managed_service_policy_id)
        || !is_opaque_identity(&authorization.managed_service_policy_digest)
        || !is_opaque_identity(&authorization.launch_lifecycle_receipt_id)
        || !is_opaque_identity(&authorization.launch_lifecycle_receipt_digest)
        || !valid_rfc3339(&authorization.launch_lifecycle_receipt_completed_at)
        || !is_opaque_identity(&authorization.provider_heartbeat_id)
        || !is_opaque_identity(&authorization.provider_heartbeat_digest)
        || !is_opaque_identity(&authorization.provider_incarnation)
        || !valid_rfc3339(&authorization.provider_heartbeat_at)
        || !is_opaque_identity(&authorization.canonical_store_id)
        || !is_opaque_identity(&authorization.startup_correlation_id)
        || authorization.startup_correlation_id != authorization.launch_lifecycle_receipt_id
        || authorization.replacement_executor_incarnation != authorization.provider_incarnation
        || authorization.provider_incarnation == authorization.abandoned_executor_incarnation
        || !strictly_before(
            &authorization.launch_lifecycle_receipt_completed_at,
            &authorization.provider_heartbeat_at,
        )
        || !not_before(
            authorization.consumed_at.as_deref().unwrap_or(""),
            &authorization.provider_heartbeat_at,
        )
        || !valid_rfc3339(&authorization.issued_at)
        || authorization.consumed_at.as_deref() != Some(authorization.issued_at.as_str())
        || !consumed
    {
        return Err(anyhow!(
            "semantic projector recovery authorization is invalid"
        ));
    }
    Ok(())
}

pub(crate) fn semantic_projector_cache(store_path: &Path) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<MemorySemanticProjectionObligation>()?;
    cache.register_entry_type::<MemorySemanticProjectionClaim>()?;
    cache.register_entry_type::<MemorySemanticProjectionAttempt>()?;
    cache.register_entry_type::<MemorySemanticIndexReceipt>()?;
    cache.register_entry_type::<MemorySemanticProjectorExecutorGrant>()?;
    cache.register_entry_type::<MemorySemanticProjectorRecoveryAuthorization>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(store_path));
    Ok(cache)
}

fn running_claim(
    scope_id: &str,
    obligation: &MemorySemanticProjectionObligation,
    executor_id: &str,
    executor_incarnation: &str,
    authority_kind: &str,
    authority_id: &str,
    epoch: u64,
    claimed_at: &str,
) -> MemorySemanticProjectionClaim {
    let fingerprint = format!(
        "{}|{}|{}|{}|{}|{}|{}",
        scope_id,
        obligation.obligation_id,
        executor_id,
        executor_incarnation,
        authority_kind,
        authority_id,
        epoch
    );
    let claim_id = format!(
        "memory-semantic-claim-{:x}",
        Sha256::digest(fingerprint.as_bytes())
    );
    MemorySemanticProjectionClaim {
        schema_version: MEMORY_SEMANTIC_PROJECTION_CLAIM_SCHEMA_VERSION.to_string(),
        scope_id: scope_id.to_string(),
        claim_id: claim_id.clone(),
        obligation_id: obligation.obligation_id.clone(),
        attempt_id: format!("memory-semantic-attempt-{claim_id}"),
        executor_id: executor_id.to_string(),
        executor_incarnation: executor_incarnation.to_string(),
        authority_kind: authority_kind.to_string(),
        authority_id: authority_id.to_string(),
        epoch,
        status: "running".to_string(),
        claimed_at: claimed_at.to_string(),
        completed_at: None,
    }
}

fn running_attempt(claim: &MemorySemanticProjectionClaim) -> MemorySemanticProjectionAttempt {
    MemorySemanticProjectionAttempt {
        schema_version: MEMORY_SEMANTIC_PROJECTION_ATTEMPT_SCHEMA_VERSION.to_string(),
        attempt_id: claim.attempt_id.clone(),
        obligation_id: claim.obligation_id.clone(),
        started_at: claim.claimed_at.clone(),
        completed_at: None,
        status: "running".to_string(),
        error: None,
        claim_id: claim.claim_id.clone(),
        claim_epoch: claim.epoch,
        executor_id: claim.executor_id.clone(),
        executor_incarnation: claim.executor_incarnation.clone(),
        authority_id: claim.authority_id.clone(),
    }
}

fn load_running_claim(
    store_path: &Path,
    claim_id: &str,
) -> Result<(
    CultCache,
    Vec<CultCacheEnvelope>,
    MemorySemanticProjectionClaim,
    MemorySemanticProjectionAttempt,
)> {
    let cache = semantic_projector_cache(store_path)?;
    let opening = SingleFileMessagePackBackingStore::new(store_path).pull_all()?;
    let claim = decode_all::<MemorySemanticProjectionClaim>(&opening)?
        .into_iter()
        .find(|claim| claim.claim_id == claim_id)
        .ok_or_else(|| anyhow!("semantic projection claim is missing"))?;
    validate_memory_semantic_projection_claim(&claim)?;
    if claim.status != "running" {
        return Err(anyhow!("semantic projection claim is not running"));
    }
    let attempt = decode_one::<MemorySemanticProjectionAttempt>(&opening, &claim.attempt_id)?
        .ok_or_else(|| anyhow!("semantic projection running attempt is missing"))?;
    validate_memory_semantic_projection_attempt(&attempt)?;
    if attempt.status != "running" || !attempt_authenticates_claim(&attempt, &claim) {
        return Err(anyhow!("semantic projection attempt does not own claim"));
    }
    Ok((cache, opening, claim, attempt))
}

fn authenticate_claim_authority(
    store_path: &Path,
    claim: &MemorySemanticProjectionClaim,
) -> Result<()> {
    let envelopes = SingleFileMessagePackBackingStore::new(store_path).pull_all()?;
    authenticate_claim_authority_from_envelopes(&envelopes, claim)?;
    if claim.authority_kind == "recovery_authorization" {
        let auth = decode_one::<MemorySemanticProjectorRecoveryAuthorization>(
            &envelopes,
            &claim.authority_id,
        )?
        .ok_or_else(|| anyhow!("semantic recovery authority disappeared"))?;
        if auth.canonical_store_id != memory_semantic_projector_store_id(store_path)? {
            return Err(anyhow!(
                "semantic recovery authority belongs to another store"
            ));
        }
    }
    Ok(())
}

fn authenticate_claim_authority_from_envelopes(
    envelopes: &[CultCacheEnvelope],
    claim: &MemorySemanticProjectionClaim,
) -> Result<()> {
    match claim.authority_kind.as_str() {
        "executor_grant" => {
            let grant =
                decode_one::<MemorySemanticProjectorExecutorGrant>(envelopes, &claim.authority_id)?
                    .ok_or_else(|| {
                        anyhow!("semantic projection claim lost consumed executor authority")
                    })?;
            validate_memory_semantic_projector_executor_grant(&grant)?;
            if grant.grant_id != claim.authority_id
                || grant.scope_id != claim.scope_id
                || grant.obligation_id != claim.obligation_id
                || grant.executor_id != claim.executor_id
                || grant.executor_incarnation != claim.executor_incarnation
                || grant.resulting_claim_id.as_deref() != Some(claim.claim_id.as_str())
                || grant.resulting_claim_epoch != Some(claim.epoch)
            {
                return Err(anyhow!(
                    "semantic projection claim executor authority disagrees"
                ));
            }
        }
        "recovery_authorization" => {
            let auth = decode_one::<MemorySemanticProjectorRecoveryAuthorization>(
                envelopes,
                &claim.authority_id,
            )?
            .ok_or_else(|| anyhow!("semantic projection claim lost consumed recovery authority"))?;
            validate_memory_semantic_projector_recovery_authorization(&auth)?;
            let abandoned_attempt =
                decode_one::<MemorySemanticProjectionAttempt>(envelopes, &auth.attempt_id)?
                    .ok_or_else(|| anyhow!("semantic recovery authority lost abandoned attempt"))?;
            validate_memory_semantic_projection_attempt(&abandoned_attempt)?;
            if auth.authorization_id != claim.authority_id
                || auth.scope_id != claim.scope_id
                || auth.obligation_id != claim.obligation_id
                || auth.replacement_executor_id != claim.executor_id
                || auth.replacement_executor_incarnation != claim.executor_incarnation
                || auth.resulting_claim_id.as_deref() != Some(claim.claim_id.as_str())
                || auth.resulting_claim_epoch != Some(claim.epoch)
                || abandoned_attempt.attempt_id != auth.attempt_id
                || abandoned_attempt.obligation_id != auth.obligation_id
                || abandoned_attempt.claim_id != auth.claim_id
                || abandoned_attempt.claim_epoch != auth.claim_epoch
                || abandoned_attempt.executor_id != auth.abandoned_executor_id
                || abandoned_attempt.executor_incarnation != auth.abandoned_executor_incarnation
                || abandoned_attempt.status != "failed"
                || abandoned_attempt.completed_at.as_deref() != auth.consumed_at.as_deref()
            {
                return Err(anyhow!(
                    "semantic projection claim recovery authority disagrees"
                ));
            }
        }
        _ => {
            return Err(anyhow!(
                "semantic projection claim authority kind is invalid"
            ));
        }
    }
    Ok(())
}

fn attempt_authenticates_claim(
    attempt: &MemorySemanticProjectionAttempt,
    claim: &MemorySemanticProjectionClaim,
) -> bool {
    attempt.attempt_id == claim.attempt_id
        && attempt.obligation_id == claim.obligation_id
        && attempt.claim_id == claim.claim_id
        && attempt.claim_epoch == claim.epoch
        && attempt.executor_id == claim.executor_id
        && attempt.executor_incarnation == claim.executor_incarnation
        && attempt.authority_id == claim.authority_id
        && attempt.started_at == claim.claimed_at
        && match (&attempt.completed_at, &claim.completed_at) {
            (None, None) => true,
            (Some(attempt_completed), Some(claim_completed)) => {
                attempt_completed == claim_completed
            }
            _ => false,
        }
}

fn authenticated_succeeded_predecessor(
    envelopes: &[CultCacheEnvelope],
    claim: &MemorySemanticProjectionClaim,
    obligation_id: &str,
) -> Result<bool> {
    if claim.status != "succeeded" || claim.obligation_id != obligation_id {
        return Ok(false);
    }
    authenticate_claim_authority_from_envelopes(envelopes, claim)?;
    let attempt = decode_one::<MemorySemanticProjectionAttempt>(envelopes, &claim.attempt_id)?
        .ok_or_else(|| anyhow!("succeeded semantic predecessor lost its exact attempt"))?;
    validate_memory_semantic_projection_attempt(&attempt)?;
    if attempt.status != "succeeded" || !attempt_authenticates_claim(&attempt, claim) {
        return Err(anyhow!(
            "succeeded semantic predecessor is not authenticated by its exact attempt"
        ));
    }
    Ok(true)
}

fn terminal_or_recovery_cas(
    store_path: &Path,
    opening: &[CultCacheEnvelope],
    claim: &MemorySemanticProjectionClaim,
    attempt: &MemorySemanticProjectionAttempt,
    mut replacements: Vec<CultCacheEnvelope>,
    extra_expected: &[CultCacheEnvelope],
) -> Result<()> {
    let obligation = exact_envelope(
        opening,
        "gamecult.epiphany.memory_semantic_projection_obligation",
        &claim.obligation_id,
    )?;
    let claim_envelope = exact_envelope(
        opening,
        "gamecult.epiphany.memory_semantic_projection_claim",
        &claim.scope_id,
    )?;
    let attempt_envelope = exact_envelope(
        opening,
        "gamecult.epiphany.memory_semantic_projection_attempt",
        &attempt.attempt_id,
    )?;
    replacements.push(obligation.clone());
    let mut expected = vec![obligation, claim_envelope, attempt_envelope];
    expected.extend_from_slice(extra_expected);
    if !SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&expected, replacements)?
    {
        return Err(anyhow!(
            "semantic projection terminal transition lost exact CAS"
        ));
    }
    Ok(())
}

fn exact_envelope(
    envelopes: &[CultCacheEnvelope],
    type_name: &str,
    key: &str,
) -> Result<CultCacheEnvelope> {
    envelopes
        .iter()
        .find(|envelope| envelope.r#type == type_name && envelope.key == key)
        .cloned()
        .ok_or_else(|| anyhow!("missing {type_name} envelope {key:?}"))
}

fn decode_one<T: DatabaseEntry + DeserializeOwned>(
    envelopes: &[CultCacheEnvelope],
    key: &str,
) -> Result<Option<T>> {
    envelopes
        .iter()
        .find(|envelope| envelope.r#type == T::TYPE && envelope.key == key)
        .map(|envelope| {
            rmp_serde::from_slice(&envelope.payload)
                .map_err(|error| anyhow!("failed to decode {} {key:?}: {error}", T::TYPE))
        })
        .transpose()
}

fn decode_all<T: DatabaseEntry + DeserializeOwned>(
    envelopes: &[CultCacheEnvelope],
) -> Result<Vec<T>> {
    envelopes
        .iter()
        .filter(|envelope| envelope.r#type == T::TYPE)
        .map(|envelope| {
            rmp_serde::from_slice(&envelope.payload).map_err(|error| {
                anyhow!("failed to decode {} {:?}: {error}", T::TYPE, envelope.key)
            })
        })
        .collect()
}

fn validate_identity_and_time(identity: &str, time: &str) -> Result<()> {
    if identity.trim().is_empty() {
        return Err(anyhow!("semantic projection executor identity is empty"));
    }
    chrono::DateTime::parse_from_rfc3339(time)
        .map_err(|_| anyhow!("semantic projection time must be RFC3339"))?;
    Ok(())
}

fn validate_opaque_identity(value: &str, label: &str) -> Result<()> {
    if !is_opaque_identity(value) {
        return Err(anyhow!("semantic projection {label} is invalid"));
    }
    Ok(())
}

fn is_opaque_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

fn valid_rfc3339(value: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(value).is_ok()
}

fn validate_idunn_semantic_recovery_evidence(
    evidence: &IdunnSemanticRecoveryEvidence,
) -> Result<()> {
    for (value, label) in [
        (&evidence.issuer_incarnation, "issuer incarnation"),
        (
            &evidence.managed_service_policy_id,
            "managed service policy identity",
        ),
        (
            &evidence.managed_service_policy_digest,
            "managed service policy digest",
        ),
        (
            &evidence.launch_lifecycle_receipt_id,
            "launch lifecycle receipt identity",
        ),
        (
            &evidence.launch_lifecycle_receipt_digest,
            "launch lifecycle receipt digest",
        ),
        (
            &evidence.provider_heartbeat_id,
            "provider heartbeat identity",
        ),
        (
            &evidence.provider_heartbeat_digest,
            "provider heartbeat digest",
        ),
        (&evidence.provider_incarnation, "provider incarnation"),
        (&evidence.canonical_store_id, "canonical store identity"),
        (&evidence.scope_id, "scope identity"),
        (&evidence.swarm_id, "swarm identity"),
        (&evidence.obligation_id, "obligation identity"),
        (&evidence.claim_id, "claim identity"),
        (
            &evidence.abandoned_executor_incarnation,
            "abandoned executor incarnation",
        ),
        (
            &evidence.startup_correlation_id,
            "startup correlation identity",
        ),
    ] {
        validate_opaque_identity(value, label)?;
    }
    if !valid_rfc3339(&evidence.launch_lifecycle_receipt_completed_at)
        || !valid_rfc3339(&evidence.provider_heartbeat_at)
        || !matches!(evidence.partition.as_str(), "mind" | "modeling")
        || evidence.claim_epoch == 0
        || evidence.provider_incarnation == evidence.abandoned_executor_incarnation
        || evidence.startup_correlation_id != evidence.launch_lifecycle_receipt_id
        || !strictly_before(
            &evidence.launch_lifecycle_receipt_completed_at,
            &evidence.provider_heartbeat_at,
        )
    {
        return Err(anyhow!("semantic recovery evidence chronology is invalid"));
    }
    Ok(())
}

fn strictly_before(earlier: &str, later: &str) -> bool {
    match (
        chrono::DateTime::parse_from_rfc3339(earlier),
        chrono::DateTime::parse_from_rfc3339(later),
    ) {
        (Ok(earlier), Ok(later)) => earlier < later,
        _ => false,
    }
}

fn not_before(later: &str, earlier: &str) -> bool {
    match (
        chrono::DateTime::parse_from_rfc3339(later),
        chrono::DateTime::parse_from_rfc3339(earlier),
    ) {
        (Ok(later), Ok(earlier)) => later >= earlier,
        _ => false,
    }
}

fn ensure_strictly_before(earlier: &str, later: &str, message: &str) -> Result<()> {
    if !strictly_before(earlier, later) {
        return Err(anyhow!(message.to_string()));
    }
    Ok(())
}

fn ensure_not_before(later: &str, earlier: &str, message: &str) -> Result<()> {
    let later = chrono::DateTime::parse_from_rfc3339(later)
        .map_err(|_| anyhow!("semantic projection later time must be RFC3339"))?;
    let earlier = chrono::DateTime::parse_from_rfc3339(earlier)
        .map_err(|_| anyhow!("semantic projection earlier time must be RFC3339"))?;
    if later < earlier {
        return Err(anyhow!(message.to_string()));
    }
    Ok(())
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
