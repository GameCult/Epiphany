use super::*;
use anyhow::Result;
use cultnet_rs::{ServiceIdentitySigner, enroll_service_identity_at};
use semver::{Version, VersionReq};
use std::collections::BTreeSet;
use tempfile::TempDir;
use uuid::Uuid;

const NOW: u64 = 1_800_000_000_000;

struct Repo {
    identity: AtlasRepositoryIdentity,
    signer: ServiceIdentitySigner<AtlasRepositorySigningIdentity>,
    trust: AtlasPublisherTrustBinding,
    body_basis: crate::RepositoryBodyObservationBasis,
    runtime_id: String,
    runtime_incarnation_id: String,
}

fn digest(seed: &str) -> String {
    super::identity::sha256(seed.as_bytes())
}

fn body_digest(seed: &str) -> String {
    digest(seed).trim_start_matches("sha256-").to_owned()
}

fn repo(temp: &TempDir, ordinal: u128, name: &str) -> Result<Repo> {
    let identity = AtlasRepositoryIdentity::new("gamecult", format!("workspace-{name}"))?;
    let signer = enroll_service_identity_at::<AtlasRepositorySigningIdentity>(
        &temp.path().join(format!("{name}.cc")),
    )?;
    let runtime_id = format!("runtime-{name}");
    let runtime_incarnation_id = format!("runtime-incarnation-{ordinal}");
    let trust = AtlasPublisherTrustBinding {
        schema_version: ATLAS_TRUST_BINDING_SCHEMA.into(),
        publisher: identity.clone(),
        signer_identity_id: signer.entry().identity_id.clone(),
        trusted_from_unix_ms: NOW - 10_000,
        expires_at_unix_ms: Some(NOW + 100_000),
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
    Ok(Repo {
        identity,
        signer,
        trust,
        body_basis,
        runtime_id,
        runtime_incarnation_id,
    })
}

fn semver_descriptor(contract_id: &str, version: &str) -> AtlasContractDescriptor {
    AtlasContractDescriptor::Semver {
        contract_id: contract_id.into(),
        version: Version::parse(version).unwrap(),
    }
}

fn semver_requirement(contract_id: &str, requirement: &str) -> AtlasContractRequirement {
    AtlasContractRequirement::Semver {
        contract_id: contract_id.into(),
        requirement: VersionReq::parse(requirement).unwrap(),
    }
}

fn offer_payload(provider: &Repo, surface_id: Uuid, version: &str) -> AtlasPublicationPayload {
    AtlasPublicationPayload::SurfaceOffer(AtlasSurfaceOffer {
        schema_version: ATLAS_SURFACE_OFFER_SCHEMA.into(),
        provider: provider.identity.clone(),
        surface_id,
        contract: semver_descriptor("contract:api", version),
        lifecycle: AtlasOfferLifecycle::Active,
    })
}

fn claim_payload(
    consumer: &Repo,
    claim_id: Uuid,
    provider: &Repo,
    surface_id: Uuid,
    requirement: &str,
    kind: AtlasEntanglementKind,
) -> AtlasPublicationPayload {
    AtlasPublicationPayload::DependencyClaim(AtlasDependencyClaim {
        schema_version: ATLAS_DEPENDENCY_CLAIM_SCHEMA.into(),
        consumer: consumer.identity.clone(),
        claim_id,
        target: AtlasDependencyTarget::Exact {
            provider: provider.identity.clone(),
            surface_id,
            requirement: semver_requirement("contract:api", requirement),
        },
        entanglement_kind: kind,
        failure_semantics: if matches!(
            kind,
            AtlasEntanglementKind::Build
                | AtlasEntanglementKind::Deployment
                | AtlasEntanglementKind::InfrastructureControl
        ) {
            AtlasFailureSemantics::FailClosed
        } else {
            AtlasFailureSemantics::HumanDecision
        },
        impact_scope: AtlasImpactScope::WholeRepository,
        lifecycle: AtlasClaimLifecycle::Active,
    })
}

fn mind_publication(
    repo: &Repo,
    sequence: u64,
    payload: AtlasPublicationPayload,
) -> Result<AtlasPublicationEnvelope> {
    let source_payload = atlas_source_payload_msgpack(&payload)?;
    let source = AtlasMindSourceVersion {
        store_id: "epiphany-mind".into(),
        document_type: payload.schema().into(),
        document_key: payload.key(),
        schema_id: Some(payload.schema().into()),
        payload_sha256: super::identity::sha256(&source_payload),
    };
    sign_atlas_publication(
        &repo.signer,
        repo.identity.clone(),
        sequence,
        repo.runtime_id.clone(),
        repo.runtime_incarnation_id.clone(),
        repo.body_basis.clone(),
        format!(
            "cultmesh://gamecult-local/atlas/{}",
            repo.identity.workspace_id
        ),
        Some(source.clone()),
        Some(AtlasMindCommitBinding {
            receipt_id: format!("mind-receipt-{sequence}"),
            receipt_sha256: digest(&format!("mind-receipt-{sequence}")),
            invariant_owner: "Mind".into(),
            source,
        }),
        NOW - 100,
        payload,
    )
}

fn status_publication(
    repo: &Repo,
    publication_sequence: u64,
    heartbeat_sequence: u64,
    heartbeat_at: u64,
    state: AtlasPublisherState,
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
    sign_atlas_publication(
        &repo.signer,
        repo.identity.clone(),
        publication_sequence,
        repo.runtime_id.clone(),
        repo.runtime_incarnation_id.clone(),
        repo.body_basis.clone(),
        format!(
            "cultmesh://gamecult-local/atlas/{}",
            repo.identity.workspace_id
        ),
        None,
        None,
        heartbeat_at,
        AtlasPublicationPayload::PublisherStatus(AtlasPublisherStatus {
            publisher: repo.identity.clone(),
            runtime_id: repo.runtime_id.clone(),
            runtime_incarnation_id: repo.runtime_incarnation_id.clone(),
            heartbeat_sequence,
            heartbeat_at_unix_ms: heartbeat_at,
            state,
            watermarks,
        }),
    )
}

fn project(
    publications: &[AtlasPublicationEnvelope],
    repos: &[&Repo],
) -> Result<AtlasEntanglementProjection> {
    project_atlas(
        publications,
        &repos
            .iter()
            .map(|repo| repo.trust.clone())
            .collect::<Vec<_>>(),
        &AtlasProjectionAudience {
            audience_id: "gamecult-operator".into(),
            gamecult_local: true,
        },
        &AtlasFreshnessPolicy {
            publisher_status_maximum_age_ms: 1_000,
            maximum_future_skew_ms: 10,
        },
        NOW,
    )
}

#[test]
fn three_repo_vertical_slice_wakes_direct_then_transitive_modeling() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let epiphany = repo(&temp, 1, "epiphany")?;
    let odin = repo(&temp, 2, "odin")?;
    let eve = repo(&temp, 3, "eve")?;
    let eve_surface = Uuid::from_u128(0xe1);
    let odin_catalog = Uuid::from_u128(0x0d1);
    let odin_eve_claim_id = Uuid::from_u128(0xc1);
    let epiphany_odin_claim_id = Uuid::from_u128(0xc2);

    let exact_offer = |owner: &Repo, surface_id, contract_id: &str, schema_id: &str| {
        AtlasPublicationPayload::SurfaceOffer(AtlasSurfaceOffer {
            schema_version: ATLAS_SURFACE_OFFER_SCHEMA.into(),
            provider: owner.identity.clone(),
            surface_id,
            contract: AtlasContractDescriptor::ExactSchema {
                contract_id: contract_id.into(),
                schema_id: schema_id.into(),
            },
            lifecycle: AtlasOfferLifecycle::Active,
        })
    };
    let eve_offer = mind_publication(
        &eve,
        1,
        exact_offer(
            &eve,
            eve_surface,
            "contract:eve-surface",
            "gamecult.eve.surface.v1",
        ),
    )?;
    let odin_offer = mind_publication(
        &odin,
        1,
        exact_offer(
            &odin,
            odin_catalog,
            "contract:odin-catalog",
            "cultmesh.odin.provider_catalog.v1",
        ),
    )?;
    let odin_claim_document = AtlasDependencyClaim {
        schema_version: ATLAS_DEPENDENCY_CLAIM_SCHEMA.into(),
        consumer: odin.identity.clone(),
        claim_id: odin_eve_claim_id,
        target: AtlasDependencyTarget::Exact {
            provider: eve.identity.clone(),
            surface_id: eve_surface,
            requirement: AtlasContractRequirement::ExactSchema {
                contract_id: "contract:eve-surface".into(),
                schema_id: "gamecult.eve.surface.v1".into(),
            },
        },
        entanglement_kind: AtlasEntanglementKind::SchemaProtocol,
        failure_semantics: AtlasFailureSemantics::FailClosed,
        impact_scope: AtlasImpactScope::LocalSurfaces {
            surface_ids: vec![odin_catalog],
        },
        lifecycle: AtlasClaimLifecycle::Active,
    };
    let odin_claim = mind_publication(
        &odin,
        2,
        AtlasPublicationPayload::DependencyClaim(odin_claim_document.clone()),
    )?;
    let epiphany_claim_document = AtlasDependencyClaim {
        schema_version: ATLAS_DEPENDENCY_CLAIM_SCHEMA.into(),
        consumer: epiphany.identity.clone(),
        claim_id: epiphany_odin_claim_id,
        target: AtlasDependencyTarget::Exact {
            provider: odin.identity.clone(),
            surface_id: odin_catalog,
            requirement: AtlasContractRequirement::ExactSchema {
                contract_id: "contract:odin-catalog".into(),
                schema_id: "cultmesh.odin.provider_catalog.v1".into(),
            },
        },
        entanglement_kind: AtlasEntanglementKind::SchemaProtocol,
        failure_semantics: AtlasFailureSemantics::FailClosed,
        impact_scope: AtlasImpactScope::WholeRepository,
        lifecycle: AtlasClaimLifecycle::Active,
    };
    let epiphany_claim = mind_publication(
        &epiphany,
        1,
        AtlasPublicationPayload::DependencyClaim(epiphany_claim_document.clone()),
    )?;
    let baseline = vec![
        eve_offer.clone(),
        odin_offer.clone(),
        odin_claim.clone(),
        epiphany_claim.clone(),
        status_publication(
            &eve,
            2,
            1,
            NOW,
            AtlasPublisherState::Serving,
            std::slice::from_ref(&eve_offer),
        )?,
        status_publication(
            &odin,
            3,
            1,
            NOW,
            AtlasPublisherState::Serving,
            &[odin_offer.clone(), odin_claim.clone()],
        )?,
        status_publication(
            &epiphany,
            2,
            1,
            NOW,
            AtlasPublisherState::Serving,
            std::slice::from_ref(&epiphany_claim),
        )?,
    ];
    let projection = project(&baseline, &[&epiphany, &odin, &eve])?;
    assert_eq!(projection.entanglements.len(), 2);
    assert!(
        projection
            .entanglements
            .iter()
            .all(|edge| edge.compatibility == AtlasCompatibility::Exact)
    );
    let eve_radius = projection
        .blast_radii
        .iter()
        .find(|radius| radius.source == eve.identity && radius.source_surface_id == eve_surface)
        .unwrap();
    assert!(eve_radius.affected.contains(&AtlasAffectedRepository {
        repository: odin.identity.clone(),
        minimum_hops: 1
    }));
    assert!(eve_radius.affected.contains(&AtlasAffectedRepository {
        repository: epiphany.identity.clone(),
        minimum_hops: 2
    }));

    let changed_eve_offer = mind_publication(
        &eve,
        3,
        exact_offer(
            &eve,
            eve_surface,
            "contract:eve-surface",
            "gamecult.eve.surface.v2",
        ),
    )?;
    let mut changed = baseline
        .into_iter()
        .filter(|publication| {
            !matches!(
                publication.statement.payload,
                AtlasPublicationPayload::PublisherStatus(_)
            )
        })
        .collect::<Vec<_>>();
    changed.push(changed_eve_offer.clone());
    changed.extend([
        status_publication(
            &eve,
            4,
            2,
            NOW,
            AtlasPublisherState::Serving,
            std::slice::from_ref(&changed_eve_offer),
        )?,
        status_publication(
            &odin,
            4,
            2,
            NOW,
            AtlasPublisherState::Serving,
            &[odin_offer, odin_claim],
        )?,
        status_publication(
            &epiphany,
            3,
            2,
            NOW,
            AtlasPublisherState::Serving,
            std::slice::from_ref(&epiphany_claim),
        )?,
    ]);
    let changed_projection = project(&changed, &[&epiphany, &odin, &eve])?;
    let lane_states = [
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
    ];
    let ingress_policy = AtlasImpactIngressPolicy {
        cooldown_after_completion_ms: 30_000,
    };
    let no_brake = AtlasImpactBrakeState {
        engaged: false,
        brake_id: None,
    };
    let odin_input = AtlasLocalClaimInput {
        source_payload_sha256: super::identity::sha256(&atlas_source_payload_msgpack(
            &AtlasPublicationPayload::DependencyClaim(odin_claim_document.clone()),
        )?),
        claim: odin_claim_document,
    };
    let epiphany_input = AtlasLocalClaimInput {
        source_payload_sha256: super::identity::sha256(&atlas_source_payload_msgpack(
            &AtlasPublicationPayload::DependencyClaim(epiphany_claim_document.clone()),
        )?),
        claim: epiphany_claim_document,
    };
    let odin_impacts = evaluate_local_atlas_impacts(
        &odin.identity,
        &changed_projection,
        &[odin_input],
        &BTreeSet::new(),
        &lane_states,
        &no_brake,
        &ingress_policy,
        NOW,
    )?;
    let epiphany_impacts = evaluate_local_atlas_impacts(
        &epiphany.identity,
        &changed_projection,
        &[epiphany_input],
        &BTreeSet::new(),
        &lane_states,
        &no_brake,
        &ingress_policy,
        NOW,
    )?;
    assert!(matches!(
        odin_impacts.proposals[0].reason,
        AtlasImpactReason::Compatibility {
            state: AtlasCompatibility::VersionMismatch
        }
    ));
    assert_eq!(odin_impacts.proposals[0].lane, AtlasImpactLane::Modeling);
    assert!(
        matches!(
            epiphany_impacts.proposals[0].reason,
            AtlasImpactReason::TransitiveBlastRadius {
                minimum_hops: 2,
                ..
            }
        ),
        "unexpected Epiphany impact: {:?}",
        epiphany_impacts.proposals
    );
    assert_eq!(
        epiphany_impacts.proposals[0].lane,
        AtlasImpactLane::Modeling
    );
    Ok(())
}

