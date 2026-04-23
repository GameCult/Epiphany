# Scratch

This file is intentionally disposable.

## Current Subgoal

- Confirm the scaffold is coherent enough to use.
- Decide whether to patch vendored Codex with the lightweight Epiphany preset now.
- Decide whether the next executable artifact should be a patch runner or a verifier helper.

## Working Notes

- The protocol should be usable by any capable model because most of the structure lives outside the model.
- The first test should be small enough to run repeatedly without becoming its own research program.
- If the protocol becomes too ceremonial, it will lose to plain prompting for the most boring reason imaginable: friction.
- The current CLI is intentionally small. It exists to keep state honest, not to impersonate a full agent runtime.
- The Codex repo already has a cleaner-than-expected collaboration-mode preset seam, so a preset-backed Epiphany mode is a tractable first implementation.

## Open Questions

- What is the smallest useful verifier stack?
- Which coding task will expose drift quickly?
- How much branch management is enough before it turns into paperwork?
- Should the first in-Codex implementation be instructions-only, or should it also add a tiny state helper tool?

Do not promote anything from here into the map unless it survives verification or repeated reuse without contradiction.
