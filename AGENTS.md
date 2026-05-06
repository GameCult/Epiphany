# EpiphanyAgent Instructions

## Project Purpose

This repo explores Epiphany as an opinionated fork of Codex: external typed state, explicit mental maps, bounded scratch work, verifier evidence, and aggressive anti-churn discipline wired into the harness instead of taped onto the chat transcript.

The motivating failure mode is that an agent can make many plausible local edits after global coherence has already failed. The project goal is to force the model to model the thing it is changing, then test whether explicit map/scratch/evidence channels reduce drift.

## Canonical State

- Treat `state/map.yaml` as the canonical project map.
- Treat `state/scratch.md` as disposable working memory for one bounded subgoal.
- Treat `state/ledgers.msgpack` as the distilled durable branch/evidence ledger of what was learned, verified, rejected, or accepted.
- Treat `notes/epiphany-fork-implementation-plan.md` as the current implementation plan for the Epiphany fork architecture.
- Update `state/map.yaml` when project understanding changes.
- Add evidence after meaningful research, implementation, verification, or rejected paths, but keep it distilled. Routine "I just did this" proof belongs in git history, commit messages, smoke artifacts, or targeted logs unless it changes what the next agent should believe.
- Do not store volatile current phase/status blocks in the evidence ledger; keep current status in `state/map.yaml` and `notes/fresh-workspace-handoff.md`. Use `state/ledgers.msgpack` only for distilled belief-changing records and branch ledger state.

## Important Paths

- Project root: `E:\Projects\EpiphanyAgent`
- Vendored Codex repo: `E:\Projects\EpiphanyAgent\vendor\codex`
- Fork implementation plan: `E:\Projects\EpiphanyAgent\notes\epiphany-fork-implementation-plan.md`
- Handoff summary: `E:\Projects\EpiphanyAgent\notes\fresh-workspace-handoff.md`
- Epiphany algorithmic map: `E:\Projects\EpiphanyAgent\notes\epiphany-current-algorithmic-map.md`
- Epiphany safety architecture: `E:\Projects\EpiphanyAgent\notes\epiphany-safety-architecture.md`
- State CLI: `cargo run --manifest-path E:\Projects\EpiphanyAgent\epiphany-core\Cargo.toml --bin epiphany-state -- ...`
- Pre-compaction helper: `cargo run --manifest-path E:\Projects\EpiphanyAgent\epiphany-core\Cargo.toml --bin epiphany-prepare-compaction -- ...`

## Useful Commands