#[test]
fn withdrawal_and_retirement_preserve_each_repositories_owned_history() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let consumer = repo(&temp, 1, "consumer-lifecycle")?;
    let provider = repo(&temp, 2, "provider-lifecycle")?;
    let surface_id = Uuid::from_u128(0x51);
    let claim_id = Uuid::from_u128(0x52);
    let mut withdrawn_offer = match offer_payload(&provider, surface_id, "1.0.0") {
        AtlasPublicationPayload::SurfaceOffer(offer) => offer,
        _ => unreachable!(),
    };
    withdrawn_offer.lifecycle = AtlasOfferLifecycle::Withdrawn;
    let withdrawn_offer = mind_publication(
        &provider,
        1,
        AtlasPublicationPayload::SurfaceOffer(withdrawn_offer),
    )?;
    let active_claim = mind_publication(
        &consumer,
        1,
        claim_payload(
            &consumer,
            claim_id,
            &provider,
            surface_id,
            "=1.0.0",
            AtlasEntanglementKind::Build,
        ),
    )?;
    let withdrawn_projection = project(
        &[
            withdrawn_offer.clone(),
            active_claim.clone(),
            status_publication(
                &provider,
                2,
                1,
                NOW,
                AtlasPublisherState::Serving,
                std::slice::from_ref(&withdrawn_offer),
            )?,
            status_publication(
                &consumer,
                2,
                1,
                NOW,
                AtlasPublisherState::Serving,
                std::slice::from_ref(&active_claim),
            )?,
        ],
        &[&consumer, &provider],
    )?;
    assert_eq!(withdrawn_projection.entanglements.len(), 1);
    assert_eq!(
        withdrawn_projection.entanglements[0].compatibility,
        AtlasCompatibility::OfferWithdrawn
    );
    assert_eq!(withdrawn_projection.blast_radii.len(), 1);

    let mut retired_claim_document = match claim_payload(
        &consumer,
        claim_id,
        &provider,
        surface_id,
        "=1.0.0",
        AtlasEntanglementKind::Build,
    ) {
        AtlasPublicationPayload::DependencyClaim(claim) => claim,
        _ => unreachable!(),
    };
    retired_claim_document.lifecycle = AtlasClaimLifecycle::Retired;
    let retired_claim = mind_publication(
        &consumer,
        3,
        AtlasPublicationPayload::DependencyClaim(retired_claim_document),
    )?;
    let active_offer =
        mind_publication(&provider, 3, offer_payload(&provider, surface_id, "1.0.0"))?;
    let retired_projection = project(
        &[
            active_offer.clone(),
            active_claim,
            retired_claim.clone(),
            status_publication(
                &provider,
                4,
                2,
                NOW,
                AtlasPublisherState::Serving,
                std::slice::from_ref(&active_offer),
            )?,
            status_publication(
                &consumer,
                4,
                2,
                NOW,
                AtlasPublisherState::Serving,
                std::slice::from_ref(&retired_claim),
            )?,
        ],
        &[&consumer, &provider],
    )?;
    assert_eq!(retired_projection.entanglements.len(), 1);
    assert_eq!(
        retired_projection.entanglements[0].compatibility,
        AtlasCompatibility::ClaimRetired
    );
    assert!(retired_projection.blast_radii.is_empty());
    Ok(())
}

