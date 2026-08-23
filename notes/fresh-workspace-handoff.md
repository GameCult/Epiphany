# Fresh workspace handoff

Updated: 2026-08-23
Branch: `codex/epiphany-shakedown-live`
Latest committed implementation cut: `39dd9fdb`
Current worktree: heartbeat concurrency-domain audit; Ox17 remains paused

## Orientation

The five-day shakedown and Model Atlas operational Gate 1 are paused. Do not
touch historical c011/proof volumes, reuse partial Gate roots, release
autonomous scheduling, register operational topology in `gamecult-ops`, race
Idunn's Yggdrasil CI/CD task with local compiler work, or wake resident
cognition without an explicit operator resumption.

Epiphany is a supervised engineering alpha. Exact source `d2ca6630` remains the
production symlink body and is inactive. Exact `5b799b12` is the current
build-affecting source. Historical live proofs remain evidence; they do not
authorize the next capstone or Gate 1.

## Current live deployment

- Epiphany source: `d2ca66301fb6af4e7d2d27fff0b772b0f0fccdf4`.
- Release: `sha256-46407552b4a0937f63d2b7f2bd09a1dacb89d671a6e3807c97209159541aef06`.
- Release witness file SHA-256:
  `348785ffb0fc3130d3b4538329870c6e6f8442a8da4a01b32fa9b7ffb1f01357`.
- Full-workspace test receipt SHA-256:
  `de0fc6b360ce03493b13208d917dc8349801f03364617d966152c85846c47482`.
- Model provider: OpenRouter `stealth/ox-alpha`, selected explicitly and
  injected through the root-owned systemd credential boundary.
- `epiphany.service`, `epiphany-heartbeat.service`, and
  `epiphany-swarm.service` are inactive with zero restarts after the hard epoch
  refusal rollback. The d2ca stores and swarm brake remain historical
  production state; no newer package is admitted there.
- Idunn source `8b972715c47731f2418d0c423cb0dd2076940bd7` is provenance-exact and
  admits Epiphany through the shared authenticated daemon-health contract.
- gamecult-ops `b47f9084` removes Docker from
  Epiphany's future Idunn compile/package actuator. Native Rust `1.95.0` is
  installed under `epiphany-builder`; current source-change builds are native.
- Ox9 and Ox10 ran only in isolated fresh capstone roots. Neither is acceptance:
  Ox9 exposed coordinator-receipt resurrection; Ox10 exposed Body retry
  starvation after strict malformed-output refusal. No public consequence was
  emitted.

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
- Exact `470d4cb5` removes historical coordinator receipts from behavioral
  routing. Family admission owners materialize typed obligations, and one pure
  current-work projection is now the scheduler input. Exact `d48f69b7`
  repairs Body Modeling retry identity. Exact `9b9b5c85` completes the shared
  law: simple model passes carry one typed attempt projection, Research and
  two-stage Planning retain their full exact lifecycles in current-work, and
  proposal Modeling preserves canonical failed/cancelled attempt identity.
  A failure changes pressure identity once without counters or event authority.

There is no compatibility aggregate, dual reader, bootstrap thread, or
migrator. Thread identifiers may survive only as immutable pass-creation
provenance; they do not own identity, currentness, or conflict.

Verification at this boundary:

- `cargo test --workspace --all-targets --locked -- --test-threads=1` passes natively;
- every Epiphany core target compiles;
- core library `491/491`;
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

The cross-repository operator-plane cut is complete: Epiphany `39adf3a4`,
Bifrost `8b9ab65`, VoidBot `78763ea`, and gamecult-ops
`ca09ba8`/`c44fb05`/`448e368`. Eve/CultMesh projections, local typed enginseer
tools, Persona transport, and governed consequence receipts remain. The
standalone Discord command daemon, bridge admission worker, operator Mind
writers, deployment gate, unit, and readiness claims do not.

Exact `ab321b34` introduces one explicit, decision-auditable model-provider
boundary. The native request remains canonical; provider selection lowers it
internally into the exact OpenAI or OpenRouter request recorded in the decision
context. Exact `547404fa` projects public provider identity without exposing
credentials. Exact `c1a6034f` preserves supervisor, heartbeat, and Resident
Self read-only physiology while the swarm brake is engaged. Exact `6b44b4d3`
publishes the shared Idunn signed-health wire schema.

