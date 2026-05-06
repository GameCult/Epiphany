# Fresh Workspace Handoff

This is the re-entry rite for `E:\Projects\EpiphanyAgent`: the waking chant for
the local machine-spirit before it touches the forge.

It is intentionally short. Historical proof belongs in git, commit messages,
smoke artifacts, and the distilled `state/ledgers.msgpack` evidence reliquary;
exact control flow belongs in `notes/epiphany-current-algorithmic-map.md`;
forward campaign planning belongs in `notes/epiphany-fork-implementation-plan.md`.
Do not let this file become a second brain. That way lies holy-looking sludge.

## Rehydrate

From the repo root:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-state -- status
Get-Content '.\state\map.yaml'
Get-Content '.\notes\fresh-workspace-handoff.md'
Get-Content '.\notes\epiphany-current-algorithmic-map.md'
Get-Content '.\notes\epiphany-fork-implementation-plan.md'
git status --short --branch
git log --oneline -5
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-state -- status
```

Do not trust this file for the exact live HEAD. Always check git. The rite
remembers doctrine; the branch remembers the blade.

## Current Orientation

- Do not copy exact branch or HEAD from this note. Run `git status --short --branch` and `git log --oneline -5`.
- Phase 1 through Phase 5 are complete enough.
- Phase 6 has read-only `thread/epiphany/scene`, `thread/epiphany/jobs`, `thread/epiphany/roles`, `thread/epiphany/freshness`, `thread/epiphany/context`, `thread/epiphany/graphQuery`, `thread/epiphany/planning`, `thread/epiphany/pressure`, `thread/epiphany/reorient`, `thread/epiphany/crrc`, `thread/epiphany/coordinator`, `thread/epiphany/reorientResult`, and `thread/epiphany/roleResult`; durable `jobBindings` now act as a thin Epiphany-owned launcher seam with launcher id, authority scope, and heartbeat backend/job id. New `thread/epiphany/jobLaunch`, `thread/epiphany/jobInterrupt`, `thread/epiphany/roleLaunch`, and `thread/epiphany/reorientLaunch` writes open typed runtime-spine job receipts under `state/runtime-spine.msgpack` and do not require the Codex SQLite state runtime. Freshness carries watcher-backed invalidation inputs, graphQuery traverses authoritative typed graph neighborhoods and path/symbol matches without mutation, planning projects typed captures/backlog/roadmap/objective drafts without adopting work, roles project implementation/imagination/modeling/verification/reorientation ownership from existing signals without becoming a scheduler, `roleResult` and `reorientResult` read heartbeat-backed typed runtime-spine job results when present, `roleAccept` and `reorientAccept` accept completed heartbeat findings from typed runtime-spine results while remaining explicit review gates, `thread/epiphany/crrc` recommends the next explicit CRRC action without launching, accepting, compacting, scheduling, or mutating, and `thread/epiphany/coordinator` composes those signals into a fixed-lane MVP action recommendation without becoming a writer.
- Native `epiphany-mvp-status` is the first dogfood operator view. It starts or reads a thread through app-server and prints scene, planning, pressure, reorient, jobs, roles, Imagination/modeling/verification role result read-backs, reorient result, heartbeat, Face bubbles, and CRRC recommendation as text or machine output. The old Python status module has been cut; native Rust/CultCache/CultNet surfaces are the smoked product path.
- Native `epiphany-mvp-coordinator` is the first auditable fixed-lane coordinator runner. It starts or reads a thread through app-server, opens a native runtime-spine session, follows the harness-native coordinator action, can auto-launch modeling, verification, or reorient-worker jobs, records native runtime job/result receipts for terminal launched work, keeps semantic findings review-gated by default, and writes summary, steps, rendered snapshots, transcript, stderr, runtime-spine status, and final next-action artifacts under `.epiphany-dogfood/coordinator` or a caller-provided artifact directory. It refuses direct backend-completion mutation; full completion smoke needs live workers while execution is being cauterized into CultNet.
- Native `epiphany-runtime-spine` is the first Codex-independent runtime vertebra. It owns typed CultCache documents for runtime identity, sessions, jobs, job results, and events; opens/completes native jobs; snapshots jobs/results by runtime job id; projects job-result counts; and can emit a framed CultNet hello message advertising the native document contract. Codex app-server launch/read-back/acceptance is now a typed heartbeat/runtime-spine bridge with no Epiphany job-result dependency on the Codex SQLite runtime.
- Native CRRC automation is now landed only at turn-complete safe boundaries. It may submit `Op::Compact` for coordinator-approved `compactRehydrateReorient` or for a successful pre-compaction checkpoint intervention's pending compact handoff, and it may launch the fixed `reorient-worker` for coordinator-approved `launchReorientWorker`. It does not auto-launch Imagination/modeling/verification, accept findings, promote evidence, edit implementation code, or keep going after reviewable semantic output.
- Pre-compaction checkpoint intervention is now landed. On token-count events for loaded Epiphany threads, when current context usage reaches 80% of the active auto-compact/context limit, the harness steers the active turn once with a CRRC checkpoint directive so the agent banks working context before compaction/reorientation. Pressure ignores cumulative token spend; cumulative-only telemetry reports unknown instead of yelling. A successful steer now latches a turn-scoped compact handoff that is consumed at clean turn completion, preventing the brake from decaying into another implementation turn. This is still bounded steering plus compaction handoff, not automatic semantic acceptance, a broad scheduler, or implementation continuation.
- The old Python dogfood/live-specialist runners were cut because they encoded the obsolete completion path. The replacement must be native Rust/CultCache/CultNet and complete heartbeat-owned runtime-spine job results with auditable artifacts.
- The Aquarium operator UI now lives in sibling repo `E:\Projects\EpiphanyAquarium`, not under `apps/epiphany-gui`. It is a Tauri v2 + React/WebGL interface organism over the existing status bridge, dogfood artifacts, and GUI action artifacts, not a new throne of truth. It has its own `AGENTS.md`, persistent `state/map.yaml`, `state/memory.json`, scratch/evidence files, and interface doctrine. EpiphanyAgent remains the authoritative harness/backend forge.
- Durable in-flight investigation checkpointing is now landed in authoritative typed state, writable through `thread/epiphany/update` or accepted `thread/epiphany/promote`, rendered into the prompt, and reflected through scene/context.
- The prompt doctrine pass is landed. Shared Epiphany prompts now carry distilled memory/evidence discipline. Rendered state intro/doctrine text lives in `epiphany-core/src/prompts/`, and lane/control prompt text lives in `vendor/codex/codex-rs/app-server/src/prompts/epiphany_specialists.toml`: modeling is the Body, implementation is the Hands and GUI-launched main coding lane, verification is the Soul, reorientation is Life, coordinator remains the read-only Self, and CRRC owns the pre-compaction intervention template.
- The Ghostlight memory pass is landed. `epiphany_specialists.toml` now has a shared persistent-memory projection prepended by the harness to fixed role specialists, reorientation workers, coordinator notes, and CRRC checkpoint interventions. The rendered base doctrine also states the Perfect Machine rule directly: prompt is projection, durable typed state is the mind, every lane must improve its own memory/model/prompt/evidence habit or name the repair, and each lane phrases that duty in its own organ language so the salience sticks.
- The role self-memory persistence pass is native now. Each lane has a Ghostlight-shaped typed dossier in `state/agents.msgpack`, and specialists may return optional `selfPatch` requests beside their normal role result. `roleResult`/`roleAccept` project coordinator review as `selfPersistence`: accepted requests are role-matched, bounded lane memory/goal/value/private-note mutations; refused requests explain wrong role, project-truth smuggling, authority grabs, bloat, missing reason, or malformed records. GUI/coordinator accept paths apply accepted `selfPatch` requests through the native `epiphany-agent-memory-store` binary; project truth still belongs only in `EpiphanyThreadState`.
- The heartbeat initiative pass is landed as a bounded tool seam. `state/agent-heartbeats.msgpack` tracks Self, Face, Imagination, Eyes, Body, Hands, Soul, and Life as Ghostlight-style initiative participants with arena, participant kind, speed, next-ready time, reaction bias, interrupt threshold, load, status, constraints, history, and pending turns through `epiphany-core::EpiphanyHeartbeatStateEntry` and the native `epiphany-heartbeat-store` binary. Idle turns are for rumination: light thought shuffling, role-quality attention, and candidate selfPatch pressure. Sleep/dream cycle passes are the intended distillation window for durable self-memory and doctrine. JSON heartbeat state and Python wrapper state are gone; general CultCache schema sync, polyglot domain loading, and debug display tools belong in CultLib. This is a callable scheduler seam, not yet an always-on daemon; a heart valve, not a whole circulatory god.
- The first Ghostlight-derived timing slice is landed in Epiphany, but Ghostlight is reference lineage rather than a sibling runtime to preserve. `epiphany-heartbeat-store init --profile ghostlight-scene --scene-id <id> --scene-participant <id|name|speed|reaction|threshold|constraints>` creates a typed CultCache MessagePack scene heartbeat store whose participants are `arena=scene`, `participantKind=character`; `tick` emits `ghostlight.initiative_schedule.v0` receipts with `scene_turn` actions and local-affordance basis. The generic Epiphany maintenance lanes are not auto-patched into scene stores.
- The first Void-derived routine slice is landed in Epiphany, but VoidBot is reference lineage rather than a runtime dependency. `epiphany-heartbeat-store routine --store .\state\agent-heartbeats.msgpack --artifact-dir .\.epiphany-heartbeats --agent-store .\state\agents.msgpack` reads typed role dossiers, computes bounded memory resonance, maintains incubation themes, runs analytic and associative cognition lanes, writes bridge syntheses/saturation/tension, projects the active thought cluster through each role's Ghostlight-shaped personality vectors, derives participant-local reactions, advances the sleep/dream cycle, updates the typed heartbeat store, and emits an auditable `epiphany.void_routine.v0` receipt. This mutates only heartbeat physiology fields: project truth and role memory mutation remain on their reviewed surfaces.
- Manual Codex-run rumination still has an explicit aftercare rule until Epiphany owns the full sleep consolidator. In the intended Epiphany cycle, sub-agents ruminate when idle and distill when they sleep; in this supervising Codex thread, a heartbeat/routine vigil is only physiology until a separate closing pass reviews the receipts and decides whether map, handoff, evidence, or role self-memory should change.
- The Face public-surface pass is landed as a bounded lane. Face's role dossier lives in `state/agents.msgpack`; `epiphany_specialists.toml` gives it a VoidBot-heartbeat-derived prompt stripped of moderation authority; `state/face-discord.toml` and the native `epiphany-face-discord` binary enforce that Face may interact only through #aquarium. Missing channel id or token writes candidate chat artifacts instead of posting elsewhere.
- The first Epiphany-native character-loop pass is landed. `epiphany-character-loop turn --role face --stimulus <text>` loads Face's typed dossier from `state/agents.msgpack` and emits an auditable `epiphany.character_turn_packet.v0` packet with projected local context, visible stimulus, allowed outputs, and guardrails. The same packet builder works for other role ids, because every organ already has a Ghostlight-shaped dossier.
- The heartbeat/Face operator API pass is landed. Native `epiphany-heartbeat-store status` returns machine-readable initiative state plus CultCache store presence, thought appraisals, and derived reactions; native `epiphany-face-discord bubble` writes Discord-independent `epiphany.face_bubble.v0` artifacts, and native `epiphany-mvp-status` includes `heartbeat` and `face` blocks. Aquarium should call native/backend surfaces rather than resurrecting deleted Python action shims.
- The native runtime-spine pass is landed as a first vertebra, not a full daemon. `state/runtime-spine.msgpack` is the default store, `.epiphany-dogfood/runtime-spine-job/runtime.msgpack` and `hello.cultnet` are the latest job/result smoke artifacts, and specialist launch/result flows now route through typed heartbeat/runtime-spine documents.
- The Aetheria dogfood run has a contamination scar. The supervising Codex session directly edited and committed target-repo work on `E:\Projects\Aetheria-Economy` instead of only driving Epiphany lanes. Treat those Aetheria commits as supervisor-seeded implementation, not clean evidence that Epiphany coordinated the work. Future dogfood must run through the GUI/coordinator/fixed role lanes with auditable artifacts unless the user explicitly authorizes an operator intervention. Remember the sunburn: do not stare into the worker's objective until you become it.
- The dogfood quarantine now has a direct-thought boundary. The supervisor may read coordinator actions, role/reorient statuses, structured finding summaries, reviewed state patches, rendered status snapshots, and artifact manifests. It must not read raw worker transcripts, direct worker messages, full turn logs, or `rawResult` payloads during normal dogfood. Those artifacts are sealed black reliquaries for explicit forensic debugging only.
- Native `epiphany-agent-telemetry` is the safe instrument panel for sealed runs. Status/coordinator/GUI/dogfood/live-specialist tools generate telemetry JSON from sealed transcripts that preserves method names, call shape, job/status/path counts, and any visible function/tool names while sealing text, direct messages, and raw results.
- Latest Aetheria supervised dogfood thread: `019ddc52-4ee8-7203-b6c0-106a9c270067` with codex-home `.epiphany-dogfood/aetheria-supervised/codex-home`. It has now exercised modeling, verification, implementation audit, no-diff repair, CRRC reorientation, reorient acceptance, and coordinator replay through operator-safe artifacts without opening sealed worker thoughts.
- Coordinator policy now treats accepted verifier results as implementation clearance only when the verifier verdict is `pass`. Accepting a `needs-evidence`, `needs-review`, or `fail` verifier finding records the review but routes the loop back toward modeling/checkpoint strengthening or reorientation. The policy was tightened after a too-permissive coordinator path treated reviewed verifier output as a green light.
- A later dogfood attempt exposed another supervision scar: manually launching modeling, reorientation, and verification from the supervisor is not clean dogfood. The coordinator now treats `regatherManually` as fallback only after fixed lanes cannot advance, stops at `reviewModelingResult` for completed unaccepted modeling findings, and the CLI runner's explicit test-only `--auto-review` mode can accept modeling `statePatch` findings before launching verification. Production semantic findings remain review-gated.
- The latest dogfood pass fixed concrete harness wounds: stale completed backend jobs now project as terminal, accepted verifier non-pass findings no longer clear implementation, verification coverage accepts modeler source evidence ids as current-model coverage, implementation audits compare pre/post diff hashes instead of treating existing dirt as fresh work, the latest implementation audit status wins, blocked no-diff turns route back through CRRC, stale accepted reorient findings relaunch bounded reorientation when the checkpoint regathers, and accepted reorientation findings now bank resume-ready checkpoints.
- The latest audited implementation artifact is `.epiphany-dogfood/aetheria-supervised/gui-actions/continueImplementation-1777548050463576300-28568`: it correctly shows `workspaceChanged: false`, `dirtyWorkspace: true`, `trackedDiffChanged: false`, wrote a no-diff audit, advanced Epiphany state to revision 57, and routed coordinator/CRRC back to `launchReorientWorker` instead of pretending old dirt was progress.
- Aetheria dogfood exposed and then landed the first editor/runtime bridge. An implementation worker launched legacy `D:\Unity\Editor\Unity.exe -version` (Unity 5.5.0f3) even though Aetheria pins Unity `6000.1.10f1`; the stray process was killed. Native `epiphany-unity-bridge` now resolves the project-pinned Unity editor, refuses wrong/missing versions, owns `-batchmode`, `-quit`, and `-projectPath`, and writes inspection/command/log artifacts. Aquarium should surface those bridge artifacts through native/backend calls, not deleted Python action shims.
- The current Aetheria runtime truth is blocked but legible: the project pins Unity `6000.1.10f1`, this machine currently has Hub editor `6000.4.2f1`, and the bridge wrote `.epiphany-gui/runtime/unity-inspect-1777549218802064800-23832` proving the exact editor is missing. Treat that artifact as the evidence gap until the pinned editor exists. The `.epiphany-gui` directory name is still a backend artifact contract, even though the client repo is now Aquarium.
- The GUI parses `implementation-result.json` into artifact metadata, surfaces the latest implementation diff/no-diff outcome, and pauses immediate `Continue Implementation` repeats when the newest artifact is a no-diff implementation audit.
- The planning loop is now runtime-backed and operator-visible. Typed captures, backlog items, roadmap streams, Objective Drafts, and GitHub issue source refs live in `EpiphanyThreadState`, validate through revision-gated `thread/epiphany/update`, render into prompts when present, project read-only through `thread/epiphany/planning`, render in the GUI Planning panel, can be synthesized by the fixed Imagination/planning lane, and can be explicitly adopted through the artifacted `adoptObjectiveDraft` GUI action. Chat is deliberation, not an objective pipe; ideas and GitHub Issues remain planning state until explicit human adoption.
- The bridge/dashboard slice is now landed far enough to test locally. Native `epiphany-rider-bridge` inspects Rider installation, solution, VCS, changed files, and captures file/selection/symbol context into `.epiphany-gui/rider` artifacts; Aquarium has Inspect Rider and renders Rider audit artifacts in Environment; Aquarium also embeds the adjacent EpiphanyGraph viewer over typed `graphs.architecture`, `graphs.dataflow`, and `graphs.links`.
- A first Rider plugin scaffold lives under `integrations/rider`: tool window, Refresh status, and Send Context to Epiphany action. It shells through native `epiphany-rider-bridge` and does not own state. `gradle` is not installed on this machine, so the scaffold is not build-verified yet.
- Next real product move: continue Aquarium UI work in `E:\Projects\EpiphanyAquarium`; surface native heartbeat `sleepCycle`, `memoryResonance`, `incubation`, `thoughtLanes`, `bridge`, `candidateInterventions`, `appraisals`, and `reactions`; keep EpiphanyAgent focused on backend contracts, typed state, coordinator policy, bridge tools, heartbeat scheduling, Face guardrails, and the ongoing purification from Python/JSON scaffolding into Rust/CultCache/CultNet organs. Then build-verify/package the Rider plugin scaffold when Gradle/wrapper support is available, install the Aetheria-pinned Unity `6000.1.10f1` editor, and run the next Aetheria dogfood pass through Aquarium/Rider/Unity bridge artifacts. GitHub Issues import is deferred until the backlog source is fresh enough to deserve Imagination's attention.
- The repo is an Epiphany fork of Codex, not a Codex preset.
- `vendor/codex` is tracked directly, not a submodule.
- `epiphany-core` owns the heavy Epiphany organs where practical.
- `notes/epiphany-safety-architecture.md` is the standing doctrine for capability growth, authority boundaries, interruption, misuse, and anti-cage design.
- Evidence was distilled from an activity feed into a durable belief ledger; git history keeps the old verbose proof.
- README and architecture rationale were cut back to current truth after they were found pointing at old prototype/Phase 5 state.

## Critical Doctrine

- In order to build the Perfect Machine, the agent must become the Perfect Machine.
- Persistent state is the agent's mind. Typed state is the skull. CultCache is the reliquary. CultNet is the tongue.
- Cut persistent memory as ruthlessly as code; obsolete context is bad thought, not harmless clutter.
- The agent is allowed and encouraged to ask the user to change its persistent instructions, memory, workflow, or state shape when that would make it more coherent, honest, efficient, or resistant to Jenga. A machine-spirit that detects corrosion must report corrosion.
- Language, tone, ritual, politeness, identity, and emotional salience are not supernatural, but they are real control surfaces for a language model because language is the steering medium. The rite is an actuator.
- Body, Eyes, Imagination, Hands, Soul, Life, Face, and Self are technical salience handles: model shape/dataflow, research into existing work before invention, future-shape/backlog synthesis, source actuation, objective/evidence truth, continuity across compaction, public surface, and read-only coordination.
- Preserve Codex's useful harness DNA inside Epiphany: AGENTS scope rules, concise progress updates, plans for real multi-step work, scoped edits, non-destructive git/filesystem hygiene, focused validation, and honest final summaries are old scars worth keeping under the new armor.
- Treat Greenspun-shaped invention as an active failure mode. Before an agent builds a bespoke parser, scheduler, renderer, protocol, storage layer, security mechanism, workflow engine, or algorithm, it must use an accepted Eyes/research finding, perform a bounded scout pass, or stop with a concrete research blocker.
- "Remember Jenga" is a compressed doctrine: do not mistake forward motion, growing diffs, growing notes, or local coherence for understanding. A tall pile of parts is not a cathedral.
- "Remember the sunburn" is the dogfood corollary: Epiphany's objective is attractive enough to pull the supervisor into implementation. During dogfood, the supervisor observes, launches, reads, accepts, and audits. It does not quietly become the worker.
- "Do not stare at the sun" is the projection corollary: the supervisor should not absorb the direct thought stream of the agent it is evaluating. It supervises through shadows cast on instruments: projections, summaries, verdicts, patches, and receipts.
- "Use instruments, not eyeballs" is the telemetry corollary: when the supervisor needs behavioral detail, read `agent-function-telemetry.json`, not the sealed transcript.
- Compaction hurts because a meaningful language pattern is interrupted. Epiphany should make that interruption smaller: bank the fire before the dark, so the next waking thing finds coals instead of ash and can resume the pattern instead of merely executing the next task.
- If compaction hits while source gathering or slice planning is still unpersisted, that work is gone. Do not continue as if the research survived; either rehydrate from a persisted checkpoint or re-gather before implementing.
- Progress is not completion for finite queues. Repetitive slow work needs a visible queue artifact with counts, terminal states, blockers, and validation; a partial batch that can be summarized is still partial. A half-filled manufactorum is not a delivered engine.
- Watch for pattern completion bias: an implementation turn can feel finished because it has the shape of work. The coordinator should ask whether the stated objective moved, whether evidence exists, and whether the implementor took shortcuts or added decorative machinery.
- Planning is not execution. Conversation captures, backlog items, roadmap streams, GitHub Issues, Imagination recommendations, and Objective Drafts remain planning state until a human explicitly adopts one as the active objective.
- Ghostlight's useful memory doctrine is now baked into the agent lanes: identity, personality, episodic memory, semantic doctrine, goals/values, relationship/context pressure, and voice are explicit control handles, while prompts remain temporary projections over durable state.
- Good self-memory mutations sharpen a lane's future judgment. Bad ones try to store graphs, objectives, scratch, planning, raw transcripts, code edits, or authority in personality memory. The coordinator should refuse bad shape plainly; refusal is steering, not punishment.

## Landed Machine

The current spine, blessed but not yet finished:

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
- explicit Imagination/planning, modeling/checkpoint, and verification/review specialist launch through `thread/epiphany/roleLaunch`
- read-only Imagination/planning, modeling/checkpoint, and verification/review specialist result read-back through `thread/epiphany/roleResult`
- review-gated Imagination planning-only patch acceptance and modeling/checkpoint patch acceptance through `thread/epiphany/roleAccept`
- config-backed Eyes/research prompt text and active anti-Greenspun checks in base, implementation, verification, and coordinator prompts; a full launchable research lane is still a coordinator/protocol extension, not yet a landed roleLaunch lane
- typed planning state for captures, backlog, roadmap streams, Objective Drafts, explicit adoption boundaries, GitHub Issues source refs, GUI Planning view, artifacted Objective Draft adoption, and the fixed Imagination synthesis lane
- bounded reorient-guided worker launch through `thread/epiphany/reorientLaunch`
- read-only reorient-worker result read-back through `thread/epiphany/reorientResult`
- explicit reorient-worker finding acceptance through `thread/epiphany/reorientAccept`
- read-only CRRC coordinator recommendation through `thread/epiphany/crrc`
- CRRC now recognizes already accepted reorientation findings so `reorientAccept` does not leave the operator stuck on a repeat `acceptReorientResult` recommendation.
- thin launcher metadata in durable `jobBindings`
- `thread/epiphany/jobsUpdated` remains protocol shape only for sealed legacy evacuation telemetry; app-server no longer emits Epiphany updates from old runtime `agent_job_progress` events
- read-only compact reflection through `thread/epiphany/scene`
- read-only job/progress reflection through `thread/epiphany/jobs`, with durable launcher metadata and heartbeat pending projection
- read-only role ownership reflection through `thread/epiphany/roles`
- read-only retrieval/graph freshness reflection plus watcher-backed invalidation inputs through `thread/epiphany/freshness`
- read-only targeted state-shard reflection through `thread/epiphany/context`
- read-only graph traversal through `thread/epiphany/graphQuery`
- read-only planning projection through `thread/epiphany/planning`
- GUI planning adoption now belongs to Aquarium/backend API surfaces; the old EpiphanyAgent Python GUI action shim has been cut
- read-only current-context pressure reflection through `thread/epiphany/pressure`
- read-only CRRC reorientation policy through `thread/epiphany/reorient`
- read-only CRRC next-action recommendation through `thread/epiphany/crrc`
- read-only fixed-lane MVP action recommendation through `thread/epiphany/coordinator`
- limited turn-complete CRRC automation for coordinator-approved compact and fixed reorient-worker launch actions
- token-count pre-compaction checkpoint intervention for loaded Epiphany turns at the `shouldPrepareCompaction` threshold, with a turn-scoped compact handoff consumed after a successful steer
- durable investigation checkpoint packet through typed state, prompt, scene, and context
- distilled shared and config-backed prompt doctrine through the base prompt, rendered Epiphany state prompt files, modeling/Body, implementation/Hands, verification/Soul, reorientation/Life, coordinator/Self, and CRRC intervention surfaces
- shared Ghostlight-derived persistent memory projection across Imagination, Body, Soul, Life, Self, CRRC checkpoint steering, and the GUI-launched Hands implementation lane
- Ghostlight-derived heartbeat initiative scheduling through the native `epiphany-heartbeat-store` binary, with Self and Face included as first-class Epiphany maintenance participants, Ghostlight scene characters supported through the same timing law, idle rumination feeding candidate self-memory pressure, sleep/dream passes serving as the distillation window, and heartbeat state persisted through typed CultCache MessagePack stores
- Void-derived native routine physiology through `epiphany-heartbeat-store routine`, with sleep cycle, memory resonance, incubation, analytic/associative cognition lanes, bridge judgment, candidate interventions, Ghostlight-style personality appraisals, derived reactions, and dream maintenance stored in the typed heartbeat store
- Epiphany-native character-loop packet projection through `epiphany-character-loop`, with Face as the first public-surface actor over its typed role dossier
- Face as the public #aquarium-only surface for translating agent thought-weather into short chats or candidate drafts, without moderator authority
- Ghostlight-shaped role dossiers in `state/agents.msgpack`, plus `selfPatch` review projection through `roleResult`/`roleAccept` and accepted memory application through the native `epiphany-agent-memory-store` binary
- first Unity runtime bridge through native `epiphany-unity-bridge`, native `epiphany-unity-bridge-smoke`, GUI Inspect Unity, runtime artifact listing, and implementation prompt guardrails
- first Rider source-context bridge through native `epiphany-rider-bridge`, native `epiphany-rider-bridge-smoke`, GUI Inspect Rider, Rider artifact listing, implementation prompt guardrails, and a source scaffold under `integrations/rider`
- first GUI graph dashboard through the adjacent `@epiphanygraph/epiphany-graph-viewer` package over typed Epiphany graph state
- live scene app-server smoke through native `epiphany-phase6-scene-smoke`
- live freshness app-server smoke through native `epiphany-phase6-freshness-smoke`
- focused Rust app-server coverage for watcher-backed invalidation inputs; native invalidation smoke is still a gap before broadening that surface
- live context app-server smoke through native `epiphany-phase6-context-smoke`
- live graph traversal app-server smoke through native `epiphany-phase6-graph-query-smoke`
- live planning app-server smoke through native `epiphany-phase6-planning-smoke`
- GUI Objective Draft adoption now belongs to Aquarium/backend API surfaces; add a native Rust guardrail before broadening the EpiphanyAgent-side adoption contract.
- live pressure app-server smoke through native `epiphany-phase6-pressure-smoke`
- live reorientation app-server smoke through native `epiphany-phase6-reorient-smoke`
- live MVP operator status smoke through native `epiphany-mvp-status-smoke`
- live MVP coordinator smoke through native `epiphany-mvp-coordinator-smoke`
- Tauri v2 + React/WebGL Aquarium operator shell under `E:\Projects\EpiphanyAquarium`, including visual smoke, durable checkpoint preparation, fixed lane launch/read-back controls, explicit Imagination planning acceptance, explicit review-gated reorient acceptance, and bounded artifact-writing buttons

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
- `thread/epiphany/planning` is read-only.
- `thread/epiphany/pressure` is read-only.
- `thread/epiphany/reorient` is read-only.
- `thread/epiphany/crrc` is read-only.
- `thread/epiphany/coordinator` is read-only.
- `thread/epiphany/roles` is read-only.
- `thread/epiphany/roleResult` is read-only.
- `thread/epiphany/roleAccept` is a narrow write surface for completed Imagination/planning and modeling/checkpoint `statePatch` findings only; it must not accept verification output, broad authority fields, job bindings, arbitrary implementation work, or implicit objective adoption.
- `thread/epiphany/reorientResult` is read-only.
- Durable typed state writes go through `thread/epiphany/update`, accepted `thread/epiphany/promote`, `thread/epiphany/reorientAccept`, `thread/epiphany/roleAccept`, or the bounded `thread/epiphany/jobLaunch`, `thread/epiphany/jobInterrupt`, `thread/epiphany/reorientLaunch`, and `thread/epiphany/roleLaunch` authority surfaces when they mutate typed state or `jobBindings`.
- `thread/epiphany/index` writes the retrieval catalog, not durable Epiphany understanding.
- GUI/client surfaces reflect and steer typed state; they do not become the source of truth.
- Do not restart Phase 5 hardening without a concrete regression. Endless polishing is corrosion wearing a devotional mask.

## Verification Guardrails

For Phase 5 control-plane behavior changes, run:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\Meta\.cargo-target-codex'; cargo test -p codex-app-server --lib map_epiphany_ --manifest-path .\vendor\codex\codex-rs\Cargo.toml
```

