# Epiphany Native Implementation Plan

This is the live campaign plan. The Codex-fork phase succeeded by starving
Codex of Epiphany authority; Epiphany is no longer implemented as a collection
of app-server routes.

## Objective

Build an inspectable native organism whose typed state, runtime, organ gates,
memory, scheduling, interfaces, and social crossings operate without Codex
owning Epiphany cognition. OpenAI subscription authentication and model
transport belong to the independent CodexConnector daemon, not Epiphany's body.

## Current Mechanism

- `epiphany-core` owns the semantic contracts, keyed Mind admission, runtime spine, organ gates,
  Resident Self physiology, Persona, and CultMesh integration. Coordinator policy,
  status, and every migrated work family consume keyed Mind plus exact runtime
  receipts. Persisted `EpiphanyThreadStateEntry`, the in-memory
  `EpiphanyThreadState` schema, generic launch/update transactions, and global
  launch revisions have been deleted with no compatibility path.
- Native binaries expose coordinator, status, state, model runtime, Persona,
  repository Body observation, governed work, and Verse operations. Deployment
  and daemon survival belong to Idunn; retrieval providers remain external
  evidence sources until a typed reasoning consumer earns them.
- CultCache stores typed state and receipts; CultMesh/CultNet carry typed local
  and federated projections.
- Codex implementation source is absent from Epiphany. The typed Connector
  contract is the only Codex-facing compile dependency.

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

- Typed reasoning bases, terminal decision contexts, exact Mind batch CAS, and
  keyed objective/RepoModel documents.
- Native coordinator facade and derived status/recommendation surfaces.
- CultCache runtime spine with worker, Mind, Eyes, Substrate Gate, Hands, Soul,
  Continuity, and coordinator receipts.
- Native OpenAI auth/model/runtime spine.
- Native heartbeat, daemon supervision, and cluster liveness surfaces.
- Native Persona memory/turn/audit machinery and Bifrost-governed public mouths.
- Native repo-work planning, Hands/Soul/Mind closure, public-proof, credit, and
  Bifrost accounting families.
- CultMesh local Verse, compact operator readbacks, Gjallar sight, Eve
  connection receipts, and three-Verse trust boundaries. Organ contract
  summaries derive directly from their CultNet owners; CultMesh persists no
  parallel contract directory. Fixed Verse and public-room policy project
  directly into the local context rather than occupying mutable store rows.
- Native runtime and keyed Mind receipts project without JSON-to-CultMesh
  shadows. Unauthenticated pre-provider Odin/Eve/tool rows and their retirement
  migrators are outside the live schema.
- Complete removal of the Epiphany Codex app-server compatibility surface and
  `epiphany-codex-bridge`.

## Active Campaign

### 0. Federate Decision-Auditable Mind

Status: source-complete; exact-package capstone open. The persisted thread head,
global revision transaction, aggregate RepoModel, generic Mind gateway, and
dual readers are deleted. Concrete invariant owners submit exact-envelope
`MindMutation` plans; CultCache batch CAS merges disjoint identities and refuses
same-identity or changed-strong-read conflicts without partial writes.

- Preserve the sealed reasoning basis, exact final native/provider request,
  governed tool observations, structured terminal decision/failure, and Mind
  commit receipt.
- Keep family requests and admission owners concrete; share only pure identity,
  continuation, binding-validation, and CAS invariants.
- Derive work from unresolved typed obligations. Events, lanes, timestamps,
  thread provenance, and aggregate view digests remain projections only.
- Assemble RepoModel and semantic work from complete keyed document sets.
  Fence namespace phantoms at the operation snapshot; do not invent a global
  head to make set membership convenient.

Exit evidence:

- concurrent Persona, Hands, Modeling, and Verification writes to disjoint
  identities all commit;
- same-key conflict, changed strong reads, and exact replay follow the declared
  merge law;
- every model-backed success or failure can be audited to its exact sealed
  projection without transcript retention;
- source guards reject persisted thread state, aggregate RepoModel authority,
  caller-authored provider storage, and generic Mind mutation mouths.

### 1. Repair Modeling

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
verdict without an explicit model-authored finding. That finding now persists
as a typed runtime document backed by a passing Soul receipt; Mind rereads it,
and repo-map/CultMesh admission carries its receipt ID. Repo-map admission is
now atomic: the typed canonical map entry and both Mind witnesses publish in one
CultCache batch, while CultMesh lowers the committed entry. The custom map
MessagePack owner is deleted. Stable phase IDs are immutable: identical retries
reread existing Soul, Modeling, and Mind/map documents, while conflicting retry
cargo fails without changing admitted state.

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
  explicit-pressure-only wakeup, and recovery across restart.
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
- Demonstrate Persona discussion with keyed typed memory, consent, disagreement,
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

Keep the five-day shakedown, Ox17 deployment, and Model Atlas operational Gate
1 paused. Exact `a721b763` leaves Epiphany with one native model event/receipt
family and one package owning native contracts plus pure provider lowering.
Audit the library-only `epiphany-openai-runtime` package next. Its production
model and Persona entrypoints already belong to `epiphany-release-bundle`; if
the separate package protects no independent compile, dependency, or test
invariant, make it the root package's library target and delete its manifest
and dependency edge.

Preserve exact provider-request sealing, the lean Connector client ABI,
OpenRouter's separate temporary release edge, keyed Mind audit/CAS guarantees,
direct Body-to-Modeling flow, Persona state, and the complete unrun Model Atlas
slice. No Epiphany-owned Rider or Unity bridge exists: Brokkr owns Unity through
typed Eve/CultMesh capabilities, and a future Rider daemon owns Rider.

The provider decision is the independent `GameCult/CodexConnector` service;
exact `6dc80f6d266db4d82566d2434adcc55a48e8ecad` owns the v2 multi-caller contract,
exact native/provider digest binding, typed tool and terminal receipts, the
pinned official Codex credential child, raw Responses transport, and durable
keyed replay. No features yields only the typed contract; default `client` adds
authenticated framing and socket transport; `daemon` adds service authority.
It links no Codex crate. Active replay identity is written before network
access; the exact encrypted completion is durable before reply; restart
ambiguity refuses rather than re-executing. Yggdrasil now runs Connector as an
independent service and Idunn target for Ghostlight. One digest-bound Epiphany
round trip remains unaccepted while Epiphany's selected edge is OpenRouter.

