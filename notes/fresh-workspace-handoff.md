# Fresh Workspace Handoff

## What This Repo Is

`EpiphanyAgent` is a prototype workspace for upgrading AI coding harnesses so they stop confusing local motion with global understanding.

The working idea is simple enough:

- keep canonical understanding in typed state
- keep scratch bounded and disposable
- keep evidence durable
- make the harness consult that state instead of pretending the transcript is a brain

The project target is now the vendored Codex core harness itself, not a sidecar wrapper first and not some random GitHub "OpenCodex" contraption held together with hope and string.

## Current Repo State

Important consequence:

- `vendor/codex` is now tracked directly in the parent repo as ordinary files
- it is **not** a submodule anymore
- `epiphany-core/` is now a sibling repo-owned crate that holds the bulk of the Epiphany prompt, rollout replay, and retrieval/indexing logic instead of leaving that whole organ smeared through vendored Codex

Recent anchor commits:

- `2042687e3035c5a86d7f6aa66306d87abcc10f2d` - vendored Codex and landed the Phase 1 Epiphany core state slice
- `c823815` - persisted the Phase 2 implementation plan and refreshed the project handoff/state
- `efd1420` - landed and pushed the Phase 2 prompt-integration slice
- `640e063` - documented the pre-compaction Epiphany workflow and synced the repo memory before compaction
- `360dfea` - landed and pushed the Phase 4 hybrid retrieval slice

Do not trust this note for the exact current HEAD; use `git log --oneline -1` if you need the live commit.

Canonical project state still lives in:

- `state/map.yaml`
- `state/scratch.md`
- `state/evidence.jsonl`

## Licensing State

- the repo now has a top-level `LICENSE` file that acts as an operative
  repository license notice rather than a policy draft
- `vendor/codex/**` remains governed by its upstream Apache-2.0 and related
  third-party notices
- Project-Authored Material outside `vendor/codex/**` is publicly licensed
  under PolyForm Noncommercial 1.0.0, with separate commercial licensing
  intended by written agreement
- external contributions require `CONTRIBUTOR_LICENSE_AGREEMENT.md` or a
  separate written agreement accepted by the Project Steward; contributors keep
  ownership but grant broad sublicensing/relicensing rights so future commercial
  licensing is not blocked by old contribution archaeology
- do not describe the repository as a whole as OSI Open Source unless and until
  the project-authored material is also published under an OSI-approved license

Current implementation plan note:

- `notes/codex-epiphany-mode-plan.md`

Current architectural/spec notes:

- `notes/codex-repository-algorithmic-map.md`
- `notes/epiphany-current-algorithmic-map.md`
- `notes/epiphany-core-harness-surfaces.md`

## Current Landed Baseline

Phase 4 repo-local hybrid retrieval is now landed on `main` at `360dfea` and pushed to `origin/main`. It is no longer a vague scaffold, a maybe, or a working-tree-only event.

Current verified working-tree follow-up after that landed baseline:

- explicit `thread/epiphany/index`
- manifest-backed Qdrant semantic persistence under `codex_home`
- local Ollama embeddings by default
- stale/fresh retrieval summaries
- read-only `thread/epiphany/retrieve` with BM25 fallback still intact
- repo-owned `epiphany-core` extraction for the heavy prompt/replay/retrieval organ, leaving vendored Codex with thin adapters plus the typed host seam

Current verified working-tree implementation spans:

- `epiphany-core/Cargo.toml`
- `epiphany-core/src/prompt.rs`
- `epiphany-core/src/rollout.rs`
- `epiphany-core/src/retrieval.rs`
- `epiphany-core/src/lib.rs`
- `vendor/codex/codex-rs/protocol/src/protocol.rs`
- `vendor/codex/codex-rs/core/src/epiphany_retrieval.rs`
- `vendor/codex/codex-rs/core/src/epiphany_rollout.rs`
- `vendor/codex/codex-rs/core/src/context/epiphany_state_instructions.rs`
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

What is wired right now:

