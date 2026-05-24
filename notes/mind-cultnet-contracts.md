# Mind CultNet Contracts

Objective: make Mind the persistent state guardian. The state is the Mind; every
sub-agent output is a thought or proposal until Mind admits it into durable
state.

Body is the neighboring gate. Body decides whether an organ may touch the repo;
Mind decides whether the resulting thought, evidence, or proposal mutates
persistent state. Do not collapse these into one throne. That is how the machine
starts lying with excellent posture.

## Authority Map

- Owner: Mind owns the decision to mutate persistent Epiphany state.
- Inputs: typed worker results, Face Interpreter intents, current state context,
  runtime/job provenance, Verse ingress receipts, operator-approved intents, and
  relevant verification evidence.
- Outputs: Mind gateway reviews, state commit receipts, state rejection
  receipts, and Verse adoption receipts.
- Derived state: role findings, reorient findings, Face side-effect intents,
  public dreams, local-area findings, `statePatch`, `selfPatch`, evidence,
  scratch, checkpoints, objectives, and graph edits are proposal-only until Mind
  accepts them.
- Forbidden writers: role acceptance, reorient acceptance, Face Interpreter,
  public Verse ingress, local-area Verse ingress, raw worker result ingestion,
  runtime job completion, and compatibility JSON-RPC routes must not directly
  decide durable state mutation.
- Shared path: every durable state effect goes through a Mind state-effect
  proposal and receives a Mind review plus commit or rejection receipt.
- Deletion line: old contract language that said "coordinator accepts memory
  mutations" is demoted. Coordinator may schedule, route, and carry intent, but
  Mind owns persistent-state admission.

## Verse Boundary

CultMesh simplifies the boundary by making Verse placement part of the contract:

- `epiphany-internal`: private thoughts, state-effect proposals, Mind reviews,
  commit receipts, rejection receipts, runtime facts, heartbeat state, role
  dossiers, and other private organ state.
- `gamecult-local`: trusted GameCult operator-safe sharing, non-secret status,
  reviewable findings, and receipts. It does not carry raw worker thought,
  private memory, or direct state mutation authority.
- `epiphany-global`: public thought weather: dreams, questions, hypotheses,
  Face posts, and adoption receipts. Nothing from this Verse mutates local
  memory, planning, doctrine, governance, or project truth without a reviewed
  local adoption receipt.

## Contract Families

- `epiphany.mind.thought`: sub-agent output submitted as thought, not durable
  state authority.
- `epiphany.mind.state_effect_proposal`: proposed durable mutation. Mind
  accepts, refuses, or holds it.
- `epiphany.mind.gateway_review`: durable receipt explaining the Mind route.
- `epiphany.mind.state_commit_receipt`: proof that Mind admitted an effect into
  persistent state.
- `epiphany.mind.state_rejection_receipt`: proof that Mind refused or held an
  effect without mutating state.
- `epiphany.mind.verse_adoption_receipt`: local adoption receipt for public or
  foreign Verse material.

The current native surfaces are in `epiphany-core::mind_gateway` and the
Verse-scoped policy documents are in `epiphany-core::cultmesh_integration`.
