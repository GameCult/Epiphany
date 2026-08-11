# Scratch

Disposable working memory for one bounded rite.

## Current Subgoal

Restore one owner for release incremental policy and measure the exact
24-binary construction path without weakening witness authority.

## Current mechanism

The root release-bundle manifest declares `[profile.release]
incremental = true`, but `release_build_command` force-sets
`CARGO_INCREMENTAL=0`. The construction helper therefore overrides the profile
owner and every `epiphany-core` change recompiles the broad crate and relinks the
24-binary graph. Exact `acd91e6e` took 5m02s despite an otherwise warm graph.

## Authority map

- Owner: the root release-bundle Cargo profile owns optimization and
  incremental policy for the one exact Cargo graph.
- Inputs: exact source commit, frozen lockfile, target triple, toolchain
  fingerprint, stable source worktree, and stable graph cache.
- Outputs: all 24 witnessed release binaries and one exact packaged-release
  witness.
- Derived state: incremental objects and fingerprints are cache mechanics, not
  release identity or publication authority.
- Forbidden writers: release construction may select the exact manifest,
  target, feature set, Cargo home, and target directory, but may not silently
  override manifest profile policy. Cache contents cannot certify a release.
- Shared paths: migration build, identical replay, and later source-changing
  warm builds use the same `release_build_command` and witness inspection.
- Cut line: remove the construction-owned `CARGO_INCREMENTAL=0` override; do
  not add a weaker shakedown package, second build graph, or cache-derived
  trust path.
- Verification: command-shape test proves the override is absent; focused and
  full construction tests pass; exact Linux migration package authenticates
  24 binaries; identical replay reproduces release and witness; the next real
  core-source successor must materially beat the 5m02s baseline while
  retaining exact witness inspection.