#[test]
fn persistent_atlas_store_keeps_events_immutable_and_heads_cas_fenced() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let owner = repo(&temp, 1, "store-owner")?;
    let store =
        AtlasCultCacheStore::new(temp.path().join("atlas-store.cc"), owner.identity.clone())?;
    assert_eq!(
        store.compare_and_swap_trust_binding(None, owner.trust.clone())?,
        AtlasStoreWriteOutcome::Applied
    );
    let publication = mind_publication(
        &owner,
        1,
        offer_payload(&owner, Uuid::from_u128(0x61), "1.0.0"),
    )?;
    assert_eq!(
        store.append_verified_publication_event(&publication, NOW)?,
        AtlasStoreWriteOutcome::Applied
    );
    assert_eq!(
        store.append_verified_publication_event(&publication, NOW)?,
        AtlasStoreWriteOutcome::AlreadyApplied
    );
    assert_eq!(store.load_publication_events()?, vec![publication]);

    let hidden_a = project_atlas(
        &[],
        &[],
        &AtlasProjectionAudience {
            audience_id: "hidden-audience".into(),
            gamecult_local: false,
        },
        &AtlasFreshnessPolicy {
            publisher_status_maximum_age_ms: 90_000,
            maximum_future_skew_ms: 5_000,
        },
        NOW,
    )?;
    assert_eq!(
        store.compare_and_swap_latest_projection(None, &hidden_a)?,
        AtlasStoreWriteOutcome::Applied
    );
    assert_eq!(
        store.compare_and_swap_latest_projection(None, &hidden_a)?,
        AtlasStoreWriteOutcome::AlreadyApplied
    );
    let hidden_b = project_atlas(
        &[],
        &[],
        &AtlasProjectionAudience {
            audience_id: "hidden-audience".into(),
            gamecult_local: false,
        },
        &AtlasFreshnessPolicy {
            publisher_status_maximum_age_ms: 90_000,
            maximum_future_skew_ms: 5_000,
        },
        NOW + 1,
    )?;
    assert_eq!(
        store.compare_and_swap_latest_projection(Some(&digest("wrong-head")), &hidden_b)?,
        AtlasStoreWriteOutcome::Conflict
    );
    assert_eq!(
        store.compare_and_swap_latest_projection(Some(&hidden_a.projection_sha256), &hidden_b,)?,
        AtlasStoreWriteOutcome::Applied
    );

    let state = AtlasImpactStateRecord {
        schema_version: ATLAS_IMPACT_STATE_RECORD_SCHEMA.into(),
        local_repository: owner.identity,
        revision: 1,
        modeling: AtlasStoredImpactLaneState {
            pending_proposal_id: None,
            last_completed_at_unix_ms: None,
        },
        soul: AtlasStoredImpactLaneState {
            pending_proposal_id: None,
            last_completed_at_unix_ms: None,
        },
        brake: AtlasStoredImpactBrakeState {
            engaged: false,
            brake_id: None,
        },
        updated_at_unix_ms: NOW,
    };
    assert_eq!(
        store.compare_and_swap_impact_update(None, &state, &[])?,
        AtlasStoreWriteOutcome::Applied
    );
    Ok(())
}

