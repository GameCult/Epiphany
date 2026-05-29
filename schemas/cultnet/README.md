# CultNet Schemas

This folder is Epiphany's published CultNet contract surface for Aquarium and
other swarm-side observers. It contains the runtime state, operator-facing
surface projections, control intents, and receipt/artifact payloads that the
runtime-spine advertises over `cultnet.schema.v0`.

The product direction is project-native agency: clients should speak to a
project or one of its Faces/Personas, then watch typed scheduling, memory, evidence, and
authority surfaces do the structuring work. Aquarium is the most direct client,
but Discord, voice/WebRTC rooms, stream overlays, native CLIs, and other trusted
tools should discover the same contracts instead of inventing private command
verbs.

## What Lives Here

- `index.json`: local registration manifest loaded by
  `epiphany-runtime-spine` when answering schema-catalog requests.
- `*.schema.json`: top-level JSON Schema receipts for payload/document shapes.

The payload receipts are deliberately practical rather than religiously
exhaustive. They publish the stable top-level structure Aquarium needs for
inspection, visualization, and control without pretending every nested app
server object deserves to be duplicated into a second baroque schema maze.

## Main Families

- `epiphany.runtime.*`: native runtime-spine identity, session, job, job
  result, and event documents.
- `epiphany.agent_*` and `epiphany.state-ledger`: durable typed state the swarm
  actually lives on, plus derived agent views such as
  `epiphany.agent_utterance_state.v0` for speech-conditioning without memory
  payloads.
- `epiphany.surface.*`: operator-safe projections for scene, freshness,
  context, graph query, coordinator, roles, jobs, Face/Persona, Rider, Unity, repo
  initialization, and related live surfaces.
- `*.intent.v0`: control documents Aquarium, a Discord bridge, a voice surface,
  or another trusted coordinator can submit through CultNet.
- receipt/artifact schemas such as `epiphany.swarm-control-receipt`,
  `epiphany.face-bubble`, `epiphany.character-turn-packet`, and
  `epiphany.repo-birth-runner`.

## Publication Path

Generate a schema-catalog response with inline schema bodies:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-runtime-spine -- schema-catalog --output .epiphany-dogfood\runtime-spine\schema-catalog.json --include-schema-json true
```

The runtime-spine merges the builtin CultNet schema registry with this local
index before answering the request, so consumers can discover both canonical
wire contracts and Epiphany-local payload contracts from one place.
