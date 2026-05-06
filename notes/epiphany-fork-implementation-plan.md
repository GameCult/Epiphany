# Epiphany Fork Implementation Plan

This is the current implementation plan for Epiphany as an opinionated fork of
Codex.

It is not a changelog. Git history, commit messages, smoke artifacts, and
targeted logs already do the proof job without turning this file into a hay
bale; `state/ledgers.msgpack` carries only distilled belief-changing evidence.

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
- `state/ledgers.msgpack`, when the detail changes what a future agent should believe
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
- live Phase 6 freshness app-server smoke coverage through native `epiphany-phase6-freshness-smoke`
- live Phase 6 watcher-backed invalidation smoke coverage in `tools/epiphany_phase6_invalidation_smoke.py`
- read-only Phase 6 targeted state-shard reflection through `thread/epiphany/context`
- live Phase 6 context app-server smoke coverage in `tools/epiphany_phase6_context_smoke.py`
- read-only Phase 6 graph traversal through `thread/epiphany/graphQuery`
- live Phase 6 graph traversal smoke coverage in `tools/epiphany_phase6_graph_query_smoke.py`
- read-only Phase 6 context-pressure reflection through `thread/epiphany/pressure`
- live Phase 6 pressure app-server smoke coverage through native `epiphany-phase6-pressure-smoke`
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
- review-gated Phase 6 modeling acceptance through `thread/epiphany/roleAccept`, limited to completed modeling/checkpoint `statePatch` findings that apply graph/frontier/checkpoint/scratch/investigation-checkpoint changes through the existing state validator
- first Phase 6 dogfood operator view through native `epiphany-mvp-status`
- first auditable Phase 6 dogfood runner through `tools/epiphany_mvp_dogfood.py`, producing local status snapshots, sealed app-server transcript diagnostics, operator-safe final status artifacts, truthful vanilla-reference output, and comparison notes
- first auditable Phase 6 fixed-lane coordinator runner through native `epiphany-mvp-coordinator`, producing coordinator summary, JSONL steps, rendered snapshots, transcript, stderr, runtime-spine status, native runtime job/result receipts for launched work, and final next-action artifacts while keeping semantic findings review-gated by default
- first Codex-independent native runtime vertebra through `epiphany-runtime-spine`, storing runtime identity, sessions, jobs, job results, and events as typed CultCache MessagePack documents, opening/completing native jobs, projecting job-result counts, and emitting a framed CultNet hello message for the native contract
- first auditable Phase 6 live-specialist runner through `tools/epiphany_mvp_live_specialist.py`, proving `roleLaunch -> agent_jobs worker -> report_agent_job_result -> roleResult` without manual backend completion
- first Phase 6 Aquarium operator shell extracted to sibling repo `E:\Projects\EpiphanyAquarium`, a Tauri v2 + React/WebGL client over the existing status bridge, dogfood artifacts, and GUI action artifacts, with its own distilled interface state/memory/doctrine plus durable checkpoint preparation, bounded status/coordinator artifact buttons, fixed modeling/verification/reorient launch and read-back buttons, and explicit review-gated reorient acceptance
- first Unity editor/runtime bridge through native `epiphany-unity-bridge`, native `epiphany-unity-bridge-smoke`, and the GUI Inspect Unity action, resolving exact project-pinned editors and writing runtime artifacts while refusing wrong or missing versions
- prompt-file ownership for major Epiphany prompt surfaces: rendered state intro/doctrine lives under `epiphany-core/src/prompts/`, while modeling/Body, research/Eyes, implementation/Hands, verification/Soul, reorientation/Life, coordinator/Self, and CRRC templates live in `vendor/codex/codex-rs/app-server/src/prompts/epiphany_specialists.toml`
- prompt-level anti-Greenspun guardrails: Epiphany keeps Codex's useful operator discipline and requires a bounded research/scout check before agents invent bespoke versions of known algorithms, parsers, schedulers, renderers, protocols, storage layers, security mechanisms, or workflow engines
- live Phase 6 reorientation app-server smoke coverage in `tools/epiphany_phase6_reorient_smoke.py`
- live Phase 6 reorient-launch app-server smoke coverage in `tools/epiphany_phase6_reorient_launch_smoke.py`
- live Phase 6 MVP status smoke coverage through native `epiphany-mvp-status-smoke`
- live Phase 6 job-control app-server smoke coverage in `tools/epiphany_phase6_job_control_smoke.py`

