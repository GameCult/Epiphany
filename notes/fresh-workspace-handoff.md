# Fresh Workspace Handoff

## Current authority — 2026-08-07 routeability cut

Exact `58cf8984975c9ae524690ff2032eaa76abf5d0d4` is published as
`sha256-6c2524f1a21fce91f658ee92eb53bbe553719295fa99d7bbe5571651ce5c3c72`
with witness
`sha256-3a674c41e7560239b4437090a950def43331649995b3d51704e1687370e340e9`.
On live v23, packaged Self launched and Mind accepted fresh Modeling results for
the release-auth and Continuity proposals. They created
`frontier-imagination-release-auth-nonbypass-trace-r1` and
`frontier-imagination-continuity-crash-restart-proof-r1`; v23 reached revision
23. Self did not select either for Imagination.

The typed eligibility projection proved the exact blocker: both source scopes
are safe but not in strict lexicographic order. Planning selection already
requires non-empty safe sorted-unique paths. Proposal admission did not, so Mind
accepted state that Self can never route. The local repair adds the same
invariant to proposal admission, publishes per-frontier eligibility in MVP
status, and tells Modeling to normalize proposal scope hints into canonical
sorted-unique scope. The focused negative admission test proves rejection is
byte-preserving; the launch-context test proves the worker sees the contract.

Do not weaken the selector or add retry state. Proposal Evolution is insert-only,
so v23 has no honest typed repair for its two invalid admitted frontiers. Seal it
as evidence after the repair lands. Next: commit and push this pass, package and
publish its exact commit, bootstrap v24, then prove native
Modeling -> Imagination -> dedicated Mind without routing overrides.

## Present condition

Epiphany remains a supervised engineering alpha running cognition and release
builds on Starfire. Yggdrasil remains the small canonical public crossing body;
its current 1.9 GiB RAM does not make it a release forge.

The active pushed branch head is `2dd2534197b38df764b790bd9377374420e01d97`.
The worktree was clean before this state refresh.

Fresh v23 is the current live shakedown Body:

`F:\Projects\.epiphany-runtime\shakedown\live-20260807-v23\runtime.cc`

v23 was bootstrapped from canonical agent state plus a fresh observation of the
current repository Body. It accepted proposal
`shakedown-v23-verdict-identity-preservation-r1`, admitted RepoModel revision 1,
and granted a one-file Hands route for
`epiphany-core/src/coordinator_launch_context.rs`.

Commit `6730e395` now loads the exact current routed frontier item and renders
its identity-bearing fields into verdict-incorporation Modeling context:
`migration_body`, `question`, `target_claim_ids`, `source_scope`,
`dependency_item_ids`, `created_at`, `recommended_next_organ`, `retired_at`,
and `superseded_by`. The worker is told to preserve these exactly while only
status, evidence, gap, and update time remain verdict-owned.

The authenticated `6730e395` release is published at:

`F:\Projects\Epiphany\.epiphany-run\releases\6730e395a32c81b236cfe665f5abe70e4e89ff05\sha256-c75e33e0cf609e0dca5ba7e9a67a68bab0a71d9ff063bc52f41e92d0c6fa59d4`

Soul accepted that consequence with verdict `needs-review`: the implementation
materially closes identity preservation, but the evidence packet did not name
the existing admission-path proof. A second one-file Hands gate therefore
produced commit `2dd25341`. Soul telemetry now names
`runtime_spine::tests::repo_model_incorporates_pass_and_nonpass_soul_verdicts_causally`,
which proves first valid admission, exact route/request/verdict/Modeling-request
bindings, idempotent replay to the same receipt, final frontier disposition,
and no remaining actionable Hands frontier. The targeted test, launch-context
test, and full library suite all pass; full result is 631 passed, 1 ignored.

The exact `2dd25341` release is published at:

`F:\Projects\Epiphany\.epiphany-run\releases\2dd2534197b38df764b790bd9377374420e01d97\sha256-125e80991b94d8ac06c973bce434c3a996ce93bf44c03505be28e702f6132a0e`

Its witness SHA-256 is
`sha256-6e9fe9fa34fe7b442b8b0f7e9ffdc1f914ebaa5accf32154fc23bcd58dc1891e`.
Soul returned `pass` and the coordinator accepted it. The following exact
release Modeling worker produced the first semantically valid incorporation
result; Mind admitted it immediately as `checkpoint-ready`. The review emitted
`roleAccept:modeling`, no `roleAdmissionRejected`, no retry, and no actionable
Hands frontier.

The v23 local Verse required its `epiphany-local` topology to be reseeded after
the ephemeral launch boundary while retaining the authenticated
`epiphany-starfire` release witness. That split advertisement is now repaired
in the v23 store and should be treated as a lifecycle shakedown finding, not as
permission to bypass release authentication.

