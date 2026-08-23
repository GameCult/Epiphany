# CultNet Schemas

This folder contains Epiphany's JSON Schema publication artifacts for typed
CultNet boundaries. Typed Rust documents registered in CultCache remain the
runtime authority; these files help foreign consumers inspect the wire shape.

The product direction is project-native agency: clients should speak to a
project or one of its Personas/Personas, then watch typed scheduling, memory, evidence, and
authority surfaces do the structuring work. Aquarium is the most direct client,
but Discord, voice/WebRTC rooms, stream overlays, native CLIs, and other trusted
tools should discover the same contracts instead of inventing private command
verbs.

## What Lives Here

- `index.json`: publication manifest for providers that expose these contracts.
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

The provider that owns a live CultMesh/CultNet surface owns its schema-catalog
response. This directory supplies publication artifacts; it is not a second
runtime registry and no standalone Epiphany catalog command impersonates a
service. The native model preflight derives its accepted document types from
the same CultCache registrations used to open the runtime Mind store.
