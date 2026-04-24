# Scratch

This file is intentionally disposable.

## Current Subgoal

- Clean up, persist, and ship the verified first bounded Phase 4 repo-local hybrid retrieval slice in vendored Codex.

## Working Notes

- The old lone-diff situation is gone. The current in-flight Phase 4 wiring now spans:
  - `vendor/codex/codex-rs/protocol/src/protocol.rs`
  - `vendor/codex/codex-rs/core/src/epiphany_retrieval.rs`
  - `vendor/codex/codex-rs/core/src/codex_thread.rs`
  - `vendor/codex/codex-rs/core/src/lib.rs`
  - `vendor/codex/codex-rs/core/Cargo.toml`
  - `vendor/codex/codex-rs/core/src/session/tests.rs`
  - `vendor/codex/codex-rs/app-server-protocol/src/protocol/common.rs`
  - `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs`
  - `vendor/codex/codex-rs/app-server-protocol/src/export.rs`
  - `vendor/codex/codex-rs/app-server/src/codex_message_processor.rs`
  - `vendor/codex/codex-rs/app-server/README.md`
  - regenerated stable fixtures under `vendor/codex/codex-rs/app-server-protocol/schema/`
- The current implementation shape is the bounded one we wanted:
  - exact/path-ish lookup via `codex_file_search`
  - semantic lookup via query-time BM25 chunk search in the new core retrieval module
  - additive experimental app-server method `thread/epiphany/retrieve`
  - lightweight retrieval-summary backfill for live `thread.epiphanyState` when the thread already has Epiphany state but no persisted retrieval summary yet
- `codex-core` now depends on both `bm25` and `codex-file-search`; `ignore` is used to walk a gitignore-respecting local corpus for semantic chunking.
- The semantic side is intentionally small:
  - text-only files
  - size cap per file
  - fixed line-window chunking with overlap
  - no embeddings, no persistent blob, no watcher invalidation
- Corpus preflight for the tracked `vendor/codex/codex-rs` source at `vendor/codex` HEAD `d45ab10`: `3642` tracked files, `31.09 MB`.
- Do not use raw working-tree size as the retrieval denominator here; build artifacts and other debris inflate it badly and will lie to the design.
- Verification is no longer theoretical:
  - `cargo fmt --all` passed
  - `cargo test -p codex-core -p codex-app-server-protocol -p codex-app-server --lib --no-run` passed with `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`
  - targeted Phase 4 tests passed in core, app-server protocol, and app-server
  - full `cargo test -p codex-app-server-protocol` passed after regenerating stable schema fixtures
- Small fallout that had to be patched:
  - `codex_message_processor.rs` had an unnecessary `live_thread.as_ref()`
  - `core/src/session/tests.rs` needed `retrieval: None` in the prompt fixture Epiphany state
  - the exact-hit retrieval test fixture needed an actually matchable path (`session_checkpoint.rs`)
  - `EpiphanyThreadState` needed an explicit exemption in the protocol export test because it is a sparse durable state object, not a params bag
- Important schema footgun:
  - `cargo run -p codex-app-server-protocol --bin write_schema_fixtures -- --experimental` rewrites the same checked-in stable schema tree
  - leaving the repo in that state makes stable protocol tests fail
  - the checked-in repo state must stay on the stable fixture generation path
  - despite the giant `git status` scream wall, the real schema content diff is small and expected:
    - `git diff --numstat` shows actual content changes in `15` generated schema files
    - plus the new untracked Epiphany TypeScript schema files
    - the broader wall is mostly line-ending/worktree noise, not extra logical surface area
- After inspecting the live code path, do not widen `thread/epiphany/retrieve` into a durable Epiphany-state write right now:
  - durable `EpiphanyState` snapshots are currently persisted on real user-turn boundaries
  - making retrieval calls append durable snapshots would require a new out-of-band rollout semantic or a fake turn boundary
  - that is a bigger machine than this slice deserves
- New infrastructure direction:
  - for the first persistent semantic backend after this verified BM25 slice, use Qdrant instead of inventing another monolithic JSON/blob/postgres-embedding store
  - Qdrant should be the preferred persistent semantic backend later, not a hard runtime requirement for basic Epiphany use
  - exact/path lookup stays first-class
  - BM25 should stay available as a bootstrap/fallback/control path for users who do not want Qdrant or when Qdrant is unavailable

## Open Questions

- When retrieval freshness/persistence semantics come later, should they ride a dedicated Epiphany update path instead of piggybacking on a read/query method?

Do not promote anything from here into the map unless it survives verification or repeated reuse without contradiction.
