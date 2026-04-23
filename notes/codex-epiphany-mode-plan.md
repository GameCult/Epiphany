# Codex Epiphany Phase 4 Implementation Plan

## Status

Planned on 2026-04-23 after landing and verifying:

- Phase 1 durable Epiphany thread state
- Phase 2 prompt integration
- a minimal Phase 3 typed client read surface via `Thread.epiphanyState`

The next job is to give Epiphany a real repo retrieval organ instead of making every future role rediscover the workspace with shell commands and optimism.

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

- `vendor/codex/codex-rs/core/src/`

Add a focused retrieval module, tentatively:

- `epiphany_retrieval.rs`

Responsibilities:

- accept a loaded thread plus query params
- resolve the relevant workspace root
- run exact retrieval and semantic retrieval behind one interface
- return typed ranked results with file paths, line anchors, excerpts, and retrieval mode metadata

Result types should be explicit and bounded:

- exact hits
- semantic hits
- optional graph-linked ids later

Do not make the first cut depend on prompt parsing or transcript scraping.

### 3. Keep the first semantic backend small and local

The first semantic backend should be workspace-local, sharded, and incremental-friendly in storage layout, even if the initial indexing policy is still crude.

Requirements:

- do not store everything in one giant JSON blob
- prefer per-workspace or per-subsystem shards
- track index revision and freshness explicitly
- keep chunk text line-anchored and path-aware

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

Keep it additive and read-only. No write/update/index-control methods yet unless they turn out to be mechanically unavoidable.

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

Implement the smallest useful retrieval baseline:

1. extend `EpiphanyThreadState` with retrieval metadata
2. add a core hybrid retrieval facade
3. expose one typed `thread/epiphany/retrieve` read method
4. document it in `app-server/README.md`
5. verify with targeted protocol/core/app-server tests

Do not start with GUI. Do not start with automatic graph invalidation. Do not pretend shell transcripts are a retrieval strategy.
