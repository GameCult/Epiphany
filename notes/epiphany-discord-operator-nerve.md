# Epiphany Discord Operator Nerve

## Objective

Yggdrasil must continue operating while Starfire is packed. The operator must
be able to inspect and direct Epiphany through Discord without turning ordinary
conversation into command authority or giving VoidBot access to Epiphany's
private stores.

Persona feedback and operator command are different nerves:

- Persona conversation is `feedback_only` at VoidBot/Bifrost and
  `resident-pressure-only` in Epiphany. It may provoke attention and an
  Imagination proposal. It cannot adopt Mind state or grant consequence.
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

- Status: read-only operator-safe projection over release, brake, resident,
  pending pressure/review counts, and provider health. It exposes no raw worker
  state or transcripts.
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
6. `review`: exact request id plus `Adopt|Refuse|Hold`, expected revision, and
   optional bounded rationale.

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
   and review counts, and Idunn/Bifrost health without private state;
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

## Current gap

The live Ygg body has the Persona feedback nerve but none of these explicit
Discord operator command crossings. `epiphany-verse-query swarm-brake` and
other local CLIs rely on filesystem authority and caller-supplied operator
labels; they are implementation primitives, not remote authentication. Generic
VoidBot `provider-status`, `queue-codex`, and job approval operate VoidBot's own
provider/job system and cannot stand in for Epiphany status, direction, or Mind
review.

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
  `write_epiphany_cultmesh_swarm_brake`)
- Epiphany operator-safe status owner:
  `EpiphanyCultMeshOperatorSnapshotEntry` in the same CultMesh module
- Epiphany frontier Mind decision owner:
  `epiphany-core/src/runtime_spine.rs` around
  `commit_repo_frontier_plan_decision`
- Ygg deployment/configuration:
  `E:\Projects\gamecult-ops\compose\voidbot.yggdrasil.yaml`,
  `compose\bifrost.yggdrasil.yaml`, and
  `runbooks\epiphany-yggdrasil-deploy.md`
