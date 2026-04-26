# EpiphanyAgent Instructions

## Project Purpose

This repo explores Epiphany as an opinionated fork of Codex: external typed state, explicit mental maps, bounded scratch work, verifier evidence, and aggressive anti-churn discipline wired into the harness instead of taped onto the chat transcript.

The motivating failure mode is that an agent can make many plausible local edits after global coherence has already failed. The project goal is to force the model to model the thing it is changing, then test whether explicit map/scratch/evidence channels reduce drift.

## Canonical State

- Treat `state/map.yaml` as the canonical project map.
- Treat `state/scratch.md` as disposable working memory for one bounded subgoal.
- Treat `state/evidence.jsonl` as the distilled durable ledger of what was learned, verified, rejected, or accepted.
- Treat `notes/epiphany-fork-implementation-plan.md` as the current implementation plan for the Epiphany fork architecture.
- Update `state/map.yaml` when project understanding changes.
- Add evidence after meaningful research, implementation, verification, or rejected paths, but keep it distilled. Routine "I just did this" proof belongs in git history, commit messages, smoke artifacts, or targeted logs unless it changes what the next agent should believe.
- Do not store volatile current phase/status blocks in this file; keep current status in `state/map.yaml` and `notes/fresh-workspace-handoff.md`. Use `state/evidence.jsonl` only for distilled belief-changing records.

## Important Paths

- Project root: `E:\Projects\EpiphanyAgent`
- Vendored Codex repo: `E:\Projects\EpiphanyAgent\vendor\codex`
- Fork implementation plan: `E:\Projects\EpiphanyAgent\notes\epiphany-fork-implementation-plan.md`
- Handoff summary: `E:\Projects\EpiphanyAgent\notes\fresh-workspace-handoff.md`
- Epiphany algorithmic map: `E:\Projects\EpiphanyAgent\notes\epiphany-current-algorithmic-map.md`
- Epiphany safety architecture: `E:\Projects\EpiphanyAgent\notes\epiphany-safety-architecture.md`
- State CLI: `E:\Projects\EpiphanyAgent\tools\epiphany_state.py`
- Pre-compaction helper: `E:\Projects\EpiphanyAgent\tools\epiphany_prepare_compaction.py`

## Useful Commands

Use the bundled Python runtime if `python` is not on PATH:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_state.py' status
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_state.py' add-evidence --type research --status ok --note '...'
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_prepare_compaction.py'
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
   - `& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_state.py' status`
3. restate the current next action from the persisted state before starting edits

After compaction, resume, or any suspicious loss of continuity:

1. rerun `epiphany_state.py status`
2. reread `state/map.yaml` and `notes/fresh-workspace-handoff.md`
3. treat the persisted next action as authoritative unless fresh evidence in the repo contradicts it

When context pressure is clearly rising:

1. stop broad exploration
2. narrow the active move to a bounded landing zone
3. persist map/handoff updates, plus distilled evidence only when the lesson changes future belief, before forced compaction hits

Do not wait for the blackout and then act surprised.

When the user says to prepare for imminent compaction:

1. run `tools/epiphany_prepare_compaction.py` before editing persistence surfaces
2. use its warnings as the checklist for map, handoff, scratch, evidence, and git hygiene
3. update only the state that actually needs to change
4. run `tools/epiphany_prepare_compaction.py` again after edits
5. fix errors, address warnings, and commit the completed persistence pass unless the work is deliberately mid-surgery

## Operating Discipline

- Before substantial edits, restate the current mechanism and intended change.
- Prefer one clear hypothesis per iteration.
- Verify with checks that reflect the real goal, not just proxy success.
- Revert or discard changes that do not clearly improve the target.
- When a change is made to fix a regression or move a benchmark and it does not fix that regression or move that benchmark, immediately revert it before trying the next hypothesis. Record the rejected path if the lesson matters.
- If the diff grows while understanding shrinks, stop implementation and switch to diagnosis.
- Keep maps and prose together; do not replace useful maps with prose-only explanations.
- Before adding natural-language explanations or metaphors to an algorithmic map, first read the relevant source paths and anchor the explanation to concrete code references. Metaphor is compression after source grounding, not a substitute for it.
- Before handoff, compaction, or phase boundaries, sync `state/map.yaml`, add distilled evidence when the lesson changes future belief, refresh `notes/fresh-workspace-handoff.md`, and make the next action explicit.
- Do not write handoff notes that trap the next session in indefinite tiny hardening work. Bounded slices are a landing discipline, not a roadmap; when a phase is complete enough, name the next larger organ to build.