Epiphany pins the contract-only surface in its adapter and enables the client
only at its release edge. Exact `87ea81db` also deletes Epiphany's obsolete
public schema for the extinct generic provider request; the shared Codex
contract belongs to Connector, while Epiphany's wrapper remains private Mind
state. Ghostlight exact `8e7d980` pins the same Connector source; its live Ygg
deployment consumes the independent service without owning it. Exact `7a125d78` removes the excluded 3,592-file
Codex implementation tree and its obsolete keeper notes; gamecult-ops exact
`cb63e77` repairs the independent deployment ownership map. No dual daemon or
refresh writer is an acceptable transition state.

Model Atlas has component proofs but no live inter-swarm collaboration run.
Preserve its publication, trust, transport, projection, Soul verification, Self
admission, wake, brake, and Eve path until Gate 1 exercises the complete crossing.
Streamlining may remove duplicate authors and process shells; it may not spend
the only end-to-end collaboration path before it has carried real traffic.

Exact `6f5d6600` reduces the immutable repository-domain receipt to the
canonical organizational repository name and exact authenticated Body hash.
Runtime/swarm/workspace belong to the Body route; envelope type/key/schema
belong to CultCache; binding time and fixed prose own nothing. The typed receipt
still prevents deployment input from relabeling the Body after admission.

Exact `952dcd9f` reduces operator-objective intake to the human assertion and
its actual provenance. CultCache owns envelope identity and schema; the
singleton Mind objective owns active state; the exact Mind commit receipt owns
atomic admission. One coordinator-binary test that only repeated the library
proof is gone, while byte-idempotency, replacement refusal, and resident-grant
provenance remain covered.

Exact `718ce9c1` applies the same ownership rule across the complete durable
repository-frontier chain. Thirteen Proposal, Planning, Research, PlanMind,
Hands, Verification, Soul, and Modeling documents no longer persist fixed
schema/version labels or contract slogans already owned by CultCache and the
runtime epoch. Their semantic identities, strong dependencies, exact source
versions, decision contexts, evidence, dispositions, and consequences remain.
Sealed worker contexts retain explicit schema and contract fields because they
are exact model input. Runtime advances to v33. Atlas is untouched and remains
protected until a real inter-swarm collaboration run exercises the complete
crossing.

Exact `a4356d1f` roots runtime schema authority in runtime identity plus swarm
binding and removes fixed schema-version echoes from thirteen child physiology
documents. Sessions, jobs, execution bindings, launches, process claims,
results, coordinator receipts, death recovery, and archives retain their exact
causal and lifecycle payloads. Runtime advances to v34; old-store refusal and
all affected consequence paths remain proven. No compatibility layer replaces
the deleted fields. Atlas and external crossing contracts are unchanged.

Exact `e60b924f` deletes the model runtime's callerless preflight JSON
self-certificate and registered-type catalog accessor. Exact `3b991e40`
collapses the packaged model runtime onto admitted `run-worker` plus
`list-decisions` and `audit-decision`. The three uncalled direct transport/tool
CLI paths, their parser/option types, and their self-contained ingress tests are
gone. Provider execution and tool continuation now have one production entry
path; transcript-free decision inspection remains. The two cuts remove 532 net
maintained lines without changing Mind/runtime schemas or Atlas.

Resident Self now solely imports authenticated Bifrost deliveries; exact
`3dda58a5` deletes the standalone Persona feedback ingress and its old Starfire
snapshot seam. Exact `56267201` deletes the three unadmitted Atlas daemon
shells while retaining their typed library owners. Exact `900c5232` deletes the
callerless frontier-proposal wrapper while retaining typed Self/runtime intake.
Exact `94098223` reduces host identity to provider-owned public-anchor
verification and deletes Epiphany's private-custody stack. Exact `b78ffb25`
then deletes the callerless Hands consequence recorder and command-description
handshake after proving that they executed nothing. Exact receipt-chain
admission remains; the coordinator reports `awaitingHandsExecutor` until a real
actuator owns execution and observed consequences. Exact `f3360248` deletes
three redundant helper-spelling/cache-policy tests while retaining the actual
source-cache recovery, cache-separation, tool-loop transition, and terminal
failure proofs. Exact `a276d0f4` then collapses the generic public host-identity
verifier into Bifrost feedback admission while preserving the exact existing
wire shape and cryptographic domains. Next audit the remaining eleven
executables by real lifecycle/privilege consumer, beginning with the Persona
Discord permit process.

Exact `f8412b69` closes the last known retention wound before that capstone:
the packaged model runtime can reconstruct a decision by context ID from its
sealed basis, exact terminal requests, tool observations, structured terminal
records, and Mind commit receipts. Worker attempt archival retains the typed
result family instead of reducing it to IDs and a digest. Runtime schema v2
refuses the superseded writable archive shape; there is no reader or migrator.

Exact `553f79d9` closes the provider-schema ownership wound exposed by the
packaged capstone. One shared Responses dialect compiler now serves role and
Persona passes: it removes provider-illegal conditional structure, preserves
supported standard-model constraints, compiles UUID formats into enforced
patterns, and leaves the full native schema as the only admission contract.
The failed package roots remain immutable evidence; no role-specific output
exception was added.

Exact `a8f3c1f0` closes the next packaged falsification at the RepoModel seed
owner. Repository bootstrap had passed immutable Git source identity where the
keyed model contract required the authenticated runtime Body binding. The
bootstrap now derives that value from the admitted Body observation, while the
RepoModel initializer independently validates the complete route before any
envelope is written. A hostile seed refuses byte-identically; no migration or
fallback identity exists.

Exact `bb823c54` closed the two wounds found by that native replay. One shared
model-pass terminal owner atomically binds a typed failure to its sealed
decision context and closes the exact Persona session. Its derived semantic
cache design was later proven unconsumed and deleted wholesale at `856648de`;
the sealed typed Body/RepoModel basis remains the current Modeling input.

