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

The singleton currently carries coordinator/Persona readiness, pending turns,
bounded scheduling history, explicit Persona transport pressure, and retention
bookkeeping. The remaining social fields are the next ownership cut: they must
become keyed Persona documents so social writes do not share the scheduler CAS
identity.

## Scheduling invariants

- An existing pending turn cannot be scheduled again.
- Cooldown begins after terminal completion, not at launch.
- Resident Self pressure may wake the coordinator; heartbeat does not create
  that pressure.
- Persona may wake only from explicit queued social pressure.
- The swarm brake prevents new Persona cognition while allowing terminal
  acknowledgement and recovery physiology to settle.
- When no typed obligation or social pressure exists, the scheduler sleeps.

Heartbeat owns no personality, mood, appraisal, rumination, dreaming, memory
graph, utterance vector, or generic self-modification surface. Durable Persona
memory and social interpretation are keyed Mind documents admitted through the
Persona decision context. Work for Modeling, Eyes, Hands, Soul, and Imagination
comes from the pure current-work projection over typed state obligations.

JSON emitted by the heartbeat CLI is an operator artifact only. CultCache
documents remain authority.
