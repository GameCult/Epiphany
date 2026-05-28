# Epiphany Architectural Teardown

This note records the May 2026 suspicion pass over Epiphany's live architecture.
It is not a product plan and not a victory lap. It is the red tag on the
machine: stop piling outward features on a control plane whose ownership has
started to sag.

The core thesis remains sound: Epiphany needs durable typed state, explicit
maps, reviewable evidence, bounded worker output, compaction-safe re-entry, and
native Rust/CultCache/CultNet contracts. The unstable part is the current
Codex app-server Epiphany control plane. It works, but it works by concentrating
too much policy, projection, lifecycle glue, and coordinator sequencing in the
host seam.

## Plain-Language Map

### Core Inputs

- user turns and operator API calls
- repository files and watcher snapshots
- rollout history and latest token telemetry
- retrieval/index manifests and Qdrant/BM25 search results
- specialist job launch requests and runtime-spine job results
- heartbeat state, role memory stores, and reviewable self patches
- local persistent project doctrine in `state/map.yaml`, handoff notes, and
  ledgers

### Durable State Stores

- `EpiphanyThreadState` in Codex protocol/session/rollout state
- native runtime-spine documents in `state/runtime-spine.msgpack`
- heartbeat physiology in `state/agent-heartbeats.msgpack`
- role dossiers in `state/agents.msgpack`
- distilled project memory in `state/map.yaml`, `state/ledgers.msgpack`, and
  handoff notes
- retrieval manifests and optional Qdrant collections outside the thread state

### Core Transformations

- prompt projection from durable state into `<epiphany_state>`
- revision-gated state mutation through `thread/epiphany/update` and accepted
  promote/role/reorient gates
- retrieval, distillation, proposal, and promotion policy in `epiphany-core`
- app-server projection of state into scene/jobs/roles/freshness/context/
  graph/planning/pressure/reorient/CRRC/coordinator views
- app-server launch/read-back/acceptance over heartbeat-backed runtime-spine
  jobs
- heartbeat and character-loop packet generation over role dossiers

### Core Outputs

- model-visible Epiphany state and specialist prompts
- client-visible `thread.epiphanyState`
- operator responses for scene, jobs, roles, freshness, context, graphQuery,
  planning, pressure, reorient, CRRC, coordinator, roleResult, and
  reorientResult
- `thread/epiphany/stateUpdated` notifications
- runtime-spine job/result/event receipts
- heartbeat/routine artifacts and Face/character-loop packets

### Stage Ownership

- Codex protocol currently owns the thread-state shape and app-server protocol
  types.
- Codex core owns session storage, rollout persistence, prompt injection, and
  the state mutation bridge.
- `epiphany-core` owns retrieval, distillation, proposal, promotion,
  runtime-spine, heartbeat, role memory, bridge CLIs, and smoke binaries.
- Codex app-server currently owns most operator projection, fixed-lane
  coordinator policy, CRRC recommendation policy, role launch packet assembly,
  role/reorient result interpretation, and action availability.

That last bullet is the rot. The host seam has become an organ.

## Required Invariants

- There must be one source of truth for project understanding.
- Derived projections must not become policy authorities.
- Runtime job lifecycle must have one owner.
- Review/acceptance identity must be keyed by stable typed ids, not matching
  summaries.
- Public operator surfaces must not expose sealed worker/raw-result payloads by
  default.
- App-server must route and adapt; it must not be the Epiphany brain.
- Experimental cognition blobs must not masquerade as stable typed doctrine.
- Read-only surfaces must be views over state, not a multiplying API religion.

## Architectural Smells

### 1. App-Server As Hidden Epiphany Brain

Current mechanism: `vendor/codex/codex-rs/app-server/src/codex_message_processor.rs`
contains the Epiphany endpoint handlers plus a large cluster of
`map_epiphany_*`, `build_epiphany_*`, `load_epiphany_*`, coordinator, CRRC, role
status, and result interpretation helpers.

Real need: expose operator surfaces through the existing Codex app-server.