Cut in this order:

1. **Landed at `79346523`:** replace the aggregate-shaped role reasoning input with one sealed typed
   projection assembled from exact keyed Mind document versions. The final
   model request must render that projection; citing keyed sources while
   rendering the old thread snapshot is explicitly forbidden.
2. **Body admission landed at `2eb95df6`; current-work projection landed at `e42788c9`, consumes one exact snapshot at `7374be5e`, typed fulfillment reuses it at `5f1bea39`, coordinator scheduling becomes its sole consumer projection at `2bcbf268`, reorientation result review becomes a single-cache typed read at `125d77a2`, and terminal disposition stays in that projection at `dfe3757b`:** one pure typed projection derives Body Modeling, Eyes continuation, proposal Modeling, frontier planning, Hands readiness, accepted manual-regather obligation, and exact Reorientation success/failure disposition from one pulled keyed Mind/runtime cache. Immutable result replay, family admission, PlanMind continuation, worker archival, Resident Self completion, and operator status validate or route against that same projection rather than reopening runtime state or accepting a parallel CRRC action. Reorientation result review likewise reads its job and structured result from one runtime snapshot and exposes no synthetic backend lifecycle.
   Modeling must derive from the current Body/RepoModel obligation; Eyes only
   from explicit external-evidence obligations. Body-generation Modeling
   identity, typed Body-to-Mind admission, sealed reasoning, and atomic decision
   admission are landed. Current-work no longer reads the external Body store;
   status consumes the shared projection. Exact `0a97eef8` deletes the later
   callerless Body-only/unresolved-Body reconstruction paths and three unused
   per-job continuation wrappers, leaving `project_current_work` as the sole
   slice assembler.
   Exact `5a047944` also removes the callerless JSON ledger-status helper,
   generic role lookup helpers, unique-string wrapper, and successful-receipt
   wrapper. Live typed owners remain; no generic utility replacement is added.
   Exact `6ccaf937` removes the one-field repository Body Mind-view alias, the
   unconsumed sorted Persona-quarantine projection, and the unconsumed generic
   repo-work planning grant. Typed Mind/quarantine owners and exact live
   coordinator/worker grants remain.
   Exact `f6a2ad7f` removes the callerless packaged-release-head and Idunn
   trust-anchor file readers. Exact release authentication and typed signed
   health admission remain with their live consumers.
   Exact `be611f24` removes the producerless Imagination consideration-review
   writer and durable contract together. Candidates remain proposal-only
   outcomes; actual adopted proposal Modeling stays in its existing lifecycle.
   Runtime writable epoch is v22.
   Exact `3140d305` removes the producerless Resident Self runtime-receipt
   document and the definition-only coordinator role-result enum. Resident Self
   state v2 refuses v1 stores; actual terminal receipts and runtime retention
   remain with their live owners.
   Exact `0091e0e1` removes the unread generic/OpenAI adapter-status documents,
   their per-pass writers and construction helpers, one self-affirming test,
   two schemas, and both runtime registrations. Exact request/provider context
   and live transport construction remain. Runtime writable epoch is v23.
   Exact `2e3489c4` removes four uninhabited generic planning document families,
   their empty reasoning-projection field, and the 256-line issue-tracker DTO
   vocabulary beneath them. Actual frontier planning and adoption remain.
   Mind/runtime writable epochs are v9/v24.
   Exact `a6f73fc4` removes the write-only Persona quarantine-pressure document.
   The terminal turn receipt and receipt-bound quarantined mentions remain the
   single blocked-consequence authority.
   Exact `fed4b857` removes the unread Continuity recovery-receipt companion and
   its entire module. Exact reorientation result, typed Mind decision/failure,
   sealed context, and commit receipt remain the sole recovery audit path.
   Runtime writable epoch is v25.
   Exact `a602fbdc` removes fifteen unconsumed state-model DTO types, two
   callerless OpenAI-runtime mouths, and eight definition-only type constants.
   Exact keyed Mind/runtime receipts, RepoModel documents, pass-local reasoning,
   and the release-owned observed worker route remain. No schema epoch changes
   because none of the deleted vocabulary was registered or persisted.
   Exact `9abadfe7` removes the unconsumed provider-frame observation DTO and
   callback APIs from both transports, plus the provider-specific transcript
   reader used only by one test assertion. Typed stream events/receipts remain;
   no audit or admission path depends on frame previews or transcript text.
   Exact `2a435eb5` advances Persona memory to v2 and tool receipts to v1 while
   deleting duplicate effect-event provenance, an always-empty relationship,
   and an always-empty raw-result reference. Mind/runtime epochs are v10/v26;
   old writable stores refuse without migration.
   Exact `ffd91c20` deletes three more derived fields: the fixed Research role
   from the then-live Eyes packets, the fixed Epiphany Persona agent from queued
   mentions,
   and the unread Resident Self retention timestamp. Their owning request/
   context, Persona queue/turn request, and chained retention digest remain.
   Persona mention v2, Resident Self state v3/retention head v1, and runtime v27
   formed that hard cut; exact `83611f9b` later deletes the Eyes packet family.
   Model Atlas remains intact for its first
   live inter-swarm collaboration run; component proofs are not acceptance.
   Exact `fe118e13` makes provider events and receipts transient transport
   values and deletes their duplicate CultCache/CultNet authority. One native
   model stream/receipt family now owns continuation, reconstruction, terminal
   validation, and retention; the exact provider request remains durable audit
   input. Runtime advances to v28, the catalog drops to 18 schemas, and the
   model-runtime include shell is gone. Atlas remains unchanged and unaccepted.
   Exact `dbed11b3` and `8bb0719b` delete the remaining hand-maintained schema
   shadow registry. Runtime-local CultCache documents are discovered from the
   native registration or projected by their owning provider; they are not
   republished as stale JSON. The catalog now contains only the exact provider
   request plus the portable Persona and work-organ contracts. The cut removes
   850 net maintained lines. Atlas remains unchanged and unaccepted.
   Exact `695af6c6` deletes thirteen unowned dependency declarations and the
   final tracked marker for the extinct OpenAI auth-spine crate. The lock graph
   contracts from 757 to 732 packages; all nine production entrypoints and the
   affected library/test targets compile individually. Package-scoped cleanup
   removes 124.2 GiB of reproducible artifacts while preserving the prebuilt
   state tool. Atlas remains unchanged and unaccepted.
   Exact `65bf044f` removes redundant schema/prose cargo from the three Hands
   consequence receipts, fourteen unused shadow type constants, and one
   callerless reorient display enum. CultCache owns document type/schema
   identity; exact intent, review, grant, job, consequence, and immutable
   receipt identity remain. Runtime advances to v29. The concurrent
   Persona/Hands-to-Verification proof and old-epoch refusal pass; package
   cleanup removes 678.6 MiB. Atlas remains unchanged and unaccepted.
   Exact `d9a196a0` makes the Proposal-Modeling request the sole owner of
   autonomous proposal origin. The companion binding document, schema,
   registration, third CAS write, collision branch, and copied worker hashes
   are gone; exact direction-result, option-ordinal, and worker-job identities
   live on the request. Runtime advances to v30. The full keyed lifecycle and
   old-store refusal pass; package cleanup removes 677.6 MiB. Atlas remains
   unchanged and unaccepted.
   Exact `fdefb889` deletes the generic session/job writers after proving every
   caller was a fixture. Family launch CAS, model execution, and coordinator
   opening are the only runtime-physiology creation owners. The cut removes 96
   net maintained lines without a schema or Atlas change.
   Exact `b3e9b229` deletes Persona's standalone delivery-evidence document and
   the conversation receipt's copied evidence-ID vector. Bifrost owns the
   durable signed crossing receipt; the Persona social terminal owns the local
   delivered consequence and folds the exact message, crossing, and digest
   identities into one receipt. Retention verifies that provider-owned receipt
   directly. Runtime advances to v36 and both changed Persona documents make a
   v2 hard cut. Atlas remains untouched and operationally unaccepted.
   Exact `ce6bff12` reduces runtime execution bindings to their owned
   request-or-intent to session/job edges. Provider, worker, reasoning-basis,
   and model-request ancestry are loaded from the native request or typed tool
   intent instead of copied into bindings; non-causal binding timestamps and
   their API arguments are gone. Runtime advances to v37. The cut removes 67
   net source lines. One focused OpenAI test generated 12.713 GiB across 19,134
   files, so its provider/runtime dependency boundary is the next source-shape
   audit. Atlas remains protected for its first live inter-swarm run.
   Exact `a6cf9383` resolves that boundary: concrete Codex/OpenRouter auth,
   credential reading, and transport move to one release-entrypoint source
   module, while `epiphany-openai-runtime` retains pure exact-request opening,
   event admission, tool intent emission, and durable audit with no Codex
   dependency. Persona's concrete runner moves to the Persona service. The
   duplicate no-tools worker route, DTOs, CLI mode, and coordinator switch are
   deleted; one governed tool-capable path remains. Provider strict-shape tests
   move to the release owner. The cut removes 176 net source lines with no new
   target, process, schema, or epoch. The same runtime consequence proof now
   uses 1.869 GiB/2,942 files instead of 12.713 GiB/19,134. The release edge
   still reaches 10.473 GiB/16,483 files through `codex-login`, which is the
   next provider-edge liability. Atlas remains untouched and unaccepted.
