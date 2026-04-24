# EpiphanyAgent Instructions

## Project Purpose

This repo explores Epiphany as an opinionated fork of Codex: external typed state, explicit mental maps, bounded scratch work, verifier evidence, and aggressive anti-churn discipline wired into the harness instead of taped onto the chat transcript.

The motivating failure mode is that an agent can make many plausible local edits after global coherence has already failed. The project goal is to force the model to model the thing it is changing, then test whether explicit map/scratch/evidence channels reduce drift.

## Canonical State

- Treat `state/map.yaml` as the canonical project map.
- Treat `state/scratch.md` as disposable working memory for one bounded subgoal.
- Treat `state/evidence.jsonl` as the durable log of what was learned, verified, rejected, or accepted.
- Treat `notes/epiphany-fork-implementation-plan.md` as the current implementation plan for the Epiphany fork architecture.
- Update `state/map.yaml` when project understanding changes.
- Append evidence after meaningful research, implementation, verification, or rejected paths.

## Current Status

The old preset-backed TUI experiment is no longer the active path.

What is already landed across vendored Codex and `epiphany-core`:

- Phase 1 durable Epiphany thread state
- Phase 2 prompt integration
- a minimal Phase 3 typed app-server/client read surface via `Thread.epiphanyState`
- Phase 4 hybrid retrieval/indexing with explicit Qdrant-backed indexing and BM25 fallback

Current next phase:

- live-smoke the explicit indexing path against the local Qdrant/Ollama services

## Important Paths

- Project root: `E:\Projects\EpiphanyAgent`
- Vendored Codex repo: `E:\Projects\EpiphanyAgent\vendor\codex`
- Fork implementation plan: `E:\Projects\EpiphanyAgent\notes\epiphany-fork-implementation-plan.md`
- Handoff summary: `E:\Projects\EpiphanyAgent\notes\fresh-workspace-handoff.md`
- Epiphany algorithmic map: `E:\Projects\EpiphanyAgent\notes\epiphany-current-algorithmic-map.md`
- State CLI: `E:\Projects\EpiphanyAgent\tools\epiphany_state.py`

## Useful Commands

Use the bundled Python runtime if `python` is not on PATH:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_state.py' status
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_state.py' add-evidence --type research --status ok --note '...'
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
3. persist map/evidence/handoff updates before forced compaction hits

Do not wait for the blackout and then act surprised.

## Operating Discipline

- Before substantial edits, restate the current mechanism and intended change.
- Prefer one clear hypothesis per iteration.
- Verify with checks that reflect the real goal, not just proxy success.
- Revert or discard changes that do not clearly improve the target.
- If the diff grows while understanding shrinks, stop implementation and switch to diagnosis.
- Keep maps and prose together; do not replace useful maps with prose-only explanations.
- Before adding natural-language explanations or metaphors to an algorithmic map, first read the relevant source paths and anchor the explanation to concrete code references. Metaphor is compression after source grounding, not a substitute for it.
- Before handoff, compaction, or phase boundaries, sync `state/map.yaml`, append `state/evidence.jsonl`, refresh `notes/fresh-workspace-handoff.md`, and make the next action explicit.
