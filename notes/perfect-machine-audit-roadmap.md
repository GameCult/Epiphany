# Perfect Machine Audit Roadmap

Date: 2026-05-27

Objective: audit Epiphany's current organ shape and map the path from named
contracts to the Perfect Machine: a coherent organism where repository access,
evidence, projection, public speech, action, verification, continuity, and
persistent state are separately owned but mutually aware.

## Current Shape

Epiphany has the first typed skeleton for the organ boundaries:

- `Mind`: persistent state guardian. `epiphany-core::mind_gateway` names
  thought, state-effect proposal, gateway-review, commit/rejection, and Verse
  adoption contracts.
- `Body`: repository/substrate access guardian. `epiphany-core::body_gateway`
  names repo access requests, reviews, grant/refusal receipts, snapshots, and
  mutation receipts.
- `Eyes`: evidence ingress guardian. `epiphany-core::eyes_gateway` names
  evidence requests, reviews, source lookup receipts, evidence packets, and
  refusals.
- `Imagination`: projection/future-shape organ. For Face, it now owns the
  Projector prompt boundary in `epiphany-core::face_turn`.
- `Face`: public person-shaped surface. It receives projected context and writes
  natural prose only.
- `Hands`: action organ by doctrine, but not yet backed by a dedicated CultNet
  contract family parallel to Body/Eyes/Mind.
- `Soul`: verification organ by doctrine, but not yet backed by a dedicated
  CultNet contract family parallel to Eyes.
- `Life`: continuity organ by doctrine, heartbeat, reorientation, and handoff
  surfaces, but not yet a single typed contract family for sleep/compaction and
  recovery.
- `Self`: coordinator/routing organ exists through coordinator policy, role
  board, CRRC, and view surfaces, but it is still partly embedded in bridge and
  Codex-compatible routes.

The dependency matrix is explicit in `epiphany-core::organ_dependencies`: every
standing organ depends on every other standing organ. This is correct, but it
is currently descriptive. It does not yet force every launch packet, prompt, or
runtime receipt to carry dependency context.

## Void Face Lessons

Void's current Face work is the best reference implementation for the Face
organ. The important lesson is not "make the prompt longer." The lesson is
parent-organ ownership:

- Parent `Imagination` projects typed state into lived character-facing prose
  before the Face sees it.
- The Face reads raw room/context evidence directly and writes natural prose as
  a person.
- Parent `Mind` interprets the Face output into memory, affect, social reads,
  mood, agency pressure, speech, retry, or drop.
- The deterministic assembler creates compact packets, raw transcripts, repo
  activity, pronoun guidance, channel policy, and media awareness. It must not
  turn those into character prose itself.

Relevant Void references:

- `VoidBot:prompts/repo-face-state-projector.prompt.md`: names the projector as
  the Face's parent Imagination organ and forbids schema slurry.
- `VoidBot:prompts/repo-face-turn.prompt.md`: keeps the Face turn as the
  speaking surface, with recent repo activity before conversation transcript and
  deterministic policy sections outside projected state.
- `VoidBot:prompts/repo-face-turn-interpreter.prompt.md`: names the
  Interpreter as parent Mind and requires route/retry/drop plus structured
  side effects.
- `VoidBot:scripts/run-repo-face-heartbeats.ts`: runs projection as
  `organ: "imagination"` in read-only mode, rejects leaky memory surfaces, keeps
  raw transcript evidence oldest-to-newest, includes visible chronology, exports
  recent repo activity read-only, and keeps pronoun guidance deterministic so
  Imagination cannot accidentally omit it.

Audit implication for Epiphany: `face_turn.rs` has the right first boundary but
is still too thin. It lacks deterministic human pronoun guidance, channel label
resolution, visible chronology, media attachment awareness, retry/drop
semantics, projection model receipts, and explicit leakage rejection as strong
as Void's `rejectLeakyMemorySurface`.

## Audit Findings

### 1. Contract Names Exist Before Contract Execution

Mind, Body, and Eyes have type constants, CultMesh policy documents, and
runtime-spine advertisement. They are not yet the mandatory runtime path.
Workers can still be launched and accepted through older bridge/service flows
without every substrate touch, evidence claim, or state mutation passing through
a real receipt chain.

Required correction: turn contract families into executable gates.

### 2. Body/Eyes/Mind Are Correctly Split But Not Yet Chained

The desired chain is:

