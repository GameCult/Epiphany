use anyhow::{Result, bail};
use cultcache_rs::{
    CultCache, CultCacheEnvelope, DatabaseEntry, SingleFileMessagePackBackingStore,
};
use cultnet_rs::{ServiceIdentityProfile, derive_service_identity_id};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::contracts::*;
use super::identity::{
    AtlasPublisherTrustBinding, AtlasRepositorySigningIdentity, sha256,
    validate_atlas_publication_statement, verify_atlas_publication,
};
use super::impact_ingress::{
    AtlasImpactBrakeState, AtlasImpactLane, AtlasImpactLaneState, AtlasImpactProposalAuthority,
    AtlasImpactScheduleDisposition, AtlasImpactSchedulingDecision, AtlasLocalImpactProposal,
};
use super::projector::atlas_projection_digest;
use super::publisher::{
    AtlasLatestPublicationPointer, AtlasLatestPublicationPointerIntent, AtlasLocalMindDocument,
    AtlasLocalPublicationSnapshot, AtlasLocalPublicationStore, AtlasPublicationBatch,
};

pub const ATLAS_LATEST_POINTER_RECORD_SCHEMA: &str = "epiphany.atlas.latest_publication_pointer.v0";
pub const ATLAS_TRUST_BINDING_RECORD_SCHEMA: &str = "epiphany.atlas.publisher_trust_binding.v0";
pub const ATLAS_IMPACT_DEDUPE_RECORD_SCHEMA: &str = "epiphany.atlas.impact_dedupe.v0";
pub const ATLAS_IMPACT_STATE_RECORD_SCHEMA: &str = "epiphany.atlas.impact_state.v0";

/// Immutable publications and dedupe rows may make a retried write already
/// true. Conflict is deliberately distinct: callers must reload rather than
/// pretending a stale CAS converged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtlasStoreWriteOutcome {
    Applied,
    AlreadyApplied,
    Conflict,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.atlas.latest_publication_pointer.v0",
    schema = "epiphany.atlas.latest_publication_pointer.v0"
)]
pub struct AtlasLatestPublicationPointerRecord {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub pointer: AtlasLatestPublicationPointer,
}

impl AtlasLatestPublicationPointerRecord {
    pub fn validate(&self) -> Result<()> {
        require_schema(&self.schema_version, ATLAS_LATEST_POINTER_RECORD_SCHEMA)?;
        self.pointer.validate()
    }
}

/// Trust is local projector policy, not a fact asserted by the publisher. The
/// revision exists only to fence concurrent local policy edits.
#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.atlas.publisher_trust_binding.v0",
    schema = "epiphany.atlas.publisher_trust_binding.v0"
)]
pub struct AtlasPublisherTrustBindingRecord {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub revision: u64,
    #[cultcache(key = 2)]
    pub binding: AtlasPublisherTrustBinding,
}

impl AtlasPublisherTrustBindingRecord {
    pub fn validate(&self) -> Result<()> {
        require_schema(&self.schema_version, ATLAS_TRUST_BINDING_RECORD_SCHEMA)?;
        if self.revision == 0 {
            bail!("Atlas stored trust binding revision must be positive")
        }
        validate_trust_binding_structure(&self.binding)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.atlas.impact_dedupe.v0",
    schema = "epiphany.atlas.impact_dedupe.v0"
)]
pub struct AtlasImpactDedupeRecord {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub local_repository: AtlasRepositoryIdentity,
    #[cultcache(key = 2)]
    pub proposal: AtlasLocalImpactProposal,
    #[cultcache(key = 3)]
    pub scheduling_decision: AtlasImpactSchedulingDecision,
    #[cultcache(key = 4)]
    pub recorded_at_unix_ms: u64,
}

