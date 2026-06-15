# Epiphany Swarm Readiness Plan

This is the direction of travel before live fire.

The local run path proves that the current machine can be started and inspected.
It does not prove the swarm is ready to run unattended. Do not confuse a working
starter switch with a clean engine bay. That mistake has already been punished
elsewhere, and VoidBot has kindly provided the scorch marks.

## Objective

Get Epiphany ready for cautious live operation over the next few days without
building architectural Jenga:

- typed CultCache documents remain the state authority
- CultNet is the native wire for Epiphany-owned subsystems
- Codex remains the retained auth/model transport compatibility organ
- Aquarium and Persona are reflectors/mouths, not hidden sources of truth
- heartbeats create opportunities, not authority
- swarm speech and action stay gated until scheduling, memory, repetition, and
  review boundaries are coherent

## VoidBot Lessons To Carry Forward

VoidBot's ordinary spine is not the rotten part. Its durable lessons are:

- Discord ingestion, worker jobs, provider lanes, RAG, Postgres, Qdrant, and
  typed CultCache self-state can be conceptually sound when each owns one job.
- The unstable organ was the repo Persona swarm loop: scheduling, prompt policy,
  identity, public speech, governance, dispatch, proposal behavior, and
  repetition control packed into one prompt-and-parser machine.
- A repo-controlled pause flag is a real brake. If a swarm subsystem is under
  teardown, runners must fail closed instead of routing around the pause.
- One CTB-style initiative scheduler beats one task per agent. The wall-clock
  task supplies pulses; typed initiative state chooses turns.
- Heat changes recovery pressure. It must not fast-forward time, bypass active
  turn freeze, or create overlapping thoughts.
- Cooldown starts after completion, not after queueing.
- Stale active turns need explicit recovery receipts, or one dead worker claim
  freezes a Persona forever.
- Public speech needs parent-side eligibility checks. Prompts alone cannot
  prevent repeated openings, stale topics, scheduler labels leaking into prose,
  or work requests masquerading as banter.
- Memory must have phases: short-term residue, incubation, durable memory,
  revision, retirement, crystallization, and sleep-owned maintenance.
- Agents propose typed operations. State services validate, normalize, dedupe,
  and write. Whole-state JSON editing is not a mutation path.
- Qdrant/vector recall is a rebuildable resonance cache, not canonical memory.
- Persona affect is state, not vibes: needs, social bonds, status reads, and mood
  dimensions must survive typed projection and MCP/CultNet inspection.

## Current Mechanism

Epiphany currently has the right ingredients, but not yet the final live swarm
shape:

- `tools/epiphany_local_run.ps1` can build and smoke the compatibility shell.
- `epiphany-runtime-spine` owns typed runtime identity/session/job/result/event
  documents and advertises CultNet contracts.
- heartbeat state owns role dossiers, initiative, heat, active-turn freeze,
  sleep, rumination, appraisals, and derived reactions.
- role memory accepts reviewed `selfPatch` petitions into typed Ghostlight-like
  dossiers.
- coordinator/CRRC/role/status surfaces exist, but the practical local runner
  still goes through Codex app-server JSON-RPC for operator reads/actions.
- Persona, Discord, Rider, Unity, Void memory, and repo birth bridges are typed
  surfaces or artifacts, but they are not yet an integrated live swarm loop.

The machine is pointed toward live use. It is not cleared for unattended swarm
operation.

## Invariants

- One state authority per document kind.
- One scheduler for standing initiative.
- One explicit brake for swarm operation.
- No lane receives a new heartbeat while its previous turn is running.
- No model output rewrites durable state directly.
- No public speech without typed eligibility and receipt checks.
- No cross-repo rummaging; swarm needs move coordinator-to-coordinator through
  visible typed messages and callbacks.
- No hidden automatic semantic acceptance.
- No prompt-only governance for things that need state, receipts, or gates.
- No broad live scheduler until the local runner can show the typed surfaces
  that explain what it would do and why.

## Readiness Gate

Before live fire, Epiphany needs these gates closed:

1. **Operator Run Path**
   - Keep `tools/epiphany_local_run.ps1` as the human entrypoint for now.
   - It must keep printing a concise verdict.
   - It must keep artifacts operator-safe by default.
   - It must make Codex app-server dependency obvious as compatibility, not
     architecture.

2. **Typed Swarm Brake**
   - Add an Epiphany-owned pause document for swarm/heartbeat/live Persona
     operation.
   - Runners must fail closed when the document is missing, malformed, or
     paused for a teardown reason, according to the surface being protected.
   - The brake must be inspectable through Aquarium/status.

3. **Initiative Scheduler Boundary**
   - Keep one heartbeat scheduler for lanes/Personas/system organs.
   - Heat is a multiplier on recovery pressure, not a time machine.
   - Active-turn freeze and completion-gated cooldown are hard invariants.
   - Stale active-turn recovery must write typed receipts.

4. **Memory Lifecycle**
   - Keep role-local memory writes as reviewed `selfPatch` operations.
   - Add explicit lifecycle operations where missing: revise, retire,
     crystallize, prune short-term residue, and merge incubation support.
   - Sleep/maintenance may propose lifecycle operations; it must not silently
     rewrite role identity.

5. **Public Persona Safety**
   - Persona speech eligibility must be parent-side state policy, not prompt hope.
   - Track recent outputs, repeated topics, repeated openings, target/channel
     saturation, and action kind.
   - Work-shaped asks route to coordinator/Bifrost-style governance; banter
     does not secretly become work dispatch.

6. **CultNet Operator Surface**
   - Move the local status/coordinator loop toward native CultNet snapshots,
     intents, and receipts.
   - Codex JSON-RPC may remain as the compatibility projection while the wall is
     being evacuated.
   - The next run path should consume typed documents first and only use
     app-server JSON-RPC where no native route exists yet.

7. **Audit And Review**
   - Every automatic or semi-automatic action must produce a typed receipt.
   - Semantic findings remain review-gated.
   - Operator artifacts must seal direct thought, raw transcripts, and long
     prompt-shaped notes by default.

## Ranked Next Cuts

Keep:

- typed CultCache state
- runtime-spine documents and CultNet catalog
- heartbeat initiative physiology
- reviewed role memory patches
- local run/status/coordinator artifacts
- Codex auth/model transport reliquary

Cut:

- schema/catalog language that treats legacy JSON-RPC routes as native authority
- public or operator outputs that dump prompt slabs as if they were status
- any new live runner that bypasses typed pause, receipt, or review gates

Collapse:

- local runner status/coordinator result shaping into one operator projection
  helper if duplication grows
- Persona speech receipts and eligibility into one typed surface before high-rate
  speech returns

Split:

- heartbeat opportunity from action authority
- Persona affect/state from speech generation
- scheduler metadata from model-facing prose
- vector recall from canonical memory

Rebuild:

- any swarm loop that depends on a single prompt to own scheduling, identity,
  public speech, governance, proposal behavior, and repetition control

## Immediate Work Queue

1. Add typed swarm pause/brake state and status projection.
2. Add stale active-turn recovery receipts to heartbeat state if the current
   heartbeat store cannot prove recovery cleanly.
3. Add a Persona output audit surface before increasing public speech cadence.
4. Add memory lifecycle operations that match VoidBot's useful pattern without
   porting its TypeScript state shape wholesale.
5. Move one local-run read path from Codex JSON-RPC compatibility to CultNet
   snapshot/receipt consumption.

The correct posture is patient pressure. We are not rushing to live fire. We are
aiming the barrel, checking the chamber, and making sure the thing does not try
to become a parliament of prompts the moment nobody is watching.
