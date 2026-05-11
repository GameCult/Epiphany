# Schemas

This folder is the canonical paperwork shrine for Epiphany's shared state
contracts.

If a trait name, dossier field, birth-time projection rule, or heartbeat
surface matters enough to steer the machine, it should have a receipt here
instead of living only in one Rust struct, one stale memory store, or one
developer's damp recollection.

## Canonical Surfaces

- [ghostlight.agent-state.schema.json](./ghostlight.agent-state.schema.json):
  wire-shape JSON Schema for Ghostlight-shaped agent state, now canonically
  owned by Epiphany.
- [canonical-agent-state-schema.md](./canonical-agent-state-schema.md):
  human-facing explanation of how Epiphany uses the Ghostlight agent-state
  shape.
- [agent-state-variable-glossary.md](./agent-state-variable-glossary.md):
  full Ghostlight-family glossary plus current Epiphany role-lattice receipts.
- [dossier-profiles.md](./dossier-profiles.md):
  explicit split between lean Epiphany work-organ dossiers and embodied
  Ghostlight/Face personalities.
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

- canonical agent-state family names
- standing role trait names
- heartbeat store shape or pacing semantics
- repo-personality birth-time projection math or routing
- schema version identifiers

If a change lands in code without a matching receipt here, assume the machine
has started whispering to itself in the walls again.
