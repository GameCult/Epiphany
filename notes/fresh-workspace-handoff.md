# Fresh workspace handoff

## Cross-container resident ownership accepted — 2026-08-09

Exact `c930f505` release
`sha256-cd684079eec277bbb434f147c4f6c679553780230822f9d30f0810ec0988100a`
with witness `sha256-c7642818f076f7109585a6e7021371815607fb7ecb141cb2b7c008335fbecec4`
passed independent 24-binary inspection. In copied state, a resident owner reached
sleeping and created `resident-self.cc.resident-self.process.lock`. A second
container was refused with exit 1 and the exact owner-already-exists error;
resident state SHA-256 remained
`8EA6CD6C30C6D2D64F850585D865B342493CEFD04C34D5870D65DFE506E1190A`.

Heartbeat tick then completed without changing the resident hash. Bounded
Heartbeat serve acquired `heartbeat.cc.heartbeat.process.lock`, emitted one
pulse, and stopped cleanly while resident Self remained live. Resident SIGTERM
exited zero in 483 ms with final `braked`. Receipts are under
`.epiphany-run/resident-lock-c930f505/` and container logs are retained.

## Warm package acceptance and resident-owner lock — 2026-08-09

Construction-owned Cargo home is accepted. Exact `9f369528` migration release
`sha256-154af0ef9a729f7f46876f99add6828d7e5503f3ecc949ac791f00f26017beca`
with witness `sha256-9b670a4acca14480840da3f09fd83ff5c97829b19f81bd6037c59c9146548cec`
took 15m04s once. The identical warm replay took 10.17s wall, Cargo 1.38s,
and passed independent 24-binary inspection without warnings or collisions.

Do not describe the resident store as globally single-writer. Heartbeat and
authenticated operator admission own separate typed CAS entries. The proven
fault is that Unix `resident-self` singleton locks use container-private temp
directories. Current source places role-specific process locks beside the
mounted store, preserving Heartbeat's distinct role lock. Windows focused
tests pass; package once and prove two-container exclusion on Linux.

## Corrected shutdown replay and Cargo-home ownership — 2026-08-09

Exact `b8138a6b` release `sha256-f3e0966333b86181bf13893a45f7636d0c6ff13720fbf535b29cca485c3fd58d`
with witness `sha256-4652c86f4a5560475de597fd162dfc7a5369df711cd00e51a3805e769b68a0ee`
passed native 24-binary inspection. Frozen coordinator PID 88 retained pending
SIGTERM, resumed, terminalized, and the resident exited zero in 1.613 seconds
with final `braked`. Restart reached `sleeping`; idle SIGTERM then exited zero
in 871 ms with final `braked`. Receipts are under
`.epiphany-run/shutdown-active8-b8138a6b/`.

The exact package took 19m26s. The Docker recipe mounted a Cargo-home volume at
`/cargo-home` without setting `CARGO_HOME`, so fresh containers still updated
and downloaded through throwaway `/usr/local/cargo`. Release construction now
owns `build_cache_root/cargo-home` and exports it to Cargo. All 19 focused tests
pass. The first exact build after this change is a one-time source-path
migration; require a second warm replay before claiming the iteration fix.

## Active resident shutdown proof — 2026-08-09

Exact authenticated `00ee3821` idle SIGTERM exited zero. In the active proof,
the packaged coordinator was frozen as PID 103, PID 1 received SIGTERM, the
child retained pending termination, and resumption terminalized the child
before the resident exited zero in 2.023 seconds. The packaged operator
projection nevertheless reported `failed`; source now maps shutdown
cancellation to `Braked`, with a focused invariant test. Package and replay
that exact correction once.

An attempted live admission also proved that a second process can write typed
pressure which the resident's in-memory state later overwrites. Preserve the
active6 receipts. This is a resident-store single-writer/command-ingress seam,
not permission to add a retrying sidecar writer.

## Release-construction bootstrap extracted — 2026-08-09

Commit `76afdaf0ce863f19e9396ef4d21ed8ff3fb39690` moves package
construction, filesystem verification, and witness inspection into
`epiphany-release-construction`. `epiphany-core` retains only live CultMesh
publication, load, and authentication. The bootstrap graph contains no
`epiphany-core`; all 19 construction tests pass and the core test harness
compiles. Exact Linux bootstrap fell from 7m57s to 30.24s cold and 4.40s warm.

The narrow tool then produced warning-free 24-binary release
`sha256-30caab9ef1ea89f9a885d87832b1e312a35288ec3f895bd5b0e83f7f7a6a1aec`
with witness
`sha256-89c2ad604254faec3e1e01f753b5dff38a402f43d009074ba4b47b1ec46fdecf`.
Cold Cargo took 12m16s. The identical warm replay took 10.16s total with Cargo
at 0.82s and reproduced both identifiers. Independent narrow-tool inspection
accepted both copies. The packaged full publisher then accepted the same
witness and changed c005's local Verse hash from `b160c610...3909` to
`83ed430f...ae81` without activating the body. This proves construction and
publication remain interoperable while owning distinct consequences.

Next assign graceful signal ownership. Shutdown must prevent new work, preserve
an unclaimed prepared launch for restart, signal an exact active child, record
shutdown terminalization without failure pressure, and exit zero only after no
child remains. Prove both idle and active paths against copied typed stores
before replacing c005.

## Exact 24-binary release published — 2026-08-09

Exact pushed source `f00c8279c966c43c6ac899d9b16280f373b45941`
completed a warning-free 24-binary Linux package in 18m44s of Cargo work. The
authenticated release is
`sha256-a7c4c2e305257664e9fe2eb64fdd5ba9e7bcb3491c5697e95082c4a3713b579e`;
its witness is
`sha256-e9984086452ae0d4887f7ab63d111469d1f801fd7712c2441611bf5793b889b8`.
Independent native inspection accepted source, inventory, hashes, and private
state exclusion. The identical warm replay completed in 10.64s total with
Cargo at 0.92s, reproduced both identifiers exactly, and passed a second
independent native inspection. Receipts live under
`.epiphany-run/linux-package-f00c8279-{3,warm}/`.

Native publication inserted the release and head into c005's main local Verse
without replacing the running 9b8df0b4 body. `local-verse.cc` changed from
SHA-256 `d88bb300f8db589f3f82fcba6781b9eb1e9828f22a55684fdaaba0e8a68dcf0d`
to `b160c610988753278afbad7f5bad436193144a92e9fd506982099ad58d213909`.
Availability does not own activation.

The remaining iteration tax is now precise. Rebuilding the exact release
publisher first took 7m57s because `epiphany-release` imports packaged-release
construction from monolithic `epiphany-core`; the package command then builds
that core again inside the 24-binary graph. Extract construction and witness
inspection into a narrow crate/tool, leaving live Verse publication as its own
authority. Prove the cold bootstrap reduction without changing release or
witness authority, then resume graceful SIGTERM ownership.

## Resident continuity, topology repair, and build-free status — 2026-08-09

Authenticated release `sha256-d1392730...` is now the running c005 body. The
old body and the first replacement container are retained as rollback receipts.
Both Docker stops sent SIGTERM and ended in exit 137 after timeout, proving that
`epiphany-swarm serve` has no graceful signal owner even while idle.

The first replacement preserved resident Self revision 6, sleeping state, no
active turn, and no prepared recovery, but native status exposed a live topology
fault: resident Self used `/state/runtime.cc`, whose next action was
`launchResearch`, while the accepted coordinated cognition chain lives in
`/state/eyes-tools/runtime.cc`. The resident was rebound without copying or
merging state. Release authentication remains `/state/local-verse.cc`; runtime,
coordination Verse, and Mind now use the established `/state/eyes-tools/`
stores. The current container ID begins `f0303137`; rollback container
`epiphany-linux-resident-self-c005-wrong-runtime-0934` preserves the bad binding.

