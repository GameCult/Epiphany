# Fresh Workspace Handoff

## What This Repo Is

`EpiphanyAgent` is a prototype workspace for upgrading AI coding harnesses so they stop confusing local motion with global understanding.

The working idea is simple enough:

- keep canonical understanding in typed state
- keep scratch bounded and disposable
- keep evidence durable
- make the harness consult that state instead of pretending the transcript is a brain

The project target is now the vendored Codex core harness itself, not a sidecar wrapper first and not some random GitHub “OpenCodex” contraption held together with hope and string.

## Current Repo State

Important consequence:

- `vendor/codex` is now tracked directly in the parent repo as ordinary files
- it is **not** a submodule anymore

Recent anchor commits before the current in-flight work:

- `2042687e3035c5a86d7f6aa66306d87abcc10f2d` - vendored Codex and landed the Phase 1 Epiphany core state slice
- `c823815` - persisted the Phase 2 implementation plan and refreshed the project handoff/state

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

Phase 2 is now done too.

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

Important Windows footgun:

- the `v8` crate blows up on cross-drive builds because it tries to create a Windows symlink during build setup
- to test `codex-core` reliably here, set:
  - `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`

Without that, you get to learn about `symlink_dir failed: ... A required privilege is not held by the client`, which is not the kind of enlightenment we were after.

## Recommended Next Implementation

Do **Phase 3 typed state exposure** next.

The durable state seam exists and the turn loop now reads it. The next clean move is to expose that state to clients without forcing GUI code to scrape prompt text.

Planned slice:

1. add typed app-server/protocol read surfaces for Epiphany thread state
2. keep the new surface additive and internal/dev-usable first
3. avoid making transcript text the canonical source for GUI state
4. leave retrieval, invalidation, and specialist-agent scheduling for later slices

Main files for the next slice:

- `vendor/codex/codex-rs/app-server/src/`
- `vendor/codex/codex-rs/app-server-protocol/src/`
- `vendor/codex/codex-rs/protocol/src/`
- `vendor/codex/codex-rs/core/src/session/`

## What Not To Do Next

Not yet:

- GUI work
- protocol event expansion
- retrieval indexing
- automatic observation promotion
- specialist-agent scheduling
- user-facing activation flows

The next slice should stay small and mean:

- expose typed state reads
- keep GUI, retrieval, and mutation logic dark

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

Phase 1 and Phase 2 are landed and verified. `vendor/codex` is first-class in the parent repo now. The next clean move is typed state exposure for clients, not GUI paint and not another architectural detour.
