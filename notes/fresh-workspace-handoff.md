# Fresh Workspace Handoff

## Organizational Yggdrasil deployment is live and braked (2026-07-19)

Epiphany's ordinary organizational product path is now deployed on Yggdrasil,
not merely proven in local smoke. Bifrost authorized each exact upstream
release and Idunn produced successful deployment receipts for Epiphany
`bcff32827a39ca793c3062cc71e02aeeac1b091f`, Bifrost
`6491f449cf0dbfa952b409de90ccdf511669a60b` with CultLib
`693df0901d75cfd8e3a0a5225e270011eeddb0be`, and VoidBot
`26093ace0e4b5d15370bd29e3118fb27c807afa0`.

All Epiphany resident services, Idunn, and VoidBot are active. The Bifrost
Persona-feedback sidecar is healthy under its own `bifrost-feedback` service
identity; it owns its private provider and delivery state, participates in the
shared CultCache lock groups, and publishes daemon-owned CultNet/RUDP health
over host networking so Idunn observes Ygg-local provenance instead of Docker
bridge NAT. The Bifrost Epiphany operator worker is running against the
loopback-only Epiphany command service on port 17875. VoidBot and Bifrost now
share the exact v1 six-command request identity; the live v0 request
substitution was deleted. Legacy v0 replay is bounded to the original four
commands. The final root-level Ygg checker passes.

Resident readiness is active and release-authenticated with
`brakeEngaged=true`. Do not release it as part of verification. Ollama exposes
`qwen3-embedding:0.6b` at 100% GPU placement on the GTX 1080; both Modeling
semantic projectors point at the Ygg-local Qdrant/Ollama path.

The Epiphany release id is
`sha256-e0a9eb079250d3f769040255f3d2805286488896c6e2c38222748d2298daf913`
with witness
`sha256-dd84cfeb730061ee38edae5d8de104b65e1cb2f3c723d62e087b68974eac814e`.

This promotion exposed an Idunn packaging actuator fault before outage:
`mktemp` pre-created the path later passed to Docker as `--cidfile`, so Docker
refused the run before any service mutation. Ops commit `71beacf` now gives
Docker a nonexistent child path inside a fresh temporary directory and cleans
the exact recorded container. The first two Epiphany promotion attempts failed
closed; the corrected retry admitted the exact release above. VoidBot's audited
artifact is `c513c8591baa931fd47b5ee5e29578f2af58af49637d4a88a44b4bda63821af8`
and its Idunn release is `20260719T182219Z-26093ace0e4b`. The final root checker
passed with no failed units and the canonical brake still engaged.

The next proof is organizational interaction, not another deployment: use
Discord to request `/epiphany status` and submit ordinary repository feedback
to the Epiphany Persona, then verify only operator-safe typed projections and
receipts. Do not test Wake while the deployment brake is the intended owner of
sleep. Conversation remains feedback pressure; it does not adopt Mind state or
grant Hands, release, or deployment authority.

Do not expand Discord Status with the current Idunn/Bifrost health packet. The
provider audit found that generic `idunn.daemon_health.v1` is unsigned: Idunn
checks typed shape, daemon id, contract, self-declared publication source and
transport, and freshness, but no pinned publisher identity or signature. Idunn
also exposes no authenticated outward projection for Epiphany to consume. The
deployed Odin lineage is the clean `E:\Projects\Odin-yggdrasil-idunn` worktree,
not the unrelated active Odin branch. Odin commit `cc80cd4` adds the generic
`idunn.signed_daemon_health.v1` and root-owned
`idunn.daemon_health_trust_binding.v1` contract foundations plus
`docs/idunn-signed-daemon-health-authority.md`. Next, wire generic Idunn
verification and monotonic admission, migrate Epiphany and Bifrost publishers,
publish an Idunn-signed outward projection, and only then let Discord Status
compose it. Unsigned legacy health may remain diagnostic during migration but
must lose managed-health authority before this rebuild is complete.

### Unattended continuity aftercare

The Ygg body has no live Starfire reference. Epiphany, operator, heartbeat,
resident Self, Idunn, VoidBot, Docker, WireGuard, SSH socket activation, and the
new authority-backup timer are enabled for boot; all current services are
active and all relevant containers use restart policies. Seven obsolete failed
transient Idunn build units were cleared so `systemctl --failed` is meaningful.

The swarm unit no longer overwrites persistent refreshed Codex auth on every
restart. `/etc/gamecult/epiphany/credentials/codex-auth.json` is a cold-start
seed copied with `cp --update=none`; `/var/lib/gamecult/epiphany/codex-home`
owns later provider refreshes. The runtime file is forced to mode `0600` so
Codex can refresh it in place, and resident readiness now requires private
owner read/write rather than accepting an unwritable `0400` seed. Only
canonical `auth.json` satisfies Ygg readiness; legacy `credentials.json` cannot
stand in for an absent canonical credential. Operator config remains
operator-authored.

`gamecult-authority-backup.timer` now seals one validated, SHA-256-addressed,
root-only recovery archive daily with 14-day retention. The exact Epiphany,
Bifrost, and VoidBot state writers are briefly frozen or paused while the
archive is opened. Idunn is deliberately not frozen: live proof showed
freeze/thaw leaves its process alive while killing its RUDP receiver. Missing
writers fail the run, and publication is refused
unless every quiesced writer is confirmed active and thawed/unpaused. Strict
snapshot `20260719T173400Z` passed checksum and tar traversal while Idunn kept
accepting provider health. It contains Bifrost release authority plus Idunn
config/trust/provenance; Idunn's live observation/command state is excluded and
reconstructed from providers after restore. Older unsafe snapshots were deleted. Release
trees, logs, media, RAG/vector projections, and Qdrant remain excluded. This is
local logical-corruption recovery on clean two-disk RAID1, not off-host backup.

Idunn's live process is now self-proven by a root-sealed manifest: exact Odin
commit `15a744c826f0aa2e8cc322a6543ea8a9afcd852a`, digest-pinned Rust image,
build artifact, installed binary and privileged surfaces, and the executable
behind the live PID. Every installed surface is root-owned at its exact safe
mode, and systemd reports the sealed unit/drop-in loaded with no pending reload.
Verification owns no lifecycle or deployment action. The
final Ygg checker requires this proof and passes with the brake engaged.

The provenance manifest disappeared after the later Bifrost promotion while
the exact installed Idunn binary, matching build artifact, root-owned surfaces,
and live PID remained unchanged. It was resealed from canonical Odin commit
`15a744c826f0aa2e8cc322a6543ea8a9afcd852a` and the pinned Rust image without a
rebuild, service restart, or deployment action. The full root-level checker is
green. Run this checker with root authority: its proof intentionally reads the
`0440` Idunn sudoers surface.

Restarting Idunn to repair the frozen RUDP nerve also proved that Docker moves
the package builder outside Idunn's systemd cgroup: the interrupted build
container survived beside the retry. The exact orphan was stopped and its
worktree removed. Ops commit `391842e` gives every future Epiphany packager a
CID file and force-removes that exact container in the actuator EXIT trap; the
installed Idunn manifest already contains the correction.

The host reports a pending reboot for installed kernel `6.8.0-136`; it still
runs `6.8.0-134`. No reboot was performed or authorized. Schedule that as an
explicit operator maintenance window before departure if desired; deployment
and readiness must never smuggle it in.

## Swarm brake observation/actuation purification (2026-07-19)

`epiphany-verse-query swarm-brake` was a mutating pseudo-query whose missing
`--brake-status` defaulted to `released`; a read-only audit therefore released
the live Ygg brake. The old command is now disabled. `swarm-brake-status` is a
read-only projection over the one runtime-scoped CultMesh key
`epiphany-local/swarm-brake`; tests prove its store bytes remain identical.
`swarm-brake-set` is the only CLI mutation surface and requires explicit
`engaged|released`, canonical document id `epiphany/swarm-brake`, actor, and
reason. The document's stable owner is `epiphany.swarm-brake`; actors remain
provenance instead of repainting authority on every command.

Release refuses absent and foreign/legacy identities. Readers continue to
honor an engaged legacy head. Idunn may explicitly adopt an already-engaged
legacy head during deployment, with no released interval; non-Idunn actors and
Discord Sleep cannot. Discord Wake releases only the canonical id+owner.
Resident readiness reads the same key. The ops actuator now engages that
canonical brake before service restart and deployment readiness requires
`brakeEngaged=true`. No Ygg service was deployed or restarted during this cut.

## Typed Discord Mind review owner (2026-07-19)

The operator-command core now implements `Reviews` and `Review` without a
parallel review store. `Reviews` is a bounded identity-only projection of
current canonical `RepoFrontierPlanMindRequest` records. `Review` binds request,
candidate id/digest, expected RepoModel revision/hash, and exact
Adopt/Refuse/Hold. Mind revalidates and commits the existing decision owner;
Hold/Refuse are terminal inert receipts and Adopt uses the existing RepoModel
CAS. It creates no Hands/Substrate, Persona, release, or deployment authority.

Decision provenance is typed (`MindWorker` or
`AuthenticatedOperatorReview`). RepoModel review/admission provenance is typed
separately (`WorkerResult` or `FrontierPlanDecision`); legacy worker ids are
Options and operator adoption writes None. Crash replay derives decision time
from packet `issuedAt` and recognizes only exact typed provenance. Candidate
mismatch is a terminal refusal; storage/corruption/CAS faults propagate. Actual
CultCache v0 tuple decoding and signed v0 delivery replay are covered; v0
cannot acquire Reviews/Review.

The service CLI now requires the canonical runtime-spine store explicitly and
returns sealed Discord-safe review projections. No successor is deployed.
Bifrost/VoidBot must advance to the v1 schemas before the Ygg rehearsal; brakes
remain engaged.

## Organizational product posture and current release blockers — 2026-07-18

This is not a special dogfood experiment. Epiphany's normal organizational
product is a resident swarm that persistently models its authorized domain,
derives bounded direction from that map, autonomously imagines improvements and
features, and exposes a repository Persona through which the organization can
converse and provide attributed feedback. Instructions are optional pressure;
they are not the only source of useful work. Feedback is likewise pressure, not
Mind adoption, Hands authority, Bifrost release authority, or Idunn deployment
authority.

The signed VoidBot -> Bifrost -> Epiphany feedback path and the resident
Self -> Imagination proposal-only path are implemented locally. The four
structural release blockers identified in the first Soul audit are now closed
in source and local proof:

- admitted feedback now lives in a dedicated `persona-feedback.cc` store, not
  shared local Verse; classification is exact and Imagination has no direct
  Persona-reply route;
- Eve connection/consensus intent and receipt authority, writers, projections,
  tool claims, tests, and smoke binaries are deleted;
- Bifrost sidecar release is exact-authority/digest bound, Node is digest-pinned,
  Idunn alone invokes deployment, and daemon-owned typed RUDP health feeds the
  existing Idunn supervisor target;
- `scripts/smoke-persona-feedback-org-path.ps1` passed the real VoidBot writer ->
  Bifrost sign -> Rust ingress -> resident consideration -> coordinator launch ->
  proposal validation path for public, organization, and private feedback while
  Mind, release, local Verse, and public stores remained byte-identical.

Live Yggdrasil deployment and the Starfire-offline Discord rehearsal remain
pending; source proof is not deployment proof.

Do not describe this as an experiment-only capability after these blockers are
closed. The bounded autonomous loop is the product.

## Workspace coverage storage cut is founded, not migrated — 2026-07-17

Live Yggdrasil evidence rejects `c94fa580` as a deployment candidate. During
that candidate, `body.cc` was rewritten by workspace-projection activity rather
than repository observation. The rewrite is attributable to the surviving
coverage writers sharing the Body store; it is not evidence that the Body
observer changed repository substrate. Body owns observed substrate only:
binding, immutable observations/manifests, and the current Body head.

Commit `261c7bc8` lands the replacement foundation. A runtime-side immutable
`epiphany.runtime.workspace_coverage_store_binding.v0` route and store-local
`gamecult.epiphany.workspace_coverage_store_binding.v0` bind a separate
transactional `workspace-coverage` store to runtime, swarm, workspace,
canonical path and file identity, exact repository Body route/envelope,
Body-binding hash, and repository source identity. Transactional keyed
CultCache admission is pinned beneath it. This is routing and storage
foundation, not completion: obligation, plan, claim, attempt, recovery,
checkpoint, progress, terminal receipt, coverage head, and retirement writers
have not yet been migrated from Body/local Verse.

Authority map:

- Owner: the workspace-coverage store owns projection lifecycle and proof;
  repository Body owns observations only.
- Inputs: the exact pinned Body route/binding and current observation basis,
  sealed inclusion/projection/model plan, authenticated launch/provider
  incarnation, Idunn recovery authority, and waited Qdrant readback.
- Outputs: immutable obligation/plan and claim/attempt/recovery history,
  checkpoint/progress events and heads, terminal receipt/coverage head, and
  retirement history.
- Derived state: current claim, progress, coverage, warming, active, and query
  eligibility are joins over those records plus freshly revalidated Body and
  Qdrant; no store path or old head is truth by itself.
- Forbidden writers: Body observer/bootstrap cannot write projection state;
  projector/checkpoint/recovery cannot write Body; local Verse, Idunn,
  heartbeat, health, deploy scripts, Qdrant, and presentation cannot advance
  coverage heads.
- Shared path: prepare, acquire, each waited batch/readback, resume/recovery,
  final whole-scroll, health, and readiness resolve the same runtime binding,
  reopen exact Body read authority, and transact only through the pinned
  coverage store. Checkpoint and progress must admit atomically.
- Cut line: migrate every projection envelope and bootstrap the binding, switch
  authority once, then remove Body/Verse writers and readers. Preserve exact
  signed envelope identities where predecessor digests depend on them. There
  is no dual-read or missing-binding fallback.

Next: perform that writer/bootstrap migration, prove Body bytes remain unchanged
through projection, and only then build a new live Yggdrasil candidate. Keep
`c94fa580` rejected; do not lengthen a timeout or reinterpret its rewrite as an
observer fault.

## Signed Yggdrasil deployment admission cut complete locally — 2026-07-17

The failed GPU pressure run was a pipeline-contract failure, not a CUDA
failure. The GTX 1080 previously sustained near-full duty, but the monolithic
workspace projection published no durable Qdrant progress before the shell
deadline. The replacement executor now checkpoints bounded batches, resumes
from authenticated checkpoints after restart, and exposes checkpoint-derived
progress independently of its heartbeat. Aggregate health is `warming` only
while exact managed-process lineage, fresh signed heartbeat, and authenticated
checkpoint advancement remain inside the 300-second no-advance lease. Only the
terminal current receipt/head can produce `active`; the deploy shell has no
projection wall-clock verdict.

The cross-repo deployment authority cut is implemented and Soul passes it.
Epiphany signs runtime health with exact release id, witness, source commit,
deployment request id, process incarnation, and sequence. Idunn verifies the
pinned public-only host anchor and persists generic health as observation only.
Promotion reads `idunn.signed_health_admission.v1`, requires it to match the
current health observation, expire within 180 seconds, and join the exact
monotonic `idunn.current_deployment_request.v1` head plus its Bifrost-authorized
`idunn.deployment_request.v2`. Health/admission and request/head transitions use
cross-process CultCache compare-exchange over exact prior envelopes. CultCache
now writes a synced unique temporary file and atomically replaces the live
store without a delete gap. Concurrent N+1/N+2 admission proof finishes at
N+2; equal-second request IDs no longer decide authority lexicographically.

First trust enrollment refuses any pre-existing app-owned private identity when
no root anchor exists. One `enroll-trust-anchor` process creates the signer and
exports its public anchor; root then pins only that public document as
`root:idunn 0640`. Later deployments must byte-match it. The deploy actuator
uses Idunn's typed verifier with the exact request/release tuple; journal prose,
generic health, and `warming` cannot promote.

Local proof: Epiphany runtime-health 8, workspace-progress 13,
workspace-projector 22, host-identity 3, supervisor 17; Odin core 15 and Idunn
45 all pass. Gamecult shell syntax passes under Git Bash. Final hostile Soul
audit reports no P0/P1. Next: commit/push all three repos, deploy updated Odin/
Idunn to Yggdrasil first, then Bifrost-authorize and Idunn-deploy the exact
Epiphany commit. Verify live signed warming, batchwise Qdrant growth, GPU duty,
terminal exact-request active admission, and restart resume. No reboot is
authorized.

## Live Body-grounded admission rite — 2026-07-16

Thread `readiness-grounding-20260716` now has a typed, immutable user objective
and an accepted code-grounded Eyes result. Cold-start objective intake, explicit
local-Verse path ownership, source tools for Eyes/Modeling, canonical result
readers, failed-result supersession, reorient auto-acceptance, and the
CRRC-regather handoff to Eyes are implemented in the current worktree.

The live rite exposed output-contract wounds one at a time. Modeling operations,
frontier items, nodes, edges, profiles, anchors, code refs, observations, and
evidence now have provider-facing typed shapes matching canonical ingress;
explicit null no longer impersonates a reviewable state patch. Failed worker
output is terminally sealed with a bounded diagnostic rather than marked
complete. The stale OpenAI-runtime Modeling fixture now carries and echoes a
Body basis and passes again.

No current Modeling admission exists yet. After the Body was committed at
`f574a194`, Modeling completed a source-grounded typed proposal over the
repository Body. Mind correctly refused it because ordinary `Evolution` also
carried `upsert_frontier`, which requires an explicit frontier request. The
prompt now makes the invariant explicit: ordinary Evolution changes nodes and
edges only. Commit this small correction, rebuild the coordinator, supersede
the refused result, and relaunch in `--mode run` with the accepted Eyes basis.
Make no source edits between the final Body observation and Mind admission.
Only after Mind accepts an exact Body-basis RepoModel patch should
`epiphany-memory-semantic repository-readiness` be run with local Ollama
`qwen3-embedding:0.6b`, 1024 dimensions, and the runtime spine.

The next nodes/edges-only proposal reached Mind and exposed one further missing
launch input: the worker invented a workspace-derived domain id and used
`active` lifecycle, while the canonical revision-0 RepoModel has domain `repo`
and permits only observed/proposed/accepted/stale/retired for repo profiles.
`<canonical_repo_model_shape>` now carries exact current revision/hash, existing
domains, and lifecycle law into every Modeling launch. The rejected result is
already superseded. Rebuild/commit this context change, then relaunch once.

Exact remaining sequence:

1. finish tests/format/docs and commit so the Body is stable;
2. relaunch Modeling against thread `readiness-grounding-20260716`;
3. inspect only the operator-safe role finding, then auto-accept if its Body
   basis, RepoModel patch, and state patch are typed and current;
4. run the live repository-readiness join and preserve its projection;
5. investigate the provider failure only through typed request/receipt
   diagnostics if it repeats; raw worker thought remains sealed.

## Counterfeit planning writers cut - 2026-07-15

Modeling and Eyes traced the repo-frontier planning family and found a typed but
disconnected island. Self's deterministic planning request is real. The
Imagination candidate and Mind Adopt/Refuse/Hold documents were only authored
by tests or arbitrary callers through public persistence functions. Generic
Imagination `statePatch.planning` is a separate thread-state vocabulary, not a
repo-frontier candidate. Adopt suppressed future request selection but changed
no RepoModel bytes and affected no Hands route.

The public candidate/adoption writers are removed; their internal functions are
temporary test scaffolding, not production authority. The full source-grounded
replacement map is `notes/repo-frontier-planning-authority-map.md`. Build the
real nerve using the claim-repair shape: Self request, coordinator-owned typed
Imagination projection and request-keyed launch binding, exclusive immutable
result echo containing the candidate, then Mind-only correlated
Adopt/Refuse/Hold. Adopt must produce a narrow admitted model transition and
downstream Hands truth; Refuse/Hold remain inert receipts. Delete the remaining
fixture writers when that lands. Core verification is green at 319 passed, one
ignored.

## Eyes-to-Modeling claim correction nerve complete - 2026-07-15

The Aetheria-shaped corrective nerve is now closed from immutable Eyes
contradiction through Modeling proposal to Mind admission. A claim repair launch
may produce one immutable role-worker result carrying the exclusive
`claimRepairRequestId` echo and `RepairClaim` purpose. Result schema and worker
output contract are versioned v1; admission receipts and their contract are v3.
Malformed RepairClaim results carrying proposal/frontier echoes, Verification
authority, state patches, or self patches are rejected before persistence.

Mind replays the current repair request, challenge, Eyes packet and admissions,
the unique coordinator launch binding, exact worker launch document hash and
typed projection, runtime/thread identity, model base, target node hash, and
exact evidence set. It permits exactly one `ReviseNode`; the challenged `claim`
text must change while id, domain, profile, kind, creation time, and lifecycle
remain invariant. Review, next model, and receipt still share the model CAS.
Challenge resolution is derived from the target node hash changing; there is no
repair flag or cleanup writer. Exact retry validates the persisted receipt and
current admitted model. Hostiles cover swapped request, adjacent/missing
evidence, extra operation, wrong target, unchanged/timestamp-only claim,
identity/lifecycle mutation, state/self authority smuggling, forged launch hash,
and counterfeit retry receipt with byte-identical refusal. Soul passed the
final boundary; the core library is green at 318 passed, one ignored.

Next: bind Self's existing frontier planning request to a real Imagination result
and Mind Adopt/Refuse/Hold admission, then build the production semantic index
over typed graph claims so Modeling's persistent map becomes searchable body
knowledge rather than a scan-heavy archive.

## Runtime worker results are actually immutable - 2026-07-14

Eyes found that the document called an immutable role-worker result was still
published through ordinary CultCache `put`, allowing a second payload at the
same runtime job identity to replace the thought before Mind admission. The
writer is now absent-only: an exact retry converges without rewriting bytes;
any different result at the same job fails without mutation; concurrent loss
reloads and accepts only exact equality. The full core library passes at 316
passed, one ignored. Claim-repair result correlation may now rely on one
persisted worker result rather than a politely named mutable slot.

## Modeling claim-repair launch authority landed - 2026-07-14

The inert `RepoModelClaimRepairRequest` now reaches Modeling through one
coordinator-owned typed context projection and one immutable, request-keyed
launch binding. The coordinator reconstructs the projection from current
canonical state; callers cannot prepopulate it, combine it with proposal
Modeling authority, substitute any causal field, or spend one repair request
twice. The launch document hash binds the exact effective document delivered to
the worker. This slice still grants no RepoModel write authority.

The launch race exposed a deeper state-transaction fault: two coordinators could
validate the same logical state and both publish. `CoordinatorStateTransaction`
now captures one exact typed cache image, validates the canonical state by
polymorphic `(type, key)` identity, and commits state plus companions through one
backing-store CAS. Immutable companions are absent-only or byte-equivalent
retries with the persisted timestamp preserved. Explicit captured replacements
may replace only the exact envelope observed at open. Imported nondefault host
state may seed an absent store once. Focused hostile tests prove one race winner,
no losing job/request/event/runtime link, immutable collision no-write behavior,
captured-replacement contention, and same-key/different-type coexistence. Soul
re-reviewed the repaired proof surface and passed it; the full core library is
green at 314 passed, one ignored.

Next: make the Modeling result echo this repair request exclusively, then add a
dedicated narrow patch purpose and Mind admission that replays the full chain and
requires the exact challenged claim bytes to change. No other result or model
revision may clear challenge pressure.

## Modeling name authority correction - 2026-07-14

Modeling is the standing embodied organ that models the Body. Proprioception is
a retired legacy name, not a parallel faculty or worker posture. The global
Codex doctrine had missed this migration and could still force Self to dispatch
`Proprioception` workers; runtime work-loop telemetry also targeted the retired
stage name. Both are corrected to Modeling. Keep legacy string handling only
where it migrates old persisted agent identity into Modeling, and keep old prose
only where it is clearly historical evidence.

## Current orientation — 2026-07-12

### Aetheria organ-structure evidence — 2026-07-13

