# Epiphany Native Implementation Plan

This is the live campaign plan. The Codex-fork phase succeeded by starving
Codex of Epiphany authority; Epiphany is no longer implemented as a collection
of app-server routes.

## Objective

Build an inspectable native organism whose typed state, runtime, organ gates,
memory, scheduling, interfaces, and social crossings can operate without Codex
owning Epiphany cognition. Retain Codex-derived code only where it provides an
earned OpenAI-compatible authentication or model-transport capability.

## Current Mechanism

- `epiphany-state-model` owns durable Mind shape.
- `epiphany-core` owns coordinator policy, state admission, runtime spine,
  surfaces, organ gates, heartbeat physiology, Persona loop, and CultMesh
  integration.
- Native binaries expose coordinator, status, state, runtime, memory, daemon,
  Persona, repo-work, and Verse operations.
- CultCache stores typed state and receipts; CultMesh/CultNet carry typed local
  and federated projections.
- Vendored Codex exposes no Epiphany route, DTO, thread-state field, rollout
  migration, scheduler, watcher, or bridge crate.

## Invariants

1. Mind is the only durable state-admission gateway.
2. Substrate Gate controls access; it does not admit state.
3. Eyes establishes provenance; Hands causes consequences; Soul verifies them.
4. Self coordinates but does not steal another organ's authority.
5. Heartbeat/Idunn own physiology and daemon survival, not project truth.
6. Persona speech and outside-world action preserve consent, identity,
   provenance, and private-state seals.
7. CultMesh projections and Eve renderers never become owners.
8. Codex remains free of Epiphany state/process/interface authority.

## Completed Foundation

- Native typed thread state and state-update validation.
- Native coordinator facade and derived status/recommendation surfaces.
- CultCache runtime spine with worker, Mind, Eyes, Substrate Gate, Hands, Soul,
  Continuity, and coordinator receipts.
- Native OpenAI auth/model/runtime spine.
- Native heartbeat, daemon supervision, and cluster liveness surfaces.
- Native Persona memory/turn/audit machinery and Bifrost-governed public mouths.
- Native repo-work planning, Hands/Soul/Mind closure, public-proof, credit, and
  Bifrost accounting families.
- CultMesh local Verse, compact operator readbacks, Gjallar sight, Eve
  connection receipts, and three-Verse trust boundaries.
- Complete removal of the Epiphany Codex app-server compatibility surface and
  `epiphany-codex-bridge`.

## Active Campaign

### 0. Unify Canonical State Transactions

Status: complete. `coordinator_state_transaction.rs` is the sole production
writer of `THREAD_STATE_KEY`; ordinary updates, launches, and Mind acceptance
share it, raw storage writers are deleted, and negative source guards prevent a
second owner.

- Define one transaction owner for canonical state revision changes.
- Make ordinary update, launch, and Mind-witness acceptance call that owner.
- Preserve operation-specific atomic companions: launch runtime envelopes and
  acceptance Mind witnesses must commit in the same cache transaction.
- Replace misleading `runtime_spine_store` state writes with an explicit
  unified store contract or genuinely separate stores with a typed transaction
  coordinator; do not rely on path aliasing by convention.
- Demote raw `thread_state_store` writers to crate-private substrate helpers.

Exit evidence:

- one named primitive owns `THREAD_STATE_KEY` writes;
- negative source tests reject direct canonical-state writes elsewhere;
- ordinary update, launch, and acceptance transaction tests pass against the
  chosen explicit store contract;
- `EpiphanyCoordinatorService` path names match real ownership.

### 1. Repair Proprioception

- Keep `notes/epiphany-current-algorithmic-map.md` aligned with source.
- Distinguish historical evidence from current mechanism in long handoffs.
- Remove live-looking references to deleted bridge/routes from current docs,
  prompts, wrapper help, and operator output.
- Add source guards where a removed authority could plausibly regrow.

Exit evidence:

- current canonical docs contain no false live paths;
- every mapped owner/file exists;
- source scans show no Epiphany Codex protocol authority.

### 2. Normalize Native Operator Contracts

Status: complete for the canonical coordinator/status boundary. The two-path
service constructor and status flags are deleted, native JSON emits `state`,
and the wrapper supplies the live unified store. Source guards reject the old
field and flags.