The next typed proposal,
`shakedown-v23-eyes-release-topology-evidence-r1`, exposed two additional
authority faults. The exact proposal request is
`repo-frontier-proposal-modeling-febeaa48f17a62ae7d5b8658bf5336890300b40e28c7b0415f62a29a1e43526f`.

First, the coordinator appended verdict-incorporation context before honoring
explicit proposal authority, so historical accepted Soul evidence hijacked the
new launch. Commit `b27b8f41` makes proposal authority exclude that historical
context. Its exact authenticated release
`sha256-72695eb146eb6ad0a8b46a66c7cee181dbdf683a9ad68230d2ac5d32fcb312b1`
was published and live-proven: replaying the same proposal request launched a
fresh Modeling worker and Mind accepted its bounded read-only Eyes frontier.

Second, Self projected only actionable Hands frontiers. The admitted Eyes
frontier therefore fell through to `awaitFrontierProposal`. Commit `d849b17e`
adds an admitted, dependency-ready, challenge-aware Eyes frontier signal and
routes it to Research without granting Hands authority. Focused routing,
runtime-spine eligibility, MVP-status, and full library tests pass; full library
result remains 631 passed, 1 ignored.

The exact `d849b17e` release was published as
`sha256-90e31c86ae45683fc762f8c99e1596eb04fab6e2e39a94beb55a6aa3f1685224`
with witness
`sha256-c560200a10f263e0984759ddc846f3422c951f443422625cb071d6d9791508d8`.
Its live no-override coordinator run still returned `awaitFrontierProposal`.
The canonical item was correctly `recommended_next_organ: Eyes`; Self withheld
it because the newer repository Body challenged its target claim. That rule is
correct for Hands and Imagination, but incoherent for Eyes, whose work is to
investigate challenged claims.

Commit `ea475aac` splits the invariant: Hands and Imagination remain blocked by
challenged targets while Eyes remains routeable. Its exact package was published
as `sha256-873027895a81cec207b23002e3902a796e5d1b7c3a11b7c0195412ece258fe63`
with witness
`sha256-e85d92fd1217e6c9837284919c1cee8e1a6a1c59d82ff7c1df3e8b38a4e3307f`.
The package returned `awaitFrontierProposal`, while the same current-tree status
derivation over v23 reports `Hands=false Eyes=true` and `launchResearch`.

Commit `84f00929` conservatively made every target root source-commit-owned and
added canonical Hands/Eyes readiness to native runtime status. Its exact release
`sha256-dc1967df0a6bed550d7fc8c6eb35853dd8f66431b1413174e5d44535b84b36bc`
launched Research from the challenged frontier. Mind accepted the first packet
as `accept-research-result-worker-70e89ac9-b09d-4de6-8524-25891c13d38b`.

The packet grounds the local Verse topology fail-closed boundary and authenticated
commit `2dd25341` via typed Hands receipt. It honestly leaves the non-bypass
repository-body authentication seam open because the hinted `cultmesh.rs` and
`release.rs` paths do not exist. Modeling must locate the actual current owner.

The run also exposed the actual cause of the earlier package/source decision
mismatch: packaged coordinator status called `native_json("epiphany-mvp-status")`,
but that binary is absent from packaged siblings, so Self silently spawned the
ambient debug executable. Commit `148d4527` replaces this with in-process status
derivation and omits auxiliary heartbeat/Persona/Void projections from routing.
Focused tests and a local executable smoke pass.

Exact `148d4527` was published as
`sha256-39ba935db21602bd52b9f0ba7781b65198ed7ad7d3722d72405c8bebbe1ccc3e`
with witness
`sha256-8fdc1f47995b564d5b48f50d1c0e4d9b8582278b3ce108169477c0a829f8092a`.
Its plan-mode artifact proved heartbeat, Persona, and Void projections are
`omitted`, so packaged Self owns routing status in-process. The Eyes-bound
reorient result cited the accepted evidence and frontier exactly and was accepted
as `accept-reorient-result-worker-c8c27852-71ba-4e65-8a34-68396ff7d5f6`.

After acceptance, Self returned manual regather even though Eyes evidence was
newer than the last Modeling boundary. Commit `4e95c42c` added the missing
single-use route. Its exact packaged replay selected `launchModeling`, then
failed before worker launch because generic Modeling context tried to commit an
older Soul frontier request whose model identity no longer matched v23. That is
an authority collision, not a bad Eyes result.

