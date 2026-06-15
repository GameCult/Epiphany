# Canonical Agent State Schema

Epiphany separates three state layers:

- `OrganState`: lean, function-shaped state for resident work organs.
- `PersonaState`: full Persona/character kernel for public person-shaped agents.
- `SwarmState`: scheduler and coordination physiology across organs and
  Personas.

The current sub-agent/substrate/protocol split is documented in
[epiphany-anatomy.md](../notes/epiphany-anatomy.md). In particular, Body,
Mind, Continuity, and Substrate Gate are not role-memory agents.

The executable shape lives in
[agent_memory.rs](/E:/Projects/EpiphanyAgent/epiphany-core/src/agent_memory.rs).
The local work-organ store still uses the `EpiphanyAgentMemoryEntry` Rust
shape, but the full Ghostlight-style state is no longer required for every
organ. The portable light organ contract is
[`epiphany.work_organ_state.v0`](./cultnet/epiphany.work_organ_state.v0.schema.json).
The portable person-state contract is
[`gamecult.persona_state.v0`](./cultnet/gamecult.persona_state.v0.schema.json).
Epiphany Persona projects its local organ-state record into the portable Persona
contract with:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-agent-memory-store -- project-persona --store .\state\agents.msgpack --role-id Persona
```

## Objective

The point of this schema is not theatrical anthropology. It is to give each
organ enough durable state to do its job without smuggling project truth into
the wrong compartment or making every lane pretend to be a public person.

Work-organ state stores:

- who an organ is
- what authority boundary it must not cross
- what it is trying to protect
- what it remembers
- what inputs, outputs, constraints, and receipts steer the current turn
- how it is currently activated

It does not store affect, social bonds, status reads, or the whole project map
in personality memory. That way lies a small cult of confused blobs.

## Stored Shape

Each record in `state/agents.msgpack` is an
`EpiphanyAgentMemoryEntry` keyed by role id:

- `coordinator` -> `epiphany.self`
- `Persona` -> `epiphany.Persona`
- `imagination` -> `epiphany.imagination`
- `research` -> `epiphany.eyes`
- `modeling` -> `epiphany.modeling`
- `implementation` -> `epiphany.hands`
- `verification` -> `epiphany.soul`

There is intentionally no `reorientation` / `epiphany.continuity` organ-state record.
Reorientation is a bounded Continuity worker/procedure, and Continuity records
typed protocol receipts rather than self-memory.

The local top-level shape is:

- `schema_version`
- `role_id`
- `world`
- `agent`
- `relationships`
- `events`
- `scenes`

The local `agent` bundle contains:

- `agent_id`
- `identity`
- `canonical_state`
- `goals`
- `memories`
- `perceived_state_overlays`

## Canonical State Families

Epiphany currently uses six canonical trait families plus values:

- `underlying_organization`
- `stable_dispositions`
- `behavioral_dimensions`
- `presentation_strategy`
- `voice_style`
- `situational_state`
- `values`

Each scalar-like trait vector is:

```json
{
  "mean": 0.5,
  "plasticity": 0.5,
  "current_activation": 0.5
}
```

Mean is baseline tendency. Plasticity is how easily the variable shifts.
Current activation is how hot it is right now.

## Epiphany-Specific Policy

Epiphany treats local role memory as two explicit profiles:

1. `work_organ`
2. `persona`

See [organ-state-profiles.md](./organ-state-profiles.md).

The important distinction is:

- Self, Imagination, Eyes, Modeling, Hands, and Soul want a lean role
  lattice with room to grow through memory, rumination, and distillation
- Epiphany Persona, VoidBot repo Personas, and Ghostlight-style characters want the
  denser, more fallible, relationship-heavy `Persona` surface
- Heartbeat/swarm state owns scheduling, initiative, cooldown, active-turn
  freeze, and sleep/rumination physiology

So the local store remains compatible with existing role memory, but the
authority split is explicit: work-organ state is not Persona state.

Epiphany does **not** require every standing sub-agent to populate the full old
Ghostlight trait inventory. That would be fake precision. The dense personality
families, affect, relationship pressure, perceived overlays, and social-read
machinery belong to `persona`.

## Mutation Boundaries

Normal reviewed `selfPatch` writes may mutate only:

- memories
- goals
- values
- private notes

They must not rewrite canonical trait bundles directly.

Canonical trait bundles currently change through two sanctioned paths:

1. initial role-shell provisioning
2. repo-personality birth-time seeding through `accept-init`

That birth-time route is documented in
[repo-personality-birth-projection.md](./repo-personality-birth-projection.md).

Persona state is expected to evolve through richer event, relationship,
appraisal, affect, and distillation flows. The current narrow `selfPatch`
contract is enough for work-organ growth, but it is not the end of the story
for portable Personas.

Portable Persona documents must carry provenance and presentation metadata.
`provenance` says which source system/document produced the state, when it was
updated/exported, and whether the document is canonical, a projection, or an
import. Timestamps use JSON Schema `date-time` format. `presentation` is
required for public Personas and carries the public surface: voice summary,
optional avatar/pronouns/renderer, home context or jurisdiction, and public
handles.

The generic forward-pressure field is `candidateActions`. VoidBot-flavored
`candidateInterventions` may exist under `voidbotProjection`, but it is a
projection of the generic action surface rather than the shared contract's
authority. Likewise, `anchoredThought.extensions` may preserve source-specific
data, but extension fields are non-authoritative unless a consumer explicitly
opts into that source contract.

Persona affect uses typed subshapes where the semantics matter: social bonds
carry subject/object/kind/trust/tension, status reads carry target/kind and
confidence, and doctrine stances carry principle/stance/action implication.
Needs remain anchored thoughts because they are closer to pressure records than
relationship facts.

Every enum that allows `custom` has a companion custom-label field, so custom
does not become a stringly-typed category leak. `candidateActions.actions` are
typed candidate-action records with action type, target, optional delivery
target, readiness, risk level, urgency, confidence, evidence, and expiry. They
may cite anchored thoughts as evidence, but they are not generic thoughts
pretending to be actions. `privateNotes` are raw strings for v0 interchange
only and must not become portable state authority without being promoted into
typed private-note records.

## Project Truth Versus Self Truth

These organ-state records are not the same thing as thread state.

Project truth belongs in typed Epiphany state such as:

- maps
- checkpoints
- planning captures
- coordinator results
- evidence ledgers
- runtime-spine receipts

Organ-state records hold role-local durable judgment, not the whole machine's
current world model.

## Related Contracts

- [agent-state-variable-glossary.md](./agent-state-variable-glossary.md)
- [organ-state-profiles.md](./organ-state-profiles.md)
- [heartbeat-state-schema.md](./heartbeat-state-schema.md)
- [repo-personality-birth-projection.md](./repo-personality-birth-projection.md)
