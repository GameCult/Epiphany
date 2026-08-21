# Fresh workspace handoff

Updated: 2026-08-21
Branch: `codex/epiphany-shakedown-live`
Latest committed implementation cut: `bb823c54`
Current worktree: canonical documentation and evidence reconciliation only

## Orientation

The five-day shakedown and Model Atlas operational Gate 1 are paused. Do not
touch historical c011/proof volumes, reuse partial Gate roots, release
autonomous scheduling, register operational topology in `gamecult-ops`, race
Idunn's Yggdrasil CI/CD task with local compiler work, or call the organism
deployment-ready.

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
- `553f79d9` makes the OpenAI Responses schema projector one explicit dialect
  compiler shared by every model pass. It types literal-only schemas, removes
  parent-relative and unsupported conditional validation from provider cargo,
  preserves supported type-specific constraints, and lowers decision-bearing
  UUID formats to exact lexical patterns. The full native schema remains the
  Mind-admission contract; no provider projection can weaken native decoding.
  This cut was forced by three preserved packaged falsifications: untyped
  literal/parent refinements, rejected `uniqueItems`, and an invalid UUID after
  over-stripping provider guidance. No role-specific prompt/schema exception
  survives.
- `a8f3c1f0` repairs the Body identity seam exposed only after the first fully
  typed packaged Modeling decision. Bootstrap had supplied Git source identity
  to the keyed RepoModel's Body-binding field. Bootstrap now uses the admitted
  `RepositoryBodyObservationBasis`, and RepoModel initialization independently
  refuses any seed that disagrees with the authenticated runtime Body route
  before writing a single envelope. The negative test proves the entire store
  remains byte-identical.
- Exact `bb823c54` closes the model-pass failure seam exposed by the native
  capstone. `EpiphanyModelPassFailure` is one shared typed terminal record over
  the sealed basis/context for role, reorient, and Persona cognition. The
  terminal owner derives the exact transport session/job from that context,
  atomically closes a still-live transport job plus the exact model session,
  and prevents generic transport failure from impersonating decision authority.
  Restart reuses the terminal record instead of inferring again. Persona success
  closes the session only after the effect document and terminal receipt are
  durable.
- Reorientation failure admission now requires that same exact
  `EpiphanyModelPassFailure`. A generic failed worker result plus a plausible
  context can no longer manufacture a Continuity decision.
- Provider request authorship is also cut at the owner. Runtime execution and
  decision sealing both derive the provider request internally from the exact
  native request. Public model-turn ingress accepts only native requests;
  caller-authored provider requests refuse before opening model authority, and
  no public provider-request storage function survives.
- Exact `bb823c54` also closes the bootstrap-only semantic work seam.
  Modeling projection work is derived on each pulse from the complete exact
  keyed RepoModel document-version set and the Mind commit receipts that own
  those versions. The cache work item is content-addressed and stays outside
  Mind mutation CAS, so unrelated graph commits remain concurrent. Older work
  cannot suppress a newly assembled basis. Each projector operation validates
  the complete current keyed namespace from its opening snapshot, and
  acquisition uses full-snapshot CAS so a concurrent disjoint insert cannot
  hide as a phantom. Modeling retention uses exact basis identity rather than
  synthetic aggregate generation or timestamp order.

There is no compatibility aggregate, dual reader, bootstrap thread, or
migrator. Thread identifiers may survive only as immutable pass-creation
provenance; they do not own identity, currentness, or conflict.

Verification at this boundary:

- `cargo test --workspace --all-targets --locked -- --test-threads=1` passes natively;
- every Epiphany core target compiles;
- core library `496/496`;
- OpenAI runtime library `25/25`;
- OpenAI Codex spine `12/12`;
- model-runtime binary `10/10`;
- OpenAI-runtime binary `10/10`;
- Persona service `1/1`.
- tool runtime `14/14`, with only the explicitly live immutable-GitHub test
  ignored;
- release construction `21/21`.

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

Model opening requires the sealed basis for worker passes and derives the
provider request internally from the native request; callers cannot supply a
second provider-shaped truth. Structured role/reorient results,
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
Exact source `5f66d6c9` adds `list-decisions` to the same runtime. It
deterministically lists only contexts with a complete validated terminal audit
chain; sealed nonterminal contexts remain physiology and are omitted. This
surface and its portable-test repairs through exact `ebc0ffe4` have now passed
Idunn's full Yggdrasil workspace gate and are sealed in the exact Linux package
described below.

Idunn compiled and tested exact source
`ebc0ffe4f341154d1902f9afe86f0a87f150179c` on upgraded Yggdrasil. The gate
recorded 133 successful test summaries, including core `499/499`, under test
receipt SHA-256
`38c36ebe12e662602d6eccbbdc520de19bff801fec90af0eec2a345306a567d8`.
It sealed 27 immutable root-owned binaries as release
`sha256-bb76728653b8e2e872b4da47f917abe4233fd6d4ae1fd573c5971c7db3922a5c`
with witness SHA-256
`4d8350fac61f90d32a2b8067731308ec3e3672a42804db29c680b0fc68ab9adc`
and `privateStateExposed=false`.

