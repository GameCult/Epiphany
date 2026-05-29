# Schemas

This folder is the canonical paperwork shrine for Epiphany's shared state
contracts.

If a trait name, organ-state field, Persona field, birth-time projection rule, or heartbeat
surface matters enough to steer the machine, it should have a receipt here
instead of living only in one Rust struct, one stale memory store, or one
developer's damp recollection.

## Canonical Surfaces

- [ghostlight.agent-state.schema.json](./ghostlight.agent-state.schema.json):
  source-lineage JSON Schema for dense Ghostlight character state.
- [cultnet/gamecult.persona_state.v0.schema.json](./cultnet/gamecult.persona_state.v0.schema.json):
  portable Persona state contract for Epiphany Face, VoidBot repo Faces, and
  Ghostlight characters. It carries explicit provenance, public presentation
  metadata, generic `candidateActions`, and a non-authoritative extension bag
  for source-specific fields; social bonds, status reads, and doctrine stances
  are typed affect records rather than generic thought blobs.
- [cultnet/epiphany.work_organ_state.v0.schema.json](./cultnet/epiphany.work_organ_state.v0.schema.json):
  light function-shaped state for Epiphany internal work organs.
- [canonical-agent-state-schema.md](./canonical-agent-state-schema.md):
  human-facing explanation of Epiphany's lean work-organ state and Persona split.
- [agent-state-variable-glossary.md](./agent-state-variable-glossary.md):
  full Persona-family glossary plus current Epiphany work-organ lattice receipts.
- [organ-state-profiles.md](./organ-state-profiles.md):
  explicit split between lean Epiphany work-organ state and portable Persona
  state.
- [agent-utterance-state-schema.md](./agent-utterance-state-schema.md):
  derived speech-conditioning subset for Weks, Aquarium, and other utterance
  surfaces; it carries identity, trait vectors, mood, and activation without
  memory records.
- [repo-personality-birth-projection.md](./repo-personality-birth-projection.md):
  deterministic birth-time path from repo terrain to newborn trait lattice and
  heartbeat seeds.
- [heartbeat-state-schema.md](./heartbeat-state-schema.md):
  typed initiative and routine-state contract for the swarm heartbeat organ.
- [cultnet/README.md](./cultnet/README.md):
  published CultNet-facing state, surface, intent, and receipt schemas that
  Aquarium and other runtimes can discover through Epiphany's schema-catalog
  response.

## Source Of Truth

The living implementation is in code:

- [agent_memory.rs](/E:/Projects/EpiphanyAgent/epiphany-core/src/agent_memory.rs)
- [heartbeat_state.rs](/E:/Projects/EpiphanyAgent/epiphany-core/src/heartbeat_state.rs)
- [epiphany-repo-personality.rs](/E:/Projects/EpiphanyAgent/epiphany-core/src/bin/epiphany-repo-personality.rs)

The rule is simple:

- code owns executable truth
- this folder owns human-readable contract receipts
- vendored copies are downstream, not the throne

## Update Discipline

When changing any of the following, update this folder in the same pass:

- canonical organ-state or Persona family names
- standing role trait names
- heartbeat store shape or pacing semantics
- repo-personality birth-time projection math or routing
- schema version identifiers

If a change lands in code without a matching receipt here, assume the machine
has started whispering to itself in the walls again.
