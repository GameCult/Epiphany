# Epiphany Core Harness Surfaces

This note defines the stable surface contract for Epiphany inside vendored
Codex.

It is not the algorithmic map. It is not the implementation plan. It is not a
graveyard for every future type shape we once thought of at 3 a.m. with a
compiler glaring at us.

Use this file to answer:

- where Epiphany state lives
- which surfaces expose it
- which surfaces may write it
- which future surfaces are still missing
- which persistent-state habits keep the harness from becoming Jenga

## Core Principle

Epiphany belongs in the core agent harness.

The GUI is a reflection and steering layer over typed state. It is not the
source of truth. If the map lives only in the UI, the headless harness, resumed
thread, alternate client, and future specialist agents are blind again.

The rule is:

- core owns authoritative thread state
- rollout persists authoritative thread state
- `epiphany-core` owns the heavy Epiphany machinery where practical
- app-server exposes typed read, write, proposal, notification, and reflection surfaces
- GUI and other clients render or steer through those surfaces

## Current Landed Surfaces

| Surface | Kind | Status | Role |
| --- | --- | --- | --- |
| `EpiphanyThreadState` | protocol/core state | landed | Authoritative typed model of objective, subgoals, invariants, graphs, retrieval, investigation checkpoint, evidence, scratch, churn, and mode. |
| `RolloutItem::EpiphanyState` | persistence | landed | Durable snapshot for resume, fork, rollback, and compaction-safe reconstruction. |
| `<epiphany_state>` prompt fragment | prompt input | landed | Bounded developer-context summary rendered from typed state, including the durable investigation packet when present. |
| `Thread.epiphanyState` | client read | landed | Typed state on hydrated thread payloads. |
| `thread/epiphany/retrieve` | read-only query | landed | Hybrid repo retrieval; exact/path plus semantic BM25/Qdrant when available. |
| `thread/epiphany/index` | retrieval catalog write | landed | Explicit semantic indexing path; updates retrieval catalog, not durable Epiphany understanding. |
| `thread/epiphany/update` | state write | landed | Revision-aware durable typed patch application. |
| `thread/epiphany/distill` | read-only proposal | landed | Turns one explicit source observation into an observation/evidence patch candidate. |
| `thread/epiphany/propose` | read-only proposal | landed | Drafts graph/frontier/churn candidates from verified evidence-backed observations. |
| `thread/epiphany/promote` | verifier gate | landed | Rejects or applies candidates through the durable update path. |
| `thread/epiphany/stateUpdated` | notification | landed | Emits updated projected state, source, revision, and changed fields after successful update/promote writes. |
| `thread/epiphany/jobLaunch` | bounded authority write | landed, live-smoked | Creates a launcher-owned durable `jobBinding`, launches the current backend adapter, and emits `stateUpdated` with source `jobLaunch`. |
| `thread/epiphany/jobInterrupt` | bounded authority write | landed, live-smoked | Interrupts the current backend adapter for a bound launcher job, clears backend identity from the durable `jobBinding`, and emits `stateUpdated` with source `jobInterrupt`. |
| `thread/epiphany/reorientLaunch` | bounded authority write | landed, live-smoked | Consumes the read-only reorientation verdict and launches one fixed `reorient-worker` job with explicit resume/regather scope over the current backend adapter. |
| `thread/epiphany/reorientResult` | read-only result read-back | landed, live-smoked | Reads the fixed or requested reorient-worker binding through the current backend adapter and projects completed worker output as a reviewable finding without promotion or mutation. |
| `thread/epiphany/reorientAccept` | explicit acceptance write | landed, live-smoked | Requires a completed reorient-worker result, then appends accepted observation/evidence and optionally updates scratch/checkpoint with a distinct state update source. |
| `thread/epiphany/jobsUpdated` | notification | landed | Emits changed launcher-bound job snapshots for real runtime progress events when the mapped payload actually changes. |
| `thread/epiphany/scene` | read-only reflection | landed, live-smoked | Compact client scene derived from authoritative Epiphany state, including checkpoint summary reflection. |
| `thread/epiphany/jobs` | read-only reflection | landed, live-smoked | Derived indexing, remap, verification, and specialist-progress slots from typed state and retrieval summaries, with durable launcher metadata plus live backend overlay when a real owner exists. |
| `thread/epiphany/freshness` | read-only reflection | landed, live-smoked | Retrieval and graph freshness lens derived from retrieval summaries plus graph frontier/churn state, with watcher-backed invalidation inputs for loaded threads. |
| `thread/epiphany/context` | read-only reflection | landed, live-smoked | Targeted state shard for graph nodes/edges, active frontier, graph checkpoint, investigation checkpoint, observations, and evidence. |
| `thread/epiphany/pressure` | read-only reflection | landed, live-smoked | Context-pressure gauge derived from token telemetry and recorded auto-compact/context limits. |
| `thread/epiphany/reorient` | read-only policy reflection | landed, live-smoked | Bounded CRRC reorientation verdict derived from checkpoint, freshness, watcher, and pressure signals; returns resume vs regather without acting on the decision. |
| `thread/epiphany/crrc` | read-only coordinator reflection | landed, live-smoked | Composes pressure, reorientation verdict, bound worker status/result, and available actions into one recommendation without launching, accepting, compacting, or mutating state. |

