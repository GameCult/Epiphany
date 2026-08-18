# Fresh workspace handoff

Updated: 2026-08-18
Branch: `codex/epiphany-shakedown-live`
Latest implementation cut: `ec1431ff`

## Orientation

The five-day shakedown and Model Atlas operational Gate 1 are paused. Do not
touch historical c011/proof volumes, reuse partial Gate roots, release
autonomous scheduling, register operational topology in `gamecult-ops`, build
on Yggdrasil, or call the organism deployment-ready.

Epiphany is a supervised engineering alpha. Historical live proofs remain
valid evidence, but current source has advanced beyond the last packaged c011
body.

## What just landed

The global thread-state authority is gone.

- `1662a012` makes coordinator policy a pure projection over keyed Mind,
  exact current-work families, Resident pressure, and runtime receipts.
- `07d891ba` deletes persisted `EpiphanyThreadStateEntry`,
  `coordinator_state_transaction`, generic coordinator state/update/launch/
  acceptance/interrupt services, and their aggregate writers.
- `ec1431ff` deletes `EpiphanyThreadState` itself, global launch revision cargo,
  generic launch/interrupt request types, aggregate prompt/freshness/graph-
  context surfaces, the obsolete Repo graph importer, and the historical
  prompt-context smoke.

There is no compatibility aggregate, dual reader, bootstrap thread, or
migrator. Thread identifiers may survive only as immutable pass-creation
provenance; they do not own identity, currentness, or conflict.

Verification at this boundary:

- every Epiphany core target compiles;
- core library `489/489`;
- OpenAI runtime library `23/23`;
- model-runtime binary `10/10`;
- OpenAI-runtime binary `10/10`;
- Persona service target compiles.

## Canonical machine now

The runtime Mind `.cc` is the sole decision-bearing store. Keyed Mind and
RepoModel documents assemble deterministic views; their projection digests are
audit/display identities, not mutable authority revisions.

`mind_transaction::commit_mind_mutation` is the canonical commit primitive.
Concrete invariant owners supply exact strong reads and complete typed writes.
Disjoint identities merge, same-identity divergence conflicts, changed strong
reads refuse without mutation, and exact replay returns the original receipt.
Model output is never silently rebased.

`current_work.rs` derives work from unresolved typed state obligations and
exact runtime receipts:

- Modeling consumes Body/RepoModel obligations directly.
- Eyes launches only for an explicit outside-evidence obligation.
- Verification consumes exact Hands consequences and invariant obligations.
- Persona consumes typed unread social/relationship state.
- Hands consumes an adopted plan/route and exact capability receipts.

Events, timestamps, role lanes, latest-result slots, and accepted-at ordering
are projections only.

## Decision-audit foundation

The intended retained chain is:

1. exact typed Mind document versions;
2. sealed `EpiphanyReasoningBasis` with a closed typed projection;
3. exact final native model request;
4. internally derived provider request;
5. exact ordered governed tool intents/receipts supplied to the pass;
6. sealed `EpiphanyDecisionContext`;
7. structured terminal decision or typed pass failure;
8. invariant-owned `MindMutation` and `EpiphanyMindCommitReceipt`.

Transcripts, SSE frames, reasoning deltas, and intermediate prose are optional
and non-authoritative. A retained decision must remain auditable after deleting
them.

Several concrete Body, Proposal, frontier-verdict Modeling, Research,
Verification, Planning/PlanMind, and Reorientation paths already use sealed
bases/contexts and keyed admission. Research is driven only by explicit
external-evidence obligations; Eyes does not gate Body Modeling.

## Remaining wound

The broader Decision-Auditable Concurrent Mind migration is incomplete.

The residual generic `MindGatewayReview` / `MindStateCommitReceipt` v0 /
`EpiphanyRoleStatePatchDocument` path still represents a generic state-effect
admission mouth outside concrete invariant-owned `MindMutation` families.
Generic role failures, Persona projector/persona/interpreter stages, and some
worker/session archival paths do not yet make exact `reasoning_basis_id` and
`decision_context_id` structurally mandatory.

Context validation must become exact and DRY:

- provider request must equal internal lowering of the native request;
- final native input/tool definitions must be exact;
- every ToolCall name/arguments and ToolResult output must bind the exact
  governed intent/terminal receipt/runtime binding;
- request-owned public-source observations and model-continuation observations
  must have distinct validated ownership;
- foreign, missing, extra, duplicated, reordered, or nonterminal observations
  must refuse byte-identically.

Retention must preserve direct reachability: worker attempt or Persona terminal
to decision context to reasoning basis. No raw stream event, request family, or
transcript may be required for that query after archival.

## Immediate next action

1. Map and delete the generic Mind gateway/state-patch authority. Replace each
   genuinely live outcome with its concrete invariant-owned mutation; do not
   add a generic mutation registry.
2. Require a sealed basis before model execution and an exact decision context
   before any structured role/reorient/Persona result or model-backed failure
   terminalizes.
3. Derive provider requests internally and close the store-backed tool-
   observation substitution matrix.
4. Preserve basis/context/decision links through worker, model-session, and
   Persona retention while allowing stream/transcript deletion.
5. Bump the writable schema epoch and refuse old stores. Historical releases
   remain readers for historical proof stores; no migrator or dual path.
6. Run fresh acceptance: concurrent Persona+Hands; evidence+Verification+
   unrelated Modeling-node writes; distinct graph writes; same-identity
   conflict; stale strong-read refusal; transcript deletion; restart/re-entry;
   and source guards for deleted aggregate/global revision paths.
7. Only then package a fresh body and resume Model Atlas Gate 1 from a new
   external root.

## Operational state that matters

- Original c011 resident and Heartbeat containers are not running; both exited
  `255` on 2026-08-11. Their named volumes are quiescent historical evidence.
- Historical accepted live release: source `465af24d`, c011 package SHA-256
  `089e0005`; it is not the current source body.
- Starfire remains the cognition/build host. Yggdrasil remains the small public
  crossing body and never builds.
- Bifrost public crossing remains active on Yggdrasil. A completed public
  Persona consequence is still unproven because the Yggdrasil Discord
  credential is absent.
- Model Atlas code and isolated proofs remain accepted. Signed Gate 1 sight,
  brake freeze, autonomous cascade, partition exercise, and endurance remain
  open.

## Git/worktree caution

The shared worktree contains unrelated pre-existing modifications, including
vendored Codex/CultMesh files and a dirty `vendor/cultnet-rs` submodule. Preserve
them. Stage only intentional migration/documentation paths. Do not use reset or
checkout to clean the tree.

## Re-entry

Read, in order:

1. `state/map.yaml`
2. this handoff
3. `notes/epiphany-current-algorithmic-map.md`
4. `notes/epiphany-fork-implementation-plan.md`

Then run the shared-target `epiphany-state.exe status`. Persisted next action is
orientation, not automatic authority to resume a live Gate.
