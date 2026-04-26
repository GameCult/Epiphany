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
| `thread/epiphany/scene` | read-only reflection | landed, live-smoked | Compact client scene derived from authoritative Epiphany state, including checkpoint summary reflection. |
| `thread/epiphany/jobs` | read-only reflection | landed, live-smoked | Derived indexing, remap, verification, and specialist-progress slots from typed state and retrieval summaries. |
| `thread/epiphany/context` | read-only reflection | landed, live-smoked | Targeted state shard for graph nodes/edges, active frontier, graph checkpoint, investigation checkpoint, observations, and evidence. |
| `thread/epiphany/pressure` | read-only reflection | landed, live-smoked | Context-pressure gauge derived from token telemetry and recorded auto-compact/context limits. |

## Write Authority

There should be one red pen.

Durable Epiphany state may change through:

- `thread/epiphany/update`
- accepted `thread/epiphany/promote`
- normal rollout persistence of the current live `EpiphanyThreadState`

The following must stay read-only:

- `thread/epiphany/retrieve`
- `thread/epiphany/distill`
- `thread/epiphany/propose`
- `thread/epiphany/scene`
- `thread/epiphany/jobs`
- `thread/epiphany/context`
- `thread/epiphany/pressure`

`thread/epiphany/index` is a write to the retrieval catalog. It is not a
license to mutate map/evidence/churn state as a side effect.

## Reflection Authority

Scene, jobs, context, and GUI surfaces may compress state for humans and
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

- live typed job/progress state for running indexing, remap, verification, and specialist work
- `thread/epiphany/jobsUpdated` or equivalent progress notifications
- richer evidence-range and graph-shard inspection beyond the landed context shard
- watcher-driven graph/retrieval/invariant invalidation
- automatic tool-output observation promotion
- typed turn intent before broad mutation
- hard mutation gates for stale map or violated invariants
- role-scoped specialist-agent registry and scheduling
- automatic runtime Compact-Rehydrate-Reorient-Continue coordination

Do not implement these as one blob. Each needs a bounded surface, a write rule,
and a verification story.

## Job And Progress Surface Direction

The first read-only job/progress reflection is landed as `thread/epiphany/jobs`.
It reports derived slots for retrieval indexing, graph remap, invariant
verification, and future specialist work. It does not start, schedule, persist,
or notify jobs.

The next missing organ is live observable long-running work.

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
- `status`
- `itemsProcessed`
- `itemsTotal`
- `progressNote`
- `lastCheckpointAt`
- `blockingReason`
- linked subgoal or graph node ids

The landed reflection surface keeps this read-only. Future live job state must
keep the same rule: do not make job state a hidden second planner.

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

The pressure signal now exists as `thread/epiphany/pressure`, and durable
investigation packets now exist in typed state plus prompt/scene/context
reflection. What does not exist yet is the runtime coordinator that consumes
those signals. Automatic CRRC still needs typed freshness/invalidity telemetry
and an explicit policy for when a checkpoint means "resume" versus "re-gather"
instead of vibes with a clipboard.

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
