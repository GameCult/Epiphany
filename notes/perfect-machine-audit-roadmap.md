# Perfect Machine Audit Roadmap

Updated: 2026-07-12

## Objective

Prepare Epiphany as a coherent organism whose repository access, evidence,
projection, public speech, action, verification, continuity, durable state,
runtime physiology, and inspection surfaces have one visible owner each.

This is a live authority audit. Historical implementation scars belong in git,
distilled evidence, or archived notes. A green local test proves only the seam
it actually observes.

## Authority Map

| Faculty | Owner | Inputs | Outputs | Current proof | Live gap |
|---|---|---|---|---|---|
| Self | native coordinator and repo-work scheduler | typed state, job/result receipts, current route pressure | bounded launch/routing decisions | coordinator acceptance tests; repo-work scheduler authority tests | heartbeat is not yet an always-on Idunn-owned process |
| Substrate Gate | `substrate_gate` | scoped read/mutation request | access grant, refusal, snapshot, mutation receipt | Research launch/acceptance and Hands gate proofs | audit remaining utility/bridge substrate touches for bypasses |
| Eyes | `eyes_gateway` | inspected source under a Substrate Gate grant | evidence packet or refusal | Research acceptance proof profile | public/foreign Verse adoption still needs a full live Eyes-to-Mind proof |
| Imagination | planning and Persona projector boundaries | typed project/person state and candidate futures | plans, projections, consensus candidates | planning/consensus smokes and Persona projector tests | Persona projector/interpreter parity with the current Void reference needs a fresh audit |
| Persona | Persona turn and public mouth edges | projected context plus visible conversation | natural speech candidate and speech audit | Persona Discord/Reddit/Bifrost smokes | Aquarium-native conversational inspection remains incomplete |
| Hands | `hands_gateway` and repo-work execution | approved intent, path scope, Substrate Gate grant | patch, command, commit, PR, rollback receipts | real repo-work execution/closure and Hands receipt-chain smokes | search for non-repo-work actuators that still bypass the chain |
| Soul | `soul_gateway` and verification phase | Hands consequence receipts and changed body | verdict/invariant/regression receipts | live repo-work Soul phase and acceptance tests | unify remaining verifier-specific projections under narrow receipt queries |
| Modeling | typed repo-work Modeling route and model runtime | immutable Soul-verified request selected by current generation | immutable finding | generation-zero/generation-one runtime, Mind revision, Idunn launch, and admission proofs | no known authority split in the repo-work path |
| Mind | canonical state transaction and gateway reviews | bounded effect proposal plus required organ proofs | durable state and commit receipt | one canonical writer/store; atomic coordinator and repo-map admission proofs | audit other state families for pre-transaction legacy writers |
| Continuity | continuity contracts plus CRRC/stale-turn repair | pressure, checkpoint, sleep, and stale active state | checkpoint, recovery, sleep-distillation, stale-turn receipts | reorient acceptance and heartbeat stale-repair tests | sleep consolidation is callable, not continuously supervised physiology |
| Nervous system | heartbeat state, runtime spine, Idunn, CultMesh/CultNet | pressure, pending turns, daemon/runtime state | scheduling, lifecycle, telemetry, interrupts | typed heartbeat pump/routine, runtime-spine jobs, Idunn service receipts | the heartbeat loop is still an attached PowerShell vigil rather than a native supervised daemon |
| Body | repository, Rust binaries, CultCache/CultMesh stores, model transport | typed commands and granted external authority | real effects and inspectable artifacts | focused smokes and live repo work | continue cutting whole-context queries and compatibility-shaped native artifacts |

## Structural Invariants

1. Self routes; it does not author specialist findings or admit durable state.
2. Substrate Gate owns substrate permission. Eyes owns evidence promotion.
3. Hands alone changes the repository; Soul alone calls the consequence
   verified; Modeling updates the machine model after Soul.
4. Mind is the only durable-state admission owner. Reviews, state mutation, and
   commit witnesses share the canonical transaction boundary where atomicity is
   required.
5. Immutable Modeling findings never change. Mind may advance one stable typed
   route to a new generation; stale generations cannot close or regain current
   authority.
6. Idunn owns process and daemon lifecycle. Self may request a launch but may
   not spawn, repair, or silently replace the child.
7. CultCache documents are state, CultMesh is the local state/discovery
   substrate, CultNet is the wire, and Eve/CultUI is the interface projection.
   JSON is display or xenos-boundary cargo, not internal authority.
8. Aquarium and other renderers inspect typed owners. They do not rebuild a
   second route graph or dashboard-shaped truth.
