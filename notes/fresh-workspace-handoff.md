# Fresh Workspace Handoff

## What This Repo Is

`EpiphanyAgent` is a prototype workspace for testing whether AI coding agents behave better when their mental model is externalized into typed state:

- `map`: canonical project/system understanding
- `scratch`: temporary local reasoning
- `evidence`: verification, acceptance, and rejection trail

The project grew from a concrete failure mode: an agent can keep making plausible local changes while the global design quietly collapses. The intended fix is not "more prompting" in the abstract, but explicit state channels and a controller discipline around when to model, think, verify, edit, revert, and speak.

## Current State

The workspace has:

- a root git repo
- a vendored clone of `openai/codex` at `vendor/codex`
- a small state CLI at `tools/epiphany_state.py`
- typed state files under `state/`
- controller action rules in `protocol/controller-actions.md`
- an eval sketch in `eval-plan.md`
- architecture rationale in `notes/architecture-rationale.md`
- a Codex patch plan in `notes/codex-epiphany-mode-plan.md`

The latest canonical status is in `state/map.yaml`.

## Key Conclusion From Codex Repo Research

The best insertion point for Epiphany mode is Codex's existing collaboration-mode preset system.

Relevant files:

- `vendor/codex/codex-rs/protocol/src/config_types.rs`
- `vendor/codex/codex-rs/models-manager/src/collaboration_mode_presets.rs`
- `vendor/codex/codex-rs/collaboration-mode-templates/src/lib.rs`
- `vendor/codex/codex-rs/core/src/context/collaboration_mode_instructions.rs`
- `vendor/codex/codex-rs/app-server/src/codex_message_processor.rs`
- `vendor/codex/codex-rs/tui/src/collaboration_modes.rs`
- `vendor/codex/codex-rs/tui/src/chatwidget.rs`
- `vendor/codex/codex-rs/tui/src/bottom_pane/footer.rs`

## Recommended Next Implementation

Build Phase 1, not the full enum-backed mode:

1. Add `codex-rs/collaboration-mode-templates/templates/epiphany.md`.
2. Export it as `EPIPHANY` from `codex-rs/collaboration-mode-templates/src/lib.rs`.
3. Add an `Epiphany` preset in `codex-rs/models-manager/src/collaboration_mode_presets.rs`.
4. Keep the preset backed by `ModeKind::Default` initially.
5. Patch the TUI to display the preset name so Epiphany is visible even though it uses Default semantics.
6. Test manually before adding a real `ModeKind::Epiphany`.

Why:

- avoids protocol/schema churn
- avoids SDK regeneration churn
- keeps tool gating predictable
- tests the real behavioral hypothesis first

## What Not To Do First

Do not immediately add `ModeKind::Epiphany` unless the user explicitly asks for the full protocol-level mode. That path touches more of the stack: protocol types, analytics, UI indicators, tool gating, generated schemas, SDK types, and snapshots.

Do not build a large state-management subsystem before testing whether the instruction/preset path helps. A tiny helper tool may be useful later, but the first value test is the mode discipline itself.

## Prototype Philosophy

The useful behavior is:

- keep a canonical map for nontrivial work
- use scratch for one bounded subgoal
- explain the current mechanism before broad edits
- prefer one hypothesis per iteration
- verify against real goals and map coherence
- revert churn that does not move the needle
- stop coding when understanding degrades

The mode should not pretend the model has native typed channels. It should use explicit external files and write rules until a real architecture exists.

## Useful Verification

From the repo root:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_state.py' status
Get-Content -Tail 5 '.\state\evidence.jsonl'
git status --short
```

The vendored Codex repo is its own clone under `vendor/codex`. Treat it as source material for now unless the user asks to patch it.
