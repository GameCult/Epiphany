use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use super::contracts::*;
use super::identity::{atlas_source_payload_msgpack, sha256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasLocalClaimInput {
    pub claim: AtlasDependencyClaim,
    pub source_payload_sha256: String,
}

impl AtlasLocalClaimInput {
    fn validate(&self, local_repository: &AtlasRepositoryIdentity) -> Result<()> {
        self.claim.validate()?;
        validate_sha256(
            &self.source_payload_sha256,
            "Atlas impact ingress local claim source digest",
        )?;
        let source_digest = sha256(&atlas_source_payload_msgpack(
            &AtlasPublicationPayload::DependencyClaim(self.claim.clone()),
        )?);
        if self.claim.consumer != *local_repository || source_digest != self.source_payload_sha256 {
            bail!("Atlas impact ingress claim is not the exact current local Mind document")
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "reason")]
pub enum AtlasImpactReason {
    MissingFromProjection,
    ProjectedClaimDiffers,
    ClaimNotCurrent {
        freshness: AtlasPublicationFreshness,
    },
    OfferNotCurrent {
        freshness: Option<AtlasPublicationFreshness>,
    },
    Compatibility {
        state: AtlasCompatibility,
    },
    Verification {
        state: AtlasVerificationState,
    },
    TransitiveBlastRadius {
        source_repository_uri: String,
        source_surface_id: Uuid,
        minimum_hops: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtlasImpactLane {
    Modeling,
    Soul,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtlasImpactProposalAuthority {
    LocalReviewOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasLocalImpactProposal {
    pub proposal_id: Uuid,
    pub dedupe_key: String,
    pub lane: AtlasImpactLane,
    pub reason: AtlasImpactReason,
    pub authority: AtlasImpactProposalAuthority,
    pub impact: AtlasDependencyImpact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasImpactLaneState {
    pub lane: AtlasImpactLane,
    pub pending_proposal_id: Option<Uuid>,
    pub last_completed_at_unix_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasImpactIngressPolicy {
    pub cooldown_after_completion_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasImpactBrakeState {
    pub engaged: bool,
    pub brake_id: Option<String>,
}

impl AtlasImpactBrakeState {
    fn validate(&self) -> Result<()> {
        match (self.engaged, &self.brake_id) {
            (true, Some(brake_id)) => validate_identifier(brake_id, "Atlas impact brake id"),
            (false, None) => Ok(()),
            _ => bail!("Atlas impact brake state is internally inconsistent"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "disposition")]
pub enum AtlasImpactScheduleDisposition {
    Schedule {
        lane: AtlasImpactLane,
    },
    VisibleOnly,
    Deduplicated,
    HeldByBrake {
        brake_id: String,
    },
    HeldByPendingLane {
        lane: AtlasImpactLane,
        pending_proposal_id: Uuid,
    },
    HeldByCooldown {
        lane: AtlasImpactLane,
        retry_at_unix_ms: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasImpactSchedulingDecision {
    pub proposal_id: Uuid,
    pub claim_id: Uuid,
    pub criticality: AtlasCriticality,
    pub disposition: AtlasImpactScheduleDisposition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasImpactIngressResult {
    pub proposals: Vec<AtlasLocalImpactProposal>,
    pub scheduling_decisions: Vec<AtlasImpactSchedulingDecision>,
}

#[allow(clippy::too_many_arguments)]
pub fn evaluate_local_atlas_impacts(
    local_repository: &AtlasRepositoryIdentity,
    projection: &AtlasEntanglementProjection,
    local_claims: &[AtlasLocalClaimInput],
    seen_dedupe_keys: &BTreeSet<String>,
    lane_states: &[AtlasImpactLaneState],
    brake: &AtlasImpactBrakeState,
    policy: &AtlasImpactIngressPolicy,
    now_unix_ms: u64,
) -> Result<AtlasImpactIngressResult> {
    local_repository.validate()?;
    require_schema(&projection.schema_version, ATLAS_PROJECTION_SCHEMA)?;
    validate_sha256(
        &projection.projection_sha256,
        "Atlas impact source projection digest",
    )?;
    validate_sorted_unique_sha256(
        &projection.source_publication_ids,
        "Atlas impact source projection publication ids",
        true,
    )?;
    brake.validate()?;

    let lanes = validated_lanes(lane_states)?;
    let mut claims = BTreeMap::<Uuid, &AtlasLocalClaimInput>::new();
    for claim in local_claims {
        claim.validate(local_repository)?;
        if claims.insert(claim.claim.claim_id, claim).is_some() {
            bail!("Atlas impact ingress received duplicate local claim ids")
        }
    }
    let mut projected = BTreeMap::<Uuid, &AtlasProjectedEntanglement>::new();
    for entanglement in projection
        .entanglements
        .iter()
        .filter(|entanglement| entanglement.consumer == *local_repository)
    {
        if projected
            .insert(entanglement.claim_id, entanglement)
            .is_some()
        {
            bail!("Atlas projection contains duplicate local entanglement ids")
        }
    }

    let mut candidates = Vec::new();
    for claim in claims.into_values() {
        if claim.claim.lifecycle == AtlasClaimLifecycle::Retired {
            continue;
        }
        let entanglement = projected.get(&claim.claim.claim_id).copied();
        let direct = impact_reason(&claim.claim, entanglement);
        let transitive = transitive_impact_reason(projection, &claim.claim);
        let selected = match &direct {
            Some((
                AtlasImpactReason::Verification {
                    state: AtlasVerificationState::Missing,
                },
                _,
            )) if transitive.is_some() => transitive,
            _ => direct.or(transitive),
        };
        let Some((reason, lane)) = selected else {
            continue;
        };
        let criticality = criticality(&claim.claim, &reason);
        let source_publication_ids = impact_source_publications(projection, entanglement, &reason);
        validate_sorted_unique_sha256(
            &source_publication_ids,
            "Atlas local impact source publication ids",
            true,
        )?;
        let dedupe_key = sha256(&rmp_serde::to_vec_named(&(
            "epiphany.model.dependency-impact-dedupe.v0",
            claim.claim.claim_id,
            &claim.source_payload_sha256,
            &source_publication_ids,
            &reason,
            criticality,
        ))?);
        let proposal_id = Uuid::new_v5(&Uuid::NAMESPACE_OID, dedupe_key.as_bytes());
        let impact = AtlasDependencyImpact {
            schema_version: ATLAS_DEPENDENCY_IMPACT_SCHEMA.into(),
            impact_id: proposal_id,
            consumer: local_repository.clone(),
            claim_id: claim.claim.claim_id,
            claim_source_payload_sha256: claim.source_payload_sha256.clone(),
            projection_sha256: projection.projection_sha256.clone(),
            source_publication_ids,
            criticality,
        };
        impact.validate()?;
        candidates.push(AtlasLocalImpactProposal {
            proposal_id,
            dedupe_key,
            lane,
            reason,
            authority: AtlasImpactProposalAuthority::LocalReviewOnly,
            impact,
        });
    }
    candidates.sort_by(|left, right| {
        (left.impact.criticality, left.impact.claim_id, &left.reason).cmp(&(
            right.impact.criticality,
            right.impact.claim_id,
            &right.reason,
        ))
    });

    let mut proposals = Vec::new();
    let mut scheduling_decisions = Vec::with_capacity(candidates.len());
    let mut reserved = BTreeMap::<AtlasImpactLane, Uuid>::new();
    for proposal in candidates {
        let disposition = if seen_dedupe_keys.contains(&proposal.dedupe_key) {
            AtlasImpactScheduleDisposition::Deduplicated
        } else if brake.engaged {
            AtlasImpactScheduleDisposition::HeldByBrake {
                brake_id: brake
                    .brake_id
                    .clone()
                    .expect("engaged brake validated above"),
            }
        } else if proposal.impact.criticality == AtlasCriticality::Informational {
            AtlasImpactScheduleDisposition::VisibleOnly
        } else if let Some(pending) = lanes[&proposal.lane].pending_proposal_id {
            AtlasImpactScheduleDisposition::HeldByPendingLane {
                lane: proposal.lane,
                pending_proposal_id: pending,
            }
        } else if let Some(pending) = reserved.get(&proposal.lane).copied() {
            AtlasImpactScheduleDisposition::HeldByPendingLane {
                lane: proposal.lane,
                pending_proposal_id: pending,
            }
        } else if let Some(retry_at_unix_ms) =
            cooldown_until(&lanes[&proposal.lane], policy.cooldown_after_completion_ms)?
                .filter(|retry_at| *retry_at > now_unix_ms)
        {
            AtlasImpactScheduleDisposition::HeldByCooldown {
                lane: proposal.lane,
                retry_at_unix_ms,
            }
        } else {
            reserved.insert(proposal.lane, proposal.proposal_id);
            AtlasImpactScheduleDisposition::Schedule {
                lane: proposal.lane,
            }
        };
        scheduling_decisions.push(AtlasImpactSchedulingDecision {
            proposal_id: proposal.proposal_id,
            claim_id: proposal.impact.claim_id,
            criticality: proposal.impact.criticality,
            disposition,
        });
        if !seen_dedupe_keys.contains(&proposal.dedupe_key) {
            proposals.push(proposal);
        }
    }

    Ok(AtlasImpactIngressResult {
        proposals,
        scheduling_decisions,
    })
}

fn validated_lanes(
    lane_states: &[AtlasImpactLaneState],
) -> Result<BTreeMap<AtlasImpactLane, &AtlasImpactLaneState>> {
    let mut lanes = BTreeMap::new();
    for state in lane_states {
        if lanes.insert(state.lane, state).is_some() {
            bail!("Atlas impact ingress received duplicate lane state")
        }
    }
    for lane in [AtlasImpactLane::Modeling, AtlasImpactLane::Soul] {
        if !lanes.contains_key(&lane) {
            bail!("Atlas impact ingress requires explicit Modeling and Soul lane state")
        }
    }
    Ok(lanes)
}

fn impact_reason(
    claim: &AtlasDependencyClaim,
    entanglement: Option<&AtlasProjectedEntanglement>,
) -> Option<(AtlasImpactReason, AtlasImpactLane)> {
    let Some(entanglement) = entanglement else {
        return Some((
            AtlasImpactReason::MissingFromProjection,
            AtlasImpactLane::Modeling,
        ));
    };
    if !projected_claim_matches(claim, entanglement) {
        return Some((
            AtlasImpactReason::ProjectedClaimDiffers,
            AtlasImpactLane::Modeling,
        ));
    }
    if entanglement.claim_freshness != AtlasPublicationFreshness::Current {
        return Some((
            AtlasImpactReason::ClaimNotCurrent {
                freshness: entanglement.claim_freshness,
            },
            AtlasImpactLane::Modeling,
        ));
    }
    if matches!(claim.target, AtlasDependencyTarget::Exact { .. })
        && entanglement.offer_freshness != Some(AtlasPublicationFreshness::Current)
    {
        return Some((
            AtlasImpactReason::OfferNotCurrent {
                freshness: entanglement.offer_freshness,
            },
            AtlasImpactLane::Modeling,
        ));
    }
    if !matches!(
        entanglement.compatibility,
        AtlasCompatibility::Exact | AtlasCompatibility::Compatible
    ) {
        return Some((
            AtlasImpactReason::Compatibility {
                state: entanglement.compatibility,
            },
            AtlasImpactLane::Modeling,
        ));
    }
    if entanglement.verification != AtlasVerificationState::Passed {
        return Some((
            AtlasImpactReason::Verification {
                state: entanglement.verification,
            },
            AtlasImpactLane::Soul,
        ));
    }
    None
}

fn projected_claim_matches(
    claim: &AtlasDependencyClaim,
    entanglement: &AtlasProjectedEntanglement,
) -> bool {
    if entanglement.consumer != claim.consumer
        || entanglement.claim_id != claim.claim_id
        || entanglement.entanglement_kind != claim.entanglement_kind
        || entanglement.failure_semantics != claim.failure_semantics
        || entanglement.impact_scope != claim.impact_scope
    {
        return false;
    }
    match &claim.target {
        AtlasDependencyTarget::Exact {
            provider,
            surface_id,
            ..
        } => {
            entanglement.provider.as_ref() == Some(provider)
                && entanglement.surface_id == Some(*surface_id)
        }
        AtlasDependencyTarget::Unresolved { .. } => {
            entanglement.provider.is_none() && entanglement.surface_id.is_none()
        }
    }
}

fn criticality(claim: &AtlasDependencyClaim, reason: &AtlasImpactReason) -> AtlasCriticality {
    if matches!(
        reason,
        AtlasImpactReason::ProjectedClaimDiffers
            | AtlasImpactReason::Verification {
                state: AtlasVerificationState::Failed
            }
    ) || matches!(
        claim.failure_semantics,
        AtlasFailureSemantics::FailClosed | AtlasFailureSemantics::HumanDecision
    ) {
        AtlasCriticality::Blocking
    } else if matches!(
        reason,
        AtlasImpactReason::Verification {
            state: AtlasVerificationState::Missing
        }
    ) && claim.entanglement_kind == AtlasEntanglementKind::LorePersona
    {
        AtlasCriticality::Informational
    } else {
        AtlasCriticality::Degrading
    }
}

fn transitive_impact_reason(
    projection: &AtlasEntanglementProjection,
    claim: &AtlasDependencyClaim,
) -> Option<(AtlasImpactReason, AtlasImpactLane)> {
    let AtlasDependencyTarget::Exact { provider, .. } = &claim.target else {
        return None;
    };
    projection.blast_radii.iter().find_map(|radius| {
        let local = radius
            .affected
            .iter()
            .find(|affected| affected.repository == claim.consumer && affected.minimum_hops >= 2)?;
        let provider_hops = if radius.source == *provider {
            0
        } else {
            radius
                .affected
                .iter()
                .find(|affected| affected.repository == *provider)?
                .minimum_hops
        };
        if local.minimum_hops != provider_hops.saturating_add(1)
            || !projection.entanglements.iter().any(|edge| {
                edge.provider.as_ref() == Some(&radius.source)
                    && edge.surface_id == Some(radius.source_surface_id)
                    && (edge.claim_freshness != AtlasPublicationFreshness::Current
                        || edge.offer_freshness != Some(AtlasPublicationFreshness::Current)
                        || !matches!(
                            edge.compatibility,
                            AtlasCompatibility::Exact | AtlasCompatibility::Compatible
                        )
                        || edge.verification == AtlasVerificationState::Failed)
            })
        {
            return None;
        }
        Some((
            AtlasImpactReason::TransitiveBlastRadius {
                source_repository_uri: radius.source.repository_uri.clone(),
                source_surface_id: radius.source_surface_id,
                minimum_hops: local.minimum_hops,
            },
            AtlasImpactLane::Modeling,
        ))
    })
}

fn impact_source_publications(
    projection: &AtlasEntanglementProjection,
    entanglement: Option<&AtlasProjectedEntanglement>,
    reason: &AtlasImpactReason,
) -> Vec<String> {
    let mut source_ids = if matches!(reason, AtlasImpactReason::TransitiveBlastRadius { .. }) {
        projection.source_publication_ids.clone()
    } else {
        entanglement
            .map(|entanglement| {
                let mut ids = vec![entanglement.claim_publication_id.clone()];
                ids.extend(entanglement.offer_publication_id.iter().cloned());
                ids
            })
            .unwrap_or_else(|| projection.source_publication_ids.clone())
    };
    source_ids.sort();
    source_ids.dedup();
    source_ids
}

fn cooldown_until(state: &AtlasImpactLaneState, cooldown_ms: u64) -> Result<Option<u64>> {
    state
        .last_completed_at_unix_ms
        .map(|completed| {
            completed
                .checked_add(cooldown_ms)
                .ok_or_else(|| anyhow::anyhow!("Atlas impact lane cooldown overflowed"))
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::VersionReq;

    fn digest(seed: &str) -> String {
        sha256(seed.as_bytes())
    }

    fn repository(workspace: &str) -> AtlasRepositoryIdentity {
        AtlasRepositoryIdentity::new("swarm", workspace).unwrap()
    }

    fn claim() -> AtlasDependencyClaim {
        AtlasDependencyClaim {
            schema_version: ATLAS_DEPENDENCY_CLAIM_SCHEMA.into(),
            consumer: repository("consumer"),
            claim_id: Uuid::from_u128(1),
            target: AtlasDependencyTarget::Exact {
                provider: repository("provider"),
                surface_id: Uuid::from_u128(2),
                requirement: AtlasContractRequirement::Semver {
                    contract_id: "contract.surface".into(),
                    requirement: VersionReq::parse("^1").unwrap(),
                },
            },
            entanglement_kind: AtlasEntanglementKind::Runtime,
            failure_semantics: AtlasFailureSemantics::Degrade,
            impact_scope: AtlasImpactScope::WholeRepository,
            lifecycle: AtlasClaimLifecycle::Active,
        }
    }

    fn local_claim() -> AtlasLocalClaimInput {
        let claim = claim();
        let source_payload_sha256 = sha256(
            &atlas_source_payload_msgpack(&AtlasPublicationPayload::DependencyClaim(claim.clone()))
                .unwrap(),
        );
        AtlasLocalClaimInput {
            claim,
            source_payload_sha256,
        }
    }

    fn entanglement(
        compatibility: AtlasCompatibility,
        verification: AtlasVerificationState,
    ) -> AtlasProjectedEntanglement {
        AtlasProjectedEntanglement {
            claim_id: Uuid::from_u128(1),
            consumer: repository("consumer"),
            provider: Some(repository("provider")),
            surface_id: Some(Uuid::from_u128(2)),
            entanglement_kind: AtlasEntanglementKind::Runtime,
            failure_semantics: AtlasFailureSemantics::Degrade,
            impact_scope: AtlasImpactScope::WholeRepository,
            claim_freshness: AtlasPublicationFreshness::Current,
            offer_freshness: Some(AtlasPublicationFreshness::Current),
            compatibility,
            verification,
            claim_publication_id: digest("claim-publication"),
            offer_publication_id: Some(digest("offer-publication")),
        }
    }

    fn projection(entanglement: AtlasProjectedEntanglement) -> AtlasEntanglementProjection {
        let mut source_publication_ids = vec![
            entanglement.claim_publication_id.clone(),
            entanglement.offer_publication_id.clone().unwrap(),
        ];
        source_publication_ids.sort();
        AtlasEntanglementProjection {
            schema_version: ATLAS_PROJECTION_SCHEMA.into(),
            audience_id: "local-consumer".into(),
            evaluated_at_unix_ms: 1_800_000_000_000,
            source_publication_ids,
            publisher_status: Vec::new(),
            entanglements: vec![entanglement],
            cycles: Vec::new(),
            blast_radii: Vec::new(),
            projection_sha256: digest("projection"),
        }
    }

    fn lanes() -> Vec<AtlasImpactLaneState> {
        vec![
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
        ]
    }

    fn no_brake() -> AtlasImpactBrakeState {
        AtlasImpactBrakeState {
            engaged: false,
            brake_id: None,
        }
    }

    #[test]
    fn incompatible_projection_yields_only_a_modeling_proposal() -> Result<()> {
        let result = evaluate_local_atlas_impacts(
            &repository("consumer"),
            &projection(entanglement(
                AtlasCompatibility::VersionMismatch,
                AtlasVerificationState::Missing,
            )),
            &[local_claim()],
            &BTreeSet::new(),
            &lanes(),
            &no_brake(),
            &AtlasImpactIngressPolicy {
                cooldown_after_completion_ms: 100,
            },
            1_800_000_000_000,
        )?;

        assert_eq!(result.proposals.len(), 1);
        assert_eq!(result.proposals[0].lane, AtlasImpactLane::Modeling);
        assert_eq!(
            result.proposals[0].authority,
            AtlasImpactProposalAuthority::LocalReviewOnly
        );
        assert!(matches!(
            result.scheduling_decisions[0].disposition,
            AtlasImpactScheduleDisposition::Schedule {
                lane: AtlasImpactLane::Modeling
            }
        ));
        result.proposals[0].impact.validate()?;
        Ok(())
    }

    #[test]
    fn informational_change_is_visible_without_waking_a_lane() -> Result<()> {
        let mut informational_claim = claim();
        informational_claim.entanglement_kind = AtlasEntanglementKind::LorePersona;
        let source_payload_sha256 = sha256(&atlas_source_payload_msgpack(
            &AtlasPublicationPayload::DependencyClaim(informational_claim.clone()),
        )?);
        let mut edge = entanglement(AtlasCompatibility::Exact, AtlasVerificationState::Missing);
        edge.entanglement_kind = AtlasEntanglementKind::LorePersona;
        let result = evaluate_local_atlas_impacts(
            &repository("consumer"),
            &projection(edge),
            &[AtlasLocalClaimInput {
                claim: informational_claim,
                source_payload_sha256,
            }],
            &BTreeSet::new(),
            &lanes(),
            &no_brake(),
            &AtlasImpactIngressPolicy {
                cooldown_after_completion_ms: 100,
            },
            1_800_000_000_000,
        )?;

        assert_eq!(
            result.proposals[0].impact.criticality,
            AtlasCriticality::Informational
        );
        assert_eq!(
            result.scheduling_decisions[0].disposition,
            AtlasImpactScheduleDisposition::VisibleOnly
        );
        Ok(())
    }

    #[test]
    fn transitive_blocking_blast_wakes_the_exact_downstream_claim() -> Result<()> {
        let mut local_claim = claim();
        local_claim.failure_semantics = AtlasFailureSemantics::FailClosed;
        let local_source = sha256(&atlas_source_payload_msgpack(
            &AtlasPublicationPayload::DependencyClaim(local_claim.clone()),
        )?);
        let mut local_edge =
            entanglement(AtlasCompatibility::Exact, AtlasVerificationState::Passed);
        local_edge.failure_semantics = AtlasFailureSemantics::FailClosed;
        let eve = repository("eve");
        let odin = repository("provider");
        let mut source_edge = local_edge.clone();
        source_edge.claim_id = Uuid::from_u128(9);
        source_edge.consumer = odin.clone();
        source_edge.provider = Some(eve.clone());
        source_edge.surface_id = Some(Uuid::from_u128(8));
        source_edge.compatibility = AtlasCompatibility::VersionMismatch;
        source_edge.claim_publication_id = digest("odin-eve-claim");
        source_edge.offer_publication_id = Some(digest("eve-offer"));
        let mut source_publication_ids = vec![
            local_edge.claim_publication_id.clone(),
            local_edge.offer_publication_id.clone().unwrap(),
            source_edge.claim_publication_id.clone(),
            source_edge.offer_publication_id.clone().unwrap(),
        ];
        source_publication_ids.sort();
        let projection = AtlasEntanglementProjection {
            schema_version: ATLAS_PROJECTION_SCHEMA.into(),
            audience_id: "gamecult-local".into(),
            evaluated_at_unix_ms: 1_800_000_000_000,
            source_publication_ids,
            publisher_status: Vec::new(),
            entanglements: vec![local_edge, source_edge],
            cycles: Vec::new(),
            blast_radii: vec![AtlasBlastRadius {
                source: eve,
                source_surface_id: Uuid::from_u128(8),
                affected: vec![
                    AtlasAffectedRepository {
                        repository: odin,
                        minimum_hops: 1,
                    },
                    AtlasAffectedRepository {
                        repository: repository("consumer"),
                        minimum_hops: 2,
                    },
                ],
            }],
            projection_sha256: digest("transitive-projection"),
        };
        let result = evaluate_local_atlas_impacts(
            &repository("consumer"),
            &projection,
            &[AtlasLocalClaimInput {
                claim: local_claim,
                source_payload_sha256: local_source,
            }],
            &BTreeSet::new(),
            &lanes(),
            &no_brake(),
            &AtlasImpactIngressPolicy {
                cooldown_after_completion_ms: 100,
            },
            1_800_000_000_000,
        )?;
        assert_eq!(result.proposals.len(), 1);
        assert_eq!(result.proposals[0].lane, AtlasImpactLane::Modeling);
        assert_eq!(
            result.proposals[0].impact.criticality,
            AtlasCriticality::Blocking
        );
        assert!(matches!(
            result.proposals[0].reason,
            AtlasImpactReason::TransitiveBlastRadius {
                minimum_hops: 2,
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn brake_preserves_new_proposal_but_dedupe_suppresses_repeat_pressure() -> Result<()> {
        let projection = projection(entanglement(
            AtlasCompatibility::VersionMismatch,
            AtlasVerificationState::Missing,
        ));
        let brake = AtlasImpactBrakeState {
            engaged: true,
            brake_id: Some("operator-brake".into()),
        };
        let first = evaluate_local_atlas_impacts(
            &repository("consumer"),
            &projection,
            &[local_claim()],
            &BTreeSet::new(),
            &lanes(),
            &brake,
            &AtlasImpactIngressPolicy {
                cooldown_after_completion_ms: 100,
            },
            1_800_000_000_000,
        )?;
        assert_eq!(first.proposals.len(), 1);
        assert!(matches!(
            first.scheduling_decisions[0].disposition,
            AtlasImpactScheduleDisposition::HeldByBrake { .. }
        ));

        let seen = BTreeSet::from([first.proposals[0].dedupe_key.clone()]);
        let repeated = evaluate_local_atlas_impacts(
            &repository("consumer"),
            &projection,
            &[local_claim()],
            &seen,
            &lanes(),
            &brake,
            &AtlasImpactIngressPolicy {
                cooldown_after_completion_ms: 100,
            },
            1_800_000_000_000,
        )?;
        assert!(repeated.proposals.is_empty());
        assert_eq!(
            repeated.scheduling_decisions[0].disposition,
            AtlasImpactScheduleDisposition::Deduplicated
        );
        Ok(())
    }

    #[test]
    fn verification_pressure_obeys_pending_lane_then_completion_cooldown() -> Result<()> {
        let projection = projection(entanglement(
            AtlasCompatibility::Exact,
            AtlasVerificationState::Missing,
        ));
        let mut lane_state = lanes();
        lane_state[1].pending_proposal_id = Some(Uuid::from_u128(99));
        let pending = evaluate_local_atlas_impacts(
            &repository("consumer"),
            &projection,
            &[local_claim()],
            &BTreeSet::new(),
            &lane_state,
            &no_brake(),
            &AtlasImpactIngressPolicy {
                cooldown_after_completion_ms: 100,
            },
            1_000,
        )?;
        assert!(matches!(
            pending.scheduling_decisions[0].disposition,
            AtlasImpactScheduleDisposition::HeldByPendingLane {
                lane: AtlasImpactLane::Soul,
                ..
            }
        ));

        lane_state[1].pending_proposal_id = None;
        lane_state[1].last_completed_at_unix_ms = Some(950);
        let cooling = evaluate_local_atlas_impacts(
            &repository("consumer"),
            &projection,
            &[local_claim()],
            &BTreeSet::new(),
            &lane_state,
            &no_brake(),
            &AtlasImpactIngressPolicy {
                cooldown_after_completion_ms: 100,
            },
            1_000,
        )?;
        assert_eq!(
            cooling.scheduling_decisions[0].disposition,
            AtlasImpactScheduleDisposition::HeldByCooldown {
                lane: AtlasImpactLane::Soul,
                retry_at_unix_ms: 1_050,
            }
        );
        Ok(())
    }
}
