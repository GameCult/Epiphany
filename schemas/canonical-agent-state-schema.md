# Canonical Agent State Schema

Epiphany uses a Ghostlight-shaped agent dossier as the canonical local memory
surface for each standing organ.

The executable shape lives in
[agent_memory.rs](/E:/Projects/EpiphanyAgent/epiphany-core/src/agent_memory.rs).
The wire contract mirrored here is
[ghostlight.agent-state.schema.json](./ghostlight.agent-state.schema.json).

## Objective

The point of this schema is not theatrical anthropology. It is to give the
machine a stable embodied memory surface that can survive compaction, role
handoff, heartbeat rumination, and birth-time initialization without smuggling
project truth into the wrong compartment.

Epiphany stores:

- who an organ is
- how it tends to behave
- what it is trying to protect
- what it remembers
- how it is currently activated

It does not store the whole project map in personality memory. That way lies a
small cult of confused blobs.

## Stored Shape

Each record in `state/agents.msgpack` is an
`EpiphanyAgentMemoryEntry` keyed by role id:

- `coordinator` -> `epiphany.self`
- `face` -> `epiphany.face`
- `imagination` -> `epiphany.imagination`
- `research` -> `epiphany.eyes`
- `modeling` -> `epiphany.body`
- `implementation` -> `epiphany.hands`
- `verification` -> `epiphany.soul`
- `reorientation` -> `epiphany.life`

The top-level shape is:

- `schema_version`
- `role_id`
- `world`
- `agent`
- `relationships`
- `events`
- `scenes`

The `agent` bundle contains:

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

Epiphany now treats the wire schema as one shared body shape with two explicit
profiles layered over it:

1. `lane_core`
2. `embodied_actor`

See [dossier-profiles.md](./dossier-profiles.md).

The important distinction is:

- most standing Epiphany organs want a lean role lattice with room to grow
  through memory, rumination, and distillation
- Face and Ghostlight-style characters want a denser, more fallible, more
  relationship-heavy personality surface

So the schema stays shared, but the expected population density changes by
profile.

Epiphany does **not** currently require every standing organ to populate the
full old Ghostlight trait inventory. That would be fake precision. But the
full embodied Ghostlight families remain canonical, available, and appropriate
for Face and character-grade agents.

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

Face and future embodied actors are still expected to evolve through richer
event, relationship, appraisal, and distillation flows. The current narrow
`selfPatch` contract is enough for lane-core growth, but it is not the end of
the story for fully embodied personalities.

## Project Truth Versus Self Truth

These dossiers are not the same thing as thread state.

Project truth belongs in typed Epiphany state such as:

- maps
- checkpoints
- planning captures
- coordinator results
- evidence ledgers
- runtime-spine receipts

Dossiers hold role-local durable judgment, not the whole machine's current
world model.

## Related Contracts

- [agent-state-variable-glossary.md](./agent-state-variable-glossary.md)
- [dossier-profiles.md](./dossier-profiles.md)
- [heartbeat-state-schema.md](./heartbeat-state-schema.md)
- [repo-personality-birth-projection.md](./repo-personality-birth-projection.md)
