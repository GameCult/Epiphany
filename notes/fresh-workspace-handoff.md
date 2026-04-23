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

Latest pushed parent commit:

- `2042687e3035c5a86d7f6aa66306d87abcc10f2d`

Important consequence:

- `vendor/codex` is now tracked directly in the parent repo as ordinary files
- it is **not** a submodule anymore

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

Important Windows footgun:

- the `v8` crate blows up on cross-drive builds because it tries to create a Windows symlink during build setup
- to test `codex-core` reliably here, set:
  - `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`

Without that, you get to learn about `symlink_dir failed: ... A required privilege is not held by the client`, which is not the kind of enlightenment we were after.

## Recommended Next Implementation

Do **Phase 2 prompt integration** next.

The durable state seam exists. Now Codex needs to actually read it during turns.

Planned slice:

1. add a dedicated `EpiphanyStateInstructions` developer-context fragment
2. render a compact, bounded summary from `EpiphanyThreadState`
3. inject it from `Session::build_initial_context`
4. activate it only when `SessionState.epiphany_state` is present
5. add prompt-facing tests and one snapshot

Main files for the next slice:

- `vendor/codex/codex-rs/core/src/context/mod.rs`
- `vendor/codex/codex-rs/core/src/context/epiphany_state_instructions.rs` (new)
- `vendor/codex/codex-rs/core/src/session/mod.rs`
- `vendor/codex/codex-rs/core/src/session/tests.rs`
- `vendor/codex/codex-rs/core/src/session/snapshots/`

## What Not To Do Next

Not yet:

- GUI work
- protocol event expansion
- retrieval indexing
- automatic observation promotion
- specialist-agent scheduling
- user-facing activation flows

The next slice should be small and mean:

- make the state matter to prompts
- keep the rest dark

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

Phase 1 is landed, verified, committed, and pushed. `vendor/codex` is first-class in the parent repo now. The next clean move is Phase 2 prompt integration, not GUI paint and not another architectural detour.
