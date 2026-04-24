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

Recent anchor commits before the current in-flight work:

- `2042687e3035c5a86d7f6aa66306d87abcc10f2d` - vendored Codex and landed the Phase 1 Epiphany core state slice
- `c823815` - persisted the Phase 2 implementation plan and refreshed the project handoff/state
- `efd1420` - landed and pushed the Phase 2 prompt-integration slice
- `640e063` - documented the pre-compaction Epiphany workflow and synced the repo memory before compaction

Do not trust this note for the exact current HEAD; use `git log --oneline -1` if you need the live commit.

Canonical project state still lives in:

- `state/map.yaml`
- `state/scratch.md`
- `state/evidence.jsonl`

Current implementation plan note:

- `notes/codex-epiphany-mode-plan.md`

Current architectural/spec notes:

- `notes/codex-repository-algorithmic-map.md`
- `notes/epiphany-current-algorithmic-map.md`
- `notes/epiphany-core-harness-surfaces.md`

## Current In-Flight Work

Phase 4 repo-local hybrid retrieval is now verified in the working tree. It is still uncommitted, but it is no longer a vague scaffold or a maybe.

Current working-tree implementation spans:

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

What is wired right now:

- `EpiphanyThreadState` already has retrieval metadata fields and types in protocol
- `codex-core` now has a new `epiphany_retrieval.rs` module
- the core retriever is query-time and hybrid:
  - exact/path-ish lookup via `codex_file_search`
  - semantic lookup via BM25 chunk search over a local gitignore-respecting corpus
- `CodexThread` now exposes:
  - `epiphany_retrieval_state()`
  - `epiphany_retrieve(...)`
- app-server protocol now declares experimental `thread/epiphany/retrieve`
- app-server message handling now routes that method to the core retriever
- live hydrated `thread.epiphanyState` reads now backfill a lightweight retrieval summary when the thread already has Epiphany state but no persisted retrieval summary yet
- basic protocol/core/app-server unit tests were added for the new types and mapping layer

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

- `cargo fmt --all` passed
- `cargo test -p codex-core -p codex-app-server-protocol -p codex-app-server --lib --no-run` passed with `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`
- targeted Phase 4 tests passed:
  - `cargo test -p codex-core --lib retrieve_workspace_`
  - `cargo test -p codex-app-server-protocol --lib thread_epiphany_retrieve`
  - `cargo test -p codex-app-server --lib map_epiphany_retrieve_response_preserves_summary_and_results`
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

The Phase 2 prompt-integration slice was implemented inside vendored Codex:

- `EpiphanyStateInstructions` renders a bounded developer fragment from `EpiphanyThreadState`
- the fragment is wrapped in `<epiphany_state> ... </epiphany_state>`
- `Session::build_initial_context` injects it immediately after collaboration-mode instructions when `SessionState.epiphany_state` exists
- resumed sessions now pull the restored Epiphany state back into the prompt path instead of leaving it inert in rollout
- prompt-facing inclusion, omission, resume, bounded-rendering, and snapshot tests were added

Main touched files for Phase 2:

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
- The latest local implementation anchor before the current in-flight retrieval slice is `d45ab10`:
  - `Add Epiphany delta algorithmic map`
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
- The next phase is **Phase 4 repo-local hybrid retrieval**:
  - give Epiphany a real repo retrieval subsystem instead of repeated file-by-file shell archaeology
  - keep it typed, additive, and GUI-friendly
- Phase 4 slice 1 is now verified in the working tree:
  - protocol/core/app-server wiring exists and passes targeted verification
  - stable app-server protocol schema fixtures were regenerated
  - `write_schema_fixtures --experimental` is a trap if it is left as the checked-in tree
- The recovered bounded implementation choice for the first slice is:
  - exact/path lookup via existing `codex_file_search`
  - semantic lookup via local BM25 chunk search
  - query-time, read-only, loaded-thread-first behavior
  - no embeddings or persistent vector store yet
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

The durable state seam exists, the turn loop reads it, clients can load typed Epiphany state directly, and the first retrieval slice is now real. The next clean move is to ship that verified slice cleanly instead of widening it before Qdrant is even online.

Current next implementation move:

1. clean up, stage, commit, and push the current verified query-time hybrid retriever as the Phase 4 slice 1 landing zone
2. do not casually add durable retrieval-summary writes to `thread/epiphany/retrieve`
3. if a follow-up is needed later, keep it narrow:
   - retrieval persistence/freshness semantics
   - probably via a dedicated Epiphany update path
   - Qdrant-backed persistent semantic indexing as the preferred backend, with BM25 fallback for no-Qdrant or degraded mode
   - not GUI work
   - not watcher-driven invalidation

## What Not To Do Next

Not yet:

- GUI work
- watcher-driven semantic invalidation
- automatic observation promotion
- specialist-agent scheduling
- user-facing activation flows

The next slice should stay small and mean:

- add typed retrieval scaffolding
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
