# Scratch

This file is disposable working memory for the current bounded subgoal.

## Current Subgoal

No active scratch subgoal.

## Source-Grounded Findings

- Existing core telemetry already has the real pressure source: `TokenUsageInfo`
  carries total usage, last usage, and `model_context_window` in
  `vendor/codex/codex-rs/protocol/src/protocol.rs`.
- Core updates that telemetry through `Session::update_token_usage_info`,
  `Session::recompute_token_usage`, and `TokenCount` events in
  `vendor/codex/codex-rs/core/src/session/mod.rs`.
- Automatic compaction is currently triggered from
  `ModelInfo::auto_compact_token_limit()` in
  `vendor/codex/codex-rs/core/src/session/turn.rs`, including pre-turn context
  limit and model-downshift paths.
- App-server already translates/replays usage through
  `thread/tokenUsage/updated` using
  `vendor/codex/codex-rs/app-server/src/codex_message_processor/token_usage_replay.rs`.
- The protocol surface `ThreadTokenUsage` in
  `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs` exposes
  `model_context_window` but not the auto-compact threshold, remaining budget,
  ratio, or CRRC recommendation.

## Next Implementation Move

Implement the smallest honest read-only pressure surface:

- add optional auto-compact threshold telemetry where token usage is created,
  preserving backward compatibility.
- add experimental `thread/epiphany/pressure` as a read-only projection over
  latest token telemetry.
- derive `unknown`, `low`, `elevated`, `high`, or `critical` pressure from real
  token usage versus the auto-compact threshold/context window.
- include a CRRC recommendation only as reflection, not action.
- do not start compaction, mutate Epiphany state, emit `stateUpdated`, create a
  scheduler, or implement automatic CRRC in this slice.

## Verification Target

Add focused protocol/app-server tests and a live smoke analogous to scene/jobs/
context that proves pressure reads live telemetry, reports `unknown` honestly
when telemetry is absent, and does not mutate or notify.
