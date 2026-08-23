# CultNet Schemas

This folder is Epiphany's published CultNet contract surface for Aquarium and
other swarm-side observers. It contains the runtime state, operator-facing
surface projections, control intents, and receipt/artifact payloads that the
runtime-spine advertises over `cultnet.schema.v0`.

The product direction is project-native agency: clients should speak to a
project or one of its Personas/Personas, then watch typed scheduling, memory, evidence, and
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
- keyed Mind documents and `epiphany.state-ledger`: durable typed state and
  exact decision/commit receipts the runtime lives on.
- model, tool, and provider-boundary contracts used by the native runtime.
- `gamecult.persona_state.v0` and `epiphany.work_organ_state.v0`: portable
  state contracts whose owners may live outside this repository.

The catalog publishes contracts the executable body actually produces or
consumes. Editor capabilities are provider-owned CultMesh/Eve surfaces, not
Epiphany-owned Rider or Unity command families. Brokkr owns Unity editor
inspection and actuation; a future Rider daemon will own Rider integration.

## Publication Path

Generate a schema-catalog response with inline schema bodies:

```powershell
cargo run -p epiphany-release-bundle --bin epiphany-runtime-spine -- schema-catalog --output .epiphany-dogfood\runtime-spine\schema-catalog.json --include-schema-json true
```

The runtime-spine merges the builtin CultNet schema registry with this local
index before answering the request, so consumers can discover both canonical
wire contracts and Epiphany-local payload contracts from one place.