9. Pending turns freeze repeat scheduling. Cooldown begins after completion.
   Stale turns require Continuity evidence, not timer-shaped amnesia.
10. Public connection preserves consent, identity, permission, provenance, and
    private-state seals at the actual mouth edge.

## Evidence Ledger

### Proven executable chains

- Research: Substrate Gate grant -> Eyes packet -> Mind admission.
- Implementation: scoped Hands intent/review -> patch/command/commit receipts.
- Verification: Hands consequence -> Soul verdict -> Modeling request.
- Repo map: immutable Modeling finding -> current typed route -> Mind/map
  transaction -> Bifrost publication gate.
- Retry: runtime-authored non-passing generation zero -> explicit Mind review ->
  generation-one route -> consumer schema preflight -> Idunn lifecycle receipt
  -> runtime-authored passing finding -> current-generation admission.
- Recovery: reorient launch -> Continuity recovery receipt -> Mind admission.
- Public crossing: Persona speech audit -> Bifrost/Heimdall authority checks ->
  governed receipt or sealed refusal.

### Proven negative boundaries

- Scheduler cannot author Modeling findings or impersonate Mind admission.
- CLI cannot counterfeit a runtime-authored Modeling finding.
- Non-passing and stale-generation findings cannot enter the repo map.
- Passing findings cannot be revised into retries.
- `epiphany-work` cannot spawn the model child; Idunn owns process launch.
- Consumer schema preflight happens before the runtime job is opened.
- Extinct Codex Epiphany DTO/route/bridge paths cannot write native state.

## Current Highest-Priority Gap: Native Heartbeat Daemon

### Current mechanism

`epiphany-heartbeat-store` owns typed `tick`, `pump`, `complete`,
`repair-stale`, `routine`, and `status` operations. It already enforces pending
turn freeze, completion-gated cooldown, adaptive pacing, swarm brakes, stale
turn repair, rumination, sleep, memory resonance, and dream maintenance.

The first native loop is landed as `epiphany-heartbeat-store serve`. It reuses
the existing routine state owner, writes per-pulse artifact directories, emits
compact sealed pulse/closure receipts, refuses a zero interval, and supports
bounded clean shutdown for verification. It does not spawn or own child
lifecycle. The old attached PowerShell rumination vigil has been deleted; it no
longer owns timing, loop survival, or cycle status.

### Intended change

The bounded Idunn launch boundary is proven: the native loop survives brake
refusals, resumes from the same persisted store, and can be launched again by
Idunn without duplicate pending turns. The heartbeat binary owns pulse timing
and typed heartbeat receipts. Idunn owns child launch, stdout/stderr artifacts,
and lifecycle receipts. Self remains only the routing organ consuming heartbeat
pressure. Existing `epiphany.cultmesh.daemon_restart_policy` is not the right
owner for the remaining step: it is keyed to standing topology daemons and
their liveness status. The heartbeat loop is an Idunn-managed child service.
The missing owner is a typed managed-service desired-state policy keyed by
service id, plus compact operator readback; do not forge an eighth standing
daemon to reuse the daemon policy table.
That desired-state document is now landed as
`epiphany.cultmesh.managed_service_policy.v0`. Idunn can write it from the same
service command/args/cwd and sealed log refs used by lifecycle launch, and can
read it beside the latest lifecycle receipt. Readback explicitly reports
process observation as unknown until reconciliation probes reality; a past
`launched` receipt is not allowed to impersonate a living process.
Managed-service reconciliation is now implemented with a native platform PID
probe (`OpenProcess`/`GetExitCodeProcess` on Windows and `kill(pid, 0)` on
Unix). It honors enabled/restart mode/cooldown, observes alive services without
duplicating them, and delegates dead/missing restarts to `service-launch`.
Launch receipts now include an attempt timestamp so restart history is
immutable instead of overwriting one service/action key. Live heartbeat proof
launched, observed alive, simulated a crash, relaunched with a distinct PID and
receipt, then observed the replacement alive on the same heartbeat store.

### Owner

- Heartbeat scheduler: when a pulse is due and which bounded routine/pump action
  runs.
- Idunn: whether the heartbeat process is running and how it restarts.
- Continuity: repair of stale active turns and sleep/compaction evidence.
- Mind: admission of durable self/memory changes proposed by heartbeat work.

### Inputs

- heartbeat CultCache store;
- agent-memory store;
- local Verse store and swarm brake;
- bounded interval, maximum iterations, and shutdown signal;
- current pressure and pending-turn state.

### Outputs

- typed heartbeat selection/completion/routine receipts;
- compact status/telemetry projection;
- Idunn service lifecycle receipt and sealed stdout/stderr artifacts;
- explicit clean shutdown or failure status.