Task `019f3fe1-9f69-74e3-97fb-a18490d72119` supplied strong positive evidence
for the Epiphany hypothesis without requiring raw worker-thought inspection.
Modeling maintained concrete migration bodies across Aetheria, EveUnity, and
CultLib; Eyes found CultCache crash consistency and SoA ownership skew; Hands
landed bounded disjoint changes; Soul rejected a registry repair that still
allowed readers to observe partial refresh; Imagination corrected a roadmap
that called unfinished migration complete; Self preserved dependency order and
kept unrelated compile skew out of coherent commits.

Prompt forms worth preserving:

- require the exact compile boundary and deletion line when research expands;
- give Hands hostile negative acceptance criteria, not only a desired output;
- explicitly allow a worker to stop and name a missing substrate primitive;
- review diffs against authority and concurrency invariants after tests pass.

The primary lesson is not Modeling persistence in isolation. The organ structure
changed the quality of the engineering: discoveries altered plans, plans bounded
disjoint cuts, Soul could reject a plausible green fix, Imagination could reject
roadmap mythology, and Self preserved dependency order without flattening those
different judgments into one running narrative. This is positive evidence for
durable organ-to-organ cognition, not merely for role-flavored prompting.

That lesson now has a concrete corrective nerve. Eyes can persist an immutable
`RepoModelClaimChallenge` against one serialized claim at one exact admitted
model revision. Runtime-spine admission replays the canonical Eyes packet,
unique current Mind receipt, model identity, and claim hash in one absent-only
CAS. Self uses the same validated challenge predicate for Imagination planning,
Hands route selection, and actionable-Hands status; only frontier targeting the
challenged claim is withheld. Eyes never writes RepoModel, and no challenge ids
were stuffed into the graph. Core proof is 304 passed/1 ignored and independent
Soul review passes. Next build a typed Modeling challenge-repair request and
Mind admission path so evidence can reshape the shared map, then let Self and
Imagination revise the campaign from that repaired substrate.

The first repair foundation is now inert and exact. A challenge remains live
across unrelated RepoModel revisions while its target claim bytes are unchanged;
an unrelated edit can no longer launder Eyes pressure away. Self can persist one
`RepoModelClaimRepairRequest` binding the challenge and Eyes packet, original
and unique current admissions, current model and claim, runtime/thread, and
affected frontier hashes in one CAS. It deliberately has no repair patch
purpose, result echo, worker launch authority, or Mind admission effect. Core
proof is 306 passed/1 ignored and Soul passes. Next wire coordinator-owned typed
Modeling launch projection/binding and exclusive result correlation before
introducing any repair actuator.

Do not substitute the first linear frontier identity proof for that larger
lesson. `Modeling -> Mind -> Self -> Hands -> Soul -> Modeling` proves that one
wound can retain causal identity. The awakened anatomy also needs typed backward
pressure: Eyes findings that challenge model claims, Imagination revisions that
change campaign shape, Soul rejections that create new evidence requirements,
and Self rerouting based on those admitted artifacts. Modeling is the shared
situational substrate, not the whole organism and not the product.

Persistence is the connective-tissue gap exposed by that success. Modeling's
repo anatomy, confirmed/changed/invalidated claims, freshness, frontier, and
semantic-index consequences must live in typed searchable state; Self must route
from those artifacts; later Hands and Soul prompts must receive the relevant
frontier and evidence. Otherwise an excellent coordinator can perform the
structure transiently, but the next awakened Self inherits only the applause.

The first safe piece of that connective tissue is now implemented. The existing
memory-graph aggregate owns a revisioned, hashed RepoModel plus typed migration
frontier; exact frontier claim IDs precede BM25 ranking; dependency prerequisites
precede dependents under budget; and the frontier's body, question, gap, status,
dependencies, and recommended organ enter dynamic role context. Canonical state
cannot be reverse-built from thread graphs, replaced by raw refresh/bootstrap,
or lost to concurrent valid patches: CultCache now provides exact-envelope CAS
and insert-if-absent beneath its cross-process sidecar lock, with hostile race
proofs. Soul rejected two earlier cuts before accepting this authority boundary.

The first causal-identity slice of that Aetheria loop is now closed. Modeling
emits a typed `RepoModelPatch`; Mind admits the exact immutable worker result,
patch bytes, evidence, review, and replacement model under one conditional
store transaction; Self selects one eligible Active frontier item from the
admitted current model; and the resulting route freezes model revision/hash,
item hash, claims, dependency order, and path scope. Hands intent, review,
Substrate grant, frontier authority, patch, command, and commit documents are
immutable insert-once records. Soul receives a typed verification request bound
to that complete chain. A later typed Modeling request binds the accepted Soul
result and permits exactly one verdict-incorporation revision of the same item:
`pass` yields `Resolved`; `needs-review`, `needs-evidence`, or `fail` yields
`Blocked`. Self derives Hands readiness from the same selection predicate, so
Blocked/Resolved items cannot be routed and no default implementation authority
survives without an admitted actionable frontier.

Soul found and forced cuts of four plausible green bypasses before accepting
the slice: ordinary Evolution could mutate routed frontier lifecycle; Hands
receipts and their intent/review/grant foundation were mutable by identity; the
Modeling request trusted caller-supplied temporal acceptance; and a split model
admission/thread-state commit could wedge on retry. Evolution is now barred
while a current-model route exists and cannot manufacture Blocked or terminal
frontier lifecycle. Every causal companion uses absent-only cross-process CAS
with exact retry. Final verdict admission replays the full authority and
consequence chain immediately before the model CAS. Verification acceptance is
reloaded from canonical thread state, and stable review identity makes the
split commit recoverable. Hostile substitution, stale-model, counterfeit,
concurrent-writer, crash-window retry, pass/nonpass, and nonselection tests are
green; the full core library proof is 303 passed and 1 ignored.

This is still not the whole Aetheria lesson. It proves durable causal identity
and backward Soul pressure, not the complete multi-organ correction ecology.
Eyes challenges and Imagination campaign revisions still need typed admission
and Self routes. The older `RepoWorkModelingRequest/Route/Finding` path is also
misnamed parallel closure machinery: it models one already-executed work item,
not repository anatomy. Preserve useful plan/execute receipts, but converge
that workflow on canonical RepoFrontier route/Hands/Soul/Modeling authority and
demote `RepoWorkMapEntry` to derived closure history. Do not let item-slug routes
or the legacy special Modeling worker close work independently of RepoModel.

Stage 1 of that convergence is now landed in the worktree and Soul-approved.
The parallel authority was cut before its replacement was enabled. Legacy
RepoWork Modeling request/route/finding/map types, worker launch, OpenAI ingress,
runtime writers, manual generation retry, Mind plan-adoption type, independent
Hands grant, CultMesh overview/readiness/public-proof writers, and closure
contract corpse are deleted. `epiphany-work` is an 82-line quarantine mouth:
every mutation/scheduling/publication command refuses before substrate access;
overview emits only an explicit `historical-only` non-authoritative projection.
Verse query cannot turn historical map rows into current gates, actions,
closure, acceptance, pending Bifrost requests, or accounting. Legitimate
Bifrost artifact-acceptance and metrics receipts still produce accounting rows
without consulting RepoWork history. The authoritative serial proof is core
294 passed/1 ignored, epiphany-work 1 passed, Verse query 13 passed, OpenAI
runtime 8 passed; independent Soul verdict is PASS.

The temporary outage is intentional. Do not resurrect old plan/run/adopt/
execute/close code to restore motion. The next build begins from the authority
map in `notes/repo-work-canonical-convergence.md`: inert proposal -> ordinary
Modeling frontier admission -> typed Self planning request -> Imagination plan
candidate -> Mind plan adoption -> distinct Self execution route -> canonical
Hands/Soul/verdict-incorporation chain. Plan readiness does not belong in
`RepoFrontierStatus`, and a generic phase router has not earned existence.

The typed pre-Hands planning foundation is now implemented and Soul-approved.
`RepoFrontierWorkProposal` is immutable, content-hashed, runtime-bound, rejects
marked private state, and has an explicit inert contract. It cannot create a
frontier or route. `RepoFrontierPlanningRequest` is a distinct Self document:
it validates the canonical model and admission, chooses the deterministic first
dependency-ready Active Imagination frontier, freezes item hash/source scope,
and uses current-model CAS. `RepoFrontierPlanCandidate` binds that request and
revalidates model/admission/item/scope; safe paths must be contained by the
frozen scope and checks/stop/rollback lists must contain real text. Mind plan
adoption binds exact candidate bytes and current anatomy. `Adopt` owns one
deterministic model+frontier claim; concurrent candidates produce one winner,
while Hold/Refuse cannot squat the reserved claim prefix. Exact retries
revalidate current state. Nothing in this foundation emits an execution route,
Hands authority, or frontier lifecycle change. Full core proof is 296 passed
and 1 ignored; independent Soul verdict is PASS.

Proposal-to-frontier admission is now explicit and Soul-approved. Direct User
intake writes one immutable, payload-hashed, nonprivate proposal bound to the
persisted runtime and thread; repository/workspace remain provenance, not
counterfeit authority. Coordinator selection is immutable, exact-time, and
race-safe. The coordinator launch transaction is the sole writer of a typed
`proposalModelingContext` projection inside the persisted Modeling launch
document; it atomically seals that document's MessagePack SHA-256 in an
immutable job/request/proposal launch binding. Prompt prose and result echoes
are correlation only. Admission reconstructs and compares the exact proposal,
selection, projection, patch base, launch hash, job, binding, runtime, and
thread chain. Ordinary Evolution cannot mutate frontier. Proposal Evolution
gets exactly one frontier operation and it must be one proposal-citing
`UpsertFrontier`; it emits no plan, route, Hands authority, or lifecycle
decision. Core proof is 302 passed/1 ignored, OpenAI runtime proof is 8 passed,
and final independent Soul verdict is PASS. Hostile proofs cover privacy
laundering, changed selection time, concurrent selection, corrupt persisted
contracts and hashes, caller-prepopulated context, stale base, swapped launch
and companions, duplicate backend binding, dual authority echoes, request
reuse, delimiter injection, and byte-identical no-write refusals.

The next artifact-contract audit then cut `epiphany.persona_chat.v0`: its live
writer and catalog had disagreed on field vocabulary, evidence fields, and
lifecycle closure. The writer is now typed Rust with closed
draft/blocked/posted status, the schema is strict and complete, and legacy
snake_case survives only on read. Proprioception then identified and Hands
rebuilt the larger false surface. `epiphany.persona_surface.v0` is now a strict,
content-free projection of discriminated artifact and consequence-owner action
references. It reports rejected rows and source failures as attention, while
owning neither speech eligibility nor Mind/Bifrost/provider consequences.
Reddit and Other now have strict typed Rust artifacts and registered schemas;
Other ends at a crossing request and can never claim publication. Its surface
action remains absent until a typed intent contract actually exists.

This reinforces the Aetheria lesson: Modeling found the catalog fiction, Hands
cut the producer boundaries, and Soul-shaped negative checks caught content
leakage and false consequence ownership. Persistent anatomy lets the next Hands
pass start at the wound instead of ceremonially rediscovering the body.

The following Aetheria-shaped pass cut two Idunn split truths. Swarm overview
now consumes the same managed-child sight as the specialist Idunn view, but
exports typed rows only; Gjallar/Eve remain downstream lowerers. Enabled child
readiness requires native process executable identity matching the lifecycle
receipt, so PID reuse cannot counterfeit life. Separately,
`deployment-aftercare-audit` no longer accepts raw caller response files, and
readiness no longer launders its cached projection. Typed Idunn deployment and
aftercare receipts must resolve and bind across identity, runtime, Verse,
result/ref, runbook, source commit, and current HEAD.

The next Modeling pass repaired the targeting organ itself. Memory-graph v1
binds repo anatomy to the actual state-store identity, accepted revision, and
SHA-256 source bytes. Cache presence, path/symbol locators, and serialized
freshness no longer make anatomy Ready. Launch and explicit refresh share one
validator; missing/changed node, edge, or link anchors stale their summaries.
There is still no production semantic index for graph claims, so Modeling index
availability is honestly unavailable rather than inferred from code search.

The repo-work adoption authority rebuild is now implemented. `run_adopt`
consumes an immutable typed `RepoWorkPlanAdoptionReview` bound to exact
workspace, plan/run paths and SHA-256 bytes, action, command, commit message,
changed paths, queued Hands review, and Substrate grant. It atomically creates
an immutable `RepoWorkHandsGrant` plus approved Hands review; Refuse/Hold,
counterfeit digests, swapped paths, and same-ID replacement are rejected.
`run_tick` can only report `awaiting-mind-review`. `run_execute` rereads the
whole chain and revalidates persisted Substrate authority before PowerShell.
Soul closure now reloads typed runtime truth and cross-binds it to the execute
receipt; a swapped valid adoption chain fails. Six compiled smokes that taught
the deleted caller-writable `mindAdoptionDecision` model were removed.

Next architectural cut: finish the Aetheria lesson rather than polishing this
gate. The proposal now reaches Modeling as an exact contestable artifact, but
the larger corrective ecology is still incomplete. Add typed Eyes challenges
that can invalidate model claims and create evidence pressure without direct
RepoModel write authority; then bind the existing planning request to an
Imagination worker result and Mind adoption. A production semantic index over
typed graph claims remains required so those organs can retrieve the living
migration map instead of rediscovering it from files. Memory-graph freshness is
substrate; durable multi-organ correction is the organism.

Epiphany is in an authority-provenance purification pass. The live question is not whether a command can produce a plausible document; it is whether the subsystem writing that document owns the fact it asserts.

The confirmed conceptual substitutions have been cut:

- read-only Verse diagnostics no longer seed or promote the state they inspect;
- Bifrost publication, public-proof publication, artifact acceptance, and metrics callers submit requests without manufacturing provider receipts;
- daemon tool invocation, Eve connection, and daemon poke callers submit intents without manufacturing provider acceptance or lifecycle results;
- Persona feedback no longer manufactures Imagination consensus;
- arbitrary operator-snapshot JSON cannot be promoted into canonical tool intent or receipt state;
- operator-run completion is derived from a fresh, contained result artifact rather than caller status;
- daemon service plan/execute authority is encoded by explicit command identity;
- synthetic receipt smokes are confined beneath `.epiphany-smoke`.
- generic local Verse bootstrap no longer publishes provider advertisements,
  Eve surfaces, or hosted tools; provider absence survives bootstrap.
- the central `provider-advertisements` / `publish-odin` compatibility mouth is
  deleted; only provider-owned bodies may publish presence to Odin.
- bulk and daemon-bounded central provider-state writers are deleted. Each
  `epiphany-cluster-daemon` heartbeat publishes liveness only; it cannot turn
  topology into an advertisement, Eve composition, or hosted-tool claim.
- cluster daemons do not bootstrap or query the full Verse. Missing persisted
  topology is a pre-write refusal; explicit operator bootstrap owns shared
  policy, topology, contracts, and brake initialization.
- daemon-supervisor production commands likewise require persisted bootstrap
  and cannot create shared state. Its two audit-smoke fixture initializers are
  hard-confined beneath `.epiphany-smoke`.
- query-CLI requester/operator mutations also require persisted bootstrap.
  Only explicit seed commands and the fixed aggregate smoke may initialize the
  shared local Verse.
- worker launch-context rendering requires the existing Verse and cannot seed
  it. The prompt-context smoke accepts no destination and is fixed beneath
  `.epiphany-smoke`.
- orphaned Bifrost/GitHub/Maintainer response and Imagination consensus
  constructors/writers are test-only. The aggregate smoke proves requests stay
  pending; production keeps readers for externally provider-authored documents.
- bulk seven-daemon readiness construction/writing is test-only. Production
  loaders enumerate topology; the aggregate smoke owns a fixed-store local
  fixture helper, while real liveness remains single-daemon authored.
- topology-derived provider builders survive only as test fixtures for legacy
  v0 shapes. Live consumers ignore provenance-free v0 Odin/Eve/tool rows;
  explicit bootstrap deletes stale rows of exactly those families. Topology
  `eve_surface_id` is address metadata only, not surface availability.
- bootstrap no longer writes default operator status. The dedicated writer was
  a dead `ready` template, so the writer and its now-ownerless schema/context/
  prompt/reader family are deleted. Source-derived operator snapshots remain.
- agent-state SoA sync requires bootstrap; report preserves missing filesystem
  state. The wrapper explicitly composes sync then report rather than hiding a
  refresh inside readback.
- query, cluster-daemon, and daemon-supervisor argument parsing no longer creates
  store parents. CultCache's single-file backing store returns empty before
  taking a lock when its file is absent, so all readers preserve filesystem
  absence without per-loader guards. Full context projection refuses a missing
  Verse; explicit bootstrap or a real writer owns CultMesh body creation.
- the generic `epiphany-eve-provider` CLI is deleted: caller-supplied provider
  identity/status was not provider participation. Eve requests remain pending;
  local response construction/writing is test-only until a real target daemon
  or authenticated ingest boundary owns the receipt.
- daemon-tool response construction/writing is also test-only. No shipped host
  daemon currently responds; the aggregate smoke proves a pending intent and a
  missing host receipt rather than fabricating Hands acceptance.
- the deployment-config family smoke and its aggregate MVP-gate row are deleted:
  they fabricated Idunn `deployed`/`complete` receipts. Epiphany exposes config
  audit/runbook requests and typed Idunn readers, but no local deployment or
  aftercare response writer.
- the artifact-acceptance/metrics response-closing smokes, Bifrost accounting
  bundle, wrapper mode, and aggregate green gate are deleted. Their request
  cargo remains open until genuine Bifrost/Maintainer response ingest.
- generic repo-work readiness approval is deleted. The former command accepted
  four arbitrary strings as Maintainer/Soul/Mind/Bifrost reviews; readiness
  remains sight-only and the request family awaits genuine reviewer evidence.
- the aggregate repo-swarm MVP gate and wrapper are deleted. It painted green
  rows from prior smoke summaries and local fixtures rather than exercising one
  live end-to-end organism. Use focused tests plus live-fire evidence instead.
- the fresh-repo MVP smoke is deleted. Its disposable Git operations were real,
  but its PR, maintainer-review, merge, Bifrost, and Soul evidence was supplied
  by the same local caller rather than those owning bodies.
- `epiphany-work publish` and `sync` are deleted. The first promoted arbitrary
  receipt strings into publication authority; the second promoted arbitrary
  merge strings plus Git ancestry into merge authority. Bifrost intent,
  provider receipts, and read-only ancestry are separate paths.
- legacy `work-publish-*.json` and `work-sync-*.json` are no longer read by
  overview, readiness, scheduling, receipt-chain, or proof-bundle paths. Dead
  writer artifacts cannot remain living authority; closed work waits at
  `awaiting-publication` until an owner-aligned provider projection exists.
- `epiphany-hands-action record-pr` is deleted. Caller-transcribed PR and
  Bifrost fields cannot create Hands publication proof; PR construction and
  persistence are test-only pending an authenticated adapter owner.
- ownerless provider-result actuators are no longer public production API.
  Eve/daemon-tool acceptance, Bifrost publication/GitHub/public-proof/artifact/
  metrics results, and Imagination consensus constructors/writers are
  `cfg(test)`; production retains their typed readers for real ingest.
- repo closure no longer accepts caller echoes of Modeling model ref, verdict,
  finding, authorship, or summary. The scheduler no longer transcribes them.
  Full closure reads the immutable runtime-spine Modeling finding directly.
- repo closure no longer accepts caller-selected verification commands or an
  optional source-grounding flag. Soul uses fixed Git consequence inspection,
  mandatory grounding, and fails when typed plan/family evidence is absent.
- callers cannot overwrite Soul verdict narration or choose Mind's immutable
  admission revision. Soul derives the explanation; initial map admission is
  revision zero and later evolution uses typed Modeling route generations.
- Hands binaries no longer construct Substrate Gate grant structs inline.
  Grants are non-exhaustive externally and come from narrow fixed-policy
  constructors owned by the `substrate_gate` module.
- Hands persistence resolves its grant rather than trusting an ID label. Grant
  identity/scope/operations/paths and approved review must match each intent
  and patch/command/commit consequence; grants are immutable.
- cluster-daemon heartbeat no longer accepts caller liveness or note fields.
  A successful owning-body heartbeat derives `ready`; degraded/down require
  observed supervisor or timeout evidence.
- verse-query's mega-smoke no longer writes synthetic Idunn service runbooks or
  lifecycle receipts into its selected store and then verifies its own state.
  Focused daemon-supervisor smokes own quarantined lifecycle fixtures.
- daemon-supervisor smoke confinement is real containment, not component-name
  matching: parent traversal and stores outside workspace `.epiphany-smoke`
  are refused before fixture seeding.
- Thirty-two repo-family and survival smokes no longer accept an independent
  deletion root. They derive `.epiphany-smoke` only after canonicalizing the
  selected repo root; `--smoke-root` is rejected before creating its target.
- Idunn lifecycle receipts no longer accept caller-transcribed executable/schema
  hashes, witness identity, required types, or a preflight-pass bit. Repo work
  consumes the runtime's real preflight directly; Idunn reports process facts.
- Windows service install exit zero is `install-command-succeeded`, not
  `installed`. Single-service completion requires SCM status/reconcile; cluster
  completion requires the observed cluster service audit and cannot cite an
  earlier copy of its own execution-audit verdict.
- Service lifecycle projection selects the actual latest family event. An old
  attention receipt no longer masquerades as current state after recovery;
  newer failures still surface immediately.
- The lifecycle `latest` mirror is event-time-owned, not last-writer-wins.
  Invalid/reversed timestamps are refused and delayed old receipts cannot move
  the mirror backward.
- Scheduler `latest` is likewise completion-time-owned. Tick timestamps are
  validated, reversed intervals are refused, and delayed replay cannot rewind
  the scheduler projection.
- `hands-action-gate/latest` is creation-time-owned. Delayed mirror replay
  cannot advertise obsolete actuator scope or record-pass commands as current.
- `role-review-event/latest` is creation-time-owned. Delayed acceptance or
  supersession readbacks cannot rewind Self/operator coordination context.
- `coordinator-run-receipt/latest` is creation-time-owned. Delayed runs cannot
  replace the current final action or artifact projection.
- Work-loop telemetry is now validated internal evidence and production-time
  ordered. Delayed or wrong-Verse packets cannot replace Soul/Modeling's
  current Hands consequence chain; future receipt lower bounds are refused.
- Repo-work map `latest` is Mind-admission-time-owned. Delayed older projections
  cannot rewind the queue map, and mirroring before admission is refused.
- Repo-work overview global `latest` is generation-time-owned. Delayed overview
  transport cannot make an obsolete queue gate or next move current.
- Repo-work readiness global `latest` is generation-time-owned. Delayed
  sight-only reports cannot replace current proof visibility.
- The writerless `repo_work_readiness_review` document family is deleted from
  schema registration, exports, loaders, receipt directory, swarm overview,
  TUI, and Bifrost accounting. Real review receipts retain their owners.
- Repo-work public-proof global `latest` is generation-time-owned. Delayed
  proof transport cannot replace the current commit/hash evidence projection.
- Agent-state SoA `latest` is generation-time-owned and local-area confined;
  delayed summaries cannot rewind prompt/swarm self-knowledge.
- Operator snapshots are validated internal-Verse observations and `latest` is
  generation-time-owned; delayed status transport cannot rewind the observed
  coordinator/runtime surface.
- Operator-run receipt admission resolves its intent by `run_id`, not by the
  unrelated global latest mirror. Intent/receipt mirrors are validated and
  owner-time ordered; concurrent runs retain independent identity.
- Prompt assembly joins Eve and daemon-tool receipts to intents only by matching
  `intent_id`; independent latest mirrors can no longer fabricate a causal
  completion in model context.
