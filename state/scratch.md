# Scratch

This file is disposable working memory for the current bounded subgoal.

## Current Subgoal

No active scratch subgoal.

## Last Completed Audit

- `README.md` was stale and still pointed at a Phase 5 next step.
- `notes/architecture-rationale.md` still described the early cheap prototype as if it were live state.
- `protocol/controller-actions.md` needed distilled-evidence wording.
- `notes/fresh-workspace-handoff.md` had exact branch/HEAD snapshot rot.
- `state/branches.json` carried volatile phase status in branch notes.

## Decision

Cut stale status from handoff/branch notes, rewrite README/rationale around the
current fork spine, and keep evidence as a belief ledger.

Next implementation move remains Phase 6: either live-smoke
`thread/epiphany/scene` or design a minimal job/progress reflection surface.

Do not promote anything from this scratch into the map unless it survives
verification or repeated reuse without contradiction.
