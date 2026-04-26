# Scratch

This file is disposable working memory for the current bounded subgoal.

## Current Subgoal

No active scratch subgoal.

## Last Completed Audit

- `thread/epiphany/jobs` now provides the first read-only job/progress reflection.
- The surface derives retrieval-index, graph-remap, verification, and specialist slots from live/stored thread state and retrieval summaries.
- It does not schedule, persist, notify, or mutate jobs.

## Decision

Keep jobs as a reflection board, not a scheduler. The next outward Phase 6 move
should add a missing sense only when there is a real consumer: targeted
graph/evidence reads, watcher/freshness inputs, or live job progress
notifications once actual long-running owners exist.

Scene and jobs smokes are guardrails now, not the next organs.

Do not promote anything from this scratch into the map unless it survives
verification or repeated reuse without contradiction.
