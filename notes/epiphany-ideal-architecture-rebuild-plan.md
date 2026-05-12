# Epiphany Ideal Architecture Rebuild Plan

This is the smallest coherent Epiphany architecture that can satisfy the
product goals without preserving current abstractions merely because they
already have names. It complements `notes/epiphany-architectural-teardown.md`:
the teardown names the rot; this note names the replacement machine.

The intent is explicit: rebuild the control-plane foundation. Do not pile
feature adapters on top of the current app-server growth. Epiphany is the core
everything else will stand on, so the correct first move is structural
purification, not more decorated scaffolding.

## Product Goals To Preserve

Epiphany must still provide:

- durable typed project/thread state that survives turns, resume, rollback, and
  compaction
- explicit maps, frontiers, scratch, evidence, and checkpoints
- bounded retrieval and indexing over a workspace
- review-gated distillation, proposal, promotion, and worker result acceptance
- observable role separation for implementation, modeling/checkpoint,
  verification/review, planning/Imagination, and reorientation
- explicit CRRC behavior based on pressure, freshness, checkpoints, and runtime
  evidence
- native runtime job/result receipts
- operator-safe Aquarium views and actions
- sealed forensic access to raw worker internals only when explicitly requested
- no GUI, worker, or app-server surface becoming a second source of truth

Anything beyond that is optional until the foundation is clean.

## Smallest Coherent Architecture

### Durable State Model

There is one durable project/thread mind: `EpiphanyState`.

It contains only facts or decisions that should survive rehydration:

- `revision`: monotonic state revision for optimistic writes
- `objective`: the active explicit mission, plus selected active subgoal
- `working_map`: architecture/dataflow graph nodes, edges, links, and code refs
- `frontier`: current active slice, dirty refs, open questions, and evidence
  gaps
- `scratch`: disposable current hypothesis, next probe, and bounded notes
- `evidence`: accepted observations, verifier receipts, implementation audits,
  rejected paths, and durable scars
- `checkpoint`: compaction/reorientation resume packet
- `planning`: captures, backlog items, roadmap streams, and Objective Drafts;
  drafts are not active authority
- `runtime_links`: stable ids linking the durable project state to active or
  recent runtime jobs/results when the association matters
- `acceptance_receipts`: typed records saying result `X` was reviewed and
  accepted/refused/applied, with the patch/evidence ids it affected

It does not contain:

- job lifecycle state
- backend-specific runtime metadata
- copied runtime status
- raw worker output
- acceptance identity derived from summaries
- experimental cognition blobs

### Runtime State Model

There is one native runtime spine: `RuntimeState`.

It owns:

- runtime identity
- sessions
- jobs
- job results
- events
- cancellation/interruption status
- artifact refs

Runtime state is machinery telemetry. It is not project truth. Durable state may
link to runtime records, but must not mirror their lifecycle fields.

### Agent Memory And Heartbeat State

Role memory is separate from project truth:

- role dossier
- stable traits, goals, values, memories
- reviewed `selfPatch` mutations

Heartbeat state is the scheduler physiology:

- participants
- pending turns
- readiness/cooldown
- stable timing and selection fields

Void/Ghostlight-derived cognition output is not stable project state until it
has a typed contract. Sleep cycle, thought lanes, appraisals, bridge judgments,
and similar thought-weather should be receipts/artifacts or explicitly
quarantined experimental documents.

## Authoritative Boundaries

### `epiphany-core`

Owns domain truth and policy:

- durable state schema and validation
- state patch application
- view/lens derivation
- coordinator policy
- CRRC policy
- role result interpretation
- acceptance policy
- distillation, proposal, promotion
- retrieval/indexing domain behavior where practical

If a rule decides what Epiphany believes, recommends, accepts, rejects, or
considers safe, it belongs here.

### Native Runtime Spine

Owns execution lifecycle:

- job creation
- job status
- job result records
- runtime events
- cancellation/interruption
- artifact references

Runtime-spine is the job table. Thread state is not.

### Codex App-Server

Owns transport and host adaptation:

