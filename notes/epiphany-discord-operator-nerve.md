# Epiphany Discord Operator Nerve

## Objective

This is Epiphany's ordinary organizational interface, not a remote-control
fixture for one deployment. A resident swarm continuously models its authorized
domain, derives bounded direction from the admitted map, and lets Imagination
suggest improvements and possible features without waiting for an instruction.
The repository Persona gives the organization a shared conversational crossing:
members can add attributed feedback, challenge the model, and discuss proposals
without becoming machine operators merely by speaking.

Explicit operator commands form a second, narrower nerve for named consequences
such as inspection, sleep, wake, direction pressure, and Mind review. Yggdrasil
continuing while Starfire is packed is the July 22 acceptance case for that
product contract, not the reason the contract exists. VoidBot must never gain
access to Epiphany's private stores.

Persona feedback and operator command are different nerves:

- Persona conversation is `feedback_only` at VoidBot/Bifrost and
  `resident-pressure-only` in Epiphany. It may provoke attention and an
  Imagination proposal, but it is not required to ignite either Modeling or
  autonomous direction. It cannot adopt Mind state or grant consequence.
- Operator commands are explicit owner-authenticated slash commands. They cross
  Bifrost as signed, expiring, target-bound typed intent and invoke one named
  Epiphany owner primitive. They are never inferred from prose.

## Authority map

### Discord / VoidBot

Owner: Discord interaction capture and owner authentication.

Inputs: one explicit `/epiphany` subcommand, Discord actor/guild/channel and
interaction identity, and command fields allowed by that subcommand.

Outputs: one immutable operator-command observation for Bifrost, or an
immediate authentication/shape refusal.

Does not own: Epiphany state, brake transitions, pressure admission, Mind
review, Bifrost signing, release, deployment, or arbitrary command execution.

### Bifrost / Heimdall

Owner: outside identity/capability association and governed crossing.

Inputs: the exact Discord observation, an operator capability associated with
the actor, configured target binding, current time, and command policy.

Outputs: one signed, expiring, nonce-bound command admission; later, one sealed
result delivery bound to the exact command admission.

Does not own: the resulting Epiphany transition or receipt. A Bifrost signature
proves who requested what crossing; it does not prove that Epiphany applied it.

### Epiphany operator-command ingress

Owner: target validation and dispatch to one existing Epiphany owner primitive.

Inputs: Bifrost trust anchor, signed command admission, exact runtime/repository,
nonce, expiry, expected prior state/revision when required, and the command's
bounded payload.

Outputs: an Epiphany-owned typed result receipt containing applied/refused/no-op,
the exact owner receipt or snapshot reference, and no private state.

Does not own: Discord identity, Bifrost capability, generic process execution,
release authorization, or deployment.

### Existing Epiphany owners

- Status: read-only operator-safe projection. The deployed v1 result currently
  carries a coordinator snapshot plus the independently owned brake state. It
  exposes no raw worker state or transcripts, but it also does not yet compose
  authenticated release, resident-readiness, Idunn, or Bifrost health.
- Sleep: swarm-brake owner engages the named cognitive/action surfaces.
- Wake: swarm-brake owner releases the brake only. Wake does not issue a grant,
  enqueue a turn, adopt a plan, invoke Hands, speak, release, or deploy.
- Directive: resident-pressure owner admits one `operator-objective` pressure
  with exact actor/capability provenance and expiry. It grants attention only.
- Reviews: the owning Mind/review gate lists bounded pending requests and
  applies exact `Adopt`, `Refuse`, or `Hold` through its existing commit
  primitive. Discord and Bifrost cannot mint the resulting receipt.

## Command families

1. `status`: read-only; idempotent; returns a sealed snapshot.
2. `sleep`: engage brake; requires expected brake identity/revision; repeated
   exact command is idempotent.
3. `wake`: release brake; requires expected brake identity/revision; produces
   no work grant.
4. `direct`: admits one expiring operator-objective pressure; never parses the
   text as a command family or consequence grant.
5. `reviews`: read-only bounded list of pending review summaries.
6. `review`: exact Mind request id, candidate id/digest, model revision/hash,
   and `Adopt|Refuse|Hold`. The result contains identities and disposition only.

Every mutating command carries a unique command id/nonce, issued-at, expires-at,
target runtime/repository, actor identity, Bifrost capability reference,
expected prior state where relevant, and exact payload digest. Replays return
the existing result only when command and payload are byte-equivalent.