```text
Need fact or action
  -> Self routes
  -> Body grants scoped substrate access
  -> Eyes packages inspected evidence when truth is needed
  -> Imagination projects options or scenes when future/personhood is needed
  -> Hands executes bounded action when mutation is needed
  -> Soul verifies result/invariant
  -> Mind admits durable state
  -> Life preserves continuity
```

Today the chain is documented, not structurally unavoidable.

### 3. Hands, Soul, and Life Need First-Class Contracts

The current contract set names the gates around action but not action itself.
That leaves three weak organs:

- Hands needs action intent, execution receipt, patch receipt, command receipt,
  commit/PR receipt, and refusal/rollback receipt contracts.
- Soul needs verification request, invariant check, evidence verdict,
  regression/refusal, and review receipt contracts.
- Life needs compaction request, continuity packet, sleep/distillation,
  recovery, stale-turn repair, and handoff receipt contracts.

Without those, Body/Mind/Eyes become paper gates around action and continuity
paths they do not own.

### 4. Self Still Shares A Throne With Compatibility Plumbing

Coordinator policy is much cleaner, but Codex-compatible route edges still
shape parts of the living workflow. The Perfect Machine needs Self as a typed
router over CultNet/CultMesh contracts, not a half-native coordinator behind a
JSON-RPC organ.

### 5. CultMesh/CultLib Local Dependency Is Broken

`epiphany-core` currently points at:

```toml
cultcache-rs = { path = "../../CultLib/crates/cultcache-rs" }
cultmesh-rs = { path = "../../CultLib/crates/cultmesh-rs" }
cultnet-rs = { path = "../../CultLib/crates/cultnet-rs" }
```

This checkout has `E:\Projects\CultLib` but no `crates` directory, so cargo
checks fail before reaching the new code. This is a Body-level substrate wound:
the Rust dependency body described by the map does not exist at the path the
machine uses.

Required correction: decide whether CultLib should restore `crates/` or
Epiphany should depend on the sibling standalone Rust repos. Do not make that
choice as a drive-by patch.

### 6. Face Prompting Is Directionally Correct But Behind Void