3. **Baseline transaction landed at `9f7b164f`; operator routing at `587c56d2`; Resident continuation and keyed acceptance at `478fb923`:** baseline Body Modeling is now thread-free from unresolved Body obligation through Launch/Wait/Review and exact Mind admission. Resident and operator callers consume the same family projection and acceptance owner. Thread ID remains immutable pass provenance only.
4. **Proposal Modeling landed at `d1b031cb`:** its immutable request, worker launch, runtime attempt/result, and `Modeling.proposal_frontier` commit now own Launch/Wait/Review and admission. The aggregate launcher refuses proposal cargo; the old selector, validator, context builder, and generic-launch hint are deleted. Body and Proposal share only the exact-envelope launch CAS invariant, not a mutable family registry. Exact `b0a4978d` deletes the later duplicate launch-binding document entirely; the worker launch is the sole launch owner.
5. **Continuation vocabulary landed at `e404c105` and was named truthfully at `fc5d1a4a`:** concrete pass families share one pure `EpiphanyAgentPassContinuationAction`; their semantic requests, projections, validators, and admission owners remain separate.
6. **Dormant claim-repair authority deleted at `20dd66c8`:** no request/binding/context schema, runtime carrier, coordinator branch, result cargo, export, or model lowering survives. The independent Eyes challenge fact remains. The same pass fixed two-snapshot Body/Proposal launch derivation and stress-proved single Proposal launch 20/20.
7. **Frontier-verdict Modeling landed at `d367e525`:** one exact Soul verdict
   atomically creates its deterministic request; the sealed role projection,
   deterministic attempt, shared Launch/Wait/Review state, and
   `Modeling.frontier_verdict` commit own the complete lifecycle. Admission
   accepts disjoint keyed changes but refuses a changed strong frontier without
   mutation. Accepted Eyes timestamps, latest-result slots, aggregate prompt
   construction, and generic role lanes cannot create or suppress this work.
8. **Frontier Research landed at `c7412998`:** the v3 request seals the exact
   frontier/dependency document closure, while the full RepoModel projection is
   audit cargo only. One deterministic attempt, terminal decision context,
   structured Research result and `Eyes.frontier_research` Mind mutation own
   Launch/Wait/Review/admission. Stale terminal decisions remain durable, exact
   strong-state changes refuse admission byte-identically, and disjoint keyed
   writes merge. Accepted Eyes, role lanes, timestamps, generic regather, and
   Modeling hints cannot launch Eyes or gate Modeling. CultCache `ba6a487`
   supplies the shared named nested-document encoding boundary used by Mind.