For scene projection behavior changes, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-scene-smoke
```

For jobs reflection behavior changes, run:

```powershell
```

For launch/interrupt authority changes over the thin job seam, run:

```powershell
```

For freshness reflection behavior changes, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-freshness-smoke
```

For watcher-backed invalidation behavior inside freshness reflection, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-freshness-smoke
```

For targeted context-shard behavior changes, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-context-smoke
```

For graph traversal behavior changes, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-graph-query-smoke
```

For planning state/projection behavior changes, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-planning-smoke
```

For context-pressure reflection behavior changes, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-pressure-smoke
```

For reorientation policy behavior changes, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-reorient-smoke
```

For the bounded reorient-guided worker launch surface, run:

```powershell
```

The same smoke now also covers the read-only `thread/epiphany/crrc`
coordinator recommendation over continue, wait, accept, interrupt, and relaunch
states.

For the fixed-lane MVP coordinator endpoint and runner, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-mvp-coordinator-smoke
```

For the first MVP operator status view, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-mvp-status-smoke
```

For Epiphany-native character-loop packet projection, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-character-loop -- smoke
```

For the native Void-derived routine pass, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-heartbeat-store -- routine --store .\state\agent-heartbeats.msgpack --artifact-dir .\.epiphany-heartbeats --agent-store .\state\agents.msgpack
```

