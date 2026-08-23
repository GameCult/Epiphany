use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use cultcache_rs::{
    CacheBackingStore, CultCache, CultCacheEnvelope, DatabaseEntry,
    SingleFileMessagePackBackingStore,
};
use cultnet_rs::{
    ServiceIdentityProfile, ServiceIdentitySigner, ServiceIdentityTrustAnchor,
    enroll_service_identity_at, export_service_identity_trust_anchor, open_service_identity_at,
};

use super::*;

pub const ATLAS_GAMECULT_LOCAL_AUDIENCE: &str = "gamecult-local";
pub const ATLAS_PUBLISH_SCOPE: &str = "atlas.publish";
pub const ATLAS_PROJECT_SCOPE: &str = "atlas.project";
pub const ATLAS_IMPACT_INGRESS_SCOPE: &str = "atlas.impact_ingress";
pub const ATLAS_PUBLISHER_HEARTBEAT_INTERVAL_MS: u64 = 30_000;
pub const ATLAS_PUBLISHER_FRESHNESS_LIMIT_MS: u64 = 90_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AtlasPublisherRunOutcome {
    HeldByBrake {
        brake_id: String,
    },
    Published {
        new_publications: usize,
        transported_publications: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasPublisherRunConfig {
    pub repository: AtlasRepositoryIdentity,
    pub runtime_id: String,
    pub runtime_incarnation_id: String,
    pub runtime_mind_store: PathBuf,
    pub local_verse_store: PathBuf,
    pub publisher_store: PathBuf,
    pub publisher_cultmesh_store: PathBuf,
    pub identity_store: PathBuf,
    pub trust_anchor_store: PathBuf,
    pub odin_endpoint: SocketAddr,
    pub now_unix_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AtlasProjectorRunOutcome {
    HeldByBrake {
        brake_id: String,
        last_projection: Option<AtlasEntanglementProjection>,
    },
    Projected {
        projection: AtlasEntanglementProjection,
        accepted_publications: usize,
        rejected_publications: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasProjectorRunConfig {
    pub projector_repository: AtlasRepositoryIdentity,
    pub runtime_id: String,
    pub local_verse_store: PathBuf,
    pub projector_store: PathBuf,
    pub projector_cultmesh_store: PathBuf,
    pub odin_endpoint: SocketAddr,
    pub now_unix_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AtlasImpactIngressRunOutcome {
    HeldByBrake {
        brake_id: String,
        last_projection: Option<AtlasEntanglementProjection>,
    },
    Evaluated {
        admitted_impacts: usize,
        scheduled_pressures: usize,
        held_impacts: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasImpactIngressRunConfig {
    pub local_repository: AtlasRepositoryIdentity,
    pub projector_repository: AtlasRepositoryIdentity,
    pub runtime_id: String,
    pub runtime_mind_store: PathBuf,
    pub local_verse_store: PathBuf,
    pub projector_store: PathBuf,
    pub impact_store: PathBuf,
    pub resident_self_store: PathBuf,
    pub cooldown_after_completion_ms: u64,
    pub now_unix_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasDependencyVerificationCommitEvidence {
    pub now_unix_ms: u64,
    pub claim_publication: AtlasStrongRead<AtlasPublicationEnvelope>,
    pub claim_publication_trust: AtlasPublisherTrustBinding,
    pub offer_publication: AtlasStrongRead<AtlasPublicationEnvelope>,
    pub offer_publication_trust: AtlasPublisherTrustBinding,
    pub evidence: AtlasEvidenceArtifactRead,
}

#[derive(Clone)]
struct LocalClaimRead {
    ingress: AtlasLocalClaimInput,
    strong: AtlasStrongRead<AtlasDependencyClaim>,
    envelope: CultCacheEnvelope,
}

#[derive(Clone, Debug)]
pub struct RuntimeAtlasMindSnapshotSource {
    runtime_store: PathBuf,
    repository: AtlasRepositoryIdentity,
    body_basis: crate::RepositoryBodyObservationBasis,
}

impl RuntimeAtlasMindSnapshotSource {
    pub fn new(
        runtime_store: impl Into<PathBuf>,
        repository: AtlasRepositoryIdentity,
        body_basis: crate::RepositoryBodyObservationBasis,
    ) -> Result<Self> {
        repository.validate()?;
        Ok(Self {
            runtime_store: runtime_store.into(),
            repository,
            body_basis,
        })
    }
}

impl AtlasLocalMindSnapshotSource for RuntimeAtlasMindSnapshotSource {
    fn load_local_atlas_mind_snapshot(&self) -> Result<AtlasLocalMindSnapshot> {
        let mut cache = crate::runtime_spine_cache(&self.runtime_store)?;
        cache.pull_all_backing_stores()?;
        if crate::load_current_runtime_repository_body_basis(&self.runtime_store)?
            != self.body_basis
        {
            bail!("Atlas publisher lost its exact current Repository Body basis")
        }
        let manifest =
            crate::authenticated_repository_body_manifest(&self.runtime_store, &self.body_basis)?;
        let snapshot = cache.snapshot_envelopes();
        let receipts = snapshot
            .iter()
            .filter(|entry| entry.r#type == crate::EpiphanyMindCommitReceipt::TYPE)
            .map(|entry| {
                Ok(rmp_serde::from_slice::<crate::EpiphanyMindCommitReceipt>(
                    &entry.payload,
                )?)
            })
            .collect::<Result<Vec<_>>>()?;
        let mut documents = Vec::new();
        for envelope in snapshot {
            let payload = match envelope.r#type.as_str() {
                ATLAS_SURFACE_OFFER_SCHEMA => AtlasPublicationPayload::SurfaceOffer(
                    rmp_serde::from_slice(&envelope.payload)
                        .context("local Atlas surface offer is malformed")?,
                ),
                ATLAS_DEPENDENCY_CLAIM_SCHEMA => AtlasPublicationPayload::DependencyClaim(
                    rmp_serde::from_slice(&envelope.payload)
                        .context("local Atlas dependency claim is malformed")?,
                ),
                ATLAS_DEPENDENCY_VERIFICATION_SCHEMA => {
                    AtlasPublicationPayload::DependencyVerification(
                        rmp_serde::from_slice(&envelope.payload)
                            .context("local Atlas dependency verification is malformed")?,
                    )
                }
                _ => continue,
            };
            if payload.owner() != &self.repository
                || payload.schema() != envelope.r#type
                || payload.key() != envelope.key
            {
                bail!("local Atlas Mind document has foreign ownership or a substituted key")
            }
            validate_atlas_body_evidence_for_manifest(&payload, &manifest)?;
            let source = atlas_source_for_payload("epiphany-mind", &payload)?;
            let receipt = latest_exact_mind_commit(&receipts, &envelope)?;
            let mind_commit = AtlasMindCommitBinding {
                receipt_id: receipt.receipt_id.clone(),
                receipt_sha256: super::identity::sha256(&rmp_serde::to_vec_named(receipt)?),
                invariant_owner: receipt.invariant_owner.clone(),
                source: source.clone(),
            };
            mind_commit.validate()?;
            documents.push(AtlasLocalMindDocument {
                payload,
                source,
                mind_commit,
            });
        }
        documents.sort_by(|left, right| {
            (left.payload.schema(), left.payload.key())
                .cmp(&(right.payload.schema(), right.payload.key()))
        });
        Ok(AtlasLocalMindSnapshot {
            publisher: self.repository.clone(),
            current_mind_documents: documents,
        })
    }
}

fn validate_atlas_body_evidence_for_manifest(
    payload: &AtlasPublicationPayload,
    manifest: &crate::RepositoryBodyManifest,
) -> Result<()> {
    let evidence = match payload {
        AtlasPublicationPayload::SurfaceOffer(offer) => &offer.body_evidence,
        AtlasPublicationPayload::DependencyClaim(claim) => &claim.body_evidence,
        AtlasPublicationPayload::DependencyVerification(_) => return Ok(()),
        AtlasPublicationPayload::PublisherStatus(_) => return Ok(()),
    };
    let entries = manifest
        .entries
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect::<std::collections::BTreeMap<_, _>>();
    for source in evidence {
        source.validate()?;
        let entry = entries.get(source.path.as_str()).ok_or_else(|| {
            anyhow::anyhow!(
                "Atlas Body evidence path {:?} is absent from the current manifest",
                source.path
            )
        })?;
        if entry.kind != "regular" || entry.raw_sha256 != source.raw_sha256 {
            bail!(
                "Atlas Body evidence path {:?} no longer has its admitted content",
                source.path
            )
        }
    }
    Ok(())
}

pub fn run_atlas_publisher_once(
    config: &AtlasPublisherRunConfig,
) -> Result<AtlasPublisherRunOutcome> {
    config.repository.validate()?;
    require_separate_paths(&[
        &config.runtime_mind_store,
        &config.publisher_store,
        &config.publisher_cultmesh_store,
        &config.identity_store,
        &config.trust_anchor_store,
    ])?;
    if let Some(brake_id) = atlas_brake_id(
        &config.local_verse_store,
        &config.runtime_id,
        ATLAS_PUBLISH_SCOPE,
    )? {
        return Ok(AtlasPublisherRunOutcome::HeldByBrake { brake_id });
    }
    let body_basis = crate::load_current_runtime_repository_body_basis(&config.runtime_mind_store)?;
    if body_basis.swarm_id != config.repository.swarm_id
        || body_basis.workspace_id != config.repository.workspace_id
        || body_basis.runtime_id != config.runtime_id
    {
        bail!("Atlas publisher repository/runtime differs from its current Body basis")
    }
    let signer = open_or_enroll_atlas_repository_identity(&config.identity_store)?;
    export_service_identity_trust_anchor(&signer, &config.trust_anchor_store)?;
    let store = AtlasCultCacheStore::new(&config.publisher_store, config.repository.clone())?;
    ensure_local_publisher_trust(&store, &signer, config.now_unix_ms)?;
    let heartbeat_sequence = next_heartbeat_sequence(&store)?;
    let mind = RuntimeAtlasMindSnapshotSource::new(
        &config.runtime_mind_store,
        config.repository.clone(),
        body_basis.clone(),
    )?;
    let adapter = AtlasPublisherSnapshotAdapter::new(&mind, &store);
    let batch = publish_local_atlas_state(
        &adapter,
        &signer,
        &AtlasPublisherContext {
            runtime_id: config.runtime_id.clone(),
            runtime_incarnation_id: config.runtime_incarnation_id.clone(),
            body_basis,
            verse_id: format!(
                "cultmesh://gamecult-local/swarm/{}/workspace/{}/atlas",
                config.repository.swarm_id, config.repository.workspace_id
            ),
            heartbeat_sequence,
            heartbeat_at_unix_ms: config.now_unix_ms,
            publisher_state: AtlasPublisherState::Serving,
        },
    )?;
    let new_publications = batch.publications.len();
    match store.commit_local_publication_batch(&batch, config.now_unix_ms)? {
        AtlasStoreWriteOutcome::Conflict => {
            bail!("Atlas publisher latest-pointer CAS conflicted; reload before retry")
        }
        AtlasStoreWriteOutcome::Applied | AtlasStoreWriteOutcome::AlreadyApplied => {}
    }
    // Re-publish the immutable local history on every heartbeat. Odin accepts
    // exact duplicates, while a missed first document publication cannot be
    // repaired by a later status-only heartbeat.
    let publications = store.load_publication_events()?;
    persist_atlas_publications(
        &config.publisher_cultmesh_store,
        &config.runtime_id,
        &publications,
    )?;
    publish_atlas_publications_rudp(
        &config.publisher_cultmesh_store,
        config.odin_endpoint,
        &config.runtime_id,
        &format!("{}-atlas-publisher", config.runtime_id),
        &publications,
    )?;
    Ok(AtlasPublisherRunOutcome::Published {
        new_publications,
        transported_publications: publications.len(),
    })
}

fn open_or_enroll_atlas_repository_identity(
    identity_store: &Path,
) -> Result<ServiceIdentitySigner<AtlasRepositorySigningIdentity>> {
    if identity_store.exists() {
        open_service_identity_at::<AtlasRepositorySigningIdentity>(identity_store)
    } else {
        enroll_service_identity_at::<AtlasRepositorySigningIdentity>(identity_store)
    }
}

pub fn run_atlas_projector_once(
    config: &AtlasProjectorRunConfig,
) -> Result<AtlasProjectorRunOutcome> {
    config.projector_repository.validate()?;
    let store =
        AtlasCultCacheStore::new(&config.projector_store, config.projector_repository.clone())?;
    if let Some(brake_id) = atlas_brake_id(
        &config.local_verse_store,
        &config.runtime_id,
        ATLAS_PROJECT_SCOPE,
    )? {
        return Ok(AtlasProjectorRunOutcome::HeldByBrake {
            brake_id,
            last_projection: store.load_latest_projection(ATLAS_GAMECULT_LOCAL_AUDIENCE)?,
        });
    }
    let incoming = query_atlas_publications_rudp(config.odin_endpoint, &config.runtime_id)?;
    let mut accepted = 0;
    let mut rejected = 0;
    for publication in incoming {
        match store.append_verified_publication_event(&publication, config.now_unix_ms) {
            Ok(AtlasStoreWriteOutcome::Applied | AtlasStoreWriteOutcome::AlreadyApplied) => {
                accepted += 1;
            }
            Ok(AtlasStoreWriteOutcome::Conflict) | Err(_) => rejected += 1,
        }
    }
    let inputs = store.load_projection_inputs(config.now_unix_ms)?;
    let projection = project_atlas(
        &inputs.publications,
        &inputs.trust_bindings,
        &AtlasProjectionAudience {
            audience_id: ATLAS_GAMECULT_LOCAL_AUDIENCE.into(),
            gamecult_local: true,
        },
        &AtlasFreshnessPolicy {
            publisher_status_maximum_age_ms: ATLAS_PUBLISHER_FRESHNESS_LIMIT_MS,
            maximum_future_skew_ms: 5_000,
        },
        config.now_unix_ms,
    )?;
    let previous = store.load_latest_projection(ATLAS_GAMECULT_LOCAL_AUDIENCE)?;
    match store.compare_and_swap_latest_projection(
        previous
            .as_ref()
            .map(|value| value.projection_sha256.as_str()),
        &projection,
    )? {
        AtlasStoreWriteOutcome::Conflict => {
            bail!("Atlas projector CAS conflicted; reload before publishing")
        }
        AtlasStoreWriteOutcome::Applied | AtlasStoreWriteOutcome::AlreadyApplied => {}
    }
    let updated_at = utc_from_unix_ms(config.now_unix_ms)?;
    let eve = project_model_atlas_eve_documents(
        &projection,
        &AtlasEvePresentationState::default(),
        config.now_unix_ms,
        &updated_at,
    )?;
    persist_atlas_projection_and_eve(
        &config.projector_cultmesh_store,
        &config.runtime_id,
        &projection,
        &eve,
    )?;
    publish_atlas_projection_and_eve_rudp(
        &config.projector_cultmesh_store,
        config.odin_endpoint,
        &config.runtime_id,
        &format!("{}-model-entanglement-projector", config.runtime_id),
        &projection,
        &eve,
    )?;
    Ok(AtlasProjectorRunOutcome::Projected {
        projection,
        accepted_publications: accepted,
        rejected_publications: rejected,
    })
}

pub fn run_atlas_impact_ingress_once(
    config: &AtlasImpactIngressRunConfig,
) -> Result<AtlasImpactIngressRunOutcome> {
    config.local_repository.validate()?;
    config.projector_repository.validate()?;
    require_separate_paths(&[
        &config.runtime_mind_store,
        &config.projector_store,
        &config.impact_store,
        &config.resident_self_store,
    ])?;
    let projector_store =
        AtlasCultCacheStore::new(&config.projector_store, config.projector_repository.clone())?;
    let projection = projector_store.load_latest_projection(ATLAS_GAMECULT_LOCAL_AUDIENCE)?;
    if let Some(brake_id) = atlas_brake_id(
        &config.local_verse_store,
        &config.runtime_id,
        ATLAS_IMPACT_INGRESS_SCOPE,
    )? {
        return Ok(AtlasImpactIngressRunOutcome::HeldByBrake {
            brake_id,
            last_projection: projection,
        });
    }
    let projection = projection.ok_or_else(|| {
        anyhow::anyhow!("Atlas impact ingress has no accepted gamecult-local projection")
    })?;
    let projection_source = AtlasMindSourceVersion {
        store_id: "epiphany-atlas-projector".into(),
        document_type: ATLAS_PROJECTION_SCHEMA.into(),
        document_key: projection.audience_id.clone(),
        schema_id: Some(ATLAS_PROJECTION_SCHEMA.into()),
        payload_sha256: super::identity::sha256(&rmp_serde::to_vec(&projection)?),
    };
    projection_source.validate()?;
    let body_basis = crate::load_current_runtime_repository_body_basis(&config.runtime_mind_store)?;
    if body_basis.runtime_id != config.runtime_id
        || body_basis.swarm_id != config.local_repository.swarm_id
        || body_basis.workspace_id != config.local_repository.workspace_id
    {
        bail!("Atlas impact ingress Body basis does not belong to its local runtime")
    }
    let claims = load_local_claim_reads(&config.runtime_mind_store, &config.local_repository)?;
    let impact_store =
        AtlasCultCacheStore::new(&config.impact_store, config.local_repository.clone())?;
    let snapshot = impact_store.load_impact_snapshot()?;
    let reconciled_lanes =
        reconcile_completed_atlas_lanes(&config.resident_self_store, &snapshot.lane_states)?;
    let ingress = evaluate_local_atlas_impacts(
        &config.local_repository,
        &projection,
        &claims
            .iter()
            .map(|value| value.ingress.clone())
            .collect::<Vec<_>>(),
        &snapshot.seen_dedupe_keys,
        &reconciled_lanes,
        &AtlasImpactBrakeState {
            engaged: false,
            brake_id: None,
        },
        &AtlasImpactIngressPolicy {
            cooldown_after_completion_ms: config.cooldown_after_completion_ms,
        },
        config.now_unix_ms,
    )?;

    let decisions = ingress
        .scheduling_decisions
        .iter()
        .map(|decision| (decision.proposal_id, decision))
        .collect::<std::collections::BTreeMap<_, _>>();
    let claims_by_id = claims
        .iter()
        .map(|claim| (claim.strong.document.claim_id, claim))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut admitted_impacts = 0;
    let mut scheduled_pressures = 0;
    let mut held_impacts = 0;
    let mut dedupe_records = Vec::new();
    let mut next_lanes = reconciled_lanes.clone();
    for proposal in ingress.proposals {
        let decision = decisions
            .get(&proposal.proposal_id)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Atlas impact proposal lost its scheduling decision"))?;
        let lane = match &decision.disposition {
            AtlasImpactScheduleDisposition::Schedule { lane } => Some(*lane),
            AtlasImpactScheduleDisposition::VisibleOnly => None,
            _ => {
                held_impacts += 1;
                continue;
            }
        };
        let claim = claims_by_id
            .get(&proposal.impact.claim_id)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Atlas impact proposal lost its exact local claim"))?;
        let current = load_local_impact_read(
            &config.runtime_mind_store,
            proposal.impact.impact_id,
            &config.local_repository,
        )?;
        if current
            .as_ref()
            .is_some_and(|(_, read)| read.document != proposal.impact)
        {
            bail!("Atlas impact id collides with a different local impact document")
        }
        if current.is_none() {
            let intent = admit_dependency_impact(AtlasDependencyImpactAdmissionInput {
                context: AtlasPlannerContext {
                    local_repository: config.local_repository.clone(),
                    current_body_basis: body_basis.clone(),
                },
                expected_claim_source: claim.strong.source.clone(),
                claim: claim.strong.clone(),
                expected_projection_source: projection_source.clone(),
                projection: AtlasStrongRead {
                    source: projection_source.clone(),
                    document: projection.clone(),
                },
                expected_current_impact_source: None,
                current_impact: None,
                proposal: proposal.clone(),
            })?;
            commit_dependency_impact_intent(
                config,
                &projector_store,
                &projection,
                &claim.envelope,
                None,
                &intent,
            )?;
            admitted_impacts += 1;
        }
        // The projection is a foreign read-only evidence surface. Revalidate
        // it after the local CAS and before the only scheduling consequence.
        if projector_store.load_latest_projection(ATLAS_GAMECULT_LOCAL_AUDIENCE)?
            != Some(projection.clone())
        {
            held_impacts += 1;
            continue;
        }
        if let Some(lane) = lane {
            if crate::enqueue_resident_self_atlas_impact_pressure(
                &config.resident_self_store,
                decision,
                config.now_unix_ms,
            )? {
                scheduled_pressures += 1;
            }
            let state = next_lanes
                .iter_mut()
                .find(|state| state.lane == lane)
                .expect("both Atlas lanes were validated by ingress");
            state.pending_proposal_id = Some(proposal.proposal_id);
        }
        dedupe_records.push(AtlasImpactDedupeRecord {
            schema_version: ATLAS_IMPACT_DEDUPE_RECORD_SCHEMA.into(),
            local_repository: config.local_repository.clone(),
            proposal,
            scheduling_decision: decision.clone(),
            recorded_at_unix_ms: config.now_unix_ms,
        });
    }
    let state_changed = next_lanes != snapshot.lane_states || !dedupe_records.is_empty();
    if state_changed {
        let next_state = impact_state_record(
            &config.local_repository,
            snapshot.state_revision,
            &next_lanes,
            config.now_unix_ms,
        )?;
        if impact_store.compare_and_swap_impact_update(
            snapshot.state_revision,
            &next_state,
            &dedupe_records,
        )? == AtlasStoreWriteOutcome::Conflict
        {
            bail!("Atlas impact state CAS conflicted; reload before scheduling again")
        }
    }
    Ok(AtlasImpactIngressRunOutcome::Evaluated {
        admitted_impacts,
        scheduled_pressures,
        held_impacts,
    })
}

pub fn pin_atlas_publisher_trust_anchor(
    projector_store: &Path,
    projector_repository: AtlasRepositoryIdentity,
    publisher: AtlasRepositoryIdentity,
    trust_anchor_store: &Path,
    expected_revision: Option<u64>,
    now_unix_ms: u64,
) -> Result<AtlasStoreWriteOutcome> {
    let anchor = read_atlas_repository_trust_anchor(trust_anchor_store)?;
    let binding = AtlasPublisherTrustBinding {
        schema_version: ATLAS_TRUST_BINDING_SCHEMA.into(),
        publisher,
        signer_identity_id: anchor.identity_id.clone(),
        trusted_from_unix_ms: now_unix_ms,
        expires_at_unix_ms: None,
        revoked: false,
        trust_anchor: anchor,
    };
    AtlasCultCacheStore::new(projector_store, projector_repository)?
        .compare_and_swap_trust_binding(expected_revision, binding)
}

pub fn commit_atlas_dependency_verification_intent(
    runtime_mind_store: &Path,
    intent: &AtlasDependencyVerificationWriteIntent,
    evidence: &AtlasDependencyVerificationCommitEvidence,
    committed_at: &str,
) -> Result<crate::EpiphanyMindCommitOutcome> {
    intent.validate()?;
    require_current_atlas_body_basis(runtime_mind_store, &intent.body_basis)?;
    let mut cache = crate::runtime_spine_cache(runtime_mind_store)?;
    cache.pull_all_backing_stores()?;

    let claim_envelope = exact_local_source(&cache, &intent.claim_source)?;
    let claim = AtlasStrongRead {
        source: intent.claim_source.clone(),
        document: rmp_serde::from_slice::<AtlasDependencyClaim>(&claim_envelope.payload)?,
    };
    let current_verification_envelope = exact_or_absent_local_source(
        &cache,
        intent.expected_current_source.as_ref(),
        ATLAS_DEPENDENCY_VERIFICATION_SCHEMA,
        &intent.next.claim_id.to_string(),
    )?;
    let current_verification = match current_verification_envelope.as_ref() {
        Some(envelope) => Some(AtlasStrongRead {
            source: intent
                .expected_current_source
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Atlas verification source vanished"))?,
            document: rmp_serde::from_slice::<AtlasDependencyVerification>(&envelope.payload)?,
        }),
        None => None,
    };
    let derived = admit_dependency_verification(AtlasDependencyVerificationAdmissionInput {
        context: AtlasPlannerContext {
            local_repository: intent.local_repository.clone(),
            current_body_basis: intent.body_basis.clone(),
        },
        now_unix_ms: evidence.now_unix_ms,
        expected_claim_source: intent.claim_source.clone(),
        claim,
        claim_publication: evidence.claim_publication.clone(),
        claim_publication_trust: evidence.claim_publication_trust.clone(),
        offer_publication: evidence.offer_publication.clone(),
        offer_publication_trust: evidence.offer_publication_trust.clone(),
        expected_current_verification_source: intent.expected_current_source.clone(),
        current_verification,
        evidence: evidence.evidence.clone(),
        verdict: intent.next.verdict,
    })?;
    if derived != *intent {
        bail!("Atlas Soul admission evidence does not rederive the exact write intent")
    }

    let mut strong_reads = vec![claim_envelope];
    if let Some(current) = current_verification_envelope {
        strong_reads.push(current);
    }
    let provenance = prepared_envelope(intent.intent_id.clone(), intent)?;
    let write = prepared_envelope(intent.next.claim_id.to_string(), &intent.next)?;
    crate::commit_typed_organ_mind_mutation(
        runtime_mind_store,
        "Soul",
        provenance,
        "Soul.atlas.dependency_verification",
        strong_reads,
        vec![write],
        committed_at,
    )
}

pub fn read_atlas_repository_trust_anchor(path: &Path) -> Result<ServiceIdentityTrustAnchor> {
    let entries = SingleFileMessagePackBackingStore::new(path).pull_all()?;
    let [entry] = entries.as_slice() else {
        bail!("Atlas trust-anchor store must contain exactly one public anchor")
    };
    if entry.r#type != AtlasRepositorySigningIdentity::TRUST_ANCHOR_TYPE
        || entry.key != AtlasRepositorySigningIdentity::TRUST_ANCHOR_KEY
        || entry.schema_id.as_deref() != Some(AtlasRepositorySigningIdentity::TRUST_ANCHOR_SCHEMA)
    {
        bail!("Atlas trust-anchor store contains the wrong typed public identity")
    }
    Ok(rmp_serde::from_slice(&entry.payload)?)
}

fn ensure_local_publisher_trust(
    store: &AtlasCultCacheStore,
    signer: &cultnet_rs::ServiceIdentitySigner<AtlasRepositorySigningIdentity>,
    now_unix_ms: u64,
) -> Result<()> {
    let binding = AtlasPublisherTrustBinding {
        schema_version: ATLAS_TRUST_BINDING_SCHEMA.into(),
        publisher: store.local_repository().clone(),
        signer_identity_id: signer.entry().identity_id.clone(),
        trusted_from_unix_ms: now_unix_ms,
        expires_at_unix_ms: None,
        revoked: false,
        trust_anchor: signer.trust_anchor()?,
    };
    let existing = store
        .load_trust_binding_records()?
        .into_iter()
        .find(|record| record.binding.publisher == *store.local_repository());
    match existing {
        None => match store.compare_and_swap_trust_binding(None, binding)? {
            AtlasStoreWriteOutcome::Applied | AtlasStoreWriteOutcome::AlreadyApplied => Ok(()),
            AtlasStoreWriteOutcome::Conflict => {
                bail!("Atlas publisher trust enrollment raced; reload before retry")
            }
        },
        Some(record) if record.binding == binding => Ok(()),
        Some(_) => bail!("Atlas publisher identity changed; operator-gated key rotation required"),
    }
}

fn require_current_atlas_body_basis(
    runtime_mind_store: &Path,
    expected: &crate::RepositoryBodyObservationBasis,
) -> Result<()> {
    if crate::load_current_runtime_repository_body_basis(runtime_mind_store)? != *expected {
        bail!("Atlas local Mind admission refused a stale repository Body basis")
    }
    Ok(())
}

fn exact_or_absent_local_source(
    cache: &CultCache,
    expected: Option<&AtlasMindSourceVersion>,
    document_type: &str,
    document_key: &str,
) -> Result<Option<CultCacheEnvelope>> {
    let current = cache
        .snapshot_envelopes()
        .into_iter()
        .find(|entry| entry.r#type == document_type && entry.key == document_key);
    match (expected, current) {
        (None, None) => Ok(None),
        (Some(expected), Some(current)) => {
            validate_local_source_envelope(expected, &current)?;
            Ok(Some(current))
        }
        (None, Some(_)) => bail!("Atlas local Mind admission expected an absent document slot"),
        (Some(_), None) => bail!("Atlas local Mind admission lost its exact strong-read source"),
    }
}

fn exact_local_source(
    cache: &CultCache,
    expected: &AtlasMindSourceVersion,
) -> Result<CultCacheEnvelope> {
    let current = cache
        .snapshot_envelopes()
        .into_iter()
        .find(|entry| entry.r#type == expected.document_type && entry.key == expected.document_key)
        .ok_or_else(|| anyhow::anyhow!("Atlas local Mind admission lost an exact source"))?;
    validate_local_source_envelope(expected, &current)?;
    Ok(current)
}

fn validate_local_source_envelope(
    expected: &AtlasMindSourceVersion,
    current: &CultCacheEnvelope,
) -> Result<()> {
    expected.validate()?;
    if expected.store_id != "epiphany-mind"
        || current.r#type != expected.document_type
        || current.key != expected.document_key
        || current.schema_id != expected.schema_id
        || super::identity::sha256(&current.payload) != expected.payload_sha256
    {
        bail!("Atlas local Mind admission refused a stale or substituted source version")
    }
    Ok(())
}

fn next_heartbeat_sequence(store: &AtlasCultCacheStore) -> Result<u64> {
    store
        .load_publication_events()?
        .iter()
        .filter_map(|publication| match &publication.statement.payload {
            AtlasPublicationPayload::PublisherStatus(status) => Some(status.heartbeat_sequence),
            _ => None,
        })
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| anyhow::anyhow!("Atlas heartbeat sequence exhausted"))
}

fn latest_exact_mind_commit<'a>(
    receipts: &'a [crate::EpiphanyMindCommitReceipt],
    document: &CultCacheEnvelope,
) -> Result<&'a crate::EpiphanyMindCommitReceipt> {
    receipts
        .iter()
        .filter(|receipt| {
            receipt.writes.iter().any(|write| {
                write.document_type == document.r#type
                    && write.document_key == document.key
                    && write.payload_msgpack == document.payload
            })
        })
        .max_by(|left, right| {
            (&left.committed_at, &left.receipt_id).cmp(&(&right.committed_at, &right.receipt_id))
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Atlas Mind document {:?}/{:?} has no exact causal Mind commit receipt",
                document.r#type,
                document.key
            )
        })
}

fn atlas_source_for_payload(
    store_id: &str,
    payload: &AtlasPublicationPayload,
) -> Result<AtlasMindSourceVersion> {
    let source = AtlasMindSourceVersion {
        store_id: store_id.into(),
        document_type: payload.schema().into(),
        document_key: payload.key(),
        schema_id: Some(payload.schema().into()),
        payload_sha256: super::identity::sha256(&atlas_source_payload_msgpack(payload)?),
    };
    source.validate()?;
    Ok(source)
}

fn load_local_claim_reads(
    runtime_store: &Path,
    repository: &AtlasRepositoryIdentity,
) -> Result<Vec<LocalClaimRead>> {
    let mut cache = crate::runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    let mut claims = Vec::new();
    for envelope in cache
        .snapshot_envelopes()
        .into_iter()
        .filter(|entry| entry.r#type == ATLAS_DEPENDENCY_CLAIM_SCHEMA)
    {
        let claim: AtlasDependencyClaim = rmp_serde::from_slice(&envelope.payload)?;
        claim.validate()?;
        if claim.consumer != *repository || envelope.key != claim.claim_id.to_string() {
            bail!("Atlas ingress refused a foreign or mis-keyed local dependency claim")
        }
        let payload = AtlasPublicationPayload::DependencyClaim(claim.clone());
        let source = atlas_source_for_payload("epiphany-mind", &payload)?;
        claims.push(LocalClaimRead {
            ingress: AtlasLocalClaimInput {
                claim: claim.clone(),
                source_payload_sha256: source.payload_sha256.clone(),
            },
            strong: AtlasStrongRead {
                source,
                document: claim,
            },
            envelope,
        });
    }
    claims.sort_by_key(|claim| claim.strong.document.claim_id);
    Ok(claims)
}

fn load_local_impact_read(
    runtime_store: &Path,
    impact_id: uuid::Uuid,
    repository: &AtlasRepositoryIdentity,
) -> Result<Option<(CultCacheEnvelope, AtlasStrongRead<AtlasDependencyImpact>)>> {
    let mut cache = crate::runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    let key = impact_id.to_string();
    let Some(envelope) = cache.get_envelope::<AtlasDependencyImpact>(&key)? else {
        return Ok(None);
    };
    let document: AtlasDependencyImpact = rmp_serde::from_slice(&envelope.payload)?;
    document.validate()?;
    if document.consumer != *repository || document.impact_id != impact_id {
        bail!("Atlas ingress found a foreign or substituted local impact")
    }
    let source = AtlasMindSourceVersion {
        store_id: "epiphany-mind".into(),
        document_type: ATLAS_DEPENDENCY_IMPACT_SCHEMA.into(),
        document_key: key,
        schema_id: Some(ATLAS_DEPENDENCY_IMPACT_SCHEMA.into()),
        payload_sha256: super::identity::sha256(&rmp_serde::to_vec(&document)?),
    };
    Ok(Some((envelope, AtlasStrongRead { source, document })))
}

fn commit_dependency_impact_intent(
    config: &AtlasImpactIngressRunConfig,
    projector_store: &AtlasCultCacheStore,
    projection: &AtlasEntanglementProjection,
    claim_envelope: &CultCacheEnvelope,
    current_impact_envelope: Option<&CultCacheEnvelope>,
    intent: &AtlasDependencyImpactWriteIntent,
) -> Result<()> {
    intent.validate()?;
    if crate::load_current_runtime_repository_body_basis(&config.runtime_mind_store)?
        != intent.body_basis
    {
        bail!("Atlas impact admission refused a stale repository Body basis")
    }
    if projector_store.load_latest_projection(ATLAS_GAMECULT_LOCAL_AUDIENCE)?
        != Some(projection.clone())
    {
        bail!("Atlas impact admission refused a stale projection strong read")
    }
    let mut cache = crate::runtime_spine_cache(&config.runtime_mind_store)?;
    cache.pull_all_backing_stores()?;
    if cache
        .snapshot_envelopes()
        .iter()
        .find(|entry| entry.r#type == claim_envelope.r#type && entry.key == claim_envelope.key)
        != Some(claim_envelope)
    {
        bail!("Atlas impact admission refused a stale local claim strong read")
    }
    let mut strong_reads = vec![claim_envelope.clone()];
    if let Some(current) = current_impact_envelope {
        strong_reads.push(current.clone());
    }
    let provenance = prepared_envelope(intent.intent_id.clone(), intent)?;
    let write = prepared_envelope(
        intent.proposal.impact.impact_id.to_string(),
        &intent.proposal.impact,
    )?;
    match crate::commit_typed_organ_mind_mutation(
        &config.runtime_mind_store,
        "epiphany-atlas-impact-ingress",
        provenance,
        "Self",
        strong_reads,
        vec![write],
        &utc_from_unix_ms(config.now_unix_ms)?,
    )? {
        crate::EpiphanyMindCommitOutcome::Committed(_) => Ok(()),
        crate::EpiphanyMindCommitOutcome::Conflict {
            document_identities,
        } => bail!("Atlas impact Mind admission lost exact CAS for {document_identities:?}"),
    }
}

fn reconcile_completed_atlas_lanes(
    resident_store: &Path,
    lanes: &[AtlasImpactLaneState],
) -> Result<Vec<AtlasImpactLaneState>> {
    let pressures = crate::resident_self_pressures(resident_store)?;
    let mut reconciled = lanes.to_vec();
    for lane in &mut reconciled {
        let Some(proposal_id) = lane.pending_proposal_id else {
            continue;
        };
        let kind = match lane.lane {
            AtlasImpactLane::Modeling => crate::RESIDENT_SELF_ATLAS_MODELING_PRESSURE_KIND,
            AtlasImpactLane::Soul => crate::RESIDENT_SELF_ATLAS_SOUL_PRESSURE_KIND,
        };
        let pressure_id = format!("{kind}-{proposal_id}");
        let pressure = pressures
            .iter()
            .find(|pressure| pressure.pressure_id == pressure_id)
            .ok_or_else(|| anyhow::anyhow!("Atlas pending lane lost its Resident Self pressure"))?;
        if pressure.status == "pending" {
            continue;
        }
        let Some(grant_id) = pressure.consumed_by_grant_id.as_deref() else {
            bail!("Atlas consumed pressure lost its Resident Self grant identity")
        };
        let grant =
            crate::resident_self_grant_lifecycle_projection(resident_store, Some(grant_id), 1)?
                .into_iter()
                .find(|grant| grant.grant_id == grant_id)
                .ok_or_else(|| {
                    anyhow::anyhow!("Atlas pressure lost its Resident Self grant lifecycle")
                })?;
        if let Some(completed_at) = grant.terminal_at_millis {
            lane.pending_proposal_id = None;
            lane.last_completed_at_unix_ms = Some(completed_at);
        }
    }
    Ok(reconciled)
}

fn impact_state_record(
    repository: &AtlasRepositoryIdentity,
    current_revision: Option<u64>,
    lanes: &[AtlasImpactLaneState],
    now_unix_ms: u64,
) -> Result<AtlasImpactStateRecord> {
    let lane = |target| {
        lanes
            .iter()
            .find(|state| state.lane == target)
            .ok_or_else(|| anyhow::anyhow!("Atlas impact state lost a required lane"))
    };
    let modeling = lane(AtlasImpactLane::Modeling)?;
    let soul = lane(AtlasImpactLane::Soul)?;
    let record = AtlasImpactStateRecord {
        schema_version: ATLAS_IMPACT_STATE_RECORD_SCHEMA.into(),
        local_repository: repository.clone(),
        revision: current_revision
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("Atlas impact state revision exhausted"))?,
        modeling: AtlasStoredImpactLaneState {
            pending_proposal_id: modeling.pending_proposal_id,
            last_completed_at_unix_ms: modeling.last_completed_at_unix_ms,
        },
        soul: AtlasStoredImpactLaneState {
            pending_proposal_id: soul.pending_proposal_id,
            last_completed_at_unix_ms: soul.last_completed_at_unix_ms,
        },
        brake: AtlasStoredImpactBrakeState {
            engaged: false,
            brake_id: None,
        },
        updated_at_unix_ms: now_unix_ms,
    };
    record.validate()?;
    Ok(record)
}

fn atlas_brake_id(
    local_verse_store: &Path,
    runtime_id: &str,
    scope: &str,
) -> Result<Option<String>> {
    let Some(brake) = crate::load_epiphany_cultmesh_swarm_brake(local_verse_store, runtime_id)?
    else {
        return Ok(None);
    };
    if brake.status == "engaged"
        && (brake.protected_surfaces.is_empty()
            || brake.protected_surfaces.iter().any(|value| value == scope))
    {
        return Ok(Some(brake.brake_id));
    }
    Ok(None)
}

fn require_separate_paths(paths: &[&PathBuf]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for path in paths {
        let absolute = if path.is_absolute() {
            (*path).clone()
        } else {
            std::env::current_dir()?.join(path)
        };
        if !seen.insert(absolute) {
            bail!("Atlas organs require separate Mind, identity, publisher, and CultMesh stores")
        }
    }
    Ok(())
}

fn utc_from_unix_ms(unix_ms: u64) -> Result<String> {
    let millis = i64::try_from(unix_ms).context("Atlas timestamp exceeds UTC range")?;
    let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(millis)
        .ok_or_else(|| anyhow::anyhow!("Atlas timestamp is outside the UTC calendar"))?;
    Ok(timestamp.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
}

fn prepared_envelope<T: DatabaseEntry>(
    key: impl Into<String>,
    value: &T,
) -> Result<CultCacheEnvelope> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<T>()?;
    Ok(cache.prepare_entry(key, value)?.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::{Version, VersionReq};

    #[test]
    fn modeling_owned_offer_and_claim_support_verification_and_body_drift() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime.cc");
        crate::initialize_runtime_spine(
            &store,
            crate::RuntimeSpineInitOptions {
                runtime_id: "atlas-local-admission".into(),
                display_name: "Atlas local admission".into(),
                created_at: "2026-08-14T00:00:00Z".into(),
            },
        )?;
        crate::runtime_spine::tests::bind_test_runtime_swarm(&store, "swarm")?;
        crate::runtime_spine::tests::bind_test_repository_body(&store, "workspace")?;
        let body = crate::observe_runtime_repository_body_basis(&store)?;
        let manifest = crate::authenticated_repository_body_manifest(&store, &body)?;
        let body_evidence = vec![AtlasBodyEvidenceRef {
            path: manifest.entries[0].path.clone(),
            raw_sha256: manifest.entries[0].raw_sha256.clone(),
        }];
        let local = AtlasRepositoryIdentity::new("swarm", "workspace")?;
        let provider = AtlasRepositoryIdentity::new("swarm", "provider")?;
        let seed = crate::EpiphanyRepoModelSeed::new(
            "atlas-modeling-seed",
            "atlas-modeling-graph",
            "swarm",
            "workspace",
            body.body_binding_sha256.clone(),
            crate::EpiphanyRepoModelSeedDocuments {
                domains: Vec::new(),
                nodes: Vec::new(),
                edges: Vec::new(),
                frontier: Vec::new(),
            },
        )?;
        crate::initialize_keyed_repo_model(&store, &seed, "2026-08-14T00:00:01Z")?;
        let proposal = crate::EpiphanyRepoModelMutationProposal::new(
            "atlas-modeling-proposal",
            "atlas-modeling-request",
            "atlas-modeling-result",
            vec!["atlas-modeling-evidence".into()],
            body.clone(),
            vec![
                crate::EpiphanyRepoModelMutationOperation::CreateSurfaceOffer {
                    label: "Local contract surface".into(),
                    contract: AtlasContractDescriptor::Semver {
                        contract_id: "contract:local".into(),
                        version: Version::parse("1.0.0")?,
                    },
                    source_refs: vec![manifest.entries[0].path.clone()],
                },
                crate::EpiphanyRepoModelMutationOperation::CreateDependencyClaim {
                    label: "Provider runtime dependency".into(),
                    target: AtlasDependencyTarget::Exact {
                        provider: provider.clone(),
                        surface_id: uuid::Uuid::from_u128(3),
                        requirement: AtlasContractRequirement::Semver {
                            contract_id: "contract:provider".into(),
                            requirement: VersionReq::parse("^1")?,
                        },
                    },
                    entanglement_kind: AtlasEntanglementKind::Runtime,
                    failure_semantics: AtlasFailureSemantics::FailClosed,
                    impact_scope: AtlasImpactScope::WholeRepository,
                    source_refs: vec![manifest.entries[0].path.clone()],
                },
            ],
        )?;
        let plan = crate::plan_repo_model_mutation(&store, &proposal)?;
        let mut cache = crate::runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let provenance = cache.prepare_entry(&proposal.proposal_id, &proposal)?.0;
        assert!(matches!(
            crate::commit_typed_organ_mind_mutation(
                &store,
                "Modeling",
                provenance,
                "Modeling.repo_model_mutation",
                plan.strong_reads,
                plan.writes,
                "2026-08-14T00:00:02Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));
        let view = crate::assemble_repo_model_view(&store)?;
        let claim = view.dependency_claims[0].clone();

        let local_signer = enroll_service_identity_at::<AtlasRepositorySigningIdentity>(
            &temp.path().join("local-atlas-identity.cc"),
        )?;
        let provider_signer = enroll_service_identity_at::<AtlasRepositorySigningIdentity>(
            &temp.path().join("provider-atlas-identity.cc"),
        )?;
        let trust = |repository: AtlasRepositoryIdentity,
                     signer: &cultnet_rs::ServiceIdentitySigner<AtlasRepositorySigningIdentity>|
         -> Result<AtlasPublisherTrustBinding> {
            Ok(AtlasPublisherTrustBinding {
                schema_version: ATLAS_TRUST_BINDING_SCHEMA.into(),
                publisher: repository,
                signer_identity_id: signer.entry().identity_id.clone(),
                trusted_from_unix_ms: 1_799_999_000_000,
                expires_at_unix_ms: None,
                revoked: false,
                trust_anchor: signer.trust_anchor()?,
            })
        };
        let publication_read = |document: AtlasPublicationEnvelope| -> Result<
            AtlasStrongRead<AtlasPublicationEnvelope>,
        > {
            Ok(AtlasStrongRead {
                source: AtlasMindSourceVersion {
                    store_id: "odin".into(),
                    document_type: ATLAS_PUBLICATION_SCHEMA.into(),
                    document_key: document.statement.publication_id.clone(),
                    schema_id: Some(ATLAS_PUBLICATION_SCHEMA.into()),
                    payload_sha256: super::super::identity::sha256(&rmp_serde::to_vec(&document)?),
                },
                document,
            })
        };
        let publish =
            |signer: &cultnet_rs::ServiceIdentitySigner<AtlasRepositorySigningIdentity>,
             repository: AtlasRepositoryIdentity,
             runtime_id: &str,
             body_basis: crate::RepositoryBodyObservationBasis,
             sequence: u64,
             payload: AtlasPublicationPayload|
             -> Result<AtlasPublicationEnvelope> {
                let source = atlas_source_for_payload("epiphany-mind", &payload)?;
                sign_atlas_publication(
                    signer,
                    repository,
                    sequence,
                    runtime_id.into(),
                    format!("incarnation-{runtime_id}"),
                    body_basis,
                    format!("cultmesh://gamecult-local/{runtime_id}/atlas"),
                    Some(source.clone()),
                    Some(AtlasMindCommitBinding {
                        receipt_id: format!("receipt-{runtime_id}-{sequence}"),
                        receipt_sha256: super::super::identity::sha256(
                            format!("receipt-{runtime_id}-{sequence}").as_bytes(),
                        ),
                        invariant_owner: "Mind".into(),
                        source,
                    }),
                    1_800_000_000_000,
                    payload,
                )
            };

        let claim_payload = AtlasPublicationPayload::DependencyClaim(claim.clone());
        let claim_publication = publication_read(publish(
            &local_signer,
            local.clone(),
            &body.runtime_id,
            body.clone(),
            1,
            claim_payload,
        )?)?;
        let provider_offer = AtlasSurfaceOffer {
            schema_version: ATLAS_SURFACE_OFFER_SCHEMA.into(),
            provider: provider.clone(),
            surface_id: uuid::Uuid::from_u128(3),
            contract: AtlasContractDescriptor::Semver {
                contract_id: "contract:provider".into(),
                version: Version::parse("1.2.0")?,
            },
            lifecycle: AtlasOfferLifecycle::Active,
            label: "Provider contract surface".into(),
            body_evidence,
        };
        let mut provider_body = body.clone();
        provider_body.workspace_id = provider.workspace_id.clone();
        provider_body.runtime_id = "runtime-provider".into();
        provider_body.observation_id = "observation-provider".into();
        let provider_runtime_id = provider_body.runtime_id.clone();
        let offer_publication = publication_read(publish(
            &provider_signer,
            provider.clone(),
            &provider_runtime_id,
            provider_body,
            1,
            AtlasPublicationPayload::SurfaceOffer(provider_offer),
        )?)?;
        let claim_source = atlas_source_for_payload(
            "epiphany-mind",
            &AtlasPublicationPayload::DependencyClaim(claim.clone()),
        )?;
        let evidence_payload = b"verified provider compatibility".to_vec();
        let verification_evidence = AtlasEvidenceArtifactRead {
            version: AtlasEvidenceArtifactVersion {
                artifact_id: "artifact:atlas-runtime-verification".into(),
                payload_sha256: super::super::identity::sha256(&evidence_payload),
            },
            payload: evidence_payload,
        };
        let verification_intent =
            admit_dependency_verification(AtlasDependencyVerificationAdmissionInput {
                context: AtlasPlannerContext {
                    local_repository: local.clone(),
                    current_body_basis: body,
                },
                now_unix_ms: 1_800_000_000_001,
                expected_claim_source: claim_source.clone(),
                claim: AtlasStrongRead {
                    source: claim_source,
                    document: claim.clone(),
                },
                claim_publication: claim_publication.clone(),
                claim_publication_trust: trust(local.clone(), &local_signer)?,
                offer_publication: offer_publication.clone(),
                offer_publication_trust: trust(provider.clone(), &provider_signer)?,
                expected_current_verification_source: None,
                current_verification: None,
                evidence: verification_evidence.clone(),
                verdict: AtlasVerificationVerdict::Passed,
            })?;
        assert!(matches!(
            commit_atlas_dependency_verification_intent(
                &store,
                &verification_intent,
                &AtlasDependencyVerificationCommitEvidence {
                    now_unix_ms: 1_800_000_000_001,
                    claim_publication,
                    claim_publication_trust: trust(
                        verification_intent.local_repository.clone(),
                        &local_signer,
                    )?,
                    offer_publication,
                    offer_publication_trust: trust(provider, &provider_signer)?,
                    evidence: verification_evidence,
                },
                "2026-08-14T00:00:04Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));

        std::fs::write(
            store
                .with_extension("workspace.body-repo")
                .join("body-seed.txt"),
            b"changed provider evidence",
        )?;
        let changed_body = crate::observe_runtime_repository_body_basis(&store)?;
        let stale_source = RuntimeAtlasMindSnapshotSource::new(&store, local, changed_body)?;
        assert!(stale_source.load_local_atlas_mind_snapshot().is_err());
        Ok(())
    }

    #[test]
    fn publisher_identity_reopens_and_exports_one_stable_public_anchor() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let identity_store = temp.path().join("publisher-identity.cc");
        let trust_anchor_store = temp.path().join("publisher-trust-anchor.cc");

        let enrolled = open_or_enroll_atlas_repository_identity(&identity_store)?;
        let first_anchor = export_service_identity_trust_anchor(&enrolled, &trust_anchor_store)?;
        let reopened = open_or_enroll_atlas_repository_identity(&identity_store)?;
        let replayed_anchor = export_service_identity_trust_anchor(&reopened, &trust_anchor_store)?;

        assert_eq!(enrolled.entry(), reopened.entry());
        assert_eq!(first_anchor, replayed_anchor);
        assert_eq!(
            read_atlas_repository_trust_anchor(&trust_anchor_store)?,
            first_anchor
        );
        Ok(())
    }
}