9. **Frontier Verification landed at `fc5d1a4a`:** one exact Hands commit and
   its deterministic Verification request publish atomically. The sealed
   request/launch/result/context lifecycle owns Launch/Wait/Review; admission
   writes the keyed audit, Soul verdict, frontier-Modeling request, and Mind
   commit receipt atomically. Disjoint keyed writes merge, exact-frontier drift
   refuses byte-identically, and replay is stable. Generic launch/acceptance,
   accepted-at/latest-result selection, dynamic telemetry prompts, and
   model-authored causal IDs no longer own this family.
   Exact `6ccc7dd2` removes the later transient 20-field receipt-chain summary;
   the request is derived directly from its patch, command, and commit owners.
10. **Frontier Planning and PlanMind landed at `c9329ed6`:** the Planning
   request seals exact frontier/dependency versions plus per-claim obligation
   guards. Current-work owns deterministic Imagination and Mind attempts;
   Imagination authors a typed candidate, while Mind alone adopts it through a
   sealed PlanMind request and family-owned frontier mutation. Claim challenge
   admission atomically updates the target claim-obligation document, so stale
   planning conflicts without turning unrelated graph writes into a global
   head. Thread provenance, generic coordinator validators/constructors,
   aggregate role lanes, and accepted-at/latest-result behavior no longer own
   this family. Exact `b55e96ea` deletes both later duplicate launch-binding
   document families. Immutable worker launches now own attempt identity and
   current jobs; exact failed results plus typed failure reviews own retry
   ancestry and are strong reads of the retry launch transaction.
11. **Reorientation landed at `d5df53ae`:** one keyed request seals the exact
   continuity projection and source document versions; deterministic attempts
   bind an exact reasoning basis and terminal decision context. The family
   owner alone admits resume/regather plus its continuity receipt. Model-backed
   failure retains a typed Mind failure document before retry. Status and
   coordinator routing use the same current-work projection. Aggregate
   launch/acceptance, checkpoint/freshness recommendation, latest binding,
   global revision acceptance, and accepted-at comparison no longer own this
   family. Exact `8dbfcf63` deletes its later duplicate launch-binding document;
   the immutable worker launch owns request/job/attempt identity, while retry
   CAS strongly reads the typed failure and exact terminal runtime result.
   The same cut deletes the duplicate Body Modeling and Imagination
   consideration binding families. Canonical job identities and immutable
   family-referenced worker launches now own all three attempt sequences.
   Their semantic requests, sealed projections, result admission, substrate
   grants, typed failures, and refusal/retry invariants remain family-owned.
   Runtime writable state advances to v19. Model Atlas and the unrun
   inter-swarm collaboration path are unchanged.
   Exact `3ed1d564` then deletes the standalone public writer for the derived
   frontier-Modeling request. Accepted Verification is the sole writer and
   atomically commits its audit, Soul verdict, and request. The deleted path had
   no production caller and survived only because an OpenAI test fabricated a
   parallel Verification/Soul tableau; that test now uses ordinary Body
   Modeling while preserving provider, contract, structured-result, and job-
   terminality proofs.
   Exact `63d2991e` then reduces the coordinator terminal receipt to the
   terminal Self decision and exact resident-launch provenance continuity
   consumes. Operator artifact inventory, provider/store echoes, arbitrary
   metadata, step count, and final-job echo stay in their existing owners or
   disappear when unread. Receipt/schema version is v1 and runtime writable
   state is v20; the published schema now describes the live resident fields.
   Exact `196222d9` then makes runtime sessions/jobs minimal lifecycle records:
   session metadata and job metadata, summary, and artifact refs disappear
   because no reader consumed them and the terminal job result already owns
   consequential summary/artifact truth. Launch options no longer accept those
   echoes. Runtime writable state is v21 and the closed session/job catalog
   schemas are v1. Model Atlas remains intact for the unrun live inter-swarm
   Gate 1.
   Exact `cd2177e8` then deletes the duplicate `ensure_runtime_session` writer.
   Its two test consumers now use strict session creation; a reused active ID
   can no longer substitute a different objective, creation time, or
   coordinator note. No persisted shape changes and runtime remains v21.
12. **Landed at `1662a012`, `07d891ba`, and `ec1431ff`:** coordinator policy now
   projects keyed current work; persisted `EpiphanyThreadStateEntry`,
   `coordinator_state_transaction`, generic coordinator services, the
   `EpiphanyThreadState` schema/prompt renderer, aggregate freshness/context
   views, generic launch/interrupt requests, and global launch revisions are
   deleted. No dual reader, bootstrap aggregate, or migrator survives.
13. **Generic Mind gateway deleted at `f7948795`:**
   `MindGatewayReview`, `MindStateCommitReceipt` v0, their generic interpreter
   prompt, runtime registration/read APIs, phantom thought/state-effect/public-
   adoption CultNet mouths, and stale launch receipt profiles are gone.
   CultMesh exposes reasoning basis, decision context, and exact Mind commit
   receipt as read-only audit projections; no runtime schema-catalog mirror
   survives.
14. **Model-backed Planning failure repaired at `1c9aafd8`; generic role patch
   deleted at `e0e75a30`:** the runtime seals
   one exact terminal decision context before writing a typed frontier-Planning
   failure, and the typed failure plus generic job result cite that same
   context. Transport-only failure remains physiological. Research emits one
   closed typed decision and its owner derives keyed Mind writes. The unowned
   generic Imagination planning lane, generic patch parser, and policy
   tribunals are gone.
15. **Decision-context binding unified at `12b1b285`:** runtime execution and
   context sealing consume the same exact model/tool binding validators.
   Provider, tool-call, tool-result, receipt, session, job, basis, and worker
   substitution refuse, and transcript delta deletion leaves the retained
   basis/context usable.
16. **Writable epoch cut landed at `01602fd3`:** Mind epoch v2 and runtime-spine
   schema v1 are admitted only through one exact identity pair. Historical or
   split runtime stores refuse before registered writers see them; foreign
   physiology stores remain outside Mind ownership. No schema migrator or dual
   reader exists.