- Bifrost prompt/accounting chains follow typed intent -> publication -> GitHub
  receipt identity. Independent latest documents cannot close publication,
  lend review/credit counts, or supply a counterfeit public reference.
- Collaboration accounting follows consensus `feedback_id`; unrelated latest
  consensus cannot close another Persona feedback lane or lend public refs.
- Artifact-acceptance and metrics receipts still lack an explicit repo-work
  request/map identity. Do not infer it from item/workspace/branch/commit;
  accounting keeps these lanes open with `requestIdentity=missing` while still
  showing provider receipt proof completeness. Repair the Bifrost-owned
  contracts before claiming concurrent-safe closure.
- Repo-work queue receipts carry typed queue/selected row objects. Swarm stop
  classification no longer reparses `QUEUE-RUN | key=value` TUI prose; legacy
  or hostile strings cannot steer gate, blocker, item, branch, or next move.
- Receipt-directory runbook selection uses typed `service_id` only. It no
  longer recovers service ownership by splitting human route prose at `::`.
- Deployment operational audit and runbook read one typed TOML model. Substring
  presence, comments, wrong sections, and silent watched-ref/script defaults no
  longer substitute for valid deployment configuration.
- Repo-work deployment closure uses that same typed model and semantic
  predicates; the duplicate substring validator is deleted.
- Secret-policy request closure parses typed TOML and validates request,
  antecedent, receipt, security-packet, and authority semantics. Comment prose
  cannot counterfeit secret/write/deployment denials.
- Dependency-policy request closure parses typed TOML for supply-chain scope,
  receipts, evidence requirements, and authority seals; comment prose cannot
  counterfeit package/lockfile/network/CI denials.
- Deployment-request closure parses typed TOML for Idunn ownership,
  antecedents, receipts, deployment packet, and authority seals; comment prose
  cannot counterfeit SSH/push/deployment denials.
- Verse-query smoke reset proves the exact built-in path and canonical strict
  containment beneath `.epiphany-smoke` immediately before recursive deletion.
- The four typed closure contract families live in
  `epiphany_work/closure_contracts.rs`; `epiphany-work` is their orchestrator.
  The module is parent-private and preserves family-specific predicates rather
  than exporting generic policy machinery.
- Bifrost provider receipt contracts still lack provider timestamp/revision.
  Do not invent consumer-side ordering; repair that in a Bifrost-owned schema.

The presentation boundary is now plain: `swarm overview` is a generic compact read-only projection. Gjallar is a downstream TUI application on Nightwing and is not an Epiphany organ, provider, owner, runtime, or architectural dependency. Eve/CultUI graphs may be lowered or composited downstream without Epiphany caring which presentation client does it.

Static cluster topology is declaration, not attendance. Bootstrap persists seven
faculty routes and desired private-Verse/daemon/Eve addresses, but missing
provider status now produces zero observed daemon rows. Topology and overview
publish explicitly declared counts separately from `observedDaemonCount`;
daemon heartbeat sight never claims an agent count. Restart-policy rows may say
`unobserved` for a desired target so Idunn can reconcile it without inventing a
body. Prompt context names these as declared routes/targets. The deleted
`unknown_daemon_status` constructor cannot put seven empty helmets on the roll.

The Epiphany-local `bifrost-ledger` mouth is deleted. It assigned external owners
and accounting closure from response-shaped documents found in a shared local
store without authenticated provider ingress. Requester-owned publication and
public-proof intents plus Persona feedback remain. Provider response absence now
has no local closure or audit route. Do not restore a ledger projection until an
ingress receipt binds provider identity/contract, transport session, exact
payload hash/document id, correlation, target runtime/Verse, admission time, and
verification result.

## Authority map

- Request owners write intents and requests.
- Provider bodies write acceptance, execution, lifecycle, and result receipts.
- Diagnostics read persisted facts; absence remains absent or unknown.
- Adapters may project edge state but cannot promote it into canonical provider truth.
- Smoke fixtures may manufacture synthetic state only inside fixed disposable roots.
- `swarm overview` owns no operational fact and performs no scheduling, publication, deployment, lifecycle, admission, or provider acceptance.

## Rehydrate

Read, in order:

1. `state/map.yaml`
2. `notes/epiphany-current-algorithmic-map.md`
3. `notes/conceptual-substitution-audit-2026-07-12.md`
4. `notes/receipt-writer-provenance-audit-2026-07-12.md`
5. `notes/epiphany-fork-implementation-plan.md`

Then run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-state -- status
```

Historical implementation detail belongs in git history, smoke artifacts, and the evidence ledger. Do not restore deleted writers or requester-authored receipts from old proof prose.

## Next real move

Continue the modeling pass beyond receipt writers. Tool-request,
metrics-request, artifact-acceptance-request, and credit-request closure now use
typed family contracts and cannot accept comment-counterfeit execution, spend,
acceptance, or credit seals.
Inventory remaining substring closure families and production paths where a
projection, cache, adapter, scheduler, coordinator, or compatibility surface
asserts a fact owned elsewhere. For each candidate, name the owner, allowed
inputs, emitted state, forbidden writers, and negative proof before changing
code. Also audit remaining non-receipt smoke binaries for path escape or
destructive scope.

`epiphany-repo-personality-smoke` reset is now canonically confined beneath
`.epiphany-smoke`; fixed default provenance no longer authorizes recursive
deletion. Continue the same resolved-containment audit across remaining smoke
reset helpers.

The 32 repeated timestamped family/lifecycle smokes no longer reset occupied
leaves. They atomically claim a fresh directory with `create_dir` and fail on
collision, removing their recursive-delete authority entirely.

Seven UUID-scoped temp helpers now use exclusive `create_dir`, not
`create_dir_all`; a pre-existing leaf is refused before work and can no longer
be adopted and later deleted.
Three coordinator launch-context test leaves were corrected as well. The full
Rust source search now contains no generated `temp_dir()` leaf adopted through
`create_dir_all`.

Publication-request closure is now typed and guarded against regaining
substring authority. Current structural count: nine typed high-authority
families, 704 remaining closure substring assertions, and 1,458 contract-organ
lines.

The former 1,458-line `closure_contracts.rs` slab is now a two-line private
facade over `closure_contracts/external_governance.rs` and
`closure_contracts/operations.rs`. Keep new contracts with their consequence
owner; do not reunify the slab or widen its parent-private API.

Sync-request closure is now typed within external governance and guarded
against substring fallback. Current count: ten typed high-authority families
and 667 remaining closure substring assertions.

External-governance counterfeit fixtures now live in the dedicated lexical
`external_governance_tests.rs`; production contract anatomy is 668 lines and
the six semantic counterfeit tests remain private and passing, including the
explicit sync push-grant attack fixture.

Provider chronology remains externally blocked for a precise reason: Bifrost
receipt contracts expose no provider timestamp or monotonic revision, so local
`latest` mirrors only know arrival order. Do not invent chronology locally. The
33 duplicate `#[cfg(test)]` residues found around provider writers/re-exports
were removed; one test gate remains on every test-only item.

All seven Bifrost/GitHub mirror loader APIs are now named
`load_arrival_latest_*`; no misleading `load_latest_*` aliases remain. The
persisted `/latest` key spelling is retained as external storage compatibility,
not interpreted as provider chronology.
The Bifrost ledger report also uses `arrival_latest_*` typed fields and
`arrivalLatestBifrost*` JSON keys; no old report aliases remain.
`EpiphanyCultMeshContext`, prompt assembly, and Verse consumers now also use
`arrival_latest_bifrost_*`; persisted storage strings retain `/latest`.
The Rust constant identifiers are now `*_ARRIVAL_LATEST_KEY` too; only their
external string values retain the `/latest` storage spelling.
Regression test `bifrost_mirrors_name_arrival_order_without_rewriting_storage_keys`
guards honest Rust vocabulary and the seven stable persisted keys together.

PR-request closure is now typed within external governance and protected by the
class no-substring guard plus an explicit malicious GitHub-grant fixture.
Current count: eleven typed high-authority families and 620 remaining closure
substring assertions.

Maintainer-review closure is now typed and class-guarded. Current count: twelve
typed high-authority families and 582 remaining substring assertions. The
explicit malicious maintainer-approval fixture now passes as a negative proof.

Verification-request closure is now typed and class-guarded. Current count:
thirteen typed high-authority families and 543 remaining substring assertions.
The explicit malicious Soul-verdict fixture now passes as a negative proof.

Closure-domain authority now has a physical workflow owner.
`closure_contracts/workflow.rs` owns typed verification request closure and
`workflow_tests.rs` owns its counterfeit-Soul-verdict fixture. The facade
composes workflow beside external governance and operations. Direct `rustfmt`
is required for these lexically included files because Cargo fmt does not find
them. Next, type and move one adoption/scheduling/work-order family into
workflow rather than feeding the external drawer. External governance should
converge on tooling plus publication/accounting; operations retains
policy/deployment.

Work-order closure is now typed inside that workflow owner. The old substring
path could accept a commented Hands denial beside a real authority grant; the
malicious fixture and generic class guard now make that impossible. Fourteen
high-authority families are typed and 507 substring assertions remain. Next
inspect adoption versus scheduling and convert the authority-bearing family
with its own negative proof.

Scheduling won that comparison because it directly names Self's queue gate,
pulse, next family, and queue-selection receipt. Its closure is now typed under
workflow ownership; a malicious commented queue denial cannot conceal a real
mutation grant, and the family is in the no-substring class guard. Fifteen
high-authority families are typed and 477 substring assertions remain. Next
type adoption, the remaining untyped workflow gate in this chain.

Adoption closure is now typed under workflow ownership as well. The decision
contract, review/state receipts, input requirements, authority denials, and
private seal are read as actual TOML fields; blank input references are
rejected. A malicious commented state-commit denial cannot hide a real grant,
and adoption is in the class guard. Sixteen high-authority families are typed
and 447 substring assertions remain. The complete adoption/scheduling/work-
order/verification chain now has one physical workflow owner.

The old residual counter was diff-derived and has been retired. Direct source
inventory now reports 426 `content.contains` occurrences across sixteen
remaining closure families. Objective Draft has moved into a new physical
`preparation.rs` owner because it is Imagination cargo, not workflow or
external governance. Its typed closure and malicious fixture prevent a
commented adoption denial from hiding a real grant. Seventeen high-authority
families are typed. Continue from the source-derived family inventory, not the
historical counter.

The readiness-review hydra is cut. Self owns routing; Maintainer, Soul, Mind,
and Bifrost are independent required reviewers; `readiness_approval_owner` is
explicitly `none` because no local aggregate writer exists. Typed external-
governance closure and a counterfeit-approval fixture protect that boundary.
Eighteen high-authority families are typed; direct source inventory reports 363
substring occurrences across fifteen remaining families.

Interpreter Brief no longer steals Mind's name. Deterministic Imagination
lowering now emits an Imagination-authored request for Mind interpretation with
`interpretation_admitted=false`; it does not claim Mind authored or owns an
interpretation that Mind never produced. Preparation owns its typed closure and
counterfeit-state-authority fixture. Nineteen high-authority families are typed;
direct source inventory reports 303 substring occurrences across fourteen
remaining families.

Planning Brief is mapped for deletion, not typing. It contains no candidate
work-item records; copied global schema/catalog/closure doctrine is read back as
`safeFamilyPlanning`, letting catalog completeness impersonate item planning.
Remove the family, generator, closure/readback, dedicated smoke, and readiness
dependency together. Preserve consensus -> interpretation request -> objective
draft -> Mind adoption. Do not replace it with another per-item catalog dump.

That deletion is complete. The generator, closure/readback, readiness row,
Verse classification, CLI advertisement, and 679-line dedicated smoke are
gone; 1,393 source lines were removed without an adapter. No production source
references remain. Direct inventory now reports 183 substring occurrences
across twelve families.

Collaboration policy/topic no longer pretend deterministic repo files are live
Persona/Eve contracts. Imagination authors proposals; Persona owns discussion;
Persona/Mind review policy; Mind admits it; Bifrost publishes. Requested public
room and Eve surface ids remain unpublished until provider receipts exist, and
downstream rendering is explicitly irrelevant. A new typed collaboration
closure domain protects both families. Twenty-two high-authority families are
typed; direct inventory reports 124 substring occurrences across ten families.

The fake provider catalogs are deleted. `repo.tool_capabilities` advertised
expected tools without a host receipt; `repo.eve_surface` invented a live
surface/rows/lowering catalog without provider publication. Dispatch,
generators, closure, Verse classification, CLI help, and both dedicated smokes
are gone—1,188 source lines, no adapters. Real tool requests/host receipts,
Odin discovery, Eve connection receipts, and provider-owned composition remain.
Direct inventory is 71 occurrences across eight families.

`repo.body_manifest` is deleted as a daemon diorama: it invented Body/Verse/Eve
identity and capabilities in an unconsumed `epiphany.toml`. Runtime state,
provider advertisements, and repo birth receipts remain authoritative. A fresh
patch-placement error that put collaboration `[policy]` fields in the manifest
generator was repaired; a direct generator unit test proves the fields now
belong to collaboration policy and the deleted manifest family is rejected.
The recursive legacy smoke timed out and spawned nested Cargo trees, so it was
not treated as proof. Direct inventory is 56 occurrences across seven families.

Doctrine review no longer has a `Maintainer/Mind` composite owner or OR gate.
Imagination authors, Self routes, Maintainer reviews, Soul verifies, Mind admits
doctrine state, and Hands mutates `AGENTS.md` under receipts. Typed operations
closure and a counterfeit-Hands fixture protect the chain. The remaining
source inventory is 17 occurrences across six presentation families.

The six presentation families are now explicitly `presentationOnly=true` and
share no formatting assertions. Common commit/path/blob evidence remains the
closure proof; summaries, headings, markers, and checkboxes no longer
impersonate Soul authority. A whole-function regression test proves zero
`content.contains` calls remain in closure.

The beyond-closure scan has started. Deployment request no longer has an
`Idunn/Maintainer` composite owner: Self routes, Maintainer/Soul/Mind/Bifrost
review independently, and Idunn alone executes and authors deployment/aftercare
outcomes. A generic patch briefly struck secret-policy fields; compilation
caught it before tests, the fields were restored, and exact deployment tests
plus the full baseline pass.

PR request no longer has a `Bifrost/GitHub` composite owner. Self routes,
Bifrost owns the publication gate, Hands performs bounded PR action, and GitHub
only supplies provider outcomes. The GitHub receipt ledger row is now
GitHub-owned. Nine external-governance fixtures, 259 library tests, and all
binaries pass.

Consensus Brief keeps its honest draft/unconverged semantics but no longer
proves them with substring presence. Preparation owns its typed consensus,
Imagination route, inputs, authority, and privacy closure; a malicious comment
cannot counterfeit adoption denial. Twenty high-authority families are typed;
direct source inventory reports 278 substring occurrences across thirteen
remaining families.

The latest structural count is 32 closure family branches, 744 remaining
substring assertions in the closure region, and 1,284 lines in
`closure_contracts.rs`. Do not blindly generate a struct forest for every
presentation-only family. The eight converted high-authority families are
covered by `typed_closure_families_have_no_substring_authority`; prioritize the
next consequence-bearing request and require a named owner.

## Verification baseline

Artifact acceptance is split: Self routes, Maintainer decides acceptance,
Bifrost records accounting, and the typed request requires the acceptance
receipt. Operator receipt and accounting rows report those separate owners.
Nine external-governance fixtures, 259 library tests, and all binaries pass.

Metrics/accounting no longer has a `Bifrost/Maintainer` composite or OR gate.
Self routes, Bifrost accounts, Maintainer owns review-load evidence, and spend/
review-load receipts are required observations without authorizing spend or
ledger mutation. Contract catalogue and operator rows use the same ownership.

The last completed code pass had 259 library tests passing and all binaries compiling. Re-run focused checks for the next touched surface; use the full library/binary baseline before committing a new architectural cut.

Secret and dependency policy requests no longer use a
`Maintainer/Soul/Bifrost` pseudo-owner or Maintainer-or-Soul seal. Self routes,
Mind admits policy, and Maintainer review, Soul verification, Mind admission,
and Bifrost publication review are separately required. Dependency policy also
requires supply-chain audit evidence. The next conceptual-substitution targets
are the surviving composite decision fields in external governance and the
composite owner labels in operator projections.

Artifact acceptance, PR publication, and readiness seals are now aligned with
their split request bodies. Maintainer acceptance cannot substitute for Bifrost
accounting; Maintainer review cannot substitute for Bifrost publication, Hands
execution, or GitHub provider evidence; and readiness approval is Maintainer-
owned instead of ownerless. The unresolved external-governance field is
upstream sync's `operator_or_maintainer_authority_required`; trace its actual
effect owner before deciding whether it is alternate human authority or another
collapsed obligation.

That trace is complete: `repo.sync_request` performs no sync effect. It asks
Bifrost for an upstream ancestry proof after publication and separately
requires Maintainer review evidence. The fictional operator-or-maintainer gate
is removed; the request still denies merge, push, sync, and Hands authority.
Next inspect composite owner labels in operator projections, starting with rows
that may be interpreted as decision or execution ownership.

Receipt-directory projection cleanup has started. Eve connection rows now name
the target provider cluster rather than `Odin/Eve`; work-loop telemetry names
Self rather than its Hands/Soul/Modeling stage route. The next projection scar
is `repo_work_stage_for_family`: it groups distinct collaboration and governance
families under composite owners. Split those stage groups by actual family
authority; do not merely rename the composites.

The stage groups are now split per safe family, with regression coverage for
tool host, Imagination draft, Mind admission, Maintainer decision, Hands
execution, Bifrost proof/accounting/publication, Soul verification, Self
scheduling, and Idunn deployment ownership. Dependency policy and readiness
are explicitly classified instead of falling through to Unknown. Next trace
the remaining composite owner labels in repo-work readiness rows around the
Idunn/Soul, Soul/Bifrost, and four-reviewer projections.

Those readiness rows are split: Idunn owns aftercare evidence, Soul verifies;
Soul owns redaction and readiness sight; Maintainer owns readiness approval;
Bifrost owns publication review; the four required reviewers are an explicit
list. Next cut arrow-shaped handoffs such as `Persona->Imagination` out of owner
fields and preserve them only as routes.

Arrow-shaped owner values are now gone. Self owns intake-consensus readback and
publishes the Persona-to-Imagination handoff separately; feedback ledger rows
use their source Persona id; Bifrost owns consensus accounting. Continue the
conceptual-substitution scan beyond owner strings: look for status, route,
receipt, or cache fields that borrow consequence authority from another organ.

The first non-owner substitution is cut in `repo.tool_request`. The old
`requesting_agent="repo Persona/Self"` and `requester_owns_request=false` shape
is replaced by requester body identity, Self routing, Persona pressure
provenance, target-host execution ownership, and requester execution denial.
The typed closure now validates the identity and authority split. Continue
looking for fields where observation, request, admission, and effect ownership
are collapsed under innocent-sounding status or cache names.

Maintainer review is no longer substitutable by generic human response. Both
the Maintainer review request and doctrine update request now require explicit
Maintainer review, matching their receipt contracts and reviewer topology.
Continue auditing remaining OR-shaped fields, but distinguish legitimate input
alternatives (for example token count or cost summary) from authority gates.

Two further OR substitutions are cut. Consensus drafts require additional
public feedback rather than "human or Persona review"; Mind/Bifrost remain the
gates. Deployment requests require both script hash and script-review ref,
because identity and approval are distinct proofs. Continue classifying the
remaining OR-shaped fields by what invariant each alternative actually proves.

Deployment git identity is also split. Requests now require both watched ref
and source commit SHA, matching Idunn's receipt. A branch/ref identifies the
mutable trigger path; the commit SHA identifies deployed bytes. Do not collapse
them back into one convenient string.

Metrics packet completeness is now explicit: token summary, cost availability
status, review duration, and review-event count are separate requirements.
This permits honest unavailable pricing without treating cost and tokens—or
duration and event count—as interchangeable. Next inspect projected accounting
status to ensure receipt presence cannot close a row whose required dimensions
are absent.

That projection check is implemented. Bifrost metrics receipts now expose the
four typed dimensions; old receipts deserialize with absent optional fields but
remain incomplete. A negative projection test proves a receipt with IDs and
summary but no token evidence cannot report complete proof. Continue with
scheduler/cache projections that derive consequential status from presence.

The first scheduler substitution is cut. `heartbeat_status_projection` no
longer translates a readable state document into `status=ready`. It reports
store state as `missing` or `loaded` and derives separate scheduler physiology
as `unconfigured`, `active`, or `attention` from participants and pending-turn
state. The focused library suite, all-bin check, and native heartbeat smoke pass.
Continue through cache/freshness projections; do not let existence, successful
decode, or a latest mirror impersonate current consequence authority.

Operator cache readback is also purified. `epiphany-operator-run latest` joins
the latest intent to its exact receipt by `run_id`; a new intent cannot borrow
an older latest completion. Its statuses are lifecycle facts (`missing`,
`requested`, `completed`, `attention`, `orphaned-receipt`), never generic
`ready`. `epiphany-operator-snapshot latest` reports present state as `loaded`.
Negative unit coverage proves intent presence is only requested and mismatched
receipt identity cannot close it. Both native smokes, 259 library tests, focused
binary tests, and the all-bin check pass.

Rider bridge topology no longer claims readiness from path existence. Rider is
`discovered`, solutions are `found`, and repository topology is `gitDetected`;
the native bridge smoke passes with the renamed evidence states.

Unity bridge uses the same boundary: project version parsing is `pinned`, exact
editor path discovery is `resolved`, and the editor package predicate is named
`present`. Operability belongs to the subsequent `runStatus` receipt. The
native Unity bridge smoke, 259 library tests, and all-bin check pass.

The generic Persona Other mouth no longer models accepted Bifrost requests as
posts. Recent speech tracks `crossing_recorded` and
`same_target_crossing_count`; actual publication still requires an external
transport receipt. The old v0 `sameTargetPostCount` spelling is serialization-
only compatibility. Native Other-mouth smoke, 259 library tests, and all-bin
compilation pass.

Persona Aquarium bubbles no longer accept caller-authored readiness. The CLI
has no `--status`; successful writes derive `projected`; the strict intent
schema has no status field; and the output schema permits only `projected`.
Native smoke returns that value and an explicit `--status ready` invocation is
rejected. The 259 library tests and all-bin check pass.

Three live Epiphany-shaped read-only organs (Eyes, Proprioception, Soul) were
used as prompt-quality probes. They converged on caller status impersonating
character physiology. `epiphany-character-loop` no longer accepts `--status`,
constructs no fake heartbeat participant, derives stimulus `received`, and
reports activation `unknown` without scheduler evidence. Its packet now matches
the registered `schemaVersion`. Proprioception also found and removed the stale
repo-intake consensus status argument and demoted aggregate Persona status from
`ready` to `loaded`. Persona prompts now explicitly separate candidate, bubble,
eligibility, Mind admission, and Bifrost/provider delivery ownership. Native
negative/smoke checks, 259 library tests, 32 work tests, and all-bin compilation
pass. Operator correction: Modeling's mandatory typed patch is intentional
because the live searchable body map is its product; confirmation passes must
bank freshness and source-grounded map confidence. Improve the patch vocabulary,
do not add a no-state-change escape. Continue the P0 Discord/Reddit arbitrary-
JSON publication receipt cut found by Eyes and Soul.

That P0 cut is complete. Epiphany now validates Bifrost's real Discord/Reddit
publication output: exact action/outcome, target, provider id or URL, canonical
crossing receipt id, and returned provenance bound to the speech audit and
authority inputs. Discord fallback IDs are deleted. Native hostile smokes prove
exit-zero `{}` cannot publish. Posted artifacts count as history only when their
embedded receipt binds back to the artifact target and audit; forged
`status=posted` files do not count. The remaining Bifrost seam is command
cardinality: Epiphany inherits a CultMesh command id, so the coordinator must
ensure one typed command id per crossing rather than letting multiple posts share
one canonical `crossing_<command>` receipt.