#[test]
fn contracts_are_closed_and_local_authority_is_explicit() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let consumer = repo(&temp, 1, "consumer")?;
    let provider = repo(&temp, 2, "provider")?;
    assert_eq!(
        consumer.identity.repository_uri,
        "gamecult://swarm/gamecult/workspace/workspace-consumer"
    );
    assert_eq!(
        <AtlasSurfaceOffer as cultcache_rs::DatabaseEntry>::TYPE,
        ATLAS_SURFACE_OFFER_SCHEMA
    );
    assert_eq!(
        <AtlasDependencyClaim as cultcache_rs::DatabaseEntry>::TYPE,
        ATLAS_DEPENDENCY_CLAIM_SCHEMA
    );
    assert_eq!(
        <AtlasDependencyVerification as cultcache_rs::DatabaseEntry>::TYPE,
        ATLAS_DEPENDENCY_VERIFICATION_SCHEMA
    );
    assert_eq!(
        <AtlasDependencyImpact as cultcache_rs::DatabaseEntry>::TYPE,
        ATLAS_DEPENDENCY_IMPACT_SCHEMA
    );
    assert_eq!(
        <AtlasPublicationEnvelope as cultcache_rs::DatabaseEntry>::TYPE,
        ATLAS_PUBLICATION_SCHEMA
    );
    assert_eq!(
        <AtlasEntanglementProjection as cultcache_rs::DatabaseEntry>::TYPE,
        ATLAS_PROJECTION_SCHEMA
    );
    let mut claim = match claim_payload(
        &consumer,
        Uuid::from_u128(1),
        &provider,
        Uuid::from_u128(2),
        "^1.2",
        AtlasEntanglementKind::Build,
    ) {
        AtlasPublicationPayload::DependencyClaim(claim) => claim,
        _ => unreachable!(),
    };
    claim.failure_semantics = AtlasFailureSemantics::Degrade;
    assert!(claim.validate().is_err());

    let offer = AtlasSurfaceOffer {
        schema_version: ATLAS_SURFACE_OFFER_SCHEMA.into(),
        provider: provider.identity,
        surface_id: Uuid::from_u128(2),
        contract: AtlasContractDescriptor::ExactSchema {
            contract_id: "contract:api".into(),
            schema_id: "schema:api:v1".into(),
        },
        lifecycle: AtlasOfferLifecycle::Active,
    };
    assert_eq!(
        evaluate_atlas_compatibility(&claim, Some(&offer)),
        AtlasCompatibility::VersionSchemeMismatch
    );
    Ok(())
}