The exact Idunn request ending `2026-08-21T20:17:11.821Z` then refused before
publication because Yggdrasil lacks the Bifrost organizational-operator
runtime identity and its VoidBot, bridge-group, private-state, anchor, and
configuration substrate. The deployment brake is engaged; Epiphany units are
inactive and disabled; `app/current` still names recovery source `267a0257`;
no `deployment.env`, runtime publication, or signed-health admission exists.
Resident Self's two Codex credential files are also absent behind that first
gate. This is a clean infrastructure blocker, not package acceptance or a
license to bootstrap another organ from the Epiphany lane.

Fresh source tests prove the retained chain, concurrent keyed commits,
old-epoch refusal, deterministic current-work re-entry, and Persona replay
without input reformulation or inference. This is source acceptance, not yet a
package/live claim.

Exact clean packages at `55fb4cf8`, `edd664db`, and `1e01c339` preserved the
three provider-boundary falsifications above. Exact clean `553f79d9` then
packaged 27 Linux binaries as release
`sha256-e6c19d2b231a73023772f1162b15dac63bd4c7957a7a1cea82a86c8d75f35ac9`
with witness
`sha256-fbee2857c558bde47f4b422b59fe675362fd1955388b084b3d44a188f8a6c82a`.
Its fresh capstone completed concurrent three-stage Persona cognition and a
terminal Body-Modeling decision after five governed source-tool rounds. Review
then refused before Mind mutation because the seeded RepoModel carried Git
source identity instead of the authenticated Body binding. That failed root is
preserved and is not replay input.

Exact clean `a8f3c1f0` now packages natively on Windows as 27 binaries, release
`sha256-19d40048f46355486e66c2abc766abb277122c552cb121a9161e6e95a471be46`,
witness
`sha256-ee8bf0574b5e7d489adc47f6dca491fe5224c8b3e61b3d2fd8c2e67c36d5f903`,
with `privateStateExposed=false`. Its fresh store projected `launchModeling`
directly from the Body obligation and launched Persona plus Modeling
concurrently. Both provider calls then failed because this task token cannot
reach `chatgpt.com`; no positive decision was fabricated.

That failure found a real architecture gap. Modeling already sealed a typed,
transcript-free failure context, but Persona left its session active. The
Exact `bb823c54` repairs the shared owner. A native negative replay produced
basis `reasoning-basis-sha256:d31179fe...`, context
`decision-context-sha256:a89ba921...`, and model-pass failure `dd058f3f...`;
the Persona session is `Completed`, `transcriptRequired=false`, and restart
left runtime SHA-256 `C7723AD7...` plus heartbeat SHA-256 `2996AAA8...`
byte-identical. Its 2.09 MB temporary replay root is
`.epiphany-run/native-model-failure-20260818-a`; evidence is distilled and the
temporary root owns no continuing proof.

Docker shakedown hygiene remains an operator invariant, but this migration and
its package are native-only. Future containerized runs must use one shared warm
cache set, exact run labels, bounded exported evidence, and terminal cleanup.
A stopped build container or cloned source volume owns no durable proof.

## Immediate next action

1. Have the Bifrost/VoidBot infrastructure owner install the missing
   organizational-operator substrate, then provision Epiphany's Codex
   credentials through their single owner. Reissue the exact Idunn deployment
   for the already tested and sealed `ebc0ffe4` package and require publication,
   runtime start, and signed-health receipts. Do not run a competing release
   build on Starfire and do not let the Epiphany lane bootstrap another organ.
2. Run the fresh-store Decision-Auditable Concurrent Mind capstone against the
   Idunn-admitted package: concurrent Persona/repository work, Body -> Modeling -> Mind,
   explicit Eyes evidence where required, Hands -> Verification -> Mind,
   `list-decisions` plus exact `audit-decision`, and process restart/re-entry
   with identical obligations and no resurrection.
3. Establish a fresh canonical Persona mouth receipt anchor before the Persona
   consequence leg. Do not transplant the old proof anchor or ship a generic
   test harness.
4. Only then resume Model Atlas Gate 1 from a new external root. Idunn remains
   the sole deployment owner; Bifrost/VoidBot bootstrap belongs to its own
   infrastructure authority.

## Operational state that matters

- Original c011 resident and Heartbeat containers are not running; both exited
  `255` on 2026-08-11. Their named volumes are quiescent historical evidence.
- Historical accepted live release: source `465af24d`, c011 package SHA-256
  `089e0005`; it is not the current source body.
- The old Yggdrasil capacity measurement is obsolete. The upgraded host is the
  intended swarm body and CI/CD machine. Idunn owns source-change compilation,
  testing, release construction, deployment, and daemon survival there; its
  current deployment truth belongs to the separate deployment task.
- Idunn is active and independent at exact source `745e0109`; its Epiphany
  source-change target is installed. Exact `ebc0ffe4` passed the full workspace
  gate and sealed successfully on Yggdrasil. Publication stopped safely at the
  missing Bifrost operator substrate; this is not a reason to fall back to
  Starfire compilation.
- Bifrost public crossing remains active on Yggdrasil. A completed public
  Persona consequence is still unproven because the Yggdrasil Discord
  credential is absent.
- Model Atlas code and isolated proofs remain accepted. Signed Gate 1 sight,
  brake freeze, autonomous cascade, partition exercise, and endurance remain
  open.
- GitHub's August 2026 service incidents were external provider downtime. They
  do not count as Epiphany source-identity or grant failures; successful lookup
  and denial-before-network remain separately required evidence.

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