Post-rebind operator projection derives `awaitFrontierProposal`, terminal
frontier planning, accepted Research, and `modelingResultAcceptedAfterResearch`.
Route `817bf422...` appears only as the route named by relinquishment receipt
`repo-frontier-relinquishment-7b846272...`; it is not actionable or recreated.
The obsolete root runtime hash remained
`c10caa321322632111a4e8f69550be4c799d78c92a78780a3066bbf6ad0a7c7c`.

The diagnostic itself exposed hidden build authority: standalone
`epiphany-mvp-status` invoked Cargo whenever optional Heartbeat/Persona/Void
helpers were absent. That fallback is deleted. Optional panels now report
unavailable, `--coordinator-only` performs the exact routing read, and status is
the 24th explicit packaged binary. Heavy OpenAI and MCP runtime dependencies are
feature-admitted so coordinator/status iteration does not compile them. Focused
proof passes 19/19 release tests and 3/3 status tests; warm runs complete in
about 1-4 seconds. Exact 24-binary packaging remains next, followed by graceful
shutdown ownership, restart proof, retention, logs, and endurance.

## Deterministic warm release published — 2026-08-09

Exact pushed source `9b8df0b487eaae12e35dbe53a82faae1a1e39954`
packaged without output-collision warnings in 12m46s after the sole-owner cut.
Native inspection accepted all 23 binaries in release
`sha256-d13927303c957db6799135a61ce6a2899235c397b525968be0bffc3b5ad2a831`
with witness
`sha256-d07277cb5f142a4cc90e9364da0bd058eeba8c524eaf8c00bd9d55f088c78de0`.
The identical warm replay completed in 10.73s total, with Cargo at 0.97s, and
reproduced both identifiers exactly. Native inspection independently accepted
the warm copy.

The native release publisher added this exact release to c005's local Verse
while the resident process remained live. Before publication,
`local-verse.cc` was 43,849 bytes with SHA-256
`60872eb9222485c00ccb0c423776f85bacaf2521ed9b80d1bcceccfb319b8eaf`.
After publication it is 51,311 bytes with SHA-256
`d88bb300f8db589f3f82fcba6781b9eb1e9828f22a55684fdaaba0e8a68dcf0d`.
The running c005 body is still pinned to the older authenticated release. Next:
commit this evidence, then perform a supervised stop/body replacement/restart
against the existing stores and prove continuity resumes at
`awaitFrontierProposal` without recreating relinquished route `817bf422`.

## Sole packaged executable owner — 2026-08-09

Exact `fe6bf006` cold packaging proved the single-process graph and produced
native-inspected 23-binary release
`sha256-7e579b3d78e7de4a8bf161125908ac41d2b9582aba24b5f5be6922d6a4940db0`
with witness
`sha256-45215f5d27d8edb5be877aad769ccd7ac971952d343dc516dea744ee4f82533a`.
The cold conversion took 36m55s. Its identical warm rerun took 11.05s total,
with Cargo completing in 1.26s, but produced a different valid release and
witness. Cargo reported output filename collisions between the selected
`epiphany-release-bundle` and `epiphany-core` packages. Four binaries differed:
`epiphany-hands-action`, `epiphany-persona-mouth-identity`,
`epiphany-repository-body`, and `epiphany-workspace-coverage-projector`.

The live authority correction makes `epiphany-release-bundle` the only package
selected for witnessed binaries. It now declares `epiphany-state` alongside the
other 22 release targets, still using the core-owned source and library. The
core package remains a developer binary surface but is no longer a packaged
executable writer. All 19 focused tests pass. Commit and push this cut, rebuild
the exact packager, then require a warning-free exact package and byte-identical
warm replay before publication.

## Deterministic single release graph — 2026-08-09

Exact pushed source `e9465c11208d648440999903770cbde245c1da3b`
produced authenticated release
`sha256-5f90294fb037ff777ca99f345e0f4b1e082b176c35bd6e16a60b46d5a687c0cb`
with witness
`sha256-403a4dd4705772d8d588f3abedb336436b9f1b4ebc037b86efbc5995f5d6a291`.
Native inspection accepted the exact source and all 23 required binaries. The
packaged `epiphany-state` hash is
`8e5e0edd3af59a9f8710d4b6b3a7fbe8243f1002acd6e1d4c415e8053d0cd7c2`.

The cold package took 24m22s: 17m03s for the root graph and 6m38s for the
separate state-steward graph. The identical cached rerun was worse at 28m30s:
21m04s for the root graph, 34.62s for the coordinator graph, and 6m42s for the
state-steward graph. It also produced a different authenticated release,
`sha256-4700e984410e0c8e735d1fcecec282235ed5d096d20b6dbbdb206a83636228fa`,
with witness
`sha256-f71519f7a27a9588715daf9f11d33031b2680659aa405c29807a39efcfcce373`.
Twenty-one binaries were byte-identical; only `epiphany-model-runtime` and
`epiphany-persona-service`, both from `epiphany-openai-runtime`, changed. The
old packager was oscillating one target cache through three feature graphs,
and incremental release codegen made the authenticated output nondeterministic.

The source cut now uses one Cargo command for both first-party packages, the
coordinator feature, and all 23 explicit binaries, with `CARGO_INCREMENTAL=0`.
The separate coordinator and state-steward build authorities are deleted. All
19 focused packaged-release tests pass. The cold Windows harness took 3m38s;
the final source rerun compiled in 47.87s and executed in 0.83s. Commit and push
this cut, then require an exact Linux package and an identical warm rerun with
both materially improved time and exact release/witness equality before
publication.

## Unified Cargo authority and shipped state steward — 2026-08-09

The first exact `6808f3f5` package attempt failed before release construction:
the one-time `cargo generate-lockfile` had advanced `allocative` from 0.3.4 to
0.3.6 (and its derive/ctor companions), which is semver-accepted but
incompatible with vendored Starlark's `hashbrown` Allocative implementation.
No release or witness was emitted. The root lock is corrected to the previously
proven trio `allocative 0.3.4`, `allocative_derive 0.3.3`, and `ctor 0.1.26`.
A release-target `cargo check --release --locked -p
epiphany-openai-auth-spine` traversed Starlark, Codex protocol/login, and the
auth spine successfully in 3m16s. Commit and push this lock repair, then advance
the clean Linux package source and retry once; do not reuse the failed package
as evidence.

The root manifest is now the single first-party Cargo workspace and root
`Cargo.lock` is the sole dependency-resolution authority. Seven child lockfiles
and three now-ignored child patch declarations were deleted. Vendored Codex and
CultCache retain their own explicitly excluded workspaces. A source guard in
the release packager rejects a missing first-party member or a regrown child
lockfile.

The exact stale-CRRC policy regression paid a one-time 5m02s first-party
workspace migration, then reran identically in 4.47s. `epiphany-state` remains
owned by `epiphany-core`, built on the same target in 23.20s, and appended typed
evidence without another graph; a full status read including container startup
took 1.12s. The rejected attempt to place it in the root release package was
stopped because that would drag the root monolith into a narrow ledger tool.

The native release packager now witnesses `state-steward` as the 23rd required
binary and builds it explicitly from package `epiphany-core` on the existing
release target. All 18 focused packager tests pass. The one-time core test
harness cost 5m33s; an identical warm run took 4.43s, and the final source
increment rebuilt/tested in 20.10s. Receipts live under
`.epiphany-run/unified-build/`. Next package the clean committed source and
prove a warm identical 23-binary release before resuming resident endurance.

## Stale CRRC authority cut and copied c005 replay — 2026-08-09

Self no longer lets a failed reorientation result older than typed frontier
relinquishment `repo-frontier-relinquishment-7b846272...` route the next organ.
The status owner compares the latest bound reorientation job update with the
relinquishment timestamp. Coordinator status demotes only stale manual-regather
pressure to routing Continue; raw CRRC remains published as
`regatherManually`. The focused causal regression passed 1/1.

A collision-safe copy of c005 `state/eyes-tools` was replayed for thread
`linux-eyes-tools-c005`. It derived `awaitFrontierProposal`,
`canAutoRun=false`, and target Imagination while source signals still reported
raw CRRC `regatherManually`. The live c005 store was not mounted. Receipts are
under the directory named by `.epiphany-run/stale-crrc-replay-current.txt`; the
valid proof is `replay-c005-eyes-output.log`. Earlier root-store/fresh-thread
replays are rejected-path receipts, not c005 evidence.

