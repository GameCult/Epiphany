# Fresh Workspace Handoff

## Current authority — 2026-08-07 v28 Continuity fault

The active branch is `codex/epiphany-shakedown-live`. Exact pushed commit
`17d4ae47ca6faa1613707010d19261a75a6b74ee` is authenticated and published as
`sha256-0afab25ce4a30895f8183966a709786c51230c11acc35dd6eb3e83ae0bb79fe4`
with witness
`sha256-2e9e8a53659e7351ca6d9b297908a35c54dde3047a654fe61f2d7a8b3d8b186c`.
The package contains 21 binaries and exposes no private state. Starfire remains
the release forge; Yggdrasil remains the small runtime target.

Fresh v27 lives at
`F:\Projects\.epiphany-runtime\shakedown\live-20260807-v27`. It was cold-started
from the packaged repository-body tool; only canonical `agents.cc` crossed from
v26. Its local Verse first proved fail-closure while unseeded, then was seeded
for `repo:F:/Projects/Epiphany` and received the exact release publication.

Generic user proposal `shakedown-v27-hands-continuity-r1` was selected without a
routing override. Packaged Self derived `launchModeling`; job
`ee6cdaf1-28d4-4ffa-8c9e-171f822524c8` completed and the first result was
accepted as `accept-modeling-result-worker-ee6cdaf1-28d4-4ffa-8c9e-171f822524c8`.
The result was source-grounded in the repaired coordinator seam but inspected
only `epiphany-mvp-coordinator.rs`. It created an active frontier recommending
Hands. Self therefore emitted ready route
`repo-frontier-route-2a6f1480b9fa431e26dee6db5290b3b234d7bbb541947737d08dcb2e9f0cb48e`.

v27 remains the generic proposal contract boundary. Fresh v28 lives at
`F:\Projects\.epiphany-runtime\shakedown\live-20260807-v28`. It cold-started
through the packaged repository-body surface, with physically separate local
Verse and release stores. Resident Self started only after receiving the raw
Bifrost trust-anchor format expected by feedback ingress.

Proposal `shakedown-v28-live-model-admission-r1` launched Modeling job
`8b35ea3f-55dc-4dd2-8c4a-420b5da374e3`; Mind accepted its first result as
`accept-modeling-result-worker-8b35ea3f-55dc-4dd2-8c4a-420b5da374e3`. Self
selected canonical Imagination and launched job
`b34d21de-6788-4935-8910-75275c257444`, PID `11132`.

PID `11132` exited without a typed result. Repeated coordinator passes continue
to project `imaginationRunning`. The source cause is in
`epiphany-openai-runtime/src/bin/epiphany-openai-runtime.rs`: with
`--max-runtime-seconds` present, `Ok(result) => result?` propagated ordinary
worker errors out of the process without calling `fail_worker_for_runtime_error`.
The unbounded branch already sealed the same error correctly.

Exact pushed commit `b8c76fd0d955e4d540e9caf2b73a2abbf784dc14` routes bounded
and unbounded completion through `seal_worker_runtime_result`. Success remains
success; every ordinary error
writes the typed failed outer worker and inner OpenAI job before exit. All six
model-runtime tests and all fourteen coordinator tests pass. It packaged 21
binaries as `sha256-f5a78dc6153edadb25b00717cee0c42df58663cdf7dde1fcd21354e87cb6b763`
with witness `sha256-7a0b060f3351cdb9dc0be09a903fd09d9e437fda89eea51ab88d8bafcb91f9e5`.
It has not been published into v28; v28 is sealed failure evidence and must not
be repaired or mutated manually.

That package run exposed a release-forge ownership fault. One exact commit was
split across manifest-specific Cargo target directories, so `epiphany-core`,
`epiphany-openai-runtime`, and `epiphany-tool-mcp-runtime` rebuilt overlapping
dependency graphs independently. The third manifest alone took 8m45s. The
working cut gives the exact commit one shared target root while retaining
separate lockfile validation, sequential builds, detached clean source, and
byte-level release witnessing. All fourteen packaged-release tests pass.

## Next action

Commit and push the release build-graph collapse, package that exact commit,
and compare elapsed time against the three-root b8c76fd0 baseline. Publish the
result only into a clean successor generation. Without overrides, prove:

1. Self derives proposal-bound Modeling and canonical Imagination.
2. Self selects and launches canonical Imagination.
3. Dedicated Mind adopts the typed candidate.
4. Hands intent exactly echoes route id, candidate digest, and plan action.
5. RepoFrontierHandsAuthority binds route, model, plan, intent, review, grant,
   and sorted scope.
6. Deliberately fail one bounded worker and prove typed terminal failure,
   restart, and Soul-verifiable closure rather than a permanent running state.

Do not add a global rule forcing generic user proposals through Imagination;
that would destroy their valid direct Hands/Eyes semantics to repair a test
fixture. Use the existing owner-aligned autonomous crossing.

After that, continue the actual readiness campaign. Persona public consequence,
retention bounds, long-duration resource behavior, and Linux/Yggdrasil cognition
remain open. Yggdrasil is the small canonical public crossing Body; Starfire
remains the temporary cognition and release forge until measured runtime demand
justifies resizing Yggdrasil.