#[test]
fn signatures_bind_owner_body_source_mind_and_payload() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let provider = repo(&temp, 1, "provider")?;
    let foreign = repo(&temp, 2, "foreign")?;
    let publication = mind_publication(
        &provider,
        1,
        offer_payload(&provider, Uuid::from_u128(10), "1.2.3"),
    )?;
    verify_atlas_publication(&provider.trust, &publication, NOW)?;
    assert!(verify_atlas_publication(&foreign.trust, &publication, NOW).is_err());

    let mut tampered = publication.clone();
    let AtlasPublicationPayload::SurfaceOffer(offer) = &mut tampered.statement.payload else {
        unreachable!()
    };
    offer.contract = semver_descriptor("contract:api", "9.9.9");
    assert!(verify_atlas_publication(&provider.trust, &tampered, NOW).is_err());

    let status = AtlasPublicationPayload::PublisherStatus(AtlasPublisherStatus {
        publisher: provider.identity.clone(),
        runtime_id: provider.runtime_id.clone(),
        runtime_incarnation_id: provider.runtime_incarnation_id.clone(),
        heartbeat_sequence: 1,
        heartbeat_at_unix_ms: NOW,
        state: AtlasPublisherState::Serving,
        watermarks: Vec::new(),
    });
    assert!(
        sign_atlas_publication(
            &provider.signer,
            provider.identity.clone(),
            2,
            provider.runtime_id.clone(),
            provider.runtime_incarnation_id.clone(),
            provider.body_basis.clone(),
            "cultmesh://gamecult-local/atlas/provider".into(),
            Some(AtlasMindSourceVersion {
                store_id: "mind".into(),
                document_type: ATLAS_PUBLICATION_SCHEMA.into(),
                document_key: "status".into(),
                schema_id: Some(ATLAS_PUBLICATION_SCHEMA.into()),
                payload_sha256: digest("status"),
            }),
            None,
            NOW,
            status,
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn visibility_precedes_signature_validation_and_join() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let provider = repo(&temp, 1, "provider")?;
    let mut counterfeit = mind_publication(
        &provider,
        1,
        offer_payload(&provider, Uuid::from_u128(10), "1.0.0"),
    )?;
    counterfeit.signature.clear();
    let projection = project_atlas(
        &[counterfeit],
        &[],
        &AtlasProjectionAudience {
            audience_id: "outside".into(),
            gamecult_local: false,
        },
        &AtlasFreshnessPolicy {
            publisher_status_maximum_age_ms: 1_000,
            maximum_future_skew_ms: 10,
        },
        NOW,
    )?;
    assert!(projection.source_publication_ids.is_empty());
    assert!(projection.entanglements.is_empty());
    Ok(())
}

#[test]
fn deterministic_join_uses_exact_status_watermarks_and_exact_verification() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let provider = repo(&temp, 1, "provider")?;
    let consumer = repo(&temp, 2, "consumer")?;
    let surface_id = Uuid::from_u128(10);
    let claim_id = Uuid::from_u128(20);
    let offer = mind_publication(&provider, 1, offer_payload(&provider, surface_id, "1.2.3"))?;
    let claim = mind_publication(
        &consumer,
        1,
        claim_payload(
            &consumer,
            claim_id,
            &provider,
            surface_id,
            "^1.2",
            AtlasEntanglementKind::SchemaProtocol,
        ),
    )?;
    let verification = mind_publication(
        &consumer,
        2,
        AtlasPublicationPayload::DependencyVerification(AtlasDependencyVerification {
            schema_version: ATLAS_DEPENDENCY_VERIFICATION_SCHEMA.into(),
            consumer: consumer.identity.clone(),
            claim_id,
            claim_publication_id: claim.statement.publication_id.clone(),
            offer_publication_id: offer.statement.publication_id.clone(),
            exact_contract: semver_descriptor("contract:api", "1.2.3"),
            verdict: AtlasVerificationVerdict::Passed,
            evidence_sha256: digest("verified"),
        }),
    )?;
    let provider_status = status_publication(
        &provider,
        100,
        1,
        NOW,
        AtlasPublisherState::Serving,
        std::slice::from_ref(&offer),
    )?;
    let consumer_status = status_publication(
        &consumer,
        100,
        1,
        NOW,
        AtlasPublisherState::Serving,
        &[claim.clone(), verification.clone()],
    )?;
    let publications = vec![
        offer.clone(),
        claim.clone(),
        verification.clone(),
        provider_status.clone(),
        consumer_status.clone(),
    ];
    let first = project(&publications, &[&provider, &consumer])?;
    assert_eq!(first.entanglements.len(), 1);
    assert_eq!(
        first.entanglements[0].compatibility,
        AtlasCompatibility::Compatible
    );
    assert_eq!(
        first.entanglements[0].verification,
        AtlasVerificationState::Passed
    );
    assert_eq!(first.source_publication_ids.len(), 5);
    assert_eq!(atlas_projection_digest(&first)?, first.projection_sha256);

    let mut reversed = publications;
    reversed.reverse();
    assert_eq!(first, project(&reversed, &[&provider, &consumer])?);

    let mut wrong = verification;
    let old_verification_id = wrong.statement.publication_id.clone();
    let AtlasPublicationPayload::DependencyVerification(value) = &mut wrong.statement.payload
    else {
        unreachable!()
    };
    value.exact_contract = semver_descriptor("contract:api", "1.2.2");
    // A local object mutation without a new Mind source and signature is dead
    // cargo, not an alternate Atlas truth.
    assert!(verify_atlas_publication(&consumer.trust, &wrong, NOW).is_err());
    let mut invalid = reversed;
    invalid.retain(|publication| publication.statement.publication_id != old_verification_id);
    invalid.push(wrong);
    assert!(project(&invalid, &[&provider, &consumer]).is_err());
    Ok(())
}