- Audit native JSON artifacts for compatibility-shaped field names and nested
  Codex response assumptions.
- Rename only when no external contract depends on the old shape; otherwise
  publish a typed migration and explicit expiry.
- Prefer CultCache/CultMesh documents as load-bearing state; keep JSON at CLI,
  schema, MCP, OpenAI, and other xenos boundaries.

Exit evidence:

- native coordinator/status consumers read native fields directly;
- wrapper summaries are projections, not reconstructed policy;
- no load-bearing JSON sidecar decides behavior.

### 3. Close The Organ Loop

Status: active. The scheduler can no longer impersonate Modeling/Mind after
Hands execution, and closure refuses deterministic fallback or a passing
verdict without an explicit model-authored finding. The remaining cut is to
persist that Modeling finding as its own typed runtime document and make Mind
admission consume it rather than accepting CLI fields directly.

- Prove Hands → Soul → Modeling → Mind → Self on a fresh repository without
  supervisor implementation or direct worker-thought inspection.
- Ensure every consequence has Substrate Gate scope, Hands receipts, Soul
  verdict, Modeling map update, and Mind admission before the next Hands turn.
- Treat no-diff, unreviewable, timeout, and regather outcomes as explicit typed
  states rather than success-shaped silence.

Exit evidence:

- fresh-repo live-fire closes one nontrivial work item;
- negative tests prove workers/Hands/Soul cannot bypass Mind;
- operator-safe artifacts explain owner, inputs, outputs, and verdicts.

### 4. Make Physiology Durable

- Finish Idunn-owned service installation/audit aftercare without moving
  elevation authority into Self or wrappers.
- Prove cooldown-after-completion, no overlapping lane heartbeat, idle sleep,
  rumination, and recovery across restart.
- Ensure scheduler and liveness receipts remain separate from project state.

Exit evidence:

- seven organ daemons survive restart under typed policies;
- complete lifecycle audits close the current elevated-service warning;
- no duplicate daemon keeper exists.

### 5. Publish Eve/CultUI Interfaces

- Emit typed Eve composition/state graphs for coordinator, organ status,
  receipts, repo work, Persona, and daemon physiology through CultMesh.
- Lower those graphs in Aquarium/browser/TUI without renderer-owned truth.
- Keep private/internal, trusted-local, and public Verse surfaces distinct.

Exit evidence:

- one composition graph renders in at least GUI and compact TUI targets;
- commands route back through typed owner intents;
- UI timeline probes observe transition-time invariants.

### 6. Prove Federated Work And Social Citizenship

- Demonstrate a Bifrost-originated work item flowing through Epiphany lanes,
  maintainer review, credit/ledger receipts, and operator-safe public proof.
- Demonstrate Persona discussion with semantic memory, consent, disagreement,
  and governed external crossing.
- Treat foreign dreams as thought weather until a reviewed adoption receipt.

Exit evidence:

- public proof contains no private worker/operator context;
- Bifrost and Heimdall ownership remains explicit;
- local agency, refusal, exit, and provenance survive federation.

## Verification Strategy

- Unit tests prove local authority and validation.
- Adjacent-organ smokes prove typed handoff.
- Native end-to-end runs prove operator/product contracts.
- Negative source and runtime checks prove old owners cannot regain authority.
- Timeline checks cover load, user/programmatic transition, mid-transition,
  settled state, and restart/re-entry where timing matters.
- Served/runtime versions and schema versions are exposed when deployment or
  cache uncertainty could impersonate logic failure.

## Cut Line

Delete or demote:

- stale current-looking bridge/route prose;
- compatibility-shaped native code with no external consumer;
- wrapper policy duplicated from Rust owners;
- whole-context serialization where a narrow typed query exists;
- tests whose only purpose is keeping extinct compatibility anatomy alive;
- any cache, registry, adapter, mode, or metadata field without a named
  invariant and owner.

## Immediate Next Action

Audit the live Hands -> Soul -> Modeling -> Mind -> Self loop against the fresh
repository exit evidence. Identify which required receipt, admission, and
negative-bypass proofs are current and which are only historical claims; close
the first missing proof rather than adding another route or summary.