Commit `6f019cf8` separates the causes: accepted Research newer than the last
Modeling boundary receives an explicit Eyes-to-Modeling handoff and cannot enter
historical Soul-verdict incorporation. The focused context test, all 18
coordinator tests, and all 14 packaged coordinator tests pass.

Exact `6f019cf8` was published as
`sha256-e33317e552aef8ed175c768da96e52952da5962f78b030ced50ad064bf1d8adf`
with witness
`sha256-94bc14b1cb6a6d2cb78cd7c7498964448e80a16f562b341f64175b02b55e40ea`.
Packaged Self launched Modeling as runtime job
`4619ec7a-d9af-4b20-8910-bb5eb371b803` from the accepted Eyes handoff without
touching the obsolete Soul frontier request. Mind accepted the first valid
result as
`accept-modeling-result-worker-4619ec7a-d9af-4b20-8910-bb5eb371b803` at state
revision 17 with evidence
`ev-modeling-eb5aa53a-e679-4993-836a-3ac332b467ef`.
`modelingResultAcceptedAfterResearch` is true, and three later coordinator
steps returned manual regather without relaunching Modeling. The single-use
route is consumed.

The accepted result preserved the remaining honest wound: current source still
does not ground a non-bypass release-authentication enforcement seam. Do not
relaunch the sealed Eyes packet merely to repeat that conclusion.

v22 is a preserved routing-deadlock witness: an explicitly selected corrective
proposal could not preempt the failed verdict-incorporation route. Do not grind
or mutate v22.

Eyes, live packaged Imagination, Persona consequence, Continuity crash/restart/closure,
retention, long-duration resource behavior, and Linux/Yggdrasil cognition are
still unproven.

## Next action

Keep v23 live and v22 sealed as the deadlock witness. The local implementation
now gives packaged Self a read-only typed lifecycle projection plus explicit MVP
actions for planning selection, exclusive Imagination launch, Mind review
request/launch, and atomic decision commit. The full adoption timeline,
Hold/Refuse terminal behavior, worker failure projection, all 19 coordinator
tests, and all 14 MVP coordinator tests pass. Commit and package this pass, then
create legitimate live Imagination pressure through the existing proposal and
proposal-Modeling path. Prove the exact package advances v23 through Imagination
and dedicated Mind without overrides or a second planning substrate.

Exact `881b2b1a` was packaged and published as
`sha256-e41a428b97a9e67c18425cf0862f15cb9f1de87d03fd15c641dec7f835f0707c`
with witness
`sha256-56ecec1ce5f23256a25dd6d341f6ff73dcf931fcd590de208d3ab3f220bce365`.
Packaged proposal intake created `shakedown-v23-release-auth-seam-r1` and
selection `repo-frontier-proposal-modeling-2a98b0ad0d4f54bc72618ee8b75e20f347bfe3bdf898d3d7247ba9b89bc244c8`.
The first packaged Self pass exposed a surviving split owner: canonical status
returned manual regather, while the CLI could only rewrite
`awaitFrontierProposal`. No worker launched and v23 remained revision 17.
The current local repair makes pending proposal selection a native status
signal, routes it before regather, and demotes the CLI ID to an equality check.
Debug status over live v23 now derives `launchModeling` and that exact request.
Package and publish the repair before retrying; do not use `881b2b1a` to bypass
its own finding.

Exact `737f2b94` was subsequently packaged and published as
`sha256-b6dfa923a10cab369d56bcbf5d87c2b2da76bc5c3a7f3007ba228f4a63a3564f`
with witness
`sha256-185bd9a3a8657bd9f6cf6a4d158d6368db73363e57ee900ad9cdee265ed6cf82`.
Live v23 proved canonical Self selected `launchModeling` without
`--proposal-modeling-request-id`. The proposal-bound result was not admitted:
it attempted `upsert_frontier` for the already-present
`frontier-eyes-release-topology-evidence-r1`, and Soul superseded the result.
The request is consumed and v23 is revision 19. Do not replay it.

The cause was missing Body context, not weak rejection. Modeling launch context
listed current domains and claims but omitted current frontier items, so the
worker could not know whether a frontier id required insert or revision. The
local repair adds `existingFrontier` to the canonical RepoModel shape and states
the typed contract: upsert only new ids, revise existing ids. The focused
canonical-model launch-context test passes. Commit/package/publish this repair,
then use one fresh legitimate proposal to continue the native lifecycle.

## Immediate re-entry

1. Run `cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-state -- status`.
2. Read `state/map.yaml` and this handoff.
3. Confirm git HEAD, the exact 6f019cf8 published witness above, and whether the
   native Imagination implementation has been committed and packaged.
4. Treat v22 as evidence, not active authority.

Replace this document when state changes. Old attempts belong in evidence and
git, not in the living handoff.
