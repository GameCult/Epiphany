use super::{
    MEMORY_SEMANTIC_PROJECTION_ATTEMPT_SCHEMA_VERSION, MemorySemanticIndexReceipt,
    MemorySemanticProjectionAttempt, MemorySemanticProjectionObligation,
    MemorySemanticProjectionSourceHead, bind_memory_semantic_index_receipt,
    memory_semantic_projection_query_eligible, validate_memory_semantic_projection_attempt,
    validate_memory_semantic_projection_obligation,
};
use anyhow::{Result, anyhow};
use cultcache_rs::{
    CacheBackingStore, CultCache, CultCacheEnvelope, DatabaseEntry,
    SingleFileMessagePackBackingStore,
};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::path::Path;

pub const MEMORY_SEMANTIC_PROJECTION_CLAIM_SCHEMA_VERSION: &str =
    "gamecult.epiphany.memory_semantic_projection_claim.v0";

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

pub(crate) fn projection_scope_id(swarm_id: &str, partition: &str) -> Result<String> {
    if swarm_id.trim().is_empty() || !matches!(partition, "mind" | "modeling") {
        return Err(anyhow!("semantic projection scope identity is invalid"));
    }
    Ok(format!(
        "memory-semantic-scope-{partition}-{:x}",
        Sha256::digest(format!("{swarm_id}|{partition}").as_bytes())
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
    receipts
        .into_iter()
        .find(|receipt| receipt.receipt_id == receipt_id)
        .map(Some)
        .ok_or_else(|| anyhow!("ready semantic projection receipt disappeared"))
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

pub fn execute_memory_semantic_projection(
    store_path: impl AsRef<Path>,
    input: &MemorySemanticProjectionInput,
    executor_id: &str,
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
    let claim =
        claim_memory_semantic_projection(store_path, &input.obligation, executor_id, &started_at)?;
    let claim = if claim.status == "succeeded" {
        if let Some(receipt) = load_memory_semantic_projection_success(
            store_path,
            &input.obligation,
            &input.authority.head,
        )? {
            return Ok(receipt);
        }
        reopen_succeeded_projection_claim(
            store_path,
            &claim,
            &input.obligation,
            executor_id,
            &now_rfc3339(),
        )?
    } else {
        claim
    };
    let raw_receipt = match super::semantic_index::index_memory_semantic_partition(
        &input.snapshot,
        &input.obligation.swarm_id,
        partition,
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

fn reopen_succeeded_projection_claim(
    store_path: &Path,
    current: &MemorySemanticProjectionClaim,
    obligation: &MemorySemanticProjectionObligation,
    executor_id: &str,
    reopened_at: &str,
) -> Result<MemorySemanticProjectionClaim> {
    validate_memory_semantic_projection_claim(current)?;
    validate_identity_and_time(executor_id, reopened_at)?;
    if current.status != "succeeded" || current.obligation_id != obligation.obligation_id {
        return Err(anyhow!(
            "semantic projection repair requires exact succeeded claim"
        ));
    }
    let cache = semantic_projector_cache(store_path)?;
    let opening = SingleFileMessagePackBackingStore::new(store_path).pull_all()?;
    let stored_claim = decode_one::<MemorySemanticProjectionClaim>(&opening, &current.scope_id)?
        .ok_or_else(|| anyhow!("semantic projection repair claim disappeared"))?;
    if stored_claim != *current {
        return Err(anyhow!("semantic projection repair claim advanced"));
    }
    let old_attempt = decode_one::<MemorySemanticProjectionAttempt>(&opening, &current.attempt_id)?
        .ok_or_else(|| anyhow!("semantic projection repair lost succeeded attempt"))?;
    if old_attempt.status != "succeeded" {
        return Err(anyhow!(
            "semantic projection repair attempt is not succeeded"
        ));
    }
    let next = running_claim(
        &current.scope_id,
        obligation,
        executor_id,
        current.epoch + 1,
        reopened_at,
    );
    let next_attempt = running_attempt(&next);
    terminal_or_recovery_cas(
        store_path,
        &opening,
        current,
        &old_attempt,
        vec![
            cache.prepare_entry(&current.scope_id, &next)?.0,
            cache
                .prepare_entry(&old_attempt.attempt_id, &old_attempt)?
                .0,
            cache
                .prepare_entry(&next_attempt.attempt_id, &next_attempt)?
                .0,
        ],
    )?;
    Ok(next)
}

pub(crate) fn claim_memory_semantic_projection(
    store_path: impl AsRef<Path>,
    obligation: &MemorySemanticProjectionObligation,
    executor_id: &str,
    claimed_at: &str,
) -> Result<MemorySemanticProjectionClaim> {
    validate_memory_semantic_projection_obligation(obligation)?;
    validate_identity_and_time(executor_id, claimed_at)?;
    let store_path = store_path.as_ref();
    let cache = semantic_projector_cache(store_path)?;
    let opening = SingleFileMessagePackBackingStore::new(store_path).pull_all()?;
    let stored =
        decode_one::<MemorySemanticProjectionObligation>(&opening, &obligation.obligation_id)?
            .ok_or_else(|| anyhow!("semantic projection claim requires persisted obligation"))?;
    if stored != *obligation {
        return Err(anyhow!("semantic projection obligation identity collision"));
    }
    let scope_id = projection_scope_id(&obligation.swarm_id, &obligation.partition)?;
    let current = decode_one::<MemorySemanticProjectionClaim>(&opening, &scope_id)?;
    if let Some(current) = &current {
        validate_memory_semantic_projection_claim(current)?;
        if current.status == "running" {
            return if current.obligation_id == obligation.obligation_id
                && current.executor_id == executor_id
            {
                Ok(current.clone())
            } else {
                Err(anyhow!(
                    "semantic projection scope is already claimed by executor {:?}",
                    current.executor_id
                ))
            };
        }
        if current.status == "succeeded" && current.obligation_id == obligation.obligation_id {
            return Ok(current.clone());
        }
    }
    let epoch = current.as_ref().map_or(1, |claim| claim.epoch + 1);
    let claim = running_claim(&scope_id, obligation, executor_id, epoch, claimed_at);
    let attempt = running_attempt(&claim);
    let obligation_envelope = exact_envelope(
        &opening,
        "gamecult.epiphany.memory_semantic_projection_obligation",
        &obligation.obligation_id,
    )?;
    let mut expected = vec![obligation_envelope.clone()];
    let mut replacements = vec![obligation_envelope];
    if current.is_some() {
        let current_envelope = exact_envelope(
            &opening,
            "gamecult.epiphany.memory_semantic_projection_claim",
            &scope_id,
        )?;
        expected.push(current_envelope);
    }
    replacements.push(cache.prepare_entry(&scope_id, &claim)?.0);
    replacements.push(cache.prepare_entry(&attempt.attempt_id, &attempt)?.0);
    if !SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&expected, replacements)?
    {
        return Err(anyhow!("semantic projection claim lost exact CAS"));
    }
    Ok(claim)
}

#[allow(dead_code)] // Deliberately withheld from callers until Idunn supplies typed recovery authority.
pub(crate) fn recover_memory_semantic_projection_claim(
    store_path: impl AsRef<Path>,
    expected_claim_id: &str,
    executor_id: &str,
    recovered_at: &str,
    reason: &str,
) -> Result<MemorySemanticProjectionClaim> {
    validate_identity_and_time(executor_id, recovered_at)?;
    if expected_claim_id.trim().is_empty() || reason.trim().is_empty() {
        return Err(anyhow!(
            "semantic projection recovery requires claim and reason"
        ));
    }
    let store_path = store_path.as_ref();
    let cache = semantic_projector_cache(store_path)?;
    let opening = SingleFileMessagePackBackingStore::new(store_path).pull_all()?;
    let current = decode_all::<MemorySemanticProjectionClaim>(&opening)?
        .into_iter()
        .find(|claim| claim.claim_id == expected_claim_id)
        .ok_or_else(|| anyhow!("semantic projection recovery claim is missing"))?;
    validate_memory_semantic_projection_claim(&current)?;
    if current.status != "running" {
        return Err(anyhow!(
            "only a running semantic projection claim can recover"
        ));
    }
    let obligation =
        decode_one::<MemorySemanticProjectionObligation>(&opening, &current.obligation_id)?
            .ok_or_else(|| anyhow!("semantic projection recovery lost obligation"))?;
    let old_attempt = decode_one::<MemorySemanticProjectionAttempt>(&opening, &current.attempt_id)?
        .ok_or_else(|| anyhow!("semantic projection recovery lost running attempt"))?;
    validate_memory_semantic_projection_attempt(&old_attempt)?;
    if old_attempt.status != "running" {
        return Err(anyhow!(
            "semantic projection recovery attempt is not running"
        ));
    }
    let next = running_claim(
        &current.scope_id,
        &obligation,
        executor_id,
        current.epoch + 1,
        recovered_at,
    );
    let next_attempt = running_attempt(&next);
    let failed_attempt = MemorySemanticProjectionAttempt {
        completed_at: Some(recovered_at.to_string()),
        status: "failed".to_string(),
        error: Some(format!("superseded by fenced recovery: {reason}")),
        ..old_attempt.clone()
    };
    terminal_or_recovery_cas(
        store_path,
        &opening,
        &current,
        &old_attempt,
        vec![
            cache.prepare_entry(&current.scope_id, &next)?.0,
            cache
                .prepare_entry(&failed_attempt.attempt_id, &failed_attempt)?
                .0,
            cache
                .prepare_entry(&next_attempt.attempt_id, &next_attempt)?
                .0,
        ],
    )?;
    Ok(next)
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
    let obligation =
        decode_one::<MemorySemanticProjectionObligation>(&opening, &claim.obligation_id)?
            .ok_or_else(|| anyhow!("semantic projection success lost obligation"))?;
    let receipt = bind_memory_semantic_index_receipt(receipt, &obligation)?;
    if !memory_semantic_projection_query_eligible(&obligation, &authority.head, &receipt) {
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

fn semantic_projector_cache(store_path: &Path) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<MemorySemanticProjectionObligation>()?;
    cache.register_entry_type::<MemorySemanticProjectionClaim>()?;
    cache.register_entry_type::<MemorySemanticProjectionAttempt>()?;
    cache.register_entry_type::<MemorySemanticIndexReceipt>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(store_path));
    Ok(cache)
}

fn running_claim(
    scope_id: &str,
    obligation: &MemorySemanticProjectionObligation,
    executor_id: &str,
    epoch: u64,
    claimed_at: &str,
) -> MemorySemanticProjectionClaim {
    let fingerprint = format!(
        "{}|{}|{}|{}",
        scope_id, obligation.obligation_id, executor_id, epoch
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
    if attempt.status != "running" || attempt.obligation_id != claim.obligation_id {
        return Err(anyhow!("semantic projection attempt does not own claim"));
    }
    Ok((cache, opening, claim, attempt))
}

fn terminal_or_recovery_cas(
    store_path: &Path,
    opening: &[CultCacheEnvelope],
    claim: &MemorySemanticProjectionClaim,
    attempt: &MemorySemanticProjectionAttempt,
    mut replacements: Vec<CultCacheEnvelope>,
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
    if !SingleFileMessagePackBackingStore::new(store_path).compare_and_swap_batch(
        &[obligation, claim_envelope, attempt_envelope],
        replacements,
    )? {
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

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION,
        MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION, SEMANTIC_PROJECTION_SCHEMA_VERSION,
    };
    use tempfile::tempdir;

    fn obligation() -> MemorySemanticProjectionObligation {
        MemorySemanticProjectionObligation {
            schema_version: MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION.to_string(),
            obligation_id: "obligation-modeling-7".to_string(),
            swarm_id: "swarm-a".to_string(),
            partition: "modeling".to_string(),
            canonical_source_id: "runtime/repo-model".to_string(),
            source_commit_id: "admission-7".to_string(),
            graph_id: "repo-model".to_string(),
            source_generation: 7,
            source_model_hash: "model-hash-7".to_string(),
            canonical_content_set_hash: "content-set-7".to_string(),
            projection_schema_version: SEMANTIC_PROJECTION_SCHEMA_VERSION.to_string(),
            created_at: "2026-07-15T04:00:00Z".to_string(),
        }
    }

    fn source_head() -> MemorySemanticProjectionSourceHead {
        let obligation = obligation();
        MemorySemanticProjectionSourceHead {
            swarm_id: obligation.swarm_id,
            partition: obligation.partition,
            canonical_source_id: obligation.canonical_source_id,
            source_commit_id: obligation.source_commit_id,
            graph_id: obligation.graph_id,
            source_generation: obligation.source_generation,
            source_model_hash: obligation.source_model_hash,
            canonical_content_set_hash: obligation.canonical_content_set_hash,
        }
    }

    fn receipt() -> MemorySemanticIndexReceipt {
        let obligation = obligation();
        MemorySemanticIndexReceipt {
            schema_version: MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "receipt-modeling-7".to_string(),
            swarm_id: obligation.swarm_id,
            partition: obligation.partition,
            collection_name: "modeling".to_string(),
            graph_id: obligation.graph_id,
            model_revision: obligation.source_generation,
            model_hash: obligation.source_model_hash,
            embedding_provider_id: "embedder".to_string(),
            embedding_model: "model".to_string(),
            vector_dimensions: 3,
            indexed_document_count: 1,
            deleted_document_count: 0,
            canonical_content_set_hash: obligation.canonical_content_set_hash,
            indexed_at: "2026-07-15T04:03:00Z".to_string(),
            status: "ready".to_string(),
            obligation_id: String::new(),
            canonical_source_id: String::new(),
            source_commit_id: String::new(),
            source_generation: obligation.source_generation,
            projection_schema_version: SEMANTIC_PROJECTION_SCHEMA_VERSION.to_string(),
        }
    }

    #[test]
    fn scope_claim_recovery_fences_old_executor_and_terminalizes_exact_success() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("projector.msgpack");
        let obligation = obligation();
        let mut cache = semantic_projector_cache(&store)?;
        cache.put(&obligation.obligation_id, &obligation)?;

        let first = claim_memory_semantic_projection(
            &store,
            &obligation,
            "executor-a",
            "2026-07-15T04:01:00Z",
        )?;
        assert!(
            claim_memory_semantic_projection(
                &store,
                &obligation,
                "executor-b",
                "2026-07-15T04:01:01Z"
            )
            .is_err()
        );
        let recovered = recover_memory_semantic_projection_claim(
            &store,
            &first.claim_id,
            "executor-b",
            "2026-07-15T04:02:00Z",
            "daemon restarted under operator recovery",
        )?;
        assert_eq!(recovered.epoch, first.epoch + 1);
        assert!(
            fail_memory_semantic_projection_claim(
                &store,
                &first.claim_id,
                "2026-07-15T04:02:30Z",
                "late old executor"
            )
            .is_err()
        );

        let mut cache = semantic_projector_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let authority_envelope = cache
            .snapshot_envelopes()
            .into_iter()
            .find(|envelope| {
                envelope.r#type == "gamecult.epiphany.memory_semantic_projection_obligation"
                    && envelope.key == obligation.obligation_id
            })
            .expect("obligation envelope");
        let bound = succeed_memory_semantic_projection_claim(
            &store,
            &recovered.claim_id,
            &MemorySemanticProjectionAuthoritySnapshot {
                head: source_head(),
                envelopes: vec![authority_envelope],
            },
            receipt(),
            "2026-07-15T04:03:00Z",
        )?;
        assert_eq!(bound.obligation_id, obligation.obligation_id);

        let mut cache = semantic_projector_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let terminal = cache
            .get::<MemorySemanticProjectionClaim>(&recovered.scope_id)?
            .expect("terminal claim");
        assert_eq!(terminal.status, "succeeded");
        let old_attempt = cache
            .get::<MemorySemanticProjectionAttempt>(&first.attempt_id)?
            .expect("fenced old attempt");
        assert_eq!(old_attempt.status, "failed");
        let new_attempt = cache
            .get::<MemorySemanticProjectionAttempt>(&recovered.attempt_id)?
            .expect("successful recovered attempt");
        assert_eq!(new_attempt.status, "succeeded");

        let repair_attempt = MemorySemanticProjectionAttempt {
            schema_version: MEMORY_SEMANTIC_PROJECTION_ATTEMPT_SCHEMA_VERSION.to_string(),
            attempt_id: "repair-after-success".to_string(),
            obligation_id: obligation.obligation_id.clone(),
            started_at: "2026-07-15T04:04:00Z".to_string(),
            completed_at: Some("2026-07-15T04:04:01Z".to_string()),
            status: "failed".to_string(),
            error: Some("observed cache corruption".to_string()),
        };
        cache.put(&repair_attempt.attempt_id, &repair_attempt)?;
        assert!(
            load_memory_semantic_projection_success(&store, &obligation, &source_head())?.is_none()
        );
        Ok(())
    }

    #[test]
    fn terminal_success_refuses_a_preexisting_receipt_identity_collision() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("receipt-collision.msgpack");
        let obligation = obligation();
        let mut cache = semantic_projector_cache(&store)?;
        cache.put(&obligation.obligation_id, &obligation)?;
        let claim = claim_memory_semantic_projection(
            &store,
            &obligation,
            "executor-a",
            "2026-07-15T04:01:00Z",
        )?;
        let mut collision = receipt();
        collision.status = "failed".to_string();
        cache.put(&collision.receipt_id, &collision)?;
        let authority_envelope = SingleFileMessagePackBackingStore::new(&store)
            .pull_all()?
            .into_iter()
            .find(|envelope| {
                envelope.r#type == "gamecult.epiphany.memory_semantic_projection_obligation"
                    && envelope.key == obligation.obligation_id
            })
            .expect("obligation envelope");
        assert!(
            succeed_memory_semantic_projection_claim(
                &store,
                &claim.claim_id,
                &MemorySemanticProjectionAuthoritySnapshot {
                    head: source_head(),
                    envelopes: vec![authority_envelope],
                },
                receipt(),
                "2026-07-15T04:03:00Z",
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn succeeded_claim_with_missing_receipt_reopens_under_a_new_fence() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("missing-receipt.msgpack");
        let obligation = obligation();
        let mut cache = semantic_projector_cache(&store)?;
        cache.put(&obligation.obligation_id, &obligation)?;
        let claim = claim_memory_semantic_projection(
            &store,
            &obligation,
            "executor-a",
            "2026-07-15T04:01:00Z",
        )?;
        let authority_envelope = SingleFileMessagePackBackingStore::new(&store)
            .pull_all()?
            .into_iter()
            .find(|envelope| {
                envelope.r#type == "gamecult.epiphany.memory_semantic_projection_obligation"
                    && envelope.key == obligation.obligation_id
            })
            .expect("obligation envelope");
        let bound = succeed_memory_semantic_projection_claim(
            &store,
            &claim.claim_id,
            &MemorySemanticProjectionAuthoritySnapshot {
                head: source_head(),
                envelopes: vec![authority_envelope],
            },
            receipt(),
            "2026-07-15T04:03:00Z",
        )?;
        let mut cache = semantic_projector_cache(&store)?;
        cache.pull_all_backing_stores()?;
        cache.delete::<MemorySemanticIndexReceipt>(&bound.receipt_id)?;
        let terminal = cache
            .get::<MemorySemanticProjectionClaim>(&claim.scope_id)?
            .expect("succeeded claim");
        let reopened = reopen_succeeded_projection_claim(
            &store,
            &terminal,
            &obligation,
            "executor-b",
            "2026-07-15T04:04:00Z",
        )?;
        assert_eq!(reopened.status, "running");
        assert_eq!(reopened.epoch, terminal.epoch + 1);
        assert_ne!(reopened.claim_id, terminal.claim_id);
        Ok(())
    }
}
