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
- repo identities with Face state, Discord roles, pending mentions, jurisdiction,
  proposal authority, and a CTB/Final-Fantasy-style initiative scheduler

## Project-Face Direction

VoidBot is no longer only a moderator with retrieval. It now has the small
working version of the Epiphany pitch: repos grow Faces.

The live pattern is:

- a repo identity has a Discord role and registered allowed channels
- that identity has repo-local Face state, not just base Void personality
- first contact can birth the Face through Epiphany repo terrain/personality
  scouting and reviewable startup artifacts
- role/display-name mentions become pending obligations, not direct hidden jobs
- an initiative scheduler chooses which Face or system agent gets the next turn
- heat can prioritize identities, repos, participant kinds, turns, channels, or
  all participants
- active turns freeze initiative until the owner-Codex job or moderation lock
  completes
- Faces may advocate for repo needs and propose changes, while implementation
  authority still requires the configured human/review path

Epiphany should absorb that as native product direction. The user should talk to
projects: "Aqua, what does AquaSynth need?", "Nibu, what is wrong with this lore
arc?", "Mimir, what seam is drifting?" The project should schedule modeling,
research, verification, memory maintenance, or a Face response from its own
typed state. The human should not have to deliver a complete implementation
brief just to make the system start thinking.

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

Repo Faces use the same law. A Face can be heated because a human mentioned it,
a repo is hot, a stream is live, or a group needs attention; that heat changes
opportunity, not authority. The Face still freezes while thinking, still returns
through typed receipts, and still needs an explicit grant before crossing from
proposal into canonical repo mutation.

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

## Shared Persona-State Contract

Epiphany should not invent its own rival Persona ontology.

The shared cross-runtime public/person-shaped payload contract is now the
GameCult Persona state shape, with Ghostlight and VoidBot as source lineage:

- payload schema id:
  `https://gamecult.dev/cultnet/gamecult.persona_state.v0.schema.json`
- payload schema version:
  `gamecult.persona_state.v0`
- intended CultNet document type:
  `gamecult.persona_state.v0`

That means Epiphany Face, VoidBot repo Faces, and Ghostlight characters can
exchange Persona state without turning every Epiphany work organ into a public
personality simulation.

The shared contract uses `candidateActions` for generic forward pressure.
VoidBot can expose the same list as `voidbotProjection.candidateInterventions`
when repo-Face routine language wants that noun, but the projection does not own
the portable Persona contract. Persona documents also carry provenance and a
public presentation surface so imported state can say where it came from, when
it was exported, whether it is canonical/projection/import data, and how the
public person should be rendered.

Source-specific fields belong in `anchoredThought.extensions`. That bag is
preservation glue, not authority. Consumers may keep it round-tripped, but they
must not let it steer shared Persona behavior unless they explicitly understand
the originating contract.

The affect layer keeps semantic bones where they matter: social bonds name
subject/object/kind/trust/tension, status reads name target/kind/confidence,
and doctrine stances name principle/stance/action implication. Needs remain
anchored thoughts because they behave like pressure records.

The baseline also requires a public `presentation` block, annotates timestamps
as JSON Schema `date-time`, and gives every `custom` enum path a companion
custom-label field. `candidateActions.actions` and `privateNotes` stay simple
for v0 interchange; if either begins carrying routing, readiness, expiry,
provenance, or action authority, promote it into a typed v1 record instead of
letting raw strings or generic thoughts grow a steering wheel.

CultNet is not just ergonomic framing. It also carries CultLib-style auth and
session semantics:

- shared connection key for both peers
- AES-GCM encrypted auth/session payloads
- server-side session-signing secret
- signed verify/reconnect tokens

So the target shape is:

- canonical work-organ state in MessagePack through CultCache-compatible
  storage
- typed CultNet document replication for live exchange
- Persona payloads for public/person-shaped agents

Narrow work-organ views stay derived and local. The shared person-state truth is
Persona-shaped.

Initial document types:

- `epiphany.thread_state`
- `epiphany.agent_memory`
- `epiphany.heartbeat_state`
- `gamecult.persona_state.v0`
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
   - Move `state/agent-heartbeats.json` into a typed MessagePack document.
     Landed: the tracked JSON state file is retired and heartbeat
     init/status/tick/complete now route through the native CultCache store.
   - Add `sleep_cycle`, `memory_resonance`, and `incubation` fields.
   - Keep the current CLI contract stable while the backing store changes.

3. **Organ and Persona memory migration**
   - Store lean Epiphany work-organ memory as `epiphany.agent_memory`.
   - Store public/person-shaped Face state as `gamecult.persona_state.v0`.
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