## Write Authority

There should be one red pen.

Durable Epiphany state may change through:

- `thread/epiphany/update`
- accepted `thread/epiphany/promote`
- `thread/epiphany/jobLaunch`
- `thread/epiphany/jobInterrupt`
- `thread/epiphany/reorientLaunch`
- `thread/epiphany/reorientAccept`
- normal rollout persistence of the current live `EpiphanyThreadState`

The following must stay read-only:

- `thread/epiphany/retrieve`
- `thread/epiphany/distill`
- `thread/epiphany/propose`
- `thread/epiphany/scene`
- `thread/epiphany/jobs`
- `thread/epiphany/freshness`
- `thread/epiphany/context`
- `thread/epiphany/pressure`
- `thread/epiphany/reorient`
- `thread/epiphany/crrc`
- `thread/epiphany/reorientResult`

`thread/epiphany/index` is a write to the retrieval catalog. It is not a
license to mutate map/evidence/churn state as a side effect.

## Reflection Authority

Scene, jobs, freshness, context, reorient, CRRC, and GUI surfaces may compress state for humans and
clients. They may not invent canonical understanding.

Reflection surfaces should:

- derive from `Thread.epiphanyState` or the same live/stored state projection
- preserve revision identity
- expose what actions are available
- make missing, stale, or stored state explicit
- avoid client-private graph, invariant, evidence, or job state

The dashboard can exist. It does not get to seize the steering wheel and declare
itself the engine.

## State Shape

The durable thread state should stay structured first and linguistic second.

The important categories are:

- objective and active subgoal
- bounded subgoals
- invariants
- architecture/dataflow graphs
- graph frontier and checkpoint
- retrieval summary
- investigation checkpoint
- job bindings
- scratch
- observations
- recent evidence
- churn
- mode
- last updated turn

Natural language belongs inside the structured model where it helps the model
and human understand the machine:

- purpose: why a thing exists
- mechanism: what it actually does
- metaphor: a compact mental handle after source grounding
- code refs: where the claim is anchored
- evidence ids: why the claim is trusted

Metaphor is compression after source context. It is not decoration for guesses.

## Missing Surfaces

These are not landed yet:

- richer evidence-range and graph-shard inspection beyond the landed context shard
- automatic watcher-driven graph/retrieval/invariant invalidation policy on top of the landed freshness reflection
- automatic tool-output observation promotion
- typed turn intent before broad mutation
- hard mutation gates for stale map or violated invariants
- role-scoped specialist-agent registry and scheduling
- automatic runtime Compact-Rehydrate-Reorient-Continue execution beyond the landed read-only coordinator recommendation

Do not implement these as one blob. Each needs a bounded surface, a write rule,
and a verification story.

## MVP Product Loop

The first product MVP should prove a complete Epiphany loop, not a complete
Epiphany universe.

Required loops:

- durable typed state for map, scratch, evidence, checkpoint, and job bindings
- role separation for implementation, modeling/checkpointing, and
  verification/review, even if the first specialists are narrow and explicit
- Compact-Rehydrate-Reorient-Continue as a first-class path driven by pressure,
  freshness, watcher, checkpoint, and reorientation signals
- read-back from bounded specialist jobs into reviewable Epiphany findings or
  proposals
- one inspectable dogfood surface that lets a non-Rust-speaking operator see
  what the harness believes and what it wants to do next

Out of scope for the MVP:

- arbitrary specialist marketplaces
- broad automatic background scheduling
- GUI-first workflows
- automatic promotion of all tool output
- a second job backend unless the current `agent_jobs` adapter blocks read-back
  or interruption

The first read-back, acceptance, and read-only coordinator blockers are landed.
The current MVP blocker is dogfood visibility: exposing scene, pressure,
reorient, jobs, result, and CRRC recommendation in a small operator-facing view
that can be tested during real coding work without reading Rust.

