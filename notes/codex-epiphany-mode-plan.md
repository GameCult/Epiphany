# Codex Epiphany Phase 4 Implementation Plan

## Status

Updated on 2026-04-24 after landing the first bounded Phase 4 slice on `main`:

- Phase 1 durable Epiphany thread state
- Phase 2 prompt integration
- a minimal Phase 3 typed client read surface via `Thread.epiphanyState`
- a verified Phase 4 slice 1 query-time hybrid retriever across protocol, core, app-server protocol, and app-server
- a verified Phase 4 slice 2 working-tree follow-up that adds explicit persistent semantic indexing through `thread/epiphany/index` while keeping `thread/epiphany/retrieve` read-only
- the heavy Epiphany-owned prompt/replay/retrieval implementation has now been extracted into the repo-owned `epiphany-core` crate, leaving vendored Codex with thin host adapters plus the typed integration seam

The next job is no longer to prove the retriever exists, and it is no longer to sketch the persistent follow-up in prose. That part is done in the working tree. The next job is to keep the landing zone honest: do not casually turn `thread/epiphany/retrieve` into a durable Epiphany-state writer just because the state object has a `retrieval` field, and do not widen the new explicit indexing path into watcher-driven or GUI-shaped machinery before it earns it.

## Summary

Land Phase 4 as a **bounded internal retrieval slice**:

- add typed retrieval state and shard/index summaries to Epiphany state
- add one additive app-server retrieval query surface for loaded threads
- support hybrid retrieval from day one:
  - exact/path/symbol/lexical results
  - semantic chunk results
- keep the slice internal/dev-usable first

After this slice:

- Epiphany can ask structured repo questions through one typed surface
- GUI work has a real data source later
- retrieval can evolve without making prompt text or shell transcripts the canonical knowledge path

This slice should still avoid:

- watcher-driven invalidation
- automatic observation promotion
- GUI implementation
- specialist-agent scheduling
- user-facing activation flows

## Phase 4 Principle

Do not build "vector search" as a separate novelty organ off to the side.

Build a **hybrid repo retrieval subsystem** with one typed query shape and one typed result shape. Exact lookup and semantic lookup should be different modes of the same machine, not rival religions.

## Recovered bounded decision

After rehydrating from the repo state and the lone dirty working-tree diff, the first bounded Phase 4 slice should be:

- query-time
- loaded-thread-first
- read-only
- hybrid from day one

Concrete choice for the first implementation:

- reuse existing `codex_file_search` substrate for exact/path-ish lookup
- use workspace-local BM25 chunk search as the initial semantic backend
- defer embeddings, vector stores, persistent indexing, and watcher-driven invalidation

Updated follow-up direction after landing the verified slice:

- keep the BM25 implementation as the proven baseline/fallback
- for the first persistent semantic backend, prefer Qdrant over JSON/blob/postgres-thrash designs, but do not make it a hard requirement for basic Epiphany use
- do not turn retrieval into vector-only soup; exact/path lookup remains first-class and BM25 remains useful as bootstrap/control/fallback when Qdrant is unavailable

Preflight note:

- use tracked source as the corpus estimate, not raw working-tree disk usage
- at `vendor/codex` HEAD `d45ab10`, tracked `codex-rs` source is `3642` files / `31.09 MB`
- raw working-tree size under `codex-rs` can be heavily inflated by build artifacts and will lie to the design

Current landed implementation note:

- the protocol scaffold still exists in `vendor/codex/codex-rs/protocol/src/protocol.rs`
- the heavier Epiphany-owned implementation now lives in:
  - `E:/Projects/EpiphanyAgent/epiphany-core/src/prompt.rs`
  - `E:/Projects/EpiphanyAgent/epiphany-core/src/rollout.rs`
  - `E:/Projects/EpiphanyAgent/epiphany-core/src/retrieval.rs`
- the vendored host seam is now wired through the working tree across:
  - `core/src/epiphany_retrieval.rs`
  - `core/src/epiphany_rollout.rs`
  - `core/src/context/epiphany_state_instructions.rs`
  - `core/src/codex_thread.rs`
  - `core/src/session/tests.rs`
  - `app-server-protocol/src/protocol/common.rs`
  - `app-server-protocol/src/protocol/v2.rs`
  - `app-server-protocol/src/export.rs`
  - `app-server/src/codex_message_processor.rs`
- the slice is now formatted, verified, committed, and pushed on `main`:
  - `cargo fmt --all` passed
  - targeted no-run compile for `codex-core`, `codex-app-server-protocol`, and `codex-app-server` passed
  - targeted Phase 4 tests passed in core, app-server protocol, and app-server
  - full `cargo test -p codex-app-server-protocol` passed after regenerating stable schema fixtures
- stable schema fixtures are the safe checked-in state for this slice; `write_schema_fixtures --experimental` rewrites the same tree and cannot be left behind as the repo state

