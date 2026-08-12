# Fresh workspace handoff

Updated: 2026-08-12

## Current state

The five-day live shakedown campaign is paused for architectural consolidation.
The source branch is `codex/epiphany-shakedown-live`. Causal-work and immutable
public-source ownership landed at `868a5be0`. The working tree contains the
second consolidation cut described below.

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

The first cut's full `epiphany-core` library passed 683 tests with one intentional
ignored cross-process helper. After worker-attempt extraction, core passes 684
active tests with one ignored helper, `epiphany-tool-mcp-runtime` passes 14
active tests with one ignored live-network proof, and OpenAI runtime passes all
39 tests. Focused stale-thread refusal preserves the exact store bytes.

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

1. Prove immutable public Eyes success and no-grant denial on a copied packaged
   body. This is the paused shakedown's first remaining live gate, not part of
   the consolidation commit.
2. Run one fresh-repository source -> Hands -> Soul -> Modeling -> Mind -> Self
   capstone before claiming organism-level readiness.

## Immediate next action

Commit and push the worker-attempt extraction, then resume with the copied
packaged public Eyes success/denial pair. Do not add new capability surfaces and
do not replace c011 before that gate closes.