What breaks if deleted: Aquarium/status/coordinator endpoints, fixed role
launch/read-back, CRRC recommendations, and many tests.

What that proves: the behavior is needed, but its current owner is wrong. The
host seam has absorbed domain policy.

Simpler architecture: move Epiphany surface derivation and coordinator policy
into `epiphany-core` modules. App-server should parse params, load thread/runtime
inputs, call a domain function, and serialize the response.

Files to change:

- `vendor/codex/codex-rs/app-server/src/codex_message_processor.rs`
- `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs`
- new `epiphany-core/src/surfaces/*`
- focused app-server tests moved or narrowed after extraction

### 2. Duplicated Job Authority

Current mechanism: `EpiphanyThreadState.job_bindings` stores launcher id,
authority scope, backend kind, and backend job id, while runtime-spine stores
the native job/result lifecycle. App-server overlays bindings onto derived jobs.

Real need: connect thread state and launched specialist/reorient work.

What breaks if deleted: roleResult/reorientResult cannot locate runtime jobs.

What that proves: the link is needed, but lifecycle ownership is duplicated.
Thread state should not mirror runtime job state.

Simpler architecture: runtime-spine is authoritative for jobs/results. Thread
state stores typed intent/link records only when project understanding needs
the association.

Files to change:

- `vendor/codex/codex-rs/protocol/src/protocol.rs`
- `vendor/codex/codex-rs/core/src/codex_thread.rs`
- `epiphany-core/src/runtime_spine.rs`
- app-server role/reorient result handlers and job projection code

### 3. Projection Endpoint Proliferation

Current mechanism: scene, jobs, roles, freshness, context, graphQuery,
planning, pressure, reorient, CRRC, and coordinator each have their own endpoint,
response type, mapper, status enums, and tests.

Real need: clients need compact operator lenses.

What breaks if deleted: the GUI loses convenient panes and existing API clients
break.

What that proves: views are needed, not that every lens deserves a first-class
protocol throne.

Simpler architecture: create a smaller `thread/epiphany/view` contract with
selectable typed lenses plus separate mutation intents. Promote only genuinely
independent commands to separate endpoints.

Files to change:

- `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs`
- `vendor/codex/codex-rs/app-server-protocol/src/protocol/common.rs`
- `vendor/codex/codex-rs/app-server/src/codex_message_processor.rs`
- Aquarium/status consumers

### 4. Stringly Accepted-Result Identity

Current mechanism: accepted role/reorient findings are detected by evidence
kind/status/summary matching.

Real need: prevent repeat acceptance and know which finding is already banked.

What breaks if deleted: repeat-acceptance bugs return.

What that proves: acceptance needs a stable receipt, not summary matching.

Simpler architecture: add typed acceptance receipts keyed by runtime
`job_result` id and role/reorient binding identity. Evidence can summarize the
accepted result, but it must not be the identity system.

Files to change:

- `vendor/codex/codex-rs/protocol/src/protocol.rs`
- `epiphany-core/src/runtime_spine.rs`
- app-server role/reorient accept/result mapping
- schema catalog documents

### 5. Raw Worker Output In Public Finding Contracts

Current mechanism: `ThreadEpiphanyRoleFinding` includes `raw_result:
serde_json::Value` and `self_patch: unknown`.

Real need: debug malformed worker output and preserve optional self-memory
requests.

What breaks if deleted: convenient forensic inspection and loosely typed
self-patch handling.

What that proves: public API is leaking internal accident. Sealed worker output
should not be normal operator surface material.

Simpler architecture: public findings expose typed projection only. Raw output
stays in artifacts and is opened only by explicit forensic/debug flow.
`selfPatch` should be a typed contract or absent.

Files to change:

- `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs`
- app-server finding mappers
- Aquarium result display
- `epiphany-core/src/agent_memory.rs` if selfPatch becomes fully typed

### 6. Heartbeat/Cognition Blob State

Current mechanism: heartbeat state has typed participants and scheduler fields,
then stores sleep cycle, memory resonance, incubation, thought lanes, bridge,
appraisals, reactions, and extra data as generic JSON values.