The current phase is Phase 6: reflection boundary and observable harness state.
The next phase direction is native-spine extraction: Codex app-server is a
temporary evacuation bridge, not the throne. Epiphany sessions, jobs, results,
events, heartbeats, role memory, and coordinator state move into CultCache-backed
domain documents spoken over CultNet, and new runtime contracts land native
first.

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
- `thread/epiphany/jobLaunch` and `thread/epiphany/jobInterrupt` are bounded authority surfaces over durable `jobBindings` plus the temporary Codex evacuation bridge, not a hidden scheduler, queue, or second runtime.
- `thread/epiphany/reorientLaunch` is a bounded runtime consumer over the reorientation verdict, not automatic CRRC, a hidden queue, or permission to keep working after drift without an explicit launch.
- `thread/epiphany/reorientResult` is a read-only result read-back surface, not a promotion gate, state writer, scheduler, or hidden continuation trigger.
- `thread/epiphany/reorientAccept` is an explicit acceptance write for completed reorient-worker findings, not automatic promotion, scheduling, or permission to continue without review.
- `thread/epiphany/freshness` is a derived reflection, not automatic watcher-driven invalidation, a mutation gate, or a hidden refresh scheduler.
- `thread/epiphany/context` is a targeted reflection, not a state writer or hidden proposal engine.
- `thread/epiphany/graphQuery` is bounded graph traversal over authoritative typed state, not retrieval, proposal, promotion, scheduling, indexing, or a state writer.
- `thread/epiphany/pressure` is a current-context pressure reflection, not cumulative spend accounting, an automatic compactor, scheduler, or CRRC coordinator.
- `thread/epiphany/reorient` is a bounded policy verdict, not an automatic runtime coordinator, scheduler, compactor, or hidden state writer.
- `thread/epiphany/crrc` is a read-only coordinator recommendation over existing signals, not a scheduler, launch button, acceptance gate, compactor, or hidden state writer.
- `thread/epiphany/coordinator` is a read-only fixed-lane MVP policy projection over existing signals, not a scheduler, launcher, acceptance gate, compactor, or hidden state writer.
- Native CRRC automation may act only at safe turn-complete boundaries and only for coordinator-approved `compactRehydrateReorient`, successful pre-compaction checkpoint handoff compact, and coordinator-approved `launchReorientWorker` actions. It must not auto-launch modeling or verification, auto-accept semantic findings, promote evidence, edit implementation code, or silently continue after unresolved drift.
- Pre-compaction checkpoint intervention may steer an active loaded Epiphany turn once at the token-count boundary when pressure reaches the existing `shouldPrepareCompaction` threshold. The steering directive is allowed to ask the agent to bank scratch/checkpoint/map/evidence before compaction/reorientation; it must not auto-accept semantic findings, promote evidence, launch arbitrary workers, or continue implementation after unresolved drift.
- `thread/epiphany/roles` is a read-only role ownership projection, not a specialist scheduler, marketplace, launcher, acceptance gate, or second job backend.
- `thread/epiphany/roleLaunch` is a bounded authority surface for fixed modeling/checkpoint and verification/review templates, not a broad scheduler or specialist marketplace.
- `thread/epiphany/roleResult` is a read-only result projection, not a promotion gate, state writer, scheduler, or hidden continuation trigger.
- `thread/epiphany/roleAccept` is a narrow modeling/checkpoint acceptance write, not automatic specialist promotion, a verifier substitute, a broad state editor, or permission for workers to accept their own output.
- The GUI may render and steer typed state, but it must not manufacture canonical understanding.
- The Unity bridge may inspect and run only the project-pinned editor through explicit bridge commands; agents must not launch PATH/default/legacy Unity directly or substitute nearby Hub versions.
- The MVP GUI target is a local Tauri v2 + React operator app over the existing app-server APIs. Tauri owns windowing and local lifecycle; app-server and typed Epiphany state remain authoritative.
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
- Keep `state/ledgers.msgpack` as a durable distilled ledger, not an activity feed.
- Revert failed code hypotheses immediately.
- Distill failed or obsolete state hypotheses just as aggressively.
- Treat unpersisted source-gathering and slice-planning work as volatile. If compaction interrupts it, the correct recovery is re-gathering from source or a persisted checkpoint, not continuing from the ghost of the old context.
- Treat repetitive slow work as a finite queue, not an attention loop. A batch, tile pass, import, migration, or repeated probe is incomplete until every required item is terminal or a concrete blocker is recorded; "pattern demonstrated" is not a done state.
- Treat pattern completion bias as a prompt and coordinator failure mode. Implementation should chase the stated objective until it is complete or concretely blocked; the coordinator should challenge implementation claims against objective progress, verifier-readable evidence, shortcuts, and pointless embellishments.

