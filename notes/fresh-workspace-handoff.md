# Fresh workspace handoff

Updated: 2026-08-14

## Current state

The five-day shakedown remains paused for architectural consolidation. The
latest behavior cut is clean and pushed on `codex/epiphany-shakedown-live`
through exact `120663fa`.

The active campaign is now the hard Decision-Auditable Concurrent Mind
migration. Its first foundation is implemented locally: typed content-addressed
reasoning bases preserve exact source envelopes and a closed projection;
terminal decision contexts preserve the exact native request, the provider
request derived from it, and every ordered governed tool intent/receipt actually
fed to the model. Worker model requests require a sealed basis, and structured
role/reorient results plus their generic job result bind the terminal context.
Tool-round-limit and repeated-tool-loop failures seal the last request context
before worker failure. Model-session archives preserve their basis/context IDs,
and fulfilled worker-attempt tombstones preserve and revalidate the exact
decision context instead of collapsing it to a digest. Exact-envelope batch CAS
merges disjoint Mind documents and explicitly refuses same-identity or
changed-strong-read conflicts. All three Persona stages now seal
predecessor-bound contexts, the effect and terminal commit atomically, and
Persona retention preserves the structured effect, conversation/consequence
receipts, terminal receipt, reasoning bases, and decision contexts while
removing stage/execution scaffolding. Exact `ee3d82c0` binds provider/runtime
errors and watchdog timeouts to the exact admitted request held by the running
pass, without inventing a persisted latest-request head. Exact `71505941` adds
the keyed Mind document family, deterministic `EpiphanyMindView`, atomic
runtime/Mind epoch identity, and a mutation gate that rejects non-Mind envelopes
or semantic key drift. The old aggregate Mind owner and keyed RepoModel remain
to be cut; the keyed view does not read or translate the aggregate.

Exact `bd5034cd` is the first live authority cut. Operator objective intake now
atomically writes one semantic objective document, its immutable typed operator
provenance, and one Mind commit receipt. It writes no thread-state aggregate;
thread identity is provenance, and replacement refuses without changing store
bytes. The commit authority is explicitly model context or operator provenance,
so operator decisions do not counterfeit model reasoning. The aggregate-only
Modeling acceptance repair writer and commands are deleted; frontier execution
inspection/amendment remains under a truthfully named binary.

The keyed RepoModel foundation now gives identity, domain, node, edge, summary,
frontier, lifecycle receipt, and per-node unresolved-claim obligations separate
CultCache identities. Its deterministic validated view is part of
`EpiphanyMindView` and has no revision/hash head. Documents store canonical
named MessagePack behind typed constructors because direct compact nesting let
skipped middle fields shift into the wrong schema positions. Mind CAS strong
reads are now genuine read fences: CultCache receives byte-identical expected
envelopes as required by its physical primitive, while the commit receipt lists
only semantic writes. A stale read-only dependency blocks the whole mutation
without partial insertion. Full core remains 690 passing with one ignored
cross-process helper.

The concrete RepoModel mutation planner now accepts a persisted typed semantic
proposal and owns dependency derivation for domains, nodes, edges, summaries,
and frontier documents. Node and edge retirement are semantic operations;
admission and lifecycle receipts remain runtime-owned. Callers do not supply a
strong-read list. One mutation can create a complete domain/node/edge/summary
slice in dependency-reversed order, while duplicate semantic identities refuse.
A fresh node retirement and a concurrent new frontier target both write the
same previously absent obligation identity, so only one can commit.

Exact `8ad16be0` cuts the first aggregate RepoModel bootstrap path.
`epiphany-repository-body bootstrap` now admits an `EpiphanyRepoModelSeed` as
typed operator provenance and atomically writes only keyed RepoModel documents,
derived claim obligations, and the generic Mind commit receipt. Identical seed
replay is inert; divergent replay and any aggregate RepoModel envelope refuse.
The seed contains no revision/hash head. Full core passes 692 tests with one
ignored cross-process helper, and the repository-body binary checks cleanly.
Exact `18d3783a` deletes the old thread-state-to-aggregate bootstrap binary.
Exact `120663fa` removes aggregate bootstrap/read behavior from launch context:
prompt assembly loads `EpiphanyRepoModelView`, derives a nonpersisted query-only
graph projection, and renders only its projection digest. The derived snapshot
has no revision/hash authority. All seven launch-context tests and the full core
suite pass.

Operational correction: the original c011 resident and Heartbeat containers
are not alive; both exited 255 on 2026-08-11. Their named volumes are quiescent
and remain untouched. Historical c011 zero-zombie and organ-circuit evidence is
still valid, but it is not current liveness. Every copied public-Eyes body is
braked and stopped. Yggdrasil remains the public crossing and never builds
Epiphany.

## Consolidation cut

The shakedown found the same missing invariants in different clothes. The
consolidation now gives each repeated invariant one named owner.

- `causal_work_identity` is the single pure derivation owner for Proposal
  Modeling, Research, frontier Planning, PlanMind, and Admitted Model Direction
  request identities. Runtime plus immutable admitted cause determine identity.
  Coordinator thread and timestamps are provenance only.
- Coordinator launch cannot restore a superseded request thread into canonical
  state. The current admitted thread owns transport; the request's creation
  thread remains inspectable provenance.
- `ImmutableGithubSource` is the single parser, validator, canonicalizer, and
  renderer for immutable public GitHub identities. Modeling selection, Eyes
  execution, and Mind receipt authentication use the same type.
