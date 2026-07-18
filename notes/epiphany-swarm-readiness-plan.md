# Epiphany Swarm Readiness Plan

This is the direction of travel before live fire.

The local run path proves that the current machine can be started and inspected.
It does not prove the swarm is ready to run unattended. Do not confuse a working
starter switch with a clean engine bay. That mistake has already been punished
elsewhere, and VoidBot has kindly provided the scorch marks.

## Objective

Get Epiphany ready for ordinary resident operation inside an organization
without building architectural Jenga. This is the product, not a special
dogfood or supervised experiment: the swarm learns its authorized domain,
maintains that model, derives bounded pressure from it, proposes improvements,
and receives organization-wide conversation and feedback through its Persona
even when nobody is feeding it a task queue.

The useful autonomous loop is:

`Body observation -> persistent Modeling map -> Self direction -> Imagination proposal`

Persona conversation and attributed organizational feedback can add pressure to
that loop. They do not replace the map, become Mind truth, invoke Hands, authorize
a release, or deploy one. Those authorities remain separately gated.

The operating constraints are:

- typed CultCache documents remain the state authority
- CultNet is the native wire for Epiphany-owned subsystems
- Codex remains the retained auth/model transport compatibility organ
- Aquarium and Persona are reflectors/mouths, not hidden sources of truth
- heartbeats create opportunities, not authority
- swarm speech and action stay gated until scheduling, memory, repetition, and
  review boundaries are coherent

The repo-swarm MVP contract lives in
`notes/epiphany-repo-swarm-mvp-contract.md`. Its correction is binding:
autonomous unbounded work inside an Epiphany-owned Body is in scope. The MVP is
not a permission cage where every local edit waits for a human. The safety
boundary is typed authority: branch-local work inside the owned repo Body may
continue autonomously, while publication, merge, privilege escalation,
cross-body mutation, and authority changes require the owning gate.

## VoidBot Lessons To Carry Forward

VoidBot's ordinary spine is not the rotten part. Its durable lessons are:

- Discord ingestion, worker jobs, provider lanes, RAG, Postgres, Qdrant, and
  typed CultCache self-state can be conceptually sound when each owns one job.
- The unstable organ was the repo Persona swarm loop: scheduling, prompt policy,
  identity, public speech, governance, dispatch, proposal behavior, and
  repetition control packed into one prompt-and-parser machine.
- A repo-controlled pause flag is a real brake. If a swarm subsystem is under
  teardown, runners must fail closed instead of routing around the pause.
- One CTB-style initiative scheduler beats one task per agent. The wall-clock
  task supplies pulses; typed initiative state chooses turns.
- Heat changes recovery pressure. It must not fast-forward time, bypass active
  turn freeze, or create overlapping thoughts.
- Cooldown starts after completion, not after queueing.
- Stale active turns need explicit recovery receipts, or one dead worker claim
  freezes a Persona forever.
- Public speech needs parent-side eligibility checks. Prompts alone cannot
  prevent repeated openings, stale topics, scheduler labels leaking into prose,
  or work requests masquerading as banter.
- Memory must have phases: short-term residue, incubation, durable memory,
  revision, retirement, crystallization, and sleep-owned maintenance.
- Agents propose typed operations. State services validate, normalize, dedupe,
  and write. Whole-state JSON editing is not a mutation path.
- Qdrant/vector recall is a rebuildable resonance cache, not canonical memory.
- Persona affect is state, not vibes: needs, social bonds, status reads, and mood
  dimensions must survive typed projection and MCP/CultNet inspection.

## Current Mechanism

Epiphany owns its state, organ scheduler, workers, process policy, and local
operator surfaces in native Rust and typed CultCache/CultMesh documents. Codex
survives only as the retained OpenAI subscription-auth/model-transport bridge;
it no longer owns Epiphany routes, prompts, scheduling, or state mutation.

- Runtime spine, Mind gateway, Substrate Gate, Eyes, Hands, Soul, Modeling,
  Continuity, heartbeat, and Persona operations have typed intent/review/receipt
  families.
- One heartbeat scheduler owns initiative, heat, active-turn freeze,
  completion-gated cooldown, stale-turn recovery, sleep, and rumination.
- The local CultMesh swarm brake is typed and enforced at runner/action edges.
- Persona public speech has parent-side repetition/saturation eligibility and
  Bifrost/Heimdall crossing gates.
- Memory has reviewed revise/retire/crystallize/prune/merge operations; Qdrant
  remains rebuildable semantic sight rather than canonical Mind.
- Idunn owns managed projector lifecycle and exact abandoned-claim recovery.
  Task Scheduler owns only current-user after-login Idunn presence.
- Mind and Modeling semantic query admission reauthenticate canonical CultCache
  authority at query time against the Yggdrasil-hosted Qdrant/Ollama substrate.

The installed host is operational after login through one witnessed immutable
release generation. Mind now owns the race-bounded whole-repository readiness
join in source: fresh Body R1/R2, exact admitted RepoModel, live Modeling
semantic vectors, live workspace coverage, complete authority snapshots, and an
atomic historical projection append. A real reboot/logon survival proof remains
the separate operator-authorized host boundary. The new join still needs to be
included in the next witnessed release before it can produce live host evidence.

## Invariants

- One state authority per document kind.
- One scheduler for standing initiative.
- One explicit brake for swarm operation.
- Autonomous branch-local work inside the swarm's owned Body is allowed and is
  the desired operating mode.
- Persistent domain observation, map maintenance, bounded rumination, and
  proposal generation continue without operator instruction.
- The repository Persona is the organization's conversational surface. Any
  admitted member may provide attributed feedback through the configured
  crossing; conversation alone grants no implementation or publication authority.