The plan should get shorter after a phase completes, not longer by default.

## What We Need To Know Next

The next unknown is not whether Epiphany can preserve, read, propose, promote,
and notify typed state. It can.

The next unknowns are:

- what bridge surface is sufficient for the next Aetheria run: named Unity operations, Aetheria-side editor probes, GUI environment status, Rider context capture, and a Rider plugin MVP
- how much controlled runtime/editor access the richer Unity bridge gives implementation and verification lanes before deeper engine probes need another slice
- how the landed watcher-backed invalidation telemetry should be consumed without turning freshness into a secret worker
- how far the read-only CRRC recommendation should go before explicit client/operator action takes over
- how much narrow coordination is needed so modeling, implementation, verification, and CRRC automation can hand off work without collapsing back into one context
- what concrete operator friction appears once the bridge-equipped GUI/operator MVP is tested on real work

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
`thread/epiphany/crrc`, native `epiphany-mvp-status`,
`thread/epiphany/coordinator`, native `epiphany-mvp-coordinator`, and
`thread/epiphany/roles`, `thread/epiphany/roleLaunch`,
`thread/epiphany/roleResult`, and `thread/epiphany/roleAccept`. A human can now ask the harness what it believes,
what it recommends, which role lane owns the next visible work, which fixed-lane
action should happen next, explicitly launch modeling/checkpoint or
verification/review workers, read their findings back without reading Rust, and
apply a reviewed modeling graph/checkpoint patch without pretending the modeler
can silently promote itself.
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
safe boundaries for compact and fixed reorient-worker launch only. Successful
pre-compaction checkpoint steering now also latches a pending compact handoff
for the completed turn, so the brake becomes a real compaction request instead
of a polite suggestion. The first
pre-compaction checkpoint intervention is now also landed: token-count pressure
events steer active loaded Epiphany turns once when current context usage reaches
80% of the active auto-compact/context limit, ignoring cumulative token spend so
CRRC does not yell at clouds. The next MVP
question is sharper: dogfood the coordinator/status/pre-compaction loop and fix
only concrete blockers. The latest self-dogfood pass found one: the dogfood
runner's manifest promised vanilla/comparison artifacts that were not actually
written. The runner now always writes the prompt, reference status, comparison,
and manifest honestly, and an explicit `--run-vanilla-reference` pass can spend
a real vanilla Codex turn and persist its transcript for comparison.

The latest Aetheria dogfood pass found the next bigger blocker, and the first
slice is now landed. The fixed lanes can catch modeling, verification, stale
job, no-diff, and reorientation failures, but the implementation lane cannot
prove engine assumptions by launching random local tools. A worker tried to
launch a legacy default Unity editor even though Aetheria pins Unity
`6000.1.10f1`. Native `epiphany-unity-bridge` now reads the project pin,
resolves only an exact Hub editor, refuses wrong or missing editors, owns the
batch/quit/projectPath command wrapper, and writes inspection/command/log
artifacts. On this machine Aetheria is correctly blocked: only Unity
`6000.4.2f1` is installed.

The detailed environment plan now lives in
`notes/epiphany-rider-unity-integration-plan.md`. Its opinionated cut is Rider
as the IDE/source-context organ, Unity as the editor/runtime fact organ, and
Epiphany as the durable coordinator/Self. The product workflow is now framed as
three integrated surfaces: Rider is the human code view for repo state, source
tree, diffs, diagnostics, and code refs; Epiphany GUI is the agent dashboard
for objectives, specialist lanes, logs/artifacts, persisted state, and
graph/control-flow views; Unity is the pinned runtime environment for tests,
probes, scene configuration, assets, shaders, and play/edit-mode evidence. The
Unity side must become editor-resident, not merely command-line: an
Aetheria-side Unity Editor package should inspect scenes, prefabs, serialized
component fields, prefab overrides, materials, shaders, ScriptableObjects, and
asset references through Unity APIs such as `SerializedObject`,
`AssetDatabase`, `PrefabUtility`, and `EditorSceneManager`, with typed
artifacts and explicit dry-run/apply boundaries for refactors. The adjacent
EpiphanyGraph React viewer is the preferred seed for GUI graph diagramming
because it consumes typed `graphs.architecture`, `graphs.dataflow`, and
`graphs.links` directly.

