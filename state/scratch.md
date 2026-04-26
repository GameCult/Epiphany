# Scratch

This file is disposable working memory for the current bounded subgoal.

## Current Subgoal

No active scratch subgoal.

## Last Completed Audit

- `thread/epiphany/scene` had protocol and mapper coverage but no live app-server smoke.
- Scene latest-record mapping reversed the durable newest-first record order.
- Async update notifications had to be drained before asserting that scene emits no state update notification.

## Decision

Add `tools/epiphany_phase6_scene_smoke.py`, fix scene latest records to preserve
newest-first order, and make the smoke prove missing/ready state, live source,
retrieval backfill, action availability, bounded latest records, and no
scene-triggered `thread/epiphany/stateUpdated`.

Next implementation move is Phase 6 job/progress reflection: design the minimal
read-only surface for indexing, remap, verification, and future specialist work.

Do not promote anything from this scratch into the map unless it survives
verification or repeated reuse without contradiction.
