# Epiphany Fork Implementation Plan

This is the current implementation plan for Epiphany as an opinionated fork of
Codex.

It is not a changelog. Git history, commit messages, smoke artifacts, and
targeted logs already do the proof job without turning this file into a hay
bale; `state/evidence.jsonl` carries only distilled belief-changing evidence.

The purpose of this note is to answer four questions:

- what exists now
- what boundaries must not be blurred
- what we have learned
- what the next real implementation organs are

## Lineage

This note started as `notes/codex-epiphany-mode-plan.md` in commit `e64eee9`.
It described Phase 1: add a durable Epiphany thread-state seam to vendored
Codex without changing normal Codex behavior.

The note was later renamed because Epiphany stopped being "a Codex mode" and
became a fork-level modeling architecture. The rename was correct. The later
append-only status drift was not. A plan that records every landed micro-slice
forever becomes the documentation equivalent of the Jenga tower.

Historical detail belongs in:

- `git log`
- `state/evidence.jsonl`, when the detail changes what a future agent should believe
- `notes/fresh-workspace-handoff.md`
- `notes/epiphany-current-algorithmic-map.md`

This file carries the distilled plan.

## Current Baseline

Phase 1 through Phase 5 are complete enough.

The landed machine now has:

- durable Epiphany thread state in Codex protocol/core rollout state
- prompt integration through a bounded `<epiphany_state>` developer fragment
- typed client read exposure through `Thread.epiphanyState`
- hybrid repo retrieval through `thread/epiphany/retrieve`
- explicit persistent semantic indexing through `thread/epiphany/index`
- repo-owned heavy implementation in `epiphany-core`
- typed state mutation through `thread/epiphany/update`
- read-only observation distillation through `thread/epiphany/distill`
- read-only map/churn proposal through `thread/epiphany/propose`
- verifier-backed promotion through `thread/epiphany/promote`
- successful-write notification through `thread/epiphany/stateUpdated`
- explicit launch/interrupt authority through `thread/epiphany/jobLaunch` and `thread/epiphany/jobInterrupt`, still backed by runtime `agent_jobs`
- thin Epiphany-owned launcher metadata in durable `jobBindings`, with launcher id, authority scope, and backend kind/job id
- live bound-runtime progress notification through `thread/epiphany/jobsUpdated`
- response-level and notification-level revision and changed-field metadata
- direct-update validation for malformed appended records and structural replacements
- proposal and promotion rules that reduce map/churn Jenga pressure
- reusable Phase 5 app-server smoke coverage in `tools/epiphany_phase5_smoke.py`
- a first Phase 6 read-only reflection surface through `thread/epiphany/scene`
- live Phase 6 scene app-server smoke coverage in `tools/epiphany_phase6_scene_smoke.py`
- read-only Phase 6 job/progress reflection through `thread/epiphany/jobs`, with durable `jobBindings` plus live runtime `agent_jobs` overlay
- live Phase 6 jobs app-server smoke coverage in `tools/epiphany_phase6_jobs_smoke.py`
- read-only Phase 6 retrieval/graph freshness reflection plus watcher-backed invalidation inputs through `thread/epiphany/freshness`
- live Phase 6 freshness app-server smoke coverage in `tools/epiphany_phase6_freshness_smoke.py`
- live Phase 6 watcher-backed invalidation smoke coverage in `tools/epiphany_phase6_invalidation_smoke.py`
- read-only Phase 6 targeted state-shard reflection through `thread/epiphany/context`
- live Phase 6 context app-server smoke coverage in `tools/epiphany_phase6_context_smoke.py`
- read-only Phase 6 context-pressure reflection through `thread/epiphany/pressure`
- live Phase 6 pressure app-server smoke coverage in `tools/epiphany_phase6_pressure_smoke.py`
- durable Phase 6 investigation checkpointing in authoritative typed state, prompt rendering, and scene/context reflection
- read-only Phase 6 CRRC reorientation policy through `thread/epiphany/reorient`
- bounded Phase 6 reorient-guided worker launch through `thread/epiphany/reorientLaunch`
- read-only Phase 6 reorient-worker result read-back through `thread/epiphany/reorientResult`
- explicit Phase 6 reorient-worker finding acceptance through `thread/epiphany/reorientAccept`
- read-only Phase 6 CRRC coordinator recommendation through `thread/epiphany/crrc`
- first Phase 6 dogfood operator view through `tools/epiphany_mvp_status.py`
- live Phase 6 reorientation app-server smoke coverage in `tools/epiphany_phase6_reorient_smoke.py`
- live Phase 6 reorient-launch app-server smoke coverage in `tools/epiphany_phase6_reorient_launch_smoke.py`
- live Phase 6 MVP status smoke coverage in `tools/epiphany_mvp_status_smoke.py`
- live Phase 6 job-control app-server smoke coverage in `tools/epiphany_phase6_job_control_smoke.py`