17. **Keyed concurrency landed at `26b6a5bf`:** simultaneous disjoint
   Modeling-node writes merge; simultaneous Verification admission and an
   unrelated Modeling-node write merge; same-identity competitors yield one
   winner and one typed conflict; exact replay returns the original receipt;
   and reopening assembles one valid graph.
18. **Persona consequence ownership landed at `7c2ebd81`:** Interpreter
   `state_note` output commits keyed Persona-memory documents through its exact
   decision context; the legacy agent-memory aggregate is no longer a writer on
   this path. A real Hands consequence and Persona Mind admission commit
   simultaneously.
19. **Persona ingress landed at `79c0e373`:** one typed Mind pass-input
   document seals the exact assembled projector input and observed source
   versions before inference; every Persona stage cites it, replay reuses it,
   substitution refuses, and generic role projections exclude Persona-private
   documents.
20. **Restart/re-entry landed at `d3300bba`:** one exact validated Mind commit
   receipt authenticates the admitted Persona input. Plan construction is
   sealed behind that loader; naked typed input refuses; terminal replay makes
   no external observation or model call. Current-work projection reopens
   identically and without mutation across Launch, Wait, completed, and
   post-Research boundaries. Build the exact clean-source package and run the
   fresh-store capstone. Only then restart Model Atlas Gate 1 from a new
   external root.
21. **Provider schema ownership landed at `553f79d9`:** one shared Responses
   dialect compiler serves role and Persona passes. Unsupported conditional
   fragments and `uniqueItems` leave provider cargo; supported standard-model
   constraints remain; literal-only variants receive explicit types; UUID
   formats lower to lexical patterns. Full native decoding remains the only
   terminal-decision admission boundary. Three failed packaged roots are
   retained as falsification evidence rather than hidden by a family prompt
   patch.
22. **RepoModel seed ownership landed at `a8f3c1f0`:** the authenticated
   runtime Body route, not Git source identity or caller convention, owns the
   keyed model's Body binding. Bootstrap derives it from the admitted
   observation and initialization reauthenticates it before writing. The exact
   `553f79d9` package proved concurrent Persona plus terminal Modeling cognition,
   then correctly exposed the bad seed during admission. Rebuild and replay the
   capstone from exact `a8f3c1f0`; do not transplant the prior decision.
23. **Terminality and provider-authorship landed at `bb823c54`; its cache projection was superseded by `856648de`:**
   a provider or contract failure seals one `EpiphanyModelPassFailure`; its
   sealed context, not caller arguments, identifies and atomically closes the
   exact model transport job/session. Role, reorient, and Persona failures share
   this owner, while generic transport results carry no decision authority.
   Provider requests are derived internally from native requests at execution
   opening, event recording, and context sealing; the public caller-authored
   provider store is deleted. Reorientation failure admission requires the
   same canonical model-pass failure rather than a generic failed worker result
   plus context. Restart cannot silently infer again. Modeling seals exact keyed
   document versions and their owning Mind commit receipts directly into its
   reasoning basis. The later cache/projector layer that duplicated this source
   set had no reasoning consumer and is gone. Concurrency is protected by keyed
   Mind CAS and exact basis identity rather than aggregate revision or timestamp
   order.
24. **Build/run retention is bounded:** Idunn on upgraded Yggdrasil owns the
   source-triggered compile/test/package/deploy path. Starfire does not run a
   parallel release build. Any containerized run must use exact run labels and remove disposable
   containers, copied-source volumes, and writable roots at terminal
   settlement. Accepted proof stores remain only when this plan or the handoff
   names the invariant they preserve. Exact source
   `470d4cb5b46f94a5490a479dba19604828e1b5d1` passed Idunn's native serialized
   Yggdrasil workspace gate, then sealed 26 binaries plus witness as release
   `sha256-ce4a287ebb915ff9410dd04e022285d82998e0bf4c9acb3b04a44db661aea90c`.
   It is deployed, authenticated-health admitted, and deliberately braked.
   Transient Epiphany containers, volumes, package roots, and the per-run test
   target are absent. The unused Discord/Bifrost/VoidBot operator bridge is
   deleted across source and deployment; Starfire is not a fallback.
25. **Decision discovery is read-only:** exact source `5f66d6c9` adds
   `list-decisions` beside `audit-decision` in the existing packaged model
   runtime. It lists only fully validated terminal decision contexts, persists
   nothing, and omits nonterminal pass physiology. That surface is now inside
   the admitted Idunn-built `470d4cb5` package.
26. **Explicit OpenRouter/Ox transport is live at `6b44b4d3`:** exact
   `ab321b34` makes provider selection typed and keeps the native request
   canonical; the provider request is internally derived and retained in the
   decision context. Exact `c1a6034f` preserves read-only physiology while the
   swarm brake is engaged. Exact `6b44b4d3` publishes Idunn's shared signed
   daemon-health schema. Idunn exact `8ddf8140` validates the canonical generic
   health record; gamecult-ops `a9e9f79`/`0c0dbd1` bind validation to the root
   trust store and inspect systemd credentials in the target mount namespace.
   Ox decisions produced in isolated capstone roots are durable audit evidence,
   not production authority or capstone acceptance.
27. **Future Epiphany builds are native at gamecult-ops `89f0d78`:** the
   earlier disposable Yggdrasil builders and their volumes were removed. The
   admitted `470d4cb5` release was built natively. The installed Idunn actuator invokes
   fixed Rust `1.95.0` directly as `epiphany-builder`; Docker, image, and CID
   authority are absent. This does not manufacture a reason to rebuild the
   already admitted body while cognition is paused.
28. **Current typed state owns routing at `470d4cb5`:** coordinator receipts,
   historical terminal slots, timestamps, and role-lane state are audit/display
   cargo only. Concrete family owners materialize unresolved obligations, one
   pure current-work projection schedules them, and Resident Self pressure is
   content-addressed to that projection plus its recommended action. Ox9 is the
   preserved falsification that forced the deletion.
