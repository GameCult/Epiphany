# Fresh workspace handoff

Epiphany remains a supervised engineering alpha. Starfire is the cognition and
release forge; Yggdrasil is the small live crossing host and is not a build
machine at its current memory budget.

## Authoritative state

- Branch: `codex/epiphany-shakedown-live`
- Pushed HEAD: `d3bfbda0c55a2295672d2c7b45c6388cc9ca9c84`
- Authenticated cognition commit: `d3bfbda0c55a2295672d2c7b45c6388cc9ca9c84`
- Release: `sha256-4bb5f67287f49d85fbc819c3c63a861cda41415d7fc8449386816cf9c59e503b`
- Witness: `sha256-e7f6c5f97b270c324b0d6109768226fd0c15415784993cb497abe6b841c36d58`
- Live workspace: `F:\Projects\.epiphany-runtime\shakedown\live-20260808-v53-hands-precedence`
- Thread: `shakedown-v49-hands-relinquishment-r1`
- Thread-state revision: `59`
- Sealed evidence: v49 through v52. Do not mutate it.

## Live causal boundary

The operator grants standing authority for bounded supervisor repair of
corrupted Epiphany runtime state. Repairs must be labeled, receipted, preserve
immutable worker evidence, and use exact compare-and-swap revision authority.

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

Do not recreate prior Eyes, Hands, Soul, or Modeling work. Do not feed the old
manual-regather loop. Use coordinator projections and receipts; raw worker
thought remains sealed.
