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
- read-only Phase 6 fixed-lane MVP coordinator recommendation through `thread/epiphany/coordinator`, composing roles, pressure, reorient, CRRC, role results, and reorient result without mutation
- limited native Phase 6 CRRC automation at turn-complete safe boundaries, restricted to coordinator-approved compact and fixed reorient-worker launch actions
- read-only Phase 6 role ownership through `thread/epiphany/roles`, projecting implementation, modeling/checkpoint, verification/review, and reorientation lanes from typed state plus jobs/CRRC/result signals
- explicit Phase 6 role launch/read-back through `thread/epiphany/roleLaunch` and read-only `thread/epiphany/roleResult`, limited to fixed modeling/checkpoint and verification/review templates over the existing job-control seam
- first Phase 6 dogfood operator view through `tools/epiphany_mvp_status.py`
- first auditable Phase 6 dogfood runner through `tools/epiphany_mvp_dogfood.py`, producing local status snapshots, raw app-server transcript, final status artifacts, vanilla-reference output, and comparison notes
- first auditable Phase 6 fixed-lane coordinator runner through `tools/epiphany_mvp_coordinator.py`, producing coordinator summary, JSONL steps, rendered snapshots, transcript, stderr, and final next-action artifacts while keeping semantic findings review-gated by default
- first auditable Phase 6 live-specialist runner through `tools/epiphany_mvp_live_specialist.py`, proving `roleLaunch -> agent_jobs worker -> report_agent_job_result -> roleResult` without manual backend completion
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
- `thread/epiphany/coordinator` is a read-only fixed-lane MVP policy projection over existing signals, not a scheduler, launcher, acceptance gate, compactor, or hidden state writer.
- Native CRRC automation may act only at safe turn-complete boundaries and only for coordinator-approved `compactRehydrateReorient` and `launchReorientWorker` actions. It must not auto-launch modeling or verification, auto-accept semantic findings, promote evidence, edit implementation code, or silently continue after unresolved drift.
- Pre-compaction checkpoint intervention may steer an active loaded Epiphany turn once at the token-count boundary when pressure reaches the existing `shouldPrepareCompaction` threshold. The steering directive is allowed to ask the agent to bank scratch/checkpoint/map/evidence before compaction/reorientation; it must not auto-accept semantic findings, promote evidence, launch arbitrary workers, or continue implementation after unresolved drift.
- `thread/epiphany/roles` is a read-only role ownership projection, not a specialist scheduler, marketplace, launcher, acceptance gate, or second job backend.
- `thread/epiphany/roleLaunch` is a bounded authority surface for fixed modeling/checkpoint and verification/review templates, not a broad scheduler or specialist marketplace.
- `thread/epiphany/roleResult` is a read-only result projection, not a promotion gate, state writer, scheduler, or hidden continuation trigger.
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
- how much narrow coordination is needed so modeling, implementation, verification, and CRRC automation can hand off work without collapsing back into one context
- what concrete operator friction appears when the CLI MVP is tested on real work

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
marketplace, broad ambient dispatcher, automatic promotion of every tool
observation, a GUI-first workflow, or an alternate job backend for the MVP. The
useful scheduler-shaped thing is narrower: an auditable coordinator that keeps
the fixed single-user lanes in sequence, makes handoffs explicit, and preserves
operator review and interruption. CRRC is harness workflow automation, not a
specialist-agent role; it watches context pressure and continuity, persists
state, triggers compact/rehydrate/reorient behavior, and may launch a bounded
reorient-worker specialist when semantic regathering is needed.

The read-back, acceptance, CRRC recommendation, first dogfood-view, fixed-lane
coordinator, first
harness-native role ownership, and first fixed role specialist launch/result
blockers are now landed as
`thread/epiphany/reorientResult`, `thread/epiphany/reorientAccept`,
`thread/epiphany/crrc`, `tools/epiphany_mvp_status.py`,
`thread/epiphany/coordinator`, `tools/epiphany_mvp_coordinator.py`, and
`thread/epiphany/roles`, `thread/epiphany/roleLaunch`, and
`thread/epiphany/roleResult`. A human can now ask the harness what it believes,
what it recommends, which role lane owns the next visible work, which fixed-lane
action should happen next, explicitly launch modeling/checkpoint or
verification/review workers, and read their findings back without reading Rust.
The first auditable dogfood pass then added
`tools/epiphany_mvp_dogfood.py`, rendered modeling/verification findings in the
status view, and fixed CRRC's repeat-acceptance recommendation after
`reorientAccept`. The live-specialist runner then proved the real worker path by
launching a modeling/checkpoint specialist, letting it inspect the smoke
workspace, report through `report_agent_job_result`, and return a completed
`checkpoint-ready` finding through `roleResult`. The coordinator runner then
proved the sequence-locked MVP policy across cold start, clean checkpoint,
modeling, verification, CRRC drift/reorient, and high-pressure compact/dry-run
paths. Native CRRC automation then wired the proved policy into turn-complete
safe boundaries for compact and fixed reorient-worker launch only. The first
pre-compaction checkpoint intervention is now also landed: token-count pressure
events steer active loaded Epiphany turns once at the `shouldPrepareCompaction`
threshold so working context can be banked before compaction. The next MVP
question is sharper: dogfood the coordinator/status/pre-compaction loop and fix
only concrete blockers.

## Phase 6 Direction

Phase 6 should grow observable harness state outward from the typed spine.

Useful candidates:

1. Put the fixed-lane coordinator and pre-compaction checkpoint loop in front of a human operator through status/artifact review, then fix concrete usability blockers.
2. Keep accepted worker findings review-gated; do not convert acceptance into automatic promotion of arbitrary worker output.
3. Keep pre-compaction intervention narrow: steer once at `shouldPrepareCompaction`, then let explicit checkpointing, compact/resume/reorient, and review gates do their jobs.

Do not spend Phase 6 polishing Phase 5 out of anxiety. The Phase 5 smoke harness
is a regression guardrail, not a ritual drum circle for summoning more tiny
hardening slices.

## Later Phases

These remain later work:

- watcher-driven semantic invalidation
- automatic observation promotion from tool output
- richer evidence and graph-shard inspection beyond the landed targeted context read
- richer role ergonomics after the fixed single-user coordinator proves useful
- mutation gates that warn or block broad writes when map freshness is stale
- broader CRRC runtime coordination beyond the landed narrow safe-boundary compact, fixed reorient-worker launch, and pre-compaction checkpoint steering actions
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

Before modifying explicit role launch/read-back behavior, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_role_smoke.py'
```

Before modifying the fixed-lane MVP coordinator endpoint or runner, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_mvp_coordinator_smoke.py'
```

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
