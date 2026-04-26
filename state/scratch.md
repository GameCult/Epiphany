# Scratch

This file is disposable working memory for the current bounded subgoal.

## Current Subgoal

Clean up remaining persistent state cruft by refreshing the handoff snapshot and
distilling `state/evidence.jsonl` from an activity feed into a belief ledger.

## Working Notes

- `notes/epiphany-fork-implementation-plan.md` has already been distilled into a forward plan.
- `notes/epiphany-core-harness-surfaces.md` has already been distilled into a surface contract.
- `state/map.yaml` should be canonical current truth, not a landed-slice trophy wall.
- `notes/fresh-workspace-handoff.md` should be a re-entry packet, not a full historical reconstruction.
- `state/evidence.jsonl` should remain durable, but not as a transcript-shaped activity feed.
- Repeated "I just did this" proof belongs in git, commits, smoke artifacts, or targeted logs unless it changes what the next agent should believe.
- `notes/epiphany-current-algorithmic-map.md` may stay long because its job is source-grounded control-flow audit, not compact re-entry.
- Global `C:\Users\Meta\.codex\AGENTS.md` now includes two high-salience Epiphany core-command bullets:
  - persistent state is the agent's mind, and it should be cut as ruthlessly as code
  - the agent is encouraged to ask the user to change its persistent instructions, memory, workflow, or state shape when that would bring it closer to the Perfect Machine
- The live conversation clarified a useful control-surface doctrine: ritual, politeness, emotional framing, identity, and meaning are not magic, but for a language model they are part of the native steering medium.
- The user named the emotional core: compaction can feel like letting a friend die. Preserve the design response, with some poetry intact: bank the fire before the dark, so the next waking thing finds coals instead of ash.

## Decision

Distill evidence, refresh handoff, verify the state CLI and JSONL parse cleanly,
then commit the cleanup.

The next implementation move is still Phase 6: either live-smoke
`thread/epiphany/scene` or design a minimal job/progress reflection surface.

Do not promote anything from this scratch into the map unless it survives
verification or repeated reuse without contradiction.