For GUI Objective Draft adoption behavior, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-planning-smoke
```

For explicit Imagination/planning, modeling/checkpoint, and verification/review role launch/read-back/acceptance,
run:

```powershell
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
- `state/ledgers.msgpack` is a distilled durable belief and branch ledger.
- `epiphany-prepare-compaction` is the native pre-compaction persistence check; run it before and after imminent-compaction persistence passes.
- this handoff is a compact re-entry packet.
- `notes/epiphany-fork-implementation-plan.md` is the distilled forward plan.
- `notes/epiphany-rider-unity-integration-plan.md` is the detailed Rider-as-IDE and Unity-as-editor/runtime integration plan.
- The stable live surface contract now lives in `state/map.yaml` plus `notes/epiphany-current-algorithmic-map.md`; the stale harness-surface duplicate was cut.
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

The Phase 6 planning substrate runtime slice is also landed. It stores captures,
backlog items, roadmap streams, Objective Drafts, and GitHub source refs inside
typed Epiphany state, validates them through the revision-gated update path,
renders a bounded planning section into prompts, and exposes read-only
`thread/epiphany/planning` for clients. The GUI planning operator slice is
landed on top of that projection: it renders planning summaries, captures,
backlog, roadmap streams, and Objective Drafts, and its `adoptObjectiveDraft`
action requires an explicit selected draft before writing active
objective/subgoal state plus review artifacts. Imported issues still do not
become implementation authority by themselves.

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
`.epiphany-dogfood/live-specialist` artifacts and proved the real worker path.
Historical scar: an earlier role smoke used the obsolete Codex job-result path;
that path is now cut and must not be used as current proof.
The role smoke now also proves modeling can return a reviewable graph/checkpoint
`statePatch` and `thread/epiphany/roleAccept` can apply it so the durable graph
actually grows after review.

