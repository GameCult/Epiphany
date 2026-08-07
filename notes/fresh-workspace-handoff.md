# Fresh Workspace Handoff

## Current authority — 2026-08-07

The active branch is `codex/epiphany-shakedown-live`. Exact pushed commit
`49233e9c155199355f1729e0a19a9778e947e9f2` packages all 21 shipped binaries as
`sha256-d24a05f98b3f653f904b425d87f93668e944db8e7975ce1c0564f5ef55ace7db`
with witness
`sha256-46181cce86b725a237eb1c82e5b68be2f9d01e3afa9c5e82cef67dfe3053b751`.
Independent inspection accepts the exact commit, runtime, binary set, bytes,
release identity, and witness; private state is absent.

Release construction has one Cargo graph owner. Its cache identity is the root
`Cargo.lock` bytes, target triple, and toolchain fingerprint. One exclusive
graph lock covers build through binary copy. Source materialization has a
separate repository-bound persistent detached worktree and exclusive lock.
Each package run force-checks the exact commit, cleans the main worktree and
recursive submodules, and verifies owner, HEAD, status, and submodule identity
with Windows long-path support before Cargo starts. Commit identity remains
release authority; neither cache is allowed to claim it.

Measured build boundaries:

- A cold unified graph built in 15m43s.
- Reusing dependencies across a random checkout path took 8m53s because every
  local/path crate received a new absolute source path.
- The persistent exact-source checkout populated those local artifacts in
  9m17s. This is setup cost, not yet the steady-state result.
- The next non-code commit must package through the same source and graph cache.
  That cross-commit timing is the acceptance proof for the optimization.

The prior live runtime is sealed at `live-20260807-v30`. Its model-only thread
proved the source gap: job opening and failed/cancelled snapshot interpretation
exist, but the live Body still lacks a proven automatic terminal-worker writer
to completed Reorientation/Verification closure. That thread exhausted its
authority and must not receive another manual-regather consent.

## Next action

1. Package the current boundary successor through the existing stable caches;
   authenticate its release and record the steady-state timing.
2. Publish only that exact authenticated release into a fresh v31 runtime with
   physically separate runtime, Verse, coverage, repository-Body, and release
   stores. Carry only canonical `agents.cc` forward.
3. Open a new typed implementation objective for rupture-to-closure. Prove
   proposal-bound Modeling, canonical Imagination, dedicated Mind adoption,
   exact adopted-plan Hands authority, one deliberate bounded-worker failure,
   typed terminalization, restart/recovery, and Soul-verifiable closure.
4. Continue the readiness campaign through Persona public consequence,
   retention bounds, endurance/resource plateau, and Linux/Yggdrasil cognition.

Do not read raw worker transcripts, mutate sealed runtimes, approve manual
regather on v30, or start concurrent Cargo against the release cache.
