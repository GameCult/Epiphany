# EpiphanyAgent Verse Service Contract

EpiphanyAgent is a service participant in the GameCult Verse. It already carries
typed state through repo-contained CultCache/CultMesh bodies; the next coherence
cut is to publish its meaningful operator/status surfaces as Eve GUI/TUI DSL so
local runtimes can inspect Epiphany without reading private run artifacts.

## Owner Map

- Owner: EpiphanyAgent owns Epiphany runtime state, organ state, role launches,
  local Verse context packets, operator status, operator snapshots, and public
  dream/export policy.
- Inputs: runtime-spine documents, heartbeat state, memory graph documents,
  role/reorient/job intents, local Verse context, Persona projections where
  applicable, and model/runtime receipts.
- Outputs: typed CultCache/CultMesh documents for internal status, operator
  snapshots, run intents/receipts, Verse policies, global room policies,
  launch-context packets, and future public dream documents.
- Derived state: JSON status files under `.epiphany-run`, smoke artifacts, and
  local operator packets are witnesses or compatibility artifacts, not durable
  owners.
- Forbidden writers: dashboards, shell wrappers, bridge adapters, app-server
  compatibility layers, and renderer surfaces must not directly mutate Epiphany
  truth. They emit command intent or read typed projections.
- Shared paths: CLI status, bridge status, local Verse query, operator snapshot,
  future Eve surface, and future compact TUI must read the same CultMesh-backed
  documents rather than each inventing a separate view.
- Deletion line: any bespoke presentation/status artifact that becomes the only
  source of a fact must be demoted behind CultCache/CultMesh or deleted.

## Current State

EpiphanyAgent already has the lower substrate:

- vendored `cultcache-rs`, `cultnet-rs`, and `cultmesh-rs`;
- `cultcache.store.v1` compatibility;
- typed local Verse/status/operator documents in `epiphany-core`;
- `tools/epiphany_local_run.ps1 -Mode status` producing operator-safe status
  artifacts and local Verse context;
- `gamecult.persona_state.v0` schema availability for public Persona
  projections.

The missing service-architecture surface is not "more JSON". It is an
Epiphany-owned Eve surface provider.

## Eve Surface Target

EpiphanyAgent should publish an Eve GUI/TUI DSL surface with these panels:

1. `Operator Status`: runtime id, bridge transport, authority owner, prompt
   authority separation, current run state, last status timestamp.
2. `Local Verse Context`: visible Verse tier, advertised schemas, local peer
   context packet, and freshness.
3. `Runtime Spine`: active jobs, role/reorient launches, accepted receipts, and
   blocked or denied intents.
4. `Memory Graph`: profile availability, thin-state warnings, last refresh, and
   public/private boundary.
5. `Persona Projection`: present only for public Persona/Persona projections using
   `gamecult.persona_state.v0`; private organ state must not leak through this
   panel.

Each panel must expose authority and freshness. If a value comes from a JSON
witness file instead of typed state, the surface should say so.

## Migration Order

1. Define the Epiphany Eve provider document/surface contract.
2. Add an `epiphany-eve-surface` read-only command that lowers the current
   CultMesh operator/status documents into Eve DSL.
3. Publish the surface through CultMesh/Odin discovery.
4. Let Eve browser/native/TUI lower the same surface.
5. Demote `.epiphany-run` JSON status files to compatibility witnesses only.

The invariant: Epiphany runtime truth stays in typed CultCache/CultMesh
documents. Eve makes it inspectable; Eve does not become the runtime brain.
