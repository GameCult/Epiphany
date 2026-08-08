# Fresh workspace handoff

Epiphany remains a supervised engineering alpha. Starfire is the current
cognition and release forge; Yggdrasil is the small live host and is not a build
machine at its present memory budget.

## Authoritative live state

- Branch: `codex/epiphany-shakedown-live`
- Pushed source release commit: `c35272c9e639ba8bfd27143fee26cf8beaccae6a`
- Live workspace: `F:\Projects\.epiphany-runtime\shakedown\live-20260808-v53-hands-precedence`
- Thread: `shakedown-v49-hands-relinquishment-r1`
- Current release: `sha256-201a11149fadf2771c9ff9166d1f9492c9ca25d77b4666a4d10d6934760f0718`
- Release witness: `sha256-c696137e8b6f6bc2e2a4ba217f7aa74c220578b95c9b66d2aa681bef0ac62538`
- Thread-state revision: `49`
- Current action: do not approve another `regatherManually` loop. Preserve v53 evidence, create a labeled supervisor correction boundary for the legacy accepted Modeling result, then replay one compliant typed Modeling-to-Imagination handoff under the exact current release.
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

The manual regather was reviewed as a bounded read-only Eyes action and approved
explicitly. Eyes job `9687af12-9c20-4f86-a949-5c6fe3a73de0` launched at revision
46, completed, and was accepted as
`accept-research-result-worker-9687af12-9c20-4f86-a949-5c6fe3a73de0` with
evidence `ev-research-7d07b100-bec4-4664-9ad9-b62e77fcf103` at revision 47.
Self correctly routed the newer Research boundary once to Modeling. Modeling job
`4e6b6022-73ab-49f1-81c8-03dddfceb29f` completed and was accepted as
`accept-modeling-result-worker-4e6b6022-73ab-49f1-81c8-03dddfceb29f` with
evidence `ev-modeling-aa0076a4-affd-4e98-a4a1-15bad24da0a3` at revision 49.

Both accepted organs say the existing `repo_frontier` family should route one
minimal Imagination design chain. Modeling emitted only generic node/edge
Evolution, however, so no active Imagination frontier exists; the previous
planning lifecycle is terminal. Self therefore falls back to
`regatherManually` again. Do not feed that loop.

The local typed Modeling-to-Imagination cut is implemented and focused-proofed.
For ordinary Modeling, `checkpoint-update-needed` now requires exactly one new
active, unadopted Imagination frontier with empty dependencies, safe scope, and
result-grounded evidence. `checkpoint-ready` and `regather-needed` cannot mutate
frontier; `nextSafeMove` remains display-only. Schema/admission hostile checks,
22 Modeling-focused core tests, all 23 coordinator tests, and all 15 OpenAI
runtime tests pass. Commit, package, and live runtime repair/proof remain.

Exact pushed commit `c35272c9e639ba8bfd27143fee26cf8beaccae6a`
packaged all 21 roles in 2m34s as release
`sha256-201a11149fadf2771c9ff9166d1f9492c9ca25d77b4666a4d10d6934760f0718`
with witness
`sha256-c696137e8b6f6bc2e2a4ba217f7aa74c220578b95c9b66d2aa681bef0ac62538`
and was published to the Starfire local Verse. The operator explicitly grants
standing supervisor authority to repair corrupted Epiphany runtime state
proactively. Every intervention must remain bounded, labeled as supervisor
repair, preserve immutable evidence, and emit receipts. Do not ask permission
again merely because the repair changes runtime state.

Iteration-cost evidence also narrowed. The approved Eyes launch committed in
719ms and completed coordinator launch in 839ms against the same live store.
Generic post-regather Modeling committed in 38.5s; verdict Modeling commits were
43.5-44.5s. The launch defect follows the Modeling carrier/persistence path,
not runtime-store size alone. Model/tool turns also needed roughly 6-10 minutes
under the widened 600-second envelope, separate from package and launch costs.

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
