# Epiphany Anatomy

This is the canonical anatomy map for Epiphany's organs, substrates, and
protocol surfaces. If older notes disagree, this document wins until the code
or map says otherwise.

## Embodied Sub-Agents

Only thinking lanes get embodiment titles.

- `Self`: coordination, routing, authority boundaries, and review posture.
- `Persona`: public/social expression and person-shaped conversation.
- `Imagination`: futures, plans, projections, objective drafts, and scene
  construction.
- `Eyes`: research, source inspection, retrieval discipline, and evidence
  ingress.
- `Proprioception`: internal model of the Body: architecture, dataflow, seams,
  graph/checkpoint anatomy, and source-grounded self-sensing.
- `Hands`: implementation, commands, patches, commits, rollbacks, and other
  actuators.
- `Soul`: verification, invariants, ethics, falsification, and promise-keeping.

Each embodied sub-agent may have lean work-organ state in `state/agents.msgpack`,
may be represented as a heartbeat participant, and may request reviewed
`selfPatch` changes to its own lane memory. Persona may additionally project or
adopt portable `Persona` state because public personhood is part of its job.

## Not Sub-Agents

These are machinery, substrate, or state surfaces. Do not give them fancy
embodiment titles, role-memory diaries, or Persona state.

- `Body`: the substrate Epiphany acts through and senses: repository, tools,
  runtime, prompts, logs, state files, permissions, interfaces, hosted systems,
  and physical/editor/runtime bridges.
- `Mind`: persistent state and durable steering context: memory, maps, goals,
  doctrine, evidence, state patches, accepted receipts, and state admission.
- `Continuity`: protocol machinery for compaction, sleep, recovery,
  stale-turn repair, reorientation receipts, handoff preservation, and
  "what survived rupture?" accounting.
- `Substrate Gate`: repository/substrate access protocol. It grants or refuses
  scoped access before Eyes reads or Hands mutates.

## Boundary Rules

- Body is sensed and changed; it does not decide.
- Mind admits durable state; it is not a chatty lane.
- Continuity preserves/rebuilds context across rupture; it is protocol
  machinery, not an embodied identity.
- Substrate Gate grants substrate access; it does not inspect evidence, mutate
  files, verify truth, or admit state.
- Reorientation is a bounded Continuity worker/procedure launched from CRRC,
  not an embodied sub-agent.
- `roleAccept` and `selfPatch` are for embodied sub-agents only.
- Continuity state changes should appear as typed continuity packets, recovery
  receipts, compaction checkpoints, stale-turn repairs, or refusals.

## Dependency Rule

Every embodied sub-agent depends on the other embodied sub-agents:

```text
Self, Persona, Imagination, Eyes, Proprioception, Hands, Soul
```

Dependency does not collapse ownership. It means a lane's prompt/context should
feel relevant pressure from the rest of the organism while decisions remain
owned by the correct boundary.

## Contract Map

- Mind contracts: `notes/mind-cultnet-contracts.md`
- Substrate Gate contracts: `notes/substrate-gate-cultnet-contracts.md`
- Eyes contracts: `notes/eyes-cultnet-contracts.md`
- Hands contracts: `notes/hands-cultnet-contracts.md`
- Soul contracts: `notes/soul-cultnet-contracts.md`
- Continuity contracts: `notes/continuity-cultnet-contracts.md`
- Sub-agent dependency contracts: `notes/organ-dependency-contracts.md`