The independent `epiphany-self-policy` lock graph paid 6m34s for this one cold
test. A misconfigured Docker invocation then omitted `CARGO_TARGET_DIR`,
compiled into `/workspace/target` for 17m48s, and was proactively stopped with
logs preserved under `.epiphany-run/stale-crrc-build/`. With the target, Cargo
home, and Rust proxy path explicitly separated, the coordinator rebuilt in
37.55s. Encode that contract in one native launcher and collapse the policy
test plus state steward into the root dependency authority. Do not add another
cache.

## Resident Mind terminal adoption and warm package proof — 2026-08-09

Exact pushed source `bd96305e60428b8ea2af822f1e1a233405c09c43` is packaged and
published as
`sha256-01dbed4b015dabccc6f17df49118bba90195993da002f109bafd73f23b1d3a8f`
with witness
`sha256-c3ab5a602de9c58e3ffb23cc5d316e5e36d94a013a697a113af01b4b2cf00808`.
The first package completed its release graph in 25m12s and its isolated
coordinator phase in 33.53s. It followed a host incident where
`C:\Users\Meta\.cargo-target-codex` occupied roughly 276 GiB, filled `C:`, and
caused rustc and Docker backend SIGBUS failures. `cargo clean --target-dir`
reclaimed the target, restored roughly 252 GiB free, and Docker was recovered by
terminating only `docker-desktop` WSL before restart. Preserve the preflight and
SIGBUS receipts under `.epiphany-run/linux-package-bd96305e/`.

The identical package was then rerun against persistent volumes
`epiphany-linux-package-source`, `epiphany-linux-package-cache`, and
`epiphany-linux-cargo-home`. It completed in 9.65 seconds wall; Cargo reported
0.81s for the ordinary release graph and 0.65s for the isolated coordinator,
and reproduced the exact release and witness. Receipts are under
`.epiphany-run/linux-package-bd96305e-warm/`. This proves the fast shakedown
path is stable cache retention plus free-disk/cache preflight. Do not clean or
partition the shared package graph for routine iterations, and never build this
graph on Yggdrasil.

The typed evidence append exposed a separate build boundary: `epiphany-state`
is not among the 22 shipped binaries, and building it standalone against the
same exact source/cache took 7m11s. Do not solve this with another graph cache.
Ship the state steward through the owned release graph or move its narrow typed
ledger-write port into an already-shipped native owner.

The published repair completed the resident frontier-planning chain. Typed
failure review
`frontier-planning-failure-review-4824d856-876f-432c-8fdd-664728938105`
superseded failed Mind job `8e5952fb-0db6-460c-b061-c75d32410cc2` and authorized
one retry. Mind job `2d2a3e8b-77f0-4f57-9125-ec3305a95843` returned a valid
typed adopt judgment without tools or external consequence. Self committed
terminal decision
`repo-frontier-plan-decision-e7d7f3c79595e09f2d2cd9e7c4614909e803b1f9b509f0fc1482d3aaab2ad6d3`
against the original candidate. The next typed action is
`continueImplementation`; execute only that admitted Hands frontier. Do not
recreate Eyes, Modeling, Imagination, either failed Mind attempt, or the
successful retry.

Hands gate `hands-intent-linux-eyes-tools-c005-0-2ad7b2ce-...` exposed the
candidate's exact rollback: discard the planning draft if later evidence shows
the coordinated resident circuit already exists. Current canonical state and
c005 receipts satisfy that condition, while the runtime artifacts needed to
reconcile it are outside the adopted safe paths (`notes/fresh-workspace-handoff.md`
and `state/map.yaml`). Hands therefore produced no mutation and recorded typed
refusal `hands-refusal-4cd85889-17d0-4d26-8a95-07d04a5cdabf`. Mind atomically
retired route `repo-frontier-route-817bf422...` through relinquishment
`repo-frontier-relinquishment-7b846272...`, advancing RepoModel revision 2 to 3.
The next coordinator snapshot falls back to historical CRRC
`regatherManually` from the old failed reorientation worker. Treat that as a
routing-precedence wound, not fresh Eyes evidence and not authority to recreate
the retired route.

## Packaged Body speed proof and resident Mind identity correction — 2026-08-09

Exact pushed source `217dd4b96746536e01dc8991b5e6f31c2c588af0` is packaged as
`sha256-3dab68c572519501c3401a8b737603e312349a4052f69f7f751478fa2631aaf3`
with witness
`sha256-2278ec15ed44a21e4bc4d20b4f64d67fb530c4bc2bc44cd1cd9d775b16838f74`.
The exact packaged observer reread the unchanged resident c005 Body as
generation 1/tree `4d1f8a23558d53622ed21be0bb51db456b532f48` in 94,521ms. This is
strong observer-boundary evidence against the old 160,183ms Modeling
`jobCommit`, but not a coordinator-to-coordinator replay because Self's typed
route had already advanced. Receipts are under
`.epiphany-run/linux-package-217dd/`; do not rebuild this release.

The resident chain completed `startFrontierPlanning`, then native Imagination
job `f62738f1-a819-431a-a454-c0b2cff5a4d8` produced candidate
`repo-frontier-plan-candidate-28f597414331a67b71449bcf28317c42ddbd78566dbe92aa41b9f54302762be8`.
Self committed Mind request
`repo-frontier-plan-mind-90e8c1995805d94d05c9e065a117c54eb8993cd93c985b9daa2915746c32977f`.
Mind attempt `411e73bb-4906-4785-be92-2a72a9290691` failed identity validation,
was explicitly superseded, and authorized one retry. Retry
`8e5952fb-0db6-460c-b061-c75d32410cc2` failed with `Mind result request
disappeared`. Direct read-only binary inspection proved the exact canonical
request, planning request, and candidate keys remain in `runtime.cc`; the model
had substituted an unknown echo. Both failures are immutable evidence.

The authority fault was in source: `RoleWorkerResultIngress` allowed model JSON
to author the outer Mind request echo plus five identities inside the typed
decision. `epiphany-openai-runtime/src/lib.rs` now derives request, planning,
Imagination-result, candidate, and candidate-digest identities exclusively from
the authenticated `RepoFrontierPlanMindContextProjection`. The model owns only
decision, rationale, and its proposed timestamp. A regression supplies five
counterfeit model identities and proves the runtime emits the canonical launch
identities; all 15 runtime library tests pass. Package once, explicitly review
and supersede failed retry `8e5952fb`, and launch only the newly authorized
attempt. Do not recreate Imagination or either failed Mind attempt.

## Linux Eyes-to-Modeling acceptance and Body observation speed cut — 2026-08-09

The fresh tool-enabled resident circuit under `/state/eyes-tools` completed its
causal organ chain. Eyes job `eaf25324-3d4c-49c1-a93d-65f16b080806` used three
governed native read-only source tools, emitted typed result
`result-worker-eaf25324-3d4c-49c1-a93d-65f16b080806`, and was accepted as
`ev-research-856fb6d1-17eb-4d71-bafc-774277e9b64e`. Self then derived
`launchModeling` because accepted Research was newer than Modeling. Modeling job
`3333e537-7451-4cfd-92bf-b69030b2ab4f` used three governed read-only tool rounds,
emitted `result-worker-3333e537-7451-4cfd-92bf-b69030b2ab4f`, and was accepted as
`ev-modeling-11da531d-3403-41a1-93c3-7224d364364a`. Typed status now reports both
`modelingResultAccepted` and `modelingResultAcceptedAfterResearch` true, so the
single-use route was consumed. The next action is `startFrontierPlanning` for
Imagination. No external consequence occurred.