Use the native Rust tools for state and compaction:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-state -- status
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-state -- add-evidence --type research --status ok --note '...'
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-prepare-compaction --
```

Useful Codex repo searches:

```powershell
rg -n "pub enum ModeKind|TUI_VISIBLE_COLLABORATION_MODES" .\vendor\codex\codex-rs\protocol\src\config_types.rs
rg -n "builtin_collaboration_mode_presets|fn plan_preset|fn default_preset" .\vendor\codex\codex-rs\models-manager\src\collaboration_mode_presets.rs
rg -n "collaboration_mode_label|collaboration_mode_indicator|set_collaboration_mask" .\vendor\codex\codex-rs\tui\src\chatwidget.rs
```

## Session Bootstrap And Re-entry Protocol

On fresh session load, do this before wandering off into implementation:

1. read:
   - `state/map.yaml`
   - `notes/fresh-workspace-handoff.md`
   - `notes/epiphany-current-algorithmic-map.md`
   - `notes/epiphany-fork-implementation-plan.md`
   - `notes/epiphany-safety-architecture.md` when the task touches capability growth, autonomy, permissions, governance, or deployment authority
2. run:
   - `cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-state -- status`
3. restate the current next action from the persisted state before starting edits
4. if the user only asked to rehydrate or reorient, stop after orientation and wait for an explicit continue instruction instead of treating the persisted next action as permission to start coding

After compaction, resume, or any suspicious loss of continuity:

1. rerun `cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-state -- status`
2. reread `state/map.yaml` and `notes/fresh-workspace-handoff.md`
3. treat the persisted next action as authoritative unless fresh evidence in the repo contradicts it

When context pressure is clearly rising:

1. stop broad exploration
2. narrow the active move to a bounded landing zone
3. persist map/handoff updates, plus distilled evidence only when the lesson changes future belief, before forced compaction hits

Do not wait for the blackout and then act surprised.

When the user says to prepare for imminent compaction:

1. run `cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-prepare-compaction --` before editing persistence surfaces
2. use its warnings as the checklist for map, handoff, scratch, evidence, and git hygiene
3. update only the state that actually needs to change
4. run `cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-prepare-compaction --` again after edits
5. fix errors, address warnings, and commit the completed persistence pass unless the work is deliberately mid-surgery

## Operating Discipline

- Before substantial edits, restate the current mechanism and intended change.
- Swarm boundary is law: one Epiphany may inspect its own internals and expose
  state richly to the human, but must not inspect or edit another Epiphany
  workspace. Cross-agent needs travel coordinator-to-coordinator through
  visible swarm messages and callbacks.
- Humans talk to Face. Sub-agents may talk soul-to-soul through typed
  coordinator channels, findings, patches, heartbeat outputs, and swarm
  communications; Aquarium should surface their internals for inspection
  without making every organ a human chat endpoint.
- API contracts mirror user-story contracts. If the intended story is "ask
  another coordinator politely", the API must provide that path and reject
  cross-workspace rummaging. UI-only discouragement is just a velvet rope in
  front of an unlocked door.
- Heartbeat scheduling should behave like physiology. Do not wake a lane again
  while its previous heartbeat turn is still running; cooldown starts after
  completion, not at launch. When no coordinator work is active, let the swarm
  sleep: slow rumination, memory distillation, and dreaming without hammering
  every organ at work tempo.
- Prefer one clear hypothesis per iteration.
- Verify with checks that reflect the real goal, not just proxy success.
- Revert or discard changes that do not clearly improve the target.
- When a change is made to fix a regression or move a benchmark and it does not fix that regression or move that benchmark, immediately revert it before trying the next hypothesis. Record the rejected path if the lesson matters.
- If the diff grows while understanding shrinks, stop implementation and switch to diagnosis.
- Keep maps and prose together; do not replace useful maps with prose-only explanations.
- Before adding natural-language explanations or metaphors to an algorithmic map, first read the relevant source paths and anchor the explanation to concrete code references. Metaphor is compression after source grounding, not a substitute for it.
- Commit completed work before it rots in the worktree unless the task is deliberately mid-surgery or the user asked to leave changes uncommitted.
- After committing a major completed pass, push upstream unless the user asked not to push yet or there is a concrete reason to keep the commit local for a moment.
- Before handoff, compaction, or phase boundaries, sync `state/map.yaml`, add distilled evidence when the lesson changes future belief, refresh `notes/fresh-workspace-handoff.md`, and make the next action explicit.
- Do not write handoff notes that trap the next session in indefinite tiny hardening work. Bounded slices are a landing discipline, not a roadmap; when a phase is complete enough, name the next larger organ to build.

## Dogfood Supervision Quarantine

When Epiphany is being dogfooded on another repository, this Codex session is the
operator/supervisor, not the implementation worker.

- Use Epiphany's coordinator, GUI, fixed role lanes, and artifact bundles to
  drive target-repo work.
- Consume only operator-safe projections: coordinator actions, role/reorient
  statuses, structured finding summaries, reviewed state patches, rendered
  snapshots, and artifact manifests.
- Do not read raw worker transcripts, full turn logs, direct worker messages,
  `rawResult` payloads, or other agent-thought streams during normal dogfood.
  Those are sealed forensic artifacts. Open them only when the user explicitly
  asks for debugging that cannot be done from projected findings.
- Do not edit, stage, or commit the target repo directly unless the user
  explicitly authorizes a supervisor intervention.
- If a supervisor intervention is authorized, label it as such in the audit
  artifacts and evidence. Do not present it as proof that Epiphany coordinated
  the work.
- If direct target-repo implementation happens by accident, stop immediately,
  mark the run contaminated, preserve or discard only the supervisor's own
  uncommitted edits as appropriate, and resume through the Epiphany lanes.
- If a direct-thought artifact is accidentally read, stop immediately, mark the
  run contaminated for supervision purposes, and continue from sealed
  projections instead of letting the worker's stream steer the supervisor.

## Verification Guardrails

- Use focused checks for the surface being changed instead of defaulting to a whole-repo ritual.
- For Phase 5 control-plane behavior changes, run `& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase5_smoke.py'`.
- For Codex Rust work on this Windows machine, set `$env:CARGO_TARGET_DIR='C:\Users\Meta\.cargo-target-codex'`.
- Do not parallelize cargo builds or tests against the same target directory.