## Repo frontier planning launch artery (2026-07-15)

Self's planning request is now v1 and binds the exact current RepoModel,
admission receipt, actionable Imagination frontier, runtime identity, and
authoritative thread. Coordinator launch commit replays that entire chain from
one transaction snapshot, forbids caller-authored planning projection and
mixed Modeling/repair/planning authority, injects the typed Imagination context,
and atomically persists a request-keyed
`RepoFrontierPlanningLaunchBinding`. Its SHA-256 covers the exact launch bytes
stored for the worker. Hostile tests cover swapped model/admission/frontier/
scope/organ/runtime/thread fields, wrong role/binding, prepopulated projection,
dual authority, single-use replay, and two concurrent launches; only one wins
and the loser leaves no job, worker launch, or opened event. Full core proof:
322 passed, 1 ignored. Next build the exclusive immutable Imagination result
echo carrying the typed candidate, then Mind-only Adopt/Refuse/Hold admission
and the narrow Adopt model transition that makes Hands routing true.

The immutable Imagination result slice is now complete. Runtime role result v2
adds an exclusive `frontier_planning_request_id` plus dedicated canonical
MessagePack `RepoFrontierPlanCandidate`. Persistence replays the exact planning
request, request-keyed launch binding, stored launch-document hash/projection,
runtime/thread, current model/admission/frontier, deterministic candidate
identity, and bounded safe paths. Planning results reject generic state, Self,
RepoModel, Verification, and Modeling authority cargo before writing. The
OpenAI ingress uses a specialized planning schema, supplies candidate
schema/contract itself, derives candidate identity from semantic cargo, and
does not expose generic patch mouths. The candidate remains embedded in the
immutable result; no standalone candidate document is written. Proof: core
325 passed/1 ignored; OpenAI runtime 9 passed; hostile missing/swapped request,
missing/adjacent/escaped candidate, wrong role, generic patch smuggling, exact
retry, and immutable collision checks pass. Next implement Mind-only terminal
decision plus the dedicated Adopt RepoModel transition from Imagination to
Hands, then delete the old internal candidate/adoption fixture writers.

## Prompted frontier admission review and canonical adoption (2026-07-15)

The repo-frontier planning nerve is complete and the preceding “Next implement
Mind” sentence is superseded. Self binds one exact admitted model/frontier
request; coordinator atomically binds the Imagination launch; Imagination
persists one immutable result with its sole embedded candidate; the runtime
derives a CAS-bound Mind review request; coordinator atomically launches a
bounded non-embodied admission-review procedure; that procedure returns one
immutable typed Adopt/Refuse/Hold judgment; and Mind's admission transaction
replays the entire chain before consequence.

Refuse and Hold are inert terminal receipts. Adopt atomically installs the exact
candidate and provenance on the canonical frontier, emits specialized review
and admission receipts, and alone changes Imagination to Hands. Self derives the
route from that model. Hands intent binds route, candidate hash, plan action,
and safe paths; Soul admission requires the exact plan command and receives the
checks, stop conditions, rollback steps, and commit message. Modeling may close
the frontier after Soul verdict only while preserving adopted execution anatomy.
There is no standalone candidate/adoption truth and no caller-authored decision.

Mind remains persistent state and admission authority, not an embodied chatty
lane. The prompted model call is explicitly a bounded admission-review
procedure serving Mind, with no heartbeat, lane memory, `roleAccept`,
`selfPatch`, or foreign organ patch mouths.

Hostile coverage includes swapped causal ids/hashes/jobs, failed worker output,
foreign cargo, duplicate/racing launch and terminal claims, generic adoption
bypass, post-adoption execution-anatomy mutation, and same-path command
substitution. The composed happy path proves Adopt through exact Hands command,
Verification/Soul, accepted Modeling incorporation, Resolved closure, and
byte-identical plan survival. Final proof: core 330 passed with one intentional
cross-process helper ignored; OpenAI runtime library 10 passed, both runtime
binary suites passed 6 each, and all core binaries check. Soul's final landing
audit passed. Exact concurrent admission retries are forced through a two-party
pre-CAS barrier in test builds; the loser reloads and authenticates the winner's
entire immutable decision/admission chain. The synchronization hook is compiled
only for tests, so production exposes no executable seam at the commit boundary.
The next organ is the production semantic index over typed graph claims so
Modeling owns a live persistent searchable Body map instead of making Hands
rediscover the repository.

The selected semantic substrate is Qdrant: available locally in Docker and
intended on the new Yggdrasil host (64 GB RAM, GTX 1080 for embedding/model
workers). It is a rebuildable projection, never canonical Mind or Modeling
truth. CultCache/CultMesh remains authoritative. Use separate typed Mind and
Modeling namespaces/collections or equivalently hard query partitions so
private doctrine/memory cannot leak into Hands Body retrieval and repository
claims cannot silently become doctrine. Every point must retain source document
id, schema version, canonical hash, provenance, authority/visibility scope, and
embedding-model version.

## Shared Mind/Modeling semantic nerve (2026-07-15)

The Qdrant organ is now live rather than planned. `semantic_backend.rs` is the
single typed Qdrant/Ollama wire boundary; workspace retrieval consumes it and
the previous local transport copy is gone. The memory graph derives stable
Mind/Modeling projection documents, stores locator/hash/provenance payloads in
physically separate collections, validates collection metadata from Qdrant,
synchronizes only exact swarm/partition scope, reloads every candidate from
current canonical state, and falls back to typed BM25 without losing truth.
Projection receipts are typed CultCache documents outside RepoModel.

`epiphany-memory-semantic` provides canonical index/context commands.
`epiphany-repo-model-bootstrap` atomically admits the initial RepoModel from
typed thread-state through runtime-spine ownership and ignores stale sibling
graph stores. The tracked schema-v0 `state/memory-graph.msgpack` is deleted.

Live local proof against Docker Qdrant v1.17.1 and Ollama
`qwen3-embedding:0.6b` (1024 dimensions) indexed 43 Mind documents into
`epiphany_mind_v1` and 3 fresh Modeling documents into
`epiphany_modeling_v1`. Four stale Modeling points were removed. Semantic
queries render canonical typed packets, ignore payload prose, exclude stale
summary descendants, and report whether semantic ranking or BM25 fallback ran.

Persona's private cache substitution is cut: its chunk schema, duplicated
Qdrant/Ollama clients, reindex-on-recall path, direct payload rendering, and
`epiphany_persona_memory_v0` collection are deleted. Heartbeat Persona recall
uses the shared Mind projection restricted to Persona's canonical domain. The
actual Modeling organ-state was also repaired: four surviving Proprioception
phrases in identity/goal/memory prose are now Modeling, validation is clean,
and the Mind collection was rebuilt from that repaired state.

Next: schedule projection refresh as completion-gated physiology, publish typed
projection health through CultMesh/Eve, and refresh the stale source anchors
that leave the current Body map honestly thin. Do not let the scheduler,
Qdrant, Yggdrasil, or presentation become state authority.

## Semantic completion-law correction (2026-07-15)

The first scheduling map exposed two surviving substitutions before daemon
work began. Persona recall derived a swarm namespace from fixed organ agent
ids, which are identical across Epiphany swarms; and Qdrant failure still used
the unconstrained canonical planner. Both are cut. `state/agents.msgpack` now
contains the immutable typed swarm identity `gamecult.epiphany.main`; Persona
recall and the semantic CLI require that exact document, and the Mind
collection was rebuilt under that scope with 43 documents. Canonical fallback
uses the same partition-constrained planner as semantic success.

`memory_graph::semantic_index` now defines typed projection obligation and
attempt documents, exact source-head and receipt causality fields, and pure
derived `pending|failed|stale|ready` health/query-eligibility law with hostile
mismatch tests. These types are foundation, not a readiness claim: runtime
admission does not yet atomically emit obligations, indexing does not yet claim
and discharge them, queries do not yet require the newest exact success, and
CultMesh/Idunn wiring deliberately remains absent. The next cut is canonical
admission ownership, not a polling loop.

## Atomic semantic admission (2026-07-15)

The causal cut is now implemented. Runtime-spine is immutably bound to the
canonical agent-Mind swarm identity. All three RepoModel writers atomically
companion canonical admission with one exact Modeling projection obligation;
the live legacy runtime was explicitly migrated without rewriting its model or
migration receipt. Mind self-patch and lifecycle admission now use a fixed-role
generation CAS that writes canonical rows, generation witness, persisted Mind
admission receipt, and Mind obligation together. Every extension re-hashes the
canonical rows and authenticates the prior witness. Bootstrap/import/repair/raw
replacement/trait seeding cannot write after a generation exists.

Core proof for the landed atomic admission pass: 343 tests passed, 0 failed, 1
intentional ignore; all binaries compile; the OpenAI runtime suites pass 22
tests. The current `state/agents.msgpack` has now been admitted without row
mutation as generation 1 under swarm `gamecult.epiphany.main`, with immutable
witness `mind-generation-eb394e34-1c73-5d0a-8691-6ff1ba6145a4`, its Mind
admission receipt, and its exact projection obligation. Repeating the migration
is idempotent and validation remains clean. Next replace the direct projector
CLI with restart-safe scope claims, attempt/terminal receipt CAS, observed
zero-document synchronization, and exact query eligibility; then add
CultMesh/Idunn physiology.

## Restart-safe semantic projector and gated query (2026-07-15)

The projection executor now serializes each `(swarm, partition)` mutation scope
with a durable executor-bound claim and fencing epoch. An internal recovery
transition proves that an abandoned attempt can be failed and fenced, but it is
not exposed until Idunn supplies typed stale/recovery authority; arbitrary peers
cannot steal a live scope. Terminal success requires the same live claim, exact
bound receipt, unchanged canonical source envelopes, and observed Qdrant scope.
The raw indexer and ordinary receipt writer are no longer public authority
surfaces. Empty projections bypass Ollama and delete only their exact scope.
Non-empty projections observe exact IDs and typed payload identities after
synchronization.

Query gating is live in the CLI and Persona heartbeat. Without the newest exact
obligation/success pair they use canonical BM25 without contacting Ollama or
Qdrant. With the pair they use Qdrant only for ranking, reload canonical typed
documents, and ignore payload prose. The live `gamecult.epiphany.main` Mind and
Modeling obligations were discharged through the protocol: 43 Mind documents
and 3 Modeling documents, both 1024-dimensional. Repeated execution returned
the same immutable receipts. Core proof: 349 passed, 0 failed, 1 intentional
ignore; all binaries type-check.

Next: derive and publish provider-owned health through CultMesh, make Idunn own
projector process survival/restart and explicit fenced recovery routing, then
run the deployability audit against daemon packaging, configuration, and live
Yggdrasil assumptions. Neither CultMesh nor Idunn may mint readiness.

## Physical semantic fence correction (2026-07-15)

Hostile recovery review found that the logical claim epoch fenced CultCache
terminal receipts but did not fence the Qdrant actuator. A resumed old process
could still mutate the shared `(swarm, partition)` points after replacement.
The physical projection is now isolated by exact obligation, claim id, and
claim epoch in its UUIDs, typed payload, mutation/observation filters, success
receipt v1, and opaque-readiness-selected query filter. Old receipt v0 decodes
only for migration inspection and cannot become ready. A second process cannot
share a running claim merely by reusing its executor label.

The readiness loader now authenticates the full persisted success chain:
obligation, succeeded scope claim, exact claim id/epoch on the receipt, and the
claim's succeeded attempt. Hostile substituted claim, epoch, and attempt
evidence fails closed. The live Mind and Modeling projections were rebuilt into
claim-owned namespaces (43 and 3 documents at 1024 dimensions), and exact live
queries used Qdrant ranking while resolving canonical documents. Core proof is
353 passed, 0 failed, 1 intentional ignore.

Deployment Eyes found the live boundary: local Qdrant and Ollama work, but no
Epiphany/Idunn projector service is installed. New Yggdrasil has healthy GPU
Ollama, Odin, and Idunn, but no Qdrant, Epiphany artifact/config, or projector
target. Local Qdrant is unexpectedly published on all host interfaces. Do not
deploy dual projectors or silently move canonical Mind/Modeling stores.

Next: cut Idunn supervisor writes to provider-owned daemon heartbeat/status;
then publish derived semantic health through CultMesh and add typed Idunn
executor/recovery authority plus open-obligation discovery. Recovery remains
withheld until that authority exists.

## Idunn/provider-status ownership cut (2026-07-15)

The supervisor no longer writes provider heartbeat/status. Idunn owns only
staleness observation, immutable poke intent/receipt events, restart command
execution, policy, and backoff. The target `epiphany-cluster-daemon` remains the
sole production owner of its status, operator action, and heartbeat timestamp.
Command exit zero now records `awaiting-provider-heartbeat`, never `ready`, and
restart pressure remains until a provider heartbeat newer than the completed
attempt proves recovery.

Poke intent/receipt v1 binds the pre-intervention heartbeat and request,
attempt, and completion times. Immutable identity and chronological `latest`
advance in one CAS: exact retry is idempotent, collision is refused, and late
replay cannot rewind sight. Receipt-directory resolution is derived only; it
cannot repair provider truth. The bounded survival rehearsal proves two
successful restart commands leave the provider envelope unchanged and produce
distinct awaiting receipts, then a real provider heartbeat resolves the
lifecycle observation and clears restart pressure.

Renderer-neutral semantic projection health now leaves the provider through
CultMesh as non-authoritative local-area sight. Publication reauthenticates the
sealed Mind or Modeling input, persisted obligation, canonical authority
envelopes, and—when ready—the complete succeeded claim/attempt/receipt chain.
The mirror carries only pending/failed/ready, canonical fingerprints,
bounded counts, provider/incarnation identity, and timestamps; it carries no
error text, path, command, payload prose, or readiness capability. Immutable
events plus a per-swarm/partition chronological CAS prevent a delayed writer
from rewinding latest sight. `epiphany-memory-semantic health` repairs the
mirror independently, and `epiphany-verse-query semantic-health` reads a named,
compact report without touching canonical state. A stale sealed input is
refused rather than published as if it described the current canonical head.

## Idunn semantic executor authority cut (2026-07-15)

Executor labels no longer open semantic mutation authority. Idunn acquisition
now atomically writes a consumed typed executor grant, a scope claim, and its
running attempt against the exact persisted obligation and predecessor claim.
The grant binds executor incarnation, purpose, Idunn incarnation, predecessor
status/id/epoch, and resulting claim id/epoch. `execute` cannot repair a
succeeded claim; `repair` requires that exact succeeded predecessor. The
projector CLI now requires the acquired claim id and authenticates its consumed
grant before Qdrant work or terminalization. Acquisition reauthenticates the
whole sealed canonical input and carries its authority envelopes through the
same CAS, so an old persisted obligation cannot mint against an advanced head.

Fenced recovery now consumes typed evidence rather than free-form reason text.
The CultMesh bridge authenticates the exact immutable Idunn poke intent and
successful `awaiting-provider-heartbeat` receipt, plus a provider-authored ready
heartbeat event that causally follows receipt completion and names the
replacement provider incarnation. The heartbeat must also name that exact
restart receipt as its startup cause. Their envelope digests enter a consumed
recovery authorization. One CAS fails the abandoned attempt, advances the
epoch, creates the replacement claim/attempt, and records that authorization.
Recovery rotates authority only; it cannot execute projection or mint success.
Provider heartbeat events are immutable and their latest pointer advances
monotonically per daemon/incarnation.

Authority map: canonical Mind/Modeling admission owns obligations; Idunn owns
executor assignment and explicit recovery decision; the projector owns claim-
bound mutation and terminal evidence; the query gate alone authenticates
readiness. Health, liveness, timers, command exits, Qdrant state, CultMesh,
Eve, and swarm overview remain derived sight and forbidden writers. Initial
execution, retry, repair, and recovered execution share the same claim-
authenticating projector primitive.

## Workstation semantic projector service body (2026-07-15)

The open-obligation pulse and single process owner are now implemented as one
workstation-local `epiphany-memory-semantic-projector` body managed by Idunn.
Its constructor requires two distinct canonical files--one Mind store and one
Modeling runtime store--and refuses unless their sealed inputs name the exact
partitions and the same immutable swarm. It takes a host OS singleton for that
canonical pair before minting its process-stable provider/executor incarnation.
Mind and Modeling remain separate authorities, claims, collections, and
receipts; they merely share one survival body.

Every pulse reloads both canonical inputs. It stores no open-obligation queue
and allows at most one global action, rotating fairly between actionable
partitions. Ready, foreign-running, and stale inputs never execute. A succeeded
claim is not itself terminal evidence: valid receipt-v2 health derives Ready,
while an exact authority-authenticated succeeded claim whose receipt is legacy
or invalid derives Repair and acquires the typed Idunn repair path. Pending and
failed inputs acquire exact Idunn execute authority before using the one
crate-private executor. A running claim owned by the current provider
incarnation resumes directly without minting a second grant; a foreign running
claim remains pressure until exact recovery. Overlapping pulses return busy,
and the serve cooldown begins only after the bounded pulse completes. A fault
loading one source does not turn the other source into false readiness or hide
its valid action.

The old production mouths are gone: `epiphany-memory-semantic` no longer has an
`index` arm or claim flag, there is no public raw execute function, and the
supervisor no longer exposes general semantic acquisition. Recovery remains a
narrow supervisor action. It authenticates the abandoned claim, exact Idunn
lifecycle intent/receipt, and causally linked replacement-provider heartbeat,
then rotates claim authority only. The running service recognizes that exact
recovered claim as its own and resumes it on a later ordinary pulse.

Idunn owns a specialized reserved service-policy writer for fixed service id
`epiphany-memory-semantic-projector-service`. It derives the packaged sibling
binary, fixed executor identity, both canonical stores, infinite serve shape,
and restart-always policy. The generic managed-service writer refuses that
reserved id; callers cannot substitute a command, service id, restart mode, or
finite child lifetime. Existing Idunn managed-service reconciliation owns
process survival and lifecycle receipts. The OS singleton prevents a service
and an interactive process from simultaneously owning the same canonical pair.

Provider heartbeat and per-partition semantic health remain derived sight.
Heartbeat `ready` means the body owns its singleton, validated the source pair,
and completed a healthy pulse/publication pass; it is not semantic query
readiness. Only the canonical success chain admits query. Pulse JSON, health,
heartbeat, Qdrant, Ollama, Eve, and swarm overview cannot create obligations,
grants, recovery, terminal evidence, or readiness.

Deployment is deliberately not live. The chosen topology is the workstation
beside its canonical stores, with Yggdrasil permitted to supply embedding over
WireGuard. Current local inspection found Docker container `voidbot-qdrant`
stopped with exit code 143. Its name records foreign ownership, so Epiphany did
not restart, adopt, reconfigure, or claim it. No semantic projector service
policy has been published and no restart proof has been claimed. Moving only
the projector to Yggdrasil or starting a second partition/projector body remains
forbidden.

Next: establish explicit Qdrant ownership and a live workstation-reachable
endpoint, confirm collection compatibility and the Yggdrasil Ollama route,
then publish the reserved projector policy. Run one bounded live pulse, prove
both derived health rows and provider heartbeat, kill the body once, and prove
Idunn restart plus causally linked heartbeat/recovery/resumption. This is a
preflight and survival proof, not permission to call deployment complete.

## Managed semantic launch authority rebuild (2026-07-15)

Deployment inspection found incompatible lifecycle stories. The packaged
managed-service launcher spawned an infinite child and wrote a mutable v0
event, while semantic recovery expected a completed daemon-poke receipt from a
blocking restart command. That command could never truthfully witness this
service.

The authority is now one causal chain. The reserved managed policy is desired
state. Idunn preallocates a UUID, injects it into the child, spawns once, and
atomically writes an immutable v1 lifecycle receipt binding child PID, spawn
completion, exact policy id and envelope digest, fixed projector daemon, and
startup correlation. Failed persistence kills and waits for the child. The
child authenticates that receipt against the current policy before constructing
the service body or publishing a pulse or heartbeat. A launch receipt proves
spawn completion, never readiness.

Semantic recovery authorization is v2 and consumes only the current policy,
its exact launch receipt, and a strictly later correlated provider heartbeat.
Advancing policy invalidates an older launch receipt. Daemon poke remains an
operator intervention surface for ordinary daemons but cannot authorize
semantic recovery. Hostile tests cover unrelated heartbeat, policy advance,
successful exact recovery, and single-use refusal. Full library proof is 371
passed/1 ignored; supervisor tests are 7/7; all binaries compile.

Infrastructure truth also changed. The retired local `voidbot-qdrant` must stay
stopped. Shared Yggdrasil Qdrant is authoritative and reaches this workstation
through the ops tunnel at `127.0.0.1:16333`; Yggdrasil Ollama is reachable at
`http://10.77.0.1:11435`. Both Epiphany collections are green 1024-dimensional
cosine collections using `qwen3-embedding:0.6b`.

Next: publish the reserved policy and place both the managed-service reconciler
and Qdrant tunnel under durable Idunn/OS survival ownership. Live proof must
cover real child handshake, bounded pulse, heartbeat and health, forced
restart/correlation, singleton refusal, and safe exact recovery/resumption
where an abandoned claim can be created without forging canonical work.

## Live semantic projector proof and canonical purification (2026-07-15)

The reserved policy is live against shared Yggdrasil Qdrant through
`127.0.0.1:16333` and Yggdrasil Ollama through `10.77.0.1:11435`. Live launch
pressure exposed three fossils that narrow tests had not reached:

- local Verse still contained the deleted ownerless
  `epiphany.cultmesh.operator_status`; an explicit backed-up migration removed
  its one key instead of registering extinct authority;
- both canonical stores contained v0 claims/attempts and index receipts from
  before Idunn fencing; explicit migration atomically retired those rows from
  active authority rather than inventing missing incarnation or grant fields;
- Mind and Modeling canonical cache catalogs registered claims, attempts, and
  receipts but not the executor-grant/recovery types now written into the same
  physical stores; both readers now register the whole semantic authority
  family.

CultCache now provides an atomic exact-envelope batch deletion primitive for
these migrations. The projector launch receipt also binds executable SHA-256,
and the child verifies both its own PID and its bytes against that receipt
before constructing the service body. Health publication errors are bounded in
the pulse output instead of disappearing into a count.

Fresh Idunn-granted projection rebuilt Mind (43 documents) and Modeling (3
documents), both 1024-dimensional and query-eligible. Live semantic context
queries ranked canonical documents for both partitions while ignoring Qdrant
payload text. A clean steady pulse inspected both sources, remained idle, and
published a ready heartbeat with no source or health faults. A second
interactive projector was refused by the host singleton. Killing child PID
22992 caused the reconciler to launch PID 29868 with receipt
`daa89e4f-4301-40eb-aa87-5b52c5017221`; the typed service-status readback proved
the new ready heartbeat names that exact receipt and executable hash.

Full proof is 375 library tests passed/1 ignored, 7/7 supervisor tests, and all
binaries compiling. The live child and reconciler remain running, but survival
is not yet deployable physiology: reconciler PID 22072 and SSH tunnel PID 22728
are detached processes, not durable OS-owned services. The existing generic
Windows `sc.exe create` path is not sufficient because this console binary does
not implement the Windows service-control handshake. Build a real service host
or explicit Task Scheduler owner with restart policy, give the Qdrant tunnel
the same durable treatment, and prove startup/reboot recovery. Live abandoned
claim recovery remains unproven; use a safe fixture rather than forging work.

## Windows survival authority rebuild (2026-07-15)

The workstation survival cut rejected the generic Windows SCM path. The
Epiphany binaries are ordinary console programs and never implemented the
Windows service-control dispatcher, so `sc.exe create`, elevated service
runbooks/audits, cluster-service install/control, and `swarm-online-runbook`
were presentation of authority the body did not possess. Those routes and
their old receipts are historical evidence only; they must not steer current
deployment, readiness, or follow-up work.

