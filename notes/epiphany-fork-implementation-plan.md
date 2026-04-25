# Epiphany Fork Implementation Plan

## Status

Updated on 2026-04-24 after landing the Phase 4 retrieval/indexing/core-extraction baseline and moving into Phase 5 semantic distillation/promotion on `main`.

This note tracks Epiphany as a fork of Codex with an opinionated modeling architecture. The point is not to offer Codex another collaboration preset. The point is to make the harness force the model to carry explicit structure about the codebase, the active subgoal, the evidence trail, and the machine it is modifying.

- Phase 1 durable Epiphany thread state
- Phase 2 prompt integration
- a minimal Phase 3 typed client read surface via `Thread.epiphanyState`
- a verified Phase 4 slice 1 query-time hybrid retriever across protocol, core, app-server protocol, and app-server
- a verified Phase 4 slice 2 follow-up that adds explicit persistent semantic indexing through `thread/epiphany/index` while keeping `thread/epiphany/retrieve` read-only
- the heavy Epiphany-owned prompt/replay/retrieval implementation has now been extracted into the repo-owned `epiphany-core` crate, leaving vendored Codex with thin host adapters plus the typed integration seam
- a first typed state-update slice that adds explicit loaded-thread-only `thread/epiphany/update`, appends observations/evidence, replaces bounded map/scratch/churn fields, bumps the state revision, and persists an immediate rollout `EpiphanyState` snapshot
- a first typed distillation/proposal slice that adds read-only loaded-thread-only `thread/epiphany/distill`, producing deterministic observation/evidence patches for explicit promotion through `thread/epiphany/update`
- a first richer promotion-policy slice that treats map/churn/frontier/checkpoint replacements as higher-risk state edits: accepted promotions must be tied to explicit observations and patch evidence, and structural graph/churn mistakes are rejected before the durable update path
- a first typed map/churn proposal slice that adds read-only loaded-thread-only `thread/epiphany/propose`, producing candidate graph frontier/churn patches from verified observations with code refs for explicit promotion through `thread/epiphany/promote`
- a first proposal-quality hardening slice that reuses existing architecture graph nodes by exact code-ref, same-path overlap, or deterministic path-node id before creating new candidate nodes, and records whether a proposal refines, expands, or updates the map in churn
- a frontier-focus hardening slice that expands proposal frontier focus through existing graph links and marks named incident graph edges active, so prompt rendering gets the relevant architecture/dataflow context instead of one lonely node with a tiny hat
- an evidence-selection/churn hardening slice that requires selected proposal observations to cite accepting `recent_evidence` and derives proposal `diff_pressure` from the candidate map delta, touched paths, selected observation count, and unresolved write risk
- a graph-semantic proposal hardening slice that scores selected observations/evidence, uses the strongest selected signal for proposal wording, and can reuse unanchored architecture graph nodes through strict unique semantic overlap while refusing ambiguous matches and leaving code-anchored nodes under concrete ref/path/id matching
- current phase: Phase 5 semantic distillation/promotion/proposal machinery

The next job is no longer to prove the retriever exists, sketch the persistent follow-up in prose, invent the first red-pen path, build the first observation proposal surface, make promotion notice broken map/churn replacements, ship the first read-only map/churn proposal surface, teach proposal to avoid duplicating already-mapped code surfaces, make proposal frontier focus follow existing graph links, make selected observations evidence-backed with non-haunted churn pressure, or rescue unanchored graph nodes with strict semantic matching. Those parts are landed. The next job is to keep the landing zone honest: do not casually turn `thread/epiphany/retrieve`, `thread/epiphany/distill`, or `thread/epiphany/propose` into durable Epiphany-state writers, and do not widen the explicit indexing/update paths into watcher-driven or GUI-shaped machinery before they earn it.

## Summary

Phase 4 lands as a **bounded internal retrieval/indexing slice**:

- add typed retrieval state and shard/index summaries to Epiphany state
- add one additive app-server retrieval query surface for loaded threads
- add one explicit semantic indexing surface for loaded threads
- support hybrid retrieval from day one:
  - exact/path/symbol/lexical results
  - semantic chunk results
- keep the slice internal/dev-usable first while the fork architecture hardens

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

After rehydrating from the repo state, the first bounded Phase 4 slice was:

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
- live app-server smoke verified the explicit Qdrant path against real local services using an ephemeral loaded thread rooted at `epiphany-core`; the smoke indexed 5 files into 198 chunks, created Qdrant collection `epiphany_workspace_fa24bab116f8d229`, and retrieved persistent semantic chunks from `src/prompt.rs`
- env/default backend configuration is good enough for the current slice; defer a first-class backend config surface until a real operator pain proves it deserves to exist

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
- the vendored host seam is now wired through:
  - `core/src/epiphany_retrieval.rs`
  - `core/src/epiphany_rollout.rs`
  - `core/src/context/epiphany_state_instructions.rs`
  - `core/src/codex_thread.rs`
  - `core/src/session/tests.rs`
  - `app-server-protocol/src/protocol/common.rs`
  - `app-server-protocol/src/protocol/v2.rs`
  - `app-server-protocol/src/export.rs`
  - `app-server/src/codex_message_processor.rs`
