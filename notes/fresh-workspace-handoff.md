# Fresh workspace handoff

Updated: 2026-08-18
Branch: `codex/epiphany-shakedown-live`
Latest implementation cut: `26b6a5bf`

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
- `f7948795` deletes the generic `MindGatewayReview` and
  `MindStateCommitReceipt` v0 authority, its interpreter prompt, runtime
  registration/read APIs, false state-effect/thought/adoption CultNet mouths,
  and launch proof profiles that required phantom gateway receipts. CultMesh
  now projects only the sealed reasoning basis, decision context, and exact
  `EpiphanyMindCommitReceipt`, all read-only.
- `1c9aafd8` seals the exact terminal decision context before a model-backed
  frontier-Planning failure is written. The typed faculty failure and generic
  runtime job now cite the same context; transport-only failure remains
  physiological and cannot impersonate a Mind decision.
- `e0e75a30` deletes `EpiphanyRoleStatePatchDocument`, its generic parser and
  three role-policy tribunals. Research now emits one closed
  `EpiphanyResearchDecision`; its admission owner derives keyed evidence,
  observation, and checkpoint writes. The old generic Imagination planning
  patch had no current-work owner and was deleted rather than generalized.
  Runtime role-result and worker output contracts advanced to v4.
- `12b1b285` makes runtime model/tool binding validation one shared owner for
  both execution and decision-context sealing. Exact native/provider lowering,
  ToolCall arguments, ToolResult bytes, terminal receipts, session/job/basis,
  and source-worker authority must describe one family. Hostile provider,
  receipt, and stored-binding substitutions refuse; reasoning bases and
  decision contexts remain valid after native/provider transcript deltas are
  deleted.
- `01602fd3` advances the Mind epoch to v2 and runtime-spine schema to v1.
  Opening any store that claims runtime or Mind identity now requires one exact
  current identity pair before registered runtime writers can see it. Old v1/v0
  stores and split identities refuse byte-identically; unrelated Persona and
  coverage physiology stores remain outside this authority. There is no schema
  migrator or dual reader.
- `26b6a5bf` closes the first fresh concurrency slice. Two independently
  planned Modeling-node writes commit simultaneously and assemble into one
  valid reopened graph; competing writes to one semantic identity yield one
  winner and one typed conflict; exact winner replay returns the original
  receipt. A real Verification admission also commits simultaneously with an
  unrelated Modeling-node write. Raw mutation primitives are crate-private,
  so external callers cannot manufacture generic Mind CAS plans.

There is no compatibility aggregate, dual reader, bootstrap thread, or
migrator. Thread identifiers may survive only as immutable pass-creation
provenance; they do not own identity, currentness, or conflict.

Verification at this boundary:

- every Epiphany core target compiles;
- core library `491/491`;
- OpenAI runtime library `24/24`;
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

Model opening already requires the sealed basis for worker passes and derives
the provider request from the native request. Structured role/reorient results,
the three Persona stages, and retained worker/session/Persona authority already
bind and preserve exact decision contexts. Concrete Body, Proposal,
frontier-verdict Modeling, Research, Verification, Planning/PlanMind, and
Reorientation admissions use keyed mutations. Research is driven only by
explicit external-evidence obligations; Eyes does not gate Body Modeling.

## Remaining wound

The broader Decision-Auditable Concurrent Mind migration is incomplete.

Context validation is now exact and DRY: execution and sealing share the same
model/tool binding owners, native/provider lowering is exact, final ToolCall/
ToolResult bytes are closed, and request-owned public-source observations stay
distinct from model-continuation observations.

Retention must preserve direct reachability: worker attempt or Persona terminal
to decision context to reasoning basis. No raw stream event, request family, or
transcript may be required for that query after archival.

## Immediate next action

1. Complete the remaining fresh acceptance: concurrent Persona+Hands through
   their concrete owners, then restart/re-entry with identical decisions and
   obligations. Distinct graph writes, same-identity conflict, stale
   strong-read refusal, simultaneous Verification+Modeling, transcript
   deletion, writable-epoch refusal, and aggregate/global-revision source
   guards are already accepted.
2. Only then package a fresh body and resume Model Atlas Gate 1 from a new
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