Real need: prototype Void/Ghostlight-derived routine physiology without freezing
premature schemas.

What breaks if deleted: Aquarium/status cannot inspect the rich cognition
routine; routine receipts lose stateful continuity.

What that proves: useful scaffolding, not stable foundation. JSON-shaped
cognition blobs inside CultCache are still blobs.

Simpler architecture: split stable heartbeat scheduling from experimental
cognition receipts. Type only the fields policy consumes; keep exploratory
thought-weather as receipts/artifacts until its contract hardens.

Files to change:

- `epiphany-core/src/heartbeat_state.rs`
- `epiphany-core/src/bin/epiphany-heartbeat-store.rs`
- `schemas/cultnet/*`
- Aquarium heartbeat/routine consumers

### 7. Generic Backend Plumbing With One Backend

Current mechanism: `EpiphanyJobBackendKind` exists but only has `Heartbeat`.

Real need: name the current backend and leave a future escape hatch.

What breaks if deleted: response shape and mapper tests churn.

What that proves: one-backend polymorphism is just-in-case architecture.

Simpler architecture: store `runtime_job_id` / `activation_job_id` directly
until a second backend exists.

Files to change:

- `vendor/codex/codex-rs/protocol/src/protocol.rs`
- `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs`
- job launch/result mappers and tests

### 8. Tests Freeze Implementation Shape

Current mechanism: many app-server tests assert exact mapper/coordinator shapes
inside the same giant file that implements the policy.

Real need: prevent regressions in coordination and operator views.

What breaks if deleted: easy behavior regressions.

What that proves: tests are needed, but they currently bless the app-server as
policy owner.

Simpler architecture: move policy tests to `epiphany-core`. App-server tests
should assert routing, serde, error handling, and adapter fidelity.

Files to change:

- `vendor/codex/codex-rs/app-server/src/codex_message_processor.rs`
- new `epiphany-core` policy test modules
- app-server protocol fixture tests

## Ranked Teardown Plan

### Keep

- durable typed `EpiphanyThreadState`
- prompt projection from typed state
- revision-gated update/promote/accept writes
- retrieval, distillation, proposal, promotion policy in `epiphany-core`
- native runtime-spine as the future job/result authority
- explicit review gates for semantic findings
- Aquarium as reflection/control surface, not source of truth

### Cut

- public `raw_result` exposure from normal role/reorient findings
- one-backend generic `BackendKind` plumbing until another backend exists
- placeholder `specialist-work` job whose purpose is preventing clients from
  inventing scheduler state
- any new feature surface that adds another app-server-owned policy mapper

### Collapse

- collapse scene/jobs/roles/freshness/context/planning/pressure/reorient/CRRC/
  coordinator into fewer selectable view lenses
- collapse `jobBindings` lifecycle mirror into runtime-spine-owned job lifecycle
  plus typed thread-state links
- collapse summary-string acceptance identity into typed acceptance receipts

### Split

- split app-server routing from Epiphany domain policy
- split stable heartbeat scheduling from experimental cognition receipts
- split public operator findings from forensic raw worker artifacts
- split protocol surface contracts from local app-server mapper convenience

### Rebuild

Rebuild the Epiphany app-server control plane. Not patch. The current surface
works, but it concentrates policy, projection, lifecycle, prompt packet
assembly, coordinator sequencing, and tests in a vendored host file. That is
architectural Jenga with a good demo standing in front of it.

The next phase is not more outward bridge/UI/Face work. The next phase is
control-plane purification:

1. extract Epiphany view/coordinator policy into `epiphany-core`
2. make runtime-spine the sole job/result lifecycle authority
3. replace summary-matching acceptance with typed receipts
4. reduce API surface proliferation into intentional view lenses and mutation
   intents
5. remove public raw-result leakage from normal operator contracts
6. type or quarantine heartbeat cognition blobs
7. only then resume outward feature work

This is now top priority because Epiphany is the foundation everything else is
supposed to stand on. You do not get a Perfect Machine by adding another patch
layer and calling the shadow a subsystem.