Idunn compiled, tested, packaged, deployed, and health-admitted exact current
source `d2ca66301fb6af4e7d2d27fff0b772b0f0fccdf4` natively on upgraded Yggdrasil. The
full workspace gate passed under test receipt SHA-256
`de0fc6b360ce03493b13208d917dc8349801f03364617d966152c85846c47482`.
Idunn sealed 26 immutable root-owned binaries plus `release-witness.ccmp` as
release `sha256-46407552b4a0937f63d2b7f2bd09a1dacb89d671a6e3807c97209159541aef06`;
the witness file SHA-256 is
`348785ffb0fc3130d3b4538329870c6e6f8442a8da4a01b32fa9b7ffb1f01357`.
The successful request is
`manual:redeploy:yggdrasil-epiphany:manual:redeploy:yggdrasil-epiphany:2026-08-22T16:35:41.670Z`.
`epiphany-operator-command` remains absent.

The provider boundary exposed two deployment ownership defects rather than
earning provider-specific compensators. Idunn's validator now consumes its
canonical authenticated daemon-health record; the root actuator supplies the
exact trust store. Runtime credential readiness is inspected inside Resident
Self's systemd mount namespace, where `LoadCredential` actually exists. These
cuts are live at Idunn `8b972715` and gamecult-ops through `b47f9084`.
No transient Epiphany container, named volume, package root, or per-run test
root remains.

The bounded redeploy request helper opens an exact, expiring brake grant and
immediately returns after waking Idunn; it does not own terminal observation or
brake closure. During the `470d4cb5` transaction the operator poll reached a
successful terminal receipt before re-engaging the brake. Exact release
binding refused contemporaneous stale Odin/Ghostlight requests, so no unrelated
target mutated, but the released record briefly survived transaction closure.
Future operator transactions must treat terminal observation plus explicit
`deployment-brake-engage` as their final step. Idunn cannot possess the private
operator identity used to close its own mutation authority.

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

The admitted `d2ca6630` release was compiled and packaged natively on
Yggdrasil with Rust `1.95.0` under `epiphany-builder`. The live Idunn actuator
contains no Docker path. A build container or cloned source volume owns no
durable proof.

Ox9 proved typed Persona and Modeling decisions, then exposed that historical
coordinator receipts could resurrect stale direction. Exact `470d4cb5` cut
that authority and made Resident Self consume only current unresolved typed
state. Ox10 proved the replacement route: Body launched Modeling directly with
no Eyes result, while Persona ran concurrently. Modeling sealed its exact
basis/context and failed closed because Ox emitted duplicate `tension` cargo;
no Mind commit occurred. Three Persona projector attempts independently timed
out opening the provider stream and each terminalized as typed failure.

Ox10 then exposed one remaining asymmetry. Body Modeling projected its semantic
work and continuation action in separate fields and omitted the failed job from
the current-work digest. The unresolved obligation remained launchable, but
Resident Self correctly refused to mint the same pressure identity twice.
Exact `d48f69b7` deletes that Body split shape. Exact `d2ca6630` adds restart-safe
resident objectives and durable recovery for a dead runtime worker, and is the
current admitted Yggdrasil body.

Private Ox12 proved the repaired Body path in package reality: Body Modeling
launched directly without Eyes, completed, and admitted; admitted-direction
Imagination also completed. Under exact `9b9b5c85`, OpenRouter then completed
proposal Modeling attempt 1 with a structured result. The result put RepoModel
node/claim identities in `dependency_item_ids`, whose contract is frontier
identity. The mutation planner correctly refused the absent frontier dependency
and changed no graph state.

That correct semantic refusal exposed a generalized lifecycle gap: model
transport success remained terminal while its failed Mind admission was not a
durable routing input. Exact `e046a4d1` adds one typed
`EpiphanyAgentPassAdmissionRefusal` Mind document for Body, proposal, and
frontier-verdict Modeling plus frontier Verification. The refusal binds exact
request, job, result, decision context, invariant owner, refusal kind, reason,
and commit. Current-work derives a fresh attempt from it; proposal retry carries
the ordered prior refusals. A successful model result remains truthful and is
not rewritten as a failed worker job. Output contracts now distinguish claim
targets from frontier dependencies. Mind/runtime/proposal context cut to
epochs v3/v4/v2; old writable stores are refused. Ox12 is therefore historical
evidence and must not resume. Local core `491/491`, coordinator `12/12`, status
`2/2`, swarm `10/10`, and all core targets pass.