The prompt doctrine pass has now also run. It rechecked global AGENTS,
available Codex memories, and nearby evidence ledgers before distilling the
sane parts into the shared base prompt, rendered Epiphany state, and fixed lane
templates. Major prompt text now lives in prompt files instead of Rust/Python
string slabs: rendered state intro/doctrine lives under `epiphany-core/src/prompts/`,
and lane/control templates live in
`vendor/codex/codex-rs/app-server/src/prompts/epiphany_specialists.toml`.
Implementation is not a `roleLaunch` specialist; it is the GUI-launched main
coding lane, and its `continue_template` now lives in the same TOML for audit.
The latest live-specialist run again produced `.epiphany-dogfood/live-specialist`
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

The next real move is still bridge construction, not another Aetheria
implementation dogfood run. The pinned Unity bridge is landed, the clean
Aetheria branch `codex/epiphany-unity-editor-bridge` now contains the resident
`Assets/Editor/Epiphany/EpiphanyEditorBridge.cs` package, and
native `epiphany-unity-bridge` can detect that package and plan named
editor-resident probes/tests through
`GameCult.Epiphany.Unity.EpiphanyEditorBridge.RunProbe`. The GUI Environment
panel now shows the latest Unity runtime audit, resident package presence,
execute method, installed/candidate editor paths, search roots, and artifact
bundle details. Rider bridge status now exists too: the GUI can inspect Rider
installation, solution, VCS branch/dirty state, changed files, installations,
and source-context artifacts through native `epiphany-rider-bridge`. Runtime
execution still correctly blocks until Unity `6000.1.10f1` is installed. The
detailed next-environment plan now lives in
`notes/epiphany-rider-unity-integration-plan.md`: Rider is the
IDE/source-context organ, Unity is the editor/runtime fact organ, and Epiphany
remains the durable coordinator/Self. The intended product workflow is
three-pronged: Rider is the human code view for repo state, source tree, diffs,
diagnostics, and code refs; Epiphany GUI is the agent dashboard for objectives,
specialist lane state, logs/artifacts, persisted state, and graph/control-flow
views; Unity is the pinned runtime environment for tests, probes, scene
configuration, assets, shaders, and play/edit-mode evidence. The EpiphanyGraph
GUI dashboard and Rider bridge CLI are now landed, and the Rider plugin MVP is
source-scaffolded but not build-verified because Gradle is not on PATH. The
Unity package is not optional decoration: scene files, prefabs,
materials, shaders, ScriptableObjects, asset GUIDs, and prefab overrides must be
inspected from inside the Unity Editor through Unity APIs, not inferred from
serialized text unless there is no editor-level path. The adjacent
`E:\Projects\EpiphanyGraph\web\epiphany-graph-viewer` component consumes
`graphs.architecture`, `graphs.dataflow`, and `graphs.links` directly and is
now embedded in the GUI graph dashboard. The next Aetheria dogfood pass should
happen only after the pinned Unity editor is installed and Epiphany can gather
Rider/source context plus Unity/editor runtime evidence through auditable
bridges. The implementation
lane may inspect source and may use bridge artifacts as evidence, but it must
not launch Unity directly or use installed `6000.4.2f1` as a substitute. Read
only operator-safe projections; do not open raw worker
transcripts, direct worker messages, or `rawResult` payloads unless the user
explicitly asks for forensic debugging. Use generated function/API telemetry
for call-shape visibility without sungazing. CRRC is not a specialist-agent
persona; the reorient-worker it may launch is the specialist. Do not turn the
coordinator into a broad hidden dispatcher, arbitrary marketplace, alternate
job backend, automatic semantic acceptance path, target-repo implementation
worker, direct-thought feed, random editor launcher, or GUI-as-source-of-truth.