The Modeling circuit peaked at 109,887,488 bytes over 2,025 cgroup samples.
Its launch receipt measured `jobCommit=160183ms`; process inspection showed the
time inside the first of two full `git add --all` Repository Body scans. The
observer formerly created a brand-new private index and object quarantine for
each equality scan. The current source cut gives one observation session sole
ownership of one private index/object quarantine, initializes it from HEAD once,
and runs both full scans against it. Scan two still performs `git add --all`,
`write-tree`, staged-entry enumeration, and raw-byte manifest hashing, so the
stable-equality invariant remains intact while Git may reuse its verified stat
cache. All 25 focused `repository_body_observer` tests pass, including explicit
proof that both scans share one private index and that session teardown removes
it. The live-worktree mechanism benchmark measured cold scans at 2,176ms and
1,656ms versus a shared
session at 1,414ms and 123ms: 3,832ms fell to 1,537ms, a 2,295ms / 60% pair
reduction. The ignored receipt is `body-index-benchmark.stdout.log`. This is
mechanism evidence, not yet packaged coordinator evidence; package once and
remeasure exact `jobCommit` before claiming the 160-second live wound closed.

Two attempted detached launches were refused before state mutation. The first
used a nonexistent container path. The next two reached the artifact cleanup
guard from `/`, so the guard correctly rejected `/workspace/.epiphany-smoke` as
outside its process-relative safety roots. The successful launch explicitly
used Docker working directory `/workspace`. Preserve these as operator
intervention receipts; they prove the guard rather than runtime corruption.

## Linux resident coordination phase boundary — 2026-08-09

Exact pushed source `c0055772750a7dd7335f65baa88307cf9e28c1e1` is packaged for Linux as
`sha256-b4faf5721a8662b30464dacb9bcc0945b34051521af110ab3d56eef59036ba7e`
with witness
`sha256-7ddc54e0817efdcc56b6ceb5dc62124502da0d178f1946db7a8e0283a23ecaa0`.
The cold root package took 12m33s plus 18.67s for the isolated coordinator and
exceeded 4.4 GiB; the exact warm package took 9.08s wall and reproduced the
same release and witness. Preserve Docker volumes `epiphany-linux-cargo-home`,
`epiphany-linux-target`, `epiphany-linux-package-source`, and
`epiphany-linux-package-cache`. Yggdrasil must not build this graph.

The isolated resident body is under `.epiphany-run/linux-resident-c005/`.
Packaged Repository Body bootstrap initially refused a foreign swarm ID, then
succeeded with authoritative Mind swarm `gamecult.epiphany.main`. The successful
pass ran from `23:10:34Z` to `23:15:50Z` and visibly performed two full-tree
`git add --all` authentication passes. Its receipt is
`body-bootstrap-success.log`; the identity refusal is
`body-swarm-identity-refusal.log`.

Heartbeat pulse 1 issued the original grant. Self first failed because the
artifact root was mounted outside the coordinator's canonical
`.epiphany-smoke` cleanup root. The typed cancellation returned pressure to
pending; heartbeat pulse 2 acknowledged the exact terminal ID and issued
attempt 2. With the same host artifacts mounted beneath
`/workspace/.epiphany-smoke`, Self completed revision 6 and returned to sleep.
This proves authenticated Linux heartbeat -> Self -> plan coordinator
coordination. The plan coordinator intentionally invoked no model or tools.

The next execute pass exposed process ownership. A one-shot coordinator
container opened reorient job `8870f824-fdf9-4240-996a-6a663cead22c`, detached
its model worker, and exited; Docker then killed the worker because PID 1 was
gone. Exact-job recovery inside the resident container correctly refused the
still-open inner job and sealed both outer and inner jobs failed. Do not replay
that job. Receipts are in `execute-bootstrap-refusals.log`,
`coordinator-execute-reorient/`, and `linux-reorient-recovery-summary.txt`.

CRRC next routed Eyes. Resident Eyes job
`ab813c57-081a-4c62-80e5-d3471ae451bf` reached a real OpenAI Codex request with
three native read-only source tools advertised and no external consequence.
Because the bounded objective promised no tools and coordinator ran with
`--no-auto-tools`, the model-selected tool path had no tool loop; assistant text
was empty and the `epiphany.worker.role_result.v3` parser failed closed. The
failure was superseded by typed receipt
`role-failure-review-859fa955-4fee-4dd5-a0a4-d75b769890be`. No tool invocation
receipt was admitted. The resident cgroup peak across Self, coordinator, and
model worker was 62,590,976 bytes (59.7 MiB), captured in
`linux-eyes-memory-summary.txt` across 54 100ms samples.

Auth residue audit found no copied `auth.json` and zero matches for access or
refresh tokens. The private runtime store contains the mounted auth file's
ChatGPT account identifier as model-transport identity metadata; no artifact or
workspace file contains it. Do not describe the store as credential-free
without this classification.

GameCult ops commit `812b427` repairs the uninstalled Yggdrasil resident service
contract: it provisions a root-owned empty MCP policy, supplies the required MCP
and Persona-classification arguments, moves coordinator artifacts beneath the
workspace's ignored `.epiphany-dogfood` root, and replaces the removed
`epiphany-tool-codex-mcp-spine` name with packaged
`epiphany-tool-mcp-runtime`. The normalized Linux wiring test and shell syntax
checks pass. Nothing was deployed or started.

Next: preserve this zero-tool circuit as sealed evidence. Create a fresh typed
objective/adoption boundary for a Linux Eyes circuit that explicitly permits
native read-only source tools, run it inside the persistent resident body,
prove tool receipts plus a valid typed role result, review/accept it, and inspect
the next routed organ. Do not rewrite the existing authoritative objective; the
coordinator correctly refuses that. Do not reinstall or start the Yggdrasil
service body merely because its static contract is now coherent.

## Retention and release-bootstrap pass — 2026-08-08

Exact corrected release `100d8854` packaged 22 binaries as
`sha256-da8be22948c6eb958edb2e7f7e03c5cc77d17463659fa14127402392194460d3`
with witness
`sha256-6a95957f7e7b2b47b42d2154ed8232d68ffe68d3b6bdf67bee6397fa6aab2027`.
Its packaged runtime-spine closed all four sessions in a fresh byte-identical
v28 copy, replayed closure idempotently, and refused post-close work. A bounded
live retention drain then reduced heartbeat artifacts from 12,511 directories
and 706.5 MB to 312 directories and 17.6 MB. The latest cognition pulse
survived; 191 deletion receipts are in the heartbeat store and the drain log
SHA-256 is `8280dd7567f839b0b038d69deb45bfb219c42092a231a2d331e0e3620a293003`.

Restarting the resident under `100d8854` exposed another Continuity fault:
serve iteration restarted at `pulse-000001` instead of recovering the highest
surviving sequence. The process was stopped after three pulses; its stdout
SHA-256 is `4a104e39d81e0766261fff815a7d4350de31a9d5b956634a9cb39a9e5ad40b22`.
Current source resumes from the highest exact pulse directory, refuses alien
directories and exhausted sequence space, skips protected current cognition
when forming retirement batches, and emits named operator receipt fields. Four
retention and three heartbeat-binary tests pass. The resident heartbeat remains
stopped pending an authenticated package of this correction; do not restart the
old binary.

That correction is now exact-package live-proven. Commit `42bc665a` packaged
release `sha256-f68fb664ee67af4c665c9a1c6162422a8944cc3b5771e4b6b1c90c23c63c48e8`
with witness
`sha256-d9c00bc57408ba8d1c9f74a09364c513819122f127fdb5888c93c8dbc68b694e`.
Its resident heartbeat resumed at pulse 012558, crossed the 320-directory
hysteresis edge at pulse 012561, emitted a named 64-directory retention receipt
for 3,615,104 bytes, continued to pulse 012562, and left 258 directories. The
three quarantined reset-era pulses were retired in that receipt. PID 19928 owns
the live process; logs are
`.epiphany-run/resident/heartbeat-v8-42bc665a.stdout.log` and `.stderr.log`.
Warm exact packaging after the root target-set transition measured 1m12s stable
phase plus 6.56s coordinator, confirming the prior 7m19s pass was a one-time
target-set relink rather than the new steady state.