- the retrieval/indexing/core-extraction baseline is now formatted, verified, committed, and pushed on `main` at `80c29e0`:
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

Follow-up addition that is now verified and pushed:

- `thread/epiphany/index`

Current bounded shape of that follow-up:

- it is the explicit write path for persistent semantic indexing
- it persists manifest metadata under `codex_home`
- it targets Qdrant as the preferred persistent semantic backend
- it uses local Ollama embeddings by default
- it leaves exact/path lookup first-class
- it keeps BM25 alive as the fallback/control path when the persistent backend is stale, missing, or unavailable
- it now runs through the sibling `epiphany-core` crate so modified Codex alone is not quietly the whole product

Follow-up addition now landed in the working tree:

- `thread/epiphany/update`

Current bounded shape of that follow-up:

- it is the explicit write path for durable Epiphany state, not a retrieval side effect
- it requires a loaded thread
- it optionally checks `expectedRevision`
- it can append observations and evidence
- it can replace bounded typed fields: objective, active subgoal, subgoals, invariants, graphs, graph frontier/checkpoint, scratch, churn, and mode
- it increments the Epiphany revision, updates `lastUpdatedTurnId` when a reference turn exists, writes live `SessionState`, persists `RolloutItem::EpiphanyState`, and flushes the rollout
- rollout replay now accepts an out-of-band Epiphany snapshot before the first real user turn so pre-turn seed updates survive resume

Follow-up addition now landed after the update surface:

- `thread/epiphany/distill`

Current bounded shape of that follow-up:

- it is a read-only proposal path, not a second state writer
- it requires a loaded thread so the response can include the current `expectedRevision`
- it normalizes one explicit source/status/text observation plus optional subject, evidence kind, and code refs
- it returns deterministic observation/evidence records inside a `ThreadEpiphanyUpdatePatch`
- callers must pass the patch to `thread/epiphany/update` to make it durable

Follow-up addition now landed after the promotion gate:

- richer `thread/epiphany/promote` policy for state replacement patches

Current bounded shape of that follow-up:

- it keeps the wire protocol stable
- it still rejects failed verifier evidence without mutation
- it now passes replacement fields into `epiphany-core` policy evaluation instead of reducing them to a generic boolean
- map/churn/frontier/checkpoint replacements must include explicit observations and patch evidence, so state edits are tied to a verified observation trail
- subgoals, invariants, graphs, graph links, frontier ids, checkpoint ids, and churn fields get lightweight structural validation before accepted patches are sent to `thread/epiphany/update`
- it still does not infer graph edits automatically; callers must submit explicit typed replacements

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

## Immediate Next Step

Treat the current retrieval baseline and explicit indexing follow-up as landed Phase 4. Treat the explicit update path, distillation proposal path, read-only map/churn proposal path, verifier-backed promotion gate, structural map/churn promotion validation, graph-node reuse, linked frontier focus, evidence-backed selection, map-delta churn pressure, selected-observation prioritization, and strict unanchored-node semantic reuse as the active Phase 5 baseline. The next machine gap is no longer "can Epiphany retrieve code?", "can it propose one durable observation patch?", "can it draft one bounded map/churn candidate?", "can it avoid duplicating an existing graph node when an observation points at already-mapped code?", "can it focus linked graph context?", "can it reject selected observations that are not backed by accepting recent evidence?", or "can it use graph language when no concrete refs exist yet?" It can. The next gap is "can Epiphany choose good observation sets and richer map deltas from tool/model output without silently promoting them?"

1. treat the current verified-and-landed query-time hybrid retriever as the Phase 4 slice 1 baseline
2. treat the verified and live-smoked `thread/epiphany/index` slice as the bounded persistent-semantic follow-up
3. keep the new `epiphany-core` boundary honest instead of letting vendored Codex re-accumulate the heavy implementation
4. do not add durable retrieval-summary writes from `thread/epiphany/retrieve` without a clean out-of-band rollout/update semantic
5. continue Phase 5 by hardening the layer above the landed distill/propose/promote/update surfaces:
   - richer typed observation distillation from tool/model outputs
   - better proposal heuristics beyond the current exact-code-ref, same-path, deterministic-id, graph-link, accepting-evidence, semantic-unanchored-node, selected-priority, and map-delta-pressure checks
   - better observation set selection beyond caller-supplied ids and evidence links
   - promotion policy beyond structural validation, but only when it can stay evidence-backed
   - verifier-backed acceptance/rejection evidence
   - no hidden retrieval writes
   - not GUI work
   - not watcher-driven invalidation
   - not specialist-agent scheduling

Do not start with GUI. Do not start with automatic graph invalidation. Do not pretend shell transcripts are a retrieval strategy.