29. **Body retry identity is normalized at `d48f69b7`:** Body Modeling no
   longer splits launch-only semantic work from a sibling action field. One
   family projection owns the exact work, continuation action, and latest job
   identity through Launch/Wait/Review. Failure therefore changes state and
   produces one fresh deterministic grant; unchanged failure replay remains
   idempotent. Ox10 is sealed failed evidence, not resumable state.
30. **Attempt identity is generalized at `9b9b5c85`:** Body, proposal and
   frontier-verdict Modeling, Verification, consideration,
   admitted-direction consideration, and Reorientation carry one shared typed
   attempt projection. Research and two-stage Planning retain their full exact
   lifecycle projections in current-work instead of lossy action/stage fields.
   Proposal attempt ordinals are canonical and contiguous; older nonterminal
   attempts refuse. The Ox12-shaped test proves one failed attempt changes
   Resident pressure exactly once and the next launch becomes attempt 1. Exact
   `d2ca6630` remains the admitted production source.
31. **Agent admission refusal is replayable at `e046a4d1`:** Body, proposal,
   and frontier-verdict Modeling plus frontier Verification write one exact
   typed refusal and commit receipt when semantic mutation or a strong-read CAS
   refuses a structured terminal result. The result remains truthful model
   transport evidence; current-work excludes it from fulfillment and launches
   a fresh attempt carrying the prior refusal. RepoModel claim targets and
   frontier dependencies are distinct in the output contract. Mind/runtime and
   proposal-context epochs cut to v3/v4/v2, so Ox12 is historical evidence and
   a new fresh-store capstone is mandatory.
32. **Persona owns one outer turn deadline at `3b958a83`:** packaged Ox13
   proved the admission-refusal lifecycle, then four exact Persona projector
   failures exposed a separate 90-second timeout hidden inside the shared
   provider transport. Persona service now accepts one explicit
   `--turn-timeout-seconds` budget, defaults it to 600 seconds, wraps the whole
   native pass once, and lowers provider transport with no independent request
   timer. The shared transport error no longer impersonates an OpenAI-direct
   path when OpenRouter owns the request. Ox13 is historical; later fresh roots
   use a non-restarting Persona unit so provider failure counts remain exact.
33. **Proposal semantics stay inside admission at `8812945e`:** proposal
   current-work does not grow a second rules tribunal for model-authored graph
   operations. The existing family mutation planner and generic RepoModel
   validator remain the semantic owners; refusal is durable typed state and a
   fresh pass receives the exact prior refusal.
34. **Historical direction proof is not current-work authority at `749d977e`:**
   a completed admitted-direction decision remains valid evidence for the
   proposal it created even when later disjoint RepoModel documents change.
   Current work is still projected from unresolved typed obligations, not from
   an aggregate model freshness comparison. Ox16 proved this cut in package
   reality before exposing the separate path-scope fault.
35. **Repository consequence authority is explicit at `5b799b12`:** the old
   `source_scope` name conflated evidence provenance with future write
   authority. The hard cut replaces it with `repository_scope`, names the
   adopted Hands narrowing `authorized_paths`, and makes generic RepoModel
   validation own canonical sorted repository-relative paths. Unresolved Eyes,
   Imagination, or Hands frontiers without a valid ceiling refuse atomically.
   Prompt schemas state that not-yet-created outputs belong in the ceiling and
   inspected evidence does not. Mind/runtime/RepoModel epochs advance to
   v4/v5/v2 with no compatibility reader. Ox16 is historical; Ox17 starts from
   a fresh store and exact package.
36. **Runtime identity is immutable at `87983ffc`/`ffcf036f`:** the v7 hard cut removes
   update time, derived document-type cargo, and temporary metadata from the
   runtime identity. Current-work launch preserves the exact identity and
   existing root-session envelopes as strong reads instead of reconstructing
   them with fresh storage timestamps. Unrelated job launches therefore share
   bootstrap identity without turning it into a global mutation head.
37. **Production binary ownership is singular at `6f93134d`:** the root release
   bundle alone owns model runtime, Persona service, and tool MCP runtime
   entrypoints. Leaf crates are libraries, Cargo metadata has no duplicate
   binary-source owner, and release construction no longer maintains a
   hardcoded package map that does not drive the build. Exact `b95a3266`
   separately deletes the superseded OpenAI debug executable; exact `c04c3aff`
   deletes the auth re-export crate.
38. **Runtime document registration has one owner at `03140a47`:** the packaged
   runtime-spine CLI, callerless Hello/schema-catalog writers, and hand-written
   mutation-contract mirror are deleted. CultCache exact `fdbf3bf` exposes its
   actual registered type identities; runtime status and model preflight read
   that registry directly. Cargo has 21 executable targets and no duplicate
   binary-source owner. Brokkr owns Unity editor capability through Eve/CultMesh;
   a future Rider daemon owns Rider. Epiphany owns neither editor bridge.
39. **Coordinator status is a projection, not a program, at `500125d5`:** the
   unconsumed packaged status executable is deleted. Its live current-work
   projection, rendering, and operator-thought sealing live once in
   `epiphany-core`; the coordinator imports that module directly. Cargo falls to
   20 executable targets without losing the operator-sealing check.
40. **Persona consequence safety is typed at `63939fa2`:** projector and Persona
   prose are inert stage outputs, so a hand-written forbidden-word tribunal
   cannot add authority. Only the Interpreter's closed effect enum can propose
   consequence. The tribunal, test-only prompt wrapper, prompt-substring test,
   and forbidden-word test are deleted; exact stage/context and typed-effect
   refusal tests remain.
41. **Tests must falsify live behavior at `bf2f39eb`:** an identical-call
   causal-ID tautology and a duplicate unknown-field check for extinct generic
   patch cargo are deleted. Family lifecycle/admission and runtime-owned
   identity-substitution checks retain the actual invariants.
42. **Public-source tests have one owner at `104bf390`:** the tool runtime's
   duplicate immutable-GitHub identity test and permanently ignored historical
   README network probe are deleted. Canonical identity parsing remains tested
   in `epiphany-core`; CI no longer carries a test it never executes.
43. **Tool execution has no static status mouth at `777dd1a5`:** the callerless
   `smoke` command, fixed JSON summary, parser branch, and branding test are
   deleted. Signed health owns liveness and typed receipts own execution; the
   surviving single command parses directly without a one-variant enum.
