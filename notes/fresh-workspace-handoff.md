# Fresh workspace handoff

Epiphany remains a supervised engineering alpha. Starfire is the current
cognition and release forge; Yggdrasil is the small live host and is not a build
machine at its present memory budget.

## Authoritative live state

- Branch: `codex/epiphany-shakedown-live`
- Pushed source release commit: `5a3532b996af4aecb1ed00de3747dd1ac7a1e053`
- Live workspace: `F:\Projects\.epiphany-runtime\shakedown\live-20260808-v53-hands-precedence`
- Thread: `shakedown-v49-hands-relinquishment-r1`
- Current release: `sha256-7d1040b8cc01429d12d30780af3ac43f46e23382fff6a3267369b403d16e4387`
- Release witness: `sha256-48202410ca55be72e0444f0d29501644608fb4b94953c9f901763dc7f6547aa6`
- Thread-state revision: `45`
- Current action: Self consumed the accepted verdict-Modeling boundary and now derives `regatherManually`, targeting Eyes. Review that typed request before granting manual regather; do not recreate the completed Hands, Soul, or Modeling work.
- Sealed evidence: v49 through v52. Do not mutate them.

## Live causal boundary

RepoModel revision 6 owns the active frontier. Mind admitted one local
supervisor execution amendment as
`repo-frontier-execution-amendment-d0d1a164c03f1298681a70b332ac3ddee423f8ab3e07b91d8dc3d74833a65875`.
It preserves the original plan and route while replacing the non-executable
planning sentence with the exact verification command. The admission is
user-authorized and packet-hash bound; it is not a Bifrost-signed remote
command.

The amended route produced a complete typed Hands consequence against commit
`0763fcbe94257eb9adbe92ab814b5a487f65551d`. Exact release `sha256-d2e7205a...`
resumed the original immutable Soul job
`8eee52bf-d96e-4609-923d-3080e644ca46`, which the prior release could not decode
after the amendment schema was introduced. Coordinator acceptance consumed the
valid Soul result and advanced thread state to revision 31.

Self then exposed two precedence faults and both are repaired locally:

1. `Pending|Running` Soul now preempts generic CRRC/regather Hands routing.
2. Accepted Soul consequences now resolve before generic CRRC/regather Hands
   routing, so a pass launches Modeling rather than minting another Hands gate.

All 23 focused coordinator routing tests pass. Operator-safe live proof is under
`.epiphany-dogfood/v53-amended-soul-r42-poll`,
`.epiphany-dogfood/v53-amended-soul-accept-r43`,
`.epiphany-dogfood/v53-post-soul-r45`, and
`.epiphany-dogfood/v53-post-soul-modeling-r46`.

The `v53-amended-soul-r41` supervisor poll is contaminated for normal dogfood
supervision because the operator tailed the structured worker stdout while
checking completion. It did not steer acceptance; use the coordinator
projections above as authoritative evidence.

## Next action

The first post-Soul Modeling job `8851bbcc-d013-4e5a-95bf-d69b4833dc29`
timed out at its 300-second boundary and was reviewed/superseded. Its retry
completed but received the older Eyes handoff because launch-context selection
checked Research freshness before the newer accepted Soul boundary. Mind
correctly rejected its resulting Evolution patch and the coordinator superseded
it at revision 35.

Launch context now lets the newest accepted organ boundary own the Modeling
brief. It also reuses the RepoModel snapshot already loaded for dynamic memory
context instead of reopening the whole runtime store to render model shape.
All seven launch-context tests and the native coordinator seam test pass.

Job `9d7cd7d8-2d43-4165-b3c5-54381126579e` completed from the correct Soul
boundary, but the static worker-output schema still allowed it to omit the
frontier request identity and emit ordinary Evolution. Mind rejected it with
`Evolution cannot bypass a current route or own verdict-driven frontier
lifecycle` and the coordinator superseded it through
`role-failure-review-962c3f59-fdce-4558-8d40-1448230554d5` at revision 56.
This repeated the prior semantic failure despite corrected context, proving
that prompt context was not the owning seam.

The live repair carries `repo_frontier_modeling_request_id` in the typed worker
launch request. The model runtime uses that field to expose a specialized
schema requiring the exact request id, `incorporate_frontier_verdict`, non-empty
evidence, and exactly one `revise_frontier` operation. Generic Evolution is no
longer representable for that launch; Mind remains final admission authority.
Finish focused tests, commit and push, package one authenticated release, then
relaunch the superseded post-Soul Modeling request without overrides. Do not
recreate Eyes, Imagination, Hands, or Soul work from this circuit.

Exact commit `61e8be38a3d194b70a4f469e7524653ecab590a8` packaged in 6m50s as
release `sha256-67ed6c8d...` with witness `sha256-c98e36f3...`. Live launch
`57272b34-8148-4668-903e-00cfe5fabfdd` proved that Evolution was no longer
representable, but Mind rejected its frontier revision because the specialized
schema still inherited an item shape that omitted `adopted_plan`; output thus
altered adopted execution anatomy. The coordinator superseded it and launched
retry `a89fe1ea-a88b-4108-a299-c99ca9b65fb7`, which completed. Review without
automatic supersession failed closed with `generic frontier revision cannot
alter adopted execution anatomy or own plan adoption`.

The repair now persists one typed verdict-Modeling launch authority containing
the exact `RepoFrontierModelingRequest` and routed `RepoFrontierItem`. Provider
schema generation const-binds every identity-bearing field, including the full
adopted plan; only verdict-owned status, evidence refs, gap, and updated_at may
vary. Persistence rejects missing, orphaned, mismatched, or non-Modeling
authority. The focused schema and Soul-to-Modeling context tests pass, as do all
15 model-runtime and all 15 coordinator tests. The completed retry remains
immutable invalid evidence; commit and package this exact repair, supersede the
old result explicitly, then pay for one corrected model turn.

Exact commit `5a3532b9` packaged 21 binaries in 5m57s as release
`sha256-7d1040b8...` with witness `sha256-48202410...`. Invalid result `a89fe1ea...`
was superseded by `role-failure-review-1d365571...`. Corrected job
`ede197eb...` proved the typed live launch but timed out at 300 seconds and was
superseded by `role-failure-review-d8e10919...`. Retry
`6362b698-0566-4377-9c14-2d6580436a92` completed under a 600-second envelope.
Mind accepted it as `accept-modeling-result-worker-6362b698-0566-4377-9c14-2d6580436a92`
with evidence `ev-modeling-cb536bd5-eca9-465f-af33-4f0c46080ab6`, advancing
thread state to revision 45. Coordinator projection r69 proves
`modelingResultAcceptedAfterResearch: true` and derives `regatherManually` for
Eyes; the causal Modeling route is consumed.

After the Modeling boundary lands, continue attacking the iteration path:

- exact `6f3cdc61` packaging rebuilt 21 first-party release binaries in 6m59s;
- Modeling launch-context assembly against the roughly 40 MiB runtime store
  still took about 48 seconds before the detached worker appeared and about 83
  seconds before the coordinator finished, even after removing one redundant
  RepoModel pull;
- the stable source/graph cache already exists, so the next cut is first-party
  crate/module fan-out plus projection cost, not another cache wrapper.

Still open: native Persona consequence, retention, crash/restart/session
closure, long-duration resource behavior, timestamp ownership for admitted Mind
decisions, and Linux/Yggdrasil cognition.
