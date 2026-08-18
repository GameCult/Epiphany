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

- `epiphany-state-model` owns shared typed semantic contracts; durable Mind is keyed CultCache owned by `epiphany-core`.
- `epiphany-core` owns keyed Mind admission, runtime spine, organ gates,
  heartbeat physiology, Persona, and CultMesh integration. Coordinator policy,
  status, and every migrated work family consume keyed Mind plus exact runtime
  receipts. Persisted `EpiphanyThreadStateEntry`, the in-memory
  `EpiphanyThreadState` schema, generic launch/update transactions, and global
  launch revisions have been deleted with no compatibility path.
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

Keep the five-day shakedown and Model Atlas operational Gate 1 paused. Complete
the remaining Decision-Auditable Concurrent Mind hard cut before another live
coordinator run. Fresh Gate preflight exposed the aggregate ownership defect;
the aggregate has now been structurally removed. The remaining gate is exact
decision-context admission/retention plus fresh concurrent and restart proof.

Cut in this order:

1. **Landed at `79346523`:** replace the aggregate-shaped role reasoning input with one sealed typed
   projection assembled from exact keyed Mind document versions. The final
   model request must render that projection; citing keyed sources while
   rendering the old thread snapshot is explicitly forbidden.
2. **Body admission landed at `2eb95df6`; current-work projection landed at `e42788c9`:** one pure typed projection now derives Body Modeling, Eyes continuation, proposal Modeling, frontier planning, and Hands readiness from keyed Mind/runtime receipts.
   Modeling must derive from the current Body/RepoModel obligation; Eyes only
   from explicit external-evidence obligations. Body-generation Modeling
   identity, typed Body-to-Mind admission, sealed reasoning, and atomic decision
   admission are landed. Current-work no longer reads the external Body store;
   status consumes the shared projection.
3. **Baseline transaction landed at `9f7b164f`; operator routing at `587c56d2`; Resident continuation and keyed acceptance at `478fb923`:** baseline Body Modeling is now thread-free from unresolved Body obligation through Launch/Wait/Review and exact Mind admission. Resident and operator callers consume the same family projection and acceptance owner. Thread ID remains immutable pass provenance only.
4. **Proposal Modeling landed at `d1b031cb`:** its immutable request, exact launch binding, runtime attempt/result, and `Modeling.proposal_frontier` commit now own Launch/Wait/Review and admission. The aggregate launcher refuses proposal cargo; the old selector, validator, context builder, and generic-launch hint are deleted. Body and Proposal share only the exact-envelope launch CAS invariant, not a mutable family registry.
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
   request-bound Eyes packet, and `Eyes.frontier_research` Mind mutation own
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
10. **Frontier Planning and PlanMind landed at `c9329ed6`:** the Planning
   request seals exact frontier/dependency versions plus per-claim obligation
   guards. Current-work owns deterministic Imagination and Mind attempts;
   Imagination authors a typed candidate, while Mind alone adopts it through a
   sealed PlanMind request and family-owned frontier mutation. Claim challenge
   admission atomically updates the target claim-obligation document, so stale
   planning conflicts without turning unrelated graph writes into a global
   head. Thread provenance, generic coordinator validators/constructors,
   aggregate role lanes, and accepted-at/latest-result behavior no longer own
   this family.
11. **Reorientation landed at `d5df53ae`:** one keyed request seals the exact
   continuity projection and source document versions; deterministic attempts
   bind an exact reasoning basis and terminal decision context. The family
   owner alone admits resume/regather plus its continuity receipt. Model-backed
   failure retains a typed Mind failure document before retry. Status and
   coordinator routing use the same current-work projection. Aggregate
   launch/acceptance, checkpoint/freshness recommendation, latest binding,
   global revision acceptance, and accepted-at comparison no longer own this
   family.
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
   CultMesh and runtime schema catalogues expose only reasoning basis, decision
   context, and exact Mind commit receipt as read-only audit projections.
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
16. Refuse old writable-store epochs and replay fresh-store disjoint
   concurrency, same-identity conflict, transcript deletion, restart, and
   re-entry. Only then restart Model Atlas Gate 1 from a new external root.

Do not preserve the aggregate for compatibility, manufacture a bootstrap
thread, release autonomous scheduling, register topology in `gamecult-ops`,
touch Yggdrasil, reuse historical/partial Gate roots, or treat semantic
readiness as coordinator acceptance.