#[test]
fn stale_status_preserves_last_known_edges_without_calling_them_current() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let provider = repo(&temp, 1, "provider")?;
    let consumer = repo(&temp, 2, "consumer")?;
    let surface_id = Uuid::from_u128(10);
    let claim_id = Uuid::from_u128(20);
    let offer = mind_publication(&provider, 1, offer_payload(&provider, surface_id, "1.0.0"))?;
    let claim = mind_publication(
        &consumer,
        1,
        claim_payload(
            &consumer,
            claim_id,
            &provider,
            surface_id,
            "=1.0.0",
            AtlasEntanglementKind::Runtime,
        ),
    )?;
    let statuses = vec![
        status_publication(
            &provider,
            100,
            1,
            NOW - 5_000,
            AtlasPublisherState::Serving,
            std::slice::from_ref(&offer),
        )?,
        status_publication(
            &consumer,
            100,
            1,
            NOW - 5_000,
            AtlasPublisherState::Serving,
            std::slice::from_ref(&claim),
        )?,
    ];
    let projection = project(
        &[offer, claim, statuses[0].clone(), statuses[1].clone()],
        &[&provider, &consumer],
    )?;
    assert_eq!(
        projection.entanglements[0].claim_freshness,
        AtlasPublicationFreshness::LastKnownStale
    );
    assert_eq!(
        projection.entanglements[0].offer_freshness,
        Some(AtlasPublicationFreshness::LastKnownStale)
    );
    assert_eq!(
        projection.entanglements[0].compatibility,
        AtlasCompatibility::Exact
    );
    assert_eq!(projection.blast_radii.len(), 1);
    Ok(())
}