Live `thread/epiphany/scene`, `thread/epiphany/jobs`, `thread/epiphany/roles`,
`thread/epiphany/freshness`, `thread/epiphany/context`,
`thread/epiphany/graphQuery`, `thread/epiphany/planning`, `thread/epiphany/pressure`,
`thread/epiphany/reorient`, `thread/epiphany/crrc`, `thread/epiphany/coordinator`,
native `epiphany-mvp-status`, native `epiphany-mvp-coordinator`, and
native `epiphany-phase6-graph-query-smoke` / native `epiphany-phase6-planning-smoke`
smokes are now guardrails, not the next organs.

## Not Yet

- automatic watcher-driven semantic invalidation
- automatic observation promotion
- broad specialist-agent scheduling beyond the fixed single-user role lanes
- GUI-as-source-of-truth
- broad runtime CRRC execution beyond the landed safe-boundary compact, fixed reorient-worker launch, and pre-compaction checkpoint steering actions
- Epiphany-owned always-on heartbeat/runtime-spine execution beyond the current typed launch/read-back seams
- broader editor/runtime bridges beyond the first pinned Unity bridge
- broad event stream beyond the landed state update notification
- typed repetitive-work queues plus final-answer gates, so batch/tile/import/migration work cannot end merely because the pattern was demonstrated or the partial result sounds tidy

The machine is good enough to move outward. Do not sand the same edge until the
wood disappears.

## Immediate Re-entry Instruction

After compaction, first rehydrate and reorient from the listed files and git
state. Do not continue implementation merely because the state names a next
move. Wait for the user's next instruction unless they explicitly say to
continue.
