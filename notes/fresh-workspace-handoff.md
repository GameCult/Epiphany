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
Exact commit `388c49bca8ece81b8ad1d41ca075a3bea9654d67` packaged 21 binaries as
`sha256-1c63d13fd18b24492376c87425f9c68859075e7032a73566823396ae4e31cb06`
with witness
`sha256-01b526d6ef87be91995e3c5ab5d053b4e6a840132d2a0612436096f31268d023`.
Exact inspection and CultMesh catalog publication passed. The release is not
active; resident daemons retain their prior pinned release.

After the build benchmark, first give adopted Hands work a typed
refusal/relinquish path that preserves Mind's plan authority, records why Hands
cannot proceed, and returns frontier ownership to a lawful replanning state.
Then replay v44, reject the incomplete plan through that path, adopt a corrected
plan including the status-composition owner, and prove CRRC -> accepted
Reorientation -> Continuity recovery -> Soul closure.

Fresh runtime
`F:\Projects\.epiphany-runtime\shakedown\live-20260808-v45-hands-relinquishment`
was copied from the pre-planning v42 boundary; v44 remains sealed. Typed
proposal `operator-hands-route-relinquishment-v1` was selected as Modeling
request
`repo-frontier-proposal-modeling-5c0ce91b6d30d16c31087088ac613f893a4b65f430cc6fb7fe20faa1bfccde6a`
with payload digest
`01e6bdc556e7cad6dbfcae93ad5f82fd57794af11b3dafb39ecb3747a19f6dfb`.
One bounded active-release coordinator step launched the proposal-bound
Modeling worker. Its typed result completed reviewable and proposal-bound, but
Mind correctly refused acceptance with `Evolution cannot bypass a current
route or own verdict-driven frontier lifecycle`. Copying v42 was therefore not
a route-free planning boundary: it retained a current-model route even though
its operator action was `awaitFrontierProposal`. Preserve v45 and artifacts in
`.epiphany-dogfood/v45-hands-relinquishment-r2`/`r3` as negative evidence; do
not weaken the guard or replay the consumed result. Reconstruct the genuinely
clean boundary used before v44, then resubmit the same typed proposal.

Persona consequence, bounded retention, crash/restart closure, endurance and
resource plateau, and Linux/Yggdrasil cognition remain open. v22 and earlier
shakedown scars remain in git and the typed evidence ledger; they are not active
rehydration state.

## Route-free recovery and planning failure — v46/v47

Fresh v46 at
`F:\Projects\.epiphany-runtime\shakedown\live-20260808-v46-route-free-relinquishment`
was bootstrapped natively from the repository Body rather than copied from an
old coordinator store. It accepted the explicit objective, selected proposal
`operator-hands-route-relinquishment-v1`, and accepted a source-grounded,
proposal-bound Modeling result. Modeling included
`epiphany-core/src/bin/epiphany-mvp-status.rs` in the frontier scope, proving the
route-free recovery boundary that v45 lacked.

Canonical Imagination then returned a candidate that failed current request
validation with `frontier planning candidate substituted request identity or
required cargo`. The generic runtime job was correctly marked `Failed`, but no
`epiphany.runtime.role_worker_result` failure projection was persisted. Self's
planning lifecycle consumes that typed role result, not the generic job, so it
remained falsely at `imaginationRunning` after the worker exited. Native
interrupt could mark the thread binding blocked but could not create the
missing typed failure or lawful retry. Preserve v46 as negative evidence; do
not edit its store or read its raw worker output.

Fresh retry v47 is
`F:\Projects\.epiphany-runtime\shakedown\live-20260808-v47-route-free-relinquishment-retry`
on thread `shakedown-v47-hands-relinquishment-r1`. Its proposal payload is
`sha256-06f21aa478a3f6374b9c42bcd0d75320dfd0c354204be69169b67a9ddb13506a`
and selected Modeling request is
`repo-frontier-proposal-modeling-ff89a288603b4b26ba86a728240d03710b45516c7bb5d3e6d02e7a63bb41cf70`.
The scope hints use the actual source owners: `hands_gateway.rs`,
`mind_gateway.rs`, `repo_model_gateway.rs`, `runtime_spine.rs`, the Hands CLI,
`epiphany-mvp-status.rs`, and the app server. Active-release proposal Modeling
completed and was accepted under the operator-safe artifact directories
`.epiphany-dogfood/v47-hands-relinquishment-modeling-r0` and
`.epiphany-dogfood/v47-hands-relinquishment-modeling-accept-r1`.

v47 canonical Imagination then failed identically to v46. Generic runtime job
`43a3855c-8445-4c0d-8fa2-85576ef6d631` is `Failed` with the same candidate
identity/cargo validator error, while frontier lifecycle remains
`imaginationRunning` with no typed result. This independent reproduction rules
out a one-off detached-process race. Do not create v48: planning cannot adopt
the repair because the failure is before the typed result Self needs to review
or retry planning.

The operator explicitly authorized proactive intervention in corrupted
Epiphany runtime state, with receipts. The labeled supervisor intervention is
now implemented in source. The OpenAI runtime canonicalizes set-shaped safe
paths before identity validation and persists a non-executable typed role
failure beside the generic failed runtime job. Self owns explicit reviewed
supersession and creates monotonic immutable retry bindings for both
Imagination planning and dedicated Mind judgment; failure alone cannot retry,
old attempts remain evidence, and candidate/decision cargo is forbidden on the
failure projection. Focused core, coordinator, and runtime tests pass,
including symmetric Imagination and Mind failure/review/retry proofs. The next
move is commit/push, one exact incremental authenticated package, and a fresh
route-free live proof. v46 and v47 remain sealed and must not be repaired in
place.
