# Fresh workspace handoff

Updated: 2026-08-18
Branch: `codex/epiphany-shakedown-live`
Latest implementation cut: `d3300bba`

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
- `7c2ebd81` removes the live Persona consequence path from the legacy
  agent-memory aggregate. Interpreter `state_note` decisions now become keyed
  `EpiphanyMindPersonaMemoryDocument` writes under the exact Interpreter
  decision context. Exact replay returns the original Mind commit. A concrete
  Hands commit and Persona admission run simultaneously and both commit.
- `79c0e373` seals the complete assembled Persona pass input as one typed Mind
  document before inference. It records the exact heartbeat, agent-memory, and
  admitted Body versions observed; all three Persona stages cite the sealed
  Mind version. Substitution refuses byte-identically, replay reuses the
  original input, and direct Git inspection is gone. Persona-only input and
  memory documents are excluded from generic role projections.
- `d3300bba` makes re-entry consume only an authenticated admitted Persona
  pass-input. The loader requires one exact `Persona.pass_input` Mind commit
  receipt, validates its content-addressed identity, and refuses a naked typed
  document. A restarted execution plan performs no heartbeat, agent-memory,
  Body, repository, or model read when that input and terminal decision already
  exist. Current-work reconstruction is identical and byte-for-byte read-only
  at Launch, Wait, completed, and post-Research boundaries.
- `f8412b69` makes decision audit an operator capability rather than a test
  claim. `reasoning_context::audit_decision_context` reconstructs the sealed
  basis, decoded typed projection, exact terminal native/provider requests,
  governed tool observations, structured terminal records, and exact Mind
  commit receipts without consulting streams, events, or current Mind state.
  Worker archival now retains the complete typed role/generic result family;
  the digest-only archive shape is deleted. Runtime schema v2 refuses the
  prior writable archive epoch.

There is no compatibility aggregate, dual reader, bootstrap thread, or
migrator. Thread identifiers may survive only as immutable pass-creation
provenance; they do not own identity, currentness, or conflict.

Verification at this boundary:

- every Epiphany core target compiles;
- core library `493/493`;
- OpenAI runtime library `24/24`;
- model-runtime binary `10/10`;
- OpenAI-runtime binary `10/10`;
- Persona service `1/1`.

## Canonical machine now

The runtime Mind `.cc` is the sole decision-bearing store. Keyed Mind and
RepoModel documents assemble deterministic views; their projection digests are
audit/display identities, not mutable authority revisions.

`reasoning_context::commit_mind_mutation` is the crate-private canonical commit
primitive.
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

## Accepted migration boundary

The source-level Decision-Auditable Concurrent Mind migration is accepted.

Context validation is now exact and DRY: execution and sealing share the same
model/tool binding owners, native/provider lowering is exact, final ToolCall/
ToolResult bytes are closed, and request-owned public-source observations stay
distinct from model-continuation observations.

Retention must preserve direct reachability: worker attempt or Persona terminal
to decision context to reasoning basis. No raw stream event, request family, or
transcript may be required for that query after archival.

The packaged `epiphany-model-runtime audit-decision` command exposes that
read-only projection by exact context ID. Its JSON is an operator/xenos
rendering only; it never writes Mind or reconstructs prompts from live state.

Fresh source tests prove the retained chain, concurrent keyed commits,
old-epoch refusal, deterministic current-work re-entry, and Persona replay
without input reformulation or inference. This is source acceptance, not yet a
package/live claim.

An exact clean `d26a99d5` Linux baseline package completed successfully with 27
binaries and release witness
`sha256-55346d97207d5a847317e654256f91378f5d5ee7cac62093539c70413f9d57a3`.
That proves the isolated package machinery and warmed cache, not the final
capstone: the exact final package must be rebuilt from `f8412b69` or its
document-only successor.

## Immediate next action

1. Build one exact clean-source package from `f8412b69` or its document-only
   successor without adding a shipped proof-only binary.
2. Run the fresh-store Decision-Auditable Concurrent Mind capstone against that
   package: concurrent Persona/repository work, Body -> Modeling -> Mind,
   explicit Eyes evidence where required, Hands -> Verification -> Mind,
   transcript-independent decision inspection, and process restart/re-entry
   with identical obligations and no resurrection.
3. Only then resume Model Atlas Gate 1 from a new external root.

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