The detailed planning substrate now lives in
`notes/epiphany-planning-substrate.md`. Its cut is deliberately separate from
the active objective: chat produces captures, captures normalize into backlog
items, backlog items group into roadmap streams, selected work becomes
Objective Drafts, and only explicit human adoption turns a draft into
`objective.current`. The plan anticipates GitHub Issues import by treating
issues as source records that land in captures first, preserving labels,
milestones, assignees, state, timestamps, comments, PR markers, and optional
Projects v2 metadata without letting GitHub become Epiphany's internal backlog
schema.

The first runtime slice is now landed: planning state lives in
`EpiphanyThreadState`, accepts typed captures/backlog/roadmap/objective drafts
through revision-gated `thread/epiphany/update`, renders into the prompt when
present, and exposes read-only `thread/epiphany/planning` for GUI clients. The
remaining product work is GUI presentation, explicit adoption/write actions,
Imagination synthesis, and real GitHub import.

The planning/future-shape role is **Imagination**. It works beside Eyes: Eyes
scouts outside reality and existing work, while Imagination shapes captures,
backlog, roadmap streams, and Objective Drafts ahead of the active run. That
work remains non-authoritative until the human explicitly adopts an objective.

## Phase 6 Direction

Phase 6 should grow observable harness state outward from the typed spine.

Useful candidates:

1. Treat the first three-pronged Rider/Epiphany GUI/Unity workflow as landed enough for local bridge testing: Aetheria-side resident Unity editor package, named Unity bridge operations over that package, GUI Environment panel, EpiphanyGraph-backed GUI graph dashboard, Rider context bridge CLI, and a Rider plugin MVP source scaffold now exist.
2. Before the next Aetheria dogfood run, build-verify or package the Rider plugin scaffold when Gradle/wrapper support is available and install the Aetheria-pinned Unity `6000.1.10f1` editor; do not rediscover that Epiphany cannot prove runtime/editor assumptions from source inspection alone.
3. Keep dogfood execution agent-run and auditable through the fixed-lane coordinator and GUI/operator view over the same status/artifact surfaces once the bridges exist.
4. Keep accepted worker findings review-gated; do not convert acceptance into automatic promotion of arbitrary worker output.
5. Keep pre-compaction intervention narrow: steer once at `shouldPrepareCompaction`, latch the compact handoff only after successful steering, then let explicit checkpointing, compact/resume/reorient, and review gates do their jobs.

Do not spend Phase 6 polishing Phase 5 out of anxiety. The Phase 5 smoke harness
is a regression guardrail, not a ritual drum circle for summoning more tiny
hardening slices.

## Aquarium Operator Plan

Build the MVP operator UI as a local Tauri v2 + React/WebGL Aquarium in
`E:\Projects\EpiphanyAquarium`.

Aquarium is not the source of truth. It is a desktop window over the same
app-server surfaces already proved by the CLI status, coordinator, dogfood, and
live-specialist runners. Its job is to make the single-user loop usable without
terminal handling, not to invent a parallel state model. EpiphanyAgent owns the
harness, typed state, coordinator policy, and bridge tools; Aquarium owns the
interface organism, visual/audio interaction grammar, and its own local
pseudo-Epiphany state.

Current shape:

1. Maintain `E:\Projects\EpiphanyAquarium` as the UI repo with `AGENTS.md`,
   `state/map.yaml`, `state/memory.json`, scratch/evidence, and interface
   doctrine.