The current phase is Phase 6: reflection boundary and observable harness state.

## Boundary Rules

These boundaries are more important than the individual method names:

- `thread/epiphany/retrieve` is read-only.
- `thread/epiphany/distill` is read-only.
- `thread/epiphany/propose` is read-only.
- Durable typed state writes go through `thread/epiphany/update`, accepted `thread/epiphany/promote`, or the bounded `thread/epiphany/jobLaunch`, `thread/epiphany/jobInterrupt`, and `thread/epiphany/reorientLaunch` authority surfaces when they mutate `jobBindings`.
- `thread/epiphany/index` may update the semantic retrieval catalog, but it is not a hidden Epiphany-state writer.
- `thread/epiphany/scene` is a client reflection, not a second source of truth.
- `thread/epiphany/jobs` is a derived reflection over retrieval summaries plus typed launcher bindings and optional backend snapshots, not a scheduler or durable runtime job store.
- `thread/epiphany/jobsUpdated` is a live notification derived from runtime progress events and launcher-bound job snapshots, not a scheduler, polling daemon, or durable runtime job store.
- `thread/epiphany/jobLaunch` and `thread/epiphany/jobInterrupt` are bounded authority surfaces over durable `jobBindings` plus the current backend adapter, not a hidden scheduler, queue, or second runtime.
- `thread/epiphany/reorientLaunch` is a bounded runtime consumer over the reorientation verdict, not automatic CRRC, a hidden queue, or permission to keep working after drift without an explicit launch.
- `thread/epiphany/reorientResult` is a read-only result read-back surface, not a promotion gate, state writer, scheduler, or hidden continuation trigger.
- `thread/epiphany/reorientAccept` is an explicit acceptance write for completed reorient-worker findings, not automatic promotion, scheduling, or permission to continue without review.
- `thread/epiphany/freshness` is a derived reflection, not automatic watcher-driven invalidation, a mutation gate, or a hidden refresh scheduler.
- `thread/epiphany/context` is a targeted reflection, not a state writer or hidden proposal engine.
- `thread/epiphany/pressure` is a context-pressure reflection, not an automatic compactor, scheduler, or CRRC coordinator.
- `thread/epiphany/reorient` is a bounded policy verdict, not an automatic runtime coordinator, scheduler, compactor, or hidden state writer.
- `thread/epiphany/crrc` is a read-only coordinator recommendation over existing signals, not a scheduler, launch button, acceptance gate, compactor, or hidden state writer.
- The GUI may render and steer typed state, but it must not manufacture canonical understanding.
- The app-server remains a host seam; Epiphany-owned machinery should live in `epiphany-core` where practical.
- Qdrant is the preferred persistent semantic backend; BM25 remains the bootstrap/fallback/control path.

If a new feature violates one of these rules, stop and redesign before writing
more Rust-flavored archaeology.

## What We Learned

State can rot exactly like code.

The failure mode is not only speculative implementation cruft. Persistent
memory can also become a pile of locally true fragments that no longer help the
next agent model the whole machine. That is the same Jenga problem with nicer
headings.

The current lessons:

- Keep the algorithmic map as the source-audited control-flow description.
- Keep the implementation plan as a distilled forward plan, not a trophy wall.
- Keep the harness-surfaces note as a surface contract, not a dump of every possible future type.
- Keep `fresh-workspace-handoff.md` as a re-entry packet, not a substitute brain.
- Keep `state/evidence.jsonl` as a durable distilled ledger, not an activity feed.
- Revert failed code hypotheses immediately.
- Distill failed or obsolete state hypotheses just as aggressively.
- Treat unpersisted source-gathering and slice-planning work as volatile. If compaction interrupts it, the correct recovery is re-gathering from source or a persisted checkpoint, not continuing from the ghost of the old context.

The plan should get shorter after a phase completes, not longer by default.

## What We Need To Know Next

The next unknown is not whether Epiphany can preserve, read, propose, promote,
and notify typed state. It can.

The next unknowns are:

