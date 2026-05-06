# VoidBot Routine Adoption Plan

VoidBot has grown the reference agent routine Epiphany should steal without
cosplaying as a Discord moderator. The useful organs are:

- heartbeat turns that choose work, rumination, or sleep/consolidation
- cooldown that starts after a heartbeat turn completes, not when it launches
- nap windows for memory consolidation, pruning, dream residue, and incubation
- canonical MessagePack state through a CultCache-style typed document store
- CultCache-native inspection/debug tooling rather than long-lived JSON shadow state
  deliberately
- code-owned schemas instead of schema drift hiding in loose files
- semantic memory vectors in Qdrant with source hashes and compact dimensions
- memory resonance, incubation queues, analytic/associative thought lanes, and a
  bridge that decides speech, draft, hold, or silence

## Target Shape

Epiphany should treat this as a shared agent routine substrate, not as another
specialist lane. The substrate owns scheduling, persistence, and memory
maintenance; the coordinator owns authority.

```text
coordinator pressure / user objective / swarm messages
  -> heartbeat scheduler
  -> work turn | rumination turn | sleep turn
  -> typed state read through CultCache-compatible store
  -> optional Qdrant memory recall / resonance
  -> bounded result, selfPatch, statePatch, dream, or callback
  -> completion receipt starts cooldown
  -> canonical MessagePack write + typed inspection/debug views
```

## Scheduler Contract

Heartbeat state keeps:

- participant identity, role, speed, load, status, readiness, reaction bias, and
  pending turn
- pacing policy with work recovery, idle recovery, sleep multiplier, and
  completion-gated cooldown
- sleep cycle state: enabled, cadence, nap duration, current nap, next nap,
  dream count, active dream themes, and last distillation summary
- history of work, rumination, sleep, completion, and refused-repeat events

Turn kinds:

- `work`: coordinator-approved lane work, specialist result, Face surface, or
  implementation continuation.
- `rumination`: role-local memory hygiene, selfPatch, stale habit detection, or
  low-pressure thought incubation.
- `sleep`: inward maintenance. The agent should distill/prune memory, strengthen
  resonance clusters, update incubation, write or refresh dreams, and speak only
  through Face if something genuinely needs to surface.

No lane may receive another heartbeat while `pending_turn.status == running`.
The completion receipt is the only normal path that starts cooldown. This is the
part that keeps the heart from turning into a tiny manager with a stopwatch and
no shame.

## Persistence Contract

Epiphany needs a Rust version of the CultCache contract because the app-server
and `epiphany-core` are Rust and should not route core state through a Node
helper just to persist its own mind.

Minimum Rust surface:

- `DocumentType<T>`: type id plus serde/schema metadata.
- `CultCacheEnvelope`: `{ key, type, payload, stored_at }`.
- `BackingStore`: `pull_all`, `push`, `delete`, optional `push_all`.
- `SingleFileMessagePackBackingStore`: atomic whole-file MessagePack store with
  external lock expectation.
- `CultCache`: register types, load stores, validate payloads through typed
  serde structs, get/get_required/get_all/put/update/delete/snapshot.
- Typed inspection/debug views should come from CultLib/CultCache tooling so
  all wire-compatible runtimes share the same schema and display assumptions.
  Repo-local JSON projections are temporary compatibility shims only.

The schema should live in annotated Rust structs using serde plus generated JSON
Schema where useful. MessagePack is canonical. JSON is the window, not the
house.

Initial document types:

- `epiphany.thread_state`
- `epiphany.role_agent_state`
- `epiphany.heartbeat_state`
- `epiphany.face_state`
- `epiphany.swarm_message`
- later: planning, graph checkpoint, memory resonance, incubation, dreams

## Vector Memory Contract

Epiphany already has Qdrant/Ollama retrieval for workspace source. Extend that
pattern to agent memory:

- collection namespace: `epiphany_memory_<instance_id>`
- point id: stable memory id or hash of document type/key/path
- vector model: default `qwen3-embedding:0.6b`
- payload: agent id, role id, memory kind, source hash, schema version, summary,
  salience, confidence, updated time, source refs
- canonical MessagePack state stores vector metadata and optionally compact
  inline vectors; Qdrant stores the retrieval vector
- CultCache inspection views may strip vector values while preserving metadata

Memory resonance is not proof. It is "these things rhyme" evidence for
rumination, Face surfacing, and sleep consolidation.

## Migration Slices

1. **Contract extraction**
   - Add Rust CultCache-compatible module in `epiphany-core`.
   - Add fixtures proving MessagePack round-trip, typed serde/schema validation,
     and CultCache inspection compatibility.

2. **Heartbeat state migration**
   - Move `state/agent-heartbeats.json` into a typed MessagePack document, then
     retire JSON once the heartbeat tool reads and writes the typed store
     directly.
   - Add `sleep_cycle`, `memory_resonance`, and `incubation` fields.
   - Keep the current CLI contract stable while the backing store changes.

3. **Role memory migration**
   - Store each Ghostlight-shaped role dossier as `epiphany.role_agent_state`.
   - Use CultLib/CultCache inspection tools for human/debug review instead of
     preserving per-role JSON as a parallel source of truth.
   - Attach semantic vector metadata to episodic, semantic, relationship, goal,
     value, musing, and dream entries.

4. **Memory organ**
   - Add a bounded memory maintenance command that embeds missing memory vectors,
     refreshes Qdrant points, builds resonance edges/clusters, updates
     incubation, and writes dream residue during sleep turns.

5. **Coordinator integration**
   - Coordinator sees heartbeat sleep state, pending turns, resonance summaries,
     and memory maintenance results.
   - Coordinator may route work, accept/refuse selfPatch, or let the swarm sleep.

6. **Aquarium visibility**
   - Expose sleep state, dreams, resonance clusters, incubating thoughts, and
     pending turns as inspectable Aquarium surfaces.

## Non-Goals

- Do not replace CRRC compaction yet.
- Do not make Qdrant canonical truth.
- Do not let sleep turns edit implementation code.
- Do not auto-accept specialist findings because a dream sounded good.
- Do not build a general database shrine before the typed MessagePack seam
  proves it deserves to live.
