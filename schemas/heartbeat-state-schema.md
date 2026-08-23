# Heartbeat State Schema

Epiphany's heartbeat store is scheduling physiology. It decides when an
already-existing typed obligation may receive a turn; it does not decide what
the organism believes or invent cognition to fill idle time.

The executable contract lives in
`epiphany-core/src/heartbeat_state/heartbeat_documents.rs` and the scheduling
law lives in `heartbeat_state.rs` and `heartbeat_store.rs`.

## Store identity

- document type: `epiphany.agent_heartbeat`
- schema version: `epiphany.agent_heartbeat.v1`
- key: `default`

The singleton carries Resident Self scheduler readiness, one pending turn, and
bounded scheduling history. Persona social state is not part of this document.
Pending mentions, immutable turn requests, terminal receipts, quarantine
records, and retention state are keyed CultCache documents owned by Persona.

## Scheduling invariants

- An existing pending turn cannot be scheduled again.
- Cooldown begins after terminal completion, not at launch.
- Resident Self pressure may wake the coordinator; heartbeat does not create
  that pressure.
- The swarm brake prevents new Resident Self cognition while allowing terminal
  acknowledgement and recovery physiology to settle.
- When no typed obligation exists, the scheduler sleeps.

Heartbeat owns no personality, mood, appraisal, rumination, dreaming, memory
graph, utterance vector, or generic self-modification surface. Durable Persona
memory and social interpretation are keyed Mind documents admitted through the
Persona decision context. Persona derives its own current turn from pending
keyed social documents; heartbeat cannot launch, block, or terminalize it. Work
for Modeling, Eyes, Hands, Soul, and Imagination comes from the pure
current-work projection over typed state obligations.

JSON emitted by the heartbeat CLI is an operator artifact only. CultCache
documents remain authority.