- `EpiphanyThreadState` already has retrieval metadata fields and types in protocol
- repo-owned `epiphany-core` now owns:
  - the prompt-state renderer used by Phase 2 prompt injection
  - the rollout replay helper used by Phase 3 stored-thread hydration and rollback/compaction-aware reads
  - the heavy retrieval/indexing engine used by the current Phase 4 work
- vendored `codex-core` now keeps thin adapters at:
  - `core/src/epiphany_retrieval.rs`
  - `core/src/epiphany_rollout.rs`
  - `core/src/context/epiphany_state_instructions.rs`
- the retrieval machine is still hybrid:
  - exact/path-ish lookup via `codex_file_search`
  - semantic lookup via BM25 chunk search over a local gitignore-respecting corpus
- `CodexThread` now exposes:
  - `epiphany_retrieval_state()`
  - `epiphany_retrieve(...)`
  - `epiphany_index(...)`
- app-server protocol now declares experimental `thread/epiphany/retrieve`
- app-server protocol now also declares experimental `thread/epiphany/index`
- app-server message handling now routes those methods through the core host seam
- live hydrated `thread.epiphanyState` reads now backfill a lightweight retrieval summary when the thread already has Epiphany state but no persisted retrieval summary yet
- basic protocol/core/app-server unit tests were added for the new types and mapping layer
- the current verified working tree additionally wires:
  - Qdrant-backed semantic indexing behind an explicit write path instead of hidden retrieval mutation
  - manifest-backed freshness checking under `codex_home`
  - local Ollama embeddings by default, with support for adjacent VoidBot-style env names as fallbacks
  - persistent semantic retrieval preference when the index is fresh, and BM25 fallback when it is stale, missing, or unavailable
- modified Codex alone is no longer the whole Epiphany organ; the heavier implementation now lives in the sibling crate and vendored Codex mostly hosts typed seams and adapters

Important recovery facts:

- existing exact/path lookup substrate already exists through `codex_file_search`
- app-server fuzzy search currently uses it in `vendor/codex/codex-rs/app-server/src/fuzzy_file_search.rs`
- `codex-core` already depends on `bm25`
- `vendor/codex/codex-rs/core/src/tools/handlers/tool_search.rs` already shows how BM25 is used in-tree
- tracked corpus preflight for `vendor/codex/codex-rs` at `vendor/codex` HEAD `d45ab10` is `3642` files / `31.09 MB`
- raw working-tree size under `codex-rs` is polluted by build artifacts and should not be used as the retrieval corpus estimate

Recovered bounded design decision:

- the first Phase 4 slice should be a query-time hybrid retriever
- reuse `codex_file_search` for exact/path-ish lookup
- use workspace-local BM25 chunk search as the initial semantic backend
- keep the first slice loaded-thread-only and read-only
- defer embeddings, vector stores, persistent indexing, and watcher-driven invalidation for later slices

Updated follow-up direction:

- the verified BM25 slice is still the right first landing zone
- the first persistent semantic store after that should target Qdrant rather than another blob-thrashing JSON/Postgres embedding store
- Qdrant should be the preferred persistent semantic backend later, not a hard requirement for basic Epiphany use
- exact/path lookup stays first-class
- BM25 should stay around as bootstrap/fallback/control for users who do not want Qdrant or when Qdrant is unavailable instead of turning the machine into vector-only confidence soup

What got verified:

- `cargo fmt --manifest-path E:\Projects\EpiphanyAgent\epiphany-core\Cargo.toml` passed
- `cargo test --manifest-path E:\Projects\EpiphanyAgent\epiphany-core\Cargo.toml` passed
- `cargo fmt --all` passed
- `cargo test -p codex-core -p codex-app-server-protocol -p codex-app-server --lib --no-run` passed with `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`
- targeted Phase 4 tests passed:
  - `cargo test -p codex-core --lib epiphany`
  - `cargo test -p codex-app-server-protocol --lib thread_epiphany_`
  - `cargo test -p codex-app-server --lib map_epiphany_`
- full `cargo test -p codex-app-server-protocol` passed after regenerating stable schema fixtures

Notable verification fixes:

