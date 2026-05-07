# Heartbeat State Schema

Epiphany's heartbeat store is the typed initiative and routine-state organ for
the swarm.

The executable truth lives in
[heartbeat_state.rs](/E:/Projects/EpiphanyAgent/epiphany-core/src/heartbeat_state.rs).

## Store Identity

- document type: `epiphany.agent_heartbeat`
- schema version: `epiphany.agent_heartbeat.v0`
- key: `default`

Related emitted schemas:

- `ghostlight.initiative_schedule.v0`
- `epiphany.void_routine.v0`
- `epiphany.agent_heartbeat_status.v0`

## Top-Level Fields

The heartbeat store currently carries:

- target heartbeat rate
- scene clock
- selection policy
- pacing policy
- participants
- history
- optional routine/sleep surfaces:
  - `sleep_cycle`
  - `memory_resonance`
  - `incubation`
  - `thought_lanes`
  - `bridge`
  - `candidate_interventions`
  - `appraisals`
  - `reactions`

## Participant Shape

Each participant tracks:

- `agent_id`
- `role_id`
- `display_name`
- `arena`
- `participant_kind`
- `initiative_speed`
- `next_ready_at`
- `reaction_bias`
- `interrupt_threshold`
- `current_load`
- `status`
- `constraints`
- last action metadata
- optional `pending_turn`

Maintenance participants are the standing Epiphany organs. Scene participants
reuse the same timing law for Ghostlight-style character turns.

## Key Policy

The important semantics are:

- cooldown starts after turn completion, not launch
- a participant with a pending turn should not be woken again
- calm systems can slow toward sleep
- urgent systems can raise tempo and concurrency
- idle turns are for rumination, not fake work
- sleep/dream passes are the intended distillation window

## Relation To Agent Dossiers

Heartbeat is physiology, not identity.

Role dossiers describe what an organ is. Heartbeat describes how often it gets
the floor, how reactive it is, and how current pressure bends that timing.

Birth-time repo personality may seed heartbeat timing once. After that, routine
state, appraisal, mood, continuity pressure, and live work should carry the
motion.
