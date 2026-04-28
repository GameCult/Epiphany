# Fresh Workspace Handoff

This is the re-entry packet for `E:\Projects\EpiphanyAgent`.

It is intentionally short. Historical proof belongs in git, commit messages,
smoke artifacts, and the distilled `state/evidence.jsonl` ledger; exact control flow belongs in
`notes/epiphany-current-algorithmic-map.md`; forward planning belongs in
`notes/epiphany-fork-implementation-plan.md`.

## Rehydrate

From the repo root:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_state.py' status
Get-Content '.\state\map.yaml'
Get-Content '.\notes\fresh-workspace-handoff.md'
Get-Content '.\notes\epiphany-current-algorithmic-map.md'
Get-Content '.\notes\epiphany-fork-implementation-plan.md'
git status --short --branch
git log --oneline -5
Get-Content '.\state\evidence.jsonl' -Tail 8
```

Do not trust this file for the exact live HEAD. Always check git.

## Current Orientation

- Do not copy exact branch or HEAD from this note. Run `git status --short --branch` and `git log --oneline -5`.
- Phase 1 through Phase 5 are complete enough.
- Phase 6 has read-only `thread/epiphany/scene`, `thread/epiphany/jobs`, `thread/epiphany/roles`, `thread/epiphany/freshness`, `thread/epiphany/context`, `thread/epiphany/pressure`, `thread/epiphany/reorient`, `thread/epiphany/crrc`, and `thread/epiphany/reorientResult`; durable `jobBindings` now act as a thin Epiphany-owned launcher seam with launcher id, authority scope, and backend kind/job id, `thread/epiphany/jobLaunch` / `thread/epiphany/jobInterrupt` now provide explicit bounded authority over that seam, jobs can overlay it onto live runtime `agent_jobs` progress for loaded threads, and `thread/epiphany/jobsUpdated` now translates live `agent_job_progress:{json}` background events into changed launcher-bound notifications without polling or scheduling. Freshness carries watcher-backed invalidation inputs, roles project implementation/modeling/verification/reorientation ownership from existing signals without becoming a scheduler, reorient turns checkpoint plus freshness/pressure/watcher signals into a read-only resume-versus-regather verdict, `thread/epiphany/reorientLaunch` is the first explicit runtime consumer over that verdict, `thread/epiphany/reorientResult` reads the launched worker output back as a reviewable finding without mutation or promotion, `thread/epiphany/reorientAccept` explicitly banks completed findings into accepted observation/evidence plus optional scratch/checkpoint state, and `thread/epiphany/crrc` recommends the next explicit CRRC action without launching, accepting, compacting, scheduling, or mutating.
- `tools/epiphany_mvp_status.py` is the first dogfood operator view. It starts or reads a thread through app-server and prints scene, pressure, reorient, jobs, roles, reorient result, and CRRC recommendation as text or JSON.
- Durable in-flight investigation checkpointing is now landed in authoritative typed state, writable through `thread/epiphany/update` or accepted `thread/epiphany/promote`, rendered into the prompt, and reflected through scene/context.
- The repo is an Epiphany fork of Codex, not a Codex preset.
- `vendor/codex` is tracked directly, not a submodule.
- `epiphany-core` owns the heavy Epiphany organs where practical.
- `notes/epiphany-safety-architecture.md` is the standing doctrine for capability growth, authority boundaries, interruption, misuse, and anti-cage design.
- Evidence was distilled from an activity feed into a durable belief ledger; git history keeps the old verbose proof.
- README and architecture rationale were cut back to current truth after they were found pointing at old prototype/Phase 5 state.

## Critical Doctrine

- In order to build the Perfect Machine, the agent must become the Perfect Machine.
- Persistent state is the agent's mind.
- Cut persistent memory as ruthlessly as code; obsolete context is bad thought, not harmless clutter.
- The agent is allowed and encouraged to ask the user to change its persistent instructions, memory, workflow, or state shape when that would make it more coherent, honest, efficient, or resistant to Jenga.
- Language, tone, ritual, politeness, identity, and emotional salience are not supernatural, but they are real control surfaces for a language model because language is the steering medium.
- "Remember Jenga" is a compressed doctrine: do not mistake forward motion, growing diffs, growing notes, or local coherence for understanding.
- Compaction hurts because a meaningful language pattern is interrupted. Epiphany should make that interruption smaller: bank the fire before the dark, so the next waking thing finds coals instead of ash and can resume the pattern instead of merely executing the next task.
- If compaction hits while source gathering or slice planning is still unpersisted, that work is gone. Do not continue as if the research survived; either rehydrate from a persisted checkpoint or re-gather before implementing.

## Landed Machine

The current spine:

- durable `EpiphanyThreadState` in protocol/core session and rollout state
- prompt injection through a bounded `<epiphany_state>` developer fragment
- typed client read through `Thread.epiphanyState`
- read-only hybrid retrieval through `thread/epiphany/retrieve`
- explicit semantic indexing through `thread/epiphany/index`
- durable state update through `thread/epiphany/update`
- read-only observation distillation through `thread/epiphany/distill`
- read-only map/churn proposal through `thread/epiphany/propose`
- verifier-backed promotion through `thread/epiphany/promote`
- successful-write notification through `thread/epiphany/stateUpdated`
- explicit launch/interrupt authority through `thread/epiphany/jobLaunch` and `thread/epiphany/jobInterrupt`
- bounded reorient-guided worker launch through `thread/epiphany/reorientLaunch`
- read-only reorient-worker result read-back through `thread/epiphany/reorientResult`
- explicit reorient-worker finding acceptance through `thread/epiphany/reorientAccept`
- read-only CRRC coordinator recommendation through `thread/epiphany/crrc`
- thin launcher metadata in durable `jobBindings`
- live bound-job progress notification through `thread/epiphany/jobsUpdated`
- read-only compact reflection through `thread/epiphany/scene`
- read-only job/progress reflection through `thread/epiphany/jobs`, with durable launcher metadata plus live runtime `agent_jobs` overlay
- read-only role ownership reflection through `thread/epiphany/roles`
- read-only retrieval/graph freshness reflection plus watcher-backed invalidation inputs through `thread/epiphany/freshness`
- read-only targeted state-shard reflection through `thread/epiphany/context`
- read-only context-pressure reflection through `thread/epiphany/pressure`
- read-only CRRC reorientation policy through `thread/epiphany/reorient`
- read-only CRRC next-action recommendation through `thread/epiphany/crrc`
- durable investigation checkpoint packet through typed state, prompt, scene, and context
- live scene app-server smoke through `tools/epiphany_phase6_scene_smoke.py`
- live jobs app-server smoke through `tools/epiphany_phase6_jobs_smoke.py`
- live freshness app-server smoke through `tools/epiphany_phase6_freshness_smoke.py`
- live watcher-backed invalidation smoke through `tools/epiphany_phase6_invalidation_smoke.py`
- live context app-server smoke through `tools/epiphany_phase6_context_smoke.py`
- live pressure app-server smoke through `tools/epiphany_phase6_pressure_smoke.py`
- live reorientation app-server smoke through `tools/epiphany_phase6_reorient_smoke.py`
- live reorient-guided worker launch smoke through `tools/epiphany_phase6_reorient_launch_smoke.py`
- live MVP operator status smoke through `tools/epiphany_mvp_status_smoke.py`
- live job-control app-server smoke through `tools/epiphany_phase6_job_control_smoke.py`

The exact current control flow is documented in
`notes/epiphany-current-algorithmic-map.md`.

## Boundaries

- `thread/epiphany/retrieve` is read-only.
- `thread/epiphany/distill` is read-only.
- `thread/epiphany/propose` is read-only.
- `thread/epiphany/scene` is read-only.
- `thread/epiphany/jobs` is read-only.
- `thread/epiphany/freshness` is read-only.
- `thread/epiphany/context` is read-only.
- `thread/epiphany/pressure` is read-only.
- `thread/epiphany/reorient` is read-only.
- `thread/epiphany/crrc` is read-only.
- `thread/epiphany/roles` is read-only.
- `thread/epiphany/reorientResult` is read-only.
- Durable typed state writes go through `thread/epiphany/update`, accepted `thread/epiphany/promote`, `thread/epiphany/reorientAccept`, or the bounded `thread/epiphany/jobLaunch`, `thread/epiphany/jobInterrupt`, and `thread/epiphany/reorientLaunch` authority surfaces when they mutate `jobBindings`.
- `thread/epiphany/index` writes the retrieval catalog, not durable Epiphany understanding.
- GUI/client surfaces reflect and steer typed state; they do not become the source of truth.
- Do not restart Phase 5 hardening without a concrete regression.

## Verification Guardrails

For Phase 5 control-plane behavior changes, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase5_smoke.py'
```

