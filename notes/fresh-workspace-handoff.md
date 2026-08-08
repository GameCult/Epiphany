# Fresh Workspace Handoff

## Live boundary — 2026-08-08

The active branch is `codex/epiphany-shakedown-live`; pushed state is
`ced86e7e`. Exact cognition source `ecfff489fe7e46f6ddcc9d58f0723411e8d94c13`
is packaged as
`sha256-6e5cd9827ab6a659c8a29f1147daf8ebec52ea3d148a16905a1401c539059d79`
with witness
`sha256-8d53746348eecf2afd2a5def5539dfb544e260104fb33e3e7d0eec83573ce3dc`.
Starfire remains the cognition and release forge; Yggdrasil remains the small
public runtime body.

Fresh runtime
`F:\Projects\.epiphany-runtime\shakedown\live-20260808-v44-clean-planning`
proved the native proposal Modeling -> Imagination -> dedicated Mind planning
circuit without overrides. Mind adopted candidate
`repo-frontier-plan-candidate-700b1404f73888840ea5981be736ad3b2406e72ec4ee7eba7f763d9db03e498c`
and Self derived Hands route
`repo-frontier-route-86b9702dd4ad3eedae2693e8af7d0b547fe00885949fa2646b3f3b6a47a6225d`.
The operator-safe gate summary is
`.epiphany-dogfood/v44-clean-planning-r5/coordinator-summary.json`.

The adopted plan cannot be executed truthfully. Its path ceiling omits
`epiphany-core/src/bin/epiphany-mvp-status.rs`, the composition owner that must
project current RepoModel admission into Reorientation and CRRC. The existing
coordinator and CRRC inputs do not independently observe that admission. No
source was changed outside the gate.

This exposes a route-lifecycle defect: an adopted Hands route has no typed
refusal, relinquish, or supersession transition when implementation inspection
falsifies its plan. `epiphany.hands.action_refusal_receipt` is named in the
contract catalog but has no persisted type, writer, CLI command, or route
terminalization semantics. The only implemented terminal path requires a
patch, command, commit, verification request, Soul verdict, and verdict
incorporation. Do not fabricate that consequence chain or mutate the v44 store
by hand.

## Active work

The exact root release bundle has now proved release incremental compilation:
its fresh-target build took 1226.69 seconds, then a touched-core rebuild took
31.47 seconds versus the 7m04s baseline (about 13.5x faster). Evidence is in
`.epiphany-run/build-benchmarks/incremental-root-20260808/result.json`.

The independent incremental-plus-`rust-lld` trial completed at 1097.14 seconds
cold and 29.34 seconds after touching core. Its evidence is under
`.epiphany-run/build-benchmarks/lld-incremental-root-20260808`. The linker saved
only 2.13 seconds (about 6.8 percent) on the iteration path and required a
toolchain-specific Windows linker location, so it is not repository policy.
Root `[profile.release] incremental = true` is the adopted portable change.
This build-configuration edit is an explicit operator-requested supervisor
intervention, not evidence that the sealed v44 Hands route performed it.

After the build benchmark, first give adopted Hands work a typed
refusal/relinquish path that preserves Mind's plan authority, records why Hands
cannot proceed, and returns frontier ownership to a lawful replanning state.
Then replay v44, reject the incomplete plan through that path, adopt a corrected
plan including the status-composition owner, and prove CRRC -> accepted
Reorientation -> Continuity recovery -> Soul closure.

Persona consequence, bounded retention, crash/restart closure, endurance and
resource plateau, and Linux/Yggdrasil cognition remain open. v22 and earlier
shakedown scars remain in git and the typed evidence ledger; they are not active
rehydration state.