- how the landed watcher-backed invalidation telemetry should be consumed without turning freshness into a secret worker
- how far the read-only CRRC recommendation should go before explicit client/operator action takes over
- whether future bounded CRRC consumers should keep reusing the landed `agent_jobs` backend through the explicit job-control seam or start defining a second backend contract
- what Phase 6 should prove before specialist scheduling begins

## MVP Cutline

The testable MVP is not "all of Epiphany." It is the smallest harness loop that
can prove the product thesis on real coding work:

1. **State loop**: an agent can externalize objective, map, scratch, evidence,
   checkpoint, and job state into typed durable state; that state survives
   turns, resume, rollback, and compaction.
2. **Role separation loop**: programming, modeling/checkpoint maintenance, and
   verification/review can be represented as distinct bounded roles instead of
   being forced into one giant context.
3. **CRRC loop**: context pressure or continuity breakage can trigger explicit
   checkpointing, rehydration, reorientation, and resume-versus-regather
   continuation based on durable evidence rather than transcript vibes.
4. **Read-back loop**: bounded specialist work launched by Epiphany, especially
   `thread/epiphany/reorientLaunch`, returns findings into a reviewable
   Epiphany surface or typed-state proposal instead of stranding them in
   generic runtime job rows.
5. **Dogfood loop**: a human can inspect the current map, scratch, evidence,
   pressure, reorientation verdict, active specialist jobs, and pending
   findings without reading Rust.

The MVP should include narrow specialists and a bounded CRRC coordinator because
those are central to the design. It should not include an arbitrary specialist
marketplace, broad ambient scheduling, automatic promotion of every tool
observation, a GUI-first workflow, or a second job backend unless the current
`agent_jobs` seam blocks the product loop.

The read-back, acceptance, coordinator, and first dogfood-view blockers are now
landed as `thread/epiphany/reorientResult`, `thread/epiphany/reorientAccept`,
`thread/epiphany/crrc`, and `tools/epiphany_mvp_status.py`. The next MVP
blocker is richer role ownership: a human can now ask the harness what it
believes and what it recommends without reading Rust, and the operator view now
shows implementation, modeling/checkpoint maintenance, verification/review, and
reorientation as distinct lanes. Those lanes are still derived views rather
than a reusable specialist registry.

## Phase 6 Direction

Phase 6 should grow observable harness state outward from the typed spine.

Useful candidates:

1. Add the smallest role-scoped specialist ownership layer that makes implementation, modeling/checkpoint maintenance, and verification/review visible as distinct work lanes without building a marketplace.
2. Keep accepted worker findings review-gated; do not convert acceptance into automatic promotion of arbitrary worker output.
3. Add targeted operator-view fields only when dogfooding exposes a real gap.

Do not spend Phase 6 polishing Phase 5 out of anxiety. The Phase 5 smoke harness
is a regression guardrail, not a ritual drum circle for summoning more tiny
hardening slices.

## Later Phases

These remain later work:

- watcher-driven semantic invalidation
- automatic observation promotion from tool output
- richer evidence and graph-shard inspection beyond the landed targeted context read
- role-scoped specialist-agent registry and scheduling
- mutation gates that warn or block broad writes when map freshness is stale
- automatic CRRC runtime coordination on top of the landed typed context-pressure telemetry
- GUI workflows for graph, evidence, job, invariant, and frontier steering

Do not start these from vibes. Each one needs a source-grounded slice plan and a
clear invariant that says what it must not break.

## Verification Guardrails

Use focused checks for the surface being changed.

Before modifying Phase 5 control-plane behavior, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase5_smoke.py'
```

Before modifying scene projection behavior, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_scene_smoke.py'
```

Before modifying jobs reflection behavior, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_jobs_smoke.py'
```

Before modifying freshness reflection behavior, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_freshness_smoke.py'
```

Before modifying watcher-backed invalidation behavior inside freshness reflection, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_invalidation_smoke.py'
```

Before modifying targeted context-shard behavior, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_context_smoke.py'
```

For app-server protocol changes, expect to run the relevant protocol tests,
regenerate stable schema fixtures when needed, and verify the generated tree is
intentional.

On this Windows machine, use:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\Meta\.cargo-target-codex'
```

Do not parallelize cargo builds or tests against the same target directory.

## Planning Rule

When this file changes, prefer replacement and distillation over accretion.

A good update should usually:

- remove obsolete phase prose
- preserve the current boundary rules
- name the next larger organ
- move historical proof into evidence or git
- leave the next agent with less to carry, not more
