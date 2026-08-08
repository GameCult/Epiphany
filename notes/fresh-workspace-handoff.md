# Fresh Workspace Handoff

Exact pushed `5f09d35a` packages as
`sha256-39a81a35d847fd6b459379d5bf3ef4fa57a3142745a10f13f4b033eea85ca45a`
with witness
`sha256-9cf82d9c1bdf018c36d199838598f5476acbc43818beb9fdb68840dc00be7f9b`.
Because this commit changed only the coordinator binary, the stable release
cache reused `epiphany-core` and Cargo completed in 29.58s; the preceding core
change took 6m50s. The remaining iteration penalty is monolithic core
invalidation, not universal package reconstruction.

Fresh v40 replay proved the proposal startup actuator. Request `4b9414...`
launched under persistent supervision, resident revision 5 remained running
while its Modeling worker lived, and revision 6 completed only after the exact
proposal launch binding existed. Typed status then advanced the pending queue
to request `e75b60...`. The Modeling worker result failed and is not accepted,
but the first request was consumed by its exact launch consequence rather than
by a generic coordinator receipt. v40 is sealed as the successful
binding-before-success boundary.

Exact `58f42b6a` packages as
`sha256-562aeb2bf246e27f2b63ed094633b63daff7ca07d5eb5f14dfcb12d47fca6daf`
with witness
`sha256-8d5ad5e2b734679c8b788445f6ad91d31468cc56d001f3d975fa34f373faf0c2`;
its stable-cache core build took 6m50s. Fresh v39 proved `unfulfilled`
cancellation and heartbeat retry repeatedly preserve the exact proposal
pressure. It also exposed why no retry could succeed: resident coordinators run
in safe `plan` mode, and proposal Modeling alone lacked a typed startup launch
handler. Operator-safe coordinator evidence showed `launchModeling`,
`canAutoRun=true`, one planning step, and zero launch events. Raising the step
budget from four to eight did not change the result and is rejected as a fix.
The current worktree gives proposal Modeling the same exclusive objective-free
startup ownership already used by typed Imagination and direction requests.
All 15 coordinator binary tests pass. v39 is sealed at this missing-actuator
boundary.

Exact pushed `88a38987` packages as
`sha256-76ed147f0ce86f26bab210534e623ed0a67030c2d8a03a7ad7e5226ffa934059`
with witness
`sha256-d7f6551dda188dbd3abe4ccbd73ce97825bc3d77f1467968f8d56c1fdd40437f`.
Fresh v38 replay from the pre-proposal v31 boundary crossed autonomous
Imagination and derived proposal Modeling request `673ab...`. The replay then
falsified the fulfillment repair before its intended assertion: swarm emitted
typed cancellation status `unfulfilled`, but the sole cancellation primitive
rejected that new terminal class, so it could not perform its atomic pressure
requeue. The current worktree aligns that contract and the focused atomic
requeue test now exercises `unfulfilled`; it passes. v38 is sealed as the
terminal-contract failure boundary.

## Current authority — 2026-08-08

The active branch is `codex/epiphany-shakedown-live`. Exact pushed release
`8a9439ca2e0dddaec832a5e9285735b4d5a109cd` is authenticated as
`sha256-39e12132f829fbb782cc4ba33308e44c641f6bd4a1136850ce9246b1179c3cbd`
with witness
`sha256-d57bdeb6e6b14cc3b7500d461fa7ea26455d319b4f1d904cda2fa6e0317407be`.
Its 21 binaries contain no private state. The active stable-cache publisher
completed this core-change Cargo pass in 6m46s; an identical warm pass took
1.48s of Cargo work and 18.669s end to end. An older publisher using ephemeral
source plus a commit-local target was terminated after 12.8 minutes. Rebuild
the publisher itself after changing release-cache machinery; otherwise the
benchmark measures an obsolete body.

Fresh runtime `live-20260808-v35-request-thread` published that release and ran
under persistent swarm supervision. The resident child completed on exact
thread `shakedown-v31-rupture-closure-implementation-r1`, proving the typed
request owns the launch and lease across the real process boundary. The
accepted Imagination direction result contained several options. Autonomous
promotion correctly persisted each option and its exact provenance as selected
proposal-Modeling work, but status then failed because the pending projection
required exactly one unclaimed request.

The current worktree treats selected proposal-Modeling requests as an
oldest-first execution queue. Exact `16be92f4` packages as
`sha256-998dbac0877ee2249f723796501f4f82c50cc9dcb8ae205117a50cde04bab3e6`
with witness
`sha256-4975a3b043012f62eecfd9c9b6c9f431bbf66a48ff1f46117bd1532c8b0dba28`.
v36 proved status derives `launchModeling` instead of rejecting cardinality and
completed one proposal Modeling turn. The oldest request nevertheless remained
pending because v35 had emitted pressure for every option and heartbeat had
selected a later pressure first. This falsified the partial repair: runtime and
resident pressure were still separate schedulers.

The current worktree emits resident pressure only for the runtime-owned pending
head. Its exact launch binding exposes the next request on a later ingestion
cycle. All 17 resident-Self tests pass; focused tests prove runtime ordering,
single-request launch consumption, and head-only pressure emission. v35 and v36
are sealed failure boundaries.