Host survival is now deliberately narrower and honest. Current-user tasks
`Epiphany-Idunn-Managed-Service-Reconciler` and
`GameCult-Yggdrasil-Tunnel` run as Meta with `InteractiveToken`/Limited
privilege, `AtLogOn` plus a one-minute recurring recovery trigger,
restart-on-failure at one-minute intervals for 999 attempts, `IgnoreNew`, zero
execution limit, and exact direct foreground actions. Task Scheduler owns
process presence after login. It does not own projector children or semantic
state. Idunn still owns the managed-service reconciliation loop and projector
launch/restart correlation. `gamecult-ops` owns the Yggdrasil tunnel profile
and foreground SSH process; Epiphany merely consumes the pinned local Qdrant
ports. No wrapper, detached grandchild, or service-manager receipt is a second
survival owner.

Live recurrence proof killed reconciler PID 24152 and observed exactly one
replacement PID 25428. Killing projector PID 29868 then caused Idunn to launch
PID 19888 with launch receipt `4d406060-6582-473a-9322-9f4ccc8b322f`;
ready heartbeat `79fe3143-28a8-40fe-9547-0287159082e6` names provider
incarnation `projector-59217a66-e3e2-44c3-b000-36af3d43b043` and the exact
startup correlation. Tunnel recurrence killed SSH PID 22652 and observed PID
30444 start at 11:37:44 with Qdrant REST healthy.

The pre-reboot deployment repair then packaged
`epiphany-memory-semantic.exe` beside the release supervisor and projector,
deployed the rebuilt supervisor, and left
`Epiphany-Idunn-Managed-Service-Reconciler` Running. Deployment queries must
receive the real transport coordinates explicitly through
`EPIPHANY_QDRANT_URL` and `EPIPHANY_OLLAMA_BASE_URL`; ambient defaults are not
deployment proof. With those variables set, both packaged Mind and Modeling
queries returned semantic ranking. The provider status command no longer calls
that sight `ready`: it returns `provider-correlated` or `provider-degraded`, is
explicitly non-authoritative, and points to semantic query admission as the
only readiness owner.

One process-lifecycle scar matters for the reboot audit. Stopping the Task
Scheduler reconciler leaves its already-detached projector child alive. A
current parent PID is therefore derived process sight, not durable custody, and
a stop/start rehearsal can accidentally reuse the old child. The real proof
must observe a fresh post-boot scheduler -> reconciler -> exactly-one-projector
chain, a new launch-correlated provider heartbeat, and successful packaged
Mind and Modeling query admission after the tunnel returns.

The limitation is part of the contract: this is after-login recovery, not
pre-login, boot-time, machine-wide, or Windows-service operation. The remaining
Soul proof is a real reboot/logon cycle showing both scheduled tasks running,
the tunnel ports restored, one Idunn-managed projector child, and a fresh
launch-correlated provider heartbeat followed by semantically ranked packaged
Mind and Modeling queries. Reboot remains a host-wide action requiring exact
live operator permission. Do not spend another pass polishing the dead SCM
rite.

## Isolated semantic abandoned-claim recovery proof (2026-07-15)

The feature-gated native `epiphany-semantic-recovery-smoke` closes the recovery
fixture wound without forging canonical Mind or Modeling work. It creates
GUID-scoped Mind, Modeling, and Verse stores. A real projector acquires epoch 1,
stalls against a fixture-local endpoint, and is killed. The supervisor then
consumes the exact current semantic policy, managed-service lifecycle receipt,
and provider heartbeat to recover epoch 2. A replacement real projector
consumes that recovery authorization through the common execution path.

Typed read-only inspection proves the abandoned attempt is failed, the recovery
authorization is consumed, the epoch-2 claim remains running under the
replacement incarnation, and the old owner cannot authenticate it. Actual
semantic readiness and query eligibility are present. Wrong-heartbeat and
single-use attempts fail without changing canonical fixture bytes.

The Qdrant collections use GUID names, are preflighted absent, and are deleted
and verified absent by exact 404. The smoke accepts no live/default Mind,
Modeling, or Verse stores and exposes no fixture-only mutation mouth. Passing
artifact:
`C:\Users\Meta\AppData\Local\Temp\epiphany-semantic-recovery-1152736f-b8a2-4cf0-ab24-c8a3993a0eb7\proof.json`.

This proves the exact recovery/resumption and fencing path. It does not prove
global deployability. The remaining deployment evidence is still the real
reboot/logon survival cycle above.

## Runtime surface capability purification (2026-07-15)

The systemic surface audit separated a real local projection from false wire
authority. Commit `65445623` remains valid: it rebuilt
`epiphany.persona_surface.v0` as a strict, content-free MVP-status projection
over typed Persona artifact references, registered the real Reddit and Other
artifact schemas, and kept speech eligibility, Mind admission, Bifrost
acceptance, publication, and provider delivery outside the projection. Those
schemas remain discoverable vocabulary and local projection validators. That
does not make the projection a CultNet runtime capability.

All seventeen unbacked `epiphany.surface.*` mutation contracts and their dead
runtime constants are now removed. None had a typed document writer, Snapshot
resolver, or action dispatcher; the compound MVP-status JSON was not a
substitute for those owners. Runtime Hello and runtime status now derive the
current executable mutation-contract list instead of trusting the historical
`supported_document_types` stored in runtime identity. The live canonical
store still carries that old field as inert history, but it cannot decide wire
or status truth. A real canonical hello-frame contained zero
`epiphany.surface.*` capabilities after the cut.

Re-admission is deliberately expensive: an owning organ must prove
`provider publish -> typed CultMesh document -> Snapshot -> schema validation
-> Eve lowering`. Any advertised action additionally needs a real typed
dispatcher and receipt path. Schema registration alone proves vocabulary, not
runtime support.

The adjacent central Eve substitution is cut. The generic provider publisher
and its heartbeat call are deleted; a cluster heartbeat now owns liveness only.
Live Odin, Eve, and daemon-tool directories expose no provenance-free v0 rows,
and explicit bootstrap retires stale rows of those exact families. The stable
topology `eve_surface_id` remains routing/address metadata and proves neither
presence nor composition. Provider availability can return only through an
owning provider's provenance-bearing typed contract; advertised actions also
need a real dispatcher and receipt path.

This does not replace the current deployment next action: complete the real
Windows reboot/logon proof for Task Scheduler, the Yggdrasil tunnel, Idunn, and
the launch-correlated semantic projector heartbeat.

## Production coordinator fixture authority cut (2026-07-15)

The production `epiphany-mvp-coordinator` is again a scheduler, not a smoke
ventriloquist. Typed coordinator status and accepted evidence own the selected
action. The normal `continueImplementation` arm remains the single shared path
that may produce a Hands gate, but caller fixture text can no longer force that
arm or suppress review.

- Owner: typed coordinator status plus accepted role/evidence state owns the
  next action; Hands/Substrate review owns implementation permission.
- Inputs: the selected runtime store, current typed thread state, accepted role
  results, and explicit review state.
- Outputs: coordinator steps, final action, run receipt, and a Hands gate only
  when the typed action reaches the shared implementation arm.
- Derived state: smoke scenarios, fixture workspaces, forced pressure, source
  drift, and dry-compaction observations are verification cargo only.
- Forbidden writers: production CLI flags, callers, wrappers, and fixture
  helpers cannot choose coordinator action, mark it auto-runnable, waive
  review, seed canonical scheduler state, or manufacture a Hands gate.
- Shared paths: production and smoke exercise the same typed status/action and
  Hands-gate machinery after their inputs exist; the smoke does not add a
  second decision owner.
- Cut line: all production fixture flags, overrides, and helpers were deleted.
  Fixture assembly lives in `epiphany-mvp-coordinator-smoke`, confined to
  `.epiphany-smoke/mvp-coordinator`.

Negative proof rejects all seven retired flags at the production mouth while
the dedicated smoke remains inside its fixed root. No legacy flag can target a
caller-selected runtime store. Eyes has banked one adjacent candidate, not a
completed finding: sibling Bifrost subprocess JSON still participates in a
readiness view and needs a focused audit for typed provider identity and
provenance before any cut is justified.

The deployment next action is unchanged and permission-bound: only with exact
live operator approval may the real reboot/logon proof run. It must establish
the restored tasks and tunnel, a fresh reconciler -> exactly-one-projector
chain, launch-correlated provider sight, and admitted semantic queries. Do not
infer reboot authority from this scheduler cleanup.

## Bifrost readiness projection deletion (2026-07-15)

Epiphany no longer treats a sibling Bifrost path, executable presence, exit
zero, parseable JSON, or caller-supplied readiness booleans as provider sight.
The `epiphany-mvp-status` sibling subprocess projection and its `bifrostBridge`
output are deleted, as are `epiphany-bifrost-bridge-status-smoke`, the aggregate
Persona readiness fields, and wrapper readiness summaries. Missing authenticated
typed provider evidence remains unknown.

The surviving Persona Discord, Reddit, and future-surface paths are mouths, not
Eyes. Epiphany owns speech eligibility and may invoke a configured Bifrost
actuator for one named crossing. A validated returned receipt is bound only to
that speech/request artifact as evidence of the single consequence; it cannot
establish provider inventory, liveness, capability, readiness, publication, or
future operability. The cross-mouth aggregate is deleted. Each Persona mouth
tests only its own fail-closed eligibility, target-bound receipt shape, and
private-state sealing.

Authority map: the participating Bifrost/provider boundary owns consequence
truth; one eligible crossing request and configured actuator coordinates are
inputs; one artifact-scoped transport/request receipt is the output; global
readiness is not derived. MVP status, wrapper summaries, cross-mouth aggregators,
sibling-path probes, and caller booleans are forbidden readiness writers. All
three mouths retain the same eligibility-to-receipt boundary without sharing
consequence authority. The deletion line was the sibling readiness projector,
both aggregate smoke binaries, aggregate output fields, and status presentation.

The deployment next action remains unchanged and permission-bound: do not
reboot without exact live operator approval. With that approval, prove both
scheduled tasks and the tunnel return after logon, establish a fresh reconciler
-> exactly-one-projector chain and launch-correlated heartbeat, then obtain
semantic ranking from packaged Mind and Modeling queries using the explicit
Qdrant and Ollama endpoints.

## Freshness and reorientation authority repair (2026-07-15)

The freshness surface no longer converts missing evidence into confidence.
Retrieval `Ready` is Clean only when `dirty_paths` is empty; a Ready label with
one or more dirty paths derives Stale and therefore cannot authorize Resume.
The legacy thread graph projection cannot produce `Ready`. Explicit frontier
dirty paths, open questions, or open gaps prove Stale; otherwise it is
Missing/Unknown because legacy checkpoint identity cannot see canonical
RepoModel admission. A watcher
with no buffered changes is also unknown/unavailable:
there is no watcher generation, cursor, start boundary, or continuity receipt
from which silence could prove cleanliness. Observed changes remain valid
positive evidence.

`recommend_reorientation` now owns the whole Resume/Regather verdict. Resume
requires a resume-ready investigation checkpoint, retrieval Clean, graph
Clean, and watcher Clean or Unknown. Retrieval or graph values other than
Clean, and watcher Dirty/Stale/Changed, force Regather with explicit reasons.
Path overlap remains explanatory derived state; it cannot rescue a stale or
unknown map. `epiphany-mvp-status`, worker launch, coordinator, and CRRC all
consume this decision. `surfaces/jobs.rs` consumes the same derived graph
freshness judgment for graph-remap work instead of maintaining a second churn
string tribunal.

Authority map: `derive_freshness` owns freshness judgment from canonical
retrieval state, explicit legacy frontier pressure, and
positive watcher observations. `recommend_reorientation` owns the action.
Their outputs are derived read projections and one reorientation decision.
Watcher silence, MVP mappings, jobs, worker launch, coordinator, and CRRC are
forbidden readiness/action writers. All launch and operator paths share the
same derivation and decision primitives.

The ungrounded `churn.graph_freshness` and
`graph_checkpoint.graph_revision` fields were deleted. Checkpoints retain
identity and frontier content; churn retains understanding, diff, warning, and
unexplained-write evidence. Canonical RepoModel revision/hash plus its exact
Mind-issued `RepoModelAdmissionReceipt` proves only admitted map identity.
Future observed-ready-at also requires fresh Body observations bracketing exact
Body-grounded model admission and retrieval coverage for one manifest root;
Mind derives the joined projection. Do not build a bridge from snapshot
metadata, watcher silence, timestamps, or historical event continuity.

## Repository Body substrate landed (2026-07-15)

Native `repository_body_observer.rs` and `epiphany-repository-body` expose an
explicit runtime-bound bind step, observe, pure status, and smoke for
`git_worktree` state. Bind pins caller workspace ID to the existing validated
runtime/swarm/source identity plus canonical Git root and policy. Two equal
isolated Git-index scans feed immutable CultCache observations/current-head CAS.
Unchanged raw manifest does not advance generation; missing status creates no
store. The observer makes no historical continuity claim and has no Ready field. Sparse checkout is
rejected, submodules are gitlink-only, and downstream joins remain absent.
The CultCache store is required to live outside the observed worktree.
Canonical path checks run before any bind/store write, global excludes are
disabled, and corrupt/non-commit HEAD fails closed. This slice persists accepted
stable observations only; failed attempts advance no head.
All Git calls share one sanitizer that removes ambient Git repository/object/
ref/index/namespace and injected-config authority before applying explicit
observation policy.
The isolated index enumerates ignore-aware UTF-8 paths/modes/gitlinks, but its
clean-filtered tree OID is auxiliary. Raw file bytes (or non-followed symlink
target bytes) feed an ordered manifest whose domain-separated SHA-256 root is
the authoritative Body identity. Manifest, observation, and manifest-root head
commit atomically. Gitlinks are nonrecursive; unrepresentable paths fail closed.
Bind now installs one immutable runtime-side Body-store route containing the
canonical external locator and exact Body-binding hash. Reads validate runtime,
swarm, workspace, path, and Body binding; a runtime cannot substitute a second
Body store. The route is the locator nerve used by the grounded chain below.

## Modeling thinks from an authenticated Body observation (2026-07-15)

Coordinator-owned Modeling launch observes the bound repository before worker
thought and seals a typed `RepositoryBodyObservationBasis` into the immutable
launch. Modeling output contract v3 requires an exact worker-authored echo.
Result ingress reloads the launch and refuses missing, swapped, or non-Modeling
basis cargo. Mind admission review v1 and receipt v5 carry the same basis; the
admission CAS validates launch/result/review equality and the referenced
historical Body artifacts before copying it into the receipt. It never samples
current Body as a substitute. A valid historical basis remains admissible after
the repository changes because it proves what Modeling saw, not timeless
freshness. Direct Mind adoption and legacy migration remain explicitly
ungrounded rather than manufacturing retroactive evidence.

## Readiness join remains deliberately impossible (2026-07-15)

Modeling audited the next ownership seam after the Body observer landed. Mind is
the only coherent owner for a derived repository-readiness projection, but the
remaining input required to emit observed-ready-at does not exist yet: exact
workspace-retrieval coverage for that same Body manifest. RepoModel admission
is now grounded to its exact pre-thought Body observation. The existing
semantic projector proves exact query eligibility for an admitted RepoModel
projection; the legacy workspace retrieval JSON manifest proves only path/size/
mtime/chunk cache agreement. Do not join its `Ready` label, empty dirty paths,
watcher silence, Git OIDs, timestamps, counts, or Qdrant presence into repository
readiness. Historical continuity was audited and rejected as another conceptual
substitution. The correct join observes Body root R1, validates every artifact,
observes R2, and emits only a time-bounded result when R1=R2. Watchers and Hands
receipts trigger recomputation; they never replace either observation. Build
order is now typed Body-bound retrieval coverage, the Mind-owned race-safe join,
then deletion of local interpretations.

## False workspace retrieval authority cut (2026-07-15)

`retrieval.rs` was production-unwired: no runtime caller, JSON manifest
persistence, path/size/mtime/chunk identity, divergent exact/semantic walkers,
path-derived Qdrant collections, and counterfeit `Ready` from missing manifests
and query-time BM25. The module and public re-exports are deleted. Legacy thread
`EpiphanyRetrievalState` is presentation-only: clean `Ready` projects Missing
and its indexing job is unavailable/unowned; explicit dirty/stale input may warn
but cannot prove coverage. The live RepoModel semantic projector remains intact.

Next build: consume an authenticated historical Body manifest, classify every
entry under one versioned policy, verify eligible bytes against Body hashes,
project into a Body-root/policy/epoch isolated collection, observe the exact
expected point set, and publish an immutable CultCache coverage receipt. Do not
resurrect the JSON manifest as a compatibility source.

## Body-bound coverage contracts landed (2026-07-15)

The native coverage substrate now admits only an authenticated historical Body
manifest, exhaustively classifies its entries under a versioned policy, and
seals Body/policy/classification identity into an immutable obligation. A typed
projection plan binds the exact expected point set and derives an epoch-isolated
Qdrant collection from Body plus projection/embedding authority; the caller has
no collection-name lever. Receipt/head validators require an exact scroll-
observed point set and exact obligation/plan join.

This is contract substrate, not operational coverage. There is deliberately no
persistence writer, projector, Qdrant call, query eligibility path, or readiness
join. Next action: build the single CAS-owned projector/store path that verifies
eligible live bytes against the selected Body manifest, executes the sealed
plan, scroll-observes the physical point set, and only then publishes the
receipt and head.

Projector ownership is now mapped before implementation. Persist all coverage
state in the Repository Body store so terminal success can CAS against the exact
current Body head; never create a runtime-store coverage oracle. Add a sealed
claim/attempt lifecycle, verified historical-byte reads, deterministic UUIDv5
point descriptors, and a plan-sealed ID-to-payload binding root. Whole-
collection typed scroll must reject duplicate, extra, missing, or payload-
mismatched points before terminal success. Qdrant writes outside the CAS may
leave orphan namespaces after races or crashes, but those namespaces own no
readiness.

The sealed acquisition/failure foundation is now implemented but intentionally
unwired. It authenticates current Body authority, verifies bytes, excludes
empty/oversize/non-UTF-8 files, derives named UTF-8-safe line chunks, and CAS-
installs immutable obligation/plan with a running claim/attempt in the Body
store. Exact failure may terminalize after Body advance; no code can publish a
coverage receipt/head. Next work must add observed-binding Qdrant execution,
terminal-success CAS, and abandoned-claim recovery before choosing either a
dedicated coverage service (preferred for ownership clarity) or a strictly
separated lane in the reserved memory semantic-projector process. No CLI shim.

## Exact coverage execution port (2026-07-15)

The crate-private projector now executes a sealed plan through the shared typed
Qdrant boundary. It validates text hashes and vector dimensions, authenticates
exact collection metadata, skips empty upserts, observes the whole typed
collection, rejects cyclic page offsets and duplicate/extra/missing/substituted
point payloads, and publishes receipt/head only through an exact terminal CAS
over Body authority, obligation, plan, claim, attempt, and the prior coverage
head seen at acquisition. Body advance and stale-head competitors cannot mint
success; ordinary failures terminalize the exact running attempt.

This is still sealed foundation, not operational coverage. The dedicated
workspace-coverage service is now the chosen owner, but claims first need an
authenticated executor incarnation/startup receipt. Its pulse also needs an
exact already-current classification and sealed chunk-text rematerialization.
Recovery must consume a newer latest reserved-service launch plus its correlated
ready heartbeat and fence the old claim in one Body-store CAS. Timeouts, generic
managed policy, Qdrant contents, and process guesses are forbidden recovery
authority. If Body advanced, terminalize the obsolete claim and acquire a fresh
plan; never resurrect it.

## Dedicated workspace coverage service proof (2026-07-16)

The dedicated service, binary, reserved supervisor policy, and specialized
CultMesh launch contract now exist. Generic policy and caller-selected command,
workspace, Body store, collection, or dimensions are rejected. Launch receipt,
PID, executable hash, runtime identity, immutable Body route, and host singleton
are authenticated before projection.

The service resolves the configured Ollama tag to its installed immutable
artifact digest on every pulse, probes its dimensions, and uses an exact Current
fast path that reads no repository files. Needed projection uses one
authenticated Body read session, verifies source bytes/chunks, embeds only after
claim acquisition, whole-scrolls Qdrant payloads plus vectors, and binds exact
point/payload/vector observations into the receipt before terminal CAS. Body
history owns retirement of terminal non-current claim collections; deletion is
metadata-gated and idempotent. Raw backend/filesystem errors no longer leak into
the operator-safe pulse projection.

Proof: 424 library tests passed, one ignored; core and OpenAI runtime all-target
checks passed. The ignored live smoke then passed against local Qdrant 1.17.1
and Ollama `qwen3-embedding:0.6b` (artifact digest
`ac6da0dfba84a81fdbfbaf330198c33cd77c4cdfc53e8bc50eb581914a15621d`,
1024 dimensions), proved exact vector observation and Current classification,
deleted its GUID-scoped collection, verified no coverage collection remained,
and restored the preflight-stopped Qdrant container state.

Recovery remains absent on purpose. A newer launch plus ready heartbeat is not
death evidence. Next build: strengthen native process observation with host+
boot and process-instance identity, then let the supervisor publish one
immutable termination observation bound to the exact old launch/PID/executable/
policy/heartbeat/provider. Only that proof plus a current replacement launch may
authorize one Body-store recovery CAS. Do not add timeout recovery.

The native process-instance probe foundation is now implemented and locally
proved. Windows uses PID + process-creation FILETIME + canonical executable,
compares exact identity through a held query/synchronize handle before
alive/exited classification, and separates Toolhelp absence, access denial, and
indeterminate query failure. Linux uses boot id + proc starttime + executable
and recognizes exact zombie exit; other Unix targets refuse to claim Linux
evidence. The old PID-only projection has no `Dead` state. This substrate does
not itself authorize recovery. Next build the enrolled OS-host identity and
specialized reserved launch/heartbeat and termination documents; only then wire
the Body recovery CAS.

The enrolled host-incarnation foundation is also implemented. Its single
non-workspace CultCache record owns an Ed25519 public identity and protected
private seed; opening validates the exact type/key/schema, platform binding,
public/private match, and immutable singleton shape. Enrollment refuses any
existing store and never regenerates malformed state. Windows uses CurrentUser
DPAPI with UI disabled and the deliberately limited assurance label
`os_user_installation_bound_best_effort`; Linux uses 0700/0600 state plus
machine-id binding and declares its cloneable baseline. No code calls this
physical-machine identity. Next remove reserved coverage authority from generic
lifecycle/heartbeat and replace it with identity-bound signed documents.

Those specialized documents are now implemented but not yet wired. The launch
is a mandatory host-signed CultMesh record over exact current reserved policy,
host record digest, boot, PID incarnation, executable path/digest, provider
incarnation, and ephemeral provider public key. The provider heartbeat is
ephemeral-key signed, binds the exact launch envelope and same tuple, and owns a
strict per-launch sequence/time CAS. Validation rechecks the current specialized
policy but does not reread the historical executable path, so deletion cannot
erase evidence. Focused hostile tests reject forged signatures, sequence gaps,
identity collisions, wrong supervisor/status/runtime, and tuple drift.

Next migrate the supervisor and projector together: fixed-size provider seed
frame over reserved-child stdin, exact process capture, signed launch persist,
child launch authentication, specialized heartbeat publication, then generic
writer rejection for reserved coverage. Do not let the new and old documents
remain co-owners after that cut.

The migration cut is now implemented and verified. Reserved-child stdin carries
one fixed binary launch-id/seed frame and requires EOF; all nonreserved service
stdin is null. Supervisor captures the spawned process incarnation, sends the
secret, host-signs/CAS-persists the specialized launch, and kills/waits on any
failure. The projector authenticates host, current policy, boot, exact process,
launch digest, and derived provider key before service construction, then emits
only specialized signed heartbeats. Generic lifecycle and daemon-heartbeat
writers reject the reserved coverage identities, and the old generic coverage
authenticator is deleted. Coverage claim/attempt v1 now binds
`managed_process_launch_id`, refusing semantic reinterpretation of v0 state.