Endurance baseline
`.epiphany-run/resident/endurance-baseline-42bc665a.json` was captured at
2026-08-08T22:08:50Z: heartbeat PID 19928 used 11.8 MB working set / 3.0 MB
private memory, heartbeat store was 1,507,503 bytes, resident-Self store 7,844
bytes, artifacts were 269 directories / 15,195,150 bytes, heartbeat stdout was
8,804 bytes, and stderr was empty. Compare future observations to this exact
artifact. Do not call direct file truncation log rotation: the current child
holds stdout/stderr handles and the launcher exits after spawn. Idunn's managed
service reconciler is the likely owner, but its current policy only opens files
at launch and has no typed size/segment authority. Map that ownership before
adding a restart timer or log helper.

The first exact `339d5a6f` build completed its Rust 1.95 graph in 17m07s but
failed because the isolated coordinator artifact was absent. Rebuilding that
single target restored the cache. A detached clean-worktree retry then exposed
the actual iteration defect: release source-cache identity was checkout-path
owned, so identical source under another worktree invalidated Cargo's absolute
path fingerprints and repaid 11m12s of workspace compilation and linking.
Current source keys exact-source cache ownership to Git's common repository
while preserving the original main-worktree identity; a linked-worktree test
proves the two views share one cache.

The resulting `sha256-cb91f60e...` package and published witness are sealed but
rejected as Continuity evidence. They were assembled by the pre-correction
debug publisher and contain the old 21-role body without
`epiphany-runtime-spine`. Inspection then found the deeper bootstrap omission:
`339d5a6f` added the role mapping but did not expose the actuator in the root
release-bundle manifest. Current source adds that binary and makes the release
test prove root-manifest presence. The current publisher, runtime-spine, and
heartbeat binaries compile from source; all 17 release tests and all four
retention tests pass. Package only after committing this corrected body.

Heartbeat pulse artifacts now have a Continuity-owned typed retention route.
Above a 256+64 hysteresis boundary it plans exactly 64 oldest pulse directories
with recursive SHA-256 manifests, writes the plan through exact CAS, deletes
only byte-identical planned members, and writes a typed completion receipt.
Pending plans recover after crashes. Unknown directories, symlinks, changed
members, root escapes, and the latest cognition artifact fail closed. The same
primitive serves the explicit command and resident heartbeat loop. No live
artifact has yet been deleted; stdout/stderr rotation remains a separate
supervisor-owned front.

## Continuity closure pass — 2026-08-08

Sealed v28 remains immutable at
`F:\Projects\.epiphany-runtime\shakedown\live-20260807-v28\runtime.cc`,
2,378,654 bytes, SHA-256
`df9d707f1a87dcf557ad638c7baf8136d9db6f72b35d57abe96759088331f864`.
A byte-identical disposable copy under
`.epiphany-run/continuity-v28-replay-20260808/` reproduced dead Imagination job
`b34d21de-6788-4935-8910-75275c257444`. Authenticated release `55660d78`
terminalized that outer job as failed with result
`result-worker-b34d21de-6788-4935-8910-75275c257444`; the copied runtime then
reported nine jobs, nine results, and zero open jobs. A second worker start was
refused because the job was already terminal. Adapter-status refresh changed
the store hash during that refusal, so job/result identity—not whole-store
byte equality—is the restart invariant.

The replay then exposed a separate architectural hole: all four v28 sessions
were still active because the runtime spine defined `Completed` but supplied no
completion owner. The current source adds Continuity-owned session closure,
`list-sessions`, and `close-session`. Closure refuses open jobs and archived
sessions, emits deterministic `session.completed`, is idempotent for an already
completed session, and prevents later job creation. The focused core test
passes. The disposable v28 copy now has zero active sessions and four closure
events; its final SHA-256 is
`06179384829bda33dfb5df78dc0923a545598ddf73e33c674abe96934f2713c5`.
Package and publish the source cut next, replay with the exact package, then
begin bounded retention. The Discord credential and stable Windows firewall
rule remain separate open fronts.

The first `230f71f4` package attempt was deliberately terminated after 211
seconds. It had correctly opened a new Rust 1.95 graph namespace, but inspection
showed `epiphany-runtime-spine` was absent from the release's required binary
set. The resulting package could not expose the session-closure actuator and
therefore could not prove its own Continuity claim. Rejected build progress is
preserved in `.epiphany-run/package-230f71f4.stderr.log`. The current source
adds `runtime-spine` as a required authenticated release role; all 17 packaged
release tests pass. Commit and package that correction instead of resuming the
invalid `230f71f4` build.

Epiphany remains a supervised engineering alpha. Starfire is the cognition and
release forge; Yggdrasil is the small live crossing host and is not a build
machine at its current memory budget.

## Authoritative state

- Branch: `codex/epiphany-shakedown-live`
- Latest pushed Epiphany commit: `339d5a6f7ba6cc11a39226f1eb6608da0339e25e`
- Authenticated release: `sha256-2f4a17126d12935088189c32caadef202a1e9b698ec2cfc8fd4aec08c0763696`
- Witness: `sha256-c0788bda36aa28b8025979378968a2c48cbbcc607a1a538351756831c2b287f4`
- Live Bifrost/CultLib runtime: `cb3239aaac963995ad012e648f9a455463a54ea2` / `f67f5122ed1bd11da016e7b820ed60145ccd0299`
- Live workspace: `F:\Projects\.epiphany-runtime\shakedown\live-20260808-v53-hands-precedence`
- Thread: `shakedown-v49-hands-relinquishment-r1`
- Thread-state revision: `59`
- Sealed evidence: v49 through v52. Do not mutate it.

## Live causal boundary

Epiphany `55660d78`, Bifrost `cb3239aa`, CultLib `f67f5122`, and CultNet
`2d0988ba` implement and deploy the Starfire-to-Yggdrasil Persona nerve. An
already-durable signed `epiphany.persona_discord_delivery_request.v0` now
travels over a Starfire-initiated CultNet/RUDP session; Bifrost admits it to
its endpoint store, retains permit/journal/Discord authority, and returns its
signed terminal `bifrost.persona_discord_delivery_receipt.v0` on the same
session. Epiphany verifies and durably stores that receipt before terminalizing
the Persona turn. A real Rust-to-Node socket smoke passes, as do all nine
focused Bifrost delivery tests. The shared transport fault was an old Rust ACK
spending an ordered data sequence; CultNet commit `2d0988ba` ports the upstream
sequence-neutral ACK correction. Runtime identity is now deployment-bound to
`epiphany-starfire`, not hardcoded as `epiphany-yggdrasil`.

Yggdrasil now runs both native feedback and Persona mouth services from the
exact immutable Bifrost/CultLib runtime. Starfire runs the packaged permit
service from the exact authenticated Epiphany release. Public trust anchors
crossed hosts; signing seeds, Discord actuation, and private execution state did
not. The first live Persona request reached model terminal and durable request
state, then its expired recovery produced signed terminal `unknown` without a
second post. It is sealed and may not be retried.

Live probing then found two cross-language/authority seams. Bifrost had encoded
the permit request as a camelCase map while Rust requires a positional struct,
and decoded Rust's positional permit as an object. Commit `514c2a6` aligns both
wire bodies; a unique permit-only Node-to-Rust-to-Node request now completes.
The second fresh Persona turn reached model terminal and consumed a permit, but
Bifrost called its Discord bridge without the required CultMesh command ID.
The bridge refused the unreceipted actuation and Bifrost correctly signed
terminal `unknown` with no message ID. Commit `cb3239aa` binds the bridge command
receipt to the signed Persona request ID and is live.

Yggdrasil has no `/srv/bifrost/env/persona-delivery.env` and no
`BIFROST_DISCORD_BOT_TOKEN` or compatible token in managed secret roots. No
third Persona request may be created until an operator-owned bot token is
installed root-only and the mouth restarted. The two existing requests are
sealed non-deliveries and may never be retried. Windows still retains
per-release firewall rules; the admitted prior authenticated permit binary PID
`19024` carries diagnostic traffic, while `gamecult-ops` `e52c7dc` contains the
durable path-independent UAC repair.