#[test]
fn missing_watermarked_document_fails_closed() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let provider = repo(&temp, 1, "provider")?;
    let offer = mind_publication(
        &provider,
        1,
        offer_payload(&provider, Uuid::from_u128(10), "1.0.0"),
    )?;
    let status = status_publication(
        &provider,
        100,
        1,
        NOW,
        AtlasPublisherState::Serving,
        &[offer],
    )?;
    assert!(project(&[status], &[&provider]).is_err());
    Ok(())
}

#[test]
fn signed_status_cannot_roll_a_document_watermark_back() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let provider = repo(&temp, 1, "provider")?;
    let surface_id = Uuid::from_u128(10);
    let old = mind_publication(&provider, 1, offer_payload(&provider, surface_id, "1.0.0"))?;
    let new = mind_publication(&provider, 2, offer_payload(&provider, surface_id, "2.0.0"))?;
    let advanced = status_publication(
        &provider,
        100,
        1,
        NOW - 100,
        AtlasPublisherState::Serving,
        std::slice::from_ref(&new),
    )?;
    let regressed = status_publication(
        &provider,
        101,
        2,
        NOW,
        AtlasPublisherState::Serving,
        std::slice::from_ref(&old),
    )?;
    assert!(
        project(&[old, new, advanced, regressed], &[&provider]).is_err(),
        "a newer signed status must not make an older source publication current"
    );
    Ok(())
}