- Mind now independently rejects a malformed but internally self-consistent
  public-source receipt. Provider validation is no longer trusted as a substitute
  for admission validation.
- `runtime_worker_attempt` owns the complete typed request-family association and
  process-status classification for one launch/process/result/archive family.
- Self decides Research launch currency from the exact current causal frontier;
  a stale terminal Research role projection cannot suppress new Eyes work.
- The typed `RepoFrontierResearchRequest` owns the exact allowed public-source
  set. One launch-to-request derivation is shared by Self coverage, Research
  acceptance, and final Mind revalidation. Capability grants, prompts, and
  locally inspected files are not substitutes for exact requested receipts.
- `RepoFrontierResearchLifecycle` owns the exact current Research continuation.
  Resident Self, coordinator policy, and status consume its typed Launch/Review
  action. A completed superseded role lane is display history, not a launch
  suppressor or compensating repair path.
- Worker reaping consumes the full `ProcessInstanceIdentity`, not a PID. The
  kernel wait occurs only after that exact incarnation is observed exited. A
  historical claim cannot reap a reused coordinator child.

Core passes 687 active tests with one intentional ignored cross-process helper.
The packaged no-grant execution gate passed under `--network none` with
byte-identical state and no receipt. Exact packaged `f90f1186`, release
`sha256-0e41c9106eb0e37a9cf3e7a4b67671d494b880b7e71fec65336a8c3e7e1129b4`,
witness `sha256-a310c72292a44f950ce7b0ab469779d02e6edd1068670fe37f990f7d82846ee4`,
then completed two copied immutable-public-source Research/Mind acceptance
cycles. Public tool receipts advanced from 53 to 61, Hands stayed false, and
both daemons stopped braked with exit zero.

The preceding `08383c6d` run is retained as falsification evidence: it proved
the typed Research lifecycle cut, then died with `ECHILD` because PID-only
reaping let a historical worker claim steal a reused coordinator child's wait
status. `f90f1186` removes the PID-only API and survived the same process
turnover.

## Authority map

- Owner: causal work identity owns deterministic request identity.
- Inputs: runtime identity plus the immutable proposal, admitted model/frontier,
  accepted result, candidate digest, or admission receipt appropriate to the
  explicit request family.
- Outputs: deterministic typed request IDs.
- Derived state: creation thread and timestamps are provenance; job, process,
  grant, model execution, and tool IDs own attempts beneath the request.
- Forbidden writers: current coordinator thread, resident projection, launch
  callback, retry loop, and worker output cannot fork or repair request identity.
- Shared paths: direct and resident launches consume the same derived identity;
  current admitted thread carries the launch without rewriting request provenance.
- Cut line: family-local hash formulas, current-thread selection gates, and
  downstream thread-restoration behavior are removed or demoted.

- Owner: immutable public-source identity owns GitHub identity grammar and
  canonical rendering.
- Inputs: owner, repository, exact 40-hex commit, and bounded repository path.
- Outputs: canonical lowercase revision, repository ref, and source ref.
- Derived state: provider URL and receipt display fields.
- Forbidden writers: tool output and mutually consistent JSON strings cannot
  define source identity.
- Shared paths: Modeling, Eyes, and Mind call the same typed owner.
- Cut line: the runtime composite parser and provider-local validators are gone.

## Deliberately separate lifecycle authorities

Do not collapse these into one enum or service:

- Heartbeat schedules and consumes acknowledgements.
- Resident Self owns grants, leases, cooldown, cancellation, and requeue.
- Coordinator owns one process incarnation and its terminal receipt.
- Runtime worker attempt owns launch, exact process claim, result, and archive.
- Mind decides adoption; fulfillment is evidence, not admission.

The narrow `runtime_worker_attempt` module now owns the serialized process-status
vocabulary, live/fulfilled/failed/retry classification, and the complete set of
typed request families carried by one launch. Runtime, resident, and coordinator
consumers call those predicates and accessors instead of maintaining local
string sets and parallel family arrays. Persisted documents and transactions are
unchanged; the lifecycle authorities above remain separate.

## Acceptance gates before shakedown resumes

1. Every model-authored terminal decision or typed failure recovers its exact
   reasoning basis and terminal request after transcript/session retention.
2. Persisted thread state, aggregate RepoModel revision/hash, timestamps,
   events, and role lanes can no longer manufacture or suppress current work.
3. Concurrent disjoint Persona, Hands, Modeling, evidence, and Verification
   mutations merge; same semantic identity and changed strong reads refuse
   without partial writes or silent model-output rebasing.
4. Run the fresh-store concurrent decision-audit capstone, including Body ->
   Modeling without Eyes and Eyes only for explicit external-evidence work.

## Immediate next action

Cut model-authored `RepoModelPatch` and aggregate admission next. The provider
schema must expose semantic operations only; runtime composes the durable
mutation proposal and the concrete family owner derives strong reads and commit
metadata. In parallel sequence, migrate focus/mode, subgoal/invariant, evidence/observation,
checkpoint, and planning admission owners from `EpiphanyThreadStateEntry` to
their keyed documents. Delete the aggregate transaction, reader, and schema
before enabling old writable-store refusal; do not translate or dual-read it.
Do not touch the original c011 volumes, resume the shakedown, add capability
surfaces, or build on Yggdrasil.