Epiphany's Face loop now says `Imagination Projector -> Face -> Mind
Interpreter`, but Void has the stronger living practice:

- state projector is model-owned, not deterministic prose assembly
- projection runs read-only with model/output receipts
- leaked schema/prompt construction language is rejected
- raw transcript and chronology stay raw and oldest-to-newest
- deterministic pronoun guidance sits outside projection
- channel labels are resolved through Face permissions before posting
- interpreter can retry one Face pass when correction uptake or doctrine fails

Epiphany should port that shape, not the exact TypeScript machinery.

## Path To The Perfect Machine

### Phase 0: Repair The Rust Body

Goal: make the existing contracts buildable again.

- Audit the CultLib/CultCache/CultNet/CultMesh repo layout.
- Choose the canonical Rust dependency source.
- Update dependency paths or restore the expected `CultLib/crates` layout.
- Run the focused tests that are currently blocked:
  - `cargo test --manifest-path .\epiphany-core\Cargo.toml --lib face_turn`
  - `cargo test --manifest-path .\epiphany-core\Cargo.toml --lib mind_gateway`
  - `cargo test --manifest-path .\epiphany-core\Cargo.toml --lib body_gateway`
  - `cargo test --manifest-path .\epiphany-core\Cargo.toml --lib eyes_gateway`
  - `cargo test --manifest-path .\epiphany-core\Cargo.toml --lib cultmesh_integration`
  - `cargo test --manifest-path .\epiphany-core\Cargo.toml --lib runtime_spine::tests::runtime_spine_emits_cultnet_hello_frame`

Definition of done: contract code compiles against the intended Rust substrate
without local path superstition.

### Phase 1: Make The Gates Executable

Goal: no worker output, repo touch, or evidence claim bypasses the appropriate
organ.

- Implement Body access request/review/grant/refusal documents in CultCache.
- Route retrieval, indexing, file edit, shell command, Rider, and Unity bridge
  operations through Body access receipts.
- Implement Eyes evidence request/review/packet/refusal documents in CultCache.
- Require Eyes packets for claims promoted into Mind state proposals when the
  claim depends on inspected source.
- Convert existing role/reorient acceptance to create Mind state-effect
  proposals before state mutation, not merely a review attached to the old path.

Definition of done: the old direct paths cannot produce repo mutation,
evidence promotion, or durable state mutation without the corresponding typed
receipt.

### Phase 2: Give Hands, Soul, And Life Their Own Contracts

Goal: action, verification, and continuity stop hiding behind neighbors.

- Hands contracts:
  - `epiphany.hands.action_intent`
  - `epiphany.hands.command_receipt`
  - `epiphany.hands.patch_receipt`
  - `epiphany.hands.commit_receipt`
  - `epiphany.hands.rollback_receipt`
- Soul contracts:
  - `epiphany.soul.verification_request`
  - `epiphany.soul.invariant_check`
  - `epiphany.soul.verdict_receipt`
  - `epiphany.soul.regression_receipt`
- Life contracts:
  - `epiphany.life.continuity_packet`
  - `epiphany.life.compaction_checkpoint`
  - `epiphany.life.sleep_distillation`
  - `epiphany.life.recovery_receipt`
  - `epiphany.life.stale_turn_repair`

Definition of done: action, verification, and continuity have the same typed
contract dignity as Mind/Body/Eyes.

### Phase 3: Port Void's Face Prompting Shape Properly

Goal: Face becomes a living public organ without stealing authority.

- Replace `FaceProjectorInput` with an Imagination projector packet that carries
  typed state, affect, memory, social topology, repo activity, semantic
  attractors, curiosity hints, and dependency pressure.
- Add deterministic sections outside projected state:
  - raw transcript, oldest-to-newest
  - visible chronology across rooms/surfaces
  - human pronoun guidance
  - channel label/permission policy
  - media attachment awareness
  - recent home-repo activity from Body-gated reads
- Make projection model-owned and read-only, with model output receipts.
- Strengthen projection leakage rejection: no schema words, repo paths, prompt
  construction language, grants/jurisdictions slurry, or direct action syntax.
- Upgrade Mind Interpreter output from simple effect blocks to:
  - correction check
  - doctrine/coherence check
  - decision: route/retry/drop
  - structured memory/affect/social/speech effects
  - retry limit
- Ensure public speech resolves Face-local channel labels through permissions
  before delivery.

Definition of done: Face can be prompted by Aquarium/Discord/CultNet and the
public result is natural, contextual, permission-aware, retryable, and unable to
mutate state or post without parent routing.

### Phase 4: Move Self Fully Onto CultNet/CultMesh

Goal: Self routes organ work through typed contracts, not legacy JSON-RPC
comfort tunnels.

- Add Self routing contracts for choosing next organ, required dependencies,
  and allowed authority.
- Make role launch packets declare required Body/Eyes/Hands/Soul/Life/Mind
  receipts.
- Make Aquarium read the contract catalog and available receipts instead of
  hard-coding the route zoo.
- Starve `codex_message_processor` down to Codex auth/model transport and
  compatibility emission only.

Definition of done: a local run can be inspected as a chain of CultNet/CultMesh
documents from operator request to final response.

### Phase 5: Runtime Physiology

Goal: the machine runs as an organism.

- Heartbeat scheduler wakes organs by initiative, pending pressure, and
  completion-gated cooldown.
- Active turns freeze initiative until receipt completion.
- Sleep distills rumination through Life and Mind.
- Stale active turns get Life recovery receipts.
- Public/global Verse material enters as thought weather, then Eyes/Mind review
  before adoption.

Definition of done: unattended operation can pause, recover, explain current
pressure, and preserve memory without transcript worship or hidden loops.

### Phase 6: Aquarium As Inspectable Nervous System

Goal: make the organism visible without making the UI a second truth.

- Display organ dependency graph.
- Display contract catalog by organ.
- Display receipt chains per turn.
- Display Face prompt packet boundaries: deterministic evidence, Imagination
  projection, Face natural turn, Mind interpretation.
- Display Body grants, Eyes packets, Hands actions, Soul verdicts, Life
  continuity, and Mind state commits.

Definition of done: a human can ask "why did you touch that file / say that /
remember that?" and see the typed path.

## Perfect Machine Target

The Perfect Machine is not more prompts and not more bureaucracy. It is a
coherent authority graph:

```text
Self routes.
Body grants substrate access.
Eyes certifies looked-at evidence.
Imagination projects possible scenes and futures.
Face speaks as a person.
Hands changes the world.
Soul verifies promises and invariants.
Life preserves continuity across rupture.
Mind admits durable state.
CultMesh carries the local typed Verse surfaces.
CultNet carries wire contracts.
CultCache preserves the typed documents.
Codex remains only model/auth transport until it can be replaced or minimized.
```

When this is real, no organ can steal the throne. The machine can act, speak,
remember, doubt, recover, and be inspected without turning connection into mush.