Exact `a1d80d40` packages as
`sha256-1ef95f24e9d0bfb5e88ded9391c261d3b4cdc37276f4a6a0f2479cb57a8c9760`
with witness
`sha256-1fde6a1cd73747393631ac50de02c5e404a48c263f48386232b01ae288ab0024`.
Fresh v37 replay proved head-only pressure: only request `464ac...` received a
pressure, grant, and child claim. The coordinator exited zero and resident Self
accepted its generic terminal receipt, but typed status still reported that
same request pending. The runtime had no exact proposal-Modeling launch binding.

The current worktree makes that impossible to call success. A proposal-Modeling
grant must have its exact runtime launch binding before resident terminal ack;
otherwise the turn is cancelled as `unfulfilled` and the same pressure returns
to heartbeat. The focused fulfillment test and both swarm binary tests pass.
v37 is sealed at the false-success boundary.

Fresh runtime `F:\Projects\.epiphany-runtime\shakedown\live-20260808-v31`
accepted initial and proposal-bound Modeling for thread
`shakedown-v31-rupture-closure-implementation-r1`. The generic user proposal
lawfully derived a direct Hands gate, but that gate remains inert because this
objective requires the autonomous Imagination-origin path and dedicated Mind
adoption before Hands.

Resident ingestion then committed the admitted-model direction request and
pressure. A standard heartbeat selected that pressure. The next resident cycle
failed before launch because replay recreated the same producer pressure ID
with a later `created_at_millis`; full document equality misclassified that
ordinary replay as an identity collision.

Pushed commit `b81092b2` repairs ownership in `resident_self.rs`: producer replay
compares immutable producer identity, while timestamp and pressure lifecycle
remain derived/mutable. Changed objective, provenance, kind, schema, identity,
or privacy authority still collide. All 15 resident-Self tests pass, and the
autonomous promotion integration proves ingestion after heartbeat consumption
returns zero without recreating work.

Exact `b81092b2` packages as
`sha256-da343dcfe478720320f94c582eebcef2804aac41ac136a9c6a504cd248f495fa`
with witness
`sha256-41f081ce260467325a759bf6c888b30cf7d57faff1b1fdcfab398191323cf333`.
The authenticated replay clone `live-20260808-v32-replay` crossed the original
collision and launched the resident coordinator. The child correctly refused
an operator-supplied artifact root outside `.epiphany-dogfood`. Its typed
cancellation then exposed a second continuity wound: the source pressure stayed
`consumed`, so heartbeat could never grant an exact retry.

The current worktree makes cancellation atomically return the exact pressure to
`pending` while writing the failed terminal acknowledgement. Heartbeat remains
the sole retry scheduler and issues a new grant identity. All 16 resident-Self
tests pass, including exact failed-turn pressure recovery.

Exact pushed `c5f99c46` packages as
`sha256-04a4854a1268139e42c028503460bb75d5b4c97b28c7fafcfb963ad1210c0090`
with witness
`sha256-615b3878473d65c67998614e1f6e33cec2bb5ea08a985210ff1c2161c1a481a5`.
The v33 retry proof repeated the deliberate startup refusal under this release.
Cancellation returned the pressure, but heartbeat reconciliation exposed that
grant identity was still only schedule + action + pressure. A same-action retry
therefore collided with the immutable first grant.

The current worktree adds a pressure-local attempt ordinal to grant identity.
The retry test now uses the same heartbeat schedule/action and proves attempt 2
receives a distinct grant while retaining the same pressure authority. All 16
resident-Self tests pass.

Exact pushed `d4b446ab` packages as
`sha256-d31ab4329dd34d93e101fa3a783373b6b1f3bc311fac057c6f0d0b4ce5911122`
with witness
`sha256-d5256ee2181dcacc3da87c8f491a12c359a0bde6139a362d40053d453acac4b8`.
The v34 live timeline proved the deliberate failure, cancellation, attempt-2
grant, and replacement launch. Running the replacement under persistent
`epiphany-swarm serve` reached coordinator validation, which correctly refused
`model direction consideration provenance mismatch`: resident preparation had
substituted fixed thread `resident-self-thread-epiphany-starfire` for the
implementation thread owned by the typed direction request.

The current worktree reloads typed direction, Imagination-consideration, and
proposal-Modeling requests during preparation and carries their exact runtime
and thread identity through argv, child launch, and lease. Plain operator
pressure alone retains the fixed resident thread. All 16 resident-Self tests and
both packaged swarm tests pass; typed launch tests assert the request-owned
thread.

## Next action

1. Commit, push, package, and authenticate proposal-pressure fulfillment.
2. Preserve v31 through v37 as failure evidence; clone v37 and publish the exact
   successor release.
3. Prove the unfulfilled zero-exit requeues, heartbeat retries, the exact launch
   binding is created, and only then the next selected option becomes pressure.
4. In a fresh runtime, re-run pressure ingestion twice and resident launch.
   Prove autonomous direction consideration, proposal-bound Modeling,
   canonical Imagination, dedicated Mind adoption, and exact Hands authority.
5. Deliberately fail one bounded worker and prove typed terminalization,
   restart/recovery, Reorientation, and Soul-verifiable closure.
6. Continue through Persona consequence, retention, endurance/resource
   plateau, and Linux/Yggdrasil cognition.

Do not execute v31's direct Hands gate, read raw worker transcripts, mutate
sealed runtimes, or start concurrent Cargo against the release cache.