The new native `epiphany-host-identity enroll|status` actuator explicitly
enrolled this Windows installation at the default LocalAppData DPAPI store;
status reopened identity
`17f041421045aa66d4c8ab0488f462dbb5ea1a7d8507dbf75216f5ec368dbb7a`.
Next implement immutable Idunn termination observation before replacement spawn,
then the exact Body recovery CAS. Timeout recovery remains forbidden.

Immutable termination evidence is now implemented. The only public writer uses
native boot/process observation; the injectable source is private to tests. A
host-signed per-launch record binds the exact current policy envelope, exact
specialized launch, exact per-launch latest signed heartbeat, enrolled host,
expected boot/PID generation/path, observed outcome, and optional exact exit or
replacement material. Publication exact-CASes those three source envelopes and
an absent immutable termination key. There is no latest termination pointer.
Alive, inaccessible, indeterminate, unknown boot, host mismatch, collision, or
moved source state refuses. Authentication reconstructs the complete persisted
chain instead of blessing the signed blob by itself.

Next wire Idunn ordering and Body recovery: predecessor termination must persist
before replacement spawn; replacement signed ready must exist before one atomic
v1 claim/attempt terminalize-and-reacquire CAS. Add negative ordering and stale-
Body tests before calling recovery operational.

The deployment next action is unchanged and permission-bound. Do not reboot
without exact live operator approval. With that approval, run the real
reboot/logon recovery proof already specified above.

## Workspace coverage exact Body recovery and Idunn reconciliation (2026-07-16)

The abandoned-claim path is typed end to end. Recovery refuses caller-owned
Body paths, reopens the runtime's authenticated Body route/basis, joins current
Body, obligation, plan, claim, and attempt, authenticates the old launch plus
immutable host-signed termination, and accepts only a causally linked
replacement launch whose current signed heartbeat is `ready`. One exact Body
CAS archives the failed owner, advances the fencing epoch, installs the
successor claim/attempt, and writes an immutable recovery receipt binding the
old claim, termination envelope, replacement launch envelope, ready heartbeat
envelope, and successor authority. Restart authentication reconstructs both
CultMesh evidence and current Body authority.

Reserved launch schema v1 adds a signed causal replacement edge. Its writer
exact-CASes a singleton `replacement-for/<old-launch>` slot with exact
termination and launch state. A second replacement refuses; the supervisor
kills a spawned loser when persistence loses CAS.

Idunn branches workspace coverage away from generic lifecycle PID receipts. It
persists or reuses termination, reuses a causal replacement after interruption,
otherwise launches through the fixed stdin/bootstrap path, waits for the exact
latest signed ready heartbeat, then invokes Body recovery. Initial/pre-claim
launches use the same specialized authority.

Proof passed: 5 process-document/recovery tests, 14 supervisor tests, 15
projector tests plus 1 ignored live Qdrant/Ollama test, and all-target check.

Next cut: termination v0 still requires a provider heartbeat. A child dying
after launch persistence but before heartbeat sequence one strands the chain.
Rebuild termination so signed launch plus enrolled host/boot/process observation
is sufficient, with heartbeat optional additional evidence. Then run a
GUID-scoped live initial launch -> death -> replacement -> Body recovery smoke.
Reboot/logon proof remains permission-bound.

## Pre-readiness workspace coverage death is recoverable (2026-07-16)

The heartbeat prerequisite was rebuilt, not relaxed with nullable fields alone.
Each specialized launch owns one typed process-evidence head. Launch initializes
it; every signed heartbeat advances it by exact CAS; termination advances and
seals it by exact CAS. Termination v1 uses signed launch plus enrolled
host/boot/process observation as sufficient death proof and includes heartbeat
id/digest only when the head names a current signed heartbeat. Heartbeat after
termination refuses, so publication and death cannot win on separate keys.

Focused proof now includes death before heartbeat sequence one, successful
heartbeat-free termination authentication, and refusal of a late heartbeat.
All six process-document/recovery tests, fourteen supervisor tests, fifteen
projector tests plus one ignored live test, and all-target check pass.

Next: create/run a GUID-scoped live smoke through real packaged supervisor and
projector binaries, local Qdrant/Ollama, exact child kill, causal replacement,
epoch+1 recovery, and restart-time receipt authentication. Restore the prior
Qdrant container state afterward. Reboot/logon remains permission-bound.

## Live packaged workspace coverage recovery proof (2026-07-16)

Feature-gated binary `epiphany-workspace-coverage-recovery-smoke` constructs
GUID-scoped repo/runtime/Body/Verse state, uses the real packaged supervisor and
projector siblings, warms local Ollama, derives its exact fenced Qdrant collection names, launches
through specialized bootstrap, catches the epoch 1 claim, kills the exact child,
runs Idunn reconciliation, verifies epoch 2 authority and fresh-store-reopen
recovery receipt authentication, kills the replacement, and removes only its
exact owned collections. Process cleanup revalidates the signed launch's native
incarnation before issuing any kill.

Passed proof:
`C:\Users\Meta\AppData\Local\Temp\epiphany-workspace-coverage-a577f820-b0e2-4733-b628-529ee3bdb143\proof.json`

Receipt: `2b44f28f-7c0f-4ad2-aefe-d7c8222e8642`. The proof binds executable
SHA-256 `d262b7f49d45f15c31589bdfb591988cabcfa9ba65564d3e61493dc00b072093`,
the exact embedded smoke-source digest, source head, and dirty tracked-source
diff digest. It verifies both owned collections absent before emitting success.
Operator orchestration restored the
local Qdrant container to its prior stopped state; that is not a harness claim.

The live rite exposed two real defects and did not pass until both were cut.
Windows boot identity no longer opens privileged PID 4; it queries native
kernel boot time and a focused test proves availability/stability under the
current normal user. Recovery no longer deadlocks its successor: the exact v3
replacement incarnation may resume its own running claim, while a wrong
incarnation remains contended. The harness uses file-backed supervisor output
because Windows grandchildren can retain captured pipe handles, and it has
exact-incarnation process cleanup and exact-owned-collection cleanup guards.

Verification: five native process-observation tests, six process/recovery tests,
fourteen supervisor tests, fifteen projector tests plus one ignored live test,
and feature-enabled all-target check pass. The separate live Qdrant/Ollama
projection test passed before the recovery smoke.

Next: audit deployability against the swarm readiness plan and live host state.
Do not conflate code-complete physiology with the permission-bound reboot/logon
proof; no reboot without explicit live operator approval.

## Fresh deployability audit (2026-07-16)

The host is operational after current-user login, not yet deployable as one
proven artifact generation. `GameCult-Yggdrasil-Tunnel` and
`Epiphany-Idunn-Managed-Service-Reconciler` are enabled/running; the Yggdrasil
Qdrant tunnel and Ollama endpoint are live; packaged Mind and Modeling queries
return semantic ranking. Local retired Qdrant remains stopped. This is real
current-boot physiology.

Four non-permission wounds remain. Installed supervisor/projector binaries
predate recovery commit `34802407`; Task Scheduler pins one mutable supervisor
path while Idunn discovers siblings by filename/existence, so mixed generations
are possible; Mind has no race-bounded whole-repository readiness join; and the
old readiness plan described already-landed June work as future work. The plan
now contains the live evidence matrix. Packaging must become an immutable typed
commit-addressed release witness consumed by Task Scheduler and Idunn. Mind
readiness must join fresh Body R1/R2, exact Body-grounded RepoModel admission,
live Modeling semantic eligibility, and live workspace-coverage Qdrant evidence.
Stored coverage `Current` is not sufficient.

The constant `queryAdmission=false` and supervisor fields naming an external
readiness authority are removed from sight-only projector/health/status
projections. Those surfaces never evaluated query admission and therefore have
no opinion. Query-time reauthentication remains the semantic admission owner.

After the witnessed release is deployed, prove exact current-boot recurrence
and Mind/Modeling query ranking. A reboot/logon proof remains the sole
permission-bound host gate; do not reboot without explicit live approval.

## First witnessed deployment (2026-07-16)

The release packager failed closed three times before publication and exposed
three substrate wounds: Git rejected verbatim/deep Windows worktree paths,
Cargo could not traverse the remaining deep vendor path, and the exact commit
did not contain `epiphany-core/Cargo.lock`. The final machine uses normalized
canonical package paths, short temporary exact-source/build roots, initialized
pinned submodule bodies, and a tracked lockfile with `--locked`. It builds from
a detached worktree at the witnessed commit; caller-supplied binaries have no
entry path.

Published release:

- source commit: `dd80f4a51c18425a6665710698a357be61154abf`
- release id: `sha256-c94109f94ee42c1257089830f399cb87cd5ad672772c274194371995ce4df923`
- witness digest: `sha256-f0394590e2e9762b7cb1579f6a06db009f4701730fe1b44ea0f20c1175aee04a`
- package root: `E:\Projects\EpiphanyAgent\.epiphany-run\releases\dd80f4a51c18425a6665710698a357be61154abf\sha256-c94109f94ee42c1257089830f399cb87cd5ad672772c274194371995ce4df923`

The typed semantic-projector policy now names the witnessed role path. Task
Scheduler pins the witnessed supervisor plus release id/digest. Reconciler PID
`21228` launched direct-child projector PID `22736`; both witnessed Mind and
Modeling query gates returned semantic ranking. After exact-path verification,
PID `22736` was killed once; Idunn launched direct child PID `19748` with new
launch receipt `86a42d51-9930-4b5d-9e61-e74c82a0d808`, ready heartbeat
`4fc91977-b635-474b-812b-c364847e2b4b`, and executable SHA-256
`1060dd64e8c2b520ec8744b67e8306db04a6618d463fc2117c352adf9e30e6b0`,
which equals the witnessed file. Task result `0x800710E0` remains the expected
IgnoreNew recurrence refusal while the foreground task is running.

The installed sibling-generation gate is closed for the current boot. Next:
build Mind's whole-repository observed-readiness join. Reboot/logon remains
permission-bound.

## Semantic receipt-v2 live migration wound (2026-07-16)

The first readiness release exposed an honest migration failure. Receipt v2
correctly made the old v1 Mind/Modeling receipts ineligible, but pulse
classification returned `Succeeded` from claim status before deriving receipt
health. Both partitions therefore stayed idle and packaged queries fell back to
canonical BM25. `Succeeded` is removed as a pulse state. Valid v2 terminal
evidence derives `Ready`; a same-obligation succeeded predecessor with missing,
legacy, or invalid receipt evidence derives `Repair`.

Repair is not opened by claim shape alone. Classification and acquisition share
one predicate that authenticates the consumed executor grant or recovery
authorization plus the exact succeeded attempt/claim binding. The repair grant
advances the epoch. A failed repair becomes a failed current claim and retries
through the ordinary execute path at the next epoch. Tests cover forged
authority refusal, v1 repair acquisition, failed repair, execute retry, v2
terminal success, final Ready, and empty-v2 Ready/nonqueryable stability. Soul
approved; the full library suite is 466 passed, 2 intentionally ignored.

Operational evidence before the fix: release
`sha256-6f9ab4848207a110e7e3836f41e6a74c0f463369ad4b04ce2d0d3831a6b262ae`
from commit `360dcb43` launched witnessed Idunn PID `29476` and direct child
projector PID `22840`, but both packaged semantic queries fell back because the
v1 receipts had no migration path. The next action is to commit this repair,
package/deploy a successor witnessed release, observe v2 semantic ranking, then
derive the first live whole-repository readiness projection. Reboot/logon still
requires explicit live operator approval.

## Mind whole-repository readiness join (2026-07-16)

The missing join now exists in source. `repository_readiness.rs` is Mind's sole
owner for a historical `gamecult.epiphany.repository_readiness_projection.v0`.
It derives every authority decision from raw snapshots and observations: fresh
Body R1, one canonical RepoModel, exactly one current v5 admission whose
historical Body basis is authenticated, the exact succeeded Modeling semantic
chain, live semantic vectors, live workspace coverage bound to R1, fresh Body
R2 with equal content identity, repeated live observations, and one complete
closing runtime-store snapshot. The final projection append uses CultCache's
new full-snapshot conditional append, so a concurrent unknown receipt or
obligation defeats the write. Stored readiness is historical evidence only and
cannot admit a later query or readiness answer.

Semantic receipts are now v2 and seal the deterministic post-scroll vector
binding root. Old, malformed, substituted, or stale receipts cannot grant query
eligibility. Exact empty projections may terminal-succeed once while remaining
non-queryable, avoiding a retry wound. Both semantic and workspace live readers
revalidate their persisted authority after Qdrant observation.

CultCache envelope identity is now the tuple `(type, key)`, not delimiter
concatenation, and `append_if_snapshot_unchanged` compares the complete
canonical snapshot beneath the single-file exclusive lock. Soul approved the
join after hostile passes for alien keys, duplicate grounded/ungrounded
admissions, forged raw semantic heads, receipt cross-field substitution,
historical Body failure, Body/authority drift, R1-bound coverage ordering, CAS
refusal, and exact concurrent idempotence.

Verification: 21/21 CultCache tests and 465/465 Epiphany library tests passed;
two existing live tests remain intentionally ignored. Next: commit/push the
CultCache submodule and parent changes, package a new witnessed release, deploy
it, then derive the first live whole-repository projection. Reboot/logon remains
permission-bound and requires explicit live operator approval.

## Yggdrasil live-deployment authority map (2026-07-16)

Live Epiphany deployment proof belongs on new Yggdrasil, not Starfire. The live
host already runs Idunn (`idunn-yggdrasil.service`), Bifrost, Odin/Hermodr,
Qdrant, and GPU-backed Ollama locally. Epiphany is not yet deployable there:
there is no Epiphany Idunn target, ops manifest/unit, immutable release body, or
daemon-owned aggregate `idunn.daemon_health.v1` publication.

Two ownership gaps must close before the target is enabled. Epiphany's semantic
projection health is component sight and cannot stand in for whole-runtime
health; the managed-service supervisor must join the authenticated packaged
release with current semantic and workspace-coverage child lineage and publish
one explicit CultNet/RUDP health contract. Separately, Bifrost's real GitHub
crossing receipts are not consumed by Idunn: Idunn currently observes
`origin/main` as desired revision without proving that Bifrost authorized that
exact commit. The new chain is Bifrost exact repository/ref/SHA authority ->
Idunn frozen deployment request/artifact -> guarded ops actuator -> immutable
release and deployed manifest -> daemon-published runtime health. Branch
movement, actuator environment variables, process liveness, and component
health are forbidden substitutes.

The intended Ygg body uses `/srv/build/Epiphany` as Idunn's mutable upstream
clone, immutable application releases below `/srv/epiphany/app/releases`, an
immutable deployed source Body below `/srv/epiphany/source/releases`, canonical
state below `/var/lib/gamecult/epiphany`, local Qdrant at `127.0.0.1:6333`, and
local Ollama at `127.0.0.1:11434`. Actual publication must cross live Bifrost;
actual deployment must be requested through live Idunn. Manual SSH deployment
is not acceptable evidence. No host reboot is authorized.

Source implementation now closes the Epiphany-owned half. The managed-service
supervisor publishes exact `idunn.daemon_health.v1` over CultNet/RUDP only
after joining the authenticated packaged release with the exact two reserved
projectors. Semantic and workspace lineage require current policy, the exact
packaged executable, launch-time process incarnation, correlated ready
heartbeat, and bounded freshness. Proven stale v2 lineage may be replaced only
through its persisted process identity; authentication uncertainty cannot kill.
Legacy v1 lifecycle receipts are read-only and enter a typed non-killing
retirement path before a v2 launch. Full library verification is 475 passed,
2 intentional live-endpoint ignores.

The adjacent source bodies are also prepared. Bifrost main commit `3bd250f`
publishes `bifrost.repository_release_authority.v1`, binds one canonical
repository/ref/SHA to a completed crossing receipt and exact GitHub proof, and
forbids simultaneous live authorities for the same ref. Odin's Ygg worktree
adds a scoped `yggdrasil-epiphany` target, selects the unique live Bifrost
authority rather than branch head, freezes it through deployment request and
artifact state, and revalidates at the privileged actuator boundary. The ops
body builds exact initialized source through a separate builder identity,
root-seals immutable source/application releases, starts Epiphany against
Ygg-local Qdrant and Ollama, requires post-candidate Idunn RUDP acceptance, and
only then publishes the v2 deployment witness. These source bodies are not yet
live on Ygg; bootstrap and deployment remain the next phase.

## Yggdrasil GPU pressure exposed missing progress authority (2026-07-17)

The live authority chain now reaches Yggdrasil. Bifrost authorized exact
Epiphany main commit `586147a751c6eec2d59cd8dc10bd17ce0d02a4d1`; Idunn fetched that exact
upstream commit, built it in the pinned Rust container, installed immutable
release `sha256-a5a43af6d2502377d2078dcb42ab98088f4a812487217436ec1af0ddd0d26b3d`,
and started both local projectors against Ygg-local Qdrant and GPU-backed
Ollama. Mind completed its first pulse, reported `providerStatus=ready`, and
published 215 vectors. Workspace coverage is a real 3,956-file Body pressure
run; a local line-based estimate is 14,734 UTF-8 chunks.

The GTX 1080 is genuinely used: the workspace projector sustains roughly
93-100% compute utilization at 2,419 MiB VRAM. The prior 32-text HTTP batches
could monopolize one request for more than thirty minutes. Commit `586147a7`
bounds batches at four; live Ollama receipts now complete continuously in
roughly 0.7-1.7 seconds per call at about 53-63 calls/minute. This proves useful
forward GPU work, not merely allocation. It also proves the full first pass is
roughly an hour at the current hardware ceiling, longer than Idunn's 50-minute
deployment-health wall clock.

Do not fix that by increasing the wall clock. The current execution primitive
embeds the entire sealed plan into memory, then performs Qdrant publication and
verification, and only returns to the binary afterward. The binary therefore
emits no signed provider heartbeat during the long pulse. Ollama logs,
`nvidia-smi`, process existence, elapsed time, and Qdrant point counts cannot
become progress authority.

Authority map for the repair:

- Owner: the authenticated workspace-coverage projector owns domain progress;
  Idunn owns deployment admission, and the supervisor only derives aggregate
  health.
- Inputs: exact packaged release/policy/managed launch, provider incarnation,
  claim and attempt epoch, sealed plan, Body observation/generation, immutable
  embedding artifact/dimensions, and acknowledged durable Qdrant checkpoint.
- Outputs: an append-only provider-signed
  `epiphany.workspace_coverage.projection_progress.v0` event plus CAS latest
  pointer. It binds sequence/time, exact authority identities, phase,
  completed/total units, and the currently bounded backend operation.
- Derived state: `warming` requires a fresh independent process heartbeat and
  fresh monotonically advancing exact-lineage progress. `stalled` follows an
  expired named operation/no-advance deadline. `active` still requires the
  canonical receipt/head and live verification; progress at 100% is not
  readiness.
- Forbidden writers: Idunn, deploy scripts, supervisor, Ollama logs, GPU
  telemetry, stdout, Qdrant counts, running claims, and free-running heartbeat
  churn cannot advance progress.
- Shared path: first projection, ordinary Body change, retry, and authenticated
  successor recovery use the same durable batch/checkpoint primitive.
- Cut line: split monolithic embed/upsert/verify execution at deterministic
  durable boundaries; keep liveness heartbeat independent; remove total wall
  duration as the health verdict. A successor launch cannot inherit work
  without an explicit authenticated resume/checkpoint contract.

Negative proof must cover frozen progress with live heartbeats, stale heartbeat
with advancing progress, replay across launch/Body/plan/model/release, sequence
or count regression, mutable totals, progress without acknowledged Qdrant
durability, 100% without a valid receipt, deleted/substituted collections,
restart inheritance without recovery authority, concurrent owners, clock
hostility, and Idunn accidentally treating warming as active.

The current live run remains diagnostic evidence only until its terminal
workspace receipt, exact aggregate RUDP health, and deployment witness land.
No reboot is authorized.

Checkpoint implementation cut: the authoritative batch checkpoint lives in
the repository Body CultCache, not the local Verse. It is admitted by one CAS
against unchanged Body/obligation/plan/running claim/running attempt and a
claim-scoped checkpoint head. Each checkpoint carries a contiguous canonical
plan range plus the exact ordered point, payload-hash, and vector-hash bindings
read back from Qdrant after a waited upsert. The signed Verse progress event
references the checkpoint's exact CultCache envelope digest; neither a random
provider hash nor Qdrant operation text is evidence. Terminal readiness still
requires the existing whole-collection scroll and receipt/head CAS.

Same-claim retry may continue only from a contiguous authenticated checkpoint
chain whose batches still re-observe exactly in Qdrant. Recovery uses a new
claim epoch and new epoch-fenced collection. A successor may authenticate and
copy predecessor checkpoint batches into its own collection after exact death
and recovery proof, then emit new-claim checkpoints citing the source envelope;
it never mutates or inherits the old collection. If live external evidence is
missing or changed, it restarts from zero. Required backend cuts are a single
bounded waited upsert, exact retrieve-by-ID with vectors/payloads, and a common
readback/checkpoint primitive for newly embedded and recovered copied points.

The first bounded-batch pressure run terminated exactly as the model predicted.
At `2026-07-17T01:23:50+02:00`, after 50 minutes of candidate-health waiting,
Idunn stopped Epiphany, left `/srv/epiphany/deploy/deployment.env` absent, and
reported `Idunn did not publish exact-candidate Epiphany daemon-health proof`.
Ollama had completed 3,389 embed HTTP calls since candidate startup. After the
Mind projection calls, that represents roughly 13.3k workspace chunks—about
90% of the estimated 14,734-chunk Body. The GPU remained usefully saturated,
but the monolithic executor had not yet upserted the accumulated workspace
vectors, so Qdrant's inherited Modeling collection remained at 22 points and
the hour of partial work was discarded. This is the live falsification of the
current execution/deployment contract. Do not reinterpret it as an Ollama or
GPU failure and do not repair it by lengthening the total wall timeout.

## Live projector checkpoint integration landed (2026-07-17)

Epiphany main commit `7362e54b` replaces the production monolithic workspace
executor. The live path now processes sealed plan-order batches of at most 128
points, performs a waited Qdrant upsert, reads those exact IDs back with payload
and vector, admits a provider-signed Body checkpoint, and publishes progress
only from that admitted checkpoint. Restart authenticates the complete
genesis-to-head checkpoint chain against current Body, claim, attempt, plan,
launch, and host authority; it re-observes every checkpointed batch before
resuming at the next ordinal.

Terminal receipt/head proof remains separate. It reconstructs independent
payload/vector expectations from the authenticated full checkpoint chain,
requires complete contiguous plan coverage, and rejects cycles, gaps,
overlaps, duplicate checkpoint or Qdrant rows, missing/extra points, and any
payload/vector substitution during the final whole scroll. The previous
circular final-vector comparison was found by Soul and cut before commit.

The provider heartbeat is now an independent signed 10-second nerve and keeps
publishing while projection blocks. Heartbeat publication, sequence, or local
projection-lock failure terminates the whole provider process; a projector may
not continue acting after losing its liveness organ. Pulse refusal remains an
operator projection and no longer contaminates heartbeat status.

Focused proof: 10 checkpoint tests, 17 projector tests, 3 service tests, the
reserved supervisor-argument test, and production checks for both projector and
supervisor binaries pass. The old all-at-once executor survives only under
`cfg(test)` for legacy pre-embedded invariant tests; it is absent from the
deployed body. The next authority cut is Idunn and aggregate health: consume the
exact provider progress document with a no-advance lease, never a generic job
progress noun or total projection wall clock.

