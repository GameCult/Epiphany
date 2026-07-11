# Native Coordinator Liberation Map

## Objective

Make `epiphany-mvp-coordinator` an Epiphany-native control loop. It must read
and mutate typed Epiphany state, launch and inspect runtime work, and accept or
refuse findings without sending `thread/epiphany/*` JSON-RPC requests through
Codex.

Codex remains the OpenAI authentication/model-transport bridge and may retain
legacy app-server wrappers for named Codex clients. It is not the coordinator's
nervous system.

## Current Mechanism

The coordinator is compiled from
`epiphany-core/src/bin/epiphany-mvp-coordinator.rs`, but starts a Codex app-server
process and uses `status_cli::AppServerClient` for the authoritative loop:

- state bootstrap and revision writes use `thread/epiphany/update`
- freshness and coordinator projections use `thread/epiphany/freshness` and
  `thread/epiphany/view`
- specialist lifecycle uses `thread/epiphany/roleLaunch`,
  `thread/epiphany/roleResult`, and `thread/epiphany/roleAccept`
- recovery lifecycle uses `thread/epiphany/reorientLaunch` and
  `thread/epiphany/reorientResult`

The typed machinery already exists below that shell:

- `epiphany-core/src/thread_state_store.rs` owns CultCache-backed typed thread
  state persistence.
- `epiphany-core/src/runtime_spine.rs` owns typed job, launch, result, event, and
  receipt persistence.
- `epiphany-core/src/agent_launch.rs` builds typed role and reorientation launch
  documents.
- `epiphany-core/src/surfaces/role_result.rs` interprets typed findings and
  constructs acceptance bundles.
- `epiphany-core/src/mind_gateway.rs`, `hands_gateway.rs`, and the receipt proof
  machinery own admission, action, and verification contracts.
- `epiphany-codex-bridge/src/mutation_service.rs` currently composes much of
  this policy behind an `EpiphanyMutationHost`, but its host contract still
  assumes a Codex-hosted compatibility snapshot and persistence hook.
- `vendor/codex/codex-rs/app-server/src/codex_message_processor/
  epiphany_mutation_routes.rs` is already mostly a transport shell over that
  bridge service. It should become a compatibility caller of the native owner,
  not the place where the owner remains hosted.

## Authority Map

### Owner

One Epiphany-native coordinator service owns the control decision and delegates
typed persistence to the thread-state store and runtime-spine.

### Inputs

- typed operator/coordinator intent
- CultCache-backed `EpiphanyThreadState`
- runtime-spine job, result, and receipt documents
- typed watcher, retrieval, token-pressure, and dynamic prompt context
- model events returned by the model-transport adapter

### Outputs

- revision-gated typed state updates
- typed runtime launch and interrupt receipts
- typed role/reorientation findings
- Mind review and state-commit receipts
- read-only typed coordinator, freshness, role, and recovery projections

### Derived State

JSON-RPC responses, app-server notifications, Aquarium panels, CLI JSON, TUI
rows, CultMesh mirrors, and operator summaries are projections. They do not own
lifecycle, scheduling, acceptance, prompt doctrine, or durable truth.

### Forbidden Writers

- `codex_message_processor.rs` and its route modules
- Codex thread/session prompt machinery
- JSON-RPC request/response types
- GUI/Aquarium clients
- CultMesh mirrors and status snapshots
- watcher or reorientation verdicts
- raw model or MCP JSON

Each may submit or translate a typed intent. None may preserve an independent
decision after the native owner responds.

### Shared Paths

Manual CLI actions, programmatic coordinator turns, heartbeat scheduling,
Aquarium actions, and legacy Codex JSON-RPC calls must use the same native
operations for:

1. state read/update
2. freshness and coordinator derivation
3. role/reorientation launch
4. result readback
5. accept/refuse
6. interrupt

### Deletion Line

Delete `status_cli::AppServerClient` from the coordinator control path. Delete
its direct `thread/epiphany/*` request construction. After the native loop is
proved, reduce Codex mutation routes to protocol parsing, native service calls,
response mapping, and compatibility notifications. Then remove bridge policy
that exists only to compensate for Codex-hosted state.

## Migration Order

This is one vertical ownership migration, landed in buildable cuts. No cut may
introduce a second authoritative store or a native/JSON-RPC mode flag.

1. Move route-independent mutation/launch/accept orchestration out of
   `epiphany-codex-bridge` into an Epiphany-native service whose host ports are
   typed state persistence, runtime-spine, clock, and context inputs. Preserve
   the existing bridge service as a thin delegating compatibility wrapper.
2. Add native read operations for state, freshness, coordinator/role views, and
   typed result readback over the same stores used by mutation operations.
