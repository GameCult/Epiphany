# Scratch

This file is intentionally disposable.

## Current Subgoal

- Keep the new `epiphany-core` extraction honest in repo memory, then smoke the explicit indexing path against live Qdrant/Ollama without widening the machine again.

## Working Notes

- The landed Phase 4 slice 1 baseline is still `360dfea` on `main`. The current working tree still carries the explicit `thread/epiphany/index` Qdrant follow-up on top of that.
- The extraction boundary is now the important truth:
  - `epiphany-core/src/retrieval.rs` owns the heavy hybrid retrieval/indexing engine
  - `epiphany-core/src/prompt.rs` owns the Epiphany prompt-state renderer
  - `epiphany-core/src/rollout.rs` owns the latest-Epiphany-state replay helper used for Phase 3 stored-thread hydration
  - `epiphany-core/src/lib.rs` re-exports the stable surface used by vendored Codex
  - `vendor/codex/codex-rs/core/src/epiphany_retrieval.rs` is now a thin re-export wrapper
  - `vendor/codex/codex-rs/core/src/epiphany_rollout.rs` is now a thin wrapper that supplies the Codex-specific user-turn-boundary predicate
  - `vendor/codex/codex-rs/core/src/context/epiphany_state_instructions.rs` is now a thin `ContextualUserFragment` adapter around `epiphany_core::render_epiphany_state(...)`
  - `vendor/codex/codex-rs/core/Cargo.toml` now depends on the sibling crate by path
- What still must live in vendored Codex because that is where the typed host seam exists:
  - protocol types in `vendor/codex/codex-rs/protocol/src/protocol.rs`
  - `CodexThread` bridge methods in `vendor/codex/codex-rs/core/src/codex_thread.rs`
  - the user-turn-boundary rule in `vendor/codex/codex-rs/core/src/context_manager/history.rs`
  - app-server protocol surfaces in `vendor/codex/codex-rs/app-server-protocol/src/protocol/common.rs` and `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs`
  - request routing in `vendor/codex/codex-rs/app-server/src/codex_message_processor.rs`
- Why this shape is worth keeping:
  - it shrinks the Apache-mixed surface
  - it makes modified Codex alone less useful as a standalone Epiphany rebuild kit
  - it preserves first-class typed integration instead of hiding the machine behind an opaque plugin blob
- Retrieval/indexing behavior is still the bounded machine we wanted:
  - exact/path-ish lookup via `codex_file_search`
  - semantic lookup via BM25 chunk search when persistent semantic state is unavailable or stale
  - explicit `thread/epiphany/index` for Qdrant-backed semantic persistence
  - read-only `thread/epiphany/retrieve`
  - manifest metadata under `codex_home`
  - local Ollama embeddings defaulting to `qwen3-embedding:0.6b`
- Corpus preflight for the tracked `vendor/codex/codex-rs` source at `vendor/codex` HEAD `d45ab10`: `3642` tracked files / `31.09 MB`.
- Do not use raw working-tree size as the retrieval denominator here; build artifacts and other sludge lie.
- Verification is now split cleanly across the repo-owned crate and the vendored host:
  - `cargo fmt --manifest-path .\\epiphany-core\\Cargo.toml`
  - `cargo test --manifest-path .\\epiphany-core\\Cargo.toml`
  - `cargo fmt --all`
  - `cargo test -p codex-core --lib epiphany`
  - `cargo test -p codex-app-server-protocol --lib thread_epiphany_`
  - `cargo test -p codex-app-server --lib map_epiphany_`
  - `cargo test -p codex-core -p codex-app-server-protocol -p codex-app-server --lib --no-run` with `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`
- Mechanical honesty still matters:
  - do not widen `thread/epiphany/retrieve` into a durable Epiphany-state write without a clean out-of-band rollout/update semantic
  - do not let `epiphany-core` sprawl into GUI or watcher machinery just because it now owns the bigger organ

## Open Questions

- After the explicit indexing path has been live-smoked, should backend config stay env-scoped for a while or earn a cleaner first-class config surface?
- How much more can move into `epiphany-core` without sacrificing the typed Codex host seam that makes the integration first-class?

Do not promote anything from here into the map unless it survives verification or repeated reuse without contradiction.