## Key Changes

### 1. Extend Epiphany state with retrieval metadata

Touch:

- `vendor/codex/codex-rs/protocol/src/protocol.rs`

Add minimal retrieval metadata to `EpiphanyThreadState`, for example:

- `retrieval`
  - `workspace_root`
  - `index_revision`
  - `status`
  - `semantic_available`
  - `last_indexed_at`
  - `shards`
  - `dirty_paths`

Keep this metadata summary-focused. Do not dump raw embeddings, giant posting lists, or per-file sludge into thread state.

### 2. Add a core retrieval facade

Touch:

- `E:/Projects/EpiphanyAgent/epiphany-core/src/retrieval.rs`
- `vendor/codex/codex-rs/core/src/epiphany_retrieval.rs`

Responsibilities:

- the repo-owned crate should accept a loaded-thread-derived workspace root plus query params
- it should run exact retrieval and semantic retrieval behind one interface
- it should return typed ranked results with file paths, line anchors, excerpts, and retrieval mode metadata
- the vendored Codex file should stay a thin adapter or re-export layer, not the place where the heavy organ grows back out of spite

Result types should be explicit and bounded:

- exact hits
- semantic hits
- optional graph-linked ids later

Do not make the first cut depend on prompt parsing or transcript scraping.

### 3. Keep the first semantic backend small and local

The first semantic backend should be workspace-local BM25 chunk retrieval built at query time over the tracked source corpus.

Requirements:

- keep chunk text line-anchored and path-aware
- keep the implementation bounded enough to understand in one sitting
- track retrieval/index freshness metadata explicitly in typed state
- do not introduce a monolithic persistent blob just to feel sophisticated

If the first semantic backend needs to be intentionally narrow, that is fine. A small honest subsystem beats a magical blob with terrible write behavior.

### 4. Add one additive app-server query surface

Touch:

- `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs`
- `vendor/codex/codex-rs/app-server/src/codex_message_processor.rs`
- `vendor/codex/codex-rs/app-server/README.md`

Add one typed method for retrieval, tentatively:

- `thread/epiphany/retrieve`

Suggested request fields:

- `threadId`
- `query`
- optional `cwd`
- optional `modes` or `preferSemantic`
- optional `limit`
- optional `pathPrefixes`

Suggested response fields:

- `query`
- `indexSummary`
- `results`

Keep it additive and read-only. That part is now landed.

Follow-up addition that is now verified in the working tree:

- `thread/epiphany/index`

Current bounded shape of that follow-up:

- it is the explicit write path for persistent semantic indexing
- it persists manifest metadata under `codex_home`
- it targets Qdrant as the preferred persistent semantic backend
- it uses local Ollama embeddings by default
- it leaves exact/path lookup first-class
- it keeps BM25 alive as the fallback/control path when the persistent backend is stale, missing, or unavailable
- it now runs through the sibling `epiphany-core` crate so modified Codex alone is not quietly the whole product

### 5. Reuse existing exact-search substrate where practical

Codex already has fuzzy/exact-ish repo search substrate. Reuse what is useful instead of building a second exact-search toy from scratch.

But do not confuse that with the whole answer. Phase 4 exists because exact file search alone is not enough.

### 6. Test the machine, not just the types

Add tests for:

1. protocol serde for the new retrieval request/response shapes
2. core ranking/merging behavior for mixed exact + semantic results
3. app-server handling for the new request
4. retrieval metadata inclusion in `EpiphanyThreadState`
5. a smoke test that proves semantic retrieval can find a concept that exact-name search would miss

## Assumptions

- Phase 4 is still internal/dev-usable only.
- Retrieval should be thread/workspace aware, not a global cross-repo soup.
- The first retrieval slice can be read-only.
- Watcher-driven invalidation belongs later; this slice only needs enough metadata to admit freshness honestly.

## Immediate Next Step After This Plan

Treat the current retrieval baseline and the new explicit indexing follow-up as real and only grow them where the current machine is still visibly missing an organ:

1. treat the current verified-and-landed query-time hybrid retriever as the Phase 4 slice 1 baseline
2. treat the verified working-tree `thread/epiphany/index` slice as the bounded persistent-semantic follow-up
3. keep the new `epiphany-core` boundary honest instead of letting vendored Codex re-accumulate the heavy implementation
4. do not add durable retrieval-summary writes from `thread/epiphany/retrieve` without a clean out-of-band rollout/update semantic
5. if a later follow-up is needed, keep it tight:
   - live smoke/operational polish for the explicit indexing path
   - maybe a cleaner config surface for backend settings if env-only starts feeling too feral
   - not GUI work
   - not watcher-driven invalidation

Do not start with GUI. Do not start with automatic graph invalidation. Do not pretend shell transcripts are a retrieval strategy.
