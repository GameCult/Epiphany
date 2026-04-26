# Scratch

This file is disposable working memory for the current bounded subgoal.

## Current Subgoal

No active scratch subgoal.

## Last Completed Audit

- `thread/epiphany/context` now provides the first targeted read-only state-shard reflection.
- The surface selects graph nodes/edges, active frontier, checkpoint, observations, direct evidence, and linked evidence from authoritative typed state.
- It reports missing requested ids and does not retrieve, propose, promote, notify, persist, or mutate state.

## Decision

Keep context as a bounded lens, not a hidden proposal engine. The next outward
Phase 6 move should add a missing sense only when there is a real consumer:
watcher/freshness inputs or live job progress notifications once actual
long-running owners exist.

Scene, jobs, and context smokes are guardrails now, not the next organs.

Do not promote anything from this scratch into the map unless it survives
verification or repeated reuse without contradiction.
