# Epiphany Model Atlas vertical slice

Updated: 2026-08-14

## Objective

Let one repository notice a provider change, understand its direct and
transitive consequences, and wake its own Modeling lane without an operator
typing `Continue`. The Atlas carries evidence between local minds. It does not
become a shared mind and it never grants Hands authority.

## Authority map

| Concern | Owner | Inputs | Output |
| --- | --- | --- | --- |
| Offered surface | Provider Modeling and local Mind | Current Body basis and semantic offer intent | `gamecult.model.surface_offer.v0` |
| Dependency claim | Consumer Modeling and local Mind | Current Body basis and semantic claim intent | `gamecult.model.dependency_claim.v0` |
| Edge verification | Consumer Soul and local Mind | Exact signed claim/offer versions and exact evidence artifact | `gamecult.model.dependency_verification.v0` |
| Publication | Repository publisher | Only its local Mind documents and exact Mind receipts | Signed `gamecult.model.atlas_publication.v0` events |
| Discovery and transport | Odin | Registered typed records | Opaque persisted/queryable publications and projections |
| Join, compatibility, cycles, blast radius | Atlas projector | Signed publications visible to one trusted audience | `gamecult.model.entanglement_projection.v0` |
| Local impact | Consumer Self and local Mind | Exact local claim plus exact projection/source set | `epiphany.model.dependency_impact.v0` |
| Lane wake | Consumer Resident Self | Admitted local impact and local lane lifecycle | Modeling or Soul pressure with no Hands grant |
| Presentation | Eve | One provider-owned retained tree | GUI and TUI lowerings of the same projection |

Nobody owns a shared edge. The projector derives each edge from a
provider-owned offer and a consumer-owned claim. Odin stores and serves those
facts but cannot normalize them into dependency truth. Eve renders them but
cannot mutate them.

## Data flow

```text
provider Mind offer -------+                         +--> Eve retained tree
                           |                         |
consumer Mind claim -------+--> signed publication --> Odin --> projector
                           |                                      |
consumer Soul verification +                                      v
                                                        signed projection
                                                                 |
                                             consumer impact ingress
                                                                 |
                                                 local Mind impact CAS
                                                                 |
                                               local Resident Self wake
```

No organ opens another repository's Mind store. Publishers read only their own
runtime Mind store. The projector reads Odin. Impact ingress reads the
projection plus its own local claims.

## Modeling authoring and Body evidence

Ordinary Modeling is the sole production authoring path for offers and claims.
Its model output names only semantic operations: create, deprecate, withdraw,
or retire. Creation includes a human label, a closed contract shape, and a
non-empty sorted set of repository-relative source paths. It cannot supply
UUIDs, source digests, causal identifiers, Body bases, CAS reads, timestamps,
receipts, or writes.

The runtime constructs `epiphany.repo_model.mutation_proposal.v2` from the
exact worker request, exact result, ordered evidence identifiers, and the Body
basis sealed into the launch. The keyed planner re-reads the current Body
basis, authenticates the manifest, resolves every requested source path to its
raw SHA-256, derives opaque UUIDv5 identities, and produces the exact Mind CAS.
An old Body observation or changed source refuses before any document is
written.

The publisher repeats the Body check at the federation boundary. It refuses to
publish an offer or claim unless every stored source path and digest still
matches the current authenticated Body manifest. The projector retains those
endpoint labels, contracts, lifecycle states, and Body references alongside
the exact applicable Soul evidence. Eve can therefore show why an edge exists,
not merely that the projector drew one.

## Invariants

- Repository identity is
  `gamecult://swarm/{swarm_id}/workspace/{workspace_id}`. Runtime and Body
  incarnation are evidence, not stable identity.
- Surface and claim identity are opaque UUIDs. Labels and paths cannot own
  identity.
- Contract compatibility is closed: SemVer, exact schema ID, or exact SHA-256
  digest. Scheme mismatch is incompatible. Natural language has no vote.
- The real repository Body digest is canonical 64-character lowercase SHA-256
  hex. Atlas publication digests use the explicit `sha256-` transport form.
  These are deliberately separate wire contracts.
- A source-version digest seals the exact compact MessagePack bytes in the
  owning CultCache record. `canonicalPayloadMsgpackSha256` separately seals the
  named canonical MessagePack published across the federation. Neither digest
  may substitute for the other.
- Local offer, claim, verification, and impact writes use dedicated typed
  planners and exact CultCache CAS. A stale Body or source version refuses the
  entire mutation.
- Model-authored offer and claim operations contain semantic intent and source
  paths only. Runtime-owned proposal v2 binds exact cause, evidence, current
  Body basis, resolved source digests, derived identities, and CAS authority.
- Publications bind repository, runtime incarnation, Body basis, Verse,
  source schema/key/version/digest, Mind receipt, canonical payload digest,
  publication time, and service signature.
- Publisher status is emitted every 30 seconds and becomes stale after 90
  seconds. A partition retains last-known edges and marks them unknown; absence
  never impersonates safety.
- Verification applies only to the exact current claim and offer versions.
- Build, deployment, and infrastructure-control cycles are blocked. Runtime,
  data, and schema cycles require declared failure semantics, remain
  review-required, and do not propagate autonomously. Informational
  governance/lore/persona cycles remain visible.
- Atlas may wake Modeling or Soul. Every pressure carries the explicit clause
  `This wake grants no Hands authority.`