Live storage pressure later exposed that the production cadence still admitted
one immutable Redb checkpoint/progress transaction per point despite the
128-point batch contracts. The cadence now aliases the shared checkpoint maximum
of 128 and is compile-time constrained by Qdrant's same 128-point transport
ceiling. Durability still follows waited upsert plus exact payload/vector
readback; only the authenticated commit unit changed. Existing one-point chains
remain valid predecessors for later larger batches when their exact Qdrant
collection survives. A progress store whose collection is absent must not be
resumed from counts alone.

## Organizational Epiphany is the product, not a remote-control experiment (2026-07-18)

The Yggdrasil deployment is the first ordinary organizational Epiphany body.
It must remain awake without continual instruction: Modeling persistently maps
the bounded domain into typed Mind state, Self extrapolates pressure and
direction from that map, and Imagination authors inspectable improvement and
feature proposals. This initiative is advisory until the appropriate adoption
and consequence authorities act; operator inactivity permits observation,
modeling, and proposals, not merge, publication, deployment, or scope growth.

The repository Persona is the organization's shared social crossing. Members
may talk to it, correct it, and provide feedback through ordinary Discord
conversation. Persona owns discussion and legibility. Conversation produces
typed feedback pressure for Modeling, Imagination, and Mind review; it does not
directly mutate canonical Mind, invoke Hands, authorize a release, or command
Idunn. Authenticated operational capabilities such as brake, objective intake,
adoption approval, release authorization, and deployment remain distinct typed
Bifrost routes with explicit receipts. VoidBot observes the room and Bifrost
routes the typed crossing; do not invent a second Discord bot or grant ordinary
chat ambient actuator authority.

The July 22 continuity criterion is stronger than remote command. Yggdrasil
must host the resident scheduler/coordinator, model runtime, tool spine, Persona
crossing, writable domain worktree, persistent Mind/Modeling state, and local
credentials. Starfire may be absent through August 5. The offline rehearsal
must prove conversation/feedback -> persistent domain map -> Imagination
proposal -> explicit adoption/review -> Hands consequence -> Bifrost exact
release authority -> Idunn deployment receipt, plus the negative that
conversation alone cannot cross any later boundary.

Live observation on 2026-07-18 sharpened the deployment boundary. Exact release
`0601e280480cccc28209fbf0630e921f3ef9056d` kept its projector incarnation alive
and advanced the Qdrant workspace collection from 1,621 to 3,278 points after
the foreground Idunn deploy actuator was no longer present. `deployment.env`
remained absent and Idunn correctly raised `dependency-unavailable` because it
could not establish release freshness. A surviving candidate and advancing
checkpoint are resumable work evidence, not deployment admission. After the
provider reaches terminal projection, the same exact authorized release must
re-enter Idunn's deployment transaction and earn the typed deployment witness;
no observer, systemd state, Qdrant count, or journal line may synthesize it.

## Resident organizational Self ownership (2026-07-18)

The resident cognition cut now has a concrete authority boundary. Standard
heartbeat alone converts one pending typed pressure into one single-consumption
Self scheduling grant. Resident Self alone turns that grant into one exact
coordinator process lease. The coordinator, model runtime, and tool spine keep
their existing authorities; the resident scheduler does not absorb them.

Inputs are an authenticated packaged-release witness, physically separate
resident/runtime/Verse/Mind stores, the swarm brake, exact process observation,
and typed operator-objective, Body-map-drift, Persona-feedback, or
Imagination-proposal pressure. Outputs are immutable pressure/grant,
preparation, child-claim, active-lease, coordinator-receipt binding, and
heartbeat terminal-ack documents. Active status and cooldown are derived from
that chain. Process exit, logs, Persona speech, proposal text, systemd state,
Eve/Gjallar projection, and Qdrant progress are forbidden writers.

All wake sources share one path: pressure -> heartbeat grant -> prepared launch
-> packaged coordinator child claim before cognition -> exact process lease ->
bound coordinator terminal receipt -> terminal acknowledgement. Preparation
and grant consumption use one CAS; brake and timeout drain the exact observed
process; coordinator exit zero without its exact receipt fails. The old
source-tree/Cargo swarm wrapper and its `online`, `run`, `run-queue`, `pulse`,
and `epiphany-work` queue authority are outside the live body. Packaging now
requires witnessed sibling roles for swarm, coordinator, model runtime, and the
Codex MCP tool spine.

Verification must remain at these typed transitions: CAS/replay, store
separation, release witness, child-claim-before-cognition, exact process
identity, brake/timeout timelines, receipt substitution, and absence of runtime
Cargo/queue paths. One nonblocking hardening note remains: the parent currently
correlates its post-launch process observation with the child's atomic claim.
That is stronger than PID custody and fails closed, but it should eventually be
replaced by an OS-authenticated parent/child launch channel where available.

This is cognition bootstrap, not deployment admission. Ygg release
`0601e280480cccc28209fbf0630e921f3ef9056d` remains a live advancing candidate
until Idunn publishes the exact typed deployment witness; resident service
installation, writable domain custody, credentials, Persona/Discord ingress,
and the Starfire-offline rehearsal remain required.

## Resident physiology and Ygg service body prepared locally (2026-07-18)

Heartbeat `serve` is now the single resident physiology loop: it reconciles an
exact terminal acknowledgement first, retains pressure while braked, emits at
most one grant while clear, never replaces an active coordinator, and performs
bounded void/sleep work only while idle. Restart resumes the typed chain rather
than guessing from process or journal sight. Heartbeat is a required witnessed
release sibling; coordinator and resident Self require the exact authenticated
runtime and an explicit provider.

`epiphany-swarm status` is non-actuating sight. Its typed resident-readiness
document binds runtime id, release id, release witness, source commit, packaged
binaries, physical store separation, heartbeat/Self freshness and coherence,
active lease sight, runtime-scoped brake, writable workspace, and credential
posture. The existing daemon supervisor remains the sole signed Idunn health
writer and may aggregate the exact heartbeat/Self provider pair. Readiness,
systemd, and PID sight cannot admit deployment.

The matching `gamecult-ops` cut defines three narrow services:
`epiphany.service` owns supervisor/projector execution and signed aggregate
health; `epiphany-heartbeat.service` owns heartbeat state and grants; and
`epiphany-swarm.service` owns resident-Self state and bounded coordinator
launches. Coordinator, model runtime, and tool spine remain witnessed child
siblings. Yggdrasil's writable organizational Body is
`/var/lib/gamecult/epiphany/workspace`; immutable
`/srv/epiphany/source/current` is release provenance. Bootstrap prepares
protected stores and service-user credentials. Deployment authenticates the
sibling set, initializes heartbeat, manages all three units transactionally,
preserves coverage checkpoints across retries, waits for typed readiness, and
lets Idunn write `deployment.env` only after admission.

This is ordinary organizational Epiphany. Persistent Modeling maps the bounded
domain; Self derives bounded attention pressure from map drift; Imagination
authors inspectable improvements and feature proposals; the organization talks
to the repository Persona through existing VoidBot/Bifrost Discord ingress.
Feedback and proposals may request cognition. They cannot adopt Mind state,
invoke Hands, authorize Bifrost release, or command Idunn. Those are separate
typed crossings and receipts.

Live source `0601e280480cccc28209fbf0630e921f3ef9056d`, release
`sha256-f9fbc9c17cfc4038c49f77dd2a238d487b723600127a982dc4ae2ab98b308a77`,
and projector PID `145165` remain the sole first-projection owner. Its Qdrant
collection advanced beyond 6,585 points. `deployment.env` and the foreground
Idunn actuator remain absent, so this is resumable provider evidence, not
admission. Do not interrupt it. After terminal proof, re-enter the exact release
through Idunn, then authorize and deploy the coordinated resident generation.
Before July 22, run Starfire-offline proof of feedback -> persistent map ->
bounded Imagination proposal -> explicit adoption/review -> Hands -> Bifrost ->
Idunn, including the negative that conversation alone crosses no later gate.

The immediate live wound precedes that work. The workspace projector is the
lifetime owner of `workspace-coverage.cc`, but the supervisor currently opens
that private store to discover and perform recovery. Replace that transitive
authority leak with a provider-signed claim sight published immediately after
acquisition and a host-signed recovery directive consumed by the replacement
projector. Heartbeat remains liveness only. The supervisor must never open or
mutate the coverage store; it declares recovery only after authenticating the
replacement claim sight. Keep Epiphany and Idunn stopped on Yggdrasil until the
corrected exact release is authorized through Bifrost and deployed through
Idunn. No host reboot is authorized.

## Workspace-coverage recovery ownership cut complete locally (2026-07-18)

The supervisor no longer opens or mutates `workspace-coverage.cc`, directly or
through an imported recovery helper. The lifetime projector owner publishes a
provider-signed claim sight before its first backend operation. Claim sight is
keyed to the exact current Body observation and generation and binds the exact
launch, provider key, coverage-store binding/file/route, Body basis, claim,
attempt envelope, plan, and recovery lineage. A signed sight from an older Body
is not current recovery authority.

After exact termination and replacement readiness, the supervisor publishes
one immutable host-signed recovery directive. It binds the authenticated old
claim sight, termination, replacement launch, readiness heartbeat, Body, and
coverage route by exact envelope digests. Reconciliation reauthenticates and
returns the same signed directive byte-for-byte; a later heartbeat cannot
rewrite it, and conflicting lineage is refused. Provider heartbeats now retain
immutable per-heartbeat evidence beside the latest pointer because durable
termination and recovery documents may cite an earlier heartbeat. Causally safe
bounded retirement remains future work; naive deletion is forbidden.

The replacement projector authenticates the directive and performs the
recovery CAS through its already-held `WorkspaceCoverageAuthority`. Recovery is
idempotent across the CAS-to-successor-sight crash window: an exact committed
successor and receipt are returned on replay, then ordinary acquisition
republishes successor claim sight. A directive naming readiness heartbeat H
remains valid when the same launch/provider has advanced normally to ready
H+1. Heartbeat remains liveness evidence and does not own the lease.

Focused proof passes: eight process-document tests, the exclusive lifetime
owner test, supervisor authority-order/prohibition test, pre-checkpoint sight,
stale-Body refusal, H/H+1 consumption, immutable directive reissue, conflicting
lineage refusal, exact CAS replay, successor sight publication, `cargo check
--bins`, and `git diff --check`. Soul found no remaining P0/P1/P2. This code is
ready to commit and push, then cross Bifrost/Idunn as an exact release while the
host remains braked.

## Live deployment and reconciliation split follow-up (2026-07-18)

Commit `0601e280480cccc28209fbf0630e921f3ef9056d` is pushed, is the sole current
Bifrost release authority, and is running through Idunn on Yggdrasil as release
`sha256-f9fbc9c17cfc4038c49f77dd2a238d487b723600127a982dc4ae2ab98b308a77`.
The detached deployment request remains in progress and `deployment.env` is
absent, so terminal admission is not yet proven. Projector PID `145165` has
remained stable; `body.cc` remained 612,961 bytes; the dedicated coverage store
remained 79,712,256 bytes while its typed state advanced; and the current Qdrant
collection advanced from 193 to 1,621 points. GPU samples show the embedding
model resident and useful intermittent load. Do not interrupt this sole owner
merely because the first full projection is slow.

Live stdout exposed a remaining conceptual substitution: every healthy claim
was routed through the termination writer and projected as
`awaiting-exact-termination`. The writer refused ExactAlive, so private state
remained safe, but alive, inaccessible, and indeterminate sight were collapsed
into dishonest recovery pressure. The local uncommitted correction gives the
supervisor one typed non-mutating process observation before recovery:
ExactAlive reports `observed-alive`; uncertain sight reports
`observation-degraded`; authority drift reports `observation-refused`; only
proven terminal sight reaches termination, replacement, readiness, and
directive.

The cut also seals the restart timeline. Claim sight is re-read after immutable
termination to close the pre-first-claim race. A persisted replacement without
a directive is recognized only through exact old-launch, termination, policy,
and causal replacement bindings. The supervisor observes that exact replacement
alive, selects a fresh signed ready heartbeat under the existing 180-second
lease, then re-observes the process immediately at the directive actuation
boundary. Terminal or uncertain replacement sight produces no directive,
termination write, or second launch. Recovery resumption, newly launched
replacement, and immutable directive replay share the same final gate.

Focused proof passes: ten workspace process-document tests, eight supervisor
workspace-reconciliation tests, all-bin check, diff check, and final Soul audit
with no P0/P1/P2. Behavioral injection proves ExactAlive-to-Missing across the
readiness window performs zero directive/termination/launch writes, while
ExactAlive-to-ExactAlive performs exactly one directive write. Commit and push
this cut next; do not replace the live release until its current projection has
landed terminal proof or a real failure requires recovery.

## Current organizational Yggdrasil deployment (2026-07-18)

This section supersedes earlier live-PID and release-progress instructions.
Release `0601e280480cccc28209fbf0630e921f3ef9056d`, projector PID `145165`, and
their Qdrant counts are historical evidence only. The old deployment transaction
was terminated through its rollback boundary, its release authority was revoked,
and its missing final witness must not be reinterpreted as admission.

The ordinary organizational feedback path is live on Yggdrasil. VoidBot runs as
its host service identity with portable observer-group access and writes atomic
typed observations through the exact Idunn-admitted CultLib runtime. Bifrost is
healthy, bound to six organization channels, and publishes daemon-owned RUDP
health accepted by Idunn. A labeled bound fixture became an immutable admission
receipt with `grantsWorkAuthority=false` and `grantsAdoptionAuthority=false`; an
unbound fixture was refused. Conversation remains feedback pressure only.

Idunn and Bifrost deployment authority were repaired from the coherent Odin
schema branch. Idunn now kills timed-out Unix command process groups, treats an
absent deployment witness as first-deployment state while refusing corrupt
witnesses, and rehydrates its preserved typed store. Bifrost's release runtime
uses pinned commits and publishes an exact deployed-revision witness. VoidBot
`7c6f0e9e77ef9d2361c9b7a8368f9fdb0b754458` is deployed and witnessed.

Epiphany `37df884c6411da10196e59b741a538f792deeee5` is authorized through Bifrost
and is being deployed only through Idunn. The canonical writable Body is
`/var/lib/gamecult/epiphany/workspace`; immutable release provenance is not a
substitute. The actuator now writes an engaged typed `scope=all` swarm brake
before starting any Epiphany service. Releasing that brake is a separate
operator action. Until Idunn admits exact signed aggregate health and publishes
`/srv/epiphany/deploy/deployment.env`, the candidate is not deployed.

## Resident organizational product and authority correction (2026-07-18)

Epiphany's resident organizational behavior is the product posture, not a
special dogfood mode. A swarm may continuously map its authorized domain and,
from the current admitted model, ask Imagination for bounded proposals without
waiting for a chat instruction. Organization feedback through the repository
Persona is additional attributed pressure, not the ignition key and never
adoption or action authority.

The old `body-map-drift` pressure was a false measurement: it hashed only the
current RepoModel and compared it to no Body or prior model. It is replaced in
`epiphany-core/src/resident_self.rs` by
`admitted-model-direction-consideration`, bound to the exact current admitted
model revision/hash and unique admission receipt and emitted only while no
terminal direction consideration exists. Imagination remains proposal-only.

Package construction and application-owned state publication are separate;
Bifrost owns release crossing and Idunn owns deployment. Verifier/status reads
consume immutable provider snapshots without acquiring writer locks. Resident
readiness joins separately owned release, runtime/Verse, provider, Mind, and
brake facts and grants no cognition or action authority. An engaged brake still
permits daemon supervision, leases, health, liveness, and read-only sight while
blocking cognition launches, cognitive-work scheduling, Hands consequences,
publication, and deployment actuation.

Exact candidate `9165862cce09739a62b20c0c7728ccceb71cfc2f` crossed Bifrost and
built immutable release
`sha256-39eed3817a10325b39b7f223fb2919c5959c88443ef82d9cc1497f6af631f365`
with witness
`sha256-9ad9cfe17fcabffb84fb392cbd5ecc1e8823482a338b38ced52e1020532e80c6`.
It was not deployed. Startup exposed three faults, so all candidate services
and the Idunn transaction were terminated through rollback. There is no
deployment witness or active-release pointer. The immutable package and stable
compiler cache remain reusable evidence only.

The successor source cuts those faults at their owners. Heartbeat reads only
its typed identities from the mixed heartbeat/readiness store and writes exact
owner entries with CAS, preserving foreign readiness. CultCache now provides a
typed in-memory envelope load primitive for this shared-store view. Cold Self
without thread state emits no direction request yet instead of failing. A
`heartbeat.scheduler` brake blocks cognitive scheduling but does not block
daemon lifecycle physiology; explicit daemon lifecycle surfaces still do.
Focused proofs pass and the full core library reports 570 passed, zero failed,
one ignored.

Candidate `86d702002acee8321438f32fb9cf69a29a667d70` built and sealed immutable
release `sha256-da19a70a7e8c0229cd59b292bf5867fd1e97fa40643ab772e918c5d49dd7ff22`
with witness
`sha256-62c65d813837dd4a939e941d798d0842ee5f6c77cddaa645f2c47a18b7c3ec44`.
It was not deployed. Projection and Heartbeat were healthy and braked, but
resident Self found a second cold-start path: feedback-driven Imagination
consideration required thread state before a thread existed. Idunn was
terminated through rollback; all services are inactive and no deployment or
active-release witness exists. The immutable package is evidence only.

That second path now treats absent thread state as a missing prerequisite: it
emits no request, writes nothing, preserves admitted feedback, and retries on a
later pulse. The causal smoke explicitly requires its fixture thread. Focused
Imagination tests, resident-Self tests, and all-bin compilation pass.

Exact successor `aa3d2330062a34145cb70b527e02fa00215eac1b` was revoked before
deployment because packaging repeated the release-root staging permission
failure with no predecessor alive. This killed the cleanup-race hypothesis.
The real fault was the builder's long-lived write lease over the shared
root-owned release directory.

Packaging is now source-generation scoped. The packager stages beneath the
exact commit directory and atomically renames the final release within that
same directory, so canonical witness paths do not move. Idunn keeps the shared
release root root-owned and grants the builder only that exact commit directory;
cleanup seals it root-owned and removes only its nested staging. Package tests
pass 12/12, all binaries compile, and the actuator passes remote Bash syntax.

Next action: commit and push the generation-scoped Epiphany and ops cuts,
install the exact root-owned actuator, then authorize and cross the new exact
commit through Bifrost and Idunn while braked. Verify the typed brake, exact
release/witness bindings, all three service owners, signed Idunn health, local
Qdrant/Ollama routes, and final deployment witness. Then run the Starfire-offline
rehearsal before 2026-07-22: organization feedback -> persistent Modeling map ->
bounded Imagination proposal -> explicit review/adoption -> Hands consequence ->
Bifrost release -> Idunn deployment, plus replay/tamper refusal and the negative
chat-only path. No host reboot is authorized.

## Generation handoff and resident mixed-store cut (2026-07-18)

Candidate `03e1afd24546a7ca37e900c47ca26ae257b14161` proved generation-scoped
packaging and sealed immutable release
`sha256-fe7f25ff60d84ae9316d7b4aa2833bd5d1b65189c9d08cc64e903a83d8f35268`.
Its first application publication failed because the exact commit parent was
still builder-owned mode 0750; cleanup sealed that parent and an automatic
retry reused the exact package and started all three services. The actuator now
seals the exact generation parent root-owned mode 0755 before handing
publication to the application owner.

The candidate was not deployed. Projection and Heartbeat were healthy and
braked, but resident Self treated the shared resident/readiness CultCache store
as if every row belonged to Self and rejected the legitimate provider-readiness
document. The exact process group was terminated, all services were stopped,
`active-release.env` was removed, and no deployment witness exists. Resident
Self now loads only its six owned typed families into its in-memory view,
rejects duplicate owned identities, and leaves foreign readiness rows intact.
Focused resident proof passes 12/12; full core proof passes 572/572 with one
ignored.

Next action: commit and push both cuts, install the exact ops actuator on
Yggdrasil, revoke obsolete Bifrost authority, authorize the new exact Epiphany
commit, and let Idunn deploy under the engaged brake. Require the immutable
package/witness, all three healthy service owners, signed aggregate health, and
`deployment.env`; then prove the ordinary organization loop and its negative
authority boundaries. No host reboot is authorized.

## Managed-policy rotation preserves historical process evidence (2026-07-19)

Exact candidate `2d4c92d24c2007858914d352b5892f599e6a20e9` sealed release
`sha256-1dd1271405e82958df2c03133b0d89d2555a8a3c7013486e1c41465ec84f6b3c`
and witness
`sha256-e01803545e557488cf2a0a42c31bb65e17ff7ead21790d7c982216901d94e12b`.
The generation handoff and resident mixed-store repairs held: all three services
started, Heartbeat remained braked, and resident Self remained stably braked.
Workspace coverage did not launch because its prior signed launch was checked
against the newly written current managed policy and became impossible to
observe, terminalize, or replace. The exact transaction was rolled back,
Bifrost authority revoked, services stopped, active pointer removed, and no
deployment witness exists.

The source authority is now split correctly. Historical host-signed launch
evidence remains authentic across policy rotation and may prove only exact
process identity and immutable termination. New process admission still
requires the exact current policy digest. Reconciliation terminalizes the stale
identity without deleting history, then launches a causally bound replacement
under the current policy. Public current-process observation continues to reject
stale launches. Focused process tests pass 11/11, supervisor authority tests
pass 24/24, full core passes 573/573 with one ignored, and all binaries compile.

Next action: commit/push the successor, cross its exact SHA through Bifrost and
Idunn under brake, and require historical terminal evidence, current-policy
replacement launch, three stable services, signed aggregate health, and the
final deployment witness before rehearsing Starfire-independent operation.

Candidate `b5405beaa95cddc837f3b7a7ca268ac9b5ae4502` then proved the historical
authenticator shipped but exposed an ordering fault: reconciliation asked
provider claim sight to authenticate before it inspected the supervisor-owned
latest launch. Claim sight correctly rejected the stale policy, so the new
rotation branch was unreachable. The candidate was rolled back, its authority
revoked, services stopped, active pointer removed, and no deployment witness
exists. Reconciliation now loads latest launch and completes stale-policy
rotation before current claim authentication. Provider claim state participates
only after a current-policy launch exists. Supervisor proof passes 24/24 and
full core remains 573/573 with one ignored.

Candidate `b8517d9d7905eb62314db675cc2f0688510aa5bd` then crossed the first two
rotation cuts: supervisor PID `1147968` launched replacement workspace
projector PID `1148157`, and the replacement published advancing ready
heartbeats under the new policy while the brake held. Recovery still stalled
because the old Body claim was readable only through current-claim
authentication, which correctly rejected its historical launch before the
mismatch branch could issue a recovery directive. The candidate was rolled
back and revoked; no deployment witness exists.

Claim sight now has separate current and recovery readers. Current
authentication remains mandatory for provider projection work and refuses a
stale launch. Recovery authentication proves only the old signed
claim/Body/route lineage. Supervisor recovery and directive writing use that
historical reader, while the replacement launch and ready heartbeat remain
bound to the current policy. The atomic lease-transfer path also authenticates
the old launch historically. The full causal test now performs real policy
rotation and proves current refusal, historical recovery, directive fencing,
lease transfer, successor current sight, exact replay, and tamper refusal. Full
core passes 573/573 with one ignored; all binaries compile.

Candidate `351355d3baee2d475f84168136380d2f50197ef8` reached live runtime with all
three service owners active and the brake engaged, but emitted
`authenticated claim sight does not name the current managed launch`. The old
claim named the original historical launch while successive failed deployment
attempts had created a valid signed chain of replacement generations. Recovery
still required the current launch to be the old claim's direct child. The exact
Idunn process group was terminated, services stopped, active pointer and
witness removed, and Bifrost authority revoked; no deployment witness exists.

