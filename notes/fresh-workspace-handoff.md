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
- Phase 6 has read-only `thread/epiphany/scene`, `thread/epiphany/jobs`, `thread/epiphany/roles`, `thread/epiphany/freshness`, `thread/epiphany/context`, `thread/epiphany/graphQuery`, `thread/epiphany/pressure`, `thread/epiphany/reorient`, `thread/epiphany/crrc`, `thread/epiphany/coordinator`, `thread/epiphany/reorientResult`, and `thread/epiphany/roleResult`; durable `jobBindings` now act as a thin Epiphany-owned launcher seam with launcher id, authority scope, and backend kind/job id, `thread/epiphany/jobLaunch` / `thread/epiphany/jobInterrupt` / `thread/epiphany/roleLaunch` now provide explicit bounded authority over that seam, jobs can overlay it onto live runtime `agent_jobs` progress for loaded threads, and `thread/epiphany/jobsUpdated` now translates live `agent_job_progress:{json}` background events into changed launcher-bound notifications without polling or scheduling. Freshness carries watcher-backed invalidation inputs, graphQuery traverses authoritative typed graph neighborhoods and path/symbol matches without mutation, roles project implementation/modeling/verification/reorientation ownership from existing signals without becoming a scheduler, `roleLaunch` can launch only fixed modeling/checkpoint or verification/review specialists, `roleResult` reads those outputs back as reviewable findings without mutation, `thread/epiphany/roleAccept` can apply a reviewed modeling/checkpoint `statePatch` through the existing state validator, reorient turns checkpoint plus freshness/pressure/watcher signals into a read-only resume-versus-regather verdict, `thread/epiphany/reorientLaunch` is the explicit runtime consumer over that verdict, `thread/epiphany/reorientResult` reads the launched worker output back as a reviewable finding without mutation or promotion, `thread/epiphany/reorientAccept` explicitly banks completed findings into accepted observation/evidence plus optional scratch/checkpoint state, `thread/epiphany/crrc` recommends the next explicit CRRC action without launching, accepting, compacting, scheduling, or mutating, and `thread/epiphany/coordinator` composes those signals into a fixed-lane MVP action recommendation without becoming a writer.
- `tools/epiphany_mvp_status.py` is the first dogfood operator view. It starts or reads a thread through app-server and prints scene, pressure, reorient, jobs, roles, modeling/verification role result read-backs, reorient result, and CRRC recommendation as text or JSON.
- `tools/epiphany_mvp_coordinator.py` is the first auditable fixed-lane coordinator runner. It starts or reads a thread through app-server, follows the harness-native coordinator action, can auto-launch modeling, verification, or reorient-worker jobs, keeps semantic findings review-gated by default, and writes summary, steps, rendered snapshots, transcript, stderr, and final next-action artifacts under `.epiphany-dogfood/coordinator` or a caller-provided artifact directory.
- Native CRRC automation is now landed only at turn-complete safe boundaries. It may submit `Op::Compact` for coordinator-approved `compactRehydrateReorient`, and it may launch the fixed `reorient-worker` for coordinator-approved `launchReorientWorker`. It does not auto-launch modeling/verification, accept findings, promote evidence, edit implementation code, or keep going after reviewable semantic output.
- Pre-compaction checkpoint intervention is now landed. On token-count events for loaded Epiphany threads, when pressure reaches the existing `shouldPrepareCompaction` threshold, the harness steers the active turn once with a CRRC checkpoint directive so the agent banks working context before compaction/reorientation. This is still bounded steering, not automatic semantic acceptance, a broad scheduler, or implementation continuation.
- `tools/epiphany_mvp_dogfood.py` is the first auditable dogfood runner. It drives a bounded state/role/CRRC/reorientation loop and writes local artifacts under `.epiphany-dogfood/mvp-loop`, including sealed app-server transcript diagnostics, rendered snapshots, operator-safe final status, vanilla-reference prompt/output, and comparison notes. The runner now writes truthful vanilla/comparison artifacts whether the optional vanilla reference is skipped, fails, or completes.
- `tools/epiphany_mvp_live_specialist.py` is the first auditable live-specialist runner. It launches a real role specialist through `thread/epiphany/roleLaunch`, lets the spawned worker report through `report_agent_job_result`, reads it back through `thread/epiphany/roleResult`, and writes local artifacts under `.epiphany-dogfood/live-specialist`.
- The first MVP GUI shell now exists under `apps/epiphany-gui`. It is a Tauri v2 + React operator console over the existing status bridge, dogfood artifacts, and GUI action artifacts, not a new source of truth. It can prepare a durable Epiphany checkpoint for a resumable operator thread, run status snapshots and coordinator-plan artifacts, launch/read the fixed modeling and verification lanes, launch/read the fixed reorient-worker, and explicitly accept completed reorient findings after review.
- Internal `agent_job:` workers now get the reporting tool independent of the user-facing CSV spawn feature, and ephemeral worker sessions can initialize the sqlite state runtime on demand so specialist reports land in the shared backend.
- Durable in-flight investigation checkpointing is now landed in authoritative typed state, writable through `thread/epiphany/update` or accepted `thread/epiphany/promote`, rendered into the prompt, and reflected through scene/context.
- The prompt doctrine pass is landed. Shared Epiphany prompts now carry distilled memory/evidence discipline, and specialist prompt text lives in `vendor/codex/codex-rs/app-server/src/prompts/epiphany_specialists.toml`: modeling protects the body, verification protects the soul, reorientation protects life, and coordinator remains the read-only Self.
- The Aetheria dogfood run has a contamination scar. The supervising Codex session directly edited and committed target-repo work on `E:\Projects\Aetheria-Economy` instead of only driving Epiphany lanes. Treat those Aetheria commits as supervisor-seeded implementation, not clean evidence that Epiphany coordinated the work. Future dogfood must run through the GUI/coordinator/fixed role lanes with auditable artifacts unless the user explicitly authorizes an operator intervention.
- The dogfood quarantine now has a direct-thought boundary. The supervisor may read coordinator actions, role/reorient statuses, structured finding summaries, reviewed state patches, rendered status snapshots, and artifact manifests. It must not read raw worker transcripts, direct worker messages, full turn logs, or `rawResult` payloads during normal dogfood. Those artifacts are sealed for explicit forensic debugging only.
- `tools/epiphany_agent_telemetry.py` is the safe instrument panel for sealed runs. Status/coordinator/GUI/dogfood/live-specialist tools generate telemetry JSON from sealed transcripts that preserves method names, call shape, job/status/path counts, and any visible function/tool names while sealing text, direct messages, and raw results.
- Latest Aetheria supervised dogfood thread: `019ddc52-4ee8-7203-b6c0-106a9c270067` with codex-home `.epiphany-dogfood/aetheria-supervised/codex-home`. CRRC compact/reorient/reorientAccept reached Epiphany state revision 16 after rollout replay fixes, and a fresh coordinator resume cleared to `continueImplementation`.
- The implementation lane is the current product blocker, but it is now harness-visible. The latest audited artifact is `.epiphany-dogfood/aetheria-supervised/gui-actions/continueImplementation-1777528376766401900-25000`: it shows `workspaceChanged: false`, wrote `implementation-audit.md`, applied a no-diff `thread/epiphany/update`, advanced Epiphany state to revision 17, and marked the investigation checkpoint `regather_required`.
- A follow-up coordinator plan at `.epiphany-dogfood/aetheria-supervised/coordinator-after-nodiff-state-20260430-plan` now returns `regatherManually` with implementation blocked, instead of continuing fake progress.
- The GUI parses `implementation-result.json` into artifact metadata, surfaces the latest implementation diff/no-diff outcome, and pauses immediate `Continue Implementation` repeats when the newest artifact is a no-diff implementation audit.
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
- Body, soul, life, and Self are technical salience handles: code shape/dataflow, objective/evidence truth, continuity across compaction, and read-only coordination.
- "Remember Jenga" is a compressed doctrine: do not mistake forward motion, growing diffs, growing notes, or local coherence for understanding.
- "Remember the sunburn" is the dogfood corollary: Epiphany's objective is attractive enough to pull the supervisor into implementation. During dogfood, the supervisor observes, launches, reads, accepts, and audits. It does not quietly become the worker.
- "Do not stare at the sun" is the projection corollary: the supervisor should not absorb the direct thought stream of the agent it is evaluating. It supervises through shadows cast on instruments: projections, summaries, verdicts, patches, and receipts.
- "Use instruments, not eyeballs" is the telemetry corollary: when the supervisor needs behavioral detail, read `agent-function-telemetry.json`, not the sealed transcript.
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
- explicit modeling/checkpoint and verification/review specialist launch through `thread/epiphany/roleLaunch`
- read-only modeling/checkpoint and verification/review specialist result read-back through `thread/epiphany/roleResult`
- review-gated modeling/checkpoint patch acceptance through `thread/epiphany/roleAccept`
- bounded reorient-guided worker launch through `thread/epiphany/reorientLaunch`
- read-only reorient-worker result read-back through `thread/epiphany/reorientResult`
- explicit reorient-worker finding acceptance through `thread/epiphany/reorientAccept`
- read-only CRRC coordinator recommendation through `thread/epiphany/crrc`
- CRRC now recognizes already accepted reorientation findings so `reorientAccept` does not leave the operator stuck on a repeat `acceptReorientResult` recommendation.
- thin launcher metadata in durable `jobBindings`
- live bound-job progress notification through `thread/epiphany/jobsUpdated`
- read-only compact reflection through `thread/epiphany/scene`
- read-only job/progress reflection through `thread/epiphany/jobs`, with durable launcher metadata plus live runtime `agent_jobs` overlay
- read-only role ownership reflection through `thread/epiphany/roles`
- read-only retrieval/graph freshness reflection plus watcher-backed invalidation inputs through `thread/epiphany/freshness`
- read-only targeted state-shard reflection through `thread/epiphany/context`
- read-only graph traversal through `thread/epiphany/graphQuery`
- read-only context-pressure reflection through `thread/epiphany/pressure`
- read-only CRRC reorientation policy through `thread/epiphany/reorient`
- read-only CRRC next-action recommendation through `thread/epiphany/crrc`
- read-only fixed-lane MVP action recommendation through `thread/epiphany/coordinator`
- limited turn-complete CRRC automation for coordinator-approved compact and fixed reorient-worker launch actions
- token-count pre-compaction checkpoint intervention for loaded Epiphany turns at the `shouldPrepareCompaction` threshold
- durable investigation checkpoint packet through typed state, prompt, scene, and context
- distilled shared and config-backed specialist prompt doctrine through the base prompt, rendered Epiphany state, modeling/body, verification/soul, reorientation/life, and coordinator/Self surfaces
- live scene app-server smoke through `tools/epiphany_phase6_scene_smoke.py`
- live jobs app-server smoke through `tools/epiphany_phase6_jobs_smoke.py`
- live freshness app-server smoke through `tools/epiphany_phase6_freshness_smoke.py`
- live watcher-backed invalidation smoke through `tools/epiphany_phase6_invalidation_smoke.py`
- live context app-server smoke through `tools/epiphany_phase6_context_smoke.py`
- live graph traversal app-server smoke through `tools/epiphany_phase6_graph_query_smoke.py`
- live pressure app-server smoke through `tools/epiphany_phase6_pressure_smoke.py`
- live reorientation app-server smoke through `tools/epiphany_phase6_reorient_smoke.py`
- live reorient-guided worker launch smoke through `tools/epiphany_phase6_reorient_launch_smoke.py`
- live MVP operator status smoke through `tools/epiphany_mvp_status_smoke.py`
- live MVP coordinator smoke through `tools/epiphany_mvp_coordinator_smoke.py`
- live job-control app-server smoke through `tools/epiphany_phase6_job_control_smoke.py`
- live specialist MVP pass through `tools/epiphany_mvp_live_specialist.py`
- Tauri v2 + React GUI operator shell under `apps/epiphany-gui`, including visual smoke, durable checkpoint preparation, fixed lane launch/read-back controls, explicit review-gated reorient acceptance, and bounded artifact-writing buttons

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
- `thread/epiphany/graphQuery` is read-only.
- `thread/epiphany/pressure` is read-only.
- `thread/epiphany/reorient` is read-only.
- `thread/epiphany/crrc` is read-only.
- `thread/epiphany/coordinator` is read-only.
- `thread/epiphany/roles` is read-only.
- `thread/epiphany/roleResult` is read-only.
- `thread/epiphany/roleAccept` is a narrow write surface for completed modeling/checkpoint `statePatch` findings only; it must not accept verification output, broad authority fields, job bindings, or arbitrary implementation work.
- `thread/epiphany/reorientResult` is read-only.
- Durable typed state writes go through `thread/epiphany/update`, accepted `thread/epiphany/promote`, `thread/epiphany/reorientAccept`, `thread/epiphany/roleAccept`, or the bounded `thread/epiphany/jobLaunch`, `thread/epiphany/jobInterrupt`, `thread/epiphany/reorientLaunch`, and `thread/epiphany/roleLaunch` authority surfaces when they mutate typed state or `jobBindings`.
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