2. Start by consuming existing app-server JSON-RPC surfaces rather than adding new protocol unless the UI exposes a real missing primitive.
3. Make the default screen an aquarium of interactive objects, not a static
   admin panel:
   - active thread and workspace
   - coordinator recommendation and reason
   - pressure, CRRC, and reorientation status
   - fixed role lanes for implementation, modeling/checkpoint, verification/review, and reorientation
   - reviewable role and reorient findings
   - job/progress list
   - artifact bundle list with links to summaries, sealed transcript/stderr receipts, and comparison files
4. Add explicit action buttons only for already-bounded authority surfaces:
   - refresh status
   - run status snapshot
   - run coordinator pass
   - inspect project-pinned Unity runtime through the bridge
   - prepare a durable checkpoint for a resumable operator thread
   - launch fixed modeling/checkpoint role
   - accept a completed modeling/checkpoint `statePatch` after review
   - launch fixed verification/review role
   - launch fixed reorient-worker when recommended
   - accept a completed reorientation finding after review
   - open artifact folder/file
5. Keep semantic acceptance gated. Aquarium may launch bounded work and display
   findings, but it must not auto-promote evidence, auto-accept worker output,
   invent arbitrary specialists, or continue implementation after unresolved
   drift.

Implementation slices:

1. **Read-only shell**: scaffold Tauri/React, connect to app-server through the
   existing MVP status bridge, render the same data as
   native `epiphany-mvp-status`, and provide artifact bundle links. This
   slice landed first under `apps/epiphany-gui` and is now extracted to
   `E:\Projects\EpiphanyAquarium`.
2. **Bounded operator actions**: status snapshot, coordinator-plan, Unity
   runtime inspection, durable checkpoint preparation, roleLaunch, roleResult,
   reorientLaunch, reorientResult, and explicit reorientAccept flows are
   landed. Review gates remain explicit; the GUI does not auto-promote
   evidence or continue implementation after semantic findings.
3. **Dogfood launcher**: wrap `tools/epiphany_mvp_dogfood.py` and
   native `epiphany-mvp-coordinator` as explicit operator actions that write
   artifact bundles and stream progress/status into Aquarium.
4. **Usability pass**: make the current recommendation, blocked lane, pending
   review, and next safe action visually obvious enough that the user can test
   the product without reading sealed transcripts unless the user explicitly asks for forensic debugging.

Verification:

- Keep the existing CLI smokes as backend guardrails.
- Keep native `epiphany-unity-bridge-smoke` for Unity bridge regressions.
- Keep `npm run smoke:visual` for browser-layout regressions and bounded
  browser-fallback action clicks in `E:\Projects\EpiphanyAquarium`.
- For native Aquarium changes, run `npm run build`, `cargo fmt --check`,
  `cargo check`, and `npm run tauri -- build --debug --no-bundle` from
  `E:\Projects\EpiphanyAquarium`.
- Use live bridge probes when action lifecycle changes; the current
  `prepareCheckpoint -> readModelingResult` probe proved a checkpoint-created
  thread can be resumed by a later process.

## Later Phases

These remain later work:

- watcher-driven semantic invalidation
- automatic observation promotion from tool output
- richer evidence UI and graph steering beyond the landed targeted context and graph traversal reads
- richer role ergonomics after the fixed single-user coordinator proves useful
- mutation gates that warn or block broad writes when map freshness is stale
- broader CRRC runtime coordination beyond the landed narrow safe-boundary compact, fixed reorient-worker launch, and pre-compaction checkpoint steering actions
- richer editor/runtime bridges beyond the first pinned Unity bridge
- richer GUI workflows for graph, evidence, job, invariant, and frontier steering after the operator console proves useful
- GUI Planning view, explicit planning capture/adoption actions, the Imagination planning role surface, and real GitHub Issues import into typed captures; the first thread-state planning store and read-only projection are landed
- typed repetitive-work queues and final-answer gates, so the coordinator and GUI can see unfinished batch work before conversational closure pretends it is done
- stronger coordinator challenge loops for implementation claims, including objective-progress checks and explicit shortcut/embellishment rejection before another implementation turn is treated as successful

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
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-freshness-smoke
```

Before modifying watcher-backed invalidation behavior inside freshness reflection, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_invalidation_smoke.py'
```

Before modifying targeted context-shard behavior, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_context_smoke.py'
```

Before modifying graph traversal behavior, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_graph_query_smoke.py'
```

Before modifying planning state or projection behavior, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase6_planning_smoke.py'
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
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-mvp-coordinator-smoke
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