44. **Repository Body verification is not a release command at `013c3bf1`:**
   the callerless `smoke` path and its temporary stores are deleted. The shipped
   binary keeps the four operational commands; 23 owner tests retain the actual
   bind, observation, projection, Git-semantics, and tamper invariants.
45. **Provider physiology cannot be counterfeited at `ce90f911`:** the
   callerless model-runtime `smoke` command is deleted. It opened no provider;
   it wrote synthetic request, tool-call, completion, and receipt documents into
   a writable runtime store. Real execution and transcript-free decision audit
   remain, and the release-owned model-runtime suite passes 12/12.
46. **Tool completion does not mirror static contracts at `0bb6a883`:** the
   run summary loses its fixed adapter/schema catalogue and caller-supplied
   store echo. Exact authority remains the typed receipt; stdout returns only
   intent ID, receipt ID, and terminal status.
47. **The sealed launch owns pass-family identity at `edb5c3a3`:** eight
   optional request-ID mirrors are deleted from worker launch/runtime options,
   and terminal role results lose the same family echoes plus copied Body basis
   and frontier route. Live routing decodes the typed launch; retained history
   uses archived request kind/ID beside the preserved structured decision.
   Success requires the family semantic payload and typed failure forbids
   success cargo. Runtime is v39 and role-result output is v5 with no legacy
   reader.
48. **Coordinator incarnation and model-session authority split at
   `cabdd6c3`:** model sessions retain only identity plus active/completed
   admission state. Coordinator start provenance is an immutable typed basis;
   one deterministic terminality identity admits either the exact full run
   receipt or exact death recovery, so competing outcomes conflict while
   unrelated document inserts still merge. Runtime is v40. The generic session
   loses objective, timestamps, notes, and four unused states; the redundant
   one-field open options and whole-snapshot append helper are deleted. The next
   subtraction target is the generic runtime job aggregate, not a lifecycle
   registry.
49. **Runtime jobs carry lifecycle, not chronology, at `a46725a1`:** every
   production job is inserted queued and terminalizes completed or failed.
   Unwritten running/review/cancelled states and creation/update clocks are
   deleted. Research derives its exact attempt set, refuses multiple live or
   successful authorities, and never selects work by timestamp order. Runtime
   is v41; the cut removes 67 maintained lines net. The remaining audit target
   is duplicated role/session identity and generic terminal-result cargo.
50. **Runtime job ownership is derived at `10a70afc`:** the generic job is only
   job identity plus Queued/Completed/Failed status. Sealed launches own
   outer-worker role, model bindings own model-session membership, and root
   membership is structural. Jobs/results lose role/session mirrors, result
   metadata disappears, launch options lose three unused fields, and duplicate
   terminal results refuse instead of sorting by time. Runtime is v42; 73
   maintained lines are removed net. Next audit generic result cargo by terminal
   family so structured mirrors disappear without deleting the only failure
   account.
51. **Generic job results do not own decision evidence at `53e869c0`:** evidence
   and artifact mirrors are deleted from the result contract and completion
   API. Typed role/reorientation outcomes remain their sole owners. The generic
   result retains only terminal physiology and the terse failure/display account
   needed where no structured outcome exists. Runtime is v43; 51 maintained
   lines are removed net. Next audit remaining summary, next-move, time, and
   context cargo by terminal family.
52. **Structured decisions have one durable owner at `8eaa96c5`:** completing a
   typed role/reorientation outcome no longer persists a generic result copy.
   The exact typed envelope is a byte-identical strong dependency in the atomic
   job/process terminalization CAS. Fulfilled archives retain the context and
   full typed role result only. Runtime is v44; archived attempts are v3. The
   source delta is +7 lines for the CAS fence while one durable duplicate family
   disappears. Next determine whether the residual failure/transport result
   contract can be deleted entirely.
53. **Generic job results are deleted at `2c3335dc`:** typed role and
   reorientation outcomes atomically close their own job/process physiology;
   native model receipts or typed model-pass failures own model terminality;
   exact process claims own process death. Reorientation failure admission and
   decision audit bind the typed failure directly. One shared outcome rule
   prevents typed failures from claiming successful process fulfillment, and
   failed archives retain the full typed failure. Runtime/Mind advance to
   v45/v11 with no compatibility reader. The cut removes 361 maintained lines
   net.
54. **Model-session retirement is retained but stripped at `dbb00c2b`:** the
   live session latch owns explicit closure between tool rounds; the private
   archive owns retired identity collision refusal after bindings are deleted.
   Six model-turn display echoes and the one-field closure wrapper are gone:
   38 net lines. No registry or replacement aggregate is added.
55. **Verification has one exact Hands input at `24023265`:** its sealed typed
   projection already carries the complete consequence chain. The live
   `read_hands_receipt` tool, three runtime readers, exports, tool/prompt/gate
   cargo, duplicate source-store argument, and advertisement assertion are
   deleted: 78 net lines. The consequence receipts and atomic Hands-to-
   Verification admission remain. Next map the missing production Hands
   actuator as an ABI boundary and demote only speculative public mouths.
56. **Hands review authority is deleted at `a01c6842`:** the coordinator's
   same-function always-approved `HandsActionReview` had no independent reviewer
   or refusal producer. Adopted route, Substrate Gate grant, minimal intent,
   and frontier authority now form the complete authorization chain. Patch,
   command, and commit consequences cite the intent and derive the remaining
   identities. Commit admission requires exactly one patch and one successful
   command; ambiguous competing receipts refuse instead of being ordered by
   timestamp. Runtime advances to v46 and the sealed Verification projection
   to v2. The cut removes 381 maintained lines net; core 152/152, coordinator
   4/4, and OpenAI runtime 21/21 pass.

Do not preserve the aggregate for compatibility, manufacture a bootstrap
thread, release autonomous scheduling, register topology in `gamecult-ops`,
race the separate Yggdrasil deployment task, reuse historical/partial Gate
roots, or treat semantic readiness as coordinator acceptance.