## Job And Progress Surface Direction

The first read-only job/progress reflection is landed as `thread/epiphany/jobs`.
It reports derived slots for retrieval indexing, graph remap, invariant
verification, and specialist work. Durable `jobBindings` now act as a thin
Epiphany-owned launcher seam: they can carry launcher identity, authority
scope, backend kind, and backend job id, and the current adapter can overlay
that launcher metadata onto live runtime `agent_jobs` snapshots. The surface
does not start, schedule, create, or notify jobs.

The first explicit bounded authority surfaces are also landed as
`thread/epiphany/jobLaunch` and `thread/epiphany/jobInterrupt`. They are
allowed to mutate durable `jobBindings` and talk to the current backend
adapter, but only to create or interrupt explicit launcher-owned work. They
are not a queue, a second scheduler, or permission to smuggle runtime policy
into the reflection surfaces.

The first bounded runtime consumer over CRRC verdicts is also landed as
`thread/epiphany/reorientLaunch`. It can only launch one fixed
`reorient-worker` binding through the same backend adapter, with explicit
resume-versus-regather scope and checkpoint-derived payload. It is not
automatic CRRC, not a background coordinator, and not a license to keep coding
after drift without an explicit launch.

The first read-back surface over that worker is also landed as
`thread/epiphany/reorientResult`. It defaults to the fixed `reorient-worker`
binding, resolves the current `agent_jobs` backend item, and projects completed
structured output as mode, summary, next safe move, checkpoint validity,
inspected files, frontier ids, evidence ids, and raw result. It does not promote
the finding, mutate typed state, schedule follow-up work, or continue the task
for the agent.

The first explicit acceptance surface over that finding is also landed as
`thread/epiphany/reorientAccept`. It requires a completed reorient-worker
result, appends an accepted observation and evidence record, and can explicitly
bank the finding into scratch or the durable investigation checkpoint when the
caller asks for those writes. It emits `thread/epiphany/stateUpdated` with
source `reorientAccept`; it does not accept pending work, auto-promote arbitrary
worker output, launch follow-up jobs, or silently continue implementation.

The first read-only coordinator over that loop is also landed as
`thread/epiphany/crrc`. It composes pressure, the resume-versus-regather
verdict, the fixed reorient-worker binding, result status/finding, and scene
actions into one recommendation such as continue, prepare checkpoint, launch
worker, wait for worker, review result, accept result, or regather manually. It
is not a scheduler, a hidden launch, a hidden accept, a compactor, or a second
source of truth. The little tyrant remains theoretical, for now.

The first live bound-runtime progress notification is also landed as
`thread/epiphany/jobsUpdated`. It rides existing `agent_job_progress:{json}`
background events from the runtime job runner, resolves matching launcher
bindings against live `agent_jobs` snapshots through the current backend
adapter, and only emits when the mapped bound-job payload actually changes. It
does not poll state runtime in a loop, start work, schedule follow-up work, or
turn the jobs read surface into a writer.

Future live job state should describe work like:

- retrieval refresh
- graph remap
- revalidation
- verification batch
- specialist task

Minimum useful fields:

- `id`
- `kind`
- `scope`
- `ownerRole`
- `launcherJobId`
- `authorityScope`
- `backendKind`
- `backendJobId`
- `status`
- `runtimeAgentJobId`
- `itemsProcessed`
- `itemsTotal`
- `activeThreadIds`
- `progressNote`
- `lastCheckpointAt`
- `blockingReason`
- linked subgoal or graph node ids

The landed reflection surface keeps this read-only. Future live job state must
keep the same rule: do not make job state a hidden second planner.

## Freshness Surface Direction

The first retrieval/graph freshness read is landed as `thread/epiphany/freshness`.

It reflects from existing sources only:

- live retrieval summaries for loaded threads, or stored retrieval summaries when that is all that survived
- graph frontier dirty paths and open-question/open-gap pressure
- graph checkpoint identity
- churn freshness hints such as `fresh` or `stale`
- live watcher-backed invalidation telemetry for loaded threads: watched root, recent changed paths, mapped graph-node hits, and active-frontier hits
- state revision and live/stored source identity

It does not mutate `SessionState`, refresh retrieval, remap graphs, emit
`thread/epiphany/stateUpdated`, schedule follow-up work, or perform automatic
semantic invalidation. It is a pressure lens with eyes, not the crew with the
crowbar.

## Context Shard Surface Direction

The first targeted context read is landed as `thread/epiphany/context`.

It selects from existing typed state only:

- architecture/dataflow nodes by id plus active frontier nodes by default
- graph edges by id plus incident edges for selected nodes
- graph links touching selected nodes
- active frontier and current graph checkpoint
- full durable investigation checkpoint packet when present
- observations by id
- evidence by id plus evidence linked from selected observations by default
- missing requested or active-frontier ids, so clients do not have to guess whether a shard was empty or stale

It does not run retrieval, draft proposals, promote observations, mutate
`SessionState`, append rollout items, or emit `thread/epiphany/stateUpdated`.
It is a bounded lens, not a clerk with a pen.

## Reorientation Policy

The first bounded CRRC policy surface is landed as `thread/epiphany/reorient`.

It consumes existing signals only:

- durable investigation checkpoint packet from authoritative typed state
- retrieval and graph freshness reflection inputs
- watcher-backed invalidation inputs for loaded threads
- context-pressure telemetry that already exists in `thread/epiphany/pressure`

It returns a read-only verdict:

- `resume` when a `resume_ready` checkpoint is still aligned with current watcher/freshness pressure
- `regather` when there is no state, no checkpoint, an explicit `regather_required` checkpoint, checkpoint-path drift, frontier drift, or an unanchored checkpoint under active staleness

It does not mutate `SessionState`, compact, schedule, notify `thread/epiphany/stateUpdated`, auto-resume work, or auto-regather source. It is the wakeup verdict, not the hand reaching for the keyboard.

The first explicit hand reaching for the keyboard is now
`thread/epiphany/reorientLaunch`, and even that hand stays on a short leash:
one fixed specialist role, one explicit launch call, one bounded checkpoint
packet, and no ambient runtime coordination.

## Compaction And CRRC

Compact-Rehydrate-Reorient-Continue is an architecture primitive, but automatic
runtime coordination is not landed yet.

Current operating rule:

- persist before compaction, handoff, and phase boundaries
- rehydrate from canonical state, handoff, algorithmic map, plan, and evidence
- restate the persisted next action
- continue with one bounded move

Future runtime behavior should:

- expose context pressure as a real runtime signal before the hard limit
- checkpoint at safe points when possible
- preserve role-specific resume packets
- distinguish what survived from what was discarded
- make compaction visible instead of pretending continuity is magic

The pressure signal now exists as `thread/epiphany/pressure`, the freshness
signal now exists as `thread/epiphany/freshness`, live watcher-backed
invalidation telemetry now exists inside that freshness surface for loaded
threads, durable investigation packets now exist in typed state plus
prompt/scene/context reflection, the first bounded policy verdict now exists as
`thread/epiphany/reorient`, explicit launch/interrupt authority now exists over
the thin job seam, one explicit `thread/epiphany/reorientLaunch` consumer can
act on that verdict without becoming a hidden scheduler, and
`thread/epiphany/reorientResult` can read that worker's finding back for human
or client review, and `thread/epiphany/reorientAccept` can explicitly bank an
accepted finding into typed observation/evidence plus optional scratch or
checkpoint state. What does not exist yet is the automatic runtime coordinator
that decides when to launch the worker or request acceptance. Automatic CRRC
still needs explicit runtime policy, bounded job ownership, clean stopping
rules, and honest result plumbing instead of vibes with a clipboard.

Compaction should squeeze scratch, not the map.

## Persistent State Hygiene

Persistent state can rot.

The anti-Jenga rule applies to docs and state files too:

- if a note stops improving the model, distill it
- if status prose becomes obsolete, replace it
- if a file starts accumulating every landed micro-slice, move history back to git and only preserve distilled evidence when it changes future belief
- if the next action is hidden under a pile of previous victories, the note has failed
- if the map is not source-grounded, do not decorate it with confident language

Canonical responsibilities:

- `state/map.yaml`: current project map and accepted design
- `state/evidence.jsonl`: distilled durable evidence, decisions, rejected paths, verification, and scars
- `notes/fresh-workspace-handoff.md`: re-entry packet
- `notes/epiphany-current-algorithmic-map.md`: source-grounded current control flow
- `notes/epiphany-fork-implementation-plan.md`: distilled forward implementation plan
- this file: stable harness surface contract

No single note should try to be all of those things. That way lies the tower.

## Surface Design Checklist

Before adding a new Epiphany surface, answer:

- Is it read, write, proposal, promotion, notification, or reflection?
- What is the authoritative state behind it?
- What must it never mutate?
- How does it preserve revision or freshness identity?
- How does it behave for missing, stored, stale, or unloaded thread state?
- Which test or smoke proves the contract?
- Which existing surface would become redundant if this is added?

If those answers are mushy, the surface is not ready.