impl AtlasImpactDedupeRecord {
    pub fn validate(&self) -> Result<()> {
        require_schema(&self.schema_version, ATLAS_IMPACT_DEDUPE_RECORD_SCHEMA)?;
        self.local_repository.validate()?;
        self.proposal.impact.validate()?;
        validate_sha256(&self.proposal.dedupe_key, "Atlas impact dedupe key")?;
        if self.recorded_at_unix_ms == 0
            || self.proposal.authority != AtlasImpactProposalAuthority::LocalReviewOnly
            || self.proposal.impact.consumer != self.local_repository
            || self.proposal.proposal_id != self.proposal.impact.impact_id
            || self.proposal.proposal_id
                != Uuid::new_v5(&Uuid::NAMESPACE_OID, self.proposal.dedupe_key.as_bytes())
            || self.scheduling_decision.proposal_id != self.proposal.proposal_id
            || self.scheduling_decision.claim_id != self.proposal.impact.claim_id
            || self.scheduling_decision.criticality != self.proposal.impact.criticality
        {
            bail!("Atlas impact dedupe row substituted proposal, owner, or decision identity")
        }
        match &self.scheduling_decision.disposition {
            AtlasImpactScheduleDisposition::Schedule { lane }
            | AtlasImpactScheduleDisposition::HeldByPendingLane { lane, .. }
            | AtlasImpactScheduleDisposition::HeldByCooldown { lane, .. }
                if *lane != self.proposal.lane =>
            {
                bail!("Atlas impact decision lane differs from its proposal lane")
            }
            AtlasImpactScheduleDisposition::Deduplicated => {
                bail!("Atlas cannot create a new dedupe row from an already-deduplicated decision")
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasStoredImpactLaneState {
    pub pending_proposal_id: Option<Uuid>,
    pub last_completed_at_unix_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasStoredImpactBrakeState {
    pub engaged: bool,
    pub brake_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.atlas.impact_state.v0",
    schema = "epiphany.atlas.impact_state.v0"
)]
pub struct AtlasImpactStateRecord {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub local_repository: AtlasRepositoryIdentity,
    #[cultcache(key = 2)]
    pub revision: u64,
    #[cultcache(key = 3)]
    pub modeling: AtlasStoredImpactLaneState,
    #[cultcache(key = 4)]
    pub soul: AtlasStoredImpactLaneState,
    #[cultcache(key = 5)]
    pub brake: AtlasStoredImpactBrakeState,
    #[cultcache(key = 6)]
    pub updated_at_unix_ms: u64,
}

impl AtlasImpactStateRecord {
    pub fn validate(&self) -> Result<()> {
        require_schema(&self.schema_version, ATLAS_IMPACT_STATE_RECORD_SCHEMA)?;
        self.local_repository.validate()?;
        if self.revision == 0 || self.updated_at_unix_ms == 0 {
            bail!("Atlas impact state revision and update time must be positive")
        }
        match (self.brake.engaged, &self.brake.brake_id) {
            (true, Some(brake_id)) => validate_identifier(brake_id, "Atlas impact state brake id")?,
            (false, None) => {}
            _ => bail!("Atlas stored impact brake state is internally inconsistent"),
        }
        Ok(())
    }

    pub fn ingress_lane_states(&self) -> Vec<AtlasImpactLaneState> {
        vec![
            AtlasImpactLaneState {
                lane: AtlasImpactLane::Modeling,
                pending_proposal_id: self.modeling.pending_proposal_id,
                last_completed_at_unix_ms: self.modeling.last_completed_at_unix_ms,
            },
            AtlasImpactLaneState {
                lane: AtlasImpactLane::Soul,
                pending_proposal_id: self.soul.pending_proposal_id,
                last_completed_at_unix_ms: self.soul.last_completed_at_unix_ms,
            },
        ]
    }

    pub fn ingress_brake_state(&self) -> AtlasImpactBrakeState {
        AtlasImpactBrakeState {
            engaged: self.brake.engaged,
            brake_id: self.brake.brake_id.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasLocalMindSnapshot {
    pub publisher: AtlasRepositoryIdentity,
    pub current_mind_documents: Vec<AtlasLocalMindDocument>,
}

/// This port can expose only one already-bounded local Mind snapshot. It has no
/// workspace selector and cannot be used to open a foreign Mind.
pub trait AtlasLocalMindSnapshotSource {
    fn load_local_atlas_mind_snapshot(&self) -> Result<AtlasLocalMindSnapshot>;
}

pub trait AtlasLatestPublicationPointerReader {
    fn local_repository(&self) -> &AtlasRepositoryIdentity;
    fn load_latest_publication_pointers(&self) -> Result<Vec<AtlasLatestPublicationPointer>>;
}

/// Joins local Mind sight to local pointer persistence for the existing pure
/// publisher. Neither side receives the other's storage authority.
pub struct AtlasPublisherSnapshotAdapter<'a, M: ?Sized, P: ?Sized> {
    mind: &'a M,
    pointers: &'a P,
}

impl<'a, M: ?Sized, P: ?Sized> AtlasPublisherSnapshotAdapter<'a, M, P> {
    pub fn new(mind: &'a M, pointers: &'a P) -> Self {
        Self { mind, pointers }
    }
}

impl<M, P> AtlasLocalPublicationStore for AtlasPublisherSnapshotAdapter<'_, M, P>
where
    M: AtlasLocalMindSnapshotSource + ?Sized,
    P: AtlasLatestPublicationPointerReader + ?Sized,
{
    fn load_local_atlas_snapshot(&self) -> Result<AtlasLocalPublicationSnapshot> {
        let snapshot = self.mind.load_local_atlas_mind_snapshot()?;
        if &snapshot.publisher != self.pointers.local_repository() {
            bail!("Atlas publisher adapter refused a local Mind/pointer repository mismatch")
        }
        Ok(AtlasLocalPublicationSnapshot {
            publisher: snapshot.publisher,
            current_mind_documents: snapshot.current_mind_documents,
            latest_pointers: self.pointers.load_latest_publication_pointers()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasProjectionStoreInputs {
    pub publications: Vec<AtlasPublicationEnvelope>,
    pub trust_bindings: Vec<AtlasPublisherTrustBinding>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasImpactStoreSnapshot {
    pub state_revision: Option<u64>,
    pub seen_dedupe_keys: BTreeSet<String>,
    pub lane_states: Vec<AtlasImpactLaneState>,
    pub brake: AtlasImpactBrakeState,
}

pub trait AtlasPublicationEventStore {
    fn load_publication_events(&self) -> Result<Vec<AtlasPublicationEnvelope>>;
    fn append_verified_publication_event(
        &self,
        publication: &AtlasPublicationEnvelope,
        now_unix_ms: u64,
    ) -> Result<AtlasStoreWriteOutcome>;
    fn commit_local_publication_batch(
        &self,
        batch: &AtlasPublicationBatch,
        now_unix_ms: u64,
    ) -> Result<AtlasStoreWriteOutcome>;
}

pub trait AtlasTrustBindingStore {
    fn load_trust_binding_records(&self) -> Result<Vec<AtlasPublisherTrustBindingRecord>>;
    fn compare_and_swap_trust_binding(
        &self,
        expected_revision: Option<u64>,
        next_binding: AtlasPublisherTrustBinding,
    ) -> Result<AtlasStoreWriteOutcome>;
}

pub trait AtlasProjectionStore {
    fn load_projection_inputs(&self, now_unix_ms: u64) -> Result<AtlasProjectionStoreInputs>;
    fn load_latest_projection(
        &self,
        audience_id: &str,
    ) -> Result<Option<AtlasEntanglementProjection>>;
    fn compare_and_swap_latest_projection(
        &self,
        expected_projection_sha256: Option<&str>,
        next: &AtlasEntanglementProjection,
    ) -> Result<AtlasStoreWriteOutcome>;
}

pub trait AtlasImpactStore {
    fn load_impact_snapshot(&self) -> Result<AtlasImpactStoreSnapshot>;
    fn compare_and_swap_impact_update(
        &self,
        expected_state_revision: Option<u64>,
        next_state: &AtlasImpactStateRecord,
        new_dedupe_records: &[AtlasImpactDedupeRecord],
    ) -> Result<AtlasStoreWriteOutcome>;
}

/// One repository-scoped Atlas persistence body. It stores federated signed
/// publications and local derived/control surfaces, but owns no dependency
/// edge and exposes no Mind-opening operation.
#[derive(Clone, Debug)]
pub struct AtlasCultCacheStore {
    path: PathBuf,
    local_repository: AtlasRepositoryIdentity,
}

impl AtlasCultCacheStore {
    pub fn new(
        path: impl Into<PathBuf>,
        local_repository: AtlasRepositoryIdentity,
    ) -> Result<Self> {
        local_repository.validate()?;
        Ok(Self {
            path: path.into(),
            local_repository,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn local_repository(&self) -> &AtlasRepositoryIdentity {
        &self.local_repository
    }

    fn load_cache(&self) -> Result<CultCache> {
        let mut cache = CultCache::new();
        cache.register_entry_type::<AtlasPublicationEnvelope>()?;
        cache.register_entry_type::<AtlasLatestPublicationPointerRecord>()?;
        cache.register_entry_type::<AtlasPublisherTrustBindingRecord>()?;
        cache.register_entry_type::<AtlasEntanglementProjection>()?;
        cache.register_entry_type::<AtlasImpactDedupeRecord>()?;
        cache.register_entry_type::<AtlasImpactStateRecord>()?;
        cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(&self.path));
        cache.pull_all_backing_stores()?;
        Ok(cache)
    }

    fn backing(&self) -> SingleFileMessagePackBackingStore {
        SingleFileMessagePackBackingStore::new(&self.path)
    }

    fn pointer_records(
        &self,
        cache: &CultCache,
    ) -> Result<Vec<(String, AtlasLatestPublicationPointerRecord)>> {
        let records = cache.get_all_with_keys::<AtlasLatestPublicationPointerRecord>()?;
        for (key, record) in &records {
            record.validate()?;
            if record.pointer.publisher != self.local_repository
                || *key != latest_pointer_key(&record.pointer)?
            {
                bail!("Atlas latest pointer store contains a foreign or mis-keyed record")
            }
            let publication = cache
                .get::<AtlasPublicationEnvelope>(&record.pointer.publication_id)?
                .ok_or_else(|| {
                    anyhow::anyhow!("Atlas latest pointer has no immutable publication event")
                })?;
            if !pointer_matches_publication(&record.pointer, &publication) {
                bail!("Atlas latest pointer differs from its immutable publication event")
            }
        }
        Ok(records)
    }

    fn trust_records(
        &self,
        cache: &CultCache,
    ) -> Result<Vec<(String, AtlasPublisherTrustBindingRecord)>> {
        let records = cache.get_all_with_keys::<AtlasPublisherTrustBindingRecord>()?;
        for (key, record) in &records {
            record.validate()?;
            if *key != trust_binding_key(&record.binding.publisher)? {
                bail!("Atlas trust binding store contains a mis-keyed record")
            }
        }
        Ok(records)
    }

    fn active_trust_directory(
        &self,
        cache: &CultCache,
        now_unix_ms: u64,
    ) -> Result<BTreeMap<String, AtlasPublisherTrustBinding>> {
        let mut active = BTreeMap::new();
        for (_, record) in self.trust_records(cache)? {
            let binding = record.binding;
            if binding.revoked
                || binding.trusted_from_unix_ms > now_unix_ms
                || binding
                    .expires_at_unix_ms
                    .is_some_and(|expires| expires <= now_unix_ms)
            {
                continue;
            }
            binding.validate_at(now_unix_ms)?;
            if active
                .insert(binding.publisher.repository_uri.clone(), binding)
                .is_some()
            {
                bail!("Atlas trust store contains conflicting active repository bindings")
            }
        }
        Ok(active)
    }

    fn publication_rows(
        &self,
        cache: &CultCache,
    ) -> Result<Vec<(String, AtlasPublicationEnvelope)>> {
        let mut rows = cache.get_all_with_keys::<AtlasPublicationEnvelope>()?;
        for (key, publication) in &rows {
            validate_atlas_publication_statement(&publication.statement)?;
            if *key != publication.statement.publication_id {
                bail!("Atlas immutable publication event is stored under the wrong id")
            }
        }
        rows.sort_by(|left, right| {
            (
                left.1.statement.published_at_unix_ms,
                &left.1.statement.publisher.repository_uri,
                left.1.statement.publication_sequence,
                &left.1.statement.publication_id,
            )
                .cmp(&(
                    right.1.statement.published_at_unix_ms,
                    &right.1.statement.publisher.repository_uri,
                    right.1.statement.publication_sequence,
                    &right.1.statement.publication_id,
                ))
        });
        Ok(rows)
    }

    fn impact_records(&self, cache: &CultCache) -> Result<Vec<(String, AtlasImpactDedupeRecord)>> {
        let records = cache.get_all_with_keys::<AtlasImpactDedupeRecord>()?;
        for (key, record) in &records {
            record.validate()?;
            if record.local_repository != self.local_repository
                || *key != record.proposal.dedupe_key
            {
                bail!("Atlas impact store contains a foreign or mis-keyed dedupe record")
            }
        }
        Ok(records)
    }

    fn current_impact_state(
        &self,
        cache: &CultCache,
    ) -> Result<Option<(CultCacheEnvelope, AtlasImpactStateRecord)>> {
        let key = impact_state_key(&self.local_repository)?;
        let Some(envelope) = cache.get_envelope::<AtlasImpactStateRecord>(&key)? else {
            return Ok(None);
        };
        let state = cache
            .get::<AtlasImpactStateRecord>(&key)?
            .expect("typed envelope and value share one cache image");
        state.validate()?;
        if state.local_repository != self.local_repository {
            bail!("Atlas impact store state belongs to a foreign repository")
        }
        Ok(Some((envelope, state)))
    }
}

impl AtlasLatestPublicationPointerReader for AtlasCultCacheStore {
    fn local_repository(&self) -> &AtlasRepositoryIdentity {
        &self.local_repository
    }

    fn load_latest_publication_pointers(&self) -> Result<Vec<AtlasLatestPublicationPointer>> {
        let cache = self.load_cache()?;
        let mut pointers = self
            .pointer_records(&cache)?
            .into_iter()
            .map(|(_, record)| record.pointer)
            .collect::<Vec<_>>();
        pointers.sort_by(|left, right| {
            (&left.source_schema, &left.source_key).cmp(&(&right.source_schema, &right.source_key))
        });
        Ok(pointers)
    }
}

impl AtlasPublicationEventStore for AtlasCultCacheStore {
    fn load_publication_events(&self) -> Result<Vec<AtlasPublicationEnvelope>> {
        Ok(self
            .publication_rows(&self.load_cache()?)?
            .into_iter()
            .map(|(_, publication)| publication)
            .collect())
    }

    fn append_verified_publication_event(
        &self,
        publication: &AtlasPublicationEnvelope,
        now_unix_ms: u64,
    ) -> Result<AtlasStoreWriteOutcome> {
        let cache = self.load_cache()?;
        let trust = self.active_trust_directory(&cache, now_unix_ms)?;
        let binding = trust
            .get(&publication.statement.publisher.repository_uri)
            .filter(|binding| {
                binding.signer_identity_id == publication.statement.publisher_identity_id
            })
            .ok_or_else(|| {
                anyhow::anyhow!("Atlas publication has no active exact trust binding")
            })?;
        verify_atlas_publication(binding, publication, now_unix_ms)?;

        let key = publication.statement.publication_id.clone();
        if let Some(existing) = cache.get::<AtlasPublicationEnvelope>(&key)? {
            if existing == *publication {
                return Ok(AtlasStoreWriteOutcome::AlreadyApplied);
            }
            bail!("Atlas publication id collides with a different immutable event")
        }
        let envelope = cache.prepare_entry(key.clone(), publication)?.0;
        if self.backing().insert_entry_if_absent(envelope)? {
            return Ok(AtlasStoreWriteOutcome::Applied);
        }
        match self.load_cache()?.get::<AtlasPublicationEnvelope>(&key)? {
            Some(existing) if existing == *publication => {
                Ok(AtlasStoreWriteOutcome::AlreadyApplied)
            }
            Some(_) => bail!("Atlas publication id raced with a different immutable event"),
            None => Ok(AtlasStoreWriteOutcome::Conflict),
        }
    }

    fn commit_local_publication_batch(
        &self,
        batch: &AtlasPublicationBatch,
        now_unix_ms: u64,
    ) -> Result<AtlasStoreWriteOutcome> {
        if batch.publications.is_empty() && batch.latest_pointer_intents.is_empty() {
            return Ok(AtlasStoreWriteOutcome::AlreadyApplied);
        }
        if batch.publications.len() != batch.latest_pointer_intents.len() {
            bail!("Atlas local publication commit requires one pointer intent per new event")
        }

        let cache = self.load_cache()?;
        let trust = self.active_trust_directory(&cache, now_unix_ms)?;
        let mut publications = BTreeMap::new();
        let mut sequences = BTreeSet::new();
        for publication in &batch.publications {
            if publication.statement.publisher != self.local_repository {
                bail!("Atlas local publication batch contains a foreign publisher")
            }
            let binding = trust
                .get(&self.local_repository.repository_uri)
                .filter(|binding| {
                    binding.signer_identity_id == publication.statement.publisher_identity_id
                })
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Atlas local publication batch has no active exact trust binding"
                    )
                })?;
            verify_atlas_publication(binding, publication, now_unix_ms)?;
            if !sequences.insert(publication.statement.publication_sequence)
                || publications
                    .insert(publication.statement.publication_id.clone(), publication)
                    .is_some()
            {
                bail!("Atlas local publication batch contains duplicate event identity or sequence")
            }
        }

        let current_pointers = self
            .pointer_records(&cache)?
            .into_iter()
            .map(|(key, record)| {
                (
                    (
                        record.pointer.source_schema.clone(),
                        record.pointer.source_key.clone(),
                    ),
                    (key, record),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let prior_maximum_sequence = current_pointers
            .values()
            .map(|(_, record)| record.pointer.publication_sequence)
            .max()
            .unwrap_or(0);
        let status_publications = publications
            .values()
            .filter(|publication| {
                matches!(
                    &publication.statement.payload,
                    AtlasPublicationPayload::PublisherStatus(_)
                )
            })
            .copied()
            .collect::<Vec<_>>();
        if status_publications.len() != 1 {
            bail!("Every non-empty Atlas local publication batch requires one status head")
        }
        let status_publication = status_publications[0];
        if publications.values().any(|publication| {
            publication.statement.publication_sequence
                > status_publication.statement.publication_sequence
        }) {
            bail!("Atlas publisher status must close the local publication batch sequence")
        }
        let mut intents = BTreeMap::<(String, String), &AtlasLatestPublicationPointerIntent>::new();
        for intent in &batch.latest_pointer_intents {
            intent.validate()?;
            if intent.publisher != self.local_repository
                || intents
                    .insert(
                        (intent.source_schema.clone(), intent.source_key.clone()),
                        intent,
                    )
                    .is_some()
            {
                bail!(
                    "Atlas local publication batch contains a foreign or duplicate pointer intent"
                )
            }
            let publication = publications
                .get(&intent.next.publication_id)
                .ok_or_else(|| anyhow::anyhow!("Atlas pointer intent has no immutable event"))?;
            if !pointer_matches_publication(&intent.next, publication) {
                bail!("Atlas pointer intent does not bind its exact immutable event")
            }
        }
        if publications.keys().any(|publication_id| {
            !intents
                .values()
                .any(|intent| &intent.next.publication_id == publication_id)
        }) {
            bail!("Atlas local publication batch would create an unpointed immutable event")
        }
        let mut prospective_pointers = current_pointers
            .iter()
            .map(|(identity, (_, record))| (identity.clone(), record.pointer.clone()))
            .collect::<BTreeMap<_, _>>();
        for (identity, intent) in &intents {
            prospective_pointers.insert(identity.clone(), intent.next.clone());
        }
        let AtlasPublicationPayload::PublisherStatus(status) =
            &status_publication.statement.payload
        else {
            unreachable!("status publication was selected by its typed payload")
        };
        for watermark in &status.watermarks {
            let pointer = prospective_pointers
                .get(&(
                    watermark.source_schema.clone(),
                    watermark.source_key.clone(),
                ))
                .ok_or_else(|| {
                    anyhow::anyhow!("Atlas status watermark has no persisted latest pointer")
                })?;
            if pointer.source_payload_sha256 != watermark.source_payload_sha256
                || pointer.publication_sequence != watermark.publication_sequence
            {
                bail!("Atlas status watermark differs from its exact persisted latest pointer")
            }
        }
        if intents.iter().any(|(identity, intent)| {
            !matches!(
                &publications[&intent.next.publication_id].statement.payload,
                AtlasPublicationPayload::PublisherStatus(_)
            ) && !status.watermarks.iter().any(|watermark| {
                identity
                    == &(
                        watermark.source_schema.clone(),
                        watermark.source_key.clone(),
                    )
                    && watermark.source_payload_sha256 == intent.next.source_payload_sha256
                    && watermark.publication_sequence == intent.next.publication_sequence
            })
        }) {
            bail!("Atlas status omitted a newly advanced local document pointer")
        }

        if publication_batch_is_applied(&cache, &intents, &publications)? {
            return Ok(AtlasStoreWriteOutcome::AlreadyApplied);
        }
        if publications
            .values()
            .any(|publication| publication.statement.publication_sequence <= prior_maximum_sequence)
        {
            bail!("Atlas local publication batch sequence did not advance its publisher head")
        }
        if intents.values().any(|intent| {
            current_pointers
                .get(&(intent.source_schema.clone(), intent.source_key.clone()))
                .is_some_and(|(_, record)| record.pointer == intent.next)
        }) {
            bail!("Atlas local publication batch is partially applied")
        }

        let mut expected = Vec::new();
        let mut replacements = Vec::new();
        for (identity, intent) in &intents {
            let current = current_pointers.get(identity);
            if current.map(|(_, record)| record.pointer.publication_id.as_str())
                != intent.expected_current_publication_id.as_deref()
            {
                return Ok(AtlasStoreWriteOutcome::Conflict);
            }
            if let Some((_, record)) = current {
                if intent.next.publication_sequence <= record.pointer.publication_sequence {
                    bail!("Atlas latest publication pointer sequence did not advance")
                }
                expected.push(
                    cache.get_required_envelope::<AtlasLatestPublicationPointerRecord>(
                        &latest_pointer_key(&record.pointer)?,
                    )?,
                );
            }
            let next_record = AtlasLatestPublicationPointerRecord {
                schema_version: ATLAS_LATEST_POINTER_RECORD_SCHEMA.into(),
                pointer: intent.next.clone(),
            };
            next_record.validate()?;
            replacements.push(
                cache
                    .prepare_entry(latest_pointer_key(&intent.next)?, &next_record)?
                    .0,
            );
        }
        for (publication_id, publication) in &publications {
            if let Some(existing) = cache.get::<AtlasPublicationEnvelope>(publication_id)? {
                if &existing != *publication {
                    bail!("Atlas publication id collides with a different immutable event")
                }
                let envelope =
                    cache.get_required_envelope::<AtlasPublicationEnvelope>(publication_id)?;
                expected.push(envelope.clone());
                replacements.push(envelope);
            } else {
                replacements.push(cache.prepare_entry(publication_id.clone(), *publication)?.0);
            }
        }

        if self
            .backing()
            .compare_and_swap_batch(&expected, replacements)?
        {
            return Ok(AtlasStoreWriteOutcome::Applied);
        }
        let raced = self.load_cache()?;
        if publication_batch_is_applied(&raced, &intents, &publications)? {
            Ok(AtlasStoreWriteOutcome::AlreadyApplied)
        } else {
            Ok(AtlasStoreWriteOutcome::Conflict)
        }
    }
}

impl AtlasTrustBindingStore for AtlasCultCacheStore {
    fn load_trust_binding_records(&self) -> Result<Vec<AtlasPublisherTrustBindingRecord>> {
        let mut records = self
            .trust_records(&self.load_cache()?)?
            .into_iter()
            .map(|(_, record)| record)
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            left.binding
                .publisher
                .repository_uri
                .cmp(&right.binding.publisher.repository_uri)
        });
        Ok(records)
    }

    fn compare_and_swap_trust_binding(
        &self,
        expected_revision: Option<u64>,
        next_binding: AtlasPublisherTrustBinding,
    ) -> Result<AtlasStoreWriteOutcome> {
        validate_trust_binding_structure(&next_binding)?;
        let revision = expected_revision
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("Atlas trust binding revision exhausted"))?;
        let next = AtlasPublisherTrustBindingRecord {
            schema_version: ATLAS_TRUST_BINDING_RECORD_SCHEMA.into(),
            revision,
            binding: next_binding,
        };
        next.validate()?;
        let cache = self.load_cache()?;
        let key = trust_binding_key(&next.binding.publisher)?;
        let current_envelope = cache.get_envelope::<AtlasPublisherTrustBindingRecord>(&key)?;
        let current = cache.get::<AtlasPublisherTrustBindingRecord>(&key)?;
        if current.as_ref() == Some(&next) {
            return Ok(AtlasStoreWriteOutcome::AlreadyApplied);
        }
        if current.as_ref().map(|record| record.revision) != expected_revision {
            return Ok(AtlasStoreWriteOutcome::Conflict);
        }
        let replacement = cache.prepare_entry(key.clone(), &next)?.0;
        let applied = match current_envelope {
            Some(expected) => self
                .backing()
                .compare_and_swap_entry(&expected, replacement)?,
            None => self.backing().insert_entry_if_absent(replacement)?,
        };
        if applied {
            return Ok(AtlasStoreWriteOutcome::Applied);
        }
        match self
            .load_cache()?
            .get::<AtlasPublisherTrustBindingRecord>(&key)?
        {
            Some(record) if record == next => Ok(AtlasStoreWriteOutcome::AlreadyApplied),
            _ => Ok(AtlasStoreWriteOutcome::Conflict),
        }
    }
}

impl AtlasProjectionStore for AtlasCultCacheStore {
    fn load_projection_inputs(&self, now_unix_ms: u64) -> Result<AtlasProjectionStoreInputs> {
        let cache = self.load_cache()?;
        let trust = self.active_trust_directory(&cache, now_unix_ms)?;
        let mut publications = Vec::new();
        for (_, publication) in self.publication_rows(&cache)? {
            let Some(binding) = trust
                .get(&publication.statement.publisher.repository_uri)
                .filter(|binding| {
                    binding.signer_identity_id == publication.statement.publisher_identity_id
                })
            else {
                continue;
            };
            verify_atlas_publication(binding, &publication, now_unix_ms)?;
            publications.push(publication);
        }
        Ok(AtlasProjectionStoreInputs {
            publications,
            trust_bindings: trust.into_values().collect(),
        })
    }

    fn load_latest_projection(
        &self,
        audience_id: &str,
    ) -> Result<Option<AtlasEntanglementProjection>> {
        validate_identifier(audience_id, "Atlas latest projection audience id")?;
        let projection = self
            .load_cache()?
            .get::<AtlasEntanglementProjection>(audience_id)?;
        if let Some(projection) = &projection {
            validate_stored_projection(projection)?;
            if projection.audience_id != audience_id {
                bail!("Atlas latest projection is stored under the wrong audience")
            }
        }
        Ok(projection)
    }

    fn compare_and_swap_latest_projection(
        &self,
        expected_projection_sha256: Option<&str>,
        next: &AtlasEntanglementProjection,
    ) -> Result<AtlasStoreWriteOutcome> {
        validate_stored_projection(next)?;
        if let Some(expected) = expected_projection_sha256 {
            validate_sha256(expected, "Atlas expected latest projection digest")?;
        }
        let cache = self.load_cache()?;
        let key = next.audience_id.clone();
        let current_envelope = cache.get_envelope::<AtlasEntanglementProjection>(&key)?;
        let current = cache.get::<AtlasEntanglementProjection>(&key)?;
        if current.as_ref() == Some(next) {
            return Ok(AtlasStoreWriteOutcome::AlreadyApplied);
        }
        if current
            .as_ref()
            .map(|projection| projection.projection_sha256.as_str())
            != expected_projection_sha256
        {
            return Ok(AtlasStoreWriteOutcome::Conflict);
        }
        let replacement = cache.prepare_entry(key.clone(), next)?.0;
        let applied = match current_envelope {
            Some(expected) => self
                .backing()
                .compare_and_swap_entry(&expected, replacement)?,
            None => self.backing().insert_entry_if_absent(replacement)?,
        };
        if applied {
            return Ok(AtlasStoreWriteOutcome::Applied);
        }
        match self
            .load_cache()?
            .get::<AtlasEntanglementProjection>(&key)?
        {
            Some(projection) if projection == *next => Ok(AtlasStoreWriteOutcome::AlreadyApplied),
            _ => Ok(AtlasStoreWriteOutcome::Conflict),
        }
    }
}

impl AtlasImpactStore for AtlasCultCacheStore {
    fn load_impact_snapshot(&self) -> Result<AtlasImpactStoreSnapshot> {
        let cache = self.load_cache()?;
        let records = self.impact_records(&cache)?;
        let seen_dedupe_keys = records
            .iter()
            .map(|(_, record)| record.proposal.dedupe_key.clone())
            .collect::<BTreeSet<_>>();
        let state = self.current_impact_state(&cache)?;
        if state.is_none() && !records.is_empty() {
            bail!("Atlas impact dedupe history exists without its CAS state head")
        }
        if let Some((_, state)) = state {
            validate_pending_impact_references(&state, records.iter().map(|(_, record)| record))?;
            Ok(AtlasImpactStoreSnapshot {
                state_revision: Some(state.revision),
                seen_dedupe_keys,
                lane_states: state.ingress_lane_states(),
                brake: state.ingress_brake_state(),
            })
        } else {
            Ok(AtlasImpactStoreSnapshot {
                state_revision: None,
                seen_dedupe_keys,
                lane_states: vec![
                    AtlasImpactLaneState {
                        lane: AtlasImpactLane::Modeling,
                        pending_proposal_id: None,
                        last_completed_at_unix_ms: None,
                    },
                    AtlasImpactLaneState {
                        lane: AtlasImpactLane::Soul,
                        pending_proposal_id: None,
                        last_completed_at_unix_ms: None,
                    },
                ],
                brake: AtlasImpactBrakeState {
                    engaged: false,
                    brake_id: None,
                },
            })
        }
    }

    fn compare_and_swap_impact_update(
        &self,
        expected_state_revision: Option<u64>,
        next_state: &AtlasImpactStateRecord,
        new_dedupe_records: &[AtlasImpactDedupeRecord],
    ) -> Result<AtlasStoreWriteOutcome> {
        next_state.validate()?;
        if next_state.local_repository != self.local_repository
            || next_state.revision
                != expected_state_revision
                    .unwrap_or(0)
                    .checked_add(1)
                    .ok_or_else(|| anyhow::anyhow!("Atlas impact state revision exhausted"))?
        {
            bail!("Atlas impact state update has the wrong local owner or next revision")
        }
        let mut new_by_key = BTreeMap::new();
        for record in new_dedupe_records {
            record.validate()?;
            if record.local_repository != self.local_repository
                || record.recorded_at_unix_ms > next_state.updated_at_unix_ms
                || new_by_key
                    .insert(record.proposal.dedupe_key.clone(), record)
                    .is_some()
            {
                bail!("Atlas impact update contains a foreign or duplicate dedupe row")
            }
        }

        let cache = self.load_cache()?;
        let current_state = self.current_impact_state(&cache)?;
        let existing_records = self
            .impact_records(&cache)?
            .into_iter()
            .collect::<BTreeMap<_, _>>();
        if impact_update_is_applied(&current_state, &existing_records, next_state, &new_by_key) {
            return Ok(AtlasStoreWriteOutcome::AlreadyApplied);
        }
        if current_state.as_ref().map(|(_, state)| state.revision) != expected_state_revision {
            return Ok(AtlasStoreWriteOutcome::Conflict);
        }
        if current_state
            .as_ref()
            .is_some_and(|(_, state)| state.updated_at_unix_ms >= next_state.updated_at_unix_ms)
        {
            bail!("Atlas impact state update time did not advance")
        }
        if new_by_key
            .keys()
            .any(|key| existing_records.contains_key(key))
        {
            bail!("Atlas impact update would rewrite immutable dedupe history")
        }

        let prospective = existing_records
            .values()
            .chain(new_by_key.values().map(|record| *record));
        validate_pending_impact_references(next_state, prospective)?;

        let mut expected = Vec::new();
        if let Some((envelope, _)) = &current_state {
            expected.push(envelope.clone());
        }
        let mut replacements = vec![
            cache
                .prepare_entry(impact_state_key(&self.local_repository)?, next_state)?
                .0,
        ];
        for (key, record) in &new_by_key {
            replacements.push(cache.prepare_entry(key.clone(), *record)?.0);
        }
        if self
            .backing()
            .compare_and_swap_batch(&expected, replacements)?
        {
            return Ok(AtlasStoreWriteOutcome::Applied);
        }

        let raced = self.load_cache()?;
        let raced_state = self.current_impact_state(&raced)?;
        let raced_records = self
            .impact_records(&raced)?
            .into_iter()
            .collect::<BTreeMap<_, _>>();
        if impact_update_is_applied(&raced_state, &raced_records, next_state, &new_by_key) {
            Ok(AtlasStoreWriteOutcome::AlreadyApplied)
        } else {
            Ok(AtlasStoreWriteOutcome::Conflict)
        }
    }
}

fn latest_pointer_key(pointer: &AtlasLatestPublicationPointer) -> Result<String> {
    Ok(sha256(&rmp_serde::to_vec_named(&(
        "epiphany.atlas.latest-publication-pointer.key.v0",
        &pointer.publisher.repository_uri,
        &pointer.source_schema,
        &pointer.source_key,
    ))?))
}

fn trust_binding_key(repository: &AtlasRepositoryIdentity) -> Result<String> {
    Ok(sha256(&rmp_serde::to_vec_named(&(
        "epiphany.atlas.publisher-trust-binding.key.v0",
        &repository.repository_uri,
    ))?))
}

fn impact_state_key(repository: &AtlasRepositoryIdentity) -> Result<String> {
    Ok(sha256(&rmp_serde::to_vec_named(&(
        "epiphany.atlas.impact-state.key.v0",
        &repository.repository_uri,
    ))?))
}

fn validate_trust_binding_structure(binding: &AtlasPublisherTrustBinding) -> Result<()> {
    require_schema(&binding.schema_version, ATLAS_TRUST_BINDING_SCHEMA)?;
    binding.publisher.validate()?;
    validate_identifier(
        &binding.signer_identity_id,
        "Atlas stored trusted signer identity id",
    )?;
    if binding.trust_anchor.schema_version != AtlasRepositorySigningIdentity::TRUST_ANCHOR_SCHEMA
        || binding.trust_anchor.identity_id != binding.signer_identity_id
        || derive_service_identity_id::<AtlasRepositorySigningIdentity>(
            &binding.trust_anchor.public_key,
        )? != binding.signer_identity_id
        || binding
            .expires_at_unix_ms
            .is_some_and(|expires| expires <= binding.trusted_from_unix_ms)
    {
        bail!("Atlas stored publisher trust binding is structurally invalid")
    }
    Ok(())
}

fn pointer_matches_publication(
    pointer: &AtlasLatestPublicationPointer,
    publication: &AtlasPublicationEnvelope,
) -> bool {
    pointer.publisher == publication.statement.publisher
        && pointer.source_schema == publication.statement.payload.schema()
        && pointer.source_key == publication.statement.payload.key()
        && pointer.source_payload_sha256 == publication.statement.canonical_payload_msgpack_sha256
        && pointer.publication_id == publication.statement.publication_id
        && pointer.publication_sequence == publication.statement.publication_sequence
}

fn publication_batch_is_applied(
    cache: &CultCache,
    intents: &BTreeMap<(String, String), &AtlasLatestPublicationPointerIntent>,
    publications: &BTreeMap<String, &AtlasPublicationEnvelope>,
) -> Result<bool> {
    for intent in intents.values() {
        let key = latest_pointer_key(&intent.next)?;
        let Some(record) = cache.get::<AtlasLatestPublicationPointerRecord>(&key)? else {
            return Ok(false);
        };
        if record.pointer != intent.next {
            return Ok(false);
        }
    }
    for (publication_id, publication) in publications {
        if cache
            .get::<AtlasPublicationEnvelope>(publication_id)?
            .as_ref()
            != Some(*publication)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn validate_stored_projection(projection: &AtlasEntanglementProjection) -> Result<()> {
    require_schema(&projection.schema_version, ATLAS_PROJECTION_SCHEMA)?;
    validate_identifier(
        &projection.audience_id,
        "Atlas stored projection audience id",
    )?;
    validate_sha256(
        &projection.projection_sha256,
        "Atlas stored projection digest",
    )?;
    validate_sorted_unique_sha256(
        &projection.source_publication_ids,
        "Atlas stored projection source publication ids",
        false,
    )?;
    if atlas_projection_digest(projection)? != projection.projection_sha256 {
        bail!("Atlas stored latest projection digest does not match its deterministic body")
    }
    Ok(())
}

fn validate_pending_impact_references<'a>(
    state: &AtlasImpactStateRecord,
    records: impl IntoIterator<Item = &'a AtlasImpactDedupeRecord>,
) -> Result<()> {
    let mut proposals = BTreeMap::new();
    for record in records {
        let scheduled = matches!(
            &record.scheduling_decision.disposition,
            AtlasImpactScheduleDisposition::Schedule { .. }
        );
        if proposals
            .insert(
                record.proposal.proposal_id,
                (record.proposal.lane, scheduled),
            )
            .is_some()
        {
            bail!("Atlas impact history contains a duplicate proposal id")
        }
    }
    for (lane, pending) in [
        (
            AtlasImpactLane::Modeling,
            state.modeling.pending_proposal_id,
        ),
        (AtlasImpactLane::Soul, state.soul.pending_proposal_id),
    ] {
        if let Some(proposal_id) = pending {
            match proposals.get(&proposal_id) {
                Some((proposal_lane, true)) if *proposal_lane == lane => {}
                _ => bail!("Atlas impact state references a missing or wrong-lane proposal"),
            }
        }
    }
    Ok(())
}

fn impact_update_is_applied(
    current_state: &Option<(CultCacheEnvelope, AtlasImpactStateRecord)>,
    existing_records: &BTreeMap<String, AtlasImpactDedupeRecord>,
    next_state: &AtlasImpactStateRecord,
    new_records: &BTreeMap<String, &AtlasImpactDedupeRecord>,
) -> bool {
    current_state
        .as_ref()
        .is_some_and(|(_, state)| state == next_state)
        && new_records
            .iter()
            .all(|(key, record)| existing_records.get(key) == Some(*record))
}