- `codex_message_processor.rs` needed the live-thread retrieval helper call to stop pretending `&CodexThread` needed `.as_ref()`
- `core/src/session/tests.rs` needed `retrieval: None` in the prompt fixture Epiphany state
- the exact-hit retrieval test fixture needed a genuinely matchable file name (`session_checkpoint.rs`)
- `EpiphanyThreadState` needed an explicit exemption in the protocol export test because sparse durable thread state intentionally uses optional fields

Important schema caveat:

- `cargo run -p codex-app-server-protocol --bin write_schema_fixtures -- --experimental` rewrites the same checked-in schema tree as stable generation
- leaving the repo in that state makes stable protocol tests fail
- the safe checked-in state for this slice is the stable fixture generation output
- after inspection, do not bolt durable Epiphany-state writes onto `thread/epiphany/retrieve` just because the retrieval summary exists:
  - current Epiphany snapshots are persisted on real user-turn boundaries
  - out-of-band durable retrieval writes would need new rollout semantics and are a larger machine than this slice deserves

## What Already Landed

Phase 1 is done.

The Phase 1 internal/dev-usable Epiphany core state slice was implemented inside vendored Codex:

- `RolloutItem::EpiphanyState(EpiphanyStateItem)` exists
- `SessionState` stores `epiphany_state`
- one Epiphany snapshot is persisted per real user turn
- rollout reconstruction restores the latest surviving snapshot across resume, rollback, and compaction
- rollout/state/app-server readers tolerate the new variant
- core persistence/replay tests were added

Main touched files:

- `vendor/codex/codex-rs/protocol/src/protocol.rs`
- `vendor/codex/codex-rs/core/src/state/session.rs`
- `vendor/codex/codex-rs/core/src/session/mod.rs`
- `vendor/codex/codex-rs/core/src/session/rollout_reconstruction.rs`
- `vendor/codex/codex-rs/core/src/session/tests.rs`
- `vendor/codex/codex-rs/core/src/session/rollout_reconstruction_tests.rs`
- plus rollout/state/app-server compatibility readers

Phase 2 is done too.

The Phase 2 prompt-integration slice was originally implemented inside vendored Codex and now renders through the repo-owned prompt crate with a thin wrapper in `codex-core`:

- `EpiphanyStateInstructions` renders a bounded developer fragment from `EpiphanyThreadState`
- the fragment is wrapped in `<epiphany_state> ... </epiphany_state>`
- `Session::build_initial_context` injects it immediately after collaboration-mode instructions when `SessionState.epiphany_state` exists
- resumed sessions now pull the restored Epiphany state back into the prompt path instead of leaving it inert in rollout
- prompt-facing inclusion, omission, resume, bounded-rendering, and snapshot tests were added

Main touched files for Phase 2:

- `epiphany-core/src/prompt.rs`
- `vendor/codex/codex-rs/core/src/context/mod.rs`
- `vendor/codex/codex-rs/core/src/context/epiphany_state_instructions.rs`
- `vendor/codex/codex-rs/core/src/session/mod.rs`
- `vendor/codex/codex-rs/core/src/session/tests.rs`
- `vendor/codex/codex-rs/core/src/session/snapshots/codex_core__session__build_initial_context_epiphany_state.snap`
- `vendor/codex/codex-rs/protocol/src/protocol.rs`

Phase 3 is landed too, in a conservative read-surface form.

What landed:

- app-server protocol `Thread` payloads now expose optional typed `epiphany_state`
- hydrated thread responses prefer live `CodexThread` state when a thread is loaded
- stored thread reads fall back to rollout reconstruction through a shared core helper that respects rollback and compaction semantics
- the minimal typed client surface is now real without making prompt text the GUI data source
- dedicated Epiphany-specific update RPCs and live state notifications are still deferred

Main touched files for Phase 3:

- `epiphany-core/src/rollout.rs`
- `vendor/codex/codex-rs/core/src/epiphany_rollout.rs`
- `vendor/codex/codex-rs/core/src/lib.rs`
- `vendor/codex/codex-rs/core/src/codex_thread.rs`
- `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs`
- `vendor/codex/codex-rs/app-server-protocol/src/protocol/common.rs`
- `vendor/codex/codex-rs/app-server/src/codex_message_processor.rs`
- `vendor/codex/codex-rs/app-server/README.md`
- compatibility fixture/test updates in `analytics`, `exec`, and `tui`