For graph traversal behavior changes, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_graph_query_smoke.py'
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

For the fixed-lane MVP coordinator endpoint and runner, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_mvp_coordinator_smoke.py'
```

For the first MVP operator status view, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_mvp_status_smoke.py'
```

For explicit modeling/checkpoint and verification/review role launch/read-back,
run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_role_smoke.py'
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

The Phase 6 graph traversal slice is also landed. It exposes read-only
`thread/epiphany/graphQuery` for explicit node/edge lookup, path/symbol lookup,
bounded neighbor traversal, and frontier-neighborhood inspection over
authoritative typed state. It returns graph records, frontier/checkpoint
identity, matched selectors, and missing ids without mutating or notifying.

The bounded CRRC coordination/policy layer is now also landed as read-only
`thread/epiphany/reorient`. It consumes the landed pressure, freshness,
watcher, and investigation-checkpoint signals to decide whether a checkpoint
still deserves `resume` or has to admit `regather`.

Also keep the guardrail in mind: a read-only reorientation verdict and read-only
CRRC recommendation are still not automatic CRRC execution. Runtime action is
still explicit even though read-only job ownership/progress reflection now has a
real runtime seam, the thin launcher boundary is landed, and one explicit
reorient-guided launch surface can consume the verdict on purpose.

The latest auditable MVP dogfood pass has run. It exercised the landed MVP loop,
produced `.epiphany-dogfood/mvp-loop-selfdogfood-vanilla2-20260429` artifacts,
ran a real Vanilla Codex reference through app-server with a persisted
transcript, and fixed the concrete blocker where the runner's manifest claimed
vanilla/comparison artifacts that did not exist. Comparison artifacts are now
honest: they record skipped, failed, or completed vanilla-reference state
instead of pretending the receipt drawer is full.

