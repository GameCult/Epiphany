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

The catalog contains only contracts that cross an owned boundary. Runtime-local
CultCache documents are discovered from the runtime's native registration and
projected by their owning CultMesh provider. They are not copied into a second
hand-maintained JSON registry.

## Main Families

- `epiphany.openai_model_request.v1`: the exact provider request crossing the
  OpenAI-compatible transport boundary.
- `gamecult.persona_state.v0`: portable public Persona state.
- `epiphany.work_organ_state.v0`: portable state for lean work organs.

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
