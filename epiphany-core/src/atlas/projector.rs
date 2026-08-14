use anyhow::{Result, bail};
use semver::Op;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::contracts::*;
use super::identity::{AtlasPublisherTrustBinding, sha256, verify_atlas_publication};

#[derive(Clone)]
struct AcceptedPublication {
    publication: AtlasPublicationEnvelope,
    freshness: AtlasPublicationFreshness,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DependencyEdge {
    claim_id: uuid::Uuid,
    consumer: AtlasRepositoryIdentity,
    provider: AtlasRepositoryIdentity,
    provider_surface_id: uuid::Uuid,
    kind: AtlasEntanglementKind,
    impact_scope: AtlasImpactScope,
}

pub fn project_atlas(
    publications: &[AtlasPublicationEnvelope],
    trust_bindings: &[AtlasPublisherTrustBinding],
    audience: &AtlasProjectionAudience,
    freshness_policy: &AtlasFreshnessPolicy,
    now_unix_ms: u64,
) -> Result<AtlasEntanglementProjection> {
    validate_identifier(&audience.audience_id, "Atlas projection audience id")?;
    freshness_policy.validate()?;

    // Visibility is decided before signature validation, indexing, latest
    // selection, or any join. A non-GameCult audience receives no hint that a
    // publisher, claim, or hidden edge exists.
    if !audience.gamecult_local {
        return finish_projection(AtlasEntanglementProjection {
            schema_version: ATLAS_PROJECTION_SCHEMA.into(),
            audience_id: audience.audience_id.clone(),
            evaluated_at_unix_ms: now_unix_ms,
            source_publication_ids: Vec::new(),
            publisher_status: Vec::new(),
            entanglements: Vec::new(),
            cycles: Vec::new(),
            blast_radii: Vec::new(),
            projection_sha256: String::new(),
        });
    }

    let trust = trust_directory(trust_bindings, now_unix_ms)?;
    let mut verified = Vec::with_capacity(publications.len());
    for publication in publications {
        if publication.statement.visibility != AtlasPublicationVisibility::GamecultLocal {
            continue;
        }
        let binding = trust
            .get(&publication.statement.publisher.repository_uri)
            .ok_or_else(|| anyhow::anyhow!("Atlas publication has no trusted local publisher"))?;
        verify_atlas_publication(binding, publication, now_unix_ms)?;
        verified.push(publication.clone());
    }

    let statuses = latest_status_publications(&verified, freshness_policy, now_unix_ms)?;
    let selected = select_exact_watermarked_publications(&verified, &statuses)?;
    let status_index = statuses
        .iter()
        .map(|status| {
            (
                status
                    .publication
                    .statement
                    .publisher
                    .repository_uri
                    .clone(),
                status,
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut offers = BTreeMap::<(String, uuid::Uuid), &AcceptedPublication>::new();
    let mut claims = BTreeMap::<(String, uuid::Uuid), &AcceptedPublication>::new();
    let mut verifications = BTreeMap::<(String, uuid::Uuid), &AcceptedPublication>::new();
    for accepted in &selected {
        match &accepted.publication.statement.payload {
            AtlasPublicationPayload::SurfaceOffer(offer) => {
                offers.insert(
                    (offer.provider.repository_uri.clone(), offer.surface_id),
                    accepted,
                );
            }
            AtlasPublicationPayload::DependencyClaim(claim) => {
                claims.insert(
                    (claim.consumer.repository_uri.clone(), claim.claim_id),
                    accepted,
                );
            }
            AtlasPublicationPayload::DependencyVerification(verification) => {
                verifications.insert(
                    (
                        verification.consumer.repository_uri.clone(),
                        verification.claim_id,
                    ),
                    accepted,
                );
            }
            AtlasPublicationPayload::PublisherStatus(_) => {
                bail!("publisher status entered the watermarked Mind-document set")
            }
        }
    }

    let mut entanglements = Vec::with_capacity(claims.len());
    let mut edges = Vec::new();
    for accepted_claim in claims.values() {
        let AtlasPublicationPayload::DependencyClaim(claim) =
            &accepted_claim.publication.statement.payload
        else {
            unreachable!("claim index is typed")
        };
        let exact_target = match &claim.target {
            AtlasDependencyTarget::Exact {
                provider,
                surface_id,
                requirement,
            } => Some((provider, *surface_id, requirement)),
            AtlasDependencyTarget::Unresolved { .. } => None,
        };
        let accepted_offer = exact_target.and_then(|(provider, surface_id, _)| {
            offers
                .get(&(provider.repository_uri.clone(), surface_id))
                .copied()
        });
        let offer =
            accepted_offer.and_then(|accepted| match &accepted.publication.statement.payload {
                AtlasPublicationPayload::SurfaceOffer(offer) => Some(offer),
                _ => None,
            });
        let compatibility = evaluate_atlas_compatibility(claim, offer);
        let accepted_verification = verifications
            .get(&(claim.consumer.repository_uri.clone(), claim.claim_id))
            .copied();
        let verification =
            evaluate_verification(accepted_claim, accepted_offer, accepted_verification);
        let exact_verification = accepted_verification.filter(|_| {
            !matches!(
                verification,
                AtlasVerificationState::Missing | AtlasVerificationState::ExactEdgeMismatch
            )
        });
        let verification_document =
            exact_verification.and_then(|accepted| match &accepted.publication.statement.payload {
                AtlasPublicationPayload::DependencyVerification(verification) => Some(verification),
                _ => None,
            });
        // The consumer's active exact claim owns dependency truth. Missing,
        // withdrawn, or incompatible provider state changes compatibility; it
        // must not erase the claimed edge or its transitive blast radius.
        if claim.lifecycle == AtlasClaimLifecycle::Active && exact_target.is_some() {
            let (provider, _, _) = exact_target.expect("active Atlas target is exact");
            edges.push(DependencyEdge {
                claim_id: claim.claim_id,
                consumer: claim.consumer.clone(),
                provider: provider.clone(),
                provider_surface_id: exact_target.expect("active Atlas target is exact").1,
                kind: claim.entanglement_kind,
                impact_scope: claim.impact_scope.clone(),
            });
        }
        entanglements.push(AtlasProjectedEntanglement {
            claim_id: claim.claim_id,
            claim_label: claim.label.clone(),
            claim_requirement: claim.target.requirement().clone(),
            claim_body_evidence: claim.body_evidence.clone(),
            consumer: claim.consumer.clone(),
            provider: exact_target.map(|(provider, _, _)| provider.clone()),
            surface_id: exact_target.map(|(_, surface_id, _)| surface_id),
            offer_label: offer.map(|offer| offer.label.clone()),
            offer_contract: offer.map(|offer| offer.contract.clone()),
            offer_lifecycle: offer.map(|offer| offer.lifecycle.clone()),
            offer_body_evidence: offer
                .map(|offer| offer.body_evidence.clone())
                .unwrap_or_default(),
            entanglement_kind: claim.entanglement_kind,
            failure_semantics: claim.failure_semantics,
            impact_scope: claim.impact_scope.clone(),
            claim_freshness: accepted_claim.freshness,
            offer_freshness: accepted_offer.map(|offer| offer.freshness),
            compatibility,
            verification,
            claim_publication_id: accepted_claim.publication.statement.publication_id.clone(),
            offer_publication_id: accepted_offer
                .map(|offer| offer.publication.statement.publication_id.clone()),
            verification_publication_id: exact_verification
                .map(|accepted| accepted.publication.statement.publication_id.clone()),
            verification_evidence_sha256: verification_document
                .map(|verification| verification.evidence_sha256.clone()),
        });
    }
    entanglements.sort_by(|left, right| {
        (
            &left.consumer.repository_uri,
            left.claim_id,
            left.provider
                .as_ref()
                .map(|provider| provider.repository_uri.as_str()),
            left.surface_id,
        )
            .cmp(&(
                &right.consumer.repository_uri,
                right.claim_id,
                right
                    .provider
                    .as_ref()
                    .map(|provider| provider.repository_uri.as_str()),
                right.surface_id,
            ))
    });
    edges.sort_by(|left, right| {
        (
            &left.consumer.repository_uri,
            &left.provider.repository_uri,
            left.provider_surface_id,
            left.claim_id,
            left.kind,
        )
            .cmp(&(
                &right.consumer.repository_uri,
                &right.provider.repository_uri,
                right.provider_surface_id,
                right.claim_id,
                right.kind,
            ))
    });
    edges.dedup();

    let mut source_publication_ids = statuses
        .iter()
        .map(|status| status.publication.statement.publication_id.clone())
        .chain(
            selected
                .iter()
                .map(|source| source.publication.statement.publication_id.clone()),
        )
        .collect::<Vec<_>>();
    source_publication_ids.sort();
    source_publication_ids.dedup();
    validate_sorted_unique_sha256(
        &source_publication_ids,
        "Atlas projection source publication ids",
        !source_publication_ids.is_empty(),
    )?;

    let publisher_status = statuses
        .iter()
        .map(|accepted| {
            let AtlasPublicationPayload::PublisherStatus(status) =
                &accepted.publication.statement.payload
            else {
                unreachable!("status set is typed")
            };
            AtlasPublisherProjectionStatus {
                publisher: status.publisher.clone(),
                runtime_id: status.runtime_id.clone(),
                runtime_incarnation_id: status.runtime_incarnation_id.clone(),
                heartbeat_sequence: status.heartbeat_sequence,
                heartbeat_at_unix_ms: status.heartbeat_at_unix_ms,
                freshness: accepted.freshness,
                watermarks: status.watermarks.clone(),
                status_publication_id: accepted.publication.statement.publication_id.clone(),
            }
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(publisher_status.len(), status_index.len());

    finish_projection(AtlasEntanglementProjection {
        schema_version: ATLAS_PROJECTION_SCHEMA.into(),
        audience_id: audience.audience_id.clone(),
        evaluated_at_unix_ms: now_unix_ms,
        source_publication_ids,
        publisher_status,
        entanglements,
        cycles: classify_cycles(&edges),
        blast_radii: derive_blast_radii(&edges),
        projection_sha256: String::new(),
    })
}

pub fn evaluate_atlas_compatibility(
    claim: &AtlasDependencyClaim,
    offer: Option<&AtlasSurfaceOffer>,
) -> AtlasCompatibility {
    if claim.lifecycle == AtlasClaimLifecycle::Retired {
        return AtlasCompatibility::ClaimRetired;
    }
    let AtlasDependencyTarget::Exact { requirement, .. } = &claim.target else {
        return AtlasCompatibility::Unresolved;
    };
    let Some(offer) = offer else {
        return AtlasCompatibility::OfferMissing;
    };
    if matches!(offer.lifecycle, AtlasOfferLifecycle::Withdrawn) {
        return AtlasCompatibility::OfferWithdrawn;
    }
    if requirement.contract_id() != offer.contract.contract_id() {
        return AtlasCompatibility::ContractIdMismatch;
    }
    match (requirement, &offer.contract) {
        (
            AtlasContractRequirement::Semver { requirement, .. },
            AtlasContractDescriptor::Semver { version, .. },
        ) if requirement.matches(version) => {
            if is_exact_semver_requirement(requirement, version) {
                AtlasCompatibility::Exact
            } else {
                AtlasCompatibility::Compatible
            }
        }
        (
            AtlasContractRequirement::ExactSchema { schema_id, .. },
            AtlasContractDescriptor::ExactSchema {
                schema_id: offered, ..
            },
        ) if schema_id == offered => AtlasCompatibility::Exact,
        (
            AtlasContractRequirement::ExactDigest { sha256, .. },
            AtlasContractDescriptor::ExactDigest {
                sha256: offered, ..
            },
        ) if sha256 == offered => AtlasCompatibility::Exact,
        (AtlasContractRequirement::Semver { .. }, AtlasContractDescriptor::Semver { .. })
        | (
            AtlasContractRequirement::ExactSchema { .. },
            AtlasContractDescriptor::ExactSchema { .. },
        )
        | (
            AtlasContractRequirement::ExactDigest { .. },
            AtlasContractDescriptor::ExactDigest { .. },
        ) => AtlasCompatibility::VersionMismatch,
        _ => AtlasCompatibility::VersionSchemeMismatch,
    }
}

pub fn classify_atlas_publisher_freshness(
    status: &AtlasPublisherStatus,
    policy: &AtlasFreshnessPolicy,
    now_unix_ms: u64,
) -> Result<AtlasPublicationFreshness> {
    if status.heartbeat_at_unix_ms > now_unix_ms.saturating_add(policy.maximum_future_skew_ms) {
        bail!("Atlas publisher status heartbeat is implausibly future-dated")
    }
    if status.state == AtlasPublisherState::Retired {
        Ok(AtlasPublicationFreshness::Retired)
    } else if now_unix_ms.saturating_sub(status.heartbeat_at_unix_ms)
        > policy.publisher_status_maximum_age_ms
    {
        Ok(AtlasPublicationFreshness::LastKnownStale)
    } else {
        Ok(AtlasPublicationFreshness::Current)
    }
}

pub fn atlas_projection_digest(projection: &AtlasEntanglementProjection) -> Result<String> {
    let mut unsigned = projection.clone();
    unsigned.projection_sha256.clear();
    Ok(sha256(
        &[
            b"gamecult.model.entanglement-projection.digest.v0\0".as_slice(),
            rmp_serde::to_vec_named(&unsigned)?.as_slice(),
        ]
        .concat(),
    ))
}

fn finish_projection(
    mut projection: AtlasEntanglementProjection,
) -> Result<AtlasEntanglementProjection> {
    projection.projection_sha256 = atlas_projection_digest(&projection)?;
    Ok(projection)
}

fn trust_directory<'a>(
    bindings: &'a [AtlasPublisherTrustBinding],
    now_unix_ms: u64,
) -> Result<BTreeMap<String, &'a AtlasPublisherTrustBinding>> {
    let mut trust = BTreeMap::new();
    for binding in bindings {
        binding.validate_at(now_unix_ms)?;
        match trust.insert(binding.publisher.repository_uri.clone(), binding) {
            None => {}
            Some(existing) if existing == binding => {}
            Some(_) => bail!("Atlas repository has conflicting trust bindings"),
        }
    }
    Ok(trust)
}

fn latest_status_publications(
    publications: &[AtlasPublicationEnvelope],
    policy: &AtlasFreshnessPolicy,
    now_unix_ms: u64,
) -> Result<Vec<AcceptedPublication>> {
    let mut histories = BTreeMap::<String, Vec<&AtlasPublicationEnvelope>>::new();
    for publication in publications {
        let AtlasPublicationPayload::PublisherStatus(status) = &publication.statement.payload
        else {
            continue;
        };
        histories
            .entry(status.publisher.repository_uri.clone())
            .or_default()
            .push(publication);
    }
    histories
        .into_values()
        .map(|history| {
            let publication = validate_status_history_and_latest(history)?;
            let AtlasPublicationPayload::PublisherStatus(status) = &publication.statement.payload
            else {
                unreachable!("status index is typed")
            };
            Ok(AcceptedPublication {
                freshness: classify_atlas_publisher_freshness(status, policy, now_unix_ms)?,
                publication: publication.clone(),
            })
        })
        .collect()
}

fn validate_status_history_and_latest(
    mut history: Vec<&AtlasPublicationEnvelope>,
) -> Result<&AtlasPublicationEnvelope> {
    history.sort_by(|left, right| {
        let AtlasPublicationPayload::PublisherStatus(left_status) = &left.statement.payload else {
            unreachable!("status history is typed")
        };
        let AtlasPublicationPayload::PublisherStatus(right_status) = &right.statement.payload
        else {
            unreachable!("status history is typed")
        };
        (
            left_status.heartbeat_sequence,
            &left.statement.publication_id,
        )
            .cmp(&(
                right_status.heartbeat_sequence,
                &right.statement.publication_id,
            ))
    });
    history.dedup();
    let mut prior_sequence = None;
    let mut prior_time = None;
    let mut retired = false;
    let mut high_water = BTreeMap::<(String, String), (u64, String)>::new();
    for publication in &history {
        let AtlasPublicationPayload::PublisherStatus(status) = &publication.statement.payload
        else {
            unreachable!("status history is typed")
        };
        if prior_sequence == Some(status.heartbeat_sequence) {
            bail!("Atlas publisher emitted conflicting status at one heartbeat sequence")
        }
        if prior_time.is_some_and(|prior| status.heartbeat_at_unix_ms <= prior) || retired {
            bail!("Atlas publisher status time regressed or advanced after retirement")
        }
        for watermark in &status.watermarks {
            let key = (
                watermark.source_schema.clone(),
                watermark.source_key.clone(),
            );
            if let Some((sequence, digest)) = high_water.get(&key) {
                if watermark.publication_sequence < *sequence
                    || (watermark.publication_sequence == *sequence
                        && watermark.source_payload_sha256 != *digest)
                {
                    bail!("Atlas publisher status watermark regressed or forked")
                }
            }
            high_water.insert(
                key,
                (
                    watermark.publication_sequence,
                    watermark.source_payload_sha256.clone(),
                ),
            );
        }
        prior_sequence = Some(status.heartbeat_sequence);
        prior_time = Some(status.heartbeat_at_unix_ms);
        retired = status.state == AtlasPublisherState::Retired;
    }
    history
        .last()
        .copied()
        .ok_or_else(|| anyhow::anyhow!("Atlas status history is empty"))
}

fn select_exact_watermarked_publications(
    publications: &[AtlasPublicationEnvelope],
    statuses: &[AcceptedPublication],
) -> Result<Vec<AcceptedPublication>> {
    let candidates = publications
        .iter()
        .filter(|publication| {
            !matches!(
                publication.statement.payload,
                AtlasPublicationPayload::PublisherStatus(_)
            )
        })
        .collect::<Vec<_>>();
    let mut selected = Vec::new();
    for accepted_status in statuses {
        let AtlasPublicationPayload::PublisherStatus(status) =
            &accepted_status.publication.statement.payload
        else {
            unreachable!("status set is typed")
        };
        for watermark in &status.watermarks {
            if !matches!(
                watermark.source_schema.as_str(),
                ATLAS_SURFACE_OFFER_SCHEMA
                    | ATLAS_DEPENDENCY_CLAIM_SCHEMA
                    | ATLAS_DEPENDENCY_VERIFICATION_SCHEMA
            ) {
                bail!("Atlas status watermark names a schema outside the accepted publication set")
            }
            let matching = candidates
                .iter()
                .filter(|publication| {
                    publication.statement.publisher == status.publisher
                        && publication.statement.payload.schema() == watermark.source_schema
                        && publication.statement.payload.key() == watermark.source_key
                        && publication.statement.canonical_payload_msgpack_sha256
                            == watermark.source_payload_sha256
                        && publication.statement.publication_sequence
                            == watermark.publication_sequence
                })
                .copied()
                .collect::<Vec<_>>();
            match matching.as_slice() {
                [publication] => selected.push(AcceptedPublication {
                    publication: (*publication).clone(),
                    freshness: accepted_status.freshness,
                }),
                [] => bail!("Atlas publisher status watermark has no exact publication"),
                _ => bail!("Atlas publisher status watermark resolves ambiguously"),
            }
        }
    }
    selected.sort_by(|left, right| {
        (
            &left.publication.statement.publisher.repository_uri,
            left.publication.statement.payload.schema(),
            left.publication.statement.payload.key(),
        )
            .cmp(&(
                &right.publication.statement.publisher.repository_uri,
                right.publication.statement.payload.schema(),
                right.publication.statement.payload.key(),
            ))
    });
    Ok(selected)
}

fn evaluate_verification(
    claim: &AcceptedPublication,
    offer: Option<&AcceptedPublication>,
    verification: Option<&AcceptedPublication>,
) -> AtlasVerificationState {
    let Some(verification) = verification else {
        return AtlasVerificationState::Missing;
    };
    if verification.freshness != AtlasPublicationFreshness::Current {
        return AtlasVerificationState::LastKnownStale;
    }
    let Some(offer) = offer else {
        return AtlasVerificationState::ExactEdgeMismatch;
    };
    let AtlasPublicationPayload::DependencyClaim(claim_document) =
        &claim.publication.statement.payload
    else {
        unreachable!("claim input is typed")
    };
    let AtlasPublicationPayload::SurfaceOffer(offer_document) =
        &offer.publication.statement.payload
    else {
        unreachable!("offer input is typed")
    };
    let AtlasPublicationPayload::DependencyVerification(verification_document) =
        &verification.publication.statement.payload
    else {
        unreachable!("verification input is typed")
    };
    if verification_document.consumer != claim_document.consumer
        || verification_document.claim_id != claim_document.claim_id
        || verification_document.claim_publication_id != claim.publication.statement.publication_id
        || verification_document.offer_publication_id != offer.publication.statement.publication_id
        || verification_document.exact_contract != offer_document.contract
    {
        return AtlasVerificationState::ExactEdgeMismatch;
    }
    match verification_document.verdict {
        AtlasVerificationVerdict::Passed => AtlasVerificationState::Passed,
        AtlasVerificationVerdict::Failed => AtlasVerificationState::Failed,
    }
}

fn is_exact_semver_requirement(
    requirement: &semver::VersionReq,
    version: &semver::Version,
) -> bool {
    requirement.comparators.len() == 1
        && requirement.comparators[0].op == Op::Exact
        && requirement.comparators[0].major == version.major
        && requirement.comparators[0].minor == Some(version.minor)
        && requirement.comparators[0].patch == Some(version.patch)
        && requirement.comparators[0].pre == version.pre
}

fn classify_cycles(edges: &[DependencyEdge]) -> Vec<AtlasProjectedCycle> {
    let nodes = edges
        .iter()
        .flat_map(|edge| [edge.consumer.clone(), edge.provider.clone()])
        .collect::<BTreeSet<_>>();
    let adjacency = dependency_adjacency(edges);
    let mut assigned = BTreeSet::new();
    let mut cycles = Vec::new();
    for node in &nodes {
        if assigned.contains(node) {
            continue;
        }
        let forward = reachable(node, &adjacency);
        let component = nodes
            .iter()
            .filter(|candidate| {
                forward.contains(*candidate) && reachable(candidate, &adjacency).contains(node)
            })
            .cloned()
            .collect::<Vec<_>>();
        assigned.extend(component.iter().cloned());
        if component.len() < 2 {
            continue;
        }
        let component_set = component.iter().cloned().collect::<BTreeSet<_>>();
        let kinds = canonical_entanglement_kinds(edges.iter().filter_map(|edge| {
            (component_set.contains(&edge.consumer) && component_set.contains(&edge.provider))
                .then_some(edge.kind)
        }));
        cycles.push(AtlasProjectedCycle {
            repositories: component,
            classification: classify_cycle_kinds(&kinds),
            entanglement_kinds: kinds,
        });
    }
    cycles.sort_by(|left, right| left.repositories.cmp(&right.repositories));
    cycles
}

fn classify_cycle_kinds(kinds: &[AtlasEntanglementKind]) -> AtlasCycleClass {
    if kinds.contains(&AtlasEntanglementKind::Build) {
        AtlasCycleClass::ForbiddenBuild
    } else if kinds.contains(&AtlasEntanglementKind::Deployment) {
        AtlasCycleClass::ForbiddenDeployment
    } else if kinds.contains(&AtlasEntanglementKind::InfrastructureControl) {
        AtlasCycleClass::ForbiddenInfrastructureControl
    } else if kinds.iter().any(|kind| {
        matches!(
            kind,
            AtlasEntanglementKind::Runtime
                | AtlasEntanglementKind::SchemaProtocol
                | AtlasEntanglementKind::DataState
        )
    }) {
        AtlasCycleClass::ReviewRequired
    } else {
        AtlasCycleClass::Informational
    }
}

fn derive_blast_radii(edges: &[DependencyEdge]) -> Vec<AtlasBlastRadius> {
    // A change begins at an offered surface. A dependency claim carries it
    // into the consumer's declared local scope. Whole-repository scope may
    // reach every surface the consumer offers; a local scope may reach only
    // those exact opaque surface ids. This is the join's only transitive rule.
    let review_cycle_claims = cycle_review_boundary_claims(edges);
    let sources = edges
        .iter()
        .map(|edge| (edge.provider.clone(), edge.provider_surface_id))
        .collect::<BTreeSet<_>>();
    let mut radii = Vec::new();
    for (source, source_surface_id) in &sources {
        let mut distances = BTreeMap::<AtlasRepositoryIdentity, u32>::new();
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::from([(source.clone(), Some(*source_surface_id), 0_u32)]);
        while let Some((affected_provider, affected_surface, distance)) = queue.pop_front() {
            if !visited.insert((affected_provider.clone(), affected_surface)) {
                continue;
            }
            for edge in edges.iter().filter(|edge| {
                edge.provider == affected_provider
                    && affected_surface.is_none_or(|surface| edge.provider_surface_id == surface)
                    && !review_cycle_claims.contains(&edge.claim_id)
            }) {
                let next_distance = distance.saturating_add(1);
                distances
                    .entry(edge.consumer.clone())
                    .and_modify(|current| *current = (*current).min(next_distance))
                    .or_insert(next_distance);
                match &edge.impact_scope {
                    AtlasImpactScope::WholeRepository => {
                        queue.push_back((edge.consumer.clone(), None, next_distance))
                    }
                    AtlasImpactScope::LocalSurfaces { surface_ids } => {
                        for surface_id in surface_ids {
                            queue.push_back((
                                edge.consumer.clone(),
                                Some(*surface_id),
                                next_distance,
                            ));
                        }
                    }
                }
            }
        }
        distances.remove(source);
        if !distances.is_empty() {
            radii.push(AtlasBlastRadius {
                source: source.clone(),
                source_surface_id: *source_surface_id,
                affected: distances
                    .into_iter()
                    .map(|(repository, minimum_hops)| AtlasAffectedRepository {
                        repository,
                        minimum_hops,
                    })
                    .collect(),
            });
        }
    }
    radii
}

fn cycle_review_boundary_claims(edges: &[DependencyEdge]) -> BTreeSet<uuid::Uuid> {
    let adjacency = dependency_adjacency(edges);
    edges
        .iter()
        .filter(|edge| {
            matches!(
                edge.kind,
                AtlasEntanglementKind::Runtime
                    | AtlasEntanglementKind::SchemaProtocol
                    | AtlasEntanglementKind::DataState
            ) && reachable(&edge.provider, &adjacency).contains(&edge.consumer)
        })
        .map(|edge| edge.claim_id)
        .collect()
}

fn dependency_adjacency(
    edges: &[DependencyEdge],
) -> BTreeMap<AtlasRepositoryIdentity, BTreeSet<AtlasRepositoryIdentity>> {
    let mut adjacency = BTreeMap::new();
    for edge in edges {
        adjacency
            .entry(edge.consumer.clone())
            .or_insert_with(BTreeSet::new)
            .insert(edge.provider.clone());
        adjacency
            .entry(edge.provider.clone())
            .or_insert_with(BTreeSet::new);
    }
    adjacency
}

fn reachable(
    start: &AtlasRepositoryIdentity,
    adjacency: &BTreeMap<AtlasRepositoryIdentity, BTreeSet<AtlasRepositoryIdentity>>,
) -> BTreeSet<AtlasRepositoryIdentity> {
    let mut visited = BTreeSet::from([start.clone()]);
    let mut queue = VecDeque::from([start.clone()]);
    while let Some(current) = queue.pop_front() {
        for next in adjacency.get(&current).into_iter().flatten() {
            if visited.insert(next.clone()) {
                queue.push_back(next.clone());
            }
        }
    }
    visited
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use cultnet_rs::{ServiceIdentitySigner, enroll_service_identity_at};
    use tempfile::TempDir;
    use uuid::Uuid;

    const NOW: u64 = 1_800_000_000_000;
    const MAXIMUM_STATUS_AGE_MS: u64 = 1_000;

    struct RepositoryFixture {
        identity: AtlasRepositoryIdentity,
        signer: ServiceIdentitySigner<super::super::identity::AtlasRepositorySigningIdentity>,
        trust: AtlasPublisherTrustBinding,
        body_basis: crate::RepositoryBodyObservationBasis,
        runtime_id: String,
        runtime_incarnation_id: String,
    }

    struct SchemaChainFixture {
        epiphany: AtlasRepositoryIdentity,
        odin: AtlasRepositoryIdentity,
        eve: AtlasRepositoryIdentity,
        epiphany_surface: Uuid,
        publications: Vec<AtlasPublicationEnvelope>,
        trust: Vec<AtlasPublisherTrustBinding>,
    }

    fn repository(temp: &TempDir, name: &str, ordinal: u128) -> Result<RepositoryFixture> {
        let identity = AtlasRepositoryIdentity::new("gamecult", format!("workspace-{name}"))?;
        let signer = enroll_service_identity_at::<
            super::super::identity::AtlasRepositorySigningIdentity,
        >(&temp.path().join(format!("atlas-{name}.cc")))?;
        let runtime_id = format!("runtime-{name}");
        let runtime_incarnation_id = format!("runtime-incarnation-{ordinal}");
        let trust = AtlasPublisherTrustBinding {
            schema_version: ATLAS_TRUST_BINDING_SCHEMA.into(),
            publisher: identity.clone(),
            signer_identity_id: signer.entry().identity_id.clone(),
            trusted_from_unix_ms: NOW - 10_000,
            expires_at_unix_ms: None,
            revoked: false,
            trust_anchor: signer.trust_anchor()?,
        };
        let body_basis = crate::RepositoryBodyObservationBasis {
            schema_version: "epiphany.repository_body.v2".into(),
            workspace_id: identity.workspace_id.clone(),
            swarm_id: identity.swarm_id.clone(),
            runtime_id: runtime_id.clone(),
            scope: ".".into(),
            body_binding_sha256: body_digest(&format!("binding-{name}")),
            observation_id: format!("observation-{name}"),
            generation: 1,
            manifest_root_sha256: body_digest(&format!("manifest-{name}")),
            scan_started_at: "2027-01-15T00:00:00Z".into(),
            scan_finished_at: "2027-01-15T00:00:01Z".into(),
        };
        Ok(RepositoryFixture {
            identity,
            signer,
            trust,
            body_basis,
            runtime_id,
            runtime_incarnation_id,
        })
    }

    fn digest(seed: &str) -> String {
        sha256(seed.as_bytes())
    }

    fn body_digest(seed: &str) -> String {
        digest(seed).trim_start_matches("sha256-").to_owned()
    }

    fn exact_schema_offer(
        provider: &RepositoryFixture,
        surface_id: Uuid,
        contract_id: &str,
        schema_id: &str,
    ) -> AtlasPublicationPayload {
        AtlasPublicationPayload::SurfaceOffer(AtlasSurfaceOffer {
            schema_version: ATLAS_SURFACE_OFFER_SCHEMA.into(),
            provider: provider.identity.clone(),
            surface_id,
            contract: AtlasContractDescriptor::ExactSchema {
                contract_id: contract_id.into(),
                schema_id: schema_id.into(),
            },
            lifecycle: AtlasOfferLifecycle::Active,
            label: "Exact schema offer".into(),
            body_evidence: vec![AtlasBodyEvidenceRef {
                path: "Cargo.toml".into(),
                raw_sha256: "0".repeat(64),
            }],
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn exact_schema_claim(
        consumer: &RepositoryFixture,
        claim_id: Uuid,
        provider: &RepositoryFixture,
        surface_id: Uuid,
        contract_id: &str,
        schema_id: &str,
        impact_scope: AtlasImpactScope,
    ) -> AtlasPublicationPayload {
        AtlasPublicationPayload::DependencyClaim(AtlasDependencyClaim {
            schema_version: ATLAS_DEPENDENCY_CLAIM_SCHEMA.into(),
            consumer: consumer.identity.clone(),
            claim_id,
            target: AtlasDependencyTarget::Exact {
                provider: provider.identity.clone(),
                surface_id,
                requirement: AtlasContractRequirement::ExactSchema {
                    contract_id: contract_id.into(),
                    schema_id: schema_id.into(),
                },
            },
            entanglement_kind: AtlasEntanglementKind::SchemaProtocol,
            failure_semantics: AtlasFailureSemantics::HumanDecision,
            impact_scope,
            lifecycle: AtlasClaimLifecycle::Active,
            label: "Exact schema dependency".into(),
            body_evidence: vec![AtlasBodyEvidenceRef {
                path: "Cargo.toml".into(),
                raw_sha256: "0".repeat(64),
            }],
        })
    }

    fn mind_publication(
        repository: &RepositoryFixture,
        sequence: u64,
        payload: AtlasPublicationPayload,
    ) -> Result<AtlasPublicationEnvelope> {
        let source_payload = super::super::identity::atlas_source_payload_msgpack(&payload)?;
        let source = AtlasMindSourceVersion {
            store_id: "epiphany-mind".into(),
            document_type: payload.schema().into(),
            document_key: payload.key(),
            schema_id: Some(payload.schema().into()),
            payload_sha256: sha256(&source_payload),
        };
        super::super::identity::sign_atlas_publication(
            &repository.signer,
            repository.identity.clone(),
            sequence,
            repository.runtime_id.clone(),
            repository.runtime_incarnation_id.clone(),
            repository.body_basis.clone(),
            format!(
                "cultmesh://gamecult-local/atlas/{}",
                repository.identity.workspace_id
            ),
            Some(source.clone()),
            Some(AtlasMindCommitBinding {
                receipt_id: format!("mind-receipt-{sequence}"),
                receipt_sha256: digest(&format!(
                    "{}-mind-receipt-{sequence}",
                    repository.identity.workspace_id
                )),
                invariant_owner: "Mind".into(),
                source,
            }),
            NOW - 100,
            payload,
        )
    }

    fn status_publication(
        repository: &RepositoryFixture,
        documents: &[AtlasPublicationEnvelope],
    ) -> Result<AtlasPublicationEnvelope> {
        let mut watermarks = documents
            .iter()
            .map(|publication| AtlasPublicationWatermark {
                source_schema: publication.statement.payload.schema().into(),
                source_key: publication.statement.payload.key(),
                source_payload_sha256: publication
                    .statement
                    .canonical_payload_msgpack_sha256
                    .clone(),
                publication_sequence: publication.statement.publication_sequence,
            })
            .collect::<Vec<_>>();
        watermarks.sort_by(|left, right| {
            (&left.source_schema, &left.source_key).cmp(&(&right.source_schema, &right.source_key))
        });
        super::super::identity::sign_atlas_publication(
            &repository.signer,
            repository.identity.clone(),
            100,
            repository.runtime_id.clone(),
            repository.runtime_incarnation_id.clone(),
            repository.body_basis.clone(),
            format!(
                "cultmesh://gamecult-local/atlas/{}",
                repository.identity.workspace_id
            ),
            None,
            None,
            NOW - 100,
            AtlasPublicationPayload::PublisherStatus(AtlasPublisherStatus {
                publisher: repository.identity.clone(),
                runtime_id: repository.runtime_id.clone(),
                runtime_incarnation_id: repository.runtime_incarnation_id.clone(),
                heartbeat_sequence: 1,
                heartbeat_at_unix_ms: NOW - 100,
                state: AtlasPublisherState::Serving,
                watermarks,
            }),
        )
    }

    fn schema_chain(temp: &TempDir) -> Result<SchemaChainFixture> {
        let epiphany = repository(temp, "epiphany", 1)?;
        let odin = repository(temp, "odin", 2)?;
        let eve = repository(temp, "eve", 3)?;
        let epiphany_surface = Uuid::from_u128(11);
        let odin_surface = Uuid::from_u128(12);

        let epiphany_offer = mind_publication(
            &epiphany,
            1,
            exact_schema_offer(
                &epiphany,
                epiphany_surface,
                "epiphany.repo-model",
                "epiphany.model.repo_model.v0",
            ),
        )?;
        let odin_offer = mind_publication(
            &odin,
            1,
            exact_schema_offer(
                &odin,
                odin_surface,
                "odin.atlas-projection",
                "gamecult.model.entanglement_projection.v0",
            ),
        )?;
        let odin_claim = mind_publication(
            &odin,
            2,
            exact_schema_claim(
                &odin,
                Uuid::from_u128(21),
                &epiphany,
                epiphany_surface,
                "epiphany.repo-model",
                "epiphany.model.repo_model.v0",
                AtlasImpactScope::WholeRepository,
            ),
        )?;
        let eve_claim = mind_publication(
            &eve,
            1,
            exact_schema_claim(
                &eve,
                Uuid::from_u128(22),
                &odin,
                odin_surface,
                "odin.atlas-projection",
                "gamecult.model.entanglement_projection.v0",
                AtlasImpactScope::WholeRepository,
            ),
        )?;
        let epiphany_status = status_publication(&epiphany, &[epiphany_offer.clone()])?;
        let odin_status = status_publication(&odin, &[odin_offer.clone(), odin_claim.clone()])?;
        let eve_status = status_publication(&eve, &[eve_claim.clone()])?;

        Ok(SchemaChainFixture {
            epiphany: epiphany.identity,
            odin: odin.identity,
            eve: eve.identity,
            epiphany_surface,
            publications: vec![
                epiphany_offer,
                odin_offer,
                odin_claim,
                eve_claim,
                epiphany_status,
                odin_status,
                eve_status,
            ],
            trust: vec![epiphany.trust, odin.trust, eve.trust],
        })
    }

    fn project_fixture(
        fixture: &SchemaChainFixture,
        now_unix_ms: u64,
    ) -> Result<AtlasEntanglementProjection> {
        project_atlas(
            &fixture.publications,
            &fixture.trust,
            &AtlasProjectionAudience {
                audience_id: "gamecult-local-tests".into(),
                gamecult_local: true,
            },
            &AtlasFreshnessPolicy {
                publisher_status_maximum_age_ms: MAXIMUM_STATUS_AGE_MS,
                maximum_future_skew_ms: 100,
            },
            now_unix_ms,
        )
    }

    fn dependency_edge(
        claim_id: u128,
        consumer: &AtlasRepositoryIdentity,
        provider: &AtlasRepositoryIdentity,
        provider_surface_id: Uuid,
        kind: AtlasEntanglementKind,
        impact_scope: AtlasImpactScope,
    ) -> DependencyEdge {
        DependencyEdge {
            claim_id: Uuid::from_u128(claim_id),
            consumer: consumer.clone(),
            provider: provider.clone(),
            provider_surface_id,
            kind,
            impact_scope,
        }
    }

    fn blast_radius<'a>(
        radii: &'a [AtlasBlastRadius],
        source: &AtlasRepositoryIdentity,
        source_surface_id: Uuid,
    ) -> &'a AtlasBlastRadius {
        radii
            .iter()
            .find(|radius| {
                radius.source == *source && radius.source_surface_id == source_surface_id
            })
            .expect("source surface blast radius")
    }

    #[test]
    fn exact_epiphany_odin_eve_schema_chain_propagates_by_surface() -> Result<()> {
        let temp = TempDir::new()?;
        let fixture = schema_chain(&temp)?;
        let projection = project_fixture(&fixture, NOW)?;
        let radius = blast_radius(
            &projection.blast_radii,
            &fixture.epiphany,
            fixture.epiphany_surface,
        );

        assert_eq!(
            radius.affected,
            vec![
                AtlasAffectedRepository {
                    repository: fixture.eve,
                    minimum_hops: 2,
                },
                AtlasAffectedRepository {
                    repository: fixture.odin,
                    minimum_hops: 1,
                },
            ]
        );
        assert!(projection.entanglements.iter().all(|edge| {
            edge.compatibility == AtlasCompatibility::Exact
                && edge.entanglement_kind == AtlasEntanglementKind::SchemaProtocol
        }));
        Ok(())
    }

    #[test]
    fn local_surface_scope_excludes_unrelated_local_offer() -> Result<()> {
        let epiphany = AtlasRepositoryIdentity::new("gamecult", "workspace-epiphany")?;
        let odin = AtlasRepositoryIdentity::new("gamecult", "workspace-odin")?;
        let eve = AtlasRepositoryIdentity::new("gamecult", "workspace-eve")?;
        let muninn = AtlasRepositoryIdentity::new("gamecult", "workspace-muninn")?;
        let epiphany_surface = Uuid::from_u128(31);
        let odin_atlas_surface = Uuid::from_u128(32);
        let odin_unrelated_surface = Uuid::from_u128(33);
        let edges = vec![
            dependency_edge(
                41,
                &odin,
                &epiphany,
                epiphany_surface,
                AtlasEntanglementKind::SchemaProtocol,
                AtlasImpactScope::LocalSurfaces {
                    surface_ids: vec![odin_atlas_surface],
                },
            ),
            dependency_edge(
                42,
                &eve,
                &odin,
                odin_unrelated_surface,
                AtlasEntanglementKind::SchemaProtocol,
                AtlasImpactScope::WholeRepository,
            ),
            dependency_edge(
                43,
                &muninn,
                &odin,
                odin_atlas_surface,
                AtlasEntanglementKind::SchemaProtocol,
                AtlasImpactScope::WholeRepository,
            ),
        ];

        let radii = derive_blast_radii(&edges);
        let radius = blast_radius(&radii, &epiphany, epiphany_surface);
        assert!(radius.affected.contains(&AtlasAffectedRepository {
            repository: odin,
            minimum_hops: 1,
        }));
        assert!(radius.affected.contains(&AtlasAffectedRepository {
            repository: muninn,
            minimum_hops: 2,
        }));
        assert!(
            !radius
                .affected
                .iter()
                .any(|affected| affected.repository == eve)
        );
        Ok(())
    }

    #[test]
    fn whole_repository_scope_reaches_every_local_offer() -> Result<()> {
        let epiphany = AtlasRepositoryIdentity::new("gamecult", "workspace-epiphany")?;
        let odin = AtlasRepositoryIdentity::new("gamecult", "workspace-odin")?;
        let eve = AtlasRepositoryIdentity::new("gamecult", "workspace-eve")?;
        let muninn = AtlasRepositoryIdentity::new("gamecult", "workspace-muninn")?;
        let epiphany_surface = Uuid::from_u128(51);
        let odin_atlas_surface = Uuid::from_u128(52);
        let odin_other_surface = Uuid::from_u128(53);
        let edges = vec![
            dependency_edge(
                61,
                &odin,
                &epiphany,
                epiphany_surface,
                AtlasEntanglementKind::SchemaProtocol,
                AtlasImpactScope::WholeRepository,
            ),
            dependency_edge(
                62,
                &eve,
                &odin,
                odin_atlas_surface,
                AtlasEntanglementKind::SchemaProtocol,
                AtlasImpactScope::WholeRepository,
            ),
            dependency_edge(
                63,
                &muninn,
                &odin,
                odin_other_surface,
                AtlasEntanglementKind::SchemaProtocol,
                AtlasImpactScope::WholeRepository,
            ),
        ];

        let radius = blast_radius(&derive_blast_radii(&edges), &epiphany, epiphany_surface).clone();
        assert!(radius.affected.contains(&AtlasAffectedRepository {
            repository: odin,
            minimum_hops: 1,
        }));
        assert!(radius.affected.contains(&AtlasAffectedRepository {
            repository: eve,
            minimum_hops: 2,
        }));
        assert!(radius.affected.contains(&AtlasAffectedRepository {
            repository: muninn,
            minimum_hops: 2,
        }));
        Ok(())
    }

    #[test]
    fn review_cycle_edges_do_not_autonomously_propagate() -> Result<()> {
        let a = AtlasRepositoryIdentity::new("gamecult", "workspace-a")?;
        let b = AtlasRepositoryIdentity::new("gamecult", "workspace-b")?;
        for (ordinal, kind) in [
            AtlasEntanglementKind::Runtime,
            AtlasEntanglementKind::DataState,
            AtlasEntanglementKind::SchemaProtocol,
        ]
        .into_iter()
        .enumerate()
        {
            let edges = vec![
                dependency_edge(
                    100 + ordinal as u128 * 2,
                    &a,
                    &b,
                    Uuid::from_u128(70 + ordinal as u128 * 2),
                    kind,
                    AtlasImpactScope::WholeRepository,
                ),
                dependency_edge(
                    101 + ordinal as u128 * 2,
                    &b,
                    &a,
                    Uuid::from_u128(71 + ordinal as u128 * 2),
                    kind,
                    AtlasImpactScope::WholeRepository,
                ),
            ];
            assert_eq!(
                classify_cycles(&edges)[0].classification,
                AtlasCycleClass::ReviewRequired
            );
            assert!(derive_blast_radii(&edges).is_empty());
        }
        Ok(())
    }

    #[test]
    fn partition_staleness_retains_blast_radius_without_claiming_current_knowledge() -> Result<()> {
        let temp = TempDir::new()?;
        let fixture = schema_chain(&temp)?;
        let current = project_fixture(&fixture, NOW)?;
        let stale = project_fixture(&fixture, NOW + MAXIMUM_STATUS_AGE_MS + 1)?;

        assert_eq!(stale.blast_radii, current.blast_radii);
        assert!(
            stale
                .publisher_status
                .iter()
                .all(|status| { status.freshness == AtlasPublicationFreshness::LastKnownStale })
        );
        assert!(stale.entanglements.iter().all(|edge| {
            edge.claim_freshness == AtlasPublicationFreshness::LastKnownStale
                && edge.offer_freshness == Some(AtlasPublicationFreshness::LastKnownStale)
        }));
        assert!(
            stale
                .publisher_status
                .iter()
                .all(|status| { status.freshness != AtlasPublicationFreshness::Current })
        );
        Ok(())
    }
}