The first live-specialist pass has also run. It produced
`.epiphany-dogfood/live-specialist` artifacts and proved the real worker path:
`roleLaunch` created a bound `agent_jobs` row, the spawned modeling worker
inspected the smoke workspace, called `report_agent_job_result`, and
`roleResult` projected a completed `checkpoint-ready` finding.
The role smoke now also proves modeling can return a reviewable graph/checkpoint
`statePatch` and `thread/epiphany/roleAccept` can apply it so the durable graph
actually grows after review.

The prompt doctrine pass has now also run. It rechecked global AGENTS,
available Codex memories, and nearby evidence ledgers before distilling the
sane parts into the shared base prompt, rendered Epiphany state, and fixed
specialist launch templates. The specialist/coordinator prompt bodies have
since been extracted from Rust into
`vendor/codex/codex-rs/app-server/src/prompts/epiphany_specialists.toml`, with
app-server parsing that bundled config at launch-template selection time. The
latest live-specialist run again produced `.epiphany-dogfood/live-specialist`
artifacts and showed the modeling worker returning the
openQuestions/evidenceGaps/risks shape from the config-backed prompt.

The fixed-lane coordinator MVP is testable, and the first pre-compaction CRRC
intervention is now wired. Limited safe-boundary CRRC execution still handles
compact and fixed reorient-worker launch actions after a turn ends; the
token-count hook now handles the earlier danger zone by steering active loaded
turns once when `shouldPrepareCompaction` is reached. The first Tauri v2 +
React operator console is now usable enough to dogfood: it renders the same
status, coordinator, role, reorient, job, and artifact surfaces through the
existing MVP status bridge; it has a Prepare Checkpoint button that creates
durable resumable Epiphany state; and it can launch/read fixed modeling,
verification, and reorient lanes plus accept completed reorient findings after
review. GUI visual smoke covers desktop and mobile screenshots and clicks the
bounded browser-fallback controls. A live bridge probe also proved
`prepareCheckpoint` creates a resumable thread and a later process can
`readModelingResult` from it.