- parse JSON-RPC params
- load Codex thread/session/runtime inputs
- call `epiphany-core`
- serialize protocol responses
- emit host notifications

The app-server must not own Epiphany domain policy. It is a bridge, not the
brain. No more extra thrones in the vestibule.

### Aquarium

Owns operator presentation and explicit actions:

- render state/views/artifacts
- request bounded actions
- display reviewable findings
- surface sealed forensic artifacts only when explicitly requested

Aquarium owns no canonical understanding.

### Prompt Rendering

Owns projection into model context:

- render bounded durable state
- render role/control prompts

Prompt rendering does not mutate state and does not repair state by implication.

## Boundary Messages And API Surfaces

The ideal control plane has fewer surfaces.

### `epiphany/state/read`

Reads authoritative durable state.

### `epiphany/state/update`

Revision-gated durable state patch. No runtime lifecycle mutation except stable
links/receipts that are project truth.

### `thread/epiphany/view`

Reads selected lenses from durable state plus runtime telemetry. It is
read-only. The request specifies lenses such as:

- `scene`
- `jobs`
- `roles`
- `freshness`
- `context`
- `graph`
- `planning`
- `pressure`
- `reorient`
- `coordinator`

These are derived outputs. They do not deserve separate protocol kingdoms
unless a lens becomes an independent product contract with its own authority
boundary.

### `epiphany/runtime/launch`

Creates a runtime-spine job and returns typed runtime ids. It may also add a
minimal durable runtime link if the project state needs the association.

### `epiphany/runtime/read`

Reads runtime session/job/result/event state from runtime-spine.

### `epiphany/runtime/interrupt`

Requests cancellation/interruption through runtime-spine.

### `epiphany/result/accept`

Accepts or refuses a typed runtime result by result id. It writes an
`acceptance_receipt`, optional durable patch, and optional evidence records.
Acceptance identity is the receipt/result id, not a summary string wearing a
fake moustache.

### `epiphany/retrieve/index`

Writes retrieval catalog/index state only.

### `epiphany/retrieve/query`

Reads retrieval catalog/search results only.

## Derived Instead Of Stored

Derive:

- scene summaries
- job view
- role lane status
- coordinator recommendation
- CRRC recommendation
- freshness status
- graph query neighborhoods
- pressure
- available actions
- whether a result has already been accepted
- role readiness

Store:

- durable project truth
- runtime lifecycle
- acceptance receipts
- evidence
- checkpoints
- role memory
- stable scheduler state

## Delete Or Collapse

Delete because it compensates for bad ownership:

- `jobBindings` as a lifecycle mirror
- one-backend `BackendKind::Heartbeat`
- `specialist-work` placeholder job
- summary-string accepted-result detection
- public `raw_result` in normal findings
- app-server-owned coordinator/CRRC policy
- app-server-owned role result interpretation
- tests that canonize mapper internals as architecture

Collapse:

- many `thread/epiphany/*` view endpoints into `thread/epiphany/view` lenses
- job binding lifecycle fields into runtime-spine jobs/results
- public findings into typed projections plus sealed forensic artifacts

Split:

- app-server routing from Epiphany policy
- stable heartbeat scheduling from experimental cognition receipts
- public operator API from forensic/debug payload access
- protocol contracts from local mapper convenience

## Impossible By Construction

The rebuilt architecture should make these impossible:

- accepting the same runtime result twice without an existing typed receipt
- inferring acceptance from a summary string
- exposing raw worker output through normal operator APIs
- mutating durable state from a view/read endpoint
- app-server policy diverging from `epiphany-core` policy
- Aquarium becoming source of truth
- launching a job whose lifecycle is not owned by runtime-spine
- storing experimental cognition blobs as stable doctrine
- adding another operator lens by copying an app-server mapper pile
- treating GitHub, Unity, Rider, Discord, or any bridge as internal project
  truth instead of external evidence/adapters

## Current Epiphany Compared To Ideal

### Sound Foundations

- durable typed `EpiphanyThreadState`
- prompt projection from durable state
- revision-gated update/promote/accept writes
- retrieval, distillation, proposal, and promotion work in `epiphany-core`
- native runtime-spine exists
- explicit review gates exist
- Aquarium is intended as reflector/control surface, not source of truth

