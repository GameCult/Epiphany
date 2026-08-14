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

## Live organs

- `epiphany-atlas-publisher` reads a local Mind store, publishes immutable
  events to its publisher CultCache/CultMesh store, and transports them to
  Odin. Its signing identity lives in a separate private `.cc` store.
- `epiphany-model-entanglement-projector` pins repository trust anchors, queries
  Odin, verifies publications, derives a deterministic projection, and
  publishes the canonical Eve provider and surface documents.
- `epiphany-atlas-impact-ingress` compares a projection with locally owned
  claims, admits exact impact documents, and asks local Resident Self to
  schedule permitted pressure.

Each organ has its own store. The local Mind, private identity, publisher
events, CultMesh publication, Odin catalog, projection, and impact scheduler
state are never collapsed into a writable mega-graph.

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

1. Engage all three Atlas brake scopes. Run the three publishers, projector,
   and impact ingresses against isolated pilot stores. Inspect publications,
   watermarks, rejections, and the retained Eve tree without lane scheduling.
2. Release Atlas scopes for Modeling and Soul while leaving existing Hands
   authorization unchanged. A temporary Eve contract change must wake Odin
   Modeling and then Epiphany Modeling without an operator prompt.
3. Place the organs under the existing daemon supervisor. Sustain the pilot
   while observing publisher/projector lag, per-publisher watermarks,
   rejection count, cycles, and pending impacts through CultMesh/Eve.
4. Only after sustained local proof, register topology and recovery runbooks in
   `gamecult-ops`. Yggdrasil and the public Verse remain outside V1.

The implementation and isolated proofs close the code slice. Gates 2 through 4
are operational evidence, not claims that can be manufactured by unit tests.

## Verification

The focused proofs cover:

- no production `RepoModelPatch`, aggregate RepoModel store, aggregate
  admission reader/writer, or aggregate memory-graph key;
- runtime-owned semantic Modeling ingress and exact keyed reasoning bases;
- local offer/claim admission through current real Body evidence and Mind CAS;
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
- Eve conformance for one canonical GUI/TUI tree with presentation-only select
  and filter commands.

Current verification commands:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\Meta\.cargo-target-codex'
cargo test -p epiphany-core --lib
cargo test -p epiphany-openai-runtime --lib
cargo test -p epiphany-release-bundle --features coordinator-runtime --bin epiphany-mvp-coordinator
cargo check --workspace --all-targets

Set-Location F:\Projects\Odin
node --test test/model-atlas-documents.test.cjs
cargo test -p odin-core

Set-Location F:\Projects\Eve
node --test web/model-atlas-conformance.test.mjs
node --test packages/eve-contracts/test/contracts.test.mjs
```

`cargo check --workspace --all-targets --all-features` still reaches the older
feature-gated `epiphany-workspace-coverage-recovery-smoke`, which imports
`current_workspace_coverage_recovery_target` and
`WorkspaceCoverageRecoveryTarget`; those symbols were already absent before
this slice. Do not restore them as an Atlas compatibility shim. The ordinary
all-target workspace and all Atlas-bearing targets compile.