#[test]
fn cycle_classes_and_blast_radius_respect_review_boundaries() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let a = repo(&temp, 1, "a")?;
    let b = repo(&temp, 2, "b")?;
    let c = repo(&temp, 3, "c")?;
    let surface_a = Uuid::from_u128(101);
    let surface_b = Uuid::from_u128(102);
    let surface_c = Uuid::from_u128(103);
    let offer_a = mind_publication(&a, 1, offer_payload(&a, surface_a, "1.0.0"))?;
    let offer_b = mind_publication(&b, 1, offer_payload(&b, surface_b, "1.0.0"))?;
    let offer_c = mind_publication(&c, 1, offer_payload(&c, surface_c, "1.0.0"))?;
    let claim_ba = mind_publication(
        &b,
        2,
        claim_payload(
            &b,
            Uuid::from_u128(201),
            &a,
            surface_a,
            "=1.0.0",
            AtlasEntanglementKind::Build,
        ),
    )?;
    let claim_cb = mind_publication(
        &c,
        2,
        claim_payload(
            &c,
            Uuid::from_u128(202),
            &b,
            surface_b,
            "=1.0.0",
            AtlasEntanglementKind::Build,
        ),
    )?;
    let claim_ac = mind_publication(
        &a,
        2,
        claim_payload(
            &a,
            Uuid::from_u128(203),
            &c,
            surface_c,
            "=1.0.0",
            AtlasEntanglementKind::Runtime,
        ),
    )?;
    let docs_a = vec![offer_a.clone(), claim_ac.clone()];
    let docs_b = vec![offer_b.clone(), claim_ba.clone()];
    let docs_c = vec![offer_c.clone(), claim_cb.clone()];
    let mut publications = [docs_a.clone(), docs_b.clone(), docs_c.clone()].concat();
    publications.extend([
        status_publication(&a, 100, 1, NOW, AtlasPublisherState::Serving, &docs_a)?,
        status_publication(&b, 100, 1, NOW, AtlasPublisherState::Serving, &docs_b)?,
        status_publication(&c, 100, 1, NOW, AtlasPublisherState::Serving, &docs_c)?,
    ]);
    let projection = project(&publications, &[&a, &b, &c])?;
    assert_eq!(projection.cycles.len(), 1);
    assert_eq!(
        projection.cycles[0].classification,
        AtlasCycleClass::ForbiddenBuild
    );
    let radius = projection
        .blast_radii
        .iter()
        .find(|radius| radius.source == a.identity)
        .expect("A has a fail-closed transitive blast radius");
    assert!(radius.affected.contains(&AtlasAffectedRepository {
        repository: b.identity.clone(),
        minimum_hops: 1,
    }));
    assert!(radius.affected.contains(&AtlasAffectedRepository {
        repository: c.identity.clone(),
        minimum_hops: 2,
    }));
    // The runtime C -> A edge closes the visible cycle, but never propagates
    // autonomously beyond its review boundary.
    assert!(!projection.blast_radii.iter().any(|radius| {
        radius.source == c.identity
            && radius
                .affected
                .iter()
                .any(|affected| affected.repository == a.identity)
    }));
    Ok(())
}

#[test]
fn pure_lore_cycle_is_informational() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let a = repo(&temp, 1, "lore-a")?;
    let b = repo(&temp, 2, "lore-b")?;
    let surface_a = Uuid::from_u128(301);
    let surface_b = Uuid::from_u128(302);
    let offer_a = mind_publication(&a, 1, offer_payload(&a, surface_a, "1.0.0"))?;
    let offer_b = mind_publication(&b, 1, offer_payload(&b, surface_b, "1.0.0"))?;
    let claim_ab = mind_publication(
        &a,
        2,
        claim_payload(
            &a,
            Uuid::from_u128(303),
            &b,
            surface_b,
            "=1.0.0",
            AtlasEntanglementKind::LorePersona,
        ),
    )?;
    let claim_ba = mind_publication(
        &b,
        2,
        claim_payload(
            &b,
            Uuid::from_u128(304),
            &a,
            surface_a,
            "=1.0.0",
            AtlasEntanglementKind::LorePersona,
        ),
    )?;
    let docs_a = vec![offer_a.clone(), claim_ab.clone()];
    let docs_b = vec![offer_b.clone(), claim_ba.clone()];
    let publications = vec![
        offer_a,
        offer_b,
        claim_ab,
        claim_ba,
        status_publication(&a, 100, 1, NOW, AtlasPublisherState::Serving, &docs_a)?,
        status_publication(&b, 100, 1, NOW, AtlasPublisherState::Serving, &docs_b)?,
    ];
    let projection = project(&publications, &[&a, &b])?;
    assert_eq!(
        projection.cycles[0].classification,
        AtlasCycleClass::Informational
    );
    assert_eq!(projection.blast_radii.len(), 2);
    Ok(())
}
