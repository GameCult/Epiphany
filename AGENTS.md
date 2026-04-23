# EpiphanyAgent Instructions

## Project Purpose

This repo explores an Epiphany mode for AI coding agents: external typed state, explicit mental maps, bounded scratch work, verifier evidence, and aggressive anti-churn discipline.

The motivating failure mode is that an agent can make many plausible local edits after global coherence has already failed. The project goal is to test whether explicit map/scratch/evidence channels reduce that drift.

## Canonical State

- Treat `state/map.yaml` as the canonical project map.
- Treat `state/scratch.md` as disposable working memory for one bounded subgoal.
- Treat `state/evidence.jsonl` as the durable log of what was learned, verified, rejected, or accepted.
- Treat `notes/codex-epiphany-mode-plan.md` as the current patch plan for adding Epiphany mode to the vendored Codex repo.
- Update `state/map.yaml` when project understanding changes.
- Append evidence after meaningful research, implementation, verification, or rejected paths.

## Current Recommendation

Do not start with full protocol enum surgery.

Start by implementing a lightweight preset-backed Epiphany mode in the vendored Codex repo:

- add an `epiphany.md` collaboration-mode template
- add an `Epiphany` collaboration preset backed by `ModeKind::Default`
- patch the TUI to display the active preset name so the mode is visible to users
- only add `ModeKind::Epiphany` later if the preset-backed experiment proves useful

## Important Paths

- Project root: `E:\Projects\EpiphanyAgent`
- Vendored Codex repo: `E:\Projects\EpiphanyAgent\vendor\codex`
- Patch plan: `E:\Projects\EpiphanyAgent\notes\codex-epiphany-mode-plan.md`
- Handoff summary: `E:\Projects\EpiphanyAgent\notes\fresh-workspace-handoff.md`
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

## Operating Discipline

- Before substantial edits, restate the current mechanism and intended change.
- Prefer one clear hypothesis per iteration.
- Verify with checks that reflect the real goal, not just proxy success.
- Revert or discard changes that do not clearly improve the target.
- If the diff grows while understanding shrinks, stop implementation and switch to diagnosis.
- Keep maps and prose together; do not replace useful maps with prose-only explanations.
