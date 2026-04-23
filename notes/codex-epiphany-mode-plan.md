# Codex Epiphany Phase 1 Implementation Plan

This is the actionable implementation note for the first Epiphany patch slice against the real `openai/codex` tree vendored under `vendor/codex`.

Phase 1 is deliberately **internal/dev-usable**, not user-facing. The goal is to give Codex a real durable Epiphany state seam in core, keep the app usable, and prove replay/persistence before touching prompts, presets, protocol notifications, or GUI reflection.

## Summary

Land Phase 1 as an **additive core slice** that persists and rehydrates structured Epiphany thread state without changing normal Codex behavior.

After this slice:

- non-Epiphany sessions still behave exactly as they do now
- the app remains usable because existing clients ignore the new metadata
- Epiphany state can be exercised and validated through automated core/rollout/resume tests

This slice does **not** include:

- prompt integration
- app-server notifications
- GUI surfaces
- collaboration-mode presets
- mutation gates
- retrieval indexing
- specialist-agent orchestration

## Key Changes

### 1. Add minimal durable Epiphany protocol types

Touch:

- `vendor/codex/codex-rs/protocol/src/protocol.rs`

Add:

- `EpiphanyThreadState`
- `EpiphanyStateItem { turn_id: Option<String>, state: EpiphanyThreadState }`
- `RolloutItem::EpiphanyState(EpiphanyStateItem)`

Phase 1 schema should stay minimal but sufficient:

- `revision`
- `objective`
- `active_subgoal_id`
- `subgoals`
- `invariants`
- `graphs`
- `graph_frontier`
- `graph_checkpoint`
- `scratch`
- `observations`
- `recent_evidence`
- `churn`
- `mode`
- `last_updated_turn_id`

Defer these to later phases:

- `agents`
- `jobs`
- `retrieval`
- separate append-only evidence/checkpoint rollout items
- separate turn intent rollout item
- any new `EventMsg` variants

All new protocol types should derive the same serialization/schema traits as neighboring rollout types.

### 2. Store Epiphany state in `SessionState`

Touch:

- `vendor/codex/codex-rs/core/src/state/session.rs`
- `vendor/codex/codex-rs/core/src/session/mod.rs`

Add:

- `epiphany_state: Option<EpiphanyThreadState>` to `SessionState`
- simple getter/setter helpers on session state

Persistence rule:

- persist one `RolloutItem::EpiphanyState(...)` per real user turn
- write it immediately after the `TurnContextItem` baseline is persisted
- skip the write when `epiphany_state` is `None`

Do not change prompt assembly or tool behavior in this slice.

### 3. Rehydrate Epiphany state on resume and fork

Touch:

- `vendor/codex/codex-rs/core/src/session/rollout_reconstruction.rs`
- the session-start/resume path that consumes `RolloutReconstruction`

Extend reconstruction to carry:

- `epiphany_state: Option<EpiphanyThreadState>`

Replay rule:

- the newest surviving `EpiphanyStateItem` wins
- rollback must discard Epiphany snapshots from rolled-back turns
- compaction must not erase the latest surviving snapshot

Treat `EpiphanyStateItem` as turn-scoped metadata in the same reverse scan that already respects rollback and compaction boundaries.

### 4. Update rollout readers to tolerate the new variant

Adding a `RolloutItem` variant will create exhaustiveness fallout across the repo. Update those sites explicitly instead of letting it turn into whack-a-mole.

At minimum touch:

- `vendor/codex/codex-rs/app-server-protocol/src/protocol/thread_history.rs`
- `vendor/codex/codex-rs/rollout/src/policy.rs`
- `vendor/codex/codex-rs/rollout/src/metadata.rs`
- `vendor/codex/codex-rs/rollout/src/list.rs`
- `vendor/codex/codex-rs/rollout/src/recorder.rs`
- any additional exhaustive `RolloutItem` matches in `core`, `tui`, `exec`, and `protocol`

Phase 1 behavior at those call sites:

- thread-history/UI builders should ignore `EpiphanyState`
- rollout metadata/listing code should treat it as non-rendered metadata
- existing user-visible thread rendering must stay unchanged

## Test Plan

### Protocol and schema

- serde round-trip for `EpiphanyThreadState`
- serde round-trip for `EpiphanyStateItem`
- any protocol/schema snapshots updated for the new `RolloutItem` variant

### Core persistence

- a session with populated Epiphany state writes `RolloutItem::EpiphanyState`
- a session with `None` Epiphany state writes nothing extra
- the Epiphany rollout item appears after the turn-context baseline for the turn

### Replay correctness

- resume restores the newest surviving Epiphany snapshot
- rollback drops snapshots from discarded turns
- compaction keeps the latest surviving Epiphany snapshot available after reconstruction

### Compatibility

- `app-server-protocol` thread reconstruction still builds the same visible turn history
- rollout list/metadata readers still work with mixed old/new rollout files
- untouched user flows remain unchanged in tests

### Compile-fallout coverage

- build/test the touched crates so all exhaustive `RolloutItem` matches are updated in one pass

## Assumptions

- Phase 1 is **internal/dev-usable**, not operator-usable or GUI-visible.
- "Testable after Phase 1" means persistence/replay behavior is verifiable through automated tests and internal session wiring, not that users can activate Epiphany from the UI yet.
- Phase 1 keeps evidence and checkpoint data embedded inside the baseline Epiphany snapshot instead of splitting them into separate rollout item variants.
- Prompt integration, app-server typed updates, GUI scene loading, retrieval, jobs, and specialist-agent scheduling are later slices.

## Immediate Next Step After This Plan

Implement the protocol/core/replay slice only:

1. add the minimal protocol types
2. wire `SessionState`
3. persist one snapshot per real user turn
4. restore it through rollout reconstruction
5. patch exhaustive `RolloutItem` matches
6. add replay/persistence tests before moving on
