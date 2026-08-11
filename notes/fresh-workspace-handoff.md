# Fresh workspace handoff

Updated: 2026-08-12

## Current state

The five-day live shakedown campaign is paused for architectural consolidation.
The source branch is `codex/epiphany-shakedown-live`; the last pushed source
before surgery is `80541843`. The working tree contains the consolidation cut
described below and must be committed only after final source checks and map
updates.

The accepted live body remains c011 at exact `465af24d` on Starfire, cognitively
braked at resident revision 384 with no active lease, admitted work, or defunct
children. Do not replace it during consolidation. Yggdrasil remains the public
crossing and never builds Epiphany.

## Consolidation cut

The shakedown found the same missing invariants across several request families.
This pass generalizes those invariants instead of continuing family-specific
repair.

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

The full `epiphany-core` library passes 683 tests with one intentional ignored
cross-process helper. `epiphany-tool-mcp-runtime` passes 14 tests with one
intentional ignored live-network proof. Focused stale-thread refusal preserves
the exact store bytes.

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

The next physical extraction is a narrow `runtime_worker_attempt` module. It
should centralize typed process-status predicates and typed-request association
without merging the authorities above or introducing a registry.

## Acceptance gates before shakedown resumes

1. Extract the worker-attempt aggregate from `runtime_spine` with no schema or
   state migration.
2. Replay full core, resident, coordinator, tool-runtime, and OpenAI-runtime
   suites after extraction.
3. Prove immutable public Eyes success and no-grant denial on a copied packaged
   body. This is the paused shakedown's first remaining live gate, not part of
   the consolidation commit.
4. Run one fresh-repository source -> Hands -> Soul -> Modeling -> Mind -> Self
   capstone before claiming organism-level readiness.

## Immediate next action

Review the current diff for any surviving family-local causal hash or public
GitHub parser. Commit and push this first consolidation cut. Then map and extract
`runtime_worker_attempt`; do not add new capability surfaces and do not replace
c011.