## Forbidden writers and cut line

- No natural-language command inference, regex tribunal, or magic prefix.
- No `/queue-codex` or shell proxy.
- No arbitrary argv, executable path, environment, or store path in a command.
- No Discord bot mount or filesystem permission for Epiphany stores.
- No wake-and-run composite.
- No conversation-derived Mind, Hands, Persona speech, release, deployment, or
  service-lifecycle authority.
- No status assembled from PID/process prose when a typed provider owns it.
- No result success manufactured by VoidBot or Bifrost.
- Release authorization/revocation remains Bifrost's separate release gate.
- Deployment and service recovery remain Idunn's separate lifecycle gate.

## Shared paths

Discord interaction -> VoidBot observation -> Bifrost admission -> Epiphany
ingress -> named owner primitive -> Epiphany result -> Bifrost delivery ->
VoidBot interaction response.

All six commands use this route. Status/reviews stop at read-only owners.
Sleep/wake share the brake transition primitive. Directive shares resident
pressure admission. Review shares the canonical Mind/review commit primitive.

## July 22 proof

With Starfire disconnected and no SSH tunnel dependency:

1. owner and non-owner Discord actors receive the correct admission/refusal;
2. status names exact deployed commit/release, brake, resident state, pressure
   and review counts, and Idunn/Bifrost health without private state; this is an
   open acceptance item, not a property of the current coordinator snapshot;
3. sleep engages the brake and is idempotent;
4. wake releases only the brake and creates no grant, job, Mind change, Hands
   authority, speech, release, or deployment;
5. direct creates exactly one expiring operator-objective pressure and no other
   authority;
6. reviews exposes only bounded summaries;
7. review applies only to the exact current gate and stale/substituted/replayed
   decisions refuse without writes;
8. every accepted/refused/no-op command returns a sealed receipt bound to the
   Discord interaction and Bifrost admission;
9. signer, target, nonce, expiry, actor capability, and payload tampering fail
   without changing Epiphany state;
10. Ygg services survive restart and continue the route without Starfire.

## Implemented Epiphany review owner (2026-07-19)

`Reviews` projects at most ten current `RepoFrontierPlanMindRequest` identities.
Mind revalidates the immutable Imagination result, candidate digest, current
RepoModel revision/hash, frontier item, runtime, and thread. Plan action,
command, paths, checks, rollback, proposal text, and private state stay sealed.

`Review` enters the existing `RepoFrontierPlanDecisionReceipt` owner. `Refuse`
and `Hold` are canonical terminal receipts without RepoModel mutation. `Adopt`
uses the existing atomic RepoModel admission CAS; it creates no route, Hands
authority, Substrate grant, Persona speech authority, release, or deployment.
Decision provenance is typed as a Mind worker result or an authenticated
operator review carrying command, admission, packet digest, and actor identity.
Downstream RepoModel provenance is independently typed as worker result or
frontier-plan decision. Absence is `None`, never an empty worker-id sentinel.

The v1 command and sealed-result contracts derive review decision time from the
immutable packet `issuedAt`, so consequence-before-result replay is independent
of the retry clock. Candidate mismatch becomes a terminal refusal; storage,
decoding, CAS, and corruption faults propagate. CultCache v0 decision tuples
decode worker provenance into explicit legacy Options. Signed v0 operator
deliveries may drain and replay the original four families, but cannot acquire
the v1 review vocabulary.

## Live deployment boundary

The Ygg body runs the v1 six-command crossing through VoidBot, Bifrost, and the
loopback-only Epiphany operator service. Bifrost commit
`6491f449cf0dbfa952b409de90ccdf511669a60b` cut the surviving v0 request
identity; VoidBot is the v1 producer and Bifrost is the v1 consumer. Legacy v0
admission/result replay remains sealed to the original four commands and is not
a second live request authority. Focused Bifrost interop tests, provider tests,
VoidBot TypeScript compilation, and the cross-runtime smoke pass. The worker is
healthy on Ygg and currently reports zero pending results.

The remaining acceptance proof needs a real owner Discord interaction; a local
fixture must not impersonate an organization member. Run `/epiphany status` and
ordinary Persona feedback, then inspect typed receipts and negative authority
deltas only. Keep the canonical deployment brake engaged and do not exercise
Wake. Generic VoidBot `provider-status`, `queue-codex`, and job approval operate
VoidBot's own provider/job system and cannot stand in for Epiphany status,
direction, or Mind review.