- `atlas.publish`, `atlas.project`, and `atlas.impact_ingress` brakes stop writes
  and scheduling while retaining the last projection read-only.

## Library organs; deployment not admitted

- The publisher library reads a local Mind store, publishes immutable events to
  its publisher CultCache/CultMesh store, and transports them to Odin. Its
  signing identity lives in a separate private `.cc` store.
- The projector library pins repository trust anchors, queries Odin, verifies
  publications, derives a deterministic projection, and publishes canonical
  Eve provider and surface documents.
- The impact-ingress library compares a projection with locally owned claims,
  admits exact impact documents, and asks local Resident Self to schedule
  permitted pressure.

These are typed library owners, not three production daemons. The former
command wrappers had no unit, live consumer, deployment phase, or independent
failure-isolation contract and are not shipped. If the operational gate is
adopted, its process topology must be designed from the admitted lifecycle
rather than resurrected from those wrappers. The local Mind, private identity,
publisher events, CultMesh publication, Odin catalog, projection, and impact
scheduler state must not collapse into a writable mega-graph.

## Starfire pilot topology

The pilot uses three independent repository identities and state roots:

| Repository Body | Repository identity | Locally owned Atlas facts |
| --- | --- | --- |
| `F:\Projects\Eve` | one Starfire Eve workspace identity | Offers `gamecult.eve.surface.v1` |
| `F:\Projects\Odin` | one Starfire Odin workspace identity | Claims Eve surface; offers `cultmesh://odin/rendezvous/provider-catalog` |
| `F:\Projects\Epiphany` | one Starfire Epiphany workspace identity | Claims Odin provider catalog |

The Odin claim marks its catalog offer as affected. The projector therefore
derives `Epiphany -> Odin`, `Odin -> Eve`, and Epiphany's two-hop blast-radius
membership when Eve changes.

Do not invent deterministic IDs from repository paths. Each local Modeling
organ creates and retains its opaque surface and claim UUIDs. Trust anchors are
pinned by exact repository coordinates; key rotation is operator-gated.

## Rollout gates

1. Keep impact ingress absent. Let ordinary Modeling admit the three local seed
   facts into isolated Mind stores, then exercise the publisher and projector
   owners until signed publications, exact watermarks, and the retained Eve
   tree exist.
   Engage `atlas.publish`, `atlas.project`, and `atlas.impact_ingress`, run one
   more publisher/projector cycle plus the first ingress cycle, and prove every
   write/schedule is held while the last projection remains visibly stale and
   read-only. A brake cannot both forbid the first publication and prove that a
   publication exists; ordering is part of the invariant.
2. Release Atlas scopes for Modeling and Soul while leaving existing Hands
   authorization unchanged. A temporary Eve contract change must wake Odin
   Modeling and then Epiphany Modeling without an operator prompt.
3. Admit only the minimum Idunn-managed process topology justified by the
   proven lifecycle and failure-isolation requirements. Sustain the pilot while
   observing publisher/projector lag, per-publisher watermarks, rejection
   count, cycles, and pending impacts through CultMesh/Eve.
4. Only after sustained local proof, register topology and recovery runbooks in
   `gamecult-ops`. Yggdrasil and the public Verse remain outside V1.

The implementation and isolated proofs close the code slice. All four rollout
gates are operational evidence, not claims that can be manufactured by unit
tests.

## Verification

The focused proofs cover:

- no production `RepoModelPatch`, aggregate RepoModel store, aggregate
  admission reader/writer, or aggregate memory-graph key;
- runtime-owned proposal-v2 construction from exact request, result, evidence,
  and launch Body basis; models provide semantic operations only;
- local offer/claim admission through current authenticated Body manifests,
  resolved source digests, derived UUIDs, exact reasoning bases, and Mind CAS;
- publisher refusal after local Body evidence drifts;
- signature tamper, trust-coordinate substitution, rollback, replay,
  watermark, and payload-size refusal;
- order-independent projection digests, closed compatibility, verification
  invalidation, visibility-before-join, cycle classes, partition staleness, and
  deterministic transitive blast radius;
- the exact Epiphany -> Odin -> Eve chain and direct/transitive Modeling
  pressure after an Eve schema change;
- impact dedupe, pending lane locks, cooldown after completion, brake behavior,
  informational visibility without wake, and absence of Hands authority;
- Odin persistence of only the registered Atlas/projection/Eve schemas without
  Odin-authored edges;
- Eve conformance for one canonical GUI/TUI tree with endpoint contracts,
  source evidence, exact Soul evidence, publisher age/watermarks, cycles, blast
  radius, and presentation-only select/filter commands.

Test totals are historical pressure signals, not an Atlas acceptance claim.
Run only the focused owner affected by a source change. The shell-removal cut
uses:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\Meta\.cargo-target-codex'
cargo test -p epiphany-core --lib atlas::runtime::tests
cargo test -p epiphany-release-construction --lib binary_suffix_follows_requested_target_not_packager_host
```

Changes to publisher, projector, impact, transport, or Eve lowering logic must
run the corresponding focused module tests. Cross-repository and operational
Gate verification belongs to Idunn on the admitted exact package; no local
workspace-wide or all-target build is part of this map.

Atlas verification belongs to the owning library and packaged runtime tests.
The old feature-gated recovery smoke executable and its smoke-only authority
exports have been deleted.