The operator expects proactive operator-enginseer intervention in corrupted
Epiphany runtime state; corruption is not a permission boundary. Repairs must
be labeled, receipted, preserve immutable worker evidence, and use exact
compare-and-swap revision authority. Record the authoritative before state,
the exact intervention, and verified after state. This standing repair
authority restores Epiphany's own invariants; it does not bypass typed gates
for new Discord, deployment, or other external consequences.

Commit `7093a8b9` supplies that narrow primitive for the legacy accepted Modeling
result which predated the typed future-frontier invariant. Revision 49 was
sealed before correction at
`sealed-evidence/v53-pre-modeling-acceptance-correction/runtime-revision-49.cc`,
76,172,850 bytes, SHA-256
`5ec47d8da386976dbb8ca1b18f214507de3c14eb4f02d31bdc3aa60d99e612f3`.

Correction
`supervisor-acceptance-correction-e2ddbf239ea45081138ad9f10089f6895887fe14130b431aab0fef1d1978004c`
removed only obsolete acceptance
`accept-modeling-result-worker-4e6b6022-73ab-49f1-81c8-03dddfceb29f`, retained
its immutable result and admitted RepoModel, and recorded prior receipt hash
`2f1af810e77997e2bf235e779609da62ec1a5148107d81a342288005e80df510`.
The old result was then reviewed under current policy and superseded as
`role-failure-review-a35d6548-ec90-48c6-80d5-84ac6e32c619`.

Fresh Modeling job `ca61f378-4998-4911-a565-06d77388bb4a` was accepted as
`accept-modeling-result-worker-ca61f378-4998-4911-a565-06d77388bb4a` with
evidence `ev-modeling-4a0caaa3-9172-44eb-a9e9-9ea68c6406fc`. It minted exactly
one active typed Imagination frontier,
`frontier-native-frontier-minimal-route-chain-design-20260808`.

Self routed that frontier over the stale display-level `regatherManually`
recommendation:

1. planning request `repo-frontier-planning-7adf9c5863cd3ca640086f9aafd1abd910aa8d28e40dcc6e08835a20af60a9aa` at revision 53;
2. completed Imagination job `2a5701fe-7fbe-4f29-b3a5-51d6e790acbd` and typed candidate `repo-frontier-plan-candidate-02c089eb385c27560252d1915b998efc7c800324971a7b915b6e9ff3d33e4646` at revision 54;
3. dedicated Mind request `repo-frontier-plan-mind-7c8f086d050fe458a298ea4d9f5202e446530e5e14fa10129f52b4ae72238bc3` at revision 55, without adoption or Hands authority.

Mind adopted the candidate, Self materialized the exact route and scoped Hands
authority, and Hands committed `29797ab8` with typed patch, command, and commit
receipts. The next live typed action is `launchVerification`.

The first Soul launch failed before worker creation because
`append_verification_hands_receipt_context` treated every admission with a
`result_id` as Modeling. The route admission actually binds Mind plan result
`result-worker-c0815013-7c61-4872-8549-4b666ceb015a` and explicit decision
`repo-frontier-plan-decision-22de32456ea73d63f613e69e8fdca876f5068f06febdc04d9f847da18af65a6e`.
The fresh Modeling acceptance remains present exactly once; the runtime did not
need correction. Pre-repair snapshots are
`.epiphany-run/v53-live-thread-acceptances-pre-repair.json` (SHA-256
`7f22d49b5b06fa84e5fdfce6066403fab88de6526cc82e4726e9d1343f400b46`)
and `.epiphany-run/v53-live-route-admission-pre-repair.json` (SHA-256
`df9002fd97d0ecfaf0750c41f397b0f560062132564b8b677d51ca70344518ac`).

The active source repair types all new plan admissions as
`FrontierPlanDecision`, makes Soul validate the immutable decision receipt, and
uses the already-persisted `frontier_plan_decision_id` as the compatibility
discriminator for v53. It does not rewrite evidence or weaken Soul.

Authenticated release `sha256-53dc65ecf1971fc39a535c3a597723e02ebe73a28936581441b4fe007c98c601`
from `d2538d46` proved the typed branch live, then failed closed because its
first exact-binding predicate incorrectly equated the decision's pre-adoption
frontier hash with the route's post-adoption hash. The follow-up source repair
binds the decision to the pre-admission model, the admission purpose to the
planning request and candidate, and the route to the admitted model. A focused
positive/negative test proves the real transition passes and a substituted
admitted route hash fails.

Exact `d3bfbda0` packaged in 29.84s for the stable phase and 5.43s for the
isolated Self/coordinator phase, then authenticated and published. Live
Verification job `f23fe2b6-ad39-41e7-a2c9-4c26d9f5f26c` launched without
overrides, completed in roughly 700 seconds, and was accepted as
`accept-verification-result-worker-f23fe2b6-ad39-41e7-a2c9-4c26d9f5f26c`
at thread revision 57. Soul verified exact Hands commit `29797ab8` and bounded
the claim to the documentation-only route-chain specification. A read-only
coordinator planning pass now derives `launchModeling` because Soul accepted
the Hands consequence.

Post-Soul Modeling job `972f4e31-9009-4b7d-863d-b2aa0295abf7` launched at
revision 58. Its live launch timings were state load 79ms, dynamic context
66ms, role augmentation 1.053s, job commit 51.482s, total 52.680s. The worker
produced exactly one verdict-incorporation patch; Mind accepted it as
`accept-modeling-result-worker-972f4e31-9009-4b7d-863d-b2aa0295abf7` at
revision 59. The frontier is resolved and the Soul-to-Modeling route is
consumed. Self now falls back to the legacy CRRC `regatherManually` projection;
do not use that historical pressure to reopen this completed frontier.

## Build-time finding

Commit `0f0b006d` made the three named owner manifests drive separate locked Cargo
builds. Focused tests passed, but the real benchmark falsified the design:

- the new aggregate owner-lock identity forced a cold graph namespace;
- the core owner group took 8m02s;
- the OpenAI runtime group then resolved its own dependency graph and began
  compiling `epiphany-core` a second time;
- the run was terminated at 16m53s before the tool-runtime group could repeat
  the wound;
- logs are `.epiphany-run/package-0f0b006d-owner-groups-attempt2.*`;
- pushed commit `a1a43892` reverts the implementation.

Do not resurrect separate owner lock universes. The local replacement retains
the one root Cargo graph and moves routing policy into `epiphany-self-policy`.
Stable coordinator contract types remain in
`epiphany-core/src/surfaces/coordinator_contract.rs`; core no longer compiles or
re-exports policy functions. Core explicitly retains all 74 non-coordinator
binaries. The root coordinator target alone enables optional feature
`coordinator-runtime`, which pulls the policy crate; ordinary release binaries
do not see it. Packaging builds ordinary bins under the root lock, then builds
only the feature-gated coordinator in the same target cache. Cache identity is
target/toolchain-stable across lock edits; Cargo owns staleness while the frozen
lock and release witness remain authority.

Exact `d910158f` is committed, pushed, authenticated, and published to
`.epiphany-run/cultmesh/local-verse.ccmp`. Release
`sha256-2d6672958cd59cae43bbbc17c8218eaf3d50f3acab3b7d6550cb6dfad5f72020`
has witness
`sha256-23d3f44ef0f632dfc52a607585469b7242ac77a489e1406d7b3e2f578979462d`.
The cold stable graph took 17m38s; the isolated policy/coordinator phase took
30.31s. A controlled warm invalidation of the policy source rebuilt exactly
`epiphany-self-policy` and the root coordinator in 6.02s (6.133s wall). All 20
stable packaged executables retained identical hashes, lengths, and write-times;
the coordinator write-time advanced. The source-cache timestamp used to trigger
Cargo staleness was restored after the measurement. The earlier 1.21s no-op
probe reported `Removed 0 files` and is rejected, not evidence.

## Current performance evidence