Recovery now proves ancestry rather than pretending retries did not happen.
It walks backward from the selected current-policy launch to the claimed
historical launch, authenticating every host-signed launch, exact parent
termination id and envelope digest, and termination-before-child chronology.
It refuses cycles, missing ancestors, broken edges, and a non-current endpoint.
Directive writing, directive receipt verification, atomic lease transfer, and
supervisor crash-resume use the same lineage proof. The supervisor publishes
the actual lineage error instead of collapsing it into a generic mismatch.
The causal test now includes an intermediate failed-deployment generation and
passes the complete recovery transaction. Full library passes 573/573 with one
ignored; supervisor authority tests pass 24/24.

Candidate `a70fe275195740ecefc2984b32ebd8ae93658c18` crossed the lineage gate and
supervisor published recovery directive `06165fb6-97bd-4c27-97e3-38af03dec72d`
from historical claim `c6df4335-7080-4d5f-80fc-4fa18a9d16ce` to current launch
`9414c32f-f53c-450c-bc19-a69f5ba04a29`. The projector still refused every
pulse because it authenticated current successor claim sight before consuming
the directive that creates that successor. Supervisor replayed the exact
directive safely, revealing a circular ordering fault rather than missing
evidence. The candidate was rolled back and revoked; no deployment witness
exists.

Projector pulse now authenticates and consumes a replacement-addressed recovery
directive before retirement or ordinary projection work. The atomic recovery
transaction is already exact-replay idempotent, so no speculative current-claim
query is needed in front of it. Current claim sight is recovery output, not
recovery input. A source-order test makes the authority ordering explicit. Full
library passes 574/574 with one ignored; supervisor authority tests pass 24/24.

## Ordinary organizational product and current readiness refusal (2026-07-19)

The Yggdrasil body is the normal organizational Epiphany product, not a special
experiment or dogfood mode. Persistent Modeling maps the authorized domain.
Resident Self may derive direction from the admitted map and ask Imagination
for bounded proposals without waiting for an instruction. Organization members
talk to the repository Persona; their attributed conversation becomes
additional feedback pressure. Conversation is not the ignition key and cannot
adopt Mind state, authorize Hands, grant Persona public speech, cross Bifrost,
or ask Idunn to deploy. Those remain distinct explicit authorities.

Exact candidate `9e1fa0f527b3bff2c5a709eff640b0d8d2f3b82d` consumed recovery
directive `6f1dc0c8-b8a5-48d0-add5-761b17368fbf`, replaced historical claim
`c6df4335-7080-4d5f-80fc-4fa18a9d16ce` with current claim
`2a355736-3792-4f0f-8633-a246219bcf1b`, and completed a 14,793-point local
Qdrant projection through Yggdrasil's GTX 1080-backed Ollama route. Its three
service owners remain active with zero restarts and the all-scope deployment
brake engaged. It is not deployed: Idunn's exact transaction PGID `1245215`
still owns the candidate, `deployment.env` is absent, and typed resident
readiness remains `warming` because both provider comparisons refuse.

Both provider rows continue to refresh, name the exact release
`sha256-0a15262fd4c240a0131755aa6eb63b4eb7feff12e7cc4f49128fbb001887b1f2`,
witness `sha256-49ae1afcf11d18c0e2807ea53f54dba35df4280f6596e848c5cfc50328e01aea`,
source commit, executable, and local status `ready`. The old diagnostic
`stale or release-substituted` collapses owner, runtime, release, witness,
source, executable, provider status, process incarnation, future time, and
freshness into one useless phrase. Replace that opaque boolean with the same
fail-closed predicates plus operator-safe failed-predicate names; do not relax
admission. Keep the bounded diagnosis window while the Idunn transaction and
brake remain exact. If transaction ownership or the brake disappears, or any
protected surface actuates, use the existing rollback boundary immediately.

The diagnostic successor `c9e9a352bde640d03a2e2394de1b22477a23ee96`
named the shared refusal exactly: both providers failed only
`process-incarnation`. The native observer compared the complete
`ProcessInstanceIdentity`, including optional `created_at_rfc3339`, while
resident readiness deliberately reconstructs authority from PID, native
creation token, and executable path without that display timestamp. The same
live process was therefore classified as replaced. `created_at_rfc3339` is now
explicitly derived/display-only; Windows and Linux compare only PID, native
creation token, and executable. Exact token and executable substitution still
refuse. Focused process-observation and resident-readiness suites pass. Replace
the braked, unadmitted `c9e9a352` transaction through Idunn after committing
this cut; do not reinterpret the current candidate as deployed.

Candidate `5e298e637d3470828fec3b26feeaa550a0ad91f9` proved resident readiness and
the installed operator check healthy under the `epiphany` runtime identity,
with all three services stable and the all-scope brake engaged. Idunn still
withheld final admission because workspace coverage had already terminalized
under a historical managed launch, while the restarted projector obeyed a
stale recovery directive before recognizing that durable terminal authority.
It first rejected the new launch as the owner of the old terminal receipt and
then refused to recover the already-succeeded claim.

Terminal coverage authority belongs to the succeeded claim and its historical
managed launch. A later daemon may authenticate and reuse the immutable signed
terminal sight, but cannot rewrite it or inherit the receipt. Projector pulse
now classifies current terminal coverage and republishes that historical sight
before considering process-recovery pressure; recovery still precedes every
nonterminal acquisition or claim path. The real checkpoint-to-terminal fixture
proves a different later launch receives the original sight byte-for-byte, and
the service ordering proof fixes terminal classification before recovery and
recovery before new Body work.

The terminal-authority correction is committed and pushed as exact successor
`519574ce0d63ffa8fa813885c6629c664a3ae2e0`.

Candidate `5e298e63` is revoked and its exact Idunn PGID `1485835` completed
the installed signal-safe rollback boundary: all three services, both current
links, both manifests, and the deployment lock were absent afterward. Bifrost
then authorized exact successor `519574ce`, and Idunn command
`manual:redeploy:yggdrasil-epiphany:unix:1784440234` owns its detached build
and deployment under the all-scope brake.

Next action: monitor Idunn's exact successor transaction. Require historical terminal-sight reuse,
three stable services, typed resident readiness, signed aggregate health, and
`deployment.env` before beginning the ordinary organizational product loop.

Candidate `519574ce0d63ffa8fa813885c6629c664a3ae2e0` sealed release
`sha256-3bfbd4640f7259613ea6146fe80d6a68c9be55edb1ad100fb3c39a4f0e040e3e`
and started all three braked services with zero restarts. The installed
pre-witness check was healthy, proving terminal coverage and resident readiness
were present, but the workspace projector still refused every pulse with
`workspace coverage launch disagrees with current managed policy`; the
supervisor then issued recovery directive `9084c09e-1da9-4187-acd4-6b214edd7fb2`
for the already-succeeded claim. Idunn correctly withheld signed admission and
`deployment.env`.

The surviving obsolete owner was inside terminal-sight authentication itself.
Both advancing and terminal sights shared current-policy launch authentication.
That is correct for a live advancing lease and wrong for immutable terminal
proof. Sight authentication now has an explicit authority split: advancement
requires a current-policy launch; terminal requires its exact historical
host-signed launch and envelope. Policy rotation cannot invalidate or re-sign
the succeeded claim's sight. The real checkpoint-to-terminal fixture now
rotates managed policy before a later daemon reuses the original sight
byte-for-byte.

Next action: commit/push this correction, revoke `519574ce`, terminate exact
Idunn transaction PGID `1534830`, verify signal-safe rollback, then cross the
exact successor through Bifrost and Idunn under brake.

Candidate `cb665dc6362027a06d46c3fc856c8740f4ab50ce` proved the projector-side
cut: release `sha256-026f83064dc1f20c6ae1d0528878ec96594d08692fb7ab3c0ccd24f48cd8aa46`
reused terminal receipt `bf744be34d4c969db6b74b4813caa24e231f341ea13a9346d16457188b1b77d9`
and remained idle across repeated pulses. All three services were stable,
braked, and the full pre-witness check became healthy. Idunn still withheld
admission because the supervisor independently ignored terminal sight and
selected recovery claim `2a355736-3792-4f0f-8633-a246219bcf1b`, publishing
bogus directive `7fb0a855-626a-4f94-bde2-cc88acad918c`.

Supervisor reconciliation now gives authenticated terminal sight its own
process-lifecycle path before recovery claim selection. It proves any current
observer is a host-signed descendant of the historical terminal launch. A live
observer reports `terminal-observer-alive`; degraded sight does not actuate; a
dead observer is terminated/evidence-fenced and replaced without writing a
coverage recovery directive. Recovery authority remains exclusively for a
nonterminal running claim. Focused supervisor authority tests prove terminal
selection precedes recovery and the entire terminal branch contains no recovery
writer or coverage-store mutation.

Authority map for the successor is now explicit:

- Owner: the projector alone owns coverage obligation, claim, recovery, receipt,
  terminal sight, and coverage-store mutation. The supervisor alone owns managed
  policy, process launch, observation, termination evidence, and causal process
  replacement.
- Inputs: the terminal branch reads authenticated historical terminal sight,
  current managed policy/latest launch, host-signed replacement lineage, and an
  exact native process observation. It does not read private coverage state.
- Outputs: operator-safe alive/degraded/refused status, or immutable termination
  evidence plus a causally bound replacement launch after proven process death.
- Derived state: `terminal-observer-alive`, degraded observation, and the current
  observer launch are process projections only. They neither own nor refresh the
  historical terminal receipt.
- Forbidden writers: no terminal branch may publish a recovery directive, open
  the coverage store, recover a claim, rewrite terminal sight, or mutate a
  succeeded claim. Recovery claim selection is unreachable after terminal sight
  is selected.
- Shared paths: historical and descendant observers share lineage
  authentication; alive, degraded, and dead classification share the exact
  native observation; dead replacement shares the ordinary immutable
  termination and `service_launch_internal` primitive.
- Cut line: the supervisor's old fall-through from succeeded terminal sight into
  recovery-claim selection is removed. Terminal coverage and observer liveness
  are no longer one authority.
- Verification layer: focused tests currently prove source ordering and the
  absence of recovery writers in the terminal branch. Live certification must
  still prove the actual terminal status, absence of a new directive, unchanged
  signed terminal sight, stable services, signed aggregate health, and final
  Idunn deployment admission. The structural tests are not that runtime proof.

The correction is committed as exact successor
`84e648eb374f107f26258ae96f05932c812dffae`. Candidate `cb665dc6` was revoked
and rolled back. The successor is crossing Bifrost and Idunn under the all-scope
brake. Next action: monitor only that exact transaction and require the live
verification layer above before beginning the ordinary organizational product
rehearsal. No host reboot is authorized.

Live certification refused `84e648eb` at the correct boundary. The supervisor
selected the terminal-observer path but emitted `terminal-observation-refused`:
the Ygg runtime has no repository Body-store binding, so it cannot authenticate
which repository Body the historical terminal sight belongs to. Idunn withheld
the deployment witness. The exact Bifrost authority was revoked and only Idunn
transaction PGID `1597264` was terminated; rollback left `epiphany.service`
inactive with no current link, active-release document, or deployment witness.
The next cut is not an observer bypass. Modeling must identify the owner of the
typed runtime-to-repository Body binding and make that binding available to
terminal observer authentication, preserving projector-only coverage authority.

The Body-binding correction is committed and pushed as `67f0fe0b44357c56c98f82499cdfa3ae7b944e09`.
The supervisor bootstrap already published the typed runtime-store binding and
repository Body-store binding; the live fault was reversed arguments at the
terminal-sight call site. The corrected candidate proved
`terminal-observer-alive`, three stable services, the all-scope brake, and a
healthy pre-witness check. It did not receive a deployment witness because
Idunn's automatic target worker published a newer request while the manual
deployment still owned the actuator, superseding the typed current-request
head. The candidate authority was revoked and exact process group terminated;
rollback left all Epiphany units, current pointers, active release, and
deployment witness absent.

Idunn's authority split is repaired in Odin commit
`eefd47f` (`codex/ygg-organizational-idunn`). A per-target local gate serializes
manual and automatic command lifetimes. More importantly, the persistent
current-request CAS chain refuses replacement until the exact predecessor
request has an exact terminal result, so restart or duplicate processes cannot
steal a live head. All 48 Idunn tests pass and an independent Soul re-audit found
no release blocker. Ygg now runs that binary as PID `1673938`, zero restarts,
SHA-256 `49e09d1989342737f6c996be267894555e8c23886f090a5ae5d334e4aa29ff8d`;
the prior binary is `/usr/local/bin/idunn.20260719-072532.bak`.

Bifrost correctly refuses reauthorization of revoked immutable authority for
`67f0fe0`. The next action is therefore to commit/push this persisted Epiphany
state as a new exact successor, authorize only that commit, and perform one
Idunn-owned Ygg deployment under the all-scope brake. Certification requires a
single stable deployment request head, `terminal-observer-alive`, three stable
services, exact signed health admission, and final `deployment.env` witness.

After certification, treat the organizational loop as normal product behavior,
not an experiment. The existing runtime autonomously turns admitted Modeling
direction into an Imagination option, and Persona feedback is authenticated
additional pressure. The missing bridge is option result -> immutable
`RepoFrontierWorkProposal` with exact model causality -> automatic Self
selection for Modeling review -> admitted frontier item -> existing Imagination
planning -> explicit Mind adoption -> Hands. Proposals and conversation must
remain structurally unable to grant Mind, Hands, Persona public speech,
Bifrost release, or Idunn deployment authority. No host reboot is authorized.

## Idunn observation/actuation authority split (2026-07-19)

The e449 successor reached exact resident readiness, stable terminal coverage,
and three healthy braked services, but produced no Idunn witness. Commit
`75bd5f2` moved the mutex without separating the automatic target task: that
task still awaited the long deployment command and could not re-evaluate the
new Epiphany RUDP health needed to finish it. The exact deployment PGID was
terminated, rollback removed all services/current pointers/witnesses, and
Bifrost authority for e449 was revoked.

Odin commit `a17b0b6` is the actual authority split. The target loop remains
continuously available for observation, evaluation, admission, and publication.
Deploy/restart consequences run separately under one per-target reservation;
manual and automatic paths share it, and stale automatic work is dropped while
the actuator is occupied. Durable request-head/result CAS remains unchanged.
All 49 Idunn tests pass. No Ygg deployment has yet used this binary.

The autonomous option-to-frontier bridge is now locally release-ready. A typed,
immutable deployment-configured repository-domain binding joins runtime,
swarm, workspace, and the exact Body binding. It is an organizational label,
not authenticated Git-remote identity. Promotion reloads the exact direction
request/result, worker result and launch, admitted model and receipt, canonical
Body Git root, and domain binding before one CAS creates an inert proposal and
Self selection for Modeling. Generic Imagination intake cannot forge this path.
Proposal Modeling may recommend only Imagination and may not carry an adopted
plan; ordinary Evolution cannot mutate frontier state. The hostile full-chain
test attempts a proposal-citing direct Hands route, is rejected with a
byte-identical store, and leaves Hands empty. Full core proof is 578 passed,
zero failed, one ignored; all binaries compile; Soul found no release blocker.

Live certification of state-only Epiphany commit `36f687e` exposed a separate
Idunn verifier wound. Epiphany sent a 2,260-byte signed active health document
with the exact request, release, witness, commit, signer, and Body terminal
receipt. Idunn received it but tried to create
`/etc/gamecult/idunn/epiphany-health-identity.ccmp.lock` while reading the
root-owned trust anchor, so no signed admission or deployment witness was
published. Making the config directory daemon-writable is forbidden because it
would let the verifier alter its own trust root. Odin commit `64bb679` uses an
atomic read-only CultCache snapshot for this one pinned-anchor read, proves no
sibling lock in a read-only directory, and passes 50/50 Idunn tests.

Next action: install exact Idunn `64bb679` on Ygg, allow the incomplete
transaction to roll back cleanly, then retry the exact Bifrost-authorized 36f
release under the all-scope brake. Require signed active admission and final
`deployment.env`. Then commit/push the autonomous bridge and certify its exact
successor separately. No host reboot is authorized.

## Ygg resident baseline certified (2026-07-19)

That crossing is now complete. Idunn `64bb679` is installed at SHA-256
`bad36e84706ec0a2354012d11aa7556fd0bd2f01484c36d3a5d0bc0b8390485f`.
The incomplete transaction rolled back with all units and candidate witnesses
absent, then typed redeploy intent
`manual:redeploy:yggdrasil-epiphany:unix:1784452916` retried exact commit
`36f687eaa34153a166b5860087c28f8847244823`. Idunn first admitted signed
degraded health while one service reconciled, then admitted the exact active
candidate and published deployment manifest v2 at `2026-07-19T09:23:14Z`.
Release id is
`sha256-a9afe96abf5b104da3d25bf5e11b967de546c57a230fa155b3cbb98c3b176385`;
witness is
`sha256-f6b4c68054ce9566002e46d49e8773924d3518afdd02798e1a09f1778359d3c2`.
Idunn, supervisor, Heartbeat, and resident Self are active with zero restarts;
the full Ygg typed check passes; no trust-anchor lock exists; the all-scope
deployment brake remains engaged.

The autonomous organizational bridge is committed and pushed at `fc733675`.
Next commit this certification state, cross only that exact successor through
Bifrost and Idunn under the brake, then prove Persona/Discord replay and tamper
boundaries plus Starfire-independent operation before granting any wake.

## Autonomous successor refused; cold-start owner corrected (2026-07-19)

Bifrost authorized and Idunn attempted exact successor
`2d21c48d7425c3d6d692e506e1981f92991d82ac`. Idunn correctly refused final
admission: `epiphany-swarm.service` restart-looped with `autonomous proposal
promotion requires thread state`. The deploy process was terminated and
rollback restored exact 36f source, active release symlink, release environment,
supervisor, and Heartbeat. Resident Self is intentionally stopped until the
corrected successor crosses; this prevents durable proposal pressure from
retriggering the old binary's fault. The all-scope brake remains engaged.

The fault belonged to promotion intake. Absent thread state during cold start
means no proposal is eligible yet; it is not corrupt state. Promotion now
returns an empty result without writing when the thread is absent. Present but
malformed authority still fails. A focused byte-identity negative test passes,
and the complete core library proof is 579 passed, zero failed, one ignored.

Separately, `gamecult-ops` commit `584b94c` moves clone/build/package/verify
before the service stop. Packaging failures can no longer blind the active
body; the remaining outage begins only at canonical Body mutation/promotion.

Next: commit/push this correction and state, revoke refused 2d authority,
authorize only the corrected exact commit, install/use the ops cut, and deploy
through Idunn under the brake. Admission requires all three units active with
no restart growth and exact signed health. Then run the Starfire-offline
Persona/Discord feedback rehearsal. No host reboot is authorized.

## Corrected organizational resident admitted (2026-07-19)

The cold-start correction is committed and pushed as
`154dc1eadd98dda25916ee40e7130770e145e874`. Cancelling the refused 2d
actuator exposed a separate Idunn recovery wound: its child process died with
the daemon, but the exact deployment head survived without a terminal result
and permanently blocked supersession. Odin commit
`15a744c826f0aa2e8cc322a6543ea8a9afcd852a` repairs the owner: fresh Idunn
startup CAS-terminalizes only an exact unresolved prior-incarnation request as
failed. It passes 51/51 tests. Ygg runs that binary at SHA-256
`c441b9e58e85aff1d869b1e07b0a12a4795c6805619ce640184dbdcfce4e7e03`.

The deployment purification cut also needed package-directory traversal.
`gamecult-ops` commits `584b94c` and `46425f9` keep packaging before the outage
and seal package directories `0555`; files retain their built execute bits.

Exact 154d is now Bifrost/Idunn admitted on Yggdrasil:

- authority digest: `d7e21f11d758e3ebaa46945ddd7acca1417ab8f8a81c3ec1932f3d5e0109e72f`
- release: `sha256-16c51bb3b4914578213f0d0b64e40e17982ad45832e073d747db903509f237ed`
- witness: `sha256-85702cf922ea32c07457630a7911b2b9196c8b58ee4baa642185a0b77902f234`
- Idunn request: `deploy:yggdrasil-epiphany:unix:1784456326`
- manifest time: `2026-07-19T10:20:04Z`

The full typed check passes. Signed health names the exact source, release, and
witness. Supervisor and Heartbeat have zero restarts. Resident Self is stable
on one MainPID; its historical restart counter remains 10 without growth. Its
operator projection is braked, revision zero, no active turn, and no private
state exposure. GPU embedding remains 100% on the GTX 1080.

Next: run the Starfire-offline real Discord/Persona feedback rehearsal wholly
on Ygg, proving replay/tamper refusal and byte-identical Mind, Hands, release,
and deployment stores. Then audit the Discord operator surface needed during
the move. No wake grant and no host reboot are authorized.

Modeling confirms that current feedback is safe cognition pressure, not remote
operation. Status beyond generic VoidBot provider availability, brake
sleep/wake, operator-objective pressure, and exact Mind review still require a
local CLI/SSH path. Starfire is not intrinsically required, but Discord has no
typed ingress for those owners. Current Discord replies are local VoidBot
repo-Face turns; resident Epiphany has no correlated Persona speech return
path yet.

The authority/design map is now
`notes/epiphany-discord-operator-nerve.md`. July-22 commands are explicit
status, sleep, wake, directive, reviews, and exact Adopt/Refuse/Hold review:
Discord authenticates, Bifrost signs an expiring capability-bound command,
Epiphany invokes one named owner primitive, and a sealed Epiphany result returns
to Discord. No chat inference, generic argv, direct store mount, wake-and-run,
or release/deploy authority is permitted.

Epiphany core commit `f1036abd` now implements Status, Sleep, Wake, and
Directive as a closed typed command family. Admission is separately
Bifrost-signed and exact-target/guild/channel/actor-capability/nonce/TTL bound.
Invalid admission writes nothing. Replays recover after crashes both before and
after the owner consequence. Sleep owns one fixed Discord brake and cannot
replace a foreign engaged brake. Wake releases only that brake; the live
deployment brake produces a terminal Refused result and survives byte-identical.
Wake creates no pressure/grant/job. Directive creates one pending
operator-objective pressure only. Focused proof is 4/4; full core is 583 passed,
zero failed, one ignored.

No live daemon/CultNet ingress exists yet, and reviews/review are not in the
core family. Bifrost and VoidBot Hands passes are now building the explicit
owner-only crossing against f1036abd. After those land, add the supervised
Epiphany network ingress and host-sealed result receipt; do not mount Epiphany
stores into either sibling.

## Live organizational feedback half proved; projection lie cut (2026-07-19)

The Ygg production Bifrost feedback provider is ready with six typed bindings.
The live stores contain one real VoidBot Discord observation, one Bifrost-signed
delivery, and one Epiphany local pressure admission. Re-import through deployed
154d is byte-identical across the delivery, admitted-feedback, Mind, runtime,
and deployment stores. An alien signing identity is refused before writes. A
substituted repository finds zero applicable deliveries and writes no
feedback/Mind/runtime state.

That zero-delivery probe exposed a diagnostic conceptual substitution:
`epiphany.persona_feedback.ingress_projection.v0` printed `status=admitted` and
`admittedCount=0`. No consumer depends on it. Successor source deletes the v0
lie and emits typed v1 status `pressure-present` or
`no-applicable-deliveries`, with `applicableDeliveryCount` and
`presentPressureCount`. Its focused test and binary check pass.

Source inspection proves the local Persona response path is independent of
observation export: VoidBot queues the repo-Persona turn first, then launches
the three-attempt export asynchronously. Export failure therefore cannot
suppress the local response.

The final live pressure-to-Imagination scheduling step is intentionally not
run while the all-scope brake is engaged. Do not release it to flatter the
smoke. Modeling is mapping the separate move-period Discord operator nerve;
before July 22, essential status/sleep/wake/directive/review must not depend on
Starfire or improvised SSH, and Persona conversation must remain pressure
rather than Mind/Hands/release/deployment authority.