3. Rewire `epiphany-mvp-coordinator` to the native service and remove
   `AppServerClient` plus every `thread/epiphany/*` request from the binary.
4. Prove bootstrap, manual/programmatic launch, pending/completed result,
   acceptance, refusal, interruption, and recovery against native stores.
5. Make Codex JSON-RPC routes delegate to the same service. Remove duplicated
   route/bridge policy and mapper-only tests that no longer protect a boundary.

## Native Organ Boundary

The native service is a façade, not a new host brain. The first extraction
proved the authority direction but allowed `epiphany-core/src/coordinator_service.rs`
to grow to 1,122 lines, almost the same size as the 1,143-line bridge mutation
service it is starving. That shape is rejected. Do not continue pouring bridge
functions into one core file.

The native body must be split before further authority moves:

- coordinator state owner: CultCache thread-state load, revision gate,
  validation, write, and changed-field report
- coordinator result reader: runtime-link selection, runtime-spine lifecycle,
  typed finding interpretation, and result notes
- coordinator acceptance organ: completed-finding admission, launch-contract
  proof checks, role/reorientation acceptance construction, receipt persistence,
  proof enforcement, and Mind commit
- coordinator launch organ: typed dynamic context, launch document construction,
  runtime-spine opening/interruption, and state linkage
- coordinator projection organ: freshness, role board, CRRC, and coordinator
  views derived from typed inputs without persistence
- coordinator façade: narrow composition over those organs; no policy bodies,
  protocol DTOs, JSON, or Codex host hooks

Each organ owns one invariant and must be testable against injected paths and
typed inputs without starting Codex. A source guard should fail if the façade
regrows policy bodies or if any native organ imports app-server protocol types.

Launch-context ownership is now native. `coordinator_launch_context` owns local
Verse context, memory-graph cuts, Hands consequence evidence, Soul/Modeling
work-loop telemetry, and bounded prompt rendering. The compatibility bridge's
`launch_context` module is only a re-export membrane. Coordinator rewiring must
use these same functions; a context-free launch is not an acceptable shortcut.

## Acceptance Transaction Wound

The compatibility bridge currently persists prerequisite receipts, writes
thread state, then writes the Mind state-commit receipt in a separate
runtime-spine store. If the final write fails, accepted durable state survives
without its promised commit witness. Moving that sequence into core unchanged
is forbidden.

Before final acceptance persistence moves, define one typed transaction law:
either co-locate state admission and its commit witness in one CultCache
transaction boundary, or persist a prepared acceptance transaction whose
recovery protocol can deterministically finish or refuse admission after
interruption. Best-effort rollback across two files is not atomicity. A negative
smoke must inject failure after prerequisite receipts and after state write and
prove no unwitnessed accepted state becomes authoritative.

## Launch Transaction Law

Launch planning now belongs to `coordinator_launch`: it validates the expected
thread revision, inspects the prior runtime binding, prepares the heartbeat
launch documents, and derives the new thread-state linkage without writing.
The Codex bridge is a compatibility actuator for that plan; it no longer owns
those policy decisions.

`commit_coordinator_job_launch` is the single launch publication owner. It
prepares runtime identity, session, job, opened event, worker request, optional
Research Substrate Gate grant, and linked thread state against one loaded
CultCache snapshot, then publishes the heterogeneous set in one atomic batch.
The compatibility bridge cannot write any member separately. Injected batch
refusal proves the prior snapshot survives with no job, worker request, grant,
or new state linkage. Rollback and reconciliation are not owners.

## Invariants

- There is one durable thread-state owner and one runtime lifecycle owner.
- Revision validation is identical for CLI, coordinator, heartbeat, Aquarium,
  and compatibility JSON-RPC callers.
- A view or watcher verdict cannot launch or accept work.
- Raw JSON is parsed once at a hostile edge and does not cross native service
  ports.
- Model transport cannot inject organ doctrine, scheduler decisions, or state
  mutation policy.
- Compatibility wrappers cannot override or repair the native result after the
  fact.

## Verification Layer

The proof must observe the stores and receipts where the invariant lives:

- unit tests for each native operation with injected state store, runtime
  store, clock, and context inputs
- adjacent-operation smokes for launch -> result -> review -> state commit
- a native coordinator smoke with no app-server binary available
- a compatibility smoke proving JSON-RPC delegates to the same native result
- negative tests proving Codex routes cannot write around revision/admission
  gates and the coordinator binary contains no `thread/epiphany/*` methods

The migration is not complete because the coordinator eventually converges on
the same state after an app-server round trip. The old path must be structurally
unable to decide the result.