- Exact `c35272c9` warm packaging of 21 binaries: 2m34s.
- Release-publisher bootstrap after a core edit: 4m11s.
- Exact `d910158f` cold stable graph: 17m38s.
- Exact `d910158f` isolated coordinator phase: 30.31s.
- Warm Self-policy invalidation: 6.02s; zero of 20 stable packaged executables relinked.
- Fresh Modeling context: state load 121ms, dynamic context 107ms.
- Fresh Modeling job commit: 51.637s; coordinator total 51.865s.
- The launch wound is the whole-store transaction, not context assembly.

## Keyed runtime and packaging cut

The current uncommitted pass after `b070e121` introduces one runtime-spine
backend owner selected by explicit extension: `.cc`/`.msgpack` retain the
sealed legacy snapshot implementation and `.redb` selects CultCache's keyed
redb implementation. All 29 runtime-spine CAS construction sites and the
coordinator state transaction now resolve through that owner; unrelated stores
retain their existing authorities. Focused backend parity proves successful and
stale CAS on both formats. All eight coordinator transaction tests pass.

The one-time migration refuses an existing destination, verifies unique typed
identities, writes source envelopes plus a typed in-store migration receipt in
one empty-destination CAS, reads back exact envelope equivalence, and rehashes
the source bytes to prove the legacy evidence was not changed. A disposable
migration of live v53 copied 9,376 envelopes from source SHA-256
`78eb340a028c88d42770345518d810841029411510a2abab233d9a89761dd5e4`; the
sorted envelope-set SHA-256 is
`bbe288e348ab3a6c8af6d64d205fe818a59218988a493f9751ed65588f2b338e`.
Artifacts and the receipt are under
`.epiphany-run/storage-benchmark-20260808-2/`; authoritative v53 was not
mutated.

Release-mode live-sized measurements are mixed but promising: legacy full read
563.70ms versus keyed 718.18ms; legacy event mutation 896.91ms versus keyed
363.61ms. Debug redb timings were misleadingly slow and are rejected for
production judgment. Do not migrate live from this proxy alone. Measure the
exact coordinator launch replacement set against the keyed v53 copy next.

That next probe changed the diagnosis. The durable
`epiphany-runtime-store-benchmark` selects typed historical transaction
envelopes by exact job identity and emits identity/digest/timing receipts on
disposable stores. The surviving launch-request batch for job `972f4e31` was
legacy 785-1,012ms versus keyed 23-32ms; its completion batch was legacy
784-1,567ms versus keyed 21-34ms. Those receipts prove keyed CAS is faster, but
also falsify the claim that snapshot CAS owned the measured 51.482s launch
bucket. Do not migrate live on that old causal story.

`commit_coordinator_job_launch` performs Modeling-only
`observe_runtime_repository_body_basis` before opening the state transaction.
That observer constructs two isolated Git trees and raw SHA-256 manifests over
3,994 files, then requires exact equality. Direct authenticated release timing
was 92.30s and created Body generation 2. The fresh-index Git phase was only
1.62s; raw file reads and metadata were the wound. Bounded parallel manifest
construction keeps the same per-file before/after metadata checks, raw hashes,
sorted root, isolated trees, and second full equality pass. All 24 observer
tests pass. Direct release observation fell to 49.52s and created generation 3.
This is material but not final; package it and measure a real launch before
claiming the original 51s path is cured.

Packaging selection was tested by replacing Cargo `--bins` with the explicit
authenticated sibling set while leaving the feature-gated coordinator in its
existing second phase. The focused 17 packaging tests passed. A deliberately
detached release benchmark that missed the shared target directory took 4m41s,
proving that target-cache location must remain tool-owned rather than operator
memory; the authenticated packager already owns a stable graph cache explicitly.

The real exact `9d671aee` package falsified that selector hypothesis. The first
phase still took 7m26s because root `--bins` already named the 20 stable release
bundle targets; it never linked core's 74 diagnostic binaries. Explicitly
listing the same 20 targets did not reduce fan-out. The change is reverted in
source. Core compilation remains the build-time owner. Exact `9d671aee` is
nevertheless authenticated and published as release
`sha256-6de91b234bdfda1149e48b9fef6710bdd8c4e3274981e4aa88fc46ec109a2e56`
with witness
`sha256-f2c0e464b1523b1a14fb500afa747f0dd95166ebd068752bf639a43827c6b9ba`.

## Open readiness work

- cut the measured 51.482s whole-store Modeling job-commit seam;
- prove native Persona cognition and external speech consequence;
- prove Continuity crash, restart, session closure, and bounded retention;
- measure long-duration resource plateau;
- profile and remove the Modeling whole-store commit path;
- prove Linux cognition on Starfire, then size Yggdrasil from measured demand.

The current Yggdrasil body was inspected read-only at
`2026-08-08T22:12:11Z`. It reports 2 vCPUs, 1.9 GiB RAM, 2 GiB swap with
516 MiB already used, 1.3 GiB memory available, and 14 GiB free on the root
filesystem. `bifrost.service`, `bifrost-persona-feedback.service`, and
`bifrost-persona-mouth.service` are active. Epiphany, its heartbeat/swarm, and
Idunn are absent, matching the current authority map. Yggdrasil may receive a
prebuilt bounded Linux smoke using isolated disposable state; it is not a Rust
build host, and the apparent available memory is not residency evidence while
the host already carries swap pressure. Build on Starfire and measure the
actual Linux process footprint before proposing a permanent Yggdrasil body.

Linux physiology is now proven without installing the Epiphany service body.
Starfire built only `epiphany-runtime-spine` from exact commit `b5140288` in
the deployment actuator's digest-pinned Rust image
`sha256:4c2fd73ef19c5ef9d54bee03b06b2839a392604fbfcd578ed948b71b37c1d7fb`.
The cold build took 4m25s and peaked near 1.77 GiB inside Docker, confirming
that Yggdrasil must not compile this crate. The dedicated Starfire Cargo volumes
then produced an unchanged warm rebuild in 1.18s. The 6,181,968-byte Linux
binary is
`sha256-14143382fdfb5eb40919facd4bc681613d1a2563e18af08b522d0e5a097880f3`.

The exact binary ran on Yggdrasil only from `/tmp` against a new disposable
CultCache store. It initialized the typed runtime catalog at 3,968 KiB peak
RSS, refused session closure while a job was open, admitted one terminal
Continuity result, closed the session, repeated closure idempotently, refused a
post-close job, and reported one completed session/job/result with three events
at 4,352 KiB peak RSS. The store was 5,465 bytes with SHA-256
`f9cccce0b64bb4b282a1fda208d346177cf1a5f91f0e02bbf4f6520c985115cc`.
The script removed the binary, store, root, and itself; a follow-up assertion
proved all four paths absent and the three Bifrost services remained active.
The retained successful receipt is
`.epiphany-run/linux-spine-b514/yggdrasil-success.receipt`, SHA-256
`5dbf03cbb2e4633464055de888f8417fbd7ad5cfdb4f04ebef34ab53790f5c00`.
The first invocation failed before CultCache initialization because the host
lacks the optional `file` utility; that decorative probe was cut. Its receipt
is retained beside the success receipt at SHA-256
`b5a55ec37c258ead9ecc745e77c16bb4fa83373b8462e15ff04fd4d8ff659ebe`.
This proves Linux typed physiology and Continuity portability, not Linux model
cognition or resident readiness.

Basic Linux model cognition is now proven on Starfire. Exact pushed Mind
`a0d56f94` produced a 27,767,120-byte `epiphany-model-runtime` in the same
digest-pinned Rust image. Its SHA-256 and native preflight witness are
`c72f33ed3b429c201eadd9a82b70a71ebce0a6d49901e13bfbefb533e3a972f1`
and `openai-runtime-preflight-c72f33ed3b429c20`. The first real request
completed through `epiphany_direct_responses_http` in 2.60 seconds, emitted two
typed events, returned exactly `Linux cognition is awake.`, and recorded no
tool intent. A fresh measured request completed in 2.26 seconds, returned
exactly `Measured Linux cognition is awake.`, and reached 20,480 KiB peak RSS
across 141 `/proc` samples at 10 ms cadence. Its typed store is 19,574 bytes,
SHA-256 `582b85cbf3c25c4bd23c6583bf43dcca1e2cda8a6956b9294fb441204d8869c2`.
Native preflight reread that store and proved the model request, stream event,
model receipt, and OpenAI receipt document types with no private state exposed.
The auth file was mounted read-only from Starfire and was not copied into the
smoke artifact root.