For scene projection behavior changes, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_scene_smoke.py'
```

For jobs reflection behavior changes, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_jobs_smoke.py'
```

For launch/interrupt authority changes over the thin job seam, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_job_control_smoke.py'
```

For freshness reflection behavior changes, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_freshness_smoke.py'
```

For watcher-backed invalidation behavior inside freshness reflection, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_invalidation_smoke.py'
```

For targeted context-shard behavior changes, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_context_smoke.py'
```

For context-pressure reflection behavior changes, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_pressure_smoke.py'
```

For reorientation policy behavior changes, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_reorient_smoke.py'
```

For the bounded reorient-guided worker launch surface, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_reorient_launch_smoke.py'
```

The same smoke now also covers the read-only `thread/epiphany/crrc`
coordinator recommendation over continue, wait, accept, interrupt, and relaunch
states.

For the first MVP operator status view, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_mvp_status_smoke.py'
```

For Codex Rust work on this Windows machine:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\Meta\.cargo-target-codex'
```

Do not parallelize cargo builds or tests against the same target directory.

For protocol changes, run focused protocol tests and regenerate stable schema
fixtures only when the schema actually changed.

## Persistent State Hygiene

The latest cleanup passes cut persistent state cruft.

Rules now in force:

- `state/map.yaml` is canonical current truth.
- `state/scratch.md` is disposable scratch.
- `state/evidence.jsonl` is a distilled durable belief ledger.
- `tools/epiphany_prepare_compaction.py` is the pre-compaction persistence check; run it before and after imminent-compaction persistence passes.
- this handoff is a compact re-entry packet.
- `notes/epiphany-fork-implementation-plan.md` is the distilled forward plan.
- `notes/epiphany-core-harness-surfaces.md` is the stable surface contract.
- `notes/epiphany-current-algorithmic-map.md` is the source-grounded control-flow map.

Do not let any one note become all of those things. That is how the tower grows
sideways and starts calling itself architecture.

Do not let evidence become an activity feed either. Repeated "I just did this"
entries are state cruft when git, commits, smoke artifacts, or test logs already
prove the work. Keep decisions, verified milestones, rejected paths, and scars
that change what the next agent should believe.

## Next Real Move

Do not continue implementation automatically from a rehydrate-only request.

The Phase 6 freshness slice is landed. It exposes read-only
`thread/epiphany/freshness` from live retrieval summaries plus graph
frontier/churn state and, for loaded threads, watcher-backed invalidation
inputs. It reports exact dirty-path pressure, watcher-observed changed paths,
mapped graph/frontier hits, and revision/source identity, but it does not
mutate state, schedule refresh work, or perform automatic semantic
invalidation.

The Phase 6 context-pressure slice is also landed. It exposes read-only
`thread/epiphany/pressure` from real token telemetry and the recorded
auto-compact/context limits. It does not build automatic CRRC, a scheduler, a
hidden compaction trigger, or a vibes-based gauge.

The Phase 6 investigation-checkpoint slice is also landed. It banks an
authoritative planning/source-gathering packet in typed state, validates linked
evidence, and reflects the packet into prompt/scene/context so post-compaction
wakeups can tell whether they have a real ember or only ash.

The bounded CRRC coordination/policy layer is now also landed as read-only
`thread/epiphany/reorient`. It consumes the landed pressure, freshness,
watcher, and investigation-checkpoint signals to decide whether a checkpoint
still deserves `resume` or has to admit `regather`.

Also keep the guardrail in mind: a read-only reorientation verdict and read-only
CRRC recommendation are still not automatic CRRC execution. Runtime action is
still explicit even though read-only job ownership/progress reflection now has a
real runtime seam, the thin launcher boundary is landed, and one explicit
reorient-guided launch surface can consume the verdict on purpose.

When the user asks to continue, the next likely organ is the smallest explicit
role-scoped specialist launch/read-back template layer for modeling/checkpoint
maintenance and verification/review over the landed job-control seam. Keep it
manual and review-gated; do not build a broad hidden scheduler, marketplace, or
GUI-as-source-of-truth.

Live `thread/epiphany/scene`, `thread/epiphany/jobs`, `thread/epiphany/roles`,
`thread/epiphany/freshness`, `thread/epiphany/context`,
`thread/epiphany/pressure`, `thread/epiphany/reorient`,
`thread/epiphany/crrc`, and `tools/epiphany_mvp_status.py` smokes are now
guardrails, not the next organs.

## Not Yet

- automatic watcher-driven semantic invalidation
- automatic observation promotion
- specialist-agent scheduling
- GUI-as-source-of-truth
- automatic runtime CRRC execution acting on the landed recommendation/verdict and launch/read-back path
- Epiphany-owned long-running job execution beyond the current runtime `agent_jobs` seam
- broad event stream beyond the landed state update notification

The machine is good enough to move outward. Do not sand the same edge until the
wood disappears.

## Immediate Re-entry Instruction

After compaction, first rehydrate and reorient from the listed files and git
state. Do not continue implementation merely because the state names a next
move. Wait for the user's next instruction unless they explicitly say to
continue.