### Durable service policy requirement

Add one typed Idunn managed-service policy with owner `Idunn` and key
`service_id`. It may read the desired command/args/cwd, enabled state, restart
mode, cooldown/backoff, expected heartbeat store, stdout/stderr artifact refs,
and latest lifecycle receipt. It emits desired-state readback and delegates
every actual start/restart to the existing service lifecycle launch primitive.
It does not own heartbeat scheduling state, daemon topology, or routine output.
The persistence/publication half is complete. Next add reconciliation that
probes the policy's last PID or an equivalent platform process witness, applies
enabled/restart-mode/cooldown intent, and delegates any restart to the existing
service lifecycle launch primitive.
Reconciliation is complete as an explicit Idunn command. The remaining
unattended boundary is to include managed-service reconciliation in Idunn's
native `serve` scheduler and replace `service-launch`'s full local-Verse context
load with the narrow swarm-brake query it actually needs.
That boundary is now complete. Service plan/launch use the narrow typed brake
loader and no longer reseed the full local Verse. A dedicated
`managed-service-serve` loop enumerates only managed-service policies and runs
their reconcile reflex independently of standing-daemon topology. Live proof
observed a heartbeat alive without duplicate launch, killed it, and the next
Idunn pulse restarted it from the same store with a distinct PID and immutable
lifecycle receipt. The next gap is inspectability: publish compact managed
service desired/observed rows through Gjallar/Eve without opening command
payloads or routine artifacts.
The first sight organ is landed as `epiphany-verse-query
gjallar-managed-services`. Native process observation now lives in a shared
core module used by both Idunn reconciliation and Gjallar projection. Rows show
desired enabled/restart mode, observed alive/dead/missing, PID, latest immutable
lifecycle receipt, compact last heartbeat pulse status/iteration/artifact, and
the private-state seal. Live proof transitioned the same row from READY/alive
to ATTN/dead after a forced process death without exposing command args or
routine payloads. Next lower this typed row family into the main Gjallar/Eve
composition graph.

### Forbidden writers

- PowerShell wall-clock loops deciding heartbeat truth;
- Self spawning or restarting the heartbeat process;
- a renderer owning scheduling state;
- sleep directly mutating durable identity without Continuity/Mind review;
- overlapping pulses while a previous pulse remains active.

### Shared paths

Manual `tick`/`pump`/`routine`, daemon pulses, recovery after restart, swarm
brake refusal, and shutdown must call the same existing store primitives. The
daemon is orchestration around those owners, not a second scheduler.

### Cut line

The attached PowerShell rumination vigil is deleted. Do not preserve its
timing, status, or cycle-directory conventions as runtime authority.

### Verification

- unit test: `serve` refuses zero/unsafe interval and overlapping active work;
- bounded smoke: two iterations use the same typed routine/pump primitives and
  stop cleanly;
- restart smoke: Idunn lifecycle receipt proves child ownership and a second
  process resumes from persisted heartbeat state without duplicate pending
  turns;
- brake smoke: an engaged swarm brake prevents mutation while the daemon stays
  observable;
- negative source check: the native heartbeat binary contains no child spawn or
  second state writer;
- operator readback: status names heartbeat owner, Idunn lifecycle owner, last
  completed pulse, next due pulse, active turn, and private-state seal.

## Subsequent Audits

1. Narrow the remaining `query_epiphany_local_verse_context` consumers. Daemon
   and operator commands should load only the typed families they own or
   project.
2. Re-audit Persona Projector -> Persona -> Mind Interpreter against current
   Void behavior and public permission boundaries.
3. Make Aquarium lower the organ graph, contract catalog, and per-turn receipt
   chains from CultMesh/Eve instead of hard-coded routes.
4. Audit non-repo-work actuators and legacy state families for bypasses around
   Substrate Gate, Hands, Soul, or the canonical Mind transaction.
5. Finish Codex starvation: retain only authenticated model transport and
   explicitly quarantined compatibility edges.

## Perfect Machine Target

```text
Self routes.
Substrate Gate grants substrate access.
Eyes certifies inspected evidence.
Imagination projects possible futures and lived context.
Persona speaks as a person.
Hands changes the world.
Soul verifies consequence and invariant.
Modeling updates the machine model from verified consequence.
Continuity preserves coherence across sleep, interruption, and rupture.
Mind admits durable state.
Idunn keeps the body alive.
CultMesh/CultNet carry typed signal.
CultCache preserves typed memory.
Eve/Aquarium make the organism inspectable without becoming its truth.
```