There is now also a dedicated Epiphany delta map:

- `notes/epiphany-current-algorithmic-map.md`

Use it when you need the answer to "how does current Epiphany differ from plain Codex right now?" without rereading the whole Codex machine map plus the whole future-facing spec.

## What Must Be Remembered Before Compaction

If a future session wakes up from compaction and starts bluffing, this is the part to staple to its forehead.

- Phase 1, Phase 2, and the minimal Phase 3 read surface are all landed and verified.
- The current landed Phase 4 anchor is `360dfea`:
  - `Land Epiphany Phase 4 hybrid retrieval slice`
- `vendor/codex` is ordinary tracked repo content, not a submodule.
- Phase 2 means Codex now **reads** Epiphany state during turn construction:
  - `SessionState.epiphany_state` is the internal activation signal
  - `Session::build_initial_context` injects a bounded `<epiphany_state>` developer fragment
  - resumed sessions reuse the restored Epiphany snapshot in the prompt path
- Phase 3 means clients can now **read** typed Epiphany state directly from hydrated `Thread` payloads:
  - `thread/start`, `thread/resume`, `thread/fork`, `thread/read`, `thread/unarchive`, and detached review-thread startup can carry `thread.epiphanyState` when present
  - loaded threads use live state
  - stored thread reads reconstruct from rollout with the same rollback/compaction semantics as core
  - there are still no dedicated Epiphany update RPCs or live `thread/epiphany/*` notifications
- The next phase is **not** GUI work.
- The current landed phase is **Phase 4 slice 1 repo-local hybrid retrieval**:
  - Epiphany now has a real typed repo retrieval subsystem instead of repeated file-by-file shell archaeology
  - the next bounded follow-up is persistent semantic indexing, not proving the retriever exists again
- The current working tree also extracted most Epiphany-owned implementation into `epiphany-core`:
  - prompt rendering, rollout replay, and retrieval/indexing logic are now repo-owned
  - vendored Codex keeps the typed protocol/thread/app-server seam plus thin adapters
- Phase 4 slice 1 is now landed on `main` and was verified before landing:
  - protocol/core/app-server wiring exists and passed targeted verification
  - stable app-server protocol schema fixtures were regenerated
  - `write_schema_fixtures --experimental` is a trap if it is left as the checked-in tree
- The recovered bounded implementation choice for the first slice is:
  - exact/path lookup via existing `codex_file_search`
  - semantic lookup via local BM25 chunk search
  - query-time, read-only, loaded-thread-first behavior
  - no embeddings or persistent vector store yet
- The recovered boundary choice for the current follow-up is:
  - keep the protocol/thread/app-server seam inside vendored Codex
  - move heavy Epiphany-owned implementation into the sibling crate when that can be done without giving up first-class typed integration
- Important Windows verification footgun still stands:
  - use `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex` for `codex-core` work on this machine
- Snapshot hygiene note:
  - the new Epiphany prompt snapshot normalizes temp skill-root paths in the test harness so it stays stable across runs

## Opinionated Workflow Rule

Epiphany should be an **opinionated software development agent**, not a generic chat model wearing a hard hat.

That means this persistence workflow is not optional ceremony. It should be part of the harness:

1. before compaction, phase boundaries, or handoff:
   - sync the canonical map
   - append evidence
   - refresh the handoff note
   - make the next action explicit
2. if context pressure is climbing toward compaction:
   - stop pretending there is plenty of room left
   - narrow the current move to a bounded landing zone
   - persist the checkpoint before the hard limit actually trips
3. compaction should happen from a known checkpoint, not at some random point in the middle of an implementation trance
4. resumed work should start from the persisted checkpoint and next action, not from vibes and transcript archaeology

If the harness cannot do that by default later, it is not opinionated enough yet.

## Verification That Already Happened

Rust and VS Build Tools were installed specifically so this would stop being interpretive dance.

Verified:

- `cargo fmt --all`
- full lib test suites passed for:
  - `codex-protocol`
  - `codex-app-server-protocol`
  - `codex-rollout`
  - `codex-state`
- targeted new `codex-core` Epiphany persistence/replay tests passed
- targeted `codex-core` `epiphany` tests passed after Phase 2
- broader `codex-core` `build_initial_context_*` coverage passed after Phase 2
- `cargo fmt --all` passed again after Phase 3
- `cargo test -p codex-core -p codex-app-server-protocol -p codex-app-server -p codex-analytics -p codex-exec -p codex-tui --lib --no-run` passed after the new `Thread.epiphanyState` field fallout was patched
- targeted Phase 3 tests passed:
  - `cargo test -p codex-core --lib latest_epiphany_state_from_rollout_items`
  - `cargo test -p codex-app-server --lib load_epiphany_state_from_rollout_path_reads_latest_snapshot`
  - `cargo test -p codex-app-server-protocol --lib serialize_client_response`
- `cargo fmt --all` passed for the Phase 4 retrieval slice
- `cargo test -p codex-core -p codex-app-server-protocol -p codex-app-server --lib --no-run` passed for the Phase 4 retrieval slice when `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`
- targeted Phase 4 tests passed:
  - `cargo test -p codex-core --lib retrieve_workspace_`
  - `cargo test -p codex-app-server-protocol --lib thread_epiphany_retrieve`
  - `cargo test -p codex-app-server --lib map_epiphany_retrieve_response_preserves_summary_and_results`
- full `cargo test -p codex-app-server-protocol` passed after restoring stable schema fixtures

Important Windows footgun:

- the `v8` crate blows up on cross-drive builds because it tries to create a Windows symlink during build setup
- to test `codex-core` reliably here, set:
  - `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`

Without that, you get to learn about `symlink_dir failed: ... A required privilege is not held by the client`, which is not the kind of enlightenment we were after.

## Recommended Next Implementation

Do not restart verification from superstition. The current Phase 4 slice is already verified in the working tree.

The durable state seam exists, the turn loop reads it, clients can load typed Epiphany state directly, and the first retrieval slice is now real and landed. The next clean move is the bounded persistent-semantic follow-up after reboot, not pretending the already-shipped slice still needs ceremony.

Current next implementation move:

1. keep the current verified-and-landed query-time hybrid retriever as the Phase 4 slice 1 baseline
2. keep the current verified working-tree `thread/epiphany/index` follow-up bounded:
   - explicit write path
   - Qdrant-backed semantic persistence
   - local Ollama embeddings by default
   - BM25 fallback still alive
3. keep the new `epiphany-core` boundary honest:
   - heavy Epiphany-owned logic in the sibling crate
   - thin adapters and typed seams in vendored Codex
4. next practical move:
   - smoke the explicit indexing path against a live loaded thread and the real local services
   - then decide whether env-scoped backend config is good enough for now or deserves a cleaner surface
5. do not casually add durable retrieval-summary writes to `thread/epiphany/retrieve`

## What Not To Do Next

Not yet:

- GUI work
- watcher-driven semantic invalidation
- automatic observation promotion
- specialist-agent scheduling
- user-facing activation flows

The next slice should stay small and mean:

- harden the explicit persistent semantic retrieval/indexing path
- keep GUI, mutation, and specialist-agent logic dark

## Verification Commands

State sanity from the repo root:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_state.py' status
Get-Content -Tail 5 '.\state\evidence.jsonl'
git status --short
```

Targeted `codex-core` test pattern on this Windows machine:

```powershell
cmd /c "\"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat\" >nul 2>&1 && set CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex && cd /d E:\Projects\EpiphanyAgent\vendor\codex\codex-rs && %USERPROFILE%\.cargo\bin\cargo.exe test -p codex-core --lib epiphany -- --list"
```

## Short Version

The repo is in a good state.

Phase 1, Phase 2, and a minimal Phase 3 typed read surface are landed and verified. `vendor/codex` is first-class in the parent repo now. The next clean move is repo-local hybrid retrieval, not GUI paint and not another architectural detour. Also: pre-compaction persistence is now an explicit design rule, not a lucky habit.