### Rotten Or Unstable Foundations

- app-server owns too much policy
- protocol surface has exploded into many endpoint-shaped conveniences
- job lifecycle is split between durable thread state and runtime-spine
- acceptance identity is stringly
- raw worker payloads leak into normal public finding contracts
- heartbeat cognition is blob-heavy
- tests freeze accidental app-server implementation shape

Direct diagnosis: stop feature work and rebuild the control-plane foundation.
The current machine is capable, but the capable part is growing in the wrong
organ.

## Migration Plan

The migration must cut toward the ideal in small reversible commits. Prefer
deleting unstable foundations early over building adapters around them.

### 1. Freeze New Surfaces

Add a guardrail that no new `thread/epiphany/*` endpoint lands until extraction
has begun.

Commit shape:

- doc/test comment or compile-visible checklist near protocol additions
- no behavior change

Purpose: stop fresh surface sprawl while the foundation is being rebuilt.

### 2. Introduce Core Surface Modules

Add `epiphany-core/src/surfaces/` with domain-level surface contracts for the
live lenses and read models:

- pressure, freshness, jobs, planning, scene, role board, reorient, CRRC, and
  coordinator view derivation
- targeted graph/evidence context and bounded graph query derivation
- role and reorient result acceptance bundle policy
- typed inputs that carry only the source snapshots each surface actually needs

Do not move behavior yet.

Purpose: create the destination organ before transplanting nerves.

### 3. Extract One Pure Read Lens

Move `pressure` or `freshness` derivation from app-server into `epiphany-core`.

Commit shape:

- app-server endpoint still exists
- app-server loads inputs and calls core
- existing smoke still passes

Preferred first lens: `pressure`, because it is pure and low-risk.

Purpose: prove the new ownership path with one small cut.

### 4. Extract CRRC And Coordinator Policy

Move CRRC recommendation and fixed-lane coordinator decision logic into
`epiphany-core`.

Commit shape:

- protocol response shape unchanged
- policy tests move to `epiphany-core`
- app-server tests narrow to routing/adapter fidelity

Purpose: remove the largest policy tumor early.

### 5. Add Typed Acceptance Receipts

Add durable `acceptance_receipts` keyed by runtime result id.

Commit shape:

- accepting a role/reorient result writes both old evidence and new receipt
- no old behavior deleted yet
- validation prevents malformed receipt ids

Purpose: build the replacement for summary-string identity.

### 6. Switch Acceptance Checks To Receipts

Use typed receipts for already-accepted role/reorient detection.

Commit shape:

- receipt path authoritative
- optional compatibility fallback for old state emits a warning/note
- tests prove duplicate acceptance is blocked by result id

Purpose: make acceptance identity real.

### 7. Delete Summary-Matching Acceptance Fallback

Remove summary-string detection once receipts are used by all live accept paths.

Purpose: delete the cursed machine. No retirement party.

### 8. Introduce Runtime Links

Add durable `runtime_links` that contain only stable association fields:

- role/scope
- runtime job id
- runtime result id when known
- related subgoal/graph ids when needed

Do not copy lifecycle status.

Commit shape:

- writes both old `jobBindings` and new `runtime_links`
- runtime-spine remains source of lifecycle truth

Purpose: prepare to demote `jobBindings`.

### 9. Switch Result Read-Back To Runtime Links

Use runtime-spine plus `runtime_links` to read role/reorient results.

Commit shape:

- `jobBindings` no longer needed for result lookup
- job view derives lifecycle from runtime-spine

Purpose: make runtime-spine the job table in practice.

### 10. Delete Or Shrink `jobBindings`

Remove lifecycle/backend fields from `jobBindings` or delete `jobBindings`
entirely if `runtime_links` covers the live need.

Purpose: stop mirroring runtime state in durable project truth.

### 11. Remove Public `raw_result`

Replace normal finding `raw_result` with typed fields plus artifact refs.

Commit shape:

- normal role/reorient result responses no longer expose raw payloads
- explicit forensic/debug path may read sealed artifact refs
- Aquarium/status surfaces consume typed projection only