The next real move is not more direct Aetheria implementation. The clean
supervised lane now reaches implementation, but the implementation agent is
reading and stopping without leaving a diff. The GUI surfaces that, immediate
repeat-mashing is blocked, and no-diff implementation now marks the checkpoint
`regather_required` so the coordinator returns `regatherManually` instead of
continuing. Treat this as the next product repair path: run bounded
modeling/reorientation to turn the checkpoint back into implementation-ready
guidance, then retry through the GUI/coordinator/fixed lanes and produce
artifacts that show who did what. Read only operator-safe projections; do not
open raw worker transcripts, direct
worker messages, or `rawResult` payloads unless the user explicitly asks for
forensic debugging. Use generated function/API telemetry for call-shape
visibility without sungazing. CRRC is not a specialist-agent persona; the
reorient-worker it may launch is the specialist. Do not turn the coordinator
into a broad hidden dispatcher, arbitrary marketplace, alternate job backend,
automatic semantic acceptance path, target-repo implementation worker,
direct-thought feed, or GUI-as-source-of-truth.

Live `thread/epiphany/scene`, `thread/epiphany/jobs`, `thread/epiphany/roles`,
`thread/epiphany/freshness`, `thread/epiphany/context`,
`thread/epiphany/graphQuery`, `thread/epiphany/pressure`, `thread/epiphany/reorient`,
`thread/epiphany/crrc`, `thread/epiphany/coordinator`,
`tools/epiphany_mvp_status.py`, `tools/epiphany_mvp_coordinator.py`, and
`tools/epiphany_phase6_graph_query_smoke.py` smokes are now guardrails, not the
next organs.

## Not Yet

- automatic watcher-driven semantic invalidation
- automatic observation promotion
- broad specialist-agent scheduling beyond the fixed single-user role lanes
- GUI-as-source-of-truth
- broad runtime CRRC execution beyond the landed safe-boundary compact, fixed reorient-worker launch, and pre-compaction checkpoint steering actions
- Epiphany-owned long-running job execution beyond the current runtime `agent_jobs` seam
- broad event stream beyond the landed state update notification

The machine is good enough to move outward. Do not sand the same edge until the
wood disappears.

## Immediate Re-entry Instruction

After compaction, first rehydrate and reorient from the listed files and git
state. Do not continue implementation merely because the state names a next
move. Wait for the user's next instruction unless they explicitly say to
continue.