The model build exposed a separate iteration defect. The root workspace and
`epiphany-core` manifests resolve version-skewed lock graphs, so the warm core
target did not prevent a second cold dependency compilation. Model-runtime took
10m57s cold and exceeded 4 GiB during parallel compilation. Once that root graph
was populated, the exact warm build took 6.04s. The immediate operational cure
is to preserve both explicit Starfire caches; the architectural cure is one
workspace/lock owner and organ crates small enough that a model-transport change
does not relink the whole organism. Do not hide this split with another cache
alias.

Receipts live under `.epiphany-run/linux-model-a0d56f94/`. Important hashes:

- measured summary:
  `sha256-fa4b4fc15238fdcf50b2f88430017a0a54430ce6eb905410129aa8f4ff966168`;
- measured assistant text:
  `sha256-dfa5ba74b9b51b324847383f8c64e38963439847c32ecce621f1262333503d13`;
- process metrics:
  `sha256-702e68b03355a8a8cfeb32bcbebe7154ab7ad3c50ee0c5c0698f30eb70090a71`;
- the rejected object-shaped request failed before opening transport or creating
  a store and is retained as
  `sha256-0c48d6546e3aad1c13f581ece988fcbdeefd4123023433bfabd11b94590f7f59`.

This proves individual Linux model transport and cognition. It does not prove
resident Self/heartbeat coordination, aggregate resident footprint, restart,
or Yggdrasil residency.

Do not recreate prior Eyes, Hands, Soul, or Modeling work. Do not feed the old
manual-regather loop. Use coordinator projections and receipts; raw worker
thought remains sealed.

## Persona consequence boundary

Persona is live across both hosts but has not yet completed a public consequence.
Starfire owns temporary Epiphany cognition and the five-second permit;
Yggdrasil owns Bifrost, the Discord binding, the actuator, the private execution
journal, and the signed terminal receipt.

Current physical state:

- Yggdrasil runs `bifrost-persona-feedback.service` and
  `bifrost-persona-mouth.service` from exact Bifrost `cb3239aa` / CultLib
  `f67f5122`; the mouth is bound to WireGuard `10.77.0.1:17876`.
- Starfire feedback ingress uses immutable CultNet snapshot export/import; it
  never copies live Bifrost `.cc` state.
- Starfire permit PID `19024` runs the prior authenticated Epiphany release at
  the Windows-admitted path and binds `10.77.0.2:17877`; exact `55660d78`
  remains published but blocked by Windows' per-release program rule.
- Request, receipt, and permit-request identities are purpose-specific. Only
  public anchors crossed hosts; private keys remain with their owners.
- The first live request reached model terminal, persisted on both sides, and
  later reconciled to signed terminal failure after its permit intent expired.
  It did not automatically repost and must never be reused.
- The second live request obtained a valid permit, then terminalized signed
  `unknown` when the bridge rejected its missing CultMesh command ID. It has no
  Discord message evidence and must never be reused. The source fix is live.
- Unique permit-only probing completes across Node/Rust; the reusable probe no
  longer collides with its own replay key.
- Yggdrasil lacks the token-bearing Discord credential required for actuation.
- Windows currently blocks inbound traffic to the new exact permit executable
  with automatic per-program rules. The path-independent scoped repair is
  `F:\Projects\gamecult-ops\scripts\allow-starfire-epiphany-persona-permit.ps1`.

Install an operator-owned bot token as root-only
`/srv/bifrost/env/persona-delivery.env`, restart the mouth, and verify only the
credential's presence. Then queue one fresh uniquely identified Persona mention
and prove model terminal, signed request, permit, Discord message ID/URL, bridge
crossing receipt, signed Bifrost receipt, Starfire admission, and heartbeat
terminalization. Transport acknowledgement is never speech success. Do not
SFTP live stores, share private keys, run Bifrost actuation on Starfire, weaken
the permit, or retry either sealed request.

## Continuity retention audit — 2026-08-09

The source-grounded retention authority map is now in
`notes/epiphany-current-algorithmic-map.md`. Heartbeat pulse directories already
have typed plan/receipt retention and in-store heartbeat history is capped at
128. Resident Self pressure/grant/claim/ack lifecycle rows and runtime-spine
coordinator receipts remain unbounded. The old `ResidentSelfRuntimeReceipt`
shape is produced only by test-only reconciliation and is not a live target.

Resident lifecycle retirement may touch only a fully closed lifecycle after
Heartbeat has consumed its terminal acknowledgement. It must atomically delete
exact witnessed envelopes, publish typed digest/count evidence with no
scheduling authority, and refuse pending pressure, unconsumed grants,
prepared/active launches, unconsumed acknowledgements, and brake/shutdown
cancellations whose pressure was requeued. Negative proof must show retained
history cannot create pending work or resurrect a coordinator turn.

Coordinator receipt deletion remains forbidden until resident/current-frontier
and recovery references are mapped. Supervisor stdout/stderr truncate at
launch but grow without bound during a long-lived process; the fatal log grows
across launches. Do not truncate beneath inherited child file descriptors.
`E:\Projects\gamecult-ops` is unavailable in this workspace, so log ownership
must wait for the infrastructure runbook rather than being guessed.

Next: implement the resident lifecycle retention organ and its negative tests,
then audit coordinator references, packaged Heartbeat retention, supervisor log
ownership, and the measured idle/active endurance plateau.

Resident lifecycle retention is now implemented in source. Resident Self's
serve/once owner invokes a configurable `--retained-closed-lifecycles` bound
(default 256). Only Heartbeat-consumed terminal lifecycles whose pressure still
binds the consumed grant and which have no prepared/active lease are eligible.
The exact pressure/grant/ack/claim envelopes are deleted in the same
snapshot-fenced CultCache write that replaces the single retention head. The
head carries only cumulative counts and a chained digest; scheduling readers do
not consume it. Requeued brake/shutdown/failure pressure remains live.

CultCache commit `fecc17c` on
`codex/resident-retention-compaction` adds the missing single-file atomic
replace-and-delete primitive and its stale-snapshot refusal proof. Twenty
resident Self tests and six swarm binary tests pass, including negative proof
that retention cannot create pending pressure/grants/acks or resurrect state.
Package and copied-state proof remain before this slice is accepted.

Exact `ed15f83a` packaged in 7m26s after one failed-closed cache repair. The
first attempt found a stale cached source worktree whose `git-common-dir`
pointed at a deleted prior clone. Only that exact
`source-fb4097c8f489c830678f32a39e818cf462bff5c1fe853bc12d106184c9bfd571`
directory was removed; Cargo home and target caches were preserved. The
replacement package produced 24 binaries, release
`sha256-4a4be03688eab1d1f7c13ac0057f0664c4658cab88471a0eac451a1d4607b85d`,
and witness
`sha256-f163170174f022c124cc2f94d195639d02f65b79bac229af3efb8333176673c4`.
Independent packaged inspection accepted exact source, inventory, witness, and
private-state exclusion without warnings.

Receipts are in `.epiphany-run/resident-retention-ed15f83a/`. Under an engaged
copied-state brake, the exact packaged Self compacted a portable fixture with
two closed lifecycles and one requeued control. The head is revision 1,
`retired_lifecycle_count=2`, `retired_envelope_count=6`, and chained digest
`sha256:69a2c5971430846590fe20ba32a1a9299726a51c997b9e9c07ec044ebbff34a6`.
The requeued control plus one independently ingested pending typed pressure
survived; grants and acknowledgements are empty. A second exact packaged pass
left the head unchanged. Active c005 remained running and untouched.

Resident lifecycle retention is accepted. Next map coordinator receipt and
semantic/runtime history references before choosing the next compaction owner.