- Upstream publication is Bifrost territory, not implicit branch-local
  authority.
- No lane receives a new heartbeat while its previous turn is running.
- No model output rewrites durable state directly.
- No public speech without typed eligibility and receipt checks.
- No cross-repo rummaging; swarm needs move coordinator-to-coordinator through
  visible typed messages and callbacks.
- No hidden automatic semantic acceptance.
- No prompt-only governance for things that need state, receipts, or gates.
- No broad live scheduler until the local runner can show the typed surfaces
  that explain what it would do and why.

## Readiness Evidence Matrix

This matrix is the live gate. A green row names its authority and proof; it is
not permission for another row to borrow the result.

| Gate | Owner | Current evidence | State |
| --- | --- | --- | --- |
| Typed brake and initiative physiology | heartbeat/CultMesh | brake, completion cooldown, and stale-turn recovery tests and receipts | closed |
| Reviewed Mind and organ-state mutation | Mind gateway and organ services | typed review/commit/lifecycle families; no whole-state model write path | closed |
| Public speech/crossing eligibility | Persona + Bifrost/Heimdall | speech-audit receipts and fail-closed identity/capability checks | closed |
| Branch-local autonomous action | Substrate Gate + Hands + Soul + Mind | scoped grants and patch/command/commit/verdict/admission chains | closed |
| Persistent domain stewardship | Modeling + Body observer + semantic projector | source-bound RepoModel admission, freshness joins, persistent CultCache state, rebuildable Qdrant projection | closed in source; live resident proof pending |
| Autonomous improvement proposals | Self + Imagination + Mind | domain pressure becomes a bounded proposal-only consideration; adoption remains a separate Mind decision | closed in source; live resident proof pending |
| Organization-wide Persona feedback | Persona + VoidBot + Bifrost + Self | attributed Discord observation, exact classification/signature/delivery, dedicated feedback store, proposal-only consideration | closed in source; live Discord proof pending |
| Semantic projection/query recovery | canonical admission + projector + query gate + Idunn | live Mind/Modeling semantic ranking and GUID-scoped exact process-recovery smokes | closed |
| Atomic installed sibling generation | release packager + Idunn | release `sha256-c94109f94ee42c1257089830f399cb87cd5ad672772c274194371995ce4df923` binds commit `dd80f4a5`, four exact binaries, task args, policy role paths, and recurrence receipts | closed |
| Whole-repository observed readiness | Mind | typed source join, hostile raw-orchestration tests, live semantic/coverage rereaders, and full-snapshot CultCache append fence | closed in source; not yet witnessed/deployed |
| Current host package | release deployment | witnessed supervisor PID 21228 and direct-child projector PID 19748; Mind/Modeling semantic query admission passed | closed for current boot |
| After-login recurrence | Task Scheduler + Idunn + ops tunnel | task recurrence and exact child recovery observed on current boot | closed for current boot |
| Reboot/logon survival | Task Scheduler + Idunn + ops tunnel | requires fresh post-boot lineage and query proof | permission-bound |

`providerStatus`, heartbeats, Qdrant collection existence, task state, watcher
silence, legacy retrieval Ready, Eve/Gjallar rows, and empty dirty-path lists are
never substitutes for the Mind readiness join or query admission.

## Ranked Next Cuts

Keep:

- typed CultCache state
- runtime-spine documents and CultNet catalog
- heartbeat initiative physiology
- reviewed role memory patches
- local run/status/coordinator artifacts
- Codex auth/model transport reliquary

Cut:

- schema/catalog language that treats legacy JSON-RPC routes as native authority
- public or operator outputs that dump prompt slabs as if they were status
- any new live runner that bypasses typed pause, receipt, or review gates

Collapse:

- local runner status/coordinator result shaping into one operator projection
  helper if duplication grows
- Persona speech receipts and eligibility into one typed surface before high-rate
  speech returns

Split:

- heartbeat opportunity from action authority
- Persona affect/state from speech generation
- scheduler metadata from model-facing prose
- vector recall from canonical memory

Rebuild:

- any swarm loop that depends on a single prompt to own scheduling, identity,
  public speech, governance, proposal behavior, and repetition control

## Immediate Work Queue

1. Let the currently authorized Yggdrasil projection finish without interrupting
   its sole store owner, then allow Idunn to admit or reject that exact release
   from terminal typed evidence.
2. Restart the already-installed corrected Yggdrasil Idunn only after that
   deployment child exits. Let Idunn discover and deploy the exact-authorized
   Bifrost Persona-feedback sidecar; do not invoke the deployment manifest as a
   parallel authority.
3. Bind the configured GameCult Discord rooms to the Epiphany Persona as
   organization feedback sources and prove attributed observation, exact
   classification, signed delivery, and dedicated-store admission.
4. Package and Bifrost-authorize the current Epiphany generation, then let Idunn
   deploy the resident supervisor, heartbeat, and swarm services with the brake
   engaged, writable Body, persistent Mind/Modeling state, and witnessed child
   binaries.
5. Before 2026-07-22, run a Starfire-offline rehearsal proving two independent
   causes of useful cognition: autonomous map-derived improvement proposals and
   organization-originated Persona feedback. Prove separately that neither path
   can adopt Mind state, invoke Hands, release, or deploy without its owning gate.
6. Reboot survival remains a separate operator-authorized proof. Do not widen
   service authority merely because the organizational loop is otherwise ready.

The correct posture is patient pressure. We are not rushing to live fire. We are
aiming the barrel, checking the chamber, and making sure the thing does not try
to become a parliament of prompts the moment nobody is watching.