Purpose: restore the operator-safe boundary.

### 12. Collapse View Endpoints Behind `thread/epiphany/view`

Add `thread/epiphany/view` with selectable lenses. Keep old endpoints as wrappers
only until their consumers have moved; wrappers are quarantine scaffolding, not
heirlooms.

Commit shape:

- new view endpoint is canonical for new consumers
- old wrappers survive only while a named consumer still needs them

Purpose: stop protocol sprawl without a flag-day break.

### 13. Migrate Consumers To `thread/epiphany/view`

Move MVP status/Aquarium/status bridge consumers lens by lens.

Commit shape:

- old endpoint wrappers remain only while migration is incomplete
- each consumer move is independently reversible

Purpose: reduce dependency on the exploded API.

### 14. Delete Old View Endpoint Wrappers In Batches

Once consumers are off wrappers, remove obsolete endpoints in small groups:

- scene/context/graph/planning
- jobs/roles
- pressure/freshness/reorient/CRRC/coordinator

Current scar: pressure, reorient, CRRC, and coordinator standalone read wrappers
are already deleted. Their live read surface is `thread/epiphany/view`; the
reorient launch/result/accept verbs remain because they are authority and review
gates, not duplicate reflection.

Purpose: actually reduce the surface area instead of preserving a compatibility
museum forever.

### 15. Split Heartbeat Stable And Experimental State

Keep typed scheduler fields in stable heartbeat state. Move cognition
thought-weather into typed receipts or artifact projections unless a field
directly drives scheduling policy.

Purpose: stop storing experimental blobs as stable doctrine.

### 16. Move Tests To Contract Level

For every extracted policy, move assertions into `epiphany-core`.

App-server tests should prove:

- params parse
- errors map correctly
- app-server passes the right inputs to core
- responses serialize

Purpose: tests should guard architecture, not embalm the old host-seam tumor.

## First Concrete Slice

The smallest coherent first implementation slice is:

1. add `epiphany-core/src/surfaces/`
2. move pressure derivation into core
3. leave `thread/epiphany/pressure` protocol unchanged as a wrapper only for the first slice
4. move pressure policy tests into core
5. run the existing pressure smoke

The first ruthless slice immediately after that is typed acceptance receipts.
Summary-string identity is not quaint. It is a small bureaucratic demon made of
string comparison, and it should be removed before more worker/result surfaces
depend on it.

## Success Criteria

The rebuild is working when:

- app-server Epiphany code is mostly routing/adaptation
- coordinator/CRRC policy lives in `epiphany-core`
- runtime-spine owns job lifecycle without durable mirror fields
- duplicate acceptance is prevented by typed receipts
- normal public findings contain no raw worker payload
- Aquarium can read one coherent view surface instead of a private verb zoo
- heartbeat scheduler state is typed and cognition experiments are quarantined
- adding a new view lens requires a core contract, not copying a mapper cluster

That is the smallest machine that still deserves the name Epiphany.

## Completion Audit

As of 2026-05-12, the documented rebuild migration is complete enough to
defend:

- app-server Epiphany code is routing/adaptation for the MVP surfaces, runtime
  loading, and protocol response assembly.
- scene, jobs, roles, planning, pressure, reorient, CRRC, coordinator,
  freshness, context, graph traversal, role/reorient result interpretation, and
  role/reorient acceptance policy have core contracts in `epiphany-core`.
- runtime-spine owns job/result lifecycle identity through `runtime_links`;
  durable `jobBindings` are only authority slots.
- live duplicate acceptance is keyed by typed acceptance receipts.
- normal public findings project typed fields and artifact refs rather than
  raw worker payloads.
- Aquarium-facing read models use the canonical `thread/epiphany/view` lens
  plus bounded read-only freshness/context/graph/result surfaces.
- heartbeat state is typed and cognition experiments remain quarantined.
- adding a new view lens now means adding or reusing a core surface contract,
  not copying another app-server mapper pile.

Remaining work after this point is outward product/bridge/dogfood work or a new
documented migration, not this teardown plan wearing a fake mustache.