## Rust/Bifrost boundary proof (2026-07-19)

The Epiphany-owned command service now admits the exact signed Bifrost command,
persists admission before consequence, recovers an identity-equal admitted
command after packet expiry, and seals the truthful completion time. Packet
expiry gates first admission; it is not a demand to falsify recovery time.
Malformed UDP is contained inside the daemon loop, and the hostile smoke proves
the same service instance can subsequently complete a CultNet/RUDP handshake
and return the exact sealed receipt.

`epiphany-operator-command-fixture` emits public Rust-produced interop bytes:
the named admission and sealed receipt, raw compact Bifrost/executor trust
anchors, the canonical executor CultCache `.cc`, and a hash/purpose/connection
manifest. Private fixture signers are quarantined during generation and deleted
before return. The ignored persistent smoke output lives at
`.epiphany-smoke/operator-command-interop-rust`; regenerate it rather than
committing generated signatures or private material.

Status now has an explicit Epiphany-owned v2 wire projection. The persisted
command result remains v1; only a Status receipt carries
`epiphany.operator_command.status_result.v2` plus typed `statusV2`. Every call
queries Idunn's public CultNet/RUDP snapshot for exactly Epiphany and Bifrost,
verifies each returned record independently against the root-pinned Idunn
projection anchor, and commits the local admission store only when the complete
set verifies. A valid record remains visible when its peer is absent, while the
set stays `incomplete`; stale admission is never reused as current sight.
Provider output contains bounded state, reason, clocks, lineage, and digest—no
signature, detail, path, key, or private state. `protocol.json` now contains the
typed `operatorStatusV2MigrationFixture` golden value for Bifrost and VoidBot
migration; neither consumer is changed by this pass.

Bifrost's JavaScript smoke consumes those Rust bytes, verifies admission and
receipt signatures plus payload digests, compares the real compact enum/byte
shapes, and rejects mutation. That proof exposed two corrected boundary faults:
Rust compact command enums are unit strings or `[variant, sole-field]` tuples,
not named command maps, and MessagePack integer sequences must be normalized to
exact byte arrays before signature verification.

Addressing Epiphany currently queues a local VoidBot repo-Face turn and
independently exports the visible prompt as remote feedback. There is not yet a
correlated live Epiphany Persona speech result -> Bifrost Discord receipt ->
original-message reply path. Describe current replies as the local repo Face;
do not claim they are round-trip speech from resident Epiphany. That return
path is later work and must not delay the move-period command nerve.

## Source anchors

- VoidBot interaction/addressing owner:
  `E:\Projects\VoidBot\apps\bot\src\discord-bot.ts`
- VoidBot feedback-only document:
  `E:\Projects\VoidBot\packages\core\src\persona-feedback-observation.ts`
- VoidBot slash commands and owner job controls:
  `E:\Projects\VoidBot\apps\bot\src\discord-bot-handlers.ts`
- Bifrost feedback documents/provider:
  `E:\Projects\Bifrost\tools\persona-feedback-documents.mjs` and
  `E:\Projects\Bifrost\tools\persona-feedback.mjs`
- Bifrost outbound operator alarm seed:
  `E:\Projects\Bifrost\tools\operator-notification.mjs`
- Epiphany signature/target admission pattern:
  `epiphany-core/src/persona_feedback_admission.rs`
- Epiphany pressure owner:
  `epiphany-core/src/resident_self.rs` (`enqueue_resident_self_pressure`)
- Epiphany brake owner:
  `epiphany-core/src/cultmesh_integration.rs`
  (`load_epiphany_cultmesh_swarm_brake`,
  `engage_epiphany_cultmesh_swarm_brake`,
  `release_epiphany_cultmesh_swarm_brake`)
- Epiphany operator-safe status owner:
  `EpiphanyCultMeshOperatorSnapshotEntry` in the same CultMesh module
- Epiphany frontier Mind decision owner:
  `epiphany-core/src/runtime_spine.rs` around
  `commit_repo_frontier_plan_decision`
- Ygg deployment/configuration:
  `E:\Projects\gamecult-ops\compose\voidbot.yggdrasil.yaml`,
  `compose\bifrost.yggdrasil.yaml`, and
  `runbooks\epiphany-yggdrasil-deploy.md`