Exact `85061129` then passed Idunn's full Yggdrasil workspace gate and sealed
release `sha256-238a7911650c406c67b710b19642451c1b1e7bdbe66e9dc7605e46e098a780dd`.
Private fresh-store Ox13 proved the new refusal lifecycle in package reality:
Body Modeling launched without Eyes and admitted, Imagination completed, and a
structurally valid proposal result wrote a typed admission-refusal commit before
attempt 1 completed with corrected semantics. Transcript-free exact audit is
retained under the Ox13 root. Persona concurrently exposed an independent
deadline-owner defect: its provider transport hardcoded 90 seconds while worker
passes used one outer 600-second budget. A stop race let systemd begin four
typed projector failures rather than the live-observed three; none committed
Mind or public speech. Ox13 is sealed, inactive, and must never resume.

Exact `3b958a83` gives Persona one explicit outer
`--turn-timeout-seconds` budget, defaults it to 600 seconds, removes the inner
provider request timer, and makes the shared error provider-neutral. The
focused provider/runtime suites pass: provider spine `17/17`, runtime library
`26/26`, model-runtime binaries `13/13` each, and Persona service `1/1`.

Ox15 is sealed historical fresh-store evidence. It proved Persona no longer
dies at the old inner transport timeout, then exposed a separate proposal
currentness fault. Exact `8812945e` keeps proposal semantics inside the family
admission owner, and exact `749d977e` makes a completed historical direction
decision valid evidence for its proposal across later disjoint RepoModel
changes without turning an aggregate model revision back into authority.

Exact `749d977e` passed Idunn's Yggdrasil gate under test receipt SHA-256
`39229d8a526a7af7e7b29cec28d27b07f89ba5f5792658947e3d1d039b9449c4`
and sealed package
`sha256-5d9bc25612dd46511620671ccd5113f3b5e0e3c060c85e97dcdc199f9126a230`.
Private fresh-store Ox16 launched direct Body Modeling without Eyes while all
three Persona stages ran concurrently. Body and proposal admission survived
later disjoint RepoModel changes. No public delivery succeeded. Planning then
remained unavailable because the active Imagination frontier used inspected
evidence paths as its supposed future write scope, omitted `OX-CAPSTONE.md`,
and was unordered. Dependent proposal pressure began to form, so Ox16 was
braked and sealed before the forest grew. No Hands consequence or capstone
marker exists; Ox16 must never resume.

Exact `5b799b12` is the hard architectural cut from that evidence. It deletes
`source_scope`/`sourceScope`, makes `repository_scope` the canonical sorted
repository-relative ceiling for future Planning/Hands consequences, and names
the adopted route narrowing `authorized_paths`. Generic RepoModel validation
owns the intrinsic path law and refuses an invalid routed frontier atomically
before any graph document is written. Inspected files and evidence remain in
their existing typed audit cargo. Mind/runtime/RepoModel epochs advance to
v4/v5/v2. Local acceptance is core `494/494`, OpenAI runtime `26/26`,
coordinator `12/12`, swarm `10/10`, and model-runtime `13/13`.

## Immediate next action

The Ox17 deployment lane is paused. Yggdrasil exposed 96 GiB of dead native
Cargo targets after interrupted full-workspace builds; exact cleanup returned
the builder target root to 4 KiB. Exact `1d5a1f17` landed the first causal cut:

- 21 duplicate core/release binary declarations are gone; each production
  entrypoint has one Cargo owner;
- 19 generic repo-request smoke executables are gone because Cargo compiled
  them but their `main()` assertions were not tests;
- the runtime-store migrator CLI and the unshipped repo-personality birth
  subsystem, parallel `agents.msgpack` initialization, schemas, prompts, and
  stale contract plans are gone;
- the non-vendor diff removes 19,252 net lines and reduces `epiphany-core` from
  75 to 30 binary targets;
- core library tests remain 494/494 and Hands remains 5/5 under the sole release
  package owner.

Exact `387afe49` is the second verified cut. It deletes the
`runtime_store_migration` module/test, all 29 remaining unshipped core
executables, the shipped `epiphany-verse-query` parallel control plane, the
local PowerShell launcher, and its orphan Rider bridge. `epiphany-core` now
owns only `epiphany-prepare-compaction`; the release bundle owns 25 runtime
executables. Focused core, supervisor, and release-construction tests pass
493/493, 32/32, and 21/21.

