# Fresh workspace handoff

Epiphany remains a supervised engineering alpha. Starfire is the current
cognition and release forge; Yggdrasil is the small live host and is not a build
machine at its present memory budget.

## Authoritative live state

- Branch: `codex/epiphany-shakedown-live`
- Pushed source release commit: `6f3cdc6168e5bf8b680e141582152b56ef10debe`
- Live workspace: `F:\Projects\.epiphany-runtime\shakedown\live-20260808-v53-hands-precedence`
- Thread: `shakedown-v49-hands-relinquishment-r1`
- Current release: `sha256-d2e7205a2b714c89892ba623b26c3cc4c98ac6b4f54dfb9a0a2a623f0d92402d`
- Release witness: `sha256-399d952e5944450775516e612d447be8b2a106e83399f35e2fb6ffb166d17876`
- Thread-state revision: `34`
- Current action: Modeling retry job `07cdae28-8f11-4cba-a19a-b847d2a4792c` is running with a 600-second budget under PID `17784`.
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
timed out at its 300-second boundary and was reviewed/superseded at revision 33.
Poll retry job `07cdae28-8f11-4cba-a19a-b847d2a4792c` only through an
operator-safe coordinator projection. Accept its first valid result, then
inspect Self's next typed action. Do not recreate Eyes, Imagination, Hands, or
Soul work from this circuit.

Commit and push the Soul-precedence repair and this Mind refresh. After the
current Modeling boundary lands, attack the iteration path:

- exact `6f3cdc61` packaging rebuilt 21 first-party release binaries in 6m59s;
- Modeling launch-context assembly against the roughly 40 MiB runtime store took
  about 90 seconds before the detached worker appeared;
- the stable source/graph cache already exists, so the next cut is first-party
  crate/module fan-out plus projection cost, not another cache wrapper.

Still open: native Persona consequence, retention, crash/restart/session
closure, long-duration resource behavior, timestamp ownership for admitted Mind
decisions, and Linux/Yggdrasil cognition.
