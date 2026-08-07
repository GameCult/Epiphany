# Fresh Workspace Handoff

## Current authority — 2026-08-07 Hands binding cut

The active branch is `codex/epiphany-shakedown-live`. Exact pushed commit
`4338b0331a374802c649aa2a647a9c39f9a9f92d` is authenticated and published as
`sha256-1023d71c625706694cc67e034172b9f33ea331594924c5e7b786d0818f077a21`
with witness
`sha256-0d351a7001c01a46b46207a9364e6221325d3811cd2cccfd72786430115282be`.
The package contains 21 binaries and exposes no private state.

Fresh v26 lives at
`F:\Projects\.epiphany-runtime\shakedown\live-20260807-v26`. It was cold-started
from the packaged repository-body tool at Git head `4338b033`; only canonical
`agents.cc` crossed from v25. The exact release was published into its fresh
local Verse.

Proposal `shakedown-v26-continuity-imagination-r1` was selected without a
routing override. Packaged Self derived `launchModeling`; job
`6683204c-d2d5-46f2-a518-c355268afcf1` completed and was accepted. Its exact
canonical Imagination frontier became eligible, Self launched Imagination job
`b2c01a84-b006-4483-afd1-1dc4af4cebf1`, and dedicated Mind job
`7daa2630-0ea2-4adf-b17a-24f0c3c7037e` returned `adopt`.

The decision commit admitted the plan and selected the resulting Hands route.
The next gate failed with `repo frontier Hands authority chain violates its full
authority contract`. The validator is correct: `record_hands_implementation_gate`
persisted empty `frontier_route_id`, `plan_candidate_sha256`, and `plan_action`
even though the selected route carried an adopted plan. v26 is failure evidence;
do not mutate or replay it.

The local cut derives those three intent fields from the selected route's
adopted plan. The existing no-plan path remains empty by derivation. The focused
regression test now seeds an adopted plan and proves persisted intent identity;
all 14 coordinator binary tests pass.

## Next action

Commit and push the Hands binding cut, then package and publish that exact
commit. Preserve v26 and cold-start a clean generation. Replay the native path
without overrides and prove:

1. Self derives proposal-bound Modeling and canonical Imagination.
2. Dedicated Mind adopts the typed candidate.
3. Hands intent exactly echoes route id, candidate digest, and plan action.
4. RepoFrontierHandsAuthority binds route, model, plan, intent, review, grant,
   and sorted scope.
5. Execute the bounded Continuity crash/restart/closure proof and capture Soul
   evidence rather than treating the gate as completion.

After that, continue the actual readiness campaign. Persona public consequence,
retention bounds, long-duration resource behavior, and Linux/Yggdrasil cognition
remain open. Yggdrasil is the small canonical public crossing Body; Starfire
remains the temporary cognition and release forge until measured runtime demand
justifies resizing Yggdrasil.