Exact `bf516f99` deletes the agent-memory migration, repair, lifecycle,
trait-seed, and SoA authorities with the tests that preserved them. It also
deletes the second CultMesh SoA mirror, its timestamp-selected latest head, and
its prompt injection. Persona now reads learned memory only from keyed
`EpiphanyMindPersonaMemoryDocument` rows in the assembled Mind view; those
documents participate in the exact projection digest and source set. Dead
test-only alternate Resident Self and direct Continuity writers are also gone.
The final core library passes 483/483 and Persona service re-entry passes 1/1.

Exact `360073a3` collapses semantic projection to one Modeling corpus and
removes 1,726 net lines. The global Mind semantic corpus, agent-memory graph
profile, dual-store projector contract, multi-source fairness cursor, legacy
semantic migrators, one-variant partition type, unused semantic traits, and
their obsolete tests are gone. Modeling retrieval now derives only live
RepoArchitecture/RepoDataflow documents from the complete exact keyed
RepoModel basis. Native core check passes and the library suite passes 472/472.

Exact `d1685df8` deletes heartbeat aggregate cognition: personality/mood timing,
appraisal, rumination, dreaming, the void routine, and its memory graph. Exact
`39dd9fdb` deletes the remaining agent-memory identity/generation store, the
64-float utterance vector, generic `selfPatch` decoding/admission, the phantom
sleep-distillation contract, stale aggregate/thread schemas, and nine more
constructor/static-vector tests. Native role ingress now refuses unknown model
cargo instead of accepting forbidden identity and silently ignoring it. Core
passes 455/455 and the provider runtime passes 27/27. The obsolete repository-
local Cargo target was 2.5 GiB and was removed; compilation remains in the
shared target.

Exact `2164bd0a` collapses heartbeat to Resident Self and Persona scheduling.
It deletes Ghostlight scene scheduling, manual tick/pump/heat/complete and
queue-mention controls, adaptive pacing/initiative heat, five fake organ lanes,
five dead schemas, and eight self-referential tests: 2,145 deletions against
118 additions. The strict heartbeat state is now `v1`; the owning binary checks
cleanly and the surviving core suite passes 448/448.

Next split pending Persona mentions, turn requests/terminal receipts, blocked
social pressure, and retention head/plan from the heartbeat singleton into
keyed CultCache identities. Heartbeat may consume a derived pressure view but
must not own social state. Continue deleting constructor/projection-copy tests
as their dead surfaces are exposed. Do not run a full-workspace compile,
package, deployment, or Ox root until the subtraction audit closes. Never
resume Ox10, Ox12, Ox13, Ox15, or Ox16.

## Operational state that matters

- Original c011 resident and Heartbeat containers are not running; both exited
  `255` on 2026-08-11. Their named volumes are quiescent historical evidence.
- Historical accepted live release: source `465af24d`, c011 package SHA-256
  `089e0005`; it is not the current source body.
- The old Yggdrasil capacity measurement is obsolete. The upgraded host is the
  intended swarm body and CI/CD machine. Idunn owns source-change compilation,
  testing, release construction, deployment, and daemon survival there; its
  current deployment truth belongs to Idunn's typed deployment receipts.
- Idunn is active, provenance-exact, and independent at installed source
  `8b972715c47731f2418d0c423cb0dd2076940bd7`. Exact Epiphany source
  `d2ca6630` remains the current symlink target; Starfire remains outside the
  compilation path. Production Epiphany units are inactive after exact
  rollback and `deployment.env` is absent. Idunn has zero restarts.
- Exact `9b9b5c85` passed Idunn's native gate under receipt SHA-256
  `6c6a71359f8c31297419665d2872f6982a89f84e197d71fc5b36a7fc86216093`
  and remains sealed as release `sha256-30a910785a380224bbdbd56a7b742c8d058fa70cb72dab3562e3907903e0d191`.
  A foreign-target helper restarted Idunn during its candidate transaction;
  exact rollback prevented false admission but exposed the missing cross-target
  transaction mutex.
- OpenRouter
  `stealth/ox-alpha` credential readiness is proven inside Resident Self's
  mount namespace. The production organism remains braked. Ox decisions exist
  only in isolated failed capstone roots and do not authorize production state.
- Bifrost public crossing remains active on Yggdrasil and publishes typed
  readiness. Its current Idunn observation is diagnostic-only rather than a
  lifecycle-authenticated admission. Frozen private Ox12 must prove
  Persona effects and exact context without emitting public speech.
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
