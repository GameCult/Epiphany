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
- `notes/epiphany-core-harness-surfaces.md`

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

## What Must Be Remembered Before Compaction

If a future session wakes up from compaction and starts bluffing, this is the part to staple to its forehead.

- Phase 1, Phase 2, and the minimal Phase 3 read surface are all landed and verified.
- The latest pushed implementation anchor before this handoff sync is `640e063`:
  - `Document pre-compaction Epiphany workflow`
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
2. compaction should happen from a known checkpoint, not at some random point in the middle of an implementation trance
3. resumed work should start from the persisted checkpoint and next action, not from vibes and transcript archaeology

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

Important Windows footgun:

- the `v8` crate blows up on cross-drive builds because it tries to create a Windows symlink during build setup
- to test `codex-core` reliably here, set:
  - `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`

Without that, you get to learn about `symlink_dir failed: ... A required privilege is not held by the client`, which is not the kind of enlightenment we were after.

## Recommended Next Implementation

Do **Phase 4 repo-local hybrid retrieval** next.

The durable state seam exists, the turn loop now reads it, and clients can now load typed Epiphany state directly. The next clean move is to stop making every future agent rediscover the repo with shell spelunking.

Planned slice:

1. define retrieval state and shard/index summaries in typed Epiphany state
2. add a bounded hybrid retrieval surface that combines exact search with semantic chunk lookup
3. keep the first retrieval slice additive and internal/dev-usable first
4. leave watcher-driven invalidation, heavy GUI reflection, and specialist-agent scheduling for later slices

Main files for the next slice:

- `vendor/codex/codex-rs/protocol/src/`
- `vendor/codex/codex-rs/core/src/`
- `vendor/codex/codex-rs/app-server/src/`
- `vendor/codex/codex-rs/app-server-protocol/src/`

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
